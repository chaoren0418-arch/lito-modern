//! T022 batch 4 (#154): skills 145–155, request shapes per PROTOCOL-CONTRACT §6.3.
use lito_modern::modern_lito::{
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{self, Model, Target},
    native_skills::{self as skills, Feedback, Pending},
};
use std::time::{Duration, Instant};

const PEER: u32 = 0xfedcba98;
const SELECTORS: [(u16, u8, u8); 5] =
    [(146, 18, 1), (147, 18, 2), (150, 18, 5), (154, 19, 1), (155, 19, 2)];
const SELF_OBJECT: [(u16, u8, u8); 3] = [(148, 18, 3), (149, 18, 4), (151, 18, 6)];
const ACTOR: [(u16, u8, u8); 2] = [(152, 18, 7), (153, 19, 0)];

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
    state.world.hp = Some([1, 50]);
    state.world.mp = Some([1, 200]);
    // Real book decoder: row 18 bits 0..7 (145–152), row 19 bits 0..2 (153–155).
    let mut book = vec![0x1e, 20];
    book.extend([0; 18]);
    book.extend([0xff, 0x07]);
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
        (145..=155).collect::<Vec<_>>()
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
fn self_object_buffs_always_send_the_caster_id_and_never_a_hovered_actor() {
    let mut state = snapshot();
    let before = serde_json::to_value(&state).unwrap();
    let hover = Target::capture(&state.world, PEER);
    for (skill, row, column) in SELF_OBJECT {
        assert!(skills::self_object(skill) && !skills::selector_only(skill));
        let expected = [0x14, row, column, 1, 0, 0, 0];
        // The formal handlers reject every object ID except the caster's.
        let choice = native_skill_selection::choice(&state.world, skill, hover).unwrap();
        assert_eq!(choice.target, None);
        assert_eq!(skills::request(&state.world, skill, choice.target).unwrap(), expected);
        assert_eq!(skills::request(&state.world, skill, Some(1)).unwrap(), expected);
        assert!(skills::request(&state.world, skill, Some(PEER)).is_err());
        assert!(skills::request(&state.world, skill, Some(0)).is_err());
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
    // A stale hover does not block a self buff.
    state.world.actors.remove(&PEER);
    for (skill, row, column) in SELF_OBJECT {
        let choice = native_skill_selection::choice(&state.world, skill, hover).unwrap();
        assert_eq!(choice.target, None);
        assert_eq!(skills::request(&state.world, skill, None).unwrap(), [0x14, row, column, 1, 0, 0, 0]);
    }
    for skill in (1..=220u16).filter(|id| !matches!(id, 148 | 149 | 151)) {
        assert!(!skills::self_object(skill), "skill={skill}");
    }
}

#[test]
fn entangle_and_erase_use_current_coordinates_ten_cells_and_the_screen_diamond() {
    for (skill, row, column) in ACTOR {
        let mut state = snapshot();
        let target = Target::capture(&state.world, PEER);
        for (dx, dy, accepted) in [
            (10, 0, true),
            (0, -10, true),
            (-10, 8, true),
            // Distance 19 is inside the diamond only on its asymmetric edge.
            (-10, 9, true),
            (10, 9, false),
            (10, 10, false),
            (11, 0, false),
            (0, -11, false),
        ] {
            let position = place(&mut state, dx, dy);
            let before = serde_json::to_value(&state).unwrap();
            let choice = native_skill_selection::choice(&state.world, skill, target);
            assert_eq!(choice.is_ok(), accepted, "skill={skill} dx={dx} dy={dy}");
            assert_eq!(
                skills::request(&state.world, skill, Some(PEER)).ok(),
                accepted.then(|| actor_body(row, column, position)),
                "skill={skill} dx={dx} dy={dy}"
            );
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
        assert!(skills::request(&state.world, skill, None).is_err());
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
    }
}

#[test]
fn return_to_nature_requires_an_explicit_other_actor_inside_the_screen() {
    let mut state = snapshot();
    let target = Target::capture(&state.world, PEER);
    for (dx, dy, accepted) in [(1, 0, true), (18, 0, true), (-9, 9, true), (20, 0, false), (0, -20, false)] {
        let position = place(&mut state, dx, dy);
        let before = serde_json::to_value(&state).unwrap();
        let choice = native_skill_selection::choice(&state.world, 145, target);
        assert_eq!(choice.is_ok(), accepted, "dx={dx} dy={dy}");
        assert_eq!(
            skills::request(&state.world, 145, Some(PEER)).ok(),
            accepted.then(|| actor_body(18, 0, position)),
            "dx={dx} dy={dy}"
        );
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
    // Never a silent self fallback: the caster is not a releasable companion.
    assert!(skills::request(&state.world, 145, None).is_err());
    assert!(skills::request(&state.world, 145, Some(1)).is_err());
    assert!(native_skill_selection::choice(&state.world, 145, None).is_err());
}

#[test]
fn actor_spells_reject_removed_hidden_dead_and_replaced_targets() {
    for skill in [145u16, 152, 153] {
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
fn every_skill_rechecks_learning_life_mana_and_control_without_prediction() {
    for skill in 145..=155u16 {
        let target = (!skills::selector_only(skill) && !skills::self_object(skill)).then_some(PEER);
        assert!(skills::request(&snapshot().world, skill, target).is_ok(), "skill={skill}");
        for change in 0..7 {
            let mut state = snapshot();
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
            assert!(skills::request(&state.world, skill, target).is_err(), "skill={skill} change={change}");
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
    }
}

#[test]
fn menu_exposes_this_batch_with_formal_reuse() {
    let state = snapshot();
    let mut menu = Model::default();
    menu.sync(7, &state.world, true);
    assert_eq!(menu.rows().len(), 11);
    assert!(menu.rows().iter().all(|row| !row.label.contains("尚未支援")), "{:?}", menu.rows());
    for (index, row) in menu.rows().iter().enumerate() {
        assert_eq!(usize::from(row.id), 145 + index);
        assert!(row.label.starts_with(&format!("#{} ", row.id)), "{}", row.label);
    }
    menu.select(Some(151)).unwrap();
    let hover = Target::capture(&state.world, PEER);
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, None);
    menu.select(Some(152)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, Some(PEER));
    assert!(menu.choice(7, &state.world, None).is_err());
    // E149 catalog reuseDelayMillis: 146 = 1200, 153 = 600, the rest 0.
    assert_eq!(
        (145..=155).map(skills::reuse_millis).collect::<Vec<_>>(),
        [0, 1200, 0, 0, 0, 0, 0, 0, 600, 0, 0]
    );
}

#[test]
fn late_release_and_elemental_rejections_follow_the_committed_cast_action() {
    let mut state = snapshot();
    let now = Instant::now();
    // 145: the formal handler publishes the cast action, then S10 79 (not a
    // SUMMON／TAMED companion) or 280 (probability roll failed).
    let release = pending(&state, now);
    assert_eq!(release.observe_release(&state.world, now), Feedback::Pending);
    state.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(release.observe_release(&state.world, now), Feedback::CastAction);
    assert!(release.hold_summon(Feedback::CastAction, now));
    assert!(!release.hold_summon(Feedback::CastAction, now + Duration::from_secs(2)));
    state.world.accept(&[0x10, 79, 0, 0]).unwrap();
    state.world.accept(&[0x10, 161, 0, 0]).unwrap();
    assert_eq!(release.observe_release(&state.world, now), Feedback::Rejected(79));
    assert!(!release.hold_summon(Feedback::Rejected(79), now));
    // Ordinary casts never read 79.
    assert_eq!(release.observe(&state.world, now), Feedback::CastAction);
    assert_ne!(Feedback::Rejected(79).label(), Feedback::Rejected(1).label());
    // An older 79 is not attributed to the next cast.
    let next = pending(&state, now);
    assert_eq!(next.observe_release(&state.world, now), Feedback::Pending);
    assert_eq!(next.observe_release(&state.world, now + Duration::from_secs(2)), Feedback::NoResponse);
    state.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    state.world.accept(&[0x10, 0x18, 1, 0]).unwrap();
    assert_eq!(next.observe_release(&state.world, now), Feedback::Rejected(280));

    // 154: S10 353 (no-recall map) also follows the cast action; it is read
    // through the existing summon observer and never by ordinary casts.
    let elemental = pending(&state, now);
    state.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(elemental.observe_summon(&state.world, now), Feedback::CastAction);
    state.world.accept(&[0x10, 0x61, 1, 0]).unwrap();
    assert_eq!(elemental.observe_summon(&state.world, now), Feedback::Rejected(353));
    assert_eq!(elemental.observe(&state.world, now), Feedback::CastAction);
    assert_eq!(elemental.observe_release(&state.world, now), Feedback::CastAction);

    state.world.generation += 1;
    assert_eq!(release.observe_release(&state.world, now), Feedback::Interrupted);
    assert_eq!(elemental.observe_summon(&state.world, now), Feedback::Interrupted);
}
