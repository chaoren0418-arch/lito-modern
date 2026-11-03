//! Genuine Java observations + unfiltered scene output, separate from effect-
//! only regressions. Actual Lineage PCM and body SPRs; no device-output claim.
use super::tests::{observation, trace_scene};
use super::*;
use crate::modern_lito::battle_body_audio::tests as body;
use crate::modern_lito::world_audio::SoundEvent;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn events(scene: &SharedScene, surface: &Surface) -> Vec<SoundEvent> {
    surface
        .audio
        .as_ref()
        .into_iter()
        .flat_map(|a| a.events.iter())
        .filter(|e| scene.audio.is_body_sequence(e.sequence))
        .cloned()
        .collect()
}
pub(crate) fn signature(events: &[SoundEvent]) -> Vec<(u64, Instant, Vec<u32>)> {
    events
        .iter()
        .map(|e| (e.sequence, e.created, e.group.candidates().to_vec()))
        .collect()
}

pub(crate) fn verify(groups: &BTreeMap<String, Vec<d::UnvalidatedData>>) {
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let cancel = AtomicBool::new(false);
    let mut coverage = [false; 6];
    let mut cues = 0;
    let mut foot_cues = BTreeSet::new();
    let mut handoffs = 0;
    let mut idle_returns = 0;
    verify_idle_tail_handoff(&root, &groups["motion"]);
    for name in [
        "motion",
        "spell77-normal",
        "spell77-npc",
        "npc",
        "arrow-player",
        "teleport",
    ] {
        let trace = &groups[name];
        let mut painter = trace_scene(&root, &trace[0]);
        let mut player_banks = vec![painter.body_sound.clone()];
        let mut npc_banks = painter
            .npcs
            .as_ref()
            .map(|n| n.sound_bank())
            .into_iter()
            .collect::<Vec<_>>();
        let initial = painter.render(Instant::now(), &cancel).unwrap();
        assert!(
            events(&painter, &initial).is_empty(),
            "baseline cannot replay history"
        );
        let start = Instant::now() + Duration::from_secs(60);
        for (index, data) in trace.iter().enumerate().skip(1) {
            let at = start + Duration::from_secs(index as u64 * 10);
            let obs = observation(data.clone());
            assert!(painter
                .observe_at(obs.clone(), at, &AtomicBool::new(true))
                .is_err());
            painter.observe_at(obs, at, &cancel).unwrap();
            let expected_pc = body::expected_player_cues(&painter.body_audio, &painter.assets);
            let expected_walk = body::expected_walk_cues(&painter.body_audio);
            let mut finishes = Vec::new();
            for actor in painter.input.scene.actors() {
                let entity = actor.entity_id as i64;
                if let Some((_, action, clock)) = painter.motion.player_audio(entity, at).unwrap() {
                    if matches!(action, BaseAction::Attack | BaseAction::Damage) {
                        let q = painter
                            .assets
                            .combat_animation_armed(actor.appearance, actor.weapon_type, action)
                            .unwrap();
                        let end = clock.clone().until(at, q.duration().unwrap()).unwrap()
                            + Duration::from_millis(584);
                        finishes.push((entity, action, end));
                    }
                }
            }
            // Sample every native PC cue boundary exactly (plus NPC frame
            // sampling), not only round milliseconds that can hide drift.
            let mut times = (0..=80)
                .map(|i| at + Duration::from_millis(i * 25))
                .collect::<BTreeSet<_>>();
            for (_, _, end) in &finishes {
                if *end <= at + Duration::from_secs(2) {
                    times.extend([
                        *end - Duration::from_nanos(1),
                        *end,
                        *end + Duration::from_nanos(1),
                    ]);
                    idle_returns += 1;
                }
            }
            for (deadline, _) in &expected_pc {
                if *deadline >= at && *deadline <= at + Duration::from_secs(2) {
                    if *deadline > at {
                        times.insert(*deadline - Duration::from_nanos(1));
                    }
                    times.insert(*deadline);
                }
            }
            for (deadline, _, _) in &expected_walk {
                if *deadline >= at && *deadline <= at + Duration::from_secs(2) {
                    if *deadline > at {
                        times.insert(*deadline - Duration::from_nanos(1));
                    }
                    times.insert(*deadline);
                }
            }
            let mut heard = BTreeMap::<u64, SoundEvent>::new();
            for now in times {
                assert!(painter.render(now, &AtomicBool::new(true)).is_err());
                let surface = painter.render(now, &cancel).unwrap();
                for (entity, action, end) in &finishes {
                    let pose = painter.motion.pose(*entity, now).unwrap();
                    assert_eq!(
                        pose.action,
                        if now < *end {
                            *action
                        } else {
                            BaseAction::Idle
                        }
                    );
                    if now >= *end {
                        assert_eq!(pose.elapsed, now.duration_since(*end));
                    }
                }
                assert_eq!(surface.observation.data(), data);
                let output = events(&painter, &surface);
                for (covered, count) in coverage.iter_mut().zip(body::coverage(&painter.body_audio))
                {
                    *covered |= count > 0;
                }
                for event in &output {
                    assert!(
                        event.created <= now
                            && now.duration_since(event.created) <= Duration::from_millis(500)
                    );
                    let bank = event
                        .bank
                        .as_ref()
                        .expect("each body source retains its own bank");
                    assert!(event
                        .group
                        .candidates()
                        .iter()
                        .all(|id| *id == 0 || bank.get(*id).is_some()));
                    let spatial = event
                        .group
                        .candidates()
                        .iter()
                        .find_map(|id| bank.get(*id).and_then(|s| s.spatial_gain()));
                    if let Some(spatial) = spatial {
                        assert!(
                            expected_walk
                                .iter()
                                .any(|(deadline, group, gain)| *deadline == event.created
                                    && *group == event.group
                                    && *gain == spatial),
                            "footstep must match its actual walk clock/range: {name}/{index}"
                        );
                        for id in event.group.candidates() {
                            let output = bank.get(*id).unwrap();
                            let original = painter.body_sound.get(*id).unwrap();
                            assert_eq!(output.sha256(), original.sha256());
                            assert_eq!(output.format().channels, 2);
                            assert_eq!(
                                output.pcm_bytes(),
                                original
                                    .with_spatial(spatial.0, spatial.1)
                                    .unwrap()
                                    .pcm_bytes()
                            );
                        }
                        foot_cues.insert((name, index, event.sequence));
                    } else if player_banks.iter().any(|b| Arc::ptr_eq(bank, b)) {
                        assert!(
                            expected_pc
                                .iter()
                                .any(|(deadline, group)| *deadline == event.created
                                    && *group == event.group),
                            "PC cue must match body-clock deadline, {name}/{index}"
                        );
                    } else {
                        assert!(npc_banks.iter().any(|b| Arc::ptr_eq(bank, b)));
                        if event.created >= at {
                            let assets = painter.npcs.as_ref().unwrap();
                            assert!(
                                data.npcs.as_ref().unwrap().actors.0.iter().any(|npc| {
                                    let pose = painter
                                        .motion
                                        .npc_sample(npc, assets, event.created)
                                        .unwrap();
                                    assets
                                        .action(
                                            NpcAppearance::new(npc.graphic as u32).unwrap(),
                                            pose.action,
                                        )
                                        .unwrap()
                                        .sound_group(pose.source)
                                        .unwrap()
                                        == Some(&event.group)
                                }),
                                "NPC cue timestamp must point to its actual displayed source frame"
                            );
                        }
                    }
                    if let Some(previous) = heard.insert(event.sequence, event.clone()) {
                        assert_eq!(
                            signature(&[previous]),
                            signature(std::slice::from_ref(event))
                        );
                    }
                }
                let repeated = painter.render(now, &cancel).unwrap();
                assert_eq!(surface.bgra, repeated.bgra);
                assert_eq!(signature(&output), signature(&events(&painter, &repeated)));
                if (name == "motion" || name == "teleport") && index == 3 && now == at {
                    let mut replacement = trace_scene(&root, data);
                    assert!(replacement
                        .carry_from(&mut painter, now, &AtomicBool::new(true))
                        .is_err());
                    replacement.carry_from(&mut painter, now, &cancel).unwrap();
                    let carried = replacement.render(now, &cancel).unwrap();
                    assert_eq!(surface.bgra, carried.bgra);
                    assert_eq!(
                        signature(&output),
                        signature(&events(&replacement, &carried))
                    );
                    assert!(painter.render(now, &cancel).is_err());
                    // Handoff retains queued banks. The next sample may add
                    // cues from the replacement bank; identity is per-event.
                    player_banks.push(replacement.body_sound.clone());
                    npc_banks.extend(replacement.npcs.as_ref().map(|n| n.sound_bank()));
                    painter = replacement;
                    handoffs += 1;
                }
            }
            cues += heard.len();
        }
        // A fresh attachment suppresses retained attack/death history, even
        // when the snapshot still contains authoritative combat receipts.
        let mut historical = trace_scene(&root, trace.last().unwrap());
        let now = Instant::now();
        let surface = historical.render(now, &cancel).unwrap();
        assert!(events(&historical, &surface).is_empty());
        let surface = painter
            .render(start + Duration::from_secs(1000), &cancel)
            .unwrap();
        assert!(
            events(&painter, &surface).is_empty(),
            "stalled render cannot burst old cues"
        );
    }
    assert!(
        coverage.into_iter().all(|v| v),
        "positive PC/NPC attack/hurt/death cue coverage {coverage:?}"
    );
    assert_eq!(handoffs, 2);
    assert!(idle_returns > 0);
    assert!(
        !foot_cues.is_empty(),
        "genuine confirmed movement must emit footsteps"
    );
    eprintln!("canonical body audio: 101 genuine Java frames decoded; 6 shared-art series, {cues} distinct body cues including {} footsteps; PC/NPC attack/hurt/death positive, exact PC/walk clocks, {idle_returns} exact idle-return boundaries plus mid-tail resource handoff, per-event PCM banks, repaint/cancel/combat+walk handoff/history/stall checks; no HWND/device output claim",foot_cues.len());
}

