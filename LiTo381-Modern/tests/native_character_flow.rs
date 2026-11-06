//! Synthetic encrypted LOGIN/GAME peers on owned ephemeral loopback ports.
//! Exercises the production controller; no Java, database, assets or game UI.
use lito_modern::modern_lito::{
    native_connection::Limits,
    native_game::pages::{Pages, Turn},
    native_login::{self, LoginController, LoginPhase},
    native_version,
    native_wire::{self, CipherState},
};
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(10);

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn fixture(bytes: &[u8], path: &[&str]) -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    for key in path {
        value = value[*key].clone();
    }
    hex(value.as_str().unwrap())
}

struct Peer {
    socket: TcpStream,
    rx: CipherState,
    tx: CipherState,
}

impl Peer {
    fn accept(listener: TcpListener) -> Self {
        listener.set_nonblocking(true).unwrap();
        let until = Instant::now() + TIMEOUT;
        let socket = loop {
            match listener.accept() {
                Ok((socket, address)) => {
                    assert!(address.ip().is_loopback());
                    break socket;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < until, "owned accept timeout");
                    thread::sleep(Duration::from_millis(2));
                }
                Err(e) => panic!("owned accept: {e}"),
            }
        };
        socket.set_nonblocking(false).unwrap();
        socket.set_read_timeout(Some(TIMEOUT)).unwrap();
        socket.set_write_timeout(Some(TIMEOUT)).unwrap();
        socket.set_nodelay(true).unwrap();
        let seed = 0x12345678u32;
        let mut peer = Self {
            socket,
            rx: CipherState::new(seed),
            tx: CipherState::new(seed),
        };
        let mut key = vec![1];
        key.extend(seed.to_le_bytes());
        peer.socket
            .write_all(&native_wire::frame(&key).unwrap())
            .unwrap();
        peer.expect(&native_version::version_request(0));
        peer.send(fixture(
            include_bytes!("fixtures/native-version-current.json"),
            &["accepted"],
        ));
        peer
    }

    fn expect(&mut self, expected: &[u8]) {
        let mut header = [0; 2];
        self.socket.read_exact(&mut header).unwrap();
        let length = usize::from(u16::from_le_bytes(header));
        assert!((3..=1024).contains(&length));
        let mut body = vec![0; length - 2];
        self.socket.read_exact(&mut body).unwrap();
        self.rx.decrypt(&mut body);
        assert_eq!(body, expected, "unexpected client request");
    }

    fn send(&mut self, mut body: Vec<u8>) {
        self.tx.encrypt(&mut body);
        self.socket
            .write_all(&native_wire::frame(&body).unwrap())
            .unwrap();
    }

    fn directory(&mut self, count: u8) {
        self.send(vec![3, count]);
        let row = fixture(
            include_bytes!("fixtures/native-game-current.json"),
            &["packets", "row"],
        );
        let name_end = row[1..].iter().position(|b| *b == 0).unwrap() + 1;
        for index in 0..count {
            let mut body = vec![0xcb];
            body.extend_from_slice(
                &native_login::encode_text(&format!("角色{}", index + 1), 27).unwrap(),
            );
            body.push(0);
            body.extend_from_slice(&row[name_end + 1..]);
            self.send(body);
        }
    }

    fn expect_login(&mut self) {
        self.expect(
            &native_login::login_request("測試帳號", "FixtureOnly", Ipv4Addr::LOCALHOST).unwrap(),
        );
    }
}

fn endpoint(body: &mut Vec<u8>, address: SocketAddr) {
    let SocketAddr::V4(address) = address else {
        panic!()
    };
    assert!(address.ip().is_loopback());
    body.extend(address.ip().octets());
    body.extend(address.port().to_le_bytes());
}

fn sources(first: SocketAddr, second: SocketAddr) -> Vec<u8> {
    let mut body = vec![0x3b, 2];
    for (name, address) in [("測試甲", first), ("測試乙", second)] {
        body.extend_from_slice(&native_login::encode_text(name, 63).unwrap());
        body.push(0);
        endpoint(&mut body, address);
    }
    body
}

fn pump(c: &mut LoginController, ready: impl Fn(&LoginController) -> bool) {
    let until = Instant::now() + TIMEOUT;
    loop {
        c.poll();
        if ready(c) {
            return;
        }
        assert!(
            !matches!(c.snapshot().phase, LoginPhase::Failed | LoginPhase::Closed),
            "unexpected controller failure: {:?}",
            c.snapshot().error
        );
        assert!(
            Instant::now() < until,
            "controller timeout: {:?}",
            c.snapshot().phase
        );
        thread::sleep(Duration::from_millis(2));
    }
}

fn stop(c: &mut LoginController) {
    c.cancel();
    let until = Instant::now() + TIMEOUT;
    while !c.is_quiescent() {
        c.poll();
        assert!(Instant::now() < until, "owned worker cleanup timeout");
        thread::sleep(Duration::from_millis(2));
    }
    c.poll();
    assert_eq!(c.snapshot().phase, LoginPhase::Closed);
}

