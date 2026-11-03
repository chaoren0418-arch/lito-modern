//! Java-selected result families into the Rust complete payload readers.
//! READ traverses the outer selected Application reader, including its bounds.
use super::{
    healing_admission::{request, selected, unhex},
    Peer,
};
use crate::modern_lito::{
    application_protocol as app, battle_action, battle_equipment as gear, battle_follow::Follower,
    battle_greater_haste as greater, battle_operation as operation, battle_payload as payload,
    battle_read as read, battle_revival as revival, battle_slow as slow,
    battle_target_haste as haste, battle_window::Stamp, battle_world as world,
};
use std::{collections::BTreeMap, io::Cursor};
fn i64_at(b: &[u8], at: usize) -> i64 {
    i64::from_be_bytes(b[at..at + 8].try_into().unwrap())
}
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}
fn guard(q: &[u8]) -> battle_action::Guard {
    battle_action::Guard {
        stamp: Stamp {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: u32_at(q, 16),
                    object_id: i64_at(q, 20) as u64,
                },
                epoch: i64_at(q, 28) as u64,
            },
            channel: u32_at(q, 36).try_into().unwrap(),
            world: [i64_at(q, 40), i64_at(q, 48), i64_at(q, 56)],
            entity: i64_at(q, 64),
            life: i64_at(q, 72),
            revision: i64_at(q, 80),
        },
        target: None,
    }
}
fn decode(
    kind: u16,
    b: &[u8],
    q: &[u8],
    ext: Option<u32>,
) -> Result<payload::UnvalidatedData, app::ApplicationError> {
    let mut cursor = Cursor::new(b);
    macro_rules! family {
        ($m:ident) => {
            match ext {
                Some(e) => $m::read_reply_selected(&mut cursor, q, selected(e))?,
                None => $m::read_reply(&mut cursor, q, 47)?,
            }
        };
    }
    let d = match kind {
        60 => {
            let envelope = match ext {
                Some(e) => app::read_response_selected(&mut cursor, selected(e), true)?,
                None => app::read_response_with_battle(&mut cursor, 47, true)?,
            };
            assert_eq!(envelope.request_id, u32_at(q, 8));
            let app::Response::BattleData(r) = envelope.response else {
                panic!("READ kind");
            };
            assert_eq!(read::encode_request(u32_at(q, 8), &r.echo).unwrap(), q);
            r.data.unwrap()
        }
        64 => {
            match ext {
                Some(e) => operation::read_reply_selected(&mut cursor, guard(q), q, selected(e))?,
                None => operation::read_reply(&mut cursor, guard(q), q, 47)?,
            }
            .data
        }
        66 => family!(world).data.unwrap(),
        68 => family!(revival).data,
        70 => family!(gear).data,
        72 => family!(greater).data,
        74 => family!(haste).data,
        76 => family!(slow).data,
        _ => panic!("unexpected request kind {kind}"),
    };
    assert_eq!(cursor.position(), b.len() as u64);
    Ok(d)
}
#[test]
#[ignore = "explicit Java selected reply codecs for eight families; real durable SET, no TLS/UI claim"]
fn java_healing_selected_all_reply_families_preserve_complete_data() {
    let mut peer = Peer::start_named(Some("healing7-replies"));
    let mut kinds = BTreeMap::new();
    let mut total = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:79");
            break;
        }
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields[0], "REPLY");
        let label = fields[1];
        let q = unhex(fields[2]);
        let b = unhex(fields[3]);
        let kind = u16::from_be_bytes(q[12..14].try_into().unwrap());
        assert_eq!(u32_at(&b, 4) as usize, b.len());
        assert_eq!(u32_at(&b, 8), u32_at(&q, 8));
        assert_eq!(u16::from_be_bytes(b[12..14].try_into().unwrap()), kind + 1);
        let d = decode(kind, &b, &q, Some(1)).unwrap_or_else(|_| panic!("selected decode {label}"));
        if kind == 60 {
            let mut blocked = Cursor::new(&b);
            assert!(app::read_response_selected(&mut blocked, selected(3), false).is_err());
            assert_eq!(blocked.position(), 16, "READ gate before body allocation");
            let mut oversized = b[..16].to_vec();
            oversized[4..8].copy_from_slice(&u32::MAX.to_be_bytes());
            let mut blocked = Cursor::new(oversized);
            assert!(app::read_response_selected(&mut blocked, selected(3), true).is_err());
            assert_eq!(blocked.position(), 16, "size gate before body allocation");
            let mut coalesced = b.clone();
            coalesced.extend_from_slice(&b);
            let mut frames = Cursor::new(coalesced);
            for n in 1..=2 {
                let r = app::read_response_selected(&mut frames, selected(3), true).unwrap();
                assert_eq!(r.request_id, u32_at(&q, 8));
                assert_eq!(frames.position(), (n * b.len()) as u64, "exactly one frame");
            }
        }
        assert_eq!(decode(kind, &b, &q, Some(3)).unwrap(), d);
        assert!(
            decode(kind, &b, &q, Some(0)).is_err(),
            "zero extension {label}"
        );
        assert!(decode(kind, &b, &q, None).is_err(), "legacy {label}");
        let raw = payload::encode(&d).unwrap();
        assert!(b.ends_with(&raw), "no projection {label}");
        let offset = b.len() - raw.len();
        // Equipment has an independent durable receipt between its fixed
        // metadata and the battle payload; lengths stay in the fixed prefix.
        let length_at = if kind == 70 {
            gear::MIN_RESULT_BYTES - 4
        } else {
            offset - 4
        };
        assert_eq!(u32_at(&b, length_at) as usize, raw.len());
        let mut f = Follower::new_selected(request(&d), selected(3)).unwrap();
        f.accept(read::UnvalidatedReply {
            echo: request(&d),
            state: read::State::Data,
            data: Some(d.clone()),
        })
        .unwrap();
        assert_eq!(f.observation().unwrap().data(), &d);
        for n in [0, 15, 16, 71, b.len() - 1] {
            assert!(
                decode(kind, &b[..n], &q, Some(3)).is_err(),
                "truncated {label}/{n}"
            );
        }
        let mut bad = b.clone();
        bad[offset + 4..offset + 6].copy_from_slice(&6u16.to_be_bytes());
        assert!(
            decode(kind, &bad, &q, Some(3)).is_err(),
            "no schema relabel {label}"
        );
        if kind != 60 {
            for at in [8, 16, 28, 40] {
                let mut bad = b.clone();
                bad[at] ^= 1;
                assert!(
                    decode(kind, &bad, &q, Some(3)).is_err(),
                    "echo {label}/{at}"
                );
            }
            let mut bad = b.clone();
            bad[4..8].copy_from_slice(&u32::MAX.to_be_bytes());
            assert!(
                decode(kind, &bad, &q, Some(3)).is_err(),
                "bounded header {label}"
            );
        }
        if label == "equipment-durable" {
            assert_eq!(d.personal.equipment.as_ref().unwrap().revision, 1);
        }
        *kinds.entry(kind).or_insert(0) += 1;
        total += 1;
        peer.ack(&b);
    }
    peer.finish();
    assert_eq!(total, 79);
    assert_eq!(
        kinds,
        BTreeMap::from([
            (60, 36),
            (64, 1),
            (66, 36),
            (68, 2),
            (70, 1),
            (72, 1),
            (74, 1),
            (76, 1)
        ])
    );
}
