use super::*;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::io::Read;
#[path = "siege_wire_golden.rs"]
mod golden;

pub(crate) fn bytes(name: &str) -> Vec<u8> {
    let (_, size, sha, encoded) = golden::GOLDENS
        .iter()
        .find(|(n, _, _, _)| *n == name)
        .unwrap();
    let packed = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .unwrap();
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(&packed[..])
        .take((*size + 1) as u64)
        .read_to_end(&mut out)
        .unwrap();
    assert_eq!(out.len(), *size);
    assert_eq!(format!("{:X}", Sha256::digest(&out)), *sha);
    assert_eq!(
        u32::from_be_bytes(out[4..8].try_into().unwrap()) as usize,
        out.len()
    );
    out
}
pub(crate) fn sample(name: &str) -> combat::Reply {
    let b = bytes(name);
    match u16::from_be_bytes(b[12..14].try_into().unwrap()) {
        23 => combat::decode_body_version(&b[16..], VERSION).unwrap(),
        43 => decode_body(&b[16..], VERSION).unwrap().combat_reply(),
        _ => panic!("wrong oracle kind"),
    }
}
fn data_bytes(name: &str) -> Vec<u8> {
    let b = bytes(name);
    let r = sample(name);
    let d = r.frame.unwrap().world.siege.unwrap();
    let prefix = [
        1,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        1,
        d.phase as u8,
        d.attacker,
        d.winner.map_or(0, |w| w + 1),
        u8::from(d.receipt.is_some()),
        d.gate.len() as u8,
        d.attack_camp.len() as u8,
        d.defense_camp.len() as u8,
        d.lives.len() as u8,
        0,
        0,
    ];
    let offset = b.windows(prefix.len()).position(|v| v == prefix).unwrap();
    let (decoded, used) = decode(&b[offset..]).unwrap();
    assert_eq!(decoded, Some(d));
    b[offset..offset + used].to_vec()
}

#[test]
fn dedicated_siege_v26_through_v28_preserve_portal_tower_and_null_replies_without_granting_disintegrate(
) {
    let mut checked = 0;
    for (name, ..) in golden::GOLDENS {
        let old = bytes(name);
        if u16::from_be_bytes(old[12..14].try_into().unwrap()) != RESULT_KIND {
            continue;
        }
        let expected = decode_body(&old[16..], 26).unwrap();
        assert!(expected.frame.as_ref().unwrap().disintegrations.is_empty());
        // Java's oracle independently asserts these exact appended bytes for
        // every dedicated tower/portal result, not an inferred new wire shape.
        let mut current = old.clone();
        current.extend([0; 8]);
        let len = current.len() as u32;
        current[4..8].copy_from_slice(&len.to_be_bytes());
        assert_eq!(decode_body(&current[16..], 27).unwrap(), expected, "{name}");
        assert_eq!(decode_body(&current[16..], 28).unwrap(), expected, "{name}");
        let app::Response::Siege(parsed) =
            app::decode_response_version(&current, 27).unwrap().response
        else {
            panic!("siege")
        };
        assert_eq!(parsed, expected);
        assert!(
            decode_body(&old[16..], 27).is_err(),
            "v27 still requires its empty appendix"
        );
        assert!(
            decode_body(&current[16..], 26).is_err(),
            "v26 still rejects trailing bytes"
        );
        for version in [0, 1, 25, 29, u16::MAX] {
            assert!(decode_body(&current[16..], version).is_err());
        }
        let mut unavailable = old[..76].to_vec();
        unavailable[4..8].copy_from_slice(&76u32.to_be_bytes());
        unavailable[72..76].copy_from_slice(&[3, 1, 0, 0]);
        let none = decode_body(&unavailable[16..], 26).unwrap();
        assert!(none.frame.is_none());
        assert_eq!(decode_body(&unavailable[16..], 27).unwrap(), none);
        assert_eq!(decode_body(&unavailable[16..], 28).unwrap(), none);
        let mut forged = current;
        let last = forged.len() - 1;
        forged[last] = 1;
        assert!(
            decode_body(&forged[16..], 27).is_err(),
            "nonzero appendix reserved byte"
        );
        checked += 1;
    }
    assert!(checked >= 4);
}

