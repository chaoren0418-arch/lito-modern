//! Current formal inventory records and deltas, owned by a selected GAME character.
//! Wire object IDs are not catalog IDs. No local spending or equipment inference.
use anyhow::{ensure, Context, Result};
use std::collections::BTreeMap;

pub const MAX_ITEMS: usize = 4096;
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Mail {
    pub id: u32,
    pub icon: u16,
    pub code: u16,
    pub sender: String,
    pub receiver: String,
    pub subject: String,
    pub content: String,
    pub template: u8,
    pub date: String,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Owner {
    pub connection: u64,
    pub selection: u64,
    pub character: String,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Item {
    pub id: u32,
    pub use_type: u8,
    pub charge: u8,
    pub icon: u16,
    pub blessing: u8,
    pub count: u32,
    pub identified: u8,
    pub name: String,
    pub detail: Vec<u8>,
}
impl Item {
    pub fn label(&self) -> String {
        self.label_with_name(&self.name)
    }
    pub fn label_with_name(&self, name: &str) -> String {
        // Native 4437F0 regroups a remaining trailing `(digits)` (a charge
        // count, for example) with thousands separators.
        let shown = super::native_text_format::inventory_view_name(&display_name(name, self.count));
        let label = format!("{shown} × {}", self.count);
        if matches!(self.use_type, 3 | 4) {
            if (1..=127).contains(&self.charge) {
                format!("{label}（剩餘次數 {}）", self.charge)
            } else { format!("{label}（無可用次數）") }
        } else { label }
    }
}
/// Name shown next to a separately listed authoritative count. The server view
/// name ends with ` (count)` for stacks above one; that exact suffix is dropped
/// so the count is not shown twice. A different number (a charge count, or a
/// name that was not refreshed with the count) is kept as sent. Presentation
/// only: the stored wire name is never changed.
pub fn display_name(name: &str, count: u32) -> String {
    let name: String = name
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let name = match name.strip_suffix(&format!(" ({count})")) {
        Some(base) if count > 1 => base.to_owned(),
        _ => name,
    };
    if name.is_empty() {
        "未命名物品".into()
    } else {
        name
    }
}
#[cfg(test)]
mod display_name_tests {
    use super::{display_name, Item};
    fn item(name: &str, count: u32, use_type: u8, charge: u8) -> Item {
        Item {
            id: 1,
            use_type,
            charge,
            icon: 0,
            blessing: 0,
            count,
            identified: 1,
            name: name.into(),
            detail: vec![],
        }
    }
    #[test]
    fn server_count_suffix_is_not_repeated_beside_the_authoritative_count() {
        assert_eq!(item("治癒藥水 (5)", 5, 0, 0).label(), "治癒藥水 × 5");
        assert_eq!(item("治癒藥水", 5, 0, 0).label(), "治癒藥水 × 5");
        assert_eq!(item("治癒藥水", 1, 0, 0).label(), "治癒藥水 × 1");
        // Equipped marker precedes the count in the server view name.
        assert_eq!(item("箭 ($9) (300)", 300, 0, 0).label(), "箭 ($9) × 300");
    }
    #[test]
    fn only_the_trailing_exact_stack_count_is_removed() {
        // Count 1 is never appended by the server: `(1)` here is a charge count.
        assert_eq!(display_name("松木魔杖 (1)", 1), "松木魔杖 (1)");
        // Charge count then stack count: one suffix goes, the charge stays.
        assert_eq!(display_name("松木魔杖 (3) (3)", 3), "松木魔杖 (3)");
        assert_eq!(display_name("松木魔杖 (7) (2)", 2), "松木魔杖 (7)");
        // A stale or unrelated number is not a duplicate of the count.
        assert_eq!(display_name("治癒藥水 (5)", 4), "治癒藥水 (5)");
        assert_eq!(display_name("治癒藥水 (25)", 5), "治癒藥水 (25)");
        assert_eq!(display_name("治癒藥水(5)", 5), "治癒藥水(5)");
        assert_eq!(display_name("(5) 治癒藥水", 5), "(5) 治癒藥水");
        assert_eq!(display_name("治癒藥水 (5) ", 5), "治癒藥水 (5) ");
    }
    #[test]
    fn control_characters_and_empty_names_keep_their_presentation() {
        assert_eq!(display_name("", 1), "未命名物品");
        assert_eq!(display_name(" (2)", 2), "未命名物品");
        assert_eq!(display_name("a\u{c}b (2)", 2), "a b");
        assert_eq!(item("", 1, 0, 0).label(), "未命名物品 × 1");
        assert_eq!(item("魔杖 (2)", 2, 3, 4).label(), "魔杖 × 2（剩餘次數 4）");
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Inventory {
    pub owner: Option<Owner>,
    pub items: BTreeMap<u32, Item>,
    /// S7C is additive, not a replace-all snapshot. The server sends removals.
    pub batches: u64,
    pub revision: u64,
    pub orphan_updates: u64,
    pub mail: Option<Mail>,
    pub mail_revision: u64,
    selection: u64,
    /// Same-socket reselect does not replay S7C. Keep one inactive character
    /// store while the selector is open, including authoritative late deltas.
    #[serde(skip)]
    retained: Option<Box<Inventory>>,
}
enum Update {
    Add(Vec<Item>, bool),
    Remove(u32),
    Name(u32, String),
    Charge(u32, u8),
    Count(u32, u32),
    Blessing(u32, u8),
    Extended(u32, String, u32, Vec<u8>),
    ReadMail(Mail),
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).context("inventory offset")?;
        let b = self
            .bytes
            .get(self.at..end)
            .context("truncated inventory field")?;
        self.at = end;
        Ok(b)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn word(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into()?))
    }
    fn dword(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn id(&mut self) -> Result<u32> {
        let id = self.dword()?;
        ensure!(id != 0, "zero inventory object");
        Ok(id)
    }
    fn string(&mut self) -> Result<String> {
        use windows::Win32::Globalization::{MultiByteToWideChar, MB_ERR_INVALID_CHARS};
        let n = self.bytes[self.at..]
            .iter()
            .take(256)
            .position(|b| *b == 0)
            .context("inventory string bound")?;
        let bytes = &self.take(n + 1)?[..n];
        if n == 0 {
            return Ok(String::new());
        }
        let mut wide = vec![0u16; n];
        let len = unsafe { MultiByteToWideChar(950, MB_ERR_INVALID_CHARS, bytes, Some(&mut wide)) };
        ensure!(len > 0, "invalid inventory MS950");
        // Keep native formatting/control bytes in the model; labels sanitize
        // controls only for the current Unicode list presentation.
        Ok(String::from_utf16(&wide[..len as usize])?)
    }
    fn detail(&mut self) -> Result<Vec<u8>> {
        let n = usize::from(self.byte()?);
        Ok(self.take(n)?.to_vec())
    }
    fn wide(&mut self, max: usize) -> Result<String> {
        let mut units = Vec::new();
        loop {
            let unit = self.word()?;
            if unit == 0 { break; }
            ensure!(units.len() < max, "mail UTF16 bound");
            units.push(unit);
        }
        let text = String::from_utf16(&units)?;
        ensure!(!text.chars().any(|c| c.is_control() && !matches!(c, '\r' | '\n' | '\t')), "mail control code");
        Ok(text)
    }
    fn item(&mut self) -> Result<Item> {
        let id = self.id()?;
        let use_type = self.byte()?;
        let charge = self.byte()?;
        let icon = self.word()?;
        let blessing = self.byte()?;
        ensure!(blessing <= 3, "inventory initial blessing");
        let count = self.dword()?;
        ensure!(count > 0, "empty inventory add");
        Ok(Item {
            id,
            use_type,
            charge,
            icon,
            blessing,
            count,
            identified: self.byte()?,
            name: self.string()?,
            detail: self.detail()?,
        })
    }
}
fn parse(body: &[u8]) -> Result<Option<Update>> {
    let op = *body.first().context("empty inventory packet")?;
    if !matches!(op, 0x0e | 0x16..=0x1a | 0x5e | 0x6f | 0x7c) {
        return Ok(None);
    }
    ensure!(body.len() <= 65533, "inventory packet bound");
    let mut r = Reader { bytes: body, at: 1 };
    let update = match op {
        0x5e => {
            let mail = Mail { id: r.id()?, icon: r.word()?, code: r.word()?,
                sender: r.string()?, receiver: r.string()?, subject: r.wide(20)?,
                content: r.wide(256)?, template: r.byte()?, date: r.string()? };
            for name in [&mail.sender, &mail.receiver] {
                ensure!(!name.is_empty() && !name.chars().any(char::is_control), "mail name");
                super::native_login::encode_text(name, 27)?;
            }
            let date = mail.date.as_bytes();
            ensure!(date.len() == 8 && date.iter().enumerate().all(|(i, b)|
                if i == 2 || i == 5 { *b == b'/' } else { b.is_ascii_digit() }), "mail date");
            Update::ReadMail(mail)
        }
        0x16 => Update::Add(vec![r.item()?], false),
        0x7c => {
            let count = r.byte()?;
            let mut rows = Vec::with_capacity(count as usize);
            for _ in 0..count {
                rows.push(r.item()?);
            }
            Update::Add(rows, true)
        }
        0x17 => Update::Remove(r.id()?),
        0x18 => Update::Name(r.id()?, r.string()?),
        0x19 => Update::Charge(r.id()?, r.byte()?),
        0x1a => Update::Count(r.id()?, r.dword()?),
        0x0e => {
            let id = r.id()?;
            let b = r.byte()?;
            ensure!(
                b <= 3 || (128..=131).contains(&b),
                "inventory blessing update"
            );
            Update::Blessing(id, b)
        }
        0x6f => {
            let id = r.id()?;
            let name = r.string()?;
            let n = r.dword()?;
            ensure!(n > 0, "empty extended inventory count");
            Update::Extended(id, name, n, r.detail()?)
        }
        _ => unreachable!(),
    };
    ensure!(r.at == body.len(), "trailing inventory fields");
    Ok(Some(update))
}
impl Inventory {
    pub fn begin(&mut self, connection: u64, character: &str) -> Result<()> {
        ensure!(connection > 0 && !character.is_empty(), "inventory owner");
        let next = self
            .selection
            .checked_add(1)
            .context("inventory selection exhausted")?;
        self.suspend();
        if let Some(previous) = self.retained.as_ref().filter(|p| {
            p.owner
                .as_ref()
                .is_some_and(|o| o.connection == connection && o.character == character)
        }) {
            self.items = previous.items.clone();
            self.batches = previous.batches;
            self.revision = previous.revision;
            self.orphan_updates = previous.orphan_updates;
        }
        self.selection = next;
        self.owner = Some(Owner {
            connection,
            selection: next,
            character: character.into(),
        });
        Ok(())
    }
    pub fn clear(&mut self) {
        self.retained = None;
        self.clear_active();
    }
    fn clear_active(&mut self) {
        self.owner = None;
        self.items.clear();
        self.batches = 0;
        self.revision = 0;
        self.orphan_updates = 0;
        self.mail = None;
        self.mail_revision = 0;
    }
    pub fn suspend(&mut self) {
        if self.owner.is_some() {
            let mut previous = self.clone();
            previous.retained = None;
            self.retained = Some(Box::new(previous));
        }
        self.clear_active();
    }
    pub fn confirm_selection(&mut self) {
        self.retained = None;
    }
    pub fn reject_selection(&mut self) {
        self.clear_active();
    }
    pub fn accept(&mut self, connection: u64, body: &[u8]) -> Result<bool> {
        if self.owner.is_none() {
            if let Some(previous) = &mut self.retained {
                return previous.accept(connection, body);
            }
        }
        let Some(update) = parse(body)? else {
            return Ok(false);
        };
        ensure!(
            self.owner
                .as_ref()
                .is_some_and(|o| o.connection == connection),
            "inventory outside selected connection"
        );
        let revision = self
            .revision
            .checked_add(1)
            .context("inventory revision exhausted")?;
        let mut next = self.items.clone();
        let mut batches = self.batches;
        let mut orphan = false;
        let mut mail = self.mail.clone();
        let mut mail_revision = self.mail_revision;
        match update {
            Update::ReadMail(value) => {
                if let Some(item) = next.get_mut(&value.id).filter(|i| i.use_type == 13) {
                    item.icon = value.icon;
                    mail_revision = mail_revision.checked_add(1).context("mail revision exhausted")?;
                    mail = Some(value);
                } else { orphan = true; }
            }
            Update::Add(rows, batch) => {
                for item in rows {
                    next.insert(item.id, item);
                }
                ensure!(next.len() <= MAX_ITEMS, "inventory item budget");
                if batch {
                    batches = batches
                        .checked_add(1)
                        .context("inventory batch count exhausted")?;
                }
            }
            Update::Remove(id) => {
                orphan = next.remove(&id).is_none();
                if mail.as_ref().is_some_and(|m| m.id == id) { mail = None; }
            }
            Update::Name(id, name) => {
                if let Some(i) = next.get_mut(&id) {
                    i.name = name
                } else {
                    orphan = true
                }
            }
            Update::Charge(id, charge) => {
                if let Some(i) = next.get_mut(&id) {
                    i.charge = charge
                } else {
                    orphan = true
                }
            }
            Update::Count(id, count) => {
                if let Some(i) = next.get_mut(&id) {
                    i.count = count
                } else {
                    orphan = true
                }
            }
            Update::Blessing(id, b) => {
                if let Some(i) = next.get_mut(&id) {
                    i.blessing = b
                } else {
                    orphan = true
                }
            }
            Update::Extended(id, name, count, detail) => {
                if let Some(i) = next.get_mut(&id) {
                    i.name = name;
                    i.count = count;
                    i.detail = detail
                } else {
                    orphan = true
                }
            }
        }
        let orphans = self
            .orphan_updates
            .checked_add(u64::from(orphan))
            .context("inventory orphan count exhausted")?;
        self.items = next;
        self.revision = revision;
        self.batches = batches;
        self.orphan_updates = orphans;
        self.mail = mail;
        self.mail_revision = mail_revision;
        Ok(true)
    }
}
