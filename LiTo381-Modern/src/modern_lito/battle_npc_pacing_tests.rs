use super::*;
use crate::modern_lito::battle_payload::npc_motion as m;

#[test]
#[ignore = "all ten shared NPC banks, paced walk/attack source and cue boundaries; no GUI or sound device"]
fn real_shared_npc_paced_cycles_and_layers() {
    let root = std::path::PathBuf::from(
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
    );
    assert!(root.is_absolute());
    let cancel = AtomicBool::new(false);
    let context =
        crate::modern_lito::shared_action_catalog::SharedActionCatalog::load(&root, &cancel)
            .unwrap();
    let t = Instant::now() + Duration::from_secs(1);
    let mut count = 0;
    for form in NpcAppearance::ALL {
        let bank = NpcAssets::load_with_catalog(&context, &[form], &cancel).unwrap();
        for heading in 0..8 {
            for millis in [100, 333, 750] {
                let (before, mut after, old, mut next) = actors(form.graphics_id(), millis);
                let (dx, dy) = crate::modern_lito::world_map::DIRECTIONS[heading];
                after.heading = heading as i32;
                after.cell = d::Cell {
                    x: before.cell.x + dx,
                    y: before.cell.y + dy,
                };
                next.last_walk.as_mut().unwrap().to = after.cell.clone();
                next.last_walk.as_mut().unwrap().heading = heading as i32;
                next.attack_millis = millis;
                for action in [NpcAction::Walk, NpcAction::Attack] {
                    let mut body = NpcBody::initial(&before, t);
                    body.change(
                        &before,
                        &after,
                        action,
                        Some(((&old).into(), (&next).into(), 100)),
                        t,
                    )
                    .unwrap();
                    let q = bank.action(form, action).unwrap();
                    let mut prefix = 0u128;
                    for source in (1..q.source_frames().len()).chain(std::iter::once(0)) {
                        let at =
                            (prefix * millis as u128 * 1_000_000).div_ceil(q.duration().as_nanos());
                        let now = t + Duration::from_nanos(at as u64);
                        let s = body.sample(&after, &bank, now).unwrap();
                        assert_eq!((s.action, s.source), (action, source));
                        assert_eq!(
                            s.cue_time(Duration::from_nanos(prefix as u64)).unwrap(),
                            now
                        );
                        assert_eq!(
                            s.walk_from.is_some(),
                            action == NpcAction::Walk && source != 0
                        );
                        let frame = bank.frame(form, s.action, heading as u8, s.source).unwrap();
                        assert_eq!(
                            (frame.action, frame.heading, frame.source),
                            (action, heading as u8, source)
                        );
                        assert!(
                            frame.body.frame.fragments.is_some()
                                && frame.shadow.frame.fragments.is_some()
                        );
                        count += 1;
                        prefix += u128::from(q.source_frames()[source].duration_ticks * 1000 / 24)
                            * 1_000_000;
                    }
                    let s = body
                        .sample(&after, &bank, t + Duration::from_millis(millis as u64))
                        .unwrap();
                    assert_eq!(
                        (s.action, s.source, s.walk_from),
                        (NpcAction::Breath, 1, None)
                    );
                }
            }
        }
    }
    assert_eq!(count, (88 + 69) * 8 * 3);
    eprintln!("ten shared NPC forms, eight headings, three authority durations: {count} walk/attack source and inverse cue boundaries, body/shadow bindings; no fabricated movement or native HWND claim");
}

fn actors(graphic: u32, millis: i32) -> (d::Npc, d::Npc, m::Actor, m::Actor) {
    let before = d::Npc {
        serial: 1,
        source: 7,
        npc_id: 100,
        graphic: graphic as i32,
        lane: 0,
        stage: 1,
        heading: 2,
        action: d::NpcAction::Idle,
        cell: d::Cell { x: 10, y: 10 },
        hp: 100,
        maximum_hp: 100,
        mp: 20,
        maximum_mp: 20,
        life: 1,
        born_revision: 1,
        changed_revision: 2,
        corpse_until_millis: 0,
        next_move_millis: 100,
        next_attack_millis: 750,
    };
    let old = m::Actor {
        serial: 1,
        life: 1,
        walk_millis: millis,
        attack_millis: 750,
        last_walk: None,
    };
    let mut after = before.clone();
    after.cell.x += 1;
    after.action = d::NpcAction::Walk;
    after.changed_revision = 3;
    after.next_move_millis = 100 + i64::from(millis);
    let mut next = old.clone();
    next.last_walk = Some(m::Walk {
        sequence: 1,
        life: 1,
        roster_revision: 3,
        elapsed_millis: 100,
        duration_millis: millis,
        from: before.cell.clone(),
        to: after.cell.clone(),
        heading: 2,
    });
    (before, after, old, next)
}

