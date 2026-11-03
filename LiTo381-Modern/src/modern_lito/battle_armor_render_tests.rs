//! Real Java schema9, real SPR/WAV/icon banks; synthetic Java terrain is labelled.
use super::*;
use crate::modern_lito::{battle_armor_effects, battle_scene};

#[test]
#[ignore = "actual Java engine schema9 and shared228/132/1562; complete headless scene with explicit synthetic geometry, not installed SQL/UI"]
fn java_armor_schema9_shared_effects_render_order_anchor_audio_and_handoff() {
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let mut peer = Peer::start_named(Some("armor-render"));
    let mut followers = BTreeMap::new();
    let mut frames = BTreeMap::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:36");
            break;
        }
        let f = line.splitn(5, ':').collect::<Vec<_>>();
        assert_eq!(f[0], "REPLY");
        let (q, b) = (unhex(f[3]), unhex(f[4]));
        let data = decode(&b, &q, selected(12)).unwrap();
        let key = (
            f[1].split('-').next().unwrap().to_owned(),
            data.personal.entity,
        );
        let follower = followers
            .entry(key)
            .or_insert_with(|| Follower::new_selected(request(&data), selected(12)).unwrap());
        follower.accept(reply(data)).unwrap();
        frames.insert(f[1].to_owned(), follower.observation().unwrap().clone());
        peer.ack(&b);
    }
    peer.finish();
    assert_eq!(frames.len(), 36);
    battle_armor_effects::tests::verify_trace(&root, &frames);
    for graphic in [0, 228] {
        let get = |stage: &str| frames[&format!("gfx{graphic}-{stage}-1")].clone();
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let mut scene = battle_scene::tests::trace_observed_scene(&root, get("loading"));
        assert!(scene.prepared_resources().is_none());
        let now = std::time::Instant::now() + std::time::Duration::from_secs(60);
        let first = scene.render(now, &cancel).unwrap();
        assert!(!first.effects_skipped);
        assert!(scene.prepared_resources().unwrap().matches(&get("loading")));
        for stage in ["active", "hit", "moved", "back", "again", "expired"] {
            let o = get(stage);
            let at =
                now + std::time::Duration::from_nanos(o.data().world.active_elapsed_nanos as u64);
            scene.observe_at(o.clone(), at, &cancel).unwrap();
            let a = scene.render(at, &cancel).unwrap();
            let b = scene.render(at, &cancel).unwrap();
            assert_eq!(a.observation.data(), o.data());
            assert_eq!(a.bgra, b.bgra, "duplicate pixels {stage}");
            assert!(!a.effects_skipped, "complete {stage}");
            let armor_cues = |s: &battle_scene::Surface| {
                s.audio.as_ref().map_or(Vec::new(), |a| {
                    a.events
                        .iter()
                        .filter(|e| e.group.candidates() == [132])
                        .map(|e| e.sequence)
                        .collect::<Vec<_>>()
                })
            };
            assert_eq!(armor_cues(&a), armor_cues(&b));
            if stage == "hit" {
                assert_eq!(armor_cues(&a).len(), if graphic == 0 { 1 } else { 5 });
                let mut replacement = battle_scene::tests::trace_observed_scene(&root, o.clone());
                replacement.carry_from(&mut scene, at, &cancel).unwrap();
                assert!(scene.render(at, &cancel).is_err());
                let carried = replacement.render(at, &cancel).unwrap();
                assert_eq!(a.bgra, carried.bgra);
                assert_eq!(armor_cues(&a), armor_cues(&carried));
                scene = replacement;
            }
        }
        scene
            .observe_at(
                get("abort"),
                now + std::time::Duration::from_secs(4),
                &cancel,
            )
            .unwrap();
        assert!(scene.prepared_resources().is_none());
        assert!(scene
            .render(now + std::time::Duration::from_secs(4), &cancel)
            .is_err());
    }
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let mut maximum =
        battle_scene::tests::trace_observed_scene(&root, frames["max-active-1"].clone());
    let now = std::time::Instant::now() + std::time::Duration::from_secs(60);
    maximum
        .observe_at(frames["max-hit-1"].clone(), now, &cancel)
        .unwrap();
    let limited = maximum.render(now, &cancel).unwrap();
    assert!(
        limited.effects_skipped,
        "real scene must expose the bounded 2049-cue loss to LOAD guard and presentation warning"
    );
    assert!(crate::modern_lito::battle_window::presentation_notice(&limited).is_some());
    assert_eq!(limited.observation.data(), frames["max-hit-1"].data());
    eprintln!("ARMOR_SHARED_RENDER:36 actual Java schema9 frames; explicit gfx0/gfx228 fixtures, real228/132/1562, native ordered multiplicity/frozen cell/dedup/cancel/age/capacity/abort/handoff; full headless scene/max-batch loss signal with synthetic geometry, not installed-map TLS or HWND");
}
