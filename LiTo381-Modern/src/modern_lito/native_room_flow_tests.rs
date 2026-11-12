//! G06.2 client-side contracts using offline packets and an owned encrypted peer.
use super::room_admin_tests::{active, delta, info, member, named, row, summary, unchanged, z};
use super::*;
use crate::modern_lito::native_room_ui;

fn update(kind: u8, value: &[u8]) -> Vec<u8> {
    let mut body = info(kind, 1);
    body.extend(value);
    body
}

fn participant(op: u8, name: &str, value: u8) -> Vec<u8> {
    let mut body = vec![op];
    z(&mut body, name);
    body.push(value);
    body
}

fn system(text: &str) -> Vec<u8> {
    let mut body = vec![0x0f, 9];
    body.extend_from_slice(&encode_text(text, 512).unwrap());
    body.push(0);
    body
}

fn world_packet(name: &str) -> Vec<u8> {
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-world-current.json"
    ))
    .unwrap();
    let hex = fixture["packets"][name].as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

fn bootstrap(c: &mut LoginController, observer: bool) {
    c.accept_game(&[0xe5, u8::from(observer)]).unwrap();
    // Offline unit fixtures have no authenticated transport/environment epoch.
    // The encrypted test below exercises controller WORLD admission with one.
    c.state.world.accept(&[0x28, 0x34, 2, 0]).unwrap();
    c.accept_game(&[0xd2]).unwrap();
    c.state.world.accept(&world_packet("status")).unwrap();
    c.accept_game(&[0xe2, 0xc0, 0x7f, 0, 0x80, 0, 0x80, 0x40, 0x80])
        .unwrap();
}

#[test]
fn mode_waits_for_its_mode_then_map_and_rejects_foreign_room() {
    let mut c = active();
    c.state.phase = LoginPhase::ModePending;
    c.deadline = Some(Instant::now() + Duration::from_secs(8));
    let deadline = c.deadline;
    c.accept_game(&update(3, &554u16.to_le_bytes())).unwrap();
    assert_eq!(c.state.phase, LoginPhase::ModePending);
    assert_eq!(c.deadline, deadline);
    let mut wrong = info(2, 2);
    wrong.push(2);
    unchanged(&mut c, &wrong);
    assert!(!c.room_mode_received);
    c.accept_game(&update(2, &[2])).unwrap();
    assert_eq!(c.state.phase, LoginPhase::ModePending);
    assert!(c.room_mode_received);
    assert_eq!(c.deadline, deadline);
    c.accept_game(&update(3, &554u16.to_le_bytes())).unwrap();
    assert_eq!(c.state.phase, LoginPhase::RoomReady);
    assert!(c.deadline.is_none() && !c.room_mode_received);
    let room = c.state.joined_room.as_ref().unwrap();
    assert_eq!((room.mode, room.map_id, room.members.len()), (2, 554, 2));
}

#[test]
fn mode_refusal_summary_restores_authority_without_guessing_success() {
    for mode in 0..=6 {
        let mut c = active();
        c.state.joined_room.as_mut().unwrap().mode = mode;
        c.state.phase = LoginPhase::ModePending;
        c.deadline = Some(Instant::now() + Duration::from_secs(8));
        let mut body = info(0, 1);
        z(&mut body, "Room");
        body.push(mode);
        body.extend(564u16.to_le_bytes());
        body.push(0);
        z(&mut body, "Owner");
        body.extend([0; 4]);
        c.accept_game(&body).unwrap();
        assert_eq!(c.state.phase, LoginPhase::RoomReady);
        assert_eq!(c.state.joined_room.as_ref().unwrap().mode, mode);
        assert!(c.deadline.is_none() && !c.room_mode_received);
    }
}

