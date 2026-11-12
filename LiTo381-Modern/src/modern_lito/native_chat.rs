//! Native chat (T060): receipts `S_SAY 0x13`, `S_SAY_CODE 0x2D` (decimal 45,
//! NPC speech) and `S_TELL 0x14`; requests `C_SAY 0x13`, `C_CHAT 0x77`,
//! `C_TELL 0x1B` and `C_ONOFF 0x0D`. Layouts and limits are those of JAR
//! E1498871 `Protocol.say`／`sayCode`／`tell`／`chatRequest`／`tellRequest`／
//! `onOffRequest` (javap, #342).
//!
//! Framing is strict: a truncated, oversized or trailing-byte packet is a
//! protocol error. Text content is tolerant, because it originates from other
//! players: bytes that are not MS950 and control characters are replaced, never
//! turned into a connection failure.
//!
//! Who receives a line is decided by the server alone. E149 fans player speech
//! out to the channel, room, rental town or classic-world map of the speaker,
//! not by distance; the 8／50-cell ranges belong to its NPC speech producer.
//! This module therefore never filters by coordinates.
use super::{native_login::LoginPhase, native_opcodes as op};
use anyhow::{ensure, Context, Result};
use std::collections::VecDeque;

/// `C_SAY`／`C_CHAT`／`C_TELL`／`S_TELL` message bytes (MS950, without NUL).
pub const MESSAGE_BYTES: usize = 255;
/// `S_SAY`／`S_SAY_CODE` message bytes: the native receivers store 511.
pub const SPEECH_BYTES: usize = 511;
/// Whisper sender／target name bytes.
pub const NAME_BYTES: usize = 27;
pub const LOG_CAPACITY: usize = 200;

/// Display groups of the filter panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Category {
    Normal,
    Shout,
    World,
    Trade,
    Whisper,
    Npc,
    Other,
}
impl Category {
    pub const ALL: [Category; 7] = [
        Self::Normal,
        Self::Shout,
        Self::World,
        Self::Trade,
        Self::Whisper,
        Self::Npc,
        Self::Other,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "一般",
            Self::Shout => "大喊",
            Self::World => "全體",
            Self::Trade => "買賣",
            Self::Whisper => "密語",
            Self::Npc => "NPC",
            Self::Other => "其他",
        }
    }
    /// Chat type byte of `S_SAY`. 0／2／3 are the values E149 itself produces
    /// for ordinary, shouted and global NPC speech; 3 and 12 are the two types
    /// its `acceptsChatType` ties to the world and trade preferences.
    pub fn of_type(kind: u8) -> Self {
        match kind {
            0 => Self::Normal,
            2 => Self::Shout,
            3 => Self::World,
            12 => Self::Trade,
            _ => Self::Other,
        }
    }
    fn bit(self) -> u8 {
        1 << (self as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Origin {
    /// `S_SAY`: the server already formatted the text as `name: message`.
    Player,
    /// `S_SAY_CODE`: `name: text`, `<name> text` (shout) or `[name] text` (global).
    Npc,
    WhisperReceived,
    /// Local record of a queued `C_TELL`. The server sends no delivery receipt.
    WhisperSent,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Entry {
    pub sequence: u64,
    pub origin: Origin,
    /// Raw chat type byte; 0 for whispers.
    pub kind: u8,
    /// Speaker object id; 0 for whispers and for id-less server hints.
    pub speaker: u32,
    /// Whisper sender or target; empty otherwise.
    pub name: String,
    /// Text as received, including `\fX` colour prefixes and `$n` references.
    pub text: String,
    /// Speaker cell from the packet; kept for a later overhead bubble.
    pub position: Option<[u16; 2]>,
}
impl Entry {
    pub fn category(&self) -> Category {
        match self.origin {
            Origin::Player => Category::of_type(self.kind),
            Origin::Npc => Category::Npc,
            Origin::WhisperReceived | Origin::WhisperSent => Category::Whisper,
        }
    }
    pub fn line(&self) -> String {
        let text = plain(&self.text);
        match self.origin {
            Origin::Player => match self.category() {
                Category::Other => format!("[頻道{}] {text}", self.kind),
                category => format!("[{}] {text}", category.label()),
            },
            Origin::Npc if self.speaker == 0 => format!("[提示] {text}"),
            Origin::Npc => match self.kind {
                2 => format!("[NPC 大喊] {text}"),
                3 => format!("[NPC 全體] {text}"),
                _ => format!("[NPC] {text}"),
            },
            Origin::WhisperReceived => format!("[密語] {}：{text}", self.name),
            Origin::WhisperSent => format!("[密語→{}] {text}", self.name),
        }
    }
}

/// Removes `\fX` colour prefixes (server hints use `\fT`／`\fW`). `$n` string
/// references are left as received; resolving them belongs to T049.
pub fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'f') {
            chars.next();
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}

/// Client-side visibility of each category. Hiding keeps the entries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    hidden: u8,
}
impl Filter {
    pub fn shows(self, category: Category) -> bool {
        self.hidden & category.bit() == 0
    }
    /// Returns whether the filter changed.
    pub fn set(&mut self, category: Category, shown: bool) -> bool {
        let before = self.hidden;
        if shown {
            self.hidden &= !category.bit();
        } else {
            self.hidden |= category.bit();
        }
        before != self.hidden
    }
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).context("chat offset")?;
        let b = self.b.get(self.at..end).context("truncated chat field")?;
        self.at = end;
        Ok(b)
    }
    fn c(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn h(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into()?))
    }
    fn d(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    /// NUL-terminated MS950 of 1..=`max` bytes.
    fn s(&mut self, max: usize) -> Result<String> {
        let n = self.b[self.at.min(self.b.len())..]
            .iter()
            .take(max + 1)
            .position(|v| *v == 0)
            .context("chat string bound")?;
        ensure!(n != 0, "empty chat string");
        Ok(decode(&self.take(n + 1)?[..n]))
    }
    fn end(&self) -> Result<()> {
        ensure!(self.at == self.b.len(), "trailing chat fields");
        Ok(())
    }
}

