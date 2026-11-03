//! Frozen real Java receipts; private clock edits are explicitly arbitration-only.
use super::*;
use crate::modern_lito::{application_protocol as app, npc_protocol as npc};
#[path = "npc_disintegrate_frame_golden.rs"]
mod golden;
pub(crate) fn reply(name: &str) -> npc::Reply {
    let hex = golden::VECTORS.iter().find(|(n, _)| *n == name).unwrap().1;
    let bytes = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    let app::Response::Npc(r) = app::decode_response_version(&bytes, 29).unwrap().response else {
        panic!("NPC fixture")
    };
    r.validate().unwrap();
    r
}
fn frame(n: &str) -> Frame {
    reply(n).frame.unwrap().combat
}
fn timing(_: BaseAppearance) -> Result<(Duration, Duration)> {
    Ok((Duration::from_millis(300), Duration::from_millis(750)))
}

#[test]
fn npc77_later_typed_melee_cancels_pending_even_when_coalesced() {
    let before = reply("normal-before").frame.unwrap();
    let hit = reply("normal-hit").frame.unwrap();
    let mut melee = hit.clone();
    // Arbitration-only input, not an invented valid network snapshot. NPC
    // melee deliberately does not update the PvP Vital.attack_revision.
    let mut exchange = npc::tests::read_reply()
        .frame
        .unwrap()
        .exchanges
        .into_iter()
        .find(|e| e.direction == npc::Direction::PlayerToNpc)
        .unwrap();
    exchange.participant = hit.combat.world.actor(1).unwrap().identity();
    exchange.revision = hit.combat.world.revision + 1;
    exchange.npc.scope = hit.scope;
    exchange.npc.serial = hit.actors[0].serial;
    exchange.npc_life = hit.actors[0].life;
    melee.exchanges = vec![exchange];
    assert!(
        super::super::npc_presentation::player_attack(&melee, 1)
            .unwrap()
            .revision
            > revision(&hit.combat, 1)
    );
    assert_eq!(melee.combat.vitals, hit.combat.vitals);
    let frozen = melee.clone();
    for coalesced in [false, true] {
        let t = Instant::now();
        let mut tracker = Tracker::new(&before.combat);
        let first = if coalesced { &melee } else { &hit };
        tracker
            .update_atomic(&first.combat, Some(first), t, timing)
            .unwrap();
        assert!(
            tracker
                .update_atomic(
                    &melee.combat,
                    Some(&melee),
                    t + Duration::from_millis(300),
                    timing
                )
                .unwrap()
                .is_empty(),
            "typed melee must cancel the pending skill77"
        );
        assert!(tracker.active.is_empty());
    }
    assert_eq!(melee, frozen);
}
#[test]
fn npc77_typed_receipt_launches_once_and_cooldown_does_not_restart() {
    for prefix in ["normal", "critical"] {
        let before = frame(&format!("{prefix}-before"));
        let hit = frame(&format!("{prefix}-hit"));
        let denied = frame(&format!("{prefix}-denied"));
        let t = Instant::now();
        let mut tracker = Tracker::new(&before);
        assert!(tracker.update(&hit, t, timing).unwrap().is_empty());
        let receipt = tracker.active[&1].event.hit.unwrap();
        assert!(
            matches!(receipt.target,Target::Npc{id,life:1} if id.serial==1 && id.scope.match_id==hit.world.match_id)
        );
        let events = tracker
            .update(&denied, t + Duration::from_millis(300), timing)
            .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].binding.target, receipt.to);
        assert_eq!(
            events[0].variant,
            if prefix == "critical" {
                Variant::Critical
            } else {
                Variant::Normal
            }
        );
        assert!(tracker
            .update(&denied, t + Duration::from_millis(500), timing)
            .unwrap()
            .is_empty());
        assert_eq!(denied.npc_disintegrations, hit.npc_disintegrations);
    }
}
#[test]
fn npc77_later_arrow_interrupts_body_and_newer_npc77_receipt_wins() {
    let t = Instant::now();
    let mut tracker = Tracker::new(&frame("normal-before"));
    tracker.update(&frame("normal-hit"), t, timing).unwrap();
    assert!(tracker
        .update(
            &frame("normal-arrow"),
            t + Duration::from_millis(100),
            timing
        )
        .unwrap()
        .is_empty());
    assert!(tracker.active.is_empty());
    let second = frame("normal-second");
    tracker
        .update(&second, t + Duration::from_secs(5), timing)
        .unwrap();
    let e = tracker
        .update(&second, t + Duration::from_millis(5300), timing)
        .unwrap();
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].binding.start_revision, revision(&second, 1) as u64);
}
#[test]
fn npc77_sparse_quest_corpse_retirement_keeps_owned_cell_not_target_pointer() {
    let hit = frame("quest-own");
    let retired = frame("quest-retired");
    let t = Instant::now();
    let mut tracker = Tracker::new(&hit);
    // Force only the presentation baseline to pre-receipt; authority frames remain untouched.
    tracker.baseline.values_mut().for_each(|r| *r = 0);
    tracker.update(&hit, t, timing).unwrap();
    assert!(tracker.active.contains_key(&2));
    assert!(!tracker.active.contains_key(&1));
    let e = tracker
        .update(&retired, t + Duration::from_millis(300), timing)
        .unwrap();
    assert_eq!(e.len(), 1);
    assert_eq!(
        e[0].binding.target,
        hit.npc_disintegrate(2).unwrap().hit.as_ref().unwrap().to
    );
}
#[test]
fn npc77_quest_new_caster_life_cancels_pending_but_target_life_is_retained() {
    let hit = frame("quest-own");
    let t = Instant::now();
    let mut tracker = Tracker::new(&hit);
    tracker.baseline.values_mut().for_each(|r| *r = 0);
    tracker.update(&hit, t, timing).unwrap();
    let mut next = hit.clone();
    // Arbitration-only clock/life input; not a new wire fixture.
    next.world
        .quest
        .as_mut()
        .unwrap()
        .lives
        .iter_mut()
        .find(|l| l.entity_id == 2)
        .unwrap()
        .respawns += 1;
    assert!(tracker
        .update(&next, t + Duration::from_millis(300), timing)
        .unwrap()
        .is_empty());
    assert!(tracker.active.is_empty());
}
#[test]
fn npc77_baseline_terminal_saved_and_render_stall_do_not_replay() {
    let t = Instant::now();
    for name in ["normal-hit", "time-attack-finished", "time-attack-saved"] {
        let f = frame(name);
        let mut tracker = Tracker::new(&f);
        assert!(tracker.update(&f, t, timing).unwrap().is_empty());
        assert!(tracker.active.is_empty());
    }
    let mut tracker = Tracker::new(&frame("normal-before"));
    let hit = frame("normal-hit");
    tracker.update(&hit, t, timing).unwrap();
    assert!(tracker
        .update(&hit, t + Duration::from_secs(10), timing)
        .unwrap()
        .is_empty());
    assert!(tracker.active.is_empty());
}
#[test]
fn npc77_world_reset_and_cross_domain_alias_cannot_reuse_cast_body() {
    let t = Instant::now();
    let hit = frame("normal-hit");
    let mut tracker = Tracker::new(&frame("normal-before"));
    tracker.update(&hit, t, timing).unwrap();
    let mut next = hit.clone();
    next.world.generation += 1;
    assert!(tracker
        .update(&next, t + Duration::from_millis(300), timing)
        .unwrap()
        .is_empty());
    assert!(tracker.active.is_empty());
    let mut mixed = hit;
    mixed.disintegrations = super::tests::frame("normal-hit").disintegrations;
    assert!(Tracker::new(&mixed).update(&mixed, t, timing).is_err());
}

