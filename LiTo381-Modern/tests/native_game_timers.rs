use lito_modern::modern_lito::{
    native_connection::{Controller, Event, Limits, Phase},
    native_game_timers::{Kind, Schedule, ALIVE_INTERVAL, SAVE_INTERVAL},
    native_version,
    native_wire::{self, CipherState},
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

#[test]
fn original_immediate_alive_and_300s_save_are_rearmed_from_actual_completion() {
    assert_eq!(ALIVE_INTERVAL, Duration::from_secs(60));
    assert_eq!(SAVE_INTERVAL, Duration::from_secs(300));
    assert_eq!((Kind::Alive.opcode(), Kind::Save.opcode()), (0x20, 0x1d));
    let t = Instant::now();
    let mut s = Schedule::default();
    assert!(s.due(t).is_empty());
    assert!(s.begin(t));
    assert!(!s.begin(t));
    assert_eq!(s.due(t), [Kind::Alive]);
    assert!(s.due(t).is_empty());
    let first = t + Duration::from_secs(7);
    s.sent(Kind::Alive, first);
    for second in [60, 120, 180, 240] {
        assert!(s
            .due(first + Duration::from_secs(second) - Duration::from_nanos(1))
            .is_empty());
        assert_eq!(s.due(first + Duration::from_secs(second)), [Kind::Alive]);
        s.sent(Kind::Alive, first + Duration::from_secs(second));
    }
    assert!(s
        .due(first + SAVE_INTERVAL - Duration::from_nanos(1))
        .is_empty());
    assert_eq!(s.due(first + SAVE_INTERVAL), [Kind::Save, Kind::Alive]);
    s.sent(Kind::Save, first + SAVE_INTERVAL);
    s.sent(Kind::Alive, first + SAVE_INTERVAL);
    assert_eq!((s.snapshot().alive_sent, s.snapshot().save_sent), (6, 1));
}
#[test]
fn delayed_dispatch_never_catches_up_and_stop_cannot_resurrect_the_owner() {
    let t = Instant::now();
    let mut s = Schedule::default();
    s.begin(t);
    assert_eq!(s.due(t), [Kind::Alive]);
    s.sent(Kind::Alive, t);
    let late = t + Duration::from_secs(3600);
    assert_eq!(s.due(late), [Kind::Alive, Kind::Save]);
    assert!(s.due(late + SAVE_INTERVAL).is_empty());
    s.sent(Kind::Alive, late);
    s.sent(Kind::Save, late + Duration::from_secs(3));
    assert!(s
        .due(late + ALIVE_INTERVAL - Duration::from_nanos(1))
        .is_empty());
    assert_eq!(s.due(late + ALIVE_INTERVAL), [Kind::Alive]);
    s.stop();
    s.sent(Kind::Alive, late + ALIVE_INTERVAL);
    assert!(!s.snapshot().enabled && !s.begin(late + Duration::from_secs(9000)));
    assert!(s.due(late + Duration::from_secs(10000)).is_empty());
}

fn read_body(socket: &mut TcpStream, rx: &mut CipherState) -> Vec<u8> {
    let mut header = [0; 2];
    socket.read_exact(&mut header).unwrap();
    let n = usize::from(u16::from_le_bytes(header));
    assert!((3..=native_wire::MAX_FRAME).contains(&n));
    let mut body = vec![0; n - 2];
    socket.read_exact(&mut body).unwrap();
    rx.decrypt(&mut body);
    body
}
fn version(socket: &mut TcpStream) -> (CipherState, CipherState) {
    let seed: u32 = 0xffff_fffe;
    let mut key = vec![1];
    key.extend(seed.to_le_bytes());
    socket
        .write_all(&native_wire::frame(&key).unwrap())
        .unwrap();
    let mut rx = CipherState::new(seed);
    assert_eq!(
        read_body(socket, &mut rx),
        native_version::version_request(0)
    );
    let v: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-version-current.json")).unwrap();
    let h = v["accepted"].as_str().unwrap();
    let mut reply: Vec<_> = (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect();
    let mut tx = CipherState::new(seed);
    tx.encrypt(&mut reply);
    socket
        .write_all(&native_wire::frame(&reply).unwrap())
        .unwrap();
    (rx, tx)
}
fn pump(c: &mut Controller, p: impl Fn(&Controller) -> bool) {
    let until = Instant::now() + Duration::from_secs(4);
    while !p(c) {
        c.poll();
        assert!(Instant::now() < until, "{:?}", c.snapshot());
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn admitted_timer_bytes_share_the_ordered_counter_and_final_quit_barrier() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = listener.local_addr().unwrap();
    let (started, first) = mpsc::channel();
    let (alive, observed) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(4)))
            .unwrap();
        let (mut rx, _) = version(&mut socket);
        // A version-accepted owner alone has no implicit timers.
        socket
            .set_read_timeout(Some(Duration::from_millis(150)))
            .unwrap();
        let mut one = [0];
        let e = socket.read(&mut one).unwrap_err();
        assert!(matches!(
            e.kind(),
            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
        ));
        started.send(()).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(4)))
            .unwrap();
        assert_eq!(read_body(&mut socket, &mut rx), [0x20]);
        alive.send(()).unwrap();
        for expected in [vec![0xcb], vec![0xca, 3], vec![0xd9]] {
            assert_eq!(read_body(&mut socket, &mut rx), expected);
        }
        assert_eq!(socket.read(&mut one).unwrap(), 0);
    });
    let mut c = Controller::default();
    assert!(c.start_game_timers(0).is_err());
    let epoch = c.start(peer, Limits::default()).unwrap();
    pump(&mut c, |c| c.snapshot().phase == Phase::VersionAccepted);
    first.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(!c.game_timers().enabled);
    let initial = c.queued_tx_counter().unwrap();
    assert!(c.start_game_timers(epoch + 1).is_err());
    assert_eq!(c.queued_tx_counter(), Some(initial));
    c.start_game_timers(epoch).unwrap();
    c.start_game_timers(epoch).unwrap();
    observed.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(c.queued_tx_counter(), Some(initial.wrapping_add(1)));
    c.send_batch(
        epoch,
        vec![Zeroizing::new(vec![0xcb]), Zeroizing::new(vec![0xca, 3])],
    )
    .unwrap();
    let barrier = c.send_final(epoch, Zeroizing::new(vec![0xd9])).unwrap();
    assert_eq!(barrier, initial.wrapping_add(5));
    assert!(c.send(epoch, Zeroizing::new(vec![0x20])).is_err());
    assert!(c.start_game_timers(epoch).is_err());
    assert!(!c.game_timers().enabled);
    let until = Instant::now() + Duration::from_secs(3);
    let mut receipt = false;
    while !receipt {
        for e in c.poll() {
            if let Event::Sent {
                epoch: e,
                tx_counter,
            } = e
            {
                receipt |= e == epoch && tx_counter == barrier;
            }
        }
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(2));
    }
    c.cancel();
    pump(&mut c, |c| c.is_quiescent());
    assert_eq!(c.snapshot().phase, Phase::Closed);
    assert!(!c.snapshot().game_timers.enabled);
    assert_eq!(c.snapshot().game_timers.alive_sent, 1);
    assert_eq!(c.snapshot().game_timers.save_sent, 0);
    server.join().unwrap();
}

