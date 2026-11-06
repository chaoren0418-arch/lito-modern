//! Controller checks plus an owned loopback peer; no game server or credentials.
use super::*;
use crate::modern_lito::native_world_scene::movement_tests::{frame, world};

fn active() -> LoginController {
    let mut c = LoginController::default();
    c.state.phase = LoginPhase::BattlePending;
    c.state.world = world();
    c
}

#[test]
fn movement_tests_controller_rejects_in_flight_before_transport_or_prediction() {
    let mut c = active();
    let stale = frame(&c.state.world, true);
    c.state.world.predict_step(0).unwrap();
    let moving = frame(&c.state.world, false);
    let before = serde_json::to_value(&c.state).unwrap();
    for f in [&stale, &moving] {
        let error = c.move_presented(0, f).unwrap_err();
        assert!(error.to_string().contains("等待目前步伐"));
        assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    }
}

#[test]
fn movement_tests_valid_preflight_but_failed_queue_cannot_predict() {
    let mut c = active();
    let f = frame(&c.state.world, true);
    let before = serde_json::to_value(&c.state).unwrap();
    let error = c.move_presented(0, &f).unwrap_err();
    assert!(error
        .to_string()
        .contains("native send owner is not current/ready"));
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    assert_eq!(c.state.world.predicted_moves, 0);
}

#[test]
fn movement_tests_controller_malformed_blink_never_creates_ack_or_lock() {
    let mut c = active();
    let before = serde_json::to_value(&c.state).unwrap();
    assert!(c.accept_game(&[0x63, 0]).is_err());
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    assert!(c
        .accept_game(&[0x63])
        .unwrap_err()
        .to_string()
        .contains("native send owner"));
    assert_eq!(
        c.state.world.movement_lock,
        super::super::native_world::MovementLock::AckPending
    );
    c.accept_game(&[0x25, 17]).unwrap();
    assert!(
        c.state.world.move_request(0).is_err(),
        "failed C_BLINK cannot qualify release"
    );
}

#[test]
fn movement_tests_encrypted_single_owner_correction_ack_release_and_next_step() {
    use crate::modern_lito::{
        native_version,
        native_wire::{self, CipherState},
        native_world::MovementLock,
    };
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
    };

    fn read(socket: &mut TcpStream, cipher: &mut CipherState) -> Vec<u8> {
        let mut header = [0; 2];
        socket.read_exact(&mut header).unwrap();
        let length = usize::from(u16::from_le_bytes(header));
        assert!((3..=native_wire::MAX_FRAME).contains(&length));
        let mut body = vec![0; length - 2];
        socket.read_exact(&mut body).unwrap();
        cipher.decrypt(&mut body);
        body
    }
    fn send(socket: &mut TcpStream, cipher: &mut CipherState, mut body: Vec<u8>) {
        cipher.encrypt(&mut body);
        socket
            .write_all(&native_wire::frame(&body).unwrap())
            .unwrap();
    }
    fn pump(c: &mut LoginController, ready: impl Fn(&LoginController) -> bool) {
        let until = Instant::now() + Duration::from_secs(10);
        while !ready(c) {
            for event in c.transport.poll() {
                if let Event::Packet { body, .. } = event {
                    c.accept_game(&body).unwrap();
                }
            }
            assert!(
                Instant::now() < until,
                "loopback timeout: {:?}",
                c.transport.snapshot()
            );
            thread::sleep(Duration::from_millis(2));
        }
    }

    let mut c = active();
    let request = c.state.world.move_request(0).unwrap();
    let own = c.state.world.own_actor().unwrap();
    let mut correction = vec![0x12];
    correction.extend(own.id.to_le_bytes());
    correction.extend(own.x.to_le_bytes());
    correction.extend((own.y - 1).to_le_bytes());
    correction.push(4);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let (release, release_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(10);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < until, "owned listener accept timeout");
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
        assert_eq!(
            read(&mut socket, &mut rx),
            native_version::version_request(0)
        );
        let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/native-version-current.json"
        ))
        .unwrap();
        let hex = fixture["accepted"].as_str().unwrap();
        let accepted = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        send(&mut socket, &mut tx, accepted);
        assert_eq!(read(&mut socket, &mut rx), request);
        // Deliberate silent bind refusal followed by authoritative recovery;
        // these packets exercise the client, not a simulated server rules claim.
        for body in [
            vec![0x25, 24],
            vec![0x25, 25],
            correction,
            vec![0x63],
            vec![0x63],
        ] {
            send(&mut socket, &mut tx, body);
        }
        assert_eq!(read(&mut socket, &mut rx), [0x5a]);
        release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        send(&mut socket, &mut tx, vec![0x25, 17]);
        send(&mut socket, &mut tx, vec![0x25, 17]);
        // A duplicate ACK or extra in-flight move appears here and fails exact matching.
        assert_eq!(read(&mut socket, &mut rx), request);
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
    let initial_counter = c.transport.queued_tx_counter().unwrap();
    let old = frame(&c.state.world, true);
    c.move_presented(0, &old).unwrap();
    assert!(c.move_presented(0, &old).is_err());
    pump(&mut c, |c| c.state.world.movement_locks == 2);
    assert_eq!(c.state.world.movement_lock, MovementLock::AwaitRelease);
    let locked = frame(&c.state.world, true);
    assert!(c.move_presented(0, &locked).is_err());
    assert_eq!(
        c.transport.queued_tx_counter(),
        Some(initial_counter.wrapping_add(7))
    );
    assert_eq!(
        (
            c.state.world.predicted_moves,
            c.state.world.local_corrections
        ),
        (1, 1)
    );
    release.send(()).unwrap();
    pump(&mut c, |c| {
        c.state.world.movement_lock == MovementLock::Open
    });
    assert!(c.move_presented(0, &locked).is_err());
    let fresh = frame(&c.state.world, true);
    c.move_presented(0, &fresh).unwrap();
    server.join().unwrap();
    assert_eq!(c.state.world.predicted_moves, 2);
    assert_eq!(
        c.transport.queued_tx_counter(),
        Some(initial_counter.wrapping_add(13))
    );
    c.transport.cancel();
    pump(&mut c, |c| c.transport.is_quiescent());
}