#[test]
fn npc77_cooperative_departure_removes_body_even_while_frozen_roster_retains_caster() {
    let hit = frame("time-attack-finished");
    let id = hit
        .npc_disintegrations
        .iter()
        .find(|s| s.revision > 0)
        .unwrap()
        .entity_id;
    for launched in [false, true] {
        let t = Instant::now();
        let mut tracker = Tracker::new(&hit);
        // Clock/arbitration-only setup; the decoded Java terminal receipt itself
        // stays unchanged. This is not a new fabricated transport snapshot.
        *tracker.baseline.get_mut(&id).unwrap() = 0;
        tracker.update(&hit, t, timing).unwrap();
        assert!(tracker.active.contains_key(&id));
        if launched {
            assert_eq!(
                tracker
                    .update(&hit, t + Duration::from_millis(300), timing)
                    .unwrap()
                    .len(),
                1
            );
        }
        let mut departed = hit.clone();
        departed.world.time_attack.as_mut().unwrap().departed_mask |= 1 << (id - 1);
        assert!(
            departed.world.actor(id).is_some(),
            "result roster deliberately retains departed members"
        );
        assert!(tracker
            .update(
                &departed,
                t + Duration::from_millis(if launched { 400 } else { 100 }),
                timing
            )
            .unwrap()
            .is_empty());
        assert!(
            !tracker.active.contains_key(&id),
            "departed member cannot override a body absent from SceneSnapshot"
        );
    }
}
