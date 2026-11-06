use lito_modern::modern_lito::{
    native_inventory::Item,
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{self, Model, Target},
    native_skills::{self as skills, Feedback, ItemTarget, Pending},
};
use std::time::{Duration, Instant};

const ITEM: u32 = 0xfedcba98;
const SELECTORS: [u16; 6] = [101, 102, 104, 105, 106, 109];

fn snapshot() -> Snapshot {
    let mut state = LoginController::default().snapshot().clone();
    let fixture: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    for key in ["world", "start", "status", "actor", "bot"] {
        let hex = fixture["packets"][key].as_str().unwrap();
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        state.world.accept(&bytes).unwrap();
    }
    let mut own = state.world.actors.remove(&ITEM).unwrap();
    own.id = 1;
    own.x = 32768;
    own.y = 32768;
    state.world.actors.insert(1, own);
    state.world.status.as_mut().unwrap().object_id = 1;
    let actor = state.world.actors.get_mut(&2).unwrap();
    actor.x = 32769;
    actor.y = 32768;
    state.world.hp = Some([1, 50]);
    state.world.mp = Some([1, 200]);
    // Exercise the real book decoder across rows 12 and 13.
    let mut book = vec![0x1e, 14];
    book.extend([0; 12]);
    book.extend([0xf0, 0x1f]);
    state.world.accept(&book).unwrap();
    state.phase = LoginPhase::BattlePending;
    state.game_connected = true;
    state.inventory.begin(7, "PeerFixture").unwrap();
    state.inventory.items.insert(
        ITEM,
        Item {
            id: ITEM,
            use_type: 1,
            charge: 0,
            icon: 1,
            blessing: 0,
            count: 1,
            identified: 1,
            name: "server weapon".into(),
            detail: vec![0],
        },
    );
    state
}

fn pending(state: &Snapshot, now: Instant) -> Pending {
    Pending {
        since: now,
        casts: state.world.own_cast_packets,
        messages: state.world.message_count,
        generation: state.world.generation,
        incarnation: state.world.own_actor().unwrap().incarnation,
    }
}

