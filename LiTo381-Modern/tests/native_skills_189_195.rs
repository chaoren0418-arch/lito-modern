//! T022 batch 8 (#158): skills 189–195, request shapes per PROTOCOL-CONTRACT §6.3
//! (① for 189／190／191／195, ⑩ for the `DragonSkillRequests.TARGETED` 192／193／194).
use lito_modern::modern_lito::{
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{self, Model, Target},
    native_skills::{self as skills, Feedback, Pending},
};
use std::time::{Duration, Instant};

const PEER: u32 = 0xfedcba98;
/// ① selector-only: (skill, row, column).
const SELECTORS: [(u16, u8, u8); 4] = [(189, 23, 4), (190, 23, 5), (191, 23, 6), (195, 24, 2)];
/// ⑩ explicit target: (skill, row, column, E149 maximumRange).
const ACTOR: [(u16, u8, u8, i32); 3] = [(192, 23, 7, 8), (193, 24, 0, 3), (194, 24, 1, 10)];

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
    // Real book decoder: row 23 bits 4..7 (189–192), row 24 bits 0..2 (193–195).
    let mut book = vec![0x1e, 25];
    book.extend([0; 23]);
    book.extend([0xf0, 0x07]);
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

fn usual_target(skill: u16) -> Option<u32> {
    ACTOR.iter().any(|row| row.0 == skill).then_some(PEER)
}

