//! Captured from ModernNpcProtocolTest's real engine fixture (not a Rust encoder).
use super::super::{
    application_protocol::{CharacterKey, Response},
    protocol as session,
};
use super::*;
use std::io::Cursor;
const GOLDEN: &str = "4c544d3100000601fedcba98002700000000000700000000000000010000000000000001000100000000000000000001000000000000000100000000000000060000000000000001000000000000000100000000000000010000010000000000000000010000000000000006000000000000000a0300000000000000000000010000000000000006006402000000022a0004726f6f6d0003000301000000000000070000000000000001000300000000000000000007000000000000000100000000000000500001000001000000000768756d616e2d3000087365727665722d37000000000000002a000000000000000100000000000000500001000101000000000768756d616e2d3100097365727665722d343201000000000000070000000000000001000000000000005000000000010000000004424f543100087365727665722d37000100010000022a84325f1245e4c25bd4e28eac3d50cab6e0252d9beaded46c1f0928a9ad15bd3a800080000000c012ef81ffbc2095f821ee3ae6e2d67fb8505b266ad2163bef49997d5e9cfce0a0a20003000000000000000000010000800a0000800a000100000000000000000000000000000bebc2000000000000000002000080140000800a000100000000000000000000000000000bebc20000000000000000030000801e0000800a000100000000000000000000000000000bebc200000000000000000001000000000000000001050000000801000007d0000007d000000000000000000000ea60000000000000000000000000000300000000000000000001001fffffffffffe5002000000000000100000000000000320000000000000032000000000000000a0100000000000000000000000000000100000000000000000000000000000008000000000000000000000000000000000000000029b9270000000000000000020020000000000001002000000000000100000000000000320000000000000032000000000000000a0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029b927000000000000000003000000000000000c000000000000002800000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000009000000000000000000000000000000000000000029b92700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000001000000000000000600000000000000070000000000000064000000000000000200000000000200030000000000000001000000580000b0ce00000671000106020000800b0000800a001ffffffffffff30020000000000001000000000000000a000000000000000a0000000000000001000000000000000200000000000000070000000000000000000000000000006400000000000003520000000000000002000000580000b0ce00000671000106040000801f0000800a00200000000000010020000000000001000000000000000a000000000000000a0000000000000001000000000000000300000000000000060000000000000000000000000000006400000000000003520001090000000007000000000000000100000000000000010000000000000001000000000000000100000000000000000000000000000008000000000000001c000000000000001c000000000000001c001fffffffffffe5000000000000000000000000000000000101090000000007000000000000000100000000000000010000000000000002000000000000000100000000000000000000000000000009000000000000001c000000000000001c000000000000001c000000000000000c00000000000000000000000000000000000001000000000700000000000000010000000000000001000000000000000100000000000000010000000000000001000000000000000a000000000000000e000000000000000e000000000000000e001ffffffffffff300000000000000000000000000000000";
const HUGE: i64 = 9_007_199_254_740_993;
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn bytes() -> Vec<u8> {
    unhex(GOLDEN)
}
fn resource_bytes() -> Vec<u8> {
    let mut packet = bytes();
    packet.extend(unhex("0001000200000000000006710000086e"));
    let length = packet.len() as u32;
    packet[4..8].copy_from_slice(&length.to_be_bytes());
    packet
}
fn resource_reply() -> Reply {
    match app::decode_response_version(&resource_bytes(), 18)
        .unwrap()
        .response
    {
        Response::Npc(reply) => reply,
        _ => panic!("wrong NPC kind"),
    }
}

pub(crate) fn isolated_guest() -> Reply {
    let mut r = resource_reply();
    r.echo.action = Action::Read;
    r.echo.sequence = 0;
    r.echo.serial = 0;
    r.echo.life = 0;
    r.echo.context.key.source_id = 42;
    let f = r.frame.as_mut().unwrap();
    f.combat.world.descriptor.as_mut().unwrap().features |= world::MatchDescriptor::ISOLATED_FORCES;
    f.combat.world.audience_lane = Some(1);
    f.combat.world.actors.retain(|a| a.entity_id == 2);
    f.combat.vitals.retain(|v| v.entity_id == 2);
    f.actors.retain(|a| a.lane == 1);
    f.exchanges.clear();
    r
}

