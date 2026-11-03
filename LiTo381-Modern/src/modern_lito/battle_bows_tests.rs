use super::*;
fn shot(data: &d::UnvalidatedData) -> d::PublicShot {
    d::PublicShot {
        count: 1,
        revision: 3,
        attack_sequence: 1,
        loading_elapsed_nanos: 0,
        life: 1,
        equipment_revision: 0,
        from: d::Cell { x: 32778, y: 32778 },
        target: d::Target::Participant {
            entity: 2,
            identity: data.room.members.0[1].identity.clone(),
        },
        target_life: 1,
        to: d::Cell { x: 32783, y: 32778 },
        sprite: 66,
        hit: true,
    }
}
#[test]
fn hit_follows_exact_source_life_only_at_launch_miss_always_keeps_receipt() {
    let data = super::super::battle_payload::data_tests::fixture();
    let mut s = shot(&data);
    assert_eq!(target(&data, &s).unwrap(), cell(&data.actors.0[1].position));
    s.target_life = 2;
    assert_eq!(target(&data, &s).unwrap(), cell(&s.to));
    s.target = d::Target::Participant {
        entity: 2,
        identity: data.room.members.0[0].identity.clone(),
    };
    assert!(target(&data, &s).is_err());
    s.hit = false;
    assert_eq!(target(&data, &s).unwrap(), cell(&s.to));
    s.hit = true;
    s.target = d::Target::Honor { entity: 2, team: 0 };
    assert!(target(&data, &s).is_err());
}
#[test]
fn same_or_adjacent_cell_ends_immediately_and_age_never_wraps() {
    assert!(recent(10, 500_000_010));
    assert!(!recent(10, 500_000_011));
    assert!(!recent(10, 9));
    assert!(!recent(i64::MIN, i64::MAX));
    let from = WorldCell { x: 32778, y: 32778 };
    assert!(
        RangedPath::new(from, from)
            .unwrap()
            .sample(Duration::ZERO)
            .finished
    );
    assert!(
        RangedPath::new(
            from,
            WorldCell {
                x: from.x + 1,
                y: from.y
            }
        )
        .unwrap()
        .sample(Duration::ZERO)
        .finished
    );
}
#[test]
fn empty_legacy_timeline_never_replays_and_requires_exact_motion_owner() {
    let obs = super::super::battle_scene::tests::observation(
        super::super::battle_payload::data_tests::fixture(),
    );
    let now = Instant::now();
    let base = Timeline::new(obs.clone(), now);
    let motion = battle_motion::Timeline::new(obs.clone(), now).unwrap();
    let assets = Assets {
        arrow: None,
        delays: BTreeMap::new(),
    };
    let out = base
        .advance(obs.clone(), now, &assets, &motion, &AtomicBool::new(false))
        .unwrap();
    assert!(out.pending.is_empty() && out.flights.is_empty() && !out.skipped);
    assert!(base
        .advance(obs.clone(), now, &assets, &motion, &AtomicBool::new(true))
        .is_err());
    assert!(base
        .advance(
            obs,
            now - Duration::from_nanos(1),
            &assets,
            &motion,
            &AtomicBool::new(false)
        )
        .is_err());
}
pub(crate) fn deadline(timeline: &Timeline, entity: i64) -> Instant {
    timeline.pending[&entity].deadline
}
pub(crate) fn counts(timeline: &Timeline) -> (usize, usize) {
    (timeline.pending.len(), timeline.flights.len())
}
pub(crate) fn without_flights(timeline: &Timeline) -> Timeline {
    let mut out = timeline.clone();
    out.flights.clear();
    out
}

pub(crate) fn verify_targets(
    data: &d::UnvalidatedData,
    timeline: &Timeline,
    entity: i64,
) -> WorldCell {
    let shot = events(data).find(|(id, _)| *id == entity).unwrap().1;
    let expected = target(data, shot).unwrap();
    let flight = timeline.flights.iter().find(|f| f.shot == *shot).unwrap();
    assert_eq!(flight.path.from(), cell(&shot.from));
    assert_eq!(flight.path.target(), expected);
    expected
}

/// Independently exercise launch edges against genuine accepted Java receipts.
pub(crate) fn verify_edges(before: Observation, shot: Observation, assets: &Assets) {
    let cancel = AtomicBool::new(false);
    let at = Instant::now() + Duration::from_secs(60);
    let base = Timeline::new(before.clone(), at);
    let motion = battle_motion::Timeline::new(before, at).unwrap();
    let start = at + Duration::from_secs(1);
    let motion = motion.advance(shot.clone(), start, &cancel).unwrap();
    let pending = base
        .advance(shot.clone(), start, assets, &motion, &cancel)
        .unwrap();
    assert_eq!(counts(&pending), (1, 0));
    let hit = deadline(&pending, 1);
    assert!(hit > start);
    assert!(base
        .advance(
            shot.clone(),
            start,
            assets,
            &battle_motion::Timeline::new(base.observation.clone(), at).unwrap(),
            &cancel
        )
        .is_err());
    assert!(pending
        .advance(shot.clone(), hit, assets, &motion, &AtomicBool::new(true))
        .is_err());
    assert_eq!(counts(&pending), (1, 0));
    assert_eq!(
        counts(
            &pending
                .advance(
                    shot.clone(),
                    hit - Duration::from_nanos(1),
                    assets,
                    &motion,
                    &cancel
                )
                .unwrap()
        ),
        (1, 0)
    );
    let launched = pending
        .advance(shot.clone(), hit, assets, &motion, &cancel)
        .unwrap();
    let expected = usize::from(
        !RangedPath::new(
            cell(&pending.pending[&1].shot.from),
            target(shot.data(), &pending.pending[&1].shot).unwrap(),
        )
        .unwrap()
        .sample(Duration::ZERO)
        .finished,
    );
    assert_eq!(counts(&launched), (0, expected));
    assert_eq!(
        counts(
            &launched
                .advance(shot.clone(), hit, assets, &motion, &cancel)
                .unwrap()
        ),
        (0, expected)
    );
    let inclusive = pending
        .advance(shot.clone(), hit + MAX_AGE, assets, &motion, &cancel)
        .unwrap();
    assert_eq!(counts(&inclusive), (0, expected));
    if expected > 0 {
        assert_eq!(inclusive.flights[0].started, hit + MAX_AGE);
    }
    let late = pending
        .advance(
            shot.clone(),
            hit + MAX_AGE + Duration::from_nanos(1),
            assets,
            &motion,
            &cancel,
        )
        .unwrap();
    assert_eq!(counts(&late), (0, 0));
    assert!(late.skipped);
    if expected > 0 {
        let mut full = pending.clone();
        full.flights = vec![launched.flights[0].clone(); MAX_FLIGHTS];
        let full = full
            .advance(shot.clone(), hit, assets, &motion, &cancel)
            .unwrap();
        assert_eq!(counts(&full), (0, MAX_FLIGHTS));
        assert!(full.skipped);
        assert_eq!(full.observation.data(), shot.data());
    }
    let attach = Timeline::new(shot.clone(), start);
    let pose = battle_motion::Timeline::new(shot.clone(), start).unwrap();
    assert_eq!(
        counts(&attach.advance(shot, start, assets, &pose, &cancel).unwrap()),
        (0, 0)
    );
}
pub(crate) fn frozen_endpoint(timeline: &Timeline) -> WorldCell {
    timeline.flights[0].path.target()
}