/// MS950 → text. Undecodable input keeps its ASCII and marks the rest with
/// `?`; control characters become spaces. Never fails on content.
fn decode(raw: &[u8]) -> String {
    let text = decode_strict(raw).unwrap_or_else(|| {
        raw.iter().map(|b| if b.is_ascii() { *b as char } else { '?' }).collect()
    });
    text.chars().map(|c| if c.is_control() { ' ' } else { c }).collect()
}
fn decode_strict(raw: &[u8]) -> Option<String> {
    use windows::Win32::Globalization::{MultiByteToWideChar, MB_ERR_INVALID_CHARS};
    if raw.is_empty() {
        return Some(String::new());
    }
    let mut wide = vec![0u16; raw.len()];
    let count = unsafe { MultiByteToWideChar(950, MB_ERR_INVALID_CHARS, raw, Some(&mut wide)) };
    (count > 0)
        .then(|| String::from_utf16(&wide[..count as usize]).ok())
        .flatten()
}

/// Exact MS950 bytes of user text, with the reason in the user's language.
/// Text that would reach the other side altered (best-fit or `?`) is refused.
fn encode(text: &str, max: usize, what: &str) -> Result<Vec<u8>> {
    use windows::{
        core::PCSTR,
        Win32::{
            Foundation::BOOL,
            Globalization::{WideCharToMultiByte, WC_NO_BEST_FIT_CHARS},
        },
    };
    ensure!(!text.is_empty(), "請輸入{what}");
    ensure!(!text.chars().any(char::is_control), "{what}不可含控制字元或換行");
    ensure!(text.len() <= 1024, "{what}過長，未送出");
    let wide: Vec<u16> = text.encode_utf16().collect();
    let mut fallback = BOOL(0);
    let mut bytes = vec![0u8; wide.len() * 2];
    let count = unsafe {
        WideCharToMultiByte(
            950,
            WC_NO_BEST_FIT_CHARS,
            &wide,
            Some(&mut bytes),
            PCSTR::null(),
            Some(&mut fallback),
        )
    };
    bytes.truncate(count.max(0) as usize);
    ensure!(
        count > 0 && !fallback.as_bool() && decode_strict(&bytes).as_deref() == Some(text),
        "{what}含有 MS950（Big5）無法表示的字，未送出"
    );
    ensure!(
        bytes.len() <= max,
        "{what}超過 {max} 位元組（目前 {}），未送出",
        bytes.len()
    );
    Ok(bytes)
}