#[test]
fn java_real_engine_frames_are_strict_at_full_signed_width() {
    for (name, _, _, _) in golden::GOLDENS {
        if matches!(*name, "preparing" | "preabort") {
            continue;
        }
        let r = sample(name);
        r.validate().unwrap();
        r.frame.as_ref().unwrap().validate().unwrap();
    }
    for name in ["wide0", "wide1"] {
        let f = sample(name).frame.unwrap();
        let d = f.world.siege.unwrap();
        assert_eq!(f.vitals.len(), 100);
        assert_eq!(d.lives.len(), 100);
        assert!(d
            .objectives
            .iter()
            .all(|t| t.max_hp == i64::MAX && t.hp == i64::MAX));
    }
    let finished = sample("inner").frame.unwrap();
    assert!(finished.world.completion.is_none());
    assert_eq!(
        finished
            .world
            .siege
            .unwrap()
            .receipt
            .unwrap()
            .recorded_at_millis,
        9_007_199_254_740_993
    );
}

#[test]
fn ordered_towers_timeout_abort_and_only_one_terminal_receipt_update_follow() {
    for names in [
        vec![
            "loading",
            "active",
            "left",
            "beforeright",
            "right",
            "beforeinner",
            "inner",
            "saved",
        ],
        vec![
            "loading",
            "active",
            "left",
            "beforeright",
            "right",
            "beforeinner",
            "inner",
            "failed",
        ],
        vec!["wide0", "timeout0"],
        vec!["wide1", "timeout1"],
        vec!["beforeabort", "aborted"],
        vec!["countdown", "countdownactive"],
    ] {
        let values: Vec<_> = names.iter().map(|n| sample(n)).collect();
        for (i, pair) in values.windows(2).enumerate() {
            assert!(
                combat::follows(Some(&pair[0]), &pair[1]),
                "{} -> {}",
                names[i],
                names[i + 1]
            );
            assert!(
                !combat::follows(Some(&pair[1]), &pair[0]),
                "rewind {}",
                names[i]
            );
        }
        assert!(
            combat::follows(Some(&values[0]), values.last().unwrap()),
            "coalesced {:?}",
            names
        );
    }
    assert!(!combat::follows(Some(&sample("saved")), &sample("failed")));
    let mut changed = sample("saved");
    changed.frame.as_mut().unwrap().world.revision += 1;
    assert!(!combat::follows(Some(&sample("saved")), &changed));
    for name in ["timeout0", "timeout1"] {
        let d = sample(name).frame.unwrap().world.siege.unwrap();
        assert_eq!(d.winner, Some(1 - d.attacker));
        assert_eq!(d.finished_elapsed_millis, d.duration_millis);
    }
}

#[test]
fn server_respawn_preserves_sequences_and_works_with_or_without_regen() {
    for suffix in ["plain", "regen"] {
        let names = [
            "lifeactive",
            "beforeportal",
            "afterportal",
            "beforedeath",
            "dead",
            "respawn",
            "afterrespawnmove",
        ];
        let chain: Vec<_> = names
            .iter()
            .map(|n| sample(&format!("{n}{suffix}")))
            .collect();
        for (i, p) in chain.windows(2).enumerate() {
            assert!(combat::follows(Some(&p[0]), &p[1]), "{} {suffix}", names[i]);
        }
        assert!(
            combat::follows(Some(&chain[0]), chain.last().unwrap()),
            "coalesced respawn {suffix}"
        );
        let dead = chain[4].frame.as_ref().unwrap();
        let alive = chain[5].frame.as_ref().unwrap();
        assert_eq!(dead.vitals[1].hp, 0);
        assert_eq!(alive.vitals[1].hp, 1);
        assert_eq!(
            dead.world.actors[1].sequence,
            alive.world.actors[1].sequence
        );
        assert_ne!(
            dead.world.actors[1].position,
            alive.world.actors[1].position
        );
        assert_eq!(dead.vitals[1].mp, alive.vitals[1].mp);
        assert_eq!(super::super::quest_life::respawn_gain(alive, 2), 1);
        assert_eq!(
            super::super::quest_life::respawn_revision(alive, 2),
            life(&alive.world, 2).unwrap().respawn_revision + 1
        );
        let mut forged = chain[5].clone();
        forged.frame.as_mut().unwrap().world.actors[1].position = dead.world.actors[1].position;
        assert!(
            !combat::follows(Some(&chain[4]), &forged),
            "noncamp respawn"
        );
        let mut forged = chain[5].clone();
        forged
            .frame
            .as_mut()
            .unwrap()
            .world
            .siege
            .as_mut()
            .unwrap()
            .lives[1]
            .hp_gain += 1;
        assert!(!combat::follows(Some(&chain[4]), &forged), "invented HP");
    }
}