#[test]
fn isolated_guest_keeps_global_identity_and_wide_health_without_opponent_data() {
    let r = isolated_guest();
    r.validate().unwrap();
    let f = r.frame.as_ref().unwrap();
    assert_eq!(f.combat.world.room.members.len(), 3);
    assert_eq!(f.combat.world.actors.len(), 1);
    assert_eq!(f.combat.world.actor(2).unwrap().key, r.echo.context.key);
    assert_eq!(f.combat.world.member(2).unwrap().team, 1);
    assert!(f.combat.world.actor(1).is_none());
    assert_eq!(f.combat.vital(2).unwrap().max_hp, HUGE);
    assert!(f.combat.vital(1).is_none());
    for case in 0..7 {
        let mut bad = r.clone();
        let f = bad.frame.as_mut().unwrap();
        match case {
            0 => f.combat.world.audience_lane = None,
            1 => f.combat.world.audience_lane = Some(0),
            2 => f.combat.world.actors[0].entity_id = 1,
            3 => f.combat.world.actors.clear(),
            4 => f.combat.vitals[0].entity_id = 1,
            5 => f
                .actors
                .push(resource_reply().frame.unwrap().actors[0].clone()),
            _ => bad.echo.context.key.source_id = 7,
        }
        assert!(bad.validate().is_err(), "isolated case {case}");
    }
}