#[test]
fn ready_and_team_wait_for_exact_self_and_accept_authoritative_refusal() {
    for (phase, opcode) in [
        (LoginPhase::ReadyPending, 0xd1),
        (LoginPhase::ForcePending, 0xd4),
    ] {
        let mut c = active();
        c.state.phase = phase;
        c.deadline = Some(Instant::now() + Duration::from_secs(8));
        let deadline = c.deadline;
        c.accept_game(&participant(opcode, "Peer(SourceB)", 1))
            .unwrap();
        assert_eq!(c.state.phase, phase);
        assert_eq!(c.deadline, deadline);
        unchanged(&mut c, &participant(opcode, "Peer", 1));
        unchanged(&mut c, &participant(opcode, "Owner", 3));
        // A refused request may echo the existing value, which still completes it.
        c.accept_game(&participant(opcode, "Owner", 0)).unwrap();
        assert_eq!(c.state.phase, LoginPhase::RoomReady);
        assert!(c.deadline.is_none());
        let own = &c.state.joined_room.as_ref().unwrap().members[0];
        assert_eq!((own.force, own.state), (0, 0));
    }
}

#[test]
fn readiness_never_synthesizes_countdown_and_server_can_cancel_it() {
    let mut c = active();
    c.accept_game(&participant(0xd1, "Owner", 1)).unwrap();
    c.accept_game(&participant(0xd1, "Peer(SourceB)", 1))
        .unwrap();
    assert_eq!(c.state.joined_room.as_ref().unwrap().countdown, None);
    assert_eq!(c.state.phase, LoginPhase::RoomReady);
    for value in [3, 2, 1] {
        c.accept_game(&[0xe1, value]).unwrap();
        assert!(native_room_ui::status(&c.state).contains(&format!("倒數 {value}")));
        assert_eq!(c.state.phase, LoginPhase::RoomReady);
        assert!(!c.state.world.started);
    }
    c.accept_game(&participant(0xd1, "Peer(SourceB)", 2))
        .unwrap();
    c.accept_game(&[0xe1, 0]).unwrap();
    assert!(!native_room_ui::status(&c.state).contains("倒數"));
    assert_eq!(c.state.phase, LoginPhase::RoomReady);
    // Shop/list-entry state 2 is retained and cannot create a battle locally.
    assert_eq!(c.state.joined_room.as_ref().unwrap().members[1].state, 2);
}

#[test]
fn chinese_room_rules_feedback_is_visible_without_changing_membership_or_waits() {
    let mut c = active();
    c.state.phase = LoginPhase::ReadyPending;
    c.deadline = Some(Instant::now() + Duration::from_secs(8));
    let deadline = c.deadline;
    let room = c.state.joined_room.clone();
    for message in [
        "無法開始：單人測試請選紅隊，ArenaBot 位於藍隊。",
        "無法開始：至少需要 2 位參戰玩家，目前 1 位（觀戰者與假人不計）。",
        "無法開始：紅隊與藍隊都必須有參戰玩家。",
        "無法開始：此房間要求雙方人數一致。",
        "無法開始：攻城模式每隊需要 15～50 位參戰玩家。",
        "Local Lobby 為測試房，不計勝敗戰績；死亡競賽單人測試會加入 ArenaBot。",
    ] {
        c.accept_game(&system(message)).unwrap();
        assert_eq!(c.state.room_system_message.as_deref(), Some(message));
        assert!(native_room_ui::status(&c.state).contains(message));
        assert_eq!(c.state.joined_room, room);
        assert_eq!(c.state.phase, LoginPhase::ReadyPending);
        assert_eq!(c.deadline, deadline);
    }
    let mut unrelated = system("其他頻道訊息");
    unrelated[1] = 1;
    let previous = c.state.room_system_message.clone();
    c.accept_game(&unrelated).unwrap();
    assert_eq!(c.state.room_system_message, previous);
    c.cancel();
    assert!(c.state.room_system_message.is_none());
}

