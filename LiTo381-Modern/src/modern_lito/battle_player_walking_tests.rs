//! Actual authority moves -> immutable motion -> native shared player pixels.
//! Explicit synthetic geometry, not a full VIP LOAD/GUI proof.
use super::*;
use crate::modern_lito::{
    compositor::Canvas,
    fixed_walk::QualifiedFixedWalk,
    player_appearance::{Assets, Form, Motion, Plan, Playback},
    polymorph_assets::{PolymorphAssets, PolymorphForm},
    resources::SharedMapSectorKey,
    shared_action_catalog::SharedActionCatalog,
    world_map::{PixelPoint, SectorInput, WorldMap},
    world_scene::BaseAction,
};
use sha2::{Digest, Sha256};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

#[test]
#[ignore = "actual Java schema11 confirmed movement and shared native raster; synthetic map, no SQL/TLS/UI or full VIP LOAD"]
fn java_public_player_walks_keep_native_body_camera_and_authority_separate() {
    let mut peer = Peer::start_named(Some("vip-appearance-walk"));
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
            assert_eq!(line, "CLOSED:44");
            break;
        }
        let v: Vec<_> = line.splitn(4, ':').collect();
        assert_eq!(v[0], "REPLY");
        let bytes = unhex(v[3]);
        let data = decode(&bytes, &unhex(v[2]), p).unwrap();
        let f = followers
            .entry(v[1].to_owned())
            .or_insert_with(|| Follower::new_selected(request(&data), p).unwrap());
        f.accept(reply(data)).unwrap();
        let o = f.observation().unwrap().clone();
        assert!(crate::modern_lito::battle_scene::baseline_counts(o.clone()).is_err());
        histories.entry(v[1].to_owned()).or_default().push(o);
        frames += 1;
        peer.ack(&bytes);
    }
    peer.finish();
    assert_eq!(frames, 44);
    let cancel = AtomicBool::new(false);
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let context = SharedActionCatalog::load(&root, &cancel).unwrap();
    let form = PolymorphForm::from_wire(5641).unwrap().unwrap();
    let poly = PolymorphAssets::load_with_catalog(&context, form, &cancel).unwrap();
    let mut samples = 0;
    let mut local = 0;
    let mut remote = 0;
    let mut switches = 0;
    for trace in histories.values() {
        assert_eq!(trace.len(), 22);
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
        let mut motion = Motion::new(first, start).unwrap();
        let mut playback = Playback::new(motion.plan().clone(), start).unwrap();
        let mut canvas = Canvas::new(800, 600).unwrap();
        // A consumer missing two confirmed steps must not invent a route.
        let skipped = motion
            .advance(
                Plan::from_observation(trace[2].clone()).unwrap(),
                start + Duration::from_millis(500),
                &cancel,
            )
            .unwrap();
        assert!(skipped
            .scene(start + Duration::from_millis(500))
            .unwrap()
            .actors()
            .iter()
            .all(|a| a.action == BaseAction::Idle));
        for (index, observation) in trace.iter().enumerate().skip(1) {
            if index == 21 {
                assert!(Plan::from_observation(observation.clone()).is_err());
                continue;
            }
            let at = start
                + Duration::from_millis(match index {
                    17 => 4050,
                    18 => 4250,
                    19 => 4300,
                    20 => 4500,
                    _ => index as u64 * 250,
                });
            let plan = Plan::from_observation(observation.clone()).unwrap();
            let prior = motion.plan().snapshot().clone();
            let saved = motion.scene(at).unwrap();
            assert!(motion
                .advance(plan.clone(), at, &AtomicBool::new(true))
                .is_err());
            assert_eq!(motion.scene(at).unwrap().actors(), saved.actors());
            let candidate = motion.advance(plan.clone(), at, &cancel).unwrap();
            if !assets.supports(&plan) {
                let before = canvas.rgba().to_vec();
                assert!(assets
                    .render_motion(&candidate, at, &mut canvas, &cancel)
                    .is_err());
                assert_eq!(canvas.rgba(), before);
                let fresh =
                    Assets::load_with_catalog(&context, map.clone(), &plan, &cancel).unwrap();
                fresh
                    .render_motion(&candidate, at, &mut canvas, &cancel)
                    .unwrap();
                assets = fresh;
                switches += 1;
            }
            motion = candidate;
            playback = playback
                .advance(plan.clone(), &assets, at, &cancel)
                .unwrap();
            let scene = motion.scene(at).unwrap();
            assert_eq!(
                playback.motion().scene(at).unwrap().actors(),
                scene.actors()
            );
            let moved = scene
                .actors()
                .iter()
                .zip(prior.actors())
                .find(|(a, b)| a.cell != b.cell);
            if let Some((walker, before)) = moved {
                assert_eq!(walker.action, BaseAction::Walk);
                let index_in_roster = scene
                    .actors()
                    .iter()
                    .position(|a| a.entity_id == walker.entity_id)
                    .unwrap();
                let wire = &observation.data().actors.0[index_in_roster];
                let mut audio_branch = playback.clone();
                assert_eq!(wire.step_interval_nanos, 200_000_000);
                let base_walk;
                let q = match plan.forms()[&walker.entity_id] {
                    Form::Polymorph { .. } => {
                        poly.action(BaseAction::Walk).unwrap().walk().unwrap()
                    }
                    Form::Base(base) => {
                        base_walk = QualifiedFixedWalk::from_player_catalog(
                            context.catalog(),
                            base.appearance,
                            base.weapon_type,
                        )
                        .unwrap();
                        &base_walk
                    }
                };
                let selected = scene
                    .actors()
                    .iter()
                    .find(|a| a.entity_id == scene.selected_entity_id())
                    .unwrap();
                if selected.entity_id == walker.entity_id {
                    local += 1;
                } else {
                    remote += 1;
                }
                for ns in [
                    0,
                    49_999_999,
                    50_000_000,
                    99_999_999,
                    100_000_000,
                    149_999_999,
                    150_000_000,
                    199_999_999,
                ] {
                    let phase = Duration::from_nanos(
                        (u128::from(ns) * q.step_duration().as_nanos() / 200_000_000) as u64,
                    );
                    let native = q
                        .start_unpaced_step(walker.heading)
                        .unwrap()
                        .advance(phase, &cancel)
                        .unwrap();
                    let info = assets
                        .render_motion(&motion, at + Duration::from_nanos(ns), &mut canvas, &cancel)
                        .unwrap();
                    let (sampled, sound) = audio_branch
                        .sample_audio(&assets, at + Duration::from_nanos(ns), &cancel)
                        .unwrap();
                    if index < 20 {
                        assert!(
                            sound.as_ref().is_none_or(|f| f.events.is_empty()),
                            "reviewed polymorph walking must not borrow base footsteps"
                        );
                    } else if let Some(frame) = sound {
                        for event in frame.events.iter() {
                            assert!(event
                                .group
                                .candidates()
                                .iter()
                                .all(|id| [0, 478, 479].contains(id)));
                            let bank = event.bank.as_ref().unwrap_or(&frame.bank);
                            assert!(event
                                .group
                                .candidates()
                                .iter()
                                .all(|id| *id == 0 || bank.get(*id).is_some()));
                        }
                    }
                    audio_branch = sampled;
                    let body = info
                        .actors
                        .iter()
                        .find(|a| a.entity_id == walker.entity_id)
                        .unwrap();
                    assert_eq!(body.cell, walker.cell); // Never publish the visual origin as authority.
                    assert_eq!(body.source_frame, native.source_frame);
                    assert_eq!(body.sprite_name, native.asset.file_name());
                    assert_eq!(body.subframe, native.asset.subframe as usize);
                    let anchor = if native.terminal {
                        walker.cell
                    } else {
                        before.cell
                    };
                    assert_eq!(body.anchor, info.projection.cell_to_pixel(anchor).unwrap());
                    if let Form::Polymorph { .. } = plan.forms()[&walker.entity_id] {
                        let frame = poly
                            .frame(BaseAction::Walk, walker.heading, native.source_frame)
                            .unwrap();
                        assert_eq!(
                            body.top_left,
                            PixelPoint {
                                // Native blocks are based at the diamond corner,
                                // while projection anchors are at its centre.
                                x: body.anchor.x + frame.body.frame.x_offset - 24,
                                y: body.anchor.y + frame.body.frame.y_offset - 12
                            }
                        );
                        assert_eq!(
                            native.source_frame,
                            if ns < 50_000_000 {
                                1
                            } else if ns < 100_000_000 {
                                2
                            } else if ns < 150_000_000 {
                                3
                            } else {
                                0
                            }
                        );
                    }
                    if selected.entity_id == walker.entity_id && !native.terminal {
                        let offset = native.local_camera_pulses.iter().fold((0, 0), |(x, y), p| {
                            (x + p.displacement.0, y + p.displacement.1)
                        });
                        assert_eq!(info.projection.anchor_cell, before.cell);
                        assert_eq!(
                            info.projection.anchor_pixel,
                            PixelPoint {
                                x: 400 - offset.0,
                                y: 300 - offset.1
                            }
                        );
                    } else {
                        assert_eq!(info.projection.anchor_cell, selected.cell);
                        assert_eq!(info.projection.anchor_pixel, PixelPoint { x: 400, y: 300 });
                    }
                    assert!(info.npcs.is_empty());
                    samples += 1;
                }
                let settled = motion.scene(at + Duration::from_millis(200)).unwrap();
                assert_eq!(settled.actors()[index_in_roster].action, BaseAction::Idle);
                // Duplicate observations do not replay the initial camera pulse.
                let repeated = motion
                    .advance(plan.clone(), at + Duration::from_millis(20), &cancel)
                    .unwrap();
                let now = at + Duration::from_millis(50);
                assets
                    .render_motion(&motion, now, &mut canvas, &cancel)
                    .unwrap();
                let pixels = canvas.rgba().to_vec();
                assets
                    .render_motion(&repeated, now, &mut canvas, &cancel)
                    .unwrap();
                assert_eq!(canvas.rgba(), pixels);
                // A fresh attachment sees the destination, never retained movement.
                assert!(Motion::new(plan, at)
                    .unwrap()
                    .scene(at)
                    .unwrap()
                    .actors()
                    .iter()
                    .all(|a| a.action == BaseAction::Idle));
            } else if index == 19 {
                // Only graphics changed; retained base weapon is still zero.
                assert_eq!(prior.actors()[1].weapon_type, scene.actors()[1].weapon_type);
                assert_eq!(scene.actors()[1].action, BaseAction::Idle);
            } else {
                assert_eq!(index, 17);
            }
            let before = canvas.rgba().to_vec();
            assert!(assets
                .render_motion(&motion, at, &mut canvas, &AtomicBool::new(true))
                .is_err());
            assert_eq!(canvas.rgba(), before);
            assert!(motion.scene(start).is_err());
        }
        let other = &histories[if observation_source(trace) == 7 {
            "walk-42"
        } else {
            "walk-7"
        }][20];
        assert!(motion
            .advance(
                Plan::from_observation(other.clone()).unwrap(),
                start + Duration::from_secs(6),
                &cancel
            )
            .is_err());
    }
    assert_eq!((samples, local, remote, switches), (288, 18, 18, 2));
    println!("PLAYER_WALK:44 Java frames;dual sources/same3333;36 confirmed steps/288 native pixel/audio samples;18 local/18 remote;8 headings;denial/duplicates/skipped history/form change/cancel/source switch;2 atomic resource replacements;polymorph walks do not borrow base footsteps;no full VIP LOAD/UI claim");
}
fn observation_source(trace: &[crate::modern_lito::battle_follow::Observation]) -> i32 {
    trace[0].data().personal.player.source
}
