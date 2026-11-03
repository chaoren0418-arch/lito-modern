//! Real Java frames and shared worker rendering, not native540/HWND evidence.
use super::*;
use crate::modern_lito::{
    npc_presentation::Plan, npc_protocol as npc, time_attack_protocol::tests::sample,
};

fn input(reply: npc::Reply, now: Instant) -> Arc<Input> {
    let f = reply.frame.as_ref().unwrap();
    let selected = f
        .combat
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(reply.echo.context.key))
        .unwrap()
        .entity_id;
    let plan = Arc::new(Plan::from_reply(&reply).unwrap());
    let mut input = Input::with_npc(
        Arc::new(f.combat.world.clone()),
        selected,
        Some(Arc::new(f.combat.clone())),
        Some(NpcInput {
            reply: Arc::new(reply),
            plan,
        }),
    )
    .unwrap();
    input.received = now;
    Arc::new(input)
}

#[test]
fn cooperative_tower_receipt_drives_pose_once_and_future_waves_keep_one_preflight() {
    let now = Instant::now();
    let before = input(sample("waiting"), now);
    let plan = before.npc.as_ref().unwrap().plan.clone();
    let mut timeline = Timeline::new(before);
    let at = now + Duration::from_millis(100);
    let tower = input(sample("tower"), at);
    let f = tower.npc.as_ref().unwrap().reply.frame.as_ref().unwrap();
    let e = &f.combat.world.time_attack.as_ref().unwrap().exchanges[0];
    let pose = crate::modern_lito::npc_presentation::player_attack(f, e.entity_id).unwrap();
    assert_eq!(pose.revision, e.revision + 1);
    assert_eq!(
        f.combat.vital(e.entity_id).unwrap().last_target_entity_id,
        0
    );
    timeline.update(tower.clone()).unwrap();
    assert_eq!(
        timeline.snapshot(at).unwrap().actors()[0].action,
        BaseAction::Attack
    );
    timeline.update(tower).unwrap();
    assert!(
        timeline
            .snapshot(at + Duration::from_millis(100))
            .unwrap()
            .actors()[0]
            .action_elapsed
            > Duration::ZERO
    );
    for (i, name) in ["fighting", "wave0", "wave1", "wave2"].iter().enumerate() {
        let value = input(
            sample(name),
            at + Duration::from_millis(200 + i as u64 * 100),
        );
        assert_eq!(value.npc.as_ref().unwrap().plan, plan);
        timeline.update(value).unwrap();
        timeline.snapshot(at + Duration::from_secs(1)).unwrap();
    }
}

#[test]
fn departed_members_keep_result_identity_but_do_not_draw_or_collide_with_survivors() {
    let mut reply = sample("departed");
    let f = reply.frame.as_mut().unwrap();
    assert!(f.combat.world.time_attack.as_ref().unwrap().departed(1));
    reply.echo.context.key = f.combat.world.actors[1].key;
    // A departed position may legitimately be reused by the remaining player.
    f.combat.world.actors[0].position = f.combat.world.actors[1].position;
    reply.validate().unwrap();
    let now = Instant::now();
    let value = input(reply, now);
    assert_eq!(value.frame.actors.len(), 2);
    let mut timeline = Timeline::new(value);
    let scene = timeline.snapshot(now).unwrap();
    assert_eq!(scene.actors().len(), 1);
    assert_eq!(scene.actors()[0].entity_id, 2);
}

#[test]
#[ignore = "requires pinned shared map554/PC/NPC/towers/SPR/WAV; worker compositor, not HWND or audio device"]
fn real_shared_time_attack_tower_collapse_wave_preload_and_ruins_do_not_reload_assets() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
    let cancel = AtomicBool::new(false);
    let bindings = crate::modern_lito::quest_resource_probe_tests::bindings();
    let map = WorldMap::load_shared(&root, &bindings, &cancel).unwrap();
    let anchor = WorldCell { x: 32778, y: 32823 };
    let tower_cell = (anchor.y - 2..=anchor.y + 2)
        .flat_map(|y| (anchor.x - 2..=anchor.x + 2).map(move |x| WorldCell { x, y }))
        .find(|c| *c != anchor && map.standing(*c))
        .unwrap();
    let manifest = world_protocol::Manifest {
        version: 1,
        map_id: 554,
        digest: map.canonical_digest(),
        sectors: bindings
            .iter()
            .map(|b| world_protocol::Sector {
                x: b.key.sector_x,
                y: b.key.sector_y,
                byte_length: b.byte_length,
                sha256: b.sha256,
            })
            .collect(),
    };
    let fixture = |name| {
        let mut r = sample(name);
        let f = r.frame.as_mut().unwrap();
        f.combat.world.manifest = manifest.clone();
        f.combat.world.actors[0].position = anchor;
        f.combat.world.time_attack.as_mut().unwrap().tower_cell = tower_cell;
        r.validate().unwrap();
        r
    };
    let now = Instant::now();
    let before = input(fixture("waiting"), now);
    crate::modern_lito::shared_action_catalog::test_counts::reset();
    let mut renderer = load_shared_renderer_npcs(
        &root,
        &before.frame,
        before.combat.as_deref(),
        before.npc.as_ref(),
        &cancel,
    )
    .unwrap();
    assert!(renderer.time_attack.is_some() && renderer.honor.is_none() && renderer.quest.is_none());
    let counts = crate::modern_lito::shared_action_catalog::test_counts::snapshot();
    assert_eq!(
        (counts.load_attempts, counts.reads, counts.parses),
        (1, 1, 1)
    );
    let mut timeline = Timeline::new(before.clone());
    let live = renderer
        .render_at_npcs(
            &timeline.snapshot(now).unwrap(),
            before.combat.as_deref(),
            before.npc.as_ref(),
            &cancel,
            now,
        )
        .unwrap();
    assert!(live.time_attack_tower.as_ref().unwrap().alive);
    assert!(live.quests.is_empty() && live.objectives.is_empty() && live.npcs.is_empty());
    assert!(live.audio.as_ref().unwrap().events.is_empty());
    let at = now + Duration::from_millis(100);
    let broken = input(fixture("tower"), at);
    timeline.update(broken.clone()).unwrap();
    let collapse = renderer
        .render_at_npcs(
            &timeline.snapshot(at).unwrap(),
            broken.combat.as_deref(),
            broken.npc.as_ref(),
            &cancel,
            at,
        )
        .unwrap();
    assert!(!collapse.time_attack_tower.as_ref().unwrap().alive);
    assert_ne!(collapse.bgra, live.bgra);
    let later = at + Duration::from_secs(10);
    let ruin = renderer
        .render_at_npcs(
            &timeline.snapshot(later).unwrap(),
            broken.combat.as_deref(),
            broken.npc.as_ref(),
            &cancel,
            later,
        )
        .unwrap();
    assert_eq!(
        ruin.time_attack_tower
            .as_ref()
            .unwrap()
            .geometry
            .source_frame,
        0
    );
    assert_eq!(
        ruin.time_attack_tower.as_ref().unwrap().id,
        live.time_attack_tower.as_ref().unwrap().id
    );
    assert_eq!(
        crate::modern_lito::shared_action_catalog::test_counts::snapshot(),
        counts
    );
    let mut changed = broken.frame.as_ref().clone();
    changed.time_attack.as_mut().unwrap().tower_cell.x += 1;
    assert!(renderer
        .time_attack
        .as_mut()
        .unwrap()
        .update(&changed, later)
        .is_err());
}
