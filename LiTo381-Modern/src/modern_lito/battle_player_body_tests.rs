//! Real authority events and real shared pixels/WAV; explicit synthetic map.
//! Headless cue verification is not device playback, complete VIP LOAD or UI.
use super::*;
use crate::modern_lito::{
    compositor::Canvas,
    player_appearance::{Assets, Form, Plan, Playback},
    polymorph_assets::{PolymorphAssets, PolymorphForm},
    resources::SharedMapSectorKey,
    shared_action_catalog::SharedActionCatalog,
    world_map::{SectorInput, WorldMap},
    world_scene::BaseAction,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

fn events(
    frame: &Option<crate::modern_lito::world_audio::AudioFrame>,
) -> Vec<(u64, Instant, Vec<u32>)> {
    frame.as_ref().map_or_else(Vec::new, |f| {
        f.events
            .iter()
            .map(|e| {
                let bank = e.bank.as_ref().unwrap_or(&f.bank);
                for id in e.group.candidates().iter().filter(|id| **id != 0) {
                    assert!(!bank.get(*id).unwrap().pcm_bytes().is_empty());
                }
                (e.sequence, e.created, e.group.candidates().to_vec())
            })
            .collect()
    })
}

#[test]
#[ignore = "actual Java schema11 combat/CAS and shared SPR/WAV; synthetic map, no SQL/TLS/audio device/UI/full VIP LOAD"]
fn java_public_player_body_pixels_and_audio_share_native_clock_without_replay() {
    let mut peer = Peer::start_named(Some("vip-appearance-body"));
    let hello = peer.line();
    let h: Vec<_> = hello.splitn(4, ':').collect();
    assert_eq!(&h[..2], &["SESSION", "240"]);
    let protocol::Message::ExtendedHello(hello, offer) = protocol::decode(&unhex(h[2])).unwrap()
    else {
        panic!("HELLO")
    };
    let mut session = protocol::ModernClientSession::new_extended(hello, offer).unwrap();
    assert_eq!(session.begin().unwrap(), unhex(h[2]));
    let ack = session.accept_ack(&unhex(h[3])).unwrap();
    let p = protocol::extensions::Selected::new(ack, session.extensions().unwrap()).unwrap();
    let mut followers = BTreeMap::new();
    let mut histories: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut frames = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:16");
            break;
        }
        let v: Vec<_> = line.splitn(4, ':').collect();
        assert_eq!(v[0], "REPLY");
        let bytes = unhex(v[3]);
        let data = decode(&bytes, &unhex(v[2]), p).unwrap();
        let follower = followers
            .entry(v[1].to_owned())
            .or_insert_with(|| Follower::new_selected(request(&data), p).unwrap());
        follower.accept(reply(data)).unwrap();
        let observation = follower.observation().unwrap().clone();
        assert!(crate::modern_lito::battle_scene::baseline_counts(observation.clone()).is_err());
        histories
            .entry(v[1].to_owned())
            .or_default()
            .push(observation);
        peer.ack(&bytes);
        frames += 1;
    }
    peer.finish();
    assert_eq!(frames, 16);
    assert_eq!(histories.len(), 4);
    let cancel = AtomicBool::new(false);
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let context = SharedActionCatalog::load(&root, &cancel).unwrap();
    let poly = PolymorphAssets::load_with_catalog(
        &context,
        PolymorphForm::from_wire(5641).unwrap().unwrap(),
        &cancel,
    )
    .unwrap();
    let mut coverage = [0; 3];
    let mut samples = 0;
    for (label, trace) in histories {
        let first = Plan::from_observation(trace[0].clone()).unwrap();
        let manifest = &trace[0].data().manifest;
        assert_eq!(manifest.sectors.0.len(), 1);
        let sector = &manifest.sectors.0[0];
        let bytes = vec![0; 49170];
        assert_eq!(sector.byte_length as usize, bytes.len());
        assert_eq!(sector.sha256, <[u8; 32]>::from(Sha256::digest(&bytes)));
        let map = Arc::new(
            WorldMap::from_sector_bytes(
                vec![SectorInput {
                    key: SharedMapSectorKey {
                        map_id: manifest.map.try_into().unwrap(),
                        sector_x: sector.x.try_into().unwrap(),
                        sector_y: sector.y.try_into().unwrap(),
                    },
                    bytes,
                }],
                &cancel,
            )
            .unwrap(),
        );
        assert_eq!(map.canonical_digest(), manifest.digest);
        let mut assets = Assets::load_with_catalog(&context, map.clone(), &first, &cancel).unwrap();
        let start = Instant::now();
        let mut live = Playback::new(first, start).unwrap();
        let (_, initial) = live.sample_audio(&assets, start, &cancel).unwrap();
        assert!(events(&initial).is_empty());
        let mut canvas = Canvas::new(800, 600).unwrap();
        for (index, observation) in trace.iter().enumerate().skip(1) {
            if observation.data().world.phase == payload::data::Phase::Aborted {
                assert!(Plan::from_observation(observation.clone()).is_err());
                continue;
            }
            let at = start + Duration::from_millis([0, 10, 20, 30, 350][index]);
            let plan = Plan::from_observation(observation.clone()).unwrap();
            assert!(live
                .advance(plan.clone(), &assets, at, &AtomicBool::new(true))
                .is_err());
            if !assets.supports(&plan) {
                assert!(live.advance(plan.clone(), &assets, at, &cancel).is_err());
                assets = Assets::load_with_catalog(&context, map.clone(), &plan, &cancel).unwrap();
            }
            live = live.advance(plan.clone(), &assets, at, &cancel).unwrap();
            let scene = live.motion().scene(at).unwrap();
            assert!(scene
                .actors()
                .iter()
                .all(|a| a.object_id == 3333 && [7, 42].contains(&a.source_id)));
            if index == 1 {
                let fresh = Playback::new(plan.clone(), at).unwrap();
                let (_, no_history) = fresh
                    .sample_audio(&assets, at + Duration::from_millis(60), &cancel)
                    .unwrap();
                assert!(events(&no_history).is_empty());
                for a in fresh.motion().scene(at).unwrap().actors() {
                    assert!(matches!(a.action, BaseAction::Idle | BaseAction::Death));
                    if a.action == BaseAction::Death {
                        assert_eq!(a.action_elapsed, Duration::MAX);
                    }
                }
                let mut boundaries = BTreeSet::from([
                    0u64,
                    40_999_999,
                    41_000_000,
                    500_000_000,
                    1_500_000_000,
                    10_000_000_000,
                ]);
                let mut expected = Vec::new();
                for actor in scene.actors() {
                    assert!(matches!(
                        plan.forms()[&actor.entity_id],
                        Form::Polymorph { .. }
                    ));
                    let (axis, sound, phase) = match actor.action {
                        BaseAction::Attack => (0, 86, 41_000_000),
                        BaseAction::Damage => (1, 88, 0),
                        BaseAction::Death => (2, 89, 0),
                        other => panic!("unexpected body {other:?}"),
                    };
                    coverage[axis] += 1;
                    expected.push((at + Duration::from_nanos(phase), vec![sound]));
                    let action = poly.action(actor.action).unwrap();
                    let mut ns = 0;
                    for source in (1..action.source_frames().len()).chain(std::iter::once(0)) {
                        boundaries.insert(ns);
                        if ns > 0 {
                            boundaries.insert(ns - 1);
                        }
                        ns += action.source_duration(source).unwrap().as_nanos() as u64;
                    }
                    boundaries.insert(ns);
                    boundaries.insert(ns + 1);
                    if actor.action != BaseAction::Death {
                        let end = if actor.action == BaseAction::Attack {
                            852_000_000
                        } else {
                            917_000_000
                        };
                        boundaries.extend([end - 1, end, end + 1]);
                    }
                }
                let mut branch = live.clone();
                let mut observed = BTreeMap::new();
                for ns in boundaries {
                    let time = at + Duration::from_nanos(ns);
                    let poses = branch.motion().scene(time).unwrap();
                    let info = assets
                        .render_motion(branch.motion(), time, &mut canvas, &cancel)
                        .unwrap();
                    for actor in poses.actors() {
                        let original = scene
                            .actors()
                            .iter()
                            .find(|a| a.entity_id == actor.entity_id)
                            .unwrap();
                        let idle_at = match original.action {
                            BaseAction::Attack => Some(852_000_000),
                            BaseAction::Damage => Some(917_000_000),
                            BaseAction::Death => None,
                            _ => unreachable!(),
                        };
                        if let Some(end) = idle_at.filter(|end| ns >= *end) {
                            assert_eq!(actor.action, BaseAction::Idle);
                            assert_eq!(actor.action_elapsed, Duration::from_nanos(ns - end));
                            assert_eq!(actor.heading, original.heading);
                        } else {
                            assert_eq!(actor.action, original.action);
                            assert_eq!(actor.action_elapsed, Duration::from_nanos(ns));
                        }
                        let q = poly.action(actor.action).unwrap();
                        let source = q.sample(actor.action_elapsed, actor.heading).unwrap().0;
                        let shown = info
                            .actors
                            .iter()
                            .find(|v| v.entity_id == actor.entity_id)
                            .unwrap();
                        assert_eq!(shown.source_frame, source);
                        assert_eq!(
                            shown.sprite_name,
                            q.body_asset(source, actor.heading).unwrap().file_name()
                        );
                        assert_eq!(shown.cell, actor.cell);
                        if actor.action != BaseAction::Idle && actor.action_elapsed >= q.duration()
                        {
                            assert_eq!(source, 0);
                        }
                        if actor.action == BaseAction::Attack && ns == 41_000_000 {
                            assert_eq!(source, 2);
                        }
                    }
                    let (sampled, frame) = branch.sample_audio(&assets, time, &cancel).unwrap();
                    for (sequence, created, ids) in events(&frame) {
                        assert!(created <= time);
                        observed.insert(sequence, (created, ids));
                    }
                    let (_, repeat) = sampled.sample_audio(&assets, time, &cancel).unwrap();
                    assert_eq!(events(&frame), events(&repeat));
                    assert!(sampled
                        .sample_audio(&assets, time, &AtomicBool::new(true))
                        .is_err());
                    if ns > 0 {
                        assert!(sampled
                            .sample_audio(&assets, time - Duration::from_nanos(1), &cancel)
                            .is_err());
                    }
                    // A repeated READ after the deadline commits the same
                    // idle phase; it must not reattach the old retained hit.
                    let repeated = sampled
                        .advance(plan.clone(), &assets, time, &cancel)
                        .unwrap();
                    assert_eq!(
                        repeated.motion().scene(time).unwrap().actors(),
                        poses.actors()
                    );
                    let (_, repeated_audio) =
                        repeated.sample_audio(&assets, time, &cancel).unwrap();
                    assert_eq!(events(&frame), events(&repeated_audio));
                    branch = repeated;
                    samples += 1;
                }
                let mut actual: Vec<_> = observed.into_values().collect();
                actual.sort();
                expected.sort();
                assert_eq!(actual, expected);
                // A late first sample consumes stale cues without replaying them.
                let (_, stale) = live
                    .sample_audio(&assets, at + Duration::from_secs(2), &cancel)
                    .unwrap();
                assert!(events(&stale).is_empty());
            }
            let (sampled, frame) = live.sample_audio(&assets, at, &cancel).unwrap();
            let repeated = sampled.advance(plan, &assets, at, &cancel).unwrap();
            let (_, again) = repeated.sample_audio(&assets, at, &cancel).unwrap();
            assert_eq!(events(&frame), events(&again));
            live = sampled;
            if label.starts_with("body-") && index == 3 {
                assert_eq!(scene.actors()[0].action, BaseAction::Idle);
                let (_, later) = live
                    .sample_audio(&assets, start + Duration::from_millis(70), &cancel)
                    .unwrap();
                assert!(
                    events(&later).iter().all(|(_, _, ids)| !ids.contains(&86)),
                    "old form attack must retire before its cue"
                );
            }
            if label.starts_with("body-") && index == 4 {
                let (_, later) = live
                    .sample_audio(&assets, at + Duration::from_millis(41), &cancel)
                    .unwrap();
                assert!(events(&later).iter().any(|(_, created, ids)| *created
                    == at + Duration::from_millis(41)
                    && ids == &[86]));
                // The second player's new attack replaces its former damage
                // tail; the old917ms damage return must not end this attack.
                let old_damage_end = start + Duration::from_millis(10 + 917);
                assert_eq!(
                    live.motion().scene(old_damage_end).unwrap().actors()[1].action,
                    BaseAction::Attack
                );
                for ns in [851_999_999, 852_000_000, 852_000_001] {
                    let pose = live.motion().scene(at + Duration::from_nanos(ns)).unwrap();
                    let actor = &pose.actors()[1];
                    assert_eq!(
                        actor.action,
                        if ns < 852_000_000 {
                            BaseAction::Attack
                        } else {
                            BaseAction::Idle
                        }
                    );
                    assets
                        .render_motion(
                            live.motion(),
                            at + Duration::from_nanos(ns),
                            &mut canvas,
                            &cancel,
                        )
                        .unwrap();
                }
                // The now-untransformed player also returns using its exact
                // existing base-renderer terminal threshold and same fixed wait.
                let actor = &scene.actors()[0];
                assert_eq!(actor.action, BaseAction::Damage);
                let q = crate::modern_lito::combat_animation::QualifiedCombatAnimation::from_catalog_armed(
                    context.catalog(), actor.appearance, actor.weapon_type, actor.action.combat().unwrap()).unwrap();
                let deadline = at + q.duration().unwrap() + Duration::from_millis(584);
                assert_eq!(
                    live.motion()
                        .scene(deadline - Duration::from_nanos(1))
                        .unwrap()
                        .actors()[0]
                        .action,
                    BaseAction::Damage
                );
                let idle = live.motion().scene(deadline).unwrap();
                assert_eq!(idle.actors()[0].action, BaseAction::Idle);
                assert_eq!(idle.actors()[0].action_elapsed, Duration::ZERO);
            }
        }
    }
    assert_eq!(coverage, [4, 2, 2]);
    println!("PLAYER_BODY: 16 actual Java frames; dual-source same3333; {samples} shared SPR samples; Attack/Damage/Death coverage={coverage:?}; WAV86 at native41ms (not24FPS); no repeat/history/stale/reset replay; Attack/Damage source0 then fixed584ms idle tail; Death never returns; base return and interruption verified; full VIP gate retained");
}