#[test]
fn every_npc_form_paces_one_complete_walk_and_keeps_authoritative_destination() {
    let t = Instant::now();
    for form in NpcAppearance::ALL {
        let bank = crate::modern_lito::npc_assets::tests::presentation_fixture(form);
        for millis in [1, 100, 333, 750, 60_000] {
            let (before, after, old, next) = actors(form.graphics_id(), millis);
            let mut body = NpcBody::initial(&before, t);
            body.change(
                &before,
                &after,
                NpcAction::Walk,
                Some(((&old).into(), (&next).into(), 100)),
                t,
            )
            .unwrap();
            let q = bank.action(form, NpcAction::Walk).unwrap();
            let step = millis as u128 * 1_000_000;
            let source_nanos = q.duration().as_nanos();
            let mut prefix = 0u128;
            for source in (1..q.source_frames().len()).chain(std::iter::once(0)) {
                let at = (prefix * step).div_ceil(source_nanos);
                let s = body
                    .sample(&after, &bank, t + Duration::from_nanos(at as u64))
                    .unwrap();
                assert_eq!((s.action, s.source), (NpcAction::Walk, source));
                assert_eq!(s.walk_from.is_some(), source != 0);
                assert_eq!(
                    s.cue_time(Duration::from_nanos(prefix as u64)).unwrap(),
                    t + Duration::from_nanos(at as u64)
                );
                if at > 0 {
                    let earlier = body
                        .sample(&after, &bank, t + Duration::from_nanos((at - 1) as u64))
                        .unwrap();
                    assert_ne!(earlier.source, source);
                }
                prefix +=
                    u128::from(q.source_frames()[source].duration_ticks * 1000 / 24) * 1_000_000;
            }
            let done = body
                .sample(&after, &bank, t + Duration::from_millis(millis as u64))
                .unwrap();
            assert_eq!(
                (done.action, done.source, done.walk_from),
                (NpcAction::Breath, 1, None)
            );
            assert_eq!(after.cell, next.last_walk.as_ref().unwrap().to);
        }
    }
}

#[test]
fn npc_attack_cues_use_the_same_authority_cycle_as_pixels() {
    let t = Instant::now();
    for form in NpcAppearance::ALL {
        let bank = crate::modern_lito::npc_assets::tests::presentation_fixture(form);
        let (before, after, old, mut next) = actors(form.graphics_id(), 100);
        for millis in [1, 73, 750, 60_000] {
            next.attack_millis = millis;
            let mut body = NpcBody::initial(&before, t);
            body.change(
                &before,
                &after,
                NpcAction::Attack,
                Some(((&old).into(), (&next).into(), 100)),
                t,
            )
            .unwrap();
            let q = bank.action(form, NpcAction::Attack).unwrap();
            let mut prefix = 0u128;
            for source in (1..q.source_frames().len()).chain(std::iter::once(0)) {
                let at = (prefix * millis as u128 * 1_000_000).div_ceil(q.duration().as_nanos());
                let s = body
                    .sample(&after, &bank, t + Duration::from_nanos(at as u64))
                    .unwrap();
                assert_eq!((s.action, s.source), (NpcAction::Attack, source));
                assert_eq!(
                    s.cue_time(Duration::from_nanos(prefix as u64)).unwrap(),
                    t + Duration::from_nanos(at as u64)
                );
                assert!(s.walk_from.is_none());
                prefix +=
                    u128::from(q.source_frames()[source].duration_ticks * 1000 / 24) * 1_000_000;
            }
            assert_eq!(
                body.sample(&after, &bank, t + Duration::from_millis(millis as u64))
                    .unwrap()
                    .action,
                NpcAction::Breath
            );
        }
    }
}

