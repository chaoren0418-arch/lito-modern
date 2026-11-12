//! Owned loopback and synthetic state only; no game/server/resource dependency.
use super::*;
use crate::modern_lito::{
    native_forced_combat::tests::{snapshot, CTRL},
    native_world_scene::movement_tests::frame,
};

fn active() -> LoginController {
    let mut c = LoginController::default();
    c.state = snapshot();
    let mut target = c.state.world.own_actor().unwrap().clone();
    target.id += 1;
    target.x += 1;
    c.state.world.actors.insert(target.id, target);
    c.state.world.learned_skills.extend([2, 4, 26, 58, 60]);
    c.state.world.mp = Some([100, 100]);
    c
}

#[test]
fn lfc1_controller_preserves_existing_rejections_and_failed_send_state() {
    for case in 0..6 {
        let mut c = active();
        let target = c.state.world.own_actor().unwrap().id + 1;
        match case {
            0 => c.state.observer = true,
            1 => c.state.world.hp = Some([0, 100]),
            2 => {
                c.state.world.actors.remove(&target);
            }
            3 => c.state.world.actors.get_mut(&target).unwrap().dead = true,
            4 => c.state.world.actors.get_mut(&target).unwrap().x += 100,
            _ => {} // Otherwise valid; no transport owner exists.
        }
        let f = frame(&c.state.world, true);
        let before = serde_json::to_value(&c.state).unwrap();
        assert!(c.attack_with_input(target, CTRL).is_err());
        assert!(c.cast_with_input(4, Some(target), &f, CTRL).is_err());
        assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
        assert!(c.attack_pending.is_none() && c.skill_pending.is_none());
        assert!(c.transport.queued_tx_counter().is_none());
    }
}

#[test]
fn lfc1_controller_encrypted_loopback_held_repeat_spell_and_pet_race() {
    use crate::modern_lito::{
        native_combat::HeldAttack,
        native_version,
        native_wire::{self, CipherState},
    };
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        thread,
    };

    fn read(socket: &mut TcpStream, cipher: &mut CipherState, expected: &[u8]) {
        let mut header = [0; 2];
        socket.read_exact(&mut header).unwrap();
        assert_eq!(usize::from(u16::from_le_bytes(header)), expected.len() + 2);
        let mut body = vec![0; expected.len()];
        socket.read_exact(&mut body).unwrap();
        cipher.decrypt(&mut body);
        assert_eq!(body, expected);
    }
    fn extended(mut body: Vec<u8>) -> Vec<u8> {
        body.extend([0x4c, 0x46, 0x43, 0x31, body[0], 1, 0x67, 0xa5]);
        body
    }
    fn pump(c: &mut LoginController, ready: impl Fn(&LoginController) -> bool) {
        let until = Instant::now() + Duration::from_secs(10);
        while !ready(c) {
            c.transport.poll();
            assert!(Instant::now() < until, "owned loopback timeout");
            thread::sleep(Duration::from_millis(2));
        }
    }
    let mut c = active();
    let target = c.state.world.own_actor().unwrap().id + 1;
    let attack = c.state.world.attack_request(target).unwrap();
    let spell = super::super::native_skills::request(&c.state.world, 4, Some(target)).unwrap();
    let self_spell = super::super::native_skills::request(&c.state.world, 26, None).unwrap();
    let own = c.state.world.own_actor().unwrap();
    let ground = [own.x + 1, own.y];
    let ground_spell =
        super::super::native_skills::ground_request(&c.state.world, 58, ground).unwrap();
    let expected = vec![
        extended(attack.clone()),
        attack.clone(),
        extended(spell.clone()),
        spell.clone(),
        spell.clone(),
        attack,
        vec![0x14, 0, 1],
        ground_spell,
        extended(self_spell),
    ];
    let bytes: u64 = expected.iter().map(|body| body.len() as u64).sum();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let (receipt_tx, receipt_rx) = std::sync::mpsc::channel();
    let (finish_tx, finish_rx) = std::sync::mpsc::channel();
    let server = thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(10);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < until, "owned accept timeout");
                    thread::sleep(Duration::from_millis(2));
                }
                Err(e) => panic!("owned listener: {e}"),
            }
        };
        socket.set_nonblocking(false).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let seed = 0x12345678u32;
        let mut key = vec![1];
        key.extend(seed.to_le_bytes());
        socket
            .write_all(&native_wire::frame(&key).unwrap())
            .unwrap();
        let mut rx = CipherState::new(seed);
        let mut tx = CipherState::new(seed);
        read(&mut socket, &mut rx, &native_version::version_request(0));
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
        for body in expected {
            read(&mut socket, &mut rx, &body);
            receipt_tx.send(()).unwrap();
        }
        finish_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    });
    c.transport
        .start(
            peer,
            Limits {
                read_chunk: 2,
                write_chunk: 3,
                ..Limits::default()
            },
        )
        .unwrap();
    pump(&mut c, |c| {
        c.transport.snapshot().phase == Phase::VersionAccepted
    });
    let initial = c.transport.queued_tx_counter().unwrap();
    // Respect both bounded queues: wait for the peer and drain worker events
    // before synthesizing the next server acknowledgement/state transition.
    let received = |c: &mut LoginController| {
        receipt_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        c.transport.poll();
    };
    let mut held = HeldAttack::default();
    held.press(target, c.transport.snapshot().epoch, &c.state.world);
    let f = frame(&c.state.world, true);
    assert!(c.continue_attack_with_input(&mut held, &f, CTRL).unwrap());
    received(&mut c);
    // Synthetic acknowledgement; this test verifies client dispatch, not server combat.
    c.attack_pending = None;
    c.state.attack_feedback = super::super::native_combat::Feedback::Acknowledged;
    c.state.world.own_attack_packets += 1;
    assert!(c
        .continue_attack_with_input(
            &mut held,
            &f,
            CombatInput {
                ctrl: false,
                ..CTRL
            }
        )
        .unwrap());
    received(&mut c);
    c.attack_pending = None;
    c.cast_with_input(4, Some(target), &f, CTRL).unwrap();
    received(&mut c);
    c.skill_pending = None;
    c.skill_ready = None;
    c.cast(4, Some(target), &f).unwrap();
    received(&mut c);
    c.skill_pending = None;
    c.skill_ready = None;
    c.state.joined_room.as_mut().unwrap().mode = 0;
    c.cast_with_input(4, Some(target), &f, CTRL).unwrap();
    received(&mut c);
    c.skill_pending = None;
    c.skill_ready = None;
    c.attack_with_input(target, CTRL).unwrap();
    received(&mut c);
    c.attack_pending = None;
    c.state.joined_room.as_mut().unwrap().mode = 1;
    c.cast_with_input(2, None, &f, CTRL).unwrap();
    received(&mut c);
    c.skill_pending = None;
    c.skill_ready = None;
    c.cast_ground(58, ground, &f).unwrap();
    received(&mut c);
    c.skill_pending = None;
    c.skill_ready = None;
    c.cast_with_input(26, None, &f, CTRL).unwrap();
    received(&mut c);
    assert_eq!(c.transport.queued_tx_counter(), Some(initial + bytes));
    finish_tx.send(()).unwrap();
    server.join().unwrap();
    c.transport.cancel();
    pump(&mut c, |c| c.transport.is_quiescent());
}
