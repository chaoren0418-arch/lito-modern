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
    /// Per cell pixel: how many of the two native halo passes cover it (0..=2).
    #[serde(skip)]
    pub halo: Vec<u8>,
}
/// Native 442A30 draw variant, chosen from the item's identified byte (+0x78)
/// and blessing (+0x70). Recovered PE 4271D92F…5C2AB6E9, static decode only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum IconStyle {
    /// Unidentified, blessing 1 or 3 (identity palettes 835B3C / 83633C).
    /// Also used for a blessing above 3, where the original indexes past its
    /// palette array.
    Plain,
    /// Identified, blessing 0: two halo passes, then the unchanged icon.
    Blessed,
    /// Identified, blessing 2: every icon pixel through the curse tint.
    Cursed,
}
impl IconStyle {
    pub fn of(identified: u8, blessing: u8) -> Self {
        match (identified != 0, blessing) {
            (true, 0) => Self::Blessed,
            (true, 2) => Self::Cursed,
            _ => Self::Plain,
        }
    }
}
/// 4B9BF0 stamps the icon's coverage into a 36x36 byte mask at these offsets,
/// one group per pass; 4B9E80 then blends the 32x32 window that starts at
/// (2,2) of the mask. Net effect: the shape moved by two pixels, then by one,
/// in the four axis directions.
const HALO_STAMPS: [[[i32; 2]; 4]; 2] = [
    [[0, 2], [2, 0], [4, 2], [2, 4]],
    [[1, 2], [2, 1], [3, 2], [2, 3]],
];
const HALO_MASK: i32 = 36;
/// Palette 83533C (4EC1C0 / 4EC1F0 through 41A2B0), RGB555 mode: each
/// destination channel moves a third of the way to white. Green is built
/// from its low three and high two bits separately, as the original's
/// per-byte tables do.
pub fn halo_blend([r, g, b]: [u16; 3]) -> [u16; 3] {
    [
        (2 * r + 31) / 3,
        (2 * (g & 7) + 7) / 3 + (2 * (g & 0x18) + 24) / 3,
        (2 * b + 31) / 3,
    ]
}
/// 4B9A90 → 4C1170 with colour `dword_50352C` (RGB555 1C00 = red 7): the
/// blend table 7A789C holds `colour + pixel / 2` per five-bit channel.
pub fn curse_tint([r, g, b]: [u16; 3]) -> [u16; 3] {
    [7 + r / 2, g / 2, b / 2]
}
fn expand5(v: u16) -> u8 {
    ((v << 3) | (v >> 2)) as u8
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
        let mut masks = [[false; (HALO_MASK * HALO_MASK) as usize]; 2];
        for (i, &v) in image.rgb555().iter().enumerate() {
            if !image.coverage()[i] {
                continue;
            }
            covered_pixels += 1;
            covered_zero += usize::from(v == 0);
            let x = (i % image.width()) as i32 + i32::from(offset[0]);
            let y = (i / image.width()) as i32 + i32::from(offset[1]);
            for (pass, stamps) in HALO_STAMPS.iter().enumerate() {
                for [dx, dy] in stamps {
                    // The original has no bounds check here; a stamp outside
                    // its mask is dropped instead of reproducing the overrun.
                    let (mx, my) = (x + dx, y + dy);
                    if (0..HALO_MASK).contains(&mx) && (0..HALO_MASK).contains(&my) {
                        masks[pass][(my * HALO_MASK + mx) as usize] = true;
                    }
                }
            }
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
        let halo = (0..CELL * CELL)
            .map(|i| {
                let at = (i / CELL + 2) * HALO_MASK as usize + i % CELL + 2;
                u8::from(masks[0][at]) + u8::from(masks[1][at])
            })
            .collect();
        Ok(Self {
            halo,
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
        self.bgra_styled(background, IconStyle::Plain)
    }
    /// The cell over `background` in the native draw order: for a blessed
    /// item the halo passes lighten the background first, then the icon's
    /// covered pixels are copied (cursed: through the tint). The original
    /// blends its RGB555 surface; here the background is reduced to five
    /// bits per channel only where a halo pixel is produced.
    pub fn bgra_styled(&self, background: [u8; 3], style: IconStyle) -> Vec<u8> {
        self.cell
            .chunks_exact(4)
            .zip(&self.halo)
            .flat_map(|(p, &passes)| {
                let c = if p[3] != 0 {
                    let rgb = [p[0], p[1], p[2]];
                    if style == IconStyle::Cursed {
                        curse_tint(rgb.map(|v| u16::from(v >> 3))).map(expand5)
                    } else {
                        rgb
                    }
                } else if style == IconStyle::Blessed && passes > 0 {
                    let mut v = background.map(|v| u16::from(v >> 3));
                    for _ in 0..passes {
                        v = halo_blend(v);
                    }
                    v.map(expand5)
                } else {
                    background
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

#[cfg(test)]
mod style_tests {
    use super::*;
    fn icon(bytes: &[u8]) -> Icon {
        Icon::decode(1, "test", "1.tbt", bytes, &AtomicBool::new(false)).unwrap()
    }
    fn pixel(bgra: &[u8], x: usize, y: usize) -> [u8; 4] {
        let at = (y * CELL + x) * 4;
        bgra[at..at + 4].try_into().unwrap()
    }
    #[test]
    fn style_follows_identified_then_blessing() {
        assert_eq!(IconStyle::of(1, 0), IconStyle::Blessed);
        assert_eq!(IconStyle::of(1, 1), IconStyle::Plain);
        assert_eq!(IconStyle::of(1, 2), IconStyle::Cursed);
        assert_eq!(IconStyle::of(1, 3), IconStyle::Plain);
        assert_eq!(IconStyle::of(255, 2), IconStyle::Cursed);
        for blessing in [0, 1, 2, 3, 128, 130] {
            assert_eq!(IconStyle::of(0, blessing), IconStyle::Plain);
        }
        assert_eq!(IconStyle::of(1, 128), IconStyle::Plain);
        assert_eq!(IconStyle::of(1, 130), IconStyle::Plain);
    }
    #[test]
    fn channel_formulas_match_the_native_tables() {
        assert_eq!(halo_blend([0, 0, 0]), [10, 10, 10]);
        assert_eq!(halo_blend([10, 10, 10]), [17, 16, 17]);
        assert_eq!(halo_blend([31, 31, 31]), [31, 31, 31]);
        assert_eq!(halo_blend([1, 9, 30]), [11, 16, 30]);
        for v in 0..32 {
            let out = halo_blend([v, v, v]);
            assert!(out.iter().all(|c| (v..=31).contains(c)), "{v}");
        }
        assert_eq!(curse_tint([0, 0, 0]), [7, 0, 0]);
        assert_eq!(curse_tint([31, 31, 31]), [22, 15, 15]);
        assert_eq!(curse_tint([9, 8, 7]), [11, 4, 3]);
    }
    #[test]
    fn blessed_halo_is_the_shape_moved_two_then_one_pixel() {
        // One white pixel at cell (4,4).
        let i = icon(&[4, 4, 1, 1, 1, 0, 1, 0xff, 0x7f]);
        let lit: Vec<(usize, usize, u8)> = (0..CELL * CELL)
            .filter(|n| i.halo[*n] != 0)
            .map(|n| (n % CELL, n / CELL, i.halo[n]))
            .collect();
        assert_eq!(
            lit,
            [(4, 2, 1), (4, 3, 1), (2, 4, 1), (3, 4, 1), (5, 4, 1), (6, 4, 1), (4, 5, 1), (4, 6, 1)]
        );
        let plain = i.bgra_styled([0, 0, 0], IconStyle::Plain);
        assert_eq!(plain, i.bgra([0, 0, 0]));
        assert_eq!(pixel(&plain, 3, 4), [0, 0, 0, 255]);
        let blessed = i.bgra_styled([0, 0, 0], IconStyle::Blessed);
        assert_eq!(pixel(&blessed, 4, 4), [255, 255, 255, 255]);
        assert_eq!(pixel(&blessed, 3, 4), [82, 82, 82, 255]);
        assert_eq!(pixel(&blessed, 2, 4), [82, 82, 82, 255]);
        assert_eq!(pixel(&blessed, 3, 3), [0, 0, 0, 255]);
        assert_eq!(pixel(&blessed, 1, 4), [0, 0, 0, 255]);
        // Untouched background keeps its full eight-bit value.
        let tinted = i.bgra_styled([1, 2, 3], IconStyle::Blessed);
        assert_eq!(pixel(&tinted, 0, 0), [3, 2, 1, 255]);
    }
    #[test]
    fn overlapping_passes_blend_twice_and_cursed_tints_only_the_icon() {
        // Two white pixels at cells (4,4) and (5,4).
        let i = icon(&[4, 4, 2, 1, 1, 0, 2, 0xff, 0x7f, 0xff, 0x7f]);
        assert_eq!(i.halo[4 * CELL + 3], 2);
        assert_eq!(i.halo[4 * CELL + 6], 2);
        assert_eq!(i.halo[4 * CELL + 2], 1);
        assert_eq!(i.halo[4 * CELL + 7], 1);
        // Covered cells are in the mask too; the icon is drawn over them.
        assert_eq!(i.halo[4 * CELL + 4], 1);
        let blessed = i.bgra_styled([0, 0, 0], IconStyle::Blessed);
        assert_eq!(pixel(&blessed, 3, 4), [140, 132, 140, 255]);
        assert_eq!(pixel(&blessed, 2, 4), [82, 82, 82, 255]);
        assert_eq!(pixel(&blessed, 4, 4), [255, 255, 255, 255]);
        let cursed = i.bgra_styled([9, 9, 9], IconStyle::Cursed);
        assert_eq!(pixel(&cursed, 4, 4), [123, 123, 181, 255]);
        assert_eq!(pixel(&cursed, 5, 4), [123, 123, 181, 255]);
        assert_eq!(pixel(&cursed, 3, 4), [9, 9, 9, 255]);
    }
    #[test]
    fn halo_near_the_cell_edge_is_clipped_not_wrapped() {
        // One pixel at cell (0,0): the left and upper stamps leave the cell.
        let i = icon(&[0, 0, 1, 1, 1, 0, 1, 0xff, 0x7f]);
        let lit: Vec<usize> = (0..CELL * CELL).filter(|n| i.halo[*n] != 0).collect();
        assert_eq!(lit, [1, 2, CELL, 2 * CELL]);
        // One pixel at cell (-1,0), outside the cell, still lights (0,0) and (1,0).
        let i = icon(&[255, 0, 1, 1, 1, 0, 1, 0xff, 0x7f]);
        assert_eq!(i.covered_pixels, 1);
        assert!(i.cell.iter().all(|v| *v == 0));
        assert_eq!((i.halo[0], i.halo[1]), (1, 1));
    }
}