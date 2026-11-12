//! T022 batch 5 (#155): skills 156–166, request shapes per PROTOCOL-CONTRACT §6.3.
use lito_modern::modern_lito::{
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{self, Model, Target},
    native_skills::{self as skills, Feedback, Pending},
};
use std::time::{Duration, Instant};

const PEER: u32 = 0xfedcba98;
const PEER_BYTES: [u8; 4] = [0x98, 0xba, 0xdc, 0xfe];
/// ① selector-only: (skill, row, column).
const SELECTORS: [(u16, u8, u8); 5] = [(156, 19, 3), (159, 19, 6), (161, 20, 0), (162, 20, 1), (164, 20, 3)];
/// ④ with the caster as the only formal target.
const SELF_BOUND: [(u16, u8, u8); 2] = [(163, 20, 2), (166, 20, 5)];
/// ④ living player or self, inside the native screen.
const FRIENDLY: [(u16, u8, u8); 2] = [(158, 19, 5), (160, 19, 7)];

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
    // Real book decoder: row 19 bits 3..7 (156–160), row 20 bits 0..5 (161–166).
    let mut book = vec![0x1e, 21];
    book.extend([0; 19]);
    book.extend([0xf8, 0x3f]);
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

/// The request each skill would normally send against the fixture peer.
fn usual(state: &Snapshot, skill: u16) -> anyhow::Result<Vec<u8>> {
    let target = matches!(skill, 157 | 158 | 160 | 165).then_some(PEER);
    skills::request(&state.world, skill, target)
}