#[test]
fn second_source_eight_characters_second_page_refusal_retry_and_return() {
    let login_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let login_address = login_listener.local_addr().unwrap();
    let game_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let game_address = game_listener.local_addr().unwrap();
    let (finish_tx, finish_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let mut login = Peer::accept(login_listener);
        login.expect_login();
        login.send(vec![0xe7]);
        login.send(sources(login_address, game_address));
        login.expect(&native_login::source_request("測試乙").unwrap());
        // The reply's index is not the source-list index (existing fixture contract).
        let mut redirect = vec![0xf4, 0];
        endpoint(&mut redirect, game_address);
        login.send(redirect);
        let mut game = Peer::accept(game_listener);
        drop(login);
        game.expect_login();
        game.send(vec![0xe6]);
        game.expect(&[0xe2]);
        game.directory(0); // stale initial directory must not be published
        game.expect(&native_login::source_request("測試乙").unwrap());
        game.directory(8);
        let select = |name| {
            let mut body = vec![0xc9];
            body.extend_from_slice(&native_login::encode_text(name, 27).unwrap());
            body.push(0);
            body
        };
        game.expect(&select("角色8"));
        game.send(vec![0xc8, 7]);
        game.expect(&select("角色5"));
        game.send(vec![0xc8, 0]);
        game.expect(&[0x20]); // production GAME entry starts its immediate Alive callback
        game.expect(&[0xca, 100]);
        game.send(vec![0xf5]);
        game.expect(&native_login::source_request("測試乙").unwrap());
        game.directory(1);
        finish_rx.recv_timeout(TIMEOUT).unwrap();
    });
    let mut c = LoginController::default();
    c.start(
        login_address,
        Limits {
            read_chunk: 2,
            write_chunk: 3,
            ..Limits::default()
        },
    )
    .unwrap();
    pump(&mut c, |c| c.snapshot().phase == LoginPhase::ReadyForLogin);
    c.login("測試帳號", "FixtureOnly").unwrap();
    pump(&mut c, |c| c.snapshot().phase == LoginPhase::SourcesReady);
    assert_eq!(c.snapshot().sources.len(), 2);
    assert!(c.select_source(2).is_err());
    c.select_source(1).unwrap();
    assert!(c.select_source(0).is_err());
    pump(&mut c, |c| c.snapshot().phase == LoginPhase::RedirectReady);
    assert_eq!(c.snapshot().selected_source.as_deref(), Some("測試乙"));
    assert_eq!(c.snapshot().redirect, Some(game_address));
    let login_epoch = c.transport().snapshot().epoch;
    c.begin_game().unwrap();
    pump(&mut c, |c| {
        c.snapshot().phase == LoginPhase::CharactersReady
    });
    let epoch = c.transport().snapshot().epoch;
    assert_ne!(login_epoch, epoch);
    assert_eq!(c.snapshot().characters.len(), 8);
    let mut pages = Pages::default();
    pages.sync(epoch, c.snapshot());
    pages.turn(Turn::Next);
    let selected = pages.index(epoch, c.snapshot(), 3).unwrap();
    assert_eq!(selected, 7);
    c.select_character(selected).unwrap();
    assert_eq!(c.snapshot().selected_character.as_deref(), Some("角色8"));
    assert!(c.select_character(0).is_err());
    assert!(pages.index(epoch, c.snapshot(), 0).is_err());
    pump(&mut c, |c| {
        c.snapshot().phase == LoginPhase::CharactersReady
    });
    pages.sync(epoch, c.snapshot());
    assert_eq!(c.snapshot().character_result, Some(7));
    assert!(c.snapshot().selected_character.is_none());
    assert_eq!(pages.range(), 4..8);
    c.select_character(pages.index(epoch, c.snapshot(), 0).unwrap())
        .unwrap();
    pump(&mut c, |c| {
        c.snapshot().phase == LoginPhase::CharacterSelected
    });
    assert_eq!(c.snapshot().selected_character.as_deref(), Some("角色5"));
    let generation = c.snapshot().directory_generation;
    c.return_characters().unwrap();
    pump(&mut c, |c| {
        c.snapshot().phase == LoginPhase::CharactersReady
    });
    assert!(c.snapshot().directory_generation > generation);
    assert!(pages.index(epoch, c.snapshot(), 0).is_err());
    pages.sync(epoch, c.snapshot());
    assert_eq!(pages.range(), 0..1);
    assert_eq!(c.snapshot().characters[0].name, "角色1");
    assert!(c.snapshot().selected_character.is_none());
    stop(&mut c);
    finish_tx.send(()).unwrap();
    server.join().unwrap();
}

#[test]
fn mismatched_redirect_cannot_start_game_handoff() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let unused = TcpListener::bind("127.0.0.1:0").unwrap();
    let wrong = unused.local_addr().unwrap();
    let (finish_tx, finish_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let mut login = Peer::accept(listener);
        login.expect_login();
        login.send(vec![0xe7]);
        login.send(sources(address, wrong));
        login.expect(&native_login::source_request("測試甲").unwrap());
        let mut redirect = vec![0xf4, 0];
        endpoint(&mut redirect, wrong);
        login.send(redirect);
        finish_rx.recv_timeout(TIMEOUT).unwrap();
    });
    let mut c = LoginController::default();
    c.start(address, Limits::default()).unwrap();
    pump(&mut c, |c| c.snapshot().phase == LoginPhase::ReadyForLogin);
    c.login("測試帳號", "FixtureOnly").unwrap();
    pump(&mut c, |c| c.snapshot().phase == LoginPhase::SourcesReady);
    c.select_source(0).unwrap();
    pump(&mut c, |c| c.snapshot().phase == LoginPhase::RedirectReady);
    let epoch = c.transport().snapshot().epoch;
    assert!(c
        .begin_game()
        .unwrap_err()
        .to_string()
        .contains("differs from selected advertised endpoint"));
    assert_eq!(c.transport().snapshot().epoch, epoch);
    assert!(!c.snapshot().game_connected);
    unused.set_nonblocking(true).unwrap();
    assert_eq!(
        unused.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    stop(&mut c);
    finish_tx.send(()).unwrap();
    server.join().unwrap();
}
