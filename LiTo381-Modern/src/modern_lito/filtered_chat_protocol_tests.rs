use super::*;
fn read() -> Request {
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
    )
}
fn reply() -> Reply {
    Reply {
        echo: read(),
        status: Status::Ok,
        membership: 1,
        sequence: 0,
        cursor: 0,
        available: true,
        truncated: false,
        owner: Peer {
            key: read().context.key,
            character_name: "A".into(),
            source_name: "S".into(),
        },
        messages: vec![],
        receipt: None,
    }
}
fn put_peer(b: &mut Vec<u8>, p: &Peer) {
    b.extend_from_slice(&p.key.source_id.to_be_bytes());
    b.extend_from_slice(&p.key.object_id.to_be_bytes());
    put_text(b, &p.character_name);
    put_text(b, &p.source_name);
}
fn put_row(b: &mut Vec<u8>, id: u64, kind: Kind, sender: &Peer, text: &str) {
    b.extend_from_slice(&id.to_be_bytes());
    b.extend_from_slice(&[kind as u8, 0, 0, 0]);
    put_peer(b, sender);
    put_text(b, text);
}
fn frame(r: &Reply) -> Vec<u8> {
    let mut b = vec![];
    put_request(&mut b, &r.echo);
    b.extend_from_slice(&[
        r.status as u8,
        (r.available as u8) | ((r.truncated as u8) << 1),
        r.receipt.is_some() as u8,
        0,
    ]);
    b.extend_from_slice(&(r.messages.len() as u16).to_be_bytes());
    b.extend_from_slice(&[0, 0]);
    for n in [r.membership, r.sequence, r.cursor] {
        b.extend_from_slice(&n.to_be_bytes());
    }
    put_peer(&mut b, &r.owner);
    for m in &r.messages {
        put_row(&mut b, m.id, m.kind, &m.sender, &m.text);
    }
    if let Some(receipt) = &r.receipt {
        put_row(
            &mut b,
            receipt.sequence,
            receipt.kind,
            &receipt.sender,
            &receipt.text,
        );
    }
    let mut out = app::header(1, RESULT_KIND, 16 + b.len()).unwrap();
    out.extend(b);
    out
}
fn accepted() -> Reply {
    let mut r = reply();
    r.echo = r.next_send(Kind::World, "世界👋".into()).unwrap();
    r.sequence = 1;
    r.receipt = Some(Receipt {
        sequence: 1,
        sender: r.owner.clone(),
        kind: r.echo.kind,
        text: r.echo.text.clone(),
    });
    r
}
#[test]
fn request_and_result_match_java_exact_goldens() {
    let b = encode_request(u32::MAX, &read(), 33).unwrap();
    assert_eq!(b.iter().map(|v|format!("{v:02x}")).collect::<String>(),"4c544d310000004cffffffff00360000000000070000000000000d05ffffffffffffffff03000000000000000000000000000000000000000000000000000000000000000000000000000000");
    let b = frame(&reply());
    assert_eq!(b.iter().map(|v|format!("{v:02x}")).collect::<String>(),"4c544d310000007e0000000100370000000000070000000000000d05ffffffffffffffff030000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000100000000000000000000000000000000000000070000000000000d05000141000153");
    assert_eq!(
        app::decode_response_version(&b, 33).unwrap().response,
        app::Response::FilteredChat(reply())
    );
    for v in 1..33 {
        assert!(encode_request(1, &read(), v).is_err());
        let mut stream = std::io::Cursor::new(b.clone());
        assert!(app::read_response_version(&mut stream, v).is_err());
        assert_eq!(stream.position(), 16);
    }
}
#[test]
fn successful_send_is_confirmed_without_self_delivery() {
    let r = accepted();
    assert!(r.messages.is_empty());
    assert!(r.confirms(&r.echo));
    assert_eq!(r.cursor, 0);
    assert_eq!(
        app::decode_response_version(&frame(&r), 33)
            .unwrap()
            .response,
        app::Response::FilteredChat(r.clone())
    );
    let mut wrong = r.clone();
    wrong.receipt = None;
    assert!(!wrong.confirms(&r.echo));
    for i in 0..5 {
        let mut wrong = r.clone();
        let receipt = wrong.receipt.as_mut().unwrap();
        match i {
            0 => receipt.sequence = 2,
            1 => receipt.sender.key.source_id = 42,
            2 => receipt.sender.character_name = "other".into(),
            3 => receipt.kind = Kind::Trade,
            _ => receipt.text = "different".into(),
        }
        assert!(wrong.validate().is_err());
    }
    assert!(!format!("{r:?}").contains("世界"));
    assert!(!format!("{:?}", r.receipt).contains("世界"));
}
#[test]
fn semantic_denials_preserve_sequence_and_do_not_require_global_unavailability() {
    let mut r = accepted();
    r.receipt = None;
    for status in [
        Status::RateLimited,
        Status::CommandRequired,
        Status::Silenced,
        Status::Unavailable,
    ] {
        r.status = status;
        assert!(r.validate().is_ok());
        assert_eq!(
            r.next_send(Kind::Trade, "retry manually".into())
                .unwrap()
                .sequence,
            2
        );
    }
    let mut read = reply();
    read.status = Status::Unavailable;
    assert!(read.validate().is_err());
    read.available = false;
    assert!(read.validate().is_ok());
    read.status = Status::Ok;
    assert!(read.validate().is_err());
    r.available = false;
    assert!(r.next_send(Kind::Normal, "no".into()).is_err());
}
#[test]
fn local_cursor_requires_contiguous_suffix_and_truthful_gap() {
    let mut r = reply();
    r.cursor = 3;
    r.truncated = true;
    r.messages.push(Message {
        id: 3,
        sender: r.owner.clone(),
        kind: Kind::Normal,
        text: "third".into(),
    });
    assert!(r.validate().is_ok());
    r.truncated = false;
    assert!(r.validate().is_err());
    r.messages.clear();
    assert!(r.validate().is_err());
    r.messages = vec![
        Message {
            id: 1,
            sender: r.owner.clone(),
            kind: Kind::Normal,
            text: "first".into(),
        },
        Message {
            id: 3,
            sender: r.owner.clone(),
            kind: Kind::Normal,
            text: "third".into(),
        },
    ];
    assert!(r.validate().is_err());
}
#[test]
fn largest_typed_batch_and_receipt_fit_bound_and_keep_stream_alignment() {
    let mut r = accepted();
    r.echo.text = "😀".repeat(128);
    r.owner.character_name = "a".repeat(256);
    r.owner.source_name = "b".repeat(256);
    r.receipt = Some(Receipt {
        sequence: 1,
        sender: r.owner.clone(),
        kind: Kind::World,
        text: r.echo.text.clone(),
    });
    r.cursor = 64;
    for i in 1..=64 {
        r.messages.push(Message {
            id: i,
            sender: r.owner.clone(),
            kind: if i % 2 == 0 {
                Kind::Trade
            } else {
                Kind::Normal
            },
            text: "😀".repeat(128),
        });
    }
    let b = frame(&r);
    assert!(b.len() > 65536 && b.len() < MAX_FRAME_BYTES);
    let mut both = b.clone();
    both.extend(&b);
    let mut stream = std::io::Cursor::new(both);
    for _ in 0..2 {
        assert_eq!(
            app::read_response_version(&mut stream, 33)
                .unwrap()
                .response,
            app::Response::FilteredChat(r.clone())
        );
    }
    assert_eq!(stream.position(), (b.len() * 2) as u64);
}
#[test]
fn malformed_reserved_enums_utf8_counts_truncation_and_oversize_fail_closed() {
    let r = accepted();
    let b = frame(&r);
    let tail = 76 + r.echo.text.len();
    for (at, value) in [
        (37, 1),
        (38, 1),
        (39, 1),
        (72, 2),
        (73, 3),
        (tail, 5),
        (tail + 1, 4),
        (tail + 2, 2),
        (tail + 3, 1),
        (tail + 5, 65),
        (tail + 6, 1),
        (b.len() - 1, 255),
    ] {
        let mut bad = b.clone();
        bad[at] = value;
        assert!(
            app::decode_response_version(&bad, 33).is_err(),
            "offset {at}"
        );
    }
    for i in 0..b.len() {
        assert!(app::decode_response_version(&b[..i], 33).is_err());
    }
    let mut extra = b.clone();
    extra.push(0);
    let size = extra.len() as u32;
    extra[4..8].copy_from_slice(&size.to_be_bytes());
    assert!(app::decode_response_version(&extra, 33).is_err());
    let mut header = b[..16].to_vec();
    header[4..8].copy_from_slice(&((MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
    let mut stream = std::io::Cursor::new(header);
    assert!(app::read_response_version(&mut stream, 33).is_err());
    assert_eq!(stream.position(), 16);
}
#[test]
fn capability_requires_new_version_and_all_qualified_preference_dependencies() {
    use super::super::protocol as p;
    let caps = 39 | p::CAP_CHAT | p::CAP_DIRECT_CHAT | p::CAP_FILTERED_CHAT;
    let mut h = p::Hello::current(1);
    h.supported_capabilities = caps;
    assert!(p::encode(p::Message::Hello(h)).is_ok());
    assert_eq!(p::CAP_FILTERED_CHAT, 16777216);
    assert_eq!(p::RUNTIME_CAPABILITIES & p::CAP_FILTERED_CHAT, 0);
    for bit in [1, 2, 4, 32, p::CAP_CHAT, p::CAP_DIRECT_CHAT] {
        assert!(p::encode(p::Message::Hello(p::Hello {
            supported_capabilities: caps & !bit,
            ..h
        }))
        .is_err());
        assert!(p::encode(p::Message::Hello(p::Hello {
            required_capabilities: caps & !bit,
            ..h
        }))
        .is_err());
    }
    for v in 1..33 {
        assert!(p::encode(p::Message::Hello(p::Hello {
            max_version: v,
            ..h
        }))
        .is_err());
        assert!(p::encode(p::Message::Ack(p::Ack {
            request_id: 1,
            version: v,
            numeric_schema: 1,
            capabilities: caps
        }))
        .is_err());
    }
}
#[test]
fn connection_confirmation_rejects_replay_rollback_and_membership_reuse_in_new_scope() {
    let mut guard = Confirmation::default();
    assert!(guard.accept(&reply()));
    let mut r = accepted();
    assert!(guard.accept(&r));
    assert!(!guard.accept(&r));
    r.echo = r.next_read();
    r.receipt = None;
    assert!(guard.accept(&r));
    let mut wrong = r.clone();
    wrong.echo.room_id = 1;
    assert!(!guard.accept(&wrong));
    wrong = r.clone();
    wrong.sequence = 0;
    assert!(!guard.accept(&wrong));
    wrong = r.clone();
    wrong.membership = 2;
    assert!(!guard.accept(&wrong));
    r.echo = Request::read(
        SelectionContext {
            key: CharacterKey {
                source_id: 42,
                object_id: 3333,
            },
            epoch: 2,
        },
        1,
        0,
    );
    r.owner.key = r.echo.context.key;
    r.membership = 2;
    assert!(guard.accept(&r));
    r.membership = 1;
    assert!(!guard.accept(&r));
}
#[test]
fn counters_are_i64_except_opaque_epoch_and_last_legal_receipt_is_valid() {
    let mut r = accepted();
    r.echo.sequence = i64::MAX as u64;
    r.sequence = r.echo.sequence;
    r.receipt.as_mut().unwrap().sequence = r.sequence;
    r.available = false;
    assert!(r.validate().is_ok());
    assert!(r.next_send(Kind::Normal, "no wrap".into()).is_err());
    r.available = true;
    assert!(r.validate().is_err());
    let mut request = read();
    request.membership = i64::MAX as u64 + 1;
    assert!(request.validate().is_err());
    request = read();
    request.after = 1;
    assert!(request.validate().is_err());
    request = read();
    request.kind = Kind::Trade;
    assert!(request.validate().is_err());
}

#[test]
fn confirmation_rejects_cursor_rollback_even_when_reply_is_valid_in_isolation() {
    let mut guard = Confirmation::default();
    let mut first = reply();
    first.cursor = 2;
    first.messages = (1..=2)
        .map(|id| Message {
            id,
            sender: first.owner.clone(),
            kind: Kind::Normal,
            text: "already confirmed".into(),
        })
        .collect();
    assert!(guard.accept(&first));
    let rollback = reply();
    assert!(rollback.validate().is_ok());
    assert!(
        !guard.accept(&rollback),
        "repeated bootstrap cannot erase confirmed history"
    );
    let mut rollback = reply();
    rollback.echo.membership = 1;
    assert!(rollback.validate().is_ok());
    assert!(
        !guard.accept(&rollback),
        "same admission must not lower the confirmed cursor"
    );
    assert!(
        guard.accept(&first),
        "a rejected reply must not mutate confirmation state"
    );
}

#[test]
fn request_preflight_requires_confirmed_admission_and_new_bootstrap_resets_only_cursor() {
    let mut guard = Confirmation::default();
    assert!(!guard.permits(&accepted().echo));
    assert!(guard.permits(&read()));
    assert!(guard.accept(&reply()));
    let mut sent = accepted();
    sent.cursor = 1;
    sent.messages.push(Message {
        id: 1,
        sender: sent.owner.clone(),
        kind: Kind::World,
        text: sent.echo.text.clone(),
    });
    assert!(guard.permits(&sent.echo));
    assert!(guard.accept(&sent));
    assert!(!guard.permits(&sent.echo));
    let mut next = sent.next_send(Kind::Normal, "next".into()).unwrap();
    assert!(guard.permits(&next));
    next.after = 2;
    assert!(!guard.permits(&next));
    next.after = 1;
    next.room_id = 8;
    assert!(!guard.permits(&next));
    let mut entered = reply();
    entered.echo.room_id = 8;
    entered.membership = 2;
    entered.sequence = 1;
    assert!(guard.permits(&entered.echo));
    assert!(guard.accept(&entered));
    assert!(!guard.permits(&sent.next_read()));
    let next = entered.next_send(Kind::Trade, "new room".into()).unwrap();
    assert_eq!(next.sequence, 2);
    assert_eq!(next.after, 0);
    assert!(guard.permits(&next));
}