fn verify_idle_tail_handoff(root: &Path, trace: &[d::UnvalidatedData]) {
    let cancel = AtomicBool::new(false);
    // The motion stream contains explicit physical attacks, unlike the
    // arrow-player spell4 stream. Locate a confirmed attack transition rather
    // than mistaking a spell/hurt/loading observation for a melee attack.
    let origin = Instant::now();
    let pair = trace
        .windows(2)
        .find(|pair| {
            if pair[0].world.phase == d::Phase::Aborted || pair[1].world.phase == d::Phase::Aborted
            {
                return false;
            }
            let prior = crate::modern_lito::battle_motion::Timeline::new(
                observation(pair[0].clone()),
                origin,
            )
            .unwrap();
            let next = prior
                .advance(
                    observation(pair[1].clone()),
                    origin + Duration::from_millis(1),
                    &cancel,
                )
                .unwrap();
            pair[1].actors.0.iter().any(|a| {
                next.pose(a.entity, origin + Duration::from_millis(1))
                    .unwrap()
                    .action
                    == BaseAction::Attack
            })
        })
        .expect("genuine player attack transition");
    let mut old = trace_scene(root, &pair[0]);
    let at = Instant::now() + Duration::from_secs(60);
    old.observe_at(observation(pair[1].clone()), at, &cancel)
        .unwrap();
    let actor = old
        .input
        .scene
        .actors()
        .iter()
        .find(|a| old.motion.pose(a.entity_id as i64, at).unwrap().action == BaseAction::Attack)
        .unwrap();
    let entity = actor.entity_id as i64;
    let q = old
        .assets
        .combat_animation_armed(actor.appearance, actor.weapon_type, BaseAction::Attack)
        .unwrap();
    let (_, _, clock) = old.motion.player_audio(entity, at).unwrap().unwrap();
    let end = clock.clone().until(at, q.duration().unwrap()).unwrap() + Duration::from_millis(584);
    let carry = end - Duration::from_millis(100);
    let before = old.render(carry, &cancel).unwrap();
    let mut new = trace_scene(root, &pair[1]);
    assert!(new
        .carry_from(&mut old, carry, &AtomicBool::new(true))
        .is_err());
    assert_eq!(before.bgra, old.render(carry, &cancel).unwrap().bgra);
    new.carry_from(&mut old, carry, &cancel).unwrap();
    assert_eq!(before.bgra, new.render(carry, &cancel).unwrap().bgra);
    assert_eq!(
        new.motion
            .pose(entity, end - Duration::from_nanos(1))
            .unwrap()
            .action,
        BaseAction::Attack
    );
    assert_eq!(
        new.motion.pose(entity, end).unwrap().action,
        BaseAction::Idle
    );
    assert_eq!(
        new.motion.pose(entity, end).unwrap().elapsed,
        Duration::ZERO
    );
    new.render(end, &cancel).unwrap();
    new.observe_at(
        observation(pair[1].clone()),
        end + Duration::from_millis(50),
        &cancel,
    )
    .unwrap();
    let pose = new
        .motion
        .pose(entity, end + Duration::from_millis(50))
        .unwrap();
    assert_eq!(pose.action, BaseAction::Idle);
    assert_eq!(pose.elapsed, Duration::from_millis(50));
    assert!(old.render(end, &cancel).is_err());
}
