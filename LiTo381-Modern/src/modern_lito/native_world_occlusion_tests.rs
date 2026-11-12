//! G07.3 (T058 1/2): terrain, sector seams, occlusion and roof entry/exit on
//! the current formal path, without client archives. The S32 and TIL bytes are
//! authored here; everything after the byte boundary is production code:
//! `WorldMap`, the static reference/group collection, `Scene`, the Layer5 roof
//! runtime, `World` wire packets with predicted steps, and the layered raster.
//! Synthetic sources prove rules, not that any shipped map renders correctly.
use super::{
    native_multisector_scene::Scene,
    native_resources::NativeStaticAssets,
    native_world::World,
    native_world_layers::{self, Body, LayeredBody},
    native_world_roofs::{self, Runtime},
    resources::SharedMapSectorKey,
    sprite::{CompactSpriteBlock, SpriteBlockReference, SpriteFrame, SpriteFrameFragments},
    world_depth::NativeDrawCell,
    world_map::{PixelPoint, SectorInput, ViewProjection, WorldCell, WorldMap},
};
use std::{
    collections::BTreeMap,
    sync::{atomic::AtomicBool, Arc},
};

const MAP: u32 = 564;
const WEST: (u16, u16) = (0x7fff, 0x8000);
const EAST: (u16, u16) = (0x8000, 0x8000);
/// Walked row. `P[2]` is the last cell of the west sector, `P[3]` the first of the east.
const ROW: u16 = 32778;
pub(super) const P: [u16; 6] = [32765, 32766, 32767, 32768, 32769, 32770];
/// RGB555 components.
pub(super) const FLOOR: [u8; 3] = [4, 20, 4];
const RED: [u8; 3] = [28, 4, 4];
const BLUE: [u8; 3] = [4, 4, 28];
const BODY: [u8; 3] = [30, 30, 2];

