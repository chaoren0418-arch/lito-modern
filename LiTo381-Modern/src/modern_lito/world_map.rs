//! Immutable shared-S32 world geometry. No LT map lookup, guessed teleport,
//! creature occupancy, projectile rule, combat, or world-start permission.
//!
//! Modern version 1 deliberately uses at least one *complete two-cardinal-edge*
//! route for a diagonal. This is stricter than retained TW NavGrid's destination
//! only movement, which is not changed. Portals and missing sectors are closed.
use super::map::{parse_s32, CellAttributes, MapError, S32Sector};
use super::resources::{
    read_shared_map_sector_with_limit, ResourceError, SharedMapSectorKey,
    MAX_SHARED_MAP_SECTOR_BYTES,
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

/// Physical geometry budget; negotiated world admission has a separate legacy limit.
pub const MAX_SECTORS: usize = 64;
pub const MAX_INPUT_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_SPAWNS: usize = 100;
pub const MANIFEST_VERSION: u16 = 1;
const SIDE: i32 = 64;
const CELLS: usize = 4096;
const PREFIX: &[u8] = b"LITO-WORLD-MAP\0";
/// Heading bit order: N, NE, E, SE, S, SW, W, NW. These are game-cell axes,
/// not screen axes; +X projects right/up and +Y projects right/down.
pub const DIRECTIONS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldCell {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelPoint {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectorBinding {
    pub key: SharedMapSectorKey,
    pub byte_length: u32,
    pub sha256: [u8; 32],
}
pub struct SectorInput {
    pub key: SharedMapSectorKey,
    pub bytes: Vec<u8>,
}
/// Nonzero entries remain explicit presentation work; geometry does not silently
/// claim that static tiles implement actor-dependent roofs or animated effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtLimitations {
    pub opacity_records: usize,
    pub sprite_effects: usize,
    pub portals: usize,
    pub negative_object_tiles: usize,
}
impl ArtLimitations {
    /// False only means these known layer gaps are absent, not that Tile assets
    /// have been loaded successfully or that the world renderer is complete.
    pub fn has_known_presentation_gaps(self) -> bool {
        self.opacity_records != 0
            || self.sprite_effects != 0
            || self.portals != 0
            || self.negative_object_tiles != 0
    }
}
#[derive(Debug)]
pub enum WorldMapError {
    InvalidManifest(&'static str),
    Resource(ResourceError),
    Decode(MapError),
    HashMismatch(SharedMapSectorKey),
    UnsupportedTail(SharedMapSectorKey),
    Cancelled,
    InvalidSpawns(&'static str),
}
impl fmt::Display for WorldMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidManifest(reason) => write!(f, "invalid shared world manifest: {reason}"),
            Self::Resource(error) => write!(f, "shared world resource: {error}"),
            Self::Decode(error) => write!(f, "shared world S32: {error}"),
            Self::HashMismatch(key) => write!(
                f,
                "shared world sector length/hash mismatch: map {} {}",
                key.map_id,
                key.file_name()
            ),
            Self::UnsupportedTail(key) => write!(
                f,
                "unsupported shared S32 trailing bytes: map {} {}",
                key.map_id,
                key.file_name()
            ),
            Self::Cancelled => f.write_str("shared world load cancelled"),
            Self::InvalidSpawns(reason) => write!(f, "invalid world spawns: {reason}"),
        }
    }
}
impl std::error::Error for WorldMapError {}
impl From<ResourceError> for WorldMapError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}
impl From<MapError> for WorldMapError {
    fn from(value: MapError) -> Self {
        Self::Decode(value)
    }
}
type Result<T> = std::result::Result<T, WorldMapError>;
fn check(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(WorldMapError::Cancelled)
    } else {
        Ok(())
    }
}
fn invalid(reason: &'static str) -> WorldMapError {
    WorldMapError::InvalidManifest(reason)
}
struct Sector {
    binding: SectorBinding,
    data: S32Sector,
    portals: BTreeSet<usize>,
    directions: Box<[u8; CELLS]>,
}
pub struct WorldMap {
    map_id: u32,
    sectors: BTreeMap<(u16, u16), Sector>,
    canonical: Vec<u8>,
    digest: [u8; 32],
}

impl WorldMap {
    /// Hash and parse the same owned input generation. Caller bytes are consumed,
    /// never re-opened by path after validation; sort order is canonical.
    pub fn from_sector_bytes(inputs: Vec<SectorInput>, cancel: &AtomicBool) -> Result<Self> {
        Self::from_sector_bytes_format(inputs, cancel, false)
    }

