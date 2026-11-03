use super::super::{protocol as session, world_protocol as world};
use super::*;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::io::Read;
#[path = "disintegrate_wire_golden.rs"]
mod golden;

fn bytes(name: &str) -> Vec<u8> {
    let (_, size, sha, text) = golden::GOLDENS.iter().find(|(n, ..)| *n == name).unwrap();
    let zip = base64::engine::general_purpose::STANDARD
        .decode(text)
        .unwrap();
    let mut raw = Vec::new();
    flate2::read::GzDecoder::new(&zip[..])
        .take((*size + 1) as u64)
        .read_to_end(&mut raw)
        .unwrap();
    assert_eq!(raw.len(), *size);
    assert_eq!(format!("{:X}", Sha256::digest(&raw)), *sha);
    raw
}
fn reply(name: &str) -> combat::Reply {
    match app::decode_response_version(&bytes(name), 27)
        .unwrap_or_else(|e| panic!("{name}: {e:?}"))
        .response
    {
        app::Response::Combat(r) => r,
        _ => panic!("wrong Java fixture kind"),
    }
}
fn frame(name: &str) -> combat::Frame {
    reply(name).frame.unwrap()
}
fn assert_chain(names: &[&str]) {
    let mut previous = None;
    for name in names {
        let current = reply(name);
        assert!(
            combat::follows(previous.as_ref(), &current),
            "transition to {name}"
        );
        previous = Some(current);
    }
}
#[test]
fn all_java_full_application_goldens_decode_with_exact_scoped_actor_ledgers() {
    assert_eq!(golden::GOLDENS.len(), 27);
    for (name, ..) in golden::GOLDENS {
        let r = reply(name);
        r.validate().unwrap();
        let f = r.frame.unwrap();
        assert_eq!(f.disintegrations.len(), f.world.actors.len());
        assert_eq!(
            f.world.actors[0].identity().character().unwrap().source_id,
            7
        );
        assert_eq!(
            f.world.actors[1].identity().character().unwrap().source_id,
            42
        );
        assert_eq!(
            f.world.actors[0].identity().character().unwrap().object_id,
            3333
        );
        assert_eq!(
            f.world.actors[1].identity().character().unwrap().object_id,
            3333
        );
        assert!(f
            .world
            .descriptor
            .unwrap()
            .has(MatchDescriptor::DISINTEGRATE));
    }
}
#[test]
fn normal_critical_denial_and_coalesced_double_cast_follow_without_replaying_damage() {
    for prefix in ["normal", "critical"] {
        let names: Vec<String> = ["loading", "active", "hit", "denied", "second"]
            .iter()
            .map(|s| format!("{prefix}-{s}"))
            .collect();
        assert_chain(&names.iter().map(String::as_str).collect::<Vec<_>>());
        let active = reply(&names[1]);
        let second = reply(&names[4]);
        assert!(combat::follows(Some(&active), &second));
        let hit = frame(&names[2]);
        let denied = frame(&names[3]);
        assert_eq!(hit.disintegrations, denied.disintegrations);
        assert_eq!(
            hit.disintegrations[0].hit.as_ref().unwrap().effect(),
            if prefix == "critical" { 11748 } else { 11746 }
        );
        assert_eq!(hit.disintegrations[2], State::initial(3, 0, 0));
        assert_eq!(hit.disintegrations[3], State::initial(4, 0, 0));
        let last = second.frame.unwrap();
        assert_eq!(last.disintegrations[0].count, 2);
        assert_eq!(last.disintegrations[0].lawful, 502);
        assert_eq!(
            last.vitals[0].mp,
            last.vitals[0].max_mp - last.disintegrations[0].mp_spent
        );
    }
    let normal = frame("normal-hit");
    let critical = frame("critical-hit");
    assert_eq!(
        critical.disintegrations[0].hit.as_ref().unwrap().raw_damage,
        2 * normal.disintegrations[0].hit.as_ref().unwrap().raw_damage
    );
}
#[test]
fn honor_respawn_and_old_life_denial_preserve_history_until_new_life_cast() {
    assert_chain(&[
        "honor-active",
        "honor-death",
        "honor-respawn",
        "honor-stale",
        "honor-recast",
    ]);
    assert!(combat::follows(
        Some(&reply("honor-active")),
        &reply("honor-recast")
    ));
    let old = frame("honor-death");
    let respawn = frame("honor-respawn");
    let recast = frame("honor-recast");
    assert_eq!(old.disintegrations, respawn.disintegrations);
    assert_eq!(respawn.vitals[1].hp, 1);
    assert_eq!(
        recast.disintegrations[0].hit.as_ref().unwrap().target_life,
        2
    );
    assert_eq!(recast.regen[1].damage_loss, 2);
}
#[test]
fn fire_and_disintegrate_coalescing_share_one_loss_and_cast_sequence_budget() {
    assert_chain(&[
        "mixed-active",
        "mixed-dis",
        "mixed-fire",
        "mixed-second",
        "mixed-heal",
    ]);
    for a in ["mixed-active", "mixed-dis"] {
        assert!(combat::follows(Some(&reply(a)), &reply("mixed-second")));
    }
    let current = frame("mixed-second");
    assert_eq!(current.disintegrations[0].count, 2);
    assert_eq!(current.fires[0].count, 1);
    assert_eq!(
        current.vitals[0].mp,
        1000 - current.disintegrations[0].mp_spent - current.fires[0].mp_spent
    );
    let mut fake = current.clone();
    fake.world.casts[0].request_sequence = 2;
    assert!(fake.validate().is_err());
    // Both receipts individually fit the ledger, but cannot consume the same
    // increase. Keep the latest hit exact and alter the older Fire checkpoint.
    let mut fake = current.clone();
    let h = fake.disintegrations[0].hit.as_ref().unwrap().clone();
    let t = fake.fires[0]
        .targets
        .iter_mut()
        .find(|t| t.target_entity_id == 2)
        .unwrap();
    t.damage_loss_after = h.damage_loss_after;
    t.hp_after = h.hp_after;
    t.hp_before = t.hp_after + t.applied_damage;
    assert!(
        fake.validate().is_ok(),
        "isolated historical frame remains arithmetically legal"
    );
    assert!(
        !follows(&frame("mixed-active"), &fake),
        "joint receipt budget must reject reuse"
    );
}
#[test]
fn terminal_completion_keeps_i64_mp_one_result_and_immutable_receipts() {
    assert_chain(&["terminal-active", "terminal-finished"]);
    let finished = reply("terminal-finished");
    assert!(combat::follows(Some(&finished), &finished));
    let mut frame = finished.frame.unwrap();
    assert!(frame.vitals[0].mp > u32::MAX as i64);
    assert_eq!(frame.vitals[1].hp, 0);
    assert_eq!(frame.world.phase, Phase::Finished);
    let old = frame.clone();
    frame.disintegrations[0].hit.as_mut().unwrap().critical =
        !frame.disintegrations[0].hit.as_ref().unwrap().critical;
    assert!(!follows(&old, &frame));
}
#[test]
fn full_payload_prefixes_padding_unknown_life_counts_and_legacy_version_fail_closed() {
    let original = bytes("normal-hit");
    let appendix = original.len() - 8 - 4 * STATE_BYTES;
    for n in 0..original.len() {
        assert!(
            app::decode_response_version(&original[..n], 27).is_err(),
            "prefix {n}"
        );
    }
    for offset in [0, 2, 7, 8 + 144, 8 + 145, 8 + 151, 8 + STATE_BYTES + 64] {
        let mut bad = original.clone();
        bad[appendix + offset] = 2;
        assert!(
            app::decode_response_version(&bad, 27).is_err(),
            "offset {offset}"
        );
    }
    let mut bad = original.clone();
    bad[appendix + 8 + 80..appendix + 8 + 88].copy_from_slice(&2i64.to_be_bytes());
    assert!(app::decode_response_version(&bad, 27).is_err());
    let mut stripped = original[..appendix].to_vec();
    let size = stripped.len() as u32;
    stripped[4..8].copy_from_slice(&size.to_be_bytes());
    assert!(
        app::decode_response_version(&stripped, 26).is_err(),
        "descriptor survives stripping and must reject"
    );
    let mut trailing = original;
    trailing.push(0);
    let size = trailing.len() as u32;
    trailing[4..8].copy_from_slice(&size.to_be_bytes());
    assert!(app::decode_response_version(&trailing, 27).is_err());
}
#[test]
fn historical_receipt_cannot_change_without_count_or_resurrect_an_old_target_life() {
    let a = frame("normal-hit");
    let mut b = frame("normal-denied");
    b.disintegrations[0].hit.as_mut().unwrap().critical = true;
    assert!(b.validate().is_ok());
    assert!(!follows(&a, &b));
    let a = frame("honor-respawn");
    let mut b = frame("honor-recast");
    b.disintegrations[0].hit.as_mut().unwrap().target_life = 1;
    assert!(b.validate().is_err());
    assert!(!follows(&a, &b));
    let a = frame("honor-death");
    let mut b = frame("honor-recast");
    b.disintegrations[0].hit.as_mut().unwrap().target_life = 1;
    assert!(!follows(&a, &b));
}
#[test]
fn every_v27_capability_dependency_and_negotiated_descriptor_are_mandatory() {
    assert_eq!(session::MAX_VERSION, 47);
    assert_eq!(session::RUNTIME_WORLD_VERSION, 37);
    assert_eq!(session::RUNTIME_CAPABILITIES & session::CAP_DISINTEGRATE, 0);
    let mut hello = session::Hello::current(1);
    hello.max_version = 27;
    hello.supported_capabilities = session::KNOWN_CAPABILITIES
        & !(session::CAP_SLOW_DATA
            | session::CAP_HASTE
            | session::CAP_GREATER_HASTE
            | session::CAP_ACTION_LIFE
            | session::CAP_NPC_DISINTEGRATE
            | session::CAP_NPC_FIRE_STORM
            | session::CAP_CHAT
            | session::CAP_DIRECT_CHAT
            | session::CAP_FILTERED_CHAT
            | session::CAP_MATCHMAKING
            | session::CAP_LARGE_MAPS
            | session::CAP_BATTLE_DATA);
    assert!(session::encode(session::Message::Hello(hello)).is_ok());
    hello.max_version = 26;
    assert!(session::encode(session::Message::Hello(hello)).is_err());
    hello.max_version = 27;
    for bit in [
        session::CAP_REGEN,
        session::CAP_MATCH_DESCRIPTOR,
        session::CAP_COMBAT,
    ] {
        let mut bad = hello;
        bad.supported_capabilities &= !bit;
        assert!(session::encode(session::Message::Hello(bad)).is_err());
    }
    let response = app::Response::Combat(reply("normal-active"));
    let mut ack = session::Ack {
        request_id: 1,
        version: 27,
        numeric_schema: 1,
        capabilities: session::KNOWN_CAPABILITIES,
    };
    assert!(world::descriptor_negotiated(&response, &ack));
    ack.version = 26;
    assert!(!world::descriptor_negotiated(&response, &ack));
    assert!(session::encode(session::Message::Ack(ack)).is_err());
    ack.version = 27;
    ack.capabilities &= !session::CAP_DISINTEGRATE;
    assert!(!world::descriptor_negotiated(&response, &ack));
    ack.capabilities = session::KNOWN_CAPABILITIES & !session::CAP_MATCH_DESCRIPTOR;
    assert!(!world::descriptor_negotiated(&response, &ack));
}
#[test]
fn fixed_width_independent_vector_and_i64_lawful_overflow_do_not_wrap() {
    let mut raw = vec![0u8; 160];
    raw[..2].copy_from_slice(&1u16.to_be_bytes());
    for (at, n) in [
        (8, 1i64),
        (16, 1),
        (24, 9),
        (32, 70),
        (40, 3),
        (48, 500),
        (56, 501),
        (72, 2),
        (80, 1),
        (88, 1),
        (112, 1000),
        (120, 800),
        (128, 200),
        (136, 200),
        (144, 200),
    ] {
        raw[at..at + 8].copy_from_slice(&n.to_be_bytes());
    }
    for (at, n) in [
        (64, 70u32),
        (68, 5000),
        (96, 10),
        (100, 10),
        (104, 11),
        (108, 10),
    ] {
        raw[at..at + 4].copy_from_slice(&n.to_be_bytes());
    }
    let (states, used) = decode(&raw, 1).unwrap();
    assert_eq!(used, 160);
    assert_eq!(states[0].lawful, 501);
    assert_eq!(states[0].hit.as_ref().unwrap().effect(), 11746);
    raw[152] = 1;
    assert_eq!(
        decode(&raw, 1).unwrap().0[0].hit.as_ref().unwrap().effect(),
        11748
    );
    for n in 0..160 {
        assert!(decode(&raw[..n], 1).is_err());
    }
    raw[48..56].copy_from_slice(&i64::MAX.to_be_bytes());
    raw[56..64].copy_from_slice(&i64::MIN.to_be_bytes());
    assert!(decode(&raw, 1).is_err());
    assert!(State::initial(1, 70, i64::MIN).validate().is_ok());
    assert!(State::initial(1, 0, 500).validate().is_err());
}

