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
/// LiTo S1E skill-row icons (`word_508648`, 176 signed WORDs). Source: unpacked
/// PE `LiTo.unpacked-oep-current.exe` SHA256 4271D92F…5C2AB6E9, VA 00508648,
/// file offset 0x108648. Index is skill ID - 1 (S1E bit `group*8+bit`).
/// The original handler (004AEEB5..004AEEF3) never reads shared SpellIcon.idx;
/// that file differs for skills 79/111/118–120/167/172 and stops at 168.
pub const LITO_SPELL_ICONS: [u16; 176] = [
    383, 384, 385, 386, 387, 657, 663, 775, 388, 389, 390, 391, 392, 755, 651, 659,
    393, 394, 395, 396, 397, 655, 848, 0, 398, 399, 400, 401, 402, 647, 746, 793,
    403, 404, 405, 406, 407, 643, 791, 749, 408, 409, 410, 411, 412, 661, 850, 726,
    413, 414, 415, 1578, 417, 622, 1570, 758, 418, 419, 420, 421, 422, 649, 784, 822,
    1579, 424, 425, 426, 427, 653, 752, 1571, 428, 429, 1577, 1582, 432, 715, 1610, 1576,
    0, 0, 0, 0, 0, 0, 1624, 1887, 1881, 2349, 2343, 0, 0, 0, 0, 0,
    1097, 1100, 1103, 1106, 1109, 1112, 1115, 1118, 1121, 1124, 1127, 1130, 1574, 1575, 1611, 0,
    894, 878, 875, 883, 885, 1751, 881, 888, 0, 0, 0, 0, 0, 0, 0, 0,
    811, 731, 846, 1755, 1744, 1731, 0, 0, 743, 802, 0, 0, 0, 0, 0, 0,
    820, 729, 871, 772, 853, 856, 761, 764, 767, 867, 733, 769, 891, 799, 723, 2340,
    720, 869, 736, 796, 739, 825, 1621, 781, 1737, 1756, 1734, 828, 2346, 2355, 2352, 2337,
];
/// `LiTo/LTtext` LTSpellDesc-c.tbl: declared 176, 147 keyed rows
/// (dword_647E20 / dword_647E24). An index is used only when its row exists.
pub const LTSPELLDESC_DECLARED: u16 = 176;
pub const LTSPELLDESC_ROWS: &[std::ops::RangeInclusive<u16>] =
    &[0..=22, 24..=79, 86..=90, 96..=110, 112..=119, 128..=133, 136..=137, 144..=175];
