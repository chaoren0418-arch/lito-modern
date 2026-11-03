//! Actual Java engine payloads into the ordinary immutable Rust observation.
//! This is selected-format admission, not TLS/controller/UI acceptance.
use super::Peer;
use crate::modern_lito::{
    application_protocol as app,
    battle_follow::Follower,
    battle_payload as payload, battle_read as read,
    protocol::{self, extensions::Selected},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn selected(ext: u32) -> Selected {
    Selected::new(
        protocol::Ack {
            request_id: 1,
            version: 47,
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES,
        },
        ext,
    )
    .unwrap()
}
pub(super) fn unhex(text: &str) -> Vec<u8> {
    assert!(text.len() % 2 == 0 && text.len() <= 2 * (payload::healing::MAX_BYTES + 4096));
    text.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
pub(super) fn request(d: &payload::UnvalidatedData) -> read::Request {
    let p = &d.personal.player;
    read::Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: p.source as u32,
                object_id: p.object as u64,
            },
            epoch: p.epoch as u64,
        },
        channel_id: p.channel as u8,
        world: d.world.identity.clone(),
    }
}
fn reply(d: payload::UnvalidatedData) -> read::UnvalidatedReply {
    read::UnvalidatedReply {
        echo: request(&d),
        state: read::State::Data,
        data: Some(d),
    }
}
fn reject(f: &mut Follower, bad: payload::UnvalidatedData) {
    let before = f.observation().unwrap().clone();
    assert!(f.accept(reply(bad)).is_err());
    assert_eq!(f.state(), read::State::Data);
    assert_eq!(f.observation().unwrap().data(), before.data());
    assert_eq!(f.observation().unwrap().personal(), before.personal());
}

#[test]
fn selected_extensions_require_full_base_and_do_not_upgrade_legacy_versions() {
    for ext in [0, 1, 3, 4, 5, 7] {
        assert_eq!(selected(ext).extensions(), ext);
    }
    for ext in [2, 6, 8, u32::MAX] {
        assert!(Selected::new(selected(0).base(), ext).is_err());
    }
    for version in [0, 1, 35, 43, 46, 48, u16::MAX] {
        assert!(Selected::new(
            protocol::Ack {
                version,
                ..selected(0).base()
            },
            1
        )
        .is_err());
    }
    for bit in 0..32 {
        if protocol::extensions::DEPENDENCIES & (1 << bit) != 0 {
            assert!(Selected::new(
                protocol::Ack {
                    capabilities: protocol::KNOWN_CAPABILITIES & !(1 << bit),
                    ..selected(0).base()
                },
                1
            )
            .is_err());
        }
    }
    for numeric_schema in [0, 2, u16::MAX] {
        assert!(Selected::new(
            protocol::Ack {
                numeric_schema,
                ..selected(0).base()
            },
            1
        )
        .is_err());
    }
    assert!(Selected::new(
        protocol::Ack {
            request_id: 0,
            ..selected(0).base()
        },
        1
    )
    .is_err());
    assert!(!selected(0).allows_data());
    assert!(selected(1).allows_data());
    assert!(!selected(1).allows_commands());
    assert!(selected(3).allows_commands());
    assert_eq!(
        payload::Format::Selected(selected(0)).maximum(),
        payload::MAX_WIRE_BYTES
    );
    assert_eq!(
        payload::Format::Selected(selected(1)).maximum(),
        payload::healing::MAX_BYTES
    );
}

