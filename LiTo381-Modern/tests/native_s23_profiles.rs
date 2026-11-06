use lito_modern::modern_lito::{
    combat_animation::NativeAudioMode, native_combat::decode_attack,
    native_combat_animation::EnteredEvent, native_projectiles::Projectiles, native_world::World,
    native_world_scene::Model,
};
use std::{path::Path, sync::atomic::AtomicBool};

fn packet(data: &[u8], key: &str) -> Vec<u8> {
    let json: serde_json::Value = serde_json::from_slice(data).unwrap();
    let hex = json["packets"][key].as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn p(key: &str) -> Vec<u8> {
    packet(include_bytes!("fixtures/native-ranged-current.json"), key)
}
fn world(key: &str) -> World {
    let mut world = World::default();
    for name in ["world", "start", "status", "actor", "bot"] {
        world
            .accept(&packet(
                include_bytes!("fixtures/native-world-current.json"),
                name,
            ))
            .unwrap();
    }
    let mut own = world.actors.remove(&0xfedcba98).unwrap();
    own.id = 1;
    own.x = 32729;
    own.y = 32797;
    own.speed = 0;
    own.brave = 0;
    world.actors.insert(1, own);
    world.status.as_mut().unwrap().object_id = 1;
    let target = world.actors.get_mut(&2).unwrap();
    target.x = 32741;
    target.y = 32801;
    world.accept(&p(key)).unwrap();
    world
}
fn hit(world: &World) -> EnteredEvent {
    EnteredEvent {
        actor: 1,
        sequence: world.projectiles.last().unwrap().action_sequence,
        action: 21,
        frame: 1,
        event: "Hit".into(),
        sound: None,
        prepared_audio: None,
    }
}

#[test]
fn native_variable_tail_preserves_flag_and_ms950_and_rejects_malformed_atomically() {
    for key in ["named_energy", "named_stone"] {
        let bytes = p(key);
        assert!(bytes.len() > 29);
        let shot = decode_attack(&bytes).unwrap().unwrap().projectile.unwrap();
        assert_eq!(shot.name, bytes[28..bytes.len() - 1]);
        assert_eq!(
            shot.retain_on_arrival,
            if key == "named_energy" { 1 } else { 0 }
        );
        let mut world = world("hit");
        let before = serde_json::to_value(&world).unwrap();
        for end in 0..bytes.len() {
            assert!(
                world.accept(&bytes[..end]).is_err(),
                "{key} truncated {end}"
            );
            assert_eq!(serde_json::to_value(&world).unwrap(), before);
        }
        let mut embedded = bytes.clone();
        embedded[29] = 0;
        let mut extra = bytes.clone();
        extra.push(0);
        let mut too_long = bytes[..28].to_vec();
        too_long.extend([b'x'; 128]);
        too_long.push(0);
        for bad in [embedded, extra, too_long] {
            assert!(world.accept(&bad).is_err());
            assert_eq!(serde_json::to_value(&world).unwrap(), before);
        }
        world.accept(&bytes).unwrap();
        assert_eq!(world.projectiles.last().unwrap().projectile, shot);
        let mut longest = bytes[..28].to_vec();
        longest.extend([b'x'; 127]);
        longest.push(0);
        assert_eq!(
            decode_attack(&longest)
                .unwrap()
                .unwrap()
                .projectile
                .unwrap()
                .name
                .len(),
            127
        );
    }
}

#[test]
fn legal_nonempty_name_reaches_model_with_original_stone_pixels() {
    let world = world("named_stone");
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(root,&world,&cancel).unwrap();
    let mut drawn = 0;
    let mut hurts = 0;
    for ms in (0..=1500).step_by(50) {
        let frame = model.draw_at(&world,&cancel,ms*1_000_000).unwrap();
        let receipt = &frame.receipt["projectiles"];
        hurts += receipt["hurt_callbacks"].as_array().unwrap().len();
        for flight in receipt["flights"].as_array().unwrap() {
            assert_eq!(flight["name_bytes"],serde_json::json!(b"stone"));
            assert_eq!(flight["sprite"],"1156-0.spr");
            assert_eq!(flight["shadow"],"1157-0.spr");
            assert_eq!(flight["drawn"],true);
            drawn+=1;
        }
    }
    assert!(drawn>0);
    assert_eq!(hurts,1);
}

#[test]
fn energy_audio_uses_original_suppression_mode_once_per_native_callback_and_retires() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    for key in ["energy", "named_energy"] {
        let mut world = world(key);
        // A retained projectile keeps the wire destination, as 40E900 specifies.
        if key == "named_energy" {
            world.actors.get_mut(&2).unwrap().x = 32745;
        }
        let mut effects = Projectiles::default();
        effects.prepare(root, &cancel, &world);
        let hit = hit(&world);
        let mut sounds = 0;
        let mut impacts = 0;
        for ms in (0..=330).step_by(30) {
            impacts += effects
                .update(&world, std::iter::once(&hit), ms * 1_000_000)
                .len();
            let events = effects.entered_events();
            assert_eq!(events.len(), 1, "{key}, {ms}");
            let cue = events[0].prepared_audio.as_ref().unwrap();
            assert_eq!(
                cue.group().mode,
                NativeAudioMode::SuppressWhileSameIdPlaying
            );
            assert_eq!(cue.group().candidates(), [97]);
            let sound = cue.prepared_sound().unwrap();
            assert_eq!(sound.sound_id(), 97);
            assert!(sound.file_bytes() > 0);
            sounds += events.len();
            assert!(effects
                .update(&world, std::iter::once(&hit), ms * 1_000_000)
                .is_empty());
            assert!(effects.entered_events().is_empty(), "duplicate callback");
        }
        assert_eq!((sounds, impacts), (12, 1));
        assert!(effects
            .update(&world, std::iter::empty(), u64::MAX)
            .is_empty());
        assert!(effects.entered_events().is_empty());
    }
}

