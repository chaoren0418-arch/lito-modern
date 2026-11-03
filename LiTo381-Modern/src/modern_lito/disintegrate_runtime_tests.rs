//! Production timeline/compositor tests with frozen Java authority receipts.
use super::*;
use crate::modern_lito::disintegrate_presentation::tests::{frame, reply};
fn input(f: &combat_protocol::Frame, at: Instant) -> Arc<Input> {
    let mut input =
        Input::with_combat(Arc::new(f.world.clone()), 1, Some(Arc::new(f.clone()))).unwrap();
    input.received = at;
    Arc::new(input)
}
#[test]
fn disintegrate_timeline_new_cast_clears_old_body_and_preserves_target_death() {
    let now = Instant::now();
    for (a, b) in [
        ("normal-active", "normal-hit"),
        ("mixed-fire", "mixed-second"),
        ("terminal-active", "terminal-finished"),
    ] {
        let before = frame(a);
        let after = frame(b);
        assert!(combat_protocol::follows(Some(&reply(a)), &reply(b)));
        let mut timeline = Timeline::new(input(&before, now));
        let at = now + Duration::from_millis(100);
        timeline.update(input(&after, at)).unwrap();
        assert!(timeline.actors[&1].pose.is_none());
        assert!(timeline.actors[&1].walk_until.is_none());
        assert_eq!(timeline.actors[&1].action_started, at);
        let scene = timeline.snapshot(at).unwrap();
        assert_eq!(scene.actors()[0].action, BaseAction::Idle);
        if b == "terminal-finished" {
            assert_eq!(
                scene
                    .actors()
                    .iter()
                    .find(|a| a.entity_id == 2)
                    .unwrap()
                    .action,
                BaseAction::Death
            );
        }
        let frozen = timeline.input.combat.clone().unwrap();
        timeline
            .update(input(&after, at + Duration::from_secs(3)))
            .unwrap();
        timeline.snapshot(at + Duration::from_secs(3)).unwrap();
        assert_eq!(timeline.actors[&1].action_started, at);
        assert_eq!(timeline.input.combat.clone().unwrap(), frozen);
    }
}

fn sound717(p: &Pixels) -> Vec<u64> {
    p.audio
        .as_ref()
        .unwrap()
        .events
        .iter()
        .filter(|e| e.group.candidates() == [717])
        .map(|e| e.sequence)
        .collect()
}
pub(super) fn capture(label: &str, pixels: &Pixels) {
    if let Some(dir) = std::env::var_os("LITO_MODERN_DISINTEGRATE_CAPTURE_DIR") {
        let dir = PathBuf::from(dir).canonicalize().unwrap();
        assert!(dir.starts_with(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .canonicalize()
                .unwrap()
        ));
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(format!("runtime-{label}.png")))
            .unwrap();
        let mut rgba = pixels.bgra.to_vec();
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        let mut encoder = png::Encoder::new(file, WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&rgba)
            .unwrap();
    }
}

