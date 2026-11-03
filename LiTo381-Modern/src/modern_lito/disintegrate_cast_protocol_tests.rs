use super::*;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
#[path = "disintegrate_command_golden.rs"]
mod command;
#[path = "disintegrate_wire_golden.rs"]
mod state;
fn raw(name: &str, goldens: &[(&str, usize, &str, &str)]) -> Vec<u8> {
    let (_, size, sha, encoded) = goldens.iter().find(|(n, ..)| *n == name).unwrap();
    let zip = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .unwrap();
    let mut raw = Vec::new();
    flate2::read::GzDecoder::new(&zip[..])
        .take(*size as u64 + 1)
        .read_to_end(&mut raw)
        .unwrap();
    assert_eq!(raw.len(), *size);
    assert_eq!(format!("{:X}", Sha256::digest(&raw)), *sha);
    raw
}
fn reply(name: &str) -> Reply {
    match app::decode_response_version(&raw(name, command::GOLDENS), 27)
        .unwrap()
        .response
    {
        app::Response::Disintegrate(r) => r,
        _ => panic!("wrong Java kind"),
    }
}
fn before(name: &str) -> combat::Reply {
    match app::decode_response_version(&raw(name, state::GOLDENS), 27)
        .unwrap()
        .response
    {
        app::Response::Combat(r) => r,
        _ => panic!("wrong Java state kind"),
    }
}
#[test]
fn exact_request_golden_and_all_integer_bounds() {
    let mut r = Request {
        context: SelectionContext {
            key: app::CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        },
        channel_id: 3,
        match_id: 9,
        generation: 6,
        sequence: 1,
        caster_life: 2,
        target_entity_id: 4,
        target_life: 3,
    };
    let expected="4c544d3100000058fedcba98002c0000000000070000000000000d05ffffffffffffffff03000000000000000000000900000000000000060000000000000001000000000000000200000000000000040000000000000003";
    let hex: String = encode_request(0xfedcba98, &r, 27)
        .unwrap()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(hex, expected);
    for v in 1..27 {
        assert!(encode_request(1, &r, v).is_err());
    }
    for value in [0, u64::MAX, i64::MAX as u64 + 1] {
        for field in 0..6 {
            let mut bad = r;
            match field {
                0 => bad.match_id = value,
                1 => bad.generation = value,
                2 => bad.sequence = value,
                3 => bad.caster_life = value,
                4 => bad.target_entity_id = value,
                _ => bad.target_life = value,
            };
            assert!(bad.validate().is_err());
        }
    }
    r.channel_id = 4;
    assert!(r.validate().is_err());
}
#[test]
fn java_typed_goldens_accept_only_exact_real_engine_transitions() {
    assert_eq!(command::GOLDENS.len(), 10);
    for (old, name) in [
        ("normal-active", "normal-hit"),
        ("normal-hit", "normal-denied"),
        ("critical-active", "critical-hit"),
        ("critical-hit", "critical-denied"),
        ("honor-active", "honor-death"),
        ("honor-respawn", "honor-stale"),
        ("honor-stale", "honor-recast"),
        ("terminal-active", "terminal"),
        ("terminal-finished", "not-active"),
    ] {
        let prior = before(old);
        let reply = reply(name);
        reply.validate().unwrap();
        assert!(reply.confirms(&reply.echo));
        assert!(
            confirms_transition(prior.frame.as_ref().unwrap(), &reply),
            "{old} -> {name}"
        );
        assert!(
            combat::follows(Some(&prior), &reply.combat_reply()),
            "{old} -> {name}"
        );
        let mut wrong = reply.echo;
        wrong.context.key.source_id = 42;
        assert!(!reply.confirms(&wrong));
    }
    assert!(reply("unavailable").frame.is_none());
}
#[test]
fn ready_checks_current_scope_incarnation_and_shared_sequence_not_local_damage() {
    let active = before("honor-active").frame.unwrap();
    let request = reply("honor-death").echo;
    assert!(request.ready_in(&active));
    let revived = before("honor-respawn").frame.unwrap();
    let stale = reply("honor-stale").echo;
    assert!(!stale.ready_in(&revived));
    let mut corrected = stale;
    corrected.target_life = 2;
    assert!(corrected.ready_in(&revived));
    for field in 0..7 {
        let mut bad = request;
        match field {
            0 => bad.context.key.source_id = 99,
            1 => bad.match_id += 1,
            2 => bad.generation += 1,
            3 => bad.caster_life += 1,
            4 => bad.target_life += 1,
            5 => bad.sequence += 1,
            _ => bad.target_entity_id = 1,
        };
        assert!(!bad.ready_in(&active));
    }
    assert!(!request.ready_in(&before("normal-loading").frame.unwrap()));
    assert!(!request.ready_in(&reply("terminal").frame.unwrap()));
}
#[test]
fn forged_echo_historical_receipt_null_success_and_old_kind_are_rejected() {
    let raw = raw("normal-hit", command::GOLDENS);
    for at in [40, 48, 56, 64, 72, 80] {
        let mut bad = raw.clone();
        bad[at..at + 8].copy_from_slice(&99u64.to_be_bytes());
        assert!(app::decode_response_version(&bad, 27).is_err());
    }
    let mut alias = raw.clone();
    alias[16..20].copy_from_slice(&42u32.to_be_bytes());
    assert!(app::decode_response_version(&alias, 27).is_err());
    let mut kind = raw.clone();
    kind[12..14].copy_from_slice(&27u16.to_be_bytes());
    assert!(app::decode_response_version(&kind, 27).is_err());
    let mut history = reply("normal-denied");
    history.frame.as_mut().unwrap().disintegrations[0]
        .hit
        .as_mut()
        .unwrap()
        .critical = true;
    history.validate().unwrap();
    assert!(!confirms_transition(
        before("normal-hit").frame.as_ref().unwrap(),
        &history
    ));
    let mut success = reply("unavailable");
    success.status = Status::Ok;
    success.reason = Reason::None;
    assert!(success.validate().is_err());
    let mut busy = reply("normal-denied");
    busy.status = Status::Busy;
    assert!(busy.validate().is_err());
    busy.status = Status::Denied;
    busy.reason = Reason::NotActive;
    assert!(busy.validate().is_err());
}
#[test]
fn header_version_size_padding_and_truncation_reject_before_body_admission() {
    let original = raw("normal-hit", command::GOLDENS);
    for n in 0..original.len() {
        assert!(
            app::decode_response_version(&original[..n], 27).is_err(),
            "prefix {n}"
        );
    }
    for at in [37, 38, 39, 88, 89, 90, 91] {
        let mut bad = original.clone();
        bad[at] = 255;
        assert!(app::decode_response_version(&bad, 27).is_err());
    }
    for v in 1..27 {
        let mut cursor = Cursor::new(&original);
        assert!(app::read_response_version(&mut cursor, v).is_err());
        assert_eq!(cursor.position(), 16);
    }
    for size in [91, 1024 * 1024 + 1, u32::MAX] {
        let mut header = original[..16].to_vec();
        header[4..8].copy_from_slice(&size.to_be_bytes());
        let mut c = Cursor::new(header);
        assert!(app::read_response_version(&mut c, 27).is_err());
        assert_eq!(c.position(), 16);
    }
    let mut trailing = original;
    trailing.push(0);
    let n = trailing.len() as u32;
    trailing[4..8].copy_from_slice(&n.to_be_bytes());
    assert!(app::decode_response_version(&trailing, 27).is_err());
}
#[test]
fn typed_null_result_requires_v27_and_both_capabilities() {
    let response = app::Response::Disintegrate(reply("unavailable"));
    let mut ack = protocol::Ack {
        request_id: 1,
        version: 27,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    };
    assert!(world::descriptor_negotiated(&response, &ack));
    for bit in [protocol::CAP_DISINTEGRATE, protocol::CAP_MATCH_DESCRIPTOR] {
        let mut bad = ack;
        bad.capabilities &= !bit;
        assert!(!world::descriptor_negotiated(&response, &bad));
    }
    ack.version = 26;
    assert!(!world::descriptor_negotiated(&response, &ack));
}
