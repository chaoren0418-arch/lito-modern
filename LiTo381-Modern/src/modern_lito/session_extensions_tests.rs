use super::*;
fn hello() -> Hello {
    Hello {
        request_id: 0xfedcba98,
        min_version: 47,
        max_version: 47,
        supported_capabilities: KNOWN_CAPABILITIES,
        required_capabilities: extensions::DEPENDENCIES,
        numeric_schema: 1,
    }
}
fn ack() -> Ack {
    Ack {
        request_id: hello().request_id,
        version: 47,
        numeric_schema: 1,
        capabilities: KNOWN_CAPABILITIES,
    }
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|v| format!("{v:02x}")).collect()
}
#[test]
fn extension_kinds_keep_legacy_word_and_match_java_golden() {
    let h = Message::ExtendedHello(hello(), extensions::HEALING);
    let a = Message::ExtendedAck(ack(), 3);
    assert_eq!(
        hex(&encode(h).unwrap()),
        "4c544d3100000028fedcba9800500000002f002fffffffff18012eff000100000000000300000003"
    );
    assert_eq!(
        hex(&encode(a).unwrap()),
        "4c544d310000001cfedcba9800510000002f0001ffffffff00000003"
    );
    assert_eq!(decode(&encode(h).unwrap()).unwrap(), h);
    assert_eq!(decode(&encode(a).unwrap()).unwrap(), a);
    assert_eq!(encode(Message::Hello(hello())).unwrap().len(), 32);
    assert_eq!(encode(Message::Ack(ack())).unwrap().len(), 24);
    assert_eq!(MAX_VERSION, 47);
}
#[test]
fn armor_data_is_explicit_and_does_not_grant_healing_or_commands() {
    for ext in [4, 5, 7] {
        let offer = extensions::Offer {
            supported: ext,
            required: ext,
        };
        let raw = encode(Message::ExtendedHello(hello(), offer)).unwrap();
        let mut client = ModernClientSession::new_extended(hello(), offer).unwrap();
        assert_eq!(client.begin().unwrap(), raw);
        client
            .accept_ack(&encode(Message::ExtendedAck(ack(), ext)).unwrap())
            .unwrap();
        let peer = extensions::Selected::new(ack(), client.extensions().unwrap()).unwrap();
        assert!(peer.allows_armor());
        assert_eq!(peer.allows_data(), ext != 4);
        assert_eq!(peer.allows_commands(), ext == 7);
    }
    assert_eq!(
        hex(&encode(Message::ExtendedHello(hello(), extensions::ARMOR)).unwrap()),
        "4c544d3100000028fedcba9800500000002f002fffffffff18012eff000100000000000400000004"
    );
    for missing in [0, 1, 3] {
        let mut client = ModernClientSession::new_extended(hello(), extensions::ARMOR).unwrap();
        client.begin().unwrap();
        assert!(client
            .accept_ack(&encode(Message::ExtendedAck(ack(), missing)).unwrap())
            .is_err());
        assert_eq!(client.state(), ClientState::Closed);
    }
    let mut old = ModernClientSession::new_extended(hello(), extensions::HEALING).unwrap();
    old.begin().unwrap();
    assert!(old
        .accept_ack(&encode(Message::ExtendedAck(ack(), 7)).unwrap())
        .is_err());
    assert_eq!(old.state(), ClientState::Closed);
    assert_eq!(extensions::HEALING.supported, 3);
    assert_eq!(extensions::HEALING_ARMOR.required, 7);
}
#[test]
fn required_extensions_are_dependency_closed_versioned_and_independent_of_legacy_high_bit() {
    for bit in (0..32)
        .map(|n| 1u32 << n)
        .filter(|b| extensions::DEPENDENCIES & b != 0)
    {
        let h = Hello {
            required_capabilities: hello().required_capabilities & !bit,
            ..hello()
        };
        assert!(encode(Message::ExtendedHello(h, extensions::HEALING)).is_err());
        assert!(encode(Message::ExtendedAck(
            Ack {
                capabilities: KNOWN_CAPABILITIES & !bit,
                ..ack()
            },
            3
        ))
        .is_err());
    }
    for bits in [2, 6, 8, u32::MAX, 1 << 31] {
        assert!(encode(Message::ExtendedHello(
            hello(),
            extensions::Offer {
                supported: bits,
                required: 0
            }
        ))
        .is_err());
        assert!(encode(Message::ExtendedAck(ack(), bits)).is_err());
    }
    for h in [
        Hello {
            min_version: 46,
            ..hello()
        },
        Hello {
            max_version: 48,
            ..hello()
        },
        Hello {
            numeric_schema: 2,
            ..hello()
        },
    ] {
        assert!(ModernClientSession::new_extended(h, extensions::HEALING).is_err());
    }
    assert!(ModernClientSession::new_extended(
        hello(),
        extensions::Offer {
            supported: 1,
            required: 3
        }
    )
    .is_err());
    let base = Ack {
        capabilities: extensions::DEPENDENCIES,
        ..ack()
    };
    assert!(extensions::allows_data(base, 1));
    assert!(!extensions::allows_commands(base, 1));
    assert!(extensions::allows_commands(base, 3));
    assert!(!extensions::allows_commands(base, 0));
}
#[test]
fn exact_ack_kind_required_and_unoffered_extensions_cannot_revive_closed_session() {
    let valid = encode(Message::ExtendedAck(ack(), 3)).unwrap();
    for bad in [
        encode(Message::Ack(ack())).unwrap(),
        encode(Message::ExtendedAck(ack(), 1)).unwrap(),
        encode(Message::ExtendedAck(
            Ack {
                request_id: 1,
                ..ack()
            },
            3,
        ))
        .unwrap(),
    ] {
        let mut s = ModernClientSession::new_extended(hello(), extensions::HEALING).unwrap();
        assert_eq!(s.extensions(), None);
        s.begin().unwrap();
        assert_eq!(s.extensions(), None);
        assert!(s.accept_ack(&bad).is_err());
        assert_eq!(s.state(), ClientState::Closed);
        assert_eq!(s.extensions(), None);
        assert!(s.accept_ack(&valid).is_err());
    }
    let mut legacy = ModernClientSession::new(hello()).unwrap();
    legacy.begin().unwrap();
    assert!(legacy.accept_ack(&valid).is_err());
    assert_eq!(legacy.extensions(), None);
    let mut data = ModernClientSession::new_extended(
        hello(),
        extensions::Offer {
            supported: 1,
            required: 1,
        },
    )
    .unwrap();
    data.begin().unwrap();
    assert!(data.accept_ack(&valid).is_err());
    assert_eq!(data.extensions(), None);
    let mut full = ModernClientSession::new_extended(hello(), extensions::HEALING).unwrap();
    full.begin().unwrap();
    assert_eq!(full.accept_ack(&valid).unwrap(), ack());
    assert_eq!(full.extensions(), Some(3));
    full.close();
    assert_eq!(full.extensions(), None);
    let mut optional = ModernClientSession::new_extended(
        hello(),
        extensions::Offer {
            supported: 3,
            required: 0,
        },
    )
    .unwrap();
    optional.begin().unwrap();
    optional
        .accept_ack(&encode(Message::ExtendedAck(ack(), 0)).unwrap())
        .unwrap();
    assert_eq!(optional.extensions(), Some(0));
}
#[test]
fn extension_readers_remain_bounded_and_consume_one_frame() {
    use std::io::Cursor;
    for m in [
        Message::ExtendedHello(hello(), extensions::HEALING),
        Message::ExtendedAck(ack(), 3),
    ] {
        let b = encode(m).unwrap();
        let mut stream = Cursor::new([b.as_slice(), b.as_slice()].concat());
        assert_eq!(read_frame(&mut stream).unwrap(), b);
        assert_eq!(stream.position() as usize, b.len());
        for n in 0..b.len() {
            assert!(decode(&b[..n]).is_err());
        }
        for offset in [0, 4, 13, 15, b.len() - 1] {
            let mut bad = b.clone();
            bad[offset] ^= if offset == b.len() - 1 { 0x80 } else { 0x7f };
            assert!(decode(&bad).is_err());
        }
        for size in [0u32, 16, 31, 41, u32::MAX] {
            let mut bad = b.clone();
            bad[4..8].copy_from_slice(&size.to_be_bytes());
            let mut stream = Cursor::new(bad);
            assert!(read_frame(&mut stream).is_err());
            assert_eq!(stream.position(), 16);
        }
        let mut extra = b.clone();
        extra.push(0);
        let size = extra.len() as u32;
        extra[4..8].copy_from_slice(&size.to_be_bytes());
        assert!(decode(&extra).is_err());
    }
}

#[test]
fn vip_world_lifecycle_is_independent_from_read_and_equipment_selection() {
    for (bits, data, gear, world) in [
        (16, true, false, false),
        (48, true, true, false),
        (80, true, false, true),
        (112, true, true, true),
    ] {
        let peer = extensions::Selected::new(ack(), bits).unwrap();
        assert_eq!(peer.allows_vip_data(), data);
        assert_eq!(peer.allows_vip_equipment(), gear);
        assert_eq!(peer.allows_vip_world(), world);
        let message = Message::ExtendedAck(ack(), bits);
        assert_eq!(decode(&encode(message).unwrap()).unwrap(), message);
    }
    for bits in [32, 64, 96, 128] {
        assert!(extensions::Selected::new(ack(), bits).is_err());
    }
}
