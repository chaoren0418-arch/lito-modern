//! G06.1 client contracts. All packets are offline fixtures; no server/DB is run.
use super::*;
use crate::modern_lito::{native_lobby, native_room::Settings, native_room_ui};

fn z(body: &mut Vec<u8>, text: &str) {
    if !text.is_empty() {
        body.extend_from_slice(&encode_text(text, 89).unwrap());
    }
    body.push(0);
}
fn header(op: u8, id: u32) -> Vec<u8> {
    let mut body = vec![op];
    body.extend(id.to_le_bytes());
    body
}
fn info(kind: u8, id: u32) -> Vec<u8> {
    let mut body = vec![0xd5, kind];
    body.extend(id.to_le_bytes());
    body
}
fn named(op: u8, id: u32, name: &str) -> Vec<u8> {
    let mut body = header(op, id);
    z(&mut body, name);
    body
}
fn joined(id: u32, name: &str, password: &str, owner: &str) -> Vec<u8> {
    let mut body = vec![0xce, 0];
    body.extend(id.to_le_bytes());
    z(&mut body, name);
    body.push(if password.is_empty() { 0 } else { 4 });
    z(&mut body, password);
    body.extend([2, 0, 8, 0, 1, 0x34, 2]);
    body.extend([0; 5]);
    z(&mut body, owner);
    body.extend([0; 5]);
    body
}
fn member(id: u32, name: &str) -> Vec<u8> {
    let mut body = named(0xcd, id, name);
    body.extend([0, 0, 0]);
    body.extend([0; 12]);
    body.extend([0, 0, 0]);
    z(&mut body, "Ready");
    body.extend([0; 4]);
    body.extend([0, 0]);
    body.extend([0; 4]);
    body.push(0);
    body.extend([0; 21]);
    body
}
fn summary(id: u32, name: &str, owner: &str) -> Vec<u8> {
    let mut body = info(0, id);
    z(&mut body, name);
    body.extend([1, 0x34, 2, 0]);
    z(&mut body, owner);
    body.extend([0; 4]);
    body
}
fn configuration(id: u32, name: &str, password: &str) -> Vec<u8> {
    let mut body = info(1, id);
    z(&mut body, name);
    z(&mut body, password);
    body.extend([if password.is_empty() { 0 } else { 4 }, 8]);
    body.extend([0; 5]);
    body
}
fn row(id: u32, name: &str) -> Vec<u8> {
    let mut body = vec![0xcc, 0];
    body.extend(id.to_le_bytes());
    z(&mut body, name);
    body.extend([0, 2, 0, 8, 0, 0, 0, 1]);
    body
}
fn delta(kind: u8, id: u32, tail: &[u8]) -> Vec<u8> {
    let mut body = vec![0xcc, kind];
    body.extend(id.to_le_bytes());
    body.extend(tail);
    body
}
fn pending(phase: LoginPhase) -> LoginController {
    let mut c = LoginController::default();
    c.state.phase = phase;
    c.state.selected_source = Some("SourceA".into());
    c.state.selected_character = Some("Owner".into());
    c.state.channel = Some(native_lobby::Channel {
        id: 0,
        users: 2,
        capacity: 200,
    });
    c.requested_channel = Some(0);
    c.deadline = Some(Instant::now() + Duration::from_secs(8));
    c
}
fn active() -> LoginController {
    let mut c = pending(LoginPhase::RoomCreating);
    c.accept_game(&joined(1, "Room", "1234", "Owner")).unwrap();
    c.accept_game(&member(1, "Owner")).unwrap();
    c.accept_game(&member(1, "Peer(SourceB)")).unwrap();
    c
}
fn unchanged(c: &mut LoginController, packet: &[u8]) {
    let before = serde_json::to_value(&c.state).unwrap();
    let deadline = c.deadline;
    assert!(
        c.accept_game(packet).is_err(),
        "invalid packet accepted: {packet:?}"
    );
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    assert_eq!(c.deadline, deadline);
}

