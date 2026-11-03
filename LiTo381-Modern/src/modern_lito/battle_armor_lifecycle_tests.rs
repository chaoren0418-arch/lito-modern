//! Actual Java gear/life transitions, complete follower + real shared scene.
//! Synthetic geometry/source rules are explicit; this is not SQL/TLS/HWND QA.
use super::*;
use crate::modern_lito::{battle_armor_effects, battle_scene};
use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

#[test]
#[ignore = "actual Java engine/durable gear, real shared resources, synthetic geometry; not native UI or SQL"]
fn java_armor_shared_scene_survives_durable_gear_death_respawn_and_finish() {
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let mut peer = Peer::start_named(Some("armor-lifecycle"));
    let mut followers = BTreeMap::new();
    let mut frames = Vec::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, format!("CLOSED:{}", frames.len()));
            break;
        }
        let f = line.splitn(5, ':').collect::<Vec<_>>();
        assert_eq!(f[0], "REPLY");
        let (q, b) = (unhex(f[3]), unhex(f[4]));
        assert_eq!(f[2], "15");
        let data = decode(&b, &q, selected(15)).unwrap();
        let key = (
            f[1].split('-').next().unwrap().to_owned(),
            data.personal.entity,
        );
        let follower = followers
            .entry(key)
            .or_insert_with(|| Follower::new_selected(request(&data), selected(15)).unwrap());
        follower
            .accept(reply(data))
            .unwrap_or_else(|e| panic!("{}: {e:?}", f[1]));
        frames.push((f[1].to_owned(), follower.observation().unwrap().clone()));
        peer.ack(&b);
    }
    peer.finish();
    assert!(frames.len() >= 38);
    battle_armor_effects::tests::verify_lifecycle(&root, &frames);
    let cancel = AtomicBool::new(false);
    for (prefix, owner) in [("gear-", "-0"), ("life-", "-1")] {
        let trace = frames
            .iter()
            .filter(|(name, _)| name.starts_with(prefix) && name.ends_with(owner))
            .collect::<Vec<_>>();
        let mut scene = battle_scene::tests::trace_observed_scene(&root, trace[0].1.clone());
        let origin = Instant::now() + Duration::from_secs(60);
        let mut at = origin;
        let mut last_armor_sequence = 0;
        for (name, o) in trace {
            let target = at.max(
                origin + Duration::from_nanos(o.data().world.active_elapsed_nanos.max(0) as u64),
            );
            // The real UI paints between network publications. In particular,
            // do not manufacture a five-second stopped-render interval while
            // waiting for Honor revival, then blame the late-arrow guard.
            while at + Duration::from_millis(100) < target {
                at += Duration::from_millis(100);
                let intermediate = scene.render(at, &cancel).unwrap();
                assert!(
                    !intermediate.effects_skipped,
                    "continuous render before {name}"
                );
            }
            at = target;
            scene
                .observe_at(o.clone(), at, &cancel)
                .unwrap_or_else(|e| panic!("observe {name}: {e:#}"));
            if name == "gear-abort-0" {
                assert!(scene.prepared_resources().is_none());
                assert!(scene.render(at, &cancel).is_err());
                continue;
            }
            let a = scene
                .render(at, &cancel)
                .unwrap_or_else(|e| panic!("render {name}: {e:#}"));
            let b = scene.render(at, &cancel).unwrap();
            assert!(!a.effects_skipped, "{name}");
            assert_eq!(
                a.observation.data(),
                o.data(),
                "no authoritative projection at {name}"
            );
            assert_eq!(a.bgra, b.bgra, "same instant {name}");
            let cues = |s: &battle_scene::Surface| {
                s.audio.as_ref().map_or(Vec::new(), |a| {
                    a.events
                        .iter()
                        .filter(|e| e.group.candidates() == [132])
                        .map(|e| e.sequence)
                        .collect::<Vec<_>>()
                })
            };
            assert_eq!(cues(&a), cues(&b), "repaint cannot replay {name}");
            if ["gear-hit-0", "life-hit-1", "gear-again-0", "life-again-1"].contains(&name.as_str())
            {
                let ids = cues(&a);
                assert_eq!(ids.len(), 1, "one fresh skill cue {name}");
                assert!(
                    ids[0] > last_armor_sequence,
                    "monotonic across life/gear {name}"
                );
                last_armor_sequence = ids[0];
            }
            if name == "gear-off-0" || name == "gear-heal-0" {
                assert_eq!(
                    cues(&a),
                    vec![last_armor_sequence],
                    "gear/heal must not restart proc"
                );
            }
            if name == "gear-heal-0" {
                assert!(a
                    .audio
                    .as_ref()
                    .unwrap()
                    .events
                    .iter()
                    .any(|e| e.group.candidates() == [109]));
            }
            if [
                "life-dead-1",
                "life-revived-1",
                "life-finished-1",
                "gear-expired-0",
            ]
            .contains(&name.as_str())
            {
                assert!(cues(&a).is_empty(), "no stale armor cue {name}");
            }
            if name == "gear-off-0" || name == "life-revived-1" {
                let mut replacement = battle_scene::tests::trace_observed_scene(&root, o.clone());
                replacement.carry_from(&mut scene, at, &cancel).unwrap();
                assert!(scene.render(at, &cancel).is_err());
                let carried = replacement.render(at, &cancel).unwrap();
                assert_eq!(a.bgra, carried.bgra, "resource handoff {name}");
                assert_eq!(cues(&a), cues(&carried));
                scene = replacement;
            }
        }
    }
    eprintln!("ARMOR_LIFECYCLE_SHARED: {} real Java schema9 frames, durable unequip/heal/re-equip/expiry, Honor death/respawn/new proc/finish, both-source atomic followers, actual shared pixels/cues/status and resource handoff; synthetic terrain/source, no SQL/TLS/HWND claim", frames.len());
}
