use super::*;
use crate::modern_lito::{application_protocol as app, combat_protocol as combat};
use base64::Engine;
use sha2::{Digest, Sha256};
use std::io::Read;
#[path = "disintegrate_wire_golden.rs"]
mod golden;

pub(crate) fn reply(name: &str) -> combat::Reply {
    let (_, len, sha, text) = golden::GOLDENS.iter().find(|(n, ..)| *n == name).unwrap();
    let zip = base64::engine::general_purpose::STANDARD
        .decode(text)
        .unwrap();
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(&zip[..])
        .take((*len + 1) as u64)
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes.len(), *len);
    assert_eq!(format!("{:X}", Sha256::digest(&bytes)), *sha);
    let app::Response::Combat(r) = app::decode_response_version(&bytes, 27).unwrap().response
    else {
        panic!("Java combat fixture")
    };
    r.validate().unwrap();
    r
}
pub(crate) fn frame(name: &str) -> Frame {
    reply(name).frame.unwrap()
}
fn timing(_: BaseAppearance) -> Result<(Duration, Duration)> {
    // Explicit timing fixture; production obtains both from qualified caster18.
    Ok((Duration::from_millis(300), Duration::from_millis(750)))
}

#[test]
fn disintegrate_receipt_once_denial_does_not_restart_fixed_cell_launch() {
    for prefix in ["normal", "critical"] {
        let initial = frame(&format!("{prefix}-active"));
        let hit = frame(&format!("{prefix}-hit"));
        let denied = frame(&format!("{prefix}-denied"));
        let frozen = hit.clone();
        let t = Instant::now();
        let mut tracker = Tracker::new(&initial);
        assert!(tracker.update(&hit, t, timing).unwrap().is_empty());
        assert!(tracker
            .update(&denied, t + Duration::from_millis(200), timing)
            .unwrap()
            .is_empty());
        let launches = tracker
            .update(&denied, t + Duration::from_millis(300), timing)
            .unwrap();
        assert_eq!(launches.len(), 1);
        let launch = &launches[0];
        assert_eq!(launch.started, t + Duration::from_millis(300));
        assert_eq!(
            launch.binding.target,
            hit.disintegrations[0].hit.as_ref().unwrap().to
        );
        assert_eq!(
            launch.variant,
            if prefix == "critical" {
                Variant::Critical
            } else {
                Variant::Normal
            }
        );
        assert_eq!(
            launch.binding.start_revision,
            hit.disintegrations[0].revision as u64
        );
        assert!(tracker
            .update(&denied, t + Duration::from_secs(1), timing)
            .unwrap()
            .is_empty());
        assert!(tracker.active.is_empty());
        assert_eq!(hit, frozen, "presentation cannot apply damage or MP");
        assert_eq!(
            launch.binding.target,
            frozen.disintegrations[0].hit.as_ref().unwrap().to
        );
    }
}
#[test]
fn disintegrate_loading_baseline_and_late_stall_never_replay_history() {
    let hit = frame("normal-hit");
    let t = Instant::now();
    let mut baseline = Tracker::new(&hit);
    assert!(baseline.update(&hit, t, timing).unwrap().is_empty());
    assert!(baseline.active.is_empty());
    let mut tracker = Tracker::new(&frame("normal-active"));
    tracker.update(&hit, t, timing).unwrap();
    assert!(tracker
        .update(&hit, t + Duration::from_secs(10), timing)
        .unwrap()
        .is_empty());
    assert!(tracker.active.is_empty());
    assert!(tracker
        .update(&hit, t + Duration::from_secs(11), timing)
        .unwrap()
        .is_empty());
    let mut tracker = Tracker::new(&frame("normal-active"));
    tracker.update(&hit, t, timing).unwrap();
    let launches = tracker
        .update(&hit, t + Duration::from_millis(900), timing)
        .unwrap();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].started,
        t + Duration::from_millis(300),
        "late paint retains 600ms age; no fresh sound"
    );
}
#[test]
fn disintegrate_newer_fire_cancels_pending_body_but_coalesced_new_dis_wins() {
    let t = Instant::now();
    let mut tracker = Tracker::new(&frame("mixed-active"));
    tracker.update(&frame("mixed-dis"), t, timing).unwrap();
    assert_eq!(tracker.active.len(), 1);
    assert!(tracker
        .update(&frame("mixed-fire"), t + Duration::from_millis(100), timing)
        .unwrap()
        .is_empty());
    assert!(tracker.active.is_empty());
    let second = frame("mixed-second");
    tracker
        .update(&second, t + Duration::from_secs(1), timing)
        .unwrap();
    let launches = tracker
        .update(&second, t + Duration::from_millis(1300), timing)
        .unwrap();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].binding.start_revision,
        second.disintegrations[0].revision as u64
    );
    let mut coalesced = Tracker::new(&frame("mixed-active"));
    coalesced.update(&second, t, timing).unwrap();
    assert_eq!(
        coalesced
            .update(&second, t + Duration::from_millis(300), timing)
            .unwrap()
            .len(),
        1,
        "only latest retained receipt, not invented intermediate casts"
    );
}
#[test]
fn disintegrate_final_kill_and_target_respawn_do_not_cancel_fixed_receipt() {
    let t = Instant::now();
    for (before, hit_name, next) in [
        ("terminal-active", "terminal-finished", "terminal-finished"),
        ("honor-active", "honor-death", "honor-respawn"),
    ] {
        let hit = frame(hit_name);
        let after = frame(next);
        let mut tracker = Tracker::new(&frame(before));
        tracker.update(&hit, t, timing).unwrap();
        let launches = tracker
            .update(&after, t + Duration::from_millis(300), timing)
            .unwrap();
        assert_eq!(launches.len(), 1, "{next}");
        assert_eq!(
            launches[0].binding.target,
            hit.disintegrations[0].hit.as_ref().unwrap().to
        );
    }
}
#[test]
fn disintegrate_caster_movement_death_life_or_later_action_cancels_unlaunched_body() {
    let t = Instant::now();
    let hit = frame("normal-hit");
    for which in 0..4 {
        let mut tracker = Tracker::new(&frame("normal-active"));
        tracker.update(&hit, t, timing).unwrap();
        let mut altered = hit.clone();
        // Body arbitration unit inputs; not claimed as independently valid wire frames.
        match which {
            0 => altered.world.actors[0].position.x += 1,
            1 => altered.world.actors[0].sequence += 1,
            2 => altered.vitals[0].hp = 0,
            _ => altered.vitals[0].attack_revision = hit.world.revision + 1,
        }
        assert!(tracker
            .update(&altered, t + Duration::from_millis(300), timing)
            .unwrap()
            .is_empty());
        assert!(tracker.active.is_empty());
    }
    let mut h = frame("honor-death");
    let mut tracker = Tracker::new(&frame("honor-active"));
    tracker.update(&h, t, timing).unwrap();
    h.honor.as_mut().unwrap().lives[0].respawn_count += 1;
    assert!(tracker
        .update(&h, t + Duration::from_millis(300), timing)
        .unwrap()
        .is_empty());
}

