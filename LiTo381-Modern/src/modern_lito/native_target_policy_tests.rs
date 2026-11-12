//! Offline fixtures and an owned loopback. No server material transaction is simulated.
use super::*;
use crate::modern_lito::{
    native_combat::HeldAttack,
    native_forced_combat::tests::{snapshot, CTRL},
    native_inventory::Item,
    native_room::{self, Member},
    native_skill_selection::{self, Target},
    native_skills::{self, Feedback},
    native_target_policy,
    native_world_scene::movement_tests::frame,
};

fn member(name: &str, force: u8) -> Member {
    let mut body = vec![0xcd, 1, 0, 0, 0];
    body.extend_from_slice(name.as_bytes());
    body.extend([0, force, 0, 0]);
    body.extend([0; 12]);
    body.extend([0, 0, 0]);
    body.extend(b"Ready\0");
    body.extend([0; 4]);
    body.extend([0, 0]);
    body.extend([0; 4]);
    body.push(0);
    body.extend([0; 21]);
    let native_room::Packet::Member { member, .. } = native_room::parse(&body).unwrap() else {
        panic!("room member fixture");
    };
    member
}

fn active() -> (LoginController, u32, u32) {
    let mut c = LoginController::default();
    c.state = snapshot();
    let own = c.state.world.own_actor().unwrap().id;
    let caster = c.state.world.actors.get_mut(&own).unwrap();
    caster.name = "Caster".into();
    caster.clan = "Clan".into();
    // These fixtures exercise combat only, with no emblem download owner.
    caster.emblem = 0;
    caster.master.clear();
    caster.companion = 0;
    let mut target = caster.clone();
    target.id += 1;
    target.x += 1;
    target.name = "Target".into();
    let other = target.id;
    c.state.world.actors.insert(other, target);
    c.state.world.learned_skills.extend([1, 4, 26, 42, 68]);
    c.state.world.mp = Some([100, 100]);
    c.state.joined_room.as_mut().unwrap().members = vec![member("Caster", 0), member("Target", 1)];
    c.state.inventory.items.insert(
        900,
        Item {
            id: 900,
            use_type: 0,
            charge: 0,
            icon: 0,
            blessing: 0,
            count: 10,
            identified: 1,
            name: "fixture material".into(),
            detail: vec![],
        },
    );
    (c, own, other)
}

#[test]
fn target_policy_self_friend_enemy_modes_and_request_scoped_ctrl() {
    let (mut c, own, other) = active();
    for mode in 0..=6 {
        c.state.joined_room.as_mut().unwrap().mode = mode;
        for force in [0, 1] {
            c.state.joined_room.as_mut().unwrap().members[1].force = force;
            for foreground in [false, true] {
                for ctrl in [false, true] {
                    let input = CombatInput { foreground, ctrl };
                    for skill in [1, 26, 42, 68] {
                        let body =
                            native_skills::request(&c.state.world, skill, Some(other)).unwrap();
                        assert_eq!(
                            native_target_policy::validate(&c.state, &body, input).is_ok(),
                            !matches!(mode, 1 | 3 | 5) || force == 0 || (foreground && ctrl),
                            "mode={mode} force={force} skill={skill} input={input:?}"
                        );
                        let self_body =
                            native_skills::request(&c.state.world, skill, Some(own)).unwrap();
                        assert!(native_target_policy::validate(&c.state, &self_body, input).is_ok());
                    }
                    let attack = c.state.world.attack_request(other).unwrap();
                    assert_eq!(
                        native_target_policy::validate(&c.state, &attack, input).is_ok(),
                        !matches!(mode, 1 | 3 | 5) || force == 0 || (foreground && ctrl)
                    );
                }
            }
        }
    }
}

