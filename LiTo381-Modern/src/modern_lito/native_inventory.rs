//! Current formal inventory records and deltas, owned by a selected GAME character.
//! Wire object IDs are not catalog IDs. No local spending or equipment inference.
use anyhow::{ensure, Context, Result};
use std::collections::BTreeMap;

pub const MAX_ITEMS: usize = 4096;
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
        let name: String = name
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect();
        format!(
            "{} × {}",
            if name.is_empty() {
                "未命名物品"
            } else {
                &name
            },
            self.count
        )
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
    if !matches!(op, 0x0e | 0x16..=0x1a | 0x6f | 0x7c) {
        return Ok(None);
    }
    ensure!(body.len() <= 65533, "inventory packet bound");
    let mut r = Reader { bytes: body, at: 1 };
    let update = match op {
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
        match update {
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
        Ok(true)
    }
}
