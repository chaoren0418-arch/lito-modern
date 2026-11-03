//! Rust intents execute in the Java engine, then both candidate and full-state
//! pending paths verify the receipt. Separate from the actual TLS/UI tests.
use super::Peer;
use crate::modern_lito::{
    application_protocol as app, battle_payload::healing as scene,
    battle_player_healing as command, protocol,
};
use std::{
    collections::BTreeSet,
    io::{Cursor, Write},
};

fn unhex(text: &str) -> Vec<u8> {
    assert!(text.len() % 2 == 0 && text.len() <= 2 * command::MAX_RESULT_BYTES);
    text.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn rejected(
    pending: &mut command::Pending,
    follower: &mut scene::Follower,
    reply: &command::CandidateReply,
    label: &str,
) {
    let saved = follower.current().unwrap().clone();
    let intent = *pending.intent();
    assert!(!pending.resolved());
    assert!(pending.accept(follower, reply).is_err(), "reject {label}");
    assert!(!pending.resolved(), "preserve pending {label}");
    assert_eq!(*pending.intent(), intent);
    assert_eq!(
        follower.current().unwrap().candidate(),
        saved.candidate(),
        "atomic world {label}"
    );
    assert_eq!(
        follower.current().unwrap().personal(),
        saved.personal(),
        "atomic private {label}"
    );
}

fn selected(extensions: u32) -> protocol::extensions::Selected {
    protocol::extensions::Selected::new(
        protocol::Ack {
            request_id: 1,
            version: 47,
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES,
        },
        extensions,
    )
    .unwrap()
}
fn full_state(before: &scene::Candidate, raw: &[u8], intent: command::Intent, label: &str) {
    use crate::modern_lito::{
        battle_operation::Lifecycle, battle_payload as payload, battle_state, battle_world as w,
        world_protocol as legacy,
    };
    let data = before.clone().into_data().unwrap();
    let peer = selected(3);
    let mut tracker = battle_state::Tracker::default();
    let q = w::Request::read(
        intent.stamp.context,
        intent.stamp.channel,
        Some(&data.world.identity),
    );
    assert_eq!(data.world.phase, payload::data::Phase::Active);
    // Explicit offline predecessor, not a fabricated network/resource ACK.
    tracker
        .accept_world(
            &w::Reply {
                echo: q,
                status: legacy::Status::Ok,
                reason: legacy::Reason::None,
                state: w::State::Data,
                attachment: Some(w::Attachment {
                    world: data.world.identity.clone(),
                    lifecycle: Lifecycle {
                        revision: data.world.revision + 1,
                        phase: legacy::Phase::Active,
                        abort: legacy::AbortReason::None,
                        settlement: None,
                        race_settlement: None,
                        race_terminal_revision: 0,
                    },
                }),
                data: Some(data.clone()),
            },
            peer,
        )
        .unwrap();
    let before = tracker.snapshot().unwrap().clone();
    let mut pending =
        command::LivePending::begin(before.observation().unwrap(), intent, peer).unwrap();
    let reply = command::decode_reply_selected(raw, &intent, peer).unwrap();
    assert!(raw.ends_with(&payload::encode(&reply.data).unwrap()));
    for ext in [0, 1] {
        assert!(command::encode_intent_selected(&intent, selected(ext)).is_err());
        let mut reader = Cursor::new(raw);
        assert!(command::read_reply_selected(&mut reader, &intent, selected(ext)).is_err());
        assert_eq!(
            reader.position(),
            0,
            "command selection before consuming header"
        );
        assert!(pending.accept(&mut tracker, &reply, selected(ext)).is_err());
        assert!(!pending.resolved());
        assert!(std::sync::Arc::ptr_eq(&before, tracker.snapshot().unwrap()));
    }
    let mut coalesced = Cursor::new([raw, raw].concat());
    for n in 1..=2 {
        assert_eq!(
            command::read_reply_selected(&mut coalesced, &intent, peer).unwrap(),
            reply
        );
        assert_eq!(coalesced.position(), (n * raw.len()) as u64);
    }
    for change in [
        (|r: &mut command::Reply| r.echo.request_id ^= 8) as fn(&mut command::Reply),
        |r| r.echo.sequence += 1,
        |r| r.echo.target.life += 1,
        |r| r.echo.stamp.context.epoch ^= 2,
        |r| r.lifecycle.revision = 0,
        |r| r.data.personal.history = None,
        |r| {
            r.data.healing.as_mut().unwrap().actors.0[0]
                .incoming
                .restored += 1
        },
        |r| r.data.actors.0[0].resources.as_mut().unwrap().flows.spent += 1,
        |r| {
            r.reason = if r.reason == command::Reason::None {
                command::Reason::NoMp
            } else {
                command::Reason::None
            }
        },
    ] {
        let mut bad = reply.clone();
        change(&mut bad);
        assert!(
            pending.accept(&mut tracker, &bad, peer).is_err(),
            "formal reject {label}"
        );
        assert!(!pending.resolved());
        assert!(std::sync::Arc::ptr_eq(&before, tracker.snapshot().unwrap()));
        assert_eq!(before.observation().unwrap().data(), &data);
    }
    pending
        .accept(&mut tracker, &reply, peer)
        .unwrap_or_else(|e| panic!("formal commit {label}: {e:?}"));
    assert!(pending.resolved());
    let after = tracker.snapshot().unwrap().clone();
    assert_eq!(after.observation().unwrap().data(), &reply.data);
    assert!(pending.accept(&mut tracker, &reply, peer).is_err());
    assert!(std::sync::Arc::ptr_eq(&after, tracker.snapshot().unwrap()));
    assert_eq!(before.observation().unwrap().data(), &data);
}

#[test]
#[ignore = "explicit Java engine peer; bidirectional candidate command, not live service/UI"]
fn java_player_healing_rust_intents_execute_and_replies_commit_atomically() {
    let mut peer = Peer::start_named(Some("healing-command"));
    let mut labels = BTreeSet::new();
    let mut reasons = BTreeSet::new();
    let mut count = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:22");
            break;
        }
        let begin: Vec<_> = line.splitn(5, ':').collect();
        assert_eq!(begin.len(), 5);
        assert_eq!(begin[0], "HEAL-BEGIN");
        let label = begin[1];
        assert!(labels.insert(label.to_owned()));
        let before = scene::decode(&unhex(begin[2])).unwrap();
        let mut follower = scene::Follower::new(
            before.data.personal.player.clone(),
            before.data.world.identity.clone(),
        )
        .unwrap();
        follower
            .accept(before.clone())
            .unwrap_or_else(|_| panic!("before {label}"));
        for target in [-1, 0, 101, i64::MAX] {
            assert!(command::Pending::begin(&follower, target, 1).is_err());
        }
        let target = begin[3].parse().unwrap();
        let id = begin[4].parse().unwrap();
        assert!(command::Pending::begin(&follower, target, 0).is_err());
        let mut pending = command::Pending::begin(&follower, target, id).unwrap();
        let intent = *pending.intent();
        let bytes = command::encode_candidate_intent(&intent).unwrap();
        assert_eq!(bytes.len(), 160);
        assert_eq!(command::decode_candidate_intent(&bytes).unwrap(), intent);
        let text = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        let input = peer.child.stdin.as_mut().unwrap();
        writeln!(input, "{text}").unwrap();
        input.flush().unwrap();

        let line = peer.line();
        let result: Vec<_> = line.splitn(3, ':').collect();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "HEAL-REPLY");
        assert_eq!(result[1], label);
        let raw = unhex(result[2]);
        let reply = command::decode_candidate_reply(&raw, &intent)
            .unwrap_or_else(|_| panic!("decode {label}"));
        reply
            .check_after(&intent, &before)
            .unwrap_or_else(|_| panic!("successor {label}"));
        reasons.insert(reply.reason as u8);
        let encoded = command::encode_candidate_reply(&reply).unwrap();
        assert_eq!(encoded, raw, "exact newly encoded receipt {label}");
        full_state(&before, &raw, intent, label);
        for version in 1..=protocol::MAX_VERSION {
            assert!(
                app::decode_response_version(&raw, version).is_err(),
                "closed old version {version}"
            );
        }
        let mut coalesced = Cursor::new([raw.as_slice(), raw.as_slice()].concat());
        assert_eq!(
            command::read_candidate_reply(&mut coalesced, &intent).unwrap(),
            reply
        );
        assert_eq!(coalesced.position() as usize, raw.len());
        assert_eq!(
            command::read_candidate_reply(&mut coalesced, &intent).unwrap(),
            reply
        );
        assert_eq!(coalesced.position() as usize, 2 * raw.len());
        for n in [0, 15, 16, 159, 160, 191, 192, raw.len() - 1] {
            assert!(command::decode_candidate_reply(&raw[..n], &intent).is_err());
        }
        let mut trailing = raw.clone();
        trailing.push(0);
        let len = trailing.len() as u32;
        trailing[4..8].copy_from_slice(&len.to_be_bytes());
        assert!(command::decode_candidate_reply(&trailing, &intent).is_err());
        for offset in [
            0, 4, 8, 13, 15, 88, 112, 136, 160, 161, 175, 184, 188, 192, 197,
        ] {
            let mut bad = raw.clone();
            bad[offset] ^= 0x7f;
            assert!(
                command::decode_candidate_reply(&bad, &intent).is_err(),
                "wire {offset} {label}"
            );
        }
        for change in [
            (|r: &mut command::CandidateReply| r.echo.request_id ^= 100)
                as fn(&mut command::CandidateReply),
            |r| r.echo.sequence += 1,
            |r| r.echo.target.life += 1,
            |r| r.echo.stamp.context.epoch ^= 2,
            |r| r.battle.data.world.revision = 0,
            |r| r.battle.data.personal.history = None,
            |r| r.battle.actors.0[0].incoming.restored += 1,
            |r| {
                r.battle.data.actors.0[0]
                    .resources
                    .as_mut()
                    .unwrap()
                    .flows
                    .spent += 1
            },
            |r| {
                r.reason = if r.reason == command::Reason::None {
                    command::Reason::NoMp
                } else {
                    command::Reason::None
                }
            },
        ] {
            let mut bad = reply.clone();
            change(&mut bad);
            rejected(&mut pending, &mut follower, &bad, label);
        }
        pending
            .accept(&mut follower, &reply)
            .unwrap_or_else(|_| panic!("commit {label}"));
        assert!(pending.resolved());
        assert_eq!(follower.current().unwrap().candidate(), &reply.battle);
        assert!(
            pending.accept(&mut follower, &reply).is_err(),
            "replay {label}"
        );
        assert_eq!(follower.current().unwrap().candidate(), &reply.battle);
        assert_eq!(*pending.intent(), intent);
        peer.ack(&encoded);
        count += 1;
    }
    peer.finish();
    assert_eq!(count, 22);
    assert_eq!(protocol::MAX_VERSION, 47);
    for label in [
        "bot",
        "self-hurt",
        "cross",
        "abort",
        "gear-initial",
        "stale-gear",
        "repriced",
        "old-life",
        "new-life",
        "concurrent-target-heal",
        "concurrent-caster-healed",
        "with-haste-slow-data",
        "stale-target",
        "caster-died-pending",
        "target-dead",
    ] {
        assert!(labels.contains(label), "coverage {label}");
    }
    assert_eq!(
        reasons,
        [0, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 13, 14]
            .into_iter()
            .collect()
    );
    eprintln!("PLAYER_HEAL_COMMAND: 22 Rust intents executed by Java, 13 result reasons, candidate and full BattleState/LivePending atomic commits, command selection/complete receipt/malformed/replay checks; old versions 1..47 closed; offline engine interoperability, not TLS/UI proof");
}
