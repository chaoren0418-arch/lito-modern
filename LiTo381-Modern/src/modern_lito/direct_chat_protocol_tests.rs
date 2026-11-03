use super::*;
fn request() -> Request {
    Request::read(
        SelectionContext {
            key: CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        },
        3,
        0,
        1,
    )
}
fn owner() -> Peer {
    Peer {
        key: request().context.key,
        character_name: "Sun Knight".into(),
        source_name: "太陽神".into(),
    }
}
fn reply() -> Reply {
    Reply {
        echo: request(),
        status: Status::Ok,
        sequence: 0,
        cursor: 0,
        available: true,
        truncated: false,
        owner: owner(),
        preferences: Preferences::default(),
        blocked: vec![],
        messages: vec![],
    }
}
fn peer(out: &mut Vec<u8>, p: &Peer) {
    out.extend_from_slice(&p.key.source_id.to_be_bytes());
    out.extend_from_slice(&p.key.object_id.to_be_bytes());
    put_text(out, &p.character_name);
    put_text(out, &p.source_name);
}
fn frame(r: &Reply) -> Vec<u8> {
    let mut b = vec![];
    put_request(&mut b, &r.echo);
    b.extend_from_slice(&[
        r.status as u8,
        (r.available as u8) | ((r.truncated as u8) << 1),
        bits(r.preferences),
        r.blocked.len() as u8,
    ]);
    b.extend_from_slice(&(r.messages.len() as u16).to_be_bytes());
    b.extend_from_slice(&[0, 0]);
    b.extend_from_slice(&r.sequence.to_be_bytes());
    b.extend_from_slice(&r.cursor.to_be_bytes());
    peer(&mut b, &r.owner);
    for a in &r.blocked {
        b.extend_from_slice(&a.source_id.to_be_bytes());
        put_text(&mut b, &a.name);
    }
    for m in &r.messages {
        b.extend_from_slice(&m.id.to_be_bytes());
        b.extend_from_slice(&m.sender_sequence.to_be_bytes());
        peer(&mut b, &m.sender);
        peer(&mut b, &m.recipient);
        put_text(&mut b, &m.text);
    }
    let mut out = app::header(1, RESULT_KIND, 16 + b.len()).unwrap();
    out.extend(b);
    out
}
fn decode(bytes: &[u8]) -> Result<Reply, ApplicationError> {
    match app::decode_response_version(bytes, 32)?.response {
        app::Response::DirectChat(r) => Ok(r),
        _ => Err(invalid()),
    }
}
#[test]
fn exact_java_request_golden_and_version_bounds() {
    let bytes = encode_request(u32::MAX, &request(), 32).unwrap();
    assert_eq!(84, bytes.len());
    assert_eq!(
        bytes.iter().map(|x| format!("{x:02x}")).collect::<String>(),
        concat!(
            "4c544d3100000054ffffffff00340000",
            "000000070000000000000d05ffffffffffffffff03000000",
            "0000000000000000000000000000000100000000000000000000000000000000",
            "000000000000000000000000"
        )
    );
    for v in 1..32 {
        assert!(encode_request(1, &request(), v).is_err());
        assert!(app::decode_response_version(&frame(&reply()), v).is_err());
    }
    assert_eq!(decode(&frame(&reply())).unwrap(), reply());
}
#[test]
fn moderated_profile_is_explicit_and_legacy_never_accepts_silenced() {
    let r = request();
    let bytes = encode_moderated_request(1, &r, 33).unwrap();
    assert_eq!(&bytes[12..14], &56u16.to_be_bytes());
    for version in 1..33 {
        assert!(encode_moderated_request(1, &r, version).is_err());
    }
    let mut denied = reply();
    denied.echo = denied
        .next_command(
            Action::Send,
            Some(Address {
                source_id: 42,
                name: "Moon Mage".into(),
            }),
            None,
            "私訊".into(),
        )
        .unwrap();
    denied.sequence = 1;
    denied.status = Status::Silenced;
    assert!(denied.validate().is_ok());
    let mut encoded = frame(&denied);
    assert!(app::decode_response_version(&encoded, 33).is_err());
    encoded[12..14].copy_from_slice(&57u16.to_be_bytes());
    assert!(
        matches!(app::decode_response_version(&encoded,33).unwrap().response,app::Response::ModeratedDirect(r) if r==denied)
    );
    for version in 1..33 {
        assert!(app::decode_response_version(&encoded, version).is_err());
    }
    for size in 0..encoded.len() {
        assert!(app::decode_response_version(&encoded[..size], 33).is_err());
    }
    denied.echo = request();
    denied.sequence = 0;
    assert!(denied.validate().is_err());
}
#[test]
fn raw_unicode_address_and_action_exclusivity() {
    for name in ["  MOON Mage  ", "İ", "ΟΣ", "角色👩‍💻"] {
        let mut r = request();
        r.action = Action::Block;
        r.sequence = 1;
        r.target = Some(Address {
            source_id: 42,
            name: name.into(),
        });
        let mut b = reply();
        b.echo = r.clone();
        b.sequence = 1;
        b.blocked.push(Address {
            source_id: 42,
            name: "canonical from authority".into(),
        });
        assert_eq!(decode(&frame(&b)).unwrap().echo, r);
        assert!(!format!("{b:?}").contains(name));
    }
    let mut r = request();
    r.membership = 0;
    assert!(r.validate().is_err());
    let mut r = request();
    r.target = Some(Address {
        source_id: 42,
        name: "Moon".into(),
    });
    assert!(r.validate().is_err());
    r.action = Action::Send;
    r.sequence = 1;
    r.text = ".de 20".into();
    assert!(r.validate().is_ok());
    r.preferences = Some(Preferences::default());
    assert!(r.validate().is_err());
    r.preferences = None;
    r.target.as_mut().unwrap().source_id = 0;
    assert!(r.validate().is_err());
    r.target.as_mut().unwrap().source_id = 42;
    r.sequence = i64::MAX as u64 + 1;
    assert!(r.validate().is_err());
}
#[test]
fn largest_bounded_unicode_batch_and_private_ownership() {
    let mut r = reply();
    r.owner.character_name = "a".repeat(256);
    r.owner.source_name = "b".repeat(256);
    let other = Peer {
        key: CharacterKey {
            source_id: 42,
            object_id: 3333,
        },
        character_name: "c".repeat(256),
        source_name: "d".repeat(256),
    };
    for i in 1..=64 {
        r.messages.push(Message {
            id: i,
            sender_sequence: i,
            sender: other.clone(),
            recipient: r.owner.clone(),
            text: "😀".repeat(128),
        });
    }
    for i in 1..=16 {
        r.blocked.push(Address {
            source_id: i,
            name: "z".repeat(256),
        });
    }
    r.cursor = 64;
    let bytes = frame(&r);
    assert!(bytes.len() > 100_000 && bytes.len() < MAX_FRAME_BYTES);
    assert_eq!(decode(&bytes).unwrap(), r);
    r.messages[0].recipient = other.clone();
    assert!(r.validate().is_err());
    r.messages[0].recipient = r.owner.clone();
    r.messages[0].recipient.character_name = "spoof".into();
    assert!(r.validate().is_err());
}
#[test]
fn malformed_reserved_flags_counts_utf8_truncation_and_allocation_bound() {
    let bytes = frame(&reply());
    for (at, value) in [
        (37, 1),
        (38, 1),
        (39, 1),
        (73, 8),
        (74, 1),
        (75, 1),
        (84, 5),
        (85, 4),
        (86, 8),
        (87, 17),
        (89, 65),
        (90, 1),
        (bytes.len() - 1, 255),
    ] {
        let mut bad = bytes.clone();
        bad[at] = value;
        assert!(decode(&bad).is_err(), "offset {at}");
    }
    for n in 0..bytes.len() {
        assert!(decode(&bytes[..n]).is_err());
    }
    let mut bad = bytes.clone();
    bad.push(0);
    let len = bad.len() as u32;
    bad[4..8].copy_from_slice(&len.to_be_bytes());
    assert!(decode(&bad).is_err());
    let mut header = bytes[..16].to_vec();
    header[4..8].copy_from_slice(&((MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
    assert!(app::decode_response_version(&header, 32).is_err());
    let mut stream = std::io::Cursor::new(header);
    assert!(app::read_response_version(&mut stream, 32).is_err());
    assert_eq!(
        stream.position(),
        16,
        "reject length before reading/allocating payload"
    );
}
#[test]
fn confirmation_rejects_rollback_replay_and_resets_only_inbox() {
    let mut c = Confirmation::default();
    assert!(c.accept(&reply()));
    let mut r = reply();
    r.echo.action = Action::Preferences;
    r.echo.sequence = 1;
    r.echo.preferences = Some(Preferences::default());
    r.sequence = 1;
    assert!(c.accept(&r));
    assert!(!c.accept(&r));
    r.echo = r.next_read();
    assert!(c.accept(&r));
    r.sequence = 0;
    assert!(!c.accept(&r));
    r.sequence = 1;
    let other = Peer {
        key: CharacterKey {
            source_id: 42,
            object_id: 3333,
        },
        character_name: "Moon".into(),
        source_name: "月亮".into(),
    };
    r.cursor = 1;
    r.messages.push(Message {
        id: 1,
        sender_sequence: 1,
        sender: other,
        recipient: r.owner.clone(),
        text: "hello".into(),
    });
    assert!(c.accept(&r));
    r.messages.clear();
    r.cursor = 0;
    assert!(!c.accept(&r));
    c.reset_inbox();
    assert!(c.accept(&r));
    r.sequence = 0;
    assert!(!c.accept(&r)); // Navigation does not erase the connection's command sequence.
}
#[test]
fn gaps_status_and_successful_send_must_confirm_actual_private_message() {
    let mut r = reply();
    let target = Address {
        source_id: 42,
        name: "  MOON  ".into(),
    };
    r.echo = r
        .next_command(Action::Send, Some(target), None, ".de 20".into())
        .unwrap();
    r.sequence = 1;
    assert!(r.validate().is_err());
    let other = Peer {
        key: CharacterKey {
            source_id: 42,
            object_id: 3333,
        },
        character_name: "Moon".into(),
        source_name: "月亮".into(),
    };
    r.messages.push(Message {
        id: 1,
        sender_sequence: 1,
        sender: r.owner.clone(),
        recipient: other,
        text: ".de 20".into(),
    });
    r.cursor = 1;
    assert!(r.validate().is_ok());
    r.messages[0].sender_sequence = 2;
    assert!(r.validate().is_err());
    r.messages[0].sender_sequence = 1;
    r.messages[0].recipient.key.source_id = 7;
    assert!(r.validate().is_err());
    r.messages[0].recipient.key.source_id = 42;
    r.cursor = 2;
    r.messages[0].id = 2;
    assert!(r.validate().is_err());
    r.truncated = true;
    assert!(r.validate().is_ok());
    r.echo = r.next_read();
    r.messages.clear();
    r.truncated = false;
    r.status = Status::RateLimited;
    assert!(r.validate().is_err());
}
#[test]
fn independently_negotiated_bit_is_not_in_default_hello() {
    use super::super::protocol::{self as p, Message as Wire};
    let mut hello = p::Hello::current(1);
    assert_eq!(0, hello.supported_capabilities & p::CAP_DIRECT_CHAT);
    hello.supported_capabilities = 39 | p::CAP_CHAT | p::CAP_DIRECT_CHAT;
    assert!(p::encode(Wire::Hello(hello)).is_ok());
    for bit in [1, 2, 4, 32, p::CAP_CHAT] {
        let mut bad = hello;
        bad.supported_capabilities &= !bit;
        assert!(p::encode(Wire::Hello(bad)).is_err());
    }
    let mut bad = hello;
    bad.max_version = 31;
    assert!(p::encode(Wire::Hello(bad)).is_err());
}

#[test]
fn missing_middle_message_is_not_a_truncated_prefix() {
    let mut r = reply();
    let other = Peer {
        key: CharacterKey {
            source_id: 42,
            object_id: 3333,
        },
        character_name: "Moon Mage".into(),
        source_name: "月亮".into(),
    };
    for id in [1, 3] {
        r.messages.push(Message {
            id,
            sender_sequence: id,
            sender: other.clone(),
            recipient: r.owner.clone(),
            text: "hello".into(),
        });
    }
    r.cursor = 3;
    assert!(r.validate().is_err());
    assert!(decode(&frame(&r)).is_err());
}
