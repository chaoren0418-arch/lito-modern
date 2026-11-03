//! Fire Storm authority/presentation regressions. The pure tests never read
//! resources; the ignored test below uses the actual SharedRenderer, not a
//! substitute sprite path. No listener, database, GUI, or audio device.
use super::*;
use crate::modern_lito::{fire_storm_protocol as fire, honor_protocol as honor};

fn input(reply: &combat_protocol::Reply, at: Instant) -> Arc<Input> {
    let frame = reply.frame.as_ref().unwrap();
    let mut input = Input::with_combat(
        Arc::new(frame.world.clone()),
        1,
        Some(Arc::new(frame.clone())),
    )
    .unwrap();
    input.received = at;
    Arc::new(input)
}
fn add_fire(mut reply: combat_protocol::Reply) -> combat_protocol::Reply {
    let frame = reply.frame.as_mut().unwrap();
    frame.fires = frame
        .world
        .actors
        .iter()
        .map(|a| {
            fire::State::initial(
                a.entity_id,
                if a.identity().character().is_some() {
                    48
                } else {
                    0
                },
            )
        })
        .collect();
    reply.validate().unwrap();
    reply
}
fn initial() -> combat_protocol::Reply {
    add_fire(honor::tests::initial())
}

/// A complete server-committed no-target cast, not a local target selector.
fn cast(prior: &combat_protocol::Reply) -> combat_protocol::Reply {
    let mut reply = prior.clone();
    let frame = reply.frame.as_mut().unwrap();
    frame.world.revision += 1;
    frame.world.casts[0].request_sequence += 1;
    let state = &mut frame.fires[0];
    state.count += 1;
    state.cast_revision = frame.world.revision;
    state.mp_spent += i64::from(state.mp_cost);
    state.last_cast_sequence = frame.world.casts[0].request_sequence;
    state.from = frame.world.actors[0].position;
    state.targets.clear();
    frame.vitals[0].mp -= i64::from(state.mp_cost);
    reply.validate().unwrap();
    assert!(combat_protocol::follows(Some(prior), &reply));
    reply
}
fn attack(prior: &combat_protocol::Reply) -> combat_protocol::Reply {
    let mut reply = prior.clone();
    let frame = reply.frame.as_mut().unwrap();
    frame.world.revision += 1;
    frame.vitals[0].attack_sequence += 1;
    frame.vitals[0].attack_revision = frame.world.revision;
    frame.vitals[0].last_target_entity_id = 2;
    reply.validate().unwrap();
    assert!(combat_protocol::follows(Some(prior), &reply));
    reply
}

#[test]
fn coalesced_fire_replaces_old_attack_pose_without_a_second_hp_event() {
    let before = initial();
    let attacked = attack(&before);
    let fired = cast(&attacked);
    let now = Instant::now();
    let mut timeline = Timeline::new(input(&before, now));
    let at = now + Duration::from_millis(100);
    timeline.update(input(&fired, at)).unwrap();
    let scene = timeline.snapshot(at).unwrap();
    assert_eq!(
        scene.actors()[0].action,
        BaseAction::Idle,
        "new fire body override must not resurrect an older melee attack"
    );
    assert!(timeline.actors[&1].pose.is_none());
    assert!(timeline.actors[&1].walk_until.is_none());
    assert_eq!(timeline.actors[&1].action_started, at);
    let history = timeline.input.combat.as_ref().unwrap().clone();
    let frozen_start = timeline.actors[&1].action_started;
    timeline
        .update(input(&fired, at + Duration::from_secs(1)))
        .unwrap();
    assert_eq!(timeline.actors[&1].action_started, frozen_start);
    for elapsed in [
        Duration::ZERO,
        Duration::from_millis(875),
        Duration::from_secs(3),
    ] {
        let _ = timeline.snapshot(at + elapsed).unwrap();
    }
    assert_eq!(
        timeline.input.combat.as_ref().unwrap().as_ref(),
        history.as_ref()
    );
    let a = attacked.frame.as_ref().unwrap();
    let b = fired.frame.as_ref().unwrap();
    assert_eq!(
        a.vitals
            .iter()
            .map(|v| (v.hp, v.hurt_revision, v.death_revision))
            .collect::<Vec<_>>(),
        b.vitals
            .iter()
            .map(|v| (v.hp, v.hurt_revision, v.death_revision))
            .collect::<Vec<_>>()
    );
    assert_eq!(b.vitals[0].mp, a.vitals[0].mp - 48);
    assert!(b.fires[0].targets.is_empty());
}

