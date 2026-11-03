use super::{super::*, Peer};
use crate::modern_lito::battle_payload::armor as codec;

#[test]
#[ignore = "explicit compiled Java real engine peer; complete candidate scene/follower, not live TLS/UI"]
fn java_complete_armor_schema8_keeps_healing_equipment_status_bots_and_lanes() {
    let mut peer = Peer::start_named(Some("armor8"));
    let mut count = 0;
    let mut states = std::collections::BTreeMap::new();
    let mut followers = std::collections::BTreeMap::new();
    let mut frames = std::collections::BTreeMap::new();
    let mut histories = std::collections::BTreeMap::<String, Vec<codec::Candidate>>::new();
    let mut rejected_scenes = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:68");
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
        let c = codec::decode(&raw).unwrap_or_else(|e| panic!("{label}: {e:?}"));
        c.check_values(&c.data.personal.player, &c.data.world.identity)
            .unwrap_or_else(|e| panic!("complete scene {label}: {e:?}"));
        let group = if label.starts_with("status") {
            label.split('-').take(2).collect::<Vec<_>>().join("-")
        } else if label.starts_with("dead") || label.starts_with("revived") {
            "life".to_owned()
        } else if label.starts_with("bots") || label.starts_with("quest") {
            label.split('-').next().unwrap().to_owned()
        } else {
            "basic".to_owned()
        };
        let key = format!("{group}-{}", c.data.personal.entity);
        histories.entry(key.clone()).or_default().push(c.clone());
        let follower = followers.entry(key).or_insert_with(|| {
            codec::Follower::new(
                c.data.personal.player.clone(),
                c.data.world.identity.clone(),
            )
            .unwrap()
        });
        let prior = follower.current().map(|v| v.candidate().clone());
        if prior.is_none() {
            let mut wrong_owner = c.clone();
            wrong_owner.data.personal.player.epoch += 1;
            assert!(follower.accept(wrong_owner).is_err());
            assert!(follower.current().is_none());
        }
        if label.starts_with("first") {
            // Same GFX from separate configured armor rows are distinct facts.
            let mut duplicate = c.clone();
            let effects = &mut duplicate.actors.0[1].state.as_mut().unwrap().effects.0;
            let mut e = effects[0].clone();
            e.applied_protection = false;
            effects.extend(std::iter::repeat_n(e, 1024 - effects.len()));
            let mut fork = follower.clone();
            let events = fork.accept(duplicate.clone()).unwrap();
            assert_eq!(events.len(), 1024);
            for (i, e) in events.iter().enumerate() {
                assert_eq!(e.id.index, i);
                assert_eq!(e.id.entity, 2);
                assert_eq!(e.id.world, c.data.world.identity);
                assert_eq!(
                    e.id.revision,
                    c.actors.0[1].state.as_ref().unwrap().revision
                );
            }
            assert!(fork.accept(duplicate).unwrap().is_empty());
        }
        let events = follower
            .accept(c.clone())
            .unwrap_or_else(|e| panic!("follower {label}: {e:?}"));
        let expected =
            if prior.is_some() && (label.starts_with("first") || label.starts_with("second")) {
                c.actors
                    .0
                    .iter()
                    .filter_map(|a| a.state.as_ref())
                    .map(|s| s.effects.0.len())
                    .sum()
            } else {
                0
            };
        assert_eq!(events.len(), expected, "events {label}");
        assert!(
            follower.accept(c.clone()).unwrap().is_empty(),
            "duplicate {label}"
        );
        if let Some(p) = prior.filter(|p| p.data.world.revision < c.data.world.revision) {
            rejected(follower, p, label);
        }
        rejected_scenes += scene_mutations(follower, &c, label);
        if label.starts_with("first") || label.starts_with("second") {
            for newer_world in [false, true] {
                let mut bad = c.clone();
                if newer_world {
                    bad.data.world.revision += 1;
                    bad.data.personal.history.as_mut().unwrap().0.world_revision += 1;
                }
                bad.actors.0[1].state.as_mut().unwrap().effects.0[0].graphics ^= 1;
                // Individually well-formed, but it rewrites an old event.
                bad.check_values(&c.data.personal.player, &c.data.world.identity)
                    .unwrap_or_else(|e| {
                        panic!("rewrite fixture {label} newer={newer_world}: {e:?}")
                    });
                rejected(follower, bad, "retained event rewrite");
            }
        }
        frames.insert(label.to_owned(), c.clone());
        assert_eq!(raw, codec::encode(&c).unwrap(), "{label}");
        assert_eq!(format!("{c:?}"), "UnvalidatedArmorBattle[redacted]");
        assert!(decode(&raw).is_err());
        assert!(encode(&c.data).is_err());
        assert!(healing::decode(&raw).is_err());
        for version in 1..=crate::modern_lito::protocol::MAX_VERSION {
            assert!(decode_versioned(&raw, version).is_err());
        }
        let ack = crate::modern_lito::protocol::Ack {
            request_id: 1,
            version: 47,
            numeric_schema: 1,
            capabilities: crate::modern_lito::protocol::KNOWN_CAPABILITIES,
        };
        let selected = crate::modern_lito::protocol::extensions::Selected::new(ack, 3).unwrap();
        assert!(decode_selected(&raw, selected).is_err());
        assert!(c
            .data
            .check_scene_values(&c.data.personal.player, &c.data.world.identity)
            .is_err());
        for mutation in 0..9 {
            let mut bad = c.clone();
            match mutation {
                0 => bad.data.world.descriptor.features &= !codec::FEATURE,
                1 => bad.actors.0[0].entity += 100,
                2 => bad.actors.0[0].state.as_mut().unwrap().entity += 1,
                3 => bad.actors.0[0].state.as_mut().unwrap().life += 1,
                4 => bad.actors.0[0].state.as_mut().unwrap().revision = i64::MAX,
                5 => bad.actors.0[0].state = None,
                6 => bad.actors.0[0].state.as_mut().unwrap().observed_nanos = i64::MAX,
                7 => {
                    bad.actors.0.push(bad.actors.0[0].clone());
                }
                _ => bad.data.room.members.0[0].identity.kind = data::MemberKind::ServerBot,
            }
            // Lane1's first visible member is entity2, not room member0.
            if mutation == 8 && c.data.world.lane == 1 {
                continue;
            }
            assert!(codec::encode(&bad).is_err(), "{label}, mutation {mutation}");
        }
        if label.starts_with("first") {
            let s = c.actors.0[1].state.as_ref().unwrap();
            assert_eq!(s.until_nanos, 3_000_000_000);
            assert!(s.effects.0[0].applied_protection);
            for mutation in 0..5 {
                let mut bad = c.clone();
                let s = bad.actors.0[1].state.as_mut().unwrap();
                match mutation {
                    0 => s.until_nanos += 1,
                    1 => s.effects.0[0].graphics = 65536,
                    2 => s.effects.0[0].graphics_mode = 3,
                    3 => s.effects.0.push(s.effects.0[0].clone()),
                    _ => bad.data.world.active_elapsed_nanos = 3_000_000_000,
                }
                assert!(codec::encode(&bad).is_err());
            }
        }
        if label.starts_with("second") {
            assert!(!c.actors.0[1].state.as_ref().unwrap().effects.0[0].applied_protection);
        }
        if label.starts_with("expired") || label.starts_with("dead") || label.starts_with("aborted")
        {
            assert!(c
                .actors
                .0
                .iter()
                .filter_map(|a| a.state.as_ref())
                .all(|s| s.until_nanos == 0 && s.effects.0.is_empty()));
        }
        if label.starts_with("revived") {
            let s = c.actors.0[1].state.as_ref().unwrap();
            assert_eq!(s.life, 1);
            assert_eq!(s.sequence, 0);
        }
        if label.starts_with("bots") {
            assert_eq!(c.actors.0.iter().filter(|a| a.state.is_none()).count(), 2);
        }
        if label.starts_with("quest") {
            assert_eq!(c.actors.0.len(), 1);
            assert_eq!(c.actors.0[0].entity, c.data.world.lane as i64 + 1);
        }
        if label.starts_with("status") {
            let key = format!(
                "{}-{}",
                label.split('-').take(2).collect::<Vec<_>>().join("-"),
                c.data.personal.entity
            );
            if label.contains("-hit-") {
                states.insert(key.clone(), c.actors.clone());
            } else {
                assert_eq!(
                    states.get(&key).unwrap(),
                    &c.actors,
                    "unrelated heal/gear must not replay armor {label}"
                );
            }
            assert_eq!(c.healing.is_some(), label.contains("-true-"));
            assert_eq!(
                c.data.personal.materials.is_some(),
                label.starts_with("status2")
            );
            match &c.status {
                healing::Status::Motion(_) => assert!(label.starts_with("status0")),
                healing::Status::Haste(_) => assert!(label.starts_with("status1")),
                healing::Status::Speed(_) => assert!(label.starts_with("status2")),
            }
            if label.contains("-unequipped-0") {
                assert_eq!(c.data.personal.equipment.as_ref().unwrap().revision, 1);
            }
            if label.contains("-cast-haste-") {
                assert!(c.data.actors.0[0]
                    .spells
                    .histories
                    .0
                    .iter()
                    .any(|s| s.lane == data::Lane::Haste && s.receipt.is_some()));
            }
            if label.contains("-cast-slow-") {
                assert!(c.data.actors.0[1]
                    .spells
                    .histories
                    .0
                    .iter()
                    .any(|s| s.lane == data::Lane::Slow && s.receipt.is_some()));
                if c.data.personal.entity == 2 {
                    assert!(c.data.personal.materials.as_ref().unwrap().uses > 0);
                }
            }
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
            let mut at = 16;
            while at < raw.len() {
                let size = u32::from_be_bytes(raw[at + 4..at + 8].try_into().unwrap()) as usize;
                for offset in [0, 2, 4] {
                    let mut b = raw.clone();
                    b[at + offset] ^= 0xff;
                    assert!(codec::decode(&b).is_err());
                }
                at += 8 + size;
            }
        }
        peer.ack(&codec::encode(&c).unwrap());
        count += 1;
    }
    assert_eq!(count, 68);
    assert_eq!(states.len(), 12);
    assert_eq!(followers.len(), 20);
    let mut pairs = 0;
    for history in histories.values() {
        for (i, a) in history.iter().enumerate() {
            for b in &history[i..] {
                a.check_successor(b)
                    .unwrap_or_else(|e| panic!("coalesced pair {pairs}: {e:?}"));
                pairs += 1;
            }
        }
    }
    assert_eq!(pairs, 176);
    assert_eq!(rejected_scenes, 604);
    eprintln!("ARMOR_SCENE_FOLLOW: frames={count} owners={} ordered_pairs={pairs} rejected_scene_mutations={rejected_scenes}", followers.len());
    for owner in 0..2 {
        let a = &frames[&format!("active-{owner}")];
        let b = &frames[&format!("second-{owner}")];
        a.check_successor(b).unwrap(); // Coalesced hit history, no HP-loss guess.
        let mut f = codec::Follower::new(
            a.data.personal.player.clone(),
            a.data.world.identity.clone(),
        )
        .unwrap();
        assert!(f.accept(b.clone()).unwrap().is_empty()); // Attach never replays.
        assert!(f
            .accept(frames[&format!("expired-{owner}")].clone())
            .unwrap()
            .is_empty());
        assert!(f
            .accept(frames[&format!("aborted-{owner}")].clone())
            .unwrap()
            .is_empty());
    }
    peer.finish();
}

