use lito_modern::modern_lito::{
    native_actor_layers::{Bank, Source},
    native_world::World,
    native_world_effects::{Effect, Mix, Target},
    native_world_layers::{self, Body, LayeredBody},
    native_world_scene::Model,
    resources::{SharedArchive, SharedArchiveKind},
    sprite,
    world_map::{PixelPoint, ViewProjection, WorldCell},
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
fn original(s: &Source) -> Vec<u8> {
    let k = if s.archive == "Sprite" {
        SharedArchiveKind::Sprite
    } else {
        SharedArchiveKind::SpriteShard(s.archive.strip_prefix("Sprite").unwrap().parse().unwrap())
    };
    let b = SharedArchive::open(root(), k)
        .unwrap()
        .read_entry(&s.member)
        .unwrap();
    assert_eq!(b.len(), s.bytes);
    assert_eq!(format!("{:X}", Sha256::digest(&b)), s.sha256);
    b
}
#[test]
fn actual_effect_fragments_match_scalar_pixels_occlusion_and_click_through_owners() {
    let cancel = AtomicBool::new(false);
    let m = reference::full_native_fixture(root(), u32::from(world().map_id.unwrap()));
    let mut bank = Bank::open(root()).unwrap();
    bank.ensure("3225-0.spr", &cancel).unwrap();
    let body_raw = sprite::decode_classic_spr_layered(
        &original(&bank.get("3225-0.spr").unwrap().source),
        Default::default(),
    )
    .unwrap();
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    let mut rows = Vec::new();
    let mut sources = vec![bank.get("3225-0.spr").unwrap().source.clone()];
    let mut occluded = 0;
    for (index, gfx) in [66, 167, 219, 221, 744, 189, 194, 197, 2231, 168]
        .into_iter()
        .enumerate()
    {
        let name = format!("{gfx}-0.spr");
        bank.ensure(&name, &cancel).unwrap();
        let source = bank.get(&name).unwrap().source.clone();
        let raw =
            sprite::decode_classic_spr_layered(&original(&source), Default::default()).unwrap();
        sources.push(source);
        let view = ViewProjection {
            anchor_cell: WorldCell {
                x: if index % 2 == 0 { 32720 } else { 32729 },
                y: if index % 2 == 0 { 32790 } else { 32797 },
            },
            anchor_pixel: PixelPoint { x: 160, y: 120 },
        };
        let frame = bank.frame(&name, 0).unwrap();
        let position = view.anchor_cell;
        let body = Body {
            id: 10,
            center: view.anchor_pixel,
            frame: bank.frame("3225-0.spr", 0).unwrap(),
        };
        let mix = if gfx == 66 { Mix::Over } else { Mix::Screen };
        let effect = Effect {
            frame,
            position,
            mix,
        };
        let base =
            native_world_layers::compose(m.scene(), view, 320, 240, &[body], &cancel).unwrap();
        let actual = native_world_layers::compose_with_effects(
            m.scene(),
            view,
            320,
            240,
            &[LayeredBody {
                body,
                attachments: vec![],
                shadow: None,
            }],
            &Default::default(),
            &[effect],
            &cancel,
        )
        .unwrap();
        let expected = reference::reference_layers(
            m.scene(),
            view,
            &[
                (10, view.anchor_pixel, &body_raw, 0, 0),
                (0, view.anchor_pixel, &raw, 0, if gfx == 66 { 4 } else { 2 }),
            ],
            320,
            240,
            true,
        );
        assert_eq!(actual.canvas.rgba(), expected.0, "gfx {gfx}");
        let mut owner_bytes = Vec::new();
        for y in 0..240 {
            for x in 0..320 {
                let hit = actual.visibility[0].1.contains(x, y);
                assert_eq!(hit, expected.1[y as usize * 320 + x as usize] == 10);
                assert_eq!(hit, base.visibility[0].1.contains(x, y));
                owner_bytes.push(u8::from(hit));
            }
        }
        // Opaque probe changes only the reference ownership, not coverage or
        // depth: proves terrain occlusion even if Screen produces no RGB change.
        let probes = [
            (10, view.anchor_pixel, &body_raw, 0, 0),
            (777, view.anchor_pixel, &raw, 0, 0),
        ];
        let no_terrain_owners =
            reference::reference_layers(m.scene(), view, &probes, 320, 240, false).1;
        let terrain_owners =
            reference::reference_layers(m.scene(), view, &probes, 320, 240, true).1;
        let terrain_occluded = no_terrain_owners
            .iter()
            .zip(&terrain_owners)
            .filter(|(a, b)| **a == 777 && **b != 777)
            .count();
        let visible_effect = base
            .canvas
            .rgba()
            .chunks_exact(4)
            .zip(actual.canvas.rgba().chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        let mut overlay = base.canvas;
        Target::Canvas(&mut overlay)
            .submit(frame, position, view, mix)
            .unwrap();
        let changed = actual
            .canvas
            .rgba()
            .chunks_exact(4)
            .zip(overlay.rgba().chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        if terrain_occluded > 0 {
            occluded += 1;
        }
        let png = out.join(format!("native-world-effect-{index}.png"));
        let mut e = png::Encoder::new(std::fs::File::create(&png).unwrap(), 320, 240);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()
            .unwrap()
            .write_image_data(actual.canvas.rgba())
            .unwrap();
        rows.push(serde_json::json!({"graphics":gfx,"sprite":name,"subframe":0,"position":[position.x,position.y],"png":png,"rgba_sha256":format!("{:x}",Sha256::digest(actual.canvas.rgba())),"owner_sha256":format!("{:x}",Sha256::digest(owner_bytes)),"overlay_different_pixels":changed,"terrain_occluded_effect_pixels":terrain_occluded,"visible_effect_pixels":visible_effect,"fragment_count":frame.fragments.as_ref().unwrap().references.len()}));
    }
    assert!(
        occluded >= 5,
        "only {occluded} effects have terrain-occluded contribution"
    );
    std::fs::write(out.join("native-world-effects-reference.json"),serde_json::to_vec_pretty(&serde_json::json!({"cases":rows,"sources":sources,"pixel_comparisons":10,"owner_surface_comparisons":10,"decoder_independent":false,"gui_tested":false})).unwrap()).unwrap();
}
#[test]
fn fragment_and_flat_sinks_have_identical_clock_events_retirement_and_generation() {
    use lito_modern::modern_lito::native_skill_effects::Effects;
    let mut w = world();
    w.effects
        .push(lito_modern::modern_lito::native_skills::Effect {
            sequence: 1,
            graphics: 221,
            target: None,
            position: [32729, 32797],
        });
    let mut a = Effects::default();
    let mut b = Effects::default();
    let cancel = AtomicBool::new(false);
    a.prepare(root(), &w, &cancel);
    b.prepare(root(), &w, &cancel);
    let v = ViewProjection {
        anchor_cell: WorldCell { x: 32729, y: 32797 },
        anchor_pixel: PixelPoint { x: 160, y: 120 },
    };
    for ns in [0, 0, 42_000_000, 300_000_000, 500_000_000, 600_000_000] {
        let mut c = lito_modern::modern_lito::compositor::Canvas::new(320, 240).unwrap();
        let mut q = vec![];
        let flat = a.draw(&w, &mut c, v, ns);
        let queued = b.draw_to(&w, &mut Target::Fragments(&mut q), v, ns);
        assert_eq!(flat.0, queued.0);
        assert_eq!(
            serde_json::to_value(flat.1).unwrap(),
            serde_json::to_value(queued.1).unwrap()
        );
        assert_eq!(
            q.len(),
            queued.0["effects"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["drawn"] == true)
                .count()
        );
    }
    w.generation += 1;
    w.effects.clear();
    let mut q = vec![];
    assert_eq!(
        b.draw_to(&w, &mut Target::Fragments(&mut q), v, 0).0["active"],
        0
    );
    assert!(q.is_empty());
}
#[test]
fn live_model_submits_all_stationary_profiles_once_without_world_mutation() {
    let mut w = world();
    for (i, gfx) in [221, 744, 189, 194, 197, 2231, 168].into_iter().enumerate() {
        w.effects
            .push(lito_modern::modern_lito::native_skills::Effect {
                sequence: i as u64 + 1,
                graphics: gfx,
                target: None,
                position: [32729, 32797],
            });
    }
    let before = serde_json::to_value(&w).unwrap();
    let cancel = AtomicBool::new(false);
    let mut m = Model::load(root(), &w, &cancel).unwrap();
    let first = m.draw_at(&w, &cancel, 0).unwrap();
    assert_eq!(
        first.receipt["fragment_composition"]["effect_submissions"],
        7
    );
    let repeat = m.draw_at(&w, &cancel, 0).unwrap();
    assert!(repeat.receipt["audio"].as_array().unwrap().is_empty());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    assert!(m.draw_at(&w, &AtomicBool::new(true), 1).is_err());
}
#[test]
fn effect_bounds_missing_fragments_and_all_arrow_heading_banks_are_checked() {
    let cancel = AtomicBool::new(false);
    let assets =
        lito_modern::modern_lito::ranged_effect::RangedAssets::load(root(), &cancel).unwrap();
    for h in 0..8 {
        for f in assets.sprite_frames(h).unwrap() {
            assert!(f.fragments.is_some());
        }
    }
    let frame = &assets.sprite_frames(0).unwrap()[0];
    let position = WorldCell { x: 32768, y: 32768 };
    let view = ViewProjection {
        anchor_cell: position,
        anchor_pixel: PixelPoint { x: 160, y: 120 },
    };
    let mut q = Vec::new();
    for _ in 0..512 {
        Target::Fragments(&mut q)
            .submit(frame, position, view, Mix::Over)
            .unwrap();
    }
    assert!(Target::Fragments(&mut q)
        .submit(frame, position, view, Mix::Over)
        .is_err());
    assert_eq!(q.len(), 512);
    let mut bad = frame.clone();
    bad.fragments = None;
    let mut q = vec![];
    assert!(Target::Fragments(&mut q)
        .submit(&bad, position, view, Mix::Over)
        .is_err());
    assert!(q.is_empty());
}