/// Channels a player can speak on from this client.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Normal,
    Shout,
    World,
    Trade,
}
impl Channel {
    pub const ALL: [Channel; 4] = [Self::Normal, Self::Shout, Self::World, Self::Trade];
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "一般",
            Self::Shout => "大喊",
            Self::World => "全體",
            Self::Trade => "買賣",
        }
    }
    pub fn kind(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::Shout => 2,
            Self::World => 3,
            Self::Trade => 12,
        }
    }
}

/// `C_SAY c(type) s(message)`; trade uses `C_CHAT`, whose type must be 12.
pub fn say_request(channel: Channel, message: &str) -> Result<Vec<u8>> {
    let text = encode(message.trim(), MESSAGE_BYTES, "訊息")?;
    let opcode = if channel == Channel::Trade { op::C_CHAT } else { op::C_SAY };
    let mut body = Vec::with_capacity(text.len() + 3);
    body.extend([opcode, channel.kind()]);
    body.extend(text);
    body.push(0);
    Ok(body)
}

/// `C_TELL s(target) s(message)`. The server trims the target name.
pub fn tell_request(target: &str, message: &str) -> Result<Vec<u8>> {
    let target = target.trim();
    ensure!(!target.chars().any(char::is_whitespace), "密語對象不可含空白");
    let name = encode(target, NAME_BYTES, "密語對象")?;
    let text = encode(message.trim(), MESSAGE_BYTES, "訊息")?;
    let mut body = Vec::with_capacity(name.len() + text.len() + 3);
    body.push(op::C_TELL);
    body.extend(name);
    body.push(0);
    body.extend(text);
    body.push(0);
    Ok(body)
}

/// Server-side delivery preferences of `C_ONOFF c(selector) c(0|1)`. E149
/// starts every session with all three enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preference {
    World,
    Whisper,
    Trade,
}
impl Preference {
    pub fn of(category: Category) -> Option<Self> {
        match category {
            Category::World => Some(Self::World),
            Category::Whisper => Some(Self::Whisper),
            Category::Trade => Some(Self::Trade),
            _ => None,
        }
    }
    fn selector(self) -> u8 {
        match self {
            Self::World => 0,
            Self::Whisper => 2,
            Self::Trade => 6,
        }
    }
}
pub fn preference_request(preference: Preference, enabled: bool) -> [u8; 3] {
    [op::C_ONOFF, preference.selector(), u8::from(enabled)]
}

/// Client phases whose server phase (CHANNEL／ROOM／BATTLE／WORLD) handles
/// `C_SAY`／`C_TELL`. Elsewhere the server drops them without a reply.
pub fn sendable(phase: LoginPhase) -> bool {
    matches!(
        phase,
        LoginPhase::LobbyReady
            | LoginPhase::RoomReady
            | LoginPhase::BattlePending
            | LoginPhase::WorldReady
    )
}