#[test]
fn death_respawn_and_new_generation_do_not_replay_retained_fire_history() {
    let before = initial();
    let fired = cast(&before);
    let dead = honor::tests::died(&fired, 1);
    let alive = honor::tests::respawn(&dead, 1);
    let now = Instant::now();
    let mut timeline = Timeline::new(input(&before, now));
    timeline
        .update(input(&fired, now + Duration::from_millis(100)))
        .unwrap();
    timeline
        .update(input(&dead, now + Duration::from_millis(200)))
        .unwrap();
    assert_eq!(
        timeline
            .snapshot(now + Duration::from_secs(30))
            .unwrap()
            .actors()[0]
            .action,
        BaseAction::Death,
        "elapsed time cannot locally respawn the caster"
    );
    let at = now + Duration::from_secs(31);
    timeline.update(input(&alive, at)).unwrap();
    assert_eq!(
        timeline.snapshot(at).unwrap().actors()[0].action,
        BaseAction::Idle
    );
    assert!(timeline.actors[&1].pose.is_none());
    assert_eq!(
        alive.frame.as_ref().unwrap().fires,
        fired.frame.as_ref().unwrap().fires
    );
    assert_eq!(
        alive.frame.as_ref().unwrap().vitals[0].mp,
        fired.frame.as_ref().unwrap().vitals[0].mp
    );
    let mut coalesced = Timeline::new(input(&before, now));
    coalesced.update(input(&alive, at)).unwrap();
    assert_eq!(
        coalesced.snapshot(at).unwrap().actors()[0].action,
        BaseAction::Idle
    );
    assert!(coalesced.actors[&1].pose.is_none());
    let mut other = alive.clone();
    other.frame.as_mut().unwrap().world.generation += 1;
    other.frame.as_mut().unwrap().world.room.summary.revision += 1;
    other.echo.generation += 1;
    other.validate().unwrap();
    assert!(
        timeline.update(input(&other, at)).is_err(),
        "a generation cannot reuse the old timeline"
    );
    let mut fresh = Timeline::new(input(&other, at));
    assert_eq!(
        fresh.snapshot(at).unwrap().actors()[0].action,
        BaseAction::Idle
    );
    assert!(fresh.actors[&1].pose.is_none());
}

