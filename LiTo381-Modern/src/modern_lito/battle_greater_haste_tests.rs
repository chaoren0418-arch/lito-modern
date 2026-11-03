use super::*;
#[test]
fn explicit_capability_needs_v42_dependencies_and_never_changes_runtime_offer() {
    use super::super::protocol as p;
    assert_eq!(p::RUNTIME_CAPABILITIES & p::CAP_GREATER_HASTE, 0);
    let mut hello = p::Hello::current(1);
    hello.max_version = 42;
    hello.supported_capabilities |= p::CAP_GREATER_HASTE | p::CAP_BATTLE_DATA | p::CAP_ACTION_LIFE;
    assert!(p::encode(p::Message::Hello(hello)).is_ok());
    let ack = p::Ack {
        request_id: 1,
        version: 42,
        numeric_schema: 1,
        capabilities: hello.supported_capabilities,
    };
    assert!(negotiated(&ack));
    assert!(p::encode(p::Message::Ack(ack)).is_ok());
    for cap in [
        p::CAP_TIMED,
        p::CAP_BATTLE_DATA,
        p::CAP_ACTION_LIFE,
        p::CAP_MATCH_DESCRIPTOR,
    ] {
        assert!(p::encode(p::Message::Ack(p::Ack {
            capabilities: ack.capabilities & !cap,
            ..ack
        }))
        .is_err());
        assert!(!negotiated(&p::Ack {
            capabilities: ack.capabilities & !cap,
            ..ack
        }));
    }
    hello.max_version = 41;
    assert!(p::encode(p::Message::Hello(hello)).is_err());
    assert!(p::encode(p::Message::Ack(p::Ack { version: 41, ..ack })).is_err());
}
#[test]
fn streaming_bounds_reject_before_body_allocation_or_read() {
    let request = encode_intent(&intent(), 42).unwrap();
    for size in [0, 16, 151, (MAX_RESULT_BYTES + 1) as u32, u32::MAX] {
        let mut header = request[..16].to_vec();
        header[4..8].copy_from_slice(&size.to_be_bytes());
        header[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
        let mut cursor = std::io::Cursor::new(header);
        assert!(read_reply(&mut cursor, &request, 42).is_err());
        assert_eq!(cursor.position(), 16);
    }
    let mut reader = std::io::Cursor::new(vec![0; 16]);
    assert!(read_reply(&mut reader, &request, 41).is_err());
    assert_eq!(reader.position(), 0);
}
fn intent() -> Intent {
    Intent {
        request_id: u32::MAX,
        stamp: Stamp {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: u64::MAX,
            },
            channel: 3,
            world: [10, 20, 30],
            entity: 2,
            life: 4,
            revision: 100,
        },
        sequence: 25,
        equipment_revision: 9,
        x: 32768,
        y: 32769,
        buff_revision: 98,
    }
}
#[test]
fn exact_intent_preserves_wide_epoch_and_requires_explicit_candidate_version() {
    let q = intent();
    let b = encode_intent(&q, 42).unwrap();
    assert_eq!(b.len(), 120);
    assert_eq!(
        &b[..16],
        &[0x4c, 0x54, 0x4d, 0x31, 0, 0, 0, 120, 255, 255, 255, 255, 0, 72, 0, 0]
    );
    assert_eq!(&b[28..36], &[255; 8]);
    assert_eq!(i64_at(&b, 88), 25);
    assert_eq!(i64_at(&b, 112), 98);
    assert_eq!(decode_intent(&b, 42).unwrap(), q);
    assert_eq!(format!("{q:?}"), "GreaterHasteIntent[redacted]");
    assert_eq!(encode_intent(&q, 43).unwrap(), b);
    assert_eq!(decode_intent(&b, 43).unwrap(), q);
    assert_eq!(encode_intent(&q, 44).unwrap(), b);
    assert_eq!(decode_intent(&b, 44).unwrap(), q);
    for v in [1, 41, super::super::protocol::MAX_VERSION + 1] {
        assert!(encode_intent(&q, v).is_err());
        assert!(decode_intent(&b, v).is_err());
    }
    assert_eq!(super::super::protocol::MAX_VERSION, 47);
}
#[test]
fn intent_and_reply_bounds_reject_truncation_padding_wrong_kind_and_unbound_echo() {
    let q = intent();
    let b = encode_intent(&q, 42).unwrap();
    for len in 0..b.len() {
        assert!(decode_intent(&b[..len], 42).is_err());
        assert!(decode_reply(&b[..len], 42, &q).is_err());
    }
    let mut extra = b.clone();
    extra.push(0);
    assert!(decode_intent(&extra, 42).is_err());
    for offset in [0, 4, 13, 15, 39] {
        let mut bad = b.clone();
        bad[offset] ^= 127;
        assert!(decode_intent(&bad, 42).is_err());
    }
    for offset in [8, 16] {
        let mut bad = b.clone();
        bad[offset..offset + 4].fill(0);
        assert!(decode_intent(&bad, 42).is_err());
    }
    for offset in [20, 28, 40, 48, 56, 64, 72, 80, 88] {
        let mut bad = b.clone();
        bad[offset..offset + 8].fill(0);
        assert!(decode_intent(&bad, 42).is_err());
    }
    let mut bad = q;
    bad.buff_revision = 101;
    assert!(encode_intent(&bad, 42).is_err());
    bad = q;
    bad.equipment_revision = -1;
    assert!(encode_intent(&bad, 42).is_err());
    assert!(decode_reply(&vec![0; MAX_RESULT_BYTES + 1], 42, &q).is_err());
}
