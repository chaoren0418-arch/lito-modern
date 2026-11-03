use lito_modern::modern_lito::{native_combat as combat, native_world::World};
fn packet(data: &[u8], key: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_slice(data).unwrap();
    let hex = v["packets"][key].as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn p(key: &str) -> Vec<u8> {
    packet(include_bytes!("fixtures/native-ranged-current.json"), key)
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
    w.actors.insert(1, own);
    w.status.as_mut().unwrap().object_id = 1;
    w
}
#[test]
fn current_jar_far_attack_vectors_preserve_native_unsigned_fields() {
    assert_eq!(
        combat::attack_request(2, 32741, 32801, true).unwrap(),
        p("request")
    );
    let a = combat::decode_attack(&p("hit")).unwrap().unwrap();
    assert_eq!(
        (a.action, a.attacker, a.target, a.damage_marker, a.heading),
        (21, 1, 2, 5, 3)
    );
    let shot = a.projectile.unwrap();
    assert_eq!(
        (shot.id, shot.graphics, shot.from, shot.to),
        (17, 66, [32729, 32797], [32741, 32801])
    );
    let a = combat::decode_attack(&p("boundary")).unwrap().unwrap();
    assert_eq!(a.attacker, u32::MAX);
    let shot = a.projectile.unwrap();
    assert_eq!((shot.id, shot.graphics), (u32::MAX, u16::MAX));
    let a = combat::decode_attack(&p("miss")).unwrap().unwrap();
    assert_eq!((a.target, a.damage_marker), (0, 0));
    let a = combat::decode_attack(&p("hit_zero")).unwrap().unwrap();
    assert_eq!((a.target, a.damage_marker), (2, 0));
    let mut w = world();
    w.actors.get_mut(&1).unwrap().x = 32729;
    w.actors.get_mut(&1).unwrap().y = 32797;
    w.actors.get_mut(&2).unwrap().x = 32741;
    w.actors.get_mut(&2).unwrap().y = 32801;
    assert!(w.attack_request(2).is_err());
    w.actors.get_mut(&1).unwrap().walk_action = 20;
    assert_eq!(w.attack_request(2).unwrap(), p("request"));
    w.actors.get_mut(&1).unwrap().walk_action = 62;
    assert_eq!(w.attack_request(2).unwrap(), p("request"));
}
#[test]
fn malformed_ranged_packets_are_atomic_and_other_effect_branches_remain_unknown() {
    let mut w = world();
    let before = serde_json::to_value(&w).unwrap();
    let b = p("hit");
    for end in 0..b.len() {
        assert!(w.accept(&b[..end]).is_err(), "truncation {end}");
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    for offset in [11, 16, 27, 28, 29] {
        let mut bad = b.clone();
        match offset {
            11 => bad[11] = 8,
            16 => {
                bad[16] = 0;
                bad[17] = 0;
            }
            29 => bad.push(0),
            _ => bad[offset] = 1,
        }
        assert!(w.accept(&bad).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    let mut bad = p("miss");
    bad[10] = 1;
    assert!(w.accept(&bad).is_err());
    let mut unknown = b;
    unknown[18] = 0x01;
    assert!(combat::decode_attack(&unknown).unwrap().is_none());
}
#[test]
fn ranged_authority_acknowledges_and_queues_launch_without_inventing_hp() {
    let mut w = world();
    let hp = w.hp;
    let ratio = w.actors[&2].hp_ratio;
    w.accept(&p("hit")).unwrap();
    assert_eq!((w.attack_packets, w.own_attack_packets), (1, 1));
    assert_eq!(w.projectiles.len(), 1);
    assert_eq!(
        w.projectiles[0].action_sequence,
        w.actors[&1].action_sequence
    );
    assert_eq!(w.actors[&1].actions.last().unwrap().action, 21);
    assert!(w.actors[&2].actions.is_empty(), "hurt waits for arrival");
    assert_eq!((w.hp, w.actors[&2].hp_ratio), (hp, ratio));
    let target_actions = w.actors[&2].actions.len();
    w.accept(&p("miss")).unwrap();
    assert_eq!(w.actors[&2].actions.len(), target_actions);
    assert_eq!(w.projectiles[1].target, 0);
    w.accept(&[0x28, 0x34, 2, 0]).unwrap();
    assert!(w.projectiles.is_empty());
    assert_eq!(w.own_attack_packets, 0);
}
#[test]
fn ranged_cues_have_a_visible_budget_and_orphan_attackers_do_not_spawn_effects() {
    let mut w = world();
    for id in 1u32..=129 {
        let mut b = p("hit");
        b[12..16].copy_from_slice(&id.to_le_bytes());
        w.accept(&b).unwrap();
    }
    assert_eq!(w.projectiles.len(), 128);
    assert_eq!(w.projectile_cues_dropped, 1);
    assert_eq!(w.projectiles[0].projectile.id, 2);
    w.actors.remove(&1);
    w.accept(&p("hit")).unwrap();
    assert_eq!(w.projectiles.len(), 128);
}
#[test]
fn original_arrow_follows_actor_hit_event_and_finishes_without_hp_mutation() {
    use lito_modern::modern_lito::native_world_scene::Model;
    use std::{path::Path, sync::atomic::AtomicBool};
    let mut w = world();
    let own = w.actors.get_mut(&1).unwrap();
    own.x = 32729;
    own.y = 32797;
    own.speed = 0;
    own.brave = 0;
    w.accept(&p("hit")).unwrap();
    let before = serde_json::to_value(&w).unwrap();
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel).unwrap();
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    let mut frames = Vec::new();
    let mut flying = 0;
    let mut arrivals = 0;
    let mut damage_frames = 0;
    for ms in (0..=1500).step_by(50) {
        let frame = model.draw_at(&w, &cancel, ms * 1_000_000).unwrap();
        let mut receipt = frame.receipt;
        assert!(receipt["projectiles"]["load_error"].is_null());
        let callbacks = receipt["projectiles"]["hurt_callbacks"].as_array().unwrap();
        arrivals += callbacks.len();
        for cb in callbacks {
            assert_eq!(cb["accepted"], true);
        }
        for actor in receipt["actors"].as_array().unwrap() {
            if actor["id"] == 2 && actor["phase"] == "damage" {
                assert_eq!(arrivals, 1, "hurt before arrow arrival");
                damage_frames += 1;
            }
        }
        if receipt["projectiles"]["flights"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["drawn"] == true)
        {
            flying += 1;
            if ms % 100 == 0 {
                let path = output.join(format!("native-arrow66-{ms}.png"));
                let mut enc = png::Encoder::new(std::fs::File::create(&path).unwrap(), 800, 600);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                enc.write_header()
                    .unwrap()
                    .write_image_data(frame.canvas.rgba())
                    .unwrap();
                receipt["png"] = serde_json::json!(path);
            }
        }
        frames.push(receipt);
    }
    assert!(flying >= 2, "arrow never flew");
    assert_eq!(arrivals, 1);
    assert!(damage_frames > 0);
    assert_eq!(frames.last().unwrap()["projectiles"]["launched"], 1);
    assert_eq!(frames.last().unwrap()["projectiles"]["active"], 0);
    assert_eq!(
        frames.last().unwrap()["projectiles"]["sources"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    std::fs::write(output.join("native-arrow66.json"),serde_json::to_vec_pretty(&serde_json::json!({"scope":"formal packet vector + original resources; not a TCP session","frames":frames,"hp_unchanged":true})).unwrap()).unwrap();
}
#[test]
fn projectile_launch_is_once_and_retired_on_actor_removal_or_new_world() {
    use lito_modern::modern_lito::{
        compositor::Canvas,
        native_combat_animation::EnteredEvent,
        native_projectiles::Projectiles,
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    use std::{path::Path, sync::atomic::AtomicBool};
    let mut w = world();
    w.accept(&p("hit")).unwrap();
    let hit = EnteredEvent {
        actor: 1,
        sequence: w.projectiles[0].action_sequence,
        action: 21,
        frame: 1,
        event: "Hit".into(),
        sound: None,
        prepared_audio: None,
    };
    let mut effects = Projectiles::default();
    effects.update(&w, std::iter::once(&hit), 0);
    effects.update(&w, std::iter::once(&hit), 1);
    w.actors.remove(&2);
    effects.update(&w, std::iter::empty(), 2);
    let mut canvas = Canvas::new(10, 10).unwrap();
    let project = ViewProjection {
        anchor_cell: WorldCell { x: 32729, y: 32797 },
        anchor_pixel: PixelPoint { x: 0, y: 0 },
    };
    let r = effects.draw(
        Path::new(r"D:\lineage開發\0-客戶端"),
        &AtomicBool::new(false),
        &mut canvas,
        project,
        2,
    );
    assert_eq!(r["launched"], 1);
    assert_eq!(r["active"], 0);
    assert!(r["sources"].is_null());
    w.accept(&[0x28, 0x34, 2, 0]).unwrap();
    effects.update(&w, std::iter::once(&hit), 3);
    let r = effects.draw(
        Path::new(r"D:\lineage開發\0-客戶端"),
        &AtomicBool::new(false),
        &mut canvas,
        project,
        3,
    );
    assert_eq!(r["launched"], 0);
    assert_eq!(r["active"], 0);
}
#[test]
fn zero_damage_hit_freezes_live_target_at_launch_and_never_retargets() {
    use lito_modern::modern_lito::{
        compositor::Canvas,
        native_combat_animation::EnteredEvent,
        native_projectiles::Projectiles,
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    use std::{path::Path, sync::atomic::AtomicBool};
    let mut w = world();
    w.accept(&p("hit_zero")).unwrap();
    let target = w.actors.get_mut(&2).unwrap();
    target.x = 32741;
    target.y = 32802;
    let hit = EnteredEvent {
        actor: 1,
        sequence: w.projectiles[0].action_sequence,
        action: 21,
        frame: 1,
        event: "Hit".into(),
        sound: None,
        prepared_audio: None,
    };
    let mut effects = Projectiles::default();
    effects.update(&w, std::iter::once(&hit), 0);
    w.actors.get_mut(&2).unwrap().y = 32803;
    effects.update(&w, std::iter::once(&hit), 1);
    let mut canvas = Canvas::new(10, 10).unwrap();
    let r = effects.draw(
        Path::new(r"D:\lineage開發\0-客戶端"),
        &AtomicBool::new(false),
        &mut canvas,
        ViewProjection {
            anchor_cell: WorldCell { x: 32729, y: 32797 },
            anchor_pixel: PixelPoint { x: 0, y: 0 },
        },
        1,
    );
    assert!(r["load_error"].is_null());
    assert_eq!(r["launched"], 1);
    assert_eq!(r["flights"][0]["to"], serde_json::json!([32741, 32802]));
}

fn launch_event(w: &World) -> lito_modern::modern_lito::native_combat_animation::EnteredEvent {
    lito_modern::modern_lito::native_combat_animation::EnteredEvent {
        actor: 1,
        sequence: w.projectiles.last().unwrap().action_sequence,
        action: 21,
        frame: 1,
        event: "Hit".into(),
        sound: None,
        prepared_audio: None,
    }
}
fn ranged_world(key: &str) -> World {
    let mut w = world();
    let target = w.actors.get_mut(&2).unwrap();
    target.x = 32741;
    target.y = 32801;
    w.accept(&p(key)).unwrap();
    w
}
#[test]
fn arrival_is_once_at_the_original_30ms_terminal_callback_even_when_frames_skip() {
    use lito_modern::modern_lito::native_projectiles::Projectiles;
    for arrival_time in [330_000_000, 10_000_000_000] {
        let w = ranged_world("hit");
        let mut effects = Projectiles::default();
        let event = launch_event(&w);
        assert!(effects.update(&w, std::iter::once(&event), 0).is_empty());
        assert!(effects
            .update(&w, std::iter::empty(), 329_999_999)
            .is_empty());
        let impacts = effects.update(&w, std::iter::empty(), arrival_time);
        assert_eq!(impacts.len(), 1);
        assert_eq!((impacts[0].target, impacts[0].projectile), (2, 17));
        assert!(effects
            .update(&w, std::iter::once(&event), arrival_time + 1)
            .is_empty());
    }
    // First callback already arrives for an adjacent target.
    let mut w = ranged_world("hit");
    w.actors.get_mut(&2).unwrap().x = 32730;
    w.actors.get_mut(&2).unwrap().y = 32797;
    let mut effects = Projectiles::default();
    assert_eq!(
        effects
            .update(&w, std::iter::once(&launch_event(&w)), 0)
            .len(),
        1
    );
}
#[test]
fn no_hurt_for_miss_zero_damage_death_removed_or_replaced_objects() {
    use lito_modern::modern_lito::native_projectiles::Projectiles;
    for case in [
        "miss",
        "hit_zero",
        "dead",
        "removed",
        "target_replaced",
        "attacker_replaced",
        "new_world",
    ] {
        let mut w = ranged_world(if case == "miss" || case == "hit_zero" {
            case
        } else {
            "hit"
        });
        let mut effects = Projectiles::default();
        assert!(effects
            .update(&w, std::iter::once(&launch_event(&w)), 0)
            .is_empty());
        match case {
            "dead" => {
                w.accept(&[0x20, 2, 0, 0, 0, 8]).unwrap();
            }
            "removed" => {
                w.accept(&[0x15, 2, 0, 0, 0]).unwrap();
            }
            "target_replaced" | "attacker_replaced" => {
                let id = if case == "target_replaced" { 2u32 } else { 1 };
                let before = w.actors[&id].incarnation;
                let mut put = packet(include_bytes!("fixtures/native-world-current.json"), "bot");
                put[5..9].copy_from_slice(&id.to_le_bytes());
                w.accept(&put).unwrap();
                assert_ne!(w.actors[&id].incarnation, before);
            }
            "new_world" => {
                w.accept(&[0x28, 0x34, 2, 0]).unwrap();
            }
            _ => {}
        }
        assert!(
            effects
                .update(&w, std::iter::empty(), 10_000_000_000)
                .is_empty(),
            "{case}"
        );
    }
}
#[test]
fn same_id_replacement_before_launch_cannot_bind_an_old_cue() {
    use lito_modern::modern_lito::native_projectiles::Projectiles;
    for id in [1u32, 2] {
        let mut w = ranged_world("hit");
        let event = launch_event(&w);
        let mut put = packet(include_bytes!("fixtures/native-world-current.json"), "bot");
        put[5..9].copy_from_slice(&id.to_le_bytes());
        w.accept(&put).unwrap();
        let mut effects = Projectiles::default();
        assert!(effects.update(&w, std::iter::once(&event), 0).is_empty());
        assert!(effects
            .update(&w, std::iter::empty(), 10_000_000_000)
            .is_empty());
    }
}
