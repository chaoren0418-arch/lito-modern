use super::super::{application_protocol::CharacterKey, direct_chat_protocol::Peer};
use super::*;
fn scope() -> Scope {
    Scope {
        context: SelectionContext {
            key: CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        },
        channel_id: 0,
        room_id: 0,
    }
}
fn owner() -> Peer {
    Peer {
        key: scope().context.key,
        character_name: "3333".into(),
        source_name: "太陽神".into(),
    }
}
fn read(s: &State) -> filtered::Reply {
    filtered::Reply {
        echo: s.filtered_read().unwrap(),
        status: filtered::Status::Ok,
        membership: if s.received { s.membership } else { 1 },
        sequence: s.sequence,
        cursor: s.cursor,
        available: true,
        truncated: false,
        owner: owner(),
        messages: vec![],
        receipt: None,
    }
}
fn ready() -> State {
    let mut s = State::default();
    s.align_profile(true, true, Some(scope()));
    s.receive_filtered(read(&s)).unwrap();
    s
}
fn row(id: u64) -> filtered::Message {
    filtered::Message {
        id,
        sender: owner(),
        kind: filtered::Kind::World,
        text: format!("訊息{id}"),
    }
}
#[test]
fn filtered_bootstrap_never_uses_legacy_log_or_claims_optimistic_delivery() {
    let mut s = State::default();
    s.align_profile(true, true, Some(scope()));
    assert!(s.waiting() && s.read().is_none() && s.token().is_none());
    s.receive_filtered(read(&s)).unwrap();
    assert!(s.filtered() && s.token().is_some());
    assert!(s.send(s.token().unwrap(), "不能用舊入口".into()).is_err());
    let request = s
        .filtered_send(
            s.token().unwrap(),
            filtered::Kind::World,
            "不收自己的世界訊息".into(),
        )
        .unwrap();
    assert!(s.filtered_messages().is_empty());
    assert_eq!(s.delivery(), Delivery::None);
    let mut reply = read(&s);
    reply.echo = request.clone();
    reply.sequence = 1;
    reply.receipt = Some(filtered::Receipt {
        sequence: 1,
        sender: owner(),
        kind: request.kind,
        text: request.text,
    });
    s.receive_filtered(reply).unwrap();
    assert_eq!(s.delivery(), Delivery::Confirmed);
    assert_eq!(s.cursor, 0);
    assert!(s.filtered_messages().is_empty());
    s.receive_filtered(read(&s)).unwrap();
    assert_eq!(s.delivery(), Delivery::Confirmed);
    assert_eq!(
        s.filtered_send(s.token().unwrap(), filtered::Kind::Trade, "下一則".into())
            .unwrap()
            .sequence,
        2
    );
}
#[test]
fn silenced_consumes_sequence_without_inventing_history_or_receipt() {
    let mut s = ready();
    let mut reply = read(&s);
    reply.echo = s
        .filtered_send(s.token().unwrap(), filtered::Kind::Normal, "被禁言".into())
        .unwrap();
    reply.status = filtered::Status::Silenced;
    reply.sequence = 1;
    s.receive_filtered(reply.clone()).unwrap();
    assert_eq!(s.delivery(), Delivery::Silenced);
    assert_eq!(s.cursor, 0);
    assert!(s.receive_filtered(reply).is_err());
    assert!(s.filtered_messages().is_empty());
    assert_eq!(
        s.filtered_send(s.token().unwrap(), filtered::Kind::Normal, "再試".into())
            .unwrap()
            .sequence,
        2
    );
}
#[test]
fn bounded_history_deduplicates_and_preserves_categories_without_rewriting() {
    let mut s = ready();
    let mut r = read(&s);
    r.cursor = 64;
    r.messages = (1..=64).map(row).collect();
    s.receive_filtered(r).unwrap();
    let rows = s.filtered_rows.clone();
    s.receive_filtered(read(&s)).unwrap();
    assert!(Arc::ptr_eq(&rows, &s.filtered_rows));
    let mut r = read(&s);
    r.echo.after = 63;
    r.cursor = 65;
    r.messages = vec![row(64), row(65)];
    s.receive_filtered(r.clone()).unwrap();
    assert_eq!(s.filtered_messages().len(), 64);
    assert!(s.history_gap());
    assert!(s.messages().is_empty());
    let before = s.clone();
    r.messages[0].kind = filtered::Kind::Trade;
    assert!(s.receive_filtered(r).is_err());
    assert_eq!(s, before);
    let mut gap = read(&s);
    gap.cursor = 70;
    gap.messages = vec![row(70)];
    gap.truncated = true;
    s.receive_filtered(gap).unwrap();
    assert!(s.history_gap());
}
#[test]
fn owner_cursor_receipt_and_page_mismatches_are_atomic() {
    let mut s = ready();
    let mut first = read(&s);
    first.cursor = 1;
    first.messages = vec![row(1)];
    s.receive_filtered(first).unwrap();
    let before = s.clone();
    let mut rollback = read(&s);
    rollback.echo.after = 0;
    rollback.cursor = 0;
    assert!(s.receive_filtered(rollback).is_err());
    assert_eq!(s, before);
    let mut renamed = read(&s);
    renamed.owner.source_name = "假來源".into();
    assert!(s.receive_filtered(renamed).is_err());
    assert_eq!(s, before);
    let mut receipt = read(&s);
    receipt.echo = s
        .filtered_send(s.token().unwrap(), filtered::Kind::Trade, "內容".into())
        .unwrap();
    receipt.sequence = 1;
    assert!(s.receive_filtered(receipt.clone()).is_err()); // OK SEND requires its exact receipt, not a made-up self row.
    receipt.receipt = Some(filtered::Receipt {
        sequence: 1,
        sender: owner(),
        kind: filtered::Kind::World,
        text: "內容".into(),
    });
    assert!(s.receive_filtered(receipt).is_err());
    assert_eq!(s, before);
}
#[test]
fn navigation_preserves_profile_but_retires_old_input_history_and_private_admission() {
    let mut s = ready();
    let token = s.token().unwrap();
    let mut r = read(&s);
    r.cursor = 1;
    r.messages = vec![row(1)];
    s.receive_filtered(r).unwrap();
    s.retain_scope(Some(Scope {
        room_id: 42,
        ..scope()
    }));
    assert!(s.filtered() && s.waiting());
    assert!(s.filtered_messages().is_empty());
    assert!(s
        .filtered_send(token, filtered::Kind::Normal, "不能導向新房間".into())
        .is_err());
    let mut fresh = read(&s);
    fresh.membership = 2;
    s.receive_filtered(fresh).unwrap();
    assert_ne!(s.token(), Some(token));
    s.retain_scope(None);
    assert!(s.token().is_none() && s.filtered_messages().is_empty());
    s.align_profile(true, false, Some(scope()));
    assert!(!s.filtered());
    assert!(s.filtered_read().is_none() && s.read().is_some());
}
