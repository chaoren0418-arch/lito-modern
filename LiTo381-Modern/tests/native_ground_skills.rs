use lito_modern::modern_lito::{
    native_ground_effects,
    native_skill_selection::{Ground, Model, Target},
    native_skills as skills,
    native_world::World,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::{path::Path, sync::atomic::AtomicBool};
fn packet(data: &[u8], key: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_slice(data).unwrap();
    let s = v["packets"][key].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn p(key: &str) -> Vec<u8> {
    packet(
        include_bytes!("fixtures/native-ground-skills-current.json"),
        key,
    )
}
fn world() -> World {
    let mut w = World::default();
    for key in ["world", "start", "status", "actor", "bot"] {
        w.accept(&packet(
            include_bytes!("fixtures/native-world-current.json"),
            key,
        ))
        .unwrap();
    }
    let mut a = w.actors.remove(&0xfedcba98).unwrap();
    a.id = 1;
    a.x = 32729;
    a.y = 32797;
    w.actors.insert(1, a);
    w.status.as_mut().unwrap().object_id = 1;
    let t = w.actors.get_mut(&2).unwrap();
    t.x = 32735;
    t.y = 32800;
    w.hp = Some([50, 50]);
    w.mp = Some([120, 120]);
    w.accept(&p("book")).unwrap();
    w
}
#[test]
fn formal_ground_request_unsigned_boundaries_range_and_authority() {
    let mut w = world();
    let before = serde_json::to_value(&w).unwrap();
    assert_eq!(
        skills::ground_request(&w, 58, [32735, 32800]).unwrap(),
        p("request")
    );
    assert!(skills::request(&w, 58, Some(2)).is_err());
    assert!(skills::ground_request(&w, 63, [32735, 32800]).is_err());
    for point in [[32729, 32797], [32738, 32797], [0, 0]] {
        assert!(skills::ground_request(&w, 58, point).is_err());
    }
    assert!(skills::ground_request(&w, 58, [32737, 32805]).is_ok());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    let a = w.actors.get_mut(&1).unwrap();
    a.x = 65534;
    a.y = 0;
    assert_eq!(
        skills::ground_request(&w, 58, [65535, 0]).unwrap(),
        p("boundary")
    );
    w.mp = Some([0, 120]);
    assert!(skills::ground_request(&w, 58, [65535, 0]).is_err());
    w.mp = Some([120, 120]);
    w.hp = Some([0, 50]);
    assert!(skills::ground_request(&w, 58, [65535, 0]).is_err());
}
#[test]
fn action18_and_location_effect_are_atomic_without_local_mana_or_damage() {
    let mut w = world();
    let before = serde_json::to_value(&w).unwrap();
    for key in ["cast", "effect"] {
        let b = p(key);
        for n in 0..b.len() {
            assert!(w.accept(&b[..n]).is_err());
            assert_eq!(serde_json::to_value(&w).unwrap(), before);
        }
    }
    w.accept(&p("cast")).unwrap();
    assert_eq!(w.own_cast_packets, 1);
    assert_eq!(w.own_attack_packets, 0);
    w.accept(&p("effect")).unwrap();
    assert_eq!(w.effects.len(), 1);
    assert_eq!(w.effects[0].position, [32735, 32800]);
    assert_eq!(w.effects[0].target, None);
    assert_eq!(w.mp, Some([120, 120]));
    assert_eq!(w.hp, Some([50, 50]));
    assert_eq!(w.actors[&2].hp_ratio, 100);
    w.accept(&p("mana")).unwrap();
    assert_eq!(w.mp, Some([60, 120]));
}
#[test]
fn ground_choice_is_separate_from_actor_target_and_fenced_by_owner() {
    let w = world();
    let mut m = Model::default();
    m.sync(9, &w, true);
    m.select(Some(58)).unwrap();
    let cell = WorldCell { x: 32735, y: 32800 };
    let ground = Ground::capture(9, &w, cell);
    assert!(m.choice(9, &w, Target::capture(&w, 2)).is_err());
    let c = m.choice_at(9, &w, Target::capture(&w, 2), ground).unwrap();
    assert_eq!(c.target, None);
    assert_eq!(c.ground, Some([32735, 32800]));
    assert!(Ground::capture(9, &w, WorldCell { x: -1, y: 0 }).is_none());
    assert!(Ground::capture(9, &w, WorldCell { x: 65536, y: 0 }).is_none());
    for variant in 0..5 {
        let mut next = w.clone();
        let mut epoch = 9;
        match variant {
            0 => epoch += 1,
            1 => next.generation += 1,
            2 => next.local_corrections += 1,
            3 => next.actors.get_mut(&1).unwrap().incarnation += 1,
            _ => {
                next.learned_skills.remove(&58);
            }
        }
        m.sync(epoch, &next, true);
        if next.learned_skills.contains(&58) {
            m.select(Some(58)).unwrap();
        }
        assert!(m.choice_at(epoch, &next, None, ground).is_err());
    }
}
#[test]
fn native_class0_enters_source1_and_dwells_terminal0_before_retirement() {
    let ticks = [1u64, 1, 2, 3, 3, 3, 3, 3, 3, 2, 2];
    let mut start = 0;
    for (i, duration) in ticks.into_iter().enumerate() {
        let source = if i == 10 { 0 } else { i + 1 };
        let ns = (start * 1_000_000_000u64).div_ceil(24);
        assert_eq!(native_ground_effects::sample(ns), (source, false));
        start += duration;
        assert_eq!(
            native_ground_effects::sample((start * 1_000_000_000u64).div_ceil(24) - 1),
            (source, false)
        );
    }
    assert_eq!(start, 26);
    assert_eq!(
        native_ground_effects::sample((26_000_000_000u64).div_ceil(24)),
        (0, true)
    );
    assert_eq!(native_ground_effects::sample(u64::MAX), (0, true));
}
#[test]
fn original_resource_eight_segments_terminal_dwell_and_same_id_sound() {
    use lito_modern::modern_lito::native_world_scene::Model as Scene;
    let mut w = world();
    let cancel = AtomicBool::new(false);
    let mut scene = Scene::load(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel).unwrap();
    let empty = scene.draw_at(&w, &cancel, 0).unwrap().receipt["rgba_sha256"].clone();
    for x in 32730u16..32738 {
        let mut b = vec![0x53];
        b.extend_from_slice(&x.to_le_bytes());
        b.extend_from_slice(&32797u16.to_le_bytes());
        b.extend_from_slice(&168u16.to_le_bytes());
        w.accept(&b).unwrap();
    }
    let authority = serde_json::to_value(&w).unwrap();
    let mut frames = Vec::new();
    let mut emitted = 0;
    let mut heard = 0;
    let mut suppressed = 0;
    for ms in (0..=1200u64).step_by(25) {
        let f = scene.draw_at(&w, &cancel, ms * 1_000_000).unwrap();
        let mut r = f.receipt;
        let fx = &r["ground_effects"];
        assert!(fx["load_error"].is_null());
        assert_eq!(fx["omitted"], 0);
        assert_eq!(r["skill_effects"]["omitted"], 0);
        assert_eq!(fx["started"], 8);
        emitted += fx["entered_events"].as_array().unwrap().len();
        for a in r["audio"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|a| a["event"]["event"] == "GroundSound(111)")
        {
            assert_eq!(a["prepared"], true);
            if a["same_id_suppressed"] == true {
                suppressed += 1;
            } else {
                heard += 1;
            }
        }
        if ms == 1000 {
            assert!(fx["effects"]
                .as_array()
                .unwrap()
                .iter()
                .all(|e| e["source_frame"] == 0 && e["drawn"] == true));
        }
        if ms == 1100 {
            assert_eq!(fx["active"], 0);
        }
        if ms == 450 {
            assert_ne!(r["rgba_sha256"], empty);
        }
        if [0, 450, 1000, 1100].contains(&ms) {
            let path = format!("target/current/verification/native-wall-{ms}.png");
            let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(&path);
            let mut enc = png::Encoder::new(std::fs::File::create(file).unwrap(), 800, 600);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header()
                .unwrap()
                .write_image_data(f.canvas.rgba())
                .unwrap();
            r["png"] = serde_json::json!(path);
        }
        frames.push(r);
        assert_eq!(serde_json::to_value(&w).unwrap(), authority);
    }
    assert_eq!((emitted, heard, suppressed), (8, 1, 7));
    std::fs::write(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/current/verification/native-wall.json"),
        serde_json::to_vec_pretty(&frames).unwrap(),
    )
    .unwrap();
    w.generation += 1;
    w.effects.clear();
    let f = scene.draw_at(&w, &cancel, 1300_000_000).unwrap();
    assert_eq!(f.receipt["ground_effects"]["started"], 0);
    assert_eq!(f.receipt["ground_effects"]["active"], 0);
}
#[test]
fn ground_effects_are_bounded_deduplicated_and_never_follow_actor_ids() {
    use lito_modern::modern_lito::{compositor::Canvas, native_skills::Effect};
    let mut w = world();
    let cancel = AtomicBool::new(false);
    let mut effects = native_ground_effects::Effects::default();
    effects.prepare(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel);
    w.effects = (1..=65)
        .map(|sequence| Effect {
            sequence,
            graphics: 168,
            target: None,
            position: [32730, 32797],
        })
        .collect();
    let projection = ViewProjection {
        anchor_cell: WorldCell { x: 32730, y: 32797 },
        anchor_pixel: PixelPoint { x: 400, y: 300 },
    };
    let mut canvas = Canvas::new(800, 600).unwrap();
    let (r, e) = effects.draw(&w, &mut canvas, projection, 0);
    assert_eq!(r["active"], 64);
    assert_eq!(r["omitted"], 1);
    assert_eq!(e.len(), 64);
    let (r, e) = effects.draw(&w, &mut canvas, projection, 0);
    assert_eq!(r["started"], 64);
    assert!(e.is_empty());
    let (r, e) = effects.draw(&w, &mut canvas, projection, 20_000_000_000);
    assert_eq!(r["active"], 0);
    assert!(e.is_empty());
    w.generation += 1;
    w.effects = vec![Effect {
        sequence: 1,
        graphics: 168,
        target: Some((1, w.actors[&1].incarnation)),
        position: [32730, 32797],
    }];
    let (r, e) = effects.draw(&w, &mut canvas, projection, 21_000_000_000);
    assert_eq!(r["active"], 0);
    assert_eq!(r["omitted"], 1);
    assert!(e.is_empty());
}

#[test]
fn original_wave_duration_gates_repeated_ground_audio_and_world_reset() {
    use lito_modern::modern_lito::{
        combat_audio::PreparedSound, native_world_scene::Model as Scene, resources,
    };
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let raw = resources::read_shared_sound_with_limit(
        root,
        resources::SharedSoundKey { sound_id: 111 },
        1 << 20,
    )
    .unwrap();
    let sound = PreparedSound::from_wav(111, raw).unwrap();
    let duration = (sound.pcm_bytes().len() as u64 * 1_000_000_000)
        .div_ceil(u64::from(sound.format().byte_rate));
    assert!(duration > 0);
    let cancel = AtomicBool::new(false);
    let mut w = world();
    let mut scene = Scene::load(root, &w, &cancel).unwrap();
    for (now, suppressed) in [(0, false), (duration - 1, true), (duration, false)] {
        w.accept(&p("effect")).unwrap();
        let f = scene.draw_at(&w, &cancel, now).unwrap();
        let sounds = f.receipt["audio"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|a| a["event"]["event"] == "GroundSound(111)")
            .collect::<Vec<_>>();
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0]["same_id_suppressed"], suppressed);
    }
    w.generation += 1;
    w.effects.clear();
    w.accept(&p("effect")).unwrap();
    let f = scene.draw_at(&w, &cancel, duration + 1).unwrap();
    let row = f.receipt["audio"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["event"]["event"] == "GroundSound(111)")
        .unwrap();
    assert_eq!(row["same_id_suppressed"], false);
}
