//! T022 batch 2 (#152): skills 110–118, request shapes per PROTOCOL-CONTRACT §6.3.
use lito_modern::modern_lito::{
    native_login::{self, LoginController, LoginPhase, Snapshot},
    native_skill_selection::{self, Model, Target},
    native_skills::{self as skills, Feedback, Pending},
};
use std::time::{Duration, Instant};

const PEER: u32 = 0xfedcba98;
const SELECTORS: [(u16, u8, u8); 6] =
    [(110, 13, 5), (111, 13, 6), (112, 13, 7), (114, 14, 1), (115, 14, 2), (117, 14, 4)];

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
    // Real book decoder: row 13 bits 5..7 (110–112), row 14 bits 0..5 (113–118).
    let mut book = vec![0x1e, 15];
    book.extend([0; 13]);
    book.extend([0xe0, 0x3f]);
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

fn true_target(id: u32, x: u16, y: u16, text: &[u8]) -> Vec<u8> {
    let mut b = vec![0x14, 14, 0];
    b.extend(id.to_le_bytes());
    b.extend(x.to_le_bytes());
    b.extend(y.to_le_bytes());
    b.extend(text);
    b.push(0);
    b
}

#[test]
fn selectors_clear_stale_hover_and_emit_exact_three_bytes_without_spending() {
    let mut state = snapshot();
    assert_eq!(
        state.world.learned_skills.iter().copied().collect::<Vec<_>>(),
        (110..=118).collect::<Vec<_>>()
    );
    let target = Target::capture(&state.world, PEER);
    state.world.actors.remove(&PEER);
    let before = serde_json::to_value(&state).unwrap();
    for (skill, row, column) in SELECTORS {
        assert!(skills::selector_only(skill));
        assert!(!skills::text_skill(skill));
        let choice = native_skill_selection::choice(&state.world, skill, target).unwrap();
        assert_eq!(choice.target, None);
        assert_eq!(skills::request(&state.world, skill, None).unwrap(), [0x14, row, column]);
        assert!(skills::request(&state.world, skill, Some(1)).is_err());
        assert!(skills::text_request(&state.world, skill, None, "x").is_err());
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn true_target_sends_unsigned_id_current_coordinates_and_optional_ms950_text() {
    let state = snapshot();
    let before = serde_json::to_value(&state).unwrap();
    assert!(skills::text_skill(113) && !skills::selector_only(113));
    let target = Target::capture(&state.world, PEER);
    let choice = native_skill_selection::choice(&state.world, 113, target).unwrap();
    assert_eq!(choice.target, Some(PEER));
    // Without typed text the annotation is a lone NUL (parser allows 0 bytes).
    assert_eq!(skills::request(&state.world, 113, Some(PEER)).unwrap(), true_target(PEER, 32769, 32768, b""));
    assert_eq!(
        skills::text_request(&state.world, 113, Some(PEER), "Boss").unwrap(),
        true_target(PEER, 32769, 32768, b"Boss")
    );
    let chinese = "集火目標";
    let encoded = native_login::encode_text(chinese, 0x34).unwrap();
    assert_eq!(encoded.len(), 8);
    assert_eq!(
        skills::text_request(&state.world, 113, Some(PEER), chinese).unwrap(),
        true_target(PEER, 32769, 32768, &encoded)
    );
    // Self is an eligible server candidate; no target means self.
    assert_eq!(skills::request(&state.world, 113, None).unwrap(), true_target(1, 32768, 32768, b""));
    assert_eq!(skills::request(&state.world, 113, Some(1)).unwrap(), true_target(1, 32768, 32768, b""));
    // Formal parser limit: ≤0x34 MS950 bytes, exact encoding, no control characters.
    assert!(skills::text_request(&state.world, 113, Some(PEER), &"a".repeat(52)).is_ok());
    assert!(skills::text_request(&state.world, 113, Some(PEER), &"a".repeat(53)).is_err());
    assert!(skills::text_request(&state.world, 113, Some(PEER), &"目".repeat(26)).is_ok());
    assert!(skills::text_request(&state.world, 113, Some(PEER), &"目".repeat(27)).is_err());
    for bad in ["a\tb", "a\nb", "\u{1}", "\u{1F600}"] {
        assert!(skills::text_request(&state.world, 113, Some(PEER), bad).is_err(), "{bad:?}");
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn true_target_rejects_offscreen_removed_hidden_dead_and_replaced_targets() {
    let mut state = snapshot();
    for (dx, dy, accepted) in [(18, 0, true), (-9, 9, true), (20, 0, false), (0, -20, false)] {
        let actor = state.world.actors.get_mut(&PEER).unwrap();
        actor.x = (32768 + dx) as u16;
        actor.y = (32768 + dy) as u16;
        assert_eq!(skills::request(&state.world, 113, Some(PEER)).is_ok(), accepted, "dx={dx} dy={dy}");
    }
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
        assert!(native_skill_selection::choice(&state.world, 113, target).is_err(), "change={change}");
    }
}

#[test]
fn clan_spells_send_one_trimmed_ms950_name_and_never_a_target() {
    let state = snapshot();
    let before = serde_json::to_value(&state).unwrap();
    for (skill, column) in [(116u16, 3u8), (118, 5)] {
        assert!(skills::text_skill(skill) && !skills::selector_only(skill));
        let mut expected = vec![0x14, 14, column];
        expected.extend(b"Ally");
        expected.push(0);
        assert_eq!(skills::text_request(&state.world, skill, None, "Ally").unwrap(), expected);
        let name = "盟友甲";
        let mut expected = vec![0x14, 14, column];
        expected.extend(native_login::encode_text(name, 0x1b).unwrap().iter());
        expected.push(0);
        assert_eq!(skills::text_request(&state.world, skill, None, name).unwrap(), expected);
        // Menu admission does not need a name; the packet does.
        let choice = native_skill_selection::choice(&state.world, skill, Target::capture(&state.world, PEER)).unwrap();
        assert_eq!(choice.target, None);
        skills::admit_named(&state.world, skill).unwrap();
        assert!(skills::request(&state.world, skill, None).is_err());
        assert!(skills::text_request(&state.world, skill, Some(PEER), "Ally").is_err());
        assert!(skills::text_request(&state.world, skill, None, &"a".repeat(27)).is_ok());
        assert!(skills::text_request(&state.world, skill, None, &"a".repeat(28)).is_err());
        assert!(skills::text_request(&state.world, skill, None, &"盟".repeat(14)).is_err());
        for bad in ["", " Ally", "Ally ", "Al\tly", "\u{1F600}"] {
            assert!(skills::text_request(&state.world, skill, None, bad).is_err(), "{bad:?}");
        }
    }
    assert!(skills::admit_named(&state.world, 113).is_err());
    assert!(skills::text_request(&state.world, 112, None, "Ally").is_err());
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn every_skill_rechecks_learning_life_mana_and_control_without_prediction() {
    for skill in 110..=118u16 {
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
            let result = match skill {
                113 => skills::text_request(&state.world, skill, Some(PEER), "Boss"),
                116 | 118 => skills::text_request(&state.world, skill, None, "Ally"),
                _ => skills::request(&state.world, skill, None),
            };
            assert!(result.is_err(), "skill={skill} change={change}");
            if matches!(skill, 116 | 118) {
                assert!(skills::admit_named(&state.world, skill).is_err(), "skill={skill} change={change}");
            }
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
    }
}

#[test]
fn menu_exposes_this_batch_with_formal_reuse() {
    let state = snapshot();
    let mut menu = Model::default();
    menu.sync(7, &state.world, true);
    assert_eq!(menu.rows().len(), 9);
    assert!(menu.rows().iter().all(|row| !row.label.contains("尚未支援")), "{:?}", menu.rows());
    menu.select(Some(116)).unwrap();
    assert_eq!(menu.choice(7, &state.world, None).unwrap().target, None);
    assert_eq!(
        (110..=118).map(skills::reuse_millis).collect::<Vec<_>>(),
        [0, 0, 2000, 0, 0, 0, 0, 0, 0]
    );
    for skill in [101, 109, 119, 221, u16::MAX] {
        assert!(!skills::text_skill(skill));
    }
}

#[test]
fn clan_rejections_are_sequenced_and_scoped_to_clan_casts() {
    let mut state = snapshot();
    let now = Instant::now();
    let clan = pending(&state, now);
    // 73 carries the typed name; later ordinary messages must not hide it.
    state.world.accept(&[0x10, 73, 0, 1, b'A', b'l', b'l', b'y', 0]).unwrap();
    state.world.accept(&[0x10, 161, 0, 0]).unwrap();
    assert_eq!(clan.observe_clan(&state.world, now), Feedback::Rejected(73));
    assert_eq!(clan.observe_clan(&state.world, now + Duration::from_secs(2)), Feedback::Rejected(73));
    // Other casts never treat these codes as their own rejection.
    assert_eq!(clan.observe(&state.world, now), Feedback::Pending);
    assert_eq!(clan.observe_item(&state.world, now), Feedback::Pending);
    let next = pending(&state, now);
    assert_eq!(next.observe_clan(&state.world, now), Feedback::Pending);
    assert_eq!(next.observe_clan(&state.world, now + Duration::from_secs(2)), Feedback::NoResponse);
    for code in [414u16, 629, 647, 1192] {
        let fresh = pending(&state, now);
        let [low, high] = code.to_le_bytes();
        state.world.accept(&[0x10, low, high, 0]).unwrap();
        assert_eq!(fresh.observe_clan(&state.world, now), Feedback::Rejected(code));
        assert_eq!(fresh.observe(&state.world, now), Feedback::Pending);
        assert_ne!(Feedback::Rejected(code).label(), Feedback::Rejected(1).label());
    }
    // The shared failure message 280 still reaches every cast kind.
    let fresh = pending(&state, now);
    state.world.accept(&[0x10, 0x18, 1, 0]).unwrap();
    assert_eq!(fresh.observe_clan(&state.world, now), Feedback::Rejected(280));
    assert_eq!(fresh.observe(&state.world, now), Feedback::Rejected(280));
    let cast = pending(&state, now);
    state.world.accept(&[0x20, 1, 0, 0, 0, 19]).unwrap();
    assert_eq!(cast.observe_clan(&state.world, now), Feedback::CastAction);
    state.world.generation += 1;
    assert_eq!(cast.observe_clan(&state.world, now), Feedback::Interrupted);
}
