use super::*;

#[test]
fn supported_lanes_do_not_confuse_target_domains_or_other_spells() {
    assert!(kind(d::Lane::FireStorm) == Some(Kind::Fire));
    assert!(kind(d::Lane::NpcFireStorm) == Some(Kind::Fire));
    assert!(kind(d::Lane::Disintegrate) == Some(Kind::Disintegrate));
    assert!(kind(d::Lane::NpcDisintegrate) == Some(Kind::Disintegrate));
    assert!(kind(d::Lane::LightArrow) == Some(Kind::Arrow));
    assert!(kind(d::Lane::NpcLightArrow) == Some(Kind::Arrow));
    for lane in [d::Lane::Teleport, d::Lane::Healing] {
        assert!(kind(lane).is_none());
    }
}
#[test]
fn receipt_age_rejects_stale_future_and_overflow_without_restarting_history() {
    assert!(recent(10, 10));
    assert!(recent(10, 500_000_010));
    assert!(!recent(10, 500_000_011));
    assert!(!recent(10, 9));
    assert!(!recent(i64::MIN, i64::MAX));
}
#[test]
fn source_cue_time_uses_ceiling_not_wakeup_time_and_checks_overflow() {
    let at = Instant::now();
    assert_eq!(cue_created(at, 0).unwrap(), at);
    assert_eq!(
        cue_created(at, 1).unwrap(),
        at + Duration::from_nanos(41_666_667)
    );
    assert_eq!(cue_created(at, 24).unwrap(), at + Duration::from_secs(1));
    assert!(cue_created(at, u64::MAX).is_err());
}

pub(crate) fn launch_deadline(timeline: &Timeline) -> Instant {
    timeline.casts.values().next().unwrap().launch_at
}