/// 004AEEEC: absent/out-of-range rows push icon 0x5B with "unknown spell".
pub const UNKNOWN_SPELL_ICON: u16 = 91;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpellIconRoute {
    Lito(u16),
    UnknownSpell,
}
impl SpellIconRoute {
    pub fn graphic(self) -> u16 {
        match self {
            Self::Lito(g) => g,
            Self::UnknownSpell => UNKNOWN_SPELL_ICON,
        }
    }
}
/// Pure mirror of the original S1E icon choice; never consults a buff table.
pub fn spell_icon_route(skill: u16) -> Result<SpellIconRoute> {
    let index = skill.checked_sub(1).context("技能 ID 0 無效")?;
    if index < LTSPELLDESC_DECLARED && LTSPELLDESC_ROWS.iter().any(|r| r.contains(&index)) {
        let graphic = LITO_SPELL_ICONS[usize::from(index)];
        ensure!(graphic != 0, "原版技能圖示表為 0");
        Ok(SpellIconRoute::Lito(graphic))
    } else {
        Ok(SpellIconRoute::UnknownSpell)
    }
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
    /// Spell IDs and graphic IDs are different namespaces: the LiTo S1E table
    /// (or the original unknown-spell icon 91) selects a shared `%d.tbt`.
    pub fn load_spell(&mut self, skill: u16, cancel: &AtomicBool) -> Result<Icon> {
        self.load(spell_icon_route(skill)?.graphic(), cancel)
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
    fn start(root: PathBuf, mut reader: Option<Reader>, ids: Vec<u16>, spells: bool) -> Result<Self> {
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
                        let reader = reader.as_mut().unwrap();
                        (if spells { reader.load_spell(id, &token) } else { reader.load(id, &token) })
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
    spells: bool,
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
            spells: false,
        }
    }
    pub fn spells(root: PathBuf) -> Self {
        let mut service = Self::new(root);
        service.spells = true;
        service
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
                match Job::start(self.root.clone(), self.reader.take(), missing.clone(), self.spells) {
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

#[cfg(test)]
mod skill_ui_tests {
    use super::*;
    #[test]
    /// Exact 181-entry coverage: every catalog skill resolves, the unknown-spell
    /// set is pinned, and the table entries that differ from shared
    /// SpellIcon.idx stay on the LiTo value.
    fn catalog_icon_route_is_complete_and_exact() {
        use crate::modern_lito::native_skill_selection::catalog::CATALOG;
        const UNKNOWN: &[u16] = &[
            112, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195,
            201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216,
            217, 218, 219, 220,
        ];
        assert_eq!(CATALOG.len(), 181);
        let rows: usize = LTSPELLDESC_ROWS.iter().map(|r| r.clone().count()).sum();
        assert_eq!(rows, 147);
        let mut unknown = Vec::new();
        for &(id, _) in CATALOG {
            match spell_icon_route(id).unwrap() {
                SpellIconRoute::Lito(g) => assert!(g != 0 && g < 20000, "skill {id}"),
                SpellIconRoute::UnknownSpell => unknown.push(id),
            }
        }
        assert_eq!(unknown, UNKNOWN);
        assert_eq!(CATALOG.len() - unknown.len(), 145);
        for (id, g) in [
            (1, 383), (2, 384), (3, 385), (4, 386), (58, 419), (63, 784), (79, 1610),
            (87, 1624), (88, 1887), (89, 1881), (90, 2349), (91, 2343), (111, 1611),
            (113, 894), (118, 1751), (132, 1755), (133, 1744), (134, 1731), (160, 2340),
            (167, 1621), (169, 1737), (170, 1756), (171, 1734), (172, 828), (173, 2346),
            (174, 2355), (175, 2352), (176, 2337),
        ] {
            assert_eq!(spell_icon_route(id).unwrap(), SpellIconRoute::Lito(g), "skill {id}");
        }
        assert_eq!(spell_icon_route(206).unwrap().graphic(), UNKNOWN_SPELL_ICON);
        // Rows absent from LTSpellDesc and indices past its declared count.
        assert_eq!(spell_icon_route(120).unwrap(), SpellIconRoute::Lito(888));
        for id in [24, 81, 92, 121, 135, 177, 221, u16::MAX] {
            assert_eq!(spell_icon_route(id).unwrap(), SpellIconRoute::UnknownSpell, "skill {id}");
        }
        assert!(spell_icon_route(0).is_err());
    }
    #[test]
    fn skill_cache_is_keyed_by_skill_and_shutdown_retires_assets() {
        let mut service = Service::spells(PathBuf::from("unused"));
        service.wanted.insert(1);
        let icon = Icon::decode(383, "test", "383.tbt", &[0, 0, 1, 1, 1, 0, 1, 0, 0], &AtomicBool::new(false)).unwrap();
        service.cache.insert(1, Ok(icon));
        assert_eq!(service.icon(1).unwrap().id, 383);
        assert!(service.icon(383).is_none());
        service.cancel();
        assert!(service.icon(1).is_none());
        assert!(service.quiescent());
        service.tick(&[1]);
        assert_eq!(service.cached(), 0);
    }
    #[test]
    #[ignore = "explicit read-only LITO_CLIENT_ROOT asset verification; no GUI or service"]
    fn real_skill_icons_and_async_service_preserve_mapping_and_missing_assets() {
        let root = PathBuf::from(std::env::var_os("LITO_CLIENT_ROOT").expect("LITO_CLIENT_ROOT"));
        let cancel = AtomicBool::new(false);
        // The embedded row set must equal the actual LTtext LTSpellDesc-c.tbl.
        let lt = root.join("LiTo");
        let pin = |n: &str| super::super::native_world_scene::read_pin(&lt.join(n), 1024 * 1024, &cancel).unwrap();
        let table = super::super::native_resources::read_native_text_optional(
            &lt, pin("LTtext.idx"), pin("LTtext.pak"), "ltspelldesc-c.tbl", &cancel,
        ).unwrap().expect("LTSpellDesc-c.tbl");
        let mut lines = table.split(|&b| b == b'\n').map(|l| l.strip_suffix(b"\r").unwrap_or(l));
        let declared: u16 = std::str::from_utf8(lines.next().unwrap()).unwrap().trim().parse().unwrap();
        assert_eq!(declared, LTSPELLDESC_DECLARED);
        let actual: BTreeSet<u16> = lines.filter_map(|l| {
            let n = l.iter().take_while(|b| b.is_ascii_digit()).count();
            (n > 0 && l.get(n) == Some(&b' ')).then(|| std::str::from_utf8(&l[..n]).unwrap().parse().unwrap())
        }).collect();
        let embedded: BTreeSet<u16> = LTSPELLDESC_ROWS.iter().flat_map(|r| r.clone()).collect();
        assert_eq!(actual, embedded);
        // Every catalog skill must decode its routed original graphic.
        let mut reader = Reader::open(&root, &cancel).unwrap();
        let mut unknown = Vec::new();
        for &(id, _) in crate::modern_lito::native_skill_selection::catalog::CATALOG {
            let route = spell_icon_route(id).unwrap();
            let icon = reader.load_spell(id, &cancel)
                .unwrap_or_else(|e| panic!("skill {id} graphic {}: {e:#}", route.graphic()));
            assert_eq!(icon.id, route.graphic());
            assert!(icon.covered_pixels > 0, "skill {id}");
            if route == SpellIconRoute::UnknownSpell { unknown.push(id); }
            println!("skill={id} graphic={} source={}/{} sha256={}", icon.id, icon.source.archive, icon.source.member, icon.source.sha256);
        }
        assert_eq!(unknown.len(), 36);
        let mut service = Service::spells(root);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            service.tick(&[1, 3, 206]);
            if service.quiescent() { break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(service.icon(1).is_some() && service.icon(3).is_some());
        assert_eq!(service.icon(206).unwrap().id, UNKNOWN_SPELL_ICON);
        assert!(service.error(206).is_none());
        service.tick(&[]);
        assert_eq!(service.cached(), 0);
        service.cancel();
        assert!(service.quiescent());
    }
}
