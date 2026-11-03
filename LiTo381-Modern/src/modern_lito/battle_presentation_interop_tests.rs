use super::*;
use crate::modern_lito::{battle_follow::Follower, battle_read as read, protocol};
use std::collections::BTreeMap;

fn request(d: &data::UnvalidatedData) -> read::Request {
    use crate::modern_lito::application_protocol::{CharacterKey, SelectionContext};
    read::Request {
        context: SelectionContext {
            key: CharacterKey {
                source_id: d.personal.player.source as u32,
                object_id: d.personal.player.object as u64,
            },
            epoch: d.personal.player.epoch as u64,
        },
        channel_id: d.personal.player.channel as u8,
        world: d.world.identity.clone(),
    }
}
fn ack(version: u16) -> protocol::Ack {
    protocol::Ack {
        request_id: 1,
        version,
        numeric_schema: protocol::NUMERIC_SCHEMA_I64_V1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    }
}
fn reply(d: data::UnvalidatedData) -> read::UnvalidatedReply {
    read::UnvalidatedReply {
        echo: request(&d),
        state: read::State::Data,
        data: Some(d),
    }
}
fn rejected(d: &data::UnvalidatedData) {
    assert!(d
        .check_scene_values(&d.personal.player, &d.world.identity)
        .is_err());
}

#[test]
#[ignore = "explicit compiled Java presentation peer; real engine/public-private codec and follower, not visual/TLS verification"]
fn schema_two_public_events_roundtrip_without_sharing_private_stocks() {
    let mut peer = Peer::start_named(Some("presentation"));
    let mut traces: BTreeMap<String, Vec<data::UnvalidatedData>> = BTreeMap::new();
    let mut frames = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:20");
            break;
        }
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields[0], "FRAME");
        let bytes = fields[3]
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let p = inspect(&bytes).unwrap();
        assert_eq!(p.schema(), PRESENTATION_SCHEMA);
        let d = decode(&bytes).unwrap();
        d.check_scene_values(&d.personal.player, &d.world.identity)
            .unwrap();
        assert_eq!(encode(&d).unwrap(), bytes);
        peer.ack(&encode(&d).unwrap());
        let mut legacy = d.clone();
        legacy.presentation = None;
        let legacy_bytes = encode(&legacy).unwrap();
        let old = inspect(&legacy_bytes).unwrap();
        for section in [
            Section::World,
            Section::Room,
            Section::Manifest,
            Section::Actors,
            Section::Honor,
            Section::Quest,
            Section::TimeAttack,
            Section::Siege,
            Section::Race,
            Section::Npcs,
            Section::Personal,
            Section::Result,
        ] {
            assert_eq!(old.section(section), p.section(section));
        }
        assert!(!read::reply_negotiated(&reply(d.clone()), &ack(35)));
        assert!(!read::reply_negotiated(&reply(legacy), &ack(36)));
        let mut bad = d.clone();
        bad.presentation.as_mut().unwrap().0.pop();
        rejected(&bad);
        let mut bad = d.clone();
        bad.presentation.as_mut().unwrap().0[1].entity = 1;
        rejected(&bad);
        if let Some(s) = &d.presentation.as_ref().unwrap().0[0].shot {
            assert_eq!(s.sprite, 66);
            assert_eq!(s.life, 1);
            assert_eq!(s.target_life, 1);
            assert!(s.hit);
            for which in 0..8 {
                let mut bad = d.clone();
                let b = bad.presentation.as_mut().unwrap().0[0]
                    .shot
                    .as_mut()
                    .unwrap();
                match which {
                    0 => b.revision = d.world.revision + 1,
                    1 => b.loading_elapsed_nanos = d.world.loading_elapsed_nanos + 1,
                    2 => b.life = 2,
                    3 => b.target_life = 2,
                    4 => b.sprite = 999,
                    5 => b.from = b.to.clone(),
                    6 => b.equipment_revision = i64::MAX,
                    _ => {
                        if let data::Target::Participant { identity, .. } = &mut b.target {
                            identity.source += 1;
                        } else {
                            panic!()
                        }
                    }
                }
                rejected(&bad);
            }
        }
        if d.presentation.as_ref().unwrap().0[0].item.is_some() {
            for which in 0..5 {
                let mut bad = d.clone();
                let b = bad.presentation.as_mut().unwrap().0[0]
                    .item
                    .as_mut()
                    .unwrap();
                match which {
                    0 => b.revision = d.world.revision + 1,
                    1 => b.loading_elapsed_nanos = -1,
                    2 => b.life = 2,
                    3 => b.effect = 999,
                    _ => b.count = b.revision,
                }
                rejected(&bad);
            }
        }
        traces.entry(fields[1].to_owned()).or_default().push(d);
        frames += 1;
    }
    peer.finish();
    assert_eq!(frames, 20);
    assert_eq!(traces.len(), 4);
    for trace in traces.values() {
        assert_eq!(trace.len(), 5);
        let mut follower = Follower::new(request(&trace[0]), ack(36)).unwrap();
        for d in trace {
            follower.accept(reply(d.clone())).unwrap();
        }
        let before = follower.observation().unwrap().data().clone();
        assert!(follower.accept(reply(trace[1].clone())).is_err());
        assert_eq!(follower.observation().unwrap().data(), &before);
    }
    for name in ["bow", "item"] {
        let a = &traces[&format!("{name}-a")];
        let b = &traces[&format!("{name}-b")];
        for (a, b) in a.iter().zip(b) {
            assert_eq!(a.presentation, b.presentation);
            assert_ne!(a.personal.player, b.personal.player);
        }
    }
    // A non-owner cannot check the other player's private ledger. Prove the
    // temporal gate itself rejects a rewritten OLD event in a newer, otherwise
    // valid public snapshot, without poisoning the retained observation.
    for (label, index) in [("bow-b", 1), ("item-b", 2)] {
        let trace = &traces[label];
        let mut follower = Follower::new(request(&trace[index]), ack(36)).unwrap();
        follower.accept(reply(trace[index].clone())).unwrap();
        let before = follower.observation().unwrap().data().clone();
        let mut changed = trace[index + 1].clone();
        let p = &mut changed.presentation.as_mut().unwrap().0[0];
        if label == "bow-b" {
            p.shot.as_mut().unwrap().from.y += 1;
        } else {
            p.item.as_mut().unwrap().cell.y += 1;
        }
        changed
            .check_scene_values(&changed.personal.player, &changed.world.identity)
            .unwrap();
        assert!(follower.accept(reply(changed)).is_err());
        assert_eq!(follower.observation().unwrap().data(), &before);
        follower.accept(reply(trace[index + 1].clone())).unwrap();
    }
    let bow = &traces["bow-a"];
    let other = &traces["bow-b"];
    assert_eq!(
        bow[1].presentation.as_ref().unwrap().0[0]
            .shot
            .as_ref()
            .unwrap()
            .loading_elapsed_nanos,
        123_000_000
    );
    assert_eq!(bow[1].presentation, bow[2].presentation);
    assert_eq!(bow[2].presentation, bow[3].presentation);
    assert_ne!(bow[3].presentation, bow[4].presentation);
    assert!(!bow[4].personal.ammo.as_ref().unwrap().stocks.0.is_empty());
    assert!(other[4].personal.ammo.as_ref().unwrap().stocks.0.is_empty());
    let items = &traces["item-a"];
    assert_eq!(
        items[1].presentation.as_ref().unwrap().0[0]
            .item
            .as_ref()
            .unwrap()
            .effect,
        189
    );
    assert_eq!(
        items[2].presentation.as_ref().unwrap().0[0]
            .item
            .as_ref()
            .unwrap()
            .effect,
        191
    );
    assert_eq!(items[2].presentation, items[3].presentation);
    assert_eq!(items[3].presentation, items[4].presentation);
    eprintln!("presentation schema2:20 real frames,4 owner streams; old twelve sections unchanged; private ammo isolated; denied/expiry retain event; two non-owner historical rewrites reject atomically");
}