#[test]
fn create_accepts_server_allocated_or_reused_id_only_after_ce_and_self() {
    for id in [1, 77, u32::MAX] {
        let mut c = pending(LoginPhase::RoomCreating);
        let deadline = c.deadline;
        c.accept_game(&row(1, "Default")).unwrap();
        c.accept_game(&row(id, "Renamed")).unwrap();
        c.accept_game(&member(id, "Peer(SourceB)")).unwrap();
        assert!(c.state.joined_room.is_none());
        assert_eq!(c.state.phase, LoginPhase::RoomCreating);
        c.accept_game(&joined(id, "Renamed", "1234", "Owner"))
            .unwrap();
        assert_eq!(c.state.phase, LoginPhase::RoomAwaitSelf);
        assert_eq!(c.deadline, deadline);
        c.accept_game(&member(id, "Owner")).unwrap();
        assert_eq!(c.state.phase, LoginPhase::RoomReady);
        assert!(c.deadline.is_none());
        assert_eq!(c.state.joined_room.as_ref().unwrap().id, id);
        assert_eq!(c.state.rooms.iter().filter(|r| r.id == id).count(), 1);
        assert!(!serde_json::to_string(&c.state).unwrap().contains("1234"));
    }
}

#[test]
fn all_formal_join_rejections_retire_early_roster_without_membership() {
    for phase in [LoginPhase::RoomCreating, LoginPhase::RoomJoining] {
        for code in [1, 3, 4, 5, 7, 10, 11, 12, 13, 14, 20, 21] {
            let mut c = pending(phase);
            if phase == LoginPhase::RoomJoining {
                c.requested_room = Some(1);
            }
            c.accept_game(&member(1, "Peer(SourceB)")).unwrap();
            c.accept_game(&[0xce, code]).unwrap();
            assert_eq!(c.state.phase, LoginPhase::LobbyReady);
            assert_eq!(c.state.room_result, Some(code));
            assert!(
                c.deadline.is_none() && c.requested_room.is_none() && c.state.joined_room.is_none()
            );
            assert!(c
                .early_roster
                .take(c.transport.snapshot().epoch, 77)
                .unwrap()
                .is_empty());
        }
    }
}

#[test]
fn edit_success_and_refusal_use_authoritative_configuration_not_draft() {
    for (name, password) in [("Edited", ""), ("Room", "1234")] {
        let mut c = active();
        c.state.phase = LoginPhase::RoomEditing;
        c.deadline = Some(Instant::now() + Duration::from_secs(8));
        let deadline = c.deadline;
        c.accept_game(&summary(1, name, "Owner")).unwrap();
        assert_eq!(c.state.phase, LoginPhase::RoomEditing);
        assert_eq!(c.deadline, deadline);
        assert!(c.state.joined_room.as_ref().unwrap().password_required);
        c.accept_game(&configuration(1, name, password)).unwrap();
        assert_eq!(c.state.phase, LoginPhase::RoomReady);
        assert!(c.deadline.is_none());
        let room = c.state.joined_room.as_ref().unwrap();
        assert_eq!(room.name, name);
        assert_eq!(room.password_required, !password.is_empty());
        assert!(!serde_json::to_string(&c.state).unwrap().contains("1234"));
    }
}

