//! Bounded shared Lineage S32 terrain decoder.
//!
//! Format source: PakViewer Lin.Helper.Core/Map/S32Reader.cs and S32Data.cs.
//! Boundary ownership: the existing launcher minimap/nav_grid.rs. This module
//! preserves the rendering layers and raw metadata that its navigation-only
//! parser discards. It does not interpret SEG, remaster/Fs32, or LT maps.

use super::resources::{SharedArchiveKind, MAX_SHARED_MAP_SECTOR_BYTES};
use std::fmt;
use std::ops::Range;

pub const FLOOR_WIDTH: usize = 128;
pub const SECTOR_SIDE: usize = 64;
const FLOOR_COUNT: usize = FLOOR_WIDTH * SECTOR_SIDE;
const CELL_COUNT: usize = SECTOR_SIDE * SECTOR_SIDE;
const MAX_LAYER_RECORDS: usize = 262_144;
const MAX_OBJECT_GROUPS: usize = 65_536;

/// S32 stores an 8-bit subtile index followed by a little-endian 16-bit
/// TileId and one uninterpreted byte. TileProvider loads `{tile_id}.til` from
/// the shared Tile archive; the subtile is an index inside that TIL file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TileReference {
    pub tile_id: u16,
    pub subtile: u8,
    pub unknown: u8,
}
impl TileReference {
    pub fn archive_kind(self) -> SharedArchiveKind {
        SharedArchiveKind::Tile
    }
    pub fn entry_name(self) -> String {
        format!("{}.til", self.tile_id)
    }
    pub fn original_bytes(self) -> [u8; 4] {
        let id = self.tile_id.to_le_bytes();
        [self.subtile, id[0], id[1], self.unknown]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroundOverlay {
    pub x: u8,
    pub y: u8,
    pub tile: TileReference,
}

/// Full signed 16-bit values are retained. Only bit zero has a navigation
/// interpretation here; zone, roof and other flags are not guessed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CellAttributes {
    pub attribute1: i16,
    pub attribute2: i16,
}
impl CellAttributes {
    pub fn north_edge_open(self) -> bool {
        self.attribute1 & 1 == 0
    }
    pub fn east_edge_open(self) -> bool {
        self.attribute2 & 1 == 0
    }
    pub fn original_bytes(self) -> [u8; 4] {
        let a = self.attribute1.to_le_bytes();
        let b = self.attribute2.to_le_bytes();
        [a[0], a[1], b[0], b[1]]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectTile {
    pub x: u8,
    pub y: u8,
    pub layer: u8,
    pub subtile: u8,
    /// The reference reader specifies a signed value for Layer4, unlike L1/L2.
    pub tile_id: i16,
    pub unknown: u8,
}
impl ObjectTile {
    /// Negative IDs remain explicit unsupported references instead of being
    /// silently reinterpreted as a large shared Tile filename.
    pub fn shared_tile_reference(self) -> Option<TileReference> {
        u16::try_from(self.tile_id)
            .ok()
            .map(|tile_id| TileReference {
                tile_id,
                subtile: self.subtile,
                unknown: self.unknown,
            })
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGroup {
    pub id: i16,
    pub tiles: Vec<ObjectTile>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpacityRecord {
    pub x: u8,
    pub y: u8,
    pub object_index: u16,
    pub kind: u8,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortalRecord {
    /// Raw locale-dependent bytes, not assumed to be UTF-8.
    pub name: Vec<u8>,
    /// Layer1 local coordinates: 128-wide x, not 64-wide game-cell x.
    pub x: u8,
    pub y: u8,
    pub target_map: u16,
    pub portal_id: i32,
    /// Native optional record suffix 100..=102; retained without gameplay meaning.
    /// Shared reference parsing never consumes this suffix.
    pub native_kind: Option<u8>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpriteEffect {
    pub sprite_id: u16,
    pub x: u16,
    pub y: u16,
    pub extended: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cardinal {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Debug)]
pub struct S32Sector {
    /// Row-major 128 x 64 half-tiles.
    pub floor: Vec<TileReference>,
    pub overlays: Vec<GroundOverlay>,
    /// Row-major 64 x 64 cells, with two independently owned boundaries each.
    pub attributes: Vec<CellAttributes>,
    pub objects: Vec<ObjectGroup>,
    pub opacity: Vec<OpacityRecord>,
    /// Raw signed Layer6 references; no implicit reinterpretation of negatives.
    pub tile_dependencies: Vec<i32>,
    pub portals: Vec<PortalRecord>,
    pub effects: Vec<SpriteEffect>,
    pub effects_extended: bool,
    /// None denotes an absent optional suffix layer, not a zero-count layer.
    pub layer_ranges: [Option<Range<usize>>; 8],
    /// Unrecognized bytes after complete Layer8 are retained and surfaced.
    /// A renderer/gameplay loader must decide whether it supports that variant.
    pub unknown_trailing: Vec<u8>,
}

impl S32Sector {
    pub fn floor_at(&self, half_x: usize, y: usize) -> Option<TileReference> {
        if half_x >= FLOOR_WIDTH || y >= SECTOR_SIDE {
            return None;
        }
        self.floor.get(y * FLOOR_WIDTH + half_x).copied()
    }
    pub fn attributes_at(&self, x: i32, y: i32) -> Option<CellAttributes> {
        if !(0..64).contains(&x) || !(0..64).contains(&y) {
            return None;
        }
        self.attributes.get(y as usize * 64 + x as usize).copied()
    }
    /// None means a boundary belongs to a neighboring sector. It must be
    /// resolved from that sector; missing data is not treated as free terrain.
    pub fn cardinal_edge_open(&self, x: i32, y: i32, direction: Cardinal) -> Option<bool> {
        self.attributes_at(x, y)?;
        match direction {
            Cardinal::North => self
                .attributes_at(x, y)
                .map(CellAttributes::north_edge_open),
            Cardinal::East => self.attributes_at(x, y).map(CellAttributes::east_edge_open),
            Cardinal::South => self
                .attributes_at(x, y + 1)
                .map(CellAttributes::north_edge_open),
            Cardinal::West => self
                .attributes_at(x - 1, y)
                .map(CellAttributes::east_edge_open),
        }
    }
    /// A standing cell can have an open edge owned by a neighbor even when
    /// both of its own attributes are blocked. This is not actor collision.
    pub fn standing_floor_open(&self, x: i32, y: i32) -> Option<bool> {
        self.attributes_at(x, y)?;
        let mut missing = false;
        for direction in [
            Cardinal::North,
            Cardinal::East,
            Cardinal::South,
            Cardinal::West,
        ] {
            match self.cardinal_edge_open(x, y, direction) {
                Some(true) => return Some(true),
                None => missing = true,
                _ => {}
            }
        }
        if missing {
            None
        } else {
            Some(false)
        }
    }
    pub fn has_unknown_tail(&self) -> bool {
        !self.unknown_trailing.is_empty()
    }
    pub fn object_tile_count(&self) -> usize {
        self.objects.iter().map(|group| group.tiles.len()).sum()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MapError {
    TooLarge {
        size: usize,
        limit: usize,
    },
    Truncated {
        layer: u8,
        offset: usize,
        required: usize,
        available: usize,
    },
    NegativeCount {
        layer: u8,
        value: i32,
    },
    TooManyRecords {
        layer: u8,
        count: usize,
        limit: usize,
    },
}
impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { size, limit } => write!(f, "S32 size {size} exceeds {limit}"),
            Self::Truncated {
                layer,
                offset,
                required,
                available,
            } => write!(
                f,
                "S32 Layer{layer} truncated at {offset}: need {required} bytes, {available} available"
            ),
            Self::NegativeCount { layer, value } => {
                write!(f, "S32 Layer{layer} has negative count {value}")
            }
            Self::TooManyRecords {
                layer,
                count,
                limit,
            } => write!(f, "S32 Layer{layer} count {count} exceeds {limit}"),
        }
    }
}
impl std::error::Error for MapError {}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
    layer: u8,
}
impl<'a> Reader<'a> {
    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }
    fn ensure(&self, count: usize) -> Result<(), MapError> {
        if count > self.remaining() {
            return Err(MapError::Truncated {
                layer: self.layer,
                offset: self.offset,
                required: count,
                available: self.remaining(),
            });
        }
        Ok(())
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], MapError> {
        self.ensure(count)?;
        let start = self.offset;
        self.offset += count;
        Ok(&self.bytes[start..self.offset])
    }
    fn u8(&mut self) -> Result<u8, MapError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, MapError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn i16(&mut self) -> Result<i16, MapError> {
        Ok(i16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, MapError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn records(&self, count: usize, size: usize, limit: usize) -> Result<(), MapError> {
        if count > limit {
            return Err(MapError::TooManyRecords {
                layer: self.layer,
                count,
                limit,
            });
        }
        // Limits are at most 262144 and item sizes at most 10, safe on i686.
        self.ensure(count * size)
    }
    fn count_i32(&mut self, size: usize, limit: usize) -> Result<usize, MapError> {
        let value = self.i32()?;
        let count = usize::try_from(value).map_err(|_| MapError::NegativeCount {
            layer: self.layer,
            value,
        })?;
        self.records(count, size, limit)?;
        Ok(count)
    }
    fn tile(&mut self) -> Result<TileReference, MapError> {
        Ok(TileReference {
            subtile: self.u8()?,
            tile_id: self.u16()?,
            unknown: self.u8()?,
        })
    }
}

/// Parse all complete known layers. Layers1–4 are mandatory; an optional
/// suffix may end exactly at a Layer4/5/6/7/8 boundary. An incomplete count or
/// record is always an error. Unknown trailing bytes after Layer8 are exposed.
pub fn parse_s32(bytes: &[u8]) -> Result<S32Sector, MapError> {
    parse_s32_format(bytes, false)
}

/// Original Native S32 portal suffix: `sub_483A80` loc_484C49..484C91
/// reads the fixed eight bytes, peeks the following byte and consumes it only
/// when signed `cl` is 100..=102. All other bytes remain the next record/count.
/// This explicit Native entry point preserves the existing Shared wire format.
pub fn parse_native_s32(bytes: &[u8]) -> Result<S32Sector, MapError> {
    parse_s32_format(bytes, true)
}

fn parse_s32_format(bytes: &[u8], native: bool) -> Result<S32Sector, MapError> {
    if bytes.len() > MAX_SHARED_MAP_SECTOR_BYTES as usize {
        return Err(MapError::TooLarge {
            size: bytes.len(),
            limit: MAX_SHARED_MAP_SECTOR_BYTES as usize,
        });
    }
    let mut reader = Reader {
        bytes,
        offset: 0,
        layer: 1,
    };
    reader.records(FLOOR_COUNT, 4, FLOOR_COUNT)?;
    let mut floor = Vec::with_capacity(FLOOR_COUNT);
    for _ in 0..FLOOR_COUNT {
        floor.push(reader.tile()?);
    }
    let floor_end = reader.offset;
    reader.layer = 2;
    let layer2_start = reader.offset;
    let count = reader.u16()? as usize;
    reader.records(count, 6, MAX_LAYER_RECORDS)?;
    let mut overlays = Vec::with_capacity(count);
    for _ in 0..count {
        overlays.push(GroundOverlay {
            x: reader.u8()?,
            y: reader.u8()?,
            tile: reader.tile()?,
        });
    }
    let layer2_end = reader.offset;
    reader.layer = 3;
    reader.records(CELL_COUNT, 4, CELL_COUNT)?;
    let mut attributes = Vec::with_capacity(CELL_COUNT);
    for _ in 0..CELL_COUNT {
        attributes.push(CellAttributes {
            attribute1: reader.i16()?,
            attribute2: reader.i16()?,
        });
    }
    let layer3_end = reader.offset;
    reader.layer = 4;
    let group_count = reader.count_i32(4, MAX_OBJECT_GROUPS)?;
    let mut objects = Vec::with_capacity(group_count);
    let mut total_objects = 0;
    for _ in 0..group_count {
        let id = reader.i16()?;
        let count = reader.u16()? as usize;
        total_objects += count;
        if total_objects > MAX_LAYER_RECORDS {
            return Err(MapError::TooManyRecords {
                layer: 4,
                count: total_objects,
                limit: MAX_LAYER_RECORDS,
            });
        }
        reader.records(count, 7, MAX_LAYER_RECORDS)?;
        let mut tiles = Vec::with_capacity(count);
        for _ in 0..count {
            tiles.push(ObjectTile {
                x: reader.u8()?,
                y: reader.u8()?,
                layer: reader.u8()?,
                subtile: reader.u8()?,
                tile_id: reader.i16()?,
                unknown: reader.u8()?,
            });
        }
        objects.push(ObjectGroup { id, tiles });
    }
    let mut sector = S32Sector {
        floor,
        overlays,
        attributes,
        objects,
        opacity: Vec::new(),
        tile_dependencies: Vec::new(),
        portals: Vec::new(),
        effects: Vec::new(),
        effects_extended: false,
        layer_ranges: [
            Some(0..floor_end),
            Some(layer2_start..layer2_end),
            Some(layer2_end..layer3_end),
            Some(layer3_end..reader.offset),
            None,
            None,
            None,
            None,
        ],
        unknown_trailing: Vec::new(),
    };
    if reader.remaining() == 0 {
        return Ok(sector);
    }
    reader.layer = 5;
    let start = reader.offset;
    let count = reader.count_i32(5, MAX_LAYER_RECORDS)?;
    sector.opacity.reserve(count);
    for _ in 0..count {
        sector.opacity.push(OpacityRecord {
            x: reader.u8()?,
            y: reader.u8()?,
            object_index: reader.u16()?,
            kind: reader.u8()?,
        });
    }
    sector.layer_ranges[4] = Some(start..reader.offset);
    if reader.remaining() == 0 {
        return Ok(sector);
    }
    reader.layer = 6;
    let start = reader.offset;
    let count = reader.count_i32(4, MAX_LAYER_RECORDS)?;
    sector.tile_dependencies.reserve(count);
    for _ in 0..count {
        sector.tile_dependencies.push(reader.i32()?);
    }
    sector.layer_ranges[5] = Some(start..reader.offset);
    if reader.remaining() == 0 {
        return Ok(sector);
    }
    reader.layer = 7;
    let start = reader.offset;
    let count = reader.u16()? as usize;
    reader.records(count, 9, MAX_LAYER_RECORDS)?;
    sector.portals.reserve(count);
    for _ in 0..count {
        let name_length = reader.u8()? as usize;
        reader.ensure(name_length + 8)?;
        let name = reader.take(name_length)?.to_vec();
        let mut portal = PortalRecord {
            name,
            x: reader.u8()?,
            y: reader.u8()?,
            target_map: reader.u16()?,
            portal_id: reader.i32()?,
            native_kind: None,
        };
        if native && reader.bytes.get(reader.offset).is_some_and(|b| (100..=102).contains(b)) {
            portal.native_kind = Some(reader.u8()?);
        }
        sector.portals.push(portal);
    }
    sector.layer_ranges[6] = Some(start..reader.offset);
    if reader.remaining() == 0 {
        return Ok(sector);
    }
    reader.layer = 8;
    let start = reader.offset;
    let packed_count = reader.u16()?;
    let count = (packed_count & 0x7fff) as usize;
    sector.effects_extended = packed_count & 0x8000 != 0;
    reader.records(
        count,
        if sector.effects_extended { 10 } else { 6 },
        MAX_LAYER_RECORDS,
    )?;
    sector.effects.reserve(count);
    for _ in 0..count {
        sector.effects.push(SpriteEffect {
            sprite_id: reader.u16()?,
            x: reader.u16()?,
            y: reader.u16()?,
            extended: if sector.effects_extended {
                Some(reader.i32()?)
            } else {
                None
            },
        });
    }
    sector.layer_ranges[7] = Some(start..reader.offset);
    sector.unknown_trailing = reader.take(reader.remaining())?.to_vec();
    Ok(sector)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_sector() -> Vec<u8> {
        let mut bytes = vec![0; FLOOR_COUNT * 4];
        bytes.extend(0u16.to_le_bytes());
        bytes.extend(vec![0; CELL_COUNT * 4]);
        bytes.extend(0i32.to_le_bytes());
        bytes
    }
    fn complete_sector(extended: bool) -> Vec<u8> {
        let mut bytes = base_sector();
        bytes.extend(1i32.to_le_bytes());
        bytes.extend([2, 3, 4, 0, 5]);
        bytes.extend(1i32.to_le_bytes());
        bytes.extend(321i32.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend([2, 0xa4, 0xa4, 12, 13]);
        bytes.extend(4u16.to_le_bytes());
        bytes.extend((-17i32).to_le_bytes());
        bytes.extend((if extended { 0x8001u16 } else { 1u16 }).to_le_bytes());
        bytes.extend(42u16.to_le_bytes());
        bytes.extend(250u16.to_le_bytes());
        bytes.extend(360u16.to_le_bytes());
        if extended {
            bytes.extend((-123i32).to_le_bytes());
        }
        bytes
    }

    #[test]
    fn floor_reference_retains_exact_packing_and_archive_identity() {
        let mut bytes = base_sector();
        bytes[..4].copy_from_slice(&[233, 25, 0, 0x91]);
        let sector = parse_s32(&bytes).unwrap();
        let tile = sector.floor_at(0, 0).unwrap();
        assert_eq!(tile.tile_id, 25);
        assert_eq!(tile.subtile, 233);
        assert_eq!(tile.unknown, 0x91);
        assert_eq!(tile.original_bytes(), [233, 25, 0, 0x91]);
        assert_eq!(tile.archive_kind(), SharedArchiveKind::Tile);
        assert_eq!(tile.entry_name(), "25.til");
        assert!(sector.floor_at(128, 0).is_none());
        assert!(sector.floor_at(0, 64).is_none());
        assert_eq!(sector.layer_ranges.iter().flatten().count(), 4);
    }

    #[test]
    fn overlay_shifts_attributes_and_objects_keep_signed_fields() {
        let mut bytes = vec![0; FLOOR_COUNT * 4];
        bytes.extend(1u16.to_le_bytes());
        bytes.extend([250, 240, 7, 0x34, 0x12, 0xfa]);
        bytes.extend(vec![0; CELL_COUNT * 4]);
        let offset = FLOOR_COUNT * 4 + 2 + 6;
        bytes[offset..offset + 4].copy_from_slice(&[1, 0x80, 4, 0x12]);
        bytes.extend(1i32.to_le_bytes());
        bytes.extend((-2i16).to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend([200, 201, 12, 15, 0xff, 0xff, 0xe1]);
        let sector = parse_s32(&bytes).unwrap();
        assert_eq!(sector.overlays[0].tile.tile_id, 0x1234);
        assert_eq!(sector.overlays[0].x, 250);
        assert_eq!(sector.attributes[0].original_bytes(), [1, 0x80, 4, 0x12]);
        assert_eq!(sector.objects[0].id, -2);
        assert_eq!(sector.objects[0].tiles[0].tile_id, -1);
        assert_eq!(sector.objects[0].tiles[0].unknown, 0xe1);
        assert!(sector.objects[0].tiles[0].shared_tile_reference().is_none());
    }

    #[test]
    fn all_known_suffix_layers_and_extended_effects_are_preserved() {
        for extended in [false, true] {
            let sector = parse_s32(&complete_sector(extended)).unwrap();
            assert_eq!(
                sector.opacity[0],
                OpacityRecord {
                    x: 2,
                    y: 3,
                    object_index: 4,
                    kind: 5
                }
            );
            assert_eq!(sector.tile_dependencies, vec![321]);
            assert_eq!(sector.portals[0].name, vec![0xa4, 0xa4]);
            assert_eq!(sector.portals[0].portal_id, -17);
            assert_eq!(sector.portals[0].target_map, 4);
            assert_eq!(
                sector.effects[0].extended,
                if extended { Some(-123) } else { None }
            );
            assert_eq!(sector.effects[0].x, 250);
            assert_eq!(sector.layer_ranges.iter().flatten().count(), 8);
            assert!(!sector.has_unknown_tail());
        }
    }

    #[test]
    fn unknown_tail_is_visible_instead_of_discarded() {
        let mut bytes = complete_sector(false);
        bytes.extend([0xde, 0xad, 0xbe, 0xef]);
        let sector = parse_s32(&bytes).unwrap();
        assert!(sector.has_unknown_tail());
        assert_eq!(sector.unknown_trailing, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn complete_optional_boundaries_are_distinct_from_truncated_records() {
        let bytes = complete_sector(true);
        let parsed = parse_s32(&bytes).unwrap();
        for layer in 3..8 {
            let end = parsed.layer_ranges[layer].as_ref().unwrap().end;
            assert!(parse_s32(&bytes[..end]).is_ok());
            if layer >= 4 {
                assert!(parse_s32(&bytes[..end - 1]).is_err());
            }
        }
        for length in [0, 32767, 32768, 32769, 49153, 49154, 49155, 49156, 49157] {
            assert!(
                parse_s32(&bytes[..length]).is_err(),
                "accepted truncation {length}"
            );
        }
    }

    #[test]
    fn negative_and_oversized_counts_fail_before_allocating() {
        for value in [-1, 65537, i32::MAX] {
            let mut bytes = base_sector();
            let end = bytes.len();
            bytes[end - 4..].copy_from_slice(&value.to_le_bytes());
            assert!(matches!(
                parse_s32(&bytes),
                Err(MapError::NegativeCount { .. } | MapError::TooManyRecords { .. })
            ));
        }
        let mut bytes = base_sector();
        bytes.extend((-1i32).to_le_bytes());
        assert!(matches!(
            parse_s32(&bytes),
            Err(MapError::NegativeCount { layer: 5, .. })
        ));
        let bytes = vec![0; MAX_SHARED_MAP_SECTOR_BYTES as usize + 1];
        assert!(matches!(parse_s32(&bytes), Err(MapError::TooLarge { .. })));
    }

    #[test]
    fn directional_boundaries_preserve_ownership_and_neighbor_requirements() {
        let mut sector = parse_s32(&base_sector()).unwrap();
        sector.attributes.fill(CellAttributes {
            attribute1: 1,
            attribute2: 1,
        });
        sector.attributes[11 * 64 + 10].attribute1 = 0;
        assert_eq!(
            sector.cardinal_edge_open(10, 10, Cardinal::North),
            Some(false)
        );
        assert_eq!(
            sector.cardinal_edge_open(10, 10, Cardinal::South),
            Some(true)
        );
        assert_eq!(
            sector.cardinal_edge_open(10, 11, Cardinal::North),
            Some(true)
        );
        assert_eq!(sector.standing_floor_open(10, 10), Some(true));
        assert_eq!(sector.standing_floor_open(20, 20), Some(false));
        assert_eq!(sector.cardinal_edge_open(0, 20, Cardinal::West), None);
        assert_eq!(sector.cardinal_edge_open(20, 63, Cardinal::South), None);
        assert_eq!(sector.standing_floor_open(20, 63), None);
        assert_eq!(sector.attributes_at(i32::MAX, -1), None);
    }

    #[test]
    #[ignore = "requires selected shared map4/7ffc7ff7.s32 via LITO_SHARED_RESOURCE_ROOT"]
    fn real_shared_map4_sector_matches_csharp_layers() {
        use super::super::resources::{read_shared_map_sector, SharedMapSectorKey};
        use sha2::{Digest, Sha256};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let key = SharedMapSectorKey {
            map_id: 4,
            sector_x: 0x7ffc,
            sector_y: 0x7ff7,
        };
        let bytes = read_shared_map_sector(std::path::Path::new(&root), key).unwrap();
        assert_eq!(bytes.len(), 104_980);
        assert_eq!(
            format!("{:X}", Sha256::digest(&bytes)),
            "A2821287AEF1624B2A1CAA3B2A9DBB49649A40A85CB23D63743A093F9A328FD6"
        );
        let sector = parse_s32(&bytes).unwrap();
        let floor: Vec<u8> = sector
            .floor
            .iter()
            .flat_map(|tile| tile.original_bytes())
            .collect();
        let attrs: Vec<u8> = sector
            .attributes
            .iter()
            .flat_map(|cell| cell.original_bytes())
            .collect();
        assert_eq!(
            format!("{:X}", Sha256::digest(&floor)),
            "A2E6638CCEE99DBC99BEF2935EC5133CD82604A8622AC589B178562641ACDDF6"
        );
        assert_eq!(
            format!("{:X}", Sha256::digest(&attrs)),
            "76A6F23511B8989AF6DBE207691F848FBBE7349BCB8514DE4F1462576E8C7A56"
        );
        assert_eq!(sector.overlays.len(), 3);
        assert_eq!(sector.object_tile_count(), 6781);
        assert_eq!(sector.opacity.len(), 1537);
        assert_eq!(sector.tile_dependencies.len(), 52);
        assert_eq!(sector.layer_ranges[2].as_ref().unwrap().start, 32788);
        assert_eq!(sector.layer_ranges[3].as_ref().unwrap().end, 97075);
        assert!(sector.portals.is_empty() && sector.effects.is_empty());
        assert!(!sector.has_unknown_tail());
        assert_eq!(sector.floor[0].tile_id, 25);
        assert_eq!(sector.floor[0].subtile, 233);
        eprintln!(
            "shared map4 {}: {} bytes; {} floor refs; {} objects; floor and directional Layer3 hashes match C#",
            key.file_name(),
            bytes.len(),
            sector.floor.len(),
            sector.object_tile_count()
        );
    }
}
