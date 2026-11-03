//! Exercise production decoding/Follower with real Java publications, not an
//! alternative admission implementation. TLS owner authentication is separate.
use super::{data, haste, payload};
use crate::modern_lito::{
    application_protocol as app, battle_follow::Follower, battle_read as read, protocol,
};

fn version(p: &haste::Candidate) -> u16 {
    haste::minimum_version(p.data.world.descriptor.features)
}
pub(super) fn attached(p: &haste::Candidate) -> payload::UnvalidatedData {
    let bytes = haste::encode(p).unwrap();
    let d = payload::decode_versioned(&bytes, version(p)).unwrap();
    assert_eq!(d.haste.as_ref().unwrap().state, p.effects);
    assert!(d.npc_motion.is_none());
    assert_eq!(payload::encode(&d).unwrap(), bytes);
    d
}
pub(super) fn reply(d: payload::UnvalidatedData) -> read::UnvalidatedReply {
    let p = &d.personal.player;
    read::UnvalidatedReply {
        echo: read::Request {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: p.source as u32,
                    object_id: p.object as u64,
                },
                epoch: p.epoch as u64,
            },
            channel_id: p.channel as u8,
            world: d.world.identity.clone(),
        },
        state: read::State::Data,
        data: Some(d),
    }
}
pub(super) fn follower(p: &haste::Candidate, version: u16, caps: u32) -> Follower {
    Follower::new(
        reply(p.data.clone()).echo,
        protocol::Ack {
            request_id: 1,
            version,
            numeric_schema: 1,
            capabilities: caps,
        },
    )
    .unwrap()
}
pub(super) fn values(p: &haste::Candidate) {
    let d = attached(p);
    let mut f = follower(p, version(p), protocol::KNOWN_CAPABILITIES);
    f.accept(reply(d.clone())).unwrap();
    let keep = f.observation().unwrap().clone();
    assert_eq!(keep.data(), &d);
    assert_eq!(keep.personal().player(), &d.personal.player);
    assert!(f.accept(reply(p.data.clone())).is_err());
    assert_eq!(f.observation().unwrap().data(), keep.data());
    for version in 35..version(p) {
        assert!(follower(
            p,
            version,
            protocol::KNOWN_CAPABILITIES & !protocol::CAP_HASTE
        )
        .accept(reply(d.clone()))
        .is_err());
    }
    if version(p) == 45 {
        assert!(follower(p, 44, protocol::KNOWN_CAPABILITIES)
            .accept(reply(d.clone()))
            .is_err());
    }
    for bit in [
        protocol::CAP_HASTE,
        protocol::CAP_TIMED,
        protocol::CAP_BATTLE_DATA,
        protocol::CAP_ACTION_LIFE,
        protocol::CAP_MATCH_DESCRIPTOR,
    ] {
        if bit == protocol::CAP_BATTLE_DATA {
            assert!(Follower::new(
                reply(d.clone()).echo,
                protocol::Ack {
                    request_id: 1,
                    version: version(p),
                    numeric_schema: 1,
                    capabilities: protocol::KNOWN_CAPABILITIES & !bit
                }
            )
            .is_err());
        } else {
            assert!(follower(p, version(p), protocol::KNOWN_CAPABILITIES & !bit)
                .accept(reply(d.clone()))
                .is_err());
        }
    }
    let mut missing = d.clone();
    missing.haste = None;
    assert!(f.accept(reply(missing)).is_err());
    let mut dual = d.clone();
    dual.npc_motion = Some(payload::npc_motion::Envelope { state: None });
    assert!(payload::encode(&dual).is_err());
    assert!(f.accept(reply(dual)).is_err());
    let mut wrong = reply(d);
    wrong.echo.context.epoch ^= 1;
    assert!(f.accept(wrong).is_err());
    assert_eq!(f.observation().unwrap().data(), keep.data());
    assert_eq!(f.state(), read::State::Data);
}
pub(super) fn transition(a: &haste::Candidate, b: &haste::Candidate) {
    let mut f = follower(a, version(a), protocol::KNOWN_CAPABILITIES);
    f.accept(reply(attached(a))).unwrap();
    let old = f.observation().unwrap().clone();
    f.accept(reply(attached(b))).unwrap();
    let new = f.observation().unwrap().clone();
    old.check_successor(&new).unwrap();
    assert_eq!(new.data().haste.as_ref().unwrap().state, b.effects);
    if a.data.world.revision < b.data.world.revision {
        assert!(f.accept(reply(attached(a))).is_err());
        assert_eq!(f.observation().unwrap().data(), new.data());
    }
}
pub(super) fn reject(a: &haste::Candidate, bad: &haste::Candidate) {
    let mut f = follower(a, version(a), protocol::KNOWN_CAPABILITIES);
    f.accept(reply(attached(a))).unwrap();
    let keep = f.observation().unwrap().clone();
    assert!(f.accept(reply(attached(bad))).is_err());
    assert_eq!(f.state(), read::State::Data);
    assert_eq!(f.observation().unwrap().data(), keep.data());
    assert_eq!(f.observation().unwrap().personal(), keep.personal());
}
