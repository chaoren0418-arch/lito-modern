//! Issue #162: native hurt guard, 4E63C0 threshold and 4E45F0 interruption
//! on an isolated synthetic catalog. No resources, World or authority writes.
use super::*;
use crate::modern_lito::{native_combat::ActionCue, sprite_actions::ActionLimits};

const MS: u64 = 1_000_000;

fn catalog() -> SpriteActionCatalog {
    let line = |id: u16| {
        format!(
            "#{id} 16 test 0.walk(1 2,0.0:12 0.1:12) 1.attack(1 3,0.0:12 0.1:12! 0.2:12) \
             2.hurt(1 1,16.0:2) 3.idle(1 2,8.0:12 8.1:12) 8.death(1 1,24.0:2) 44.hold(1 1,0.0:240)\n"
        )
    };
    let text: String = [0u16, 5902, 5920, 1245].into_iter().map(line).collect();
    SpriteActionCatalog::parse(text.as_bytes(), ActionLimits::default()).unwrap()
}

fn actor(graphics: u16) -> Actor {
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/native-world-current.json"
    )))
    .unwrap();
    let hex = fixture["packets"]["actor"].as_str().unwrap();
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    let mut a = crate::modern_lito::native_world::actor(&bytes).unwrap();
    a.graphics = graphics;
    a.speed = 0;
    a.brave = 0;
    a.walk_action = 0;
    a.dead = false;
    a.incarnation = 1;
    a.actions.clear();
    a.action_sequence = 0;
    a.motion_from = None;
    a
}

fn push(a: &mut Actor, action: u8) {
    a.action_sequence += 1;
    a.actions.push(ActionCue { sequence: a.action_sequence, action, source: "action" });
}

fn pending(p: &Playback, a: &Actor) -> Vec<u8> {
    p.states[&a.id].queue.iter().map(|c| c.action).collect()
}

#[test]
fn server_hurts_queue_until_more_than_three_pending_then_drop_consecutive_action2() {
    let c = catalog();
    let mut p = Playback::default();
    let mut a = actor(0);
    push(&mut a, 1);
    let (pose, _) = p.sample(&c, &a, 0).unwrap();
    let pose = pose.unwrap();
    assert_eq!((pose.action, pose.phase), (1, "action"));
    for n in 1..=8u64 {
        push(&mut a, 2);
        p.sample(&c, &a, n * MS).unwrap();
    }
    // 4E63C0 appends while pending <= 3 (the old modern policy kept one).
    assert_eq!(pending(&p, &a), [2, 2, 2, 2]);
    // An attack is never suppressed; the hurt behind it starts a new run.
    push(&mut a, 1);
    p.sample(&c, &a, 9 * MS).unwrap();
    push(&mut a, 2);
    p.sample(&c, &a, 10 * MS).unwrap();
    push(&mut a, 2);
    p.sample(&c, &a, 11 * MS).unwrap();
    assert_eq!(pending(&p, &a), [2, 2, 2, 2, 1, 2]);
    // Both attacks still enter their Hit frame exactly once.
    let mut hits = 0;
    let mut now = 12 * MS;
    while now < 20_000 * MS {
        hits += p.sample(&c, &a, now).unwrap().1.iter().filter(|e| e.action == 1 && e.event == "Hit").count();
        now += 50 * MS;
    }
    assert_eq!(hits, 2);
    assert!(pending(&p, &a).is_empty());
}

#[test]
fn exempt_bodies_never_take_server_or_arrival_hurt() {
    let c = catalog();
    for graphics in [5902u16, 5920, 1245] {
        let mut p = Playback::default();
        let mut a = actor(graphics);
        push(&mut a, 2);
        let (pose, _) = p.sample(&c, &a, 0).unwrap();
        assert!(pose.is_none(), "{graphics}");
        assert!(!p.impact(&a, 99), "{graphics}");
        push(&mut a, 1);
        push(&mut a, 2);
        let (pose, _) = p.sample(&c, &a, MS).unwrap();
        let pose = pose.unwrap();
        assert_eq!((pose.action, pose.phase), (1, "action"), "{graphics}");
        assert!(pending(&p, &a).is_empty(), "{graphics}");
    }
}

#[test]
fn queued_hurt_is_dropped_at_start_after_polymorph_into_exempt_body() {
    let c = catalog();
    let mut p = Playback::default();
    let mut a = actor(0);
    push(&mut a, 1);
    p.sample(&c, &a, 0).unwrap();
    push(&mut a, 2);
    p.sample(&c, &a, MS).unwrap();
    assert_eq!(pending(&p, &a), [2]);
    a.graphics = 5902;
    let (pose, _) = p.sample(&c, &a, 10_000 * MS).unwrap();
    assert!(pose.is_none());
    assert!(pending(&p, &a).is_empty());
}

#[test]
fn arrival_interrupts_a_standing_attack_and_skips_its_remaining_hit() {
    let c = catalog();
    let mut p = Playback::default();
    let mut a = actor(0);
    push(&mut a, 1);
    p.sample(&c, &a, 0).unwrap();
    push(&mut a, 2);
    p.sample(&c, &a, MS).unwrap();
    assert!(p.impact(&a, 500));
    // 4E45F0 replaces the running action; pending commands stay behind it.
    assert!(p.states[&a.id].active.is_none());
    assert_eq!(pending(&p, &a), [2, 2]);
    let (pose, events) = p.sample(&c, &a, MS).unwrap();
    let pose = pose.unwrap();
    assert_eq!((pose.phase, pose.elapsed_ns), ("damage", 0));
    assert!(events.iter().all(|e| e.action == 2));
    let mut later = vec![];
    for n in 2..400u64 {
        later.extend(p.sample(&c, &a, n * 50 * MS).unwrap().1);
    }
    assert!(later.iter().all(|e| !(e.action == 1 && e.event == "Hit")));
}

#[test]
fn arrival_during_running_walk_enqueues_with_the_same_threshold() {
    let c = catalog();
    let mut p = Playback::default();
    let a = actor(0);
    p.hold_for_motion(&a, 0).unwrap();
    for n in 0..4 {
        assert!(p.impact(&a, 100 + n), "pending {n}");
    }
    assert_eq!(pending(&p, &a), [2, 2, 2, 2]);
    assert!(!p.impact(&a, 200));
    // After the walk boundary the body is standing again: interruption.
    p.sample(&c, &a, MS).unwrap();
    assert!(!p.states[&a.id].walking);
    assert!(p.impact(&a, 201));
    assert_eq!(pending(&p, &a).len(), 4);
}

#[test]
fn arrival_is_ignored_for_44_45_death_and_pending_death() {
    let c = catalog();
    let mut p = Playback::default();
    let mut a = actor(0);
    push(&mut a, 44);
    p.sample(&c, &a, 0).unwrap();
    assert!(!p.impact(&a, 9));
    let mut b = actor(0);
    b.id = a.id.wrapping_add(1);
    push(&mut b, 8);
    p.sample(&c, &b, 0).unwrap();
    assert!(!p.impact(&b, 9));
    let mut d = actor(0);
    d.id = a.id.wrapping_add(2);
    push(&mut d, 1);
    p.sample(&c, &d, 0).unwrap();
    p.states.get_mut(&d.id).unwrap().queue.push_back(ActionCue { sequence: 50, action: 8, source: "action" });
    assert!(!p.impact(&d, 9));
}
