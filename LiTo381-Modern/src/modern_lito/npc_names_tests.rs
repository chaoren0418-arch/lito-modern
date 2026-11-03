use super::*;

// Frozen from ModernNpcNamesTest's actual Java engine and v28 encoder.
const JAVA_V28: &str = "4c544d310000040efedcba9800270000000000070000000000000001000000000000000100000000000000000000000100000000000000010000000000000005000000000000000000000000000000000000000000000000000001000000000000000001000000000000000500000000000000050300000000000000000000010000000000000005006402000000022a0004726f6f6d0002000201000000000000070000000000000001000200000000000000000007000000000000000100000000000000500001000001000000000768756d616e2d3000087365727665722d37000000000000002a000000000000000100000000000000500001000101000000000768756d616e2d3100097365727665722d3432000100010000022a84325f1245e4c25bd4e28eac3d50cab6e0252d9beaded46c1f0928a9ad15bd3a800080000000c012ef81ffbc2095f821ee3ae6e2d67fb8505b266ad2163bef49997d5e9cfce0a0a20002000000000000000000010000800a0000800a000100000000000000000000000000000bebc2000000000000000002000080140000800a000100000000000000000000000000000bebc200000000000000000001000000000000000001050000000801000007d0000007d000000000000000000000ea60000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000010000000000000064000000000000006400000000000000320000000000000032000000000000000a0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029b9270000000000000000020000000000000064000000000000006400000000000000320000000000000032000000000000000a0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029b9270000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000001000000000000000500000000000000020000000000000000000000000000000100000000000100000000000000000001000000580000b0ce00000671000102000000800b0000800a00000000000001180000000000000118000000000000000a000000000000000a0000000000000001000000000000000200000000000000020000000000000000000000000000006400000000000000000001000200000000000006710000086e0001000200000000000000580000b0ce000006710006476f626c696e0003243431000000580000b0cf0000086e0004576f6c660000";

#[test]
fn java_v28_full_packet_retains_typed_npcs_and_requires_the_complete_name_appendix() {
    let packet: Vec<_> = JAVA_V28
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect();
    let response = app::decode_response_version(&packet, 28).unwrap();
    let app::Response::Npc(reply) = response.response else {
        panic!("wrong NPC reply");
    };
    reply.validate().unwrap();
    let frame = reply.frame.as_ref().unwrap();
    assert_eq!(frame.names.as_ref().unwrap(), &labels());
    assert_eq!(frame.actors.len(), 1);
    let actor = &frame.actors[0];
    assert_eq!(
        frame
            .name(Id {
                scope: frame.scope,
                serial: actor.serial
            })
            .unwrap()
            .name,
        "Goblin"
    );
    assert_eq!(actor.hp, 280);
    let mut old = packet[..packet.len() - 53].to_vec();
    let length = old.len() as u32;
    old[4..8].copy_from_slice(&length.to_be_bytes());
    let app::Response::Npc(old_reply) = app::decode_response_version(&old, 27).unwrap().response
    else {
        panic!("wrong old reply");
    };
    assert!(old_reply.frame.as_ref().unwrap().names.is_none());
    assert_eq!(old_reply.frame.as_ref().unwrap().actors, frame.actors);
    assert!(app::decode_response_version(&old, 28).is_err());
    assert!(app::decode_response_version(&packet, 27).is_err());
    assert!(app::decode_response_version(&packet, 29).is_err());
    for n in old.len()..packet.len() {
        let mut short = packet[..n].to_vec();
        short[4..8].copy_from_slice(&(n as u32).to_be_bytes());
        assert!(
            app::decode_response_version(&short, 28).is_err(),
            "length {n}"
        );
    }
}

// Independent literal: schema/count/reserved + two source/NPC/graphic/text rows.
const WIRE: &str = "0001000200000000000000580000b0ce000006710006476f626c696e0003243431000000580000b0cf0000086e0004576f6c660000";
fn bytes() -> Vec<u8> {
    WIRE.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
pub(super) fn labels() -> Vec<Name> {
    let b = bytes();
    let mut r = Reader { bytes: &b, pos: 0 };
    let rows = r.names().unwrap();
    assert_eq!(r.pos, b.len());
    rows
}

#[test]
fn exact_name_appendix_is_bounded_utf8_and_has_no_database_or_player_key() {
    let rows = labels();
    assert_eq!(bytes().len(), 53);
    assert_eq!(rows[0].source_id, 88);
    assert_eq!(rows[0].npc_id, 45262);
    assert_eq!(rows[0].graphic, 1649);
    assert_eq!(
        (rows[0].name.as_str(), rows[0].name_id.as_str()),
        ("Goblin", "$41")
    );
    assert_eq!(
        (rows[1].name.as_str(), rows[1].name_id.as_str()),
        ("Wolf", "")
    );
    let mut unicode = rows[0].clone();
    unicode.name = "骷髏&(太陽神)".into();
    assert!(unicode.validate());
    for text in [
        "".into(),
        "  ".into(),
        "怪\n物".into(),
        "x".repeat(257),
        "怪".repeat(86),
    ] {
        unicode.name = text;
        assert!(!unicode.validate());
    }
}

#[test]
fn name_appendix_rejects_truncation_reserved_duplicate_unknown_graphic_and_invalid_strings() {
    let packet = bytes();
    for n in 0..packet.len() {
        assert!(
            Reader {
                bytes: &packet[..n],
                pos: 0
            }
            .names()
            .is_err(),
            "length {n}"
        );
    }
    for (offset, value) in [
        (1, 2),
        (3, 0),
        (3, 129),
        (4, 1),
        (8, 255),
        (19, 255),
        (21, 255),
        (22, 255),
        (22, 10),
        (40, 206),
    ] {
        let mut bad = packet.clone();
        bad[offset] = value;
        assert!(
            Reader {
                bytes: &bad,
                pos: 0
            }
            .names()
            .is_err(),
            "offset {offset}"
        );
    }
}

#[test]
fn catalog_names_bind_source_graphic_and_full_generation_not_live_spawn_count() {
    let mut reply = tests::read_reply();
    let frame = reply.frame.as_mut().unwrap();
    frame.resources = Some(vec![1649, 2158]);
    frame.names = Some(labels());
    frame.validate().unwrap();
    let id = Id {
        scope: frame.scope,
        serial: frame.actors[0].serial,
    };
    assert_eq!(frame.name(id).unwrap().name, "Goblin");
    assert!(frame
        .name(Id {
            scope: Scope {
                generation: id.scope.generation + 1,
                ..id.scope
            },
            ..id
        })
        .is_none());
    assert!(follows(Some(&reply), &reply));
    let mut changed = reply.clone();
    changed.frame.as_mut().unwrap().names.as_mut().unwrap()[0].name =
        "renamed during battle".into();
    assert!(!follows(Some(&reply), &changed));
    for fault in 0..5 {
        let mut bad = reply.clone();
        let frame = bad.frame.as_mut().unwrap();
        match fault {
            0 => frame.names.as_mut().unwrap()[0].source_id = 7,
            1 => frame.names.as_mut().unwrap()[0].npc_id = 45261,
            2 => frame.names.as_mut().unwrap()[0].graphic = 2158,
            3 => frame.resources = None,
            _ => frame.names.as_mut().unwrap().pop().map(|_| ()).unwrap(),
        }
        assert!(frame.validate().is_err(), "fault {fault}");
    }
    // Future unspawned definitions are retained and cannot be silently replaced
    // by a label set for the current visible lane/actors only.
    assert_eq!(
        reply.frame.as_ref().unwrap().names.as_ref().unwrap().len(),
        2
    );
}
