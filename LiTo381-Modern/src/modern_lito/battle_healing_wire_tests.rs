use super::{super::*, Peer};
use crate::modern_lito::battle_payload::healing as codec;

#[test]
#[ignore = "explicit compiled Java real engine peer; not negotiated TLS/GUI acceptance"]
fn java_complete_healing_battle_schema7_accounts_status_and_owner_round_trip() {
    let mut peer = Peer::start_named(Some("healing7"));
    let mut count = 0;
    let mut before = std::collections::BTreeMap::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:30");
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
        let c = codec::decode(&raw).unwrap_or_else(|_| panic!("decode {label}"));
        c.check_healing_accounts(&c.data.personal.player, &c.data.world.identity)
            .unwrap_or_else(|_| panic!("accounts {label}"));
        assert_eq!(raw, codec::encode(&c).unwrap(), "roundtrip {label}");
        assert!(decode(&raw).is_err());
        assert!(encode(&c.data).is_err());
        for version in 1..=crate::modern_lito::protocol::MAX_VERSION {
            assert!(decode_versioned(&raw, version).is_err());
        }
        assert!(c
            .data
            .check_scene_values(&c.data.personal.player, &c.data.world.identity)
            .is_err());
        let mut wrong = c.data.personal.player.clone();
        wrong.epoch ^= 2;
        assert!(c
            .check_healing_accounts(&wrong, &c.data.world.identity)
            .is_err());
        let mut bad = c.clone();
        bad.actors.0[0].incoming.restored += 1;
        assert!(bad
            .check_healing_accounts(&bad.data.personal.player, &bad.data.world.identity)
            .is_err());
        let mut bad = c.clone();
        bad.actors.0[0].entity = i64::MIN;
        assert!(bad
            .check_healing_accounts(&bad.data.personal.player, &bad.data.world.identity)
            .is_err());
        let mut bad = c.clone();
        bad.data.actors.0[0].resources.as_mut().unwrap().flows.spent += 1;
        assert!(bad
            .check_healing_accounts(&bad.data.personal.player, &bad.data.world.identity)
            .is_err());
        let mut bad = c.clone();
        bad.actors.0.reverse();
        assert!(codec::encode(&bad).is_err());
        let mut bad = c.clone();
        bad.data.world.descriptor.features &= !codec::FEATURE;
        assert!(codec::encode(&bad).is_err());
        if label.starts_with("denied") {
            let key = c.data.personal.player.source;
            let old: &codec::Candidate = before.get(&key).unwrap();
            assert_eq!(old.actors, c.actors);
            // A denied command consumes its request sequence/world revision,
            // but cannot rewrite the successful healing or resource accounts.
            let a = &old.data.personal.history.as_ref().unwrap().0;
            let b = &c.data.personal.history.as_ref().unwrap().0;
            assert_eq!(a.resources, b.resources);
            assert_eq!(a.healing, b.healing);
            assert!(b.world_revision >= a.world_revision);
        }
        if label == "self-7" {
            assert!(c.actors.0[0].incoming.restored > 0);
            assert!(
                c.data
                    .personal
                    .history
                    .as_ref()
                    .unwrap()
                    .0
                    .resources
                    .flows
                    .damaged
                    > 0
            );
        }
        if label.starts_with("status1") {
            assert!(matches!(c.status, codec::Status::Haste(_)));
        }
        if label.starts_with("status2") {
            assert!(matches!(c.status, codec::Status::Speed(_)));
            assert!(c.data.personal.materials.is_some());
        }
        if count == 0 {
            for n in 0..raw.len() {
                let mut b = raw[..n].to_vec();
                if n >= 12 {
                    b[8..12].copy_from_slice(&(n as u32).to_be_bytes());
                }
                assert!(codec::decode(&b).is_err(), "prefix {n}");
            }
            let mut b = raw.clone();
            b.push(0);
            let n = b.len() as u32;
            b[8..12].copy_from_slice(&n.to_be_bytes());
            assert!(codec::decode(&b).is_err());
            for offset in [0, 4, 6, 12, 14, 16, 18, 20] {
                let mut b = raw.clone();
                b[offset] ^= 0x80;
                assert!(codec::decode(&b).is_err(), "header {offset}");
            }
            let mut bad = c.clone();
            bad.actors.0.pop();
            assert!(codec::encode(&bad).is_err());
        }
        if let Some(receipt) = c.actors.0[0]
            .outgoing
            .as_ref()
            .and_then(|o| o.receipt.as_ref())
        {
            assert_eq!(receipt.impact.caster.identity.source, 7);
            let mut bad = c.clone();
            bad.actors.0[0]
                .outgoing
                .as_mut()
                .unwrap()
                .receipt
                .as_mut()
                .unwrap()
                .impact
                .effects = 16;
            assert!(bad
                .check_healing_accounts(&bad.data.personal.player, &bad.data.world.identity)
                .is_err());
        }
        peer.ack(&codec::encode(&c).unwrap());
        before.insert(c.data.personal.player.source, c);
        count += 1;
    }
    assert_eq!(count, 30);
    peer.finish();
}