#[test]
#[ignore = "shared Lineage caster18/11746-49/717 and actual compositor; frozen Java authority on synthetic S32, not HWND/device"]
fn actual_disintegrate_runtime_normal_critical_kill_dwell_and_audio_are_receipt_driven() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
    let cancel = AtomicBool::new(false);
    for (prefix, end) in [
        ("normal", "hit"),
        ("critical", "hit"),
        ("terminal", "finished"),
    ] {
        let before = frame(&format!("{prefix}-active"));
        let after = frame(&format!("{prefix}-{end}"));
        let frozen = after.clone();
        // Same synthetic open S32 as the frozen Java engine fixture; only the
        // authority/pixels/SPR/audio integration is claimed here, not real532.
        let map = crate::modern_lito::siege_space::tests::map(&before.world);
        assert_eq!(map.canonical_digest(), before.world.manifest.digest);
        let mut renderer =
            load_qualified_map_renderer(&root, &before.world, Some(&before), None, &cancel, map)
                .unwrap();
        let t = Instant::now();
        let mut timeline = Timeline::new(input(&before, t));
        let p = renderer
            .render_at(&timeline.snapshot(t).unwrap(), Some(&before), &cancel, t)
            .unwrap();
        assert!(sound717(&p).is_empty());
        let cast_at = t + Duration::from_millis(50);
        timeline.update(input(&after, cast_at)).unwrap();
        let scene = timeline.snapshot(cast_at).unwrap();
        let appearance = scene
            .actors()
            .iter()
            .find(|a| a.entity_id == 1)
            .unwrap()
            .appearance;
        let delay = renderer
            .light_arrow
            .as_ref()
            .unwrap()
            .assets
            .cast_launch_delay(appearance)
            .unwrap();
        assert!(delay > Duration::ZERO);
        renderer
            .render_at(&scene, Some(&after), &cancel, cast_at)
            .unwrap();
        assert!(renderer
            .disintegrate
            .as_ref()
            .unwrap()
            .draws(cast_at)
            .unwrap()
            .is_empty());
        let launch_at = cast_at + delay;
        let scene = timeline.snapshot(launch_at).unwrap();
        let first = renderer
            .render_at(&scene, Some(&after), &cancel, launch_at)
            .unwrap();
        assert_eq!(sound717(&first).len(), 1);
        let repeat = renderer
            .render_at(&scene, Some(&after), &cancel, launch_at)
            .unwrap();
        assert_eq!(first.bgra, repeat.bgra);
        assert_eq!(sound717(&first), sound717(&repeat));
        let at = launch_at + Duration::from_millis(125);
        let scene = timeline.snapshot(at).unwrap();
        let visible = renderer
            .render_at(&scene, Some(&after), &cancel, at)
            .unwrap();
        capture(prefix, &visible);
        let dis = renderer.disintegrate.as_ref().unwrap();
        let effects = dis.draws(at).unwrap();
        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0].binding().target,
            after.disintegrations[0].hit.as_ref().unwrap().to
        );
        let overrides = dis
            .overrides(&renderer.light_arrow.as_ref().unwrap().assets, at)
            .unwrap();
        let roof = renderer
            .assets
            .sample_roofs(&mut renderer.roofs, &scene, at)
            .unwrap();
        let mut without = Canvas::new(WIDTH, HEIGHT).unwrap();
        let info = renderer
            .assets
            .render_into_npcs_roofs_effects(
                &scene,
                &mut without,
                &cancel,
                &overrides,
                &[],
                &[],
                Some(&roof),
                &[],
            )
            .unwrap();
        draw_health_bars(&mut without, &info.actors, &after.vitals).unwrap();
        let mut without_bgra = Vec::new();
        without.copy_bgra(&mut without_bgra).unwrap();
        assert_ne!(
            visible.bgra.as_ref(),
            without_bgra.as_slice(),
            "same body/death/HP/camera/time, only skill77 effect omitted"
        );
        assert_eq!(visible.actors, info.actors);
        // Keep source0's terminal dwell, then unlink exactly at 2500ms.
        for ns in [2_375_000_000, 2_499_999_999] {
            let at = launch_at + Duration::from_nanos(ns);
            let p = renderer
                .render_at(&timeline.snapshot(at).unwrap(), Some(&after), &cancel, at)
                .unwrap();
            assert_eq!(
                renderer
                    .disintegrate
                    .as_ref()
                    .unwrap()
                    .draws(at)
                    .unwrap()
                    .len(),
                1
            );
            assert!(sound717(&p).is_empty());
        }
        let at = launch_at + Duration::from_millis(2500);
        let p = renderer
            .render_at(&timeline.snapshot(at).unwrap(), Some(&after), &cancel, at)
            .unwrap();
        assert!(renderer
            .disintegrate
            .as_ref()
            .unwrap()
            .draws(at)
            .unwrap()
            .is_empty());
        assert!(renderer
            .disintegrate
            .as_ref()
            .unwrap()
            .overrides(&renderer.light_arrow.as_ref().unwrap().assets, at)
            .unwrap()
            .is_empty());
        assert!(sound717(&p).is_empty());
        assert_eq!(after, frozen);
        let p2 = renderer
            .render_at(&timeline.snapshot(at).unwrap(), Some(&after), &cancel, at)
            .unwrap();
        assert_eq!(p.bgra, p2.bgra);
    }
}