#[test]
fn target_policy_uses_current_roster_not_clan_graphics_or_source() {
    let (mut c, _, other) = active();
    let body = native_skills::request(&c.state.world, 26, Some(other)).unwrap();
    // Same clan and same graphics still form opposing teams.
    assert!(native_target_policy::validate(&c.state, &body, CombatInput::default()).is_err());
    c.state.joined_room.as_mut().unwrap().members[1].force = 0;
    assert!(native_target_policy::validate(&c.state, &body, CombatInput::default()).is_ok());
    c.state.joined_room.as_mut().unwrap().members[1].force = 1;
    assert!(native_target_policy::validate(&c.state, &body, CombatInput::default()).is_err());
    // Unlisted actors and named companions keep the existing server type gates.
    c.state.world.actors.get_mut(&other).unwrap().master = "Caster".into();
    assert!(native_target_policy::validate(&c.state, &body, CombatInput::default()).is_ok());
    c.state.world.actors.get_mut(&other).unwrap().master.clear();
    c.state.world.actors.get_mut(&other).unwrap().name = "NPC".into();
    assert!(native_target_policy::validate(&c.state, &body, CombatInput::default()).is_ok());
    c.state.world.actors.get_mut(&other).unwrap().name = "Target".into();
    let mut duplicate = c.state.world.actors[&other].clone();
    duplicate.id += 1;
    c.state.world.actors.insert(duplicate.id, duplicate);
    assert!(native_target_policy::validate(&c.state, &body, CTRL).is_err());
}

#[test]
fn target_policy_self_26_42_68_exact_seven_bytes_without_ctrl() {
    let (mut c, own, _) = active();
    c.state.world.actors.get_mut(&own).unwrap().clan.clear();
    for (skill, row, column) in [(26, 3, 1), (42, 5, 1), (68, 8, 3)] {
        let mut expected = vec![0x14, row, column];
        expected.extend(own.to_le_bytes());
        for selected in [None, Some(Target::capture(&c.state.world, own).unwrap())] {
            let choice = native_skill_selection::choice(&c.state.world, skill, selected).unwrap();
            let mut body = native_skills::request(&c.state.world, skill, choice.target).unwrap();
            native_target_policy::validate(&c.state, &body, CombatInput::default()).unwrap();
            native_forced_combat::append(&mut body, CombatInput::default(), &c.state);
            assert_eq!(body, expected);
            assert_eq!(body.len(), 7);
        }
    }
    assert_eq!(native_skills::reuse_millis(68), 1500);
}

#[test]
fn target_policy_ctrl_never_bypasses_client_gates_or_spends_on_failed_send() {
    for case in 0..13 {
        for input in [CombatInput::default(), CTRL] {
            let (mut c, own, other) = active();
            c.state.joined_room.as_mut().unwrap().members[1].force = 0;
            match case {
                0 => c.state.observer = true,
                1 => c.state.world.observer_mode = true,
                2 => c.state.world.hp = Some([0, 100]),
                3 => c.state.world.actors.get_mut(&own).unwrap().dead = true,
                4 => {
                    c.state.world.actors.remove(&other);
                }
                5 => c.state.world.actors.get_mut(&other).unwrap().dead = true,
                6 => c.state.world.actors.get_mut(&other).unwrap().x += 100,
                7 => {
                    c.state
                        .world
                        .actors
                        .get_mut(&other)
                        .unwrap()
                        .draw_state
                        .invisible = true
                }
                8 => c.state.world.actors.get_mut(&other).unwrap().clan = "Other".into(),
                9 => c.state.world.actors.get_mut(&own).unwrap().clan.clear(),
                10 => c.state.battle_withdrawal_pending = true,
                11 => {
                    c.state.world.movement_blocks.insert(1);
                }
                _ => {} // Valid target with no transport owner: queueing must fail atomically.
            }
            let f = frame(&c.state.world, true);
            let before = serde_json::to_value(&c.state).unwrap();
            for skill in [26, 42, 68] {
                assert!(
                    c.cast_with_input(skill, Some(other), &f, input).is_err(),
                    "case={case}"
                );
                assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
                assert!(c.skill_pending.is_none() && c.skill_target_pending.is_none());
                assert!(c.skill_ready.is_none() && c.transport.queued_tx_counter().is_none());
            }
        }
    }
}