#[test]
fn owner_transfer_uses_exact_display_identity_and_invalidates_open_form() {
    let mut c = active();
    let draft = native_room_ui::Draft::capture(0, &c.state, true).unwrap();
    let choice = native_room_ui::MemberChoice::capture(0, &c.state, 1).unwrap();
    let mut owner = info(5, 1);
    z(&mut owner, "Peer(SourceB)");
    c.accept_game(&owner).unwrap();
    assert!(draft.validate(0, &c.state).is_err());
    assert!(choice.resolve(0, &c.state).is_err());
    let settings = Settings {
        name: "No",
        password: "",
        attributes: 0,
        capacity: 8,
        options: None,
    };
    assert!(c
        .edit_room(&settings)
        .unwrap_err()
        .to_string()
        .contains("only owner"));
    assert!(c
        .kick_member("Peer(SourceB)")
        .unwrap_err()
        .to_string()
        .contains("only owner"));
    c.accept_game(&named(0xd0, 1, "Peer(SourceB)")).unwrap();
    // Departure alone must not infer a replacement owner from member order.
    assert_eq!(c.state.joined_room.as_ref().unwrap().owner, "Peer(SourceB)");
    let mut owner = info(5, 1);
    z(&mut owner, "Owner");
    c.accept_game(&owner).unwrap();
    assert!(native_room_ui::Draft::capture(0, &c.state, true).is_ok());
    let mut foreign = info(5, 2);
    z(&mut foreign, "Intruder");
    unchanged(&mut c, &foreign);
    foreign[2..6].fill(0);
    unchanged(&mut c, &foreign);
}

#[test]
fn kick_acceptance_waits_for_target_departure_and_rejection_keeps_roster() {
    for accepted in [false, true] {
        let mut c = active();
        c.state.phase = LoginPhase::KickPending;
        c.kick_pending = Some((1, "Peer(SourceB)".into(), false));
        c.deadline = Some(Instant::now() + Duration::from_secs(8));
        let deadline = c.deadline;
        c.accept_game(&[0xfc, u8::from(!accepted), 0, 0, 0])
            .unwrap();
        assert_eq!(c.state.joined_room.as_ref().unwrap().members.len(), 2);
        if accepted {
            unchanged(&mut c, &[0xfc, 0, 0, 0, 0]);
            let mut notice = vec![0xd6];
            z(&mut notice, "Peer(SourceB)");
            c.accept_game(&notice).unwrap();
            assert_eq!(c.state.phase, LoginPhase::KickPending);
            assert_eq!(c.deadline, deadline);
            assert_eq!(c.state.joined_room.as_ref().unwrap().members.len(), 2);
            c.accept_game(&named(0xd0, 1, "Peer(SourceB)")).unwrap();
            assert_eq!(c.state.joined_room.as_ref().unwrap().members.len(), 1);
        }
        assert_eq!(c.state.phase, LoginPhase::RoomReady);
        assert!(c.kick_pending.is_none() && c.deadline.is_none());
        assert_eq!(c.state.room_kick_result, Some(accepted));
    }
    let mut c = active();
    c.state.phase = LoginPhase::KickPending;
    unchanged(&mut c, &[0xfc, 0, 0, 0, 0]);
}

#[test]
fn directory_cross_source_updates_reuse_rename_and_delete_by_server_id() {
    for source in ["SourceA", "SourceB"] {
        let mut c = pending(LoginPhase::LobbyReady);
        c.state.selected_source = Some(source.into());
        c.accept_game(&row(1, "Default")).unwrap();
        let choice = native_room_ui::RoomChoice::capture(0, &c.state, 0).unwrap();
        for packet in [
            row(77, "Independent"),
            delta(1, 1, b"Renamed\0"),
            delta(2, 1, &[4]),
            delta(3, 1, &[3, 0]),
            delta(4, 1, &[16, 0]),
            delta(5, 1, &[0, 0, 2]),
        ] {
            c.accept_game(&packet).unwrap();
        }
        let r = &c.state.rooms[choice.join_index(0, &c.state).unwrap()];
        assert_eq!(
            (
                r.id,
                r.name.as_str(),
                r.attributes,
                r.users,
                r.capacity,
                r.option
            ),
            (1, "Renamed", 4, 3, 16, 2)
        );
        assert_eq!(c.state.rooms.len(), 2);
        assert!(c.state.joined_room.is_none());
        c.accept_game(&header(0xcf, 77)).unwrap();
        c.accept_game(&delta(1, 77, b"Stale\0")).unwrap();
        assert_eq!(c.state.rooms.len(), 1);
        c.accept_game(&row(1, "Default")).unwrap();
        assert_eq!(choice.join_index(0, &c.state).unwrap(), 0);
        assert_eq!(c.state.rooms[0].name, "Default");
    }
}

