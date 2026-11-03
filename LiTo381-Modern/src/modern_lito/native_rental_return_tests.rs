//! Adversarial return packets must not retire a live rental world or extend its deadline.
use super::*;

fn returning(quit_pending: bool) -> LoginController {
    let mut c = LoginController::default();
    c.state.phase = LoginPhase::ReturningCharacters;
    c.state.selected_character = Some("ReturnFixture".into());
    c.state.rental_set = Some(0);
    c.state.rental_restart_acks = 1;
    c.rental_restart_quit_pending = quit_pending;
    c.state.joined_room = Some(super::super::native_room::Room {
        id: 60001, name: "Rental".into(), attributes: 0, password_required: false,
        users_summary: 1, capacity: 200, mode: 6, map_id: 500, summary: [0; 5],
        owner: "ReturnFixture".into(), options: [0; 4], selections: vec![],
        members: vec![], countdown: None,
    });
    c.deadline = Some(Instant::now() + Duration::from_secs(8));
    c
}

fn departure(room: u32, name: &str) -> Vec<u8> {
    let mut body = vec![0xd0];
    body.extend(room.to_le_bytes());
    body.extend(name.as_bytes());
    body.push(0);
    body
}

fn rejected_without_mutation(c: &mut LoginController, body: &[u8], reason: &str) {
    let before = serde_json::to_value(&c.state).unwrap();
    let deadline = c.deadline;
    let pending = c.rental_restart_quit_pending;
    let error = c.accept_game(body).expect_err("invalid return packet accepted");
    assert!(error.to_string().contains(reason), "unexpected error: {error:#}");
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before);
    assert_eq!(c.deadline, deadline);
    assert_eq!(c.rental_restart_quit_pending, pending);
}

#[test]
fn quit_before_restart_is_rejected() {
    let mut c = returning(false);
    c.state.phase = LoginPhase::RentalRestartPending;
    c.state.rental_restart_acks = 0;
    rejected_without_mutation(&mut c, &[0xf0, 0], "rental quit acknowledgement");
}

#[test]
fn duplicate_and_malformed_quit_are_rejected() {
    for body in [vec![0xf0], vec![0xf0, 1], vec![0xf0, 0, 0]] {
        rejected_without_mutation(&mut returning(true), &body, "rental quit acknowledgement");
    }
    rejected_without_mutation(&mut returning(false), &[0xf0, 0], "rental quit acknowledgement");
}

#[test]
fn repeated_restart_is_rejected() {
    for body in [vec![0x65], vec![0x65, 0]] {
        rejected_without_mutation(&mut returning(true), &body, "rental restart acknowledgement");
    }
}

#[test]
fn departure_before_quit_cannot_retire_room() {
    rejected_without_mutation(&mut returning(true), &departure(60001, "ReturnFixture"),
        "rental departure preceded quit acknowledgement");
}

#[test]
fn source_refresh_requires_quit_and_own_departure() {
    rejected_without_mutation(&mut returning(true), &[0xf5], "source refresh preceded rental quit");
    rejected_without_mutation(&mut returning(false), &[0xf5], "source refresh preceded rental departure");
}

#[test]
fn own_departure_after_quit_retires_only_once() {
    let mut c = returning(false);
    let deadline = c.deadline;
    let body = departure(60001, "ReturnFixture");
    c.accept_game(&body).unwrap();
    assert!(c.state.joined_room.is_none());
    assert_eq!(c.state.phase, LoginPhase::ReturningCharacters);
    assert_eq!(c.state.rental_set, Some(0));
    assert_eq!(c.deadline, deadline);
    // Repeated own departures are rejected by the existing membership contract.
    rejected_without_mutation(&mut c, &body, "departure without current or known retired room");
    assert!(c.state.joined_room.is_none());
    assert_eq!(c.deadline, deadline);
}


