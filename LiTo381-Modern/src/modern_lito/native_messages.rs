//! Server text: numeric notices (S4B), system text (S0F), message codes (S10)
//! and the LTstring lines the native lobby shows for refusals.
//!
//! Static evidence only: recovered PE SHA256 4271D92F…5C2AB6E9, decoded with
//! Capstone; the original binary was never executed.
//! - 48F600 (S4B `h`): a message box with `string-c[id]`. 273..=275 are
//!   `sprintf`ed with a URL row chosen by the region byte 64B6DC; 307 and 437
//!   with the hours and minutes left in 75F73C; 438 and 317 are shown as is.
//! - 4A4190 (S0F `cs`): the text is split at line feeds and written with a
//!   colour chosen by the kind byte (503440 palette entry 32 for kind 1, 44
//!   for 3, 39 for 12, 11 for 4, 30 for 11, otherwise 24). Kind 9 first goes
//!   through 4A4140, which rewrites `\f>` to `\f1`.
//! - 4A4670 (S10 `hc` + strings): a code below the string-c row count is
//!   formatted by 4A44D0; `code - 2000` below the LTstring row count is
//!   `sprintf`ed with arguments read straight from the packet in a shape
//!   chosen per row (table 4A4BAC). Codes 834 and 875 never produce text.
//! - 4A44D0: `%0`..`%9` insert the argument after `$n` expansion (4D44A0);
//!   `%d`, `%o` and `%s` are Korean particle slots that emit nothing unless
//!   the code page is 949; `%` before any other character emits that
//!   character.
//! - 4A76D0 (S_C8), 4A7850 (S_C9), 4A8BA0 (S_CE): result code to LTstring row.
//!
//! Presentation only; wire fields and server decisions are never changed.
use super::native_text_format::{self, PALETTE};
use anyhow::{ensure, Context, Result};
use std::collections::VecDeque;

pub const MAX_TABLE_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_ROWS: usize = 100_000;
/// Entries kept in [`Log`]; older ones are dropped.
pub const LOG_CAPACITY: usize = 8;
/// Text lines the HUD status area shows.
pub const HUD_LINES: usize = 3;
/// First S10 code served by LTstring.
pub const LITO_CODE_BASE: u16 = 2000;

/// One `*-c.tbl` string table: a decimal row count, then one row per line.
#[derive(Clone, Debug, Default)]
pub struct Table {
    declared: u32,
    rows: Vec<String>,
    /// Rows whose bytes were not valid CP950; they are kept as empty rows so
    /// later rows keep their numbers.
    pub undecodable: usize,
}
impl Table {
    pub fn from_rows<I, S>(rows: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let rows: Vec<String> = rows.into_iter().map(Into::into).collect();
        Self {
            declared: rows.len() as u32,
            rows,
            undecodable: 0,
        }
    }
    /// Unlike the item-name tables, string tables contain tabs and are read
    /// leniently: a tab becomes a space, and rows after the declared count
    /// are ignored.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= MAX_TABLE_BYTES, "string table byte budget");
        let mut pieces: Vec<&[u8]> = bytes
            .split(|b| *b == b'\n')
            .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
            .collect();
        // A final line feed leaves one empty piece that is not a row.
        if bytes.ends_with(b"\n") {
            pieces.pop();
        }
        let mut lines = pieces.into_iter();
        let header = lines.next().context("string table header")?;
        let header = std::str::from_utf8(header)
            .ok()
            .map(str::trim)
            .filter(|h| !h.is_empty() && h.len() <= 6 && h.bytes().all(|b| b.is_ascii_digit()))
            .context("string table header")?;
        let declared: u32 = header.parse().context("string table header")?;
        ensure!(
            declared > 0 && declared as usize <= MAX_ROWS,
            "string table row count"
        );
        let mut rows = Vec::new();
        let mut undecodable = 0;
        for (number, raw) in lines.take(declared as usize).enumerate() {
            match super::descriptions::decode_cp950(raw, number + 2) {
                Ok(text) => rows.push(
                    text.chars()
                        .map(|c| if c.is_control() { ' ' } else { c })
                        .collect(),
                ),
                Err(_) => {
                    undecodable += 1;
                    rows.push(String::new());
                }
            }
        }
        ensure!(!rows.is_empty(), "string table has no rows");
        Ok(Self {
            declared,
            rows,
            undecodable,
        })
    }
    pub fn declared(&self) -> u32 {
        self.declared
    }
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub fn get(&self, row: usize) -> Option<&str> {
        self.rows.get(row).map(String::as_str)
    }
}

