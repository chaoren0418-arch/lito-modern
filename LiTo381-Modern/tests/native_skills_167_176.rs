//! T022 batch 6 (#156): skills 167–176, request shapes per PROTOCOL-CONTRACT §6.3.
use lito_modern::modern_lito::{
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{self, Model, Target},
    native_skills::{self as skills, Feedback, Pending},
};
use std::time::{Duration, Instant};

const PEER: u32 = 0xfedcba98;
const SELECTORS: [(u16, u8, u8); 5] =
    [(169, 21, 0), (171, 21, 2), (172, 21, 3), (175, 21, 6), (176, 21, 7)];
/// ⑩ d(target) h(x) h(y): (skill, row, column).
const ACTOR: [(u16, u8, u8); 3] = [(167, 20, 6), (173, 21, 4), (174, 21, 5)];

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
    // Real book decoder: row 20 bits 6..7 (167–168), row 21 bits 0..7 (169–176).
    let mut book = vec![0x1e, 22];
    book.extend([0; 20]);
    book.extend([0xc0, 0xff]);
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
        (167..=176).collect::<Vec<_>>()
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
    for skill in [167u16, 168, 170, 173, 174] {
        assert!(!skills::selector_only(skill), "skill={skill}");
    }
}

#[test]
fn iron_skin_always_sends_the_caster_id_and_never_a_hovered_actor() {
    let mut state = snapshot();
    let before = serde_json::to_value(&state).unwrap();
    let hover = Target::capture(&state.world, PEER);
    let expected = [0x14, 20, 7, 1, 0, 0, 0];
    assert!(skills::self_object(168) && !skills::selector_only(168));
    // Formal `useIronSkinSpell` answers 280 to every object ID except the caster's.
    let choice = native_skill_selection::choice(&state.world, 168, hover).unwrap();
    assert_eq!(choice.target, None);
    assert_eq!(skills::request(&state.world, 168, choice.target).unwrap(), expected);
    assert_eq!(skills::request(&state.world, 168, Some(1)).unwrap(), expected);
    assert!(skills::request(&state.world, 168, Some(PEER)).is_err());
    assert!(skills::request(&state.world, 168, Some(0)).is_err());
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
    // A stale hover does not block the self buff.
    state.world.actors.remove(&PEER);
    let choice = native_skill_selection::choice(&state.world, 168, hover).unwrap();
    assert_eq!(choice.target, None);
    assert_eq!(skills::request(&state.world, 168, None).unwrap(), expected);
    for skill in (167..=176u16).filter(|id| *id != 168) {
        assert!(!skills::self_object(skill), "skill={skill}");
    }
}

