use super::*;
fn request(d: &UnvalidatedData) -> read::Request {
    read::Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: d.personal.player.source as u32,
                object_id: d.personal.player.object as u64,
            },
            epoch: d.personal.player.epoch as u64,
        },
        channel_id: d.personal.player.channel as u8,
        world: d.world.identity.clone(),
    }
}
fn ack() -> protocol::Ack {
    protocol::Ack {
        request_id: 1,
        version: 35,
        numeric_schema: 1,
        capabilities: protocol::RUNTIME_CAPABILITIES | protocol::CAP_BATTLE_DATA,
    }
}
fn reply(d: &UnvalidatedData) -> read::UnvalidatedReply {
    read::UnvalidatedReply {
        echo: request(d),
        state: read::State::Data,
        data: Some(d.clone()),
    }
}
// The fixture is provided by the existing codec tests, not a new legacy Frame.
#[test]
fn preparing_unavailable_are_service_states_and_cannot_resurrect() {
    let d = crate::modern_lito::battle_payload::data_tests::fixture();
    let r = request(&d);
    let mut f = Follower::new(r.clone(), ack()).unwrap();
    for state in [
        read::State::Preparing,
        read::State::Preparing,
        read::State::Unavailable,
        read::State::Unavailable,
    ] {
        f.accept(read::UnvalidatedReply {
            echo: r.clone(),
            state,
            data: None,
        })
        .unwrap();
    }
    assert!(f.accept(reply(&d)).is_err());
    assert_eq!(f.state(), read::State::Unavailable);
    assert!(f.observation().is_none());
}
#[test]
fn same_engine_clock_observation_is_allowed_but_mutation_and_clock_regression_are_atomic() {
    let d = crate::modern_lito::battle_payload::data_tests::fixture();
    let mut f = Follower::new(request(&d), ack()).unwrap();
    f.accept(reply(&d)).unwrap();
    let retained = f.observation().unwrap().clone();
    let mut next = d.clone();
    next.world.loading_elapsed_nanos = 10;
    f.accept(reply(&next)).unwrap();
    assert_eq!(retained.data().world.loading_elapsed_nanos, 0);
    assert!(f.accept(reply(&d)).is_err());
    assert_eq!(f.observation().unwrap().data(), &next);
    let mut bad = next.clone();
    bad.actors.0[0].heading = 1;
    assert!(f.accept(reply(&bad)).is_err());
    assert_eq!(f.observation().unwrap().data(), &next);
    assert_eq!(format!("{f:?}"), "BattleFollower[redacted]");
    let mut foreign = reply(&next);
    foreign.echo.context.epoch = 7;
    assert!(f.accept(foreign).is_err());
    assert_eq!(f.observation().unwrap().data(), &next);
    assert!(f
        .accept(read::UnvalidatedReply {
            echo: request(&d),
            state: read::State::Preparing,
            data: None
        })
        .is_err());
}
