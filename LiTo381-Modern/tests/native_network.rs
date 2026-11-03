use lito_modern::modern_lito::{
    native_connection::{Controller, Event, Limits, Phase},
    native_version::{self, VersionReply},
    native_wire::{self, CipherState, EofState, FrameDecoder},
};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    sync::mpsc,
    time::{Duration, Instant},
};

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn contract() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("fixtures/native-version-current.json")).unwrap()
}

#[test]
fn deployed_jar_version_request_and_replies_match_typed_native_contract() {
    let data = contract();
    assert_eq!(data["supported"], true);
    assert_eq!(
        native_version::version_request(0).as_slice(),
        hex(data["request"].as_str().unwrap())
    );
    let accepted = hex(data["accepted"].as_str().unwrap());
    match native_version::parse_version(&accepted).unwrap() {
        VersionReply::Accepted(info) => {
            assert_eq!(info.revision, 0x0734fd33);
            assert_eq!(info.server_type, Some(0x77d82));
            assert_eq!(info.country, 3);
        }
        _ => panic!("accepted"),
    }
    assert_eq!(
        native_version::parse_version(&hex(data["rejected"].as_str().unwrap())).unwrap(),
        VersionReply::Rejected
    );
    for len in 0..accepted.len() {
        assert!(native_version::parse_version(&accepted[..len]).is_err());
    }
    let mut trailing = accepted.clone();
    trailing.push(1);
    assert!(native_version::parse_version(&trailing).is_err());
    let mut wrong = accepted;
    wrong[1] = 2;
    assert!(native_version::parse_version(&wrong).is_err());
}

#[test]
fn fragmented_and_coalesced_frames_preserve_full_u16_payload_and_eof_state() {
    let large: Vec<u8> = (0..native_wire::MAX_BODY).map(|i| i as u8).collect();
    let mut stream = native_wire::frame(&[1, 2, 3]).unwrap();
    stream.extend(native_wire::frame(&large).unwrap());
    stream.extend(native_wire::frame(&[]).unwrap());
    for chunk in [1, 2, 3, 4096, stream.len()] {
        let mut decoder = FrameDecoder::new(native_wire::MAX_FRAME).unwrap();
        let mut got = Vec::new();
        for bytes in stream.chunks(chunk) {
            let mut at = 0;
            while at < bytes.len() {
                let (used, frame) = decoder.feed(&bytes[at..]).unwrap();
                assert!(used > 0);
                at += used;
                if let Some(body) = frame {
                    got.push(body);
                }
            }
        }
        assert_eq!(got, vec![vec![1, 2, 3], large.clone(), vec![]]);
        assert_eq!(decoder.eof_state(), EofState::Clean);
    }
    assert!(native_wire::frame(&vec![0; native_wire::MAX_BODY + 1]).is_err());
    let mut invalid = FrameDecoder::new(32).unwrap();
    assert!(invalid.feed(&[33, 0]).is_err());
    assert!(invalid.feed(&[2, 0]).is_err());
    let mut partial = FrameDecoder::new(32).unwrap();
    partial.feed(&[5]).unwrap();
    assert_eq!(partial.eof_state(), EofState::PartialHeader);
    partial.feed(&[0, 1]).unwrap();
    assert_eq!(partial.eof_state(), EofState::PartialBody);
}

#[test]
fn shared_cipher_retains_unsigned_counter_carry_for_large_packets() {
    let vectors: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/formal-cipher-0.6.42.json")).unwrap();
    for sequence in vectors["sequences"].as_array().unwrap() {
        let seed = sequence["seed"].as_u64().unwrap() as u32;
        let mut tx = CipherState::new(seed);
        let mut rx = CipherState::new(seed);
        for packet in sequence["packets"].as_array().unwrap() {
            let plain = hex(packet["plain"].as_str().unwrap());
            let expected = hex(packet["cipher"].as_str().unwrap());
            let mut bytes = plain.clone();
            tx.encrypt(&mut bytes);
            assert_eq!(bytes, expected);
            rx.decrypt(&mut bytes);
            assert_eq!(bytes, plain);
            assert_eq!(
                tx.counter(),
                packet["after"].as_str().unwrap().parse::<u64>().unwrap()
            );
            assert_eq!(rx.counter(), tx.counter());
        }
        let mut large = vec![0x85; native_wire::MAX_BODY];
        let before = tx.counter();
        tx.encrypt(&mut large);
        rx.decrypt(&mut large);
        assert!(large.iter().all(|b| *b == 0x85));
        assert_eq!(
            tx.counter(),
            before.wrapping_add(native_wire::MAX_BODY as u64)
        );
        assert_eq!(rx.counter(), tx.counter());
    }
}