#[test]
fn disintegrate_hurt_replaces_body_without_losing_committed_effect() {
    for npc in [false, true] {
        let (initial, hit) = if npc {
            (
                super::npc_tests::reply("normal-before")
                    .frame
                    .unwrap()
                    .combat,
                super::npc_tests::reply("normal-hit").frame.unwrap().combat,
            )
        } else {
            (frame("normal-active"), frame("normal-hit"))
        };
        for coalesced in [false, true] {
            let t = Instant::now();
            let mut tracker = Tracker::new(&initial);
            let mut hurt = hit.clone();
            // Isolated pose arbitration, not a fabricated transport receipt.
            hurt.vitals[0].hurt_revision = hit.world.revision + 1;
            hurt.vitals[0].hp -= 1;
            let frozen = hurt.clone();
            tracker
                .update(if coalesced { &hurt } else { &hit }, t, timing)
                .unwrap();
            assert!(tracker
                .update(&hurt, t + Duration::from_millis(100), timing)
                .unwrap()
                .is_empty());
            assert!(!tracker.active[&1].body_visible, "hurt pose owns the body");
            let launches = tracker
                .update(&hurt, t + Duration::from_millis(300), timing)
                .unwrap();
            assert_eq!(launches.len(), 1, "npc={npc}, coalesced={coalesced}");
            assert_eq!(launches[0].started, t + Duration::from_millis(300));
            assert!(tracker
                .update(&hurt, t + Duration::from_millis(700), timing)
                .unwrap()
                .is_empty());
            assert_eq!(hurt, frozen, "effect scheduling cannot change authority");
        }
    }
}
#[test]
fn disintegrate_world_reset_clears_pending_and_pins_current_history() {
    let t = Instant::now();
    let hit = frame("normal-hit");
    for which in 0..4 {
        let mut tracker = Tracker::new(&frame("normal-active"));
        tracker.update(&hit, t, timing).unwrap();
        let mut next = hit.clone();
        match which {
            0 => next.world.match_id += 1,
            1 => next.world.generation += 1,
            2 => next.world.manifest.digest[0] ^= 1,
            _ => next.world.phase = Phase::Aborted,
        }
        assert!(tracker.resets(&next));
        assert!(tracker
            .update(&next, t + Duration::from_secs(1), timing)
            .unwrap()
            .is_empty());
        assert!(tracker.active.is_empty());
        assert!(tracker
            .update(&next, t + Duration::from_secs(2), timing)
            .unwrap()
            .is_empty());
    }
}

