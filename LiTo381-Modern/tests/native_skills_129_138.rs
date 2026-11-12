//! T022 batch 3 (#153): skills 129, 130, 132–134, 137, 138; request shapes per
//! PROTOCOL-CONTRACT §6.3, range/reuse per the E149 catalog.
use lito_modern::modern_lito::{
    native_inventory::{Item, Owner},
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{self, Model, Target},
    native_skills::{self as skills, Feedback, Pending},
    native_triple_arrow as arrows,
};
use std::time::Instant;

const PEER: u32 = 0xfedcba98;
const IDS: [u16; 7] = [129, 130, 132, 133, 134, 137, 138];
const SELECTORS: [(u16, u8, u8); 5] =
    [(129, 16, 0), (130, 16, 1), (134, 16, 5), (137, 17, 0), (138, 17, 1)];
const TARGETED: [(u16, u8, u8); 2] = [(132, 16, 3), (133, 16, 4)];

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
    let mut own = state.world.actors.remove(&PEER).unwrap();
    own.id = 1;
    own.x = 32768;
    own.y = 32768;
    own.walk_action = skills::BOW_WALK_ACTION;
    state.world.actors.insert(1, own);
    state.world.status.as_mut().unwrap().object_id = 1;
    let mut actor = state.world.actors.remove(&2).unwrap();
    actor.id = PEER;
    actor.x = 32769;
    actor.y = 32768;
    state.world.actors.insert(PEER, actor);
    state.world.hp = Some([20, 50]);
    state.world.mp = Some([30, 200]);
    // Real book decoder: row 16 bits 0,1,3,4,5 (129,130,132–134), row 17 bits 0,1 (137,138).
    let mut book = vec![0x1e, 18];
    book.extend([0; 16]);
    book.extend([0x3b, 0x03]);
    state.world.accept(&book).unwrap();
    state.phase = LoginPhase::BattlePending;
    state.game_connected = true;
    state
}

fn tail(row: u8, column: u8, id: u32, x: u16, y: u16) -> Vec<u8> {
    let mut b = vec![0x14, row, column];
    b.extend(id.to_le_bytes());
    b.extend(x.to_le_bytes());
    b.extend(y.to_le_bytes());
    b
}

fn place(state: &mut Snapshot, dx: i32, dy: i32) {
    let actor = state.world.actors.get_mut(&PEER).unwrap();
    actor.x = (32768 + dx) as u16;
    actor.y = (32768 + dy) as u16;
}

fn arrow(id: u32, use_type: u8, count: u32) -> Item {
    Item { id, use_type, charge: 0, icon: 7, blessing: 1, count, identified: 1, name: "$61".into(), detail: vec![] }
}

