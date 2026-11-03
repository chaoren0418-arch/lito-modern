//! Java authority + qualified shared asset integration. Geometry substitutions
//! below are explicit compositor fixtures, never a native Quest layout claim.
use super::*;
use crate::modern_lito::{npc_presentation::Plan, npc_protocol as npc, quest_protocol as quest};

fn input(reply: npc::Reply, now: Instant) -> Arc<Input> {
    let f = reply.frame.as_ref().unwrap();
    let plan = Arc::new(Plan::from_reply(&reply).unwrap());
    let mut value = Input::with_npc(
        Arc::new(f.combat.world.clone()),
        1,
        Some(Arc::new(f.combat.clone())),
        Some(NpcInput {
            reply: Arc::new(reply),
            plan,
        }),
    )
    .unwrap();
    value.received = now;
    Arc::new(value)
}

#[test]
fn quest_receipt_drives_player_pose_without_counterfeit_pc_target_and_poll_does_not_replay() {
    let now = Instant::now();
    let mut timeline = Timeline::new(input(quest::tests::sample("before"), now));
    let next = now + Duration::from_millis(100);
    let won = quest::tests::sample("win");
    assert_eq!(
        won.frame.as_ref().unwrap().combat.vitals[0].attack_revision,
        0
    );
    assert_eq!(
        won.frame.as_ref().unwrap().combat.vitals[0].last_target_entity_id,
        0
    );
    timeline.update(input(won.clone(), next)).unwrap();
    assert_eq!(
        timeline.snapshot(next).unwrap().actors()[0].action,
        BaseAction::Attack
    );
    timeline
        .update(input(won, next + Duration::from_millis(100)))
        .unwrap();
    assert!(
        timeline
            .snapshot(next + Duration::from_millis(100))
            .unwrap()
            .actors()[0]
            .action_elapsed
            > Duration::ZERO
    );
}

#[test]
fn quest_death_confirmation_clears_corpse_pose_and_coalesced_old_damage_without_walking() {
    use crate::modern_lito::quest_life::tests::sample;
    let now = Instant::now();
    let dead_at = now + Duration::from_millis(100);
    let revived_at = dead_at + Duration::from_millis(100);
    let before = input(sample("before"), now);
    let dead = input(sample("dead"), dead_at);
    let revived = input(sample("revived"), revived_at);
    for coalesced in [false, true] {
        let mut timeline = Timeline::new(before.clone());
        if !coalesced {
            timeline.update(dead.clone()).unwrap();
            assert_eq!(
                timeline.snapshot(dead_at).unwrap().actors()[0].action,
                BaseAction::Death
            );
        }
        timeline.update(revived.clone()).unwrap();
        let scene = timeline.snapshot(revived_at).unwrap();
        assert_eq!(scene.actors()[0].action, BaseAction::Idle);
        assert!(timeline.actors[&1].walk_until.is_none());
        timeline.update(revived.clone()).unwrap();
        assert_eq!(
            timeline
                .snapshot(revived_at + Duration::from_secs(5))
                .unwrap()
                .actors()[0]
                .action,
            BaseAction::Idle
        );
    }
}