#[test]
fn refresh_deltas_stay_in_pending_batch_and_never_resurrect_deleted_rows() {
    let mut c = pending(LoginPhase::LobbyReady);
    c.accept_game(&row(1, "Old")).unwrap();
    c.state.phase = LoginPhase::LoadingRooms;
    c.pending_rooms = Some(vec![]);
    c.accept_game(&delta(1, 1, b"Unknown\0")).unwrap();
    assert!(c.pending_rooms.as_ref().unwrap().is_empty());
    c.accept_game(&row(1, "New")).unwrap();
    c.accept_game(&delta(1, 1, b"Updated\0")).unwrap();
    c.accept_game(&row(2, "Deleted")).unwrap();
    c.accept_game(&header(0xcf, 2)).unwrap();
    assert_eq!(c.state.rooms[0].name, "Old");
    c.accept_game(&[0xca, 0, 2, 0, 200, 0]).unwrap();
    assert_eq!(c.state.rooms.len(), 1);
    assert_eq!(c.state.rooms[0].name, "Updated");
    c.state.phase = LoginPhase::CharactersReady;
    unchanged(&mut c, &delta(1, 1, b"WrongPhase\0"));
}

#[test]
fn malformed_directory_and_settings_packets_are_atomic() {
    let packets = [
        delta(1, 1, b"New\0"),
        delta(2, 1, &[4]),
        delta(3, 1, &[3, 0]),
        delta(4, 1, &[8, 0]),
        delta(5, 1, &[0, 0, 1]),
        configuration(1, "New", "1234"),
    ];
    for packet in packets {
        for n in 0..packet.len() {
            unchanged(&mut active(), &packet[..n]);
        }
        let mut tail = packet.clone();
        tail.push(0);
        unchanged(&mut active(), &tail);
    }
    for options in [
        [8, 0, 1, 0],
        [0, 3, 1, 0],
        [0, 0, 0, 1],
        [0, 0, 128, 0],
        [0, 0, 1, 32],
    ] {
        let mut body = configuration(1, "Bad", "");
        let at = body.len() - 5;
        body[at..at + 4].copy_from_slice(&options);
        unchanged(&mut active(), &body);
    }
    let mut wrong = configuration(2, "Foreign", "");
    unchanged(&mut active(), &wrong);
    wrong[2..6].fill(0);
    unchanged(&mut active(), &wrong);
    unchanged(&mut active(), &delta(1, 1, b"Embedded\0Nul\0"));
}

#[test]
fn valid_actions_with_failed_transport_do_not_optimistically_change_state() {
    let s = Settings {
        name: "Changed",
        password: "",
        attributes: 0,
        capacity: 8,
        options: None,
    };
    let mut c = pending(LoginPhase::LobbyReady);
    c.accept_game(&row(1, "Room")).unwrap();
    let before = serde_json::to_value(&c.state).unwrap();
    let deadline = c.deadline;
    assert!(c.create_room(&s).is_err());
    assert!(c.join_room(0, "1234").is_err());
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    assert_eq!(c.deadline, deadline);
    let mut c = active();
    let before = serde_json::to_value(&c.state).unwrap();
    assert!(c.edit_room(&s).is_err());
    assert!(c.kick_member("Peer(SourceB)").is_err());
    assert!(c
        .kick_member("Owner")
        .unwrap_err()
        .to_string()
        .contains("current peer"));
    assert!(c
        .kick_member("Peer")
        .unwrap_err()
        .to_string()
        .contains("current peer"));
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    assert!(c.kick_pending.is_none());
}

