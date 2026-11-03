//! Actual Java publications: candidate facts plus production v44 admission,
//! atomic Follower/Observation timelines and exact bytes. No TLS or UI here.
use super::{
    super::{self as payload, data, haste},
    Peer,
};
#[path = "battle_haste_admission_tests.rs"]
mod admission;
#[path = "battle_haste_contract_tests.rs"]
mod contract;
#[path = "battle_target_haste_wire_tests.rs"]
mod target_command;
#[path = "battle_haste_time_attack_wire_tests.rs"]
mod time_attack;
fn reject(p: &haste::Candidate, edit: impl FnOnce(&mut haste::Candidate)) {
    let mut bad = p.clone();
    edit(&mut bad);
    let parsed = haste::decode(&haste::encode(&bad).unwrap()).unwrap();
    assert!(parsed.check_effects().is_err());
    assert!(parsed
        .check_values(&p.data.personal.player, &p.data.world.identity)
        .is_err());
    admission::reject(p, &parsed);
    p.check_effects().unwrap();
}

#[test]
#[ignore = "explicit compiled Java schema5 peer; production v44 Follower, no TLS or UI"]
fn java_haste43_schema5_complete_targets_owner_history_and_legacy_rejection() {
    let mut peer = Peer::start_named(Some("haste43"));
    let mut frames = Vec::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:77");
            break;
        }
        let fields = line.splitn(3, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], "HASTE43");
        let hex = fields[2];
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let p = haste::decode(&bytes).unwrap();
        p.check_effects().unwrap();
        p.check_values(&p.data.personal.player, &p.data.world.identity)
            .unwrap_or_else(|e| panic!("{}: {e:?}", fields[1]));
        assert_eq!(p.effects.world, p.data.world.identity);
        assert_eq!(p.effects.revision, p.data.world.revision);
        assert_eq!(haste::encode(&p).unwrap(), bytes);
        peer.ack(&bytes);
        // Old versions and a stripped extension remain closed; only explicit
        // v44+ decoding attaches the complete data for negotiated admission.
        for version in (1..=43).chain([crate::modern_lito::protocol::MAX_VERSION + 1, u16::MAX]) {
            assert!(payload::decode_versioned(&bytes, version).is_err());
        }
        assert_eq!(
            payload::decode_versioned(&bytes, 45).unwrap(),
            payload::decode_versioned(&bytes, 44).unwrap()
        );
        assert!(payload::decode(&bytes).is_err());
        assert!(payload::encode(&p.data).is_err());
        assert!(p
            .data
            .check_scene_values(&p.data.personal.player, &p.data.world.identity)
            .is_err());
        assert!(p.data.npc_motion.is_none());
        admission::values(&p);
        let history = &p.data.personal.history.as_ref().unwrap().0;
        let h = crate::modern_lito::equipment_history_wire::encode_with_schema(history, 3).unwrap();
        assert_eq!(
            *history,
            crate::modern_lito::equipment_history_wire::decode_with_schema(&h, 3).unwrap()
        );
        for schema in [1, 2] {
            assert!(
                crate::modern_lito::equipment_history_wire::decode_with_schema(&h, schema).is_err()
            );
        }
        frames.push((fields[1].to_owned(), p, bytes));
    }
    peer.finish();
    assert_eq!(frames.len(), 77);
    let frame = |label: &str| &frames.iter().find(|f| f.0 == label).unwrap().1;
    let mut transitions = 0;
    for stages in [
        vec![
            "pc-loading",
            "pc-active",
            "pc-target",
            "pc-other",
            "pc-refresh",
            "pc-repeat",
            "pc-expired",
            "pc-abort",
        ],
        vec![
            "npc-normal",
            "npc-haste",
            "npc-walk",
            "npc-refresh",
            "npc-expired",
            "npc-resumed",
            "npc-abort",
        ],
        vec![
            "gear-initial",
            "gear-cast",
            "gear-set",
            "gear-recast",
            "gear-revived",
        ],
        vec!["mixed-43", "mixed-54", "mixed-back", "mixed-potion"],
        vec!["quest-start", "quest-combat", "quest-finished"],
        vec!["corpse-cast", "corpse-dead", "corpse-retired"],
        vec![
            "elimination-start",
            "elimination-haste",
            "elimination-finished",
        ],
    ] {
        for owner in 1..=2 {
            for i in 0..stages.len() {
                for j in i..stages.len() {
                    let x = format!("{}-{owner}", stages[i]);
                    let y = format!("{}-{owner}", stages[j]);
                    let a = frame(&x);
                    let b = frame(&y);
                    a.check_successor(b)
                        .unwrap_or_else(|e| panic!("{x} -> {y}: {e:?}"));
                    admission::transition(a, b);
                    if b.data.world.revision > a.data.world.revision {
                        assert!(b.check_successor(a).is_err());
                    }
                    transitions += 1;
                }
            }
        }
    }
    for (i, a) in ["bot-start-1", "bot-target-1", "bot-abort-1"]
        .iter()
        .enumerate()
    {
        for b in ["bot-start-1", "bot-target-1", "bot-abort-1"]
            .iter()
            .skip(i)
        {
            frame(a).check_successor(frame(b)).unwrap();
            admission::transition(frame(a), frame(b));
            transitions += 1;
        }
    }
    assert_eq!(transitions, 220);
    assert!(frame("pc-expired-1").effects.targets.0.is_empty());
    let resumed = &frame("npc-resumed-1")
        .effects
        .npcs
        .as_ref()
        .unwrap()
        .actors
        .0[0];
    assert_eq!(resumed.walk_speed.haste_until_millis, 0);
    assert_eq!(resumed.last_walk.as_ref().unwrap().duration_millis, 100);
    assert!(frame("npc-resumed-1").effects.targets.0.is_empty());
    assert_eq!(
        frame("elimination-finished-1").data.world.phase,
        data::Phase::Finished
    );
    assert!(frame("elimination-finished-1").effects.targets.0.is_empty());
    contract::verify(&frames);
    assert!(frame("pc-loading-1").effects.targets.0.is_empty());
    assert_eq!(
        frame("pc-wide-1").data.actors.0[0]
            .resources
            .as_ref()
            .unwrap()
            .vitals
            .mp,
        4_999_999_960
    );
    assert_eq!(
        frame("pc-wide-1").data.actors.0[0]
            .resources
            .as_ref()
            .unwrap()
            .vitals
            .maximum_mp,
        5_000_000_000
    );
    assert_eq!(frame("pc-other-1").effects.targets.0.len(), 2);
    assert_eq!(frame("pc-other-1").effects, frame("pc-other-2").effects);
    assert_ne!(
        frame("pc-other-1").data.personal.player,
        frame("pc-other-2").data.personal.player
    );
    let before = &frame("npc-haste-1").effects.npcs.as_ref().unwrap().actors.0[0];
    assert_eq!(before.walk_millis, 100);
    assert_eq!(before.last_walk.as_ref().unwrap().duration_millis, 100);
    assert_eq!(before.walk_speed.haste_until_millis, 0);
    assert!(before.speed.haste_until_millis > 0);
    let stepped = &frame("npc-walk-1").effects.npcs.as_ref().unwrap().actors.0[0];
    assert_eq!(stepped.last_walk.as_ref().unwrap().duration_millis, 75);
    let refreshed = &frame("npc-refresh-1").effects;
    assert_ne!(refreshed.targets.0[0].active, refreshed.targets.0[0].walk);
    assert_eq!(
        refreshed.npcs.as_ref().unwrap().actors.0[0].last_walk,
        stepped.last_walk
    );
    let expired = &frame("npc-expired-1").effects;
    assert!(expired.targets.0[0].active.is_none());
    assert!(expired.targets.0[0].walk.is_some());
    assert_eq!(
        expired.npcs.as_ref().unwrap().actors.0[0]
            .speed
            .haste_until_millis,
        0
    );
    assert_eq!(
        expired.npcs.as_ref().unwrap().actors.0[0].last_walk,
        stepped.last_walk
    );
    for i in 1..=2 {
        let p = frame(&format!("quest-lanes-{i}"));
        assert_eq!(p.data.actors.0.len(), 1);
        assert_eq!(p.data.actors.0[0].entity, i);
        assert_eq!(p.effects.targets.0.len(), 1);
        assert_eq!(
            p.effects.targets.0[0]
                .active
                .as_ref()
                .unwrap()
                .caster
                .entity,
            i
        );
    }
    let full = frame("npc-full-1");
    assert!(frame("mixed-54-1").effects.targets.0.is_empty());
    assert_eq!(
        frame("mixed-54-1").data.actors.0[0]
            .buffs
            .as_ref()
            .unwrap()
            .haste_origin,
        data::HasteOrigin::GreaterHaste
    );
    assert_eq!(frame("mixed-back-1").effects.targets.0.len(), 1);
    assert!(frame("mixed-potion-1").effects.targets.0.is_empty());
    assert_eq!(
        frame("mixed-potion-1").data.actors.0[0]
            .buffs
            .as_ref()
            .unwrap()
            .haste_origin,
        data::HasteOrigin::Potion
    );
    assert_eq!(frame("pc-full-100").effects.targets.0.len(), 100);
    assert_eq!(frame("pc-full-100").data.personal.entity, 100);
    assert_eq!(frame("bot-target-1").effects.targets.0.len(), 1);
    assert!(
        matches!(&frame("bot-target-1").effects.targets.0[0].target,haste::Target::Player(p) if p.identity.kind==data::MemberKind::ServerBot)
    );
    let won = frame("quest-finished-2");
    assert_eq!(won.data.world.phase, data::Phase::Finished);
    assert!(won.effects.targets.0[0].active.is_none());
    assert!(won.effects.targets.0[0].walk.is_some());
    let gear = frame("gear-recast-1");
    assert_eq!(gear.effects.targets.0.len(), 2);
    assert_eq!(
        gear.effects.targets.0[1]
            .active
            .as_ref()
            .unwrap()
            .receipt
            .spent,
        35
    );
    assert_eq!(
        gear.effects.targets.0[0]
            .active
            .as_ref()
            .unwrap()
            .receipt
            .spent,
        75
    );
    assert!(
        frame("gear-revived-1").data.actors.0[0]
            .resources
            .as_ref()
            .unwrap()
            .life
            > frame("gear-cast-1").data.actors.0[0]
                .resources
                .as_ref()
                .unwrap()
                .life
    );
    assert_eq!(full.effects.targets.0.len(), 128);
    assert_eq!(full.effects.npcs.as_ref().unwrap().actors.0.len(), 128);
    let last = full.effects.targets.0[127].active.as_ref().unwrap();
    assert_eq!(last.receipt.casts, 128);
    assert_eq!(last.receipt.spent, 5120);
    assert!(matches!(last.receipt.detail, data::Detail::Haste(_)));
    let pc = frame("pc-other-1");
    reject(pc, |p| p.effects.world.generation += 1);
    reject(pc, |p| p.effects.revision -= 1);
    reject(pc, |p| {
        p.effects.targets.0.remove(0);
    });
    reject(pc, |p| p.effects.targets.0.swap(0, 1));
    reject(pc, |p| {
        p.effects.targets.0[0]
            .active
            .as_mut()
            .unwrap()
            .caster
            .identity
            .source += 1
    });
    reject(pc, |p| {
        p.effects.targets.0[0]
            .active
            .as_mut()
            .unwrap()
            .receipt
            .spent += 1
    });
    reject(pc, |p| {
        p.effects.targets.0[0]
            .active
            .as_mut()
            .unwrap()
            .receipt
            .loading_elapsed_nanos += 1
    });
    reject(pc, |p| {
        p.data.actors.0[0].buffs.as_mut().unwrap().haste_until_nanos += 1
    });
    let refreshed = frame("npc-refresh-1");
    reject(refreshed, |p| p.effects.targets.0[0].walk = None);
    reject(refreshed, |p| {
        p.effects.targets.0[0].active = p.effects.targets.0[0].walk.clone()
    });
    reject(refreshed, |p| {
        p.effects.npcs.as_mut().unwrap().actors.0[0]
            .last_walk
            .as_mut()
            .unwrap()
            .duration_millis = 100
    });
    reject(refreshed, |p| {
        p.effects.npcs.as_mut().unwrap().actors.0[0].walk_speed =
            p.effects.npcs.as_ref().unwrap().actors.0[0].speed.clone()
    });
    reject(refreshed, |p| {
        p.effects.npcs.as_mut().unwrap().actors.0[0].id.serial = i64::MAX
    });
    reject(refreshed, |p| {
        p.data.npcs.as_mut().unwrap().actors.0[0].born_revision = i64::MIN
    });
    reject(frame("quest-lanes-1"), |p| {
        p.effects = frame("quest-lanes-2").effects.clone()
    });
    reject(full, |p| {
        p.effects.targets.0[127]
            .active
            .as_mut()
            .unwrap()
            .receipt
            .casts -= 1
    });
    let mut too_many = full.clone();
    let extra = too_many.effects.npcs.as_ref().unwrap().actors.0[0].clone();
    too_many.effects.npcs.as_mut().unwrap().actors.0.push(extra);
    assert!(haste::encode(&too_many).is_err());
    let mut too_many = full.clone();
    while too_many.effects.targets.0.len() <= 228 {
        too_many
            .effects
            .targets
            .0
            .push(full.effects.targets.0[0].clone());
    }
    assert!(haste::encode(&too_many).is_err());
    for (_, _, bytes) in &frames {
        let mut cuts = (0..bytes.len()).step_by(97).collect::<Vec<_>>();
        cuts.extend(0..64.min(bytes.len()));
        cuts.extend(bytes.len().saturating_sub(16)..bytes.len());
        for end in cuts {
            assert!(haste::decode(&bytes[..end]).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(haste::decode(&trailing).is_err());
        for schema in [0u16, 1, 2, 3, 4, 6, u16::MAX] {
            let mut wrong = bytes.clone();
            wrong[4..6].copy_from_slice(&schema.to_be_bytes());
            assert!(haste::decode(&wrong).is_err());
        }
        for count in [0u16, 13, 15, u16::MAX] {
            let mut wrong = bytes.clone();
            wrong[12..14].copy_from_slice(&count.to_be_bytes());
            assert!(haste::decode(&wrong).is_err());
        }
    }
}
#[test]
#[ignore = "explicit Java schema5 publications and shared755/action19; target PC/BOT/NPC visuals and NPC pace, no HWND"]
fn java_haste43_shared_target_visual_and_npc_motion() {
    let mut peer = Peer::start_named(Some("haste43"));
    let mut frames = std::collections::BTreeMap::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:77");
            break;
        }
        let fields = line.splitn(3, ':').collect::<Vec<_>>();
        assert_eq!(fields[0], "HASTE43");
        let hex = fields[2];
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let d = payload::decode_versioned(&bytes, 44).unwrap();
        peer.ack(&bytes);
        frames.insert(fields[1].to_owned(), d);
    }
    peer.finish();
    for (before, used) in [
        ("pc-active-1", "pc-target-1"),
        ("npc-normal-1", "npc-haste-1"),
        ("bot-start-1", "bot-target-1"),
    ] {
        crate::modern_lito::battle_target_haste_visual::tests::verify(
            frames[before].clone(),
            frames[used].clone(),
        );
    }
    crate::modern_lito::battle_motion::npc_pacing_tests::verify_haste_frames(&frames);
}