#[test]
#[ignore = "requires pinned Lineage map554/PC/NPC/towers/SPR/WAV; worker compositor, no HWND or real audio device"]
fn real_shared_quest_preflight_damage_collapse_ruin_and_typed_geometry_never_reload_assets() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
    let cancel = AtomicBool::new(false);
    let bindings = crate::modern_lito::quest_resource_probe_tests::bindings();
    let map = WorldMap::load_shared(&root, &bindings, &cancel).unwrap();
    let anchor = WorldCell { x: 32778, y: 32823 };
    let mut cells = Vec::new();
    for y in anchor.y - 2..=anchor.y + 2 {
        for x in anchor.x - 2..=anchor.x + 2 {
            let c = WorldCell { x, y };
            if c != anchor && map.standing(c) {
                cells.push(c);
            }
        }
    }
    assert!(cells.len() >= 2);
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
        let mut reply = quest::tests::sample(name);
        let frame = reply.frame.as_mut().unwrap();
        frame.combat.world.manifest = manifest.clone();
        frame.combat.world.actors[0].position = anchor;
        frame.combat.world.quest.as_mut().unwrap().targets[0].cell = cells[0];
        for a in &mut frame.actors {
            a.cell = cells[1];
        }
        reply.validate().unwrap();
        reply
    };
    let now = Instant::now();
    let before = input(fixture("before"), now);
    crate::modern_lito::shared_action_catalog::test_counts::reset();
    let mut renderer = load_shared_renderer_npcs(
        &root,
        &before.frame,
        before.combat.as_deref(),
        before.npc.as_ref(),
        &cancel,
    )
    .unwrap();
    assert!(renderer.quest.is_some());
    assert!(renderer.honor.is_none());
    let counts = crate::modern_lito::shared_action_catalog::test_counts::snapshot();
    assert_eq!(
        (counts.load_attempts, counts.reads, counts.parses),
        (1, 1, 1)
    );
    let mut timeline = Timeline::new(before.clone());
    let scene = timeline.snapshot(now).unwrap();
    let live = renderer
        .render_at_npcs(
            &scene,
            before.combat.as_deref(),
            before.npc.as_ref(),
            &cancel,
            now,
        )
        .unwrap();
    assert_eq!(
        (live.actors.len(), live.quests.len(), live.objectives.len()),
        (1, 1, 0)
    );
    assert!(live.quests[0].alive);
    assert_eq!(live.quests[0].id.slot, 4);
    assert_eq!(live.quests[0].geometry.graphics_id, 2665);
    assert!(
        live.audio.as_ref().unwrap().events.is_empty(),
        "historical receipt must be silent"
    );
    let next = now + Duration::from_millis(100);
    let won = input(fixture("win"), next);
    timeline.update(won.clone()).unwrap();
    let scene = timeline.snapshot(next).unwrap();
    let collapse = renderer
        .render_at_npcs(
            &scene,
            won.combat.as_deref(),
            won.npc.as_ref(),
            &cancel,
            next,
        )
        .unwrap();
    assert!(!collapse.quests[0].alive);
    assert_ne!(collapse.bgra, live.bgra);
    let later = next + Duration::from_secs(10);
    let scene = timeline.snapshot(later).unwrap();
    let ruin = renderer
        .render_at_npcs(
            &scene,
            won.combat.as_deref(),
            won.npc.as_ref(),
            &cancel,
            later,
        )
        .unwrap();
    assert_eq!(ruin.quests[0].geometry.source_frame, 0);
    assert_eq!(collapse.quests[0].id, ruin.quests[0].id);
    assert!(!ruin.quests[0].alive);
    assert_eq!(
        crate::modern_lito::shared_action_catalog::test_counts::snapshot(),
        counts
    );
    // Renderer occupancy follows authority: ruins can share a cell with a PC.
    let mut overlap = scene.clone();
    let mut actors = overlap.actors().to_vec();
    actors[0].cell = cells[0];
    // Scene construction still validates identities and geometry through the
    // same builder used by Timeline, never by inventing a network entity ID.
    overlap = SceneSnapshot::new(
        overlap.world_id(),
        overlap.revision(),
        overlap.map_digest(),
        overlap.selected_entity_id(),
        actors,
    )
    .unwrap();
    let draws = renderer
        .quest
        .as_ref()
        .unwrap()
        .draws(&won.frame, later)
        .unwrap();
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    renderer
        .assets
        .render_into_npcs(&overlap, &mut canvas, &cancel, &[], &draws, &[])
        .unwrap();
    let mut wrong = before.frame.as_ref().clone();
    wrong.quest.as_mut().unwrap().targets[0].cell.x += 1;
    assert!(renderer
        .quest
        .as_mut()
        .unwrap()
        .update(&wrong, later, false)
        .is_err());
}