#[test]
fn two_receipts_are_one_cast_and_effect_hit_cannot_reapply_or_restart_hurt() {
    let mut before = initial();
    let f = before.frame.as_mut().unwrap();
    let origin = f.world.actors[0].position;
    f.world.actors[1].position = WorldCell {
        x: origin.x + 1,
        y: origin.y,
    };
    f.honor.as_mut().unwrap().objectives[1].position = WorldCell {
        x: origin.x + 2,
        y: origin.y,
    };
    before.validate().unwrap();
    let mut fired = cast(&before);
    let f = fired.frame.as_mut().unwrap();
    let revision = f.world.revision;
    let hp = f.vitals[1].hp;
    f.vitals[1].hp -= 10;
    f.vitals[1].hurt_revision = revision;
    f.regen[1].damage_loss += 10;
    let tower = &mut f.honor.as_mut().unwrap().objectives[1];
    let tower_hp = tower.hp;
    tower.hp -= 10;
    tower.damage_loss += 10;
    tower.state_revision = revision;
    tower.hurt_revision = revision;
    tower.action = honor::action_for_hp(tower.hp, tower.max_hp);
    tower.last_actor_entity_id = 1;
    tower.last_kind = honor::Interaction::FireStorm;
    f.fires[0].targets = vec![
        fire::Target {
            target_entity_id: 2,
            hp_before: hp,
            hp_after: hp - 10,
            applied_damage: 10,
            damage_loss_after: 10,
            respawn_count: 0,
            position: f.world.actors[1].position,
        },
        fire::Target {
            target_entity_id: 4,
            hp_before: tower_hp,
            hp_after: tower_hp - 10,
            applied_damage: 10,
            damage_loss_after: 10,
            respawn_count: 0,
            position: tower.position,
        },
    ];
    fired.validate().unwrap();
    assert!(combat_protocol::follows(Some(&before), &fired));
    let now = Instant::now();
    let mut timeline = Timeline::new(input(&before, now));
    let at = now + Duration::from_millis(10);
    timeline.update(input(&fired, at)).unwrap();
    assert_eq!(
        timeline.snapshot(at).unwrap().actors()[1].action,
        BaseAction::Damage
    );
    let hurt_start = timeline.actors[&2].action_started;
    let frozen = timeline.input.combat.as_ref().unwrap().clone();
    for ms in [875, 2125, 2375, 5000] {
        let _ = timeline.snapshot(at + Duration::from_millis(ms)).unwrap();
        assert_eq!(timeline.actors[&2].action_started, hurt_start);
    }
    assert_eq!(
        timeline.input.combat.as_ref().unwrap().as_ref(),
        frozen.as_ref()
    );
    assert_eq!(frozen.fires[0].count, 1);
    assert_eq!(frozen.fires[0].targets.len(), 2);
    assert_eq!(frozen.fires[0].mp_spent, 48);
}

fn sound710(pixels: &Pixels) -> Vec<u64> {
    pixels
        .audio
        .as_ref()
        .unwrap()
        .events
        .iter()
        .filter(|event| event.group.candidates() == [710])
        .map(|event| event.sequence)
        .collect()
}

/// Only remove the1819 group, retaining the same caster override, scene,
/// camera, HP bars, objective art, elapsed times and authority Frame.
fn isolated(
    renderer: &mut SharedRenderer,
    scene: &SceneSnapshot,
    frame: &combat_protocol::Frame,
    cancel: &AtomicBool,
    at: Instant,
) -> (Pixels, Pixels) {
    let actual = renderer.render_at(scene, Some(frame), cancel, at).unwrap();
    let effects = std::mem::take(&mut renderer.fire_storm.as_mut().unwrap().effects);
    assert_eq!(
        effects.len(),
        1,
        "one cast owns one four-copy group, including zero targets"
    );
    let control = renderer.render_at(scene, Some(frame), cancel, at).unwrap();
    assert!(
        renderer.fire_storm.as_ref().unwrap().effects.is_empty(),
        "same revision cannot respawn a taken effect"
    );
    assert_eq!(actual.actors, control.actors);
    assert_eq!(actual.objectives, control.objectives);
    assert_eq!(
        sound710(&actual),
        sound710(&control),
        "control must not duplicate710"
    );
    renderer.fire_storm.as_mut().unwrap().effects = effects;
    (actual, control)
}

