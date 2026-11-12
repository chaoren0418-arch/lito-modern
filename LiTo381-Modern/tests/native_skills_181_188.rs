//! T022 batch 7 (#157): Dragon Knight skills 181–188, request shapes per
//! PROTOCOL-CONTRACT §6.3 (① for 181／182／185／186, ⑩ for 183／184／187／188).
use lito_modern::modern_lito::{
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{self, Model, Target},
    native_skills::{self as skills, Feedback, Pending},
};
use std::time::{Duration, Instant};

const PEER: u32 = 0xfedcba98;
const SELECTORS: [(u16, u8, u8); 4] = [(181, 22, 4), (182, 22, 5), (185, 23, 0), (186, 23, 1)];
/// (skill, row, column, E149 maximumRange)
const ACTOR: [(u16, u8, u8, i32); 4] =
    [(183, 22, 6, 3), (184, 22, 7, 10), (187, 23, 2, 2), (188, 23, 3, 3)];

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
    state.world.actors.insert(1, own);
    state.world.status.as_mut().unwrap().object_id = 1;
    let mut actor = state.world.actors.remove(&2).unwrap();
    actor.id = PEER;
    actor.x = 32769;
    actor.y = 32768;
    state.world.actors.insert(PEER, actor);
    state.world.hp = Some([50, 50]);
    state.world.mp = Some([1, 200]);
    // Real book decoder: row 22 bits 4..7 (181–184), row 23 bits 0..3 (185–188).
    let mut book = vec![0x1e, 24];
    book.extend([0; 22]);
    book.extend([0xf0, 0x0f]);
    state.world.accept(&book).unwrap();
    state.phase = LoginPhase::BattlePending;
    state.game_connected = true;
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

fn place(state: &mut Snapshot, dx: i32, dy: i32) -> [u16; 2] {
    let actor = state.world.actors.get_mut(&PEER).unwrap();
    actor.x = (32768 + dx) as u16;
    actor.y = (32768 + dy) as u16;
    [actor.x, actor.y]
}

fn actor_body(row: u8, column: u8, position: [u16; 2]) -> Vec<u8> {
    let mut b = vec![0x14, row, column];
    b.extend(PEER.to_le_bytes());
    b.extend(position[0].to_le_bytes());
    b.extend(position[1].to_le_bytes());
    b
}

