use lito_modern::modern_lito::{
    compositor::Canvas,
    native_skill_effects::Effects,
    native_skill_selection::{Ground, Model, Target},
    native_skills as skills,
    native_world::World,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::atomic::AtomicBool};
#[test]
fn completion_waits_for_sampled_effect_and_caster_with_bounded_diagnostics() {
    use lito_modern::modern_lito::native_combat_probe::reagent_completion_ready as ready;
    use std::time::Duration;
    let mut fx = serde_json::json!({"started":1,"active":1,"omitted":0,"load_error":null});
    for ms in [3600, 9999] {
        assert!(!ready(&fx, 1, true, Duration::from_millis(ms)).unwrap());
    }
    let error = ready(&fx, 1, true, Duration::from_secs(10))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("timed out")
            && error.contains("effects=")
            && error.contains("caster_ready=true")
    );
    fx["active"] = serde_json::json!(0);
    assert!(!ready(&fx, 1, false, Duration::from_secs(4)).unwrap());
    assert!(ready(&fx, 1, false, Duration::from_secs(10)).is_err());
    assert!(ready(&fx, 1, true, Duration::from_secs(4)).unwrap());
    for (key, value) in [
        ("load_error", serde_json::json!("bad archive")),
        ("omitted", serde_json::json!(1)),
        ("started", serde_json::json!(2)),
        ("active", serde_json::json!(2)),
    ] {
        let mut bad = fx.clone();
        bad[key] = value;
        assert!(ready(&bad, 1, true, Duration::from_secs(4)).is_err());
    }
    let refused = serde_json::json!({"started":0,"active":0,"omitted":0,"load_error":null});
    assert!(ready(&refused, 0, true, Duration::from_secs(3)).unwrap());
}
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
        include_bytes!("fixtures/native-reagent-skills-current.json"),
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
    w.mp = Some([80, 80]);
    w.accept(&p("book")).unwrap();
    w
}

