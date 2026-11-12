//! Owned loopback and pure controller tests; never a formal service or game.
use super::*;
use crate::modern_lito::{
    native_skills as skills,
    native_world_scene::movement_tests::{frame, world},
};

fn active() -> LoginController {
    let mut c = LoginController::default();
    c.state.phase = LoginPhase::BattlePending;
    c.state.world = world();
    c.state.world.mp = Some([20, 20]);
    c.state
        .world
        .learned_skills
        .extend([1, 2, 3, 4, 58, 63, 77]);
    c
}
fn pending(c: &mut LoginController, now: Instant, target: Option<u32>) {
    let w = &c.state.world;
    c.skill_pending = Some(skills::Pending {
        since: now,
        casts: w.own_cast_packets,
        messages: w.message_count,
        generation: w.generation,
        incarnation: w.own_actor().unwrap().incarnation,
    });
    c.skill_context = Some(skills::CastContext::capture(
        c.transport.snapshot().epoch,
        w,
        2,
        target,
    ));
    c.skill_ready = Some(now + Duration::from_millis(10));
    c.state.skill_feedback = skills::Feedback::Pending;
}

#[test]
fn interrupted_requests_retire_even_when_restriction_is_released_in_one_batch() {
    for change in 0..10 {
        let mut c = active();
        let now = Instant::now();
        pending(&mut c, now, None);
        let own = c.state.world.own_actor().unwrap().id;
        match change {
            0 => c.state.world.generation += 1,
            1 => c.state.world.local_corrections += 1,
            2 => c.state.world.actors.get_mut(&own).unwrap().incarnation += 1,
            3 => c.state.world.actors.get_mut(&own).unwrap().dead = true,
            4 => c.state.world.hp = Some([0, 50]),
            5 => {
                c.state.world.actors.remove(&own);
            }
            6 => c.state.phase = LoginPhase::LobbyReady,
            7 => {
                c.state.world.accept(&[0x25, 24]).unwrap();
                c.state.world.accept(&[0x25, 25]).unwrap();
                assert!(c.state.world.movement_blocks.is_empty());
            }
            8 => {
                c.state.world.learned_skills.remove(&2);
            }
            _ => {
                c.skill_context = Some(skills::CastContext::capture(
                    c.transport.snapshot().epoch + 1,
                    &c.state.world,
                    2,
                    None,
                ))
            }
        }
        let authority = serde_json::to_value(&c.state.world).unwrap();
        c.observe_skill(now);
        assert_eq!(
            c.state.skill_feedback,
            skills::Feedback::Interrupted,
            "change {change}"
        );
        assert!(c.skill_pending.is_none() && c.skill_context.is_none());
        assert_eq!(
            c.skill_ready,
            Some(now + Duration::from_millis(10)),
            "interrupt does not erase reuse"
        );
        assert_eq!(serde_json::to_value(&c.state.world).unwrap(), authority);
    }
}

#[test]
fn target_disappearance_interrupts_pending_but_not_an_already_acknowledged_cast() {
    for acknowledged in [false, true] {
        let mut c = active();
        let mut target = c.state.world.own_actor().unwrap().clone();
        target.id = 2;
        target.x += 1;
        c.state.world.actors.insert(2, target);
        let now = Instant::now();
        pending(&mut c, now, Some(2));
        c.state.world.actors.remove(&2);
        if acknowledged {
            c.state.world.own_cast_packets += 1;
        }
        c.observe_skill(now);
        assert_eq!(
            c.state.skill_feedback,
            if acknowledged {
                skills::Feedback::CastAction
            } else {
                skills::Feedback::Interrupted
            }
        );
    }
}

#[test]
fn rejection_timeout_and_failed_send_do_not_mutate_authority_or_create_retries() {
    let mut c = active();
    let now = Instant::now();
    pending(&mut c, now, None);
    c.state.world.accept(&[0x10, 22, 1, 0]).unwrap();
    c.observe_skill(now);
    assert_eq!(c.state.skill_feedback, skills::Feedback::Rejected(278));
    pending(&mut c, now, None);
    c.observe_skill(now + Duration::from_secs(2));
    assert_eq!(c.state.skill_feedback, skills::Feedback::NoResponse);
    assert!(c.skill_pending.is_none());
    let before = serde_json::to_value(&c.state.world).unwrap();
    let f = frame(&c.state.world, true);
    assert!(c
        .cast_request_at(2, None, None, None, None, &f, CombatInput::default(), now + Duration::from_secs(3))
        .unwrap_err()
        .to_string()
        .contains("native send owner"));
    assert!(c.skill_pending.is_none() && c.skill_context.is_none());
    assert_eq!(serde_json::to_value(&c.state.world).unwrap(), before);
}

