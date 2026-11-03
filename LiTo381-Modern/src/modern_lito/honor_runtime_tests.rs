//! Fixed-clock authority/presentation regressions. Only the ignored test reads
//! actual shared assets; no test opens a listener, database or audio device.
use super::*;
use crate::modern_lito::{
    honor_healing_effect, honor_protocol as honor, map532_reference_tests, tower_assets,
    world_scene::ObjectiveDraw,
};

fn input(reply: &combat_protocol::Reply, at: Instant) -> Arc<Input> {
    let frame = reply.frame.as_ref().unwrap();
    let mut value = Input::with_combat(
        Arc::new(frame.world.clone()),
        1,
        Some(Arc::new(frame.clone())),
    )
    .unwrap();
    value.received = at;
    Arc::new(value)
}

fn tower_hit(prior: &combat_protocol::Reply, amount: i64) -> combat_protocol::Reply {
    let mut next = prior.clone();
    let f = next.frame.as_mut().unwrap();
    f.world.revision += 1;
    let v = &mut f.vitals[0];
    v.attack_sequence += 1;
    v.attack_revision = f.world.revision;
    v.last_target_entity_id = 4;
    let tower = &mut f.honor.as_mut().unwrap().objectives[1];
    tower.hp -= amount;
    tower.damage_loss += amount;
    tower.state_revision = f.world.revision;
    tower.hurt_revision = f.world.revision;
    tower.last_actor_entity_id = 1;
    tower.last_kind = honor::Interaction::Physical;
    tower.action = honor::action_for_hp(tower.hp, tower.max_hp);
    next.validate().unwrap();
    assert!(combat_protocol::follows(Some(prior), &next));
    next
}

#[test]
fn wall_clock_cannot_respawn_a_dead_actor_but_authority_can_without_resetting_history() {
    let start = Instant::now();
    let initial = honor::tests::initial();
    let healed = honor::tests::heal(&initial, 0).combat_reply().unwrap();
    let dead = honor::tests::died(&healed, 1);
    let mut timeline = Timeline::new(input(&initial, start));
    timeline
        .update(input(&healed, start + Duration::from_millis(100)))
        .unwrap();
    timeline
        .update(input(&dead, start + Duration::from_secs(1)))
        .unwrap();
    for seconds in [1, 6, 60] {
        assert_eq!(
            timeline
                .snapshot(start + Duration::from_secs(seconds))
                .unwrap()
                .actors()[0]
                .action,
            BaseAction::Death
        );
    }
    let alive = honor::tests::respawn(&dead, 1);
    let at = start + Duration::from_secs(61);
    timeline.update(input(&alive, at)).unwrap();
    assert_eq!(
        timeline.snapshot(at).unwrap().actors()[0].action,
        BaseAction::Idle
    );
    let a = alive.frame.as_ref().unwrap();
    let d = dead.frame.as_ref().unwrap();
    assert_eq!(a.vitals[0].mp, d.vitals[0].mp);
    assert_eq!(a.items, d.items);
    assert_eq!(a.world.casts, d.world.casts);
    assert_eq!(
        a.honor.as_ref().unwrap().heals,
        d.honor.as_ref().unwrap().heals
    );
    assert_eq!(a.vitals[0].death_revision, d.vitals[0].death_revision);
    let baseline = timeline.actors[&1].action_started;
    timeline
        .update(input(&alive, at + Duration::from_secs(1)))
        .unwrap();
    assert_eq!(timeline.actors[&1].action_started, baseline);
    assert_eq!(
        timeline
            .snapshot(at + Duration::from_secs(1))
            .unwrap()
            .actors()[0]
            .action,
        BaseAction::Idle
    );
}

#[test]
fn coalesced_heal_death_and_respawn_do_not_replay_old_poses() {
    let initial = honor::tests::initial();
    let healed = honor::tests::heal(&initial, 0).combat_reply().unwrap();
    let dead = honor::tests::died(&healed, 1);
    let alive = honor::tests::respawn(&dead, 1);
    let start = Instant::now();
    let mut timeline = Timeline::new(input(&initial, start));
    timeline
        .update(input(&alive, start + Duration::from_secs(6)))
        .unwrap();
    let scene = timeline.snapshot(start + Duration::from_secs(6)).unwrap();
    assert!(scene.actors().iter().all(|a| a.action == BaseAction::Idle));
    assert_eq!(
        scene.actors()[0].cell,
        dead.frame.as_ref().unwrap().world.actors[0].position
    );
    assert!(timeline.actors[&1].pose.is_none());
    assert!(timeline.actors[&1].walk_until.is_none());
    assert_eq!(
        alive.frame.as_ref().unwrap().honor.as_ref().unwrap().heals[0].count,
        1
    );
}