/// `string-c.tbl` (Text) and `LTstring-c.tbl` (LiTo/LTtext).
#[derive(Clone, Debug, Default)]
pub struct Strings {
    pub base: Table,
    pub lito: Table,
}

pub enum Arg<'a> {
    Text(&'a str),
    Number(i64),
}

fn cells(text: &str) -> usize {
    // The original pads by CP950 bytes: one per ASCII character, two otherwise.
    text.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

/// The part of C `sprintf` the string tables use: `%s`, `%d`/`%i`/`%u` with
/// the `-`, `+`, `0` flags and a width, and `%%`. Any other conversion is
/// copied as written. A missing argument prints nothing (the original would
/// read whatever is on its stack).
pub fn sprintf(template: &str, args: &[Arg<'_>]) -> String {
    let chars: Vec<char> = template.chars().collect();
    let mut out = String::new();
    let mut next = 0;
    let mut at = 0;
    while at < chars.len() {
        if chars[at] != '%' {
            out.push(chars[at]);
            at += 1;
            continue;
        }
        let start = at;
        at += 1;
        let (mut left, mut plus, mut zero) = (false, false, false);
        while let Some(c) = chars.get(at) {
            match c {
                '-' => left = true,
                '+' => plus = true,
                '0' => zero = true,
                _ => break,
            }
            at += 1;
        }
        let mut width = 0usize;
        while let Some(d) = chars.get(at).and_then(|c| c.to_digit(10)) {
            width = (width * 10 + d as usize).min(512);
            at += 1;
        }
        let body = match chars.get(at) {
            Some('%') => {
                out.push('%');
                at += 1;
                continue;
            }
            Some('d' | 'i' | 'u') => {
                let value = match args.get(next) {
                    Some(Arg::Number(n)) => Some(*n),
                    _ => None,
                };
                next += 1;
                match value {
                    None => String::new(),
                    Some(n) => {
                        let sign = if n < 0 {
                            "-"
                        } else if plus {
                            "+"
                        } else {
                            ""
                        };
                        let digits = n.unsigned_abs().to_string();
                        let fill = width.saturating_sub(sign.len() + digits.len());
                        if zero && !left {
                            format!("{sign}{}{digits}", "0".repeat(fill))
                        } else {
                            format!("{sign}{digits}")
                        }
                    }
                }
            }
            Some('s') => {
                let value = match args.get(next) {
                    Some(Arg::Text(t)) => (*t).to_owned(),
                    Some(Arg::Number(n)) => n.to_string(),
                    None => String::new(),
                };
                next += 1;
                value
            }
            _ => {
                out.extend(&chars[start..at]);
                continue;
            }
        };
        at += 1;
        let pad = " ".repeat(width.saturating_sub(cells(&body)));
        if left {
            out.push_str(&body);
            out.push_str(&pad);
        } else {
            out.push_str(&pad);
            out.push_str(&body);
        }
    }
    out
}

/// Native 4A44D0 on a CP950 client. `args` are already `$n`-expanded.
pub fn fill(template: &str, args: &[String]) -> String {
    let mut out = String::new();
    let mut chars = template.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.next() {
            None => break,
            Some(digit @ '0'..='9') => {
                if let Some(arg) = args.get(digit as usize - '0' as usize) {
                    out.push_str(arg);
                }
            }
            Some('d' | 'o' | 's') => {}
            Some(other) => out.push(other),
        }
    }
    out
}

/// How 4A4670 reads the arguments of an LTstring row from the packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LitoShape {
    /// No argument is read.
    Plain,
    /// One little-endian 32-bit number.
    Number,
    /// One string.
    Text,
    /// Two strings.
    TwoTexts,
    /// One string, passed through the `$n` expansion first.
    Described,
}
/// Table 4A4BAC (rows 110..=286; every other row is [`LitoShape::Plain`]).
pub fn lito_shape(row: usize) -> LitoShape {
    match row {
        110 | 186 | 268 | 285 | 286 => LitoShape::Number,
        249 | 252 | 254 | 258 | 259 | 260 | 263 | 266 => LitoShape::Text,
        261 | 262 => LitoShape::TwoTexts,
        276 => LitoShape::Described,
        _ => LitoShape::Plain,
    }
}

