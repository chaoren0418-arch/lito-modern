//! Paired schema3 data qualification, deliberately NOT negotiated gameplay.
use super::{
    super::{
        codec::{Reader, Wire, Writer},
        data::*,
        *,
    },
    Peer,
};
use crate::modern_lito::{
    application_protocol as app, battle_follow::Follower, battle_read as read,
    equipment_history_wire as history, protocol, timed_protocol,
};

fn primitive<T: Wire>(value: &T, schema: u16) -> Result<Vec<u8>, app::ApplicationError> {
    let mut n = Writer::count();
    n.schema = schema;
    n.write(value)?;
    let mut w = Writer::allocated(n.pos)?;
    w.schema = schema;
    w.write(value)?;
    Ok(w.bytes.unwrap())
}
fn parse<T: Wire>(bytes: &[u8], schema: u16) -> Result<T, app::ApplicationError> {
    let mut r = Reader::new(bytes);
    r.schema = schema;
    let value = r.read()?;
    r.finish()?;
    Ok(value)
}
#[test]
fn greater_haste_data_tags_are_appended_and_old_schemas_refuse_new_details() {
    assert_eq!(primitive(&Lane::GreaterHaste, 3).unwrap(), [8]);
    for schema in [1, 2] {
        assert!(primitive(&Lane::GreaterHaste, schema).is_err());
        assert!(parse::<Lane>(&[8], schema).is_err());
    }
    assert!(parse::<Lane>(&[9], 3).is_err());
    let detail = Detail::GreaterHaste {
        active_elapsed_nanos: 2_000_000_000,
        haste_until_nanos: 2_402_000_000_000,
    };
    let bytes = primitive(&detail, 3).unwrap();
    assert_eq!(bytes.len(), 17);
    assert_eq!(bytes[0], 6);
    assert_eq!(parse::<Detail>(&bytes, 3).unwrap(), detail);
    for schema in [1, 2] {
        assert!(primitive(&detail, schema).is_err());
        assert!(parse::<Detail>(&bytes, schema).is_err());
    }
    let mut bad = bytes.clone();
    bad[0] = 7;
    assert!(parse::<Detail>(&bad, 3).is_err());
    assert!(parse::<Detail>(&bytes[..16], 3).is_err());
}
#[test]
fn greater_haste_origin_does_not_change_old_buff_bytes_or_old_potion_bound() {
    let mut b = Buffs {
        entity: 1,
        revision: 2,
        observed_nanos: 0,
        haste_until_nanos: 300_000_000_000,
        brave_until_nanos: 0,
        wisdom_until_nanos: 0,
        brave_kind: 0,
        slow_until_nanos: 0,
        slow_revision: 0,
        haste_origin: HasteOrigin::Potion,
    };
    let old = primitive(&b, 1).unwrap();
    assert_eq!(old.len(), 52);
    assert_eq!(old, primitive(&b, 2).unwrap());
    assert_eq!(parse::<Buffs>(&old, 1).unwrap(), b);
    b.haste_origin = HasteOrigin::GreaterHaste;
    b.haste_until_nanos = 2_400_000_000_000;
    assert!(primitive(&b, 1).is_err());
    assert!(primitive(&b, 2).is_err());
    let bytes = primitive(&b, 3).unwrap();
    assert_eq!(bytes.len(), 53);
    assert_eq!(bytes[52], 2);
    assert_eq!(parse::<Buffs>(&bytes, 3).unwrap(), b);
    let mut bad = bytes;
    bad[52] = 3;
    assert!(parse::<Buffs>(&bad, 3).is_err());
    let mut state = timed_protocol::State::initial(1);
    state.revision = 2;
    state.haste_until_nanos = b.haste_until_nanos;
    assert!(state.validate().is_err());
    assert!(state.validate_greater_haste().is_ok());
    state.haste_until_nanos += 1;
    assert!(state.validate_greater_haste().is_err());
    state.haste_until_nanos = 0;
    assert!(state.validate_greater_haste().is_err());
}
#[test]
fn greater_haste_schema_requires_feature_and_presentation_without_admitting_gameplay() {
    let mut d = data_tests::fixture();
    d.world.descriptor.features |= 1 << 21;
    assert!(encode(&d).is_err());
    d.presentation = Some(List(vec![]));
    let bytes = encode(&d).unwrap();
    assert_eq!(inspect(&bytes).unwrap().schema(), 3);
    assert_eq!(decode(&bytes).unwrap(), d);
    assert!(d
        .check_scene_values(&d.personal.player, &d.world.identity)
        .is_err());
    assert!(decode_legacy(&bytes).is_err());
    let mut changed = bytes;
    changed[5] = 2;
    assert!(decode(&changed).is_err());
}
fn spell(d: &UnvalidatedData) -> &Spell {
    d.actors.0[0]
        .spells
        .histories
        .0
        .iter()
        .find(|s| s.lane == Lane::GreaterHaste)
        .unwrap()
}
fn spell_mut(d: &mut UnvalidatedData) -> &mut Spell {
    d.actors.0[0]
        .spells
        .histories
        .0
        .iter_mut()
        .find(|s| s.lane == Lane::GreaterHaste)
        .unwrap()
}
fn request(d: &UnvalidatedData) -> read::Request {
    read::Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: d.personal.player.source as u32,
                object_id: d.personal.player.object as u64,
            },
            epoch: d.personal.player.epoch as u64,
        },
        channel_id: d.personal.player.channel as u8,
        world: d.world.identity.clone(),
    }
}
fn rejects(d: &UnvalidatedData, edit: impl FnOnce(&mut UnvalidatedData)) {
    let mut changed = d.clone();
    edit(&mut changed);
    assert!(
        changed
            .check_scene_values(&d.personal.player, &d.world.identity)
            .is_err(),
        "invalid scene accepted"
    );
}
fn legacy_rejects(d: &UnvalidatedData, bytes: &[u8]) {
    assert!(decode_legacy(bytes).is_err());
    let ack = protocol::Ack {
        request_id: 1,
        version: 41,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    };
    let q = request(d);
    let reply = read::UnvalidatedReply {
        echo: q.clone(),
        state: read::State::Data,
        data: Some(d.clone()),
    };
    assert!(!read::reply_negotiated(&reply, &ack));
    let mut follower = Follower::new(q.clone(), ack).unwrap();
    assert!(follower.accept(reply).is_err());
    assert!(follower.observation().is_none());
    // Exercise the existing wire decoder as well as the typed-reply gate.
    let encoded = read::encode_request(1, &q).unwrap();
    let mut body = encoded[16..].to_vec();
    body.extend([1, 0, 0, 0]);
    body.extend((bytes.len() as u32).to_be_bytes());
    body.extend(bytes);
    assert!(read::decode_body(&body, 41).is_err());
}
#[test]
#[ignore = "explicit Java engine + durable equipment peer; schema3 data only, no command/TLS/UI admission"]
fn java_greater_haste_schema3_source_cost_gear_life_and_legacy_rejection() {
    let mut peer = Peer::start_named(Some("greater-haste"));
    let labels = [
        "initial",
        "cast-a",
        "cast-b",
        "denied",
        "potion",
        "refresh",
        "aborted",
        "gear-initial",
        "gear-cast",
        "gear-set",
        "gear-recast",
        "gear-revived",
    ];
    let mut frames = Vec::new();
    for label in labels {
        let line = peer.line();
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "FRAME");
        assert_eq!(fields[1], label);
        let facts = fields[2]
            .split(',')
            .map(|x| x.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(facts.len(), 30);
        assert_eq!(fields[3].len() % 2, 0);
        assert!(fields[3].len() <= MAX_PRESENTATION_BYTES * 2);
        let bytes = fields[3]
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(inspect(&bytes).unwrap().schema(), 3);
        let d = decode(&bytes).unwrap();
        let rebuilt = encode(&d).unwrap();
        assert_eq!(rebuilt, bytes, "{label}");
        assert_eq!(decode(&rebuilt).unwrap(), d);
        d.check_scene_values(&d.personal.player, &d.world.identity)
            .unwrap_or_else(|_| panic!("{label}: scene rejected"));
        assert_eq!(
            [
                d.world.identity.room,
                d.world.identity.match_id,
                d.world.identity.generation,
                d.world.revision,
                i64::from(d.personal.player.source),
                d.personal.player.object,
                d.personal.player.epoch,
                d.personal.entity
            ],
            facts[..8]
        );
        let v = &d.actors.0[0].vital.as_ref().unwrap().values;
        assert_eq!([v.hp, v.maximum_hp, v.mp, v.maximum_mp], facts[18..22]);
        let h = &d.personal.history.as_ref().unwrap().0;
        assert_eq!(h.resources.flows.spent, facts[28]);
        let hbytes = history::encode_with_schema(h, 2).unwrap();
        assert_eq!(history::decode_with_schema(&hbytes, 2).unwrap(), *h);
        assert!(history::encode(h).is_err());
        assert!(history::decode(&hbytes).is_err());
        legacy_rejects(&d, &bytes);
        // The new prefix cannot be downgraded, nor can nested LHI2 be relabeled LHI1.
        let mut bad = bytes.clone();
        bad[5] = 2;
        assert!(decode(&bad).is_err());
        let offsets = bytes
            .windows(6)
            .enumerate()
            .filter_map(|(i, b)| (b == [0x4c, 0x48, 0x49, 0x31, 0, 2]).then_some(i))
            .collect::<Vec<_>>();
        assert_eq!(offsets.len(), 1);
        let mut bad = bytes.clone();
        bad[offsets[0] + 5] = 1;
        assert!(decode(&bad).is_err());
        assert!(decode(&bytes[..bytes.len() - 1]).is_err());
        let mut bad = bytes.clone();
        bad.push(0);
        assert!(decode(&bad).is_err());
        peer.ack(&rebuilt);
        frames.push(d);
    }
    assert_eq!(peer.line(), "CLOSED:12");
    peer.finish();
    let initial = &frames[0];
    let cast = &frames[1];
    let other = &frames[2];
    let denied = &frames[3];
    assert!(spell(initial).receipt.is_none());
    assert_eq!(spell(cast).ledger.spent, 60);
    assert_eq!(cast.personal.player.object, other.personal.player.object);
    assert_ne!(cast.personal.player.source, other.personal.player.source);
    assert_eq!(cast.actors, other.actors);
    assert_eq!(
        other
            .personal
            .history
            .as_ref()
            .unwrap()
            .0
            .resources
            .flows
            .spent,
        0
    );
    assert_eq!(cast.actors.0[0].resources, denied.actors.0[0].resources);
    assert_eq!(spell(cast), spell(denied));
    assert!(denied.actors.0[0].spells.sequence > cast.actors.0[0].spells.sequence);
    assert_eq!(
        frames[4].actors.0[0].buffs.as_ref().unwrap().haste_origin,
        HasteOrigin::Potion
    );
    assert_eq!(
        frames[5].actors.0[0]
            .buffs
            .as_ref()
            .unwrap()
            .haste_until_nanos,
        2_402_000_000_000
    );
    assert_eq!(
        frames[6].actors.0[0].buffs.as_ref().unwrap().haste_origin,
        HasteOrigin::None
    );
    assert_eq!(spell(&frames[8]).ledger.spent, 54);
    assert_eq!(spell(&frames[9]).ledger.cost, 60);
    assert_eq!(spell(&frames[8]).last, spell(&frames[9]).last);
    assert_eq!(spell(&frames[10]).ledger.spent, 114);
    assert_eq!(spell(&frames[10]).ledger.prior_spent, 54);
    crate::modern_lito::battle_haste::tests::verify_frames(&frames);
    assert_eq!(
        spell(&frames[10])
            .last
            .as_ref()
            .unwrap()
            .before
            .equipment_revision,
        1
    );
    assert_eq!(
        frames[11].actors.0[0].buffs.as_ref().unwrap().haste_origin,
        HasteOrigin::None
    );
    assert!(
        frames[11].actors.0[0].resources.as_ref().unwrap().life
            > frames[10].actors.0[0].resources.as_ref().unwrap().life
    );
    rejects(cast, |d| {
        d.actors.0[0].buffs.as_mut().unwrap().haste_origin = HasteOrigin::Potion
    });
    rejects(cast, |d| {
        d.actors.0[0].buffs.as_mut().unwrap().haste_origin = HasteOrigin::None
    });
    rejects(cast, |d| {
        d.actors.0[0].buffs.as_mut().unwrap().haste_until_nanos -= 1
    });
    rejects(cast, |d| spell_mut(d).receipt = None);
    rejects(cast, |d| {
        spell_mut(d).receipt.as_mut().unwrap().detail = Detail::Ordinary
    });
    rejects(cast, |d| {
        spell_mut(d).receipt.as_mut().unwrap().detail = Detail::GreaterHaste {
            active_elapsed_nanos: i64::MIN,
            haste_until_nanos: 2_400_000_000_000,
        }
    });
    rejects(cast, |d| {
        spell_mut(d)
            .receipt
            .as_mut()
            .unwrap()
            .impact
            .usage
            .before
            .life += 1
    });
    rejects(cast, |d| {
        spell_mut(d).receipt.as_mut().unwrap().loading_elapsed_nanos += 1
    });
    rejects(cast, |d| d.world.descriptor.features &= !(1 << 21));
    rejects(cast, |d| d.personal.player.source = 42);
    rejects(cast, |d| d.personal.player.epoch ^= 1);
    rejects(cast, |d| d.personal.skills.0.retain(|s| s.id != 54));
    for d in [&frames[4], &frames[6], &frames[11]] {
        rejects(d, |bad| {
            if let Detail::GreaterHaste {
                active_elapsed_nanos,
                haste_until_nanos,
            } = &mut spell_mut(bad).receipt.as_mut().unwrap().detail
            {
                *active_elapsed_nanos += 1;
                *haste_until_nanos += 1;
            } else {
                panic!("retained greater-haste detail missing");
            }
        });
    }
}