#[test]
fn objective_attack_target_never_becomes_a_participant_or_resets_the_other_actor() {
    let initial = honor::tests::initial();
    let hit = tower_hit(&initial, 26);
    let start = Instant::now();
    let mut timeline = Timeline::new(input(&initial, start));
    timeline
        .update(input(&hit, start + Duration::from_millis(100)))
        .unwrap();
    let scene = timeline
        .snapshot(start + Duration::from_millis(100))
        .unwrap();
    assert_eq!(
        scene
            .actors()
            .iter()
            .map(|a| a.entity_id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(scene.actors()[0].action, BaseAction::Attack);
    assert_eq!(scene.actors()[1].action, BaseAction::Idle);
    assert_eq!(timeline.actors[&2].action_started, start);
    assert_eq!(
        honor::target(hit.frame.as_ref().unwrap(), 4),
        Some(honor::Target::Objective(4))
    );
    assert!(!timeline.actors.contains_key(&4));
}

pub(super) fn actual_fixture() -> combat_protocol::Reply {
    let mut reply = honor::tests::initial();
    let frame = reply.frame.as_mut().unwrap();
    let hex = "b483dc9c8fefefcdbd515680317fc5e44090046f6490e8d165f5f989bf6d5fdc";
    frame.world.manifest = world_protocol::Manifest {
        version: 1,
        map_id: 532,
        digest: std::array::from_fn(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap()),
        sectors: map532_reference_tests::bindings()
            .iter()
            .map(|b| world_protocol::Sector {
                x: b.key.sector_x,
                y: b.key.sector_y,
                byte_length: b.byte_length,
                sha256: b.sha256,
            })
            .collect(),
    };
    // Explicit fixture camps already independently qualified on the full map;
    // these are not claimed to be native production Honor placements.
    frame.world.actors[0].position = WorldCell { x: 32792, y: 32843 };
    frame.world.actors[1].position = WorldCell { x: 32798, y: 32843 };
    let state = frame.honor.as_mut().unwrap();
    state.objectives[0].position = WorldCell { x: 32792, y: 32842 };
    state.objectives[1].position = WorldCell { x: 32798, y: 32842 };
    state.objectives[1].graphics_id = 2669;
    reply.validate().unwrap();
    reply
}

#[test]
#[ignore = "requires LITO_SHARED_RESOURCE_ROOT; true six-sector532/tower/744 compositor and fixed-clock audio log, no audio device"]
fn actual_honor_assets_layers_terminal_hold_and_respawn_are_authority_driven() {
    let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
    let cancel = AtomicBool::new(false);
    let initial = actual_fixture();
    let frame = initial.frame.as_ref().unwrap();
    let mut renderer =
        load_shared_renderer(Path::new(&root), &frame.world, Some(frame), &cancel).unwrap();
    let start = Instant::now();
    let mut timeline = Timeline::new(input(&initial, start));
    let scene = timeline.snapshot(start).unwrap();

    {
        let presentation = renderer.honor.as_ref().unwrap();
        for gfx in tower_assets::GRAPHICS {
            let duration = presentation.assets.state_duration(gfx, 35).unwrap();
            assert!(duration > Duration::ZERO);
            let held = presentation.assets.state_frame(gfx, 35, duration).unwrap();
            let later = presentation
                .assets
                .state_frame(gfx, 35, duration + Duration::from_secs(60))
                .unwrap();
            assert_eq!((held.source_frame, held.finished), (0, true));
            assert_eq!((later.source_frame, later.finished), (0, true));
            assert_eq!(held.frame.rgba, later.frame.rgba);
            let mut playback = presentation.assets.start_state(gfx, 35).unwrap();
            assert_eq!(playback.advance(Duration::ZERO).unwrap().source_frame, 1);
            assert_eq!(playback.advance(duration).unwrap().source_frame, 0);
            assert!(playback.advance(duration).unwrap().crossed_cues.is_empty());
        }
        assert_eq!(
            presentation.healing.profile(),
            honor_healing_effect::HonorHealingProfile::SharedClassicBasicHealV1
        );
        assert_eq!(honor_healing_effect::SOURCE_TEMPLATE_GRAPHIC, 11722);
        assert_eq!(honor_healing_effect::PRESENTATION_GRAPHIC, 744);
        assert_eq!(presentation.healing.duration(), Duration::from_millis(2500));
        assert_eq!(
            presentation
                .healing
                .effect_frame(Duration::ZERO)
                .unwrap()
                .source_frame,
            1
        );
        let end = presentation
            .healing
            .effect_frame(Duration::from_millis(2500))
            .unwrap();
        assert_eq!((end.source_frame, end.finished), (0, true));

        // Exercise the actual scene sort and native RGB555 shadow route, not an
        // independent guessed alpha value or a sprite-only replacement renderer.
        let towers = &frame.honor.as_ref().unwrap().objectives;
        let mut draws: Vec<_> = towers
            .iter()
            .map(|o| {
                let body = presentation
                    .assets
                    .state_frame(o.graphics_id as u32, 0, Duration::ZERO)
                    .unwrap();
                let shadow = presentation
                    .assets
                    .shadow_frame(o.graphics_id as u32, 0, Duration::ZERO)
                    .unwrap();
                ObjectiveDraw {
                    blocking: true,
                    entity_id: o.entity_id,
                    cell: o.position,
                    frame: body.frame,
                    shadow: Some(shadow.frame),
                    graphics_id: body.graphics_id,
                    source_frame: body.source_frame,
                    sprite_name: body.sprite_name,
                    subframe: body.subframe,
                }
            })
            .collect();
        let mut full = Canvas::new(WIDTH, HEIGHT).unwrap();
        let info = renderer
            .assets
            .render_into_objectives(&scene, &mut full, &cancel, &[], &draws)
            .unwrap();
        let mut reverse = Canvas::new(WIDTH, HEIGHT).unwrap();
        draws.reverse();
        let reverse_info = renderer
            .assets
            .render_into_objectives(&scene, &mut reverse, &cancel, &[], &draws)
            .unwrap();
        assert_eq!(
            full.rgba(),
            reverse.rgba(),
            "caller objective order cannot override scene depth"
        );
        assert_eq!(info.objectives, reverse_info.objectives);
        assert_eq!(info.actors, reverse_info.actors);
        assert_eq!(
            info.actors
                .iter()
                .map(|a| a.entity_id)
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from([1, 2])
        );
        assert!(
            info.actors.windows(2).all(|pair| (
                pair[0].anchor.y,
                pair[0].anchor.x,
                pair[0].entity_id
            ) <= (
                pair[1].anchor.y,
                pair[1].anchor.x,
                pair[1].entity_id
            )),
            "placements retain actual scene depth, not numeric entity order"
        );
        assert_eq!(info.objectives.len(), 2);
        assert!(info
            .objectives
            .iter()
            .all(|o| [3, 4].contains(&o.entity_id)));
        let mut body_only = Canvas::new(WIDTH, HEIGHT).unwrap();
        for draw in &mut draws {
            draw.shadow = None;
        }
        renderer
            .assets
            .render_into_objectives(&scene, &mut body_only, &cancel, &[], &draws)
            .unwrap();
        assert_ne!(
            full.rgba(),
            body_only.rgba(),
            "actual shadows must affect world pixels"
        );
        // Deliberately wrong order: blending shadows over already drawn bodies
        // must differ from the production shadow-before-body/depth composition.
        for placement in &info.objectives {
            let o = towers
                .iter()
                .find(|o| o.entity_id == placement.entity_id)
                .unwrap();
            let shadow = presentation
                .assets
                .shadow_frame(o.graphics_id as u32, 0, Duration::ZERO)
                .unwrap();
            body_only
                .blit_native_shadow_rgb555(
                    &shadow.frame.rgba,
                    shadow.frame.width,
                    shadow.frame.height,
                    placement.anchor.x + shadow.frame.x_offset,
                    placement.anchor.y + shadow.frame.y_offset,
                )
                .unwrap();
        }
        assert_ne!(
            full.rgba(),
            body_only.rgba(),
            "shadow cannot be composited over the tower body"
        );
    }

    renderer
        .render_at(&scene, Some(frame), &cancel, start)
        .unwrap();
    let first = tower_hit(&initial, 26);
    let at = start + Duration::from_millis(100);
    timeline.update(input(&first, at)).unwrap();
    renderer
        .render_at(
            &timeline.snapshot(at).unwrap(),
            first.frame.as_ref(),
            &cancel,
            at,
        )
        .unwrap();
    assert_eq!(renderer.honor.as_ref().unwrap().towers[&4].action, 32);
    assert_eq!(
        renderer.honor.as_ref().unwrap().towers[&4].started,
        Some(at)
    );
    let second = tower_hit(&first, 1);
    let later = start + Duration::from_millis(200);
    timeline.update(input(&second, later)).unwrap();
    renderer
        .render_at(
            &timeline.snapshot(later).unwrap(),
            second.frame.as_ref(),
            &cancel,
            later,
        )
        .unwrap();
    assert_eq!(
        renderer.honor.as_ref().unwrap().towers[&4].started,
        Some(at),
        "new interaction revision with action32 must not restart it"
    );

    let healed = honor::tests::heal(&second, 0).combat_reply().unwrap();
    let cast_at = start + Duration::from_millis(400);
    timeline.update(input(&healed, cast_at)).unwrap();
    renderer
        .render_at(
            &timeline.snapshot(cast_at).unwrap(),
            healed.frame.as_ref(),
            &cancel,
            cast_at,
        )
        .unwrap();
    let sample_at = cast_at + Duration::from_millis(250);
    let sample = timeline.snapshot(sample_at).unwrap();
    let actual = renderer
        .render_at(&sample, healed.frame.as_ref(), &cancel, sample_at)
        .unwrap();
    let effects = std::mem::take(&mut renderer.honor.as_mut().unwrap().effects);
    assert_eq!(effects.len(), 1);
    let control = renderer
        .render_at(&sample, healed.frame.as_ref(), &cancel, sample_at)
        .unwrap();
    assert_ne!(
        actual.bgra, control.bgra,
        "same scene/clock/caster with only actual744 removed"
    );
    renderer.honor.as_mut().unwrap().effects = effects;
    let expired = cast_at + Duration::from_millis(2500);
    let expired_pixels = renderer
        .render_at(
            &timeline.snapshot(expired).unwrap(),
            healed.frame.as_ref(),
            &cancel,
            expired,
        )
        .unwrap();
    assert!(renderer.honor.as_ref().unwrap().effects.is_empty());
    let repeated = renderer
        .render_at(
            &timeline.snapshot(expired).unwrap(),
            healed.frame.as_ref(),
            &cancel,
            expired,
        )
        .unwrap();
    assert_eq!(expired_pixels.bgra, repeated.bgra);

    let dead = honor::tests::died(&healed, 1);
    let dead_at = start + Duration::from_secs(3);
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
    let alive_at = start + Duration::from_secs(8);
    timeline.update(input(&alive, alive_at)).unwrap();
    let living = timeline.snapshot(alive_at).unwrap();
    assert_eq!(living.actors()[0].action, BaseAction::Idle);
    let pixels = renderer
        .render_at(&living, alive.frame.as_ref(), &cancel, alive_at)
        .unwrap();
    assert!(
        pixels.audio.as_ref().unwrap().events.is_empty(),
        "authority respawn cannot replay old death/heal audio"
    );
    assert!(renderer.honor.as_ref().unwrap().effects.is_empty());
    let old_casts = std::mem::take(&mut renderer.honor.as_mut().unwrap().casts);
    assert_eq!(
        old_casts.len(),
        1,
        "retained heal history remains available for replay prevention"
    );
    let no_old_cast = renderer
        .render_at(&living, alive.frame.as_ref(), &cancel, alive_at)
        .unwrap();
    assert_eq!(
        pixels.bgra, no_old_cast.bgra,
        "retained old heal must not override the living body"
    );
    renderer.honor.as_mut().unwrap().casts = old_casts;
    assert_eq!(
        alive.frame.as_ref().unwrap().vitals[0].mp,
        healed.frame.as_ref().unwrap().vitals[0].mp
    );
    assert_eq!(
        alive.frame.as_ref().unwrap().items,
        healed.frame.as_ref().unwrap().items
    );
}