#[test]
fn encrypted_loopback_create_edit_kick_leave_password_retry() {
    use crate::modern_lito::{
        native_version,
        native_wire::{self, CipherState},
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
                match event {
                    Event::Packet { body, .. } => {
                        c.accept_game(&body).unwrap();
                    }
                    _ => {}
                }
            }
            assert!(
                Instant::now() < until,
                "loopback deadline: {:?}",
                c.state.phase
            );
            thread::sleep(Duration::from_millis(2));
        }
    }
    // Literal requests are independent of Settings::request; replies follow the
    // documented ClientSession ordering. This peer does not emulate GameHub rules.
    let mut kick = vec![0xd4];
    z(&mut kick, "Peer(SourceB)");
    let mut notice = vec![0xd6];
    z(&mut notice, "Peer(SourceB)");
    let script = vec![
        (
            b"\xccRoom\0\x041234\0\x08\0".to_vec(),
            vec![
                joined(1, "Room", "1234", "Owner"),
                member(1, "Owner"),
                member(1, "Peer(SourceB)"),
            ],
        ),
        (
            b"\xddEdited\0\0\0\x08\0".to_vec(),
            vec![
                summary(1, "Edited", "Owner"),
                configuration(1, "Edited", ""),
            ],
        ),
        (kick.clone(), vec![vec![0xfc, 1, 0, 0, 0]]),
        (
            kick,
            vec![
                vec![0xfc, 0, 0, 0, 0],
                notice,
                named(0xd0, 1, "Peer(SourceB)"),
            ],
        ),
        (vec![0xd0], vec![named(0xd0, 1, "Owner")]),
        (vec![0xce], vec![row(1, "Default")]),
        (vec![0xca, 0], vec![vec![0xca, 0, 1, 0, 200, 0]]),
        (b"\xcf\x01\0\0\0bad\0".to_vec(), vec![vec![0xce, 11]]),
        (
            b"\xcf\x01\0\0\x001234\0".to_vec(),
            vec![
                joined(1, "Room", "1234", "Owner"),
                member(1, "Owner"),
                member(1, "Peer(SourceB)"),
            ],
        ),
    ];
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
                    assert!(Instant::now() < until, "owned listener timeout");
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
        send(
            &mut socket,
            &mut tx,
            (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect(),
        );
        for (request, replies) in script {
            assert_eq!(read(&mut socket, &mut rx), request);
            for reply in replies {
                send(&mut socket, &mut tx, reply);
            }
        }
        release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        // The second join is now confirmed by the client. Exercise the kicked
        // recipient path separately from the requester's S_FC acknowledgement.
        let mut notice = vec![0xd6];
        z(&mut notice, "Owner");
        send(&mut socket, &mut tx, notice);
        send(&mut socket, &mut tx, named(0xd0, 1, "Owner"));
        assert_eq!(read(&mut socket, &mut rx), [0xce]);
        send(&mut socket, &mut tx, row(1, "Default"));
        assert_eq!(read(&mut socket, &mut rx), [0xca, 0]);
        send(&mut socket, &mut tx, vec![0xca, 0, 1, 0, 200, 0]);
        release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    });
    let mut c = pending(LoginPhase::LobbyReady);
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
    for capacity in [0, 1, 101, 200] {
        assert!(c
            .create_room(&Settings {
                name: "Invalid",
                password: "",
                attributes: 0,
                capacity,
                options: None,
            })
            .is_err());
        assert_eq!(c.state.phase, LoginPhase::LobbyReady);
    }
    c.create_room(&Settings {
        name: "Room",
        password: "1234",
        attributes: 4,
        capacity: 8,
        options: None,
    })
    .unwrap();
    assert!(c.state.joined_room.is_none());
    pump(&mut c, |c| {
        c.state.phase == LoginPhase::RoomReady
            && c.state
                .joined_room
                .as_ref()
                .is_some_and(|r| r.members.len() == 2)
    });
    c.edit_room(&Settings {
        name: "Edited",
        password: "",
        attributes: 0,
        capacity: 8,
        options: None,
    })
    .unwrap();
    assert_eq!(c.state.joined_room.as_ref().unwrap().name, "Room");
    pump(&mut c, |c| c.state.phase == LoginPhase::RoomReady);
    assert!(!c.state.joined_room.as_ref().unwrap().password_required);
    for accepted in [false, true] {
        c.kick_member("Peer(SourceB)").unwrap();
        assert_eq!(c.state.joined_room.as_ref().unwrap().members.len(), 2);
        pump(&mut c, |c| c.state.phase == LoginPhase::RoomReady);
        assert_eq!(c.state.room_kick_result, Some(accepted));
    }
    c.leave_room().unwrap();
    pump(&mut c, |c| c.state.phase == LoginPhase::LobbyReady);
    assert!(c.state.joined_room.is_none());
    assert_eq!(c.state.rooms[0].name, "Default");
    c.join_room(0, "bad").unwrap();
    pump(&mut c, |c| c.state.phase == LoginPhase::LobbyReady);
    assert_eq!(c.state.room_result, Some(11));
    assert!(c.state.joined_room.is_none());
    c.join_room(0, "1234").unwrap();
    pump(&mut c, |c| c.state.phase == LoginPhase::RoomReady);
    assert_eq!(c.state.room_result, Some(0));
    assert!(c.state.room_kick_result.is_none() && c.state.last_room_kick.is_none());
    release.send(()).unwrap();
    pump(&mut c, |c| c.state.phase == LoginPhase::LobbyReady);
    assert_eq!(c.state.last_room_kick.as_deref(), Some("Owner"));
    assert!(c.state.joined_room.is_none());
    release.send(()).unwrap();
    c.cancel();
    server.join().unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    while !c.is_quiescent() {
        c.poll();
        assert!(Instant::now() < until, "owned worker shutdown timeout");
        thread::sleep(Duration::from_millis(2));
    }
    assert!(c.is_quiescent());
}

