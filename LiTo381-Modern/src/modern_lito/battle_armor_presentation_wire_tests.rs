//! Real Java schema9 application replies and ordinary immutable admission.
//! Does not qualify assets, issue armor LOAD or claim rendered/native UI.
use super::{decode, healing, reject, reply, request, selected, unhex, Peer};
use crate::modern_lito::{battle_follow::Follower, battle_payload as payload, protocol};
use payload::armor_presentation as p;
use std::collections::BTreeMap;

fn metadata(d: &payload::UnvalidatedData) -> &p::Presentation {
    d.armor.as_ref().unwrap().presentation.as_ref().unwrap()
}
fn metadata_mut(d: &mut payload::UnvalidatedData) -> &mut p::Presentation {
    d.armor.as_mut().unwrap().presentation.as_mut().unwrap()
}
#[test]
fn presentation_has_explicit_dependent_selection_and_a_separate_bounded_capacity() {
    use protocol::extensions as e;
    assert_eq!(e::HEALING.required, 3);
    assert_eq!(e::ARMOR.required, 4);
    assert_eq!(e::HEALING_ARMOR.required, 7);
    assert_eq!(e::ARMOR_PRESENTATION.required, 12);
    assert_eq!(e::HEALING_ARMOR_PRESENTATION.required, 15);
    for ext in [12, 13, 15] {
        let peer = selected(ext);
        assert!(peer.allows_armor_presentation() && peer.allows_armor());
        assert_eq!(peer.allows_data(), ext != 12);
        assert_eq!(peer.allows_commands(), ext == 15);
        assert_eq!(payload::Format::Selected(peer).maximum(), p::MAX_BYTES);
    }
    for ext in [0, 1, 3, 4, 5, 7] {
        assert!(!selected(ext).allows_armor_presentation());
    }
    for ext in [2, 6, 8, 9, 10, 11, 14, 32, u32::MAX] {
        assert!(e::Selected::new(selected(15).base(), ext).is_err());
    }
    assert_eq!(p::LIMIT, 22654);
    assert_eq!(p::BATCH_LIMIT, 18485);
}
fn framing(raw: &[u8]) {
    assert_eq!(&raw[4..6], &9u16.to_be_bytes());
    let mut offset = 16;
    for id in 1..=17u16 {
        assert_eq!(&raw[offset..offset + 2], &id.to_be_bytes());
        let length = u32::from_be_bytes(raw[offset + 4..offset + 8].try_into().unwrap()) as usize;
        if id == 17 {
            assert_eq!(offset + 8 + length, raw.len());
            for (at, bytes) in [
                (offset, 18u16.to_be_bytes().to_vec()),
                (offset + 2, 1u16.to_be_bytes().to_vec()),
                (offset + 4, ((p::LIMIT + 1) as u32).to_be_bytes().to_vec()),
                (
                    offset + 8 + 56,
                    ((p::GRAPHICS_LIMIT + 1) as u32).to_be_bytes().to_vec(),
                ),
            ] {
                let mut bad = raw.to_vec();
                bad[at..at + bytes.len()].copy_from_slice(&bytes);
                assert!(p::decode(&bad).is_err());
            }
        }
        offset += 8 + length;
    }
    for n in [0, 15, 16, raw.len() - 1] {
        assert!(p::decode(&raw[..n]).is_err());
    }
    let mut bad = raw.to_vec();
    bad.push(0);
    assert!(p::decode(&bad).is_err());
    for (at, value) in [(6, 1), (12, 1), (13, 16), (14, 1)] {
        let mut bad = raw.to_vec();
        bad[at] = value;
        assert!(p::decode(&bad).is_err());
    }
}
fn mutations(f: &mut Follower, d: &payload::UnvalidatedData) -> usize {
    let mut count = 0;
    for mutate in [
        (|p: &mut p::Presentation| p.world.generation += 1) as fn(&mut p::Presentation),
        |p| p.world_revision += 1,
        |p| p.entity += 1,
        |p| p.life += 1,
        |p| p.equipment_revision += 1,
        |p| p.required.graphics.0 = vec![228, 228],
        |p| p.required.graphics.0 = vec![11101],
        |p| p.required.graphics.0 = vec![65536],
        |p| p.required.icons.0 = vec![0],
    ] {
        let mut bad = d.clone();
        mutate(metadata_mut(&mut bad));
        assert!(payload::encode(&bad).is_err());
        reject(f, bad);
        count += 1;
    }
    let mut missing = d.clone();
    missing.armor.as_mut().unwrap().presentation = None;
    // Old codec remains valid in isolation, but it cannot downgrade this session.
    let old = payload::encode(&missing).unwrap();
    assert_eq!(&old[4..6], &8u16.to_be_bytes());
    assert!(p::decode(&old).is_err());
    reject(f, missing);
    count += 1;
    if metadata(d).batch.is_some() {
        for mutate in [
            (|p: &mut p::Presentation| p.batch.as_mut().unwrap().sequence += 1)
                as fn(&mut p::Presentation),
            |p| p.batch.as_mut().unwrap().revision += 1,
            |p| p.batch.as_mut().unwrap().observed_nanos += 1,
            |p| p.batch.as_mut().unwrap().anchor.move_sequence = i64::MAX,
            |p| p.batch.as_mut().unwrap().anchor.cell.x = i32::MAX,
            |p| p.batch.as_mut().unwrap().visuals.0[0].ordinal += 1,
            |p| p.batch.as_mut().unwrap().visuals.0[0].graphic += 1,
            |p| p.batch.as_mut().unwrap().visuals.0[0].role = p::Role::SkillCast,
            |p| {
                p.batch.as_mut().unwrap().visuals.0.pop();
            },
            |p| {
                let v = p.batch.as_ref().unwrap().visuals.0[0].clone();
                p.batch.as_mut().unwrap().visuals.0.push(v);
            },
            |p| p.required.graphics.0 = vec![228],
            |p| p.batch = None,
        ] {
            let mut bad = d.clone();
            mutate(metadata_mut(&mut bad));
            assert!(payload::encode(&bad).is_err());
            reject(f, bad);
            count += 1;
        }
    }
    count
}