#[test]
fn disintegrate_pace_change_preserves_old_phase_and_launch_wall_deadline() {
    let hit = frame("normal-hit");
    let t = Instant::now();
    let mut tracker = Tracker::new(&frame("normal-active"));
    tracker.update(&hit, t, timing).unwrap();
    let mut faster = hit.clone();
    // Isolated pace arbitration input: server-authority qualification is tested
    // by timed_protocol; no synthetic potion receipt is passed through transport.
    let mut timed = crate::modern_lito::timed_protocol::State::initial(1);
    timed.revision = hit.world.revision;
    timed.haste_until_nanos = 1_000_000_000;
    faster.timed = vec![timed];
    tracker
        .update(&faster, t + Duration::from_millis(100), timing)
        .unwrap();
    assert_eq!(
        tracker.active[&1]
            .body
            .peek(t + Duration::from_millis(100))
            .unwrap(),
        Duration::from_millis(100)
    );
    let deadline = tracker.active[&1].launch_at.unwrap();
    assert!(deadline < t + Duration::from_millis(300));
    assert!(tracker
        .update(&faster, deadline - Duration::from_nanos(1), timing)
        .unwrap()
        .is_empty());
    // PacedClock carries fixed Q32 phase and conservatively rounds each
    // deadline upward; splitting the final nanosecond can add at most 1ns.
    // Do not demand an exact rational clock from the existing body scheduler.
    let tick = deadline + Duration::from_nanos(1);
    let launched = tracker.update(&faster, tick, timing).unwrap();
    assert_eq!(launched.len(), 1);
    assert!((deadline..=tick).contains(&launched[0].started));
    let started = launched[0].started;
    let mut normal = faster;
    normal.timed.clear();
    assert!(tracker
        .update(&normal, deadline + Duration::from_millis(100), timing)
        .unwrap()
        .is_empty());
    assert_eq!(
        launched[0].started, started,
        "effect clock is not retimed with the actor"
    );
}

#[test]
fn disintegrate_local_event_ids_are_unique_across_successive_receipts() {
    let t = Instant::now();
    let mut tracker = Tracker::new(&frame("normal-active"));
    let first = frame("normal-hit");
    tracker.update(&first, t, timing).unwrap();
    let a = tracker
        .update(&first, t + Duration::from_millis(300), timing)
        .unwrap();
    let second = frame("normal-second");
    tracker
        .update(&second, t + Duration::from_secs(5), timing)
        .unwrap();
    let b = tracker
        .update(&second, t + Duration::from_millis(5300), timing)
        .unwrap();
    assert_eq!((a[0].binding.event_id, b[0].binding.event_id), (1, 2));
    assert!(b[0].binding.start_revision > a[0].binding.start_revision);
}