#[test]
fn admin_timeout_retires_pending_authority_instead_of_inventing_success() {
    for phase in [
        LoginPhase::RoomCreating,
        LoginPhase::RoomJoining,
        LoginPhase::RoomAwaitSelf,
        LoginPhase::RoomEditing,
        LoginPhase::KickPending,
    ] {
        let mut c = active();
        c.state.phase = phase;
        c.kick_pending = Some((1, "Peer(SourceB)".into(), false));
        c.deadline = Some(Instant::now() - Duration::from_millis(1));
        c.poll();
        assert_eq!(c.state.phase, LoginPhase::Failed);
        assert!(c.state.joined_room.is_none() && c.kick_pending.is_none());
        assert!(c.requested_room.is_none() && c.deadline.is_none());
        assert!(c.is_quiescent());
    }
}

#[test]
fn joined_settings_reject_invalid_domains_before_accepting_membership() {
    for (offset, value) in [(8, 1), (8, 4), (12, 1), (12, 201), (14, 7)] {
        let mut body = joined(1, "R", "", "Owner");
        body[offset] = value;
        unchanged(&mut pending(LoginPhase::RoomCreating), &body);
    }
    let mut body = joined(1, "R", "", "Owner");
    let option_at = body.len() - 5;
    body[option_at] = 8;
    unchanged(&mut pending(LoginPhase::RoomCreating), &body);
    let mut body = joined(1, "R", "", "Owner");
    body[12] = 200;
    // The wire domain includes rental capacity 200, while ordinary outgoing
    // create/edit actions remain limited to the Taiwan native range 2..100.
    let mut c = pending(LoginPhase::RoomCreating);
    c.accept_game(&body).unwrap();
    assert_eq!(c.state.joined_room.as_ref().unwrap().capacity, 200);
}
