use super::*;
use crate::modern_lito::protocol::{Ack, KNOWN_CAPABILITIES};
use std::io::Cursor;
fn peer(bits: u32) -> Selected {
    Selected::new(
        Ack {
            request_id: 1,
            version: 47,
            numeric_schema: 1,
            capabilities: KNOWN_CAPABILITIES,
        },
        bits,
    )
    .unwrap()
}
fn request() -> base::Request {
    base::Request::read(
        app::SelectionContext {
            key: app::CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: 1,
        },
        0,
        None,
    )
}
fn detached(q: &[u8]) -> Vec<u8> {
    let mut r = q.to_vec();
    r.resize(base::MIN_RESULT_BYTES, 0);
    r[4..8].copy_from_slice(&(base::MIN_RESULT_BYTES as u32).to_be_bytes());
    r[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
    r
}
#[test]
fn explicit_selection_and_request_validation_happen_before_io() {
    let q = encode_request(1, &request(), peer(80)).unwrap();
    let r = detached(&q);
    for bits in [0, 16, 48, 63] {
        assert!(encode_request(1, &request(), peer(bits)).is_err());
        let mut input = Cursor::new(&r);
        assert!(read_reply(&mut input, &q, peer(bits)).is_err());
        assert_eq!(input.position(), 0);
    }
    for offset in [0, 4, 8, 12, 14, 16, 28, 36, 40, 41, 48, 64, 80] {
        let mut bad = q.clone();
        bad[offset] ^= 0xff;
        let mut input = Cursor::new(&r);
        // Nonzero request IDs and positive identities remain legal but must
        // echo exactly. Reserved/header/action/digest mutations fail pre-I/O.
        assert!(read_reply(&mut input, &bad, peer(112)).is_err());
    }
    for n in 0..q.len() {
        let mut input = Cursor::new(&r);
        assert!(read_reply(&mut input, &q[..n], peer(112)).is_err());
        assert_eq!(input.position(), 0);
    }
}
#[test]
fn bounded_reply_consumes_one_frame_and_rejects_legacy_opcode() {
    let q = encode_request(1, &request(), peer(112)).unwrap();
    let r = detached(&q);
    let mut input = Cursor::new([r.clone(), r.clone()].concat());
    let reply = read_reply(&mut input, &q, peer(112)).unwrap();
    assert_eq!(input.position(), 168);
    assert_eq!(reply.state(), base::State::Detached);
    assert!(reply.data().is_none());
    assert_eq!(reply.echo(), &request());
    assert_eq!(reply.status(), super::super::world_protocol::Status::Ok);
    assert_eq!(reply.reason(), super::super::world_protocol::Reason::None);
    for n in 0..r.len() {
        assert!(read_reply(&mut Cursor::new(&r[..n]), &q, peer(112)).is_err());
    }
    for offset in [
        0, 4, 8, 12, 14, 16, 40, 112, 113, 114, 115, 116, 140, 148, 149, 150, 151, 152, 160, 164,
    ] {
        let mut bad = r.clone();
        bad[offset] ^= 0xff;
        assert!(
            read_reply(&mut Cursor::new(bad), &q, peer(112)).is_err(),
            "offset {offset}"
        );
    }
    for n in [0, 16, 167, MAX_RESULT_BYTES as u32 + 1, u32::MAX] {
        let mut bad = r.clone();
        bad[4..8].copy_from_slice(&n.to_be_bytes());
        let mut input = Cursor::new(bad);
        assert!(read_reply(&mut input, &q, peer(112)).is_err());
        assert_eq!(input.position(), 16);
    }
    let legacy = base::encode_request(1, &request(), 47).unwrap();
    assert!(base::read_reply_selected(&mut Cursor::new(&r), &legacy, peer(112)).is_err());
    let mut old = r.clone();
    old[12..14].copy_from_slice(&base::RESULT_KIND.to_be_bytes());
    assert!(read_reply(&mut Cursor::new(&old), &q, peer(112)).is_err());
    assert!(base::read_reply_selected(&mut Cursor::new(&old), &legacy, peer(112)).is_ok());
}
#[test]
fn preparing_identity_and_lifecycle_must_match_exact_request() {
    let mut request = request();
    request.action = base::Action::Prepare;
    request.room = 9;
    request.room_revision = 5;
    let q = encode_request(4, &request, peer(80)).unwrap();
    let mut r = detached(&q);
    r[114] = 1;
    for (at, n) in [(116, 9i64), (124, 20), (132, 6), (140, 1)] {
        r[at..at + 8].copy_from_slice(&n.to_be_bytes());
    }
    r[148] = super::super::world_protocol::Phase::Preparing as u8;
    let decoded = read_reply(&mut Cursor::new(&r), &q, peer(80)).unwrap();
    assert_eq!(decoded.state(), base::State::BeforeEngine);
    for at in [116, 124, 132, 140, 148, 149, 150, 151, 152] {
        let mut bad = r.clone();
        bad[at] ^= 0xff;
        assert!(
            read_reply(&mut Cursor::new(bad), &q, peer(80)).is_err(),
            "offset {at}"
        );
    }
}

fn preparation(match_id: i64) -> Reply {
    let mut q = request();
    q.action = base::Action::Prepare;
    q.room = 9;
    q.room_revision = 5;
    Reply(base::Reply {
        echo: q,
        status: super::super::world_protocol::Status::Ok,
        reason: super::super::world_protocol::Reason::None,
        state: base::State::BeforeEngine,
        attachment: Some(base::Attachment {
            world: payload::data::WorldId {
                room: 9,
                match_id,
                generation: 6,
            },
            lifecycle: super::super::battle_operation::Lifecycle {
                revision: 1,
                phase: super::super::world_protocol::Phase::Preparing,
                abort: super::super::world_protocol::AbortReason::None,
                settlement: None,
                race_settlement: None,
                race_terminal_revision: 0,
            },
        }),
        data: None,
    })
}
#[test]
fn vip_preparation_family_retirement_and_match_watermark_are_atomic() {
    use super::super::{battle_state::Tracker, world_protocol as w};
    use std::sync::Arc;
    let mut t = Tracker::default();
    let r = preparation(20);
    t.accept_vip_world(&r, peer(112)).unwrap();
    let retained = t.snapshot().unwrap().clone();
    assert!(retained.full_vip());
    assert!(retained.observation().is_none());
    let read = base::Request::read(request().context, 0, Some(&r.attachment().unwrap().world));
    assert!(t.admits_vip_world(&read));
    assert!(!t.admits_world(&read));
    assert!(t.accept_world(&r.0, peer(112)).is_err());
    assert!(t.accept_vip_world(&r, peer(112)).is_err());
    assert!(Arc::ptr_eq(&retained, t.snapshot().unwrap()));
    let mut leave = r.clone();
    leave.0.echo = base::Request {
        action: base::Action::Leave,
        ..read.clone()
    };
    leave.0.state = base::State::Detached;
    leave.0.attachment = None;
    let mut wrong = leave.clone();
    wrong.0.echo.context.epoch += 1;
    assert!(t.accept_vip_world(&wrong, peer(112)).is_err());
    assert!(Arc::ptr_eq(&retained, t.snapshot().unwrap()));
    t.accept_vip_world(&leave, peer(112)).unwrap();
    assert!(t.snapshot().unwrap().attachment().is_none());
    assert!(!t.admits_vip_world(&read));
    assert!(t.accept_vip_world(&r, peer(112)).is_err());
    t.retire();
    assert!(t.snapshot().is_none());
    assert!(!t.admits_vip_world(&read));
    t.accept_vip_world(&preparation(21), peer(112)).unwrap();
    assert_eq!(
        retained.attachment().unwrap().lifecycle.phase,
        w::Phase::Preparing
    );
    let mut old = Tracker::default();
    old.accept_world(&r.0, peer(112)).unwrap();
    let old_arc = old.snapshot().unwrap().clone();
    assert!(!old.admits_vip_world(&read));
    assert!(old.accept_vip_world(&r, peer(112)).is_err());
    assert!(Arc::ptr_eq(&old_arc, old.snapshot().unwrap()));
}
#[test]
fn aborted_pre_engine_receipt_freezes_chronology_without_manufacturing_data() {
    use super::super::{battle_state::Tracker, world_protocol as w};
    use std::sync::Arc;
    let mut t = Tracker::default();
    let r = preparation(20);
    t.accept_vip_world(&r, peer(112)).unwrap();
    let mut aborted = r.clone();
    aborted.0.echo =
        base::Request::read(request().context, 0, Some(&r.attachment().unwrap().world));
    let l = &mut aborted.0.attachment.as_mut().unwrap().lifecycle;
    l.revision = 2;
    l.phase = w::Phase::Aborted;
    l.abort = w::AbortReason::Disconnected;
    t.accept_vip_world(&aborted, peer(112)).unwrap();
    let retained = t.snapshot().unwrap().clone();
    assert!(retained.observation().is_none() && retained.presentation_observation().is_none());
    for at in 0..5 {
        let mut bad = aborted.clone();
        match at {
            0 => bad.0.attachment.as_mut().unwrap().lifecycle.revision = 1,
            1 => bad.0.attachment.as_mut().unwrap().lifecycle.abort = w::AbortReason::None,
            2 => bad.0.attachment.as_mut().unwrap().world.match_id += 1,
            3 => bad.0.echo.context.key.source_id = 42,
            _ => {
                let l = &mut bad.0.attachment.as_mut().unwrap().lifecycle;
                l.revision = 3;
                l.phase = w::Phase::Preparing;
                l.abort = w::AbortReason::None;
            }
        }
        assert!(t.accept_vip_world(&bad, peer(112)).is_err());
        assert!(Arc::ptr_eq(&retained, t.snapshot().unwrap()));
    }
    assert!(t.accept_vip_world(&aborted, peer(48)).is_err());
    assert!(Arc::ptr_eq(&retained, t.snapshot().unwrap()));
}
