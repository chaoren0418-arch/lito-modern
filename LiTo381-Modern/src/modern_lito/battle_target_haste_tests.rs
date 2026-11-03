use super::*;
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
        target: Target::Player {
            entity: 1,
            kind: 0,
            source: 42,
            id: 3333,
            life: 5,
            equipment_revision: 8,
            x: 32767,
            y: 32768,
            buff_revision: 97,
        },
    }
}
#[test]
fn explicit_v44_opt_in_does_not_enable_the_player_runtime() {
    use super::super::protocol as p;
    assert_eq!(p::RUNTIME_CAPABILITIES & p::CAP_HASTE, 0);
    let mut hello = p::Hello::current(1);
    hello.max_version = 44;
    hello.supported_capabilities |= p::CAP_HASTE | p::CAP_BATTLE_DATA | p::CAP_ACTION_LIFE;
    assert!(p::encode(p::Message::Hello(hello)).is_ok());
    let ack = p::Ack {
        request_id: 1,
        version: 44,
        numeric_schema: 1,
        capabilities: hello.supported_capabilities,
    };
    assert!(negotiated(&ack));
    assert!(p::encode(p::Message::Ack(ack)).is_ok());
    for cap in [
        p::CAP_HASTE,
        p::CAP_TIMED,
        p::CAP_BATTLE_DATA,
        p::CAP_ACTION_LIFE,
        p::CAP_MATCH_DESCRIPTOR,
    ] {
        assert!(!negotiated(&p::Ack {
            capabilities: ack.capabilities & !cap,
            ..ack
        }));
    }
    assert!(!negotiated(&p::Ack { version: 43, ..ack }));
    hello.max_version = 43;
    assert!(p::encode(p::Message::Hello(hello)).is_err());
}
#[test]
fn player_and_bot_targets_preserve_full_identity_and_exact_field_offsets() {
    for kind in [0, 1] {
        let mut q = intent();
        if let Target::Player { kind: k, .. } = &mut q.target {
            *k = kind;
        }
        let b = encode_intent(&q, 44).unwrap();
        assert_eq!(b.len(), 184);
        assert_eq!(
            &b[..16],
            &[0x4c, 0x54, 0x4d, 0x31, 0, 0, 0, 184, 255, 255, 255, 255, 0, 74, 0, 0]
        );
        assert_eq!(&b[28..36], &[255; 8]);
        assert_eq!(i64_at(&b, 88), 25);
        assert_eq!(&b[120..128], &[0; 8]);
        assert_eq!(i64_at(&b, 128), 1);
        assert_eq!(u32_at(&b, 136), u32::from(kind));
        assert_eq!(u32_at(&b, 140), 42);
        assert_eq!(i64_at(&b, 144), 3333);
        assert_eq!(i64_at(&b, 152), 5);
        assert_eq!(i64_at(&b, 160), 8);
        assert_eq!(u32_at(&b, 168), 32767);
        assert_eq!(u32_at(&b, 172), 32768);
        assert_eq!(i64_at(&b, 176), 97);
        assert_eq!(decode_intent(&b, 44).unwrap(), q);
        assert_eq!(encode_intent(&q, 45).unwrap(), b);
        assert_eq!(decode_intent(&b, 45).unwrap(), q);
        for version in [1, 42, 43, crate::modern_lito::protocol::MAX_VERSION + 1] {
            assert!(encode_intent(&q, version).is_err());
            assert!(decode_intent(&b, version).is_err());
        }
        assert_eq!(format!("{q:?}"), "TargetHasteIntent[redacted]");
        assert_eq!(format!("{:?}", q.target), "HasteTarget[redacted]");
    }
}
#[test]
fn npc_target_preserves_world_generation_life_cell_and_roster_stamp() {
    let mut q = intent();
    q.target = Target::Npc {
        world: q.stamp.world,
        serial: i64::MAX,
        life: 6,
        changed_revision: 99,
        x: i32::MIN,
        y: i32::MAX,
    };
    let b = encode_intent(&q, 44).unwrap();
    assert_eq!(&b[120..128], &[1, 0, 0, 0, 0, 0, 0, 0]);
    for (offset, value) in [
        (128, 10),
        (136, 20),
        (144, 30),
        (152, i64::MAX),
        (160, 6),
        (168, 99),
    ] {
        assert_eq!(i64_at(&b, offset), value);
    }
    assert_eq!(u32_at(&b, 176) as i32, i32::MIN);
    assert_eq!(u32_at(&b, 180) as i32, i32::MAX);
    assert_eq!(decode_intent(&b, 44).unwrap(), q);
    for offset in [128, 136, 144, 152, 160, 168] {
        let mut bad = b.clone();
        bad[offset..offset + 8].fill(0);
        assert!(decode_intent(&bad, 44).is_err());
    }
    for (offset, value) in [(128, 11), (136, 21), (144, 31), (168, 101)] {
        let mut bad = b.clone();
        bad[offset..offset + 8].copy_from_slice(&i64::to_be_bytes(value));
        assert!(decode_intent(&bad, 44).is_err());
    }
}
#[test]
fn malformed_or_truncated_targets_never_decode_as_another_target_kind() {
    let q = intent();
    let b = encode_intent(&q, 44).unwrap();
    for len in 0..b.len() {
        assert!(decode_intent(&b[..len], 44).is_err());
        assert!(decode_reply(&b[..len], 44, &q).is_err());
    }
    let mut extra = b.clone();
    extra.push(0);
    assert!(decode_intent(&extra, 44).is_err());
    for offset in [
        0, 4, 13, 15, 39, 120, 121, 122, 123, 124, 125, 126, 127, 136, 137, 138, 139,
    ] {
        let mut bad = b.clone();
        bad[offset] ^= 127;
        assert!(decode_intent(&bad, 44).is_err(), "offset {offset}");
    }
    for offset in [8, 16, 140] {
        let mut bad = b.clone();
        bad[offset..offset + 4].fill(0);
        assert!(decode_intent(&bad, 44).is_err());
    }
    for offset in [20, 28, 40, 48, 56, 64, 72, 80, 88, 128, 144, 152] {
        let mut bad = b.clone();
        bad[offset..offset + 8].fill(0);
        assert!(decode_intent(&bad, 44).is_err());
    }
    for (offset, value) in [(96, -1), (112, 101), (128, 101), (160, -1), (176, 101)] {
        let mut bad = b.clone();
        bad[offset..offset + 8].copy_from_slice(&i64::to_be_bytes(value));
        assert!(decode_intent(&bad, 44).is_err());
    }
    assert!(decode_reply(&vec![0; MAX_RESULT_BYTES + 1], 44, &q).is_err());
}
#[test]
fn stream_bounds_kind_id_and_reserved_bytes_fail_before_body_read() {
    let request = encode_intent(&intent(), 44).unwrap();
    for size in [0, 16, 215, (MAX_RESULT_BYTES + 1) as u32, u32::MAX] {
        let mut header = request[..16].to_vec();
        header[4..8].copy_from_slice(&size.to_be_bytes());
        header[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
        let mut reader = std::io::Cursor::new(header);
        assert!(read_reply(&mut reader, &request, 44).is_err());
        assert_eq!(reader.position(), 16);
    }
    for offset in [0, 8, 12, 14, 15] {
        let mut header = request[..16].to_vec();
        header[4..8].copy_from_slice(&(RESULT_PREFIX_BYTES as u32).to_be_bytes());
        header[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
        header[offset] ^= 1;
        let mut reader = std::io::Cursor::new(header);
        assert!(read_reply(&mut reader, &request, 44).is_err());
        assert_eq!(reader.position(), 16);
    }
    let mut reader = std::io::Cursor::new(vec![0; 16]);
    assert!(read_reply(&mut reader, &request, 43).is_err());
    assert_eq!(reader.position(), 0);
}
