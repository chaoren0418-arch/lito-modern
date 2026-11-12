//! Native description namespaces and an owned background loader for the bag.
//! Names are presentation only: never change wire fields or infer catalog IDs.
use super::{
    descriptions::DescriptionCatalog,
    native_inventory::Item,
    resources::{SharedArchive, SharedArchiveKind},
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::JoinHandle,
};

pub const MAX_INPUT_BYTES: usize = 768; // Includes UTF-8 expansion of 255 MS950 bytes.
pub const MAX_OUTPUT_BYTES: usize = 4096;
#[derive(Clone, Debug, serde::Serialize)]
pub struct Source {
    pub archive: String,
    pub member: String,
    pub bytes: usize,
    pub sha256: String,
    pub declared: u32,
    pub available: usize,
}
pub struct Catalog {
    base: DescriptionCatalog,
    lito: DescriptionCatalog,
    pub sources: Vec<Source>,
    /// `string-c.tbl`／`LTstring-c.tbl` for notices and system messages. A
    /// failure here never withholds the item names; see `strings_error`.
    pub strings: Option<super::native_messages::Strings>,
    pub strings_error: Option<String>,
    pub string_sources: Vec<Source>,
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "inventory names cancelled");
    Ok(())
}
impl Catalog {
    pub fn from_tables(base: &[u8], lito: &[u8]) -> Result<Self> {
        Self::parse(base, lito, Vec::new(), &AtomicBool::new(false))
    }
    fn parse(base: &[u8], lito: &[u8], sources: Vec<Source>, cancel: &AtomicBool) -> Result<Self> {
        Ok(Self {
            base: DescriptionCatalog::parse_cancellable(base, cancel)?,
            lito: DescriptionCatalog::parse_cancellable(lito, cancel)?,
            sources,
            strings: None,
            strings_error: None,
            string_sources: Vec::new(),
        })
    }
    /// The base table comes from the shared Text archive and the LiTo table
    /// from LiTo/LTtext, the same routing as the description tables.
    fn load_strings(
        lt: &Path,
        index: super::native_resources::BytePin,
        pak: super::native_resources::BytePin,
        base_archive: &mut Option<SharedArchive>,
        root: &Path,
        sources: &mut Vec<Source>,
        cancel: &AtomicBool,
    ) -> Result<super::native_messages::Strings> {
        use super::native_messages::{Strings, Table, MAX_TABLE_BYTES};
        let mut tables = Vec::new();
        for member in ["string-c.tbl", "ltstring-c.tbl"] {
            active(cancel)?;
            let (bytes, archive) = match super::native_resources::read_native_text_optional(
                lt, index, pak, member, cancel,
            )? {
                Some(bytes) => (bytes, "LiTo/LTtext"),
                None => {
                    if base_archive.is_none() {
                        *base_archive = Some(SharedArchive::open(root, SharedArchiveKind::Text)?);
                    }
                    let a = base_archive.as_mut().unwrap();
                    ensure!(
                        a.entry(member)?.size as usize <= MAX_TABLE_BYTES,
                        "string table budget"
                    );
                    (a.read_entry(member)?, "Text")
                }
            };
            let table = Table::parse(&bytes).with_context(|| member.to_owned())?;
            sources.push(Source {
                archive: archive.into(),
                member: member.into(),
                bytes: bytes.len(),
                sha256: format!("{:X}", Sha256::digest(&bytes)),
                declared: table.declared(),
                available: table.len(),
            });
            tables.push(table);
        }
        let lito = tables.pop().context("string tables")?;
        let base = tables.pop().context("string tables")?;
        Ok(Strings { base, lito })
    }
    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let lt = root.join("LiTo");
        let index =
            super::native_world_scene::read_pin(&lt.join("LTtext.idx"), 1024 * 1024, cancel)?;
        let pak = super::native_world_scene::read_pin(&lt.join("LTtext.pak"), 1024 * 1024, cancel)?;
        let mut base_archive = None;
        let mut tables = Vec::new();
        let mut sources = Vec::new();
        for member in ["desc-c.tbl", "ltdesc-c.tbl"] {
            let (bytes, archive) = match super::native_resources::read_native_text_optional(
                &lt, index, pak, member, cancel,
            )? {
                Some(bytes) => (bytes, "LiTo/LTtext"),
                None => {
                    if base_archive.is_none() {
                        base_archive = Some(SharedArchive::open(root, SharedArchiveKind::Text)?);
                    }
                    let a = base_archive.as_mut().unwrap();
                    ensure!(
                        a.entry(member)?.size as usize <= super::descriptions::MAX_TABLE_BYTES,
                        "description table budget"
                    );
                    (a.read_entry(member)?, "Text")
                }
            };
            active(cancel)?;
            let parsed = DescriptionCatalog::parse_cancellable(&bytes, cancel)?;
            sources.push(Source {
                archive: archive.into(),
                member: member.into(),
                bytes: bytes.len(),
                sha256: format!("{:X}", Sha256::digest(&bytes)),
                declared: parsed.declared_count(),
                available: parsed.available_count(),
            });
            tables.push(bytes);
        }
        let mut string_sources = Vec::new();
        let strings = Self::load_strings(
            &lt,
            index,
            pak,
            &mut base_archive,
            root,
            &mut string_sources,
            cancel,
        );
        active(cancel)?;
        let mut catalog = Self::parse(&tables[0], &tables[1], sources, cancel)?;
        match strings {
            Ok(strings) => {
                catalog.string_sources = string_sources;
                catalog.strings = Some(strings);
            }
            Err(error) => catalog.strings_error = Some(format!("{error:#}")),
        }
        Ok(catalog)
    }
    /// 4A44D0 expands every S10 argument with 4D44A0; text this module
    /// would reject is shown as received.
    pub fn expand_argument(&self, argument: &str, character: Option<&str>) -> String {
        self.expand(argument, character)
            .unwrap_or_else(|_| argument.to_owned())
    }
    /// [`Self::expand`] without a runtime name: an entry holding the `%i`
    /// template is then an error, so the caller keeps the original text.
    pub fn resolve(&self, token: &str) -> Result<String> {
        self.expand(token, None)
    }
    /// Native 4D44A0. `$digits` (one optional `;` consumed) is looked up in
    /// BASE when the ID is below its actual row count, otherwise in LTdesc
    /// with 20000 removed; an ID in neither table yields `???`. The entry is
    /// copied literally and never rescanned, so there is no recursive
    /// reference, and its inline `\f?` codes are left for the text writer
    /// (`native_text_format`). The first `%i` of an entry is replaced by the
    /// runtime string at 75F71C, which 492F1E fills from the entered
    /// character's record: `character` here. `$` before a non-digit is
    /// dropped.
    ///
    /// Not native: budgets are this module's, a reference above `u32` and a
    /// wholly blank result are errors.
    pub fn expand(&self, token: &str, character: Option<&str>) -> Result<String> {
        ensure!(
            !token.trim().is_empty()
                && token.len() <= MAX_INPUT_BYTES
                && !token.chars().any(char::is_control),
            "invalid item name"
        );
        ensure!(
            character.is_none_or(|c| !c.is_empty() && !c.chars().any(char::is_control)),
            "invalid runtime name"
        );
        let bytes = token.as_bytes();
        let mut at = 0;
        let mut start = 0;
        let mut count = 0;
        let mut out = String::new();
        let append = |out: &mut String, value: &str| -> Result<()> {
            ensure!(
                out.len() + value.len() <= MAX_OUTPUT_BYTES,
                "resolved name budget"
            );
            out.push_str(value);
            Ok(())
        };
        while at < bytes.len() {
            if bytes[at] != b'$' {
                at += 1;
                continue;
            }
            append(&mut out, &token[start..at])?;
            at += 1;
            start = at;
            if !bytes.get(at).is_some_and(u8::is_ascii_digit) {
                continue;
            }
            let mut id = 0u32;
            while let Some(d) = bytes.get(at).filter(|d| d.is_ascii_digit()) {
                id = id
                    .checked_mul(10)
                    .and_then(|n| n.checked_add(u32::from(*d - b'0')))
                    .context("description reference overflow")?;
                at += 1;
            }
            if bytes.get(at) == Some(&b';') {
                at += 1;
            }
            count += 1;
            ensure!(count <= 128, "description reference budget");
            let entry = if (id as usize) < self.base.available_count() {
                self.base.entry(id).ok()
            } else if id >= 20000 && ((id - 20000) as usize) < self.lito.available_count() {
                self.lito.entry(id - 20000).ok()
            } else {
                None
            };
            match entry.map(|e| (e, e.find("%i"))) {
                None => append(&mut out, "???")?,
                Some((e, None)) => append(&mut out, e)?,
                Some((e, Some(p))) => {
                    append(&mut out, &e[..p])?;
                    append(&mut out, character.context("runtime name unavailable")?)?;
                    append(&mut out, &e[p + 2..])?;
                }
            }
            start = at;
        }
        append(&mut out, &token[start..])?;
        ensure!(!out.trim().is_empty(), "blank resolved name");
        Ok(out)
    }
}
/// Hover/world form of an expanded name: escapes removed.
pub fn plain(name: &str) -> String {
    super::native_text_format::plain(name)
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Label {
    pub id: u32,
    pub name: String,
    pub count: u32,
    pub text: String,
    pub resolved: bool,
    /// Native 442980 name colour (RGB555) for the identified/blessing pair;
    /// the normal white where the original table has no entry.
    pub color: u16,
}
pub fn label(item: &Item, catalog: Option<&Catalog>) -> Label {
    label_as(item, catalog, None)
}
/// `character` is the entered character's name, the `%i` runtime string.
/// A resolved name is shown without its inline format codes.
pub fn label_as(item: &Item, catalog: Option<&Catalog>, character: Option<&str>) -> Label {
    let translated = catalog
        .and_then(|c| c.expand(&item.name, character).ok())
        .map(|name| plain(&name));
    let resolved = translated.is_some();
    let name = translated.unwrap_or_else(|| item.name.clone());
    Label {
        id: item.id,
        count: item.count,
        text: item.label_with_name(&name),
        name,
        resolved,
        color: super::native_text_format::name_color(item.identified, item.blessing)
            .unwrap_or(super::native_text_format::WHITE),
    }
}
/// One resource root per service; the worker never captures a character/item.
/// Poll joins only finished work. Cancellation discards even completed results.
pub struct Service {
    receiver: mpsc::Receiver<Result<Catalog>>,
    worker: Option<JoinHandle<()>>,
    cancel: Arc<AtomicBool>,
    catalog: Option<Catalog>,
    pub error: Option<String>,
}
impl Service {
    pub fn start(root: PathBuf) -> Result<Self> {
        let (send, receiver) = mpsc::sync_channel(1);
        let cancel = Arc::new(AtomicBool::new(false));
        let token = cancel.clone();
        let worker = std::thread::Builder::new()
            .name("native-inventory-names".into())
            .spawn(move || {
                let _ = send.send(Catalog::load(&root, &token));
            })?;
        Ok(Self {
            receiver,
            worker: Some(worker),
            cancel,
            catalog: None,
            error: None,
        })
    }
    pub fn poll(&mut self) -> bool {
        if !self.worker.as_ref().is_some_and(|w| w.is_finished()) {
            return false;
        }
        let result = self
            .receiver
            .try_recv()
            .context("name loader ended without result")
            .and_then(|v| v);
        let _ = self.worker.take().unwrap().join();
        if !self.cancel.load(Ordering::Acquire) {
            match result {
                Ok(c) => self.catalog = Some(c),
                Err(e) => self.error = Some(format!("{e:#}")),
            }
        }
        true
    }
    pub fn catalog(&self) -> Option<&Catalog> {
        self.catalog.as_ref()
    }
    pub fn quiescent(&self) -> bool {
        self.worker.is_none()
    }
    pub fn cancel(&mut self) {
        self.cancel.store(true, Ordering::Release);
        self.catalog = None;
        self.error = None;
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        self.cancel();
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}

#[cfg(test)]
mod expand_tests {
    use super::*;
    fn catalog() -> Catalog {
        Catalog::from_tables(
            b"6\r\nSword\r\n\r\n%i's ring of %i\r\n\\fTtrial knight\r\n$0\r\n150% power",
            b"3\r\nLito sword\r\nLito shield",
        )
        .unwrap()
    }
    fn item(name: &str, count: u32, identified: u8, blessing: u8) -> Item {
        Item {
            id: 7,
            use_type: 0,
            charge: 0,
            icon: 80,
            blessing,
            count,
            identified,
            name: name.into(),
            detail: vec![],
        }
    }
    #[test]
    fn missing_references_are_question_marks_and_entries_are_never_rescanned() {
        let c = catalog();
        assert_eq!(c.resolve("$0 (2)").unwrap(), "Sword (2)");
        assert_eq!(c.resolve("$0;x$00000;").unwrap(), "SwordxSword");
        assert_eq!(c.resolve("$5").unwrap(), "150% power");
        assert_eq!(c.resolve("$20001").unwrap(), "Lito shield");
        for missing in ["$6", "$19999", "$20002", "$20003", "$4000000000"] {
            assert_eq!(c.resolve(missing).unwrap(), "???", "{missing}");
        }
        assert_eq!(c.resolve("+1 $6 (3)").unwrap(), "+1 ??? (3)");
        // Entry 4 is the literal text `$0`; the original copies it as is.
        assert_eq!(c.resolve("$4").unwrap(), "$0");
        assert!(c.resolve("$4294967296").is_err());
    }
    #[test]
    fn dollar_before_a_non_digit_is_dropped_and_blank_results_are_rejected() {
        let c = catalog();
        assert_eq!(c.resolve("$x").unwrap(), "x");
        assert_eq!(c.resolve("1$").unwrap(), "1");
        assert_eq!(c.resolve("$-1").unwrap(), "-1");
        assert_eq!(c.resolve("a$$0").unwrap(), "aSword");
        assert_eq!(c.resolve("a$1b").unwrap(), "ab");
        assert!(c.resolve("$").is_err());
        assert!(c.resolve("$1").is_err());
        assert!(c.resolve("$1 ").is_err());
    }
    #[test]
    fn runtime_template_takes_the_character_name_once() {
        let c = catalog();
        assert!(c.resolve("$2").is_err());
        assert_eq!(
            c.expand("$2 (1)", Some("Hero")).unwrap(),
            "Hero's ring of %i (1)"
        );
        assert!(c.expand("$2", Some("")).is_err());
        assert!(c.expand("$0", Some("a\nb")).is_err());
        assert_eq!(c.expand("$0", Some("Hero")).unwrap(), "Sword");
        let i = item("$2", 1, 1, 1);
        assert!(!label(&i, Some(&c)).resolved);
        assert_eq!(label(&i, Some(&c)).name, "$2");
        let row = label_as(&i, Some(&c), Some("Hero"));
        assert!(row.resolved);
        assert_eq!(row.text, "Hero's ring of %i × 1");
    }
    #[test]
    fn labels_drop_format_codes_and_carry_the_native_name_colour() {
        let c = catalog();
        assert_eq!(c.resolve("$3").unwrap(), "\\fTtrial knight");
        let row = label(&item("$3 (5000)", 5000, 1, 0), Some(&c));
        assert_eq!(row.name, "trial knight (5000)");
        assert_eq!(row.text, "trial knight × 5000");
        assert_eq!(row.color, 0x7fd6);
        // A number that is not the stack count stays and is regrouped.
        let row = label(&item("$3 (5000)", 1, 1, 2), Some(&c));
        assert_eq!(row.text, "trial knight (5,000) × 1");
        assert_eq!(row.color, 0x5ca7);
        assert_eq!(label(&item("$0", 1, 1, 1), Some(&c)).color, 0x7fff);
        assert_eq!(label(&item("$0", 1, 0, 0), Some(&c)).color, 0x5ef7);
        assert_eq!(label(&item("$0", 1, 1, 3), Some(&c)).color, 0x5ef7);
        assert_eq!(label(&item("$0", 1, 1, 131), Some(&c)).color, 0x7fff);
        // Unresolved names keep the wire text, format codes included.
        let raw = label(&item("\\fT$3", 1, 1, 1), None);
        assert_eq!((raw.resolved, raw.name.as_str()), (false, "\\fT$3"));
    }
}