use lito_modern::modern_lito::{
    native_combat,
    native_combat_animation::Playback,
    native_world::World,
    sprite_actions::{ActionLimits, SpriteActionCatalog},
};
#[test]
fn npc_damage_bursts_preserve_spell_events_authority_and_death_preemption() {
    use lito_modern::modern_lito::{native_world_animation::resolve, resources::{SharedArchive, SharedArchiveKind}};
    let bytes = SharedArchive::open(std::path::Path::new(r"D:\lineage開發\0-客戶端"), SharedArchiveKind::Text)
        .unwrap().read_entry("list.spr").unwrap();
    let catalog = SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default()).unwrap();
    let mut w = world();
    let authority = (w.hp, w.mp, w.actors[&1].hp_ratio);
    let mut playback = Playback::default();
    let action = |id: u32, value: u8| {let mut p = vec![0x20];p.extend(id.to_le_bytes());p.push(value);p};
    // Seven NPCs can deliver many hurt cues during a single spell body.
    w.accept(&action(1, 19)).unwrap();
    // First flood arrives before any render sample: the bounded world history
    // must retain the earlier spell instead of evicting its Hit cue.
    for _ in 0..80 {w.accept(&action(1, 2)).unwrap();}
    assert_eq!(w.actors[&1].actions.iter().map(|c| c.action).collect::<Vec<_>>(), [19, 2]);
    let mut events = playback.sample(&catalog, &w.actors[&1], 0).unwrap().1;
    for n in 1..=80 {
        w.accept(&action(1, 2)).unwrap();
        events.extend(playback.sample(&catalog, &w.actors[&1], n * 1_000_000).unwrap().1);
    }
    // A second acknowledged spell separates the two hurt bursts. Its Hit
    // must survive coalescing, including synthetic projectile arrivals.
    w.accept(&action(1, 19)).unwrap();
    for _ in 0..80 {w.accept(&action(1, 2)).unwrap();}
    assert_eq!(w.actors[&1].actions.iter().filter(|c| c.action == 19).count(), 2);
    events.extend(playback.sample(&catalog, &w.actors[&1], 81_000_000).unwrap().1);
    assert!(!playback.impact(&w.actors[&1], 100_000));
    assert!(!playback.impact(&w.actors[&1], 100_001));
    for n in 82..=160 {
        w.accept(&action(1, 2)).unwrap();
        events.extend(playback.sample(&catalog, &w.actors[&1], n * 1_000_000).unwrap().1);
    }
    let (pose, more) = playback.sample(&catalog, &w.actors[&1], 10_000_000_000).unwrap();
    events.extend(more);
    assert!(pose.is_none());
    let clip = resolve(&catalog, u32::from(w.actors[&1].graphics), 19).unwrap();
    let expected_hits = clip.animation.frames.iter().flat_map(|f| &f.events)
        .filter(|e| matches!(e, lito_modern::modern_lito::sprite_actions::FrameEvent::Hit)).count();
    assert!(expected_hits > 0);
    assert_eq!(events.iter().filter(|e| e.action == 19 && e.event == "Hit").count(), expected_hits * 2);
    let expected_sounds = clip.animation.frames.iter().flat_map(|f| &f.events)
        .filter(|e| matches!(e, lito_modern::modern_lito::sprite_actions::FrameEvent::Sound(_))).count();
    assert_eq!(events.iter().filter(|e| e.action == 19 && e.sound.is_some()).count(), expected_sounds * 2);
    assert!(playback.sample(&catalog, &w.actors[&1], 10_000_000_000).unwrap().1.is_empty());
    assert_eq!((w.hp, w.mp, w.actors[&1].hp_ratio), authority);
    w.accept(&action(1, 19)).unwrap();
    playback.sample(&catalog, &w.actors[&1], 11_000_000_000).unwrap();
    for _ in 0..80 {w.accept(&action(1, 2)).unwrap();}
    w.accept(&action(1, 8)).unwrap();
    let (pose, events) = playback.sample(&catalog, &w.actors[&1], 11_100_000_000).unwrap();
    assert_eq!(pose.unwrap().phase, "death");
    assert!(events.iter().all(|e| e.action == 8));
    assert!(!playback.impact(&w.actors[&1], 100_002));
}
fn packet(file: &[u8], key: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_slice(file).unwrap();
    let s = v["packets"][key].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn p(k: &str) -> Vec<u8> {
    packet(include_bytes!("fixtures/native-combat-current.json"), k)
}
#[test]
fn death_confirmation_requires_authoritative_zero_hp_and_death() {
    let mut w = world();
    let before = serde_json::to_value(&w).unwrap();
    assert!(w.accept(&p("ask_confirm_death")).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    w.accept(&p("own_death")).unwrap();
    assert!(w.accept(&p("ask_confirm_death")).is_err());
    w.accept(&p("own_hp_zero")).unwrap();
    w.accept(&p("ask_confirm_death")).unwrap();
    assert!(w.death_confirmation_pending);
    assert!(w.accept(&p("ask_confirm_death")).is_err());
    w.death_confirmation_pending = false;
    w.last_death_confirmed_action = w.own_actor().map(|a| a.action_sequence);
    assert!(w.accept(&p("ask_confirm_death")).is_err());
    assert!(w.attack_request(2).is_err());
    assert!(w.move_request(0).is_err());
    assert_eq!(p("confirm_death"), [0xe0]);
}
#[test]
fn formal_movement_release_does_not_cure_controls_or_predict_actor_state() {
    let mut w = world();
    for mode in [2, 6, 22] {w.accept(&[0x25, mode]).unwrap();}
    w.accept(&[0x63]).unwrap();
    let hp = w.hp;
    let mp = w.mp;
    let actors = serde_json::to_value(&w.actors).unwrap();
    let interruptions = w.control_interruptions;
    w.accept(&[0x25, 17]).unwrap();
    assert_eq!(w.movement_blocks.iter().copied().collect::<Vec<_>>(), [2, 6, 22]);
    assert_eq!(w.control_interruptions, interruptions);
    assert_eq!((w.hp, w.mp), (hp, mp));
    assert_eq!(serde_json::to_value(&w.actors).unwrap(), actors);
    assert_eq!(w.movement_locks, 1);
    let before = serde_json::to_value(&w).unwrap();
    for bad in [vec![0x25], vec![0x25, 17, 0], vec![0x25, 16], vec![0x25, 18]] {
        assert!(w.accept(&bad).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    for mode in [3, 7, 23] {w.accept(&[0x25, mode]).unwrap();}
    assert!(w.movement_blocks.is_empty());
}
#[test]
fn attack_feedback_distinguishes_reply_timeout_and_interrupted_authority() {
    use native_combat::{Feedback, Pending};
    use std::time::{Duration, Instant};
    let mut w = world();
    let start = Instant::now();
    let pending = Pending {
        since: start,
        count: 0,
        target: 2,
        generation: w.generation,
    };
    assert_eq!(pending.observe(&w, start), Feedback::Pending);
    assert_eq!(
        pending.observe(&w, start + Duration::from_secs(2)),
        Feedback::NoResponse
    );
    w.own_attack_packets = 1;
    assert_eq!(
        pending.observe(&w, start + Duration::from_secs(3)),
        Feedback::Acknowledged
    );
    w.generation += 1;
    assert_eq!(pending.observe(&w, start), Feedback::Interrupted);
    w.generation -= 1;
    w.own_attack_packets = 0;
    w.actors.remove(&2);
    assert_eq!(pending.observe(&w, start), Feedback::Interrupted);
    assert!(!native_combat::target_label(w.own_actor().unwrap()).contains("255%"));
    w.actors.get_mut(&1).unwrap().hp_ratio = 255;
    assert!(native_combat::target_label(w.own_actor().unwrap()).contains("未知"));
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
    own.speed = 0;
    own.hp_ratio = 100;
    w.status.as_mut().unwrap().object_id = 1;
    w.actors.insert(1, own);
    w
}
fn adjacent_world() -> World {
    let mut w = world();
    let bot = w.actors[&2].clone();
    let own = w.actors.get_mut(&1).unwrap();
    own.x = bot.x - 1;
    own.y = bot.y;
    w.hp = Some([50, 50]);
    w
}
#[test]
fn held_attack_waits_for_authority_and_animation_and_stops_on_release_timeout() {
    use native_combat::{Feedback as F, HeldAttack};
    let mut w = adjacent_world();
    let mut held = HeldAttack::default();
    held.press(2, 7, &w);
    assert_eq!(held.ready(7, &w, F::Idle, true, false), None);
    assert_eq!(held.ready(7, &w, F::Idle, true, true), Some(2));
    held.sent(&w);
    assert_eq!(held.ready(7, &w, F::Pending, true, true), None);
    assert_eq!(held.ready(7, &w, F::Acknowledged, true, true), None);
    w.own_attack_packets += 1;
    assert_eq!(held.ready(7, &w, F::Acknowledged, true, false), None);
    assert_eq!(held.ready(7, &w, F::Acknowledged, true, true), Some(2));
    held.sent(&w);
    held.clear(); // Release, move, target change, Escape, or focus loss cancels the press.
    w.own_attack_packets += 1;
    assert_eq!(held.ready(7, &w, F::Acknowledged, true, true), None);
    held.press(2, 7, &w);
    held.sent(&w);
    assert_eq!(held.ready(7, &w, F::NoResponse, true, true), None);
    assert!(!held.active());
    w.own_attack_packets += 1; // Late echo cannot rearm cancelled input.
    assert_eq!(held.ready(7, &w, F::Acknowledged, true, true), None);
    held.press(2, 7, &w); // A fresh user press can retry after timeout.
    assert_eq!(held.ready(7, &w, F::NoResponse, true, true), Some(2));
}
#[test]
fn held_attack_retires_world_owner_correction_and_invalid_targets() {
    use native_combat::{Feedback as F, HeldAttack};
    for change in 0..11 {
        let mut w = adjacent_world();
        let mut held = HeldAttack::default();
        held.press(2, 7, &w);
        held.sent(&w);
        w.own_attack_packets += 1;
        match change {
            0 => w.generation += 1,
            1 => w.local_corrections += 1,
            2 => w.actors.get_mut(&2).unwrap().dead = true,
            3 => {
                w.actors.remove(&2);
            }
            4 => w.actors.get_mut(&2).unwrap().x += 2,
            5 => {
                w.movement_blocks.insert(1);
            }
            6 => w.hp = Some([0, 50]),
            7 => w.battle_result = Some(1),
            _ => (),
        }
        assert_eq!(
            held.ready(
                if change == 8 { 8 } else { 7 },
                &w,
                if change == 9 {
                    F::Interrupted
                } else {
                    F::Acknowledged
                },
                change != 10,
                true
            ),
            None
        );
        assert!(!held.active(), "cancellation {change}");
    }
}
#[test]
fn formal_melee_vectors_and_atomic_rejection_never_infer_hp() {
    assert_eq!(
        native_combat::request(2, 32741, 32801).unwrap(),
        p("request")
    );
    assert_eq!(
        native_combat::decode_attack(&p("clamped"))
            .unwrap()
            .unwrap()
            .damage_marker,
        255
    );
    let mut w = world();
    let hp = w.hp;
    w.accept(&p("hit")).unwrap();
    assert_eq!(w.hp, hp);
    assert_eq!(w.actors[&2].hp_ratio, 100);
    assert_eq!(w.actors[&1].actions[0].action, 1);
    assert_eq!(w.actors[&2].actions[0].action, 2);
    let before = serde_json::to_value(&w).unwrap();
    for n in 0..16 {
        assert!(w.accept(&p("hit")[..n]).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    let mut trailing = p("hit");
    trailing.push(0);
    assert!(w.accept(&trailing).is_err());
    let mut heading = p("hit");
    heading[11] = 8;
    assert!(w.accept(&heading).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    let mut ranged = p("hit");
    ranged[12] = 1;
    assert!(w.accept(&ranged).is_err());
    w.accept(&p("miss")).unwrap();
    assert_eq!(w.actors[&2].actions.len(), 1);
    w.accept(&p("ratio_50")).unwrap();
    assert_eq!(w.actors[&2].hp_ratio, 50);
    assert_eq!(w.hp, hp);
}
#[test]
fn target_range_death_result_and_scoreboard_guards() {
    let mut w = world();
    assert!(w.attack_request(2).is_ok());
    assert!(w.attack_request(1).is_err());
    w.actors.get_mut(&2).unwrap().x += 3;
    assert!(w.attack_request(2).is_err());
    w.actors.get_mut(&2).unwrap().x -= 3;
    let before = serde_json::to_value(&w).unwrap();
    assert!(w.accept(&p("score")).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    w.accept(&p("death")).unwrap();
    assert!(w.actors[&2].dead);
    assert!(w.attack_request(2).is_err());
    w.accept(&p("result_FORCE_0_WON")).unwrap();
    assert_eq!(w.battle_result, Some(0));
    assert!(w.move_request(0).is_err());
    w.accept(&p("score")).unwrap();
    assert_eq!(w.scores[0].name, "測試角色");
    assert_eq!(w.scores[0].kills, 1);
    w.accept(&[0x28, 0x34, 2, 0]).unwrap();
    assert!(w.battle_result.is_none());
    assert!(w.scores.is_empty());
}
#[test]
fn held_attack_requires_a_new_press_after_even_a_brief_control_interruption(){
    use lito_modern::modern_lito::native_combat::{HeldAttack,Feedback};
    let mut w=world();let mut held=HeldAttack::default();
    for mode in [2,4,6,10,12,22,24] {
        held.press(2,2,&w);assert_eq!(held.ready(2,&w,Feedback::Idle,true,true),Some(2));
        w.accept(&[0x25,mode]).unwrap();w.accept(&[0x25,mode+1]).unwrap();
        assert!(w.attack_request(2).is_ok());assert_eq!(held.ready(2,&w,Feedback::Idle,true,true),None);assert!(!held.active());
    }
    held.press(2,2,&w);w.accept(&[0x63]).unwrap();assert_eq!(held.ready(2,&w,Feedback::Idle,true,true),None);
    held.press(2,2,&w);assert_eq!(held.ready(2,&w,Feedback::Idle,true,true),Some(2));
}
#[test]
fn orphan_and_malformed_updates_do_not_create_actors() {
    let mut w = world();
    w.actors.remove(&2);
    let count = w.actors.len();
    w.accept(&p("death")).unwrap();
    w.accept(&p("ratio_0")).unwrap();
    assert_eq!(w.orphan_updates, 2);
    assert_eq!(w.actors.len(), count);
    let before = serde_json::to_value(&w).unwrap();
    for b in [
        vec![0x68, 2, 0, 0, 0, 101],
        vec![0x20, 0, 0, 0, 0, 8],
        vec![0xd3, 2, 0, 0, 0],
    ] {
        assert!(w.accept(&b).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
}
#[test]
fn original_resource_attack_events_fire_once_and_death_holds() {
    use lito_modern::modern_lito::{
        native_world_animation::resolve,
        resources::{SharedArchive, SharedArchiveKind},
    };
    let root = std::path::Path::new(r"D:\lineage開發\0-客戶端");
    let bytes = SharedArchive::open(root, SharedArchiveKind::Text)
        .unwrap()
        .read_entry("list.spr")
        .unwrap();
    let catalog =
        SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default()).unwrap();
    let mut w = world();
    w.accept(&p("hit")).unwrap();
    w.accept(&p("counter")).unwrap();
    let mut playback = Playback::default();
    let own = &w.actors[&1];
    let (pose, mut events) = playback.sample(&catalog, own, 0).unwrap();
    assert_eq!(pose.unwrap().phase, "attack");
    let (_, more) = playback.sample(&catalog, own, 500_000_000).unwrap();
    events.extend(more);
    assert!(playback
        .sample(&catalog, own, 500_000_000)
        .unwrap()
        .1
        .is_empty());
    let (pose, more) = playback.sample(&catalog, own, 2_000_000_000).unwrap();
    events.extend(more);
    assert!(pose.is_none());
    assert_eq!(events.iter().filter(|e| e.sound == Some(1539)).count(), 1);
    assert_eq!(events.iter().filter(|e| e.event == "Hit").count(), 1);
    assert!(events.iter().any(|e| e.sound == Some(1537)));
    assert!(playback
        .sample(&catalog, own, 3_000_000_000)
        .unwrap()
        .1
        .is_empty());
    w.accept(&p("death")).unwrap();
    let bot = &w.actors[&2];
    assert_eq!(
        playback
            .sample(&catalog, bot, 3_000_000_000)
            .unwrap()
            .0
            .unwrap()
            .phase,
        "death"
    );
    let (pose, _) = playback.sample(&catalog, bot, 9_000_000_000).unwrap();
    let pose = pose.unwrap();
    assert_eq!(pose.phase, "death");
    assert_eq!(
        pose.elapsed_ns,
        resolve(&catalog, 61, 8).unwrap().duration_ns()
    );
    assert!(playback
        .sample(&catalog, bot, 10_000_000_000)
        .unwrap()
        .1
        .is_empty());
    w.accept(&packet(
        include_bytes!("fixtures/native-world-current.json"),
        "bot",
    ))
    .unwrap();
    assert!(playback
        .sample(&catalog, &w.actors[&2], 11_000_000_000)
        .unwrap()
        .0
        .is_none());
    // An arrival cannot borrow time from the preceding idle sample.
    let bot = &w.actors[&2];
    assert!(playback.impact(bot, 10_000));
    let (hurt, entered) = playback.sample(&catalog, bot, 11_000_000_000).unwrap();
    let hurt = hurt.unwrap();
    assert_eq!((hurt.phase, hurt.elapsed_ns), ("damage", 0));
    assert!(playback
        .sample(&catalog, bot, 11_000_000_000)
        .unwrap()
        .1
        .is_empty());
    // Replace and immediately attack before the next sample: a higher command
    // sequence must still retire the old presentation-only hurt.
    w.accept(&packet(
        include_bytes!("fixtures/native-world-current.json"),
        "bot",
    ))
    .unwrap();
    w.accept(&p("counter")).unwrap();
    let (pose, _) = playback
        .sample(&catalog, &w.actors[&2], 11_100_000_000)
        .unwrap();
    assert_eq!(pose.unwrap().phase, "attack");
    w.accept(&p("death")).unwrap();
    assert!(!playback.impact(&w.actors[&2], 10_001));
    let _ = entered;
}