fn pump_until(controller: &mut Controller, predicate: impl Fn(&Controller) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !predicate(controller) {
        controller.poll();
        assert!(
            Instant::now() < deadline,
            "controller wait: {:?}",
            controller.snapshot()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn stalled_seed_can_cancel_and_reconnect_without_stale_owner_or_thread() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = listener.local_addr().unwrap();
    let (ready_tx, ready_rx) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        ready_tx.send(()).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut byte = [0];
        assert_eq!(stream.read(&mut byte).unwrap(), 0);
    });
    let mut client = Controller::default();
    let first = client.start(peer, Limits::default()).unwrap();
    ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let start = Instant::now();
    client.cancel();
    assert!(client.start(peer, Limits::default()).is_err());
    pump_until(&mut client, |c| c.is_quiescent());
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(client.snapshot().phase, Phase::Closed);
    assert!(client.snapshot().negotiated.is_none());
    server.join().unwrap();
    // A fresh failed connection gets a new epoch, not an accepted old owner.
    let unused = TcpListener::bind("127.0.0.1:0").unwrap();
    let absent = unused.local_addr().unwrap();
    drop(unused);
    let second = client.start(absent, Limits::default()).unwrap();
    assert_eq!(second, first + 1);
    pump_until(&mut client, |c| c.is_quiescent());
    assert_eq!(client.snapshot().phase, Phase::Failed);
    assert!(client.snapshot().negotiated.is_none());
}

#[test]
fn accepted_connection_keeps_socket_until_explicit_cancel_then_reuses_epoch() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = listener.local_addr().unwrap();
    let accepted = hex(contract()["accepted"].as_str().unwrap());
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        socket
            .write_all(&[7, 0, 1, 0xfe, 0xff, 0xff, 0xff])
            .unwrap();
        let mut request = [0; 12];
        socket.read_exact(&mut request).unwrap();
        let mut cipher = CipherState::new(0xfffffffe);
        let mut body = request[2..].to_vec();
        cipher.decrypt(&mut body);
        assert_eq!(body, native_version::version_request(0));
        let mut outgoing = CipherState::new(0xfffffffe);
        let mut response = accepted;
        outgoing.encrypt(&mut response);
        socket
            .write_all(&native_wire::frame(&response).unwrap())
            .unwrap();
        let mut byte = [0];
        assert_eq!(socket.read(&mut byte).unwrap(), 0);
    });
    let mut client = Controller::default();
    client
        .start(
            peer,
            Limits {
                read_chunk: 1,
                write_chunk: 3,
                ..Limits::default()
            },
        )
        .unwrap();
    pump_until(&mut client, |c| {
        c.snapshot().phase == Phase::VersionAccepted
    });
    assert!(!client.is_quiescent());
    let info = client.snapshot().negotiated.as_ref().unwrap();
    assert_eq!(
        info.rx_counter,
        native_wire::initial_counter(0xfffffffe) + 30
    );
    assert_eq!(
        info.tx_counter,
        native_wire::initial_counter(0xfffffffe) + 10
    );
    client.cancel();
    pump_until(&mut client, |c| c.is_quiescent());
    assert_eq!(client.snapshot().phase, Phase::Closed);
    server.join().unwrap();
}

#[test]
fn invalid_limits_and_non_loopback_cli_probe_do_not_start_worker() {
    let mut client = Controller::default();
    assert!(client
        .start(
            "127.0.0.1:1".parse::<SocketAddr>().unwrap(),
            Limits {
                max_frame: 31,
                ..Limits::default()
            }
        )
        .is_err());
    assert!(client.is_quiescent());
    assert!(lito_modern::modern_lito::native_connection::probe_loopback(
        "192.0.2.1:1".parse().unwrap()
    )
    .is_err());
}

