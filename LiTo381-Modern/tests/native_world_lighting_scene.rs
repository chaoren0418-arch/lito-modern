// Lifetime assertions share the actual immutable source root. Serialize all
// tests in this executable so another test cannot keep a valid owner alive.
static SOURCE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn source_test_guard() -> std::sync::MutexGuard<'static, ()> {
    SOURCE_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
// Actual resource-backed Model and asynchronous Presentation, no test server.
use lito_modern::modern_lito::{
    native_environment::State, native_render_environment::Bound, native_world::World,
    native_world_scene::Model, native_world_view::Presentation,
};
use std::{
    path::Path,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
fn root() -> &'static Path {
    Path::new(r"D:\lineage開發\0-客戶端")
}
fn current() -> (World, State) {
    let f: serde_json::Value = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/native-world-current.json"
    )))
    .unwrap();
    let mut world = World::default();
    world.relations.begin(2).unwrap();
    for key in ["world", "start", "status", "actor", "bot", "hp", "mp"] {
        let text = f["packets"][key].as_str().unwrap();
        let bytes: Vec<_> = (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect();
        assert!(world.accept(&bytes).unwrap());
    }
    assert!(world.view_ready());
    let mut env = State::default();
    env.begin(2).unwrap();
    env.map_changed(2, world.map_id.unwrap(), Some(0)).unwrap();
    (world, env)
}
#[test]
fn real_model_keeps_full_bright_exact_and_blind_light_at_interpolated_body_center() {
    let _source_guard = source_test_guard();
    let (mut world, mut env) = current();
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(root(), &world, &cancel).unwrap();
    let unbound = model.draw_at(&world, &cancel, 0).unwrap();
    let full = model
        .draw_at_labels_environment(
            &world,
            &cancel,
            0,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    assert_eq!(unbound.canvas.rgba(), full.canvas.rgba());
    assert_eq!(
        full.receipt["world_lighting"]["reason"],
        "native_full_bright_mask_bypass"
    );
    env.accept(2, &[0x0a, 1]).unwrap();
    world.predict_step(0).unwrap();
    model
        .draw_at_labels_environment(
            &world,
            &cancel,
            0,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    let authority = serde_json::to_value(&world).unwrap();
    let frame = model
        .draw_at_labels_environment(
            &world,
            &cancel,
            50_000_000,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    let own = world.own_actor().unwrap().id;
    let body = frame.receipt["actors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == own)
        .unwrap();
    assert_eq!(body["phase"], "walk");
    let center = &frame.receipt["world_lighting"]["composition"]["blits"][0]["center"];
    assert_eq!(
        center,
        &serde_json::json!([
            body["screen"][0].as_i64().unwrap(),
            body["screen"][1].as_i64().unwrap() + 6
        ])
    );
    assert_eq!(
        frame.environment_key(),
        Some(Bound::capture(2, &world, &env).unwrap().key())
    );
    assert_ne!(frame.receipt["rgba_sha256"], full.receipt["rgba_sha256"]);
    assert!(frame.input_ready(&world));
    assert!(frame.sounds.is_empty());
    assert_eq!(serde_json::to_value(&world).unwrap(), authority);
}

#[test]
fn real_water_pixels_use_dry_walk_camera_history_and_phase_before_minimap_overlay() {
    let _source_guard = source_test_guard();
    let (mut world, mut env) = current();
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(root(), &world, &cancel).unwrap();
    model
        .draw_at_labels_environment(
            &world,
            &cancel,
            0,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    world.predict_step(2).unwrap();
    model
        .draw_at_labels_environment(
            &world,
            &cancel,
            0,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    model
        .draw_at_labels_environment(
            &world,
            &cancel,
            50_000_000,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    let dry = model
        .draw_at_labels_environment(
            &world,
            &cancel,
            5_000_000_000,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    let authority = serde_json::to_value(&world).unwrap();
    env.map_changed(2, world.map_id.unwrap(), Some(1)).unwrap();
    let wet = model
        .draw_at_labels_environment(
            &world,
            &cancel,
            5_000_000_000,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    let extra = &wet.receipt["world_lighting"]["water_extra"];
    assert_eq!(extra["active"], true);
    assert_eq!(extra["pixels"]["transformed"], 480000);
    assert_eq!(extra["scroll"], serde_json::json!([616, 12]));
    assert_eq!(extra["phase"], 0);
    assert_ne!(wet.canvas.rgba(), dry.canvas.rgba());
    assert!(wet
        .canvas
        .rgba()
        .chunks_exact(4)
        .zip(dry.canvas.rgba().chunks_exact(4))
        .all(|(a, b)| a[3] == b[3]));
    // The opaque own minimap cross is drawn after the water pass.
    let at = (101 * 800 + 700) * 4;
    assert_eq!(&wet.canvas.rgba()[at..at + 4], &[255, 255, 255, 255]);
    let due = env
        .water_clock
        .core
        .events()
        .iter()
        .find(|e| e.callback == lito_modern::modern_lito::native_water_clock::Callback::Water)
        .unwrap()
        .due;
    env.water_clock.tick(2, || due).unwrap();
    let wave = model
        .draw_at_labels_environment(
            &world,
            &cancel,
            5_000_000_000,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    assert_eq!(wave.receipt["world_lighting"]["water_extra"]["phase"], 1);
    assert_ne!(wave.canvas.rgba(), wet.canvas.rgba());
    assert_eq!(serde_json::to_value(&world).unwrap(), authority);
}

#[test]
fn actual_async_new_model_restores_same_game_scroll_without_adding_map_jump() {
    let _source_guard = source_test_guard();
    let (mut world, mut env) = current();
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(root(), &world, &cancel).unwrap();
    world.predict_step(2).unwrap();
    model
        .draw_at_labels_environment(
            &world,
            &cancel,
            0,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    model
        .draw_at_labels_environment(
            &world,
            &cancel,
            5_000_000_000,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    let history = model.water_history_for(2).unwrap();
    assert_eq!(history.values(), [616, 12]);
    world.generation += 1;
    for actor in world.actors.values_mut() {
        actor.motion_from = None;
    }
    env.map_changed(2, world.map_id.unwrap(), Some(1)).unwrap();
    let mut job=lito_modern::modern_lito::native_world_scene::Job::with_model_at_labels_environment_history(
        root().to_path_buf(),world.clone(),None,0,None,Some(Bound::capture(2,&world,&env).unwrap()),Some(model.light_remap()),Some(history)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        if let Some(result) = job.poll() {
            let completed = result.unwrap();
            assert_eq!(
                completed.frame.receipt["world_lighting"]["water_extra"]["scroll"],
                serde_json::json!([616, 12])
            );
            break;
        }
        assert!(Instant::now() < deadline, "water model replacement timeout");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn legacy_actual_stale_environment_job_defers_wire_sound_once_and_never_publishes_old_pixels() {
    let _source_guard = source_test_guard();
    let (mut world, mut env) = current();
    assert!(world.accept(&[0x4a, 0, 109, 0]).unwrap());
    let mut view = Presentation::default();
    view.tick(root(), 2, &world, &env, true);
    env.accept(2, &[0x0a, 1]).unwrap();
    let expected = Bound::capture(2, &world, &env).unwrap().key();
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut published = false;
    while Instant::now() < deadline {
        view.tick(root(), 2, &world, &env, true);
        assert!(view.error.is_none(), "{:?}", view.error);
        let delivery_scope = view.audio_scope(2, world.generation);
        if let Some(frame) = view.frame.as_mut() {
            assert_eq!(frame.environment_key(), Some(expected));
            assert_eq!(view.retired_environment_results, 1);
            assert_eq!(frame.receipt["deferred_environment_audio"], 1);
            assert_eq!(frame.sounds.len(), 1);
            assert_eq!(frame.legacy_audio.len(), 1);
            let cue = frame.legacy_audio[0].clone();
            assert_eq!(cue.event().unwrap().sound, Some(109));
            let mut sink = LegacySink::default();
            assert!(matches!(
                cue.submit(&delivery_scope, &mut sink).unwrap(),
                lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Played(_)
            ));
            assert_eq!(
                cue.clone().submit(&delivery_scope, &mut sink).unwrap(),
                lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::AlreadyConsumed
            );
            assert_eq!(sink.plays, vec![109]);
            frame.sounds.clear();
            frame.legacy_audio.clear();
            published = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(published, "environment-bound replacement frame timed out");
    let mut next = false;
    while Instant::now() < deadline {
        view.tick(root(), 2, &world, &env, true);
        assert!(view.error.is_none(), "{:?}", view.error);
        if let Some(frame) = view.frame.as_ref() {
            assert_eq!(frame.environment_key(), Some(expected));
            assert!(frame.sounds.is_empty());
            assert!(frame.legacy_audio.is_empty());
            if frame.receipt["render_profile_us"]["composition_frames"]
                .as_u64()
                .unwrap()
                >= 3
            {
                next = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(next, "no subsequent animation frame");
    view.cancel();
    while !view.quiescent() && Instant::now() < deadline {
        view.tick(root(), 2, &world, &env, false);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(view.quiescent());
    assert!(view.frame.is_none());
}

#[test]
fn real_water_producer_tracks_the_same_body_pose_and_native_death_marker() {
    let _source_guard = source_test_guard();
    let (mut world, mut env) = current();
    env.map_changed(2, world.map_id.unwrap(), Some(1)).unwrap();
    let cancel = AtomicBool::new(false);
    let own = world.own_actor().unwrap().id;
    let mut model = Model::load(root(), &world, &cancel).unwrap();
    let frame = model
        .draw_at_labels_environment(
            &world,
            &cancel,
            0,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    let light = &frame.receipt["world_lighting"];
    assert_eq!(light["source_classification_complete"], true);
    assert_eq!(light["water_source_classes"][own.to_string()], 5);
    assert_eq!(light["composition"]["blits"][0]["range"], 14);
    let mut death = vec![0x20];
    death.extend(own.to_le_bytes());
    death.push(8);
    world.accept(&death).unwrap();
    let snapshot = serde_json::to_value(&world).unwrap();
    let frame = model
        .draw_at_labels_environment(
            &world,
            &cancel,
            50_000_000,
            None,
            Some(&Bound::capture(2, &world, &env).unwrap()),
        )
        .unwrap();
    let body = frame.receipt["actors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == own)
        .unwrap();
    assert_eq!(body["action"], 8);
    let light = &frame.receipt["world_lighting"];
    assert_eq!(light["water_source_classes"][own.to_string()], 3);
    assert_eq!(light["composition"]["blits"][0]["range"], 5);
    assert_eq!(
        light["composition"]["blits"][0]["center"],
        serde_json::json!([
            body["screen"][0].as_i64().unwrap(),
            body["screen"][1].as_i64().unwrap() + 6
        ])
    );
    assert_eq!(serde_json::to_value(&world).unwrap(), snapshot);
}

#[test]
fn real_catalog_blind_two_uses_walking_source_cell_then_terminal_destination_without_authority_writes(
) {
    let _source_guard = source_test_guard();
    use lito_modern::modern_lito::{
        native_world_lighting::metadata,
        resources::{SharedArchive, SharedArchiveKind},
        sprite_actions::{ActionLimits, SpriteActionCatalog},
    };
    let (mut world, mut env) = current();
    let own = world.own_actor().unwrap().clone();
    let text = SharedArchive::open(root(), SharedArchiveKind::Text)
        .unwrap()
        .read_entry("list.spr")
        .unwrap();
    let catalog =
        SpriteActionCatalog::parse_for_inspection(&text, ActionLimits::default()).unwrap();
    assert_eq!(metadata(&catalog, own.graphics).unwrap()[0], 5);
    let remote = *world.actors.keys().find(|id| **id != own.id).unwrap();
    let actor = world.actors.get_mut(&remote).unwrap();
    actor.x = own.x;
    actor.y = own.y + 15;
    actor.graphics = own.graphics;
    actor.walk_action = own.walk_action;
    env.accept(2, &[0x0a, 2]).unwrap();
    world.predict_step(0).unwrap();
    let authority = serde_json::to_value(&world).unwrap();
    let bound = Bound::capture(2, &world, &env).unwrap();
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(root(), &world, &cancel).unwrap();
    let start = model
        .draw_at_labels_environment(&world, &cancel, 0, None, Some(&bound))
        .unwrap();
    let body = |frame: &lito_modern::modern_lito::native_world_scene::Frame| {
        frame.receipt["actors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == own.id)
            .unwrap()
            .clone()
    };
    assert_eq!(body(&start)["phase"], "walk");
    assert_eq!(
        start.receipt["world_lighting"]["composition"]["outside_bounds"],
        0
    );
    assert_eq!(
        start.receipt["world_lighting"]["composition"]["blits"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let end = model
        .draw_at_labels_environment(&world, &cancel, 2_000_000_000, None, Some(&bound))
        .unwrap();
    assert_eq!(body(&end)["phase"], "idle");
    assert_eq!(
        end.receipt["world_lighting"]["composition"]["outside_bounds"],
        1
    );
    assert_eq!(
        end.receipt["world_lighting"]["composition"]["blits"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(serde_json::to_value(&world).unwrap(), authority);
}

fn presentation_frame(
    view: &mut Presentation,
    root: &Path,
    epoch: u64,
    world: &World,
    env: &State,
) {
    // Existing asynchronous resource tests use this functional harness limit.
    // This is not a cold-load or GUI latency acceptance measurement.
    let end = Instant::now() + Duration::from_secs(90);
    loop {
        view.tick(root, epoch, world, env, true);
        assert!(view.error.is_none(), "{:?}", view.error);
        if view.frame.is_some() {
            return;
        }
        assert!(
            Instant::now() < end,
            "source-owner presentation frame timeout"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn inactive_presentation(
    view: &mut Presentation,
    root: &Path,
    epoch: u64,
    world: &World,
    env: &State,
    explicit_cancel: bool,
) {
    if explicit_cancel {
        view.cancel();
    }
    let end = Instant::now() + Duration::from_secs(3);
    loop {
        view.tick(root, epoch, world, env, false);
        if view.quiescent() {
            break;
        }
        assert!(
            Instant::now() < end,
            "source-owner render cancellation did not quiesce"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(view.frame.is_none());
}
#[test]
fn presentation_sources_survive_scene_retirement_without_retaining_old_input() {
    let _source_guard = source_test_guard();
    use lito_modern::modern_lito::native_resources::{Limits, SharedNativeArchives};
    let (mut world, mut env) = current();
    let mut view = Presentation::default();
    presentation_frame(&mut view, root(), 2, &world, &env);
    let old = view.frame.take().unwrap();
    let old_input = old.input_surface(&world).unwrap();
    inactive_presentation(&mut view, root(), 2, &world, &env, false);
    let probe = SharedNativeArchives::open(
        &root().join("LiTo"),
        Limits::default(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(
        probe.reused_source_owner(),
        "inactive Presentation lost its verified source lease"
    );
    drop(probe);
    world.generation += 1;
    world.visual_revision += 1;
    assert!(!old.input_ready(&world) && !old_input.input_ready(&world));
    presentation_frame(&mut view, root(), 2, &world, &env);
    assert_eq!(
        view.frame.as_ref().unwrap().receipt["load_profile_us"]["shared_archive_sources_reused"],
        true
    );
    inactive_presentation(&mut view, root(), 2, &world, &env, false);
    world.generation += 1;
    world.visual_revision += 1;
    world.relations.begin(3).unwrap();
    env.begin(3).unwrap();
    env.map_changed(3, world.map_id.unwrap(), Some(0)).unwrap();
    presentation_frame(&mut view, root(), 3, &world, &env);
    let frame = view.frame.as_ref().unwrap();
    assert_eq!(frame.environment_key().unwrap().epoch, 3);
    assert_eq!(
        frame.receipt["load_profile_us"]["shared_archive_sources_reused"],
        true
    );
    assert!(!old_input.input_ready(&world));
    inactive_presentation(&mut view, root(), 3, &world, &env, true);
    let released = SharedNativeArchives::open(
        &root().join("LiTo"),
        Limits::default(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(
        !released.reused_source_owner(),
        "cancel retained native source handles after worker quiescence"
    );
}
#[test]
fn presentation_root_change_retires_old_model_and_source_leases() {
    let _source_guard = source_test_guard();
    use lito_modern::modern_lito::native_resources::{Limits, SharedNativeArchives};
    let (world, env) = current();
    let mut view = Presentation::default();
    presentation_frame(&mut view, root(), 2, &world, &env);
    let invalid = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/current/verification/resource-loading/presentation-other-root");
    std::fs::create_dir_all(&invalid).unwrap();
    let end = Instant::now() + Duration::from_secs(90);
    loop {
        view.tick(&invalid, 2, &world, &env, true);
        assert!(
            view.frame.is_none(),
            "old-root pixels survived the source-root change"
        );
        if view.error.is_some() && view.quiescent() {
            break;
        }
        assert!(
            Instant::now() < end,
            "different-root job did not fail closed"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let released = SharedNativeArchives::open(
        &root().join("LiTo"),
        Limits::default(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(
        !released.reused_source_owner(),
        "old root retained by Presentation after root change"
    );
    drop(released);
    inactive_presentation(&mut view, &invalid, 2, &world, &env, true);
}
#[test]
fn presentation_cancel_does_not_recapture_sources_from_a_pending_render() {
    let _source_guard = source_test_guard();
    use lito_modern::modern_lito::native_resources::{Limits, SharedNativeArchives};
    let (world, env) = current();
    let mut view = Presentation::default();
    presentation_frame(&mut view, root(), 2, &world, &env);
    std::thread::sleep(Duration::from_millis(55));
    view.tick(root(), 2, &world, &env, true);
    assert!(
        !view.quiescent(),
        "expected a pending ordinary animation render"
    );
    inactive_presentation(&mut view, root(), 2, &world, &env, true);
    let released = SharedNativeArchives::open(
        &root().join("LiTo"),
        Limits::default(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(
        !released.reused_source_owner(),
        "cancel recaptured a retired job's source lease"
    );
}
#[test]
fn presentation_job_rejects_reused_model_from_other_resource_root() {
    let _source_guard = source_test_guard();
    let (world, _) = current();
    let model = Model::load(root(), &world, &AtomicBool::new(false)).unwrap();
    let invalid = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/current/verification/resource-loading/job-other-root");
    std::fs::create_dir_all(&invalid).unwrap();
    let mut job =
        lito_modern::modern_lito::native_world_scene::Job::with_model(invalid, world, Some(model))
            .unwrap();
    let end = Instant::now() + Duration::from_secs(90);
    loop {
        if let Some(result) = job.poll() {
            assert!(
                result.is_err(),
                "Job reused an old-root Model despite a different requested resource root"
            );
            break;
        }
        assert!(Instant::now() < end, "different-root Model check timeout");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn presentation_resume_same_scene_rejects_retired_render_contexts() {
    let _source_guard = source_test_guard();
    let (world, env) = current();
    let mut view = Presentation::default();
    presentation_frame(&mut view, root(), 2, &world, &env);
    for _ in 0..3 {
        std::thread::sleep(Duration::from_millis(55));
        view.tick(root(), 2, &world, &env, true);
        assert!(
            !view.quiescent(),
            "expected an ordinary pending render before retirement"
        );
        view.tick(root(), 2, &world, &env, false);
        assert!(view.frame.is_none());
        view.tick(root(), 2, &world, &env, true);
        assert!(
            view.frame.is_none(),
            "retired context republished on same-scene resume"
        );
        presentation_frame(&mut view, root(), 2, &world, &env);
        let frame = view.frame.as_ref().unwrap();
        assert_eq!(
            frame.receipt["render_profile_us"]["composition_frames"], 1,
            "retired model/animation clock reused on resume"
        );
        assert_eq!(
            frame.receipt["load_profile_us"]["shared_archive_sources_reused"],
            true
        );
    }
    assert!(view.retired_context_results + view.retired_context_errors >= 3);
    inactive_presentation(&mut view, root(), 2, &world, &env, true);
}

#[derive(Default)]
struct LegacySink {
    plays: Vec<u32>,
}
impl lito_modern::modern_lito::native_audio_delivery::legacy::LegacyPlayer for LegacySink {
    fn play_legacy(
        &mut self,
        sound: &lito_modern::modern_lito::combat_audio::PreparedSound,
    ) -> Result<
        lito_modern::modern_lito::combat_audio::PlayOutcome,
        lito_modern::modern_lito::combat_audio::PoolError,
    > {
        self.plays.push(sound.sound_id());
        Ok(
            lito_modern::modern_lito::combat_audio::PlayOutcome::Started {
                sound_id: sound.sound_id(),
                slot: 0,
            },
        )
    }
}
#[test]
fn legacy_actual_publication_binds_an_unplayed_cue_and_cancel_retires_its_owner() {
    let _source_guard = source_test_guard();
    let (mut world, env) = current();
    assert!(world.accept(&[0x4a, 0, 109, 0]).unwrap());
    let authority = serde_json::to_value(&world).unwrap();
    let mut view = Presentation::default();
    let deadline = Instant::now() + Duration::from_secs(90);
    let held = loop {
        view.tick(root(), 2, &world, &env, true);
        assert!(view.error.is_none(), "{:?}", view.error);
        if let Some(frame) = view.frame.as_ref() {
            assert_eq!(frame.sounds.len(), 1);
            assert_eq!(frame.legacy_audio.len(), 1);
            let cue = frame.legacy_audio[0].clone();
            assert_eq!(cue.event().unwrap().sound, Some(109));
            break cue;
        }
        assert!(Instant::now() < deadline, "source legacy frame timed out");
        std::thread::sleep(Duration::from_millis(10));
    };
    view.cancel();
    let retired_scope = view.audio_scope(2, world.generation);
    let mut sink = LegacySink::default();
    assert_eq!(
        held.submit(&retired_scope, &mut sink).unwrap(),
        lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Stale
    );
    assert_eq!(
        held.clone().submit(&retired_scope, &mut sink).unwrap(),
        lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::AlreadyConsumed
    );
    assert!(sink.plays.is_empty());
    while !view.quiescent() && Instant::now() < deadline {
        view.tick(root(), 2, &world, &env, false);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(view.quiescent());
    assert!(view.frame.is_none());
    assert_eq!(serde_json::to_value(&world).unwrap(), authority);
}


#[test]
fn ordinary_wire_cue_never_replays_after_same_world_model_recreation() {
    let _source_guard=source_test_guard();let(mut world,env)=current();
    assert!(world.accept(&[0x4a,0,109,0]).unwrap());let authority=serde_json::to_value(&world).unwrap();
    let mut view=Presentation::default();let deadline=Instant::now()+Duration::from_secs(90);
    let held=loop {
        view.tick(root(),2,&world,&env,true);assert!(view.error.is_none(),"{:?}",view.error);
        if let Some(frame)=view.frame.as_ref(){assert_eq!(frame.legacy_audio.len(),1);break frame.legacy_audio[0].clone();}
        assert!(Instant::now()<deadline,"initial resource frame timed out");std::thread::sleep(Duration::from_millis(10));
    };
    view.cancel();let scope=view.audio_scope(2,world.generation);let mut sink=LegacySink::default();
    let old_first=held.submit(&scope,&mut sink).unwrap();let old_repeat=held.clone().submit(&scope,&mut sink).unwrap();
    assert_eq!(old_first,lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Stale);
    assert_eq!(old_repeat,lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::AlreadyConsumed);assert!(sink.plays.is_empty());
    while !view.quiescent(){view.tick(root(),2,&world,&env,false);assert!(Instant::now()<deadline,"retirement timed out");std::thread::sleep(Duration::from_millis(10));}
    assert!(view.frame.is_none());
    loop {
        view.tick(root(),2,&world,&env,true);assert!(view.error.is_none(),"{:?}",view.error);
        if view.frame.is_some(){break;}
        assert!(Instant::now()<deadline,"recreated resource frame timed out");std::thread::sleep(Duration::from_millis(10));
    }
    let frame=view.frame.as_ref().unwrap();assert_eq!(serde_json::to_value(&world).unwrap(),authority);
    let observed=serde_json::json!({"same_authority_world_unchanged":true,"old_first":format!("{old_first:?}"),"old_repeat":format!("{old_repeat:?}"),"old_deliveries":sink.plays.len(),"new_Model_raw_sounds":frame.sounds.len(),"new_Model_legacy_cues":frame.legacy_audio.len(),"new_Model_events":frame.legacy_audio.iter().map(|c|format!("{:?}",c.event())).collect::<Vec<_>>(),"world_generation":world.generation,"wire_sound_sequence":world.wire_sound_sequence,"wire_audio":frame.receipt["wire_audio"],"normal_GUI_verified":false,"live_server_sessions":0,"normal_product_source_modified":false});
    let _=observed;
    assert!(frame.legacy_audio.is_empty()&&frame.sounds.is_empty(),"ordinary wire cue replayed after same-World Model recreation");
    assert!(world.accept(&[0x4a,0,109,0]).unwrap());let fresh_authority=serde_json::to_value(&world).unwrap();
    wire_admission_consume(&mut view,2,&world,&env,2,&mut sink);assert_eq!(sink.plays.len(),1);assert_eq!(serde_json::to_value(&world).unwrap(),fresh_authority);
    view.cancel();while !view.quiescent(){view.tick(root(),2,&world,&env,false);std::thread::sleep(Duration::from_millis(10));}
    let(mut new_world,_)=current();assert!(new_world.accept(&[0x4a,0,109,0]).unwrap());
    let mut new_env=State::default();new_env.begin(3).unwrap();new_env.map_changed(3,new_world.map_id.unwrap(),Some(0)).unwrap();
    let new_authority=serde_json::to_value(&new_world).unwrap();wire_admission_consume(&mut view,3,&new_world,&new_env,1,&mut sink);
    assert_eq!(sink.plays.len(),2);assert_eq!(serde_json::to_value(&new_world).unwrap(),new_authority);

}

fn wire_admission_consume(view:&mut Presentation,connection:u64,world:&World,env:&State,sequence:u64,sink:&mut LegacySink) {
    let deadline=Instant::now()+Duration::from_secs(90);
    let cue=loop {
        view.tick(root(),connection,world,env,true);assert!(view.error.is_none(),"{:?}",view.error);
        if let Some(frame)=view.frame.as_ref(){if let Some(cue)=frame.legacy_audio.iter().find(|c|c.event().is_some_and(|e|e.actor==0&&e.sequence==sequence)){assert_eq!(frame.legacy_audio.len(),1);assert_eq!(frame.sounds.len(),1);break cue.clone();}}
        assert!(Instant::now()<deadline,"fresh wire cue timed out");std::thread::sleep(Duration::from_millis(10));
    };
    let scope=view.audio_scope(connection,world.generation);
    assert!(matches!(cue.submit(&scope,sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Played(_)));
    assert_eq!(cue.clone().submit(&scope,sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::AlreadyConsumed);
}
#[test]
fn ordinary_pending_wire_is_retired_before_worker_can_publish() {
    let _source_guard=source_test_guard();let(mut world,env)=current();assert!(world.accept(&[0x4a,0,109,0]).unwrap());
    let authority=serde_json::to_value(&world).unwrap();let mut view=Presentation::default();let deadline=Instant::now()+Duration::from_secs(90);
    view.tick(root(),2,&world,&env,true);assert!(!view.quiescent());view.cancel();
    while !view.quiescent(){view.tick(root(),2,&world,&env,false);assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
    loop {view.tick(root(),2,&world,&env,true);assert!(view.error.is_none(),"{:?}",view.error);if let Some(frame)=view.frame.as_ref(){assert!(frame.sounds.is_empty()&&frame.legacy_audio.is_empty(),"cancelled submitted wire event reappeared");break;}assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
    assert_eq!(serde_json::to_value(&world).unwrap(),authority);assert!(world.accept(&[0x4a,0,109,0]).unwrap());
    let fresh_authority=serde_json::to_value(&world).unwrap();let mut sink=LegacySink::default();wire_admission_consume(&mut view,2,&world,&env,2,&mut sink);assert_eq!(sink.plays.len(),1);assert_eq!(serde_json::to_value(&world).unwrap(),fresh_authority);
}

fn actor_packet(world:&mut World)->(u32,u64) {
    let own=world.own_actor().unwrap().id;let mut packet=vec![0x20];packet.extend(own.to_le_bytes());packet.push(68);assert!(world.accept(&packet).unwrap());(own,world.actors[&own].action_sequence)
}
fn actor_cue(view:&mut Presentation,world:&World,env:&State,own:u32,sequence:u64)->lito_modern::modern_lito::native_audio_delivery::legacy::PreparedLegacyCue {
    let deadline=Instant::now()+Duration::from_secs(90);
    loop {
        view.tick(root(),2,world,env,true);assert!(view.error.is_none(),"{:?}",view.error);
        if let Some(frame)=view.frame.as_mut(){
            if let Some(cue)=frame.legacy_audio.iter().find(|cue|cue.event().is_some_and(|e|e.actor==own&&e.sequence==sequence&&e.action==68)){return cue.clone();}
            frame.sounds.clear();frame.legacy_audio.clear();
        }
        assert!(Instant::now()<deadline,"new actor cue timed out");std::thread::sleep(Duration::from_millis(10));
    }
}
fn actor_retire_and_recreate(pending:bool) {
    let _guard=source_test_guard();let(mut world,env)=current();let(own,sequence)=actor_packet(&mut world);
    let authority=serde_json::to_value(&world).unwrap();let mut view=Presentation::default();
    let held=if pending{view.tick(root(),2,&world,&env,true);assert!(!view.quiescent());None}else{Some(actor_cue(&mut view,&world,&env,own,sequence))};
    view.cancel();let mut sink=LegacySink::default();
    if let Some(cue)=held {
        let scope=view.audio_scope(2,world.generation);
        assert_eq!(cue.submit(&scope,&mut sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Stale);
        assert_eq!(cue.clone().submit(&scope,&mut sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::AlreadyConsumed);
        assert!(sink.plays.is_empty());
    }
    let deadline=Instant::now()+Duration::from_secs(90);
    while !view.quiescent(){view.tick(root(),2,&world,&env,false);assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
    let mut first=None;let mut frames=std::collections::BTreeSet::new();
    loop {
        view.tick(root(),2,&world,&env,true);assert!(view.error.is_none(),"{:?}",view.error);
        if let Some(frame)=view.frame.as_ref(){
            assert!(frame.legacy_audio.is_empty()&&frame.sounds.is_empty(),"retired actor event replayed");
            frames.insert(frame.receipt["elapsed_ns"].as_u64().unwrap());
            let started=first.get_or_insert_with(Instant::now);
            if started.elapsed()>=Duration::from_millis(850)&&frames.len()>=3{break;}
        }
        assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(serde_json::to_value(&world).unwrap(),authority);
    let(_,fresh)=actor_packet(&mut world);assert!(fresh>sequence);let fresh_authority=serde_json::to_value(&world).unwrap();
    let cue=actor_cue(&mut view,&world,&env,own,fresh);let scope=view.audio_scope(2,world.generation);
    assert_eq!(cue.event().unwrap().sound,Some(1534));
    assert!(matches!(cue.submit(&scope,&mut sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Played(_)));
    assert_eq!(cue.clone().submit(&scope,&mut sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::AlreadyConsumed);
    assert_eq!(sink.plays,vec![1534]);assert_eq!(serde_json::to_value(&world).unwrap(),fresh_authority);
    let o=serde_json::json!({"pending":pending,"retired_sequence":sequence,"fresh_sequence":fresh,"recreated_distinct_frames":frames.len(),"retired_deliveries":0,"fresh_deliveries":sink.plays,"authority_unchanged":true,"physical_audio_verified":false});
    let _=o;
}
#[test]
fn ordinary_actor_published_event_stays_retired_and_fresh_action_plays_once(){actor_retire_and_recreate(false);}
#[test]
fn ordinary_actor_pending_event_stays_retired_and_fresh_action_plays_once(){actor_retire_and_recreate(true);}


#[test]
fn ordinary_direct_model_map_change_cannot_keep_previous_world_death() {
 let _guard=source_test_guard();let(mut a,_)=current();let own=a.own_actor().unwrap().id;
 let packet=|action:u8|{let mut p=vec![0x20];p.extend(own.to_le_bytes());p.push(action);p};
 assert!(a.accept(&packet(8)).unwrap());let stop=AtomicBool::new(false);let mut model=Model::load(root(),&a,&stop).unwrap();model.draw_at(&a,&stop,0).unwrap();
 let old=model.draw_at(&a,&stop,60_000_000_000).unwrap();let old_actor=old.receipt["actors"].as_array().unwrap().iter().find(|r|r["id"]==own).unwrap();assert_eq!(old_actor["phase"],"death");assert_eq!(old_actor["action_complete"],true);
 let(mut b,_)=current();assert!(b.accept(&packet(68)).unwrap());assert_eq!(a.actors[&own].incarnation,b.actors[&own].incarnation);assert_eq!(a.actors[&own].action_sequence,b.actors[&own].action_sequence);
 b.generation=a.generation+1;b.map_id=Some(500);{let actor=b.actors.get_mut(&own).unwrap();actor.x=32768;actor.y=32832;}
 let authority=serde_json::to_value(&b).unwrap();let frame=model.draw_at(&b,&stop,61_000_000_000).unwrap();assert_eq!(serde_json::to_value(&b).unwrap(),authority);let actor=frame.receipt["actors"].as_array().unwrap().iter().find(|r|r["id"]==own).unwrap();
 let o=serde_json::json!({"authority_unchanged":true,"actor_alive":!b.actors[&own].dead,"old_world_generation":a.generation,"new_world_generation":b.generation,"old_map":a.map_id,"new_map":b.map_id,"new_actor_phase":actor["phase"],"new_actor_action":actor["action"],"new_actor_receipt":actor,"synthetic_authority_resource_fixture":true,"live_server_sessions":0,"physical_audio_verified":false});
 let _=o;assert_ne!(actor["phase"],"death","previous world death leaked into new alive actor after map change");
}


fn admission_quiesce(view:&mut Presentation,world:&World,env:&State) {
    view.cancel();let end=Instant::now()+Duration::from_secs(90);
    while !view.quiescent(){view.tick(root(),2,world,env,false);assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(10));}
}
fn admission_no_old(view:&mut Presentation,world:&World,env:&State) {
    let end=Instant::now()+Duration::from_secs(90);let mut first=None;let mut frames=std::collections::BTreeSet::new();
    loop {
        view.tick(root(),2,world,env,true);assert!(view.error.is_none(),"{:?}",view.error);
        if let Some(frame)=view.frame.as_ref(){
            assert!(frame.legacy_audio.is_empty()&&frame.sounds.is_empty(),"retired cue reappeared after source ABA");
            frames.insert(frame.receipt["elapsed_ns"].as_u64().unwrap());
            let start=first.get_or_insert_with(Instant::now);if start.elapsed()>=Duration::from_millis(850)&&frames.len()>=3{break;}
        }
        assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn admission_lifecycle_invalid_source_ABA_retires_actor_and_wire_cues() {
    let _guard=source_test_guard();let(mut world,env)=current();let(own,sequence)=actor_packet(&mut world);
    assert!(world.accept(&[0x4a,0,109,0]).unwrap());let authority=serde_json::to_value(&world).unwrap();let mut view=Presentation::default();
    let held=actor_cue(&mut view,&world,&env,own,sequence);
    let invalid=Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/ordinary-admission-v2/source-B-invalid");std::fs::create_dir_all(&invalid).unwrap();
    let end=Instant::now()+Duration::from_secs(90);
    loop{view.tick(&invalid,2,&world,&env,true);assert!(view.frame.is_none());if view.error.is_some()&&view.quiescent(){break;}assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(10));}
    let mut sink=LegacySink::default();let scope=view.audio_scope(2,world.generation);
    assert_eq!(held.submit(&scope,&mut sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Stale);assert!(sink.plays.is_empty());
    admission_no_old(&mut view,&world,&env);assert_eq!(serde_json::to_value(&world).unwrap(),authority);
    let(_,fresh)=actor_packet(&mut world);let cue=actor_cue(&mut view,&world,&env,own,fresh);let scope=view.audio_scope(2,world.generation);
    assert!(matches!(cue.submit(&scope,&mut sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Played(_)));assert_eq!(sink.plays,vec![1534]);
}
#[test]
fn admission_lifecycle_reused_actor_id_and_new_connection_accept_fresh_actions() {
    let _guard=source_test_guard();let(mut world,env)=current();let(own,sequence)=actor_packet(&mut world);let mut view=Presentation::default();let _held=actor_cue(&mut view,&world,&env,own,sequence);
    admission_quiesce(&mut view,&world,&env);let incarnation=world.actors[&own].incarnation;
    let f:serde_json::Value=serde_json::from_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/native-world-current.json"))).unwrap();let text=f["packets"]["actor"].as_str().unwrap();let packet:Vec<_>=(0..text.len()).step_by(2).map(|i|u8::from_str_radix(&text[i..i+2],16).unwrap()).collect();assert!(world.accept(&packet).unwrap());assert_ne!(world.actors[&own].incarnation,incarnation);
    let(_,fresh)=actor_packet(&mut world);let authority=serde_json::to_value(&world).unwrap();let cue=actor_cue(&mut view,&world,&env,own,fresh);let mut sink=LegacySink::default();assert!(matches!(cue.submit(&view.audio_scope(2,world.generation),&mut sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Played(_)));assert_eq!(sink.plays,vec![1534]);assert_eq!(serde_json::to_value(&world).unwrap(),authority);
    admission_quiesce(&mut view,&world,&env);
    let(mut world,_)=current();let(own,sequence)=actor_packet(&mut world);let mut env=State::default();env.begin(3).unwrap();env.map_changed(3,world.map_id.unwrap(),Some(0)).unwrap();
    let end=Instant::now()+Duration::from_secs(90);let cue=loop{view.tick(root(),3,&world,&env,true);assert!(view.error.is_none(),"{:?}",view.error);if let Some(frame)=view.frame.as_mut(){if let Some(cue)=frame.legacy_audio.iter().find(|c|c.event().is_some_and(|e|e.actor==own&&e.sequence==sequence)){break cue.clone();}frame.legacy_audio.clear();frame.sounds.clear();}assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(10));};
    assert!(matches!(cue.submit(&view.audio_scope(3,world.generation),&mut sink).unwrap(),lito_modern::modern_lito::native_audio_delivery::DeliveryOutcome::Played(_)));assert_eq!(sink.plays,vec![1534,1534]);
}
#[test]
fn admission_lifecycle_cancelled_pending_death_holds_final_pose() {
    let _guard=source_test_guard();let(mut world,env)=current();let own=world.own_actor().unwrap().id;let mut p=vec![0x20];p.extend(own.to_le_bytes());p.push(8);assert!(world.accept(&p).unwrap());assert!(world.actors[&own].dead);let authority=serde_json::to_value(&world).unwrap();let mut view=Presentation::default();
    view.tick(root(),2,&world,&env,true);assert!(!view.quiescent());admission_quiesce(&mut view,&world,&env);
    let end=Instant::now()+Duration::from_secs(90);let mut first=None;
    loop{
        view.tick(root(),2,&world,&env,true);assert!(view.error.is_none(),"{:?}",view.error);
        if let Some(frame)=view.frame.as_ref(){let actor=frame.receipt["actors"].as_array().unwrap().iter().find(|a|a["id"]==own).unwrap();assert_eq!(actor["phase"],"death");assert_eq!(actor["action"],8);assert_eq!(actor["action_complete"],true);assert!(frame.legacy_audio.is_empty()&&frame.sounds.is_empty());let start=first.get_or_insert_with(Instant::now);if start.elapsed()>=Duration::from_millis(250){break;}}
        assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(serde_json::to_value(&world).unwrap(),authority);
}

fn generation_fixture()->(Model,World,World) {
    let(mut old,_)=current();let own=old.own_actor().unwrap().id;let packet=|action:u8|{let mut p=vec![0x20];p.extend(own.to_le_bytes());p.push(action);p};
    assert!(old.accept(&packet(8)).unwrap());let stop=AtomicBool::new(false);let mut model=Model::load(root(),&old,&stop).unwrap();model.draw_at(&old,&stop,0).unwrap();let frame=model.draw_at(&old,&stop,60_000_000_000).unwrap();assert_eq!(generation_actor(&frame,own)["phase"],"death");assert_eq!(generation_actor(&frame,own)["action_complete"],true);
    let(mut fresh,_)=current();assert!(fresh.accept(&packet(68)).unwrap());assert_eq!(old.actors[&own].incarnation,fresh.actors[&own].incarnation);assert_eq!(old.actors[&own].action_sequence,fresh.actors[&own].action_sequence);fresh.generation=old.generation+1;assert_eq!(old.map_id,fresh.map_id);assert!(!fresh.actors[&own].dead);(model,old,fresh)
}
fn generation_actor(frame:&lito_modern::modern_lito::native_world_scene::Frame,id:u32)->&serde_json::Value {frame.receipt["actors"].as_array().unwrap().iter().find(|row|row["id"]==id).unwrap()}
#[test]
fn generation_model_direct_same_map_new_world_has_fresh_action_and_sound_once() {
    let _guard=source_test_guard();let(mut model,_,fresh)=generation_fixture();let authority=serde_json::to_value(&fresh).unwrap();let own=fresh.own_actor().unwrap().id;let stop=AtomicBool::new(false);
    let first=model.draw_at(&fresh,&stop,0).unwrap();assert_eq!(generation_actor(&first,own)["phase"],"action");assert_eq!(generation_actor(&first,own)["action"],68);assert_eq!(first.receipt["world_generation"],fresh.generation);
    let frame=model.draw_at(&fresh,&stop,1_000_000_000).unwrap();assert_eq!(frame.legacy_audio.iter().filter(|c|c.event().is_some_and(|e|e.actor==own&&e.action==68&&e.sound==Some(1534))).count(),1);
    let repeat=model.draw_at(&fresh,&stop,1_000_000_000).unwrap();assert!(repeat.legacy_audio.is_empty()&&repeat.sounds.is_empty());assert_eq!(serde_json::to_value(&fresh).unwrap(),authority);
}
#[test]
fn generation_model_job_rejects_same_map_old_world_with_restarted_clock() {
    let _guard=source_test_guard();let(model,_,fresh)=generation_fixture();let authority=serde_json::to_value(&fresh).unwrap();let own=fresh.own_actor().unwrap().id;
    let mut job=lito_modern::modern_lito::native_world_scene::Job::with_model_at_labels_environment(root().to_path_buf(),fresh.clone(),Some(model),0,None,None).unwrap();let end=Instant::now()+Duration::from_secs(90);
    let done=loop{if let Some(result)=job.poll(){break result.unwrap();}assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(10));};assert_eq!(generation_actor(&done.frame,own)["phase"],"action");assert_eq!(generation_actor(&done.frame,own)["action"],68);assert_eq!(done.frame.receipt["world_generation"],fresh.generation);assert_eq!(serde_json::to_value(&fresh).unwrap(),authority);
}
#[test]
fn generation_model_cancelled_transition_preserves_old_model_then_new_world_succeeds() {
    let _guard=source_test_guard();let(mut model,old,fresh)=generation_fixture();let authority=serde_json::to_value(&fresh).unwrap();let navigation=model.navigation();let own=fresh.own_actor().unwrap().id;
    assert!(model.draw_at(&fresh,&AtomicBool::new(true),0).is_err());assert!(std::sync::Arc::ptr_eq(&navigation,&model.navigation()));let frame=model.draw_at(&old,&AtomicBool::new(false),61_000_000_000).unwrap();assert_eq!(generation_actor(&frame,own)["phase"],"death");
    let frame=model.draw_at(&fresh,&AtomicBool::new(false),0).unwrap();assert_eq!(generation_actor(&frame,own)["phase"],"action");assert_eq!(generation_actor(&frame,own)["action"],68);assert_eq!(serde_json::to_value(&fresh).unwrap(),authority);
}
