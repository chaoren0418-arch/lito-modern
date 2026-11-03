use super::*;
use crate::modern_lito::{
    audio_gain::tests::wave,
    combat_audio::PreparedSound,
    sprite_actions::{ActionLimits, SpriteActionCatalog},
};

fn walk() -> QualifiedFixedWalk {
    let catalog = SpriteActionCatalog::parse_for_inspection(
        b"#0 64 player\n4.walk(1 8,0.0:2 0.1:2<478 0.2:2 0.3:2 0.4:2 0.5:2<479 0.6:2 0.7:2)\n",
        ActionLimits::default(),
    )
    .unwrap();
    QualifiedFixedWalk::from_player_catalog(&catalog, BaseAppearance { class: 0, sex: 0 }, 1)
        .unwrap()
}
fn sound_bank() -> Arc<SoundBank> {
    Arc::new(
        SoundBank::playback_subset(
            [478, 479]
                .map(|id| {
                    (
                        id,
                        PreparedSound::from_wav(id, wave(16, 1, &[0, 128, 255, 127])).unwrap(),
                    )
                })
                .into(),
        )
        .unwrap(),
    )
}
fn fixture(at: Instant) -> (Timeline, Motion, Arc<SoundBank>) {
    use crate::modern_lito::{battle_payload::data as d, battle_scene::tests::observation};
    let mut data = crate::modern_lito::battle_payload::data_tests::fixture();
    data.world.revision = 2;
    data.world.phase = d::Phase::Active;
    data.world.loading_elapsed_nanos = 100_000_000;
    data.world.active_elapsed_nanos = 0;
    for actor in &mut data.actors.0 {
        actor.loaded = true;
        actor.step_interval_nanos = 200_000_000;
    }
    let motion = Motion::new(observation(data.clone()), at).unwrap();
    data.world.revision += 1;
    data.actors.0[0].position.x += 1;
    data.actors.0[0].heading = 2;
    data.actors.0[0].last_move_sequence += 1;
    let obs = observation(data);
    let motion = motion
        .advance(obs.clone(), at, &AtomicBool::new(false))
        .unwrap();
    let mut audio = Timeline::new(obs, at);
    audio.walkers.insert(
        1,
        Track {
            key: PlayerKey {
                entity: 1,
                life: 1,
                appearance: BaseAppearance { class: 0, sex: 0 },
                weapon: 1,
                logical_form: 0,
                action: BaseAction::Walk,
                revision: 1,
            },
            cues: schedule(
                &walk(),
                200_000_000,
                motion.walk_audio(1, at).unwrap().unwrap(),
                at,
            )
            .unwrap(),
            spatial: (127, 64),
        },
    );
    (audio, motion, sound_bank())
}

