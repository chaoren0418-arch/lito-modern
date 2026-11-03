//! Java real authority/store/engine -> full receipt -> Rust atomic admission.
//! Headless pipe; no Main/TLS/new-player-command/resource/UI activation claim.
use super::{healing_admission::selected, Peer};
use crate::modern_lito::{
    battle_follow::Follower, battle_payload as payload, battle_read as read,
    battle_vip_equipment as gear, protocol,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Cursor,
};

fn unhex(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0 && s.len() <= 2 * gear::MAX_RESULT_BYTES);
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn read_request(r: &gear::Reply) -> read::Request {
    let s = r.echo.stamp;
    read::Request {
        context: s.context,
        channel_id: s.channel,
        world: payload::data::WorldId {
            room: s.world[0],
            match_id: s.world[1],
            generation: s.world[2],
        },
    }
}
fn reject(f: &mut Follower, r: &gear::Reply, expected: &gear::Request) {
    let old = f.observation().unwrap().clone();
    let state = f.state();
    assert!(r.accept(expected, f).is_err());
    assert_eq!(f.state(), state);
    assert_eq!(f.observation().unwrap().data(), old.data());
    assert_eq!(f.observation().unwrap().personal(), old.personal());
}
#[test]
#[ignore = "compiled Java full VIP authority/engine/durable CAS with selected inspection/SET wire and atomic follower; no TLS/UI"]
fn java_vip_equipment_receipts_keep_independent_versions_and_atomic_acceptance() {
    let mut peer = Peer::start_named(Some("vip-equipment"));
    let line = peer.line();
    let s: Vec<_> = line.splitn(3, ':').collect();
    assert_eq!(s[0], "SESSION");
    let hello = unhex(s[1]);
    let protocol::Message::ExtendedHello(h, offer) = protocol::decode(&hello).unwrap() else {
        panic!("extended hello")
    };
    let mut session = protocol::ModernClientSession::new_extended(h, offer).unwrap();
    assert_eq!(session.begin().unwrap(), hello);
    let ack = session.accept_ack(&unhex(s[2])).unwrap();
    assert_eq!(session.extensions(), Some(63));
    let selection = protocol::extensions::Selected::new(ack, 63).unwrap();
    assert!(selection.allows_vip_equipment());
    let mut followers = BTreeMap::new();
    let mut quotes: BTreeMap<String, gear::Quote> = BTreeMap::new();
    let mut outcomes = BTreeSet::new();
    let mut count = 0;
    let mut mutations = 0;
    let mut stale_quotes = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:17");
            break;
        }
        let s: Vec<_> = line.splitn(4, ':').collect();
        assert_eq!(s[0], "REPLY");
        let label = s[1];
        let q = unhex(s[2]);
        let bytes = unhex(s[3]);
        let mut input = Cursor::new(&bytes);
        let r = gear::read_reply(&mut input, &q, selection)
            .unwrap_or_else(|e| panic!("{label}: {e:?}"));
        assert_eq!(input.position(), bytes.len() as u64);
        outcomes.insert(r.outcome as u8);
        let id = u32::from_be_bytes(q[8..12].try_into().unwrap());
        let encoded = gear::encode_reply(id, &r, selection).unwrap();
        assert_eq!(encoded, bytes);
        assert_eq!(gear::encode_request(id, &r.echo, selection).unwrap(), q);
        assert!(!read::command_data_negotiated_selected(&r.data, selection));
        assert!(payload::Format::Selected(selection)
            .decode_command(&bytes[gear::RESULT_PREFIX_BYTES..])
            .is_err());
        for ext in [0, 3, 7, 15, 16, 17, 28, 29, 31] {
            let mut cursor = Cursor::new(&bytes);
            assert!(gear::read_reply(&mut cursor, &q, selected(ext)).is_err());
            assert_eq!(cursor.position(), 0);
        }
        let mut both = bytes.clone();
        both.extend_from_slice(&bytes);
        let mut cursor = Cursor::new(both);
        assert!(gear::read_reply(&mut cursor, &q, selection).is_ok());
        assert_eq!(cursor.position(), bytes.len() as u64);
        for at in [0, 8, 14, 156, 157, 158, 159, 208, 231, 232] {
            let mut bad = bytes.clone();
            bad[at] ^= 0xff;
            assert!(
                gear::read_reply(&mut Cursor::new(&bad), &q, selection).is_err(),
                "{label} wire mutation {at}"
            );
            mutations += 1;
        }
        for len in [0, 15, 156, 235, bytes.len() - 1] {
            assert!(gear::read_reply(&mut Cursor::new(&bytes[..len]), &q, selection).is_err());
            mutations += 1;
        }
        let key = label.split('-').next().unwrap().to_string();
        let f = followers.entry(key.clone()).or_insert_with(|| {
            let mut f = Follower::new_selected(read_request(&r), selection).unwrap();
            f.accept(read::UnvalidatedReply {
                echo: read_request(&r),
                state: read::State::Data,
                data: Some(r.data.clone()),
            })
            .unwrap();
            f
        });
        for kind in 0..7 {
            let mut bad = r.clone();
            match kind {
                0 => bad.durable_revision += 1,
                1 => bad.echo.stamp.world[0] += 1,
                2 => bad.data.vip.as_mut().unwrap().owner.versions.saved += 1,
                3 => bad.data.vip.as_mut().unwrap().owner.versions.runtime += 1,
                4 => bad.lifecycle.revision = bad.data.world.revision,
                5 => {
                    bad.token = if bad.token.is_some() {
                        None
                    } else {
                        Some([0; 32])
                    }
                }
                6 => {
                    bad.denial = if bad.denial.is_some() {
                        None
                    } else {
                        Some(crate::modern_lito::equipment::Reason::UnknownItem)
                    }
                }
                _ => unreachable!(),
            }
            reject(f, &bad, &r.echo);
            mutations += 1;
        }
        let mut wrong = r.echo;
        wrong.desired = !wrong.desired;
        reject(f, &r, &wrong);
        mutations += 1;
        let mut empty = Follower::new_selected(read_request(&r), selection).unwrap();
        assert!(r.accept(&r.echo, &mut empty).is_err());
        assert!(empty.observation().is_none());
        mutations += 1;
        let old_versions = gear::Request::inspect(f.observation().unwrap())
            .ok()
            .map(|q| q.versions);
        let quote = r
            .accept(&r.echo, f)
            .unwrap_or_else(|e| panic!("{label} accept: {e:?}"));
        assert_eq!(
            quote.is_some(),
            matches!(
                r.outcome,
                gear::Outcome::Inspected | gear::Outcome::Unchanged | gear::Outcome::Applied
            )
        );
        let new_versions = gear::Request::inspect(f.observation().unwrap())
            .ok()
            .map(|q| q.versions);
        if old_versions != new_versions {
            if let Some(old) = quotes.get(&key) {
                assert!(old.set(f.observation().unwrap(), 1, false).is_err());
                stale_quotes += 1;
            }
        }
        if let Some(token) = quote {
            let request = token.set(f.observation().unwrap(), 1, false).unwrap();
            assert_eq!(
                request.versions,
                [
                    r.data.vip.as_ref().unwrap().owner.versions.saved,
                    r.data.vip.as_ref().unwrap().owner.versions.runtime,
                    r.data.vip.as_ref().unwrap().owner.versions.lifecycle
                ]
            );
            quotes.insert(key, token);
        }
        if label == "expiry-inspect" {
            let v = &r.data.vip.as_ref().unwrap().owner.versions;
            assert_eq!([v.saved, v.runtime, v.lifecycle], [0, 1, 1]);
        }
        if label == "expiry-ordinary" {
            let v = &r.data.vip.as_ref().unwrap().owner.versions;
            assert_eq!([v.saved, v.runtime, v.lifecycle], [1, 2, 2]);
            assert_eq!(r.durable_revision, 1);
        }
        if label == "wide-inspect" {
            assert_eq!(
                r.data.vip.as_ref().unwrap().owner.abilities.0[7],
                4_294_967_299
            );
        }
        if label == "retired-save" {
            assert!(quote.is_none());
            assert_eq!(r.durable_revision, 1);
            assert_eq!(r.data.vip.as_ref().unwrap().owner.versions.saved, 0);
        }
        peer.ack(&encoded);
        count += 1;
    }
    assert_eq!(count, 17);
    assert_eq!(outcomes, (0..=6).collect());
    assert_eq!(followers.len(), 4);
    assert_eq!(stale_quotes, 5);
    for (from, to) in [("wide", "moon"), ("moon", "wide")] {
        assert!(quotes[from]
            .set(followers[to].observation().unwrap(), 1, false)
            .is_err());
    }
    peer.finish();
    eprintln!("VIP_EQUIPMENT: {count} real authority receipts; 7 outcomes; 4 source/world streams; {mutations} wire/atomic rejections; {stale_quotes} stale quotes; 2 cross-source quotes; independent saved/runtime/lifecycle; not TLS/UI");
}
