//! Production v46 decode + atomic Follower with real Java publications.
//! A read is not TLS authentication, a cast receipt or renderer permission.
use super::{payload, speed};
use crate::modern_lito::{
    application_protocol as app, battle_follow::Follower, battle_read as read, protocol,
};
pub(super) fn attached(p: &speed::Candidate) -> payload::UnvalidatedData {
    let bytes = speed::encode(p).unwrap();
    let d = payload::decode_versioned(&bytes, 46).unwrap();
    assert_eq!(d.speed.as_ref().unwrap().state, p.effects);
    assert!(d.haste.is_none() && d.npc_motion.is_none());
    assert_eq!(payload::encode(&d).unwrap(), bytes);
    let wrong = payload::npc_motion::Candidate {
        data: d.clone(),
        motion: None,
    };
    assert!(wrong
        .check_values(&d.personal.player, &d.world.identity)
        .is_err());
    assert!(payload::npc_motion::encode(&wrong).is_err());
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
fn follower(p: &speed::Candidate, version: u16, caps: u32) -> Follower {
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
pub(super) fn values(p: &speed::Candidate) {
    let d = attached(p);
    let mut f = follower(p, 46, protocol::KNOWN_CAPABILITIES);
    f.accept(reply(d.clone())).unwrap();
    let keep = f.observation().unwrap().clone();
    assert_eq!(keep.data(), &d);
    assert_eq!(keep.personal().player(), &d.personal.player);
    assert!(f.accept(reply(p.data.clone())).is_err());
    for version in 35..46 {
        assert!(follower(
            p,
            version,
            protocol::KNOWN_CAPABILITIES & !protocol::CAP_SLOW_DATA
        )
        .accept(reply(d.clone()))
        .is_err());
    }
    for bit in [
        protocol::CAP_SLOW_DATA,
        protocol::CAP_TIMED,
        protocol::CAP_ACTION_LIFE,
        protocol::CAP_MATCH_DESCRIPTOR,
    ] {
        assert!(follower(p, 46, protocol::KNOWN_CAPABILITIES & !bit)
            .accept(reply(d.clone()))
            .is_err());
    }
    assert!(Follower::new(
        reply(d.clone()).echo,
        protocol::Ack {
            request_id: 1,
            version: 46,
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES & !protocol::CAP_BATTLE_DATA
        }
    )
    .is_err());
    let mut missing = d.clone();
    missing.speed = None;
    assert!(f.accept(reply(missing)).is_err());
    let mut dual = d.clone();
    dual.npc_motion = Some(payload::npc_motion::Envelope { state: None });
    assert!(payload::encode(&dual).is_err());
    assert!(f.accept(reply(dual)).is_err());
    let mut wrong = reply(d);
    wrong.echo.context.epoch ^= 1;
    assert!(f.accept(wrong).is_err());
    assert_eq!(f.observation().unwrap().data(), keep.data());
    assert_eq!(f.observation().unwrap().personal(), keep.personal());
}
pub(super) fn transition(a: &speed::Candidate, b: &speed::Candidate) {
    let mut f = follower(a, 46, protocol::KNOWN_CAPABILITIES);
    f.accept(reply(attached(a))).unwrap();
    let old = f.observation().unwrap().clone();
    f.accept(reply(attached(b))).unwrap();
    let new = f.observation().unwrap().clone();
    old.check_successor(&new).unwrap();
    assert_eq!(new.data().speed.as_ref().unwrap().state, b.effects);
    if a.data.world.revision < b.data.world.revision {
        assert!(f.accept(reply(attached(a))).is_err());
        assert_eq!(f.observation().unwrap().data(), new.data());
    }
}
pub(super) fn reject(a: &speed::Candidate, b: &speed::Candidate) {
    let mut f = follower(a, 46, protocol::KNOWN_CAPABILITIES);
    f.accept(reply(attached(a))).unwrap();
    let keep = f.observation().unwrap().clone();
    // Syntactic errors may fail even before admission. Otherwise, the same
    // production Follower must reject atomically, not partially install stock.
    if let Ok(bytes) = speed::encode(b) {
        let d = payload::decode_versioned(&bytes, 46).unwrap();
        assert!(f.accept(reply(d)).is_err());
    } else {
        assert!(b
            .check_values(&a.data.personal.player, &a.data.world.identity)
            .is_err());
    }
    assert_eq!(f.observation().unwrap().data(), keep.data());
    assert_eq!(f.observation().unwrap().personal(), keep.personal());
}
