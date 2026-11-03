use super::*;
use crate::modern_lito::{
    application_protocol as app,
    battle_follow::Follower,
    battle_payload::data as d,
    battle_read, protocol,
    world_map::{PixelPoint, ViewProjection, WorldCell},
    world_scene,
};
use std::sync::atomic::AtomicUsize;

pub(crate) fn observation(data: d::UnvalidatedData) -> Observation {
    let request = battle_read::Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: data.personal.player.source as u32,
                object_id: data.personal.player.object as u64,
            },
            epoch: data.personal.player.epoch as u64,
        },
        channel_id: data.personal.player.channel as u8,
        world: data.world.identity.clone(),
    };
    let mut follower = Follower::new(
        request.clone(),
        protocol::Ack {
            request_id: 1,
            version: 35,
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES,
        },
    )
    .unwrap();
    follower
        .accept(battle_read::UnvalidatedReply {
            echo: request,
            state: battle_read::State::Data,
            data: Some(data),
        })
        .unwrap();
    follower.observation().unwrap().clone()
}
fn sample(clock: i64) -> Observation {
    let mut data = crate::modern_lito::battle_payload::data_tests::fixture();
    data.world.loading_elapsed_nanos = clock;
    observation(data)
}
#[test]
fn live_navigation_never_uses_lagged_pixels_or_a_retired_same_world_attachment() {
    let (map, mut data) = crate::modern_lito::battle_space::tests::sample();
    let cancel = AtomicBool::new(false);
    let old = observation(data.clone());
    let space =
        crate::modern_lito::battle_space::Space::qualify(map, old.clone(), &cancel).unwrap();
    let mut runtime = BattleRuntime::new(PathBuf::from("D:/never-opened-navigation-test")).unwrap();
    runtime.attach(old.clone()).unwrap();
    let publish = |runtime: &mut BattleRuntime, obs: Observation| {
        let input = runtime.desired.as_ref().unwrap().clone();
        let mut surface = fake_surface(obs.clone());
        surface.navigation = Some(space.snapshot(obs, &cancel).unwrap());
        runtime.published = Some(Arc::new(Output {
            input,
            surface: Arc::new(surface),
            audio: PreparedAudio::empty(runtime.preferences.snapshot()),
            resources: None,
        }));
    };
    publish(&mut runtime, old.clone());
    let retained = runtime.navigation().unwrap();
    assert!(retained.matches(&old));
    data.world.loading_elapsed_nanos = 100;
    let next = observation(data);
    runtime.update(next.clone()).unwrap();
    assert!(
        runtime.navigation().is_none(),
        "old pixels are not new occupancy"
    );
    publish(&mut runtime, next.clone());
    assert!(runtime.navigation().unwrap().matches(&next));
    runtime.clear();
    assert!(runtime.navigation().is_none());
    runtime.attach(old).unwrap();
    assert!(runtime.navigation().is_none());
    assert!(retained.observation().data().world.loading_elapsed_nanos == 0);
    runtime.shutdown();
    assert!(runtime.navigation().is_none());
    assert!(runtime.is_quiescent());
}
#[test]
fn route_terrain_survives_pixel_lag_without_reusing_stale_occupancy_or_another_attachment() {
    let (map, mut data) = crate::modern_lito::battle_space::tests::sample();
    data.world.phase = d::Phase::Active;
    data.world.revision = 2;
    data.world.loading_elapsed_nanos = 100_000_000;
    data.world.active_elapsed_nanos = 0;
    for a in &mut data.actors.0 {
        a.loaded = true;
        a.step_interval_nanos = 200_000_000;
    }
    let old = observation(data.clone());
    let cancel = AtomicBool::new(false);
    let space = crate::modern_lito::battle_space::Space::qualify(map.clone(), old.clone(), &cancel)
        .unwrap();
    let mut runtime =
        BattleRuntime::new(PathBuf::from("D:/never-opened-static-terrain-test")).unwrap();
    runtime.attach(old.clone()).unwrap();
    assert!(
        runtime.routing_map(&old).is_none(),
        "no map proof before completed shared renderer"
    );
    let mut surface = fake_surface(old.clone());
    surface.navigation = Some(space.snapshot(old.clone(), &cancel).unwrap());
    runtime.published = Some(Arc::new(Output {
        input: runtime.desired.as_ref().unwrap().clone(),
        surface: Arc::new(surface),
        audio: PreparedAudio::empty(runtime.preferences.snapshot()),
        resources: None,
    }));
    assert!(Arc::ptr_eq(&runtime.routing_map(&old).unwrap(), &map));
    let retained = runtime.navigation().unwrap();
    data.world.loading_elapsed_nanos += 100_000_000;
    data.world.active_elapsed_nanos += 100_000_000;
    let tick = observation(data.clone());
    runtime.update(tick.clone()).unwrap();
    assert!(runtime.navigation().is_none());
    assert!(
        runtime.routing_map(&tick).is_some(),
        "clock updates cannot lock static terrain behind pixels"
    );
    assert!(
        runtime.routing_map(&old).is_none(),
        "consumer must bring current authority"
    );
    data.world.loading_elapsed_nanos += 200_000_000;
    data.world.active_elapsed_nanos += 200_000_000;
    data.world.revision += 1;
    data.actors.0[1].position.x += 1;
    data.actors.0[1].last_move_sequence += 1;
    data.actors.0[1].heading = 2;
    let moved = observation(data.clone());
    runtime.update(moved.clone()).unwrap();
    assert!(
        runtime.navigation().is_none(),
        "old dynamic blocker snapshot must remain unavailable"
    );
    assert!(runtime.routing_map(&moved).is_some());
    assert!(retained
        .blocked()
        .contains(&WorldCell { x: 32770, y: 32768 }));
    assert!(!retained
        .blocked()
        .contains(&WorldCell { x: 32771, y: 32768 }));
    data.personal.player.epoch += 1;
    assert!(runtime.routing_map(&observation(data)).is_none());
    runtime.failed = Some("unit-test-renderer-failure");
    assert!(runtime.routing_map(&moved).is_none());
    runtime.failed = None;
    runtime.clear();
    runtime.attach(moved.clone()).unwrap();
    assert!(
        runtime.routing_map(&moved).is_none(),
        "same-world reentry cannot reuse retired pixels"
    );
    runtime.shutdown();
    assert!(runtime.routing_map(&moved).is_none());
    assert!(runtime.is_quiescent());
}
#[derive(Default)]
struct Counts {
    loads: AtomicUsize,
    active: AtomicUsize,
    maximum: AtomicUsize,
    renders: AtomicUsize,
    off_thread: AtomicBool,
}
struct Fake {
    observation: Observation,
    counts: Arc<Counts>,
    reload: bool,
    invalid: bool,
    panic: bool,
}
impl Drop for Fake {
    fn drop(&mut self) {
        self.counts.active.fetch_sub(1, Ordering::AcqRel);
    }
}
impl Renderer for Fake {
    fn requires_reload(&self, next: &Observation) -> Result<bool> {
        Ok(self.reload
            && self.observation.data().world.loading_elapsed_nanos < 50
            && next.data().world.loading_elapsed_nanos >= 50)
    }
    fn observe(&mut self, next: Observation, _: &AtomicBool) -> Result<()> {
        self.observation.check_successor(&next)?;
        self.observation = next;
        Ok(())
    }
    fn render(&mut self, _: Instant, _: &AtomicBool) -> Result<Surface> {
        assert!(!self.panic, "synthetic render panic");
        self.counts.renders.fetch_add(1, Ordering::AcqRel);
        let mut surface = fake_surface(self.observation.clone());
        if self.invalid {
            surface.geometry.revision += 1;
        }
        Ok(surface)
    }
}
fn fake_surface(observation: Observation) -> Surface {
    let data = observation.data();
    Surface {
        geometry: world_scene::RenderInfo {
            world_id: data.world.identity.match_id as u64,
            revision: data.world.revision as u64,
            projection: ViewProjection {
                anchor_cell: WorldCell { x: 32768, y: 32768 },
                anchor_pixel: PixelPoint { x: 400, y: 300 },
            },
            visible_tiles: 0,
            actors: vec![],
            objectives: vec![],
            npcs: vec![],
        },
        bgra: vec![data.world.loading_elapsed_nanos as u8; 800 * 600 * 4].into(),
        observation,
        width: 800,
        height: 600,
        objectives: vec![],
        catalog_sha256: [0; 32],
        navigation: None,
        minimap: None,
        audio: None,
        effects_skipped: false,
        limitations: world_scene::SceneLimitations {
            opacity_records: 0,
            sprite_effects: 0,
            portals: 0,
            actor_object_occlusion_unimplemented: true,
            actor_dependent_roofs_unimplemented: true,
            native_roof_timers_and_aliases_unverified: true,
            native_actor_update_order_unverified: true,
            weapon_and_transformation_art_unimplemented: true,
            native_actor_foot_pivot_unverified: true,
            unplayed_sound_ids: vec![],
            unrendered_base_effect_ids: vec![],
            unrendered_base_effects: vec![],
        },
    }
}
// Synthetic certificate is available only in cfg(test); this tests ownership,
// not real asset qualification (covered by shared-resource TLS integration).
fn preflight_sample(certified: bool) -> (BattleRuntime, Observation, Arc<Surface>) {
    let (map, data) = crate::modern_lito::battle_space::tests::sample();
    let obs = observation(data);
    let mut runtime = BattleRuntime::new(PathBuf::from("D:/never-opened-preflight-test")).unwrap();
    runtime.attach(obs.clone()).unwrap();
    let space =
        crate::modern_lito::battle_space::Space::qualify(map, obs.clone(), &AtomicBool::new(false))
            .unwrap();
    let mut surface = fake_surface(obs.clone());
    surface.navigation = Some(
        space
            .snapshot(obs.clone(), &AtomicBool::new(false))
            .unwrap(),
    );
    let surface = Arc::new(surface);
    runtime.published = Some(Arc::new(Output {
        input: runtime.desired.as_ref().unwrap().clone(),
        surface: surface.clone(),
        audio: PreparedAudio::empty(runtime.preferences.snapshot()),
        resources: certified.then(|| {
            super::super::battle_scene::PreparedResources::synthetic_for_runtime_test(obs.clone())
        }),
    }));
    (runtime, obs, surface)
}
#[test]
fn canonical_load_requires_sealed_resources_not_only_pixels_and_navigation() {
    let (runtime, obs, surface) = preflight_sample(false);
    assert!(runtime.navigation().is_some());
    assert!(!runtime.prepared(&surface));
    assert!(runtime.loaded_request(&surface, &obs).is_none());
    let (mut runtime, obs, surface) = preflight_sample(true);
    let q = runtime.loaded_request(&surface, &obs).unwrap();
    assert_eq!(q.action, super::super::battle_world::Action::Loaded);
    assert_eq!(q.digest, obs.data().manifest.digest);
    assert_eq!(q.room, obs.data().world.identity.room);
    assert_eq!(q.context.epoch as i64, obs.data().personal.player.epoch);
    assert!(runtime
        .loaded_request(&fake_surface(obs.clone()), &obs)
        .is_none());
    runtime.failed = Some("synthetic failure");
    assert!(runtime.loaded_request(&surface, &obs).is_none());
}
#[test]
fn loading_clock_progress_is_allowed_but_changed_actors_owner_and_manifest_are_not() {
    let (mut runtime, obs, surface) = preflight_sample(true);
    let mut data = obs.data().clone();
    data.world.loading_elapsed_nanos = 1;
    let clock = observation(data.clone());
    assert!(runtime.loaded_request(&surface, &clock).is_some());
    runtime.update(clock.clone()).unwrap();
    assert!(runtime.loaded_request(&surface, &clock).is_some());
    assert!(
        runtime.loaded_request(&surface, &obs).is_none(),
        "old server clock"
    );
    data.world.revision += 1;
    data.actors.0[1].loaded = true;
    let guest_loaded = observation(data.clone());
    assert!(runtime.loaded_request(&surface, &guest_loaded).is_none());
    let mut data = obs.data().clone();
    data.personal.player.epoch += 1;
    assert!(runtime
        .loaded_request(&surface, &observation(data))
        .is_none());
    let mut data = obs.data().clone();
    data.manifest.digest[0] ^= 1;
    assert!(runtime
        .loaded_request(&surface, &observation(data))
        .is_none());
}
#[test]
fn canonical_load_refuses_already_loaded_active_abort_shutdown_and_reused_attachment() {
    let (mut runtime, obs, surface) = preflight_sample(true);
    let mut data = obs.data().clone();
    data.world.revision += 1;
    data.actors.0[0].loaded = true;
    assert!(runtime
        .loaded_request(&surface, &observation(data.clone()))
        .is_none());
    data.world.phase = d::Phase::Active;
    data.world.active_elapsed_nanos = 0;
    for actor in &mut data.actors.0 {
        actor.loaded = true;
    }
    assert!(runtime
        .loaded_request(&surface, &observation(data))
        .is_none());
    runtime.clear();
    runtime.attach(obs.clone()).unwrap();
    assert!(runtime.loaded_request(&surface, &obs).is_none());
    let (mut runtime, obs, surface) = preflight_sample(true);
    let mut data = obs.data().clone();
    data.world.revision += 1;
    data.world.phase = d::Phase::Aborted;
    data.world.abort = d::Abort::ServerAbort;
    runtime.update(observation(data)).unwrap();
    assert!(runtime.loaded_request(&surface, &obs).is_none());
    runtime.shutdown();
    assert!(runtime.is_quiescent());
}
#[test]
fn resource_proof_must_match_the_completed_surface_and_retain_navigation() {
    let (mut runtime, obs, surface) = preflight_sample(true);
    let mut different = obs.data().clone();
    different.world.loading_elapsed_nanos += 1;
    let output = Arc::get_mut(runtime.published.as_mut().unwrap()).unwrap();
    output.resources = Some(
        super::super::battle_scene::PreparedResources::synthetic_for_runtime_test(observation(
            different,
        )),
    );
    assert!(runtime.loaded_request(&surface, &obs).is_none());
    let output = Arc::get_mut(runtime.published.as_mut().unwrap()).unwrap();
    output.resources = Some(
        super::super::battle_scene::PreparedResources::synthetic_for_runtime_test(obs.clone()),
    );
    let mut fake = fake_surface(obs.clone());
    fake.effects_skipped = true;
    fake.navigation = surface.navigation.clone();
    let skipped = Arc::new(fake);
    output.surface = skipped.clone();
    assert!(runtime.loaded_request(&skipped, &obs).is_none());
    let output = Arc::get_mut(runtime.published.as_mut().unwrap()).unwrap();
    let absent = Arc::new(fake_surface(obs.clone()));
    output.surface = absent.clone();
    assert!(!runtime.prepared(&absent));
}
fn loader(
    counts: Arc<Counts>,
    hold: Option<Arc<AtomicBool>>,
    reload: bool,
    invalid: bool,
    panic: bool,
) -> Loader {
    let owner = thread::current().id();
    Arc::new(move |_, observation, _| {
        counts
            .off_thread
            .store(thread::current().id() != owner, Ordering::Release);
        counts.loads.fetch_add(1, Ordering::AcqRel);
        let active = counts.active.fetch_add(1, Ordering::AcqRel) + 1;
        counts.maximum.fetch_max(active, Ordering::AcqRel);
        let fake = Fake {
            observation,
            counts: counts.clone(),
            reload,
            invalid,
            panic,
        };
        if let Some(hold) = &hold {
            while hold.load(Ordering::Acquire) {
                thread::sleep(Duration::from_millis(1));
            }
        }
        Ok(Box::new(fake))
    })
}
fn runtime(loader: Loader) -> BattleRuntime {
    BattleRuntime::with_loader(
        PathBuf::from("synthetic-root-never-opened"),
        loader,
        Arc::new(SharedPreferences::default()),
    )
}

