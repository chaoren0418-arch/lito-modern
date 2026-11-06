//! T028-1: offline current-source qualification; no peer, DB, GUI or audio device.
use lito_modern::modern_lito::{
    compositor::Canvas,
    native_class1,
    native_combat_animation::EnteredEvent,
    native_skill_effects,
    native_skill_selection::{self, Target},
    native_skills,
    native_world::World,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::{
    path::PathBuf,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

const SKILLS: [u16; 10] = [6, 8, 9, 10, 11, 18, 19, 28, 35, 57];
#[test]
fn current_launcher_healing_definition_bytes_are_pinned() {
    use lito_modern::modern_lito::tw_action_catalog::SharedTwActionCatalog;
    let catalog = SharedTwActionCatalog::load_selected(
        &root().join("launcher"),
        &[11722, 11723, 11724, 11725, 11726].into(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let hex = |bytes: [u8; 32]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    assert_eq!(
        hex(catalog.pack_sha256()),
        "92b512a32349d0775e1036f1832e21f1141ae66e64821d6978f0879246e59ca7"
    );
    assert_eq!(
        hex(catalog.text_sha256()),
        "28c4fc600833fb8c609ab195a67ff926e8ffa571100d8d4b389a23be8c2c8db8"
    );
    assert_eq!(catalog.encoded_bytes(), 1_406_534);
    assert_eq!(catalog.decoded_bytes(), 5_990_880);
    let expected = [
        (
            11722,
            "9cc35d88f4d3bd7d591c1b51d6cb9c8508acc89c8c74ea321492e9fe5852bc26",
        ),
        (
            11723,
            "c5a8fd92c3274e1b6f75803ed619f63d3e22ffa8fdb383c4f08332a03cc9e4ba",
        ),
        (
            11724,
            "fe88a43f89cb908d644d6b1ffd59a405fa7d83a2839c170024955cdbc52db93a",
        ),
        (
            11725,
            "dfe98eb4f3f4b8a3286ec489a65564454c18750832d80790e68e7bc7c8d45ae4",
        ),
        (
            11726,
            "8b311b0403933ca6981bbda5ea7627747ba638192448ff468b0c7a362ab303be",
        ),
    ];
    assert_eq!(catalog.definitions().len(), expected.len());
    for (definition, (id, hash)) in catalog.definitions().iter().zip(expected) {
        assert_eq!(definition.sprite_id, id);
        assert_eq!(hex(definition.sha256), hash);
    }
}
fn root() -> PathBuf {
    std::env::var_os("LITO_SHARED_RESOURCE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"D:\lineage開發\0-客戶端"))
}
fn world() -> World {
    let data: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for key in ["world", "start", "status", "actor", "bot"] {
        let s = data["packets"][key].as_str().unwrap();
        let packet: Vec<_> = (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect();
        w.accept(&packet).unwrap();
    }
    let mut own = w.actors.remove(&0xfedcba98).unwrap();
    own.id = 1;
    own.x = 32729;
    own.y = 32797;
    w.actors.insert(1, own);
    w.status.as_mut().unwrap().object_id = 1;
    let other = w.actors.get_mut(&2).unwrap();
    other.x = 32730;
    other.y = 32797;
    // Exercise both the NPC fixture and a separate player-shaped actor. This
    // remains an offline fixture, not evidence of a server-approved target.
    let mut third = w.actors[&1].clone();
    third.id = 0xf1234567;
    third.x = 32728;
    third.incarnation += 1;
    w.actors.insert(third.id, third);
    w.learned_skills.extend(SKILLS);
    w.hp = Some([70, 100]);
    w.mp = Some([24, 24]);
    w
}
fn view() -> ViewProjection {
    ViewProjection {
        anchor_cell: WorldCell { x: 32729, y: 32797 },
        anchor_pixel: PixelPoint { x: 160, y: 120 },
    }
}

#[test]
fn all_ten_selectors_use_current_other_target_and_preserve_authority() {
    let mut w = world();
    for skill in SKILLS {
        let object_only = matches!(skill, 8 | 9 | 19 | 35 | 57);
        for id in [2u32, 0xf1234567] {
            let before = serde_json::to_value(&w).unwrap();
            let mut expected = vec![0x14, ((skill - 1) / 8) as u8, ((skill - 1) % 8) as u8];
            expected.extend(id.to_le_bytes());
            if !object_only {
                expected.extend(w.actors[&id].x.to_le_bytes());
                expected.extend(w.actors[&id].y.to_le_bytes());
            }
            let target = Target::capture(&w, id).unwrap();
            assert_eq!(
                native_skill_selection::choice(&w, skill, Some(target))
                    .unwrap()
                    .target,
                Some(id)
            );
            assert_eq!(
                native_skills::request(&w, skill, Some(id)).unwrap(),
                expected,
                "skill {skill}"
            );
            assert_eq!(serde_json::to_value(&w).unwrap(), before);
        }
        assert_eq!(native_skills::request(&w, skill, None).is_ok(), object_only);
        assert_eq!(
            native_skills::request(&w, skill, Some(1)).is_ok(),
            object_only
        );
        // These are request range boundaries, independent of graphics playback.
        let range = match skill {
            10 => 3,
            11 => 1,
            28 => 4,
            8 | 9 => 18,
            19 | 35 | 57 => 19,
            _ => 10,
        };
        w.actors.get_mut(&2).unwrap().x = 32729 + range;
        assert!(
            native_skills::request(&w, skill, Some(2)).is_ok(),
            "skill {skill} boundary"
        );
        // Screen-diamond skills have a separate asymmetric 19-cell boundary.
        w.actors.get_mut(&2).unwrap().x = 32729
            + if matches!(skill, 8 | 9) {
                20
            } else {
                range + 1
            };
        assert!(
            native_skills::request(&w, skill, Some(2)).is_err(),
            "skill {skill} outside"
        );
        if matches!(skill, 8 | 9) {
            for (dx, dy, allowed) in [
                (-19i32, 0i32, true),
                (19, 0, false),
                (17, -2, true),
                (18, -1, false),
            ] {
                let target = w.actors.get_mut(&2).unwrap();
                target.x = (32729 + dx) as u16;
                target.y = (32797 + dy) as u16;
                assert_eq!(native_skills::request(&w, skill, Some(2)).is_ok(), allowed);
            }
            w.actors.get_mut(&2).unwrap().y = 32797;
        }
        w.actors.get_mut(&2).unwrap().x = 32730;
    }
}

#[test]
fn all_ten_reject_stale_dead_invisible_missing_and_unlearned_targets() {
    for skill in SKILLS {
        for invalid in 0..5 {
            let mut w = world();
            let target = Target::capture(&w, 2).unwrap();
            match invalid {
                0 => w.actors.get_mut(&2).unwrap().incarnation += 1,
                1 => w.actors.get_mut(&2).unwrap().dead = true,
                2 => w.actors.get_mut(&2).unwrap().draw_state.invisible = true,
                3 => {
                    w.actors.remove(&2);
                }
                _ => {
                    w.learned_skills.remove(&skill);
                }
            }
            let before = serde_json::to_value(&w).unwrap();
            assert!(
                native_skill_selection::choice(&w, skill, Some(target)).is_err(),
                "skill {skill}, invalid {invalid}"
            );
            assert_eq!(serde_json::to_value(&w).unwrap(), before);
        }
    }
}

#[test]
fn all_ten_pending_timeout_boundary_and_server_rejection_precedence() {
    for skill in SKILLS {
        let mut w = world();
        // Low nonzero MP is not a locally predicted server permission decision.
        w.mp = Some([1, 24]);
        native_skills::request(&w, skill, Some(2)).unwrap();
        let now = Instant::now();
        let pending = native_skills::Pending {
            since: now,
            casts: w.own_cast_packets,
            messages: w.message_count,
            generation: w.generation,
            incarnation: w.actors[&1].incarnation,
        };
        let before = serde_json::to_value(&w).unwrap();
        assert_eq!(
            pending.observe(&w, now + Duration::from_secs(2) - Duration::from_nanos(1)),
            native_skills::Feedback::Pending
        );
        assert_eq!(
            pending.observe(&w, now + Duration::from_secs(2)),
            native_skills::Feedback::NoResponse
        );
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
        // S_MESSAGE 280 must win even after an own cast action arrived.
        w.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
        assert_eq!(
            pending.observe(&w, now),
            native_skills::Feedback::CastAction
        );
        w.accept(&[0x10, 0x18, 1, 0]).unwrap();
        assert_eq!(
            pending.observe(&w, now + Duration::from_secs(2)),
            native_skills::Feedback::Rejected(280)
        );
        assert_eq!(w.hp, Some([70, 100]));
        assert_eq!(w.mp, Some([1, 24]));
    }
}

fn effect(w: &mut World, id: u32, gfx: u16) {
    let mut packet = vec![0x37];
    packet.extend(id.to_le_bytes());
    packet.extend(gfx.to_le_bytes());
    w.accept(&packet).unwrap();
}

#[test]
fn late_other_target_callbacks_start_a_fresh_dwell_and_map_change_retires() {
    for (gfx, source, next_dwell) in [
        (2165, 2, 166),
        (871, 2, 41),
        (745, 2, 166),
        (754, 3, 83),
        (11723, 2, 125),
        (11724, 2, 125),
        (11725, 2, 125),
    ] {
        let mut w = world();
        effect(&mut w, 2, gfx);
        let mut fx = native_skill_effects::Effects::default();
        fx.prepare(&root(), &w, &AtomicBool::new(false));
        let mut canvas = Canvas::new(320, 240).unwrap();
        fx.draw(&w, &mut canvas, view(), 0);
        let before = serde_json::to_value(&w).unwrap();
        let (r, events) = fx.draw(&w, &mut canvas, view(), 10_000_000_000);
        assert_eq!(r["omitted"], 0, "gfx {gfx}: {r}");
        assert_eq!(
            r["active"], 1,
            "a late poll cannot catch up all positive dwells"
        );
        assert_eq!(r["effects"][0]["source_frame"], source);
        assert_eq!(r["effects"][0]["next_callback_ms"], 10_000 + next_dwell);
        assert!(events.is_empty());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
        w.accept(&[0x28, 0x34, 2, 0]).unwrap();
        let (r, events) = fx.draw(&w, &mut canvas, view(), 10_001_000_000);
        assert_eq!(r["active"], 0);
        assert_eq!(r["started"], 0);
        assert!(events.is_empty());
    }
}

#[test]
fn other_target_finite_effects_exact_boundaries_natural_expiry_and_no_replay() {
    // Source-order dwell milliseconds (source1..last, then terminal source0).
    // Pinned definitions: holy2165, cure871, poison745, TU754; TW heals and
    // legacy aliases are separate wire namespaces with the same 21 x 125ms clock.
    let cases = [
        (8, 2165, 804, vec![166; 13], 1),
        (9, 871, 141, vec![41, 41, 41, 41, 41, 125], 1),
        (
            11,
            745,
            117,
            vec![125, 166, 166, 166, 166, 166, 166, 125],
            1,
        ),
        (
            18,
            754,
            102,
            vec![
                41, 83, 83, 41, 41, 41, 41, 41, 250, 41, 41, 41, 41, 41, 83, 83, 83, 125, 125,
            ],
            2,
        ),
        (19, 11723, 114, vec![125; 21], 1),
        (35, 11724, 115, vec![125; 21], 1),
        (57, 11725, 126, vec![125; 21], 1),
        (19, 829, 114, vec![125; 21], 1),
        (35, 830, 115, vec![125; 21], 1),
        (57, 831, 126, vec![125; 21], 1),
    ];
    for (skill, gfx, sound, dwells, first_source) in cases {
        let mut w = world();
        w.learned_skills.clear();
        w.learned_skills.insert(skill);
        let id = 0xf1234567;
        effect(&mut w, id, gfx);
        let position = [w.actors[&id].x, w.actors[&id].y];
        let incarnation = w.actors[&id].incarnation;
        let mut fx = native_skill_effects::Effects::default();
        fx.prepare(&root(), &w, &AtomicBool::new(false));
        let mut canvas = Canvas::new(320, 240).unwrap();
        // Integer clock birth: 100.9ms; first deadline is 100ms + dwell.
        let birth = 100_900_000;
        let (r, events) = fx.draw(&w, &mut canvas, view(), birth);
        assert!(r["load_error"].is_null(), "gfx {gfx}: {r}");
        assert_eq!(r["omitted"], 0, "gfx {gfx}: {r}");
        assert_eq!(r["started"], 1);
        assert_eq!(r["active"], 1);
        assert_eq!(r["effects"][0]["source_frame"], first_source, "gfx {gfx}");
        assert_eq!(
            events.iter().filter_map(|e| e.sound).collect::<Vec<_>>(),
            [sound]
        );
        assert!(events.iter().all(|e| e.actor == id));
        assert_eq!(
            r["effects"][0]["target"],
            serde_json::json!([id, incarnation])
        );
        // S37 is a frozen-position effect. Actor movement/removal/reuse does not
        // relocate it, refresh its timer, or mutate the replacement's HP/status.
        w.actors.get_mut(&id).unwrap().x += 3;
        let mut replacement = w.actors.remove(&id).unwrap();
        replacement.incarnation += 1;
        w.actors.insert(id, replacement);
        let before = serde_json::to_value(&w).unwrap();
        let mut elapsed = 0u64;
        for (index, dwell) in dwells.iter().enumerate() {
            elapsed += dwell;
            let deadline = (100 + elapsed) * 1_000_000;
            let (pre, events) = fx.draw(&w, &mut canvas, view(), deadline - 1);
            assert_eq!(pre["active"], 1, "gfx {gfx}, boundary {index}");
            assert!(events.is_empty());
            assert_eq!(pre["effects"][0]["position"], serde_json::json!(position));
            assert_eq!(
                pre["effects"][0]["next_callback_ms"], elapsed,
                "gfx {gfx}, boundary {index}: {pre}"
            );
            let (at, events) = fx.draw(&w, &mut canvas, view(), deadline);
            assert!(events.is_empty());
            assert_eq!(at["omitted"], 0);
            let finished = index + 1 == dwells.len();
            assert_eq!(
                at["active"],
                usize::from(!finished),
                "gfx {gfx}, boundary {index}: {at}"
            );
            assert_eq!(at["effects"][0]["finished"], finished);
            let source = if finished || (first_source == 1 && index + 2 == dwells.len()) {
                0
            } else {
                first_source + index + 1
            };
            assert_eq!(
                at["effects"][0]["source_frame"], source,
                "gfx {gfx}, boundary {index}"
            );
            assert_eq!(serde_json::to_value(&w).unwrap(), before);
        }
        for now in [(100 + elapsed) * 1_000_000, 10_000_000_000] {
            let (r, events) = fx.draw(&w, &mut canvas, view(), now);
            assert_eq!(r["active"], 0);
            assert_eq!(r["started"], 1);
            assert!(r["effects"].as_array().unwrap().is_empty() && events.is_empty());
        }
    }
}

fn attack(w: &mut World, gfx: u16, id: u32) -> EnteredEvent {
    let mut packet = vec![0x23, 18];
    packet.extend(1u32.to_le_bytes());
    packet.extend(id.to_le_bytes());
    packet.extend([10, 2]);
    packet.extend(77u32.to_le_bytes());
    packet.extend(gfx.to_le_bytes());
    packet.push(0);
    for n in [32729u16, 32797, w.actors[&id].x, w.actors[&id].y] {
        packet.extend(n.to_le_bytes());
    }
    packet.extend([0, 0]);
    w.accept(&packet).unwrap();
    EnteredEvent {
        actor: 1,
        sequence: w.projectiles.last().unwrap().action_sequence,
        action: 18,
        frame: 1,
        event: "Hit".into(),
        sound: None,
        prepared_audio: None,
    }
}

#[test]
fn chill_and_vampiric_other_targets_hit_and_expire_at_exact_boundaries() {
    for (skill, gfx, sound) in [(10, 252, 110), (28, 236, 103)] {
        for id in [2, 0xf1234567] {
            let mut w = world();
            w.learned_skills.clear();
            w.learned_skills.insert(skill);
            let event = attack(&mut w, gfx, id);
            let before = serde_json::to_value(&w).unwrap();
            let mut fx = native_class1::Effects::default();
            fx.prepare(&root(), &w, &AtomicBool::new(false));
            let mut canvas = Canvas::new(320, 240).unwrap();
            assert!(fx
                .update(&w, std::iter::once(&event), 100_900_000)
                .is_empty());
            let (r, events) = fx.draw(&mut canvas, view(), 100_900_000);
            assert!(r["load_error"].is_null());
            assert_eq!(r["omitted"], 0);
            assert_eq!(
                events.iter().filter_map(|e| e.sound).collect::<Vec<_>>(),
                [sound]
            );
            for (ms, source) in [(41u64, 2), (124, 3), (249, 4), (415, 5), (456, 0), (581, 0)] {
                let due = (100 + ms) * 1_000_000;
                assert!(fx.update(&w, std::iter::once(&event), due - 1).is_empty());
                assert_eq!(fx.draw(&mut canvas, view(), due - 1).0["active"], 1);
                let impacts = fx.update(&w, std::iter::once(&event), due);
                assert_eq!(impacts.len(), usize::from(ms == 415));
                if ms == 415 {
                    assert_eq!(impacts[0].target, id);
                }
                let (r, events) = fx.draw(&mut canvas, view(), due);
                assert_eq!(r["effects"][0]["source_frame"], source);
                assert_eq!(r["active"], usize::from(ms != 581));
                assert_eq!(r["started"], 1);
                assert_eq!(r["omitted"], 0);
                assert!(events.is_empty());
                assert_eq!(serde_json::to_value(&w).unwrap(), before);
            }
            assert!(fx
                .update(&w, std::iter::once(&event), 10_000_000_000)
                .is_empty());
            let (r, events) = fx.draw(&mut canvas, view(), 10_000_000_000);
            assert_eq!(r["active"], 0);
            assert_eq!(r["hits"], 1);
            assert!(events.is_empty());
        }
    }
}