#[test]
fn selectors_clear_stale_hover_and_emit_exact_three_bytes_without_spending() {
    let mut state = snapshot();
    assert_eq!(
        state
            .world
            .learned_skills
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        (101..=109).collect::<Vec<_>>()
    );
    let target = Target::capture(&state.world, 2);
    state.world.actors.remove(&2);
    let before = serde_json::to_value(&state).unwrap();
    for (skill, row, column) in [
        (101, 12, 4),
        (102, 12, 5),
        (104, 12, 7),
        (105, 13, 0),
        (106, 13, 1),
        (109, 13, 4),
    ] {
        assert!(skills::selector_only(skill));
        let choice = native_skill_selection::choice(&state.world, skill, target).unwrap();
        assert_eq!(choice.target, None);
        assert_eq!(
            skills::request(&state.world, skill, choice.target).unwrap(),
            [0x14, row, column]
        );
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn actor_spells_use_current_unsigned_target_coordinates_and_formal_range() {
    for (skill, row, column, range) in [(103, 12, 6, 3), (108, 13, 3, 1)] {
        let mut state = snapshot();
        let mut actor = state.world.actors.remove(&2).unwrap();
        actor.id = ITEM;
        state.world.actors.insert(ITEM, actor);
        let target = Target::capture(&state.world, ITEM);
        for (dx, dy, accepted) in [
            (range, range, true),
            (-range, -range, true),
            (range + 1, 0, false),
            (0, -range - 1, false),
        ] {
            let actor = state.world.actors.get_mut(&ITEM).unwrap();
            actor.x = (32768 + dx) as u16;
            actor.y = (32768 + dy) as u16;
            let position = [actor.x, actor.y];
            let before = serde_json::to_value(&state).unwrap();
            let choice = native_skill_selection::choice(&state.world, skill, target);
            assert_eq!(choice.is_ok(), accepted, "skill={skill} dx={dx} dy={dy}");
            if accepted {
                let mut expected = vec![0x14, row, column, 0x98, 0xba, 0xdc, 0xfe];
                expected.extend(position[0].to_le_bytes());
                expected.extend(position[1].to_le_bytes());
                assert_eq!(
                    skills::request(&state.world, skill, Some(ITEM)).unwrap(),
                    expected
                );
            }
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
        assert!(skills::request(&state.world, skill, None).is_err());
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
    }
}

#[test]
fn actor_spells_reject_removed_hidden_dead_and_replaced_targets() {
    for skill in [103, 108] {
        for change in 0..5 {
            let mut state = snapshot();
            let target = Target::capture(&state.world, 2);
            match change {
                0 => {
                    state.world.actors.remove(&2);
                }
                1 => state.world.actors.get_mut(&2).unwrap().dead = true,
                2 => state.world.actors.get_mut(&2).unwrap().incarnation += 1,
                3 => {
                    state.world.accept(&[0x34, 2, 0, 0, 0, 1]).unwrap();
                }
                _ => state.world.generation += 1,
            }
            assert!(native_skill_selection::choice(&state.world, skill, target).is_err());
        }
    }
}

#[test]
fn every_skill_rechecks_learning_life_mana_and_control_without_prediction() {
    for skill in 101..=109 {
        for change in 0..7 {
            let mut state = snapshot();
            let item = ItemTarget::capture(&state, 7, ITEM).unwrap();
            match change {
                0 => {
                    state.world.learned_skills.remove(&skill);
                }
                1 => state.world.mp = Some([0, 200]),
                2 => state.world.mp = None,
                3 => state.world.hp = Some([0, 50]),
                4 => state.world.actors.get_mut(&1).unwrap().dead = true,
                5 => {
                    state.world.movement_blocks.insert(1);
                }
                _ => state.world.battle_result = Some(0),
            }
            let before = serde_json::to_value(&state).unwrap();
            let result = if skill == 107 {
                skills::item_request(&state, 7, skill, &item)
            } else {
                skills::request(
                    &state.world,
                    skill,
                    (!SELECTORS.contains(&skill)).then_some(2),
                )
            };
            assert!(result.is_err(), "skill={skill} change={change}");
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
    }
}

#[test]
fn shadow_fang_uses_only_explicit_inventory_intent_and_preserves_authority() {
    let state = snapshot();
    let item = ItemTarget::capture(&state, 7, ITEM).unwrap();
    let before = serde_json::to_value(&state).unwrap();
    assert_eq!(
        skills::item_request(&state, 7, 107, &item).unwrap(),
        [0x14, 13, 2, 0x98, 0xba, 0xdc, 0xfe]
    );
    assert!(ItemTarget::capture(&state, 7, 1).is_err());
    for target in [None, Some(1), Some(2), Some(ITEM)] {
        assert!(skills::request(&state.world, 107, target).is_err());
    }
    assert!(skills::ground_request(&state.world, 107, [32768, 32768]).is_err());
    assert!(skills::item_request(&state, 7, 108, &item).is_err());
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn shadow_fang_revokes_inventory_aba_scene_owner_and_observer_changes() {
    let state = snapshot();
    let item = ItemTarget::capture(&state, 7, ITEM).unwrap();
    assert!(skills::item_request(&state, 8, 107, &item).is_err());
    for change in 0..13 {
        let mut stale = state.clone();
        match change {
            0 => stale.inventory.owner.as_mut().unwrap().selection += 1,
            1 => stale.world.generation += 1,
            2 => stale.world.control_interruptions += 1,
            3 => stale.world.local_corrections += 1,
            4 => stale.world.actors.get_mut(&1).unwrap().incarnation += 1,
            5 => stale.inventory.revision += 2,
            6 => stale.inventory.items.get_mut(&ITEM).unwrap().detail = vec![1],
            7 => stale.world.map_id = None,
            8 => stale.game_connected = false,
            9 => stale.phase = LoginPhase::LobbyReady,
            10 => stale.observer = true,
            11 => stale.world.observer_mode = true,
            _ => {
                stale.inventory.items.remove(&ITEM);
            }
        }
        assert!(
            skills::item_request(&stale, 7, 107, &item).is_err(),
            "change={change}"
        );
    }
}

#[test]
fn menu_exposes_this_batch_and_retains_existing_selector_behavior() {
    let mut state = snapshot();
    let mut menu = Model::default();
    menu.sync(7, &state.world, true);
    assert_eq!(menu.rows().len(), 9);
    assert!(menu
        .rows()
        .iter()
        .all(|row| !row.label.contains("尚未支援")));
    menu.select(Some(107)).unwrap();
    assert_eq!(menu.selected_skill(7, &state.world).unwrap(), 107);
    assert!(menu.choice(7, &state.world, None).is_err());
    assert!(menu.selected_skill(8, &state.world).is_err());
    state.world.learned_skills.remove(&107);
    assert!(menu.selected_skill(7, &state.world).is_err());
    menu.sync(7, &state.world, true);
    assert_eq!(menu.selected(), None);
    for skill in [14, 31, 32, 49, 52, 54, 72, 78, 206] {
        state.world.learned_skills.insert(skill);
        assert!(skills::selector_only(skill));
        assert_eq!(
            native_skill_selection::choice(&state.world, skill, Target::capture(&state.world, 2))
                .unwrap()
                .target,
            None
        );
    }
    for skill in [221, u16::MAX] {
        state.world.learned_skills.insert(skill);
        assert!(!skills::selector_only(skill));
        assert!(!skills::item_targeted(skill));
        assert!(skills::request(&state.world, skill, None).is_err());
    }
    assert_eq!(
        (101..=109).map(skills::reuse_millis).collect::<Vec<_>>(),
        [0, 2990, 990, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn received_effect_speed_strength_and_inventory_updates_remain_authoritative() {
    let mut state = snapshot();
    for graphics in [2945u16, 2946, 2947, 2948, 2949, 11765, 2951, 2952, 3909] {
        let mut body = vec![0x37, 1, 0, 0, 0];
        body.extend(graphics.to_le_bytes());
        let before = serde_json::to_value(&state).unwrap();
        assert!(state.world.accept(&body[..6]).is_err());
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
        state.world.accept(&body).unwrap();
        assert_eq!(state.world.effects.last().unwrap().graphics, graphics);
        assert_eq!(
            state.world.effects.last().unwrap().target,
            Some((1, state.world.actors[&1].incarnation))
        );
    }
    state.world.accept(&[0x62, 1, 0, 0, 0, 4, 0xc0, 3]).unwrap();
    assert_eq!(
        (
            state.world.actors[&1].brave,
            state.world.actors[&1].brave_seconds
        ),
        (4, 960)
    );
    state.world.accept(&[0x6b, 0xc0, 3, 22, 7, 2]).unwrap();
    assert_eq!(state.world.status.unwrap().strength, 22);
    assert_eq!(state.world.ability_buffs[&0x6b].received_seconds, 960);
    let item = ItemTarget::capture(&state, 7, ITEM).unwrap();
    state
        .inventory
        .accept(
            7,
            &[0x6f, 0x98, 0xba, 0xdc, 0xfe, b'W', 0, 1, 0, 0, 0, 1, 7],
        )
        .unwrap();
    assert_eq!(state.inventory.items[&ITEM].detail, [7]);
    assert!(skills::item_request(&state, 7, 107, &item).is_err());
    state.world.accept(&[0x62, 1, 0, 0, 0, 0, 0, 0]).unwrap();
    assert_eq!(state.world.actors[&1].brave, 0);
    state.world.accept(&[0x28, 0x34, 2, 0]).unwrap();
    assert!(state.world.effects.is_empty());
}

#[test]
fn item_rejection_is_scoped_and_cast_feedback_never_infers_success() {
    let mut state = snapshot();
    let now = Instant::now();
    let initial = pending(&state, now);
    for code in [161u16, 308] {
        let [low, high] = code.to_le_bytes();
        state.world.accept(&[0x10, low, high, 0]).unwrap();
        assert_eq!(initial.observe_item(&state.world, now), Feedback::Pending);
    }
    state.world.accept(&[0x10, 79, 0, 0]).unwrap();
    assert_eq!(
        initial.observe_item(&state.world, now),
        Feedback::Rejected(79)
    );
    assert_eq!(initial.observe(&state.world, now), Feedback::Pending);
    let next = pending(&state, now);
    assert_eq!(next.observe_item(&state.world, now), Feedback::Pending);
    assert_eq!(
        next.observe_item(&state.world, now + Duration::from_secs(2)),
        Feedback::NoResponse
    );
    for code in [278u16, 280, 299] {
        let [low, high] = code.to_le_bytes();
        state.world.accept(&[0x10, low, high, 0]).unwrap();
        assert_eq!(
            next.observe_item(&state.world, now),
            Feedback::Rejected(code)
        );
    }
    let cast = pending(&state, now);
    state.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(cast.observe_item(&state.world, now), Feedback::CastAction);
    state.world.generation += 1;
    assert_eq!(cast.observe_item(&state.world, now), Feedback::Interrupted);
}