/// The chat text of an S10 message, or `None` where the original writes
/// nothing: codes 834 and 875 (client actions), a code in neither table, and
/// an empty row.
///
/// Not reproduced: the one-shot suppression of code 73 (flag 63DF4C), the
/// region-5 condition on code 81, and [`LitoShape::Number`] rows, whose
/// binary argument the world reader (which decodes S10 as strings) does not
/// keep; those rows give `None`.
pub fn code_text(
    strings: &Strings,
    code: u16,
    args: &[String],
    expand: &dyn Fn(&str) -> String,
) -> Option<String> {
    // The original compares the code as a signed 16-bit value.
    if code >= 0x8000 || matches!(code, 834 | 875) {
        return None;
    }
    let text = if let Some(template) = strings.base.get(usize::from(code)) {
        let expanded: Vec<String> = args.iter().map(|a| expand(a)).collect();
        fill(template, &expanded)
    } else {
        let row = usize::from(code.checked_sub(LITO_CODE_BASE)?);
        let template = strings.lito.get(row)?;
        match lito_shape(row) {
            LitoShape::Plain => sprintf(template, &[]),
            LitoShape::Number => return None,
            LitoShape::Text => sprintf(template, &[Arg::Text(args.first()?)]),
            LitoShape::TwoTexts => {
                sprintf(template, &[Arg::Text(args.first()?), Arg::Text(args.get(1)?)])
            }
            LitoShape::Described => sprintf(template, &[Arg::Text(&expand(args.first()?))]),
        }
    };
    (!text.is_empty()).then_some(text)
}

/// Row 431: the URL 48F600 inserts into notices 273..=275 for region 3 below
/// service version 0x340. Rows 245 and 247 (regions 0 and 1) hold the same
/// text in the Taiwan table.
pub const NOTICE_URL_ROW: usize = 431;

/// The message-box text of a numeric S4B notice, inline format codes
/// removed. `None` for an ID outside the table, an empty row, and 307／437,
/// which need the remaining play time this client does not track.
pub fn notice_text(base: &Table, id: u16) -> Option<String> {
    if id >= 0x8000 {
        return None;
    }
    let row = base.get(usize::from(id))?;
    let text = match id {
        273..=275 => sprintf(row, &[Arg::Text(base.get(NOTICE_URL_ROW).unwrap_or(""))]),
        307 | 437 => return None,
        _ => row.to_owned(),
    };
    let text = native_text_format::plain(&text);
    (!text.trim().is_empty()).then_some(text)
}

/// 4A4190: RGB555 base colour of an S0F line.
pub fn system_color(kind: u8) -> u16 {
    PALETTE[match kind {
        1 => 32,
        3 => 44,
        4 => 11,
        11 => 30,
        12 => 39,
        _ => 24,
    }]
}
/// 4A4670: S10 lines use palette entry 24.
pub const CODE_COLOR: u16 = PALETTE[24];

/// 4A4140, applied to kind 9 only: `\f>` becomes `\f1`. (It also rewrites the
/// argument of `\b`, which draws nothing here.)
pub fn sanitize_kind9(text: &str) -> String {
    text.replace("\\f>", "\\f1")
}