#[test]
fn bot_directory_count_uses_wire_values_and_roster_uses_display_identity() {
    let mut c = active();
    c.accept_game(&row(1, "Local Lobby")).unwrap();
    for users in [1u16, 2, 2, 0, 200] {
        c.accept_game(&delta(3, 1, &users.to_le_bytes())).unwrap();
        assert_eq!(c.state.rooms[0].users, users);
        assert_eq!(c.state.joined_room.as_ref().unwrap().members.len(), 2);
    }
    c.accept_game(&member(1, "ArenaBot")).unwrap();
    c.accept_game(&participant(0xd4, "ArenaBot", 1)).unwrap();
    c.accept_game(&participant(0xd1, "ArenaBot", 1)).unwrap();
    assert_eq!(c.state.joined_room.as_ref().unwrap().members[2].force, 1);
    c.accept_game(&named(0xd0, 1, "ArenaBot")).unwrap();
    assert_eq!(c.state.joined_room.as_ref().unwrap().members.len(), 2);
    assert_eq!(c.state.rooms[0].users, 200);
}

#[test]
fn room_names_do_not_suppress_settlement_or_rewrite_statistics() {
    for name in ["Local Lobby", "Player Room"] {
        let mut c = active();
        c.accept_game(&summary(1, name, "Owner")).unwrap();
        let stats = c.state.joined_room.as_ref().unwrap().members.clone();
        bootstrap(&mut c, true);
        c.accept_game(&[0xd3, 0, 0, 0, 0]).unwrap();
        let mut score = vec![0xdb];
        z(&mut score, "Owner");
        for value in [100u32, 2, 1, 0] {
            score.extend(value.to_le_bytes());
        }
        score.extend([0, 0]);
        c.accept_game(&score).unwrap();
        assert_eq!(c.state.phase, LoginPhase::BattleResult);
        assert_eq!(c.state.world.scores[0].points, 100);
        assert_eq!(c.state.joined_room.as_ref().unwrap().members, stats);
    }
}

#[test]
fn observer_camera_is_local_and_active_departure_cannot_settle_other_players() {
    let mut c = active();
    bootstrap(&mut c, true);
    let tx = c.transport.queued_tx_counter();
    let room = c.state.joined_room.clone();
    let actors = serde_json::to_value(&c.state.world.actors).unwrap();
    assert!(c.state.world.view_ready() && !c.state.world.ready());
    assert!(c.state.world.own_actor().is_none());
    assert!(c.pan_observer(0).unwrap());
    assert!(c.move_step(0).is_err());
    assert!(c.attack(1).is_err());
    assert!(c.set_ready(true).is_err());
    assert!(c.leave_room().is_err());
    assert!(c.return_from_battle().is_err());
    assert!(c
        .request_battle_withdrawal()
        .unwrap_err()
        .to_string()
        .contains("伺服器支援"));
    assert_eq!(c.state.phase, LoginPhase::BattlePending);
    assert_eq!(c.state.joined_room, room);
    assert_eq!(serde_json::to_value(&c.state.world.actors).unwrap(), actors);
    assert_eq!(c.transport.queued_tx_counter(), tx);
    assert!(!c.state.battle_withdrawal_pending);
}

#[test]
fn rejected_send_preserves_room_state_message_and_deadline() {
    let mut c = active();
    c.accept_game(&system("尚未開始")).unwrap();
    let before = serde_json::to_value(&c.state).unwrap();
    let deadline = c.deadline;
    assert!(c.set_force(1).is_err());
    assert!(c.step_room_mode(1).is_err());
    assert!(c.select_room_map(564).is_err());
    assert!(c.set_ready(true).is_err());
    assert!(c.set_observer(true).is_err());
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    assert_eq!(c.deadline, deadline);
    assert!(!c.room_mode_received);
}

#[test]
fn pending_map_keeps_room_page_and_cannot_release_pending_mode() {
    let mut c = active();
    c.state.phase = LoginPhase::MapPending;
    assert!(native_room_ui::room_page(&c.state));
    c.accept_game(&update(2, &[2])).unwrap();
    assert_eq!(c.state.phase, LoginPhase::MapPending);
    c.accept_game(&update(3, &554u16.to_le_bytes())).unwrap();
    assert_eq!(c.state.phase, LoginPhase::RoomReady);
    assert!(!c.room_mode_received);
    c.state.phase = LoginPhase::MapPending;
    c.deadline = Some(Instant::now() + Duration::from_secs(8));
    c.accept_game(&summary(1, "Room", "Owner")).unwrap();
    assert_eq!(c.state.phase, LoginPhase::RoomReady);
    assert!(c.deadline.is_none());
}