#[test]
#[ignore = "actual Java schema9 selected application replies; full follower/command and mutation proof; not TLS/assets/UI"]
fn java_armor_presentation_schema9_preserves_owner_anchor_multiplicity_and_atomic_admission() {
    let mut peer = Peer::start_named(Some("armor9"));
    let mut sessions = BTreeMap::new();
    let mut followers = BTreeMap::new();
    let mut histories: BTreeMap<String, Vec<payload::UnvalidatedData>> = BTreeMap::new();
    let mut frames = BTreeMap::new();
    let mut heal_before = None;
    let mut count = 0;
    let mut rejected = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:94");
            break;
        }
        if line.starts_with("SESSION:") {
            let fields = line.splitn(4, ':').collect::<Vec<_>>();
            let ext = fields[1].parse::<u32>().unwrap();
            let raw = unhex(fields[2]);
            let protocol::Message::ExtendedHello(h, offer) = protocol::decode(&raw).unwrap() else {
                panic!("extended hello")
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
        let selection = sessions[&ext];
        let q = unhex(fields[3]);
        let b = unhex(fields[4]);
        if label == "heal-cast" {
            healing(heal_before.as_ref().unwrap(), &b, &q, selection);
            peer.ack(&b);
            count += 1;
            continue;
        }
        let d = decode(&b, &q, selection).unwrap_or_else(|e| panic!("{label}: {e:?}"));
        let raw = &b[72..];
        assert_eq!(p::decode(raw).unwrap(), d);
        assert_eq!(payload::encode(&d).unwrap(), raw);
        framing(raw);
        assert!(payload::armor::decode(raw).is_err());
        assert!(payload::healing::decode(raw).is_err());
        for old in [0, 1, 3, 4, 5, 7] {
            assert!(decode(&b, &q, selected(old)).is_err());
        }
        for version in 1..=47 {
            assert!(payload::decode_versioned(raw, version).is_err());
        }
        for ext in [13, 15] {
            assert_eq!(decode(&b, &q, sessions[&ext]).unwrap(), d);
        }
        if d.armor.as_ref().unwrap().healing.is_some() {
            assert!(decode(&b, &q, sessions[&12]).is_err());
        }
        let group = if label.starts_with("status") {
            label.split('-').take(2).collect::<Vec<_>>().join("-")
        } else if label.starts_with("dead") || label.starts_with("revived") {
            "life".to_owned()
        } else if ["bots", "quest", "heal-", "anchor", "mode", "max"]
            .iter()
            .any(|p| label.starts_with(p))
        {
            label.split('-').next().unwrap().to_owned()
        } else {
            "basic".to_owned()
        };
        let key = format!("{group}-{}", d.personal.entity);
        let f = followers
            .entry(key.clone())
            .or_insert_with(|| Follower::new_selected(request(&d), selection).unwrap());
        f.accept(reply(d.clone()))
            .unwrap_or_else(|e| panic!("follow {label}: {e:?}"));
        assert_eq!(f.observation().unwrap().data(), &d);
        f.accept(reply(d.clone())).unwrap();
        // Aborted worlds are never paintable. Input acceptance is only syntax,
        // not the shared-resource/LOAD proof exercised by the renderer tests.
        let scene =
            crate::modern_lito::battle_scene::baseline_counts(f.observation().unwrap().clone());
        assert_eq!(
            scene.is_ok(),
            d.world.phase != payload::data::Phase::Aborted,
            "{label}: {scene:?}"
        );
        rejected += mutations(f, &d);
        if label == "heal-before" {
            heal_before = Some(d.clone());
        } else {
            histories.entry(key).or_default().push(d.clone());
        }
        frames.insert(label.to_owned(), d);
        peer.ack(&b);
        count += 1;
    }
    peer.finish();
    assert_eq!(count, 94);
    assert_eq!(sessions.len(), 3);
    let mut pairs = 0;
    for history in histories.values() {
        for (i, a) in history.iter().enumerate() {
            for b in history.iter().skip(i) {
                let mut f = Follower::new_selected(request(a), sessions[&15]).unwrap();
                f.accept(reply(a.clone())).unwrap();
                f.accept(reply(b.clone())).unwrap();
                if a.world.revision < b.world.revision {
                    reject(&mut f, a.clone());
                }
                pairs += 1;
            }
        }
    }
    // Stored owner batch must not follow a later move or teleport snapshot.
    let hit = &frames["anchor-hit-1"];
    let moved = &frames["anchor-moved-1"];
    let teleport = &frames["anchor-teleport-1"];
    assert_eq!(metadata(hit).batch, metadata(moved).batch);
    assert_eq!(metadata(hit).batch, metadata(teleport).batch);
    let mut f = Follower::new_selected(request(hit), sessions[&12]).unwrap();
    f.accept(reply(hit.clone())).unwrap();
    let mut bad = moved.clone();
    metadata_mut(&mut bad)
        .batch
        .as_mut()
        .unwrap()
        .anchor
        .move_sequence += 1;
    bad.check_scene_values(&bad.personal.player, &bad.world.identity)
        .unwrap();
    reject(&mut f, bad);
    let mut bad = moved.clone();
    metadata_mut(&mut bad).required.graphics.0.push(65535);
    bad.check_scene_values(&bad.personal.player, &bad.world.identity)
        .unwrap();
    reject(&mut f, bad);
    f.accept(reply(moved.clone())).unwrap();
    f.accept(reply(teleport.clone())).unwrap();
    assert!(metadata(&frames["anchor-clear-1"]).batch.is_none());
    for (mode, n, again) in [(0, 3, 2), (1, 3, 2), (2, 5, 4)] {
        assert_eq!(
            metadata(&frames[&format!("mode{mode}-hit-1")])
                .batch
                .as_ref()
                .unwrap()
                .visuals
                .0
                .len(),
            n
        );
        assert_eq!(
            metadata(&frames[&format!("mode{mode}-again-1")])
                .batch
                .as_ref()
                .unwrap()
                .visuals
                .0
                .len(),
            again
        );
    }
    let max = metadata(&frames["max-hit-1"]);
    assert_eq!(max.required.graphics.0, vec![228, 11101]);
    assert_eq!(max.batch.as_ref().unwrap().visuals.0.len(), 2049);
    assert!(metadata(&frames["max-hit-0"]).batch.is_none());
    eprintln!("ARMOR_PRESENTATION_SELECTED: {count} replies; {} owner streams; {pairs} ordered pairs; {rejected} atomic rejections plus frozen anchor/requirements; 2049 ordered visuals; schema8 unchanged, no armor rendering/LOAD claim",histories.len());
}
