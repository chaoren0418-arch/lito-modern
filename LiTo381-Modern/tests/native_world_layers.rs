use lito_modern::modern_lito::{
    native_world::World,
    native_world_layers::{self, Body},
    native_world_scene::Model,
    resources::{SharedArchive, SharedArchiveKind},
    sprite,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::atomic::AtomicBool};
fn world() -> World {
    let j: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for k in ["world", "start", "status", "actor", "bot"] {
        let h = j["packets"][k].as_str().unwrap();
        let b: Vec<_> = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect();
        w.accept(&b).unwrap();
    }
    w
}
fn model() -> Model {
    Model::load(
        Path::new(r"D:\lineage開發\0-客戶端"),
        &world(),
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn raw(name: &str) -> Vec<u8> {
    SharedArchive::open(
        Path::new(r"D:\lineage開發\0-客戶端"),
        SharedArchiveKind::Sprite,
    )
    .unwrap()
    .read_entry(name)
    .unwrap()
}
#[path = "support/native_fragment_reference.rs"]
mod fragment_reference;
use fragment_reference::{reference, reference_inner};
fn verify(
    out: &native_world_layers::Output,
    expected: &(Vec<u8>, Vec<u32>),
    width: usize,
    height: usize,
) {
    assert_eq!(out.canvas.rgba(), expected.0);
    for (id, mask) in &out.visibility {
        for y in 0..height {
            for x in 0..width {
                assert_eq!(
                    mask.contains(x as i64, y as i64),
                    expected.1[y * width + x] == *id,
                    "owner{id} at{x},{y}"
                );
            }
        }
        assert!(!mask.contains(-1, 0));
        assert!(!mask.contains(width as i64, 0));
    }
}
fn png(path: &Path, rgba: &[u8], w: u32, h: u32) {
    let f = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(f, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(rgba).unwrap();
}
#[test]
fn actual_terrain_body_and_ground_fragments_match_independent_pixels_and_owners() {
    let m = fragment_reference::full_native_fixture(Path::new(r"D:\lineage開發\0-客戶端"), u32::from(world().map_id.unwrap()));
    let scene = m.scene();
    let cancel = AtomicBool::new(false);
    let raws = [raw("3225-0.spr"), raw("773-0.spr")];
    let compact: Vec<_> = raws
        .iter()
        .map(|b| sprite::decode_world_spr_with_limits(b, Default::default()).unwrap())
        .collect();
    let layered: Vec<_> = raws
        .iter()
        .map(|b| sprite::decode_classic_spr_layered(b, Default::default()).unwrap())
        .collect();
    let mut rows = Vec::new();
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    for (index, (wx, wy, dx, dy)) in [
        (32729, 32797, 0, 0),
        (32720, 32790, 0, 0),
        (32727, 32794, 0, 0),
        (32729, 32797, 11, 5),
        (32729, 32797, -13, -7),
        (32729, 32797, 24, 12),
        (32729, 32797, -24, -12),
        (32768, 32768, 0, 0),
    ]
    .into_iter()
    .enumerate()
    {
        let view = ViewProjection {
            anchor_cell: WorldCell { x: wx, y: wy },
            anchor_pixel: PixelPoint { x: 160, y: 120 },
        };
        let center = PixelPoint {
            x: 160 + dx,
            y: 120 + dy,
        };
        let item = PixelPoint { x: 184, y: 132 };
        let bodies = [
            Body {
                id: 10,
                center,
                frame: &compact[0].frames[0],
            },
            Body {
                id: 20,
                center: item,
                frame: &compact[1].frames[0],
            },
        ];
        let r = native_world_layers::compose(scene, view, 320, 240, &bodies, &cancel).unwrap();
        let expected = reference(
            scene,
            view,
            &[(10, center, &layered[0], 0), (20, item, &layered[1], 0)],
            320,
            240,
        );
        verify(&r, &expected, 320, 240);
        let unoccluded = reference_inner(
            scene,
            view,
            &[(10, center, &layered[0], 0), (20, item, &layered[1], 0)],
            320,
            240,
            false,
        );
        let occluded = unoccluded
            .1
            .iter()
            .zip(&expected.1)
            .filter(|(before, after)| **before == 10 && **after != 10)
            .count();
        if index == 1 {
            assert!(
                occluded > 0 && expected.1.contains(&10),
                "actual wall must partially occlude body"
            );
        }

        let file = out.join(format!("native-layer-reference-{index}.png"));
        png(&file, r.canvas.rgba(), 320, 240);
        rows.push(serde_json::json!({"world":[wx,wy],"residual":[dx,dy],"png":file,"rgba_sha256":format!("{:x}",Sha256::digest(r.canvas.rgba())),"visible_actor_pixels":expected.1.iter().filter(|v|**v==10).count(),"visible_item_pixels":expected.1.iter().filter(|v|**v==20).count(),"terrain_occluded_actor_pixels":occluded,"terrain_fragments":r.terrain_fragments}));
    }
    std::fs::write(out.join("native-layer-reference.json"),serde_json::to_vec_pretty(&serde_json::json!({"cases":rows,"pixel_comparisons":8,"independent_composition":true,"decoder_independent":false,"source_raw_sha256":raws.iter().map(|v|format!("{:x}",Sha256::digest(v))).collect::<Vec<_>>(),"native_runtime_insertion_equivalent":false,"gui_tested":false})).unwrap()).unwrap();
}
#[test]
fn same_depth_overlap_preserves_reverse_insertion_and_transparent_holes() {
    let m = fragment_reference::full_native_fixture(Path::new(r"D:\lineage開發\0-客戶端"), u32::from(world().map_id.unwrap()));
    let data = raw("773-0.spr");
    let compact = sprite::decode_world_spr_with_limits(&data, Default::default()).unwrap();
    let layered = sprite::decode_classic_spr_layered(&data, Default::default()).unwrap();
    let view = ViewProjection {
        anchor_cell: WorldCell { x: 32729, y: 32797 },
        anchor_pixel: PixelPoint { x: 80, y: 60 },
    };
    let a = Body {
        id: 10,
        center: view.anchor_pixel,
        frame: &compact.frames[0],
    };
    let b = Body {
        id: 20,
        center: view.anchor_pixel,
        frame: &compact.frames[0],
    };
    let out =
        native_world_layers::compose(m.scene(), view, 160, 120, &[a, b], &AtomicBool::new(false))
            .unwrap();
    let expected = reference(
        m.scene(),
        view,
        &[
            (10, view.anchor_pixel, &layered, 0),
            (20, view.anchor_pixel, &layered, 0),
        ],
        160,
        120,
    );
    verify(&out, &expected, 160, 120);
    assert!(expected.1.contains(&10));
    assert!(!expected.1.contains(&20));
    assert!(expected.1.contains(&0));
}
#[test]
fn scene_composition_rejects_invalid_identity_budget_and_unqualified_sprites() {
    let m = fragment_reference::full_native_fixture(Path::new(r"D:\lineage開發\0-客戶端"), u32::from(world().map_id.unwrap()));
    let data = raw("773-0.spr");
    let frame = sprite::decode_classic_spr(&data).unwrap();
    let view = ViewProjection {
        anchor_cell: WorldCell { x: 32729, y: 32797 },
        anchor_pixel: PixelPoint { x: 80, y: 60 },
    };
    let body = || Body {
        id: 10,
        center: view.anchor_pixel,
        frame: &frame.frames[0],
    };
    let cancel = AtomicBool::new(false);
    assert!(native_world_layers::compose(m.scene(), view, 160, 120, &[body()], &cancel).is_err());
    assert!(
        native_world_layers::compose(m.scene(), view, 160, 120, &[], &AtomicBool::new(true))
            .is_err()
    );
    assert!(native_world_layers::compose(m.scene(), view, 0, 120, &[], &cancel).is_err());
    assert!(native_world_layers::compose(m.scene(), view, 1025, 120, &[], &cancel).is_err());
    assert!(
        native_world_layers::compose(m.scene(), view, 160, 120, &[body(), body()], &cancel)
            .is_err()
    );
    let too_many: Vec<_> = (1..=129)
        .map(|id| Body {
            id,
            center: view.anchor_pixel,
            frame: &frame.frames[0],
        })
        .collect();
    assert!(native_world_layers::compose(m.scene(), view, 160, 120, &too_many, &cancel).is_err());
}
#[test]
fn pixel_pick_preserves_authority_and_stale_frame_fences() {
    let mut w = world();
    let id = w.own_actor().unwrap().id;
    let mut m = model();
    let cancel = AtomicBool::new(false);
    let mut elapsed = 0;
    let mut covered = 0;
    let mut visible = 0;
    for (x, y) in [
        (32729, 32797),
        (32720, 32790),
        (32725, 32786),
        (32730, 32788),
        (32716, 32783),
        (32717, 32782),
    ] {
        let a = w.actors.get_mut(&id).unwrap();
        a.x = x;
        a.y = y;
        a.motion_revision += 1;
        w.visual_revision += 1;
        m.draw_at(&w, &cancel, elapsed).unwrap();
        elapsed += 2_000_000_000;
        let f = m.draw_at(&w, &cancel, elapsed).unwrap();
        assert!(f.input_ready(&w));
        let r = f.hits.iter().find(|(i, _)| *i == id).unwrap().1;
        let own = &f.visibility.iter().find(|(i, _)| *i == id).unwrap().1;
        for y in r[1].max(0)..r[3].min(600) {
            for x in r[0].max(0)..r[2].min(800) {
                if own.contains(x.into(), y.into()) {
                    visible += 1;
                    assert_ne!(f.pick(x, y, id), Some(id));
                } else {
                    covered += 1;
                }
            }
        }
        w.visual_revision += 1;
        assert!(!f.input_ready(&w));
        assert!(f.pick_current(&w, 400, 300, id).is_none());
    }
    assert!(visible > 0 && covered > 0);
}

#[test]
fn ground_pointer_uses_visible_pixels_and_never_rebinds_a_stale_item() {
    use lito_modern::modern_lito::{
        native_ground_pickup::Choice,
        native_login::{LoginController, LoginPhase},
    };
    let mut s = LoginController::default().snapshot().clone();
    s.world = world();
    s.phase = LoginPhase::BattlePending;
    s.game_connected = true;
    s.inventory.begin(2, "測試角色").unwrap();
    s.world.hp = Some([50, 50]);
    let own = s.world.own_actor().unwrap().id;
    {
        let a = s.world.actors.get_mut(&own).unwrap();
        a.x = 32729;
        a.y = 32797;
    }
    let j: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-ground-pickup-current.json"))
            .unwrap();
    let h = j["packets"]["ground"].as_str().unwrap();
    let b: Vec<_> = (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect();
    s.world.accept(&b).unwrap();
    let id = 0xf1234567;
    s.world.actors.get_mut(&id).unwrap().x += 1;
    let cancel = AtomicBool::new(false);
    let mut m = model();
    let f = m.draw_at(&s.world, &cancel, 0).unwrap();
    let point = (0..600)
        .flat_map(|y| (0..800).map(move |x| (x, y)))
        .find(|&(x, y)| f.pick_current(&s.world, x, y, own) == Some(id))
        .expect("visible original ground pixel");
    let c = Choice::at(&s, &f, point.0, point.1).unwrap();
    assert_eq!(c.item.id, id);
    let r = f.hits.iter().find(|(i, _)| *i == id).unwrap().1;
    let hole = (r[1]..r[3])
        .flat_map(|y| (r[0]..r[2]).map(move |x| (x, y)))
        .find(|&(x, y)| f.pick(x, y, own) != Some(id))
        .unwrap();
    assert!(Choice::at(&s, &f, hole.0, hole.1).is_err());
    s.world.visual_revision += 1;
    assert!(Choice::at(&s, &f, point.0, point.1).is_err());
    s.world.actors.get_mut(&id).unwrap().incarnation += 1;
    assert!(c.validate(&s).is_err());
}