#[test]
fn timed_out_room_requests_retire_mode_tracking_messages_and_observer_return() {
    for phase in [
        LoginPhase::ModePending,
        LoginPhase::MapPending,
        LoginPhase::ForcePending,
        LoginPhase::ReadyPending,
        LoginPhase::ObserverPending,
    ] {
        let mut c = active();
        c.accept_game(&system("先前的房間訊息")).unwrap();
        c.state.phase = phase;
        c.room_mode_received = true;
        c.observer_return = Some(LoginPhase::RoomReady);
        c.deadline = Some(Instant::now() - Duration::from_millis(1));
        c.poll();
        assert_eq!(c.state.phase, LoginPhase::Failed);
        assert!(c.state.joined_room.is_none() && c.state.room_system_message.is_none());
        assert!(!c.room_mode_received && c.observer_return.is_none());
        assert!(c.deadline.is_none() && c.is_quiescent());
    }
}

#[test]
fn malformed_room_system_messages_preserve_snapshot() {
    let mut c = active();
    c.accept_game(&system("已有訊息")).unwrap();
    unchanged(&mut c, &[0x0f, 9]);
    unchanged(&mut c, &[0x0f, 9, 0]);
    let mut trailing = system("無法開始");
    trailing.push(0);
    unchanged(&mut c, &trailing);
}

