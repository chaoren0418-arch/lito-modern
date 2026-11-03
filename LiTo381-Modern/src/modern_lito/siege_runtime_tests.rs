//! Real Java authority fixtures plus shared SPR/Tile/audio compositor evidence.
//! Synthetic S32 arena; this is NOT native 547/563/573 layout or HWND acceptance.
use super::*;
use crate::modern_lito::{siege_presentation as presentation, siege_protocol as siege};
fn input(name: &str, now: Instant) -> Arc<Input> {
    let f = siege::tests::sample(name).frame.unwrap();
    let mut input = Input::with_combat(Arc::new(f.world.clone()), 1, Some(Arc::new(f))).unwrap();
    input.received = now;
    Arc::new(input)
}
#[test]
fn siege_tower_attacks_use_objective_receipts_without_aliasing_player_events() {
    let now = Instant::now();
    let before = input("active", now);
    let mut timeline = Timeline::new(before);
    let next = input("left", now + Duration::from_millis(100));
    let d = next.frame.siege.as_ref().unwrap();
    let entity = d.objectives[0].last_actor;
    let vital = next.combat.as_ref().unwrap().vital(entity).unwrap();
    assert_eq!(vital.last_target_entity_id, 0);
    assert_eq!(vital.attack_revision, 0);
    assert_eq!(
        presentation::player_attack(&next.frame, entity)
            .unwrap()
            .revision,
        d.objectives[0].revision + 1
    );
    timeline.update(next.clone()).unwrap();
    let pose = |scene: SceneSnapshot| {
        scene
            .actors()
            .iter()
            .find(|a| a.entity_id == entity)
            .unwrap()
            .action
    };
    assert_eq!(
        pose(timeline.snapshot(next.received).unwrap()),
        BaseAction::Attack
    );
    timeline.update(next.clone()).unwrap();
    assert!(
        timeline
            .snapshot(next.received + Duration::from_millis(100))
            .unwrap()
            .actors()
            .iter()
            .find(|a| a.entity_id == entity)
            .unwrap()
            .action_elapsed
            > Duration::ZERO
    );
}
#[test]
fn siege_portal_and_respawn_are_relocations_not_long_walks() {
    for suffix in ["plain", "regen"] {
        let now = Instant::now();
        let before = input(&format!("beforeportal{suffix}"), now);
        let next = input(&format!("portal{suffix}"), now + Duration::from_millis(100));
        assert!(presentation::portalled(&before.frame, &next.frame, 2));
        let mut timeline = Timeline::new(before);
        timeline.update(next.clone()).unwrap();
        assert_eq!(
            timeline.snapshot(next.received).unwrap().actors()[1].action,
            BaseAction::Idle
        );
        let before = input(&format!("dead{suffix}"), now);
        let next = input(
            &format!("respawn{suffix}"),
            now + Duration::from_millis(100),
        );
        let mut timeline = Timeline::new(before);
        timeline.update(next.clone()).unwrap();
        assert_eq!(
            timeline.snapshot(next.received).unwrap().actors()[1].action,
            BaseAction::Idle
        );
    }
}
#[test]
fn siege_hud_has_role_wide_hp_and_durable_source_scoped_results_without_fake_clock() {
    let mut w = siege::tests::sample("active").frame.unwrap().world;
    let lines = presentation::hud(&w, 1).join(" ");
    assert!(lines.contains("設定時限") && lines.contains("非剩餘倒數"));
    assert!(presentation::result_notice(&w, w.actors[0].key).is_none());
    w = siege::tests::sample("inner").frame.unwrap().world;
    let keys = [w.actors[0].key, w.actors[1].key];
    assert_eq!(keys[0].object_id, keys[1].object_id);
    assert_ne!(keys[0].source_id, keys[1].source_id);
    let win = w.siege.as_ref().unwrap().winner.unwrap();
    for key in keys {
        let actor = w.actors.iter().find(|a| a.key == key).unwrap();
        let expected = if w.member(actor.entity_id).unwrap().team == win {
            "本場勝利"
        } else {
            "本場敗北"
        };
        assert!(presentation::result_notice(&w, key)
            .unwrap()
            .starts_with(expected));
    }
    for (state, word) in [
        (world_protocol::SettlementState::Pending, "保存尚未確認"),
        (world_protocol::SettlementState::Saved, "非累計場數"),
        (world_protocol::SettlementState::Failed, "不可重複計入"),
    ] {
        w.siege.as_mut().unwrap().receipt.as_mut().unwrap().state = state;
        assert!(presentation::result_notice(&w, keys[0])
            .unwrap()
            .contains(word));
    }
}
#[test]
#[ignore = "requires shared Lineage SPR/Tile/WAV; synthetic S32 arena, not native map or HWND acceptance"]
fn real_shared_siege_three_towers_gate_portals_collapse_and_audio_use_one_preload() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
    let now = Instant::now();
    let cancel = AtomicBool::new(false);
    let first = input("active", now);
    let map = crate::modern_lito::siege_space::tests::map(&first.frame);
    crate::modern_lito::shared_action_catalog::test_counts::reset();
    let mut renderer = load_qualified_map_renderer(
        &root,
        &first.frame,
        first.combat.as_deref(),
        None,
        &cancel,
        map,
    )
    .unwrap();
    assert!(renderer.siege.is_some() && renderer.honor.is_none() && renderer.npc.is_none());
    let counts = crate::modern_lito::shared_action_catalog::test_counts::snapshot();
    assert_eq!(
        (counts.load_attempts, counts.reads, counts.parses),
        (1, 1, 1)
    );
    let mut timeline = Timeline::new(first.clone());
    let live = renderer
        .render_at(
            &timeline.snapshot(now).unwrap(),
            first.combat.as_deref(),
            &cancel,
            now,
        )
        .unwrap();
    assert!(presentation::valid_placements(
        &first.frame,
        &live.siege_towers
    ));
    assert_eq!(live.siege_towers.iter().filter(|p| p.alive).count(), 3);
    assert!(live.audio.as_ref().unwrap().events.is_empty());
    let mut previous = live.bgra;
    for (i, name) in [
        "left",
        "beforeright",
        "right",
        "beforeinner",
        "inner",
        "saved",
    ]
    .into_iter()
    .enumerate()
    {
        let at = now + Duration::from_millis(500 * (i as u64 + 1));
        let next = input(name, at);
        timeline.update(next.clone()).unwrap();
        let pixels = renderer
            .render_at(
                &timeline.snapshot(at).unwrap(),
                next.combat.as_deref(),
                &cancel,
                at,
            )
            .unwrap();
        assert_eq!(pixels.siege_towers.len(), 3);
        assert!(pixels.objectives.is_empty() && pixels.quests.is_empty() && pixels.npcs.is_empty());
        assert!(presentation::valid_placements(
            &next.frame,
            &pixels.siege_towers
        ));
        if name != "saved" {
            assert_ne!(pixels.bgra, previous);
        }
        previous = pixels.bgra;
    }
    assert_eq!(
        crate::modern_lito::shared_action_catalog::test_counts::snapshot(),
        counts
    );
    let mut changed = first.frame.as_ref().clone();
    changed.siege.as_mut().unwrap().portal_inside.x += 1;
    assert!(renderer
        .siege
        .as_mut()
        .unwrap()
        .update(&changed, now)
        .is_err());
}