#[test]
fn exact_pixel_boundary_inverse_handles_rounding_and_pace_changes() {
    let at = Instant::now();
    let walk = walk();
    let mut clock = PacedClock::new(at);
    let mut cues = schedule(&walk, 200_000_003, &clock, at).unwrap();
    assert_eq!(cues.len(), 2);
    assert_eq!(cues[0].phase, Duration::ZERO);
    assert_eq!(cues[1].phase, Duration::from_nanos(100_000_002));
    for (cue, expected) in cues.iter().zip([1, 5]) {
        for (offset, source) in if expected == 1 {
            vec![(0, 1)]
        } else {
            vec![(-1, 4), (0, 5), (1, 5)]
        } {
            let phase = (cue.phase.as_nanos() as i128 + offset) as u128
                * walk.step_duration().as_nanos()
                / 200_000_003;
            let sample = walk
                .start_unpaced_step(0)
                .unwrap()
                .advance(Duration::from_nanos(phase as u64), &AtomicBool::new(false))
                .unwrap();
            assert_eq!(sample.source_frame, source);
        }
    }
    clock
        .retime(at + Duration::from_millis(50), 200, 100)
        .unwrap();
    retime(&mut cues, &clock, at + Duration::from_millis(50)).unwrap();
    assert_eq!(cues[0].deadline, at);
    assert_eq!(cues[1].deadline, at + Duration::from_nanos(75_000_001));
    assert!(schedule(&walk, 0, &clock, at).is_err());
    assert!(schedule(&walk, 10_000_000_001, &clock, at).is_err());
}
#[test]
fn footsteps_emit_once_drop_catchup_and_expire_with_the_visible_step() {
    let at = Instant::now();
    let cancel = AtomicBool::new(false);
    let (original, motion, bank) = fixture(at);
    let (first, frame) = original.sample(&motion, &bank, None, at, &cancel).unwrap();
    assert_eq!(frame.unwrap().events.len(), 1);
    let (again, _) = first.sample(&motion, &bank, None, at, &cancel).unwrap();
    assert_eq!(again.log.last_sequence(), 1);
    let (second, frame) = again
        .sample(
            &motion,
            &bank,
            None,
            at + Duration::from_millis(100),
            &cancel,
        )
        .unwrap();
    let frame = frame.unwrap();
    assert_eq!(
        frame.events.len(),
        1,
        "new marker withdraws the old queued foot"
    );
    let mut delayed_ui = crate::modern_lito::battle_audio::Mixer::default();
    let delivered = delayed_ui
        .merge(
            [None, None, None, None, None, None, Some(frame)],
            at + Duration::from_millis(100),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        delivered.events.len(),
        1,
        "a stalled UI cannot receive both feet together"
    );
    assert_eq!(delivered.events[0].group.candidates(), [479]);
    assert_eq!(second.log.last_sequence(), 2);
    let (done, frame) = second
        .sample(
            &motion,
            &bank,
            None,
            at + Duration::from_millis(200),
            &cancel,
        )
        .unwrap();
    assert!(frame.unwrap().events.is_empty());
    assert!(done.walkers.is_empty() && done.owners.is_empty());
    let (late, frame) = original
        .sample(
            &motion,
            &bank,
            None,
            at + Duration::from_millis(150),
            &cancel,
        )
        .unwrap();
    let frame = frame.unwrap();
    assert_eq!(
        frame.events.len(),
        1,
        "do not stack both feet after a stall"
    );
    assert_eq!(frame.events[0].group.candidates(), [479]);
    assert_eq!(frame.events[0].created, at + Duration::from_millis(100));
    assert!(late.walkers[&1].cues.iter().all(|c| c.consumed));
    assert!(original.walkers[&1].cues.iter().all(|c| !c.consumed));
    let (_, frame) = original
        .sample(&motion, &bank, None, at + Duration::from_secs(1), &cancel)
        .unwrap();
    assert!(frame.unwrap().events.is_empty());
}
#[test]
fn footstep_failures_distance_silence_and_owner_retirement_are_atomic() {
    let at = Instant::now();
    let cancel = AtomicBool::new(false);
    let (original, motion, bank) = fixture(at);
    let empty = Arc::new(SoundBank::playback_subset(BTreeMap::new()).unwrap());
    assert!(original.sample(&motion, &empty, None, at, &cancel).is_err());
    assert!(original
        .sample(&motion, &bank, None, at, &AtomicBool::new(true))
        .is_err());
    assert!(original
        .sample(&motion, &bank, None, at - Duration::from_nanos(1), &cancel)
        .is_err());
    assert!(original.walkers[&1].cues.iter().all(|c| !c.consumed));
    assert_eq!(original.log.last_sequence(), 0);
    let mut far = original.clone();
    far.walkers.get_mut(&1).unwrap().spatial = (0, 64);
    let (mut silent, frame) = far.sample(&motion, &bank, None, at, &cancel).unwrap();
    assert!(frame.unwrap().events.is_empty());
    silent.walkers.get_mut(&1).unwrap().spatial = (127, 64);
    let (_, frame) = silent.sample(&motion, &bank, None, at, &cancel).unwrap();
    assert!(
        frame.unwrap().events.is_empty(),
        "moving into range cannot replay an old marker"
    );
    let (sampled, _) = original.sample(&motion, &bank, None, at, &cancel).unwrap();
    for axis in 0..4 {
        let mut next = sampled.clone();
        let key = &mut next.walkers.get_mut(&1).unwrap().key;
        match axis {
            0 => key.life += 1,
            1 => key.weapon = 0,
            2 => key.revision += 1,
            _ => key.appearance.sex = 1,
        }
        next.prune();
        assert!(next.log.snapshot(at).is_empty());
    }
}
#[test]
fn native_range_geometry_is_bounded_symmetric_and_not_screen_coordinate_based() {
    let origin = WorldCell { x: 32768, y: 32768 };
    for (dx, dy, expected) in [
        (0, 0, (127, 64)),
        (4, 0, (116, 46)),
        (-4, 0, (116, 82)),
        (0, 4, (116, 46)),
        (3, 4, (110, 33)),
        (13, 0, (9, 6)),
        (14, 0, (0, 64)),
        (9, 9, (0, 1)),
        (-9, -9, (0, 127)),
    ] {
        assert_eq!(
            spatial(
                WorldCell {
                    x: origin.x + dx,
                    y: origin.y + dy
                },
                origin
            ),
            expected
        );
    }
    assert_eq!(
        spatial(
            WorldCell {
                x: i32::MAX,
                y: i32::MIN
            },
            WorldCell {
                x: i32::MIN,
                y: i32::MAX
            }
        ),
        (0, 64)
    );
}