fn word(c: [u8; 3]) -> [u8; 2] {
    ((u16::from(c[0]) << 10) | (u16::from(c[1]) << 5) | u16::from(c[2])).to_le_bytes()
}
pub(super) fn rgb(c: [u8; 3]) -> [u8; 3] {
    c.map(|v| (v << 3) | (v >> 2))
}
fn til(blocks: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = (blocks.len() as u32).to_le_bytes().to_vec();
    let mut offset = 0u32;
    for block in blocks {
        bytes.extend(offset.to_le_bytes());
        offset += block.len() as u32;
    }
    bytes.extend(offset.to_le_bytes());
    for block in blocks {
        bytes.extend(block);
    }
    bytes
}
fn diamond(color: [u8; 3]) -> Vec<u8> {
    let mut block = vec![0u8];
    for _ in 0..288 {
        block.extend(word(color));
    }
    block
}
/// Full 24x24 RLE object block: one 24-pixel segment per row.
fn object(kind: u8, color: [u8; 3]) -> Vec<u8> {
    let mut block = vec![kind, 0, 0, 24, 24];
    for _ in 0..24 {
        block.extend([1u8, 0, 24]);
        for _ in 0..24 {
            block.extend(word(color));
        }
    }
    block
}
fn tiles() -> BTreeMap<u16, Vec<u8>> {
    BTreeMap::from([
        (1, til(&[diamond(FLOOR)])),
        (2, til(&[object(3, RED), object(2, BLUE)])),
    ])
}
/// `(serialized id, [(x, y, depth, subtile)])` Layer4 groups of tile 2 and
/// `[x, y, group ordinal, kind]` Layer5 records.
type Groups<'a> = &'a [(i16, &'a [[u8; 4]])];
fn s32(groups: Groups<'_>, layer5: &[[u16; 4]]) -> Vec<u8> {
    let mut raw = Vec::new();
    for _ in 0..128 * 64 {
        // subtile 0 of 1.til, the only floor source.
        raw.extend([0u8, 1, 0, 0]);
    }
    raw.extend(0u16.to_le_bytes());
    raw.extend(std::iter::repeat_n(0u8, 64 * 64 * 4));
    raw.extend((groups.len() as i32).to_le_bytes());
    for (id, members) in groups {
        raw.extend(id.to_le_bytes());
        raw.extend((members.len() as u16).to_le_bytes());
        for [x, y, depth, subtile] in *members {
            raw.extend([*x, *y, *depth, *subtile]);
            raw.extend(2i16.to_le_bytes());
            raw.push(0);
        }
    }
    raw.extend((layer5.len() as i32).to_le_bytes());
    for [x, y, group, kind] in layer5 {
        raw.extend([*x as u8, *y as u8]);
        raw.extend(group.to_le_bytes());
        raw.push(*kind as u8);
    }
    raw
}
fn map(west: Vec<u8>, east: Vec<u8>) -> WorldMap {
    let input = |(sector_x, sector_y): (u16, u16), bytes| SectorInput {
        key: SharedMapSectorKey {
            map_id: MAP,
            sector_x,
            sector_y,
        },
        bytes,
    };
    WorldMap::from_native_sector_bytes(
        vec![input(WEST, west), input(EAST, east)],
        &AtomicBool::new(false),
    )
    .unwrap()
}
/// West ordinal 0 keeps serialized id 70 and an unrelated tile; ordinal 1
/// (serialized -7) is the roof over `P[2]` whose second tile is authored at
/// local x 128, i.e. inside the east sector over `P[3]`. Its off-sector
/// Layer5 record (x 130 would wrap to `P[4]`) must stay ignored. East ordinal
/// 0 is a hidden-kind group over `P[4]`; ordinal 1 is an uncontrolled wall
/// over `P[5]`.
fn west_sector() -> Vec<u8> {
    s32(
        &[
            (70, &[[10, 40, 20, 1]]),
            (-7, &[[126, 10, 20, 0], [128, 10, 20, 0]]),
        ],
        &[[124, 10, 1, 0], [126, 10, 1, 0], [130, 10, 1, 0]],
    )
}
fn east_sector() -> Vec<u8> {
    s32(
        &[(-7, &[[2, 10, 20, 1]]), (70, &[[4, 10, 20, 0]])],
        &[[2, 10, 0, 1]],
    )
}
fn scene_from(map: WorldMap, resident: Option<[u16; 2]>) -> Scene {
    let cancel = AtomicBool::new(false);
    Scene::new_cancel(
        NativeStaticAssets::from_owned_sources_for_test(map, tiles(), resident, &cancel).unwrap(),
        &cancel,
    )
    .unwrap()
}
pub(super) fn scene() -> Scene {
    scene_from(map(west_sector(), east_sector()), None)
}
pub(super) fn cell(x: u16) -> WorldCell {
    WorldCell {
        x: i32::from(x),
        y: i32::from(ROW),
    }
}
/// Fixed inspection view: `P[3]` at the centre of a 320x240 surface.
pub(super) fn view() -> ViewProjection {
    ViewProjection {
        anchor_cell: cell(P[3]),
        anchor_pixel: PixelPoint { x: 160, y: 120 },
    }
}
/// A point covered by the cell's own even half-cell stack only: the next scan
/// row starts 12 pixels lower, earlier rows are painted underneath.
pub(super) fn sample(x: u16) -> (i64, i64) {
    let origin = NativeDrawCell::from_world(cell(x))
        .unwrap()
        .screen_origin(view())
        .unwrap();
    (i64::from(origin.x) + 12, i64::from(origin.y) + 6)
}
fn body_frame() -> SpriteFrame {
    let [r, g, b] = rgb(BODY);
    let rgba = [r, g, b, 255].repeat(576);
    SpriteFrame {
        fragments: Some(SpriteFrameFragments {
            references: vec![SpriteBlockReference {
                half_x: 0,
                y: 0,
                depth: 10,
                block_id: 0,
            }],
            atlas: Arc::new(vec![CompactSpriteBlock::test_rgba(&rgba)]),
        }),
        x_offset: 0,
        y_offset: 0,
        width: 24,
        height: 24,
        frame_type: 0,
        unknown1: 0,
        unknown2: 0,
        rgba,
    }
}
struct Shot {
    output: native_world_layers::Output,
}
impl Shot {
    fn take(scene: &Scene, roofs: &native_world_roofs::Frame, own: Option<WorldCell>) -> Self {
        let frame = body_frame();
        let bodies: Vec<_> = own
            .map(|own| LayeredBody {
                body: Body {
                    id: 7,
                    center: view().cell_to_pixel(own).unwrap(),
                    frame: &frame,
                },
                attachments: Vec::new(),
                shadow: None,
            })
            .into_iter()
            .collect();
        Self {
            output: native_world_layers::compose_with_roofs(
                scene,
                view(),
                320,
                240,
                &bodies,
                roofs,
                &AtomicBool::new(false),
            )
            .unwrap(),
        }
    }
    fn rgb(&self, x: u16) -> [u8; 3] {
        let (px, py) = sample(x);
        let at = (py as usize * 320 + px as usize) * 4;
        self.output.canvas.rgba()[at..at + 3].try_into().unwrap()
    }
    /// The body owns the sampled pixel in the shared hit surface.
    fn body_owns(&self, x: u16) -> bool {
        let (px, py) = sample(x);
        self.output.visibility[0].1.contains(px, py)
    }
}
fn half(source: [u8; 3], under: [u8; 3]) -> [u8; 3] {
    rgb(std::array::from_fn(|c| (source[c] + under[c]) / 2))
}
fn weighted(source: [u8; 3], under: [u8; 3], weight: u16) -> [u8; 3] {
    rgb(std::array::from_fn(|c| {
        ((u16::from(source[c]) * weight + u16::from(under[c]) * (31 - weight)) / 31) as u8
    }))
}

fn packets() -> serde_json::Value {
    serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-world-current.json"
    ))
    .unwrap()
}
fn packet(name: &str) -> Vec<u8> {
    let all = packets();
    let hex = all["packets"][name].as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
/// The recorded own S_PUT_OBJECT with only its coordinate words replaced.
fn own_put(x: u16) -> Vec<u8> {
    let mut body = packet("actor");
    body[1..3].copy_from_slice(&x.to_le_bytes());
    body[3..5].copy_from_slice(&ROW.to_le_bytes());
    body
}
/// Recorded S_WORLD(564), S_START and S_STATUS, then the own actor at `x`.
fn world_at(x: u16) -> World {
    let mut world = World::default();
    for name in ["world", "start", "status"] {
        assert!(world.accept(&packet(name)).unwrap(), "{name}");
    }
    assert!(world.accept(&own_put(x)).unwrap());
    world.accept(&packet("hp")).unwrap();
    assert_eq!(world.map_id.map(u32::from), Some(MAP));
    assert!(world.ready());
    world
}
const EAST_HEADING: u8 = 2;
fn ms(value: u64) -> u64 {
    value * 1_000_000
}
/// `normal` shade of one controlled group in a frame receipt.
fn shade(frame: &native_world_roofs::Frame, sector: (u16, u16), ordinal: u16) -> serde_json::Value {
    frame.receipt()["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["sector"] == serde_json::json!([sector.0, sector.1]) && g["ordinal"] == ordinal)
        .map_or(serde_json::json!("Native"), |g| g["normal"].clone())
}
fn weight(value: u8) -> serde_json::Value {
    serde_json::json!({ "Weight": value })
}

#[test]
fn seam_roof_groups_keep_sector_ordinals_and_reject_wrapped_layer5_records() {
    let scene = scene();
    let definition = scene.roofs().receipt();
    assert_eq!(definition["trigger_cells"], 3);
    assert_eq!(definition["controlled_groups"], 2);
    assert_eq!(definition["ignored_outside_sector"], 1);
    // 8192 floors per sector plus 3 west and 2 east Layer4 references.
    assert_eq!(scene.assets().draw_references(), 2 * 8192 + 5);
    let none = Default::default();
    let shot = Shot::take(&scene, &none, None);
    assert_eq!(shot.rgb(P[0]), rgb(FLOOR));
    assert_eq!(shot.rgb(P[1]), rgb(FLOOR));
    assert_eq!(shot.rgb(P[2]), rgb(RED));
    assert_eq!(shot.rgb(P[3]), rgb(RED));
    assert_eq!(shot.rgb(P[4]), rgb(BLUE));
    assert_eq!(shot.rgb(P[5]), rgb(RED));
    // The same west tile authored inside the east sector lands on the same
    // native half-cell: identical pixels, but then it is an east group.
    let moved = scene_from(
        map(
            s32(
                &[(70, &[[10, 40, 20, 1]]), (-7, &[[126, 10, 20, 0]])],
                &[[124, 10, 1, 0], [126, 10, 1, 0], [130, 10, 1, 0]],
            ),
            s32(
                &[
                    (-7, &[[2, 10, 20, 1]]),
                    (70, &[[4, 10, 20, 0]]),
                    (5, &[[0, 10, 20, 0]]),
                ],
                &[[2, 10, 0, 1]],
            ),
        ),
        None,
    );
    assert_eq!(
        Shot::take(&moved, &none, None).output.canvas.rgba(),
        shot.output.canvas.rgba()
    );
    let mut runtime = Runtime::default();
    let inside = runtime
        .sample(moved.roofs(), [1, 2, 3], cell(P[1]), ms(0))
        .unwrap();
    let settled = runtime
        .sample(moved.roofs(), [1, 2, 3], cell(P[1]), ms(200))
        .unwrap();
    assert_eq!(shade(&inside, WEST, 1), weight(31));
    assert_eq!(shade(&settled, WEST, 1), "Half");
    let moved_shot = Shot::take(&moved, &settled, None);
    assert_eq!(moved_shot.rgb(P[2]), half(RED, FLOOR));
    // The east copy is not a member of the west group and stays opaque.
    assert_eq!(moved_shot.rgb(P[3]), rgb(RED));
    // A frame sampled for one geometry is refused by another.
    assert!(native_world_layers::compose_with_roofs(
        &scene,
        view(),
        320,
        240,
        &[],
        &settled,
        &AtomicBool::new(false)
    )
    .is_err());
    // Layer5 may not address a group ordinal the sector does not have, nor an
    // unqualified kind, when the record is inside the sector.
    for record in [[2, 10, 2, 1], [2, 10, 0, 2]] {
        let cancel = AtomicBool::new(false);
        let bad = map(west_sector(), s32(&[(-7, &[[2, 10, 20, 1]]), (70, &[[4, 10, 20, 0]])], &[record]));
        let assets =
            NativeStaticAssets::from_owned_sources_for_test(bad, tiles(), None, &cancel).unwrap();
        assert!(Scene::new_cancel(assets, &cancel).is_err());
    }
}

#[test]
fn wire_route_enters_and_leaves_roofs_across_the_sector_seam() {
    let scene = scene();
    let roofs = scene.roofs();
    let mut runtime = Runtime::default();
    let mut world = world_at(P[0]);
    let at = |world: &World, runtime: &mut Runtime, elapsed_ms: u64| {
        let before = serde_json::to_value(world).unwrap();
        let (identity, cell) = native_world_roofs::scope(world).unwrap();
        let frame = runtime.sample(roofs, identity, cell, ms(elapsed_ms)).unwrap();
        let shot = Shot::take(&scene, &frame, Some(cell));
        // Presentation never writes world authority.
        assert_eq!(before, serde_json::to_value(world).unwrap());
        (frame, shot, cell)
    };

    // Outside every trigger: all groups are source-opaque, the body is exposed.
    let (frame, shot, own) = at(&world, &mut runtime, 0);
    assert_eq!(own, cell(P[0]));
    assert!(frame.receipt()["groups"].as_array().unwrap().is_empty());
    assert_eq!(shot.rgb(P[0]), rgb(BODY));
    assert!(shot.body_owns(P[0]));
    assert_eq!(
        [shot.rgb(P[2]), shot.rgb(P[3]), shot.rgb(P[4]), shot.rgb(P[5])],
        [rgb(RED), rgb(RED), rgb(BLUE), rgb(RED)]
    );

    // Predicted step onto the first west trigger: the 50 ms / step 4 ramp starts
    // from weight 31 and settles on the stable half average. The roof tile that
    // the west group owns inside the east sector follows the same group.
    world.predict_step(EAST_HEADING).unwrap();
    let (frame, _, own) = at(&world, &mut runtime, 100);
    assert_eq!(own, cell(P[1]));
    assert_eq!(shade(&frame, WEST, 1), weight(31));
    assert_eq!(shade(&frame, WEST, 0), "Native");
    assert_eq!(shade(&at(&world, &mut runtime, 150).0, WEST, 1), weight(27));
    let (frame, shot, _) = at(&world, &mut runtime, 300);
    assert_eq!(shade(&frame, WEST, 1), "Half");
    assert_eq!(shade(&frame, EAST, 0), "Native");
    assert_eq!(shot.rgb(P[2]), half(RED, FLOOR));
    assert_eq!(shot.rgb(P[3]), half(RED, FLOOR));
    assert_eq!(shot.rgb(P[4]), rgb(BLUE));
    assert_eq!(shot.rgb(P[5]), rgb(RED));

    // Second trigger of the same group: no restart; the body under the half
    // roof keeps the pixel in the hit surface.
    world.predict_step(EAST_HEADING).unwrap();
    let (frame, shot, own) = at(&world, &mut runtime, 320);
    assert_eq!(own, cell(P[2]));
    assert_eq!(shade(&frame, WEST, 1), "Half");
    assert_eq!(shot.rgb(P[2]), half(RED, BODY));
    assert!(shot.body_owns(P[2]));

    // Crossing the seam leaves the west triggers. The body now stands under
    // the west group's tile in the east sector while it returns: weight 17
    // first, then source-opaque, which takes the pixel away from the body.
    world.predict_step(EAST_HEADING).unwrap();
    let (frame, shot, own) = at(&world, &mut runtime, 400);
    assert_eq!(own, cell(P[3]));
    assert_eq!(shade(&frame, WEST, 1), weight(17));
    assert_eq!(shot.rgb(P[3]), weighted(RED, BODY, 17));
    assert!(shot.body_owns(P[3]));
    assert_eq!(shot.rgb(P[2]), weighted(RED, FLOOR, 17));
    let (frame, shot, _) = at(&world, &mut runtime, 600);
    assert!(frame.receipt()["groups"].as_array().unwrap().is_empty());
    assert_eq!(shot.rgb(P[3]), rgb(RED));
    assert!(!shot.body_owns(P[3]));
    assert_eq!(shot.rgb(P[2]), rgb(RED));

    // East hidden-kind trigger. The west group's off-sector record for this
    // very cell is not applied.
    world.predict_step(EAST_HEADING).unwrap();
    let (frame, _, own) = at(&world, &mut runtime, 700);
    assert_eq!(own, cell(P[4]));
    assert_eq!(shade(&frame, EAST, 0), weight(31));
    assert_eq!(shade(&frame, WEST, 1), "Native");
    assert_eq!(shade(&at(&world, &mut runtime, 1050).0, EAST, 0), weight(3));
    let (frame, shot, _) = at(&world, &mut runtime, 1100);
    assert_eq!(shade(&frame, EAST, 0), "Hidden");
    assert_eq!(shot.rgb(P[4]), rgb(BODY));
    assert!(shot.body_owns(P[4]));
    assert_eq!(shot.rgb(P[3]), rgb(RED));
    assert_eq!(shot.rgb(P[5]), rgb(RED));

    // Leaving it: the hidden group reappears from weight 1, and the
    // uncontrolled wall over the new cell occludes the body throughout.
    world.predict_step(EAST_HEADING).unwrap();
    let (frame, shot, own) = at(&world, &mut runtime, 1150);
    assert_eq!(own, cell(P[5]));
    assert_eq!(shade(&frame, EAST, 0), weight(1));
    assert_eq!(shot.rgb(P[4]), weighted(BLUE, FLOOR, 1));
    assert_eq!(shot.rgb(P[5]), rgb(RED));
    assert!(!shot.body_owns(P[5]));
    let (frame, shot, _) = at(&world, &mut runtime, 1600);
    assert!(frame.receipt()["groups"].as_array().unwrap().is_empty());
    assert_eq!(shot.rgb(P[4]), rgb(BLUE));
    assert_eq!(world.predicted_moves, 5);

    // A new own S_PUT_OBJECT is a new incarnation: the trigger it lands on
    // starts a fresh ramp instead of inheriting an older one.
    let previous = native_world_roofs::scope(&world).unwrap().0;
    assert!(world.accept(&own_put(P[4])).unwrap());
    let next = native_world_roofs::scope(&world).unwrap().0;
    assert_eq!(previous[..2], next[..2]);
    assert_ne!(previous[2], next[2]);
    let (frame, _, own) = at(&world, &mut runtime, 1700);
    assert_eq!(own, cell(P[4]));
    assert_eq!(shade(&frame, EAST, 0), weight(31));
    // The elapsed clock is monotonic for the owning runtime.
    let (identity, own) = native_world_roofs::scope(&world).unwrap();
    assert!(runtime.sample(roofs, identity, own, ms(1699)).is_err());

    // An observer without a ready view has no roof scope at all.
    world.set_observer_mode(true);
    assert!(native_world_roofs::scope(&world).is_err());
}

#[test]
fn resident_terrain_window_matches_the_whole_map_for_every_camera_of_its_block() {
    let whole = scene();
    // The 8x8 block that contains the last west column next to the seam.
    let block = [P[2] & !7, ROW & !7];
    let window = scene_from(map(west_sector(), east_sector()), Some([P[2], ROW]));
    assert!(window.assets().draw_references() < whole.assets().draw_references() / 2);
    assert_eq!(window.assets().resident_sector_count(), 2);
    assert_eq!(window.anchor(), whole.anchor());
    let anchor = whole.anchor();
    let none = Default::default();
    let cancel = AtomicBool::new(false);
    for (dx, dy) in [(0, 0), (7, 0), (0, 7), (7, 7), (3, 4)] {
        // The world presenter's camera: the own cell at the centre of 800x600.
        let (x, y) = (
            i32::from(block[0]) + dx - anchor.x,
            i32::from(block[1]) + dy - anchor.y,
        );
        let view = ViewProjection {
            anchor_cell: anchor,
            anchor_pixel: PixelPoint {
                x: 400 - 24 * (x + y),
                y: 300 - 12 * (-x + y),
            },
        };
        let compose = |scene: &Scene| {
            native_world_layers::compose_with_roofs(scene, view, 800, 600, &[], &none, &cancel)
                .unwrap()
        };
        let (full, partial) = (compose(&whole), compose(&window));
        assert!(full.terrain_fragments > 300, "{dx} {dy}");
        assert_eq!(full.terrain_fragments, partial.terrain_fragments, "{dx} {dy}");
        assert!(full.canvas.rgba() == partial.canvas.rgba(), "{dx} {dy}");
    }
    // A view cell outside the source geometry is rejected, not clamped.
    assert!(NativeStaticAssets::from_owned_sources_for_test(
        map(west_sector(), east_sector()),
        tiles(),
        Some([P[2], 32768 + 64]),
        &cancel
    )
    .is_err());
}