#[test]
#[ignore = "explicit Java engine; complete negotiated snapshot admission and atomic catchup, not live TLS/UI"]
fn java_healing_selected_complete_observation_and_atomic_catchup() {
    let mut peer = Peer::start_named(Some("healing7-scene"));
    let mut followers = BTreeMap::new();
    let mut streams: BTreeMap<(String, i32), Vec<payload::UnvalidatedData>> = BTreeMap::new();
    let mut status = BTreeSet::new();
    let mut count = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:66");
            break;
        }
        let fields = line.splitn(3, ':').collect::<Vec<_>>();
        assert_eq!(fields[0], "FRAME");
        let label = fields[1];
        let raw = unhex(fields[2]);
        let c = payload::healing::decode(&raw).unwrap();
        let d = payload::decode_selected(&raw, selected(1)).unwrap();
        assert_eq!(payload::decode_selected(&raw, selected(3)).unwrap(), d);
        assert_eq!(
            payload::encode(&d).unwrap(),
            raw,
            "complete roundtrip {label}"
        );
        assert_eq!(d.personal, c.data.personal);
        assert_eq!(d.actors, c.data.actors);
        assert_eq!(d.healing.as_ref().unwrap().actors, c.actors);
        match &c.status {
            payload::healing::Status::Motion(m) => {
                assert_eq!(d.motion_state(), Some(m));
                status.insert("motion");
            }
            payload::healing::Status::Haste(h) => {
                assert_eq!(d.haste_state(), Some(h));
                status.insert("haste");
            }
            payload::healing::Status::Speed(s) => {
                assert_eq!(d.speed_state(), Some(s));
                status.insert("speed");
            }
        }
        assert!(payload::decode_selected(&raw, selected(0)).is_err());
        for version in 1..=47 {
            assert!(payload::decode_versioned(&raw, version).is_err());
        }
        let mut old = Follower::new(request(&d), selected(0).base()).unwrap();
        assert!(old.accept(reply(d.clone())).is_err());
        assert!(old.observation().is_none());
        let group = if label.starts_with("status1") {
            "status1"
        } else if label.starts_with("status2") {
            "status2"
        } else if label.starts_with("gear-") {
            "gear"
        } else if label.starts_with("regen-") {
            "regen"
        } else if label.starts_with("npc-") {
            "npc"
        } else {
            "basic"
        };
        let key = (group.to_owned(), d.personal.player.source);
        let checked = c.checked(&d.personal.player, &d.world.identity).unwrap();
        for ext in [1, 3] {
            let f = followers
                .entry((key.clone(), ext))
                .or_insert_with(|| Follower::new_selected(request(&d), selected(ext)).unwrap());
            f.accept(reply(d.clone()))
                .unwrap_or_else(|_| panic!("selected successor {label}/{ext}"));
            assert_eq!(f.observation().unwrap().data(), &d);
            assert_eq!(f.observation().unwrap().personal(), checked.personal());
            for change in [
                (|v: &mut payload::UnvalidatedData| v.healing = None)
                    as fn(&mut payload::UnvalidatedData),
                |v| v.world.descriptor.features &= !payload::healing::FEATURE,
                |v| v.healing.as_mut().unwrap().actors.0[0].incoming.restored += 1,
                |v| v.world.loading_elapsed_nanos = -1,
                |v| v.actors.0[0].heading = 9,
                |v| v.actors.0[0].position.x = i32::MAX,
                |v| v.personal.player.epoch ^= 2,
                |v| v.world.identity.generation += 1,
                |v| v.personal.history = None,
                |v| v.presentation = None,
                |v| v.npc_motion = Some(payload::npc_motion::Envelope { state: None }),
            ] {
                let mut bad = d.clone();
                change(&mut bad);
                reject(f, bad);
            }
        }
        streams.entry(key).or_default().push(d);
        peer.ack(&raw);
        count += 1;
    }
    assert_eq!(count, 66);
    assert_eq!(status, BTreeSet::from(["motion", "haste", "speed"]));
    peer.finish();
    let mut pairs = 0;
    for frames in streams.values() {
        for (i, a) in frames.iter().enumerate() {
            for b in frames.iter().skip(i) {
                let mut f = Follower::new_selected(request(a), selected(3)).unwrap();
                f.accept(reply(a.clone())).unwrap();
                let held = f.observation().unwrap().clone();
                f.accept(reply(b.clone())).unwrap();
                held.check_successor(f.observation().unwrap()).unwrap();
                assert_eq!(held.data(), a);
                if a.world.revision < b.world.revision {
                    reject(&mut f, a.clone());
                }
                pairs += 1;
            }
        }
    }
    assert_eq!(pairs, 232);
}