#[test]
fn animation_boundary_is_resource_clock_and_respects_speed_and_longer_reuse() {
    use crate::modern_lito::{
        native_world_animation::Animator, sprite_actions::SpriteActionCatalog,
    };
    // 37 ticks / 25fps = exactly 1480ms. A fixture clock, not a claimed
    // replacement for every real sprite's animation or the server timeline.
    let catalog = SpriteActionCatalog::parse(b"#0 32 test\n110.framerate(25)\n0.walk(1 1,0.0:1)\n3.idle(1 1,8.0:1)\n19.cast(1 1,16.0:37)\n", Default::default()).unwrap();
    for (speed, end) in [(0, 1480u64), (1, 1110), (2, 2960)] {
        let mut c = active();
        let now = Instant::now();
        let id = c.state.world.own_actor().unwrap().id;
        let a = c.state.world.actors.get_mut(&id).unwrap();
        a.graphics = 0;
        a.walk_action = 0;
        a.speed = speed;
        a.brave = 0;
        a.motion_from = None;
        a.action_sequence += 1;
        a.actions = vec![crate::modern_lito::native_combat::ActionCue {
            sequence: a.action_sequence,
            action: 19,
            source: "action",
        }];
        let mut animation = Animator::default();
        c.skill_ready = Some(now + Duration::from_millis(10));
        for ms in [0, 10, end - 1, end] {
            let pose = animation
                .pose(&catalog, c.state.world.own_actor().unwrap(), ms * 1_000_000)
                .unwrap();
            let f = frame(&c.state.world, pose.phase == "idle");
            assert_eq!(
                c.skill_action_ready(&f, now + Duration::from_millis(ms)),
                ms == end,
                "speed={speed}, ms={ms}"
            );
            if ms < end {
                let before = serde_json::to_value(&c.state.world).unwrap();
                assert!(c
                    .cast_request_at(2, None, None, None, None, &f, CombatInput::default(), now + Duration::from_millis(ms))
                    .unwrap_err()
                    .to_string()
                    .contains("等待施法動作完成"));
                assert_eq!(serde_json::to_value(&c.state.world).unwrap(), before);
            } else {
                c.skill_ready = Some(now + Duration::from_millis(5000));
                assert!(!c.skill_action_ready(&f, now + Duration::from_millis(ms)));
                assert!(c.skill_action_ready(&f, now + Duration::from_millis(5000)));
            }
        }
    }
}

#[test]
fn owned_encrypted_transport_sends_once_until_ack_and_animation_complete() {
    use crate::modern_lito::{
        native_version,
        native_wire::{self, CipherState},
    };
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        thread,
    };
    fn read(s: &mut TcpStream, cipher: &mut CipherState) -> Vec<u8> {
        let mut h = [0; 2];
        s.read_exact(&mut h).unwrap();
        let n = usize::from(u16::from_le_bytes(h));
        assert!((3..=native_wire::MAX_FRAME).contains(&n));
        let mut b = vec![0; n - 2];
        s.read_exact(&mut b).unwrap();
        cipher.decrypt(&mut b);
        b
    }
    fn send(s: &mut TcpStream, cipher: &mut CipherState, mut b: Vec<u8>) {
        cipher.encrypt(&mut b);
        s.write_all(&native_wire::frame(&b).unwrap()).unwrap();
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut socket = loop {
            match listener.accept() {
                Ok((s, _)) => break s,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline);
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
        let seed = 0x12345678u32;
        let mut key = vec![1];
        key.extend(seed.to_le_bytes());
        socket
            .write_all(&native_wire::frame(&key).unwrap())
            .unwrap();
        let mut rx = CipherState::new(seed);
        let mut tx = CipherState::new(seed);
        assert_eq!(
            read(&mut socket, &mut rx),
            native_version::version_request(0)
        );
        let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/native-version-current.json"
        ))
        .unwrap();
        let hex = fixture["accepted"].as_str().unwrap();
        send(
            &mut socket,
            &mut tx,
            (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect(),
        );
        assert_eq!(read(&mut socket, &mut rx), [0x14, 0, 1]);
        assert_eq!(read(&mut socket, &mut rx), [0x14, 0, 2]);
    });
    let mut c = active();
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
    let now = Instant::now();
    let ready = frame(&c.state.world, true);
    let mut book = crate::modern_lito::native_skill_selection::Model::default();
    book.sync(c.transport.snapshot().epoch, &c.state.world, true);
    book.bind(0, Some(2)).unwrap();
    book.bind(1, Some(3)).unwrap();
    let first = book
        .bound(c.transport.snapshot().epoch, &c.state.world, 0)
        .unwrap();
    let before = serde_json::to_value(&c.state.world).unwrap();
    c.cast_request_at(first, None, None, None, None, &ready, CombatInput::default(), now).unwrap();
    let count = c.transport.queued_tx_counter().unwrap();
    for slot in [0, 1, 0, 1] {
        let id = book
            .bound(c.transport.snapshot().epoch, &c.state.world, slot)
            .unwrap();
        assert!(c
            .cast_request_at(id, None, None, None, None, &ready, CombatInput::default(), now + Duration::from_millis(10))
            .is_err());
        assert_eq!(c.transport.queued_tx_counter(), Some(count));
    }
    assert_eq!(serde_json::to_value(&c.state.world).unwrap(), before);
    // Simulated receipt explicitly exercises client admission; not server skill rules.
    c.state.world.own_cast_packets += 1;
    c.observe_skill(now + Duration::from_millis(10));
    let animating = frame(&c.state.world, false);
    assert!(c
        .cast_request_at(3, None, None, None, None, &animating, CombatInput::default(), now + Duration::from_millis(1479))
        .is_err());
    assert_eq!(c.transport.queued_tx_counter(), Some(count));
    c.cast_request_at(3, None, None, None, None, &ready, CombatInput::default(), now + Duration::from_millis(1480))
        .unwrap();
    // The cipher counter advances by payload bytes, not by packet count.
    // The owned peer verifies the exact two decrypted requests.
    assert_ne!(c.transport.queued_tx_counter(), Some(count));
    server.join().unwrap();
    c.transport.cancel();
    while !c.transport.is_quiescent() {
        c.transport.poll();
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(2));
    }
}
