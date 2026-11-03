//! Frozen Java v31 snapshots; actual shared sprite compositor on synthetic S32.
//! This is not production-map, live TLS, SQL, HWND or audio-device evidence.
use super::*;
use crate::modern_lito::{
    npc_fire_storm_protocol::tests::snapshot, npc_presentation::Plan, npc_protocol as npc,
};

fn input(r: npc::Reply, p: Arc<Plan>, at: Instant) -> Arc<Input> {
    let f = r.frame.as_ref().unwrap();
    let id = f
        .combat
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(r.echo.context.key))
        .unwrap()
        .entity_id;
    let mut i = Input::with_npc(
        Arc::new(f.combat.world.clone()),
        id,
        Some(Arc::new(f.combat.clone())),
        Some(NpcInput {
            reply: Arc::new(r),
            plan: p,
        }),
    )
    .unwrap();
    i.received = at;
    Arc::new(i)
}
fn visual(f: &combat_protocol::Frame) -> FireVisual {
    let event = super::super::fire_storm_presentation::events(f)
        .into_iter()
        .find(|e| e.cast_revision > 0)
        .unwrap();
    let a = f.world.actor(event.entity_id).unwrap();
    let m = f.world.member(event.entity_id).unwrap();
    FireVisual {
        event,
        body: super::super::paced_animation::PacedClock::new(Instant::now()),
        appearance: BaseAppearance {
            class: m.class as u8,
            sex: m.sex,
        },
        move_sequence: a.sequence,
        launched: false,
        launch_at: None,
        body_visible: true,
    }
}
#[test]
fn npc70_plan_pins_spell_resources_even_for_empty_loading_roster() {
    for (scenario, loading, active) in [
        ("normal", "loading", "before"),
        ("time", "loading", "active"),
        ("554", "loading-own", "active-own"),
    ] {
        let r = snapshot(scenario, loading);
        let f = r.frame.as_ref().unwrap();
        assert!(f.actors.is_empty());
        let plan = Plan::from_reply(&r).unwrap();
        plan.validate(&snapshot(scenario, active)).unwrap();
        let unpinned = Plan::new(
            f.scope,
            f.resources
                .as_ref()
                .unwrap()
                .iter()
                .map(|g| crate::modern_lito::npc_assets::NpcAppearance::new(*g).unwrap())
                .collect(),
        )
        .unwrap();
        assert!(
            unpinned.validate(&r).is_err(),
            "unpinned caster/effect resources must not pass LOADED"
        );
    }
}
#[test]
fn npc70_pending_pose_is_cancelled_by_newer_spell_death_life_or_departure() {
    let hit = snapshot("normal", "hit");
    let cast = visual(&hit.frame.as_ref().unwrap().combat);
    assert!(fire_body_current(
        &hit.frame.as_ref().unwrap().combat,
        &cast
    ));
    assert!(fire_body_current(
        &snapshot("normal", "denied").frame.unwrap().combat,
        &cast
    ));
    for name in ["arrow", "disintegrate"] {
        assert!(
            !fire_body_current(&snapshot("normal", name).frame.unwrap().combat, &cast),
            "{name}"
        );
    }
    let prior = snapshot("lives", "prior");
    let cast = visual(&prior.frame.unwrap().combat);
    for name in ["dead", "revived", "next-life"] {
        assert!(
            !fire_body_current(&snapshot("lives", name).frame.unwrap().combat, &cast),
            "{name}"
        );
    }
    let tower = snapshot("time", "tower");
    let cast = visual(&tower.frame.unwrap().combat);
    assert!(!fire_body_current(
        &snapshot("time", "departed").frame.unwrap().combat,
        &cast
    ));
}
#[test]
fn npc70_timeline_uses_confirmed_event_without_mutating_npc_damage_or_restarting_on_denial() {
    let before = snapshot("normal", "before");
    let hit = snapshot("normal", "hit");
    let denied = snapshot("normal", "denied");
    let plan = Arc::new(Plan::from_reply(&before).unwrap());
    let t = Instant::now();
    let mut timeline = Timeline::new(input(before, plan.clone(), t));
    let at = t + Duration::from_millis(100);
    timeline
        .update(input(hit.clone(), plan.clone(), at))
        .unwrap();
    let id = timeline.input.identity.selected_entity_id;
    assert!(timeline.actors[&id].pose.is_none());
    assert!(timeline.actors[&id].walk_until.is_none());
    assert_eq!(timeline.actors[&id].action_started, at);
    timeline
        .update(input(denied.clone(), plan, at + Duration::from_millis(50)))
        .unwrap();
    assert_eq!(timeline.actors[&id].action_started, at);
    for ms in [0, 875, 2375, 9000] {
        timeline.snapshot(at + Duration::from_millis(ms)).unwrap();
        assert_eq!(timeline.input.npc.as_ref().unwrap().reply.as_ref(), &denied);
    }
    assert_eq!(hit, snapshot("normal", "hit"));
}
#[test]
fn npc70_effect_cells_are_qualified_including_retained_structure_receipts() {
    for (scenario, name) in [("normal", "hit"), ("time", "tower"), ("554", "cast-1-own")] {
        let mut f = snapshot(scenario, name).frame.unwrap().combat;
        let map = crate::modern_lito::siege_space::tests::map(&f.world);
        assert_eq!(map.canonical_digest(), f.world.manifest.digest);
        validate_fire_geometry(&f, &map).unwrap();
        let s = f.npc_fires.iter_mut().find(|s| s.count > 0).unwrap();
        s.origin = Some(WorldCell { x: 0, y: 0 });
        assert!(validate_fire_geometry(&f, &map).is_err());
    }
}
fn sound(p: &Pixels) -> Vec<u64> {
    p.audio
        .as_ref()
        .unwrap()
        .events
        .iter()
        .filter(|e| e.group.candidates() == [710])
        .map(|e| e.sequence)
        .collect()
}
#[test]
#[ignore = "LITO_SHARED_RESOURCE_ROOT; real shared caster18/SPR1819/Sound710 and compositor, frozen Java NPC70 on synthetic S32; no TLS/SQL/HWND/device"]
fn actual_npc70_runtime_preloads_once_and_renders_one_owned_group_without_replay() {
    let root =
        PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"));
    let cancel = AtomicBool::new(false);
    let loading = snapshot("normal", "loading");
    let before = snapshot("normal", "before");
    let hit = snapshot("normal", "hit");
    let denied = snapshot("normal", "denied");
    let plan = Arc::new(Plan::from_reply(&loading).unwrap());
    let first = NpcInput {
        reply: Arc::new(loading.clone()),
        plan: plan.clone(),
    };
    let f = &loading.frame.as_ref().unwrap().combat;
    let map = crate::modern_lito::siege_space::tests::map(&f.world);
    assert_eq!(map.canonical_digest(), f.world.manifest.digest);
    let mut renderer =
        load_qualified_map_renderer(&root, &f.world, Some(f), Some(&first), &cancel, map).unwrap();
    assert!(
        renderer.fire_storm.is_some(),
        "empty LOADING must already own shared assets"
    );
    let t = Instant::now();
    let mut timeline = Timeline::new(input(loading, plan.clone(), t));
    timeline
        .update(input(before.clone(), plan.clone(), t))
        .unwrap();
    let n = NpcInput {
        reply: Arc::new(before.clone()),
        plan: plan.clone(),
    };
    let initial = renderer
        .render_at_npcs(
            &timeline.snapshot(t).unwrap(),
            Some(&before.frame.as_ref().unwrap().combat),
            Some(&n),
            &cancel,
            t,
        )
        .unwrap();
    assert!(sound(&initial).is_empty());
    assert!(renderer.fire_storm.as_ref().unwrap().active.is_empty());
    let at = t + Duration::from_millis(100);
    timeline
        .update(input(hit.clone(), plan.clone(), at))
        .unwrap();
    let scene = timeline.snapshot(at).unwrap();
    let appearance = scene
        .actors()
        .iter()
        .find(|a| a.entity_id == scene.selected_entity_id())
        .unwrap()
        .appearance;
    let delay = renderer
        .fire_storm
        .as_ref()
        .unwrap()
        .assets
        .cast_launch_delay(appearance)
        .unwrap();
    let n = NpcInput {
        reply: Arc::new(hit.clone()),
        plan: plan.clone(),
    };
    let f = &hit.frame.as_ref().unwrap().combat;
    renderer
        .render_at_npcs(&scene, Some(f), Some(&n), &cancel, at)
        .unwrap();
    if delay > Duration::ZERO {
        assert!(renderer.fire_storm.as_ref().unwrap().effects.is_empty());
    }
    let launch = at + delay;
    let scene = timeline.snapshot(launch).unwrap();
    let p = renderer
        .render_at_npcs(&scene, Some(f), Some(&n), &cancel, launch)
        .unwrap();
    let ids = sound(&p);
    assert_eq!(ids.len(), 1);
    let repeated = renderer
        .render_at_npcs(&scene, Some(f), Some(&n), &cancel, launch)
        .unwrap();
    assert_eq!(p.bgra, repeated.bgra);
    assert_eq!(ids, sound(&repeated));
    assert_eq!(renderer.fire_storm.as_ref().unwrap().effects.len(), 1);
    assert_eq!(
        renderer.fire_storm.as_ref().unwrap().effects[0].position,
        f.npc_fires[0].origin.unwrap()
    );
    // Only remove the1819 group. The same NPC/body/HP/camera/time remains.
    let visible_at = launch + Duration::from_millis(125);
    let scene = timeline.snapshot(visible_at).unwrap();
    let visible = renderer
        .render_at_npcs(&scene, Some(f), Some(&n), &cancel, visible_at)
        .unwrap();
    let effects = std::mem::take(&mut renderer.fire_storm.as_mut().unwrap().effects);
    assert_eq!(effects.len(), 1);
    let control = renderer
        .render_at_npcs(&scene, Some(f), Some(&n), &cancel, visible_at)
        .unwrap();
    assert_ne!(visible.bgra, control.bgra);
    assert_eq!(visible.actors, control.actors);
    assert_eq!(visible.npcs, control.npcs);
    assert!(renderer.fire_storm.as_ref().unwrap().effects.is_empty());
    renderer.fire_storm.as_mut().unwrap().effects = effects;
    let later = launch + Duration::from_secs(3);
    timeline
        .update(input(denied.clone(), plan.clone(), later))
        .unwrap();
    let n = NpcInput {
        reply: Arc::new(denied.clone()),
        plan,
    };
    let p = renderer
        .render_at_npcs(
            &timeline.snapshot(later).unwrap(),
            Some(&denied.frame.as_ref().unwrap().combat),
            Some(&n),
            &cancel,
            later,
        )
        .unwrap();
    assert!(renderer.fire_storm.as_ref().unwrap().effects.is_empty());
    assert!(sound(&p).is_empty());
    assert_eq!(
        hit,
        snapshot("normal", "hit"),
        "presentation cannot mutate authority"
    );
    super::fire_storm_runtime_tests::assert_hurt_keeps_pending_effect(
        renderer.fire_storm.as_mut().unwrap(),
        &renderer.map,
        &before.frame.as_ref().unwrap().combat,
        f,
    );
    assert_npc_melee_replaces_pending_body_not_launched_effect(
        renderer.fire_storm.as_mut().unwrap(),
        &renderer.map,
        before.frame.as_ref().unwrap(),
        hit.frame.as_ref().unwrap(),
    );
}