#[test]
fn encrypted_loopback_modes_teams_readiness_observer_and_settlement_return() {
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
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready(c) {
            for event in c.transport.poll() {
                if let Event::Packet { body, .. } = event {
                    c.accept_game(&body).unwrap();
                }
            }
            assert!(
                Instant::now() < deadline,
                "loopback timeout {:?}",
                c.state.phase
            );
            thread::sleep(Duration::from_millis(2));
        }
    }

    // Literal requests independently check production encoders. Replies are a
    // fixed script, not an execution of server rules or any production service.
    let modes = [
        (5, 510u16),
        (2, 554),
        (4, 540),
        (3, 573),
        (0, 565),
        (1, 564),
    ];
    let mut script = vec![
        (vec![0xe1, 1], vec![vec![0xe5, 1]]),
        (vec![0xe1, 0], vec![vec![0xe5, 0]]),
        (vec![0xd3, 1], vec![participant(0xd4, "Owner", 0)]),
        (
            vec![0xd3, 1],
            vec![participant(0xd4, "Owner", 1), vec![0xe1, 0]],
        ),
        (
            vec![0xd2, 1],
            vec![participant(0xd1, "Owner", 1), vec![0xe1, 3]],
        ),
        (
            vec![0xd2, 0],
            vec![participant(0xd1, "Owner", 0), vec![0xe1, 0]],
        ),
    ];
    for (mode, map) in modes {
        script.push((
            vec![0xd1, 2, 1],
            vec![
                participant(0xd1, "Peer(SourceB)", 0),
                update(2, &[mode]),
                update(3, &map.to_le_bytes()),
            ],
        ));
    }
    script.push((vec![0xd1, 2, 2], vec![summary(1, "Room", "Owner")]));
    script.push((
        vec![0xd1, 3, 0x34, 2],
        vec![update(3, &564u16.to_le_bytes())],
    ));
    script.push((
        vec![0xe1, 1],
        vec![
            vec![0xe5, 1],
            vec![0x28, 0x34, 2, 0],
            vec![0xd2],
            world_packet("status"),
            vec![0xe2, 0xc0, 0x7f, 0, 0x80, 0, 0x80, 0x40, 0x80],
            vec![0xd3, 0, 0, 0, 0],
        ],
    ));
    script.push((vec![0xd9], vec![vec![0xf0, 0]]));
    script.push((vec![0xd0], vec![named(0xd0, 1, "Owner")]));
    script.push((vec![0xce], vec![row(1, "Local Lobby")]));
    script.push((vec![0xca, 0], vec![vec![0xca, 0, 1, 0, 200, 0]]));

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let (done, completion) = mpsc::channel();
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline);
                    thread::sleep(Duration::from_millis(2));
                }
                Err(error) => panic!("owned listener {error}"),
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
        let version: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/native-version-current.json"
        ))
        .unwrap();
        let hex = version["accepted"].as_str().unwrap();
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
        completion.recv_timeout(Duration::from_secs(10)).unwrap();
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
    pump(&mut c, |c| {
        c.transport.snapshot().phase == Phase::VersionAccepted
    });
    c.state
        .environment
        .begin(c.transport.snapshot().epoch)
        .unwrap();
    for observer in [true, false] {
        c.set_observer(observer).unwrap();
        assert_ne!(c.state.observer, observer);
        assert_eq!(c.state.phase, LoginPhase::ObserverPending);
        pump(&mut c, |c| c.state.phase == LoginPhase::RoomReady);
        assert_eq!(c.state.observer, observer);
    }
    for expected in [0, 1] {
        c.accept_game(&participant(0xd1, "Owner", 1)).unwrap();
        c.accept_game(&[0xe1, 3]).unwrap();
        c.set_force(1).unwrap();
        assert_eq!(c.state.joined_room.as_ref().unwrap().members[0].force, 0);
        pump(&mut c, |c| {
            c.state.phase == LoginPhase::RoomReady
                && (expected == 0 || c.state.joined_room.as_ref().unwrap().countdown == Some(0))
        });
        assert_eq!(
            c.state.joined_room.as_ref().unwrap().members[0].force,
            expected
        );
    }
    for ready in [true, false] {
        c.accept_game(&system("先前拒絕訊息")).unwrap();
        c.set_ready(ready).unwrap();
        assert!(c.state.room_system_message.is_none());
        assert_eq!(c.state.phase, LoginPhase::ReadyPending);
        pump(&mut c, |c| {
            c.state.phase == LoginPhase::RoomReady
                && c.state.joined_room.as_ref().unwrap().countdown
                    == Some(if ready { 3 } else { 0 })
        });
        assert_eq!(
            c.state.joined_room.as_ref().unwrap().members[0].state,
            u8::from(ready)
        );
    }
    for (mode, map) in modes {
        c.accept_game(&participant(0xd1, "Peer(SourceB)", 1))
            .unwrap();
        let before = c.state.joined_room.clone();
        c.step_room_mode(1).unwrap();
        assert_eq!(c.state.joined_room, before);
        pump(&mut c, |c| c.state.phase == LoginPhase::RoomReady);
        let room = c.state.joined_room.as_ref().unwrap();
        assert_eq!((room.mode, room.map_id), (mode, map));
    }
    c.step_room_mode(2).unwrap();
    pump(&mut c, |c| c.state.phase == LoginPhase::RoomReady);
    assert_eq!(c.state.joined_room.as_ref().unwrap().mode, 1);
    c.select_room_map(564).unwrap();
    assert!(native_room_ui::room_page(&c.state));
    pump(&mut c, |c| c.state.phase == LoginPhase::RoomReady);
    c.set_observer(true).unwrap();
    pump(&mut c, |c| c.state.phase == LoginPhase::BattleResult);
    assert!(c.state.world.view_ready());
    let generation = c.state.world.generation;
    c.return_from_battle().unwrap();
    assert_eq!(c.state.phase, LoginPhase::ResultReturning);
    assert!(c.return_from_battle().is_err());
    pump(&mut c, |c| c.state.phase == LoginPhase::LobbyReady);
    assert!(c.state.joined_room.is_none() && c.state.world.observer_camera.is_none());
    assert!(!c.state.world.view_ready());
    assert_eq!(c.state.world.generation, generation);
    assert!(c.state.room_system_message.is_none());
    done.send(()).unwrap();
    c.cancel();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !c.is_quiescent() && Instant::now() < deadline {
        c.poll();
        thread::sleep(Duration::from_millis(2));
    }
    server.join().unwrap();
    assert!(c.is_quiescent());
}