#[test]
fn target_policy_68_retained_selection_cannot_bind_replacement_or_fall_back_to_self() {
    for case in 0..6 {
        let (mut c, _, other) = active();
        let target = Target::capture(&c.state.world, other).unwrap();
        match case {
            0 => {
                c.state.world.actors.remove(&other);
            }
            1 => c.state.world.actors.get_mut(&other).unwrap().incarnation += 1,
            2 => c.state.world.actors.get_mut(&other).unwrap().dead = true,
            3 => {
                c.state
                    .world
                    .actors
                    .get_mut(&other)
                    .unwrap()
                    .target_revision += 1
            }
            4 => c.state.world.generation += 1,
            _ => {
                c.state
                    .world
                    .actors
                    .get_mut(&other)
                    .unwrap()
                    .draw_state
                    .invisible = true
            }
        }
        let f = frame(&c.state.world, true);
        let before = serde_json::to_value(&c.state).unwrap();
        let error = c
            .cast_selected_with_input(68, Some(target), &f, CTRL)
            .unwrap_err();
        assert!(error.to_string().contains("選取的目標已失效"));
        assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
        assert!(c.skill_pending.is_none() && c.skill_ready.is_none());
    }
}

#[test]
fn target_policy_68_pending_retires_before_late_cast_action_without_material_mutation() {
    for case in 0..5 {
        let (mut c, own, other) = active();
        let target = Target::capture(&c.state.world, other).unwrap();
        c.skill_target_pending = Some(target);
        c.skill_pending = Some(native_skills::Pending {
            since: Instant::now(),
            casts: c.state.world.own_cast_packets,
            messages: c.state.world.message_count,
            generation: c.state.world.generation,
            incarnation: c.state.world.actors[&own].incarnation,
        });
        match case {
            0 => {
                c.state.world.actors.remove(&other);
            }
            1 => c.state.world.actors.get_mut(&other).unwrap().incarnation += 1,
            2 => c.state.world.actors.get_mut(&other).unwrap().dead = true,
            3 => c.state.world.generation += 1,
            _ => c.state.world.actors.get_mut(&own).unwrap().incarnation += 1,
        }
        c.state.world.own_cast_packets += 1;
        let inventory = c.state.inventory.clone();
        let mp = c.state.world.mp;
        c.poll();
        assert_eq!(c.state.phase, LoginPhase::BattlePending);
        assert_eq!(c.state.skill_feedback, Feedback::Interrupted);
        assert!(c.skill_pending.is_none() && c.skill_target_pending.is_none());
        assert_eq!(c.state.inventory, inventory);
        assert_eq!(c.state.world.mp, mp);
    }
}

#[test]
fn target_policy_68_deferred_body_is_dropped_when_receiver_retires_before_frame_zero() {
    // case 0..=4 retire the receiver; case 5 keeps it current (body retained).
    for self_target in [false, true] {
        for case in 0..6 {
            let (mut c, own, other) = active();
            let receiver = if self_target { own } else { other };
            let body = native_skills::request(&c.state.world, 68, Some(receiver)).unwrap();
            c.skill_target_pending = Some(Target::capture(&c.state.world, receiver).unwrap());
            c.skill_pending = Some(native_skills::Pending {
                since: Instant::now(),
                casts: c.state.world.own_cast_packets,
                messages: c.state.world.message_count,
                generation: c.state.world.generation,
                incarnation: c.state.world.actors[&own].incarnation,
            });
            c.player_callbacks.defer_for_test(Zeroizing::new(body));
            let actor = c.state.world.actors.get_mut(&receiver).unwrap();
            match case {
                0 => actor.target_revision += 1,
                1 => actor.incarnation += 1,
                2 => actor.dead = true,
                // The own actor is always drawable; use a replacement instead.
                3 if self_target => actor.incarnation += 2,
                3 => actor.draw_state.invisible = true,
                4 => c.state.world.generation += 1,
                _ => {}
            }
            let inventory = c.state.inventory.clone();
            let mp = c.state.world.mp;
            let stale = c.discard_stale_deferred_target();
            assert_eq!(stale, case != 5, "self={self_target} case={case}");
            assert_eq!(c.player_callbacks.pending(), case == 5);
            assert!(c.transport.queued_tx_counter().is_none());
            if case != 5 {
                c.poll();
                assert_eq!(c.state.skill_feedback, Feedback::Interrupted);
                assert!(c.skill_pending.is_none() && c.skill_target_pending.is_none());
                assert!(!c.player_callbacks.pending());
            }
            assert_eq!(c.state.inventory, inventory);
            assert_eq!(c.state.world.mp, mp);
        }
    }
}

