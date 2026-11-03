//! Selected Java application frames into the ordinary immutable follower.
//! In-process handshake and pipe transport only, not TLS/WorldService/UI.
use super::{
    healing_admission::{request, selected},
    Peer,
};
use crate::modern_lito::{
    application_protocol as app, battle_follow::Follower, battle_payload as payload,
    battle_player_healing as command, battle_read as read, protocol,
};
use std::{collections::BTreeMap, io::Cursor};

#[path = "battle_armor_presentation_tests.rs"]
mod presentation;
#[path = "battle_armor_presentation_wire_tests.rs"]
mod presentation_wire;
#[path = "battle_armor_render_tests.rs"]
mod render;
#[path = "battle_armor_lifecycle_tests.rs"]
mod lifecycle;

fn unhex(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0 && s.len() <= 2 * (payload::armor_presentation::MAX_BYTES + 4096));
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn reply(d: payload::UnvalidatedData) -> read::UnvalidatedReply {
    read::UnvalidatedReply {
        echo: request(&d),
        state: read::State::Data,
        data: Some(d),
    }
}
fn decode(
    b: &[u8],
    q: &[u8],
    peer: protocol::extensions::Selected,
) -> Result<payload::UnvalidatedData, app::ApplicationError> {
    let mut cursor = Cursor::new(b);
    let e = app::read_response_selected(&mut cursor, peer, true)?;
    let app::Response::BattleData(r) = e.response else {
        panic!("expected READ");
    };
    assert_eq!(read::encode_request(e.request_id, &r.echo).unwrap(), q);
    assert_eq!(cursor.position(), b.len() as u64);
    Ok(r.data.unwrap())
}
fn reject(f: &mut Follower, bad: payload::UnvalidatedData) {
    let saved = f.observation().unwrap().clone();
    assert!(f.accept(reply(bad)).is_err());
    assert_eq!(f.state(), read::State::Data);
    assert_eq!(f.observation().unwrap().data(), saved.data());
    assert_eq!(f.observation().unwrap().personal(), saved.personal());
}
fn healing(
    before: &payload::UnvalidatedData,
    b: &[u8],
    q: &[u8],
    peer: protocol::extensions::Selected,
) {
    use crate::modern_lito::{
        battle_operation::Lifecycle, battle_state::Tracker, battle_world as w,
        world_protocol as legacy,
    };
    let intent = command::decode_candidate_intent(q).unwrap();
    assert_eq!(command::encode_intent_selected(&intent, peer).unwrap(), q);
    let result = command::read_reply_selected(&mut Cursor::new(b), &intent, peer).unwrap();
    assert_eq!(result.reason, command::Reason::None);
    assert!(result.data.armor.is_some());
    assert!(b.ends_with(&payload::encode(&result.data).unwrap()));
    assert!(command::decode_candidate_reply(b, &intent).is_err());
    let mut tracker = Tracker::default();
    tracker
        .accept_world(
            &w::Reply {
                echo: w::Request::read(
                    intent.stamp.context,
                    intent.stamp.channel,
                    Some(&before.world.identity),
                ),
                status: legacy::Status::Ok,
                reason: legacy::Reason::None,
                state: w::State::Data,
                attachment: Some(w::Attachment {
                    world: before.world.identity.clone(),
                    lifecycle: Lifecycle {
                        revision: before.world.revision + 1,
                        phase: legacy::Phase::Active,
                        abort: legacy::AbortReason::None,
                        settlement: None,
                        race_settlement: None,
                        race_terminal_revision: 0,
                    },
                }),
                data: Some(before.clone()),
            },
            peer,
        )
        .unwrap();
    let saved = tracker.snapshot().unwrap().clone();
    let mut pending =
        command::LivePending::begin(saved.observation().unwrap(), intent, peer).unwrap();
    for ext in [0, 1, 4, 5] {
        let mut blocked = Cursor::new(b);
        assert!(command::read_reply_selected(&mut blocked, &intent, selected(ext)).is_err());
        assert_eq!(
            blocked.position(),
            0,
            "no command capability before reading"
        );
        assert!(pending
            .accept(&mut tracker, &result, selected(ext))
            .is_err());
        assert!(!pending.resolved());
        assert!(std::sync::Arc::ptr_eq(&saved, tracker.snapshot().unwrap()));
    }
    assert!(command::read_reply_selected(&mut Cursor::new(b), &intent, selected(3)).is_err());
    for change in [
        (|r: &mut command::Reply| r.data.armor = None) as fn(&mut command::Reply),
        |r| {
            r.data.armor.as_mut().unwrap().healing.as_mut().unwrap().0[0]
                .incoming
                .restored += 1
        },
        |r| r.data.personal.history = None,
        |r| r.echo.sequence += 1,
        |r| r.lifecycle.revision = 0,
    ] {
        let mut bad = result.clone();
        change(&mut bad);
        assert!(pending.accept(&mut tracker, &bad, peer).is_err());
        assert!(!pending.resolved());
        assert!(std::sync::Arc::ptr_eq(&saved, tracker.snapshot().unwrap()));
    }
    let mut coalesced = Cursor::new([b, b].concat());
    for n in 1..=2 {
        assert_eq!(
            command::read_reply_selected(&mut coalesced, &intent, peer).unwrap(),
            result
        );
        assert_eq!(coalesced.position(), (n * b.len()) as u64);
    }
    let mut oversized = b[..16].to_vec();
    oversized[4..8].copy_from_slice(
        &((command::RESULT_PREFIX_BYTES + payload::Format::Selected(peer).maximum() + 1) as u32)
            .to_be_bytes(),
    );
    let mut blocked = Cursor::new(oversized);
    assert!(command::read_reply_selected(&mut blocked, &intent, peer).is_err());
    assert_eq!(blocked.position(), 16);
    // A header just beyond the old healing bound reaches the bounded body
    // reader only with ARMOR. The intentionally truncated body is NOT valid.
    let mut larger = b[..17].to_vec();
    larger[4..8].copy_from_slice(&((command::MAX_RESULT_BYTES + 1) as u32).to_be_bytes());
    for (ext, consumed) in [(3, 16), (7, 17)] {
        let mut truncated = Cursor::new(&larger);
        assert!(command::read_reply_selected(&mut truncated, &intent, selected(ext)).is_err());
        assert_eq!(truncated.position(), consumed);
    }
    pending.accept(&mut tracker, &result, peer).unwrap();
    assert!(pending.resolved());
    assert_eq!(
        tracker.snapshot().unwrap().observation().unwrap().data(),
        &result.data
    );
    let committed = tracker.snapshot().unwrap().clone();
    assert!(pending.accept(&mut tracker, &result, peer).is_err());
    assert!(std::sync::Arc::ptr_eq(
        &committed,
        tracker.snapshot().unwrap()
    ));
}

