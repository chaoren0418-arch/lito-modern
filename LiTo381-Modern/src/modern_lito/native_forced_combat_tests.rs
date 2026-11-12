use super::*;
use crate::modern_lito::{
    native_combat, native_login::LoginController, native_room::Room, native_wire,
};

pub(crate) const CTRL: Input = Input {
    foreground: true,
    ctrl: true,
};

pub(crate) fn snapshot() -> Snapshot {
    let mut state = LoginController::default().snapshot().clone();
    state.phase = LoginPhase::BattlePending;
    state.world = crate::modern_lito::native_world_scene::movement_tests::world();
    state.joined_room = Some(Room {
        id: 1,
        name: "fixture".into(),
        attributes: 0,
        password_required: false,
        users_summary: 2,
        capacity: 2,
        mode: 1,
        map_id: 0,
        summary: [0; 5],
        owner: "fixture".into(),
        options: [0; 4],
        selections: vec![],
        members: vec![],
        countdown: None,
    });
    state
}

fn spell(skill: u16, target: u32, coordinates: bool) -> Vec<u8> {
    let mut body = vec![0x14, ((skill - 1) / 8) as u8, ((skill - 1) % 8) as u8];
    body.extend(target.to_le_bytes());
    if coordinates {
        body.extend([0x12, 0x80, 0x34, 0x80]);
    }
    body
}

#[test]
fn lfc1_attack_and_object_spell_golden_bytes_and_lengths() {
    let state = snapshot();
    for (body, total) in [
        (
            vec![0x17, 0x44, 0x33, 0x22, 0x11, 0x12, 0x80, 0x34, 0x80],
            19,
        ),
        (
            vec![0x18, 0x44, 0x33, 0x22, 0x11, 0x12, 0x80, 0x34, 0x80],
            19,
        ),
        (
            vec![0x7a, 0x44, 0x33, 0x22, 0x11, 0x12, 0x80, 0x34, 0x80],
            19,
        ),
        (vec![0x14, 3, 1, 0x44, 0x33, 0x22, 0x11], 17),
        (
            vec![0x14, 0, 3, 0x44, 0x33, 0x22, 0x11, 0x12, 0x80, 0x34, 0x80],
            21,
        ),
    ] {
        let mut extended = body.clone();
        append(&mut extended, CTRL, &state);
        assert_eq!(&extended[..body.len()], body);
        assert_eq!(
            &extended[body.len()..],
            &[0x4c, 0x46, 0x43, 0x31, body[0], 1, 0x67, 0xa5]
        );
        let frame = native_wire::frame(&extended).unwrap();
        assert_eq!(frame.len(), total);
        assert_eq!(&frame[..2], &[total as u8, 0]);
        // An already extended body is never extended twice.
        append(&mut extended, CTRL, &state);
        assert_eq!(extended.len() + 2, total);
    }
}

#[test]
fn lfc1_all_input_combinations_require_foreground_and_ctrl() {
    let state = snapshot();
    for foreground in [false, true] {
        for ctrl in [false, true] {
            let original = native_combat::request(7, 1, 2).unwrap();
            let mut body = original.clone();
            append(&mut body, Input { foreground, ctrl }, &state);
            assert_eq!(
                body.len(),
                original.len() + if foreground && ctrl { 8 } else { 0 }
            );
            if !foreground || !ctrl {
                assert_eq!(body, original);
            }
        }
    }
}