struct AudioRenderer {
    observation: Observation,
    frame: super::super::world_audio::AudioFrame,
}
impl Renderer for AudioRenderer {
    fn requires_reload(&self, _: &Observation) -> Result<bool> {
        Ok(false)
    }
    fn observe(&mut self, next: Observation, _: &AtomicBool) -> Result<()> {
        self.observation.check_successor(&next)?;
        self.observation = next;
        Ok(())
    }
    fn render(&mut self, _: Instant, _: &AtomicBool) -> Result<Surface> {
        let mut surface = fake_surface(self.observation.clone());
        surface.audio = Some(self.frame.clone());
        Ok(surface)
    }
}
fn audio_loader(
    banks: Vec<Arc<super::super::combat_audio::SoundBank>>,
    loads: Arc<AtomicUsize>,
) -> Loader {
    let thread = thread::current().id();
    Arc::new(move |_, observation, _| {
        assert_ne!(
            thread::current().id(),
            thread,
            "audio preparation belongs to worker"
        );
        loads.fetch_add(1, Ordering::AcqRel);
        let now = Instant::now();
        let mut frame = super::super::battle_playback::tests::frame(1, now, banks[0].clone());
        frame.events = banks
            .iter()
            .enumerate()
            .map(|(index, bank)| {
                let mut e = frame.events[0].clone();
                e.sequence = index as u64 + 1;
                e.bank = Some(bank.clone());
                e
            })
            .collect::<Vec<_>>()
            .into();
        Ok(Box::new(AudioRenderer { observation, frame }))
    })
}
fn active_sample() -> Observation {
    let mut data = sample(0).data().clone();
    data.world.revision = 2;
    data.world.phase = Phase::Active;
    data.world.loading_elapsed_nanos = 100_000_000;
    data.world.active_elapsed_nanos = 0;
    for actor in &mut data.actors.0 {
        actor.loaded = true;
        actor.step_interval_nanos = 200_000_000;
    }
    observation(data)
}
fn ready_audio(runtime: &mut BattleRuntime, gain: Option<u8>) -> Arc<Surface> {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        if let Status::Ready(surface) = runtime.poll() {
            let view = runtime.audio_view();
            if view
                .publication
                .is_some_and(|(_, p)| p.gain == gain && p.preferences == view.preferences)
            {
                return surface;
            }
        }
        assert!(
            Instant::now() < end,
            "canonical audio settings did not converge"
        );
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn live_gain_is_worker_prepared_without_new_world_revision_reload_or_old_surface_mutation() {
    use super::super::{
        battle_playback::tests::{bank, Recorder},
        preferences::Preferences,
    };
    let loads = Arc::new(AtomicUsize::new(0));
    let preferences = Arc::new(SharedPreferences::default());
    let banks = vec![bank(&[0, 128, 255, 127]), bank(&[255, 128, 0, 129])];
    let mut runtime = BattleRuntime::with_loader(
        PathBuf::from("synthetic-no-file-io"),
        audio_loader(banks.clone(), loads.clone()),
        preferences.clone(),
    );
    let observation = active_sample();
    runtime.attach(observation.clone()).unwrap();
    let mut output = Recorder::new();
    output.tick(&runtime, true, Instant::now());
    let first = ready_audio(&mut runtime, Some(100));
    assert_eq!(output.tick(&runtime, true, Instant::now()).started, 2);
    let owner = runtime.desired.as_ref().unwrap().attachment.clone();
    for gain in [50, 0, 37, 100] {
        preferences
            .apply(Preferences {
                volume_percent: gain,
                ..Preferences::default()
            })
            .unwrap();
        // A retained publication at the old gain is safe while the worker
        // catches up, including muted recovery and same engine revision.
        assert_eq!(output.tick(&runtime, true, Instant::now()).started, 0);
        let current = ready_audio(&mut runtime, Some(gain));
        assert_eq!(current.observation.data(), observation.data());
        assert_eq!(current.bgra, first.bgra);
        for (event, source) in current.audio.as_ref().unwrap().events.iter().zip(&banks) {
            let expected = source
                .get(97)
                .unwrap()
                .with_gain(if gain == 0 { 100 } else { gain })
                .unwrap();
            assert_eq!(
                event.bank.as_ref().unwrap().get(97).unwrap().pcm_bytes(),
                expected.pcm_bytes()
            );
        }
        assert_eq!(output.tick(&runtime, true, Instant::now()).started, 0);
        assert_eq!(
            first.audio.as_ref().unwrap().events[0]
                .bank
                .as_ref()
                .unwrap()
                .get(97)
                .unwrap()
                .pcm_bytes(),
            [0, 128, 255, 127]
        );
        assert!(runtime.audio_notice().is_none());
    }
    assert_eq!(loads.load(Ordering::Acquire), 1);
    runtime.clear();
    assert_eq!(output.tick(&runtime, true, Instant::now()).started, 0);
    runtime.attach(observation).unwrap();
    assert!(!Arc::ptr_eq(
        &owner,
        &runtime.desired.as_ref().unwrap().attachment
    ));
    ready_audio(&mut runtime, Some(100));
    assert_eq!(output.tick(&runtime, true, Instant::now()).started, 2);
    assert_eq!(output.count(), 4);
    runtime.shutdown();
    output.tick(&runtime, true, Instant::now());
    retire(&mut runtime);
}

#[test]
fn failed_gain_publishes_usable_pixels_with_silent_cue_history_and_visible_notice() {
    use super::super::{
        battle_playback::tests::Recorder, combat_audio::SoundBank, preferences::Preferences,
    };
    let preferences = Arc::new(SharedPreferences::default());
    preferences
        .apply(Preferences {
            volume_percent: 50,
            ..Preferences::default()
        })
        .unwrap();
    let bank = Arc::new(SoundBank::playback_subset(std::collections::BTreeMap::new()).unwrap());
    let mut runtime = BattleRuntime::with_loader(
        PathBuf::from("synthetic-no-file-io"),
        audio_loader(vec![bank], Arc::new(AtomicUsize::new(0))),
        preferences,
    );
    runtime.attach(active_sample()).unwrap();
    let surface = ready_audio(&mut runtime, None);
    assert_eq!(surface.bgra.len(), 800 * 600 * 4);
    assert_eq!(surface.audio.as_ref().unwrap().events.len(), 1);
    assert!(runtime.audio_notice().unwrap().contains("已靜音"));
    let mut output = Recorder::new();
    assert_eq!(output.tick(&runtime, true, Instant::now()).started, 0);
    assert_eq!(output.count(), 0);
    runtime.shutdown();
    output.tick(&runtime, true, Instant::now());
    retire(&mut runtime);
}

#[test]
#[cfg(windows)]
#[ignore = "explicit LITO_NATIVE_AUDIO_OPT_IN=1 + shared root; plays shared Sound97 at 10 percent via real WinMM then stops; synthetic observation/terrain"]
fn actual_shared_worker_gain_and_attachment_reach_native_device_then_release() {
    use super::super::{
        battle_playback::BattleAudioOutput,
        combat_animation::classify_native_cues,
        combat_audio::{BankLimits, SoundBank},
        preferences::Preferences,
        sprite_actions::FrameEvent,
    };
    assert_eq!(
        std::env::var("LITO_NATIVE_AUDIO_OPT_IN").as_deref(),
        Ok("1")
    );
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let group = classify_native_cues(&[FrameEvent::Sound(97)])
        .unwrap()
        .audio
        .unwrap();
    let bank = Arc::new(
        SoundBank::load_shared(
            &root,
            &[group],
            BankLimits::default(),
            &AtomicBool::new(false),
        )
        .unwrap(),
    );
    let preferences = Arc::new(SharedPreferences::default());
    preferences
        .apply(Preferences {
            volume_percent: 10,
            ..Preferences::default()
        })
        .unwrap();
    let mut runtime = BattleRuntime::with_loader(
        root,
        audio_loader(vec![bank], Arc::new(AtomicUsize::new(0))),
        preferences,
    );
    let mut output = BattleAudioOutput::new().unwrap();
    assert!(output.is_quiescent(), "construction must not open a device");
    runtime.attach(active_sample()).unwrap();
    output.tick(&runtime, true, Instant::now());
    ready_audio(&mut runtime, Some(10));
    let result = output.tick(&runtime, true, Instant::now());
    assert!(
        !result.device_failed,
        "actual native driver rejected prepared PCM: {result:?}"
    );
    assert_eq!(result.started, 1);
    assert_eq!(output.tick(&runtime, true, Instant::now()).started, 0);
    // Briefly permit the real voice to run. Never change Windows mixer volume.
    thread::sleep(Duration::from_millis(100));
    output.stop();
    assert_eq!(
        output.tick(&runtime, true, Instant::now()).started,
        0,
        "stop cannot reset watermark"
    );
    runtime.shutdown();
    let end = Instant::now() + Duration::from_secs(5);
    while !runtime.is_quiescent() || !output.is_quiescent() {
        runtime.poll();
        output.tick(&runtime, true, Instant::now());
        assert!(
            Instant::now() < end,
            "native audio/worker cleanup did not converge"
        );
        thread::sleep(Duration::from_millis(1));
    }
    eprintln!("native canonical audio: shared Sound97, worker-prepared gain10, actual WinMM start=1, duplicate/stop no replay, retained voices=0 and worker joined; synthetic observation, not live-server/UI or acoustic proof");
}
fn wait(runtime: &mut BattleRuntime, predicate: impl Fn(&Status) -> bool) -> Status {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let status = runtime.poll();
        if predicate(&status) {
            return status;
        }
        assert!(
            Instant::now() < end,
            "canonical worker did not converge: {status:?}"
        );
        thread::sleep(Duration::from_millis(1));
    }
}
fn ready(runtime: &mut BattleRuntime, clock: i64) -> Arc<Surface> {
    let Status::Ready(surface) = wait(
        runtime,
        |s| matches!(s, Status::Ready(p) if p.observation.data().world.loading_elapsed_nanos == clock),
    ) else {
        unreachable!()
    };
    surface
}
fn retire(runtime: &mut BattleRuntime) {
    let end = Instant::now() + Duration::from_secs(5);
    while !runtime.is_quiescent() {
        runtime.poll();
        assert!(Instant::now() < end, "canonical handles not joined");
        thread::sleep(Duration::from_millis(1));
    }
}
struct Release(Arc<AtomicBool>);
impl Drop for Release {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[test]
fn off_thread_resources_exact_clock_updates_immutable_private_surface_and_shutdown() {
    assert!(BattleRuntime::new(PathBuf::from("relative")).is_err());
    let counts = Arc::new(Counts::default());
    let mut runtime = runtime(loader(counts.clone(), None, false, false, false));
    runtime.attach(sample(0)).unwrap();
    assert_eq!(
        counts.loads.load(Ordering::Acquire),
        0,
        "attach does not read resources"
    );
    let first = ready(&mut runtime, 0);
    runtime.update(sample(50)).unwrap();
    let second = ready(&mut runtime, 50);
    assert_eq!(first.bgra[0], 0);
    assert_eq!(second.bgra[0], 50);
    assert_eq!(
        first.observation.personal().revision(),
        second.observation.personal().revision()
    );
    assert!(counts.off_thread.load(Ordering::Acquire));
    assert_eq!(counts.loads.load(Ordering::Acquire), 1);
    assert!(runtime.update(sample(0)).is_err());
    assert_eq!(
        runtime.observation().unwrap().data(),
        second.observation.data()
    );
    runtime.shutdown();
    retire(&mut runtime);
    assert_eq!(counts.active.load(Ordering::Acquire), 0);
    assert!(runtime.attach(sample(100)).is_err());
    assert!(runtime.update(sample(100)).is_err());
    assert!(matches!(runtime.poll(), Status::Idle));
}

#[test]
fn foreign_owner_epoch_channel_world_manifest_and_abort_cannot_rebind() {
    let mut runtime = runtime(loader(
        Arc::new(Counts::default()),
        None,
        false,
        false,
        false,
    ));
    let original = sample(10);
    runtime.attach(original.clone()).unwrap();
    for case in 0..6 {
        let mut foreign = original.data().clone();
        match case {
            0 => {
                foreign.personal.player.source = 8;
                foreign.personal.entity = 2;
            }
            1 => foreign.personal.player.epoch = 20,
            2 => foreign.personal.player.channel = 1,
            3 => foreign.world.identity.match_id += 1,
            4 => foreign.manifest.digest[0] ^= 1,
            5 => {
                foreign.personal.player.epoch = 20;
                foreign.world.phase = Phase::Aborted;
                foreign.world.abort = d::Abort::ServerAbort;
            }
            _ => unreachable!(),
        }
        assert!(runtime.update(observation(foreign)).is_err(), "case {case}");
        assert_eq!(runtime.observation().unwrap().data(), original.data());
    }
    assert!(runtime.attach(original).is_err());
    runtime.shutdown();
    retire(&mut runtime);
}

#[test]
fn terminal_abort_removes_pixels_and_rejects_resurrection_until_explicit_clear() {
    let mut runtime = runtime(loader(
        Arc::new(Counts::default()),
        None,
        false,
        false,
        false,
    ));
    runtime.attach(sample(0)).unwrap();
    let kept = ready(&mut runtime, 0);
    let mut aborted = sample(1).data().clone();
    aborted.world.revision += 1;
    aborted.world.phase = Phase::Aborted;
    aborted.world.abort = d::Abort::ServerAbort;
    let aborted = observation(aborted);
    runtime.update(aborted.clone()).unwrap();
    assert!(matches!(runtime.poll(), Status::Retired));
    assert!(runtime.published.is_none());
    assert!(runtime.update(sample(2)).is_err());
    runtime.update(aborted).unwrap();
    retire(&mut runtime);
    assert_eq!(kept.observation.data().world.phase, Phase::Loading);
    runtime.clear();
    runtime.attach(sample(2)).unwrap();
    ready(&mut runtime, 2);
    runtime.shutdown();
    retire(&mut runtime);
}

#[test]
fn cancellation_during_blocked_load_does_not_publish_or_detach() {
    let counts = Arc::new(Counts::default());
    let hold = Arc::new(AtomicBool::new(true));
    let mut runtime = runtime(loader(
        counts.clone(),
        Some(hold.clone()),
        false,
        false,
        false,
    ));
    let release = Release(hold); // release before runtime Drop on assertion unwind
    runtime.attach(sample(0)).unwrap();
    wait(&mut runtime, |_| counts.loads.load(Ordering::Acquire) == 1);
    let start = Instant::now();
    runtime.shutdown();
    assert!(matches!(runtime.poll(), Status::Stopping));
    assert!(start.elapsed() < Duration::from_millis(200));
    assert!(!runtime.is_quiescent());
    drop(release);
    retire(&mut runtime);
    assert_eq!(counts.renders.load(Ordering::Acquire), 0);
    assert_eq!(counts.active.load(Ordering::Acquire), 0);
}

#[test]
fn rapid_clear_and_reattach_keep_two_workers_and_only_latest_pending_input() {
    let counts = Arc::new(Counts::default());
    let hold = Arc::new(AtomicBool::new(true));
    let mut runtime = runtime(loader(
        counts.clone(),
        Some(hold.clone()),
        false,
        false,
        false,
    ));
    let release = Release(hold);
    runtime.attach(sample(0)).unwrap();
    wait(&mut runtime, |_| counts.loads.load(Ordering::Acquire) == 1);
    runtime.clear();
    runtime.attach(sample(1)).unwrap();
    wait(&mut runtime, |_| counts.loads.load(Ordering::Acquire) == 2);
    for clock in 2..20 {
        runtime.clear();
        runtime.attach(sample(clock)).unwrap();
        runtime.poll();
    }
    assert_eq!(counts.loads.load(Ordering::Acquire), 2);
    assert_eq!(counts.maximum.load(Ordering::Acquire), 2);
    assert!(runtime.published.is_none());
    drop(release);
    ready(&mut runtime, 19);
    assert_eq!(counts.loads.load(Ordering::Acquire), 3);
    assert_eq!(counts.maximum.load(Ordering::Acquire), 2);
    runtime.shutdown();
    retire(&mut runtime);
}

#[test]
fn blocked_mailboxes_do_not_block_ui_submit_poll_or_forget_latest_clock() {
    let mut runtime = runtime(loader(
        Arc::new(Counts::default()),
        None,
        false,
        false,
        false,
    ));
    runtime.attach(sample(0)).unwrap();
    ready(&mut runtime, 0);
    let inbox = runtime.active.as_ref().unwrap().input.clone();
    let outbox = runtime.active.as_ref().unwrap().output.clone();
    let input_lock = inbox.lock().unwrap();
    let output_lock = outbox.lock().unwrap();
    let start = Instant::now();
    for i in 1..50 {
        runtime.update(sample(i)).unwrap();
        runtime.poll();
        let progress = runtime.diagnostic_progress();
        assert_eq!(progress.desired, Some(i as u64 + 1));
        assert_eq!(progress.pending, progress.desired);
        assert_eq!(progress.published, Some(1));
        assert_eq!(
            progress.active,
            Some(diagnostics_tests::Worker {
                current_attachment: true,
                cancelled: false,
                finished: false,
                input: diagnostics_tests::Mailbox::Busy,
                output: diagnostics_tests::Mailbox::Busy,
            })
        );
        assert!(progress.retiring.is_none() && !progress.closing && !progress.failed);
    }
    assert!(start.elapsed() < Duration::from_millis(200));
    assert_eq!(
        runtime
            .pending
            .as_ref()
            .unwrap()
            .observation
            .data()
            .world
            .loading_elapsed_nanos,
        49
    );
    drop(output_lock);
    drop(input_lock);
    ready(&mut runtime, 49);
    let progress = runtime.diagnostic_progress();
    assert_eq!(progress.desired, Some(50));
    assert_eq!(progress.pending, None);
    assert_eq!(progress.published, Some(50));
    runtime.shutdown();
    retire(&mut runtime);
    assert_eq!(
        runtime.diagnostic_progress(),
        diagnostics_tests::Progress {
            desired: None,
            pending: None,
            published: None,
            active: None,
            retiring: None,
            closing: true,
            failed: false,
        }
    );
}

#[test]
fn renderer_mismatch_and_panic_fail_closed_without_retry_and_join_all_handles() {
    for (invalid, panic) in [(true, false), (false, true)] {
        let counts = Arc::new(Counts::default());
        let mut runtime = runtime(loader(counts.clone(), None, false, invalid, panic));
        runtime.attach(sample(0)).unwrap();
        wait(&mut runtime, |s| matches!(s, Status::Failed(_)));
        assert!(runtime.published.is_none());
        for _ in 0..10 {
            runtime.poll();
        }
        assert_eq!(counts.loads.load(Ordering::Acquire), 1);
        assert!(runtime.update(sample(1)).is_err());
        assert!(runtime.is_quiescent());
        runtime.shutdown();
        retire(&mut runtime);
        assert_eq!(counts.active.load(Ordering::Acquire), 0);
    }
}

#[test]
fn raw_resource_errors_are_redacted_and_require_explicit_retry() {
    let counts = Arc::new(AtomicUsize::new(0));
    let c = counts.clone();
    let mut runtime = runtime(Arc::new(move |_, _, _| {
        c.fetch_add(1, Ordering::AcqRel);
        anyhow::bail!("sensitive local archive path")
    }));
    runtime.attach(sample(0)).unwrap();
    let status = wait(&mut runtime, |s| matches!(s, Status::Failed(_)));
    assert!(matches!(
        status,
        Status::Failed("canonical shared resources could not be loaded")
    ));
    for _ in 0..5 {
        runtime.poll();
    }
    assert_eq!(counts.load(Ordering::Acquire), 1);
    runtime.clear();
    runtime.attach(sample(0)).unwrap();
    wait(&mut runtime, |s| matches!(s, Status::Failed(_)));
    assert_eq!(counts.load(Ordering::Acquire), 2);
    runtime.shutdown();
    retire(&mut runtime);
}

#[test]
fn legitimate_resource_transition_reloads_on_worker_then_publishes_one_snapshot() {
    let counts = Arc::new(Counts::default());
    let mut runtime = runtime(loader(counts.clone(), None, true, false, false));
    runtime.attach(sample(0)).unwrap();
    let kept = ready(&mut runtime, 0);
    runtime.update(sample(50)).unwrap();
    ready(&mut runtime, 50);
    assert_eq!(counts.loads.load(Ordering::Acquire), 2);
    assert_eq!(kept.observation.data().world.loading_elapsed_nanos, 0);
    runtime.update(sample(60)).unwrap();
    ready(&mut runtime, 60);
    assert_eq!(counts.loads.load(Ordering::Acquire), 2);
    runtime.shutdown();
    retire(&mut runtime);
}

#[test]
fn drop_owns_and_joins_render_worker_without_polling_shutdown() {
    let counts = Arc::new(Counts::default());
    let mut runtime = runtime(loader(counts.clone(), None, false, false, false));
    runtime.attach(sample(0)).unwrap();
    ready(&mut runtime, 0);
    drop(runtime);
    assert_eq!(counts.active.load(Ordering::Acquire), 0);
}

#[test]
fn mailbox_publication_allows_lag_without_clock_rollback_future_or_old_attachment() {
    let mut runtime = runtime(loader(
        Arc::new(Counts::default()),
        None,
        false,
        false,
        false,
    ));
    runtime.attach(sample(0)).unwrap();
    let first = runtime.desired.clone().unwrap();
    runtime.update(sample(50)).unwrap();
    let second = runtime.desired.clone().unwrap();
    runtime.update(sample(60)).unwrap();
    let outbox = Arc::new(Mutex::new(None));
    // No actual thread: directly exercise the nonblocking publication gate.
    runtime.active = Some(Job {
        attachment: first.attachment.clone(),
        cancel: Arc::new(AtomicBool::new(false)),
        input: Arc::new(Mutex::new(first.clone())),
        output: outbox.clone(),
        handle: None,
    });
    let put = |input: Arc<Submission>| {
        *outbox.lock().unwrap() = Some(Arc::new(Output {
            surface: Arc::new(fake_surface(input.observation.clone())),
            audio: PreparedAudio::empty(
                super::super::preferences::SharedPreferences::default().snapshot(),
            ),
            input,
            resources: None,
        }));
    };
    put(first.clone());
    runtime.read_output();
    assert_eq!(runtime.published.as_ref().unwrap().input.ordinal, 1);
    put(second);
    runtime.read_output();
    assert_eq!(runtime.published.as_ref().unwrap().input.ordinal, 2);
    put(first);
    runtime.read_output();
    assert_eq!(runtime.published.as_ref().unwrap().input.ordinal, 2);
    let desired = runtime.desired.as_ref().unwrap();
    put(Arc::new(Submission {
        attachment: desired.attachment.clone(),
        ordinal: desired.ordinal + 1,
        observation: sample(70),
    }));
    runtime.read_output();
    assert_eq!(runtime.published.as_ref().unwrap().input.ordinal, 2);
    put(Arc::new(Submission {
        attachment: Attachment::new(SharedPreferences::default().snapshot()),
        ordinal: 3,
        observation: sample(60),
    }));
    runtime.read_output();
    assert_eq!(runtime.published.as_ref().unwrap().input.ordinal, 2);
    runtime.clear();
    runtime.read_output();
    assert!(runtime.published.is_none());
}
