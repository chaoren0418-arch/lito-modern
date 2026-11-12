//! T022 batch 9 (#159): skills 201–205 and 207–210, request shapes per
//! PROTOCOL-CONTRACT §6.3. 206 Concentration was wired earlier (#101).
use lito_modern::modern_lito::{
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{self, Model, Target},
    native_skills::{self as skills, Feedback, Pending},
};
use std::time::{Duration, Instant};

const PEER: u32 = 0xfedcba98;
const SELECTORS: [(u16, u8, u8); 3] = [(201, 25, 0), (205, 25, 4), (210, 26, 1)];
/// ⑩ attacks: (skill, row, column, E149 maximumRange).
const ATTACKS: [(u16, u8, u8, i32); 4] = [(202, 25, 1, 3), (203, 25, 2, 1), (207, 25, 6, 3), (208, 25, 7, 3)];
/// ⑩ PC buffs that also accept the caster: (skill, row, column).
const BUFFS: [(u16, u8, u8); 2] = [(204, 25, 3), (209, 26, 0)];

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
    // Real book decoder: row 25 bits 0..7 (201–208), row 26 bits 0..1 (209–210).
    let mut book = vec![0x1e, 27];
    book.extend([0; 25]);
    book.extend([0xff, 0x03]);
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

fn actor_body(row: u8, column: u8, id: u32, position: [u16; 2]) -> Vec<u8> {
    let mut b = vec![0x14, row, column];
    b.extend(id.to_le_bytes());
    b.extend(position[0].to_le_bytes());
    b.extend(position[1].to_le_bytes());
    b
}

