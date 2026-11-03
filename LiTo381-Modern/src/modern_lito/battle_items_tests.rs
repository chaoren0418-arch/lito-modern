use super::*;

#[test]
fn receipt_age_has_an_exact_nonnegative_bounded_window() {
    assert_eq!(age(10, 10), Some(Duration::ZERO));
    assert_eq!(age(10, 500_000_010), Some(MAX_AGE));
    assert_eq!(age(10, 500_000_011), None);
    assert_eq!(age(10, 9), None);
    assert_eq!(age(i64::MIN, i64::MAX), None);
}
#[test]
fn legacy_empty_baseline_has_no_assets_effects_or_replay() {
    let observation = crate::modern_lito::battle_scene::tests::observation(
        crate::modern_lito::battle_payload::data_tests::fixture(),
    );
    let now = Instant::now();
    let timeline = Timeline::new(observation.clone(), now);
    let assets = Assets {
        healing: None,
        timed: None,
    };
    let mut next = timeline
        .advance(observation.clone(), now, &assets, &AtomicBool::new(false))
        .unwrap();
    assert!(next.effects.is_empty());
    assert!(!next.skipped);
    assert!(next.audio(&assets, now).is_none());
    assert!(timeline
        .advance(observation.clone(), now, &assets, &AtomicBool::new(true))
        .is_err());
    assert!(timeline
        .advance(
            observation,
            now - Duration::from_nanos(1),
            &assets,
            &AtomicBool::new(false)
        )
        .is_err());
}
pub(crate) fn without_effects(timeline: &Timeline) -> Timeline {
    let mut result = timeline.clone();
    result.effects.clear();
    result
}
pub(crate) fn count(timeline: &Timeline) -> usize {
    timeline.effects.len()
}

pub(crate) fn verify_coalesced(before: Observation, last: Observation, assets: &Assets) {
    let at = Instant::now() + Duration::from_secs(60);
    let base = Timeline::new(before, at);
    let mut delayed = base
        .advance(last.clone(), at, assets, &AtomicBool::new(false))
        .unwrap();
    assert!(
        delayed.skipped,
        "coalesced/stale uses must be reported, not invented"
    );
    assert_eq!(
        delayed.effects.len(),
        1,
        "old actor1 event is stale; only current actor2 event survives"
    );
    assert_eq!(delayed.effects[0].receipt.effect, 751);
    assert_eq!(delayed.effects[0].started, at - Duration::from_millis(100));
    let audio = delayed.audio(assets, at).unwrap();
    assert_eq!(audio.events.len(), 1);
    assert_eq!(audio.events[0].group.candidates(), &[121]);
    assert_eq!(audio.events[0].created, at - Duration::from_millis(100));
    assert_eq!(delayed.observation.data(), last.data());
}

/// Positive receipts are genuine Java engine commits, including non-owner views.
pub(crate) fn verify(before: Observation, used: Observation, moved: Observation, assets: &Assets) {
    let now = Instant::now() + Duration::from_secs(60);
    let cancel = AtomicBool::new(false);
    let base = Timeline::new(before.clone(), now - Duration::from_secs(1));
    let first = base.advance(used.clone(), now, assets, &cancel).unwrap();
    assert_eq!(first.effects.len(), 1);
    assert!(!first.skipped);
    let event = first.effects[0].clone();
    assert_eq!(event.started, now);
    let duplicate = first.advance(used.clone(), now, assets, &cancel).unwrap();
    assert_eq!(duplicate.effects.len(), 1);
    assert_eq!(duplicate.effects[0].started, now);
    let historical = Timeline::new(used.clone(), now)
        .advance(used.clone(), now, assets, &cancel)
        .unwrap();
    assert!(historical.effects.is_empty());
    assert!(first
        .advance(
            moved.clone(),
            now + Duration::from_millis(100),
            assets,
            &AtomicBool::new(true)
        )
        .is_err());
    assert_eq!(first.observation.data(), used.data());
    let after = first
        .advance(
            moved.clone(),
            now + Duration::from_millis(100),
            assets,
            &cancel,
        )
        .unwrap();
    assert_eq!(after.effects[0].receipt.cell, event.receipt.cell);
    assert_eq!(after.effects[0].started, now);
    let delayed = base
        .advance(moved, now + Duration::from_millis(100), assets, &cancel)
        .unwrap();
    assert_eq!(
        delayed.effects[0].started, now,
        "late receipt does not restart the overlay"
    );
    let expired = after
        .advance(
            after.observation.clone(),
            now + event.duration,
            assets,
            &cancel,
        )
        .unwrap();
    assert!(expired.effects.is_empty());
    let mut saturated = base.clone();
    saturated.effects = vec![event; MAX_EFFECTS];
    let overflow = saturated
        .advance(used.clone(), now, assets, &cancel)
        .unwrap();
    assert_eq!(overflow.effects.len(), MAX_EFFECTS);
    assert!(overflow.skipped);
    assert_eq!(
        overflow.observation.data(),
        used.data(),
        "visual capacity never rejects accepted authority"
    );
}
