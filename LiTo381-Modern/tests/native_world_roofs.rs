use lito_modern::modern_lito::{
    native_actor_layers::Bank,
    native_world::World,
    native_world_layers::{self, Body, LayeredBody},
    native_world_roofs::{Definition, Runtime},
    native_world_scene::Model,
    resources::SharedMapSectorKey,
    sprite,
    world_map::{PixelPoint, SectorInput, ViewProjection, WorldCell, WorldMap},
};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::atomic::AtomicBool};
#[path = "support/native_fragment_reference.rs"]
mod reference;
fn root() -> &'static Path {
    Path::new(r"D:\lineage開發\0-客戶端")
}
fn world() -> World {
    let j: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for k in ["world", "start", "status", "actor", "bot"] {
        let h = j["packets"][k].as_str().unwrap();
        w.accept(
            &(0..h.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>(),
        )
        .unwrap();
    }
    w
}
fn fixture(records: &[[u8; 5]]) -> WorldMap {
    let mut raw = vec![0; 32768 + 2 + 16384];
    raw.extend(2i32.to_le_bytes());
    for id in [70i16, -7] {
        raw.extend(id.to_le_bytes());
        raw.extend(0u16.to_le_bytes());
    }
    raw.extend((records.len() as i32).to_le_bytes());
    for r in records {
        raw.extend(r);
    }
    WorldMap::from_sector_bytes(
        vec![SectorInput {
            key: SharedMapSectorKey {
                map_id: 514,
                sector_x: 0x8000,
                sector_y: 0x8000,
            },
            bytes: raw,
        }],
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn shade(f: &lito_modern::modern_lito::native_world_roofs::Frame) -> serde_json::Value {
    f.receipt()["groups"]
        .as_array()
        .unwrap()
        .first()
        .map_or(serde_json::json!("Native"), |r| r["normal"].clone())
}
#[test]
fn exact_half_cell_group_ordinals_and_bad_metadata_are_qualified() {
    let d = Definition::from_map(&fixture(&[
        [0, 0, 1, 0, 0],
        [1, 0, 0, 0, 1],
        [128, 0, 255, 255, 255],
    ]))
    .unwrap();
    assert_eq!(d.receipt()["ignored_outside_sector"], 1);
    let mut r = Runtime::default();
    let f = r
        .sample(&d, [1, 2, 3], WorldCell { x: 32768, y: 32768 }, 0)
        .unwrap();
    assert_eq!(f.receipt()["groups"][0]["ordinal"], 1);
    assert_eq!(f.receipt()["groups"].as_array().unwrap().len(), 1);
    for records in [vec![[0, 0, 2, 0, 0]], vec![[0, 0, 0, 0, 2]]] {
        assert!(Definition::from_map(&fixture(&records)).is_err());
    }
}
#[test]
fn roof_enter_leave_reverse_scope_and_clock_follow_bounded_quanta() {
    let inside = WorldCell { x: 32768, y: 32768 };
    let outside = WorldCell { x: 32769, y: 32768 };
    let d = Definition::from_map(&fixture(&[[0, 0, 0, 0, 0]])).unwrap();
    let mut r = Runtime::default();
    for (ms, w) in [(0, 31), (49, 31), (50, 27), (100, 23), (150, 19)] {
        assert_eq!(
            shade(&r.sample(&d, [1, 2, 3], inside, ms * 1_000_000).unwrap()),
            serde_json::json!({"Weight":w})
        );
    }
    assert_eq!(
        shade(&r.sample(&d, [1, 2, 3], inside, 200_000_000).unwrap()),
        "Half"
    );
    for (ms, w) in [(250, 17), (299, 17), (300, 21), (350, 25), (400, 29)] {
        assert_eq!(
            shade(&r.sample(&d, [1, 2, 3], outside, ms * 1_000_000).unwrap()),
            serde_json::json!({"Weight":w})
        );
    }
    assert_eq!(
        shade(&r.sample(&d, [1, 2, 3], outside, 450_000_000).unwrap()),
        "Native"
    );
    assert!(r.sample(&d, [1, 2, 3], inside, 1).is_err());
    assert!(r.sample(&d, [0, 2, 3], inside, 450_000_000).is_err());
    let hidden = Definition::from_map(&fixture(&[[0, 0, 0, 0, 1]])).unwrap();
    let mut r = Runtime::default();
    r.sample(&hidden, [1, 2, 3], inside, 0).unwrap();
    assert_eq!(
        shade(&r.sample(&hidden, [1, 2, 3], inside, 400_000_000).unwrap()),
        "Hidden"
    );
    assert_eq!(
        shade(&r.sample(&hidden, [1, 2, 3], outside, 450_000_000).unwrap()),
        serde_json::json!({"Weight":1})
    );
    assert_eq!(
        shade(&r.sample(&hidden, [1, 2, 3], inside, 550_000_000).unwrap()),
        serde_json::json!({"Weight":9})
    );
    assert_eq!(
        shade(&r.sample(&hidden, [1, 2, 4], inside, 550_000_000).unwrap()),
        serde_json::json!({"Weight":31})
    );
    assert_eq!(
        shade(
            &r.sample(&hidden, [1, 2, 4], inside, 3_600_000_000_000)
                .unwrap()
        ),
        "Hidden"
    );
}
#[test]
fn real_roof_groups_and_source_pixels_match_scalar_composition_and_owners() {
    let cancel = AtomicBool::new(false);
    let model = reference::full_native_fixture(root(), u32::from(world().map_id.unwrap()));
    let scene = model.scene();
    let binding = scene.assets().geometry().bindings()[0];
    let sector = scene.assets().geometry().sector(binding.key).unwrap();
    assert_eq!(sector.opacity.len(), 390);
    let record = sector
        .opacity
        .iter()
        .find(|r| r.x < 128 && r.y < 64 && r.x % 2 == 0)
        .unwrap();
    let (ox, oy) = binding.key.world_origin();
    let inside = WorldCell {
        x: ox + i32::from(record.x) / 2,
        y: oy + i32::from(record.y),
    };
    let outside = WorldCell { x: ox, y: oy };
    let view = ViewProjection {
        anchor_cell: inside,
        anchor_pixel: PixelPoint { x: 160, y: 120 },
    };
    let mut bank = Bank::open(root()).unwrap();
    bank.ensure("3225-0.spr", &cancel).unwrap();
    let source = bank.get("3225-0.spr").unwrap().source.clone();
    let raw = lito_modern::modern_lito::resources::SharedArchive::open(
        root(),
        lito_modern::modern_lito::resources::SharedArchiveKind::Sprite,
    )
    .unwrap()
    .read_entry("3225-0.spr")
    .unwrap();
    let decoded = sprite::decode_classic_spr_layered(&raw, Default::default()).unwrap();
    let mut clock = Runtime::default();
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    let mut cases = Vec::new();
    for (index, (ms, cell)) in [
        (0, inside),
        (50, inside),
        (100, inside),
        (150, inside),
        (200, inside),
        (250, outside),
        (350, outside),
        (450, outside),
    ]
    .into_iter()
    .enumerate()
    {
        let roof = clock
            .sample(scene.roofs(), [1, 1, 1], cell, ms * 1_000_000)
            .unwrap();
        let body = LayeredBody {
            body: Body {
                id: 1,
                center: view.anchor_pixel,
                frame: bank.frame("3225-0.spr", 0).unwrap(),
            },
            attachments: vec![],
            shadow: None,
        };
        let actual =
            native_world_layers::compose_with_roofs(scene, view, 320, 240, &[body], &roof, &cancel)
                .unwrap();
        let expected = reference::reference_roofs(
            scene,
            view,
            &[(1, view.anchor_pixel, &decoded, 0, 0)],
            320,
            240,
            true,
            Some(roof.receipt()),
        );
        assert_eq!(actual.canvas.rgba(), expected.0, "sample {index}");
        for y in 0..240 {
            for x in 0..320 {
                assert_eq!(
                    actual.visibility[0].1.contains(x, y),
                    expected.1[y as usize * 320 + x as usize] == 1
                );
            }
        }
        let png = output.join(format!("native-roof-reference-{index}.png"));
        let mut encoder = png::Encoder::new(std::fs::File::create(&png).unwrap(), 320, 240);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(actual.canvas.rgba())
            .unwrap();
        cases.push(serde_json::json!({"ms":ms,"roof":roof.receipt(),"png":png,"rgba_sha256":format!("{:x}",Sha256::digest(actual.canvas.rgba())),"owner_sha256":format!("{:x}",Sha256::digest(expected.1.iter().flat_map(|v|v.to_le_bytes()).collect::<Vec<_>>())),"visible_body_pixels":expected.1.iter().filter(|&&v|v==1).count()}));
    }
    assert!(
        cases
            .windows(2)
            .filter(|w| w[0]["rgba_sha256"] != w[1]["rgba_sha256"])
            .count()
            >= 5
    );
    let p = scene.assets().verified_plan();
    let hex = |b: &[u8]| b.iter().map(|v| format!("{v:02X}")).collect::<String>();
    let plan = serde_json::json!({"map_id":p.map_id,"canonical_sha256":hex(&p.canonical_sha256),"sectors":p.sectors.iter().map(|s|serde_json::json!({"map":s.key.map_id,"x":s.key.x,"y":s.key.y,"bytes":s.pin.bytes,"sha256":hex(&s.pin.sha256)})).collect::<Vec<_>>(),"archives":p.archives.iter().map(|a|serde_json::json!({"role":format!("{:?}",a.role),"bytes":a.pin.bytes,"sha256":hex(&a.pin.sha256)})).collect::<Vec<_>>()});
    assert!(
        cases[4]["visible_body_pixels"].as_u64().unwrap()
            > cases[0]["visible_body_pixels"].as_u64().unwrap()
    );
    let report = serde_json::json!({"cases":cases,"pixel_comparisons":8,"owner_comparisons":8,"definition":scene.roofs().receipt(),"body_source":source,"prepared_bytes":scene.prepared_bytes(),"plan":plan,"decoder_independent":false,"rgb565_native_parity":false,"gui_tested":false});
    std::fs::write(
        output.join("native-roof-reference.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
#[test]
fn live_roof_runtime_changes_presentation_without_mutating_world_authority() {
    let cancel = AtomicBool::new(false);
    let mut w = world();
    let own = w.own_actor().unwrap().id;
    let mut model = Model::load(root(), &w, &cancel).unwrap();
    {
        let a = w.actors.get_mut(&own).unwrap();
        a.x = 32720;
        a.y = 32804;
    }
    let before = serde_json::to_value(&w).unwrap();
    let a = model.draw_at(&w, &cancel, 0).unwrap();
    let b = model.draw_at(&w, &cancel, 200_000_000).unwrap();
    assert_eq!(a.receipt["fragment_composition"]["roof_runtime"], true);
    assert!(!a.receipt["roofs"]["groups"].as_array().unwrap().is_empty());
    assert_ne!(a.receipt["roofs"]["groups"], b.receipt["roofs"]["groups"]);
    assert_eq!(before, serde_json::to_value(&w).unwrap());
    w.actors.get_mut(&own).unwrap().incarnation += 1;
    let c = model.draw_at(&w, &cancel, 250_000_000).unwrap();
    assert_ne!(
        b.receipt["roofs"]["identity"],
        c.receipt["roofs"]["identity"]
    );
    assert!(model
        .draw_at(&w, &AtomicBool::new(true), 300_000_000)
        .is_err());
}

#[test]
fn stale_map_roof_frames_and_cancelled_composition_are_rejected() {
    let cancel = AtomicBool::new(false);
    let model = reference::full_native_fixture(root(), u32::from(world().map_id.unwrap()));
    let scene = model.scene();
    let foreign = Definition::from_map(&fixture(&[[0, 0, 0, 0, 1]])).unwrap();
    let mut runtime = Runtime::default();
    let old = runtime
        .sample(&foreign, [1, 2, 3], WorldCell { x: 32768, y: 32768 }, 0)
        .unwrap();
    let view = ViewProjection {
        anchor_cell: WorldCell { x: 32720, y: 32804 },
        anchor_pixel: PixelPoint { x: 160, y: 120 },
    };
    assert!(
        native_world_layers::compose_with_roofs(scene, view, 320, 240, &[], &old, &cancel).is_err()
    );
    let valid = runtime
        .sample(scene.roofs(), [1, 2, 3], view.anchor_cell, 500_000_000)
        .unwrap();
    assert_eq!(
        valid.receipt()["groups"][0]["normal"],
        serde_json::json!({"Weight":31})
    );
    assert!(native_world_layers::compose_with_roofs(
        scene,
        view,
        320,
        240,
        &[],
        &valid,
        &AtomicBool::new(true)
    )
    .is_err());
    assert!(
        native_world_layers::compose_with_roofs(scene, view, 320, 240, &[], &valid, &cancel)
            .is_ok()
    );
}

#[test]
fn real_hidden_roof_groups_use_source_565_without_fixture_substitution() {
    let cancel = AtomicBool::new(false);
    let model = reference::full_native_fixture(root(), 565);
    let scene = model.scene();
    let binding = scene
        .assets()
        .geometry()
        .bindings()
        .into_iter()
        .find(|b| {
            scene
                .assets()
                .geometry()
                .sector(b.key)
                .unwrap()
                .opacity
                .iter()
                .any(|r| r.kind == 1 && r.x < 128 && r.y < 64 && r.x % 2 == 0)
        })
        .unwrap();
    let sector = scene.assets().geometry().sector(binding.key).unwrap();
    assert!(!sector.opacity.is_empty());
    let record = sector
        .opacity
        .iter()
        .find(|r| r.kind == 1 && r.x < 128 && r.y < 64 && r.x % 2 == 0)
        .unwrap();
    let (ox, oy) = binding.key.world_origin();
    let inside = WorldCell {
        x: ox + i32::from(record.x) / 2,
        y: oy + i32::from(record.y),
    };
    let outside = WorldCell { x: ox, y: oy };
    let view = ViewProjection {
        anchor_cell: inside,
        anchor_pixel: PixelPoint { x: 160, y: 120 },
    };
    let mut bank = Bank::open(root()).unwrap();
    bank.ensure("3225-0.spr", &cancel).unwrap();
    let source = bank.get("3225-0.spr").unwrap().source.clone();
    let raw = lito_modern::modern_lito::resources::SharedArchive::open(
        root(),
        lito_modern::modern_lito::resources::SharedArchiveKind::Sprite,
    )
    .unwrap()
    .read_entry("3225-0.spr")
    .unwrap();
    let decoded = sprite::decode_classic_spr_layered(&raw, Default::default()).unwrap();
    let mut clock = Runtime::default();
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    let mut cases = Vec::new();
    for (index, (ms, cell)) in [
        (0, inside),
        (200, inside),
        (400, inside),
        (450, outside),
        (650, outside),
        (850, outside),
    ]
    .into_iter()
    .enumerate()
    {
        let roof = clock
            .sample(scene.roofs(), [1, 1, 1], cell, ms * 1_000_000)
            .unwrap();
        let body = LayeredBody {
            body: Body {
                id: 1,
                center: view.anchor_pixel,
                frame: bank.frame("3225-0.spr", 0).unwrap(),
            },
            attachments: vec![],
            shadow: None,
        };
        let actual =
            native_world_layers::compose_with_roofs(scene, view, 320, 240, &[body], &roof, &cancel)
                .unwrap();
        let expected = reference::reference_roofs(
            scene,
            view,
            &[(1, view.anchor_pixel, &decoded, 0, 0)],
            320,
            240,
            true,
            Some(roof.receipt()),
        );
        assert_eq!(actual.canvas.rgba(), expected.0, "sample {index}");
        for y in 0..240 {
            for x in 0..320 {
                assert_eq!(
                    actual.visibility[0].1.contains(x, y),
                    expected.1[y as usize * 320 + x as usize] == 1
                );
            }
        }
        let png = output.join(format!("native-roof-hidden-reference-{index}.png"));
        let mut encoder = png::Encoder::new(std::fs::File::create(&png).unwrap(), 320, 240);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(actual.canvas.rgba())
            .unwrap();
        cases.push(serde_json::json!({"ms":ms,"roof":roof.receipt(),"png":png,"rgba_sha256":format!("{:x}",Sha256::digest(actual.canvas.rgba())),"owner_sha256":format!("{:x}",Sha256::digest(expected.1.iter().flat_map(|v|v.to_le_bytes()).collect::<Vec<_>>())),"visible_body_pixels":expected.1.iter().filter(|&&v|v==1).count()}));
    }
    assert!(
        cases
            .windows(2)
            .filter(|w| w[0]["rgba_sha256"] != w[1]["rgba_sha256"])
            .count()
            >= 5
    );
    let p = scene.assets().verified_plan();
    let hex = |b: &[u8]| b.iter().map(|v| format!("{v:02X}")).collect::<String>();
    let plan = serde_json::json!({"map_id":p.map_id,"canonical_sha256":hex(&p.canonical_sha256),"sectors":p.sectors.iter().map(|s|serde_json::json!({"map":s.key.map_id,"x":s.key.x,"y":s.key.y,"bytes":s.pin.bytes,"sha256":hex(&s.pin.sha256)})).collect::<Vec<_>>(),"archives":p.archives.iter().map(|a|serde_json::json!({"role":format!("{:?}",a.role),"bytes":a.pin.bytes,"sha256":hex(&a.pin.sha256)})).collect::<Vec<_>>()});
    assert!(
        cases[4]["visible_body_pixels"].as_u64().unwrap()
            > cases[0]["visible_body_pixels"].as_u64().unwrap()
    );
    let report = serde_json::json!({"cases":cases,"pixel_comparisons":6,"owner_comparisons":6,"definition":scene.roofs().receipt(),"body_source":source,"prepared_bytes":scene.prepared_bytes(),"plan":plan,"decoder_independent":false,"rgb565_native_parity":false,"gui_tested":false});
    std::fs::write(
        output.join("native-roof-hidden-reference.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