#[test]
fn selectors_clear_stale_hover_and_emit_exact_three_bytes_without_spending() {
    let mut state = snapshot();
    assert_eq!(
        state.world.learned_skills.iter().copied().collect::<Vec<_>>(),
        (189..=195).collect::<Vec<_>>()
    );
    let target = Target::capture(&state.world, PEER);
    state.world.actors.remove(&PEER);
    let before = serde_json::to_value(&state).unwrap();
    for (skill, row, column) in SELECTORS {
        assert_eq!((skill - 1) / 8, u16::from(row));
        assert_eq!((skill - 1) % 8, u16::from(column));
        assert!(skills::selector_only(skill));
        assert!(!skills::self_object(skill) && !skills::self_bound(skill) && !skills::corpse_targeted(skill));
        assert!(!skills::text_skill(skill) && !skills::item_targeted(skill) && !skills::summon_feedback(skill));
        // A stale hover (actor already gone) never blocks or leaks into the cast.
        let choice = native_skill_selection::choice(&state.world, skill, target).unwrap();
        assert_eq!((choice.target, choice.ground), (None, None));
        assert_eq!(skills::request(&state.world, skill, None).unwrap(), [0x14, row, column]);
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
        assert!(skills::request(&state.world, skill, Some(PEER)).is_err());
        assert!(skills::ground_request(&state.world, skill, [32768, 32768]).is_err());
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn explicit_targets_send_eleven_bytes_with_current_coordinates_inside_formal_range() {
    for (skill, row, column, range) in ACTOR {
        assert_eq!((skill - 1) / 8, u16::from(row));
        assert_eq!((skill - 1) % 8, u16::from(column));
        assert!(!skills::selector_only(skill) && !skills::self_object(skill) && !skills::self_bound(skill));
        assert!(!skills::text_skill(skill) && !skills::item_targeted(skill) && !skills::corpse_targeted(skill));
        let mut state = snapshot();
        let target = Target::capture(&state.world, PEER);
        for (dx, dy) in [
            (1, 0),
            (range, 0),
            (0, -range),
            (-range, range - 1),
            (range, range),
            (range + 1, 0),
            (0, -(range + 1)),
            (-(range + 1), range + 1),
        ] {
            // 194 additionally requires the native screen diamond (formal WORLD
            // line-freeze handler); for range ≤ 8 the diamond always holds.
            let accepted = dx.abs().max(dy.abs()) <= range
                && (skill != 194
                    || skills::in_native_screen([32768, 32768], [(32768 + dx) as u16, (32768 + dy) as u16]));
            let position = place(&mut state, dx, dy);
            let before = serde_json::to_value(&state).unwrap();
            let choice = native_skill_selection::choice(&state.world, skill, target);
            assert_eq!(choice.is_ok(), accepted, "skill={skill} dx={dx} dy={dy}");
            if let Ok(choice) = choice {
                assert_eq!((choice.target, choice.ground), (Some(PEER), None));
            }
            let body = skills::request(&state.world, skill, Some(PEER)).ok();
            assert_eq!(body, accepted.then(|| actor_body(row, column, position)), "skill={skill} dx={dx} dy={dy}");
            // DragonSkillRequests.dispatch rejects every length except 11.
            assert!(body.is_none_or(|b| b.len() == 11));
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
        // The server never guesses a victim for these: no target, no request.
        place(&mut state, 1, 0);
        assert!(skills::request(&state.world, skill, None).is_err());
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
        assert!(skills::request(&state.world, skill, Some(0)).is_err());
        assert!(native_skill_selection::choice(&state.world, skill, None).is_err());
        assert!(skills::ground_request(&state.world, skill, [32769, 32768]).is_err());
    }
}

#[test]
fn freezing_breath_applies_the_asymmetric_screen_diamond_at_ten_cells() {
    let mut state = snapshot();
    for (dx, dy, accepted) in [(-10, 9, true), (-10, 8, true), (10, 8, true), (10, 9, false), (10, 10, false), (-10, -10, false)] {
        let position = place(&mut state, dx, dy);
        assert_eq!(
            skills::in_native_screen([32768, 32768], position),
            accepted,
            "dx={dx} dy={dy}"
        );
        assert_eq!(
            skills::request(&state.world, 194, Some(PEER)).ok(),
            accepted.then(|| actor_body(24, 1, position)),
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
    for skill in 189..=195u16 {
        let target = usual_target(skill);
        assert_eq!(target.is_none(), skills::selector_only(skill), "skill={skill}");
        // Not vacuous: the unmodified state admits the request.
        skills::request(&snapshot().world, skill, target).unwrap_or_else(|e| panic!("skill={skill}: {e:#}"));
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
        assert!(skills::hp_cost_only(skill));
        for mp in [Some([0, 200]), None] {
            let mut state = snapshot();
            state.world.mp = mp;
            let before = serde_json::to_value(&state).unwrap();
            assert!(skills::request(&state.world, skill, target).is_ok(), "skill={skill} mp={mp:?}");
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
        // Low HP is not predicted either; the server answers 279.
        let mut state = snapshot();
        state.world.hp = Some([1, 50]);
        assert!(skills::request(&state.world, skill, target).is_ok(), "skill={skill} hp=1");
    }
    for skill in (1..=220u16).filter(|id| !(189..=195).contains(id)) {
        assert!(!skills::hp_cost_only(skill), "skill={skill}");
    }
    // MP-paid skills keep their gate: a learned ordinary selector still needs MP.
    let mut state = snapshot();
    state.world.learned_skills.insert(14);
    assert!(skills::request(&state.world, 14, None).is_ok());
    state.world.mp = Some([0, 200]);
    assert!(skills::request(&state.world, 14, None).is_err());
}

#[test]
fn menu_exposes_this_batch_with_formal_reuse_and_cast_clock() {
    let state = snapshot();
    let mut menu = Model::default();
    menu.sync(7, &state.world, true);
    assert_eq!(menu.rows().len(), 7);
    assert!(menu.rows().iter().all(|row| !row.label.contains("尚未支援")), "{:?}", menu.rows());
    for (index, row) in menu.rows().iter().enumerate() {
        assert_eq!(usize::from(row.id), 189 + index);
        assert!(row.label.starts_with(&format!("#{} ", row.id)), "{}", row.label);
        assert!(row.label.contains("消耗 HP"), "{}", row.label);
    }
    let hover = Target::capture(&state.world, PEER);
    menu.select(Some(190)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, None);
    menu.select(Some(192)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, Some(PEER));
    assert!(menu.choice(7, &state.world, None).is_err());
    assert!(menu.choice(8, &state.world, hover).is_err());
    // E149 catalog reuseDelayMillis and clientCastClockMillis for 189..=195.
    assert_eq!(
        (189..=195).map(skills::reuse_millis).collect::<Vec<_>>(),
        [10, 0, 1000, 4000, 1000, 500, 0]
    );
    assert_eq!(
        (189..=195).map(skills::client_clock_millis).collect::<Vec<_>>(),
        [0, 0, 30, 1800, 0, 1800, 0]
    );
    // Every ID in the batch has exactly one request family.
    for skill in 189..=195u16 {
        let selector = SELECTORS.iter().any(|row| row.0 == skill);
        let actor = ACTOR.iter().any(|row| row.0 == skill);
        assert!(selector ^ actor, "skill={skill}");
        assert_eq!(skills::selector_only(skill), selector, "skill={skill}");
    }
    // Neighbouring unwired IDs stay untouched by this batch.
    for skill in [196u16, 200, 221] {
        assert!(!skills::selector_only(skill) && !skills::hp_cost_only(skill), "skill={skill}");
    }
}

#[test]
fn hp_material_and_probability_rejections_are_sequenced_per_cast() {
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
    // 190／195: missing awakening material is answered with 299 and no action.
    state.world.accept(&[0x10, 0x2b, 1, 0]).unwrap();
    assert_eq!(next.observe(&state.world, now), Feedback::Rejected(299));
    assert_ne!(Feedback::Rejected(299).label(), Feedback::Rejected(1).label());
    // 193 failureMessage 280 received after the cast action wins over the action.
    let third = pending(&state, now);
    state.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(third.observe(&state.world, now), Feedback::CastAction);
    state.world.accept(&[0x10, 0x18, 1, 0]).unwrap();
    assert_eq!(third.observe(&state.world, now), Feedback::Rejected(280));
    state.world.generation += 1;
    assert_eq!(third.observe(&state.world, now), Feedback::Interrupted);
}