#[test]
fn npc_walk_age_skips_expired_discontinuous_or_old_life_events_without_replay() {
    let t = Instant::now() + Duration::from_secs(1);
    let form = NpcAppearance::new(1578).unwrap();
    let bank = crate::modern_lito::npc_assets::tests::presentation_fixture(form);
    let (before, after, old, next) = actors(1578, 100);
    for age in [0, 1, 49, 99, 100, 101, 10_000] {
        let mut body = NpcBody::initial(&before, t);
        body.change(
            &before,
            &after,
            NpcAction::Walk,
            Some(((&old).into(), (&next).into(), 100 + age)),
            t,
        )
        .unwrap();
        let s = body.sample(&after, &bank, t).unwrap();
        if age < 100 {
            assert_eq!(s.action, NpcAction::Walk);
            assert_eq!(s.started, t - Duration::from_millis(age as u64));
        } else {
            assert_eq!(s.action, NpcAction::Breath);
            assert!(s.walk_from.is_none() && s.stamp.revision.is_none());
        }
    }
    for variant in 0..3 {
        let mut next = next.clone();
        match variant {
            0 => next.last_walk.as_mut().unwrap().sequence = 0,
            1 => next.last_walk.as_mut().unwrap().from.x -= 1,
            _ => next.last_walk.as_mut().unwrap().life += 1,
        }
        let mut body = NpcBody::initial(&before, t);
        body.change(
            &before,
            &after,
            NpcAction::Walk,
            Some(((&old).into(), (&next).into(), 100)),
            t,
        )
        .unwrap();
        assert_eq!(
            body.sample(&after, &bank, t).unwrap().action,
            NpcAction::Breath
        );
        assert!(body.walk_from.is_none());
    }
}

#[test]
fn coalesced_npc_updates_animate_only_the_latest_confirmed_step() {
    let t = Instant::now() + Duration::from_secs(1);
    let form = NpcAppearance::new(1578).unwrap();
    let bank = crate::modern_lito::npc_assets::tests::presentation_fixture(form);
    let (before, mut after, old, mut next) = actors(1578, 100);
    // A 200ms reader receives the second confirmed 100ms step. Its starting
    // cell is in the accepted world's validated tail, not an invented route
    // from the previous rendered cell. Never stretch/replay the missed step.
    let w = next.last_walk.as_mut().unwrap();
    w.sequence = 2;
    w.elapsed_millis = 200;
    w.from.x += 1;
    w.to.x += 1;
    after.cell = w.to.clone();
    after.next_move_millis = 300;
    let origin = super::super::world_map::WorldCell {
        x: w.from.x,
        y: w.from.y,
    };
    for age in [0, 25, 75, 99] {
        let mut body = NpcBody::initial(&before, t);
        body.change(
            &before,
            &after,
            NpcAction::Walk,
            Some(((&old).into(), (&next).into(), 200 + age)),
            t,
        )
        .unwrap();
        let s = body.sample(&after, &bank, t).unwrap();
        assert_eq!(
            s.action,
            NpcAction::Walk,
            "confirmed current step must survive a coalesced read"
        );
        assert_eq!(s.started, t - Duration::from_millis(age as u64));
        assert_eq!(s.walk_from, (s.source != 0).then_some(origin));
        assert_eq!(
            body.sample(&after, &bank, t + Duration::from_millis((100 - age) as u64))
                .unwrap()
                .action,
            NpcAction::Breath
        );
        assert_eq!(after.cell.x, before.cell.x + 2);
    }
}

#[test]
fn npc_first_attachment_new_life_death_and_body_replacement_clear_walk_origin() {
    let t = Instant::now();
    let form = NpcAppearance::new(1578).unwrap();
    let bank = crate::modern_lito::npc_assets::tests::presentation_fixture(form);
    let (before, mut after, old, next) = actors(1578, 100);
    assert_eq!(
        NpcBody::initial(&after, t)
            .sample(&after, &bank, t)
            .unwrap()
            .action,
        NpcAction::Breath
    );
    let mut body = NpcBody::initial(&before, t);
    for action in [
        NpcAction::Attack,
        NpcAction::Damage,
        NpcAction::Death,
        NpcAction::Breath,
    ] {
        body.change(
            &before,
            &after,
            NpcAction::Walk,
            Some(((&old).into(), (&next).into(), 100)),
            t,
        )
        .unwrap();
        assert!(body.walk_from.is_some());
        body.change(
            &before,
            &after,
            action,
            Some(((&old).into(), (&next).into(), 100)),
            t,
        )
        .unwrap();
        assert!(body.walk_from.is_none());
        assert_eq!(body.duration.is_some(), action == NpcAction::Attack);
    }
    after.life += 1;
    assert!(NpcBody::initial(&after, t).walk_from.is_none());
    after.hp = 0;
    let dead = NpcBody::initial(&after, t)
        .sample(&after, &bank, t)
        .unwrap();
    assert_eq!(
        (dead.action, dead.source, dead.walk_from),
        (NpcAction::Death, 0, None)
    );
    assert!(dead.stamp.historical_death);
}

