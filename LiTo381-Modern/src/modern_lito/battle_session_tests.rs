use super::*;
use crate::modern_lito::battle_payload::{data, data_tests};

fn fixture() -> (protocol::Ack, read::UnvalidatedReply) {
    let data = data_tests::fixture();
    let player = &data.personal.player;
    let request = read::Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: player.source as u32,
                object_id: player.object as u64,
            },
            epoch: player.epoch as u64,
        },
        channel_id: player.channel as u8,
        world: data.world.identity.clone(),
    };
    (
        protocol::Ack {
            request_id: 1,
            version: 35,
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES,
        },
        read::UnvalidatedReply {
            echo: request,
            state: read::State::Data,
            data: Some(data),
        },
    )
}

#[test]
fn unnegotiated_session_never_requests_canonical_data() {
    let (ack, reply) = fixture();
    for ack in [
        protocol::Ack { version: 34, ..ack },
        protocol::Ack {
            capabilities: ack.capabilities & !protocol::CAP_BATTLE_DATA,
            ..ack
        },
    ] {
        let mut tracker = Tracker::new(ack);
        assert!(!tracker.align(Some(reply.echo.clone())).unwrap());
        assert!(tracker.read_request().is_none());
        assert_eq!(tracker.state(), State::default());
        assert!(tracker.accept(reply.clone()).is_err());
    }
}

#[test]
fn publication_is_atomic_immutable_redacted_and_not_a_receipt() {
    let (ack, reply) = fixture();
    let mut tracker = Tracker::new(ack);
    tracker.align(Some(reply.echo.clone())).unwrap();
    assert_eq!(tracker.state().status(), Some(read::State::Preparing));
    assert!(tracker.state().observation().is_none());
    tracker.accept(reply.clone()).unwrap();
    let old = tracker.state();
    assert_eq!(old.status(), Some(read::State::Data));
    assert_eq!(
        old.observation().unwrap().data(),
        reply.data.as_ref().unwrap()
    );
    assert!(!format!("{old:?}").contains("3333"));
    let mut invalid_scope = reply.echo.clone();
    invalid_scope.world.generation = 0;
    assert!(tracker.align(Some(invalid_scope)).is_err());
    assert_eq!(tracker.state(), old, "invalid replacement is atomic");
    for field in 0..7 {
        let mut bad = reply.clone();
        match field {
            0 => bad.echo.context.epoch += 1,
            1 => bad.echo.context.key.source_id += 1,
            2 => bad.echo.context.key.object_id += 1,
            3 => bad.echo.channel_id ^= 1,
            4 => bad.echo.world.generation += 1,
            5 => bad.data.as_mut().unwrap().world.revision -= 1,
            _ => bad.data.as_mut().unwrap().actors.0[0].position.x += 1,
        }
        assert!(tracker.accept(bad).is_err());
        assert_eq!(tracker.state(), old);
    }
    let mut next = reply.clone();
    next.data.as_mut().unwrap().world.loading_elapsed_nanos += 1;
    tracker.accept(next.clone()).unwrap();
    assert_ne!(tracker.state(), old);
    assert_eq!(
        old.observation().unwrap().data(),
        reply.data.as_ref().unwrap()
    );
    assert_eq!(
        tracker.state().observation().unwrap().data(),
        next.data.as_ref().unwrap()
    );
}

#[test]
fn every_scope_component_retires_old_private_data_and_rejects_late_replies() {
    let (ack, reply) = fixture();
    for field in 0..7 {
        let mut tracker = Tracker::new(ack);
        tracker.align(Some(reply.echo.clone())).unwrap();
        tracker.accept(reply.clone()).unwrap();
        let mut next = reply.echo.clone();
        match field {
            0 => next.context.epoch += 1,
            1 => next.context.key.source_id += 1,
            2 => next.context.key.object_id += 1,
            3 => next.channel_id ^= 1,
            4 => next.world.room += 1,
            5 => next.world.match_id += 1,
            _ => next.world.generation += 1,
        }
        assert!(tracker.align(Some(next.clone())).unwrap());
        assert_eq!(tracker.state().request(), Some(&next));
        assert!(tracker.state().observation().is_none());
        assert!(tracker.accept(reply.clone()).is_err());
        assert!(tracker.align(None).unwrap());
        assert!(tracker.read_request().is_none());
        assert!(tracker.accept(reply.clone()).is_err());
        assert!(tracker.align(Some(reply.echo.clone())).unwrap());
        assert!(tracker.state().observation().is_none());
        tracker.accept(reply.clone()).unwrap();
    }
}

#[test]
fn unavailable_stops_polling_until_a_new_attachment_and_v36_requires_presentation() {
    let (ack, reply) = fixture();
    let mut tracker = Tracker::new(ack);
    tracker.align(Some(reply.echo.clone())).unwrap();
    tracker
        .accept(read::UnvalidatedReply {
            echo: reply.echo.clone(),
            state: read::State::Unavailable,
            data: None,
        })
        .unwrap();
    assert!(tracker.read_request().is_none());
    assert!(!tracker.align(Some(reply.echo.clone())).unwrap());
    assert!(tracker.accept(reply.clone()).is_err());
    tracker.align(None).unwrap();
    tracker.align(Some(reply.echo.clone())).unwrap();
    assert!(tracker.read_request().is_some());
    let mut tracker = Tracker::new(protocol::Ack { version: 36, ..ack });
    tracker.align(Some(reply.echo.clone())).unwrap();
    assert!(tracker.accept(reply.clone()).is_err());
    let mut next = reply;
    let d = next.data.as_mut().unwrap();
    d.presentation = Some(data::List(
        d.actors
            .0
            .iter()
            .map(|a| data::PublicPresentation {
                entity: a.entity,
                shot: None,
                item: None,
            })
            .collect(),
    ));
    tracker.accept(next).unwrap();
    assert!(tracker.state().observation().is_some());
    let mut published = tracker.state();
    published.retain(None);
    assert_eq!(published, State::default());
}