#[test]
fn selectors_clear_stale_hover_and_emit_exact_three_bytes_without_spending() {
    let mut state = snapshot();
    assert_eq!(
        state.world.learned_skills.iter().copied().collect::<Vec<_>>(),
        (201..=210).collect::<Vec<_>>()
    );
    let target = Target::capture(&state.world, PEER);
    state.world.actors.remove(&PEER);
    let before = serde_json::to_value(&state).unwrap();
    for (skill, row, column) in SELECTORS {
        assert!(skills::selector_only(skill) && !skills::illusion_targeted(skill));
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
fn wrapper_skills_always_send_eleven_bytes_and_are_never_selector_only() {
    // Formal `IllusionSkillRequests.dispatch` drops every length except 11.
    let state = snapshot();
    // 211..=220 (#160) share `illusion_targeted`; this file only owns 201..=210.
    let wrapped: Vec<u16> = (201..=210).filter(|id| skills::illusion_targeted(*id)).collect();
    assert_eq!(wrapped, [202, 203, 204, 207, 208, 209]);
    for skill in wrapped {
        assert!(!skills::selector_only(skill) && !skills::self_object(skill), "skill={skill}");
        assert!(!skills::self_bound(skill) && !skills::text_skill(skill), "skill={skill}");
        assert!(!skills::item_targeted(skill) && !skills::corpse_targeted(skill), "skill={skill}");
        let body = skills::request(&state.world, skill, Some(PEER)).unwrap();
        assert_eq!(body.len(), 11, "skill={skill}");
        assert_eq!(&body[3..7], &PEER.to_le_bytes(), "skill={skill}");
    }
}

#[test]
fn illusion_buffs_send_the_caster_or_a_living_actor_within_three_tiles() {
    for (skill, row, column) in BUFFS {
        let mut state = snapshot();
        let hover = Target::capture(&state.world, PEER);
        // No selection means the caster with the caster's own coordinates.
        let own = actor_body(row, column, 1, [32768, 32768]);
        assert_eq!(native_skill_selection::choice(&state.world, skill, None).unwrap().target, None);
        assert_eq!(skills::request(&state.world, skill, None).unwrap(), own);
        assert_eq!(skills::request(&state.world, skill, Some(1)).unwrap(), own);
        for (dx, dy, accepted) in [
            (1, 0, true),
            (3, 0, true),
            (0, -3, true),
            (-3, 3, true),
            (4, 0, false),
            (0, -4, false),
            (3, 4, false),
        ] {
            let position = place(&mut state, dx, dy);
            let before = serde_json::to_value(&state).unwrap();
            // An explicit selection out of range is refused, never replaced by the caster.
            let choice = native_skill_selection::choice(&state.world, skill, hover);
            assert_eq!(
                choice.as_ref().ok().map(|c| c.target),
                accepted.then_some(Some(PEER)),
                "skill={skill} dx={dx} dy={dy}"
            );
            assert_eq!(
                skills::request(&state.world, skill, Some(PEER)).ok(),
                accepted.then(|| actor_body(row, column, PEER, position)),
                "skill={skill} dx={dx} dy={dy}"
            );
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
        assert!(skills::request(&state.world, skill, Some(0)).is_err());
    }
}

#[test]
fn illusion_attacks_use_current_coordinates_and_their_catalog_ranges() {
    for (skill, row, column, range) in ATTACKS {
        let mut state = snapshot();
        let target = Target::capture(&state.world, PEER);
        for (dx, dy, accepted) in [
            (range, 0, true),
            (0, -range, true),
            (-range, range, true),
            (range + 1, 0, false),
            (0, -(range + 1), false),
            (range, range + 1, false),
        ] {
            let position = place(&mut state, dx, dy);
            let before = serde_json::to_value(&state).unwrap();
            let choice = native_skill_selection::choice(&state.world, skill, target);
            assert_eq!(choice.is_ok(), accepted, "skill={skill} dx={dx} dy={dy}");
            assert_eq!(
                skills::request(&state.world, skill, Some(PEER)).ok(),
                accepted.then(|| actor_body(row, column, PEER, position)),
                "skill={skill} dx={dx} dy={dy}"
            );
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
        // Formal `useAttackSpell` drops a self target; never a silent fallback.
        assert!(skills::request(&state.world, skill, None).is_err());
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
        assert!(native_skill_selection::choice(&state.world, skill, None).is_err());
    }
}

#[test]
fn actor_spells_reject_removed_hidden_dead_and_replaced_targets() {
    for skill in [202u16, 203, 204, 207, 208, 209] {
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
            // 204／209 included: a stale explicit selection never falls back to the caster.
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
    for skill in 201..=210u16 {
        let target = ATTACKS.iter().any(|entry| entry.0 == skill).then_some(PEER);
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
fn menu_exposes_this_batch_with_formal_reuse_and_original_cast_clocks() {
    let state = snapshot();
    let mut menu = Model::default();
    menu.sync(7, &state.world, true);
    assert_eq!(menu.rows().len(), 10);
    assert!(menu.rows().iter().all(|row| !row.label.contains("尚未支援")), "{:?}", menu.rows());
    for (index, row) in menu.rows().iter().enumerate() {
        assert_eq!(usize::from(row.id), 201 + index);
        assert!(row.label.starts_with(&format!("#{} ", row.id)), "{}", row.label);
    }
    let hover = Target::capture(&state.world, PEER);
    menu.select(Some(205)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, None);
    menu.select(Some(204)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, Some(PEER));
    assert_eq!(menu.choice(7, &state.world, None).unwrap().target, None);
    menu.select(Some(202)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, Some(PEER));
    assert!(menu.choice(7, &state.world, None).is_err());
    // E149 catalog reuseDelayMillis (206 unchanged from #101).
    assert_eq!(
        (201..=210).map(skills::reuse_millis).collect::<Vec<_>>(),
        [10, 3000, 600, 2000, 5000, 1000, 3000, 2000, 2000, 5000]
    );
    // Recovered PE word_635774[skill - 1], equal to E149 clientCastClockMillis.
    assert_eq!(
        (201..=210).map(skills::client_clock_millis).collect::<Vec<_>>(),
        [1800, 0, 0, 0, 0, 1200, 0, 0, 64, 0]
    );
}

#[test]
fn cube_placement_refusal_is_a_coded_rejection_and_silence_is_never_success() {
    let mut state = snapshot();
    let now = Instant::now();
    // 205／210: the formal handlers answer S10 1412 when the cube cannot be placed.
    let cube = pending(&state, now);
    assert_eq!(cube.observe(&state.world, now), Feedback::Pending);
    state.world.accept(&[0x10, 0x84, 0x05, 0]).unwrap();
    assert_eq!(cube.observe(&state.world, now), Feedback::Rejected(1412));
    assert_ne!(Feedback::Rejected(1412).label(), Feedback::Rejected(280).label());
    assert_ne!(Feedback::Rejected(1412).label(), Feedback::Rejected(1).label());
    // An older 1412 is not attributed to the next cast; 278 is the MP refusal.
    let next = pending(&state, now);
    assert_eq!(next.observe(&state.world, now), Feedback::Pending);
    state.world.accept(&[0x10, 0x16, 1, 0]).unwrap();
    assert_eq!(next.observe(&state.world, now), Feedback::Rejected(278));

    // A missing cube reagent, an ineligible 204／209 target and an out-of-range
    // attack get no reply from the formal handlers: the client reports the
    // timeout and never treats an MP change as an acknowledgement.
    let silent = pending(&state, now);
    state.world.mp = Some([190, 200]);
    assert_eq!(silent.observe(&state.world, now + Duration::from_millis(1999)), Feedback::Pending);
    assert_eq!(silent.observe(&state.world, now + Duration::from_secs(2)), Feedback::NoResponse);
    // The committed cast publishes the caster's action 19.
    state.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(silent.observe(&state.world, now), Feedback::CastAction);

    state.world.generation += 1;
    assert_eq!(cube.observe(&state.world, now), Feedback::Interrupted);
    assert_eq!(silent.observe(&state.world, now), Feedback::Interrupted);
}
