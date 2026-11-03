//! Native half-cell draw ordering, without native process hooks or addresses.
//!
//! Evidence: LiTo disassembly SHA256
//! 2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A.
//! 44014B -> 4CD9D0 -> 483770 inserts SPR fragments into the SAME half-cell
//! lists as S32 objects. 488C34..488F2B paints one cell's floor then its list,
//! scanning screen rows top-to-bottom and columns left-to-right. 483815 uses
//! `new_depth <= old_depth`: the newest insertion paints FIRST on a tie.
//!
//! WorldScene/world_raster use these keys for map and SPR fragments. They do
//! not select actor-dependent roof state (world_roofs owns it), all native
//! TIL blends or actor update/insertion history; modern snapshot order is explicit.

use std::cmp::Reverse;

use super::sprite::SpriteBlockReference;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeDrawCell {
    /// Native half-cell X, NOT a server whole-cell X.
    pub half_x: i32,
    pub y: i32,
}

/// Comparable only within one scene and one insertion-sequence domain.
/// Keep source insertion sequence; actor IDs cannot substitute for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NativeDrawKey {
    row: i64,
    column: i64,
    depth: i16,
    insertion: Reverse<u64>,
}

impl NativeDrawCell {
    pub(crate) fn scan_row(self) -> i64 {
        self.scan_coordinates().0
    }

    /// Conservative scan-row band including every 24px fragment that can
    /// touch the viewport. Camera/negative division stays in checked i64.
    pub(crate) fn visible_rows(
        view: super::world_map::ViewProjection,
        height: u32,
    ) -> Option<std::ops::RangeInclusive<i64>> {
        let row = Self::from_world(view.anchor_cell)?.scan_row();
        let top = i64::from(view.anchor_pixel.y);
        Some(
            (row + (-top - 24).div_euclid(6) - 2)
                ..=(row + (i64::from(height) - top + 24).div_euclid(6) + 2),
        )
    }
    /// 4A2C10 packet words -> 4E1870 actor+2C/+30. Server X is a
    /// whole-cell coordinate; the native sprite/map grid uses half cells.
    pub fn from_world(cell: super::world_map::WorldCell) -> Option<Self> {
        Some(Self {
            half_x: i32::try_from(2 * i64::from(cell.x) - 0x8000).ok()?,
            y: cell.y,
        })
    }

    /// Native cell origin in the modern world projection. A server cell's
    /// diamond center is 24px right and 12px below its even half-tile origin.
    pub fn screen_origin(
        self,
        view: super::world_map::ViewProjection,
    ) -> Option<super::world_map::PixelPoint> {
        let origin = Self::from_world(view.anchor_cell)?;
        let (row, column) = origin.scan_coordinates();
        let (next_row, next_column) = self.scan_coordinates();
        Some(super::world_map::PixelPoint {
            x: i32::try_from(i64::from(view.anchor_pixel.x) - 24 + 12 * (next_column - column))
                .ok()?,
            y: i32::try_from(i64::from(view.anchor_pixel.y) - 12 + 6 * (next_row - row)).ok()?,
        })
    }
    pub fn fragment(self, reference: SpriteBlockReference) -> Option<Self> {
        Some(Self {
            half_x: self.half_x.checked_add(i32::from(reference.half_x))?,
            y: self.y.checked_add(i32::from(reference.y))?,
        })
    }

    /// Camera-independent scaled coordinates from native 4B8FC0. Using i64
    /// avoids overflow even for malformed/extreme i32 input before a caller's
    /// map/camera qualification. Half-cell parity is significant.
    fn scan_coordinates(self) -> (i64, i64) {
        let half_x = i64::from(self.half_x);
        let y = i64::from(self.y);
        (2 * y - (half_x & !1), ((half_x + 1) & !1) + 2 * y)
    }

    pub fn floor_key(self) -> NativeDrawKey {
        let (row, column) = self.scan_coordinates();
        NativeDrawKey {
            row,
            column,
            depth: -1,
            insertion: Reverse(0),
        }
    }