#[test]
fn legacy_npc_cycles_and_timestamp_bounds_remain_explicit() {
    let t = Instant::now();
    let form = NpcAppearance::new(1578).unwrap();
    let bank = crate::modern_lito::npc_assets::tests::presentation_fixture(form);
    let (before, after, _, _) = actors(1578, 100);
    let mut body = NpcBody::initial(&before, t);
    body.change(&before, &after, NpcAction::Walk, None, t)
        .unwrap();
    let q = bank.action(form, NpcAction::Walk).unwrap();
    let s = body
        .sample(&after, &bank, t + Duration::from_millis(100))
        .unwrap();
    assert_eq!(s.action, NpcAction::Walk);
    assert!(s.walk_from.is_none());
    assert_eq!(
        s.source,
        q.sample(Duration::from_millis(100), after.heading as u8)
            .unwrap()
            .0
    );
    assert!(body
        .sample(&after, &bank, t - Duration::from_nanos(1))
        .is_err());
    assert!(scaled_npc_time(
        Duration::ZERO,
        Duration::ZERO,
        Duration::from_millis(1),
        false
    )
    .is_err());
    assert!(scaled_npc_time(Duration::MAX, Duration::from_nanos(1), Duration::MAX, false).is_err());
}
pub(crate) fn verify_haste_frames(frames: &std::collections::BTreeMap<String, d::UnvalidatedData>) {
    let obs = crate::modern_lito::battle_scene::tests::observation;
    let normal = &frames["npc-normal-1"];
    let hasted = &frames["npc-haste-1"];
    let walked = &frames["npc-walk-1"];
    let expired = &frames["npc-expired-1"];
    let resumed = &frames["npc-resumed-1"];
    let npc = |d: &d::UnvalidatedData| d.npcs.as_ref().unwrap().actors.0[0].clone();
    let old = npc(normal);
    let current = npc(hasted);
    let before = npc_pace(normal, &old).unwrap().unwrap().0;
    let haste = npc_pace(hasted, &current).unwrap().unwrap().0;
    assert_eq!(
        haste.attack_millis,
        ((before.attack_millis as f64 * 0.755) as i32).max(1)
    );
    assert_eq!(
        haste.last_walk, before.last_walk,
        "grant never rewrites committed walk"
    );
    let walk = npc_pace(walked, &npc(walked)).unwrap().unwrap().0;
    let after = npc_pace(expired, &npc(expired)).unwrap().unwrap().0;
    assert_eq!(after.attack_millis, before.attack_millis);
    assert_eq!(
        walk.last_walk, after.last_walk,
        "expiry never rewrites committed walk"
    );
    let base = normal
        .haste
        .as_ref()
        .unwrap()
        .state
        .npcs
        .as_ref()
        .unwrap()
        .actors
        .0[0]
        .walk_millis;
    assert_eq!(
        walk.last_walk.unwrap().duration_millis,
        ((base as f64 * 0.755) as i32).max(1)
    );
    assert_eq!(
        npc_pace(resumed, &npc(resumed))
            .unwrap()
            .unwrap()
            .0
            .last_walk
            .unwrap()
            .duration_millis,
        base
    );
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let cancel = AtomicBool::new(false);
    let catalog =
        crate::modern_lito::shared_action_catalog::SharedActionCatalog::load(&root, &cancel)
            .unwrap();
    let bank = NpcAssets::load_with_catalog(
        &catalog,
        &[NpcAppearance::new(old.graphic as u32).unwrap()],
        &cancel,
    )
    .unwrap();
    let now = Instant::now() + Duration::from_secs(2);
    let t = Timeline::new(obs(normal.clone()), now).unwrap();
    let t = t
        .advance(
            obs(hasted.clone()),
            now + Duration::from_millis(50),
            &cancel,
        )
        .unwrap();
    let t = t
        .advance(
            obs(walked.clone()),
            now + Duration::from_millis(100),
            &cancel,
        )
        .unwrap();
    let sample = t
        .npc_sample(&npc(walked), &bank, now + Duration::from_millis(100))
        .unwrap();
    assert_eq!(sample.action, NpcAction::Walk);
    assert_eq!(
        sample.real_duration,
        Duration::from_millis(walk.last_walk.unwrap().duration_millis as u64)
    );
    let t = t
        .advance(
            obs(expired.clone()),
            now + Duration::from_secs(1202),
            &cancel,
        )
        .unwrap();
    let t = t
        .advance(
            obs(resumed.clone()),
            now + Duration::from_millis(1_202_100),
            &cancel,
        )
        .unwrap();
    assert_eq!(
        t.npc_sample(&npc(resumed), &bank, now + Duration::from_millis(1_202_100))
            .unwrap()
            .real_duration,
        Duration::from_millis(base as u64)
    );
}