/// Bounded chat history of one selected character on one connection.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Log {
    owner: Option<(u64, String)>,
    pub entries: VecDeque<Entry>,
    pub sequence: u64,
    pub dropped: u64,
    /// Advances on every change, including clearing.
    pub revision: u64,
}
impl Log {
    pub fn owns(body: &[u8]) -> bool {
        matches!(body.first(), Some(&op::S_SAY | &op::S_SAY_CODE | &op::S_TELL))
    }
    /// A different connection or character never sees the previous history.
    pub fn bind(&mut self, epoch: u64, character: Option<&str>) {
        let current = self.owner.as_ref().map(|(e, c)| (*e, c.as_str()));
        if current != character.map(|c| (epoch, c)) {
            self.owner = character.map(|c| (epoch, c.to_owned()));
            if !self.entries.is_empty() {
                self.entries.clear();
            }
            self.revision += 1;
        }
    }
    fn push(&mut self, mut entry: Entry) {
        self.sequence += 1;
        entry.sequence = self.sequence;
        if self.entries.len() == LOG_CAPACITY {
            self.entries.pop_front();
            self.dropped += 1;
        }
        self.entries.push_back(entry);
        self.revision += 1;
    }
    /// False when the opcode is not a chat receipt. Errors leave the log intact.
    pub fn accept(&mut self, body: &[u8]) -> Result<bool> {
        let Some(&opcode) = body.first() else { return Ok(false) };
        let mut r = Reader { b: body, at: 1 };
        let entry = match opcode {
            op::S_SAY | op::S_SAY_CODE => {
                let kind = r.c()?;
                let speaker = r.d()?;
                let text = r.s(SPEECH_BYTES)?;
                let position = [r.h()?, r.h()?];
                r.end()?;
                let npc = opcode == op::S_SAY_CODE;
                ensure!(!npc || kind <= 3, "NPC speech type outside 0..3");
                Entry {
                    sequence: 0,
                    origin: if npc { Origin::Npc } else { Origin::Player },
                    kind,
                    speaker,
                    name: String::new(),
                    text,
                    position: Some(position),
                }
            }
            op::S_TELL => {
                let name = r.s(NAME_BYTES)?;
                let text = r.s(MESSAGE_BYTES)?;
                r.end()?;
                Entry {
                    sequence: 0,
                    origin: Origin::WhisperReceived,
                    kind: 0,
                    speaker: 0,
                    name,
                    text,
                    position: None,
                }
            }
            _ => return Ok(false),
        };
        self.push(entry);
        Ok(true)
    }
    /// Call only after the matching `C_TELL` entered the send queue.
    pub fn whisper_sent(&mut self, target: &str, message: &str) {
        self.push(Entry {
            sequence: 0,
            origin: Origin::WhisperSent,
            kind: 0,
            speaker: 0,
            name: target.trim().to_owned(),
            text: message.trim().to_owned(),
            position: None,
        });
    }
    /// CRLF-separated lines of the shown categories, oldest first.
    pub fn render(&self, filter: Filter) -> String {
        let mut out = String::new();
        if self.dropped != 0 {
            out.push_str(&format!("（較早的 {} 則訊息已捨棄）", self.dropped));
        }
        for entry in self.entries.iter().filter(|e| filter.shows(e.category())) {
            if !out.is_empty() {
                out.push_str("\r\n");
            }
            out.push_str(&entry.line());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms950(text: &str) -> Vec<u8> {
        encode(text, 4096, "測試文字").unwrap()
    }
    fn speech(opcode: u8, kind: u8, id: u32, text: &[u8], x: u16, y: u16) -> Vec<u8> {
        let mut b = vec![opcode, kind];
        b.extend(id.to_le_bytes());
        b.extend(text);
        b.push(0);
        b.extend(x.to_le_bytes());
        b.extend(y.to_le_bytes());
        b
    }
    fn tell(sender: &[u8], text: &[u8]) -> Vec<u8> {
        let mut b = vec![op::S_TELL];
        b.extend(sender);
        b.push(0);
        b.extend(text);
        b.push(0);
        b
    }

    #[test]
    fn npc_speech_opcode_45_keeps_type_speaker_text_and_coordinates() {
        let mut log = Log::default();
        // NativeDeChatChat／Shouting／Global text shapes, unsigned object id.
        for (kind, text) in [(0, "守衛: 你好"), (2, "<守衛> 有入侵者"), (3, "[公告員] 活動開始")] {
            let body = speech(op::S_SAY_CODE, kind, 0xF000_0001, &ms950(text), 33080, 32790);
            assert_eq!(body[0], 45);
            assert!(log.accept(&body).unwrap());
            let entry = log.entries.back().unwrap();
            assert_eq!(
                (entry.origin, entry.kind, entry.speaker, entry.text.as_str(), entry.position),
                (Origin::Npc, kind, 0xF000_0001, text, Some([33080, 32790]))
            );
            assert_eq!(entry.category(), Category::Npc);
        }
        assert_eq!(
            log.render(Filter::default()),
            "[NPC] 守衛: 你好\r\n[NPC 大喊] <守衛> 有入侵者\r\n[NPC 全體] [公告員] 活動開始"
        );
        assert_eq!((log.sequence, log.entries.len()), (3, 3));
    }

    #[test]
    fn npc_speech_rejects_bad_framing_without_changing_the_log() {
        let mut log = Log::default();
        let good = speech(op::S_SAY_CODE, 0, 7, &ms950("守衛: 你好"), 1, 2);
        // Type 4 is outside the native receiver's 0..3 range.
        assert!(log.accept(&speech(op::S_SAY_CODE, 4, 7, b"a", 1, 2)).is_err());
        // Empty text, missing coordinates, trailing byte, missing terminator.
        assert!(log.accept(&speech(op::S_SAY_CODE, 0, 7, b"", 1, 2)).is_err());
        assert!(log.accept(&good[..good.len() - 1]).is_err());
        let mut trailing = good.clone();
        trailing.push(0);
        assert!(log.accept(&trailing).is_err());
        assert!(log.accept(&[op::S_SAY_CODE, 0, 7, 0, 0, 0, b'a']).is_err());
        // 511 bytes is the storage limit; 512 is refused.
        assert!(log.accept(&speech(op::S_SAY_CODE, 0, 7, &[b'a'; 512], 1, 2)).is_err());
        assert_eq!((log.entries.len(), log.sequence, log.revision), (0, 0, 0));
        assert!(log.accept(&speech(op::S_SAY_CODE, 0, 7, &[b'a'; 511], 1, 2)).unwrap());
        assert_eq!(log.entries[0].text.len(), 511);
    }

    #[test]
    fn id_zero_server_hint_strips_colour_prefix_and_keeps_string_reference() {
        // Protocol.questHelpMessage: type 0, object id 0, `\fT` prefix.
        let mut log = Log::default();
        assert!(log
            .accept(&speech(op::S_SAY_CODE, 0, 0, &ms950("\\fT任務完成！"), 100, 200))
            .unwrap());
        assert!(log
            .accept(&speech(op::S_SAY_CODE, 2, 9, &ms950("<$1234> $5678"), 100, 200))
            .unwrap());
        assert_eq!(log.entries[0].text, "\\fT任務完成！");
        assert_eq!(
            log.render(Filter::default()),
            "[提示] 任務完成！\r\n[NPC 大喊] <$1234> $5678"
        );
        assert_eq!(plain("a\\fWb\\f"), "ab");
        assert_eq!(plain("\\n stays"), "\\n stays");
    }

    #[test]
    fn player_speech_is_grouped_by_type_and_never_by_distance() {
        let mut log = Log::default();
        for (kind, text) in [(0, "甲: 早"), (2, "甲: 喂"), (3, "甲: 大家好"), (12, "甲: 收劍"), (4, "甲: 盟")] {
            // Far-away coordinates: delivery is the server's decision.
            assert!(log.accept(&speech(op::S_SAY, kind, 0x1000_0001, &ms950(text), 65535, 0)).unwrap());
        }
        let categories: Vec<_> = log.entries.iter().map(Entry::category).collect();
        assert_eq!(
            categories,
            [Category::Normal, Category::Shout, Category::World, Category::Trade, Category::Other]
        );
        assert_eq!(
            log.render(Filter::default()),
            "[一般] 甲: 早\r\n[大喊] 甲: 喂\r\n[全體] 甲: 大家好\r\n[買賣] 甲: 收劍\r\n[頻道4] 甲: 盟"
        );
        // S_SAY accepts any type byte; S_SAY_CODE does not.
        assert!(log.accept(&speech(op::S_SAY, 255, 1, b"x: y", 0, 0)).unwrap());
    }

    #[test]
    fn whisper_receipt_and_local_sent_record() {
        let mut log = Log::default();
        assert!(log.accept(&tell(&ms950("乙"), &ms950("在嗎"))).unwrap());
        log.whisper_sent(" 乙 ", " 在 ");
        assert_eq!(log.render(Filter::default()), "[密語] 乙：在嗎\r\n[密語→乙] 在");
        assert_eq!(log.entries[0].position, None);
        // Sender 27 bytes and message 255 bytes are the limits.
        assert!(log.accept(&tell(&[b'n'; 27], &[b'm'; 255])).unwrap());
        assert!(log.accept(&tell(&[b'n'; 28], b"m")).is_err());
        assert!(log.accept(&tell(b"n", &[b'm'; 256])).is_err());
        assert!(log.accept(&tell(b"", b"m")).is_err());
        assert!(log.accept(&tell(b"n", b"")).is_err());
        let mut trailing = tell(b"n", b"m");
        trailing.push(1);
        assert!(log.accept(&trailing).is_err());
        assert_eq!(log.entries.len(), 3);
    }

    #[test]
    fn foreign_opcodes_are_not_claimed() {
        let mut log = Log::default();
        let bodies: [&[u8]; 4] = [&[], &[0x0f, 9, b'a', 0], &[0x10, 1, 0, 0], &[0x12, 0, 0, 0, 0]];
        for body in bodies {
            assert!(!Log::owns(body));
            assert!(!log.accept(body).unwrap());
        }
        assert!(Log::owns(&[op::S_SAY]) && Log::owns(&[op::S_SAY_CODE]) && Log::owns(&[op::S_TELL]));
        assert_eq!(log.revision, 0);
    }

    #[test]
    fn hostile_text_content_never_fails_the_packet() {
        let mut log = Log::default();
        // Lone lead byte, then a tab and a bell inside otherwise valid text.
        assert!(log.accept(&speech(op::S_SAY, 0, 1, &[b'a', 0xA4, b'!'], 0, 0)).is_ok());
        assert!(log.accept(&speech(op::S_SAY, 0, 1, b"a\tb\x07c", 0, 0)).unwrap());
        assert_eq!(log.entries[1].text, "a b c");
        assert!(log.entries.iter().all(|e| !e.text.chars().any(char::is_control)));
        // A long valid line decodes exactly (no chunk boundary damage).
        let long = "中".repeat(255);
        assert!(log.accept(&speech(op::S_SAY, 0, 1, &ms950(&long), 0, 0)).unwrap());
        assert_eq!(log.entries[2].text, long);
    }

    #[test]
    fn filter_hides_categories_without_discarding_entries() {
        let mut log = Log::default();
        log.accept(&speech(op::S_SAY, 0, 1, &ms950("甲: 一般"), 0, 0)).unwrap();
        log.accept(&speech(op::S_SAY, 3, 1, &ms950("甲: 全體"), 0, 0)).unwrap();
        log.accept(&speech(op::S_SAY_CODE, 0, 2, &ms950("守衛: 站住"), 0, 0)).unwrap();
        log.accept(&tell(&ms950("乙"), &ms950("嗨"))).unwrap();
        let mut filter = Filter::default();
        assert!(Category::ALL.iter().all(|c| filter.shows(*c)));
        assert!(filter.set(Category::World, false));
        assert!(!filter.set(Category::World, false));
        assert!(filter.set(Category::Npc, false));
        assert_eq!(log.render(filter), "[一般] 甲: 一般\r\n[密語] 乙：嗨");
        // New lines of a hidden category stay hidden, then return when shown.
        log.accept(&speech(op::S_SAY, 3, 1, &ms950("甲: 再說"), 0, 0)).unwrap();
        assert_eq!(log.render(filter), "[一般] 甲: 一般\r\n[密語] 乙：嗨");
        assert!(filter.set(Category::World, true));
        assert_eq!(
            log.render(filter),
            "[一般] 甲: 一般\r\n[全體] 甲: 全體\r\n[密語] 乙：嗨\r\n[全體] 甲: 再說"
        );
        for category in Category::ALL {
            filter.set(category, false);
        }
        assert_eq!(log.render(filter), "");
        assert_eq!(log.entries.len(), 5);
    }

    #[test]
    fn history_is_bounded_and_bound_to_connection_and_character() {
        let mut log = Log::default();
        log.bind(3, Some("甲"));
        let first = log.revision;
        log.bind(3, Some("甲"));
        assert_eq!(log.revision, first);
        for i in 0..LOG_CAPACITY + 5 {
            log.accept(&speech(op::S_SAY, 0, 1, format!("a: {i}").as_bytes(), 0, 0)).unwrap();
        }
        assert_eq!((log.entries.len(), log.dropped), (LOG_CAPACITY, 5));
        assert_eq!(log.entries[0].text, "a: 5");
        assert_eq!(log.entries[0].sequence, 6);
        assert!(log.render(Filter::default()).starts_with("（較早的 5 則訊息已捨棄）\r\n[一般] a: 5"));
        let before = log.revision;
        log.bind(3, Some("乙"));
        assert!(log.entries.is_empty() && log.revision > before);
        log.whisper_sent("丙", "嗨");
        log.bind(4, Some("乙"));
        assert!(log.entries.is_empty());
        log.whisper_sent("丙", "嗨");
        log.bind(4, None);
        assert!(log.entries.is_empty());
    }

    #[test]
    fn say_requests_match_chat_request_layout() {
        assert_eq!(say_request(Channel::Normal, " hi ").unwrap(), [0x13, 0, b'h', b'i', 0]);
        assert_eq!(say_request(Channel::Shout, "hi").unwrap()[..2], [0x13, 2]);
        assert_eq!(say_request(Channel::World, "hi").unwrap()[..2], [0x13, 3]);
        // C_CHAT is accepted only with type 12.
        assert_eq!(say_request(Channel::Trade, "hi").unwrap(), [0x77, 12, b'h', b'i', 0]);
        let chinese = say_request(Channel::Normal, "你好").unwrap();
        assert_eq!(chinese, [0x13, 0, 0xA7, 0x41, 0xA6, 0x6E, 0]);
        // GM commands travel as ordinary speech.
        assert_eq!(say_request(Channel::Normal, ".loc").unwrap(), [0x13, 0, b'.', b'l', b'o', b'c', 0]);
    }

    #[test]
    fn say_requests_refuse_what_the_server_would_reject_or_garble() {
        for empty in ["", "   ", "\u{3000}"] {
            assert!(say_request(Channel::Normal, empty).is_err(), "{empty:?}");
        }
        assert!(say_request(Channel::Normal, "a\nb").is_err());
        // 255 MS950 bytes pass, 256 do not; a double-byte character counts two.
        assert!(say_request(Channel::Normal, &"a".repeat(255)).is_ok());
        assert!(say_request(Channel::Normal, &"a".repeat(256)).is_err());
        assert!(say_request(Channel::Normal, &"中".repeat(127)).is_ok());
        assert!(say_request(Channel::Normal, &"中".repeat(128)).is_err());
        // Not representable in MS950: refused, never sent as `?`.
        let error = say_request(Channel::Normal, "😀").unwrap_err().to_string();
        assert!(error.contains("MS950"), "{error}");
        assert!(say_request(Channel::Normal, "한글").is_err());
    }

    #[test]
    fn tell_request_matches_tell_request_layout_and_limits() {
        assert_eq!(tell_request(" bob ", " hi ").unwrap(), [0x1b, b'b', b'o', b'b', 0, b'h', b'i', 0]);
        assert!(tell_request("", "hi").is_err());
        assert!(tell_request("bob", "").is_err());
        assert!(tell_request("b ob", "hi").is_err());
        assert!(tell_request(&"n".repeat(27), "hi").is_ok());
        assert!(tell_request(&"n".repeat(28), "hi").is_err());
        assert!(tell_request(&"中".repeat(14), "hi").is_err());
        assert!(tell_request("bob", &"a".repeat(256)).is_err());
        assert!(tell_request("😀", "hi").is_err());
    }

    #[test]
    fn preference_requests_use_the_three_server_selectors() {
        assert_eq!(preference_request(Preference::World, false), [0x0d, 0, 0]);
        assert_eq!(preference_request(Preference::Whisper, true), [0x0d, 2, 1]);
        assert_eq!(preference_request(Preference::Trade, false), [0x0d, 6, 0]);
        assert_eq!(Preference::of(Category::World), Some(Preference::World));
        assert_eq!(Preference::of(Category::Whisper), Some(Preference::Whisper));
        assert_eq!(Preference::of(Category::Trade), Some(Preference::Trade));
        for local in [Category::Normal, Category::Shout, Category::Npc, Category::Other] {
            assert_eq!(Preference::of(local), None);
        }
    }

    #[test]
    fn only_channel_room_battle_and_world_phases_can_send() {
        use LoginPhase::*;
        for phase in [LobbyReady, RoomReady, BattlePending, WorldReady] {
            assert!(sendable(phase), "{phase:?}");
        }
        for phase in [Disconnected, ReadyForLogin, CharactersReady, CharacterSelected, RoomJoining,
            WorldEntering, BattleResult, ResultReturning, Closing, Failed] {
            assert!(!sendable(phase), "{phase:?}");
        }
    }
}