#[test]
#[ignore = "LITO_SHARED_RESOURCE_ROOT; actual SharedRenderer six-sector532/caster18/1819 four-copy layer, fixed clocks and sound log; no device/SQL/GUI"]
fn actual_fire_group_zero_targets_terminal_dwell_and_history_are_isolated() {
    let root =
        std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared resource root");
    let cancel = AtomicBool::new(false);
    let before = add_fire(super::honor_runtime_tests::actual_fixture());
    let initial_frame = before.frame.as_ref().unwrap();
    let mut renderer = load_shared_renderer(
        Path::new(&root),
        &initial_frame.world,
        Some(initial_frame),
        &cancel,
    )
    .unwrap();
    let now = Instant::now();
    let mut timeline = Timeline::new(input(&before, now));
    renderer
        .render_at(
            &timeline.snapshot(now).unwrap(),
            Some(initial_frame),
            &cancel,
            now,
        )
        .unwrap();
    assert!(renderer.fire_storm.as_ref().unwrap().active.is_empty());
    let fired = cast(&before);
    let frame = fired.frame.as_ref().unwrap();
    let frozen = frame.clone();
    assert!(frame.fires[0].targets.is_empty());
    let cast_at = now + Duration::from_millis(100);
    timeline.update(input(&fired, cast_at)).unwrap();
    let first_scene = timeline.snapshot(cast_at).unwrap();
    let appearance = first_scene
        .actors()
        .iter()
        .find(|a| a.entity_id == 1)
        .unwrap()
        .appearance;
    let fire_assets = &renderer.fire_storm.as_ref().unwrap().assets;
    let delay = fire_assets.cast_launch_delay(appearance).unwrap();
    assert!(
        delay > Duration::ZERO,
        "this independently qualified wizard fixture has a nonzero caster Hit delay"
    );
    assert_eq!(
        fire_assets.effect_terminal_start(),
        Duration::from_millis(2125)
    );
    assert_eq!(fire_assets.effect_duration(), Duration::from_millis(2375));
    let before_hit = cast_at + delay - Duration::from_nanos(1);
    renderer
        .render_at(&first_scene, Some(frame), &cancel, cast_at)
        .unwrap();
    let pre_scene = timeline.snapshot(before_hit).unwrap();
    let pre = renderer
        .render_at(&pre_scene, Some(frame), &cancel, before_hit)
        .unwrap();
    assert!(renderer.fire_storm.as_ref().unwrap().effects.is_empty());
    assert!(
        sound710(&pre).is_empty(),
        "constructor/pending cast must not start710 before caster Hit"
    );
    let launched_at = cast_at + delay;
    let launch_scene = timeline.snapshot(launched_at).unwrap();
    let launch = renderer
        .render_at(&launch_scene, Some(frame), &cancel, launched_at)
        .unwrap();
    assert_eq!(renderer.fire_storm.as_ref().unwrap().effects.len(), 1);
    assert_eq!(
        renderer.fire_storm.as_ref().unwrap().effects[0].position,
        frame.fires[0].from
    );
    assert_eq!(
        renderer.fire_storm.as_ref().unwrap().effects[0].started,
        launched_at
    );
    let ids = sound710(&launch);
    assert_eq!(
        ids.len(),
        1,
        "four native copies share one same-ID-suppressed sound owner"
    );
    let repeated = renderer
        .render_at(&launch_scene, Some(frame), &cancel, launched_at)
        .unwrap();
    assert_eq!(launch.bgra, repeated.bgra);
    assert_eq!(sound710(&repeated), ids);
    let visible_at = launched_at + Duration::from_millis(125);
    let visible_scene = timeline.snapshot(visible_at).unwrap();
    let (visible, control) = isolated(&mut renderer, &visible_scene, frame, &cancel, visible_at);
    assert_ne!(
        visible.bgra, control.bgra,
        "same real scene/body/HP/camera/time; only1819 changed"
    );
    assert_eq!(sound710(&visible), ids);
    // Independently compose four literal offsets onto the control canvas, then
    // restore the same post-effect bars. The modern floor-screen policy has a
    // separate1024-pair C# oracle; also reject the former opaque-overlay result.
    let mut expected = Canvas::new(WIDTH, HEIGHT).unwrap();
    expected.copy_from(&renderer.canvas).unwrap(); // canvas is the control render
    let mut opaque = Canvas::new(WIDTH, HEIGHT).unwrap();
    opaque.copy_from(&renderer.canvas).unwrap();
    let anchor = control
        .actors
        .iter()
        .find(|a| a.entity_id == 1)
        .unwrap()
        .anchor;
    let source = renderer
        .fire_storm
        .as_ref()
        .unwrap()
        .assets
        .effect_frame(Duration::from_millis(125))
        .unwrap();
    for (dx, dy) in [(48, 0), (-48, 0), (0, 24), (0, -24)] {
        expected
            .blit_screen_rgb555(
                &source.frame.rgba,
                source.frame.width,
                source.frame.height,
                anchor.x + dx + source.frame.x_offset,
                anchor.y + dy + source.frame.y_offset,
            )
            .unwrap();
        opaque
            .blit_rgba(
                &source.frame.rgba,
                source.frame.width,
                source.frame.height,
                anchor.x + dx + source.frame.x_offset,
                anchor.y + dy + source.frame.y_offset,
            )
            .unwrap();
    }
    draw_health_bars(&mut expected, &control.actors, &frame.vitals).unwrap();
    draw_health_bars(&mut opaque, &control.actors, &frame.vitals).unwrap();
    for objective in &frame.honor.as_ref().unwrap().objectives {
        let placement = control
            .objectives
            .iter()
            .find(|o| o.entity_id == objective.entity_id)
            .unwrap();
        draw_objective_health_bar(&mut expected, placement, objective).unwrap();
        draw_objective_health_bar(&mut opaque, placement, objective).unwrap();
    }
    let mut expected_bgra = Vec::new();
    expected.copy_bgra(&mut expected_bgra).unwrap();
    assert_eq!(visible.bgra.as_ref(), expected_bgra.as_slice());
    let mut opaque_bgra = Vec::new();
    opaque.copy_bgra(&mut opaque_bgra).unwrap();
    assert_ne!(
        visible.bgra.as_ref(),
        opaque_bgra.as_slice(),
        "actual1819 attr8 must not regress to the dark opaque four-copy overlay"
    );
    let at_hit = launched_at + Duration::from_millis(875);
    let hurt_before = timeline
        .actors
        .iter()
        .map(|(id, a)| (*id, a.action_started))
        .collect::<Vec<_>>();
    let hit_scene = timeline.snapshot(at_hit).unwrap();
    let hit = renderer
        .render_at(&hit_scene, Some(frame), &cancel, at_hit)
        .unwrap();
    assert!(
        sound710(&hit).is_empty(),
        "expired710 does not replay on the effect's own Hit"
    );
    assert_eq!(
        timeline
            .actors
            .iter()
            .map(|(id, a)| (*id, a.action_started))
            .collect::<Vec<_>>(),
        hurt_before
    );
    assert_eq!(
        frame, &frozen,
        "render cues cannot apply HP/MP/hurt changes"
    );
    for elapsed in [
        Duration::from_millis(2125),
        Duration::from_nanos(2_374_999_999),
    ] {
        let at = launched_at + elapsed;
        let scene = timeline.snapshot(at).unwrap();
        let (actual, control) = isolated(&mut renderer, &scene, frame, &cancel, at);
        let terminal = renderer
            .fire_storm
            .as_ref()
            .unwrap()
            .assets
            .effect_frame(elapsed)
            .unwrap();
        assert_eq!(
            (terminal.source_frame, terminal.subframe, terminal.finished),
            (0, 13, false)
        );
        // Coverage alone no longer implies a visible tail: covered black is
        // neutral under screen. Compare the actual terminal composition rather
        // than assuming every covered source pixel must obscure its backdrop.
        let mut terminal_expected = Canvas::new(WIDTH, HEIGHT).unwrap();
        terminal_expected.copy_from(&renderer.canvas).unwrap(); // identical no-effect control
        let terminal_anchor = control
            .actors
            .iter()
            .find(|a| a.entity_id == 1)
            .unwrap()
            .anchor;
        for (dx, dy) in [(48, 0), (-48, 0), (0, 24), (0, -24)] {
            terminal_expected
                .blit_screen_rgb555(
                    &terminal.frame.rgba,
                    terminal.frame.width,
                    terminal.frame.height,
                    terminal_anchor.x + dx + terminal.frame.x_offset,
                    terminal_anchor.y + dy + terminal.frame.y_offset,
                )
                .unwrap();
        }
        draw_health_bars(&mut terminal_expected, &control.actors, &frame.vitals).unwrap();
        for objective in &frame.honor.as_ref().unwrap().objectives {
            let placement = control
                .objectives
                .iter()
                .find(|o| o.entity_id == objective.entity_id)
                .unwrap();
            draw_objective_health_bar(&mut terminal_expected, placement, objective).unwrap();
        }
        let mut terminal_bgra = Vec::new();
        terminal_expected.copy_bgra(&mut terminal_bgra).unwrap();
        assert_eq!(
            actual.bgra.as_ref(),
            terminal_bgra.as_slice(),
            "the verified terminal frame is still composited throughout its six-tick dwell"
        );
    }
    let ended_at = launched_at + Duration::from_millis(2375);
    let ended_scene = timeline.snapshot(ended_at).unwrap();
    let ended = renderer
        .render_at(&ended_scene, Some(frame), &cancel, ended_at)
        .unwrap();
    assert!(renderer.fire_storm.as_ref().unwrap().effects.is_empty());
    assert!(
        renderer.fire_storm.as_ref().unwrap().active.is_empty(),
        "this caster body has also completed by the effect terminal"
    );
    let retired_layer = renderer.fire_storm.take();
    let no_layer = renderer
        .render_at(&ended_scene, Some(frame), &cancel, ended_at)
        .unwrap();
    assert_eq!(
        ended.actors, no_layer.actors,
        "removing a completed family must not change the body"
    );
    assert_eq!(ended.objectives, no_layer.objectives);
    assert_eq!(
        ended.bgra, no_layer.bgra,
        "terminal expiry exactly restores the same state without this layer"
    );
    renderer.fire_storm = retired_layer;
    let again = renderer
        .render_at(&ended_scene, Some(frame), &cancel, ended_at)
        .unwrap();
    assert_eq!(ended.bgra, again.bgra);
    assert!(sound710(&again).is_empty());
    let dead = honor::tests::died(&fired, 1);
    let dead_at = ended_at + Duration::from_millis(100);
    timeline.update(input(&dead, dead_at)).unwrap();
    renderer
        .render_at(
            &timeline.snapshot(dead_at).unwrap(),
            dead.frame.as_ref(),
            &cancel,
            dead_at,
        )
        .unwrap();
    let alive = honor::tests::respawn(&dead, 1);
    let alive_at = dead_at + Duration::from_secs(5);
    timeline.update(input(&alive, alive_at)).unwrap();
    let scene = timeline.snapshot(alive_at).unwrap();
    let revived = renderer
        .render_at(&scene, alive.frame.as_ref(), &cancel, alive_at)
        .unwrap();
    assert!(renderer.fire_storm.as_ref().unwrap().active.is_empty());
    assert!(renderer.fire_storm.as_ref().unwrap().effects.is_empty());
    assert!(sound710(&revived).is_empty());
    assert_eq!(scene.actors()[0].action, BaseAction::Idle);
    // A new world resource owner baselines retained history; it must not play
    // old710 or old1819, even when the server retains a nonzero cast count.
    let mut other = alive.clone();
    other.echo.generation += 1;
    other.frame.as_mut().unwrap().world.generation += 1;
    other.frame.as_mut().unwrap().world.room.summary.revision += 1;
    other.validate().unwrap();
    let other_frame = other.frame.as_ref().unwrap();
    drop(renderer);
    let mut fresh = load_shared_renderer(
        Path::new(&root),
        &other_frame.world,
        Some(other_frame),
        &cancel,
    )
    .unwrap();
    let mut fresh_time = Timeline::new(input(&other, alive_at));
    let historical = fresh
        .render_at(
            &fresh_time.snapshot(alive_at).unwrap(),
            Some(other_frame),
            &cancel,
            alive_at,
        )
        .unwrap();
    assert!(fresh.fire_storm.as_ref().unwrap().active.is_empty());
    assert!(fresh.fire_storm.as_ref().unwrap().effects.is_empty());
    assert!(historical.audio.as_ref().unwrap().events.is_empty());
    assert_eq!(other_frame.fires[0].count, 1);
    // A newer death BEFORE the pending caster Hit cancels that unlaunched
    // group. Neither the later timer nor an authoritative respawn may revive it.
    let pending = cast(&other);
    let pending_at = alive_at + Duration::from_millis(10);
    fresh_time.update(input(&pending, pending_at)).unwrap();
    fresh
        .render_at(
            &fresh_time.snapshot(pending_at).unwrap(),
            pending.frame.as_ref(),
            &cancel,
            pending_at,
        )
        .unwrap();
    assert!(fresh.fire_storm.as_ref().unwrap().effects.is_empty());
    let interrupted = honor::tests::died(&pending, 1);
    let interrupted_at = pending_at + Duration::from_nanos(1);
    fresh_time
        .update(input(&interrupted, interrupted_at))
        .unwrap();
    fresh
        .render_at(
            &fresh_time.snapshot(interrupted_at).unwrap(),
            interrupted.frame.as_ref(),
            &cancel,
            interrupted_at,
        )
        .unwrap();
    assert!(fresh.fire_storm.as_ref().unwrap().active.is_empty());
    let after_delay = interrupted_at + delay + Duration::from_secs(1);
    let after = fresh
        .render_at(
            &fresh_time.snapshot(after_delay).unwrap(),
            interrupted.frame.as_ref(),
            &cancel,
            after_delay,
        )
        .unwrap();
    assert!(fresh.fire_storm.as_ref().unwrap().effects.is_empty());
    assert!(sound710(&after).is_empty());
    let restored = honor::tests::respawn(&interrupted, 1);
    let restored_at = after_delay + Duration::from_secs(5);
    fresh_time.update(input(&restored, restored_at)).unwrap();
    let after_respawn = fresh
        .render_at(
            &fresh_time.snapshot(restored_at).unwrap(),
            restored.frame.as_ref(),
            &cancel,
            restored_at,
        )
        .unwrap();
    assert!(fresh.fire_storm.as_ref().unwrap().active.is_empty());
    assert!(fresh.fire_storm.as_ref().unwrap().effects.is_empty());
    assert!(sound710(&after_respawn).is_empty());

    // Observe a pending cast, then stall rendering beyond both its first Hit
    // and the complete effect dwell. The same authoritative frame must not
    // become a newly started effect/sound when the worker resumes.
    drop(fresh);
    let mut late_before = before.clone();
    late_before.echo.generation += 2;
    late_before.frame.as_mut().unwrap().world.generation += 2;
    late_before
        .frame
        .as_mut()
        .unwrap()
        .world
        .room
        .summary
        .revision += 2;
    late_before.validate().unwrap();
    let late_initial = late_before.frame.as_ref().unwrap();
    let mut stalled = load_shared_renderer(
        Path::new(&root),
        &late_initial.world,
        Some(late_initial),
        &cancel,
    )
    .unwrap();
    let baseline_at = restored_at + Duration::from_secs(1);
    let mut stalled_time = Timeline::new(input(&late_before, baseline_at));
    let baseline = stalled
        .render_at(
            &stalled_time.snapshot(baseline_at).unwrap(),
            Some(late_initial),
            &cancel,
            baseline_at,
        )
        .unwrap();
    assert!(baseline.audio.as_ref().unwrap().events.is_empty());
    let late_cast = cast(&late_before);
    let late_frame = late_cast.frame.as_ref().unwrap();
    let late_frozen = late_frame.clone();
    let observed_at = baseline_at + Duration::from_millis(10);
    stalled_time.update(input(&late_cast, observed_at)).unwrap();
    let observed = stalled
        .render_at(
            &stalled_time.snapshot(observed_at).unwrap(),
            Some(late_frame),
            &cancel,
            observed_at,
        )
        .unwrap();
    let pending_visual = &stalled.fire_storm.as_ref().unwrap().active[&1];
    assert!(!pending_visual.launched);
    assert_eq!(pending_visual.launch_at, Some(observed_at + delay));
    assert!(stalled.fire_storm.as_ref().unwrap().effects.is_empty());
    assert!(observed.audio.as_ref().unwrap().events.is_empty());
    let resumed_at = observed_at + Duration::from_secs(10);
    stalled_time.update(input(&late_cast, resumed_at)).unwrap();
    let resumed_scene = stalled_time.snapshot(resumed_at).unwrap();
    let resumed = stalled
        .render_at(&resumed_scene, Some(late_frame), &cancel, resumed_at)
        .unwrap();
    assert!(
        stalled.fire_storm.as_ref().unwrap().effects.is_empty(),
        "a missed first-Hit deadline cannot reset a stale effect's age to zero"
    );
    assert!(stalled.fire_storm.as_ref().unwrap().active.is_empty());
    assert!(
        resumed.audio.as_ref().unwrap().events.is_empty(),
        "resuming the same observed cast must not append a fresh710 sequence"
    );
    let expired_family = stalled.fire_storm.take();
    let absent = stalled
        .render_at(&resumed_scene, Some(late_frame), &cancel, resumed_at)
        .unwrap();
    assert_eq!(resumed.actors, absent.actors);
    assert_eq!(resumed.objectives, absent.objectives);
    assert_eq!(
        resumed.bgra, absent.bgra,
        "the fully expired group is pixel-identical to the same scene without the layer"
    );
    stalled.fire_storm = expired_family;
    let repeated = stalled
        .render_at(&resumed_scene, Some(late_frame), &cancel, resumed_at)
        .unwrap();
    assert_eq!(resumed.bgra, repeated.bgra);
    assert!(repeated.audio.as_ref().unwrap().events.is_empty());
    assert_eq!(late_frame, &late_frozen);
    assert_hurt_keeps_pending_effect(
        stalled.fire_storm.as_mut().unwrap(),
        &stalled.map,
        late_before.frame.as_ref().unwrap(),
        late_frame,
    );
}

