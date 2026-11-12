//! GAME ingress -> C_ADDR send queue over an owned loopback peer. The peer only
//! replays deployed wire shapes; it is not the game server and proves no
//! server-side acceptance (the deployed server sends no acceptance packet).
use super::*;
use crate::modern_lito::{
    native_integrity::{seeded_crc, Status, CRC_SEED, LEGACY_ANCHOR_BYTES},
    native_version,
    native_wire::{self, CipherState},
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

const CHALLENGE: [u8; 14] = [0x7b, 33, 0x98, 0xba, 0xdc, 0xfe, 0x40, 0xd8, 0x4e, 0, 5, 0, 0, 0];
const FRESH: [u8; 14] = [0x7b, 33, 0x11, 0x22, 0x33, 0x44, 0x40, 0xd8, 0x4e, 0, 5, 0, 0, 0];
const UNKNOWN: [u8; 14] = [0x7b, 33, 1, 0, 0, 0, 0x41, 0xd8, 0x4e, 0, 5, 0, 0, 0];

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
    socket.write_all(&native_wire::frame(&body).unwrap()).unwrap();
}

fn pump(c: &mut LoginController, ready: impl Fn(&LoginController) -> bool) {
    let until = Instant::now() + Duration::from_secs(10);
    while !ready(c) {
        for event in c.transport.poll() {
            if let Event::Packet { body, .. } = event {
                c.accept_game(&body).unwrap();
            }
        }
        assert!(Instant::now() < until, "loopback timeout: {:?}", c.transport.snapshot());
        thread::sleep(Duration::from_millis(2));
    }
}

/// Expected C_ADDR for a challenge id, built independently of the producer.
fn expected(id: [u8; 4]) -> Vec<u8> {
    let mut body = vec![0x2a];
    body.extend(id);
    body.extend(0x1569_D958u32.to_le_bytes());
    body.extend([0; 4]);
    body
}

#[test]
fn integrity_controller_answers_each_supported_challenge_once_on_owning_connection() {
    assert_eq!(seeded_crc(&LEGACY_ANCHOR_BYTES, CRC_SEED), 0x1569_D958);
    let mut c = LoginController::default();
    assert!(c.state.world.accept(&[0x28, 5, 0, 0]).unwrap());
    assert!(c.state.world.accept(&[0xd2]).unwrap());
    c.state.phase = LoginPhase::BattlePending;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
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
        socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        socket.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
        let seed = 0x12345678u32;
        let mut key = vec![1];
        key.extend(seed.to_le_bytes());
        socket.write_all(&native_wire::frame(&key).unwrap()).unwrap();
        let mut rx = CipherState::new(seed);
        let mut tx = CipherState::new(seed);
        assert_eq!(read(&mut socket, &mut rx), native_version::version_request(0));
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
        // Duplicate and out-of-profile challenges must not produce frames;
        // any extra frame would precede FRESH's reply and fail exact matching.
        for body in [CHALLENGE, CHALLENGE, UNKNOWN, FRESH] {
            send(&mut socket, &mut tx, body.to_vec());
        }
        assert_eq!(read(&mut socket, &mut rx), expected([0x98, 0xba, 0xdc, 0xfe]));
        assert_eq!(read(&mut socket, &mut rx), expected([0x11, 0x22, 0x33, 0x44]));
    });
    c.transport
        .start(peer, Limits { read_chunk: 2, write_chunk: 3, ..Limits::default() })
        .unwrap();
    pump(&mut c, |c| c.transport.snapshot().phase == Phase::VersionAccepted);
    pump(&mut c, |c| c.state.world.integrity_challenge == Some([0x44332211, 0x004E_D840, 5]));
    assert_eq!(c.state.world.integrity_status, Status::ReplyQueued);
    assert_eq!(c.state.phase, LoginPhase::BattlePending);
    server.join().unwrap();
    c.transport.cancel();
    pump(&mut c, |c| c.transport.is_quiescent());
}
