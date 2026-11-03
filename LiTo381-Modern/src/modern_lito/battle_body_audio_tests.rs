use super::*;
use crate::modern_lito::{combat_animation::classify_native_cues, sprite_actions::FrameEvent};

fn fixture(at: Instant) -> (Timeline, Motion, Arc<SoundBank>) {
    let obs = crate::modern_lito::battle_scene::tests::observation(
        crate::modern_lito::battle_payload::data_tests::fixture(),
    );
    let motion = Motion::new(obs.clone(), at).unwrap();
    let mut timeline = Timeline::new(obs, at);
    let key = PlayerKey {
        entity: 1,
        life: 1,
        appearance: BaseAppearance { class: 1, sex: 0 },
        weapon: 0,
        logical_form: 0,
        action: BaseAction::Attack,
        revision: 2,
    };
    timeline.players.insert(
        1,
        PlayerTrack {
            key,
            bank: None,
            cues: vec![Cue {
                phase: Duration::ZERO,
                group: classify_native_cues(&[FrameEvent::Sound(0)])
                    .unwrap()
                    .audio
                    .unwrap(),
                deadline: at,
                consumed: false,
            }],
        },
    );
    (
        timeline,
        motion,
        Arc::new(SoundBank::playback_subset(BTreeMap::new()).unwrap()),
    )
}

#[test]
fn age_boundary_consumption_and_retirement_are_finite() {
    let at = Instant::now();
    let cancel = AtomicBool::new(false);
    let (original, motion, bank) = fixture(at);
    for (age, count) in [
        (Duration::from_millis(500), 1),
        (Duration::from_millis(500) + Duration::from_nanos(1), 0),
    ] {
        let now = at + age;
        let (mut sampled, _) = original.sample(&motion, &bank, None, now, &cancel).unwrap();
        assert_eq!(sampled.log.snapshot(now).len(), count);
        assert!(sampled.players[&1].cues[0].consumed);
        let sequence = sampled.log.last_sequence();
        let (mut again, _) = sampled.sample(&motion, &bank, None, now, &cancel).unwrap();
        assert_eq!(again.log.last_sequence(), sequence);
        assert_eq!(again.log.snapshot(now).len(), count);
        again.players.clear();
        again.prune();
        assert!(again.log.snapshot(now).is_empty());
        assert!(again.owners.is_empty());
        assert!(!original.players[&1].cues[0].consumed);
    }
}

#[test]
fn failed_samples_do_not_consume_cues_or_poison_the_clock() {
    let at = Instant::now();
    let cancel = AtomicBool::new(false);
    let (mut original, motion, bank) = fixture(at);
    assert!(original
        .sample(&motion, &bank, None, at, &AtomicBool::new(true))
        .is_err());
    assert!(original
        .sample(&motion, &bank, None, at - Duration::from_nanos(1), &cancel)
        .is_err());
    original.players.get_mut(&1).unwrap().cues[0].group =
        classify_native_cues(&[FrameEvent::Sound(97)])
            .unwrap()
            .audio
            .unwrap();
    assert!(original.sample(&motion, &bank, None, at, &cancel).is_err());
    assert_eq!(original.sampled, at);
    assert_eq!(original.log.last_sequence(), 0);
    assert!(!original.players[&1].cues[0].consumed);
    assert!(original.owners.is_empty());
}
#[test]
fn body_cue_retiming_preserves_fractional_phase_and_already_crossed_deadlines() {
    let at = Instant::now();
    let (original, _, _) = fixture(at);
    let mut cues = original.players[&1].cues.clone();
    cues[0].phase = Duration::from_millis(41);
    cues[0].deadline = at + cues[0].phase;
    let mut clock = PacedClock::new(at);
    let changed = at + Duration::from_millis(10);
    clock.retime(changed, 200_000_000, 151_000_000).unwrap();
    retime(&mut cues, &clock, changed).unwrap();
    // 10ms already consumed; remaining31ms *151/200 =23.405ms.
    let deadline = at + Duration::from_micros(33405);
    assert_eq!(cues[0].deadline, deadline);
    assert!(clock.peek(deadline - Duration::from_nanos(1)).unwrap() < cues[0].phase);
    assert_eq!(clock.peek(deadline).unwrap(), cues[0].phase);
    let later = at + Duration::from_millis(40);
    clock.retime(later, 200_000_000, 400_000_000).unwrap();
    retime(&mut cues, &clock, later).unwrap();
    assert_eq!(
        cues[0].deadline, deadline,
        "undelivered crossed cue cannot move forward"
    );
    cues[0].consumed = true;
    retime(&mut cues, &clock, later).unwrap();
    assert!(cues[0].consumed);
}

#[test]
fn incarnation_weapon_action_and_revision_each_retire_old_owner_cues() {
    let at = Instant::now();
    let (original, motion, bank) = fixture(at);
    let (sampled, _) = original
        .sample(&motion, &bank, None, at, &AtomicBool::new(false))
        .unwrap();
    for axis in 0..6 {
        let mut changed = sampled.clone();
        let key = &mut changed.players.get_mut(&1).unwrap().key;
        match axis {
            0 => key.life += 1,
            1 => key.weapon = 1,
            2 => key.action = BaseAction::Damage,
            3 => key.revision += 1,
            4 => key.appearance.sex = 1,
            _ => key.logical_form = 5641,
        }
        changed.prune();
        assert!(changed.log.snapshot(at).is_empty());
        assert!(changed.owners.is_empty());
    }
}