#[test]
fn dedicated_tower_and_portal_confirm_exact_action_scope_sequence_and_endpoint() {
    for (before, after) in [
        ("active", "left"),
        ("beforeright", "right"),
        ("beforeinner", "inner"),
        ("beforeportalplain", "portalplain"),
        ("beforeportalregen", "portalregen"),
    ] {
        let bytes = bytes(after);
        let reply = decode_body(&bytes[16..], VERSION).unwrap();
        let mut prior = sample(before);
        prior.echo.context = reply.echo.context;
        assert!(reply.confirms(&reply.echo, &prior), "{after}");
        let encoded = encode_request(72, &reply.echo).unwrap();
        assert_eq!(&encoded[16..], &bytes[16..72]);
        for change in 0..5 {
            let mut request = reply.echo;
            match change {
                0 => request.context.key.source_id += 1,
                1 => request.context.epoch += 1,
                2 => request.scope.room_id += 1,
                3 => request.scope.generation += 1,
                _ => request.sequence += 1,
            }
            assert!(!reply.confirms(&request, &prior));
        }
        assert!(decode_body(&bytes[16..], 25).is_err());
        assert!(decode_body(&bytes[16..], 27).is_err());
        for i in [21, 22, 23, 56, 57, 58, 59] {
            let mut bad = bytes[16..].to_vec();
            bad[i] = 254;
            assert!(decode_body(&bad, VERSION).is_err(), "header {i}");
        }
        for n in [0, 55, 59, 60, 100, bytes.len() - 17] {
            assert!(
                decode_body(&bytes[16..16 + n], VERSION).is_err(),
                "truncated {n}"
            );
        }
        let mut trailing = bytes[16..].to_vec();
        trailing.push(0);
        assert!(decode_body(&trailing, VERSION).is_err());
    }
}

#[test]
fn appendix_all_truncations_padding_counts_and_signed_limits_fail_closed() {
    let b = data_bytes("inner");
    for n in 0..b.len() {
        assert!(decode(&b[..n]).is_err(), "appendix prefix {n}");
    }
    for offset in [0, 1, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19] {
        let mut bad = b.clone();
        bad[offset] = 255;
        assert!(decode(&bad).is_err(), "appendix header {offset}");
    }
    let mut bad = b.clone();
    let last = bad.len() - 1;
    bad[last] = 1;
    assert!(decode(&bad).is_err(), "receipt padding");
    // First tower max HP cannot cross the signed-i64 boundary.
    let mut bad = b;
    bad[96..104].copy_from_slice(&u64::MAX.to_be_bytes());
    assert!(decode(&bad).is_err());
}

#[test]
fn policy_life_objective_and_receipt_tampering_never_advances_history() {
    let old = sample("active");
    let next = sample("right");
    for i in 0..7 {
        let mut altered = next.clone();
        let w = &mut altered.frame.as_mut().unwrap().world;
        let d = w.siege.as_mut().unwrap();
        match i {
            0 => d.attacker ^= 1,
            1 => d.attack_camp.swap(1, 2),
            2 => d.gate_revision -= 1,
            3 => d.objectives[0].last_actor = 2,
            4 => d.objectives[0].hp = 1,
            5 => d.lives[0].entity_id = 2,
            _ => d.respawn_millis += 1,
        }
        assert!(!combat::follows(Some(&old), &altered), "mutated {i}");
    }
    let old = sample("inner");
    for i in 0..4 {
        let mut altered = sample("saved");
        let r = altered
            .frame
            .as_mut()
            .unwrap()
            .world
            .siege
            .as_mut()
            .unwrap()
            .receipt
            .as_mut()
            .unwrap();
        match i {
            0 => r.id[0] ^= 1,
            1 => r.revision += 1,
            2 => r.recorded_at_millis += 1,
            _ => r.state = SettlementState::Pending,
        }
        assert!(!combat::follows(Some(&old), &altered), "receipt {i}");
    }
}

#[test]
fn siege_application_requires_v26_and_descriptor_even_for_null_denials() {
    assert!(super::super::protocol::MAX_VERSION >= 26);
    let bytes = bytes("left");
    let decoded = app::decode_response_version(&bytes, VERSION).unwrap();
    assert!(matches!(&decoded.response, app::Response::Siege(_)));
    assert!(app::decode_response_version(&bytes, 25).is_err());
    let mut ack = super::super::protocol::Ack {
        request_id: 1,
        version: 26,
        numeric_schema: 1,
        capabilities: super::super::protocol::RUNTIME_CAPABILITIES,
    };
    assert!(world::descriptor_negotiated(&decoded.response, &ack));
    ack.version = 25;
    assert!(!world::descriptor_negotiated(&decoded.response, &ack));
    ack.version = 26;
    ack.capabilities &= !super::super::protocol::CAP_MATCH_DESCRIPTOR;
    assert!(!world::descriptor_negotiated(&decoded.response, &ack));
    let app::Response::Siege(mut null) = decoded.response else {
        unreachable!()
    };
    null.frame = None;
    null.status = combat::Status::Unavailable;
    null.reason = Reason::NotActive;
    assert!(!world::descriptor_negotiated(
        &app::Response::Siege(null),
        &ack
    ));
    for n in 0..bytes.len() {
        assert!(app::decode_response_version(&bytes[..n], 26).is_err());
    }
}

