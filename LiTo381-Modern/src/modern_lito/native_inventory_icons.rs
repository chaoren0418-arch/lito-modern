//! Native TBT/LBT inventory assets. Intrinsic asset keys never identify items.
use super::{
    inventory_icons::{decode_tbt_cancellable, MAX_TBT_BYTES},
    native_resources::NativeIconArchive,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::JoinHandle,
};
pub const MAX_VISIBLE: usize = 64;
pub const CELL: usize = 32;
fn active(c: &AtomicBool) -> Result<()> {
    ensure!(!c.load(Ordering::Acquire), "icon loading cancelled");
    Ok(())
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Source {
    pub archive: String,
    pub member: String,
    pub bytes: usize,
    pub sha256: String,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Icon {
    pub id: u16,
    pub source: Source,
    pub offset: [i8; 2],
    pub width: usize,
    pub height: usize,
    pub covered_pixels: usize,
    pub covered_zero: usize,
    pub cell_sha256: String,
    #[serde(skip)]
    pub cell: Vec<u8>,
}
impl Icon {
    pub fn decode(
        id: u16,
        archive: &str,
        member: &str,
        bytes: &[u8],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let image = decode_tbt_cancellable(bytes, cancel)?;
        ensure!(
            image.height() <= 127,
            "native signed TBT height unsupported"
        );
        let offset = image.raw_offset().map(|n| n as i8);
        let mut cell = vec![0; CELL * CELL * 4];
        let mut covered_pixels = 0;
        let mut covered_zero = 0;
        for (i, &v) in image.rgb555().iter().enumerate() {
            if !image.coverage()[i] {
                continue;
            }
            covered_pixels += 1;
            covered_zero += usize::from(v == 0);
            let x = (i % image.width()) as i32 + i32::from(offset[0]);
            let y = (i / image.width()) as i32 + i32::from(offset[1]);
            if !(0..CELL as i32).contains(&x) || !(0..CELL as i32).contains(&y) {
                continue;
            }
            let rgb = [
                ((v >> 10) & 31) as u8,
                ((v >> 5) & 31) as u8,
                (v & 31) as u8,
            ]
            .map(|v| (v << 3) | (v >> 2));
            let at = (y as usize * CELL + x as usize) * 4;
            cell[at..at + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        active(cancel)?;
        Ok(Self {
            id,
            source: Source {
                archive: archive.into(),
                member: member.into(),
                bytes: bytes.len(),
                sha256: format!("{:X}", Sha256::digest(bytes)),
            },
            offset,
            width: image.width(),
            height: image.height(),
            covered_pixels,
            covered_zero,
            cell_sha256: format!("{:X}", Sha256::digest(&cell)),
            cell,
        })
    }
    /// Native span coverage, including encoded black, is opaque. Uncovered
    /// pixels alone show the list background; no RGB-zero preview key is used.
    pub fn bgra(&self, background: [u8; 3]) -> Vec<u8> {
        self.cell
            .chunks_exact(4)
            .flat_map(|p| {
                let c = if p[3] == 0 {
                    background
                } else {
                    [p[0], p[1], p[2]]
                };
                [c[2], c[1], c[0], 255]
            })
            .collect()
    }
}
pub struct Reader {
    base: SharedArchive,
    shards: [Option<SharedArchive>; 16],
    native: NativeIconArchive,
    root: PathBuf,
    ordinary: u32,
    lito: u32,
}
fn list_count(b: &[u8]) -> Result<u32> {
    ensure!(b.len() <= 16, "icon list budget");
    let digits = b.strip_suffix(b"\r\n").context("icon count CRLF")?;
    ensure!(
        !digits.is_empty() && digits.iter().all(u8::is_ascii_digit),
        "icon count digits"
    );
    let count = std::str::from_utf8(digits)?.parse::<u32>()?;
    ensure!(count > 0 && count <= 1_000_000, "icon count bound");
    Ok(count)
}
fn shared(a: &mut SharedArchive, name: &str, limit: usize) -> Result<Option<Vec<u8>>> {
    match a.entry(name) {
        Ok(e) => ensure!(e.size as usize <= limit, "icon payload budget"),
        Err(ResourceError::MissingEntry(_)) => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    Ok(Some(a.read_entry(name)?))
}
impl Reader {
    pub fn open(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let lt = root.join("LiTo");
        let idx = super::native_world_scene::read_pin(&lt.join("LTicon.idx"), 1024 * 1024, cancel)?;
        let pak =
            super::native_world_scene::read_pin(&lt.join("LTicon.pak"), 32 * 1024 * 1024, cancel)?;
        let native = NativeIconArchive::open(&lt, idx, pak, cancel)?;
        let mut r = Self {
            base: SharedArchive::open(root, SharedArchiveKind::Sprite)?,
            shards: std::array::from_fn(|_| None),
            native,
            root: root.into(),
            ordinary: 0,
            lito: 0,
        };
        r.ordinary = list_count(&r.member("list.ico", true, 16, cancel)?.1)?;
        r.lito = list_count(&r.member("LTlist.ico", false, 16, cancel)?.1)?;
        Ok(r)
    }
    fn member(
        &mut self,
        name: &str,
        sharded: bool,
        limit: usize,
        cancel: &AtomicBool,
    ) -> Result<(String, Vec<u8>)> {
        active(cancel)?;
        if sharded {
            let shard = (name.bytes().map(usize::from).sum::<usize>()) % 16;
            if self.shards[shard].is_none() {
                self.shards[shard] = Some(SharedArchive::open(
                    &self.root,
                    SharedArchiveKind::SpriteShard(shard as u8),
                )?);
            }
            if let Some(bytes) = shared(self.shards[shard].as_mut().unwrap(), name, limit)? {
                active(cancel)?;
                return Ok((format!("Sprite{shard:02}"), bytes));
            }
        }
        if let Some(bytes) = self.native.read_optional(name, limit, cancel)? {
            return Ok(("LiTo/LTicon".into(), bytes));
        }
        let bytes =
            shared(&mut self.base, name, limit)?.context("native inventory icon member absent")?;
        active(cancel)?;
        Ok(("Sprite".into(), bytes))
    }
    pub fn load(&mut self, id: u16, cancel: &AtomicBool) -> Result<Icon> {
        ensure!(id <= i16::MAX as u16, "negative native icon identity");
        let (name, sharded) = if id < 20000 {
            ensure!(u32::from(id) < self.ordinary, "ordinary icon outside list");
            (format!("{id}.tbt"), true)
        } else {
            let lt = u32::from(id) - 20000;
            ensure!(lt < self.lito, "LiTo icon outside list");
            (format!("{lt}.lbt"), false)
        };
        let (archive, bytes) = self.member(&name, sharded, MAX_TBT_BYTES, cancel)?;
        Icon::decode(id, &archive, &name, &bytes, cancel)
    }
}
type Entry = std::result::Result<Icon, String>;
struct Completion {
    reader: Option<Reader>,
    entries: Vec<(u16, Entry)>,
    failed: bool,
}
struct Job {
    worker: Option<JoinHandle<()>>,
    receiver: mpsc::Receiver<Completion>,
    cancel: Arc<AtomicBool>,
}
impl Job {
    fn start(root: PathBuf, mut reader: Option<Reader>, ids: Vec<u16>) -> Result<Self> {
        let (send, receiver) = mpsc::sync_channel(1);
        let cancel = Arc::new(AtomicBool::new(false));
        let token = cancel.clone();
        let worker = std::thread::Builder::new()
            .name("native-inventory-icons".into())
            .spawn(move || {
                let mut entries = Vec::new();
                let error = if reader.is_none() {
                    match Reader::open(&root, &token) {
                        Ok(r) => {
                            reader = Some(r);
                            None
                        }
                        Err(e) => Some(format!("{e:#}")),
                    }
                } else {
                    None
                };
                for id in ids {
                    if token.load(Ordering::Acquire) {
                        break;
                    }
                    let value = if let Some(error) = &error {
                        Err(error.clone())
                    } else {
                        reader
                            .as_mut()
                            .unwrap()
                            .load(id, &token)
                            .map_err(|e| format!("{e:#}"))
                    };
                    if token.load(Ordering::Acquire) {
                        break;
                    }
                    entries.push((id, value));
                }
                let _ = send.send(Completion {
                    reader,
                    entries,
                    failed: false,
                });
            })?;
        Ok(Self {
            worker: Some(worker),
            receiver,
            cancel,
        })
    }
    fn poll(&mut self) -> Option<Completion> {
        if !self.worker.as_ref().is_some_and(|w| w.is_finished()) {
            return None;
        }
        let _ = self.worker.take().unwrap().join();
        Some(self.receiver.try_recv().unwrap_or(Completion {
            reader: None,
            entries: vec![],
            failed: true,
        }))
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}
/// Cache only intrinsic assets requested by current visible rows. It never
/// captures character/item IDs or publishes row-index assignments.
pub struct Service {
    root: PathBuf,
    reader: Option<Reader>,
    job: Option<Job>,
    wanted: BTreeSet<u16>,
    cache: BTreeMap<u16, Entry>,
    stopped: bool,
}
impl Service {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            reader: None,
            job: None,
            wanted: BTreeSet::new(),
            cache: BTreeMap::new(),
            stopped: false,
        }
    }
    pub fn tick(&mut self, ids: &[u16]) -> bool {
        let wanted = if self.stopped {
            BTreeSet::new()
        } else {
            ids.iter().take(MAX_VISIBLE).copied().collect()
        };
        let mut changed = self.wanted != wanted;
        if changed {
            if let Some(job) = &self.job {
                job.cancel.store(true, Ordering::Release);
            }
        }
        self.wanted = wanted;
        self.cache.retain(|id, _| self.wanted.contains(id));
        if let Some(done) = self.job.as_mut().and_then(Job::poll) {
            self.job = None;
            changed = true;
            if !self.stopped {
                self.reader = done.reader;
                if done.failed {
                    for &id in &self.wanted {
                        self.cache
                            .insert(id, Err("icon worker ended without result".into()));
                    }
                }
                for (id, result) in done.entries {
                    if self.wanted.contains(&id) {
                        self.cache.insert(id, result);
                    }
                }
            }
        }
        if !self.stopped && self.job.is_none() {
            let missing: Vec<_> = self
                .wanted
                .iter()
                .filter(|id| !self.cache.contains_key(id))
                .copied()
                .collect();
            if !missing.is_empty() {
                match Job::start(self.root.clone(), self.reader.take(), missing.clone()) {
                    Ok(job) => self.job = Some(job),
                    Err(e) => {
                        for id in missing {
                            self.cache.insert(id, Err(e.to_string()));
                        }
                        changed = true;
                    }
                }
            }
        }
        changed
    }
    pub fn icon(&self, id: u16) -> Option<&Icon> {
        self.cache.get(&id).and_then(|r| r.as_ref().ok())
    }
    pub fn error(&self, id: u16) -> Option<&str> {
        self.cache
            .get(&id)
            .and_then(|r| r.as_ref().err())
            .map(String::as_str)
    }
    pub fn cached(&self) -> usize {
        self.cache.len()
    }
    pub fn quiescent(&self) -> bool {
        self.job.is_none()
    }
    pub fn cancel(&mut self) {
        self.stopped = true;
        self.wanted.clear();
        self.cache.clear();
        self.reader = None;
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Release);
        }
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        self.cancel();
    }
}