#[test]
#[ignore = "Natural original 60s/300s intervals and no-poll/backpressure test; execute explicitly"]
fn natural_callbacks_continue_with_a_full_inbound_event_queue_and_no_UI_polling() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = listener.local_addr().unwrap();
    let (sent, received) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(320)))
            .unwrap();
        let (mut rx, mut tx) = version(&mut socket);
        let start = Instant::now();
        let mut rows = Vec::new();
        assert_eq!(read_body(&mut socket, &mut rx), [0x20]);
        rows.push((0x20, start.elapsed().as_millis()));
        for _ in 0..12 {
            let mut b = vec![0x21, 0, 0, 0, 0];
            tx.encrypt(&mut b);
            socket.write_all(&native_wire::frame(&b).unwrap()).unwrap();
        }
        while rows.len() < 7 {
            let b = read_body(&mut socket, &mut rx);
            assert!(matches!(b.as_slice(), [0x20] | [0x1d]));
            rows.push((b[0], start.elapsed().as_millis()));
        }
        assert_eq!(
            rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            [0x20, 0x20, 0x20, 0x20, 0x20, 0x1d, 0x20]
        );
        let times: Vec<_> = rows.iter().filter(|r| r.0 == 0x20).map(|r| r.1).collect();
        for pair in times.windows(2) {
            assert!((59_900..=62_000).contains(&(pair[1] - pair[0])));
        }
        assert!((299_900..=303_000).contains(&rows[5].1));
        sent.send(rows).unwrap();
        let mut end = [0];
        // Cancelling while deliberately retaining unread inbound frames can
        // close the peer with FIN or a Winsock reset. All payload/deadline
        // checks above must finish before either terminal outcome is accepted.
        match socket.read(&mut end) {
            Ok(0) => {}
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
            other => panic!("unexpected post-cancel socket outcome: {other:?}"),
        }
    });
    let mut c = Controller::default();
    let epoch = c.start(peer, Limits::default()).unwrap();
    pump(&mut c, |c| c.snapshot().phase == Phase::VersionAccepted);
    c.start_game_timers(epoch).unwrap();
    // No transport polling during the entire five-minute original callback run.
    let rows = received.recv_timeout(Duration::from_secs(315)).unwrap();
    assert_eq!(rows.len(), 7);
    let until = Instant::now() + Duration::from_secs(2);
    while c.game_timers().alive_sent != 6 || c.game_timers().save_sent != 1 {
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(2));
    }
    let stats = c.game_timers();
    assert_eq!((stats.alive_sent, stats.save_sent), (6, 1));
    c.cancel();
    pump(&mut c, |c| c.is_quiescent());
    server.join().unwrap();
}