    /// Native-only portal suffix parsing; original owned input hashes and all
    /// geometry/canonical/budget validation remain identical to Shared loading.
    pub fn from_native_sector_bytes(inputs: Vec<SectorInput>, cancel: &AtomicBool) -> Result<Self> {
        Self::from_sector_bytes_format(inputs, cancel, true)
    }

    fn from_sector_bytes_format(inputs: Vec<SectorInput>, cancel: &AtomicBool, native: bool) -> Result<Self> {
        check(cancel)?;
        if inputs.is_empty() || inputs.len() > MAX_SECTORS {
            return Err(invalid("sector count"));
        }
        let map_id = inputs[0].key.map_id;
        if map_id > i32::MAX as u32 {
            return Err(invalid("map identity"));
        }
        let mut total = 0usize;
        let mut keys = BTreeSet::new();
        for input in &inputs {
            total = total
                .checked_add(input.bytes.len())
                .ok_or_else(|| invalid("input overflow"))?;
            if input.key.map_id != map_id
                || input.bytes.is_empty()
                || input.bytes.len() > MAX_SHARED_MAP_SECTOR_BYTES as usize
                || total > MAX_INPUT_BYTES
                || !keys.insert((input.key.sector_x, input.key.sector_y))
            {
                return Err(invalid("input identity/size/duplicate"));
            }
        }
        let mut sectors = BTreeMap::new();
        for input in inputs {
            check(cancel)?;
            let binding = SectorBinding {
                key: input.key,
                byte_length: input.bytes.len() as u32,
                sha256: Sha256::digest(&input.bytes).into(),
            };
            let data = if native { super::map::parse_native_s32(&input.bytes)? } else { parse_s32(&input.bytes)? };
            if data.has_unknown_tail() {
                return Err(WorldMapError::UnsupportedTail(input.key));
            }
            let mut portals = BTreeSet::new();
            for portal in &data.portals {
                if portal.x >= 128 || portal.y >= 64 {
                    return Err(invalid("portal local coordinate"));
                }
                portals.insert(usize::from(portal.y) * 64 + usize::from(portal.x) / 2);
            }
            sectors.insert(
                (input.key.sector_x, input.key.sector_y),
                Sector {
                    binding,
                    data,
                    portals,
                    directions: Box::new([0; CELLS]),
                },
            );
        }
        let mut map = Self {
            map_id,
            sectors,
            canonical: Vec::new(),
            digest: [0; 32],
        };
        let mut masks = Vec::with_capacity(map.sectors.len());
        for (&key, sector) in &map.sectors {
            let mut values = Box::new([0u8; CELLS]);
            let (ox, oy) = sector.binding.key.world_origin();
            for y in 0..SIDE {
                check(cancel)?;
                for x in 0..SIDE {
                    let cell = WorldCell {
                        x: ox + x,
                        y: oy + y,
                    };
                    let mut mask = 0;
                    for (heading, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
                        if map.raw_step(
                            cell,
                            WorldCell {
                                x: cell.x + dx,
                                y: cell.y + dy,
                            },
                        ) {
                            mask |= 1 << heading;
                        }
                    }
                    values[(y * SIDE + x) as usize] = mask;
                }
            }
            masks.push((key, values));
        }
        for (key, values) in masks {
            map.sectors.get_mut(&key).unwrap().directions = values;
        }
        map.canonical.extend_from_slice(PREFIX);
        map.canonical
            .extend_from_slice(&MANIFEST_VERSION.to_be_bytes());
        map.canonical.extend_from_slice(&map_id.to_be_bytes());
        map.canonical
            .extend_from_slice(&(map.sectors.len() as u16).to_be_bytes());
        for sector in map.sectors.values() {
            let binding = sector.binding;
            let (ox, oy) = binding.key.world_origin();
            map.canonical
                .extend_from_slice(&binding.key.sector_x.to_be_bytes());
            map.canonical
                .extend_from_slice(&binding.key.sector_y.to_be_bytes());
            map.canonical.extend_from_slice(&ox.to_be_bytes());
            map.canonical.extend_from_slice(&oy.to_be_bytes());
            map.canonical.extend_from_slice(&64u16.to_be_bytes());
            map.canonical.extend_from_slice(&64u16.to_be_bytes());
            map.canonical
                .extend_from_slice(&binding.byte_length.to_be_bytes());
            map.canonical.extend_from_slice(&binding.sha256);
            map.canonical.extend_from_slice(sector.directions.as_ref());
        }
        check(cancel)?;
        map.digest = Sha256::digest(&map.canonical).into();
        Ok(map)
    }
    /// Explicit manifest only. No directory discovery, neighboring-sector guess,
    /// map-ID alias, LT fallback or alternate filename is permitted.
    pub fn load_shared(
        root: &Path,
        expected: &[SectorBinding],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        check(cancel)?;
        if expected.is_empty() || expected.len() > MAX_SECTORS {
            return Err(invalid("sector count"));
        }
        let mut total = 0usize;
        let mut keys = BTreeSet::new();
        let map = expected[0].key.map_id;
        for binding in expected {
            total = total
                .checked_add(binding.byte_length as usize)
                .ok_or_else(|| invalid("input overflow"))?;
            if map > i32::MAX as u32
                || binding.key.map_id != map
                || binding.byte_length == 0
                || u64::from(binding.byte_length) > MAX_SHARED_MAP_SECTOR_BYTES
                || total > MAX_INPUT_BYTES
                || !keys.insert((binding.key.sector_x, binding.key.sector_y))
            {
                return Err(invalid("manifest identity/size/duplicate"));
            }
        }
        let mut inputs = Vec::with_capacity(expected.len());
        let mut remaining = MAX_INPUT_BYTES;
        for binding in expected {
            check(cancel)?;
            let bytes = read_shared_map_sector_with_limit(
                root,
                binding.key,
                u64::from(binding.byte_length).min(remaining as u64),
            )?;
            if bytes.len() != binding.byte_length as usize
                || <[u8; 32]>::from(Sha256::digest(&bytes)) != binding.sha256
            {
                return Err(WorldMapError::HashMismatch(binding.key));
            }
            remaining -= bytes.len();
            inputs.push(SectorInput {
                key: binding.key,
                bytes,
            });
        }
        Self::from_sector_bytes(inputs, cancel)
    }
    pub fn map_id(&self) -> u32 {
        self.map_id
    }
    pub fn bindings(&self) -> Vec<SectorBinding> {
        self.sectors.values().map(|sector| sector.binding).collect()
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn canonical_digest(&self) -> [u8; 32] {
        self.digest
    }
    pub fn sector(&self, key: SharedMapSectorKey) -> Option<&S32Sector> {
        if key.map_id != self.map_id {
            return None;
        }
        self.sectors
            .get(&(key.sector_x, key.sector_y))
            .map(|sector| &sector.data)
    }
    pub fn art_limitations(&self, key: SharedMapSectorKey) -> Option<ArtLimitations> {
        let sector = self.sector(key)?;
        Some(ArtLimitations {
            opacity_records: sector.opacity.len(),
            sprite_effects: sector.effects.len(),
            portals: sector.portals.len(),
            negative_object_tiles: sector
                .objects
                .iter()
                .flat_map(|group| &group.tiles)
                .filter(|tile| tile.tile_id < 0)
                .count(),
        })
    }
    fn locate(&self, cell: WorldCell) -> Option<(&Sector, usize)> {
        let x = i64::from(cell.x) - 32768;
        let y = i64::from(cell.y) - 32768;
        let sx = u16::try_from(x.div_euclid(64) + 32768).ok()?;
        let sy = u16::try_from(y.div_euclid(64) + 32768).ok()?;
        let sector = self.sectors.get(&(sx, sy))?;
        Some((sector, (y.rem_euclid(64) * 64 + x.rem_euclid(64)) as usize))
    }
    pub fn contains(&self, cell: WorldCell) -> bool {
        self.locate(cell).is_some()
    }
    pub fn direction_mask(&self, cell: WorldCell) -> u8 {
        self.locate(cell)
            .map(|(sector, index)| sector.directions[index])
            .unwrap_or(0)
    }
    pub fn standing(&self, cell: WorldCell) -> bool {
        self.direction_mask(cell) & 0x55 != 0
    }
    pub fn can_step(&self, from: WorldCell, to: WorldCell) -> bool {
        let dx = i64::from(to.x) - i64::from(from.x);
        let dy = i64::from(to.y) - i64::from(from.y);
        DIRECTIONS
            .iter()
            .position(|&(x, y)| i64::from(x) == dx && i64::from(y) == dy)
            .is_some_and(|heading| self.direction_mask(from) & (1 << heading) != 0)
    }
    fn attribute(&self, cell: WorldCell) -> Option<CellAttributes> {
        let (sector, index) = self.locate(cell)?;
        if sector.portals.contains(&index) {
            None
        } else {
            sector.data.attributes.get(index).copied()
        }
    }
    fn raw_cardinal(&self, from: WorldCell, to: WorldCell) -> bool {
        let Some(a) = self.attribute(from) else {
            return false;
        };
        let Some(b) = self.attribute(to) else {
            return false;
        };
        match (
            i64::from(to.x) - i64::from(from.x),
            i64::from(to.y) - i64::from(from.y),
        ) {
            (0, -1) => a.north_edge_open(),
            (1, 0) => a.east_edge_open(),
            (0, 1) => b.north_edge_open(),
            (-1, 0) => b.east_edge_open(),
            _ => false,
        }
    }
    fn raw_step(&self, from: WorldCell, to: WorldCell) -> bool {
        let dx = i64::from(to.x) - i64::from(from.x);
        let dy = i64::from(to.y) - i64::from(from.y);
        if dx.abs() > 1 || dy.abs() > 1 || (dx == 0 && dy == 0) {
            return false;
        }
        if dx == 0 || dy == 0 {
            return self.raw_cardinal(from, to);
        }
        let via_x = WorldCell { x: to.x, y: from.y };
        let via_y = WorldCell { x: from.x, y: to.y };
        (self.raw_cardinal(from, via_x) && self.raw_cardinal(via_x, to))
            || (self.raw_cardinal(from, via_y) && self.raw_cardinal(via_y, to))
    }
    /// Explicit seats, no randomized/fallback spawn. All must be distinct,
    /// standable and connected within the bound manifest; occupancy is separate.
    pub fn validate_spawns(&self, spawns: &[WorldCell], cancel: &AtomicBool) -> Result<()> {
        check(cancel)?;
        if spawns.is_empty() || spawns.len() > MAX_SPAWNS {
            return Err(WorldMapError::InvalidSpawns("count"));
        }
        let unique: BTreeSet<_> = spawns.iter().copied().collect();
        if unique.len() != spawns.len() || spawns.iter().any(|&cell| !self.standing(cell)) {
            return Err(WorldMapError::InvalidSpawns("duplicate/blocked/outside"));
        }
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::from([spawns[0]]);
        visited.insert(spawns[0]);
        while let Some(cell) = queue.pop_front() {
            check(cancel)?;
            for &(dx, dy) in &DIRECTIONS {
                let next = WorldCell {
                    x: cell.x + dx,
                    y: cell.y + dy,
                };
                if self.can_step(cell, next) && visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        if unique.iter().any(|cell| !visited.contains(cell)) {
            return Err(WorldMapError::InvalidSpawns("disconnected"));
        }
        Ok(())
    }
}

/// Sector-local center of the two 24x24 half tiles used by scene::project_half_tile:
/// even half origin +(24,12). No camera/window coordinate is mixed into geometry.
pub fn cell_to_sector_pixel(key: SharedMapSectorKey, cell: WorldCell) -> Option<PixelPoint> {
    let (ox, oy) = key.world_origin();
    let x = i64::from(cell.x) - i64::from(ox);
    let y = i64::from(cell.y) - i64::from(oy);
    if !(0..64).contains(&x) || !(0..64).contains(&y) {
        return None;
    }
    Some(PixelPoint {
        x: (24 * (x + y) + 24) as i32,
        y: (12 * (63 - x + y) + 12) as i32,
    })
}
/// Pixel selection uses half-open diamond boundaries: inverse local coordinates
/// [n-1/2,n+1/2) select n. Ties consistently choose the positive-axis cell.
pub fn sector_pixel_to_cell(key: SharedMapSectorKey, pixel: PixelPoint) -> Option<WorldCell> {
    let a = i64::from(pixel.x) - 24;
    let b = i64::from(pixel.y) - 768;
    let x = (a - 2 * b + 24).div_euclid(48);
    let y = (a + 2 * b + 24).div_euclid(48);
    if !(0..64).contains(&x) || !(0..64).contains(&y) {
        return None;
    }
    let (ox, oy) = key.world_origin();
    Some(WorldCell {
        x: ox + x as i32,
        y: oy + y as i32,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewProjection {
    pub anchor_cell: WorldCell,
    pub anchor_pixel: PixelPoint,
}
impl ViewProjection {
    pub fn cell_to_pixel(self, cell: WorldCell) -> Option<PixelPoint> {
        let dx = i64::from(cell.x) - i64::from(self.anchor_cell.x);
        let dy = i64::from(cell.y) - i64::from(self.anchor_cell.y);
        Some(PixelPoint {
            x: i32::try_from(i64::from(self.anchor_pixel.x) + 24 * (dx + dy)).ok()?,
            y: i32::try_from(i64::from(self.anchor_pixel.y) + 12 * (dy - dx)).ok()?,
        })
    }
    pub fn pixel_to_cell(self, pixel: PixelPoint) -> Option<WorldCell> {
        let a = i64::from(pixel.x) - i64::from(self.anchor_pixel.x);
        let b = i64::from(pixel.y) - i64::from(self.anchor_pixel.y);
        Some(WorldCell {
            x: i32::try_from(i64::from(self.anchor_cell.x) + (a - 2 * b + 24).div_euclid(48))
                .ok()?,
            y: i32::try_from(i64::from(self.anchor_cell.y) + (a + 2 * b + 24).div_euclid(48))
                .ok()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_sixty_four_sectors_keep_bidirectional_cross_sector_navigation() {
        let input = (0..64)
            .map(|i| SectorInput {
                key: key(0x8000 + i % 8, 0x8000 + i / 8),
                bytes: fixture(true, &[], &[]),
            })
            .collect();
        let map = WorldMap::from_sector_bytes(input, &AtomicBool::new(false)).unwrap();
        assert_eq!(map.bindings().len(), 64);
        assert!(map.can_step(cell(63, 31), cell(64, 31)));
        assert!(map.can_step(cell(64, 31), cell(63, 31)));
        assert_eq!(MAX_INPUT_BYTES, 32 * 1024 * 1024);
    }
    fn key(x: u16, y: u16) -> SharedMapSectorKey {
        SharedMapSectorKey {
            map_id: 514,
            sector_x: x,
            sector_y: y,
        }
    }
    fn key0() -> SharedMapSectorKey {
        key(0x8000, 0x8000)
    }
    fn cell(x: i32, y: i32) -> WorldCell {
        WorldCell {
            x: 32768 + x,
            y: 32768 + y,
        }
    }
    fn fixture(
        all_open: bool,
        edges: &[(usize, usize, bool, bool)],
        portals: &[(u8, u8)],
    ) -> Vec<u8> {
        let mut bytes = vec![0; 32768];
        bytes.extend(0u16.to_le_bytes());
        for y in 0..64 {
            for x in 0..64 {
                let pair = edges
                    .iter()
                    .find(|&&(ex, ey, _, _)| ex == x && ey == y)
                    .map(|&(_, _, n, e)| (n, e))
                    .unwrap_or((all_open, all_open));
                bytes.extend(i16::from(!pair.0).to_le_bytes());
                bytes.extend(i16::from(!pair.1).to_le_bytes());
            }
        }
        bytes.extend(0i32.to_le_bytes());
        bytes.extend(0i32.to_le_bytes());
        bytes.extend(0i32.to_le_bytes());
        bytes.extend((portals.len() as u16).to_le_bytes());
        for &(x, y) in portals {
            bytes.extend([0, x, y]);
            bytes.extend(4u16.to_le_bytes());
            bytes.extend(1i32.to_le_bytes());
        }
        bytes.extend(0u16.to_le_bytes());
        bytes
    }
    fn make(bytes: Vec<u8>) -> WorldMap {
        WorldMap::from_sector_bytes(
            vec![SectorInput { key: key0(), bytes }],
            &AtomicBool::new(false),
        )
        .unwrap()
    }
    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|value| format!("{value:02x}")).collect()
    }
    #[test]
    fn cardinal_boundary_ownership_and_modern_diagonal_require_complete_route() {
        let map = make(fixture(
            false,
            &[(10, 10, false, true), (11, 10, true, false)],
            &[],
        ));
        assert!(map.can_step(cell(10, 10), cell(11, 10)));
        assert!(map.can_step(cell(11, 10), cell(10, 10)));
        assert!(
            map.can_step(cell(10, 10), cell(11, 9)),
            "via east then north"
        );
        assert!(
            map.can_step(cell(11, 9), cell(10, 10)),
            "same two boundaries reverse"
        );
        assert!(!map.can_step(cell(10, 10), cell(10, 9)));
        assert!(!map.can_step(cell(10, 10), cell(12, 10)));
        assert!(!map.can_step(cell(10, 10), cell(10, 10)));
        let broken = make(fixture(
            false,
            &[(10, 10, false, true), (11, 9, true, true)],
            &[],
        ));
        assert!(broken.standing(cell(11, 9)));
        assert!(
            !broken.can_step(cell(10, 10), cell(11, 9)),
            "destination standing alone cannot cross corner wall"
        );
    }
    #[test]
    fn manifest_order_and_loaded_neighbor_edges_are_exact_no_missing_sector_fallback() {
        let one = make(fixture(true, &[], &[]));
        assert_eq!(one.direction_mask(cell(0, 0)), 0b00011100);
        assert!(!one.can_step(cell(63, 5), cell(64, 5)));
        let make_inputs = |reverse: bool| {
            let mut values = vec![
                SectorInput {
                    key: key0(),
                    bytes: fixture(true, &[], &[]),
                },
                SectorInput {
                    key: key(0x8001, 0x8000),
                    bytes: fixture(true, &[], &[]),
                },
            ];
            if reverse {
                values.reverse();
            }
            values
        };
        let forward =
            WorldMap::from_sector_bytes(make_inputs(false), &AtomicBool::new(false)).unwrap();
        let reverse =
            WorldMap::from_sector_bytes(make_inputs(true), &AtomicBool::new(false)).unwrap();
        assert_eq!(forward.canonical_bytes(), reverse.canonical_bytes());
        assert!(forward.can_step(cell(63, 5), cell(64, 5)));
        assert!(forward.can_step(cell(64, 5), cell(63, 5)));
        assert_ne!(one.canonical_digest(), forward.canonical_digest());
        assert_eq!(&one.canonical_bytes()[..PREFIX.len()], PREFIX);
        assert_eq!(one.canonical_bytes().len(), 23 + 4148);
        assert_eq!(&one.canonical_bytes()[15..23], &[0, 1, 0, 0, 2, 2, 0, 1]);
    }
    #[test]
    fn canonical_open_sector_golden_matches_independent_big_endian_fixture() {
        // Independently produced with a 49170-byte all-zero S32 and geometric
        // in-bounds neighbor masks, not with this Rust encoder or the Java map.
        let map = WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: key(0x7fff, 0x8000),
                bytes: fixture(true, &[], &[]),
            }],
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(map.bindings()[0].byte_length, 49170);
        assert_eq!(
            hex(&map.bindings()[0].sha256),
            "ef81ffbc2095f821ee3ae6e2d67fb8505b266ad2163bef49997d5e9cfce0a0a2"
        );
        assert_eq!(map.canonical_bytes().len(), 4171);
        assert_eq!(
            hex(&map.canonical_digest()),
            "5dd1c01c5f862a2b6bc4ac3b81a731871777feed9788e20bbdc15a2ce9dffa5e"
        );
    }
    #[test]
    fn portals_close_standing_cardinal_and_diagonal_and_spawns_are_unique_connected() {
        let map = make(fixture(true, &[], &[(22, 10)]));
        assert!(!map.standing(cell(11, 10)));
        assert!(!map.can_step(cell(10, 10), cell(11, 10)));
        assert!(!map.can_step(cell(10, 9), cell(11, 10)));
        map.validate_spawns(&[cell(10, 10), cell(12, 10)], &AtomicBool::new(false))
            .unwrap();
        for points in [
            vec![],
            vec![cell(10, 10), cell(10, 10)],
            vec![cell(11, 10)],
            vec![cell(64, 10)],
            vec![cell(10, 10); 101],
        ] {
            assert!(map
                .validate_spawns(&points, &AtomicBool::new(false))
                .is_err());
        }
        let isolated = make(fixture(
            false,
            &[(1, 1, false, true), (10, 10, false, true)],
            &[],
        ));
        assert!(isolated
            .validate_spawns(&[cell(1, 1), cell(10, 10)], &AtomicBool::new(false))
            .is_err());
    }
    #[test]
    fn all_cells_project_to_shared_half_tile_pair_center_and_inverse_exactly() {
        let selected = key(0x7fff, 0x8000);
        let (ox, oy) = selected.world_origin();
        for y in 0..64 {
            for x in 0..64 {
                let world = WorldCell {
                    x: ox + x,
                    y: oy + y,
                };
                let pixel = cell_to_sector_pixel(selected, world).unwrap();
                let half = super::super::scene::project_half_tile((2 * x) as u8, y as u8);
                assert_eq!(
                    pixel,
                    PixelPoint {
                        x: half.0 + 24,
                        y: half.1 + 12
                    }
                );
                assert_eq!(sector_pixel_to_cell(selected, pixel), Some(world));
                let projection = ViewProjection {
                    anchor_cell: world,
                    anchor_pixel: PixelPoint { x: 400, y: 300 },
                };
                for (dx, dy) in DIRECTIONS {
                    let next = WorldCell {
                        x: world.x + dx,
                        y: world.y + dy,
                    };
                    assert_eq!(
                        projection.pixel_to_cell(projection.cell_to_pixel(next).unwrap()),
                        Some(next)
                    );
                }
            }
        }
        let first = WorldCell { x: ox, y: oy };
        let center = cell_to_sector_pixel(selected, first).unwrap();
        assert!(sector_pixel_to_cell(
            selected,
            PixelPoint {
                x: center.x - 25,
                y: center.y
            }
        )
        .is_none());
        assert_eq!(
            sector_pixel_to_cell(
                selected,
                PixelPoint {
                    x: center.x + 24,
                    y: center.y
                }
            ),
            Some(WorldCell {
                x: ox + 1,
                y: oy + 1
            })
        );
        assert!(cell_to_sector_pixel(selected, WorldCell { x: ox - 1, y: oy }).is_none());
        let projection = ViewProjection {
            anchor_cell: WorldCell {
                x: i32::MIN,
                y: i32::MIN,
            },
            anchor_pixel: PixelPoint {
                x: i32::MAX,
                y: i32::MAX,
            },
        };
        assert!(projection
            .cell_to_pixel(WorldCell {
                x: i32::MAX,
                y: i32::MAX
            })
            .is_none());
    }
    #[test]
    fn malformed_duplicate_cross_map_oversized_tail_and_cancellation_fail_closed() {
        let cancel = AtomicBool::new(false);
        assert!(WorldMap::from_sector_bytes(vec![], &cancel).is_err());
        assert!(WorldMap::from_sector_bytes(
            vec![
                SectorInput {
                    key: key0(),
                    bytes: fixture(true, &[], &[])
                },
                SectorInput {
                    key: key0(),
                    bytes: fixture(true, &[], &[])
                }
            ],
            &cancel
        )
        .is_err());
        let mut other = key(0x8001, 0x8000);
        other.map_id = 4;
        assert!(WorldMap::from_sector_bytes(
            vec![
                SectorInput {
                    key: key0(),
                    bytes: fixture(true, &[], &[])
                },
                SectorInput {
                    key: other,
                    bytes: fixture(true, &[], &[])
                }
            ],
            &cancel
        )
        .is_err());
        let mut tail = fixture(true, &[], &[]);
        tail.push(1);
        assert!(matches!(
            WorldMap::from_sector_bytes(
                vec![SectorInput {
                    key: key0(),
                    bytes: tail
                }],
                &cancel
            ),
            Err(WorldMapError::UnsupportedTail(_))
        ));
        let mut broken = fixture(true, &[], &[]);
        broken.truncate(32770);
        assert!(WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: key0(),
                bytes: broken
            }],
            &cancel
        )
        .is_err());
        assert!(WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: key0(),
                bytes: fixture(true, &[], &[(128, 1)])
            }],
            &cancel
        )
        .is_err());
        assert!(WorldMap::from_sector_bytes(
            (0..65)
                .map(|x| SectorInput {
                    key: key(x, 0),
                    bytes: fixture(true, &[], &[])
                })
                .collect(),
            &cancel
        )
        .is_err());
        let manifest = [SectorBinding {
            key: key0(),
            byte_length: MAX_SHARED_MAP_SECTOR_BYTES as u32 + 1,
            sha256: [0; 32],
        }];
        assert!(
            WorldMap::load_shared(Path::new("intentionally-unavailable"), &manifest, &cancel)
                .is_err()
        );
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            WorldMap::from_sector_bytes(
                vec![SectorInput {
                    key: key0(),
                    bytes: fixture(true, &[], &[])
                }],
                &cancel
            ),
            Err(WorldMapError::Cancelled)
        ));
    }
    fn actual_binding() -> SectorBinding {
        SectorBinding {
            key: key(0x7fff, 0x8000),
            byte_length: 137784,
            sha256: [
                0xcc, 0xfe, 0x52, 0xb6, 0x14, 0xdf, 0xbd, 0xf9, 0x7f, 0xd7, 0x13, 0xf8, 0x0d, 0x61,
                0xe4, 0xcd, 0xd3, 0x5e, 0x7c, 0x8c, 0x30, 0x00, 0xa0, 0x6b, 0x4b, 0xba, 0x88, 0x68,
                0xc9, 0x6d, 0x12, 0x71,
            ],
        }
    }
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; bounded actual shared map514 geometry and in-memory static render"]
    fn real_shared_map514_binding_connected_spawns_and_static_art_are_verified_without_lt() {
        use super::super::{
            resources::{SharedArchive, SharedArchiveKind},
            scene::{render_tiles, static_draw_list, CameraRect},
        };
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared resource root");
        let cancel = AtomicBool::new(false);
        let binding = actual_binding();
        let map = WorldMap::load_shared(Path::new(&root), &[binding], &cancel).unwrap();
        assert_eq!(map.bindings(), vec![binding]);
        assert_eq!(
            hex(&map.canonical_digest()),
            "2feecc83de156a03c62e44effb7106067b24098a9ef7250cc471f7bf826da7a8"
        );
        assert_eq!(
            hex(&Sha256::digest(
                map.sectors.values().next().unwrap().directions.as_ref()
            )),
            "01fbe2c5df7e16640dd09a54171857f2aebe9ce9792e017a3b378c16a22cbede"
        );
        let spawns = [
            WorldCell { x: 32724, y: 32784 },
            WorldCell { x: 32725, y: 32784 },
        ];
        map.validate_spawns(&spawns, &cancel).unwrap();
        assert!(map.can_step(spawns[0], spawns[1]));
        assert_eq!(map.direction_mask(spawns[0]), 60);
        let sector = map.sector(binding.key).unwrap();
        assert_eq!(sector.layer_ranges[2].as_ref().unwrap().start, 32770);
        assert!(sector.overlays.is_empty());
        assert_eq!(sector.object_tile_count(), 12387);
        assert!(sector.portals.is_empty());
        assert!(!sector.has_unknown_tail());
        let limits = map.art_limitations(binding.key).unwrap();
        assert_eq!(limits.opacity_records, 17);
        assert_eq!(limits.sprite_effects, 36);
        assert!(limits.has_known_presentation_gaps());
        let anchor = cell_to_sector_pixel(binding.key, spawns[0]).unwrap();
        let camera = CameraRect {
            x: anchor.x - 400,
            y: anchor.y - 300,
            width: 800,
            height: 600,
        };
        let scene = render_tiles(
            &mut SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap(),
            &static_draw_list(sector).unwrap(),
            camera,
        )
        .unwrap();
        assert_eq!(scene.visible_tiles, 3703);
        assert_eq!(scene.tile_sets_loaded, 24);
        // Reviewed modern static-render baseline. Independent C# per-tile
        // goldens live in tile.rs; this is not native BIN frame equivalence.
        assert_eq!(
            hex(&Sha256::digest(scene.canvas.rgba())),
            "083ce3d0137a001e13a536659886cdd20e13c3865fa74a3e78baedc38c3871c7"
        );
        let standing = map
            .sectors
            .values()
            .map(|sector| {
                sector
                    .directions
                    .iter()
                    .filter(|mask| **mask & 0x55 != 0)
                    .count()
            })
            .sum::<usize>();
        assert_eq!(standing, 986);
        eprintln!(
            "shared514 sourceSHA={} geometrySHA={} canonicalBytes={} standing={} spawns=32724,32784;32725,32784 staticRGBA={} visible={} tilesets={} opacity17/effects36-not-complete-world",
            hex(&binding.sha256),
            hex(&map.canonical_digest()),
            map.canonical_bytes().len(),
            standing,
            hex(&Sha256::digest(scene.canvas.rgba())),
            scene.visible_tiles,
            scene.tile_sets_loaded
        );
        // Explicit diagnostic artifact only; ordinary resource verification is memory-only.
        // This is static terrain, NOT a screenshot of a running game or proof of roof/effect support.
        if std::env::var("LITO_WORLD_MAP_WRITE_REFERENCE").as_deref() == Ok("1") {
            let directory =
                Path::new(env!("CARGO_MANIFEST_DIR")).join("target/modern-lito-render-tests");
            std::fs::create_dir_all(&directory).unwrap();
            let output = directory.join("shared-map514-static-reference.png");
            let file = std::fs::File::create(&output).unwrap();
            let mut encoder = png::Encoder::new(file, scene.canvas.width(), scene.canvas.height());
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(scene.canvas.rgba())
                .unwrap();
            eprintln!("Static shared 514 terrain diagnostic: {}", output.display());
        }
        let mut wrong = binding;
        wrong.sha256[0] ^= 1;
        assert!(matches!(
            WorldMap::load_shared(Path::new(&root), &[wrong], &cancel),
            Err(WorldMapError::HashMismatch(_))
        ));
    }
}