#[test]
fn an_old_typed_cast_denial_cannot_acknowledge_a_disintegrate_commit() {
    use super::super::spell_protocol as spell;
    let old = reply("normal-active");
    let next = frame("normal-hit");
    let echo = spell::CastRequest {
        context: old.echo.context,
        channel_id: old.echo.channel_id,
        match_id: old.echo.match_id,
        generation: old.echo.generation,
        sequence: 1,
        skill_id: spell::TELEPORT,
        target_entity_id: 0,
    };
    let denial = spell::CastReply {
        echo,
        status: spell::Status::Denied,
        reason: spell::Reason::TooFast,
        frame: Some(next),
    };
    assert!(
        denial.validate().is_ok(),
        "a frame in isolation is not a request transition proof"
    );
    assert!(!spell::confirms_transition(
        old.frame.as_ref().unwrap(),
        &denial
    ));
}

#[test]
fn recovery_then_recast_uses_new_mp_anchor_without_rewriting_old_damage() {
    assert_chain(&[
        "regen-active",
        "regen-hit",
        "regen-hp",
        "regen-mp",
        "regen-recast",
    ]);
    let hit = frame("regen-hit");
    let hp = frame("regen-hp");
    let mp = frame("regen-mp");
    let next = frame("regen-recast");
    assert!(hp.regen[1].hp_gain > 0);
    assert!(mp.regen[0].mp_gain > 0);
    assert_eq!(hit.disintegrations, mp.disintegrations);
    assert_eq!(
        next.vitals[0].mp,
        next.vitals[0].max_mp + next.regen[0].mp_gain
            - next.regen[0].mp_loss
            - next.disintegrations[0].mp_spent
    );
    assert!(combat::follows(
        Some(&reply("regen-hit")),
        &reply("regen-recast")
    ));
    let mut bad = next.clone();
    bad.vitals[0].mp = mp.vitals[0].mp;
    assert!(bad.validate().is_err());
    let mut bad = next;
    bad.disintegrations[0].revision = bad.regen[0].mp_revision;
    assert!(bad.validate().is_err());
}