#[test]
fn lfc1_only_active_combat_rooms_allow_extension() {
    let original = spell(4, 7, true);
    for mode in [0, 1, 2, 3, 4, 5, 6, 7, 255] {
        for phase in [
            LoginPhase::RoomReady,
            LoginPhase::BattlePending,
            LoginPhase::BattleResult,
            LoginPhase::WorldReady,
            LoginPhase::Disconnected,
        ] {
            let mut state = snapshot();
            state.phase = phase;
            state.joined_room.as_mut().unwrap().mode = mode;
            let mut body = original.clone();
            append(&mut body, CTRL, &state);
            assert_eq!(
                body.len() > original.len(),
                phase == LoginPhase::BattlePending && (1..=5).contains(&mode)
            );
        }
    }
    for case in 0..5 {
        let mut state = snapshot();
        match case {
            0 => state.joined_room = None,
            1 => state.observer = true,
            2 => state.world.observer_mode = true,
            3 => state.battle_withdrawal_pending = true,
            _ => state.world = Default::default(),
        }
        let mut body = original.clone();
        append(&mut body, CTRL, &state);
        assert_eq!(body, original);
    }
}

#[test]
fn lfc1_non_object_spell_layouts_never_receive_extension() {
    let state = snapshot();
    // Even nonzero coordinate/tick/item/selector bytes cannot become an actor ID.
    for skill in [5, 12, 21, 51, 58, 60, 63, 69, 116, 118] {
        for coordinates in [false, true] {
            let original = spell(skill, 0x80348012, coordinates);
            let mut body = original.clone();
            append(&mut body, CTRL, &state);
            assert_eq!(body, original, "skill {skill}");
        }
    }
}

#[test]
fn lfc1_zero_target_absent_target_invalid_skill_and_other_packets_unchanged() {
    let state = snapshot();
    let mut cases = vec![
        vec![],
        vec![0x14, 0, 1],
        vec![0x17],
        spell(4, 0, true),
        spell(26, 0, false),
        spell(221, 7, false),
        vec![0x14, 0, 8, 7, 0, 0, 0],
        vec![0x14, 255, 0, 7, 0, 0, 0],
    ];
    for opcode in [0x17, 0x18, 0x7a] {
        cases.push(vec![opcode, 0, 0, 0, 0, 1, 2, 3, 4]);
        for len in [0, 1, 5, 7, 8, 10, 11, 12, 1024] {
            let mut body = vec![1; len];
            if let Some(first) = body.first_mut() {
                *first = opcode;
            }
            cases.push(body);
        }
    }
    for opcode in [0x03, 0x0c, 0x5a, 0xe0, 0xd9, 0xff] {
        cases.push(vec![opcode; 9]);
    }
    for original in cases {
        let mut body = original.clone();
        append(&mut body, CTRL, &state);
        assert_eq!(body, original);
    }
}

#[test]
fn lfc1_cipher_counter_and_next_ordinary_frame_remain_aligned() {
    let state = snapshot();
    let original = spell(4, 0x11223344, true);
    let mut tx = native_wire::CipherState::new(0x12345678);
    let mut rx = tx.clone();
    let initial = tx.counter();
    let mut bytes = 0;
    for input in [
        CTRL,
        Input {
            ctrl: false,
            ..CTRL
        },
        Input {
            foreground: false,
            ..CTRL
        },
        CTRL,
    ] {
        let mut plain = original.clone();
        append(&mut plain, input, &state);
        bytes += plain.len() as u64;
        let mut cipher = plain.clone();
        tx.encrypt(&mut cipher);
        let wire = native_wire::frame(&cipher).unwrap();
        let mut decoder = native_wire::FrameDecoder::new(native_wire::MAX_FRAME).unwrap();
        let mut decoded = None;
        for byte in wire {
            let (_, body) = decoder.feed(&[byte]).unwrap();
            if body.is_some() {
                decoded = body;
            }
        }
        let mut decoded = decoded.unwrap();
        rx.decrypt(&mut decoded);
        assert_eq!(decoded, plain);
        assert_eq!(tx.counter(), initial + bytes);
        assert_eq!(rx.counter(), tx.counter());
        if !input.ctrl || !input.foreground {
            assert_eq!(decoded, original);
        }
    }
}

#[test]
fn t021_stone_object_never_receives_actor_force_trailer() {
    let state = snapshot();
    let original = spell(100, 0xfedcba98, false);
    let mut body = original.clone();
    append(&mut body, CTRL, &state);
    assert_eq!(body, original);
}
