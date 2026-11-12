//! Owned encrypted loopback + actual map/SPR composition. No production endpoint.
use super::*;
use crate::modern_lito::{
    native_connection::IngressReceipt,
    native_version,
    native_wire::{self, CipherState},
    native_world_scene::movement_tests,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

struct Peer {
    socket: TcpStream,
    rx: CipherState,
    tx: CipherState,
}
impl Peer {
    fn read(&mut self) -> Vec<u8> {
        let mut header = [0; 2];
        self.socket.read_exact(&mut header).unwrap();
        let len = usize::from(u16::from_le_bytes(header));
        assert!((3..=native_wire::MAX_FRAME).contains(&len));
        let mut body = vec![0; len - 2];
        self.socket.read_exact(&mut body).unwrap();
        self.rx.decrypt(&mut body);
        body
    }
    fn send(&mut self, mut body: Vec<u8>) {
        self.tx.encrypt(&mut body);
        self.socket
            .write_all(&native_wire::frame(&body).unwrap())
            .unwrap();
    }
}
fn connected() -> (LoginController, Peer) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut c = LoginController::default();
    c.transport
        .start(listener.local_addr().unwrap(), Limits::default())
        .unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    let socket = loop {
        match listener.accept() {
            Ok((socket, _)) => break socket,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < until);
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
    let mut peer = Peer {
        socket,
        rx: CipherState::new(seed),
        tx: CipherState::new(seed),
    };
    let mut key = vec![1];
    key.extend(seed.to_le_bytes());
    peer.socket
        .write_all(&native_wire::frame(&key).unwrap())
        .unwrap();
    assert_eq!(peer.read(), native_version::version_request(0));
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-version-current.json"
    ))
    .unwrap();
    let hex = fixture["accepted"].as_str().unwrap();
    peer.send(
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect(),
    );
    while c.transport.snapshot().phase != Phase::VersionAccepted {
        c.transport.poll();
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(2));
    }
    c.state.phase = LoginPhase::BattlePending;
    c.state.world = movement_tests::world();
    let epoch = c.transport.snapshot().epoch;
    c.state.world.learned_skills.insert(14);
    c.state.environment.begin(epoch).unwrap();
    c.state
        .environment
        .map_changed(epoch, c.state.world.map_id.unwrap(), Some(0))
        .unwrap();
    c.state
        .environment
        .water_poll(epoch, Some(c.state.world.generation))
        .unwrap();
    c.presentation_callbacks = true;
    (c, peer)
}
fn root() -> std::path::PathBuf {
    std::env::var_os("LITO_STATIC2941_ROOT")
        .expect("explicit source root")
        .into()
}
fn body(c: &LoginController, graphics: u16) -> Vec<u8> {
    let mut body = vec![0x37];
    body.extend(c.state.world.own_actor().unwrap().id.to_le_bytes());
    body.extend(graphics.to_le_bytes());
    body
}
fn close(c: &mut LoginController) {
    c.transport.cancel();
    let until = Instant::now() + Duration::from_secs(10);
    while !c.transport.is_quiescent() {
        c.transport.poll();
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn rejected_send_does_not_arm_the_s37_169_branch() {
    let mut c = LoginController::default();
    c.state.world = movement_tests::world();
    assert!(c.send_player_action(vec![0x14, 12, 0]).is_err());
    assert!(!c.player_callbacks.action_sent);
}

#[test]
#[ignore = "LITO_STATIC2941_ROOT; actual map/SPR composition and owned loopback"]
fn controller_move_callback_composes_real_source_frames_and_enqueues_static_before_return() {
    use crate::modern_lito::native_static2941::CallbackHost;
    let (mut c, mut peer) = connected();
    let root = root();
    let mut view = Presentation::default();
    let epoch = c.transport.snapshot().epoch;
    view.callback_draw(&root, epoch, &c.state.world, &c.state.environment)
        .unwrap();
    // The protocol fixture's spawn is on a native map edge. Select a real
    // passable westward edge in the loaded geometry, never replace navigation.
    let actor = c.state.world.own_actor().unwrap();
    let (id, x, y) = (actor.id, i32::from(actor.x), i32::from(actor.y));
    let navigation = &view.frame.as_ref().unwrap().navigation;
    let (x, y) = ((y - 32)..=(y + 32))
        .flat_map(|y| ((x - 32)..=(x + 32)).map(move |x| (x, y)))
        .find(|&(x, y)| navigation.can_step(x, y, 6))
        .expect("native fixture has a passable direction-6 edge");
    let actor = c.state.world.actors.get_mut(&id).unwrap();
    actor.x = u16::try_from(x).unwrap();
    actor.y = u16::try_from(y).unwrap();
    c.state.world.visual_revision += 1;
    view.callback_draw(&root, epoch, &c.state.world, &c.state.environment)
        .unwrap();
    let frame = view.frame.as_ref().unwrap();
    assert!(frame.player_walk_plan.is_some());
    let expected = c.state.world.move_request(6).unwrap();
    let before = crate::modern_lito::native_clock::sample().unwrap();
    c.move_presented(6, frame).unwrap();
    assert_eq!(peer.read(), expected);
    // Initial source frame is composed through the production renderer host.
    {
        let mut driver = DrawHost {
            root: &root,
            view: &mut view,
            environment: &c.state.environment,
            transport: &mut c.transport,
        };
        let mut clock = || Ok(before);
        let mut host = player::Host {
            control: &mut c.player_callbacks,
            driver: &mut driver,
            clock: &mut clock,
        };
        host.run(&mut c.static2941, &mut c.state.world, epoch, false)
            .unwrap();
    }
    let packet = body(&c, 2941);
    let pending_request =
        crate::modern_lito::native_skills::request(&c.state.world, 14, None).unwrap();
    let tx_before = c.transport.queued_tx_counter();
    c.cast(14, None, view.frame.as_ref().unwrap()).unwrap();
    assert_eq!(
        c.transport.queued_tx_counter(),
        tx_before,
        "pending action sent before frame zero"
    );
    c.accept_game_received(&packet, Some(IngressReceipt::for_test(epoch, 1)))
        .unwrap();
    let lease = c
        .state
        .world
        .static_effect_receipts
        .values()
        .last()
        .unwrap()
        .lease()
        .unwrap()
        .clone();
    assert!(lease.waiting());
    let cutoff = before.wrapping_add(10000);
    {
        let mut driver = DrawHost {
            root: &root,
            view: &mut view,
            environment: &c.state.environment,
            transport: &mut c.transport,
        };
        let mut clock = || Ok(cutoff.wrapping_add(1));
        let mut host = player::Host {
            control: &mut c.player_callbacks,
            driver: &mut driver,
            clock: &mut clock,
        };
        c.static2941
            .drain_with(&mut c.state.world, epoch, cutoff, &mut host)
            .unwrap();
        assert!(
            lease.drawable(),
            "draw enqueue used a fresh sample beyond the fixed outer cutoff"
        );
        assert_eq!(
            lease.schedule().unwrap().deadline_ms,
            cutoff.wrapping_add(1)
        );
        assert!(host
            .driver
            .view
            .frame
            .as_ref()
            .unwrap()
            .bgra
            .iter()
            .any(|b| *b != 0));
        let retired = c
            .static2941
            .drain_with(&mut c.state.world, epoch, cutoff.wrapping_add(1), &mut host)
            .unwrap();
        assert_eq!(retired, [lease.sequence()]);
        assert!(!host
            .driver
            .view
            .frame
            .as_ref()
            .unwrap()
            .static_current(&c.state.world, epoch));
        // A subsequent composition really removes the retired fragments.
        host.draw(&mut c.static2941, &mut c.state.world, epoch)
            .unwrap();
        assert!(host
            .driver
            .view
            .frame
            .as_ref()
            .unwrap()
            .static_current(&c.state.world, epoch));
    }
    assert!(c.state.world.player_walk.as_ref().unwrap().complete);
    assert_eq!(peer.read(), pending_request);
    assert!(c.player_callbacks.action_sent);
    close(&mut c);
}

#[test]
#[ignore = "LITO_STATIC2941_ROOT; actual map/SPR composition and owned loopback"]
fn controller_sent_action_and_accepted_169_execute_the_real_nested_draw_pump() {
    let (mut c, mut peer) = connected();
    let root = root();
    let epoch = c.transport.snapshot().epoch;
    let mut view = Presentation::default();
    c.state.world.learned_skills.insert(14);
    view.callback_draw(&root, epoch, &c.state.world, &c.state.environment)
        .unwrap();
    let request = crate::modern_lito::native_skills::request(&c.state.world, 14, None).unwrap();
    c.cast(14, None, view.frame.as_ref().unwrap()).unwrap();
    assert_eq!(peer.read(), request);
    let packet = body(&c, 169);
    c.game = true;
    peer.send(packet);
    let until = Instant::now() + Duration::from_secs(20);
    while !c.player_callbacks.pump_completed {
        c.poll_presented(&root, &mut view);
        assert!(c.state.error.is_none(), "{:?}", c.state.error);
        assert!(Instant::now() < until, "169 production pump timeout");
        thread::sleep(Duration::from_millis(2));
    }
    assert!(view.frame.as_ref().unwrap().input_ready(&c.state.world));
    close(&mut c);
}
