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
        })
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
        Self::parse(&tables[0], &tables[1], sources, cancel)
    }
    /// The recovered helper checks the actual BASE count first, then routes
    /// references >=20000 to LTdesc with the offset removed. No recursive lookup.
    pub fn resolve(&self, token: &str) -> Result<String> {
        ensure!(
            !token.trim().is_empty()
                && token.len() <= MAX_INPUT_BYTES
                && !token.chars().any(char::is_control),
            "invalid item name"
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
            ensure!(
                bytes.get(at).is_some_and(u8::is_ascii_digit),
                "invalid description reference"
            );
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
            let value = if (id as usize) < self.base.available_count() {
                self.base.entry(id)?
            } else if id >= 20000 {
                self.lito.entry(id - 20000)?
            } else {
                self.base.entry(id)?
            };
            // Preserve the complete original name on unsupported runtime/format
            // semantics, rather than publishing a partially substituted label.
            ensure!(
                !value.trim().is_empty()
                    && !value.contains(['$', '\\'])
                    && !value
                        .as_bytes()
                        .windows(2)
                        .any(|w| w[0] == b'%' && w[1].is_ascii_alphabetic()),
                "unsupported description entry"
            );
            append(&mut out, value)?;
            start = at;
        }
        append(&mut out, &token[start..])?;
        Ok(out)
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Label {
    pub id: u32,
    pub name: String,
    pub count: u32,
    pub text: String,
    pub resolved: bool,
}
pub fn label(item: &Item, catalog: Option<&Catalog>) -> Label {
    let translated = catalog.and_then(|c| c.resolve(&item.name).ok());
    let resolved = translated.is_some();
    let name = translated.unwrap_or_else(|| item.name.clone());
    Label {
        id: item.id,
        count: item.count,
        text: item.label_with_name(&name),
        name,
        resolved,
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