#[test]
fn water_life_sends_seven_bytes_for_self_or_a_living_actor_on_the_screen() {
    let mut state = snapshot();
    let hover = Target::capture(&state.world, PEER);
    let own = [0x14, 21, 1, 1, 0, 0, 0];
    let peer = [0x14, 21, 1, 0x98, 0xba, 0xdc, 0xfe];
    // No selection means the caster; an explicit selection is never replaced.
    assert_eq!(native_skill_selection::choice(&state.world, 170, None).unwrap().target, None);
    assert_eq!(skills::request(&state.world, 170, None).unwrap(), own);
    assert_eq!(skills::request(&state.world, 170, Some(1)).unwrap(), own);
    for (dx, dy, accepted) in [
        (1, 0, true),
        (18, 0, true),
        (-9, 9, true),
        // Distance 19 is inside the diamond only on its asymmetric edge.
        (-10, 9, true),
        (10, 9, false),
        (20, 0, false),
        (0, -20, false),
    ] {
        place(&mut state, dx, dy);
        let before = serde_json::to_value(&state).unwrap();
        let choice = native_skill_selection::choice(&state.world, 170, hover);
        assert_eq!(choice.as_ref().ok().map(|c| c.target), accepted.then_some(Some(PEER)), "dx={dx} dy={dy}");
        assert_eq!(
            skills::request(&state.world, 170, Some(PEER)).ok(),
            accepted.then(|| peer.to_vec()),
            "dx={dx} dy={dy}"
        );
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
    assert!(skills::request(&state.world, 170, Some(0)).is_err());
}

#[test]
fn shackle_pollute_and_gale_use_current_coordinates_and_their_own_ranges() {
    // 167: original click range 8; 173: formal range 3; 174: click range 10
    // and the native screen diamond.
    let cases: [(u16, &[(i32, i32, bool)]); 3] = [
        (167, &[(8, 0, true), (0, -8, true), (-8, 8, true), (9, 0, false), (0, -9, false), (8, 9, false)]),
        (173, &[(3, 0, true), (0, -3, true), (-3, 3, true), (4, 0, false), (0, -4, false), (3, 4, false)]),
        (174, &[(10, 0, true), (0, -10, true), (-10, 8, true), (-10, 9, true), (10, 9, false),
            (10, 10, false), (11, 0, false), (0, -11, false)]),
    ];
    for (skill, offsets) in cases {
        let (_, row, column) = ACTOR.into_iter().find(|entry| entry.0 == skill).unwrap();
        let mut state = snapshot();
        let target = Target::capture(&state.world, PEER);
        for &(dx, dy, accepted) in offsets {
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
        // Never a silent self fallback.
        assert!(skills::request(&state.world, skill, None).is_err());
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
        assert!(native_skill_selection::choice(&state.world, skill, None).is_err());
    }
}

#[test]
fn actor_spells_reject_removed_hidden_dead_and_replaced_targets() {
    for skill in [167u16, 170, 173, 174] {
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
            // 170 included: a stale explicit selection never falls back to the caster.
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
    for skill in 167..=176u16 {
        let target = matches!(skill, 167 | 173 | 174).then_some(PEER);
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
        assert_eq!(usize::from(row.id), 167 + index);
        assert!(row.label.starts_with(&format!("#{} ", row.id)), "{}", row.label);
    }
    let hover = Target::capture(&state.world, PEER);
    menu.select(Some(168)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, None);
    menu.select(Some(170)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, Some(PEER));
    assert_eq!(menu.choice(7, &state.world, None).unwrap().target, None);
    menu.select(Some(173)).unwrap();
    assert_eq!(menu.choice(7, &state.world, hover).unwrap().target, Some(PEER));
    assert!(menu.choice(7, &state.world, None).is_err());
    // E149 catalog reuseDelayMillis.
    assert_eq!(
        (167..=176).map(skills::reuse_millis).collect::<Vec<_>>(),
        [100, 0, 100, 100, 100, 0, 1000, 3000, 300, 100]
    );
    // Recovered PE word_635774[skill - 1].
    assert_eq!(
        (167..=176).map(skills::client_clock_millis).collect::<Vec<_>>(),
        [600, 0, 100, 400, 300, 0, 1000, 3000, 5000, 100]
    );
}

#[test]
fn feedback_reads_coded_rejections_and_never_invents_a_storm_walk_reply() {
    let mut state = snapshot();
    let now = Instant::now();
    // 167: the formal handler publishes the cast action, then S10 280 when no
    // target was shackled. Both are read by the ordinary observer.
    let shackle = pending(&state, now);
    assert_eq!(shackle.observe(&state.world, now), Feedback::Pending);
    state.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(shackle.observe(&state.world, now), Feedback::CastAction);
    state.world.accept(&[0x10, 0x18, 1, 0]).unwrap();
    assert_eq!(shackle.observe(&state.world, now), Feedback::Rejected(280));
    // An older 280 is not attributed to the next cast; 299 is the missing reagent.
    let next = pending(&state, now);
    assert_eq!(next.observe(&state.world, now), Feedback::Pending);
    state.world.accept(&[0x10, 0x2b, 1, 0]).unwrap();
    assert_eq!(next.observe(&state.world, now), Feedback::Rejected(299));
    assert_ne!(Feedback::Rejected(299).label(), Feedback::Rejected(280).label());

    // 172: formal row sends S_MANA_POINT only (no action, effect or message).
    // The client does not treat an MP change as a cast acknowledgement.
    let storm = pending(&state, now);
    assert_eq!(storm.observe(&state.world, now), Feedback::Pending);
    state.world.mp = Some([190, 200]);
    assert_eq!(storm.observe(&state.world, now + Duration::from_millis(1999)), Feedback::Pending);
    assert_eq!(storm.observe(&state.world, now + Duration::from_secs(2)), Feedback::NoResponse);

    state.world.generation += 1;
    assert_eq!(shackle.observe(&state.world, now), Feedback::Interrupted);
    assert_eq!(storm.observe(&state.world, now), Feedback::Interrupted);
}
