use super::*;

#[test]
fn receipt_age_is_inclusive_bounded_and_checked() {
    assert_eq!(age(1, 1), Some(Duration::ZERO));
    assert_eq!(age(1, 500_000_001), Some(Duration::from_millis(500)));
    assert_eq!(age(1, 500_000_002), None);
    assert_eq!(age(1, 0), None);
    assert_eq!(age(i64::MIN, i64::MAX), None);
}
#[test]
fn empty_initial_attachment_never_requires_or_invents_healing_assets() {
    let data = super::super::battle_payload::data_tests::fixture();
    let obs = super::super::battle_scene::tests::observation(data);
    assert!(!enabled(&obs));
    let at = Instant::now();
    let timeline = Timeline::new(obs.clone(), at);
    let bank = Assets { effect: None };
    let next = timeline
        .advance(obs.clone(), at, &bank, &AtomicBool::new(false))
        .unwrap();
    assert!(next.casts.is_empty() && next.effects.is_empty() && !next.skipped);
    assert!(timeline
        .advance(obs.clone(), at, &bank, &AtomicBool::new(true))
        .is_err());
    assert!(timeline
        .advance(
            obs,
            at - Duration::from_nanos(1),
            &bank,
            &AtomicBool::new(false)
        )
        .is_err());
}
pub(crate) fn count(t: &Timeline) -> usize {
    t.effects.len()
}
pub(crate) fn verify_player_edges(
    before: Observation,
    used: Observation,
    assets: &Assets,
    caster: i64,
) {
    let cancel = AtomicBool::new(false);
    let at = Instant::now() + Duration::from_secs(60);
    let t = Timeline::new(before, at);
    let accepted = t.advance(used.clone(), at, assets, &cancel).unwrap();
    let r = visuals(used.data(), caster)
        .unwrap()
        .into_iter()
        .find(|r| r.player)
        .unwrap();
    assert_eq!(accepted.effects.len(), 1);
    assert_eq!(accepted.effects[0].position, r.position);
    assert_eq!(
        accepted.effects[0].started,
        at - age(r.elapsed, used.data().world.loading_elapsed_nanos).unwrap()
    );
    assert!(t.effects.is_empty() && t.casts.is_empty());
    assert!(t
        .advance(used.clone(), at, assets, &AtomicBool::new(true))
        .is_err());
    let repeated = accepted.advance(used.clone(), at, assets, &cancel).unwrap();
    assert_eq!(repeated.effects.len(), 1);
    let historical = Timeline::new(used.clone(), at)
        .advance(used.clone(), at, assets, &cancel)
        .unwrap();
    assert!(historical.effects.is_empty() && historical.casts.is_empty());
    assert!(accepted
        .advance(
            used.clone(),
            at + Duration::from_millis(2500),
            assets,
            &cancel
        )
        .unwrap()
        .effects
        .is_empty());
    let mut expired_audio = accepted
        .advance(
            used.clone(),
            at + Duration::from_millis(501),
            assets,
            &cancel,
        )
        .unwrap();
    assert!(expired_audio
        .audio(assets, at + Duration::from_millis(501))
        .unwrap()
        .events
        .is_empty());
    let a = used
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == caster)
        .unwrap();
    let cast = accepted.casts.get(&caster).unwrap();
    for change in [
        (|a: &mut d::Actor| a.vital.as_mut().unwrap().hurt_revision = i64::MAX)
            as fn(&mut d::Actor),
        |a| a.vital.as_mut().unwrap().values.hp = 0,
        |a| a.last_move_sequence += 1,
        |a| a.resources.as_mut().unwrap().life += 1,
        |a| a.resources.as_mut().unwrap().equipment_revision += 1,
    ] {
        let mut changed = a.clone();
        change(&mut changed);
        assert!(!current(used.data(), &changed, cast).unwrap());
    }
}
pub(crate) fn body_elapsed(t: &Timeline, entity: i64, at: Instant) -> Duration {
    t.casts.get(&entity).unwrap().body.peek(at).unwrap()
}
pub(crate) fn verify_coalesced(
    base: Observation,
    recent: Observation,
    stale: Observation,
    assets: &Assets,
) {
    let at = Instant::now() + Duration::from_secs(60);
    let cancel = AtomicBool::new(false);
    let t = Timeline::new(base, at);
    let last = t.advance(recent, at, assets, &cancel).unwrap();
    assert!(last.skipped);
    assert_eq!(last.effects.len(), 1);
    let old = t.advance(stale, at, assets, &cancel).unwrap();
    assert!(old.skipped && old.effects.is_empty());
}
pub(crate) fn without_effects(t: &Timeline) -> Timeline {
    let mut result = t.clone();
    result.effects.clear();
    result
}
pub(crate) fn verify_edges(
    base: Observation,
    used: Observation,
    later: Observation,
    assets: &Assets,
) {
    let cancel = AtomicBool::new(false);
    let at = Instant::now() + Duration::from_secs(60);
    let first = Timeline::new(base.clone(), at);
    let mut accepted = first.advance(used.clone(), at, assets, &cancel).unwrap();
    assert_eq!(accepted.effects.len(), 1);
    assert_eq!(accepted.effects[0].started, at);
    let r = receipt(&used.data().actors.0[0]).unwrap();
    assert_eq!(accepted.effects[0].position, target(r).unwrap());
    assert_eq!(
        assets.effect.as_ref().unwrap().duration(),
        Duration::from_millis(2500)
    );
    let heard = accepted.audio(assets, at).unwrap();
    assert_eq!(heard.events.len(), 1);
    assert_eq!(heard.events[0].group.candidates(), [109]);
    assert!(heard.events[0].bank.as_ref().unwrap().get(109).is_some());
    let mut twice = accepted.advance(used.clone(), at, assets, &cancel).unwrap();
    assert_eq!(twice.effects.len(), 1);
    assert_eq!(
        twice.audio(assets, at).unwrap().events[0].sequence,
        heard.events[0].sequence
    );
    assert!(first
        .advance(used.clone(), at, assets, &AtomicBool::new(true))
        .is_err());
    assert!(first.effects.is_empty());
    assert!(accepted.advance(base, at, assets, &cancel).is_err());
    assert_eq!(accepted.effects.len(), 1);
    let historical = Timeline::new(used.clone(), at)
        .advance(used.clone(), at, assets, &cancel)
        .unwrap();
    assert!(historical.effects.is_empty() && historical.casts.is_empty());
    let mut stale_audio = accepted
        .advance(
            used.clone(),
            at + Duration::from_millis(501),
            assets,
            &cancel,
        )
        .unwrap();
    assert!(stale_audio
        .audio(assets, at + Duration::from_millis(501))
        .unwrap()
        .events
        .is_empty());
    assert_eq!(stale_audio.effects.len(), 1);
    assert!(stale_audio
        .advance(
            used.clone(),
            at + Duration::from_millis(2500),
            assets,
            &cancel
        )
        .unwrap()
        .effects
        .is_empty());
    let mut saturated = first.clone();
    saturated.effects = vec![accepted.effects[0].clone(); MAX_EFFECTS];
    let full = saturated
        .advance(used.clone(), at, assets, &cancel)
        .unwrap();
    assert!(full.skipped);
    assert_eq!(full.effects.len(), MAX_EFFECTS);
    let mut old = used.data().clone();
    let nanos = old.world.loading_elapsed_nanos;
    old.world.loading_elapsed_nanos += 500_000_000;
    old.world.active_elapsed_nanos += 500_000_000;
    // Only these presentation clocks advance; all authority facts stay exact.
    let old = super::super::battle_scene::tests::observation(old);
    let boundary = first.advance(old, at, assets, &cancel).unwrap();
    assert_eq!(boundary.effects[0].started, at - Duration::from_millis(500));
    let mut too_old = used.data().clone();
    too_old.world.loading_elapsed_nanos = nanos + 500_000_001;
    too_old.world.active_elapsed_nanos += 500_000_001;
    let too_old = super::super::battle_scene::tests::observation(too_old);
    let suppressed = first.advance(too_old, at, assets, &cancel).unwrap();
    assert!(suppressed.effects.is_empty() && suppressed.casts.is_empty() && suppressed.skipped);
    let mut mixed = accepted.clone();
    let actor = &used.data().actors.0[0];
    let cast = mixed.casts.get(&actor.entity).unwrap().clone();
    assert!(current(used.data(), actor, &cast).unwrap());
    for axis in 0..5 {
        let mut a = actor.clone();
        match axis {
            0 => a.vital.as_mut().unwrap().hurt_revision = r.impact.usage.revision + 1,
            1 => a.vital.as_mut().unwrap().values.hp = 0,
            2 => a.last_move_sequence += 1,
            3 => a.resources.as_mut().unwrap().life += 1,
            _ => a.resources.as_mut().unwrap().equipment_revision += 1,
        }
        assert!(!current(used.data(), &a, &cast).unwrap());
    }
    mixed = mixed
        .advance(later, at + Duration::from_millis(700), assets, &cancel)
        .unwrap();
    assert_eq!(
        mixed.effects.len(),
        1,
        "caster activity cannot erase committed fixed effect"
    );
    assert!(
        mixed.casts.is_empty(),
        "MOVE or hurt takes the body without undoing healing"
    );
}