#[test]
fn preparing_and_preengine_abort_have_only_world_data_and_preserve_exact_roles() {
    let preparing = world::decode_body_version(&bytes("preparing")[16..], VERSION).unwrap();
    let aborted = world::decode_body_version(&bytes("preabort")[16..], VERSION).unwrap();
    let loading = sample("preloading").world_reply().unwrap();
    assert!(world::follows(Some(&preparing), &aborted));
    assert!(world::follows(Some(&preparing), &loading));
    assert_eq!(preparing.frame.as_ref().unwrap().revision, 1);
    assert_eq!(aborted.frame.as_ref().unwrap().revision, 2);
    for r in [&preparing, &aborted] {
        let w = r.frame.as_ref().unwrap();
        assert!(w.actors.iter().all(|a| !a.loaded && a.sequence == 0));
        assert!(w
            .siege
            .as_ref()
            .unwrap()
            .lives
            .iter()
            .all(|l| l.deaths == 0));
    }
}

#[test]
fn coalesced_teleport_then_respawn_uses_latest_engine_relocation_not_old_cast_target() {
    let chain: Vec<_> = [
        "beforeteleportlife",
        "teleportlife",
        "teleportdead",
        "teleportrespawn",
    ]
    .iter()
    .map(|n| sample(n))
    .collect();
    for (i, p) in chain.windows(2).enumerate() {
        assert!(combat::follows(Some(&p[0]), &p[1]), "relocation {i}");
    }
    assert!(combat::follows(Some(&chain[0]), &chain[3]));
    let before = &chain[0].frame.as_ref().unwrap().world;
    let after = &chain[3].frame.as_ref().unwrap().world;
    assert_eq!(before.actors[1].sequence, after.actors[1].sequence);
    assert_ne!(after.actors[1].position, after.casts[1].to);
    assert!(respawn_is_latest_relocation(before, after, 2));
    let mut bad = chain[3].clone();
    let w = &mut bad.frame.as_mut().unwrap().world;
    w.actors[1].position = w.casts[1].to;
    assert!(!combat::follows(Some(&chain[0]), &bad));
}

#[test]
fn dedicated_request_never_aliases_actor_ids_or_contains_client_destination() {
    let context = SelectionContext {
        key: app::CharacterKey {
            source_id: 7,
            object_id: 3333,
        },
        epoch: 1,
    };
    let scope = Scope {
        room_id: 1,
        match_id: 2,
        generation: 3,
    };
    for action in [
        Action::AttackTower(Slot::OuterLeft),
        Action::AttackTower(Slot::OuterRight),
        Action::AttackTower(Slot::Inner),
        Action::Portal,
    ] {
        let q = Request {
            context,
            channel_id: 0,
            scope,
            sequence: 1,
            action,
        };
        let b = encode_request(71, &q).unwrap();
        assert_eq!(b.len(), 72);
        assert_eq!(&b[..4], b"LTM1");
        assert_eq!(&b[12..16], &[0, 42, 0, 0]);
        assert_eq!(b[39], 0);
        assert_eq!(&b[40..48], &1u64.to_be_bytes());
        assert_eq!(&b[64..72], &1u64.to_be_bytes());
        if action == Action::Portal {
            assert_eq!(&b[37..39], &[1, 255]);
        }
        for n in [0, i64::MAX as u64 + 1, u64::MAX] {
            let mut bad = q;
            bad.sequence = n;
            assert!(encode_request(1, &bad).is_err());
            let mut bad = q;
            bad.scope.room_id = n;
            assert!(encode_request(1, &bad).is_err());
        }
    }
}

#[test]
fn disabled_appendix_still_requires_all_reserved_bytes() {
    assert_eq!(decode(&[0; 8]).unwrap(), (None, 8));
    for n in 0..8 {
        assert!(decode(&vec![0; n]).is_err());
    }
    for i in 0..8 {
        let mut b = [0; 8];
        b[i] = 1;
        assert!(decode(&b).is_err());
    }
}