/// 417560／417860: one chat line per line feed, a carriage return before it
/// dropped. Format codes are kept; empty lines are not written.
pub fn split_lines(text: &str) -> Vec<String> {
    text.split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

/// A server text as received, before any table lookup.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Body {
    System { kind: u8, text: String },
    Code { code: u16, args: Vec<String> },
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Entry {
    pub sequence: u64,
    pub body: Body,
}
/// The last [`LOG_CAPACITY`] S0F／S10 messages in arrival order.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Log {
    pub sequence: u64,
    pub entries: VecDeque<Entry>,
}
impl Log {
    pub fn push(&mut self, body: Body) {
        self.sequence += 1;
        if self.entries.len() == LOG_CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(Entry {
            sequence: self.sequence,
            body,
        });
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub sequence: u64,
    /// Inline format codes kept; see [`native_text_format::runs`].
    pub text: String,
    /// RGB555 base colour.
    pub color: u16,
}
impl Line {
    pub fn plain(&self) -> String {
        native_text_format::plain(&self.text)
    }
}

/// Chat lines for the logged messages. Without tables an S10 message is
/// reported by number, never guessed; with tables a code the original would
/// not print gives no line.
pub fn lines(log: &Log, strings: Option<&Strings>, expand: &dyn Fn(&str) -> String) -> Vec<Line> {
    let mut out = Vec::new();
    for entry in &log.entries {
        let (texts, color) = match &entry.body {
            Body::System { kind, text } => {
                let text = if *kind == 9 {
                    sanitize_kind9(text)
                } else {
                    text.clone()
                };
                (split_lines(&text), system_color(*kind))
            }
            Body::Code { code, args } => (
                match strings {
                    Some(strings) => code_text(strings, *code, args, expand)
                        .map(|text| split_lines(&text))
                        .unwrap_or_default(),
                    None => vec![format!("系統訊息 #{code}（文字表尚未載入）")],
                },
                CODE_COLOR,
            ),
        };
        out.extend(texts.into_iter().map(|text| Line {
            sequence: entry.sequence,
            text,
            color,
        }));
    }
    out
}

/// The last [`HUD_LINES`] visible lines as a block for the HUD status text:
/// empty, or each line preceded by a line break.
pub fn hud_block(log: &Log, strings: Option<&Strings>, expand: &dyn Fn(&str) -> String) -> String {
    let visible: Vec<String> = lines(log, strings, expand)
        .iter()
        .map(Line::plain)
        .filter(|text| !text.trim().is_empty())
        .collect();
    let skip = visible.len().saturating_sub(HUD_LINES);
    visible[skip..]
        .iter()
        .map(|text| format!("\r\n{text}"))
        .collect()
}

/// 4A76D0: LTstring row shown when S_C8 refuses a character.
pub fn character_refusal_row(result: u8) -> Option<usize> {
    match result {
        7 => Some(93),
        8 => Some(92),
        _ => None,
    }
}
/// 4A7850: LTstring row shown when S_C9 refuses a channel.
pub fn channel_refusal_row(result: u8) -> Option<usize> {
    match result {
        1 => Some(94),
        10 => Some(95),
        _ => None,
    }
}
/// 4A8BA0: LTstring row shown when S_CE refuses a room. Result 5 (row 30) is
/// only shown by the original while a join is pending, which is the only
/// time this client reports a refusal.
pub fn room_refusal_row(result: u8) -> Option<usize> {
    match result {
        3 => Some(28),
        4 | 9 => Some(29),
        5 => Some(30),
        7 => Some(85),
        10 => Some(31),
        11 => Some(32),
        12 => Some(33),
        13 => Some(34),
        14 => Some(96),
        20 => Some(294),
        21 => Some(295),
        _ => None,
    }
}
/// A refusal row as plain text; `None` when the table is absent or the row
/// is empty, so the caller keeps its own wording.
pub fn lito_line(strings: Option<&Strings>, row: Option<usize>) -> Option<String> {
    let text = native_text_format::plain(strings?.lito.get(row?)?);
    (!text.trim().is_empty()).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(arg: &str) -> String {
        arg.to_owned()
    }
    fn base() -> Table {
        let mut rows = vec![String::new(); 440];
        rows[73] = "\\f1%0%d 不在線上。".into();
        rows[74] = "\\f1%0%o 無法使用。".into();
        rows[79] = "\\f1沒有任何事情發生。".into();
        rows[166] = "\\f1%0%s %4%1%3 %2。".into();
        rows[273] = "您的使用期限剩最後一天。請在%s上儲值。".into();
        rows[307] = "剩餘使用時間為 %d個小時 %d分。".into();
        rows[317] = "目前所選的伺服器上有太多使用者。".into();
        rows[353] = "在這附近無法召喚怪物。".into();
        rows[400] = "稅金%n %0金幣 100%% %".into();
        rows[431] = "http://www。lineage。com。tw/".into();
        rows[438] = "本公司受到報警。".into();
        Table::from_rows(rows)
    }
    fn lito() -> Table {
        let mut rows = vec![String::new(); 296];
        rows[26] = "%04d號戰鬥室，戰鬥開始前可先整理道具".into();
        rows[28] = "戰鬥室人數已滿，無法參與。".into();
        rows[32] = "密碼錯誤。".into();
        rows[94] = "並非目前服務中的頻道。".into();
        rows[95] = "該角色無法進入此頻道。".into();
        rows[110] = "目前的戰鬥模式需 %d名以上的人員。".into();
        rows[216] = "光之寶石不足".into();
        rows[249] = "%s 血盟成立了".into();
        rows[261] = "血盟王族賦予 %s %s稱號".into();
        rows[276] = "因使用期限已到，%s的幻象道具消失了。".into();
        Table::from_rows(rows)
    }
    fn strings() -> Strings {
        Strings {
            base: base(),
            lito: lito(),
        }
    }

    #[test]
    fn code_templates_follow_4a44d0() {
        let s = strings();
        let args = |a: &[&str]| a.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
        // The particle slot after the argument prints nothing on CP950.
        assert_eq!(
            code_text(&s, 73, &args(&["Hero"]), &raw).unwrap(),
            "\\f1Hero 不在線上。"
        );
        assert_eq!(
            code_text(&s, 74, &args(&["$5"]), &|a: &str| a.replace("$5", "劍")).unwrap(),
            "\\f1劍 無法使用。"
        );
        // Arguments are positional; a missing one prints nothing.
        assert_eq!(
            code_text(&s, 166, &args(&["a", "b", "c", "d", "e"]), &raw).unwrap(),
            "\\f1a ebd c。"
        );
        assert_eq!(code_text(&s, 166, &args(&["a"]), &raw).unwrap(), "\\f1a  。");
        assert_eq!(code_text(&s, 79, &[], &raw).unwrap(), "\\f1沒有任何事情發生。");
        // `%n` is not a line break here: any other character is copied; a
        // final `%` ends the text.
        assert_eq!(
            code_text(&s, 400, &args(&["30"]), &raw).unwrap(),
            "稅金n 30金幣 100% "
        );
        // Empty row, row outside both tables, client-action codes, negative.
        assert_eq!(code_text(&s, 1, &[], &raw), None);
        assert_eq!(code_text(&s, 440, &[], &raw), None);
        assert_eq!(code_text(&s, 1999, &[], &raw), None);
        assert_eq!(code_text(&s, 2296, &[], &raw), None);
        assert_eq!(code_text(&s, 834, &[], &raw), None);
        assert_eq!(code_text(&s, 875, &[], &raw), None);
        assert_eq!(code_text(&s, 0x8000, &[], &raw), None);
    }

    #[test]
    fn lito_codes_use_the_per_row_argument_shape() {
        let s = strings();
        let args = |a: &[&str]| a.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
        assert_eq!(code_text(&s, 2216, &[], &raw).unwrap(), "光之寶石不足");
        assert_eq!(
            code_text(&s, 2249, &args(&["光明"]), &raw).unwrap(),
            "光明 血盟成立了"
        );
        assert_eq!(
            code_text(&s, 2261, &args(&["Hero", "勇者"]), &raw).unwrap(),
            "血盟王族賦予 Hero 勇者稱號"
        );
        assert_eq!(
            code_text(&s, 2276, &args(&["$7"]), &|a: &str| a.replace("$7", "幻象之劍")).unwrap(),
            "因使用期限已到，幻象之劍的幻象道具消失了。"
        );
        // Only row 276 expands its argument.
        assert_eq!(
            code_text(&s, 2249, &args(&["$7"]), &|_: &str| "x".to_owned()).unwrap(),
            "$7 血盟成立了"
        );
        // A row that needs strings the packet did not carry, and a row whose
        // argument is a binary number, give nothing.
        assert_eq!(code_text(&s, 2249, &[], &raw), None);
        assert_eq!(code_text(&s, 2261, &args(&["Hero"]), &raw), None);
        assert_eq!(code_text(&s, 2110, &args(&["4"]), &raw), None);
        assert_eq!(lito_shape(268), LitoShape::Number);
        assert_eq!(lito_shape(262), LitoShape::TwoTexts);
        assert_eq!(lito_shape(109), LitoShape::Plain);
        assert_eq!(lito_shape(287), LitoShape::Plain);
    }

    #[test]
    fn sprintf_covers_the_conversions_the_tables_use() {
        let n = Arg::Number;
        assert_eq!(sprintf("[%04d] %s", &[n(7), Arg::Text("房間")]), "[0007] 房間");
        assert_eq!(sprintf("勝率：%d%%", &[n(55)]), "勝率：55%");
        assert_eq!(sprintf("%4d名", &[n(12)]), "  12名");
        assert_eq!(sprintf("%+d|%+d|%d", &[n(3), n(-3), n(-3)]), "+3|-3|-3");
        assert_eq!(sprintf("%-6s|%6s|", &[Arg::Text("王族"), Arg::Text("ab")]), "王族  |    ab|");
        assert_eq!(sprintf("%-4d|%04d", &[n(5), n(-5)]), "5   |-005");
        assert_eq!(sprintf("%s %s", &[Arg::Text("a")]), "a ");
        assert_eq!(sprintf("%d", &[Arg::Text("a")]), "");
        assert_eq!(sprintf("%s", &[n(9)]), "9");
        assert_eq!(sprintf("100%", &[]), "100%");
        assert_eq!(sprintf("%x %5.2f", &[n(1)]), "%x %5.2f");
        assert_eq!(sprintf("", &[]), "");
    }

    #[test]
    fn notices_use_the_string_table_row() {
        let b = base();
        assert_eq!(notice_text(&b, 438).unwrap(), "本公司受到報警。");
        assert_eq!(notice_text(&b, 317).unwrap(), "目前所選的伺服器上有太多使用者。");
        assert_eq!(
            notice_text(&b, 273).unwrap(),
            "您的使用期限剩最後一天。請在http://www。lineage。com。tw/上儲值。"
        );
        // Format codes are for the box writer, not text.
        assert_eq!(notice_text(&b, 79).unwrap(), "沒有任何事情發生。");
        // Needs the remaining time; empty row; outside the table; negative.
        assert_eq!(notice_text(&b, 307), None);
        assert_eq!(notice_text(&b, 2), None);
        assert_eq!(notice_text(&b, 440), None);
        assert_eq!(notice_text(&b, 0xffff), None);
    }

    #[test]
    fn system_text_colours_and_line_splitting_follow_4a4190() {
        assert_eq!(system_color(1), 0x7f39);
        assert_eq!(system_color(3), 0x57bd);
        assert_eq!(system_color(4), 0x63b6);
        assert_eq!(system_color(11), 0x675f);
        assert_eq!(system_color(12), 0x5e38);
        for kind in [0, 2, 5, 9, 10, 13, 255] {
            assert_eq!(system_color(kind), 0x7fd6, "{kind}");
        }
        assert_eq!(CODE_COLOR, 0x7fd6);
        assert_eq!(split_lines("a\r\nb\n\nc"), ["a", "b", "c"]);
        assert_eq!(split_lines("a\rb"), ["a\rb"]);
        assert!(split_lines("").is_empty());
        assert_eq!(sanitize_kind9("\\f>x\\f2y"), "\\f1x\\f2y");
    }

    #[test]
    fn log_is_bounded_and_lines_keep_arrival_order() {
        let mut log = Log::default();
        for n in 0..LOG_CAPACITY as u16 + 2 {
            log.push(Body::Code {
                code: n,
                args: vec![],
            });
        }
        assert_eq!(log.entries.len(), LOG_CAPACITY);
        assert_eq!(log.entries.front().unwrap().sequence, 3);
        assert_eq!(log.sequence, LOG_CAPACITY as u64 + 2);

        let s = strings();
        let mut log = Log::default();
        log.push(Body::System {
            kind: 9,
            text: "\\f>光之寶石不足。".into(),
        });
        log.push(Body::Code {
            code: 353,
            args: vec![],
        });
        log.push(Body::Code {
            code: 834,
            args: vec![],
        });
        log.push(Body::System {
            kind: 3,
            text: "第一行\n第二行".into(),
        });
        let shown = lines(&log, Some(&s), &raw);
        assert_eq!(
            shown
                .iter()
                .map(|l| (l.sequence, l.text.as_str(), l.color))
                .collect::<Vec<_>>(),
            [
                (1, "\\f1光之寶石不足。", 0x7fd6),
                (2, "在這附近無法召喚怪物。", 0x7fd6),
                (4, "第一行", 0x57bd),
                (4, "第二行", 0x57bd),
            ]
        );
        assert_eq!(shown[0].plain(), "光之寶石不足。");
        // The HUD keeps the newest lines.
        assert_eq!(
            hud_block(&log, Some(&s), &raw),
            "\r\n在這附近無法召喚怪物。\r\n第一行\r\n第二行"
        );
        // Without tables a code is reported by number.
        assert_eq!(
            hud_block(&log, None, &raw),
            "\r\n系統訊息 #834（文字表尚未載入）\r\n第一行\r\n第二行"
        );
        assert_eq!(hud_block(&Log::default(), Some(&s), &raw), "");
    }

    #[test]
    fn refusal_rows_match_the_native_handlers() {
        let s = strings();
        assert_eq!(character_refusal_row(7), Some(93));
        assert_eq!(character_refusal_row(8), Some(92));
        assert_eq!(character_refusal_row(1), None);
        assert_eq!(channel_refusal_row(1), Some(94));
        assert_eq!(channel_refusal_row(10), Some(95));
        for silent in [0, 2, 3, 4, 9, 11] {
            assert_eq!(channel_refusal_row(silent), None, "{silent}");
        }
        let rooms: Vec<_> = (0u8..=22).map(room_refusal_row).collect();
        assert_eq!(
            rooms,
            [
                None, None, None, Some(28), Some(29), Some(30), None, Some(85), None, Some(29),
                Some(31), Some(32), Some(33), Some(34), Some(96), None, None, None, None, None,
                Some(294), Some(295), None
            ]
        );
        assert_eq!(
            lito_line(Some(&s), room_refusal_row(11)).as_deref(),
            Some("密碼錯誤。")
        );
        assert_eq!(
            lito_line(Some(&s), channel_refusal_row(10)).as_deref(),
            Some("該角色無法進入此頻道。")
        );
        // Empty row, no row for the result, no table: the caller's own text.
        assert_eq!(lito_line(Some(&s), room_refusal_row(12)), None);
        assert_eq!(lito_line(Some(&s), room_refusal_row(1)), None);
        assert_eq!(lito_line(None, room_refusal_row(11)), None);
    }

    #[cfg(windows)]
    #[test]
    fn tables_are_parsed_leniently() {
        // "王族" in CP950, a tab, a row past the declared count.
        let t = Table::parse(b"3\r\n\xa4\xfd\xb1\xda\r\na\tb\r\n\r\nextra\r\n").unwrap();
        assert_eq!((t.declared(), t.len()), (3, 3));
        assert_eq!(t.get(0), Some("王族"));
        assert_eq!(t.get(1), Some("a b"));
        assert_eq!(t.get(2), Some(""));
        assert_eq!(t.get(3), None);
        // Fewer rows than declared, bare line feeds, an invalid lead byte.
        let t = Table::parse(b"5 \nok\n\xa4\n").unwrap();
        assert_eq!((t.declared(), t.len(), t.undecodable), (5, 2, 1));
        assert_eq!((t.get(0), t.get(1), t.get(2)), (Some("ok"), Some(""), None));
        for bad in [&b""[..], b"x\r\na", b"0\r\na", b"1234567\r\na"] {
            assert!(Table::parse(bad).is_err());
        }
    }
}
