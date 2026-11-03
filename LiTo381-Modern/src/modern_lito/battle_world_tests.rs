use super::*;
fn data() -> payload::UnvalidatedData {
    let mut d = payload::data_tests::fixture();
    d.presentation = Some(d::List(
        d.actors
            .0
            .iter()
            .map(|a| d::PublicPresentation {
                entity: a.entity,
                shot: None,
                item: None,
            })
            .collect(),
    ));
    d
}
fn request() -> Request {
    Request::read(
        app::SelectionContext {
            key: app::CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: 1u64 << 63,
        },
        0,
        Some(&data().world.identity),
    )
}
fn response(
    q: &Request,
    state: State,
    data: Option<&payload::UnvalidatedData>,
) -> (Vec<u8>, Vec<u8>) {
    let expected = encode_request(9, q, VERSION).unwrap();
    let mut b = expected.clone();
    b[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
    b.extend([0, 0, state as u8, 0]);
    if state == State::Detached {
        b.extend([0; 48]);
    } else {
        for n in [101i64, 102, 103] {
            b.extend(n.to_be_bytes());
        }
        b.extend(data.map_or(1, |d| d.world.revision + 1).to_be_bytes());
        b.extend([data.map_or(0, |d| d.world.phase as u8 + 1), 0, 0, 0]);
        b.extend([0; 12]);
    }
    let payload = data
        .map(|d| payload::encode(d).unwrap())
        .unwrap_or_default();
    b.extend((payload.len() as u32).to_be_bytes());
    b.extend(payload);
    let n = b.len() as u32;
    b[4..8].copy_from_slice(&n.to_be_bytes());
    (expected, b)
}
fn parse(q: &Request, state: State, d: Option<&payload::UnvalidatedData>) -> Reply {
    let (expected, bytes) = response(q, state, d);
    read_reply(&mut bytes.as_slice(), &expected, VERSION).unwrap()
}
#[test]
fn detached_prepare_data_loaded_and_leave_keep_distinct_authority() {
    let q = request();
    let empty = Request::read(q.context, q.channel, None);
    let detached = parse(&empty, State::Detached, None);
    assert!(detached.data.is_none() && detached.attachment.is_none());
    let prepare = Request {
        action: Action::Prepare,
        room: 101,
        room_revision: 102,
        match_id: 0,
        generation: 0,
        ..q.clone()
    };
    let started = parse(&prepare, State::BeforeEngine, None);
    assert!(started.data.is_none());
    assert_eq!(
        started.attachment.unwrap().lifecycle.phase,
        world::Phase::Preparing
    );
    let d = data();
    let loading = parse(&q, State::Data, Some(&d));
    assert_eq!(loading.data, Some(d.clone()));
    assert_eq!(
        loading.attachment.unwrap().lifecycle.phase,
        world::Phase::Loading
    );
    let mut loaded = d.clone();
    loaded.actors.0[0].loaded = true;
    let ql = Request {
        action: Action::Loaded,
        digest: d.manifest.digest,
        ..q.clone()
    };
    assert!(
        parse(&ql, State::Data, Some(&loaded))
            .data
            .unwrap()
            .actors
            .0[0]
            .loaded
    );
    let leave = Request {
        action: Action::Leave,
        ..q
    };
    assert_eq!(parse(&leave, State::Detached, None).state, State::Detached);
    assert_eq!(format!("{detached:?}"), "BattleWorldReply[redacted]");
    assert_eq!(protocol::RUNTIME_WORLD_VERSION, 37);
}
#[test]
fn preparation_failure_is_an_attached_abort_without_fake_data_or_load_ack() {
    let q = request();
    let (expected, mut bytes) = response(&q, State::BeforeEngine, None);
    bytes[REQUEST_BYTES + 28..REQUEST_BYTES + 36].copy_from_slice(&2i64.to_be_bytes());
    bytes[REQUEST_BYTES + 36] = 4;
    bytes[REQUEST_BYTES + 37] = 7;
    let failed = read_reply(&mut bytes.as_slice(), &expected, VERSION).unwrap();
    assert!(failed.data.is_none());
    assert_eq!(
        failed.attachment.unwrap().lifecycle.abort,
        world::AbortReason::PreparationFailed
    );
    let loaded = Request {
        action: Action::Loaded,
        digest: [1; 32],
        ..q
    };
    let (expected, bytes) = response(&loaded, State::BeforeEngine, None);
    assert!(read_reply(&mut bytes.as_slice(), &expected, VERSION).is_err());
}
#[test]
fn no_old_version_or_invalid_request_can_enter_the_new_channel() {
    let q = request();
    let (expected, bytes) = response(&q, State::Data, Some(&data()));
    for v in [1, 35, 37, 38, protocol::MAX_VERSION + 1] {
        assert!(encode_request(9, &q, v).is_err());
        assert!(read_reply(&mut bytes.as_slice(), &expected, v).is_err());
    }
    for n in 0..8 {
        let mut bad = q.clone();
        match n {
            0 => bad.channel = 4,
            1 => bad.room = 0,
            2 => bad.match_id = 0,
            3 => bad.generation = 0,
            4 => bad.room_revision = 1,
            5 => bad.digest = [1; 32],
            6 => bad.context.key.source_id = 0,
            _ => bad.context.epoch = 0,
        }
        assert!(encode_request(9, &bad, VERSION).is_err());
    }
    assert!(encode_request(0, &q, VERSION).is_err());
    let ack = protocol::Ack {
        request_id: 1,
        version: 39,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    };
    assert!(negotiated(&ack));
    assert!(!negotiated(&protocol::Ack { version: 38, ..ack }));
    assert!(!negotiated(&protocol::Ack {
        capabilities: ack.capabilities & !protocol::CAP_ACTION_LIFE,
        ..ack
    }));
}
#[test]
fn exact_echo_truncation_coalescing_and_metadata_are_bounded() {
    let q = request();
    let (expected, bytes) = response(&q, State::Data, Some(&data()));
    for n in 0..bytes.len() {
        assert!(
            read_reply(&mut &bytes[..n], &expected, VERSION).is_err(),
            "truncation {n}"
        );
    }
    for at in [
        0,
        8,
        12,
        14,
        16,
        28,
        40,
        41,
        48,
        64,
        80,
        REQUEST_BYTES + 3,
        REQUEST_BYTES + 48,
        REQUEST_BYTES + 52,
    ] {
        let mut bad = bytes.clone();
        bad[at] ^= 1;
        assert!(
            read_reply(&mut bad.as_slice(), &expected, VERSION).is_err(),
            "offset {at}"
        );
    }
    for at in [0, 1, 2, 36, 37, 38, 39] {
        let mut bad = bytes.clone();
        bad[REQUEST_BYTES + at] = 255;
        assert!(read_reply(&mut bad.as_slice(), &expected, VERSION).is_err());
    }
    let mut both = bytes.clone();
    both.extend(&bytes);
    let mut input = both.as_slice();
    read_reply(&mut input, &expected, VERSION).unwrap();
    assert_eq!(input, bytes);
    read_reply(&mut input, &expected, VERSION).unwrap();
    assert!(input.is_empty());
}
#[test]
fn scope_manifest_actor_and_saved_state_cannot_be_forged() {
    let q = request();
    for n in 0..6 {
        let mut d = data();
        match n {
            0 => d.personal.player.source += 1,
            1 => d.personal.player.object += 1,
            2 => d.personal.player.epoch += 1,
            3 => d.personal.player.channel += 1,
            4 => d.world.identity.match_id += 1,
            _ => d.presentation = None,
        };
        let (expected, bytes) = response(&q, State::Data, Some(&d));
        assert!(read_reply(&mut bytes.as_slice(), &expected, VERSION).is_err());
    }
    let (expected, mut bytes) = response(&q, State::Data, Some(&data()));
    bytes[REQUEST_BYTES + 38] = 2;
    assert!(read_reply(&mut bytes.as_slice(), &expected, VERSION).is_err());
    let loaded = Request {
        action: Action::Loaded,
        digest: [2; 32],
        ..q
    };
    let (expected, bytes) = response(&loaded, State::Data, Some(&data()));
    assert!(read_reply(&mut bytes.as_slice(), &expected, VERSION).is_err());
}
#[test]
fn header_bounds_are_checked_before_any_body_read() {
    struct Only(std::io::Cursor<Vec<u8>>);
    impl Read for Only {
        fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
            assert!(self.0.position() < 16, "body read after invalid header");
            self.0.read(b)
        }
    }
    let (expected, bytes) = response(&request(), State::Data, Some(&data()));
    for n in [
        0,
        16,
        MIN_RESULT_BYTES as u32 - 1,
        MAX_RESULT_BYTES as u32 + 1,
        u32::MAX,
    ] {
        let mut header = bytes[..16].to_vec();
        header[4..8].copy_from_slice(&n.to_be_bytes());
        assert!(read_reply(&mut Only(std::io::Cursor::new(header)), &expected, VERSION).is_err());
    }
}
