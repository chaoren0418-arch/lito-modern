//! Owned loopback only: decrypt real framed packets through the production worker.
use super::*;
use std::net::TcpListener;

#[test]
fn static2941_ingress_survives_burst_backpressure_with_packet_identity() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let peer = listener.local_addr().unwrap();
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-version-current.json"
    ))
    .unwrap();
    let hex = fixture["accepted"].as_str().unwrap();
    let accepted: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    let (sent, ready) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let end = Instant::now() + Duration::from_secs(5);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    assert!(Instant::now() < end, "loopback connect timeout");
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(e) => panic!("{e}"),
            }
        };
        socket.set_nonblocking(false).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        socket
            .write_all(&[7, 0, 1, 0xfe, 0xff, 0xff, 0xff])
            .unwrap();
        socket.read_exact(&mut [0; 12]).unwrap();
        let mut cipher = CipherState::new(0xfffffffe);
        let mut response = accepted;
        cipher.encrypt(&mut response);
        let mut bytes = native_wire::frame(&response).unwrap();
        for i in 0..32_u8 {
            let mut body = vec![0x37, i, 0, 0, 0, 0x7d, 0x0b];
            cipher.encrypt(&mut body);
            bytes.extend(native_wire::frame(&body).unwrap());
        }
        socket.write_all(&bytes).unwrap();
        sent.send(()).unwrap();
        // Keep the socket owned until the client cancels after draining.
        assert_eq!(socket.read(&mut [0]).unwrap(), 0);
    });
    let mut client = Controller::default();
    let epoch = client.start(peer, Limits::default()).unwrap();
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    // Receiver has 8 slots; the burst necessarily exercises backpressure.
    std::thread::sleep(Duration::from_millis(80));
    let mut receipts = Vec::new();
    let end = Instant::now() + Duration::from_secs(5);
    while receipts.len() < 32 {
        for event in client.poll() {
            if let Event::Packet {
                epoch: observed,
                body,
                rx_counter,
                receipt,
            } = event
            {
                assert_eq!(observed, epoch);
                assert_eq!(receipt.connection_epoch(), epoch);
                assert_eq!(receipt.rx_counter(), rx_counter);
                assert_eq!(
                    rx_counter,
                    native_wire::initial_counter(0xfffffffe) + 30 + 7 * (receipts.len() as u64 + 1)
                );
                assert_eq!(body, [0x37, receipts.len() as u8, 0, 0, 0, 0x7d, 0x0b]);
                assert!(receipt.decoded_at() <= Instant::now());
                if let Some(previous) = receipts.last() {
                    let previous: &IngressReceipt = previous;
                    assert!(previous.decoded_at() <= receipt.decoded_at());
                }
                receipts.push(receipt);
            }
        }
        assert!(Instant::now() < end, "ingress burst stalled");
        std::thread::sleep(Duration::from_millis(2));
    }
    client.cancel();
    while !client.is_quiescent() {
        client.poll();
        assert!(Instant::now() < end, "ingress worker did not retire");
        std::thread::sleep(Duration::from_millis(2));
    }
    server.join().unwrap();
    assert_eq!(receipts.len(), 32);
}