#[test]
#[ignore = "explicit compiled Java item-effects peer plus installed shared Lineage resources; synthetic server geometry, not GUI/TLS"]
fn public_item_events_render_shared_art_and_preserve_skill_cues() {
    let traces = public_traces("item-effects", 38);
    crate::modern_lito::battle_scene::tests::verify_item_scenes(&traces);
}

#[test]
#[ignore = "explicit compiled Java bow-effects peer and installed shared SPR66/bow21 assets; synthetic geometry, not GUI/TLS"]
fn public_bows_use_native_hit_launch_and_immutable_class_three_flights() {
    let traces = public_traces("bow-effects", 44);
    crate::modern_lito::battle_scene::tests::verify_bow_scenes(&traces);
}

#[test]
#[ignore = "compiled Java healing-effects plus shared action19/744/Sound109; real engine, synthetic geometry, no TLS/GUI"]
fn confirmed_healing_uses_shared_action19_fixed744_and_deduplicated109() {
    let traces = public_traces("healing-effects", 36);
    crate::modern_lito::battle_scene::tests::verify_healing_scenes(&traces);
}

fn public_traces(mode: &str, expected: usize) -> BTreeMap<String, Vec<data::UnvalidatedData>> {
    let mut peer = Peer::start_named(Some(mode));
    let mut traces: BTreeMap<String, Vec<data::UnvalidatedData>> = BTreeMap::new();
    let mut count = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, format!("CLOSED:{expected}"));
            break;
        }
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields[0], "FRAME");
        let bytes = fields[3]
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let data = decode(&bytes).unwrap();
        data.check_scene_values(&data.personal.player, &data.world.identity)
            .unwrap();
        assert_eq!(encode(&data).unwrap(), bytes);
        peer.ack(&bytes);
        traces.entry(fields[1].to_owned()).or_default().push(data);
        count += 1;
    }
    peer.finish();
    assert_eq!(count, expected);
    for trace in traces.values() {
        let mut follower = Follower::new(request(&trace[0]), ack(36)).unwrap();
        for d in trace {
            follower.accept(reply(d.clone())).unwrap();
        }
    }
    traces
}