/// Isolated pose arbitration, not a fabricated authority packet. Both the PC
/// and NPC real-resource tests run this against their own qualified assets.
pub(super) fn assert_hurt_keeps_pending_effect(
    fire: &mut FirePresentation,
    map: &WorldMap,
    before: &combat_protocol::Frame,
    hit: &combat_protocol::Frame,
) {
    let event = crate::modern_lito::fire_storm_presentation::events(hit)
        .into_iter()
        .find(|s| s.cast_revision > 0)
        .unwrap();
    let member = hit.world.member(event.entity_id).unwrap();
    let appearance = BaseAppearance {
        class: member.class as u8,
        sex: member.sex,
    };
    let delay = fire.assets.cast_launch_delay(appearance).unwrap();
    assert!(
        delay > Duration::ZERO,
        "this fixture must exercise pending Hit"
    );
    for coalesced in [false, true] {
        fire.identity = (hit.world.match_id, hit.world.generation);
        fire.baseline = crate::modern_lito::fire_storm_presentation::events(before)
            .iter()
            .map(|s| (s.entity_id, s.cast_revision))
            .collect();
        fire.active.clear();
        fire.effects.clear();
        let mut hurt = hit.clone();
        let v = hurt
            .vitals
            .iter_mut()
            .find(|v| v.entity_id == event.entity_id)
            .unwrap();
        v.hurt_revision = hit.world.revision + 1;
        v.hp -= 1;
        let frozen = hurt.clone();
        let at = Instant::now();
        fire.update(if coalesced { &hurt } else { hit }, map, None, at)
            .unwrap();
        fire.update(&hurt, map, None, at + delay / 2).unwrap();
        assert!(!fire.active[&event.entity_id].body_visible);
        assert!(fire.effects.is_empty());
        fire.update(&hurt, map, None, at + delay).unwrap();
        assert_eq!(fire.effects.len(), 1, "coalesced={coalesced}");
        assert_eq!(fire.effects[0].started, at + delay);
        assert_eq!(fire.effects[0].position, event.from);
        fire.update(&hurt, map, None, at + delay + Duration::from_millis(5))
            .unwrap();
        assert_eq!(fire.effects.len(), 1);
        assert_eq!(hurt, frozen, "visuals cannot reapply damage or spend MP");
    }
}
