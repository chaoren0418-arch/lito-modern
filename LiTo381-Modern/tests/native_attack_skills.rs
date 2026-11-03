use lito_modern::modern_lito::{
    native_combat as combat, native_skills as skills, native_world::World,
};
fn packet(bytes: &[u8], key: &str) -> Vec<u8> {
    let d: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let s = d["packets"][key].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn p(key: &str) -> Vec<u8> {
    packet(
        include_bytes!("fixtures/native-attack-skills-current.json"),
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
    let mut own = w.actors.remove(&0xfedcba98).unwrap();
    own.id = 1;
    own.x = 32729;
    own.y = 32797;
    w.actors.insert(1, own);
    w.status.as_mut().unwrap().object_id = 1;
    let t = w.actors.get_mut(&2).unwrap();
    t.x = 32735;
    t.y = 32800;
    w.hp = Some([50, 50]);
    w.mp = Some([10, 10]);
    w.accept(&p("book")).unwrap();
    w
}
#[test]
fn formal_attack_spell_layout_and_request_guards() {
    let mut w = world();
    assert_eq!(skills::request(&w, 4, Some(2)).unwrap(), p("request"));
    assert!(skills::request(&w, 4, None).is_err());
    assert!(skills::request(&w, 4, Some(1)).is_err());
    w.actors.get_mut(&2).unwrap().x = 32740;
    assert!(skills::request(&w, 4, Some(2)).is_err());
    let a = combat::decode_attack(&p("hit")).unwrap().unwrap();
    let shot = a.projectile.unwrap();
    assert_eq!((a.action, shot.flight_type, shot.graphics), (18, 0, 167));
    let a = combat::decode_attack(&p("boundary")).unwrap().unwrap();
    assert_eq!(a.attacker, u32::MAX);
    assert_eq!(a.projectile.unwrap().id, u32::MAX);
    for key in ["hit", "miss", "zero", "adjacent"] {
        assert!(combat::decode_attack(&p(key)).unwrap().is_some());
    }
}
#[test]
fn spell_authority_is_atomic_and_does_not_preempt_arrival_hurt() {
    let mut w = world();
    let before = serde_json::to_value(&w).unwrap();
    let b = p("hit");
    for n in 0..b.len() {
        assert!(w.accept(&b[..n]).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    w.accept(&b).unwrap();
    assert_eq!(w.own_cast_packets, 1);
    assert_eq!(w.own_attack_packets, 0);
    assert!(w.actors[&2].actions.is_empty());
    assert_eq!(w.mp, Some([10, 10]));
    assert_eq!(w.hp, Some([50, 50]));
    let pending = skills::Pending {
        since: std::time::Instant::now(),
        casts: 0,
        messages: 0,
        generation: w.generation,
        incarnation: w.actors[&1].incarnation,
    };
    assert_eq!(
        pending.observe(&w, std::time::Instant::now()),
        skills::Feedback::CastAction
    );
}
#[test]
fn original_class4_flight_impact_audio_and_hurt_are_once() {
    use lito_modern::modern_lito::native_world_scene::Model;
    use std::{path::Path, sync::atomic::AtomicBool};
    let cancel = AtomicBool::new(false);
    let mut w = world();
    let mut model = Model::load(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel).unwrap();
    model.draw_at(&w, &cancel, 0).unwrap();
    w.accept(&p("hit")).unwrap();
    let authority = serde_json::to_value(&w).unwrap();
    let mut frames = Vec::new();
    let mut callbacks = Vec::new();
    let mut sound97 = 0;
    let mut gfx = std::collections::BTreeSet::new();
    for ms in (0..=3000).step_by(50) {
        let f = model.draw_at(&w, &cancel, ms * 1_000_000).unwrap();
        let mut r = f.receipt;
        let s = &r["spell_projectiles"];
        assert!(s["load_error"].is_null());
        assert_eq!(s["omitted"], 0);
        for c in s["hurt_callbacks"].as_array().unwrap() {
            assert_eq!(c["accepted"], true);
            callbacks.push(ms);
        }
        for e in s["entered_events"].as_array().unwrap() {
            if e["sound"] == 97 {
                sound97 += 1;
            }
        }
        for f in s["flights"].as_array().unwrap() {
            if f["drawn"] == true {
                gfx.insert(f["graphics"].as_u64().unwrap());
            }
        }
        for a in r["actors"].as_array().unwrap() {
            if a["id"] == 2 && a["phase"] == "damage" {
                assert_eq!(callbacks.len(), 1);
            }
        }
        if [0, 250, 500, 750, 1000, 1500, 2000, 2450, 3000].contains(&ms) {
            let p = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("target/current/verification/native-bolt-{ms}.png"));
            let mut e = png::Encoder::new(std::fs::File::create(&p).unwrap(), 800, 600);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header()
                .unwrap()
                .write_image_data(f.canvas.rgba())
                .unwrap();
            r["png"] = serde_json::json!(p);
        }
        frames.push(r);
    }
    assert_eq!(callbacks.len(), 1);
    assert_eq!(sound97, 1);
    assert_eq!(gfx, std::collections::BTreeSet::from([167, 219]));
    assert_eq!(frames.last().unwrap()["spell_projectiles"]["active"], 0);
    assert_eq!(serde_json::to_value(&w).unwrap(), authority);
    std::fs::write(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-bolt.json"),
        serde_json::to_vec_pretty(&frames).unwrap(),
    )
    .unwrap();
}
#[test]
fn class4_launch_freezes_target_and_retires_reused_identity() {
    use lito_modern::modern_lito::{
        compositor::Canvas,
        native_combat_animation::EnteredEvent,
        native_spell_projectiles::Projectiles,
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    let mut w = world();
    w.accept(&p("hit")).unwrap();
    let cue = w.projectiles[0].clone();
    let mut engine = Projectiles::default();
    engine.prepare(
        std::path::Path::new(r"D:\lineage開發\0-客戶端"),
        &w,
        &std::sync::atomic::AtomicBool::new(false),
    );
    let e = EnteredEvent {
        actor: 1,
        sequence: cue.action_sequence,
        action: 18,
        frame: 1,
        event: "Hit".into(),
        sound: None,
        prepared_audio: None,
    };
    assert!(engine.update(&w, [&e].into_iter(), 0).is_empty());
    w.actors.get_mut(&2).unwrap().x += 1;
    let mut canvas = Canvas::new(800, 600).unwrap();
    let projection = ViewProjection {
        anchor_cell: WorldCell { x: 32729, y: 32797 },
        anchor_pixel: PixelPoint { x: 400, y: 300 },
    };
    let (r, _) = engine.draw(&mut canvas, projection, 60_000_000);
    assert_eq!(r["flights"][0]["to"], serde_json::json!([32735, 32800]));
    engine.update(&w, [&e].into_iter(), 60_000_000);
    let (r, _) = engine.draw(&mut canvas, projection, 60_000_000);
    assert_eq!(r["launched"], 1);
    w.actors.get_mut(&2).unwrap().incarnation += 1;
    assert!(engine
        .update(&w, std::iter::empty(), 120_000_000)
        .is_empty());
    let (r, _) = engine.draw(&mut canvas, projection, 120_000_000);
    assert_eq!(r["active"], 0);
    w.generation += 1;
    engine.update(&w, std::iter::empty(), 180_000_000);
    let (r, _) = engine.draw(&mut canvas, projection, 180_000_000);
    assert_eq!(r["launched"], 0);
}
#[test]
fn class4_zero_distance_zero_damage_and_miss_keep_authority() {
    use lito_modern::modern_lito::{
        compositor::Canvas,
        native_combat_animation::EnteredEvent,
        native_spell_projectiles::Projectiles,
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    let mut engine = Projectiles::default();
    let projection = ViewProjection {
        anchor_cell: WorldCell { x: 32729, y: 32797 },
        anchor_pixel: PixelPoint { x: 400, y: 300 },
    };
    let mut canvas = Canvas::new(800, 600).unwrap();
    for (index, key) in ["hit", "zero", "miss"].into_iter().enumerate() {
        let mut w = world();
        w.generation = index as u64 + 1;
        let t = w.actors.get_mut(&2).unwrap();
        t.x = 32729;
        t.y = 32797;
        w.accept(&p(key)).unwrap();
        let authority = serde_json::to_value(&w).unwrap();
        engine.prepare(
            std::path::Path::new(r"D:\lineage開發\0-客戶端"),
            &w,
            &std::sync::atomic::AtomicBool::new(false),
        );
        let e = EnteredEvent {
            actor: 1,
            sequence: w.projectiles[0].action_sequence,
            action: 18,
            frame: 1,
            event: "Hit".into(),
            sound: None,
        prepared_audio: None,
        };
        let impacts = engine.update(&w, [&e].into_iter(), 0);
        assert_eq!(impacts.len(), usize::from(key == "hit"));
        assert!(engine
            .update(&w, std::iter::empty(), 3_000_000_000)
            .is_empty());
        let (r, events) = engine.draw(&mut canvas, projection, 3_000_000_000);
        assert_eq!(r["arrivals"], 1);
        assert_eq!(r["active"], 0);
        assert_eq!(events.iter().filter(|e| e.sound == Some(97)).count(), 1);
        assert!(engine
            .draw(&mut canvas, projection, 3_000_000_000)
            .1
            .is_empty());
        assert_eq!(serde_json::to_value(&w).unwrap(), authority);
    }
}


// Current C02 skill77 request/selection contract; server retains MP/lawful authority.
mod disintegrate_request_contract {
use lito_modern::modern_lito::{
    native_skill_selection::{self, Model as Selection, Target},
    native_skills,
    native_world::World,
};
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn world() -> World {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for name in ["world", "start", "status", "actor", "hp", "mp"] {
        assert!(w
            .accept(&hex(fixture["packets"][name].as_str().unwrap()))
            .unwrap());
    }
    let own = w.status.unwrap().object_id;
    w.actors.get_mut(&own).unwrap().x = 32768;
    w.actors.get_mut(&own).unwrap().y = 32768;
    let mut t = w.actors[&own].clone();
    t.id = 200;
    t.x = 32770;
    t.y = 32769;
    t.incarnation += 1;
    w.actors.insert(200, t);
    w.learned_skills.insert(77);
    assert!(w.ready());
    w
}
#[test]
fn current_c02_exact_object_coordinates_and_readonly_authority() {
    let w = world();
    let before = serde_json::to_value(&w).unwrap();
    let expected = hex("140904c800000002800180");
    assert_eq!(native_skills::request(&w, 77, Some(200)).unwrap(), expected);
    assert_eq!(expected.len(), 11);
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
}
#[test]
fn range_ten_boundary_and_target_changes_use_current_received_coordinates() {
    let mut w = world();
    let own = w.own_actor().unwrap().id;
    let x = w.actors[&own].x;
    w.actors.get_mut(&200).unwrap().x = x + 10;
    assert!(native_skills::request(&w, 77, Some(200)).is_ok());
    w.actors.get_mut(&200).unwrap().x = x + 11;
    assert!(native_skills::request(&w, 77, Some(200)).is_err());
    w.actors.get_mut(&200).unwrap().x = x - 10;
    let b = native_skills::request(&w, 77, Some(200)).unwrap();
    assert_eq!(&b[7..9], &(x - 10).to_le_bytes());
}
#[test]
fn invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests() {
    let w = world();
    let own = w.own_actor().unwrap().id;
    for mode in 0..11 {
        let mut v = w.clone();
        let target = match mode {
            0 => None,
            1 => Some(own),
            _ => Some(200),
        };
        match mode {
            2 => {
                v.actors.remove(&200);
            }
            3 => v.actors.get_mut(&200).unwrap().dead = true,
            4 => {
                v.learned_skills.remove(&77);
            }
            5 => v.hp = Some([0, 100]),
            6 => v.mp = Some([0, 100]),
            7 => {
                v.movement_blocks.insert(1);
            }
            8 => v.actors.get_mut(&own).unwrap().dead = true,
            9 => v.started = false,
            10 => v.actors.get_mut(&200).unwrap().draw_state.invisible = true,
            _ => {}
        }
        let before = serde_json::to_value(&v).unwrap();
        assert!(
            native_skills::request(&v, 77, target).is_err(),
            "mode {mode}"
        );
        assert_eq!(serde_json::to_value(&v).unwrap(), before);
    }
}
#[test]
fn mp_and_lawful_are_server_permission_inputs_and_never_locally_spent() {
    let mut w = world();
    let own = w.own_actor().unwrap().id;
    w.mp = Some([1, 100]);
    w.actors.get_mut(&own).unwrap().lawful = 0;
    let before = serde_json::to_value(&w).unwrap();
    assert!(native_skills::request(&w, 77, Some(200)).is_ok());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    assert_eq!(native_skills::reuse_millis(77), 5000);
    assert_eq!(native_skills::client_clock_millis(77), 0);
}
#[test]
fn selected_learned_skill_has_native_label_and_rejects_stale_life_world_or_connection() {
    let mut w = world();
    let target = Target::capture(&w, 200).unwrap();
    let mut selection = Selection::default();
    selection.sync(1, &w, true);
    let row = selection.rows().iter().find(|row| row.id == 77).unwrap();
    assert!(row.label.contains("究極光裂術"));
    assert!(!row.label.contains("尚未支援"));
    selection.select(Some(77)).unwrap();
    assert_eq!(selection.choice(1, &w, Some(target)).unwrap().skill, 77);
    assert!(selection.choice(2, &w, Some(target)).is_err());
    w.actors.get_mut(&200).unwrap().incarnation += 1;
    assert!(native_skill_selection::choice(&w, 77, Some(target)).is_err());
    w.generation += 1;
    assert!(selection.choice(1, &w, Some(target)).is_err());
}

}

// Captured current C02 lawful refusal; no local permission or resource prediction.
#[test]
fn captured_c02_disintegrate_lawful352_is_a_rejection_without_authority_or_cast() {
    use std::time::{Instant,Duration};
    let mut w=world();
    let now=Instant::now();
    let pending=skills::Pending { since:now, casts:w.own_cast_packets, messages:w.message_count, generation:w.generation, incarnation:w.own_actor().unwrap().incarnation };
    let before=(w.hp,w.mp,w.own_cast_packets,serde_json::to_value(&w.projectiles).unwrap());
    assert!(w.accept(&[0x10,0x60,0x01,0x01,b'$',b'9',b'6',b'7',0]).unwrap());
    assert_eq!(w.last_message,Some((352,vec!["$967".into()])));
    assert_eq!((w.hp,w.mp,w.own_cast_packets,serde_json::to_value(&w.projectiles).unwrap()),before);
    assert_eq!(pending.observe(&w,now+Duration::from_secs(3)),skills::Feedback::Rejected(352));
    assert_eq!(w.last_skill_rejection,Some((pending.messages+1,352)));
}

mod call_lightning_request_contract {
use lito_modern::modern_lito::{
    native_skill_selection::{self, Model as Selection, Target},
    native_skills,
    native_world::World,
};
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn world() -> World {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for name in ["world", "start", "status", "actor", "hp", "mp"] {
        assert!(w
            .accept(&hex(fixture["packets"][name].as_str().unwrap()))
            .unwrap());
    }
    let own = w.status.unwrap().object_id;
    w.actors.get_mut(&own).unwrap().x = 32768;
    w.actors.get_mut(&own).unwrap().y = 32768;
    let mut t = w.actors[&own].clone();
    t.id = 200;
    t.x = 32770;
    t.y = 32769;
    t.incarnation += 1;
    w.actors.insert(200, t);
    w.learned_skills.insert(34);
    assert!(w.ready());
    w
}
#[test]
fn current_c02_exact_object_coordinates_and_readonly_authority() {
    let w = world();
    let before = serde_json::to_value(&w).unwrap();
    let expected = hex("140401c800000002800180");
    assert_eq!(native_skills::request(&w, 34, Some(200)).unwrap(), expected);
    assert_eq!(expected.len(), 11);
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
}
#[test]
fn range_six_boundary_and_target_changes_use_current_received_coordinates() {
    let mut w = world();
    let own = w.own_actor().unwrap().id;
    let x = w.actors[&own].x;
    w.actors.get_mut(&200).unwrap().x = x + 6;
    assert!(native_skills::request(&w, 34, Some(200)).is_ok());
    w.actors.get_mut(&200).unwrap().x = x + 7;
    assert!(native_skills::request(&w, 34, Some(200)).is_err());
    w.actors.get_mut(&200).unwrap().x = x - 6;
    let b = native_skills::request(&w, 34, Some(200)).unwrap();
    assert_eq!(&b[7..9], &(x - 6).to_le_bytes());
}
#[test]
fn invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests() {
    let w = world();
    let own = w.own_actor().unwrap().id;
    for mode in 0..11 {
        let mut v = w.clone();
        let target = match mode {
            0 => None,
            1 => Some(own),
            _ => Some(200),
        };
        match mode {
            2 => {
                v.actors.remove(&200);
            }
            3 => v.actors.get_mut(&200).unwrap().dead = true,
            4 => {
                v.learned_skills.remove(&34);
            }
            5 => v.hp = Some([0, 100]),
            6 => v.mp = Some([0, 100]),
            7 => {
                v.movement_blocks.insert(1);
            }
            8 => v.actors.get_mut(&own).unwrap().dead = true,
            9 => v.started = false,
            10 => v.actors.get_mut(&200).unwrap().draw_state.invisible = true,
            _ => {}
        }
        let before = serde_json::to_value(&v).unwrap();
        assert!(
            native_skills::request(&v, 34, target).is_err(),
            "mode {mode}"
        );
        assert_eq!(serde_json::to_value(&v).unwrap(), before);
    }
}
#[test]
fn mp_and_lawful_are_server_permission_inputs_and_never_locally_spent() {
    let mut w = world();
    let own = w.own_actor().unwrap().id;
    w.mp = Some([1, 100]);
    w.actors.get_mut(&own).unwrap().lawful = 0;
    let before = serde_json::to_value(&w).unwrap();
    assert!(native_skills::request(&w, 34, Some(200)).is_ok());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    assert_eq!(native_skills::reuse_millis(34), 10);
    assert_eq!(native_skills::client_clock_millis(34), 0);
}
#[test]
fn selected_learned_skill_has_native_label_and_rejects_stale_life_world_or_connection() {
    let mut w = world();
    let target = Target::capture(&w, 200).unwrap();
    let mut selection = Selection::default();
    selection.sync(1, &w, true);
    let row = selection.rows().iter().find(|row| row.id == 34).unwrap();
    assert!(row.label.contains("極道落雷"));
    assert!(!row.label.contains("尚未支援"));
    selection.select(Some(34)).unwrap();
    assert_eq!(selection.choice(1, &w, Some(target)).unwrap().skill, 34);
    assert!(selection.choice(2, &w, Some(target)).is_err());
    w.actors.get_mut(&200).unwrap().incarnation += 1;
    assert!(native_skill_selection::choice(&w, 34, Some(target)).is_err());
    w.generation += 1;
    assert!(selection.choice(1, &w, Some(target)).is_err());
}

}