#[test]
#[ignore = "explicit Java engine command contract peer; not registered TLS/UI permission"]
fn java_greater_haste_command_echo_success_denial_and_atomic_candidate_history() {
    use crate::modern_lito::battle_greater_haste as contract;
    let mut peer = Peer::start_named(Some("greater-haste-command"));
    let hex = |s: &str| {
        assert_eq!(s.len() % 2, 0);
        assert!(s.len() <= contract::MAX_RESULT_BYTES * 2);
        s.as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>()
    };
    let expected = [
        ("loading", contract::Reason::NotActive),
        ("success", contract::Reason::None),
        ("cooldown", contract::Reason::TooFast),
        ("other-source", contract::Reason::None),
        ("refresh", contract::Reason::None),
        ("retired", contract::Reason::NotActive),
        ("last-mp", contract::Reason::None),
        ("no-mp", contract::Reason::NoMp),
        ("intervening-spend", contract::Reason::TooFast),
    ];
    for (label, reason) in expected {
        let line = peer.line();
        let fields = line.splitn(5, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 5);
        assert_eq!(fields[0], "HASTE");
        assert_eq!(fields[1], label);
        let before = decode(&hex(fields[2])).unwrap();
        let request = hex(fields[3]);
        let bytes = hex(fields[4]);
        let q = contract::decode_intent(&request, 42).unwrap();
        assert_eq!(q.request_id, u32::MAX);
        assert_eq!(contract::encode_intent(&q, 42).unwrap(), request);
        let reply = contract::decode_reply(&bytes, 42, &q).unwrap();
        // Actual framed reader must tolerate fragmented delivery and leave the
        // following coalesced frame unread; the command echo remains independent.
        struct Fragmented(std::io::Cursor<Vec<u8>>);
        impl std::io::Read for Fragmented {
            fn read(&mut self, dst: &mut [u8]) -> std::io::Result<usize> {
                let n = dst.len().min(3);
                std::io::Read::read(&mut self.0, &mut dst[..n])
            }
        }
        let mut joined = bytes.clone();
        joined.extend_from_slice(b"NEXT");
        let mut stream = Fragmented(std::io::Cursor::new(joined));
        assert_eq!(
            contract::read_reply(&mut stream, &request, 42).unwrap(),
            reply
        );
        assert_eq!(stream.0.position() as usize, bytes.len());
        let mut short = std::io::Cursor::new(&bytes[..bytes.len() - 1]);
        assert!(contract::read_reply(&mut short, &request, 42).is_err());
        assert_eq!(reply.reason, reason);
        assert_eq!(
            reply.validate_after(&q, &before).is_ok(),
            label != "intervening-spend",
            "{label}: pending command must account for all own spending"
        );
        assert_eq!(format!("{reply:?}"), "GreaterHasteReceipt[redacted]");
        let unchanged = before.clone();
        let mut wrong = q;
        wrong.request_id -= 1;
        assert!(contract::decode_reply(&bytes, 42, &wrong).is_err());
        wrong = q;
        wrong.sequence += 1;
        assert!(contract::decode_reply(&bytes, 42, &wrong).is_err());
        wrong = q;
        wrong.stamp.context.epoch ^= 1;
        assert!(contract::decode_reply(&bytes, 42, &wrong).is_err());
        for version in [1, 41, 44] {
            assert!(contract::decode_reply(&bytes, version, &q).is_err());
        }
        for offset in [12, 14, 121, 123, 147, 148] {
            let mut bad = bytes.clone();
            bad[offset] ^= 127;
            assert!(contract::decode_reply(&bad, 42, &q).is_err());
        }
        let mut bad = bytes.clone();
        bad[120] = 10;
        assert!(contract::decode_reply(&bad, 42, &q).is_err());
        bad = bytes.clone();
        bad[157] = 2;
        assert!(contract::decode_reply(&bad, 42, &q).is_err());
        assert!(contract::decode_reply(&bytes[..bytes.len() - 1], 42, &q).is_err());
        bad = bytes.clone();
        bad.push(0);
        assert!(contract::decode_reply(&bad, 42, &q).is_err());
        let mut changed = reply.clone();
        changed.echo.sequence += 1;
        assert!(changed.validate_after(&q, &before).is_err(), "{label}");
        let mut changed = reply.clone();
        changed.echo.request_id -= 1;
        assert!(changed.validate_after(&q, &before).is_err(), "{label}");
        let mut old = before.clone();
        old.personal.player.source = if old.personal.player.source == 7 {
            42
        } else {
            7
        };
        assert!(reply.validate_after(&q, &old).is_err());
        assert_eq!(
            before, unchanged,
            "failed candidates never consume previous state"
        );
        if reason == contract::Reason::None {
            let mut lie = reply.clone();
            lie.reason = contract::Reason::TooFast;
            assert!(lie.validate().is_err());
        } else if reason != contract::Reason::NotActive {
            let mut lie = reply.clone();
            lie.reason = contract::Reason::None;
            assert!(lie.validate().is_err());
        }
        peer.ack(&bytes);
    }
    assert_eq!(peer.line(), "CLOSED:9");
    peer.finish();
}
