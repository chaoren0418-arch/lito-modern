//! Complete real-engine candidate scenes, not negotiated transport/UI admission.
use super::Peer;
use crate::modern_lito::battle_payload::healing as codec;
use std::collections::{BTreeMap, BTreeSet};

fn rejected(follower: &mut codec::Follower, bad: codec::Candidate, label: &str) {
    let saved = follower.current().unwrap().clone();
    assert!(follower.accept(bad).is_err(), "reject {label}");
    assert_eq!(
        follower.current().unwrap().candidate(),
        saved.candidate(),
        "atomic scene {label}"
    );
    assert_eq!(
        follower.current().unwrap().personal(),
        saved.personal(),
        "atomic private facts {label}"
    );
}

#[test]
#[ignore = "explicit compiled Java real engine; full schema7 values and temporal proof"]
fn java_healing_schema7_complete_scene_atomic_follower_and_catchup() {
    let mut peer = Peer::start_named(Some("healing7-scene"));
    let mut streams: BTreeMap<(String, i32), Vec<codec::Candidate>> = BTreeMap::new();
    let mut followers = BTreeMap::new();
    let mut count = 0;
    let mut coverage = BTreeSet::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:66");
            break;
        }
        let fields: Vec<_> = line.splitn(3, ':').collect();
        assert_eq!(fields[0], "FRAME");
        let label = fields[1];
        let raw: Vec<u8> = fields[2]
            .as_bytes()
            .chunks_exact(2)
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
            .collect();
        let c = codec::decode(&raw).unwrap();
        let encoded = codec::encode(&c).unwrap();
        assert_eq!(raw, encoded, "exact full roundtrip {label}");
        for version in 1..=crate::modern_lito::protocol::MAX_VERSION {
            assert!(crate::modern_lito::battle_payload::decode_versioned(&raw, version).is_err());
        }
        assert!(c
            .data
            .check_scene_values(&c.data.personal.player, &c.data.world.identity)
            .is_err());
        let mut stripped = c.data.clone();
        stripped.world.descriptor.features &= !codec::FEATURE;
        assert!(stripped
            .check_scene_values(&c.data.personal.player, &c.data.world.identity)
            .is_err());
        c.check_values(&c.data.personal.player, &c.data.world.identity)
            .unwrap_or_else(|_| panic!("complete scene {label}"));
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
        let key = (group.to_owned(), c.data.personal.player.source);
        let follower = followers.entry(key.clone()).or_insert_with(|| {
            codec::Follower::new(
                c.data.personal.player.clone(),
                c.data.world.identity.clone(),
            )
            .unwrap()
        });
        follower
            .accept(c.clone())
            .unwrap_or_else(|_| panic!("successor {label}"));
        assert_eq!(follower.current().unwrap().candidate(), &c);
        let mut bad = c.clone();
        bad.actors.0[0].incoming.restored += 1;
        rejected(follower, bad, label);
        // These corruptions deliberately leave healing accounting valid. The
        // full original fields must still be checked, not stripped/relabelled.
        for change in [
            (
                "unknown feature",
                (|b: &mut codec::Candidate| b.data.world.descriptor.features |= 1 << 28)
                    as fn(&mut codec::Candidate),
            ),
            ("world position", |b| {
                b.data.actors.0[0].position.x = i32::MAX
            }),
            ("heading", |b| b.data.actors.0[0].heading = 9),
        ] {
            let mut bad = c.clone();
            (change.1)(&mut bad);
            assert!(
                bad.check_healing_accounts(&bad.data.personal.player, &bad.data.world.identity)
                    .is_ok(),
                "independent {} {label}",
                change.0
            );
            assert!(
                bad.check_values(&bad.data.personal.player, &bad.data.world.identity)
                    .is_err(),
                "full {} {label}",
                change.0
            );
            rejected(follower, bad, label);
        }
        let mut bad = c.clone();
        bad.data.personal.player.epoch ^= 2;
        rejected(follower, bad, label);
        let mut bad = c.clone();
        bad.data.world.loading_elapsed_nanos = -1;
        rejected(follower, bad, label);
        let mut bad = c.clone();
        bad.data.personal.history = None;
        rejected(follower, bad, label);
        let mut bad = c.clone();
        bad.data.presentation = None;
        rejected(follower, bad, label);
        let mut bad = c.clone();
        bad.data.world.descriptor.features &= !codec::FEATURE;
        rejected(follower, bad, label);
        if let Some(index) = c.actors.0.iter().position(|h| {
            h.entity != c.data.personal.entity
                && h.incoming.received > 0
                && h.incoming.last.as_ref().unwrap().usage.revision > h.incoming.received + 1
        }) {
            let mut bad = c.clone();
            bad.actors.0[index].incoming.received += 1;
            assert!(bad
                .check_healing_accounts(&bad.data.personal.player, &bad.data.world.identity)
                .is_ok());
            assert!(bad
                .check_values(&bad.data.personal.player, &bad.data.world.identity)
                .is_err());
            rejected(follower, bad, "global event conservation");
            coverage.insert("conservation");
        }
        if let Some(index) = c.actors.0.iter().position(|h| {
            h.entity != c.data.personal.entity
                && h.outgoing
                    .as_ref()
                    .and_then(|o| o.receipt.as_ref())
                    .is_some_and(|r| r.loading_elapsed_nanos > 0)
        }) {
            let mut bad = c.clone();
            bad.actors.0[index]
                .outgoing
                .as_mut()
                .unwrap()
                .receipt
                .as_mut()
                .unwrap()
                .loading_elapsed_nanos -= 1;
            // A different, locally plausible historic timestamp must not
            // rewrite an already accepted receipt at the same world revision.
            if bad
                .check_values(&bad.data.personal.player, &bad.data.world.identity)
                .is_ok()
            {
                rejected(follower, bad, "same revision history rewrite");
                coverage.insert("history rewrite");
            }
        }
        match &c.status {
            codec::Status::Motion(Some(_)) => {
                coverage.insert("npc");
                let mut bad = c.clone();
                bad.status = codec::Status::Motion(None);
                rejected(follower, bad, label);
            }
            codec::Status::Haste(_) => {
                coverage.insert("haste");
                let mut bad = c.clone();
                if let codec::Status::Haste(s) = &mut bad.status {
                    s.revision = -1;
                }
                rejected(follower, bad, label);
            }
            codec::Status::Speed(_) => {
                coverage.insert("slow");
                let mut bad = c.clone();
                bad.data.personal.materials = None;
                rejected(follower, bad, label);
            }
            _ => {}
        }
        if c.data
            .actors
            .0
            .iter()
            .any(|a| a.resources.as_ref().unwrap().life > 0)
        {
            coverage.insert("respawn");
        }
        if c.data
            .actors
            .0
            .iter()
            .any(|a| a.resources.as_ref().unwrap().equipment_revision > 0)
        {
            coverage.insert("gear");
        }
        if c.data
            .actors
            .0
            .iter()
            .any(|a| a.regeneration.as_ref().is_some_and(|g| g.item_gain > 0))
        {
            coverage.insert("potion");
        }
        if c.data
            .actors
            .0
            .iter()
            .any(|a| a.resources.as_ref().unwrap().flows.mp_gain > 0)
        {
            coverage.insert("regen");
        }
        if c.actors.0.iter().any(|a| a.incoming.restored > 0) {
            coverage.insert("healed damage");
        }
        if c.data.world.lane >= 0 {
            coverage.insert("isolated lane");
            assert_eq!(c.data.actors.0.len(), 1);
        }
        streams.entry(key).or_default().push(c);
        peer.ack(&encoded);
        count += 1;
    }
    assert_eq!(count, 66);
    peer.finish();
    // All ordered pairs: retained receipts must remain valid even when the
    // client observes several operations at once, not just adjacent frames.
    let mut pairs = 0;
    for (key, frames) in streams {
        for (i, old) in frames.iter().enumerate() {
            for (j, new) in frames.iter().enumerate().skip(i) {
                old.check_successor(new)
                    .unwrap_or_else(|_| panic!("catchup {key:?} {i}->{j}"));
                pairs += 1;
                if old.data.world.revision < new.data.world.revision {
                    let mut f = codec::Follower::new(
                        new.data.personal.player.clone(),
                        new.data.world.identity.clone(),
                    )
                    .unwrap();
                    f.accept(new.clone()).unwrap();
                    rejected(&mut f, old.clone(), "rollback");
                }
            }
        }
    }
    assert_eq!(pairs, 232);
    assert_eq!(
        coverage,
        BTreeSet::from([
            "npc",
            "haste",
            "slow",
            "respawn",
            "gear",
            "potion",
            "regen",
            "healed damage",
            "isolated lane",
            "conservation",
            "history rewrite"
        ])
    );
}

#[test]
fn healing_candidate_follower_has_explicit_valid_context_and_no_initial_view() {
    use crate::modern_lito::battle_payload::data::{Player, WorldId};
    let p = Player {
        source: 7,
        object: 3333,
        epoch: i64::MIN,
        channel: 3,
    };
    let w = WorldId {
        room: 1,
        match_id: 2,
        generation: 3,
    };
    let f = codec::Follower::new(p.clone(), w.clone()).unwrap();
    assert!(f.current().is_none());
    assert_eq!(format!("{f:?}"), "CandidateHealingFollower[redacted]");
    for bad in [
        Player {
            source: 0,
            ..p.clone()
        },
        Player {
            object: 0,
            ..p.clone()
        },
        Player {
            epoch: 0,
            ..p.clone()
        },
        Player {
            channel: 4,
            ..p.clone()
        },
    ] {
        assert!(codec::Follower::new(bad, w.clone()).is_err());
    }
    for bad in [
        WorldId {
            room: 0,
            ..w.clone()
        },
        WorldId {
            match_id: 0,
            ..w.clone()
        },
        WorldId {
            generation: 0,
            ..w.clone()
        },
    ] {
        assert!(codec::Follower::new(p.clone(), bad).is_err());
    }
}