#[test]
fn special_flights_fence_identity_world_and_dead_target_without_replaying_sound() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    for key in ["stone", "energy"] {
        for change in ["attacker", "target", "world", "dead"] {
            let mut world = world(key);
            let mut effects = Projectiles::default();
            effects.prepare(root, &cancel, &world);
            effects.update(&world, std::iter::once(&hit(&world)), 0);
            match change {
                "attacker" => world.actors.get_mut(&1).unwrap().incarnation += 1,
                "target" => world.actors.get_mut(&2).unwrap().incarnation += 1,
                "world" => {
                    world.accept(&[0x28, 0x34, 2, 0]).unwrap();
                }
                _ => world.actors.get_mut(&2).unwrap().dead = true,
            }
            assert!(effects
                .update(&world, std::iter::empty(), 1_000_000_000)
                .is_empty());
            if change != "dead" {
                assert!(effects.entered_events().is_empty());
            }
            effects.update(&world, std::iter::empty(), 2_000_000_000);
            assert!(effects.entered_events().is_empty());
        }
    }
}

#[test]
fn unqualified_tw_resources_remain_explicitly_unavailable_for_both_writer_types() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    for (graphics, section) in [
        (11738, "cone_cold_effects"),
        (11746, "disintegrate_effects"),
        (11748, "disintegrate_effects"),
    ] {
        for flight_type in [0, 127] {
            let world = world(&format!("class_{graphics}_type{flight_type}"));
            let before = serde_json::to_value(&world).unwrap();
            let mut model = Model::load(root, &world, &cancel).unwrap();
            for ms in [0, 100, 250, 500, 1000] {
                let frame = model.draw_at(&world, &cancel, ms * 1_000_000).unwrap();
                let receipt = &frame.receipt[section];
                assert_eq!(receipt["presentation"]["state"], "unavailable");
                assert!(receipt["presentation"]["reason"]
                    .as_str()
                    .unwrap()
                    .contains("unqualified current TW"));
                assert_eq!(receipt["assets_ready"], false);
                assert_eq!(receipt["effect_submissions"], 0);
                assert!(receipt["hurt_callbacks"].as_array().unwrap().is_empty());
            }
            assert_eq!(serde_json::to_value(&world).unwrap(), before);
        }
    }
}

#[test]
fn e149_writer_class1_class4_class5_type_variants_reach_actual_model_pixels_and_callbacks() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    let output =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/issue-167");
    std::fs::create_dir_all(&output).unwrap();
    let mut evidence = Vec::new();
    for (graphics, section) in [
        (252, "class1_effects"),
        (236, "class1_effects"),
        (1801, "class1_effects"),
        (167, "spell_projectiles"),
        (1583, "spell_projectiles"),
        (1799, "spell_projectiles"),
        (1797, "spell_projectiles"),
    ] {
        for flight_type in [0, 127] {
            let key = format!("class_{graphics}_type{flight_type}");
            let world = world(&key);
            let shot = &world.projectiles[0].projectile;
            assert_eq!((shot.graphics, shot.flight_type), (graphics, flight_type));
            assert_eq!((world.own_cast_packets, world.own_attack_packets), (1, 0));
            assert!(world.actors[&2].actions.is_empty(), "premature hurt {key}");
            let before = serde_json::to_value(&world).unwrap();
            let mut model = Model::load(root, &world, &cancel).unwrap();
            let mut drawn = 0;
            let mut hurts = 0;
            let mut last = serde_json::Value::Null;
            for ms in (0..=4000).step_by(50) {
                let frame = model.draw_at(&world, &cancel, ms * 1_000_000).unwrap();
                let receipt = &frame.receipt[section];
                assert!(receipt["load_error"].is_null(), "{key} {receipt}");
                hurts += receipt["hurt_callbacks"].as_array().unwrap().len();
                let visible = receipt["flights"]
                    .as_array()
                    .or_else(|| receipt["effects"].as_array())
                    .is_some_and(|rows| rows.iter().any(|row| row["drawn"] == true))
                    || receipt["effect_submissions"].as_u64().unwrap_or(0) > 0;
                if visible {
                    drawn += 1;
                    assert!(frame.canvas.rgba().iter().any(|v| *v != 0));
                }
                last = receipt.clone();
            }
            assert!(drawn > 0, "no effect {key} {last}");
            assert_eq!(hurts, 1, "hurt callbacks {key}");
            assert_eq!(last["active"], 0, "retirement {key}");
            assert_eq!(serde_json::to_value(&world).unwrap(), before);
            evidence.push(
                serde_json::json!({"packet":key,"graphics":graphics,"flight_type":flight_type,
                "section":section,"drawn_frames":drawn,"hurt_callbacks":hurts,"last":last}),
            );
        }
    }
    std::fs::write(
        output.join("s23-model-matrix.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
}