#[test]
fn selectors_clear_stale_hover_and_emit_exact_three_bytes_without_spending() {
    let mut state = snapshot();
    assert_eq!(
        state.world.learned_skills.iter().copied().collect::<Vec<_>>(),
        (181..=188).collect::<Vec<_>>()
    );
    let target = Target::capture(&state.world, PEER);
    state.world.actors.remove(&PEER);
    let before = serde_json::to_value(&state).unwrap();
    for (skill, row, column) in SELECTORS {
        assert!(skills::selector_only(skill) && !skills::self_object(skill));
        assert!(!skills::text_skill(skill) && !skills::item_targeted(skill));
        let choice = native_skill_selection::choice(&state.world, skill, target).unwrap();
        assert_eq!((choice.target, choice.ground), (None, None));
        assert_eq!(skills::request(&state.world, skill, None).unwrap(), [0x14, row, column]);
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
        assert!(skills::request(&state.world, skill, Some(PEER)).is_err());
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn explicit_targets_send_eleven_bytes_with_current_coordinates_inside_formal_range() {
    for (skill, row, column, range) in ACTOR {
        assert!(!skills::selector_only(skill) && !skills::self_object(skill));
        let mut state = snapshot();
        let target = Target::capture(&state.world, PEER);
        for (dx, dy) in [
            (range, 0),
            (0, -range),
            (-range, range - 1),
            (range, range),
            (range + 1, 0),
            (0, -(range + 1)),
            (-(range + 1), range + 1),
        ] {
            // 184 additionally requires the native screen diamond (formal
            // useAreaAttackSpell); for range ≤ 3 the diamond always holds.
            let accepted = dx.abs().max(dy.abs()) <= range
                && (skill != 184 || skills::in_native_screen([32768, 32768], [(32768 + dx) as u16, (32768 + dy) as u16]));
            let position = place(&mut state, dx, dy);
            let before = serde_json::to_value(&state).unwrap();
            let choice = native_skill_selection::choice(&state.world, skill, target);
            assert_eq!(choice.is_ok(), accepted, "skill={skill} dx={dx} dy={dy}");
            let body = skills::request(&state.world, skill, Some(PEER)).ok();
            assert_eq!(body, accepted.then(|| actor_body(row, column, position)), "skill={skill} dx={dx} dy={dy}");
            // DragonSkillRequests.dispatch rejects every length except 11.
            assert!(body.is_none_or(|b| b.len() == 11));
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
        // The server never guesses a victim for these: no target, no request.
        assert!(skills::request(&state.world, skill, None).is_err());
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
        assert!(native_skill_selection::choice(&state.world, skill, None).is_err());
    }
}

#[test]
fn magma_breath_applies_the_asymmetric_screen_diamond_at_ten_cells() {
    let mut state = snapshot();
    for (dx, dy, accepted) in [(-10, 9, true), (10, 9, false), (10, 10, false), (-10, 8, true)] {
        let position = place(&mut state, dx, dy);
        assert_eq!(
            skills::request(&state.world, 184, Some(PEER)).ok(),
            accepted.then(|| actor_body(22, 7, position)),
            "dx={dx} dy={dy}"
        );
    }
}

#[test]
fn actor_spells_reject_removed_hidden_dead_and_replaced_targets() {
    for (skill, ..) in ACTOR {
        for change in 0..5 {
            let mut state = snapshot();
            let target = Target::capture(&state.world, PEER);
            assert!(native_skill_selection::choice(&state.world, skill, target).is_ok());
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
            if matches!(change, 0 | 1 | 3) {
                assert!(skills::request(&state.world, skill, Some(PEER)).is_err(), "skill={skill} change={change}");
            }
        }
    }
}

#[test]
fn every_skill_rechecks_learning_life_and_control_but_not_the_unused_mp_bar() {
    for skill in 181..=188u16 {
        let target = (!skills::selector_only(skill)).then_some(PEER);
        assert!(skills::request(&snapshot().world, skill, target).is_ok(), "skill={skill}");
        for change in 0..5 {
            let mut state = snapshot();
            match change {
                0 => {
                    state.world.learned_skills.remove(&skill);
                }
                1 => state.world.hp = Some([0, 50]),
                2 => state.world.actors.get_mut(&1).unwrap().dead = true,
                3 => {
                    state.world.movement_blocks.insert(1);
                }
                _ => state.world.battle_result = Some(0),
            }
            let before = serde_json::to_value(&state).unwrap();
            assert!(skills::request(&state.world, skill, target).is_err(), "skill={skill} change={change}");
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
        // E149 mpCost is 0 for the whole batch: the formal handlers only test
        // `playerHp <= hpCost` (S10 279), so an empty MP bar is not a refusal.
        assert!(skills::hp_paid(skill));
        let mut state = snapshot();
        state.world.mp = Some([0, 200]);
        let before = serde_json::to_value(&state).unwrap();
        assert!(skills::request(&state.world, skill, target).is_ok(), "skill={skill} mp=0");
        // Low HP is not predicted either; the server answers 279.
        state.world.hp = Some([1, 50]);
        assert!(skills::request(&state.world, skill, target).is_ok(), "skill={skill} hp=1");
        state.world.hp = Some([50, 50]);
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
    for skill in (1..=220u16).filter(|id| !(181..=188).contains(id)) {
        assert!(!skills::hp_paid(skill), "skill={skill}");
    }
}

#[test]
fn menu_exposes_this_batch_with_formal_reuse_and_cast_clock() {
    let state = snapshot();
    let mut menu = Model::default();
    menu.sync(7, &state.world, true);
    assert_eq!(menu.rows().len(), 8);
    assert!(menu.rows().iter().all(|row| !row.label.contains("尚未支援")), "{:?}", menu.rows());
    for (index, row) in menu.rows().iter().enumerate() {
        assert_eq!(usize::from(row.id), 181 + index);
        assert!(row.label.starts_with(&format!("#{} ", row.id)), "{}", row.label);
    }
    let hover = Target::capture(&state.world, PEER);
    menu.select(Some(181)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, None);
    menu.select(Some(187)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, Some(PEER));
    assert!(menu.choice(7, &state.world, None).is_err());
    // E149 catalog reuseDelayMillis and clientCastClockMillis for 181..=188.
    assert_eq!(
        (181..=188).map(skills::reuse_millis).collect::<Vec<_>>(),
        [10, 2000, 1000, 1000, 0, 10, 2000, 1000]
    );
    assert_eq!(
        (181..=188).map(skills::client_clock_millis).collect::<Vec<_>>(),
        [0, 3600, 1800, 0, 0, 0, 0, 1200]
    );
}

#[test]
fn hp_shortage_and_probability_failure_are_sequenced_rejections() {
    let mut state = snapshot();
    let now = Instant::now();
    // 279 (playerHp <= hpCost) arrives instead of the cast action.
    let cast = pending(&state, now);
    assert_eq!(cast.observe(&state.world, now), Feedback::Pending);
    state.world.accept(&[0x10, 0x17, 1, 0]).unwrap();
    assert_eq!(cast.observe(&state.world, now), Feedback::Rejected(279));
    assert_ne!(Feedback::Rejected(279).label(), Feedback::Rejected(1).label());
    // An older 279 is not attributed to the next cast.
    let next = pending(&state, now);
    assert_eq!(next.observe(&state.world, now), Feedback::Pending);
    assert_eq!(next.observe(&state.world, now + Duration::from_secs(2)), Feedback::NoResponse);
    // 183／188 failureMessage 280 received in the same poll as the cast action
    // wins over the action.
    state.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(next.observe(&state.world, now), Feedback::CastAction);
    state.world.accept(&[0x10, 0x18, 1, 0]).unwrap();
    assert_eq!(next.observe(&state.world, now), Feedback::Rejected(280));
    state.world.generation += 1;
    assert_eq!(next.observe(&state.world, now), Feedback::Interrupted);
}

#[test]
fn foe_slayer_is_acknowledged_by_its_own_physical_strikes() {
    let mut state = snapshot();
    let now = Instant::now();
    let late = now + Duration::from_secs(2);
    // 187: the formal handler sends no S_ACTION; it runs three direct physical
    // attacks, so the own physical reply count is the acknowledgement.
    let cast = pending(&state, now);
    let attacks = state.world.own_attack_packets;
    assert_eq!(cast.observe_strikes(&state.world, now, attacks), Feedback::Pending);
    assert_eq!(cast.observe_strikes(&state.world, late, attacks), Feedback::NoResponse);
    state.world.own_attack_packets += 1;
    assert_eq!(cast.observe_strikes(&state.world, now, attacks), Feedback::CastAction);
    assert_eq!(cast.observe_strikes(&state.world, late, attacks), Feedback::CastAction);
    // Ordinary casts never read physical replies.
    assert_eq!(cast.observe(&state.world, now), Feedback::Pending);
    assert_eq!(cast.observe(&state.world, late), Feedback::NoResponse);
    // A reply counted before the request is not an acknowledgement.
    let next = pending(&state, now);
    let attacks = state.world.own_attack_packets;
    assert_eq!(next.observe_strikes(&state.world, now, attacks), Feedback::Pending);
    // A coded rejection and an interruption keep their priority.
    state.world.accept(&[0x10, 0x17, 1, 0]).unwrap();
    state.world.own_attack_packets += 1;
    assert_eq!(next.observe_strikes(&state.world, now, attacks), Feedback::Rejected(279));
    state.world.generation += 1;
    assert_eq!(next.observe_strikes(&state.world, now, attacks), Feedback::Interrupted);
}