    pub fn overlay_key(self, depth: u8, insertion: u64) -> NativeDrawKey {
        let (row, column) = self.scan_coordinates();
        NativeDrawKey {
            row,
            column,
            depth: i16::from(depth),
            insertion: Reverse(insertion),
        }
    }

    /// Fragment displacement relative to this native anchor, in 24x12 pixels.
    /// This is NOT the offset between the modern world-cell center and SPR
    /// origin. In particular an odd native anchor changes odd-A placement.
    pub fn fragment_pixel_offset(self, reference: SpriteBlockReference) -> Option<(i64, i64)> {
        let translated = self.fragment(reference)?;
        let (row, column) = self.scan_coordinates();
        let (next_row, next_column) = translated.scan_coordinates();
        Some((12 * (next_column - column), 6 * (next_row - row)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(a: i8, b: i8) -> SpriteBlockReference {
        SpriteBlockReference {
            half_x: a,
            y: b,
            depth: 18,
            block_id: 0,
        }
    }

    #[test]
    fn conservative_row_index_never_discards_a_visible_fragment() {
        use super::super::world_map::{PixelPoint, ViewProjection, WorldCell};
        for height in [1, 24, 600, 4096] {
            for anchor_y in [-700, 0, 300, 5000] {
                let view = ViewProjection {
                    anchor_cell: WorldCell { x: 32724, y: 32784 },
                    anchor_pixel: PixelPoint {
                        x: 400,
                        y: anchor_y,
                    },
                };
                let rows = NativeDrawCell::visible_rows(view, height).unwrap();
                let origin = NativeDrawCell::from_world(view.anchor_cell).unwrap();
                for dx in -128..=128 {
                    for dy in -512..=512 {
                        let cell = NativeDrawCell {
                            half_x: origin.half_x + dx,
                            y: origin.y + dy,
                        };
                        let p = cell.screen_origin(view).unwrap();
                        if p.y < i32::try_from(height).unwrap() && p.y + 24 > 0 {
                            assert!(rows.contains(&cell.scan_row()));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn packet_whole_cells_and_map_half_cells_share_the_world_projection() {
        use super::super::world_map::{PixelPoint, ViewProjection, WorldCell};
        for anchor in [
            WorldCell { x: 32768, y: 32768 },
            WorldCell { x: 32724, y: 32784 },
            WorldCell { x: -1, y: -2 },
        ] {
            let view = ViewProjection {
                anchor_cell: anchor,
                anchor_pixel: PixelPoint { x: 400, y: 300 },
            };
            for dx in -70..=70 {
                for dy in -70..=70 {
                    let world = WorldCell {
                        x: anchor.x + dx,
                        y: anchor.y + dy,
                    };
                    let native = NativeDrawCell::from_world(world).unwrap();
                    assert_eq!(native.half_x, world.x * 2 - 32768);
                    let center = view.cell_to_pixel(world).unwrap();
                    assert_eq!(
                        native.screen_origin(view),
                        Some(PixelPoint {
                            x: center.x - 24,
                            y: center.y - 12
                        })
                    );
                    assert_eq!(
                        NativeDrawCell {
                            half_x: native.half_x + 1,
                            ..native
                        }
                        .screen_origin(view),
                        Some(PixelPoint {
                            x: center.x,
                            y: center.y - 12
                        })
                    );
                }
            }
        }
        assert!(NativeDrawCell::from_world(WorldCell { x: i32::MAX, y: 0 }).is_none());
        assert!(NativeDrawCell::from_world(WorldCell { x: i32::MIN, y: 0 }).is_none());
    }

    #[test]
    fn all_signed_source_offsets_match_even_anchor_projection_but_odd_anchors_differ() {
        let even = NativeDrawCell {
            half_x: 32768,
            y: 32768,
        };
        let odd = NativeDrawCell {
            half_x: 32769,
            ..even
        };
        for a in i8::MIN..=i8::MAX {
            for b in i8::MIN..=i8::MAX {
                // Independent arithmetic form used by the reference SPR
                // inspector; not a call back into scan_coordinates.
                let half = i64::from(a).div_euclid(2);
                let expected = (
                    24 * (i64::from(b) + i64::from(a) - half),
                    12 * (i64::from(b) - half),
                );
                assert_eq!(even.fragment_pixel_offset(reference(a, b)), Some(expected));
                let odd_expected = if a & 1 == 0 {
                    expected
                } else {
                    (expected.0 - 24, expected.1 - 12)
                };
                assert_eq!(
                    odd.fragment_pixel_offset(reference(a, b)),
                    Some(odd_expected)
                );
            }
        }
    }

    #[test]
    fn native_loop_scan_beats_global_layer_and_actor_anchor_sorting() {
        // Execute the parity changes in 488EBE/488F03, not an inferred actor
        // screen-Y order. It is the tile-fragment cell that controls this scan.
        for start_half_x in [-3, -2, 0, 1, 32768, 32769] {
            let mut row_start = NativeDrawCell {
                half_x: start_half_x,
                y: 0,
            };
            let mut scan = Vec::new();
            for _ in 0..12 {
                let mut cell = row_start;
                for _ in 0..20 {
                    scan.push(cell);
                    if cell.half_x & 1 != 0 {
                        cell.y += 1;
                    }
                    cell.half_x += 1;
                }
                if row_start.half_x & 1 != 0 {
                    row_start.y += 1;
                }
                row_start.half_x -= 1;
            }
            assert!(scan
                .windows(2)
                .all(|pair| pair[0].overlay_key(255, 0) < pair[1].floor_key()));
        }
    }

    #[test]
    fn newest_equal_depth_insertion_is_painted_first_and_floor_precedes_every_overlay() {
        let cell = NativeDrawCell { half_x: 0, y: 0 };
        let inputs = [(18, 1), (7, 2), (18, 3), (255, 4), (0, 5), (18, 6)];
        // Literal insertion contract of 483815, independent of derived keys.
        let mut native = Vec::new();
        for item in inputs {
            let position = native
                .iter()
                .position(|old: &(u8, u64)| item.0 <= old.0)
                .unwrap_or(native.len());
            native.insert(position, item);
        }
        let mut ordered = inputs;
        ordered.sort_by_key(|&(depth, insertion)| cell.overlay_key(depth, insertion));
        assert_eq!(ordered.as_slice(), native);
        assert_eq!(
            ordered,
            [(0, 5), (7, 2), (18, 6), (18, 3), (18, 1), (255, 4)]
        );
        assert!(cell.floor_key() < cell.overlay_key(0, u64::MAX));
    }

    #[test]
    fn overflow_is_rejected_and_extreme_keys_do_not_wrap() {
        for (cell, block) in [
            (
                NativeDrawCell {
                    half_x: i32::MAX,
                    y: 0,
                },
                reference(1, 0),
            ),
            (
                NativeDrawCell {
                    half_x: i32::MIN,
                    y: 0,
                },
                reference(-1, 0),
            ),
            (
                NativeDrawCell {
                    half_x: 0,
                    y: i32::MAX,
                },
                reference(0, 1),
            ),
            (
                NativeDrawCell {
                    half_x: 0,
                    y: i32::MIN,
                },
                reference(0, -1),
            ),
        ] {
            assert_eq!(cell.fragment(block), None);
            assert_eq!(cell.fragment_pixel_offset(block), None);
        }
        let rear = NativeDrawCell {
            half_x: i32::MAX,
            y: i32::MIN,
        };
        let front = NativeDrawCell {
            half_x: i32::MIN,
            y: i32::MAX,
        };
        assert!(rear.overlay_key(255, 0) < front.floor_key());
    }
}
