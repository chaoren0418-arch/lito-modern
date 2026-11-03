//! Real service publication after durable gear changes -> v39 passive world
//! and SAVED lifecycle bytes. Not a network SET, SQL writer or UI acceptance.
use super::*;
use crate::modern_lito::{battle_world as w, world_protocol};

fn unhex(text: &str, max: usize) -> Vec<u8> {
    assert!(text.len() % 2 == 0 && text.len() <= max * 2);
    text.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
#[test]
#[ignore = "explicit Java v39 world lifecycle after durable gear; passive SAVED refresh and departure, no TLS/SQL/GUI"]
fn java_v39_passive_world_mixed_equipment_saved_and_departure() {
    let mut peer = Peer::start_named(Some("world-lifecycle"));
    let mut replies = Vec::new();
    let ack = crate::modern_lito::protocol::Ack {
        request_id: 1,
        version: 39,
        numeric_schema: 1,
        capabilities: crate::modern_lito::protocol::KNOWN_CAPABILITIES,
    };
    let mut owners = [
        crate::modern_lito::battle_state::Tracker::default(),
        crate::modern_lito::battle_state::Tracker::default(),
    ];
    for label in [
        "mixed-read-a",
        "mixed-read-b",
        "mixed-loaded",
        "saved-read-a",
        "leave-a",
        "saved-read-b",
        "detached-a",
        "leave-b",
    ] {
        let line = peer.line();
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "WORLD");
        assert_eq!(fields[1], label);
        let expected = unhex(fields[2], w::REQUEST_BYTES);
        let bytes = unhex(fields[3], w::MAX_RESULT_BYTES);
        let reply = w::read_reply(&mut bytes.as_slice(), &expected, 39).unwrap();
        let owner = usize::from(matches!(label, "mixed-read-b" | "saved-read-b" | "leave-b"));
        owners[owner]
            .accept_world(&reply, ack)
            .expect("real mixed-gear lifecycle chronology");
        let snapshot = owners[owner].snapshot().unwrap().clone();
        assert!(snapshot.attachment() == reply.attachment.as_ref());
        assert_eq!(
            snapshot.observation().map(|o| o.data()),
            reply.data.as_ref()
        );
        if label.starts_with("saved-read") {
            let mut regressed = reply.clone();
            let life = &mut regressed.attachment.as_mut().unwrap().lifecycle;
            life.revision += 1;
            life.settlement = Some(world_protocol::SettlementState::Pending);
            assert!(owners[owner].accept_world(&regressed, ack).is_err());
            assert!(std::sync::Arc::ptr_eq(
                &snapshot,
                owners[owner].snapshot().unwrap()
            ));
            let mut changed = reply.clone();
            changed.data.as_mut().unwrap().world.loading_elapsed_nanos += 1;
            changed.data.as_mut().unwrap().world.active_elapsed_nanos += 1;
            assert!(owners[owner].accept_world(&changed, ack).is_err());
            assert!(std::sync::Arc::ptr_eq(
                &snapshot,
                owners[owner].snapshot().unwrap()
            ));
        }
        assert_eq!(reply.status, world_protocol::Status::Ok);
        assert_eq!(
            w::encode_request(
                u32::from_be_bytes(expected[8..12].try_into().unwrap()),
                &reply.echo,
                39
            )
            .unwrap(),
            expected
        );
        if let Some(d) = &reply.data {
            assert!(d.presentation.is_some());
            assert_eq!(&bytes[w::MIN_RESULT_BYTES..], encode(d).unwrap());
            assert_eq!(
                d.personal.player.source as u32,
                reply.echo.context.key.source_id
            );
        }
        peer.ack(&bytes);
        replies.push(reply);
    }
    assert_eq!(peer.line(), "CLOSED:8");
    peer.finish();
    for (owner, initial) in [(0, 0), (1, 1)] {
        assert!(owners[owner].snapshot().unwrap().attachment().is_none());
        assert!(
            owners[owner].accept_world(&replies[initial], ack).is_err(),
            "departed world cannot resurrect"
        );
    }
    for index in [0, 2, 3] {
        assert_eq!(
            replies[index]
                .data
                .as_ref()
                .unwrap()
                .personal
                .equipment
                .as_ref()
                .unwrap()
                .revision,
            1
        );
    }
    assert_ne!(
        replies[0].data.as_ref().unwrap().personal.player,
        replies[1].data.as_ref().unwrap().personal.player
    );
    assert_eq!(
        replies[0].data.as_ref().unwrap().actors,
        replies[1].data.as_ref().unwrap().actors
    );
    assert_eq!(replies[2].echo.action, w::Action::Loaded);
    assert_eq!(
        replies[0].attachment.as_ref().unwrap().lifecycle.phase,
        world_protocol::Phase::Active
    );
    assert!(replies[0].data.as_ref().unwrap().result.is_none());
    for index in [3, 5] {
        let r = &replies[index];
        let d = r.data.as_ref().unwrap();
        let life = &r.attachment.as_ref().unwrap().lifecycle;
        assert_eq!(life.phase, world_protocol::Phase::Finished);
        assert_eq!(
            life.settlement,
            Some(world_protocol::SettlementState::Saved)
        );
        assert!(life.revision > d.world.revision + 1);
        assert!(d.result.is_some());
    }
    assert_eq!(
        replies[3].data.as_ref().unwrap().result,
        replies[5].data.as_ref().unwrap().result
    );
    for index in [4, 6, 7] {
        assert_eq!(replies[index].state, w::State::Detached);
        assert!(replies[index].attachment.is_none() && replies[index].data.is_none());
    }
}