#[test]
fn target_policy_68_wire_remove_reput_and_resurrection_retire_selection() {
    for resurrect in [false, true] {
        let (mut c, own, other) = active();
        let selected = Target::capture(&c.state.world, other).unwrap();
        if resurrect {
            let mut death = vec![0x20];
            death.extend(other.to_le_bytes());
            death.push(8);
            c.state.world.accept(&death).unwrap();
            let mut revive = vec![0x11];
            revive.extend(other.to_le_bytes());
            revive.push(0);
            revive.extend(own.to_le_bytes());
            revive.extend(0u16.to_le_bytes());
            c.state.world.accept(&revive).unwrap();
        } else {
            let mut remove = vec![0x15];
            remove.extend(other.to_le_bytes());
            c.state.world.accept(&remove).unwrap();
            assert!(selected.resolve(&c.state.world).is_none());
            let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
                "../../tests/fixtures/native-world-current.json"
            ))
            .unwrap();
            let hex = fixture["packets"]["actor"].as_str().unwrap();
            let mut put: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            put[5..9].copy_from_slice(&other.to_le_bytes());
            c.state.world.accept(&put).unwrap();
        }
        assert!(!c.state.world.actors[&other].dead);
        assert!(selected.resolve(&c.state.world).is_none());
        let f = frame(&c.state.world, true);
        let before = serde_json::to_value(&c.state).unwrap();
        assert!(c
            .cast_selected_with_input(68, Some(selected), &f, CTRL)
            .is_err());
        assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    }
}

