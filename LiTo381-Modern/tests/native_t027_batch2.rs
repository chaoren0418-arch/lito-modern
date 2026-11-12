//! Issue #146: offline packets and original-resource rendering; no game/service.
use lito_modern::modern_lito::{
    native_attack_many,
    native_environment::State as Environment,
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{Model, Target},
    native_skills as skills, native_world_effects,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::time::{Duration, Instant};
const ITEM: u32 = 0xfedcba98;
const IDS: [u16; 8] = [29, 30, 33, 36, 37, 39, 40, 41];
fn fixture(data: &[u8], key: &str) -> Vec<u8> {
    let value: serde_json::Value = serde_json::from_slice(data).unwrap();
    let text = value["packets"][key].as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn state() -> Snapshot {
    let mut s = LoginController::default().snapshot().clone();
    for key in ["world", "start", "status", "actor", "bot"] {
        s.world
            .accept(&fixture(
                include_bytes!("fixtures/native-world-current.json"),
                key,
            ))
            .unwrap();
    }
    let mut own = s.world.actors.remove(&ITEM).unwrap();
    own.id = 1;
    own.x = 32770;
    own.y = 32818;
    s.world.actors.insert(1, own);
    s.world.status.as_mut().unwrap().object_id = 1;
    let actor = s.world.actors.get_mut(&2).unwrap();
    actor.x = 32771;
    actor.y = 32818;
    s.phase = LoginPhase::WorldReady;
    s.game_connected = true;
    s.selected_character = Some("Batch2".into());
    s.selected_source = Some("fixture".into());
    s.world.hp = Some([50, 50]);
    s.world.mp = Some([100, 100]);
    // Formal S1E learned-book bits, all eight IDs and no fabricated learned state.
    s.world.accept(&[0x1e, 6, 0, 0, 0, 0x30, 0xd9, 1]).unwrap();
    s
}
fn effect(id: u32, gfx: u16) -> Vec<u8> {
    let mut b = vec![0x37];
    b.extend(id.to_le_bytes());
    b.extend(gfx.to_le_bytes());
    b
}
fn many(gfx: u16) -> Vec<u8> {
    let mut b = vec![0x39, 18];
    b.extend(1u32.to_le_bytes());
    b.extend(32770u16.to_le_bytes());
    b.extend(32818u16.to_le_bytes());
    b.push(2);
    b.extend(9u32.to_le_bytes());
    b.extend(gfx.to_le_bytes());
    b.extend([8, 0, 0]);
    b.extend(2u16.to_le_bytes());
    b.extend(2u32.to_le_bytes());
    b.push(0x20);
    b.extend(3u32.to_le_bytes());
    b.push(0);
    b
}

fn request(s: &Snapshot, id: u16) -> anyhow::Result<Vec<u8>> {
    skills::request(&s.world, id, Some(2))
}
fn corpse(s: &mut Snapshot) {
    s.world.accept(&[0x20, 2, 0, 0, 0, 8]).unwrap();
    assert!(s.world.actors[&2].dead);
}
fn pending(s: &Snapshot, now: Instant) -> skills::Pending {
    skills::Pending {
        since: now,
        casts: s.world.own_cast_packets,
        messages: s.world.message_count,
        generation: s.world.generation,
        incarnation: s.world.own_actor().unwrap().incarnation,
    }
}
fn message(s: &mut Snapshot, code: u16) {
    let mut body = vec![0x10];
    body.extend(code.to_le_bytes());
    body.push(0);
    s.world.accept(&body).unwrap();
}

#[test]
fn eight_wire_shapes_self_policy_and_costs_are_server_owned() {
    let s = state();
    assert_eq!(
        s.world.learned_skills.iter().copied().collect::<Vec<_>>(),
        IDS
    );
    for (id, row, col, xy) in [
        (29, 3, 4, true),
        (30, 3, 5, true),
        (33, 4, 0, true),
        (36, 4, 3, true),
        (37, 4, 4, false),
        (39, 4, 6, true),
        (40, 4, 7, true),
        (41, 5, 0, false),
    ] {
        let mut s = s.clone();
        if id == 41 {
            corpse(&mut s);
        }
        let before = serde_json::to_value(&s).unwrap();
        let mut expected = vec![0x14, row, col, 2, 0, 0, 0];
        if xy {
            expected.extend([3, 0x80, 0x32, 0x80]);
        }
        assert_eq!(request(&s, id).unwrap(), expected, "{id}");
        assert_eq!(serde_json::to_value(&s).unwrap(), before);
        // Even HP at/below base cost is sent to authority; no invented rejection.
        for hp in [1, 19, 20, 21, 49, 50, 51, 79, 80, 81] {
            s.world.hp = Some([hp, 100]);
            s.world.mp = Some([1, 100]);
            let before = serde_json::to_value(&s).unwrap();
            assert!(request(&s, id).is_ok());
            assert_eq!(serde_json::to_value(&s).unwrap(), before);
        }
    }
    assert_eq!(
        skills::request(&s.world, 37, None).unwrap(),
        [0x14, 4, 4, 1, 0, 0, 0]
    );
    assert_eq!(
        skills::request(&s.world, 39, None).unwrap(),
        [0x14, 4, 6, 1, 0, 0, 0, 2, 0x80, 0x32, 0x80]
    );
    for id in [37, 39] {
        assert!(skills::request(&s.world, id, Some(999)).is_err());
    }
    for id in [29, 30, 33, 36, 40, 41] {
        assert!(skills::request(&s.world, id, None).is_err());
        assert!(skills::request(&s.world, id, Some(1)).is_err());
    }
    assert_eq!(
        IDS.map(skills::reuse_millis),
        [500, 500, 5000, 10, 10, 500, 500, 10]
    );
    assert_eq!(skills::client_clock_millis(33), 5000);
}

#[test]
fn all_skills_enforce_caster_and_target_gates_without_partial_mutation() {
    for id in IDS {
        for change in 0..10 {
            let mut s = state();
            if id == 41 {
                corpse(&mut s);
            }
            match change {
                0 => {
                    s.world.learned_skills.remove(&id);
                }
                1 => s.world.hp = Some([0, 50]),
                2 => s.world.mp = Some([0, 100]),
                3 => s.world.actors.get_mut(&1).unwrap().dead = true,
                4 => {
                    s.world.accept(&[0x25, 2]).unwrap();
                }
                5 => s.world.started = false,
                6 => s.world.battle_result = Some(0),
                7 => {
                    s.world.actors.remove(&2);
                }
                8 => s.world.actors.get_mut(&2).unwrap().dead = id != 41,
                _ => s.world.actors.get_mut(&2).unwrap().draw_state.invisible = true,
            }
            let before = serde_json::to_value(&s).unwrap();
            assert!(request(&s, id).is_err(), "{id}/{change}");
            assert_eq!(serde_json::to_value(&s).unwrap(), before);
        }
    }
}

#[test]
fn ranges_include_diagonals_and_native_screen_asymmetric_edges() {
    for (id, range) in [(29, 8), (30, 8), (33, 2), (39, 4), (40, 3)] {
        for (dx, dy) in [(1, 0), (1, 1), (-1, 1), (-1, -1), (0, -1)] {
            for delta in [0, 1] {
                let mut s = state();
                let a = s.world.actors.get_mut(&2).unwrap();
                a.x = (32770 + dx * (range + delta)) as u16;
                a.y = (32818 + dy * (range + delta)) as u16;
                assert_eq!(
                    request(&s, id).is_ok(),
                    delta == 0,
                    "{id}/{dx}/{dy}/{delta}"
                );
            }
        }
    }
    for id in [36, 37, 41] {
        for (dx, dy, ok) in [
            (18, 0, true),
            (-18, 0, true),
            (10, 9, false),
            (-10, -9, false),
            (10, -9, true),
            (-10, 9, true),
            (20, 0, false),
        ] {
            let mut s = state();
            if id == 41 {
                corpse(&mut s);
            }
            let a = s.world.actors.get_mut(&2).unwrap();
            a.x = (32770 + dx) as u16;
            a.y = (32818 + dy) as u16;
            assert_eq!(request(&s, id).is_ok(), ok, "{id}/{dx}/{dy}");
        }
    }
}

#[test]
fn menu_uses_learned_ids_and_rejects_stale_owner_target_and_book() {
    for id in IDS {
        let mut s = state();
        if id == 41 {
            corpse(&mut s);
        }
        let mut menu = Model::default();
        menu.sync(7, &s.world, true);
        menu.select(Some(id)).unwrap();
        assert!(!menu
            .rows()
            .iter()
            .find(|r| r.id == id)
            .unwrap()
            .label
            .contains("尚未支援"));
        let target = Target::capture(&s.world, 2);
        assert_eq!(menu.choice(7, &s.world, target).unwrap().skill, id);
        assert!(menu.choice(8, &s.world, target).is_err());
        for change in 0..6 {
            let mut n = s.clone();
            match change {
                0 => n.world.actors.get_mut(&2).unwrap().incarnation += 1,
                1 => n.world.actors.get_mut(&2).unwrap().target_revision += 1,
                2 => n.world.local_corrections += 1,
                3 => n.world.generation += 1,
                4 => {
                    n.world.learned_skills.remove(&id);
                }
                _ => n.world.actors.get_mut(&1).unwrap().incarnation += 1,
            }
            assert!(menu.choice(7, &n.world, target).is_err(), "{id}/{change}");
        }
    }
}

#[test]
fn corpse_intent_never_upgrades_live_selection_or_survives_revival_and_aba() {
    let mut s = state();
    let live = Target::capture(&s.world, 2).unwrap();
    corpse(&mut s);
    assert!(live.resolve_for_skill(&s.world, 41).is_none());
    let dead = Target::capture(&s.world, 2).unwrap();
    assert_eq!(dead.resolve_for_skill(&s.world, 41), Some(2));
    assert!(dead.resolve(&s.world).is_none());
    for id in [29, 30, 33, 36, 37, 39, 40] {
        assert!(dead.resolve_for_skill(&s.world, id).is_none());
    }
    let mut n = s.clone();
    n.world.actors.get_mut(&2).unwrap().dead = false;
    assert!(dead.resolve_for_skill(&n.world, 41).is_none());
    assert!(dead.resolve(&n.world).is_none());
    for change in 0..4 {
        let mut n = s.clone();
        match change {
            0 => n.world.actors.get_mut(&2).unwrap().action_sequence += 1,
            1 => n.world.actors.get_mut(&2).unwrap().incarnation += 1,
            2 => n.world.actors.get_mut(&2).unwrap().target_revision += 1,
            _ => n.world.generation += 1,
        }
        assert!(dead.resolve_for_skill(&n.world, 41).is_none());
    }
    s.world.accept(&[0x15, 2, 0, 0, 0]).unwrap();
    assert!(dead.resolve_for_skill(&s.world, 41).is_none());
}

#[test]
fn object_replies_are_atomic_and_never_imply_status_cost_or_companion_success() {
    for (id, gfx) in [
        (29, 752),
        (33, 746),
        (36, 228),
        (37, 227),
        (39, 2171),
        (40, 2175),
        (41, 226),
    ] {
        let mut s = state();
        if id == 41 {
            corpse(&mut s);
        }
        request(&s, id).unwrap();
        let packet = effect(2, gfx);
        let before = serde_json::to_value(&s.world).unwrap();
        for end in 0..packet.len() {
            assert!(s.world.accept(&packet[..end]).is_err());
            assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
        }
        let mut extra = packet.clone();
        extra.push(0);
        assert!(s.world.accept(&extra).is_err());
        s.world.accept(&packet).unwrap();
        let cue = s.world.effects.last().unwrap();
        assert_eq!(cue.graphics, gfx);
        assert_eq!(cue.target, Some((2, s.world.actors[&2].incarnation)));
        assert_eq!(s.world.hp, Some([50, 50]));
        assert_eq!(s.world.mp, Some([100, 100]));
        assert!(s.world.movement_blocks.is_empty());
        assert_eq!(s.world.actors.len(), 2);
    }
}

#[test]
fn slow_paralysis_cure_and_darkness_change_only_on_authoritative_packets() {
    let mut s = state();
    request(&s, 29).unwrap();
    s.world.accept(&effect(2, 752)).unwrap();
    let speed = s.world.actors[&2].speed;
    s.world.accept(&[0x29, 2, 0, 0, 0, 2, 64, 0]).unwrap();
    assert_eq!(s.world.actors[&2].speed, 2);
    s.world.accept(&[0x29, 2, 0, 0, 0, 0, 0, 0]).unwrap();
    assert_eq!(s.world.actors[&2].speed, 0);
    assert_eq!(speed, 0);
    request(&s, 33).unwrap();
    s.world.accept(&effect(2, 746)).unwrap();
    assert!(s.world.movement_blocks.is_empty());
    s.world.accept(&[0x25, 2]).unwrap();
    assert!(s.world.movement_blocks.contains(&2));
    s.world.accept(&effect(1, 227)).unwrap();
    assert!(s.world.movement_blocks.contains(&2));
    s.world.accept(&[0x25, 3]).unwrap();
    assert!(s.world.movement_blocks.is_empty());
    let mut env = Environment::default();
    env.begin(7).unwrap();
    request(&s, 40).unwrap();
    s.world.accept(&effect(2, 2175)).unwrap();
    assert_eq!(env.blind, 0);
    env.accept(7, &[0x0a, 1]).unwrap();
    assert_eq!(env.blind, 1);
    env.accept(7, &[0x0a, 0]).unwrap();
    assert_eq!(env.blind, 0);
    request(&s, 39).unwrap();
    s.world.accept(&effect(2, 2171)).unwrap();
    assert_eq!(s.world.hp, Some([50, 50]));
    assert_eq!(s.world.mp, Some([100, 100]));
    s.world.accept(&[0x0d, 1, 0, 50, 0]).unwrap();
    s.world.accept(&[0x4d, 120, 0, 200, 0]).unwrap();
    assert_eq!(s.world.hp, Some([1, 50]));
    assert_eq!(s.world.mp, Some([120, 200]));
}

#[test]
fn rejection_capacity_and_silent_timeout_do_not_create_success() {
    let now = Instant::now();
    for id in IDS {
        let mut s = state();
        if id == 41 {
            corpse(&mut s);
        }
        request(&s, id).unwrap();
        let p = pending(&s, now);
        assert_eq!(p.observe(&s.world, now), skills::Feedback::Pending);
        assert_eq!(
            p.observe(&s.world, now + Duration::from_secs(2)),
            skills::Feedback::NoResponse
        );
        for code in [278, 280, 299, 319] {
            let mut n = s.clone();
            message(&mut n, code);
            n.world
                .accept(&effect(2, if id == 41 { 226 } else { 228 }))
                .unwrap();
            assert_eq!(p.observe(&n.world, now), skills::Feedback::Rejected(code));
            assert_eq!(n.world.hp, s.world.hp);
            assert_eq!(n.world.mp, s.world.mp);
            assert_eq!(n.world.actors.len(), 2);
        }
        s.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
        assert_eq!(p.observe(&s.world, now), skills::Feedback::CastAction);
        let old = pending(&s, now);
        assert_eq!(old.observe(&s.world, now), skills::Feedback::Pending);
        s.world.generation += 1;
        assert_eq!(p.observe(&s.world, now), skills::Feedback::Interrupted);
    }
}

#[test]
fn earth_jail_receipt_preserves_order_misses_frozen_origin_and_atomicity() {
    let mut s = state();
    request(&s, 30).unwrap();
    let packet = many(1805);
    let before = serde_json::to_value(&s.world).unwrap();
    for end in 0..packet.len() {
        assert!(s.world.accept(&packet[..end]).is_err());
        assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
    }
    for byte in [1, 17] {
        let mut wrong = packet.clone();
        wrong[byte] = if byte == 1 { 19 } else { 0 };
        assert!(native_attack_many::decode(&wrong).unwrap().is_none());
    }
    let mut wrong = packet.clone();
    *wrong.last_mut().unwrap() = 1;
    assert!(s.world.accept(&wrong).is_err());
    assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
    s.world.accept(&packet).unwrap();
    let cue = &s.world.many_attacks[0];
    assert_eq!(
        cue.targets,
        [
            (2, true, Some(s.world.actors[&2].incarnation)),
            (3, false, None)
        ]
    );
    assert_eq!(cue.target_origin, Some([32771, 32818]));
    s.world.actors.get_mut(&2).unwrap().x += 8;
    assert_eq!(s.world.many_attacks[0].target_origin, Some([32771, 32818]));
    let mut attack = native_attack_many::decode(&packet).unwrap().unwrap();
    s.world.actors.remove(&2);
    for (heading, (dx, dy)) in [
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ]
    .into_iter()
    .enumerate()
    {
        attack.heading = heading as u8;
        assert_eq!(
            native_attack_many::capture_target_origin(&attack, &s.world.actors),
            Some([32770 + dx, 32818 + dy])
        );
    }
    assert_eq!(s.world.hp, Some([50, 50]));
    assert_eq!(s.world.mp, Some([100, 100]));
}

#[test]
#[ignore = "original resources; memory rendering and audio preparation only"]
fn original_finite_effects_preserve_cadence_mix_sound_and_consumed_corpse_position() {
    use lito_modern::modern_lito::{
        native_skill_effects::Effects,
        resources::{SharedArchive, SharedArchiveKind},
        sprite_actions::{ActionKey, ActionLimits, FrameEvent, SpriteActionCatalog},
    };
    use std::{path::Path, sync::atomic::AtomicBool};
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    let raw = SharedArchive::open(root, SharedArchiveKind::Text)
        .unwrap()
        .read_entry("list.spr")
        .unwrap();
    let catalog = SpriteActionCatalog::parse_for_inspection(&raw, ActionLimits::default()).unwrap();
    for (gfx, count, sound) in [
        (752, 16, None),
        (746, 8, Some(116)),
        (228, 7, Some(132)),
        (227, 7, Some(122)),
        (2171, 9, Some(806)),
        (2175, 8, Some(802)),
        (226, 7, Some(139)),
    ] {
        let animation = catalog
            .resolve_action(gfx, ActionKey::new(0))
            .unwrap()
            .animation;
        assert_eq!(animation.frames.len(), count);
        let sounds: Vec<_> = animation
            .frames
            .iter()
            .flat_map(|f| &f.events)
            .filter_map(|e| {
                if let FrameEvent::Sound(s) = e {
                    Some(*s)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(sounds, sound.into_iter().collect::<Vec<_>>());
        if gfx == 752 {
            assert_eq!(
                animation
                    .frames
                    .iter()
                    .map(|f| f.subframe)
                    .collect::<Vec<_>>(),
                [7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6]
            );
            assert!(animation.frames.iter().all(|f| f.duration_ticks == 3));
        }
        let mut s = state();
        if gfx == 226 {
            corpse(&mut s);
        }
        s.world.accept(&effect(2, gfx as u16)).unwrap();
        let mut effects = Effects::default();
        effects.prepare(root, &s.world, &cancel);
        let view = ViewProjection {
            anchor_cell: WorldCell { x: 32770, y: 32818 },
            anchor_pixel: PixelPoint { x: 400, y: 300 },
        };
        let mut drawn = false;
        let mut sounds = Vec::new();
        let mut last = serde_json::Value::Null;
        for step in 0..401 {
            if step == 3 {
                s.world.accept(&[0x15, 2, 0, 0, 0]).unwrap();
            }
            let before = serde_json::to_value(&s.world).unwrap();
            let mut fragments = Vec::new();
            let (report, audio) = effects.draw_to(
                &s.world,
                &mut native_world_effects::Target::Fragments(&mut fragments),
                view,
                step * 10_000_000,
            );
            assert!(report["load_error"].is_null(), "{gfx}: {report}");
            assert_eq!(report["omitted"], 0, "{report}");
            for fragment in &fragments {
                assert_eq!(fragment.position, WorldCell { x: 32771, y: 32818 });
                assert!(match gfx {
                    2175 => matches!(fragment.mix, native_world_effects::Mix::Multiply),
                    _ => matches!(fragment.mix, native_world_effects::Mix::Screen),
                });
            }
            drawn |= step > 3 && !fragments.is_empty();
            sounds.extend(audio);
            last = report;
            assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
        }
        assert!(drawn, "{gfx}");
        assert_eq!(last["active"], 0);
        assert_eq!(
            sounds.iter().filter_map(|e| e.sound).collect::<Vec<_>>(),
            sound.into_iter().collect::<Vec<_>>()
        );
        assert!(sounds.iter().all(|e| e.prepared_audio.is_some()));
        s.world.generation += 1;
        s.world.effects.clear();
        let mut fragments = Vec::new();
        let (report, audio) = effects.draw_to(
            &s.world,
            &mut native_world_effects::Target::Fragments(&mut fragments),
            view,
            5_000_000_000,
        );
        assert_eq!(report["active"], 0);
        assert!(audio.is_empty());
        assert!(fragments.is_empty());
    }
}

#[test]
#[ignore = "original 1805/1806/1807 and WAV718; memory rendering only"]
fn original_earth_jail_draws_three_layers_and_two_hits_without_replaying_on_duplicate_event() {
    use lito_modern::modern_lito::native_combat_animation::EnteredEvent;
    use std::{path::Path, sync::atomic::AtomicBool};
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let mut s = state();
    s.world.accept(&many(1805)).unwrap();
    let cue = s.world.many_attacks[0].clone();
    s.world.actors.get_mut(&2).unwrap().x += 3;
    let before = serde_json::to_value(&s.world).unwrap();
    let hit = EnteredEvent {
        prepared_audio: None,
        actor: 1,
        sequence: cue.action_sequence,
        action: 18,
        frame: 1,
        event: "Hit".into(),
        sound: None,
    };
    let mut effects = native_attack_many::Effects::default();
    effects.prepare(root, &s.world, &AtomicBool::new(false));
    let view = ViewProjection {
        anchor_cell: WorldCell { x: 32770, y: 32818 },
        anchor_pixel: PixelPoint { x: 400, y: 300 },
    };
    let mut audio = Vec::new();
    let mut impacts = Vec::new();
    let mut drawn = false;
    let mut clothing_only = false;
    let mut terminal_dwell = false;
    let mut last = serde_json::Value::Null;
    for step in 0..601 {
        impacts.extend(effects.update(&s.world, [&hit].into_iter(), step * 10_000_000));
        let mut fragments = Vec::new();
        let (report, events) = effects.draw_to(
            &mut native_world_effects::Target::Fragments(&mut fragments),
            view,
            step * 10_000_000,
        );
        assert!(report["load_error"].is_null(), "{report}");
        assert_eq!(report["omitted"], 0, "{report}");
        if !fragments.is_empty() {
            assert!(fragments
                .iter()
                .all(|f| f.position == WorldCell { x: 32771, y: 32818 }));
            let subframe = report["effects"][0]["subframe"].as_u64().unwrap();
            if subframe >= 15 {
                assert_eq!(fragments.len(), 1);
                assert!(matches!(
                    fragments[0].mix,
                    native_world_effects::Mix::Screen
                ));
                clothing_only = true;
                terminal_dwell |= report["effects"][0]["source_frame"] == 0;
            } else {
                assert_eq!(fragments.len(), 3);
                assert!(matches!(fragments[0].mix, native_world_effects::Mix::Over));
                assert!(matches!(
                    fragments[1].mix,
                    native_world_effects::Mix::Screen
                ));
                assert!(matches!(
                    fragments[2].mix,
                    native_world_effects::Mix::Shadow
                ));
            }
            drawn = true;
        }
        audio.extend(events);
        last = report;
    }
    assert!(drawn);
    assert!(clothing_only && terminal_dwell);
    assert_eq!(impacts.len(), 2);
    assert!(impacts.iter().all(|i| i.target == 2));
    assert_eq!(last["started"], 1);
    assert_eq!(last["active"], 0);
    assert_eq!(audio.len(), 1);
    assert_eq!(audio[0].sound, Some(718));
    assert!(audio[0].prepared_audio.is_some());
    assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
}