#[test]
#[ignore = "Requires owned C02 LOGIN/GAME fixture and original 60s/300s intervals; execute explicitly"]
fn formal_character_admission_natural_callbacks_cancel_and_reauthenticate() {
    use lito_modern::modern_lito::native_login::{LoginController, LoginPhase};
    let peer = std::env::var("LITO_GAME_TIMER_LOGIN")
        .expect("owned fixture LOGIN endpoint")
        .parse()
        .unwrap();
    let out =
        std::path::PathBuf::from(std::env::var_os("LITO_GAME_TIMER_OUTPUT").expect("owned output"));
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap();
    assert!(out.canonicalize().unwrap().starts_with(&workspace));
    let mut c = LoginController::default();
    let mut results = Vec::new();
    let mut prior_epoch = 0;
    for round in 0..2 {
        c.start(
            peer,
            Limits {
                read_chunk: 2,
                write_chunk: 3,
                ..Limits::default()
            },
        )
        .unwrap();
        let started = Instant::now();
        let mut submitted = false;
        loop {
            c.poll();
            assert!(
                started.elapsed() < Duration::from_secs(15),
                "bootstrap {:?}",
                c.snapshot().phase
            );
            if c.snapshot()
                .notice
                .as_ref()
                .is_some_and(|n| n.acknowledgement().is_some())
            {
                c.acknowledge_notice().unwrap();
            }
            match c.snapshot().phase {
                LoginPhase::ReadyForLogin if !submitted => {
                    c.login("PeerFixture", "TestOnlyPassword").unwrap();
                    submitted = true;
                }
                LoginPhase::SourcesReady => c.select_source(1).unwrap(),
                LoginPhase::RedirectReady => c.begin_game().unwrap(),
                LoginPhase::CharactersReady => break,
                LoginPhase::Failed | LoginPhase::Closed => panic!("{:?}", c.snapshot().error),
                _ => {}
            }
            std::thread::sleep(Duration::from_millis(3));
        }
        let epoch = c.transport().snapshot().epoch;
        assert!(epoch > prior_epoch);
        prior_epoch = epoch;
        let unadmitted = Instant::now();
        let quiet = if round == 0 {
            Duration::from_secs(61)
        } else {
            Duration::from_millis(250)
        };
        while unadmitted.elapsed() < quiet {
            c.poll();
            assert_eq!(c.snapshot().phase, LoginPhase::CharactersReady);
            let t = c.transport().game_timers();
            assert!(!t.enabled && t.alive_sent == 0 && t.save_sent == 0);
            std::fs::write(out.join("progress.json"),serde_json::to_vec(&serde_json::json!({"round":round,"phase":"authenticated_before_character_admission","elapsed_ms":unadmitted.elapsed().as_millis(),"epoch":epoch,"timers":t})).unwrap()).unwrap();
            std::thread::sleep(Duration::from_millis(50));
        }
        let unadmitted_ms = unadmitted.elapsed().as_millis();
        c.select_character(0).unwrap();
        let admitted = Instant::now();
        let mut selected = false;
        loop {
            c.poll();
            let phase = c.snapshot().phase;
            assert!(
                !matches!(phase, LoginPhase::Failed | LoginPhase::Closed),
                "{:?}",
                c.snapshot().error
            );
            if phase == LoginPhase::CharacterSelected {
                selected = true;
            }
            let t = c.transport().game_timers();
            let done = if round == 0 {
                t.alive_sent >= 6 && t.save_sent == 1
            } else {
                t.alive_sent >= 2 && t.save_sent == 0
            };
            std::fs::write(out.join("progress.json"),serde_json::to_vec(&serde_json::json!({"round":round,"phase":"admitted_original_intervals","elapsed_ms":admitted.elapsed().as_millis(),"epoch":epoch,"timers":t,"login_phase":phase})).unwrap()).unwrap();
            if selected && done {
                assert!(
                    admitted.elapsed()
                        >= if round == 0 {
                            Duration::from_secs(300)
                        } else {
                            Duration::from_secs(60)
                        }
                );
                results.push(serde_json::json!({"round":round,"epoch":epoch,"unadmitted_ms":unadmitted_ms,"admitted_ms":admitted.elapsed().as_millis(),"timers":t}));
                break;
            }
            assert!(
                admitted.elapsed()
                    < if round == 0 {
                        Duration::from_secs(315)
                    } else {
                        Duration::from_secs(75)
                    },
                "original interval timeout"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        c.cancel();
        let deadline = Instant::now() + Duration::from_secs(4);
        while !c.is_quiescent() {
            c.poll();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(3));
        }
        c.poll();
        assert_eq!(c.snapshot().phase, LoginPhase::Closed);
        assert!(!c.transport().game_timers().enabled);
    }
    std::fs::write(out.join("client-result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"formal_timers_natural_intervals_and_new_GAME_epoch_verified","rounds":results,"quiescent":c.is_quiescent(),"calendar_locally_predicted":false})).unwrap()).unwrap();
}