#[test]
fn target_policy_controller_loopback_checks_bytes_and_revalidates_each_dispatch() {
    use crate::modern_lito::{
        native_version,
        native_wire::{self, CipherState},
    };
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    let (mut c, own, other) = active();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let (received_tx, received_rx) = std::sync::mpsc::channel();
    let (finish_tx, finish_rx) = std::sync::mpsc::channel();
    let peer = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "owned accept timeout");
                    thread::sleep(Duration::from_millis(2));
                }
                Err(e) => panic!("owned accept: {e}"),
            }
        };
        socket.set_nonblocking(false).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let seed = 0x193193u32;
        let mut key = vec![1];
        key.extend(seed.to_le_bytes());
        socket
            .write_all(&native_wire::frame(&key).unwrap())
            .unwrap();
        let mut rx = CipherState::new(seed);
        let mut tx = CipherState::new(seed);
        let mut read = || {
            let mut header = [0; 2];
            socket.read_exact(&mut header).unwrap();
            let length = usize::from(u16::from_le_bytes(header));
            assert!((3..=64).contains(&length));
            let mut body = vec![0; length - 2];
            socket.read_exact(&mut body).unwrap();
            rx.decrypt(&mut body);
            body
        };
        assert_eq!(read(), native_version::version_request(0));
        drop(read);
        let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/native-version-current.json"
        ))
        .unwrap();
        let hex = fixture["accepted"].as_str().unwrap();
        let mut accepted: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        tx.encrypt(&mut accepted);
        socket
            .write_all(&native_wire::frame(&accepted).unwrap())
            .unwrap();
        for _ in 0..9 {
            let mut header = [0; 2];
            socket.read_exact(&mut header).unwrap();
            let length = usize::from(u16::from_le_bytes(header));
            assert!((3..=64).contains(&length));
            let mut body = vec![0; length - 2];
            socket.read_exact(&mut body).unwrap();
            rx.decrypt(&mut body);
            received_tx.send(body).unwrap();
        }
        finish_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    });
    c.transport
        .start(
            address,
            Limits {
                read_chunk: 2,
                write_chunk: 3,
                ..Limits::default()
            },
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while c.transport.snapshot().phase != Phase::VersionAccepted {
        c.transport.poll();
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(2));
    }
    let mut counter = c.transport.queued_tx_counter().unwrap();
    let inventory = c.state.inventory.clone();
    let mp = c.state.world.mp;
    let f = frame(&c.state.world, true);
    let mut receive = |c: &mut LoginController, expected: Vec<u8>| {
        assert_eq!(
            received_rx.recv_timeout(Duration::from_secs(10)).unwrap(),
            expected
        );
        counter += expected.len() as u64;
        assert_eq!(c.transport.queued_tx_counter(), Some(counter));
        c.transport.poll();
        assert_eq!(c.state.inventory, inventory);
        assert_eq!(c.state.world.mp, mp);
        // Synthetic authority releases only the request throttle between sends.
        c.skill_pending = None;
        c.skill_target_pending = None;
        c.skill_ready = None;
    };
    for skill in [26, 42, 68] {
        c.cast_selected_with_input(skill, None, &f, CombatInput::default())
            .unwrap();
        let mut expected = vec![0x14, ((skill - 1) / 8) as u8, ((skill - 1) % 8) as u8];
        expected.extend(own.to_le_bytes());
        if skill == 68 {
            assert_eq!(
                c.skill_target_pending.unwrap().resolve(&c.state.world),
                Some(own)
            );
        }
        receive(&mut c, expected.clone());
        let selected = Target::capture(&c.state.world, own).unwrap();
        c.cast_selected_with_input(skill, Some(selected), &f, CombatInput::default())
            .unwrap();
        receive(&mut c, expected);
    }
    // A legal friendly support cast needs no Ctrl; the same player changing
    // teams is re-evaluated immediately, with no persistent Ctrl authorization.
    c.state.joined_room.as_mut().unwrap().members[1].force = 0;
    let selected = Target::capture(&c.state.world, other).unwrap();
    c.cast_selected_with_input(68, Some(selected), &f, CombatInput::default())
        .unwrap();
    assert_eq!(
        c.skill_target_pending.unwrap().resolve(&c.state.world),
        Some(other)
    );
    let mut body = vec![0x14, 8, 3];
    body.extend(other.to_le_bytes());
    receive(&mut c, body.clone());
    c.state.joined_room.as_mut().unwrap().members[1].force = 1;
    let before = c.transport.queued_tx_counter();
    for input in [
        CombatInput::default(),
        CombatInput {
            foreground: false,
            ctrl: true,
        },
    ] {
        assert!(c
            .cast_selected_with_input(68, Some(selected), &f, input)
            .is_err());
        assert_eq!(c.transport.queued_tx_counter(), before);
        assert!(c.skill_pending.is_none() && c.skill_ready.is_none());
    }
    c.cast_selected_with_input(68, Some(selected), &f, CTRL)
        .unwrap();
    body.extend([0x4c, 0x46, 0x43, 0x31, 0x14, 1, 0x67, 0xa5]);
    receive(&mut c, body);
    let mut held = HeldAttack::default();
    held.press(other, c.transport.snapshot().epoch, &c.state.world);
    assert!(c.continue_attack_with_input(&mut held, &f, CTRL).unwrap());
    let mut attack = c.state.world.attack_request(other).unwrap();
    attack.extend([0x4c, 0x46, 0x43, 0x31, 0x17, 1, 0x67, 0xa5]);
    receive(&mut c, attack);
    c.attack_pending = None;
    c.state.world.own_attack_packets += 1;
    c.state.attack_feedback = super::super::native_combat::Feedback::Acknowledged;
    let before = c.transport.queued_tx_counter();
    assert!(c
        .continue_attack_with_input(&mut held, &f, CombatInput::default())
        .is_err());
    assert!(!held.active());
    assert_eq!(c.transport.queued_tx_counter(), before);
    assert_eq!(c.state.inventory, inventory);
    finish_tx.send(()).unwrap();
    peer.join().unwrap();
    c.transport.cancel();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !c.transport.is_quiescent() {
        c.transport.poll();
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(2));
    }
}