#[test]
fn selectors_clear_stale_hover_and_emit_exact_three_bytes_without_spending() {
    let mut state = snapshot();
    assert_eq!(
        state.world.learned_skills.iter().copied().collect::<Vec<_>>(),
        (156..=166).collect::<Vec<_>>()
    );
    let target = Target::capture(&state.world, PEER);
    state.world.actors.remove(&PEER);
    let before = serde_json::to_value(&state).unwrap();
    for (skill, row, column) in SELECTORS {
        assert!(skills::selector_only(skill));
        assert!(!skills::self_bound(skill) && !skills::corpse_targeted(skill) && !skills::text_skill(skill));
        let choice = native_skill_selection::choice(&state.world, skill, target).unwrap();
        assert_eq!(choice.target, None);
        assert_eq!(skills::request(&state.world, skill, None).unwrap(), [0x14, row, column]);
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
        assert!(skills::ground_request(&state.world, skill, [32768, 32768]).is_err());
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn weapon_buffs_always_send_the_caster_object_and_never_a_hovered_actor() {
    let state = snapshot();
    let before = serde_json::to_value(&state).unwrap();
    for (skill, row, column) in SELF_BOUND {
        assert!(skills::self_bound(skill) && !skills::selector_only(skill));
        let expected = [0x14, row, column, 1, 0, 0, 0];
        // A live hovered peer is dropped by the shared selection path.
        let choice =
            native_skill_selection::choice(&state.world, skill, Target::capture(&state.world, PEER)).unwrap();
        assert_eq!(choice.target, None);
        assert_eq!(skills::request(&state.world, skill, choice.target).unwrap(), expected);
        assert_eq!(skills::request(&state.world, skill, Some(1)).unwrap(), expected);
        // Formal handler answers 280 for any other object; never send it.
        assert!(skills::request(&state.world, skill, Some(PEER)).is_err());
        assert!(skills::request(&state.world, skill, Some(0)).is_err());
    }
    // A stale hover (actor already gone) must not block the self cast either.
    let mut gone = snapshot();
    let stale = Target::capture(&gone.world, PEER);
    gone.world.actors.remove(&PEER);
    for (skill, ..) in SELF_BOUND {
        assert_eq!(native_skill_selection::choice(&gone.world, skill, stale).unwrap().target, None);
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn friendly_buffs_send_unsigned_object_or_self_inside_the_native_screen() {
    for (skill, row, column) in FRIENDLY {
        let mut state = snapshot();
        let target = Target::capture(&state.world, PEER);
        let mut expected = vec![0x14, row, column];
        expected.extend(PEER_BYTES);
        let choice = native_skill_selection::choice(&state.world, skill, target).unwrap();
        assert_eq!(choice.target, Some(PEER));
        assert_eq!(skills::request(&state.world, skill, Some(PEER)).unwrap(), expected);
        // No selection means the caster (formal self fallback).
        assert_eq!(skills::request(&state.world, skill, None).unwrap(), [0x14, row, column, 1, 0, 0, 0]);
        assert_eq!(skills::request(&state.world, skill, Some(1)).unwrap(), [0x14, row, column, 1, 0, 0, 0]);
        for (dx, dy, accepted) in [(18, 0, true), (-9, 9, true), (20, 0, false), (0, -20, false)] {
            place(&mut state, dx, dy);
            let before = serde_json::to_value(&state).unwrap();
            assert_eq!(skills::request(&state.world, skill, Some(PEER)).is_ok(), accepted, "skill={skill} dx={dx} dy={dy}");
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
    }
}

#[test]
fn earth_bind_sends_target_with_current_coordinates_and_rejects_self() {
    let mut state = snapshot();
    let target = Target::capture(&state.world, PEER);
    assert!(!skills::selector_only(157) && !skills::self_bound(157) && !skills::corpse_targeted(157));
    for (dx, dy, accepted) in [(1, 0, true), (18, 0, true), (-9, 9, true), (20, 0, false), (0, -20, false)] {
        let position = place(&mut state, dx, dy);
        let before = serde_json::to_value(&state).unwrap();
        let choice = native_skill_selection::choice(&state.world, 157, target);
        assert_eq!(choice.is_ok(), accepted, "dx={dx} dy={dy}");
        if accepted {
            let mut expected = vec![0x14, 19, 4];
            expected.extend(PEER_BYTES);
            expected.extend(position[0].to_le_bytes());
            expected.extend(position[1].to_le_bytes());
            assert_eq!(expected.len(), 11);
            assert_eq!(skills::request(&state.world, 157, choice.unwrap().target).unwrap(), expected);
        }
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
    place(&mut state, 1, 0);
    assert!(skills::request(&state.world, 157, None).is_err());
    assert!(skills::request(&state.world, 157, Some(1)).is_err());
    assert!(skills::ground_request(&state.world, 157, [32769, 32768]).is_err());
}

#[test]
fn living_target_spells_reject_removed_hidden_dead_and_replaced_targets() {
    for skill in [157u16, 158, 160] {
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
            assert!(native_skill_selection::choice(&state.world, skill, target).is_err(), "skill={skill} change={change}");
        }
        // A corpse is never an acceptable direct target either.
        let mut state = snapshot();
        state.world.actors.get_mut(&PEER).unwrap().dead = true;
        assert!(skills::request(&state.world, skill, Some(PEER)).is_err());
    }
}

#[test]
fn call_of_nature_needs_a_captured_corpse_within_ten_tiles() {
    let mut state = snapshot();
    assert!(skills::corpse_targeted(165) && skills::corpse_targeted(41));
    assert_eq!(skills::CALL_OF_NATURE_RANGE, 10);
    // Living peer: neither the request nor a living selection is accepted.
    let live = Target::capture(&state.world, PEER);
    assert!(skills::request(&state.world, 165, Some(PEER)).is_err());
    assert!(native_skill_selection::choice(&state.world, 165, live).is_err());
    state.world.actors.get_mut(&PEER).unwrap().dead = true;
    // A selection made while alive never upgrades to a corpse authorization.
    assert!(native_skill_selection::choice(&state.world, 165, live).is_err());
    let corpse = Target::capture(&state.world, PEER);
    let mut expected = vec![0x14, 20, 4];
    expected.extend(PEER_BYTES);
    // Chebyshev range from the E149 catalog; the server applies no screen diamond.
    for (dx, dy, accepted) in [(1, 0, true), (10, 10, true), (-10, 3, true), (11, 0, false), (0, -11, false)] {
        place(&mut state, dx, dy);
        let before = serde_json::to_value(&state).unwrap();
        let choice = native_skill_selection::choice(&state.world, 165, corpse);
        assert_eq!(choice.is_ok(), accepted, "dx={dx} dy={dy}");
        if accepted {
            assert_eq!(choice.unwrap().target, Some(PEER));
            assert_eq!(skills::request(&state.world, 165, Some(PEER)).unwrap(), expected);
        }
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
    place(&mut state, 1, 0);
    assert!(skills::request(&state.world, 165, None).is_err());
    assert!(skills::request(&state.world, 165, Some(1)).is_err());
    // The corpse selection is scoped to corpse skills only.
    for skill in [157u16, 158, 160] {
        assert!(native_skill_selection::choice(&state.world, skill, corpse).is_err(), "skill={skill}");
    }
    // Revival, replacement, hiding and a new world all revoke it.
    for change in 0..5 {
        let mut next = state.clone();
        match change {
            0 => next.world.actors.get_mut(&PEER).unwrap().dead = false,
            1 => next.world.actors.get_mut(&PEER).unwrap().incarnation += 1,
            2 => next.world.actors.get_mut(&PEER).unwrap().action_sequence += 1,
            3 => {
                next.world.actors.remove(&PEER);
            }
            _ => next.world.generation += 1,
        }
        assert!(native_skill_selection::choice(&next.world, 165, corpse).is_err(), "change={change}");
    }
}

#[test]
fn every_skill_rechecks_learning_life_mana_and_control_without_prediction() {
    for skill in 156..=166u16 {
        let mut base = snapshot();
        if skill == 165 {
            base.world.actors.get_mut(&PEER).unwrap().dead = true;
        }
        // Not vacuous: the unmodified state admits the request.
        usual(&base, skill).unwrap_or_else(|e| panic!("skill={skill}: {e:#}"));
        for change in 0..7 {
            let mut state = base.clone();
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
            assert!(usual(&state, skill).is_err(), "skill={skill} change={change}");
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
    }
}

#[test]
fn menu_exposes_this_batch_with_formal_reuse_and_classification() {
    let state = snapshot();
    let mut menu = Model::default();
    menu.sync(7, &state.world, true);
    assert_eq!(menu.rows().len(), 11);
    assert!(menu.rows().iter().all(|row| !row.label.contains("尚未支援")), "{:?}", menu.rows());
    menu.select(Some(163)).unwrap();
    assert_eq!(menu.choice(7, &state.world, Target::capture(&state.world, PEER)).unwrap().target, None);
    menu.select(Some(158)).unwrap();
    assert_eq!(menu.choice(7, &state.world, Target::capture(&state.world, PEER)).unwrap().target, Some(PEER));
    assert!(menu.choice(8, &state.world, None).is_err());
    assert_eq!(
        (156..=166).map(skills::reuse_millis).collect::<Vec<_>>(),
        [0, 600, 0, 0, 0, 9000, 0, 0, 300, 0, 0]
    );
    // Every ID in the batch has exactly one request family; none is item/text.
    for skill in 156..=166u16 {
        let families = [
            SELECTORS.iter().any(|row| row.0 == skill),
            SELF_BOUND.iter().any(|row| row.0 == skill),
            FRIENDLY.iter().any(|row| row.0 == skill),
            skill == 157,
            skill == 165,
        ];
        assert_eq!(families.iter().filter(|hit| **hit).count(), 1, "skill={skill}");
        assert_eq!(skills::selector_only(skill), families[0], "skill={skill}");
        assert_eq!(skills::self_bound(skill), families[1], "skill={skill}");
        assert_eq!(skills::corpse_targeted(skill), families[4], "skill={skill}");
        assert!(!skills::item_targeted(skill) && !skills::text_skill(skill), "skill={skill}");
    }
    // Neighbouring batches stay untouched by these helpers.
    for skill in [155u16, 167, 221, u16::MAX] {
        assert!(!skills::self_bound(skill) && !skills::corpse_targeted(skill));
    }
}

#[test]
fn greater_elemental_map_rejection_is_scoped_to_summon_feedback() {
    assert!(skills::summon_feedback(162) && skills::summon_feedback(51));
    for skill in (156..=166u16).filter(|skill| *skill != 162) {
        assert!(!skills::summon_feedback(skill), "skill={skill}");
    }
    let mut state = snapshot();
    let now = Instant::now();
    let cast = pending(&state, now);
    assert_eq!(cast.observe_summon(&state.world, now), Feedback::Pending);
    // 353 may follow the cast action because MP and crystals are already committed.
    state.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(cast.observe_summon(&state.world, now), Feedback::CastAction);
    assert!(cast.hold_summon(Feedback::CastAction, now + Duration::from_millis(500)));
    state.world.accept(&[0x10, 0x61, 0x01, 0]).unwrap();
    assert_eq!(cast.observe_summon(&state.world, now), Feedback::Rejected(353));
    assert_ne!(Feedback::Rejected(353).label(), Feedback::Rejected(1).label());
    // Ordinary casts never read the summon-only rejection.
    assert_eq!(cast.observe(&state.world, now), Feedback::CastAction);
    // A later cast does not inherit the earlier sequenced 353.
    let next = pending(&state, now);
    assert_eq!(next.observe_summon(&state.world, now), Feedback::Pending);
    assert_eq!(next.observe_summon(&state.world, now + Duration::from_secs(2)), Feedback::NoResponse);
    // Missing crystals (299) and MP (278) reach every cast kind.
    for code in [299u16, 278] {
        let fresh = pending(&state, now);
        let [low, high] = code.to_le_bytes();
        state.world.accept(&[0x10, low, high, 0]).unwrap();
        assert_eq!(fresh.observe_summon(&state.world, now), Feedback::Rejected(code));
        assert_eq!(fresh.observe(&state.world, now), Feedback::Rejected(code));
    }
    state.world.generation += 1;
    assert_eq!(cast.observe_summon(&state.world, now), Feedback::Interrupted);
}