#[test]
fn selectors_clear_stale_hover_and_emit_exact_three_bytes_without_spending() {
    let mut state = snapshot();
    assert_eq!(state.world.learned_skills.iter().copied().collect::<Vec<_>>(), IDS);
    let target = Target::capture(&state.world, PEER);
    state.world.actors.remove(&PEER);
    let before = serde_json::to_value(&state).unwrap();
    for (skill, row, column) in SELECTORS {
        assert!(skills::selector_only(skill) && !skills::text_skill(skill) && !skills::item_targeted(skill));
        let choice = native_skill_selection::choice(&state.world, skill, target).unwrap();
        assert_eq!(choice.target, None);
        assert_eq!(skills::request(&state.world, skill, None).unwrap(), [0x14, row, column]);
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
        assert!(skills::text_request(&state.world, skill, None, "x").is_err());
    }
    // 131 keeps its own arm and 135／136 stay outside this batch.
    for skill in [131u16, 135, 136, 139] {
        assert!(!skills::selector_only(skill), "{skill}");
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn triple_arrow_and_elemental_fall_down_send_unsigned_id_and_current_coordinates() {
    let mut state = snapshot();
    place(&mut state, 3, -2);
    let before = serde_json::to_value(&state).unwrap();
    for (skill, row, column) in TARGETED {
        assert!(!skills::selector_only(skill) && !skills::text_skill(skill));
        let body = skills::request(&state.world, skill, Some(PEER)).unwrap();
        assert_eq!(body, tail(row, column, PEER, 32771, 32766));
        assert_eq!(body.len(), 11);
        let choice =
            native_skill_selection::choice(&state.world, skill, Target::capture(&state.world, PEER)).unwrap();
        assert_eq!(choice.target, Some(PEER));
        // Neither spell may fall back to, or be aimed at, the caster.
        assert!(skills::request(&state.world, skill, None).is_err());
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
        assert!(skills::request(&state.world, skill, Some(0x7fff_fff0)).is_err());
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn targets_recheck_screen_range_life_visibility_and_identity() {
    // (dx, dy, 132 screen-only, 133 screen and ten cells)
    for (dx, dy, triple, fall) in [
        (10, 0, true, true),
        (10, 8, true, true),
        (-10, -8, true, true),
        (10, 10, false, false),
        (11, 0, true, false),
        (18, 0, true, false),
        (-9, 9, true, true),
        (20, 0, false, false),
        (0, -20, false, false),
    ] {
        let mut state = snapshot();
        place(&mut state, dx, dy);
        assert_eq!(skills::request(&state.world, 132, Some(PEER)).is_ok(), triple, "132 dx={dx} dy={dy}");
        assert_eq!(skills::request(&state.world, 133, Some(PEER)).is_ok(), fall, "133 dx={dx} dy={dy}");
    }
    for skill in [132u16, 133] {
        for change in 0..5 {
            let mut state = snapshot();
            let target = Target::capture(&state.world, PEER);
            match change {
                0 => {
                    state.world.actors.remove(&PEER);
                }
                1 => state.world.actors.get_mut(&PEER).unwrap().dead = true,
                2 => state.world.actors.get_mut(&PEER).unwrap().incarnation += 1,
                3 => {
                    state.world.accept(&[0x34, 0x98, 0xba, 0xdc, 0xfe, 1]).unwrap();
                }
                _ => state.world.generation += 1,
            }
            assert!(
                native_skill_selection::choice(&state.world, skill, target).is_err(),
                "skill={skill} change={change}"
            );
            if change < 2 || change == 3 {
                assert!(skills::request(&state.world, skill, Some(PEER)).is_err(), "skill={skill} change={change}");
            }
        }
    }
}

#[test]
fn triple_arrow_requires_bow_stance_and_server_counted_arrows() {
    let mut state = snapshot();
    for stance in [0u8, 4, 21, 62] {
        state.world.actors.get_mut(&1).unwrap().walk_action = stance;
        assert!(skills::request(&state.world, 132, Some(PEER)).is_err(), "stance={stance}");
        // The stance gate belongs to Triple Arrow only.
        assert!(skills::request(&state.world, 133, Some(PEER)).is_ok(), "stance={stance}");
    }
    state.world.actors.get_mut(&1).unwrap().walk_action = skills::BOW_WALK_ACTION;
    assert!(skills::request(&state.world, 132, Some(PEER)).is_ok());

    // Controller preflight: only 132 needs arrows, and never without an owned inventory.
    assert!(arrows::preflight(132, &state.inventory).is_err());
    for skill in [129u16, 130, 133, 134, 137, 138] {
        assert_eq!(arrows::preflight(skill, &state.inventory).unwrap(), None, "{skill}");
    }
    state.inventory.owner = Some(Owner { connection: 1, selection: 1, character: "elf".into() });
    state.inventory.items.insert(5, arrow(5, 0, 10));
    assert!(arrows::preflight(132, &state.inventory).is_err(), "non-arrow items are not ammunition");
    state.inventory.items.insert(PEER, arrow(PEER, arrows::ARROW_USE_TYPE, 10));
    let ledger = arrows::preflight(132, &state.inventory).unwrap().unwrap();
    assert_eq!(ledger.expected(), 3);
    // Nothing is shown or spent until the server's inventory receipt arrives.
    assert_eq!(arrows::receipt(&ledger, &state.inventory), None);
    let mut count = vec![0x1a];
    count.extend(PEER.to_le_bytes());
    count.extend(7u32.to_le_bytes());
    state.inventory.accept(1, &count).unwrap();
    assert_eq!(arrows::receipt(&ledger, &state.inventory).as_deref(), Some("伺服器扣除 3 支箭"));
    // A later ordinary shot is not attributed to the cast.
    let mut count = vec![0x1a];
    count.extend(PEER.to_le_bytes());
    count.extend(6u32.to_le_bytes());
    state.inventory.accept(1, &count).unwrap();
    assert_eq!(arrows::receipt(&ledger, &state.inventory), None);
}

#[test]
fn body_to_mind_is_the_only_cast_admitted_with_an_empty_mana_pool() {
    for mp in [Some([0, 200]), None] {
        let mut state = snapshot();
        state.world.mp = mp;
        let before = serde_json::to_value(&state).unwrap();
        assert_eq!(skills::request(&state.world, 130, None).unwrap(), [0x14, 16, 1]);
        for skill in [129u16, 134, 137, 138] {
            assert!(skills::request(&state.world, skill, None).is_err(), "skill={skill}");
        }
        for skill in [132u16, 133] {
            assert!(skills::request(&state.world, skill, Some(PEER)).is_err(), "skill={skill}");
        }
        // HP cost, MP gain and the 279 rejection stay server-owned.
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
}

#[test]
fn every_skill_rechecks_learning_life_and_control_without_prediction() {
    for skill in IDS {
        for change in 0..6 {
            let mut state = snapshot();
            match change {
                0 => {
                    state.world.learned_skills.remove(&skill);
                }
                1 => state.world.hp = Some([0, 50]),
                2 => state.world.hp = None,
                3 => state.world.actors.get_mut(&1).unwrap().dead = true,
                4 => {
                    state.world.movement_blocks.insert(1);
                }
                _ => state.world.battle_result = Some(0),
            }
            let before = serde_json::to_value(&state).unwrap();
            let target = matches!(skill, 132 | 133).then_some(PEER);
            assert!(skills::request(&state.world, skill, target).is_err(), "skill={skill} change={change}");
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
    }
}

#[test]
fn menu_exposes_this_batch_with_formal_reuse_and_rejections() {
    let mut state = snapshot();
    let mut menu = Model::default();
    menu.sync(7, &state.world, true);
    assert_eq!(menu.rows().iter().map(|row| row.id).collect::<Vec<_>>(), IDS);
    assert!(menu.rows().iter().all(|row| !row.label.contains("尚未支援")), "{:?}", menu.rows());
    menu.select(Some(132)).unwrap();
    let target = Target::capture(&state.world, PEER);
    assert_eq!(menu.choice(7, &state.world, target).unwrap().target, Some(PEER));
    assert!(menu.choice(7, &state.world, None).is_err());
    menu.select(Some(134)).unwrap();
    assert_eq!(menu.choice(7, &state.world, target).unwrap().target, None);
    assert_eq!(IDS.map(skills::reuse_millis), [0, 100, 100, 100, 1000, 0, 0]);

    // 278 MP／279 HP／299 reagent／280 failed roll／285 silence・polymorph／316 overweight.
    let now = Instant::now();
    for code in [278u16, 279, 299, 280, 285, 316] {
        let pending = Pending {
            since: now,
            casts: state.world.own_cast_packets,
            messages: state.world.message_count,
            generation: state.world.generation,
            incarnation: state.world.own_actor().unwrap().incarnation,
        };
        assert_eq!(pending.observe(&state.world, now), Feedback::Pending);
        let [low, high] = code.to_le_bytes();
        state.world.accept(&[0x10, low, high, 0]).unwrap();
        assert_eq!(pending.observe(&state.world, now), Feedback::Rejected(code));
    }
}
