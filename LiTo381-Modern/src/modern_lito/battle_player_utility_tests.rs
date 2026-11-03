//! Real schema11 utility casts and shared resources, synthetic map only.
use super::*;
use crate::modern_lito::{
    compositor::Canvas,
    player_appearance::{Assets, Plan, Playback},
    resources::SharedMapSectorKey,
    shared_action_catalog::SharedActionCatalog,
    world_map::{SectorInput, WorldMap},
};
use sha2::{Digest, Sha256};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};
fn sounds(
    frame: &Option<crate::modern_lito::world_audio::AudioFrame>,
) -> Vec<(u64, Instant, Vec<u32>)> {
    frame.as_ref().map_or_else(Vec::new, |a| {
        a.events
            .iter()
            .map(|e| {
                let bank = e.bank.as_ref().unwrap_or(&a.bank);
                for id in e.group.candidates().iter().filter(|id| **id != 0) {
                    assert!(!bank.get(*id).unwrap().pcm_bytes().is_empty());
                }
                (e.sequence, e.created, e.group.candidates().to_vec())
            })
            .collect()
    })
}
#[test]
#[ignore = "actual Java schema11 utility receipts and shared SPR/Tile/WAV; no UI/device/SQL/TLS/full VIP LOAD"]
fn java_public_utility_spells_compose_polymorph_body_effects_and_allow_post_teleport_move() {
    let mut peer = Peer::start_named(Some("vip-appearance-utility"));
    let line = peer.line();
    let h: Vec<_> = line.splitn(4, ':').collect();
    assert_eq!(&h[..2], &["SESSION", "241"]);
    let protocol::Message::ExtendedHello(hello, offer) = protocol::decode(&unhex(h[2])).unwrap()
    else {
        panic!("extended hello")
    };
    let mut session = protocol::ModernClientSession::new_extended(hello, offer).unwrap();
    session.begin().unwrap();
    let ack = session.accept_ack(&unhex(h[3])).unwrap();
    let p = protocol::extensions::Selected::new(ack, session.extensions().unwrap()).unwrap();
    let mut followers = BTreeMap::new();
    let mut histories: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut count = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:60");
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
        peer.ack(&bytes);
        count += 1;
    }
    peer.finish();
    assert_eq!((count, histories.len()), (60, 10));
    let cancel = AtomicBool::new(false);
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
    let mut samples = 0;
    let mut teleports = 0;
    let mut audible = 0;
    for (name, trace) in histories {
        assert_eq!(trace.len(), 6);
        assert_eq!(trace[1].data(), trace[2].data());
        let skill: i32 = name
            .split('-')
            .next()
            .unwrap()
            .trim_start_matches("utility")
            .parse()
            .unwrap();
        let before = Plan::from_observation(trace[0].clone()).unwrap();
        let m = &trace[0].data().manifest;
        assert_eq!(m.sectors.0.len(), 1);
        let s = &m.sectors.0[0];
        let bytes = vec![0; 49170];
        assert_eq!(s.sha256, <[u8; 32]>::from(Sha256::digest(&bytes)));
        let map = Arc::new(
            WorldMap::from_sector_bytes(
                vec![SectorInput {
                    key: SharedMapSectorKey {
                        map_id: m.map.try_into().unwrap(),
                        sector_x: s.x.try_into().unwrap(),
                        sector_y: s.y.try_into().unwrap(),
                    },
                    bytes,
                }],
                &cancel,
            )
            .unwrap(),
        );
        let assets = Assets::load_with_catalog(&catalog, map.clone(), &before, &cancel).unwrap();
        let origin = Instant::now();
        let at = origin + Duration::from_secs(1);
        let plan = Plan::from_observation(trace[1].clone()).unwrap();
        let baseline = Playback::new(before.clone(), origin).unwrap();
        let live = baseline
            .advance(plan.clone(), &assets, at, &cancel)
            .unwrap();
        let caster = &trace[1].data().actors.0[0];
        let base = caster.cast_interval_nanos as u128;
        let effective = if skill == 54 {
            ((base / 1_000_000) as f64 * 0.755) as u128 * 1_000_000
        } else {
            base
        };
        assert_eq!(
            crate::modern_lito::battle_motion::paced_nanos(caster, base as u64, false) as u128,
            effective
        );
        let boundary = |phase: u128| -> u64 { (phase * effective).div_ceil(base) as u64 };
        let terminal = boundary(395_000_000);
        let idle = terminal + 584_000_000;
        let mut points = vec![0, idle - 4, idle, idle + 1, 1_500_000_000];
        for phase in [104_000_000, 208_000_000, 312_000_000, 395_000_000] {
            let t = boundary(phase);
            points.extend([t - 4, t + 4]);
        }
        points.sort();
        points.dedup();
        let mut canvas = Canvas::new(800, 600).unwrap();
        let mut branch = live.clone();
        let mut heard = BTreeMap::new();
        for ns in points {
            let now = at + Duration::from_nanos(ns);
            let prior = canvas.rgba().to_vec();
            assert!(assets
                .render_playback(&branch, now, &mut canvas, &AtomicBool::new(true))
                .is_err());
            assert_eq!(prior, canvas.rgba());
            let (next, info, a) = assets
                .render_playback(&branch, now, &mut canvas, &cancel)
                .unwrap();
            let painted = canvas.rgba().to_vec();
            // Optional CPU-rendered evidence, never a screenshot/GUI claim.
            if matches!(name.as_str(), "utility5-7" | "utility54-7")
                && matches!(ns, 0 | 1_500_000_000)
            {
                if let Some(dir) = std::env::var_os("LITO_MODERN_UTILITY_EVIDENCE") {
                    let path = std::path::PathBuf::from(dir).join(format!("{name}-{ns}.png"));
                    let file = std::fs::File::options()
                        .write(true)
                        .create_new(true)
                        .open(path)
                        .unwrap();
                    let mut encoder = png::Encoder::new(file, canvas.width(), canvas.height());
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    encoder
                        .write_header()
                        .unwrap()
                        .write_image_data(&painted)
                        .unwrap();
                }
            }
            let signature = sounds(&a);
            let (repeat, _, again) = assets
                .render_playback(&next, now, &mut canvas, &cancel)
                .unwrap();
            assert_eq!(painted, canvas.rgba());
            assert_eq!(signature, sounds(&again));
            let duplicate = repeat
                .advance(
                    Plan::from_observation(trace[2].clone()).unwrap(),
                    &assets,
                    now,
                    &cancel,
                )
                .unwrap();
            let (same, _, a3) = assets
                .render_playback(&duplicate, now, &mut canvas, &cancel)
                .unwrap();
            assert_eq!(painted, canvas.rgba());
            assert_eq!(signature, sounds(&a3));
            let draw = info.actors.iter().find(|a| a.entity_id == 1).unwrap();
            assert_eq!(
                (draw.cell.x, draw.cell.y),
                (caster.position.x, caster.position.y)
            );
            if ns < idle {
                let phase = ns as u128 * base / effective;
                let expected = if phase < 104_000_000 {
                    1
                } else if phase < 208_000_000 {
                    2
                } else if phase < 312_000_000 {
                    3
                } else if phase < 395_000_000 {
                    4
                } else {
                    0
                };
                assert_eq!(draw.graphics_id, 240);
                assert_eq!(draw.source_frame, expected, "{name}/{ns}");
                assert!(
                    draw.sprite_name.starts_with("240-4"),
                    "{name}: {}",
                    draw.sprite_name
                );
            } else {
                assert!(
                    !draw.sprite_name.starts_with("240-4"),
                    "{name}/{ns} expected idle at {idle}, got {}",
                    draw.sprite_name
                );
                if ns == idle {
                    assert_eq!(draw.source_frame, 1, "idle must start at its own origin");
                }
            }
            for (seq, when, ids) in signature {
                heard.insert(seq, (when, ids));
            }
            branch = same;
            samples += 1;
        }
        if skill == 5 {
            teleports += 1;
            assert_ne!(trace[0].data().actors.0[0].position, caster.position);
            assert!(heard
                .values()
                .any(|(when, ids)| *when == at && ids.contains(&119)));
            audible += 1;
        } else if skill == 1 {
            assert!(!heard.is_empty());
            audible += 1;
        } else {
            assert!(
                heard.is_empty(),
                "haste/slow bodies and effects have no invented audio"
            );
        }
        let fresh = Playback::new(plan.clone(), at).unwrap();
        let (_, fresh_info, a) = assets
            .render_playback(
                &fresh,
                at + Duration::from_millis(104),
                &mut canvas,
                &cancel,
            )
            .unwrap();
        assert!(sounds(&a).is_empty());
        assert!(!fresh_info
            .actors
            .iter()
            .find(|a| a.entity_id == 1)
            .unwrap()
            .sprite_name
            .starts_with("240-4"));
        let changed = Plan::from_observation(trace[3].clone()).unwrap();
        let replacement = Assets::load_with_catalog(&catalog, map, &changed, &cancel).unwrap();
        assert!(live
            .advance(
                changed.clone(),
                &assets,
                at + Duration::from_millis(10),
                &cancel
            )
            .is_err());
        let switched = live
            .advance(
                changed,
                &replacement,
                at + Duration::from_millis(10),
                &cancel,
            )
            .unwrap();
        let (_, info, _) = replacement
            .render_playback(
                &switched,
                at + Duration::from_millis(100),
                &mut canvas,
                &cancel,
            )
            .unwrap();
        assert_ne!(
            info.actors
                .iter()
                .find(|a| a.entity_id == 1)
                .unwrap()
                .graphics_id,
            240
        );
        let moving = Plan::from_observation(trace[4].clone()).unwrap();
        let moved = switched
            .advance(
                moving.clone(),
                &replacement,
                at + Duration::from_secs(3),
                &cancel,
            )
            .unwrap();
        let (_, info, _) = replacement
            .render_playback(&moved, at + Duration::from_secs(3), &mut canvas, &cancel)
            .unwrap();
        let draw = info.actors.iter().find(|a| a.entity_id == 1).unwrap();
        assert_eq!(draw.cell, moving.snapshot().actors()[0].cell);
        assert_ne!(draw.cell, plan.snapshot().actors()[0].cell);
        assert!(assets
            .render_playback(&branch, at, &mut canvas, &cancel)
            .is_err());
        assert!(Plan::from_observation(trace[5].clone()).is_err());
        assert_eq!(
            live.motion().plan().observation().data(),
            trace[1].data(),
            "presentation never mutates authority"
        );
    }
    assert_eq!((teleports, audible), (2, 4));
    eprintln!("PLAYER_UTILITY:60 real Java frames/10 dual-source streams;{samples} shared pixel/audio samples;5/1/43/54/29 action19 logical48FPS, paced caster/fixed584ms tail;119/healing PCM;duplicates/cancel/history/shape change/post-teleport MOVE/abort;synthetic map,no full VIP LOAD/UI");
}