#[test]
fn newly_visible_midstep_waits_for_fresh_motion_and_hidden_events_are_withdrawn() {
    use crate::modern_lito::{
        participant::MemberKind,
        world_scene::{tests::synthetic_assets, ActorState},
    };
    let at = Instant::now();
    let cancel = AtomicBool::new(false);
    let (timeline, motion, bank) = fixture(at);
    let actor = ActorState {
        kind: MemberKind::Human,
        entity_id: 1,
        source_id: 1,
        object_id: 1,
        appearance: BaseAppearance { class: 0, sex: 0 },
        weapon_type: 1,
        cell: WorldCell { x: 32768, y: 32768 },
        heading: 2,
        action: BaseAction::Walk,
        action_elapsed: Duration::ZERO,
    };
    let visible = SceneSnapshot::new(1, 3, [1; 32], 1, vec![actor.clone()]).unwrap();
    let hidden = SceneSnapshot::new_with_hidden(1, 3, [1; 32], 1, vec![actor], &[1]).unwrap();
    let assets = synthetic_assets();
    let (sampled, _) = timeline.sample(&motion, &bank, None, at, &cancel).unwrap();
    let hidden_at = at + Duration::from_millis(20);
    let mut withdrawn = sampled
        .advance(
            sampled.observation.clone(),
            &motion,
            &hidden,
            &assets,
            None,
            hidden_at,
            &cancel,
        )
        .unwrap();
    assert!(withdrawn.walkers.is_empty());
    assert!(withdrawn.log.snapshot(hidden_at).is_empty());
    let resumed = withdrawn
        .advance(
            withdrawn.observation.clone(),
            &motion,
            &visible,
            &assets,
            None,
            at + Duration::from_millis(50),
            &cancel,
        )
        .unwrap();
    assert!(
        resumed.walkers.is_empty(),
        "no newly attached middle-of-step history"
    );
}