#[test]
fn v20_sparse_world_wire_requires_explicit_correct_audience_and_rejects_downgrade() {
    let r = isolated_guest();
    let reply = world::Reply {
        echo: world::Request::read(r.echo.context, r.echo.channel_id),
        status: world::Status::Ok,
        reason: world::Reason::None,
        frame: Some(r.frame.as_ref().unwrap().combat.world.clone()),
    };
    let bytes = world::tests::wire_complete_version(&reply, 20);
    assert_eq!(
        app::decode_response_version(&bytes, 20).unwrap().response,
        Response::World(reply.clone())
    );
    assert!(app::decode_response_version(&bytes, 19).is_err());
    for code in [0, 1, 3, 255] {
        let mut bad = bytes.clone();
        let n = bad.len();
        bad[n - 4] = code;
        assert!(app::decode_response_version(&bad, 20).is_err());
    }
    for tail in 1..=3 {
        let mut bad = bytes.clone();
        let n = bad.len();
        bad[n - tail] = 1;
        assert!(app::decode_response_version(&bad, 20).is_err());
    }
    let mut legacy = bytes[..bytes.len() - 4].to_vec();
    let n = legacy.len() as u32;
    legacy[4..8].copy_from_slice(&n.to_be_bytes());
    assert!(app::decode_response_version(&legacy, 19).is_err());
    assert!(app::decode_response_version(&legacy, 20).is_err());
    let mut ack = session::Ack {
        request_id: 1,
        version: 20,
        numeric_schema: 1,
        capabilities: session::KNOWN_CAPABILITIES,
    };
    assert!(world::descriptor_negotiated(
        &Response::World(reply.clone()),
        &ack
    ));
    ack.version = 19;
    assert!(!world::descriptor_negotiated(&Response::World(reply), &ack));
}
#[test]
fn v23_npc_teleport_preserves_sparse_cast_identity_and_rejects_v22() {
    let r = isolated_guest();
    let mut frame = r.frame.unwrap().combat.world;
    frame.descriptor.as_mut().unwrap().features |= world::MatchDescriptor::SPELLS;
    frame.casts = vec![super::super::spell_protocol::CastState::initial(2, 1200)];
    frame.validate(r.echo.context.key).unwrap();
    let reply = world::Reply {
        echo: world::Request::read(r.echo.context, r.echo.channel_id),
        status: world::Status::Ok,
        reason: world::Reason::None,
        frame: Some(frame),
    };
    let bytes = world::tests::wire_complete_version(&reply, 23);
    assert_eq!(
        app::decode_response_version(&bytes, 23).unwrap().response,
        Response::World(reply.clone())
    );
    assert!(app::decode_response_version(&bytes, 22).is_err());
    let mut ack = session::Ack {
        request_id: 1,
        version: 23,
        numeric_schema: 1,
        capabilities: session::KNOWN_CAPABILITIES,
    };
    assert!(world::descriptor_negotiated(
        &Response::World(reply.clone()),
        &ack
    ));
    ack.version = 22;
    assert!(!world::descriptor_negotiated(&Response::World(reply), &ack));
}
#[test]
fn version_eighteen_appends_frozen_future_forms_and_preserves_legacy_snapshot() {
    let old = reply();
    let mut planned = resource_reply();
    assert_eq!(resource_bytes().len(), bytes().len() + 16);
    assert_eq!(
        planned.frame.as_mut().unwrap().resources.take(),
        Some(vec![1649, 2158])
    );
    assert_eq!(old, planned);
    assert!(app::decode_response_version(&resource_bytes(), 17).is_err());
    assert!(app::decode_response_version(&bytes(), 18).is_err());
}
#[test]
fn resource_extension_rejects_bad_schema_counts_order_reserved_and_truncation() {
    let good = resource_bytes();
    let base = good.len() - 16;
    for (offset, value) in [(1, 2), (3, 0), (3, 11), (7, 1), (11, 0), (15, 0)] {
        let mut bad = good.clone();
        bad[base + offset] = value;
        assert!(
            app::decode_response_version(&bad, 18).is_err(),
            "offset {offset}"
        );
    }
    for n in base..good.len() {
        let mut bad = good[..n].to_vec();
        bad[4..8].copy_from_slice(&(n as u32).to_be_bytes());
        assert!(app::decode_response_version(&bad, 18).is_err());
    }
    for forms in [
        vec![],
        vec![1649, 1649],
        vec![2158, 1649],
        vec![2158],
        vec![9999],
    ] {
        let mut bad = resource_reply();
        bad.frame.as_mut().unwrap().resources = Some(forms);
        assert!(bad.validate().is_err());
    }
}
#[test]
fn resource_roster_is_immutable_within_match_and_plan_requires_server_preflight() {
    use super::super::npc_presentation::Plan;
    let mut first = resource_reply();
    first.echo.action = Action::Read;
    first.echo.sequence = 0;
    first.echo.serial = 0;
    first.echo.life = 0;
    let plan = Plan::from_reply(&first).unwrap();
    assert!(plan.validate(&first).is_ok());
    assert!(Plan::from_reply(&reply()).is_err());
    let mut next = first.clone();
    let frame = next.frame.as_mut().unwrap();
    frame.revision += 1;
    frame.combat.world.revision += 1;
    frame.elapsed_millis += 1;
    assert!(follows(Some(&first), &next));
    next.frame.as_mut().unwrap().resources = Some(vec![1649]);
    assert!(next.validate().is_ok()); // Removing an unspawned form is still a generation change.
    assert!(!follows(Some(&first), &next));
    assert!(plan.validate(&next).is_err());
    next.frame.as_mut().unwrap().resources = None;
    assert!(!follows(Some(&first), &next));
}
fn reply() -> Reply {
    let packet = app::decode_response_version(&bytes(), 17).unwrap();
    assert_eq!(packet.request_id, 0xfedcba98);
    match packet.response {
        Response::Npc(r) => r,
        _ => panic!("wrong NPC kind"),
    }
}
pub(crate) fn read_reply() -> Reply {
    let mut r = reply();
    r.echo.action = Action::Read;
    r.echo.sequence = 0;
    r.echo.serial = 0;
    r.echo.life = 0;
    r
}
fn advanced() -> (Reply, Reply) {
    let a = read_reply();
    let mut b = a.clone();
    let f = b.frame.as_mut().unwrap();
    f.combat.world.revision += 1;
    f.revision += 1;
    f.elapsed_millis += 1;
    assert!(b.validate().is_ok());
    assert!(follows(Some(&a), &b));
    (a, b)
}
#[test]
fn java_engine_bytes_preserve_wide_hp_and_separate_human_bot_npc_identity() {
    let r = reply();
    assert!(r.confirms(&r.echo));
    let f = r.frame.unwrap();
    assert_eq!(
        f.scope,
        Scope {
            room_id: 1,
            match_id: 1,
            generation: 6
        }
    );
    assert_eq!(
        (f.revision, f.combat.world.revision, f.elapsed_millis),
        (7, 10, 100)
    );
    assert_eq!((f.spawned, f.actors.len(), f.exchanges.len()), (2, 2, 3));
    assert_eq!(
        (
            f.actors[0].catalog_source_id,
            f.actors[0].npc_id,
            f.actors[0].graphic
        ),
        (88, 45262, 1649)
    );
    assert_eq!((f.actors[0].hp, f.actors[0].max_hp), (HUGE - 14, HUGE));
    assert_eq!((f.actors[0].mp, f.actors[0].max_mp), (10, 10));
    assert_eq!(f.actors[0].action, NpcAction::Hurt);
    assert_eq!(f.actors[1].action, NpcAction::Attack);
    let human = f.exchanges[0].participant;
    let bot = f.exchanges[1].participant;
    assert_eq!((human.source_id, human.id), (bot.source_id, bot.id));
    assert_eq!(human.kind, MemberKind::Human);
    assert_eq!(bot.kind, MemberKind::ServerBot);
    assert_ne!(human, bot);
    assert_eq!(
        (
            f.exchanges[0].outcome.applied_damage,
            f.exchanges[0].outcome.remaining_hp
        ),
        (28, HUGE - 28)
    );
    assert_eq!(
        (
            f.exchanges[1].outcome.applied_damage,
            f.exchanges[1].outcome.remaining_hp
        ),
        (28, 12)
    );
    assert_eq!(f.exchanges[2].outcome.applied_damage, 14);
    assert_eq!(f.combat.vitals[0].attack_sequence, 1);
    assert_eq!(f.combat.vitals[0].last_target_entity_id, 0); // Never encode NPC as player entity.
    assert_eq!(f.combat.vitals[0].attack_revision, 0);
    assert_eq!(
        f.actor(Id {
            scope: Scope {
                generation: 7,
                ..f.scope
            },
            serial: 1
        }),
        None
    );
}
#[test]
fn request_matches_java_literal_and_has_no_client_damage_or_clock() {
    let r = Request {
        context: SelectionContext {
            key: CharacterKey {
                source_id: 42,
                object_id: i64::MAX as u64,
            },
            epoch: u64::MAX,
        },
        channel_id: 3,
        action: Action::Attack,
        world: Scope {
            room_id: 11,
            match_id: 22,
            generation: 33,
        },
        sequence: i64::MAX as u64,
        serial: 44,
        life: 55,
    };
    let expected = unhex(
        "4c544d3100000058fedcba98002600000000002a7fffffffffffffffffffffffffffffff03010000000000000000000b000000000000001600000000000000217fffffffffffffff000000000000002c0000000000000037",
    );
    assert_eq!(encode_request(0xfedcba98, &r).unwrap(), expected);
    assert_eq!(expected.len(), 88);
    assert_eq!(
        r.target(),
        Some(Id {
            scope: r.world,
            serial: 44
        })
    );
    let mut read = r;
    read.action = Action::Read;
    assert!(encode_request(1, &read).is_err());
    read.sequence = 0;
    read.serial = 0;
    read.life = 0;
    assert!(encode_request(1, &read).is_ok());
    assert!(read.target().is_none());
    for case in 0..8 {
        let mut bad = r;
        match case {
            0 => bad.context.key.source_id = 0,
            1 => bad.context.key.object_id = u64::MAX,
            2 => bad.context.epoch = 0,
            3 => bad.channel_id = 4,
            4 => bad.world.generation = 0,
            5 => bad.sequence = 0,
            6 => bad.serial = u64::MAX,
            _ => bad.life = 0,
        }
        assert!(encode_request(1, &bad).is_err(), "case {case}");
    }
}
#[test]
fn npc_capability_is_explicit_and_not_in_runtime_offer() {
    assert_eq!(RESOURCE_VERSION, 18);
    assert!(session::MAX_VERSION >= RESOURCE_VERSION);
    assert_eq!(
        session::KNOWN_CAPABILITIES
            & !(session::CAP_SLOW_DATA
                | session::CAP_HASTE
                | session::CAP_GREATER_HASTE
                | session::CAP_ACTION_LIFE
                | session::CAP_NPC_FIRE_STORM
                | session::CAP_CHAT
                | session::CAP_DIRECT_CHAT
                | session::CAP_FILTERED_CHAT
                | session::CAP_MATCHMAKING
                | session::CAP_LARGE_MAPS
                | session::CAP_BATTLE_DATA),
        2097151
    );
    let runtime = session::Hello::current(1);
    assert_eq!(runtime.supported_capabilities, 262143);
    let mut hello = runtime;
    hello.supported_capabilities = session::KNOWN_CAPABILITIES;
    assert!(session::encode(session::Message::Hello(hello)).is_ok());
    hello.max_version = 16;
    assert!(session::encode(session::Message::Hello(hello)).is_err());
    for cap in [session::CAP_COMBAT, session::CAP_MATCH_DESCRIPTOR] {
        hello.max_version = 17;
        hello.supported_capabilities = session::KNOWN_CAPABILITIES & !cap;
        assert!(session::encode(session::Message::Hello(hello)).is_err());
    }
    let response = Response::Npc(reply());
    let mut ack = session::Ack {
        request_id: 1,
        version: 17,
        numeric_schema: 1,
        capabilities: session::KNOWN_CAPABILITIES,
    };
    assert!(world::descriptor_negotiated(&response, &ack));
    ack.capabilities = session::RUNTIME_CAPABILITIES;
    assert!(!world::descriptor_negotiated(&response, &ack));
    ack.capabilities = session::KNOWN_CAPABILITIES;
    ack.version = 16;
    assert!(!world::descriptor_negotiated(&response, &ack));
}
#[test]
fn all_truncations_trailing_reserved_flags_counts_and_old_versions_reject() {
    let original = bytes();
    for n in 0..original.len() {
        assert!(
            app::decode_response_version(&original[..n], 17).is_err(),
            "prefix {n}"
        );
    }
    let mut bad = original.clone();
    bad.push(0);
    let n = bad.len() as u32;
    bad[4..8].copy_from_slice(&n.to_be_bytes());
    assert!(app::decode_response_version(&bad, 17).is_err());
    let appendix = 945;
    assert_eq!(original.len() - appendix, 56 + 2 * 112 + 3 * 104);
    for at in [15, 39, 91, appendix + 49, appendix + 50, appendix + 280 + 3] {
        let mut bad = original.clone();
        bad[at] = 1;
        assert!(
            app::decode_response_version(&bad, 17).is_err(),
            "reserved {at}"
        );
    }
    for at in [appendix + 52, appendix + 54] {
        let mut bad = original.clone();
        bad[at..at + 2].copy_from_slice(&u16::MAX.to_be_bytes());
        assert!(app::decode_response_version(&bad, 17).is_err());
    }
    for (at, value) in [
        (appendix + 56 + 23, 5),
        (appendix + 280, 2),
        (appendix + 280 + 1, 2),
        (appendix + 280 + 2, 16),
    ] {
        let mut bad = original.clone();
        bad[at] = value;
        assert!(app::decode_response_version(&bad, 17).is_err());
    }
    for v in 1..17 {
        assert!(app::decode_response_version(&original, v).is_err());
    }
}
#[test]
fn stream_consumes_one_frame_and_rejects_v16_or_oversized_header_before_body() {
    let one = bytes();
    let mut coalesced = Cursor::new([one.clone(), one.clone()].concat());
    assert_eq!(
        app::read_response_version(&mut coalesced, 17).unwrap(),
        app::decode_response_version(&one, 17).unwrap()
    );
    assert_eq!(coalesced.position(), one.len() as u64);
    app::read_response_version(&mut coalesced, 17).unwrap();
    assert_eq!(coalesced.position(), (one.len() * 2) as u64);
    let mut old = Cursor::new(one.clone());
    assert!(app::read_response_version(&mut old, 16).is_err());
    assert_eq!(old.position(), 16);
    let mut header = one[..16].to_vec();
    header[4..8].copy_from_slice(&((MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
    let mut stream = Cursor::new(header);
    assert!(app::read_response_version(&mut stream, 17).is_err());
    assert_eq!(stream.position(), 16);
}
#[test]
fn success_requires_exact_source_epoch_scope_sequence_target_and_life() {
    let r = reply();
    for case in 0..7 {
        let mut request = r.echo;
        match case {
            0 => request.context.key.source_id = 42,
            1 => request.context.epoch += 1,
            2 => request.world.generation += 1,
            3 => request.sequence += 1,
            4 => request.serial += 1,
            5 => request.life += 1,
            _ => request.channel_id = 1,
        }
        assert!(!r.confirms(&request), "case {case}");
    }
    for case in 0..4 {
        let mut bad = r.clone();
        match case {
            0 => bad.echo.serial = 2,
            1 => bad.echo.life = 2,
            2 => bad.echo.sequence = 2,
            _ => bad
                .frame
                .as_mut()
                .unwrap()
                .exchanges
                .pop()
                .map(|_| ())
                .unwrap(),
        }
        assert!(bad.validate().is_err());
    }
}
#[test]
fn invalid_numeric_namespace_descriptor_and_occupancy_are_rejected() {
    for case in 0..13 {
        let mut r = read_reply();
        let f = r.frame.as_mut().unwrap();
        match case {
            0 => f.actors[0].catalog_source_id = 0,
            1 => f.actors[0].hp = -1,
            2 => f.actors[0].max_hp = f.actors[0].hp - 1,
            3 => f.actors[0].graphic = 999999,
            4 => f.actors[0].cell = f.combat.world.actors[0].position,
            5 => f.actors[1].cell = f.actors[0].cell,
            6 => f.actors[0].action = NpcAction::Death,
            7 => f.actors[0].life = 0,
            8 => f.actors[0].next_attack_millis = MAX_ELAPSED + 60_001,
            9 => f.scope.generation += 1,
            10 => {
                f.combat.world.descriptor.as_mut().unwrap().features |=
                    world::MatchDescriptor::HONOR
            }
            11 => f.exchanges[2].outcome.applied_damage = 15,
            _ => f.exchanges[2].outcome.remaining_hp -= 1,
        }
        assert!(r.validate().is_err(), "case {case}");
    }
    let mut r = read_reply();
    let f = r.frame.as_mut().unwrap();
    f.actors[0].max_hp = i64::MAX;
    f.actors[0].mp = i64::MAX;
    f.actors[0].max_mp = i64::MAX;
    assert!(r.validate().is_ok());
}
#[test]
fn unchanged_and_skipped_snapshots_follow_without_replaying_damage() {
    let (a, b) = advanced();
    assert!(follows(None, &a));
    assert!(follows(Some(&a), &a));
    assert!(follows(Some(&a), &b));
    assert!(!follows(Some(&b), &a));
    assert_eq!(
        a.frame.as_ref().unwrap().combat.vitals,
        b.frame.as_ref().unwrap().combat.vitals
    );
    for case in 0..6 {
        let mut bad = b.clone();
        let f = bad.frame.as_mut().unwrap();
        match case {
            0 => f.elapsed_millis = 99,
            1 => f.spawned = 1,
            2 => f.revision = 6,
            3 => bad.echo.context.epoch += 1,
            4 => bad.echo.channel_id = 1,
            _ => bad.echo.world.generation += 1,
        }
        assert!(!follows(Some(&a), &bad), "case {case}");
    }
}
#[test]
fn frozen_catalog_fields_and_unchanged_revision_cannot_mutate() {
    let (a, b) = advanced();
    for case in 0..9 {
        let mut bad = b.clone();
        let f = bad.frame.as_mut().unwrap();
        let n = &mut f.actors[1];
        match case {
            0 => n.catalog_source_id += 1,
            1 => n.npc_id += 1,
            2 => n.graphic = 2158,
            3 => n.max_hp += 1,
            4 => n.max_mp += 1,
            5 => n.stage = 2,
            6 => n.lane = 1,
            7 => n.born_revision += 1,
            _ => n.heading = 0,
        }
        assert!(!follows(Some(&a), &bad), "case {case}");
    }
    let mut changed = b.clone();
    let f = changed.frame.as_mut().unwrap();
    f.actors[1].heading = 0;
    f.actors[1].changed_revision = f.revision;
    assert!(follows(Some(&a), &changed));
    changed.frame.as_mut().unwrap().revision = a.frame.as_ref().unwrap().revision;
    assert!(!follows(Some(&a), &changed));
}
#[test]
fn corpse_retirement_and_new_serial_do_not_resurrect_old_npc() {
    let (a, mut dead) = advanced();
    let f = dead.frame.as_mut().unwrap();
    let n = &mut f.actors[1];
    n.hp = 0;
    n.action = NpcAction::Death;
    n.corpse_until_millis = 200;
    n.next_move_millis = 0;
    n.next_attack_millis = 0;
    n.changed_revision = f.revision;
    assert!(follows(Some(&a), &dead));
    let mut missing = dead.clone();
    let f = missing.frame.as_mut().unwrap();
    f.combat.world.revision += 1;
    f.revision += 1;
    f.actors.pop();
    f.exchanges
        .retain(|e| e.direction != Direction::NpcToParticipant || e.npc.serial != 2);
    assert!(!follows(Some(&dead), &missing));
    missing.frame.as_mut().unwrap().elapsed_millis = 200;
    assert!(follows(Some(&dead), &missing));
    let mut returned = missing.clone();
    let f = returned.frame.as_mut().unwrap();
    f.combat.world.revision += 1;
    f.revision += 1;
    let mut n = a.frame.as_ref().unwrap().actors[1].clone();
    n.born_revision = f.revision;
    n.changed_revision = f.revision;
    f.actors.push(n);
    assert!(!follows(Some(&missing), &returned));
    let f = returned.frame.as_mut().unwrap();
    f.spawned = 3;
    f.actors[1].serial = 3;
    assert!(follows(Some(&missing), &returned));
    let mut revival = dead.clone();
    let f = revival.frame.as_mut().unwrap();
    f.combat.world.revision += 1;
    f.revision += 1;
    let n = &mut f.actors[1];
    n.hp = 10;
    n.action = NpcAction::Idle;
    n.corpse_until_millis = 0;
    n.changed_revision = f.revision;
    assert!(!follows(Some(&dead), &revival));
    revival.frame.as_mut().unwrap().actors[1].life += 1;
    assert!(follows(Some(&dead), &revival));
}
#[test]
fn receipts_cannot_be_removed_rewritten_or_inserted_retroactively() {
    let (a, b) = advanced();
    for index in 0..3 {
        let mut bad = b.clone();
        bad.frame.as_mut().unwrap().exchanges.remove(index);
        assert!(!follows(Some(&a), &bad));
    }
    let mut bad = b.clone();
    bad.frame.as_mut().unwrap().exchanges[0].outcome.critical = false;
    assert!(!follows(Some(&a), &bad));
    let mut before = a.clone();
    before.frame.as_mut().unwrap().exchanges.remove(0);
    assert!(before.validate().is_ok());
    assert!(!follows(Some(&before), &b));
    let mut new = b.clone();
    let f = new.frame.as_mut().unwrap();
    let mut e = f.exchanges.remove(0);
    e.revision = f.combat.world.revision;
    f.actors[0].action = NpcAction::Attack;
    f.actors[0].changed_revision = f.revision;
    f.combat.vitals[0].hurt_revision = e.revision;
    f.exchanges.push(e);
    assert!(new.validate().is_ok());
    assert!(follows(Some(&a), &new));
}
#[test]
fn aborted_snapshot_retires_all_npcs_and_is_terminal() {
    let (a, mut b) = advanced();
    let f = b.frame.as_mut().unwrap();
    f.combat.world.phase = world::Phase::Aborted;
    f.combat.world.abort_reason = world::AbortReason::ServerAbort;
    f.closed = true;
    f.actors.clear();
    f.exchanges
        .retain(|e| e.direction == Direction::PlayerToNpc);
    assert!(b.validate().is_ok());
    assert!(follows(Some(&a), &b));
    assert!(follows(Some(&b), &b));
    assert!(!follows(Some(&b), &a));
    let mut future = b.clone();
    future.frame.as_mut().unwrap().combat.world.revision += 1;
    assert!(!follows(Some(&b), &future));
}
#[test]
fn unavailable_is_explicit_and_does_not_erase_established_state() {
    let a = read_reply();
    let mut b = a.clone();
    b.frame = None;
    b.status = Status::Unavailable;
    b.reason = Reason::NotActive;
    assert!(b.validate().is_ok());
    assert!(follows(None, &b));
    let response = Response::Npc(b.clone());
    assert!(!world::descriptor_negotiated(
        &response,
        &session::Ack {
            request_id: 1,
            version: 17,
            numeric_schema: 1,
            capabilities: session::RUNTIME_CAPABILITIES,
        }
    ));
    assert!(!follows(Some(&a), &b));
    assert!(follows(Some(&b), &a));
    b.status = Status::Ok;
    b.reason = Reason::None;
    assert!(b.validate().is_err());
}

#[test]
fn receipts_never_predict_future_life_or_replay_a_player_sequence_at_new_revision() {
    let (a, mut b) = advanced();
    b.frame.as_mut().unwrap().exchanges[0].npc_life = 2;
    assert!(b.validate().is_err());
    let (_, mut b) = advanced();
    let f = b.frame.as_mut().unwrap();
    let e = f.exchanges.last_mut().unwrap();
    e.revision = f.combat.world.revision;
    assert!(b.validate().is_ok());
    assert!(!follows(Some(&a), &b));
    let f = b.frame.as_mut().unwrap();
    f.exchanges.last_mut().unwrap().request_sequence += 1;
    f.combat.vitals[0].attack_sequence += 1;
    assert!(follows(Some(&a), &b));
}
