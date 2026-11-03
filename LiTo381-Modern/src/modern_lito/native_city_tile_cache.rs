//! Private per-city RGBA residence. All necessary slots were decoded successfully
//! before this storage is constructed. Immutable complete member bytes remain
//! bounded separately; eviction changes residence, never source qualification.
use super::*;
use std::cell::RefCell;
use std::sync::Arc;

pub const DEFAULT_CACHE_BYTES: usize = 16 << 20;
const TILE_BYTES: usize = 24 * 24 * 4;
type Key = (u16, u8);

#[derive(Default)]
pub struct CacheProgress {
    pub phase: AtomicUsize,
    pub decoded_slots: AtomicUsize,
    pub evicted_slots: AtomicUsize,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct CacheMetrics {
    pub capacity_bytes: usize,
    pub resident_decoded_bytes: usize,
    pub peak_resident_decoded_bytes: usize,
    pub resident_slots: usize,
    pub qualified_slots: usize,
    pub whole_required_decoded_bytes: usize,
    pub encoded_resident_bytes: usize,
    pub validation_peak_decoded_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub completed_frames: u64,
    pub maximum_frame_keys: usize,
    pub maximum_frame_decoded_bytes: usize,
}

pub(super) struct EncodedMember {
    pub raw: Vec<u8>,
    pub qualified: BTreeSet<usize>,
}
struct Entry {
    tile: DecodedTile,
    stamp: u64,
}
pub(super) struct PagedTiles {
    members: BTreeMap<u16, EncodedMember>,
    resident: BTreeMap<Key, Entry>,
    order: BTreeSet<(u64, Key)>,
    stamp: u64,
    metrics: CacheMetrics,
    progress: Arc<CacheProgress>,
}
pub(super) enum TileStorage {
    Eager(BTreeMap<Key, DecodedTile>),
    Paged(RefCell<PagedTiles>),
}

impl TileStorage {
    pub fn paged(members: BTreeMap<u16, EncodedMember>, capacity: usize, qualified: usize, encoded_bytes: usize) -> Result<Self> {
        ensure!((TILE_BYTES..=DEFAULT_CACHE_BYTES).contains(&capacity), "city cache residence limit");
        Ok(Self::Paged(RefCell::new(PagedTiles {
            members, resident: BTreeMap::new(), order: BTreeSet::new(), stamp: 0,
            metrics: CacheMetrics {
                capacity_bytes: capacity, resident_decoded_bytes: 0, peak_resident_decoded_bytes: 0,
                resident_slots: 0, qualified_slots: qualified, whole_required_decoded_bytes: qualified.checked_mul(TILE_BYTES).context("city qualified pixel overflow")?,
                encoded_resident_bytes: encoded_bytes, validation_peak_decoded_bytes: TILE_BYTES,
                hits: 0, misses: 0, evictions: 0, completed_frames: 0, maximum_frame_keys: 0, maximum_frame_decoded_bytes: 0,
            },
            progress: Arc::new(CacheProgress::default()),
        })))
    }
    pub fn metrics(&self) -> Option<CacheMetrics> {
        match self { Self::Eager(_) => None, Self::Paged(c) => Some(c.borrow().metrics) }
    }
    pub fn progress(&self) -> Option<Arc<CacheProgress>> {
        match self { Self::Eager(_) => None, Self::Paged(c) => Some(c.borrow().progress.clone()) }
    }
    pub fn draw(&self, fragments: &[Fragment], canvas: &mut Canvas, cancel: &AtomicBool) -> Result<()> {
        match self {
            Self::Eager(tiles) => {
                for f in fragments {
                    active(cancel)?;
                    let t = tiles.get(&f.tile).context("city required pixel coverage absent")?;
                    canvas.blit_rgba(&t.rgba, t.width, t.height, f.point.x, f.point.y)?;
                }
            }
            Self::Paged(c) => {
                let mut c = c.try_borrow_mut().context("city cache render already borrowed")?;
                active(cancel)?;
                let completed_frames = c.metrics.completed_frames.checked_add(1).context("city cache frame counter overflow")?;
                let needed: BTreeSet<_> = fragments.iter().map(|f| f.tile).collect();
                c.prepare(&needed, cancel)?;
                c.progress.phase.store(3, Ordering::Release);
                for f in fragments {
                    active(cancel)?;
                    let t = &c.resident.get(&f.tile).context("city visible qualified tile absent")?.tile;
                    canvas.blit_rgba(&t.rgba, t.width, t.height, f.point.x, f.point.y)?;
                }
                active(cancel)?;
                c.metrics.completed_frames = completed_frames;
                c.progress.phase.store(4, Ordering::Release);
            }
        }
        Ok(())
    }
}

impl PagedTiles {
    fn prepare(&mut self, needed: &BTreeSet<Key>, cancel: &AtomicBool) -> Result<()> {
        active(cancel)?;
        self.progress.phase.store(1, Ordering::Release);
        let working_bytes = needed.len().checked_mul(TILE_BYTES).context("city cache visible size overflow")?;
        ensure!(working_bytes <= self.metrics.capacity_bytes, "city cache visible pixel budget");
        // Check the entire frame's eligibility and arithmetic before eviction.
        for (id, slot) in needed {
            ensure!(self.members.get(id).is_some_and(|m| m.qualified.contains(&usize::from(*slot))), "city cache unqualified slot request");
        }
        let missing: Vec<_> = needed.iter().filter(|k| !self.resident.contains_key(k)).copied().collect();
        let next_stamp = self.stamp.checked_add((needed.len() + missing.len()) as u64).context("city cache stamp overflow")?;
        let max_slots = self.metrics.capacity_bytes / TILE_BYTES;
        let remove = (self.resident.len() + missing.len()).saturating_sub(max_slots);
        let victims: Vec<_> = self.order.iter().filter(|(_, k)| !needed.contains(k)).take(remove).copied().collect();
        ensure!(victims.len() == remove, "city cache protected frame residence exhausted");
        self.metrics.evictions.checked_add(remove as u64).context("city cache eviction counter overflow")?;
        let hits = self.metrics.hits.checked_add((needed.len() - missing.len()) as u64).context("city cache hit counter overflow")?;
        self.metrics.misses.checked_add(missing.len() as u64).context("city cache miss counter overflow")?;
        self.metrics.maximum_frame_keys = self.metrics.maximum_frame_keys.max(needed.len());
        self.metrics.maximum_frame_decoded_bytes = self.metrics.maximum_frame_decoded_bytes.max(working_bytes);
        // Evict all necessary victims first. A newly decoded temporary tile plus
        // residence must already fit; insertion never momentarily exceeds the cap.
        for (stamp, key) in victims {
            active(cancel)?;
            self.order.remove(&(stamp, key));
            self.resident.remove(&key).context("city cache LRU entry missing")?;
            self.metrics.resident_slots -= 1;
            self.metrics.resident_decoded_bytes -= TILE_BYTES;
            self.metrics.evictions += 1;
            self.progress.evicted_slots.fetch_add(1, Ordering::Release);
        }
        self.metrics.hits = hits;
        self.progress.phase.store(2, Ordering::Release);
        for (id, slot) in missing {
            active(cancel)?;
            ensure!(self.metrics.resident_decoded_bytes + TILE_BYTES <= self.metrics.capacity_bytes, "city cache transient pixel budget");
            let raw = &self.members.get(&id).context("city cache member absent")?.raw;
            let input = ValidatedTileInput::new_with_empty_tail(raw, TileLimits { max_input_bytes: raw.len(), max_total_pixels: 24 * 24, ..Default::default() })?;
            let tile = input.decode_selected(&BTreeSet::from([usize::from(slot)]), false)?.remove(&usize::from(slot)).context("city cache qualified slot absent")?;
            ensure!(tile.rgba.len() == TILE_BYTES && tile.rgba.capacity() == TILE_BYTES, "city cache tile allocation shape");
            active(cancel)?;
            self.stamp += 1;
            self.resident.insert((id, slot), Entry { tile, stamp: self.stamp });
            self.order.insert((self.stamp, (id, slot)));
            // Publish LRU membership immediately, so cancellation during a batch
            // leaves every retained entry eligible for later protected eviction.
            self.metrics.resident_slots += 1;
            self.metrics.resident_decoded_bytes += TILE_BYTES;
            self.metrics.peak_resident_decoded_bytes = self.metrics.peak_resident_decoded_bytes.max(self.metrics.resident_decoded_bytes);
            self.metrics.misses = self.metrics.misses.checked_add(1).context("city cache miss counter overflow")?;
            self.progress.decoded_slots.fetch_add(1, Ordering::Release);
        }
        for key in needed {
            let entry = self.resident.get_mut(key).context("city cache prepared entry absent")?;
            if entry.stamp != 0 { self.order.remove(&(entry.stamp, *key)); }
            self.stamp += 1;
            entry.stamp = self.stamp;
            self.order.insert((self.stamp, *key));
        }
        debug_assert_eq!(self.stamp, next_stamp);
        active(cancel)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "native_city_tile_cache_failed_frame_tests.rs"]
mod failed_frame_tests;