#[test]
fn per_footstep_pcm_budget_and_candidate_qualification_are_enforced() {
    let cancel = AtomicBool::new(false);
    let group =
        classify_native_cues(&[crate::modern_lito::sprite_actions::FrameEvent::Effect(478)])
            .unwrap()
            .audio
            .unwrap();
    for bytes in [128 * 1024, 128 * 1024 + 2] {
        let source = Arc::new(
            SoundBank::playback_subset(BTreeMap::from([(
                478,
                PreparedSound::from_wav(478, wave(16, 1, &vec![0; bytes])).unwrap(),
            )]))
            .unwrap(),
        );
        assert_eq!(
            bank(&source, &group, (127, 64), &cancel).is_ok(),
            bytes == 128 * 1024
        );
    }
    assert!(bank(&sound_bank(), &group, (127, 64), &AtomicBool::new(true)).is_err());
    let other = classify_native_cues(&[crate::modern_lito::sprite_actions::FrameEvent::Effect(97)])
        .unwrap()
        .audio
        .unwrap();
    assert!(bank(&sound_bank(), &other, (127, 64), &cancel).is_err());
}
#[test]
#[ignore = "explicit shared Lineage catalog and footstep PCM; no device/UI"]
fn real_shared_player_footsteps_match_all_qualified_walk_boundaries_and_pcm() {
    use crate::modern_lito::{
        combat_animation::qualify_weapon_appearance, combat_audio::BankLimits,
        shared_action_catalog::SharedActionCatalog,
    };
    let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root");
    let cancel = AtomicBool::new(false);
    let catalog = SharedActionCatalog::load(std::path::Path::new(&root), &cancel).unwrap();
    let at = Instant::now();
    let mut groups = Vec::new();
    let mut families = 0;
    let mut markers = 0;
    for class in 0..7 {
        for sex in 0..2 {
            for weapon in 0..=17 {
                let form = BaseAppearance { class, sex };
                if qualify_weapon_appearance(form, weapon).is_err() {
                    continue;
                }
                let walk = QualifiedFixedWalk::from_player_catalog(catalog.catalog(), form, weapon)
                    .unwrap();
                for step in [200_000_003, 1_000_000_000] {
                    let cues = schedule(&walk, step, &PacedClock::new(at), at).unwrap();
                    let expected = walk
                        .source_frames()
                        .iter()
                        .filter(|f| !f.events.is_empty())
                        .count();
                    assert_eq!(cues.len(), expected);
                    for cue in &cues {
                        let phase = cue.phase.as_nanos() * walk.step_duration().as_nanos()
                            / u128::from(step);
                        let pose = walk
                            .start_unpaced_step(0)
                            .unwrap()
                            .advance(Duration::from_nanos(phase as u64), &cancel)
                            .unwrap();
                        assert_eq!(
                            classify_native_cues(&walk.source_frames()[pose.source_frame].events)
                                .unwrap()
                                .audio,
                            Some(cue.group.clone())
                        );
                        if !cue.phase.is_zero() {
                            let before = (cue.phase.as_nanos() - 1)
                                * walk.step_duration().as_nanos()
                                / u128::from(step);
                            let before = walk
                                .start_unpaced_step(0)
                                .unwrap()
                                .advance(Duration::from_nanos(before as u64), &cancel)
                                .unwrap();
                            assert_ne!(pose.source_frame, before.source_frame);
                        }
                        groups.push(cue.group.clone());
                        markers += 1;
                    }
                }
                families += 1;
            }
        }
    }
    assert_eq!(families, 144);
    assert!(markers > 0);
    let shared =
        Arc::new(SoundBank::load_shared(&root, &groups, BankLimits::default(), &cancel).unwrap());
    for group in &groups {
        let playback = bank(&shared, group, (116, 46), &cancel).unwrap();
        for id in group.candidates() {
            let original = shared.get(*id).unwrap();
            let output = playback.get(*id).unwrap();
            assert_eq!(output.sha256(), original.sha256());
            assert_eq!(output.spatial_gain(), Some((116, 46)));
            assert_eq!(output.format().channels, 2);
            assert_eq!(
                output.pcm_bytes(),
                original.with_spatial(116, 46).unwrap().pcm_bytes()
            );
        }
    }
    eprintln!("shared footsteps: {families} qualified families, {markers} exact cue boundaries across 2 intervals, actual 478/479 WAV decode and bounded stereo PCM; no device/UI claim");
}