#[test]
fn armor_selection_has_its_own_capacity_without_changing_healing_defaults() {
    assert_eq!(protocol::extensions::HEALING.supported, 3);
    for ext in [4, 5, 7] {
        assert!(selected(ext).allows_armor());
        assert_eq!(
            payload::Format::Selected(selected(ext)).maximum(),
            payload::armor::MAX_BYTES
        );
    }
    assert_eq!(
        payload::Format::Selected(selected(3)).maximum(),
        payload::healing::MAX_BYTES
    );
    assert!(payload::armor::MAX_BYTES > payload::healing::MAX_BYTES);
}

#[test]
#[ignore = "explicit Java selected armor handshake/replies/follower/commands; not TLS, configured server activation or UI"]
fn java_armor_selected_preserves_full_state_and_atomic_healing() {
    let mut peer = Peer::start_named(Some("armor-selected"));
    let mut sessions = BTreeMap::new();
    let mut followers = BTreeMap::new();
    let mut histories: BTreeMap<String, Vec<payload::UnvalidatedData>> = BTreeMap::new();
    let mut heal_before = None;
    let mut count = 0;
    let mut rejects = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:70");
            break;
        }
        if line.starts_with("SESSION:") {
            let fields = line.splitn(4, ':').collect::<Vec<_>>();
            let ext = fields[1].parse::<u32>().unwrap();
            let raw = unhex(fields[2]);
            let protocol::Message::ExtendedHello(h, offer) = protocol::decode(&raw).unwrap() else {
                panic!("extended hello");
            };
            let mut client = protocol::ModernClientSession::new_extended(h, offer).unwrap();
            assert_eq!(client.begin().unwrap(), raw);
            let ack = client.accept_ack(&unhex(fields[3])).unwrap();
            assert_eq!(client.extensions(), Some(ext));
            assert!(sessions
                .insert(ext, protocol::extensions::Selected::new(ack, ext).unwrap())
                .is_none());
            continue;
        }
        let fields = line.splitn(5, ':').collect::<Vec<_>>();
        assert_eq!(fields[0], "REPLY");
        let label = fields[1];
        let ext = fields[2].parse::<u32>().unwrap();
        let selected_peer = sessions[&ext];
        let q = unhex(fields[3]);
        let b = unhex(fields[4]);
        if label == "heal-cast" {
            healing(heal_before.as_ref().unwrap(), &b, &q, selected_peer);
            peer.ack(&b);
            count += 1;
            continue;
        }
        let d = decode(&b, &q, selected_peer).unwrap_or_else(|e| panic!("{label}: {e:?}"));
        let raw = &b[72..];
        let candidate = payload::armor::decode(raw).unwrap();
        assert_eq!(d.armor.as_ref().unwrap().actors, candidate.actors);
        assert_eq!(d.armor.as_ref().unwrap().healing, candidate.healing);
        assert_eq!(d.armor.as_ref().unwrap().status, candidate.status);
        assert_eq!(d.personal, candidate.data.personal);
        assert_eq!(payload::encode(&d).unwrap(), raw);
        for e in [5, 7] {
            assert_eq!(decode(&b, &q, sessions[&e]).unwrap(), d);
        }
        for e in [0, 1, 3] {
            assert!(decode(&b, &q, selected(e)).is_err());
        }
        if d.armor.as_ref().unwrap().healing.is_some() {
            assert!(decode(&b, &q, sessions[&4]).is_err());
        }
        for version in 1..=47 {
            assert!(payload::decode_versioned(raw, version).is_err());
        }
        let mut blocked = Cursor::new(&b);
        assert!(app::read_response_selected(&mut blocked, selected_peer, false).is_err());
        assert_eq!(blocked.position(), 16);
        let mut oversized = b[..16].to_vec();
        oversized[4..8].copy_from_slice(&u32::MAX.to_be_bytes());
        let mut blocked = Cursor::new(oversized);
        assert!(app::read_response_selected(&mut blocked, selected_peer, true).is_err());
        assert_eq!(blocked.position(), 16);
        for n in [0, 15, 16, 71, b.len() - 1] {
            assert!(decode(&b[..n], &q, selected_peer).is_err());
        }
        let mut coalesced = Cursor::new([b.as_slice(), b.as_slice()].concat());
        for n in 1..=2 {
            app::read_response_selected(&mut coalesced, selected_peer, true).unwrap();
            assert_eq!(coalesced.position(), (n * b.len()) as u64);
        }
        let group = if label.starts_with("status") {
            label.split('-').take(2).collect::<Vec<_>>().join("-")
        } else if label.starts_with("dead") || label.starts_with("revived") {
            "life".to_owned()
        } else if label.starts_with("bots")
            || label.starts_with("quest")
            || label.starts_with("heal-")
        {
            label.split('-').next().unwrap().to_owned()
        } else {
            "basic".to_owned()
        };
        let key = format!("{group}-{}", d.personal.entity);
        let f = followers
            .entry(key.clone())
            .or_insert_with(|| Follower::new_selected(request(&d), selected_peer).unwrap());
        f.accept(reply(d.clone()))
            .unwrap_or_else(|e| panic!("follow {label}: {e:?}"));
        let checked = candidate
            .checked(&d.personal.player, &d.world.identity)
            .unwrap();
        assert_eq!(f.observation().unwrap().personal(), checked.personal());
        assert_eq!(f.observation().unwrap().data(), &d);
        assert!(
            crate::modern_lito::battle_scene::baseline_counts(f.observation().unwrap().clone())
                .is_err(),
            "no unqualified armor rendering"
        );
        f.accept(reply(d.clone())).unwrap();
        for change in [
            (|v: &mut payload::UnvalidatedData| v.armor = None)
                as fn(&mut payload::UnvalidatedData),
            |v| v.world.descriptor.features &= !payload::armor::FEATURE,
            |v| v.armor.as_mut().unwrap().actors.0[0].entity += 100,
            |v| v.personal.history = None,
            |v| v.personal.player.epoch ^= 2,
            |v| v.world.identity.generation += 1,
            |v| v.presentation = None,
            |v| v.npc_motion = Some(payload::npc_motion::Envelope { state: None }),
        ] {
            let mut bad = d.clone();
            change(&mut bad);
            reject(f, bad);
            rejects += 1;
        }
        if label.starts_with("first") || label.starts_with("second") {
            let mut bad = d.clone();
            bad.world.revision += 1;
            bad.personal.history.as_mut().unwrap().0.world_revision += 1;
            bad.armor.as_mut().unwrap().actors.0[1]
                .state
                .as_mut()
                .unwrap()
                .effects
                .0[0]
                .graphics ^= 1;
            bad.check_scene_values(&d.personal.player, &d.world.identity)
                .unwrap();
            reject(f, bad);
            rejects += 1;
        }
        if label == "heal-before" {
            let mut larger = b[..17].to_vec();
            larger[4..8].copy_from_slice(
                &((read::MIN_RESULT_BYTES + payload::healing::MAX_BYTES + 1) as u32).to_be_bytes(),
            );
            for (ext, consumed) in [(3, 16), (7, 17)] {
                let mut truncated = Cursor::new(&larger);
                assert!(app::read_response_selected(&mut truncated, selected(ext), true).is_err());
                assert_eq!(truncated.position(), consumed);
            }
            heal_before = Some(d.clone());
        } else {
            histories.entry(key).or_default().push(d);
        }
        peer.ack(&b);
        count += 1;
    }
    peer.finish();
    assert_eq!(count, 70);
    assert_eq!(sessions.len(), 3);
    assert_eq!(histories.len(), 20);
    let mut pairs = 0;
    for frames in histories.values() {
        for (i, a) in frames.iter().enumerate() {
            for b in frames.iter().skip(i) {
                let mut f = Follower::new_selected(request(a), sessions[&7]).unwrap();
                f.accept(reply(a.clone())).unwrap();
                let saved = f.observation().unwrap().clone();
                f.accept(reply(b.clone())).unwrap();
                saved.check_successor(f.observation().unwrap()).unwrap();
                assert_eq!(saved.data(), a);
                if a.world.revision < b.world.revision {
                    reject(&mut f, a.clone());
                }
                pairs += 1;
            }
        }
    }
    assert_eq!(pairs, 176);
    assert_eq!(rejects, 556);
    eprintln!("ARMOR_SELECTED: 3 actual handshake pairs; 70 application replies; 20 owner streams; 176 ordered pairs; 556 atomic scene rejections; complete healing LivePending; selected capacity and legacy rejection; no TLS/Main/UI claim");
}