/// Positive inputs come from the actual Java Engine, never locally fabricated
/// legacy frames. Synthetic mutations below test only the pure body predicate.
pub(crate) fn verify(
    before: Observation,
    cast: Observation,
    later: Observation,
    assets: &Assets,
    scene: &SceneSnapshot,
) {
    let cancel = AtomicBool::new(false);
    let t = Instant::now();
    let at = t + Duration::from_secs(1);
    let initial = Timeline::new(before.clone(), t);
    let pending = initial.advance(cast.clone(), at, assets, &cancel).unwrap();
    assert_eq!(pending.casts.len(), 1);
    assert!(pending.effects.is_empty());
    let c = pending.casts.values().next().unwrap();
    let launch = c.launch_at;
    assert!(launch > at);
    assert_eq!(pending.overrides(scene, assets, at).unwrap().len(), 1);
    let receipt = c.receipt.clone();
    let body = c.clone();
    let data = cast.data();
    let actor = &data.actors.0[0];
    let member = &data.room.members.0[0];
    assert!(current(data, actor, member, c));
    let mut hurt = actor.clone();
    hurt.vital.as_mut().unwrap().hurt_revision = receipt.impact.usage.revision + 1;
    assert!(
        current(data, &hurt, member, c),
        "hurt cannot cancel pending effect ownership"
    );
    let mut moved = actor.clone();
    moved.last_move_sequence += 1;
    assert!(!current(data, &moved, member, c));
    let mut dead = actor.clone();
    dead.vital.as_mut().unwrap().values.hp = 0;
    assert!(!current(data, &dead, member, c));
    let mut equipment = actor.clone();
    equipment.resources.as_mut().unwrap().equipment_revision += 1;
    assert!(!current(data, &equipment, member, c));
    let mut life = actor.clone();
    life.resources.as_mut().unwrap().life += 1;
    assert!(!current(data, &life, member, c));

    let duplicate = pending.advance(cast.clone(), at, assets, &cancel).unwrap();
    assert_eq!(duplicate.casts.values().next().unwrap().launch_at, launch);
    assert_eq!(duplicate.next_event, 0);
    assert!(pending
        .advance(cast.clone(), at - Duration::from_nanos(1), assets, &cancel)
        .is_err());
    assert!(pending
        .advance(cast.clone(), launch, assets, &AtomicBool::new(true))
        .is_err());
    assert_eq!(
        pending.next_event, 0,
        "failed advance leaves original pending owner intact"
    );

    let mut launched = pending
        .advance(cast.clone(), launch, assets, &cancel)
        .unwrap();
    assert_eq!(
        launched.effects.len(),
        1,
        "one event even for empty/multiple victims"
    );
    assert_eq!(launched.next_event, 1);
    let audio = launched.audio(assets, launch).unwrap();
    assert_eq!(audio.events.len(), 1);
    let sound = if body.kind == Kind::Fire { 710 } else { 717 };
    assert!(audio.events[0].group.candidates().contains(&sound));
    assert_eq!(audio.events[0].created, launch);
    assert!(audio.events[0].bank.as_ref().unwrap().get(sound).is_some());
    let mut twice = launched
        .advance(cast.clone(), launch, assets, &cancel)
        .unwrap();
    let repeated = twice.audio(assets, launch).unwrap();
    assert_eq!(audio.events[0].sequence, repeated.events[0].sequence);
    assert_eq!(twice.next_event, 1);
    assert_eq!(twice.effects.len(), 1);
    // Exercise the bounded queue with already-owned, already-consumed effects;
    // the next launch is reported as skipped rather than growing unboundedly.
    let mut saturated = pending.clone();
    saturated.effects = vec![launched.effects[0].clone(); MAX_EFFECTS];
    let saturated = saturated
        .advance(cast.clone(), launch, assets, &cancel)
        .unwrap();
    assert_eq!(saturated.effects.len(), MAX_EFFECTS);
    assert!(saturated.skipped);
    assert_eq!(saturated.next_event, 0);
    let retained = twice
        .advance(
            later.clone(),
            launch + Duration::from_millis(100),
            assets,
            &cancel,
        )
        .unwrap();
    assert_eq!(
        retained.effects.len(),
        1,
        "target respawn/retained receipt cannot erase an owned effect"
    );
    match &retained.effects[0].playback {
        Playback::Arrow { .. } => panic!("arrow has a separate native-launch test"),
        Playback::Fire { origin, .. } => {
            assert_eq!(*origin, cell(&receipt.impact.usage.before.cell));
            let projection = ViewProjection {
                anchor_cell: *origin,
                anchor_pixel: super::super::world_map::PixelPoint { x: 400, y: 300 },
            };
            let mut actual = Canvas::new(800, 600).unwrap();
            actual.clear([20, 30, 40]);
            let mut expected = Canvas::new(800, 600).unwrap();
            expected.clear([20, 30, 40]);
            retained
                .draw_projected(
                    &mut actual,
                    projection,
                    assets,
                    launch + Duration::from_millis(100),
                    &cancel,
                )
                .unwrap();
            assets
                .fire
                .as_ref()
                .unwrap()
                .draw_effect(
                    &mut expected,
                    projection.anchor_pixel,
                    Duration::from_millis(100),
                )
                .unwrap();
            assert_eq!(
                actual.rgba(),
                expected.rgba(),
                "exact four-copy receipt-origin composition"
            );
        }
        Playback::Disintegrate(playback) => {
            assert_eq!(
                playback.binding().target,
                cell(&receipt.impact.targets.0[0].cell)
            );
            assert_eq!(
                playback.binding().start_revision,
                receipt.impact.usage.revision as u64
            );
            assert_eq!(playback.binding().map_digest, data.manifest.digest);
            let d::Detail::Disintegrate { critical, .. } = receipt.detail else {
                panic!()
            };
            assert_eq!(
                playback.variant(),
                if critical {
                    Variant::Critical
                } else {
                    Variant::Normal
                }
            );
            assert_eq!(
                retained
                    .disintegrations(launch + Duration::from_millis(100))
                    .unwrap()
                    .len(),
                1
            );
        }
    }
    let mut expired_audio = retained
        .advance(
            later.clone(),
            launch + Duration::from_millis(501),
            assets,
            &cancel,
        )
        .unwrap();
    assert!(expired_audio
        .audio(assets, launch + Duration::from_millis(501))
        .unwrap()
        .events
        .is_empty());
    let historical = Timeline::new(cast.clone(), at)
        .advance(cast.clone(), launch, assets, &cancel)
        .unwrap();
    assert!(historical.casts.is_empty() && historical.effects.is_empty());
    let done = at + Duration::from_secs(20);
    let mut stalled = pending
        .advance(cast.clone(), done, assets, &cancel)
        .unwrap();
    assert!(stalled.effects.is_empty() && stalled.casts.is_empty());
    assert_eq!(
        stalled.next_event, 0,
        "a stalled wakeup must not restart expired effects"
    );
    assert!(stalled.audio(assets, done).unwrap().events.is_empty());
    let mut silent_late = pending
        .advance(
            cast.clone(),
            launch + Duration::from_millis(501),
            assets,
            &cancel,
        )
        .unwrap();
    assert_eq!(silent_late.effects.len(), 1);
    assert!(silent_late
        .audio(assets, launch + Duration::from_millis(501))
        .unwrap()
        .events
        .is_empty());
    let finished = retained
        .advance(later.clone(), done, assets, &cancel)
        .unwrap();
    assert!(finished.effects.is_empty());
    if later.data().world.loading_elapsed_nanos - receipt.loading_elapsed_nanos > 500_000_000 {
        let skipped = initial.advance(later, at, assets, &cancel).unwrap();
        assert!(
            skipped.casts.is_empty() && skipped.effects.is_empty(),
            "old coalesced receipt is not fresh"
        );
    }
    assert_eq!(pending.observation.data(), cast.data());
}
