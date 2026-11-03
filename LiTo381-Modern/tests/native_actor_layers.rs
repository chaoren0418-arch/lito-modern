use lito_modern::modern_lito::{
    resources::{SharedArchive, SharedArchiveKind},
    sprite_actions::{ActionLimits, SpriteActionCatalog},
};
use std::path::Path;
#[test]
fn inspect_actual_actor_layer_catalog() {
    let raw = SharedArchive::open(
        Path::new(r"D:\lineage開發\0-客戶端"),
        SharedArchiveKind::Text,
    )
    .unwrap()
    .read_entry("list.spr")
    .unwrap();
    let c = SpriteActionCatalog::parse_for_inspection(&raw, ActionLimits::default()).unwrap();
    let mut ids = vec![
        0, 61, 936, 143, 834, 460, 148, 237, 727, 98, 1544, 773, 3225, 3226, 6262,
    ];
    let mut rows = Vec::new();
    let mut done = std::collections::BTreeSet::new();
    while let Some(id) = ids.pop() {
        if !done.insert(id) {
            continue;
        }
        let s = &c.sprites[&id];
        let attrs:Vec<_>=s.attributes.iter().map(|a|serde_json::json!({"id":a.key.id,"value":String::from_utf8_lossy(&a.body_bytes)})).collect();
        for a in &s.attributes {
            if a.key.id == 101 {
                ids.push(String::from_utf8_lossy(&a.body_bytes).parse().unwrap());
            }
            if a.key.id == 105 {
                ids.extend(
                    String::from_utf8_lossy(&a.body_bytes)
                        .split_whitespace()
                        .skip(1)
                        .map(|s| s.parse::<u32>().unwrap()),
                );
            }
        }
        rows.push(serde_json::json!({"logical":id,"graphics":s.graphics_id,"image_count":s.image_count,"header":String::from_utf8_lossy(&s.header_bytes),"issue":format!("{:?}",s.header_issue),"attributes":attrs}));
    }
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/current/verification/native-actor-layer-catalog.json");
    std::fs::write(p, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    assert!(done.contains(&3226) && done.contains(&6262));
}

use lito_modern::modern_lito::{
    native_actor_layers::{Bank, Mix, Plan, Source},
    native_world_animation,
    native_world_layers::{self, Attachment, Body, LayeredBody},
    sprite,
    sprite_actions::{ActionKey, Directionality},
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
#[path = "support/native_fragment_reference.rs"]
mod reference;
fn catalog() -> SpriteActionCatalog {
    let b = SharedArchive::open(
        Path::new(r"D:\lineage開發\0-客戶端"),
        SharedArchiveKind::Text,
    )
    .unwrap()
    .read_entry("list.spr")
    .unwrap();
    SpriteActionCatalog::parse_for_inspection(&b, ActionLimits::default()).unwrap()
}
fn root() -> &'static Path {
    Path::new(r"D:\lineage開發\0-客戶端")
}
fn world() -> lito_modern::modern_lito::native_world::World {
    let j: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = lito_modern::modern_lito::native_world::World::default();
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
fn original(s: &Source) -> Vec<u8> {
    let kind = if s.archive == "Sprite" {
        SharedArchiveKind::Sprite
    } else {
        SharedArchiveKind::SpriteShard(s.archive.strip_prefix("Sprite").unwrap().parse().unwrap())
    };
    let b = SharedArchive::open(root(), kind)
        .unwrap()
        .read_entry(&s.member)
        .unwrap();
    assert_eq!(b.len(), s.bytes);
    assert_eq!(format!("{:X}", Sha256::digest(&b)), s.sha256);
    b
}
#[test]
fn body_image_and_subframe_bind_real_layers_across_actions_and_directions() {
    let c = catalog();
    let mut bank = Bank::open(root()).unwrap();
    let cancel = AtomicBool::new(false);
    let mut sources = BTreeMap::new();
    let mut samples = 0usize;
    let mut missing_bodies = std::collections::BTreeSet::new();
    let mut missing = std::collections::BTreeSet::new();
    let mut missing_shadows = std::collections::BTreeSet::new();
    for id in [0, 61, 727, 936] {
        let plan = Plan::from_catalog(&c, id).unwrap();
        assert!(plan.shadow.is_some());
        for action in [0, 1, 3, 4, 8, 15, 18, 19] {
            let Ok(clip) = native_world_animation::resolve(&c, id, action) else {
                continue;
            };
            for f in &clip.animation.frames {
                for heading in 0..if clip.animation.directionality == Directionality::EightWay {
                    8
                } else {
                    1
                } {
                    let image = f.image_id + heading;
                    let body = format!("{}-{image}.spr", clip.graphics);
                    if let Err(e) = bank.ensure(&body, &cancel) {
                        assert!(matches!(e.downcast_ref::<lito_modern::modern_lito::resources::ResourceError>(), Some(lito_modern::modern_lito::resources::ResourceError::MissingEntry(_))), "{e:#}");
                        missing_bodies.insert(body);
                        continue;
                    }
                    let bindings = plan.bind(image).unwrap();
                    let prepared = bank
                        .prepare(&bindings, f.subframe as usize, &cancel)
                        .unwrap();
                    missing.extend(prepared.missing_attachments);
                    missing_shadows.extend(prepared.missing_shadow);
                    let bindings = prepared.bindings;
                    bank.frame(&body, f.subframe as usize).unwrap();
                    for name in std::iter::once(&body)
                        .chain(bindings.attachments.iter().map(|(n, _)| n))
                        .chain(bindings.shadow.iter())
                    {
                        let loaded = bank.get(name).unwrap();
                        sources.insert(name.clone(), loaded.source.clone());
                    }
                    samples += 1;
                }
            }
        }
    }
    let plan = Plan::from_catalog(&c, 1544).unwrap();
    bank.prepare(&plan.bind(0).unwrap(), 0, &cancel).unwrap();
    assert!(samples > 1000);
    assert!(sources
        .values()
        .any(|s| s.archive.starts_with("Sprite") && s.archive != "Sprite"));
    assert!(sources.values().any(|s| s.archive == "Sprite"));
    assert!(bank.bytes() <= 64 << 20);
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/current/verification/native-actor-layer-coverage.json");
    std::fs::write(p,serde_json::to_vec_pretty(&serde_json::json!({"samples":samples,"missing_bodies":missing_bodies,"missing_attachments":missing,"missing_shadows":missing_shadows,"sources":sources.values().collect::<Vec<_>>(),"cache_bytes":bank.bytes(),"members":bank.len(),"native_dynamic_flags_equivalent":false})).unwrap()).unwrap();
}
#[test]
fn catalog_attributes_fail_closed_and_logical_alias_keeps_its_layers() {
    let mut c = catalog();
    let p = Plan::from_catalog(&c, 0).unwrap();
    assert_eq!(p.shadow.as_ref().unwrap().graphics, 3226);
    assert_eq!(p.attachments[0].graphics, 6262);
    assert_eq!(p.attachments[0].mix, Mix::Screen);
    assert!(Plan::from_catalog(&c, 3225).unwrap().shadow.is_none());
    assert!(Plan::from_catalog(&c, 773).unwrap().attachments.is_empty());
    assert!(p.bind(312).is_err());
    let baseline = c.sprites[&0].clone();
    for bad in [
        "2 6262",
        "9 6262 6262 6262 6262 6262 6262 6262 6262 6262",
        "1 -1",
        "1 999999",
        "1 6262 extra",
    ] {
        c.sprites.insert(0, baseline.clone());
        c.sprites
            .get_mut(&0)
            .unwrap()
            .attributes
            .iter_mut()
            .find(|a| a.key == ActionKey::new(105))
            .unwrap()
            .body_bytes = bad.as_bytes().to_vec();
        assert!(Plan::from_catalog(&c, 0).is_err(), "{bad}");
    }
    c.sprites.insert(0, baseline.clone());
    let duplicate = c.sprites[&0]
        .attributes
        .iter()
        .find(|a| a.key.id == 101)
        .unwrap()
        .clone();
    c.sprites.get_mut(&0).unwrap().attributes.push(duplicate);
    assert!(Plan::from_catalog(&c, 0).is_err());
    c.sprites.insert(0, baseline);
    c.sprites
        .get_mut(&6262)
        .unwrap()
        .attributes
        .iter_mut()
        .find(|a| a.key.id == 104)
        .unwrap()
        .body_bytes = b"99".to_vec();
    assert!(Plan::from_catalog(&c, 0).is_err());
    let mut bank = Bank::open(root()).unwrap();
    assert!(bank.ensure("3225-0.spr", &AtomicBool::new(true)).is_err());
    assert_eq!(bank.len(), 0);
    bank.ensure("3225-0.spr", &AtomicBool::new(false)).unwrap();
    assert!(bank.frame("3225-0.spr", usize::MAX).is_err());
}
#[test]
fn original_body_attachment_shadow_composition_matches_independent_rgb555_and_hit_surfaces() {
    let c = catalog();
    let mut bank = Bank::open(root()).unwrap();
    let cancel = AtomicBool::new(false);
    let model = reference::full_native_fixture(root(), u32::from(world().map_id.unwrap()));
    let mut rows = Vec::new();
    let mut source_pins = BTreeMap::new();
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    let mut changed = 0;
    for (index, (id, action, heading, frame_index, shift)) in [
        (0, 3, 0, 2, 0),
        (0, 15, 3, 1, 0),
        (61, 1, 6, 3, 0),
        (727, 3, 2, 1, 0),
        (1544, 0, 0, 0, 0),
        (0, 3, 0, 3, 11),
    ]
    .into_iter()
    .enumerate()
    {
        let (graphics, image, sub) = if id == 1544 {
            (1544, 0, 0)
        } else {
            let clip = native_world_animation::resolve(&c, id, action).unwrap();
            let f = &clip.animation.frames[frame_index];
            (clip.graphics, f.image_id + heading, f.subframe as usize)
        };
        let body_name = format!("{graphics}-{image}.spr");
        let binding = Plan::from_catalog(&c, id).unwrap().bind(image).unwrap();
        bank.ensure(&body_name, &cancel).unwrap();
        let prepared = bank.prepare(&binding, sub, &cancel).unwrap();
        let binding = prepared.bindings;
        let view = ViewProjection {
            anchor_cell: WorldCell {
                x: if index == 1 { 32720 } else { 32729 },
                y: if index == 1 { 32790 } else { 32797 },
            },
            anchor_pixel: PixelPoint { x: 160, y: 120 },
        };
        let center = PixelPoint {
            x: 160 + shift,
            y: 120 + shift / 2,
        };
        let body = Body {
            id: 10,
            center,
            frame: bank.frame(&body_name, sub).unwrap(),
        };
        let base =
            native_world_layers::compose(model.scene(), view, 320, 240, &[body], &cancel).unwrap();
        let attachments: Vec<_> = binding
            .attachments
            .iter()
            .map(|(n, m)| Attachment {
                frame: bank.frame(n, sub).unwrap(),
                mix: *m,
            })
            .collect();
        let shadow = binding.shadow.as_ref().map(|n| bank.frame(n, sub).unwrap());
        let layered = LayeredBody {
            body,
            attachments,
            shadow,
        };
        let actual = native_world_layers::compose_with_layers(
            model.scene(),
            view,
            320,
            240,
            &[layered],
            &cancel,
        )
        .unwrap();
        let mut raw_layers = Vec::new();
        for (name, mode) in std::iter::once((&body_name, 0u8))
            .chain(
                binding
                    .attachments
                    .iter()
                    .map(|(n, m)| (n, if *m == Mix::Screen { 2 } else { 0 })),
            )
            .chain(binding.shadow.iter().map(|n| (n, 1)))
        {
            let source = bank.get(name).unwrap().source.clone();
            let raw = original(&source);
            source_pins.insert(name.clone(), source);
            raw_layers.push((
                sprite::decode_classic_spr_layered(&raw, Default::default()).unwrap(),
                mode,
            ));
        }
        let refs: Vec<_> = raw_layers
            .iter()
            .map(|(s, m)| (if *m == 1 { 0 } else { 10 }, center, s, sub, *m))
            .collect();
        let expected = reference::reference_layers(model.scene(), view, &refs, 320, 240, true);
        assert_eq!(actual.canvas.rgba(), expected.0);
        for y in 0..240 {
            for x in 0..320 {
                assert_eq!(
                    actual.visibility[0].1.contains(x, y),
                    expected.1[y as usize * 320 + x as usize] == 10
                );
                assert_eq!(
                    actual.visibility[0].1.contains(x, y),
                    base.visibility[0].1.contains(x, y)
                );
            }
        }
        if actual.canvas.rgba() != base.canvas.rgba() {
            changed += 1;
        }
        let path = output.join(format!("native-actor-layer-reference-{index}.png"));
        let mut encoder = png::Encoder::new(std::fs::File::create(&path).unwrap(), 320, 240);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(actual.canvas.rgba())
            .unwrap();
        rows.push(serde_json::json!({"logical":id,"image":image,"subframe":sub,"heading":heading,"bindings":binding,"missing_attachments":prepared.missing_attachments,"png":path,"rgba_sha256":format!("{:x}",Sha256::digest(actual.canvas.rgba())),"base_rgba_sha256":format!("{:x}",Sha256::digest(base.canvas.rgba()))}));
    }
    assert!(changed >= 4);
    std::fs::write(output.join("native-actor-layer-reference.json"),serde_json::to_vec_pretty(&serde_json::json!({"cases":rows,"sources":source_pins.values().collect::<Vec<_>>(),"pixel_comparisons":6,"owner_surface_comparisons":6,"rgb555_reference":true,"native_rgb565_parity":false,"decoder_independent":false,"gui_tested":false})).unwrap()).unwrap();
}
#[test]
fn live_scene_layers_follow_body_without_own_clocks_events_or_stale_actor_ids() {
    let cancel = AtomicBool::new(false);
    let mut w = world();
    let mut model =
        lito_modern::modern_lito::native_world_scene::Model::load(root(), &w, &cancel).unwrap();
    let before = serde_json::to_value(&w).unwrap();
    for t in [0, 100_000_000, 700_000_000, 1_500_000_000] {
        let f = model.draw_at(&w, &cancel, t).unwrap();
        for a in f.receipt["actors"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|a| a["rendered"] == true)
        {
            assert!(a["layer_error"].is_null());
            for l in a["layers"].as_array().unwrap() {
                assert_eq!(l["subframe"], a["sprite_subframe"]);
                assert_eq!(
                    l["source"]["member"].as_str().unwrap().rsplit('-').next(),
                    a["sprite_member"].as_str().unwrap().rsplit('-').next()
                );
                assert_eq!(l["body_linked"], true);
            }
        }
    }
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    w.actors.remove(&2);
    w.visual_revision += 1;
    let f = model.draw_at(&w, &cancel, 2_000_000_000).unwrap();
    assert!(f.receipt["actors"]
        .as_array()
        .unwrap()
        .iter()
        .all(|a| a["id"] != 2));
}