fn rejected(f: &mut codec::Follower, bad: codec::Candidate, label: &str) {
    let prior = f.current().unwrap().candidate().clone();
    assert!(f.accept(bad).is_err(), "must reject {label}");
    assert_eq!(
        f.current().unwrap().candidate(),
        &prior,
        "atomic rejection {label}"
    );
}
fn scene_mutations(f: &mut codec::Follower, c: &codec::Candidate, label: &str) -> usize {
    let mut count = 8;
    for mutation in 0..8 {
        let mut bad = c.clone();
        match mutation {
            0 => bad.data.personal.player.epoch += 1,
            1 => bad.data.world.identity.generation += 1,
            2 => bad.data.actors.0[0].position.x = i32::MAX,
            3 => bad.data.actors.0[0].vital.as_mut().unwrap().values.hp += 1,
            4 => bad.data.world.descriptor.features |= 1 << 30,
            5 => bad.data.personal.entity += 100,
            6 => bad.data.world.loading_elapsed_nanos = -1,
            _ => bad.actors.0[0].state.as_mut().unwrap().life += 1,
        }
        rejected(f, bad, &format!("{label} scene mutation {mutation}"));
    }
    if c.healing.is_some() {
        let mut bad = c.clone();
        bad.healing.as_mut().unwrap().0[0].incoming.received += 1;
        rejected(f, bad, label);
        count += 1;
    }
    let mut bad = c.clone();
    match &mut bad.status {
        healing::Status::Speed(s) => s.revision += 1,
        healing::Status::Haste(s) => s.revision += 1,
        healing::Status::Motion(_) => return count,
    }
    rejected(f, bad, label);
    count + 1
}