#[test]
fn formal_reagent_request_self_screen_boundary_and_no_prediction() {
    let mut w = world();
    let before = serde_json::to_value(&w).unwrap();
    assert_eq!(
        skills::ground_request(&w, 63, [32735, 32800]).unwrap(),
        p("request")
    );
    assert_eq!(
        skills::ground_request(&w, 63, [32729, 32797]).unwrap(),
        p("self")
    );
    assert!(skills::request(&w, 63, Some(2)).is_err());
    let oracle: serde_json::Value = serde_json::from_slice(include_bytes!(
        "fixtures/native-reagent-skills-current.json"
    ))
    .unwrap();
    let screen = oracle["screen_dx_dy_minus20_through20"]
        .as_str()
        .unwrap()
        .as_bytes();
    let mut i = 0;
    for dx in -20..=20 {
        for dy in -20..=20 {
            let point = [(32729 + dx) as u16, (32797 + dy) as u16];
            assert_eq!(
                skills::ground_request(&w, 63, point).is_ok(),
                screen[i] == b'1',
                "{dx},{dy}"
            );
            i += 1;
        }
    }
    assert_eq!(i, 1681);
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    let a = w.actors.get_mut(&1).unwrap();
    a.x = 65534;
    a.y = 0;
    assert_eq!(
        skills::ground_request(&w, 63, [65535, 0]).unwrap(),
        p("boundary")
    );
    assert!(!skills::in_native_screen([0, 0], [65535, 65535]));
    w.mp = Some([2, 80]); // The authoritative server decides effective cost and reagent eligibility.
    assert!(skills::ground_request(&w, 63, [65535, 0]).is_ok());
    for variation in 0..5 {
        let mut n = w.clone();
        match variation {
            0 => n.mp = Some([0, 80]),
            1 => n.hp = Some([0, 50]),
            2 => n.actors.get_mut(&1).unwrap().dead = true,
            3 => {
                n.learned_skills.clear();
            }
            _ => {
                n.status = None;
            }
        }
        assert!(skills::ground_request(&n, 63, [65535, 0]).is_err());
    }
}
#[test]
fn life_stream_choice_is_owned_ground_and_invalidated_by_correction() {
    let w = world();
    let mut m = Model::default();
    m.sync(9, &w, true);
    m.select(Some(63)).unwrap();
    let ground = Ground::capture(9, &w, WorldCell { x: 32729, y: 32797 });
    let choice = m.choice_at(9, &w, Target::capture(&w, 2), ground).unwrap();
    assert_eq!(choice.skill, 63);
    assert_eq!(choice.target, None);
    assert_eq!(choice.ground, Some([32729, 32797]));
    for variation in 0..5 {
        let mut n = w.clone();
        let mut epoch = 9;
        match variation {
            0 => epoch += 1,
            1 => n.generation += 1,
            2 => n.local_corrections += 1,
            3 => n.actors.get_mut(&1).unwrap().incarnation += 1,
            _ => {
                n.learned_skills.clear();
            }
        }
        m.sync(epoch, &n, true);
        if n.learned_skills.contains(&63) {
            m.select(Some(63)).unwrap();
        }
        assert!(m.choice_at(epoch, &n, None, ground).is_err());
    }
}
#[test]
fn formal_action19_effect_and_mana_keep_authority_atomic() {
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
    w.accept(&p("effect")).unwrap();
    assert_eq!(w.own_cast_packets, 1);
    assert_eq!(w.own_attack_packets, 0);
    assert_eq!(w.effects[0].graphics, 2231);
    assert_eq!(w.effects[0].target, None);
    assert_eq!(w.effects[0].position, [32735, 32800]);
    assert_eq!(w.mp, Some([80, 80]));
    assert_eq!(w.hp, Some([50, 50]));
    assert_eq!(w.actors[&2].hp_ratio, 100);
    w.accept(&p("mana")).unwrap();
    assert_eq!(w.mp, Some([60, 80]));
}
fn projection() -> ViewProjection {
    ViewProjection {
        anchor_cell: WorldCell { x: 32735, y: 32800 },
        anchor_pixel: PixelPoint { x: 400, y: 300 },
    }
}
#[test]
fn explicit_modern_original_action3_once_has_nineteen_frames_and_one_sound() {
    let mut w = world();
    w.accept(&p("effect")).unwrap();
    let unchanged = serde_json::to_value(&w).unwrap();
    let mut fx = Effects::reagent();
    fx.prepare(
        Path::new(r"D:\lineage開發\0-客戶端"),
        &w,
        &AtomicBool::new(false),
    );
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    std::fs::create_dir_all(&output).unwrap();
    let mut rows = vec![];
    let mut sounds = 0;
    for tick in (0..=76u64).step_by(4).chain([320 * 24]) {
        let now = (tick * 1_000_000_000).div_ceil(24);
        let mut c = Canvas::new(800, 600).unwrap();
        let (mut r, audio) = fx.draw(&w, &mut c, projection(), now);
        assert!(r["load_error"].is_null(), "{r}");
        assert_eq!(r["presentation"], "modern_explicit_action3_once");
        assert_eq!(r["started"], 1);
        assert_eq!(r["omitted"], 0);
        assert_eq!(r["active"], if tick < 76 { 1 } else { 0 });
        for e in r["effects"].as_array().unwrap() {
            assert_eq!(e["action"], 3);
            assert_eq!(e["source_frame"], if tick < 72 { tick / 4 + 1 } else { 0 });
            assert_eq!(e["drawn"], tick < 76);
        }
        for a in audio {
            assert_eq!(a.sound, Some(841));
            assert_eq!(a.action, 3);
            assert_eq!(a.frame, 1);
            sounds += 1;
        }
        assert_eq!(r["sources"].as_array().unwrap().len(), 1);
        r["tick"] = serde_json::json!(tick);
        r["rgba_sha256"] = serde_json::json!(format!("{:X}", Sha256::digest(c.rgba())));
        if [0, 36, 72, 76].contains(&tick) {
            let file = output.join(format!("native-reagent-effect-{tick}.png"));
            let mut e = png::Encoder::new(std::fs::File::create(&file).unwrap(), 800, 600);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header()
                .unwrap()
                .write_image_data(c.rgba())
                .unwrap();
            r["png"] = serde_json::json!(file);
        }
        rows.push(r);
        assert_eq!(serde_json::to_value(&w).unwrap(), unchanged);
    }
    assert_eq!(sounds, 1);
    assert_ne!(rows[18]["rgba_sha256"], rows[19]["rgba_sha256"]);
    std::fs::write(
        output.join("native-reagent-effects.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
#[test]
fn reagent_effect_is_fixed_bounded_reset_and_not_an_actor_effect() {
    let mut w = world();
    w.accept(&p("effect")).unwrap();
    let mut fx = Effects::reagent();
    let cancel = AtomicBool::new(false);
    fx.prepare(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel);
    let mut c = Canvas::new(800, 600).unwrap();
    assert_eq!(fx.draw(&w, &mut c, projection(), 0).1.len(), 1);
    w.actors.clear();
    let (r, a) = fx.draw(&w, &mut c, projection(), 2_000_000_000);
    assert_eq!(r["active"], 1);
    assert_eq!(
        r["effects"][0]["position"],
        serde_json::json!([32735, 32800])
    );
    assert!(a.is_empty());
    let (r, a) = fx.draw(&w, &mut c, projection(), 4_000_000_000);
    assert_eq!(r["active"], 0);
    assert!(a.is_empty());
    w.generation += 1;
    w.effects.clear();
    let (r, a) = fx.draw(&w, &mut c, projection(), 5_000_000_000);
    assert_eq!(r["started"], 0);
    assert!(a.is_empty());
    for sequence in 1..=66 {
        w.effects.push(skills::Effect {
            sequence,
            graphics: 2231,
            target: if sequence == 1 { Some((1, 1)) } else { None },
            position: [32735, 32800],
        });
    }
    let (r, a) = fx.draw(&w, &mut c, projection(), 6_000_000_000);
    assert_eq!(r["active"], 64);
    assert_eq!(r["omitted"], 2);
    assert_eq!(a.len(), 64);
    let (r, a) = fx.draw(&w, &mut c, projection(), 6_000_000_000);
    assert_eq!(r["started"], 64);
    assert!(a.is_empty());
    for mut other in [Effects::default(), Effects::items()] {
        other.prepare(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel);
        let (r, a) = other.draw(&w, &mut c, projection(), 0);
        assert_eq!(r["omitted"], 0);
        assert_eq!(r["started"], 0);
        assert!(a.is_empty());
    }
}
#[test]
fn world_scene_prepares_original_sound_once_and_preserves_server_state() {
    use lito_modern::modern_lito::native_world_scene::Model as Scene;
    let mut w = world();
    let cancel = AtomicBool::new(false);
    let mut scene = Scene::load(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel).unwrap();
    w.accept(&p("effect")).unwrap();
    let unchanged = serde_json::to_value(&w).unwrap();
    let mut audio = vec![];
    for now in [
        0,
        500_000_000,
        3_100_000_000,
        3_200_000_000,
        320_000_000_000,
    ] {
        let f = scene.draw_at(&w, &cancel, now).unwrap();
        assert_eq!(f.receipt["reagent_effects"]["started"], 1);
        assert_eq!(f.receipt["reagent_effects"]["omitted"], 0);
        for a in f.receipt["audio"].as_array().unwrap() {
            if a["event"]["sound"] == 841 {
                assert_eq!(a["prepared"], true);
                assert_ne!(a["same_id_suppressed"], true);
                audio.push(a.clone());
            }
        }
        assert_eq!(serde_json::to_value(&w).unwrap(), unchanged);
    }
    assert_eq!(audio.len(), 1);
    std::fs::write(
        "target/current/verification/native-reagent-audio.json",
        serde_json::to_vec_pretty(&audio).unwrap(),
    )
    .unwrap();
}