#[test]
fn coalesced_post_version_packet_survives_peer_eof_and_owner_retirement() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = listener.local_addr().unwrap();
    let accepted = hex(contract()["accepted"].as_str().unwrap());
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        socket
            .write_all(&[7, 0, 1, 0xfe, 0xff, 0xff, 0xff])
            .unwrap();
        let mut request = [0; 12];
        socket.read_exact(&mut request).unwrap();
        let mut cipher = CipherState::new(0xfffffffe);
        let mut response = accepted;
        cipher.encrypt(&mut response);
        let mut bytes = native_wire::frame(&response).unwrap();
        let mut packet = vec![0x55, 0x80, 0xff];
        cipher.encrypt(&mut packet);
        bytes.extend(native_wire::frame(&packet).unwrap());
        socket.write_all(&bytes).unwrap();
    });
    let mut client = Controller::default();
    let epoch = client.start(peer, Limits::default()).unwrap();
    let mut packets = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !client.is_quiescent() {
        for event in client.poll() {
            if let Event::Packet {
                epoch: received,
                body,
                ..
            } = event
            {
                assert_eq!(received, epoch);
                packets.push(body);
            }
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    server.join().unwrap();
    assert_eq!(packets, vec![vec![0x55, 0x80, 0xff]]);
    assert_eq!(client.snapshot().phase, Phase::Failed);
    assert!(client
        .snapshot()
        .error
        .as_ref()
        .unwrap()
        .contains("EOF (Clean)"));
    assert_eq!(
        client.snapshot().negotiated.as_ref().unwrap().rx_counter,
        native_wire::initial_counter(0xfffffffe) + 33
    );
}

#[test]
fn full_event_queue_preserves_burst_and_allows_cancel_without_consumer() {
    for cancel_without_drain in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let peer = listener.local_addr().unwrap();
        let accepted = hex(contract()["accepted"].as_str().unwrap());
        let (ready_tx, ready_rx) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            socket
                .write_all(&[7, 0, 1, 0xfe, 0xff, 0xff, 0xff])
                .unwrap();
            let mut request = [0; 12];
            socket.read_exact(&mut request).unwrap();
            let mut cipher = CipherState::new(0xfffffffe);
            let mut response = accepted;
            cipher.encrypt(&mut response);
            let mut bytes = native_wire::frame(&response).unwrap();
            for i in 0..32u8 {
                let mut body = vec![0x55, i];
                cipher.encrypt(&mut body);
                bytes.extend(native_wire::frame(&body).unwrap());
            }
            socket.write_all(&bytes).unwrap();
            ready_tx.send(()).unwrap();
            let mut byte = [0];
            assert_eq!(socket.read(&mut byte).unwrap(), 0);
        });
        let mut client = Controller::default();
        client.start(peer, Limits::default()).unwrap();
        ready_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        std::thread::sleep(Duration::from_millis(80));
        let mut packets = Vec::new();
        if !cancel_without_drain {
            let deadline = Instant::now() + Duration::from_secs(3);
            while packets.len() < 32 {
                for event in client.poll() {
                    if let Event::Packet { body, .. } = event {
                        packets.push(body);
                    }
                }
                assert!(
                    Instant::now() < deadline,
                    "burst stalled: {:?}",
                    client.snapshot()
                );
                std::thread::sleep(Duration::from_millis(2));
            }
            assert_eq!(
                packets,
                (0..32u8).map(|i| vec![0x55, i]).collect::<Vec<_>>()
            );
            assert_eq!(
                client.snapshot().negotiated.as_ref().unwrap().rx_counter,
                native_wire::initial_counter(0xfffffffe) + 30 + 64
            );
        }
        let start = Instant::now();
        client.cancel();
        pump_until(&mut client, |c| c.is_quiescent());
        assert!(start.elapsed() < Duration::from_secs(1));
        assert_eq!(client.snapshot().phase, Phase::Closed);
        server.join().unwrap();
    }
}

#[test]
fn command_batch_is_admitted_atomically_and_written_as_two_ordered_frames() {
    use zeroize::Zeroizing;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = listener.local_addr().unwrap();
    let accepted = hex(contract()["accepted"].as_str().unwrap());
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        socket
            .write_all(&[7, 0, 1, 0xfe, 0xff, 0xff, 0xff])
            .unwrap();
        let mut request = [0; 12];
        socket.read_exact(&mut request).unwrap();
        let mut rx = CipherState::new(0xfffffffe);
        rx.decrypt(&mut request[2..]);
        assert_eq!(&request[2..], native_version::version_request(0));
        let mut tx = CipherState::new(0xfffffffe);
        let mut body = accepted;
        tx.encrypt(&mut body);
        socket
            .write_all(&native_wire::frame(&body).unwrap())
            .unwrap();
        for expected in [vec![0xcb], vec![0xca, 3], vec![0xd9]] {
            let mut header = [0; 2];
            socket.read_exact(&mut header).unwrap();
            let mut body = vec![0; usize::from(u16::from_le_bytes(header)) - 2];
            socket.read_exact(&mut body).unwrap();
            rx.decrypt(&mut body);
            assert_eq!(body, expected);
        }
        let mut byte = [0];
        assert_eq!(socket.read(&mut byte).unwrap(), 0);
    });
    let mut c = Controller::default();
    let epoch = c
        .start(
            peer,
            Limits {
                write_chunk: 1,
                ..Limits::default()
            },
        )
        .unwrap();
    pump_until(&mut c, |c| c.snapshot().phase == Phase::VersionAccepted);
    let queued=c.queued_tx_counter().unwrap();
    assert!(c.send_batch(epoch, vec![]).is_err());
    assert!(c
        .send_batch(
            epoch,
            vec![Zeroizing::new(vec![0xcb]), Zeroizing::new(vec![])]
        )
        .is_err());
    assert!(c
        .send_batch(epoch, vec![Zeroizing::new(vec![0]); 3])
        .is_err());
    assert_eq!(c.queued_tx_counter(),Some(queued));
    c.send_batch(
        epoch,
        vec![Zeroizing::new(vec![0xcb]), Zeroizing::new(vec![0xca, 3])],
    )
    .unwrap();
    c.send(epoch,Zeroizing::new(vec![0xd9])).unwrap();
    let quit_barrier=c.queued_tx_counter().unwrap();
    assert_eq!(quit_barrier,queued.wrapping_add(4));
    pump_until(&mut c, |c| {
        c.snapshot().negotiated.as_ref().unwrap().tx_counter
            == quit_barrier
    });
    c.cancel();
    pump_until(&mut c, |c| c.is_quiescent());
    server.join().unwrap();
}
