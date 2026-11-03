//! Genuine Java skill4/70/77 receipts and shared logical5641 pixels/effects.
//! Synthetic map geometry; no player HWND, SQL/TLS or full VIP LOAD claim.
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
#[ignore = "actual Java schema11 casts and shared SPR/Tile/WAV; no GUI/device/SQL/TLS/full VIP LOAD"]
fn java_public_polymorph_casts_use_logical_clock_and_atomic_spell_presentation() {
    let mut peer = Peer::start_named(Some("vip-appearance-cast"));
    let hello = peer.line();
    let h: Vec<_> = hello.splitn(4, ':').collect();
    assert_eq!(&h[..2], &["SESSION", "240"]);
    let protocol::Message::ExtendedHello(hello, offer) = protocol::decode(&unhex(h[2])).unwrap()
    else {
        panic!("hello")
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
            assert_eq!(line, "CLOSED:30");
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
        let observation = f.observation().unwrap().clone();
        assert!(crate::modern_lito::battle_scene::baseline_counts(observation.clone()).is_err());
        histories
            .entry(v[1].to_owned())
            .or_default()
            .push(observation);
        peer.ack(&bytes);
        count += 1;
    }
    peer.finish();
    assert_eq!(count, 30);
    assert_eq!(histories.len(), 6);
    let cancel = AtomicBool::new(false);
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
    let mut pixels = 0;
    let mut heard_casts = 0;
    for (name, trace) in histories {
        assert_eq!(trace.len(), 5);
        let first = Plan::from_observation(trace[0].clone()).unwrap();
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
        let assets = Assets::load_with_catalog(&catalog, map.clone(), &first, &cancel).unwrap();
        let origin = Instant::now();
        let at = origin + Duration::from_millis(10);
        let baseline = Playback::new(first, origin).unwrap();
        let plan = Plan::from_observation(trace[1].clone()).unwrap();
        let live = baseline
            .advance(plan.clone(), &assets, at, &cancel)
            .unwrap();
        assert_eq!(trace[1].data(), trace[2].data());
        let mut canvas = Canvas::new(800, 600).unwrap();
        let mut branch = live.clone();
        let mut heard = BTreeMap::new();
        let points = [
            0,
            40_999_999,
            41_000_000,
            103_000_000,
            186_000_000,
            205_999_999,
            206_000_000,
            206_000_001,
            331_000_000,
            413_999_999,
            414_000_000,
            500_000_000,
            997_999_999,
            998_000_000,
            1_500_000_000,
            3_000_000_000,
        ];
        for ns in points {
            let now = at + Duration::from_nanos(ns);
            let saved = canvas.rgba().to_vec();
            assert!(assets
                .render_playback(&branch, now, &mut canvas, &AtomicBool::new(true))
                .is_err());
            assert_eq!(canvas.rgba(), saved);
            let (sampled, info, audio) = assets
                .render_playback(&branch, now, &mut canvas, &cancel)
                .unwrap();
            let painted = canvas.rgba().to_vec();
            let signature = sounds(&audio);
            let (repeat, again, a2) = assets
                .render_playback(&sampled, now, &mut canvas, &cancel)
                .unwrap();
            assert_eq!(canvas.rgba(), painted);
            assert_eq!(signature, sounds(&a2));
            assert_eq!(info.actors.len(), again.actors.len());
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
            assert_eq!(canvas.rgba(), painted);
            assert_eq!(signature, sounds(&a3));
            for actor in &info.actors {
                assert!(actor.entity_id == 1 || actor.entity_id == 2);
                assert_eq!(
                    actor.cell,
                    plan.snapshot()
                        .actors()
                        .iter()
                        .find(|a| a.entity_id == actor.entity_id)
                        .unwrap()
                        .cell
                );
            }
            let caster = info.actors.iter().find(|a| a.entity_id == 1).unwrap();
            if ns < 998_000_000 {
                let expected = if ns < 41_000_000 {
                    1
                } else if ns < 103_000_000 {
                    2
                } else if ns < 186_000_000 {
                    3
                } else if ns < 206_000_000 {
                    4
                } else if ns < 331_000_000 {
                    5
                } else if ns < 414_000_000 {
                    6
                } else {
                    0
                };
                assert_eq!(caster.graphics_id, 240);
                assert_eq!(caster.source_frame, expected, "{name}/{ns}");
                assert!(caster.sprite_name.starts_with("240-3"));
            } else {
                assert!(!caster.sprite_name.starts_with("240-3"));
                if ns == 998_000_000 {
                    assert_eq!(
                        caster.source_frame, 1,
                        "idle must start, not reveal a continuously running clock"
                    );
                }
            }
            for (seq, created, ids) in signature {
                assert!(created <= now);
                heard.insert(seq, (created, ids));
            }
            branch = same;
            pixels += 1;
        }
        assert!(
            !heard.is_empty(),
            "spell effects must supply actual PCM: {name}"
        );
        if name.starts_with("cast4-") {
            assert!(
                heard.values().any(
                    |(when, ids)| *when == at + Duration::from_millis(206) && ids.contains(&97)
                ),
                "logical launch, not bank24FPS"
            );
        }
        heard_casts += 1;
        // An initial/reconnected scene must not replay retained cast history.
        let fresh = Playback::new(plan.clone(), at).unwrap();
        let (_, info, a) = assets
            .render_playback(
                &fresh,
                at + Duration::from_millis(206),
                &mut canvas,
                &cancel,
            )
            .unwrap();
        assert!(sounds(&a).is_empty());
        assert!(!info
            .actors
            .iter()
            .find(|a| a.entity_id == 1)
            .unwrap()
            .sprite_name
            .starts_with("240-3"));
        // Equip transaction changes the public shape before visual launch.
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
        let (_, info, a) = replacement
            .render_playback(
                &switched,
                at + Duration::from_millis(206),
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
        // The target's confirmed hurt cue remains valid. Only the cancelled
        // caster/effect is withdrawn; do not silence another actor's audio.
        assert!(sounds(&a)
            .iter()
            .all(|(_, when, ids)| *when == at && ids.as_slice() == [88]));
        assert!(Plan::from_observation(trace[4].clone()).is_err());
        assert!(assets
            .render_playback(&branch, at, &mut canvas, &cancel)
            .is_err());
    }
    assert_eq!(heard_casts, 6);
    assert_eq!(pixels, 96);
    eprintln!("PLAYER_CAST:30 actual Java frames;6 dual-source skill4/70/77 streams;96 shared render samples; caster48FPS/launch206ms/terminal414ms/fixed584ms tail; source ownership, effects/PCM, duplicate/history/cancel/shape change/abort; no full VIP LOAD/UI");
}