fn reconnect_fixture(peer: std::net::SocketAddr) -> LoginController {
    let mut c=returning(true);
    c.login_peer=Some(peer);c.credentials=Some((Zeroizing::new("OwnedReturn".into()),Zeroizing::new("TestOnlyPassword".into())));
    c.state.selected_source=Some("測試乙".into());c.state.game_connected=true;
    c.state.world.generation=17;c.state.world.map_id=Some(500);
    c.deadline=Some(Instant::now()-Duration::from_millis(1));c
}
fn owned_listener()->std::net::TcpListener {
    let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();listener
}
#[test]
fn unacknowledged_return_timeout_cannot_open_login_or_keep_credentials() {
    let listener=owned_listener();let mut c=reconnect_fixture(listener.local_addr().unwrap());
    c.state.rental_restart_acks=0;c.poll();
    assert_eq!(c.snapshot().phase,LoginPhase::Failed);assert!(c.credentials.is_none());
    assert_eq!(c.transport.snapshot().epoch,0);
    assert_eq!(listener.accept().unwrap_err().kind(),std::io::ErrorKind::WouldBlock);
    assert!(c.snapshot().joined_room.is_none() && c.snapshot().rooms.is_empty());
    assert!(c.snapshot().world.map_id.is_none() && c.snapshot().selected_character.is_none());
    assert!(c.snapshot().rental_set.is_none() && c.snapshot().services.is_none());
    assert_eq!(c.snapshot().world.generation,17);
}
#[test]
fn missing_source_return_timeout_cannot_connect_an_owned_login_peer() {
    let listener=owned_listener();let mut c=reconnect_fixture(listener.local_addr().unwrap());
    c.state.selected_source=None;c.poll();
    assert_eq!(c.snapshot().phase,LoginPhase::Failed);assert!(c.credentials.is_none());
    assert!(c.rental_return_reconnect.is_none());assert_eq!(c.transport.snapshot().epoch,0);
    assert_eq!(listener.accept().unwrap_err().kind(),std::io::ErrorKind::WouldBlock);
    assert!(c.snapshot().joined_room.is_none() && c.snapshot().rooms.is_empty());
    assert!(c.snapshot().world.map_id.is_none() && c.snapshot().selected_character.is_none());
    assert!(c.snapshot().rental_set.is_none() && c.snapshot().services.is_none());
    assert_eq!(c.snapshot().world.generation,17);
}
#[test]
fn cancellation_after_return_retirement_prevents_login_and_drops_credentials() {
    let listener=owned_listener();let mut c=reconnect_fixture(listener.local_addr().unwrap());
    c.begin_rental_return_reconnect().unwrap();
    assert!(c.snapshot().world.map_id.is_none()&&c.snapshot().joined_room.is_none());
    assert_eq!(c.snapshot().world.generation,17);
    c.cancel();c.poll();
    assert!(c.is_quiescent()&&c.credentials.is_none()&&c.rental_return_reconnect.is_none());
    assert_eq!(c.transport.snapshot().epoch,0);
    assert_eq!(listener.accept().unwrap_err().kind(),std::io::ErrorKind::WouldBlock);
}
#[test]
fn acknowledged_return_timeout_retires_world_before_one_owned_login_connection() {
    let listener=owned_listener();let mut c=reconnect_fixture(listener.local_addr().unwrap());c.poll();
    assert!(c.snapshot().world.map_id.is_none()&&c.snapshot().joined_room.is_none());
    assert!(c.snapshot().selected_character.is_none()&&c.snapshot().inventory.items.is_empty());
    assert_eq!(c.snapshot().world.generation,17);assert_eq!(c.transport.snapshot().epoch,1);
    let end=Instant::now()+Duration::from_secs(3);
    let socket=loop {match listener.accept(){Ok((socket,_))=>break socket,Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>{assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(5));},Err(e)=>panic!("{e}")}};
    for _ in 0..8{c.poll();std::thread::sleep(Duration::from_millis(2));}
    assert_eq!(c.transport.snapshot().epoch,1);
    assert_eq!(listener.accept().unwrap_err().kind(),std::io::ErrorKind::WouldBlock);
    c.cancel();let end=Instant::now()+Duration::from_secs(3);
    while !c.is_quiescent(){assert!(Instant::now()<end);c.poll();std::thread::sleep(Duration::from_millis(5));}
    assert!(c.credentials.is_none()&&c.rental_return_reconnect.is_none());drop(socket);
}

#[test]
fn cancel_live_world_retires_snapshot_without_losing_generation() {
    let listener=owned_listener();
    let mut c=reconnect_fixture(listener.local_addr().unwrap());
    c.state.phase=LoginPhase::WorldReady;
    c.rental_restart_quit_pending=false;
    c.state.world.learned_skills.insert(43);
    c.state.world.mp=Some([70,110]);
    assert!(c.teleport.is_none()&&c.recovery_character.is_none());
    c.cancel();c.poll();
    assert!(c.is_quiescent()&&c.credentials.is_none());
    assert!(c.state.world.map_id.is_none()&&c.state.world.mp.is_none());
    assert!(c.state.world.learned_skills.is_empty()&&c.state.world.effects.is_empty());
    assert!(c.state.joined_room.is_none()&&c.state.selected_character.is_none());
    assert!(c.state.services.is_none()&&c.state.inventory.owner.is_none());
    assert!(!c.state.game_connected&&c.state.rental_set.is_none());
    assert_eq!(c.state.world.generation,17);
    assert_eq!(c.transport.snapshot().epoch,0);
    assert_eq!(listener.accept().unwrap_err().kind(),std::io::ErrorKind::WouldBlock);
}
