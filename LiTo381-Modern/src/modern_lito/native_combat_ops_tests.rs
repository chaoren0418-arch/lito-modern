use super::*;
use crate::modern_lito::native_combat::{self, Feedback, HeldAttack, Ledger, Pending};
use std::time::{Duration, Instant};

fn packet(file: &[u8], key: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_slice(file).unwrap();
    let s = v["packets"][key].as_str().unwrap();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}
/// Same offline fixture as tests/native_combat.rs: own id 1, bot id 2.
fn world() -> World {
    let mut w = World::default();
    for key in ["world", "start", "status", "actor", "bot"] {
        w.accept(&packet(include_bytes!("../../tests/fixtures/native-world-current.json"), key)).unwrap();
    }
    let mut own = w.actors.remove(&0xfedcba98).unwrap();
    own.id = 1;
    own.speed = 0;
    own.hp_ratio = 100;
    w.status.as_mut().unwrap().object_id = 1;
    w.actors.insert(1, own);
    w.hp = Some([50, 50]);
    w.status.as_mut().unwrap().weight_gauge = 0;
    w
}
fn adjacent() -> World {
    let mut w = world();
    let bot = w.actors[&2].clone();
    let own = w.actors.get_mut(&1).unwrap();
    own.x = bot.x - 1;
    own.y = bot.y;
    w
}
fn own_swing(attacker: u32, heading: u8) -> Vec<u8> {
    let mut b = vec![0x23, 1];
    b.extend(attacker.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    b.push(0);
    b.push(heading);
    b.extend(0u32.to_le_bytes());
    b
}
fn own_hit(attacker: u32, target: u32) -> Vec<u8> {
    let mut b = vec![0x23, 1];
    b.extend(attacker.to_le_bytes());
    b.extend(target.to_le_bytes());
    b.push(1);
    b.push(2);
    b.extend(0u32.to_le_bytes());
    b
}

#[test]
fn empty_request_matches_recorded_native_packet() {
    assert_eq!(
        native_combat::empty_request(0x7fde, 0x801d),
        [0x17, 0x00, 0x00, 0x00, 0x00, 0xde, 0x7f, 0x1d, 0x80]
    );
}

#[test]
fn empty_melee_targets_the_cell_ahead_and_builds_target_zero_c17() {
    let w = world();
    let own = w.own_actor().unwrap().clone();
    for heading in 0..8u8 {
        let (x, y) = crate::modern_lito::native_world::destination(own.x, own.y, heading).unwrap();
        assert_eq!(empty_melee_target(&w, false, heading), Ok([x, y]));
        let body = empty_melee_request(&w, false, heading).unwrap();
        assert_eq!(body, native_combat::empty_request(x, y));
        assert_eq!(&body[..5], [0x17, 0, 0, 0, 0]);
    }
    assert_eq!(empty_melee_target(&w, false, 8), Err(Refusal::InvalidCoordinate));
}

#[test]
fn empty_melee_refusals_follow_formal_empty_melee_conditions() {
    let base = world();
    let mut w = base.clone();
    w.hp = Some([0, 50]);
    assert_eq!(empty_melee_target(&w, false, 0), Err(Refusal::Dead));
    let mut w = base.clone();
    w.actors.get_mut(&1).unwrap().dead = true;
    assert_eq!(empty_melee_target(&w, false, 0), Err(Refusal::Dead));
    assert_eq!(empty_melee_target(&base, true, 0), Err(Refusal::Observer));
    let mut w = base.clone();
    w.actors.get_mut(&1).unwrap().draw_state.invisible = true;
    assert_eq!(empty_melee_target(&w, false, 0), Err(Refusal::Invisible));
    let mut w = base.clone();
    w.movement_blocks.insert(2);
    assert_eq!(empty_melee_target(&w, false, 0), Err(Refusal::Paralyzed));
    let mut w = base.clone();
    w.status.as_mut().unwrap().weight_gauge = OVERWEIGHT_GAUGE;
    assert_eq!(empty_melee_target(&w, false, 0), Err(Refusal::Overweight));
    w.status.as_mut().unwrap().weight_gauge = OVERWEIGHT_GAUGE - 1;
    assert!(empty_melee_target(&w, false, 0).is_ok());
    for stance in [20, 62] {
        let mut w = base.clone();
        w.actors.get_mut(&1).unwrap().walk_action = stance;
        assert_eq!(empty_melee_target(&w, false, 0), Err(Refusal::RangedStance));
    }
    let mut w = base.clone();
    w.actors.get_mut(&1).unwrap().x = 1;
    assert_eq!(empty_melee_target(&w, false, 6), Err(Refusal::InvalidCoordinate));
    let mut w = base.clone();
    w.battle_result = Some(0);
    assert_eq!(empty_melee_target(&w, false, 0), Err(Refusal::BattleEnded));
    for r in [Refusal::Dead, Refusal::Observer, Refusal::Invisible, Refusal::Paralyzed,
        Refusal::Overweight, Refusal::RangedStance, Refusal::InvalidCoordinate, Refusal::Pacing] {
        assert!(!r.text().is_empty());
        assert_eq!(Feedback::Refused(r).text(), r.text());
    }
}

#[test]
fn native_screen_mirror_matches_formal_bounds() {
    assert!(in_native_screen(100, 100, 109, 109));
    assert!(!in_native_screen(100, 100, 110, 110));
    assert!(!in_native_screen(100, 100, 120, 100));
    // Distance 19: edge distance from (sx-18, sy-18) must be 19..=52.
    assert!(!in_native_screen(100, 100, 119, 100)); // edge 37 + 18 = 55
    assert!(in_native_screen(100, 100, 81, 100)); // edge 1 + 18 = 19
}

#[test]
fn native_screen_distance_19_uses_edge_rule() {
    for (tx, ty) in [(119u16, 100u16), (100, 119), (81, 100), (100, 81), (110, 109)] {
        let (sx, sy) = (100i32, 100i32);
        let edge = (i32::from(tx) - (sx - 18)).abs() + (i32::from(ty) - (sy - 18)).abs();
        assert_eq!(in_native_screen(100, 100, tx, ty), (19..=52).contains(&edge), "{tx},{ty}");
    }
}

#[test]
fn attack_interval_uses_attack_only_multipliers() {
    assert_eq!(attack_interval_ms(1000, 0, 0), 1000);
    assert_eq!(attack_interval_ms(1000, 1, 0), 755);
    assert_eq!(attack_interval_ms(1000, 2, 0), 2000);
    assert_eq!(attack_interval_ms(1000, 0, 6), 775);
    assert_eq!(attack_interval_ms(1000, 0, 1), 755);
    assert_eq!(attack_interval_ms(1000, 0, 5), 755);
    assert_eq!(attack_interval_ms(1000, 0, 3), 769);
    // Movement-only brave4 does not accelerate attacks.
    assert_eq!(attack_interval_ms(1000, 0, 4), 1000);
    assert_eq!(attack_interval_ms(1000, 1, 1), 570);
    assert_eq!(attack_interval_ms(0, 0, 0), 1);
}

#[test]
fn pacer_refuses_inside_interval_only() {
    let now = Instant::now();
    let mut p = Pacer::default();
    assert_eq!(p.admit(now), Ok(()));
    p.commit(now, 600);
    assert_eq!(p.admit(now + Duration::from_millis(599)), Err(Refusal::Pacing));
    assert_eq!(p.admit(now + Duration::from_millis(600)), Ok(()));
    p.clear();
    assert_eq!(p.admit(now), Ok(()));
}

#[test]
fn chase_step_is_greedy_bounded_and_respects_gate() {
    let open = |_: i32, _: i32, _: u8| true;
    assert_eq!(chase_step([10, 10], [11, 11], 1, open), ChaseStep::InReach);
    assert_eq!(chase_step([10, 10], [10, 5], 1, open), ChaseStep::Step(0));
    assert_eq!(chase_step([10, 10], [15, 15], 1, open), ChaseStep::Step(3));
    assert_eq!(chase_step([10, 10], [5, 10], 1, open), ChaseStep::Step(6));
    assert_eq!(chase_step([10, 10], [30, 10], 19, open), ChaseStep::Step(2));
    assert_eq!(chase_step([10, 10], [29, 10], 19, open), ChaseStep::InReach);
    // Direct heading blocked: a neighbour that still reduces distance.
    let no_direct = |_: i32, _: i32, h: u8| h != 3;
    assert_eq!(chase_step([10, 10], [15, 15], 1, no_direct), ChaseStep::Step(2));
    // Straight line: neighbours (diagonals) still reduce Chebyshev distance.
    assert_eq!(chase_step([10, 10], [10, 5], 1, |_, _, h| h != 0), ChaseStep::Step(7));
    assert_eq!(chase_step([10, 10], [15, 15], 1, |_, _, _| false), ChaseStep::Blocked);
    assert!(CHASE_STEP_LIMIT > 0);
}

#[test]
fn target_zero_swing_reply_is_decoded_and_acknowledges_empty_request() {
    let mut w = world();
    let start = Instant::now();
    let pending = Pending::new(&w, 0, start);
    // Target zero is never "absent": the empty request waits for its reply.
    assert_eq!(pending.observe(&w, start), Feedback::Pending);
    let a = native_combat::decode_attack(&own_swing(1, 4)).unwrap().unwrap();
    assert_eq!((a.target, a.damage_marker, a.heading), (0, 0, 4));
    let mut damaged = own_swing(1, 4);
    damaged[10] = 1;
    assert!(native_combat::decode_attack(&damaged).is_err());
    let hp = w.hp;
    w.accept(&own_swing(1, 4)).unwrap();
    assert_eq!(w.own_attack_packets, 1);
    assert_eq!(w.own_attack_log, [(1, 0)]);
    assert_eq!(w.hp, hp);
    assert_eq!(w.actors[&1].heading, 4);
    assert_eq!(pending.observe(&w, start), Feedback::Acknowledged);
}

#[test]
fn late_reply_for_other_target_does_not_acknowledge_retry() {
    let mut w = adjacent();
    let start = Instant::now();
    let mut ledger = Ledger::default();
    // First request to the bot timed out, then the player retried an empty swing.
    let first = Pending::new(&w, 2, start);
    assert_eq!(first.observe(&w, start + Duration::from_secs(2)), Feedback::NoResponse);
    let retry = Pending::new(&w, 0, start + Duration::from_secs(2));
    // The late reply for the bot arrives during the retry.
    w.accept(&own_hit(1, 2)).unwrap();
    assert_eq!(retry.observe(&w, start + Duration::from_millis(2100)), Feedback::Pending);
    assert_eq!(ledger.observe(&w, Some(&retry)), 1);
    assert_eq!((ledger.late, ledger.acknowledged), (1, 0));
    // The retry's own reply then acknowledges it.
    w.accept(&own_swing(1, 2)).unwrap();
    assert_eq!(retry.observe(&w, start + Duration::from_millis(2200)), Feedback::Acknowledged);
    assert_eq!(ledger.observe(&w, Some(&retry)), 0);
    assert_eq!((ledger.late, ledger.acknowledged), (1, 1));
    // A reply with no pending request is late.
    w.accept(&own_hit(1, 2)).unwrap();
    assert_eq!(ledger.observe(&w, None), 1);
    // A replaced world resets without counting history.
    let fresh = world();
    assert_eq!(ledger.observe(&fresh, None), 0);
    assert_eq!(ledger.seen, 0);
}

#[test]
fn same_target_retry_after_timeout_is_acknowledged_by_next_reply() {
    let mut w = adjacent();
    let start = Instant::now();
    let retry = Pending::new(&w, 2, start);
    w.accept(&own_hit(1, 2)).unwrap();
    // No request sequence: a same-target reply acknowledges the newest request.
    assert_eq!(retry.observe(&w, start), Feedback::Acknowledged);
    // A request committed after that reply is not acknowledged by it.
    let next = Pending::new(&w, 2, start);
    assert_eq!(next.observe(&w, start), Feedback::Pending);
}

#[test]
fn overweight_message_rejects_pending_attack_with_hint() {
    let mut w = adjacent();
    let start = Instant::now();
    let pending = Pending::new(&w, 2, start);
    w.message_count += 1;
    w.last_message = Some((native_combat::MESSAGE_ATTACK_OVERWEIGHT, vec![]));
    assert_eq!(pending.observe(&w, start), Feedback::Rejected(110));
    assert_eq!(Feedback::Rejected(110).text(), "負重過重，無法攻擊。");
    // Unrelated message codes do not end the wait.
    let pending = Pending::new(&w, 2, start);
    w.message_count += 1;
    w.last_message = Some((268, vec![]));
    assert_eq!(pending.observe(&w, start), Feedback::Pending);
    assert!(!Feedback::Rejected(1).text().is_empty());
}

#[test]
fn chase_press_survives_out_of_reach_and_plain_press_retires() {
    let mut w = world();
    {
        let bot = w.actors[&2].clone();
        let own = w.actors.get_mut(&1).unwrap();
        own.x = bot.x - 5;
        own.y = bot.y;
    }
    assert!(w.attack_out_of_reach(2));
    assert_eq!(w.attack_reach(2).map(|r| r.0), Some(5));
    let mut held = HeldAttack::default();
    held.press(2, 7, &w);
    assert_eq!(held.ready(7, &w, Feedback::Idle, true, true), None);
    assert!(!held.active());
    held.press_chase(2, 7, &w);
    assert_eq!(held.ready(7, &w, Feedback::Idle, true, true), None);
    assert!(held.active() && held.chasing());
    // A stale NoResponse from before the press does not end the chase.
    assert_eq!(held.ready(7, &w, Feedback::NoResponse, true, true), None);
    assert!(held.active());
    // Arriving in reach turns the same press into a normal attack.
    {
        let bot = w.actors[&2].clone();
        w.actors.get_mut(&1).unwrap().x = bot.x - 1;
    }
    assert!(!w.attack_out_of_reach(2));
    assert_eq!(held.ready(7, &w, Feedback::Idle, true, true), Some(2));
    // Dead target is not chased.
    let mut dead = w.clone();
    dead.actors.get_mut(&2).unwrap().dead = true;
    assert!(!dead.attack_out_of_reach(2) && dead.attack_reach(2).is_none());
}

#[test]
fn target_ring_tone_reflects_local_gates() {
    let mut w = adjacent();
    assert_eq!(target_ring(&w, None), None);
    assert_eq!(target_ring(&w, Some(99)), None);
    let ring = target_ring(&w, Some(2)).unwrap();
    assert_eq!((ring.id, ring.cell, ring.tone), (2, [w.actors[&2].x, w.actors[&2].y], RingTone::InReach));
    w.actors.get_mut(&1).unwrap().x -= 3;
    assert_eq!(target_ring(&w, Some(2)).unwrap().tone, RingTone::OutOfReach);
    assert_eq!(target_ring(&w, Some(1)).unwrap().tone, RingTone::Neutral);
    w.actors.get_mut(&2).unwrap().dead = true;
    assert_eq!(target_ring(&w, Some(2)).unwrap().tone, RingTone::Dead);
    let tones = [RingTone::InReach, RingTone::OutOfReach, RingTone::Dead, RingTone::Neutral];
    for (i, a) in tones.iter().enumerate() {
        for b in &tones[i + 1..] {
            assert_ne!(a.rgb(), b.rgb());
        }
    }
}

/// Alpha hit: click ownership is the per-pixel owner plane written by the
/// raster; transparent and translucent sprite pixels never claim a click.
#[test]
fn alpha_hit_owner_plane_ignores_transparent_and_translucent_pixels() {
    let mut canvas = crate::modern_lito::compositor::Canvas::new(24, 24).unwrap();
    let mut owners = vec![0u16; 24 * 24];
    let mut rgba = vec![0u8; 24 * 24 * 4];
    for (i, alpha) in [(0usize, 255u8), (1, 128), (2, 0), (3, 255)] {
        rgba[i * 4..i * 4 + 4].copy_from_slice(&[200, 100, 50, alpha]);
    }
    canvas.blit_world_fragment(&rgba, 0, 0, &mut owners, 7).unwrap();
    assert_eq!(&owners[..5], [7, 0, 0, 7, 0]);
    // A later opaque fragment of another owner takes the overlapped pixel only.
    let mut top = vec![0u8; 24 * 24 * 4];
    top[3 * 4..3 * 4 + 4].copy_from_slice(&[1, 2, 3, 255]);
    canvas.blit_world_fragment(&top, 0, 0, &mut owners, 9).unwrap();
    assert_eq!(&owners[..5], [7, 0, 0, 9, 0]);
}

#[test]
fn ring_bounds_center_on_projected_cell_and_clip_offscreen() {
    use crate::modern_lito::world_map::{PixelPoint, ViewProjection, WorldCell};
    let projection = ViewProjection { anchor_cell: WorldCell { x: 100, y: 100 }, anchor_pixel: PixelPoint { x: 400, y: 300 } };
    assert_eq!(ring_bounds(projection, [100, 100]), Some([376, 288, 424, 312]));
    assert_eq!(ring_bounds(projection, [101, 100]), Some([400, 276, 448, 300]));
    assert_eq!(ring_bounds(projection, [200, 200]), None);
}