/// Inspect producer ownership without identifying sounds by numeric IDs; a
/// skill and body may legally use the same WAV and must stay independent.
pub(crate) fn coverage(timeline: &Timeline) -> [usize; 6] {
    let mut counts = [0; 6];
    for owner in timeline.owners.values() {
        let index = match owner {
            Owner::Walk(_) => continue,
            Owner::Player(k) => match k.action {
                BaseAction::Attack => 0,
                BaseAction::Damage => 1,
                BaseAction::Death => 2,
                _ => continue,
            },
            Owner::Npc(k) => match timeline.npcs[&k.serial].marker.map(|m| m.0) {
                Some(NpcAction::Attack) => 3,
                Some(NpcAction::Damage) => 4,
                Some(NpcAction::Death) => 5,
                _ => continue,
            },
        };
        counts[index] += 1;
    }
    counts
}

pub(crate) fn expected_player_cues(
    timeline: &Timeline,
    assets: &WorldSceneAssets,
) -> Vec<(Instant, NativeAudioGroup)> {
    for track in timeline.players.values() {
        let key = &track.key;
        let animation = assets
            .combat_animation_armed(key.appearance, key.weapon, key.action)
            .unwrap()
            .animation();
        let mut tick = 0u64;
        let mut expected = Vec::new();
        for source in (1..animation.frames.len()).chain(std::iter::once(0)) {
            let frame = &animation.frames[source];
            if let Some(group) = classify_native_cues(&frame.events).unwrap().audio {
                expected.push((
                    Duration::from_nanos(
                        (u128::from(tick) * 1_000_000_000)
                            .div_ceil(u128::from(animation.frames_per_second))
                            as u64,
                    ),
                    group,
                ));
            }
            tick += u64::from(frame.duration_ticks);
        }
        assert_eq!(
            track
                .cues
                .iter()
                .map(|c| (c.phase, c.group.clone()))
                .collect::<Vec<_>>(),
            expected,
            "schedule must follow actual source 1..N-1,0 cues, not Hit or guessed sounds"
        );
    }
    timeline
        .players
        .values()
        .flat_map(|p| p.cues.iter())
        .map(|c| (c.deadline, c.group.clone()))
        .collect()
}

pub(crate) fn expected_walk_cues(
    timeline: &Timeline,
) -> Vec<(Instant, NativeAudioGroup, (u8, u8))> {
    timeline
        .walkers
        .values()
        .filter(|p| p.spatial.0 > 0)
        .flat_map(|p| {
            p.cues
                .iter()
                .map(|c| (c.deadline, c.group.clone(), p.spatial))
        })
        .collect()
}

#[test]
fn pace_change_moves_only_uncrossed_deadlines_and_never_rewinds_consumed_cues() {
    let at = Instant::now();
    let group = classify_native_cues(&[FrameEvent::Sound(0)])
        .unwrap()
        .audio
        .unwrap();
    let mut cues = [100, 300, 600].map(|ms| Cue {
        phase: Duration::from_millis(ms),
        group: group.clone(),
        deadline: at + Duration::from_millis(ms),
        consumed: false,
    });
    let mut clock = PacedClock::new(at);
    clock
        .retime(at + Duration::from_millis(200), 1000, 500)
        .unwrap();
    retime(&mut cues, &clock, at + Duration::from_millis(200)).unwrap();
    assert_eq!(cues[0].deadline, at + Duration::from_millis(100));
    assert_eq!(cues[1].deadline, at + Duration::from_millis(250));
    assert_eq!(cues[2].deadline, at + Duration::from_millis(400));
    cues[1].consumed = true;
    clock
        .retime(at + Duration::from_millis(300), 1000, 1000)
        .unwrap();
    retime(&mut cues, &clock, at + Duration::from_millis(300)).unwrap();
    assert_eq!(cues[0].deadline, at + Duration::from_millis(100));
    assert_eq!(cues[1].deadline, at + Duration::from_millis(250));
    assert_eq!(cues[2].deadline, at + Duration::from_millis(500));
}

pub(crate) fn verify_pace_update(old: &Timeline, next: &Timeline, motion: &Motion, now: Instant) {
    assert!(
        !old.players.is_empty(),
        "real bow trace must own a body action"
    );
    let mut checked = 0;
    for (id, before) in &old.players {
        let Some(after) = next.players.get(id).filter(|p| p.key == before.key) else {
            continue;
        };
        let (_, _, clock) = motion.player_audio(*id, now).unwrap().unwrap();
        assert_eq!(before.cues.len(), after.cues.len());
        for (a, b) in before.cues.iter().zip(&after.cues) {
            assert_eq!(a.phase, b.phase);
            assert_eq!(a.group, b.group);
            assert_eq!(a.consumed, b.consumed);
            let expected = if !a.consumed && a.deadline > now {
                clock.clone().until(now, a.phase).unwrap()
            } else {
                a.deadline
            };
            assert_eq!(b.deadline, expected);
            checked += 1;
        }
    }
    assert!(checked > 0, "real bow action must contain shared cues");
}