fn assert_npc_melee_replaces_pending_body_not_launched_effect(
    fire: &mut FirePresentation,
    map: &WorldMap,
    before: &npc::Frame,
    hit: &npc::Frame,
) {
    // Isolated pose arbitration only, NOT a newly valid protocol snapshot.
    // The real engine keeps NPC melee in typed exchanges and leaves the PvP
    // Vital.attack_revision untouched (ModernBattleEngine.attackNpc).
    let id = hit.combat.npc_fires[0].entity_id;
    let mut melee = hit.clone();
    let mut receipt = npc::tests::read_reply()
        .frame
        .unwrap()
        .exchanges
        .into_iter()
        .find(|e| e.direction == npc::Direction::PlayerToNpc)
        .unwrap();
    receipt.participant = hit.combat.world.actor(id).unwrap().identity();
    receipt.revision = hit.combat.world.revision + 1;
    receipt.npc.scope = hit.scope;
    receipt.npc.serial = hit.actors[0].serial;
    receipt.npc_life = hit.actors[0].life;
    melee
        .exchanges
        .retain(|e| e.direction != npc::Direction::PlayerToNpc);
    melee.exchanges.push(receipt);
    let frozen = melee.clone();
    let m = hit.combat.world.member(id).unwrap();
    let delay = fire
        .assets
        .cast_launch_delay(BaseAppearance {
            class: m.class as u8,
            sex: m.sex,
        })
        .unwrap();
    assert!(delay > Duration::ZERO);
    for phase in 0..3 {
        fire.identity = (hit.scope.match_id, hit.scope.generation);
        fire.baseline = crate::modern_lito::fire_storm_presentation::events(&before.combat)
            .iter()
            .map(|e| (e.entity_id, e.cast_revision))
            .collect();
        fire.active.clear();
        fire.effects.clear();
        let at = Instant::now();
        fire.update(
            &hit.combat,
            map,
            Some(if phase == 0 { &melee } else { hit }),
            at,
        )
        .unwrap();
        if phase == 2 {
            fire.update(&hit.combat, map, Some(hit), at + delay)
                .unwrap();
            assert_eq!(fire.effects.len(), 1);
        }
        fire.update(
            &melee.combat,
            map,
            Some(&melee),
            at + delay + Duration::from_millis(1),
        )
        .unwrap();
        assert!(fire.active.is_empty());
        assert_eq!(fire.effects.len(), usize::from(phase == 2), "phase {phase}");
        assert_eq!(melee, frozen);
    }
}
