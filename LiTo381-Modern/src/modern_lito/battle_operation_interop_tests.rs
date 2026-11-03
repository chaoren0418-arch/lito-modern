//! Real WorldService -> v38 bytes -> Rust validation. Durable internal SET,
//! mixed-price histories and SAVED writer receipt, not a network SET or SQL test.
use super::*;
use crate::modern_lito::{
    application_protocol as app, battle_action, battle_operation as op, battle_window::Stamp,
    world_protocol,
};

fn unhex(text: &str, max: usize) -> Vec<u8> {
    assert!(text.len() % 2 == 0 && text.len() <= max * 2);
    text.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
#[test]
#[ignore = "explicit Java v38 canonical operation peer after durable gear; no legacy conversion, network SET, SQL or HWND"]
fn java_v38_operation_mixed_equipment_history_and_saved_receipt() {
    let mut peer = Peer::start_named(Some("operations"));
    let mut replies = Vec::new();
    for label in ["mixed-move", "mixed-no-mp", "mixed-saved"] {
        let line = peer.line();
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "OPERATION");
        assert_eq!(fields[1], label);
        let expected = unhex(fields[2], battle_action::MAX_REQUEST_BYTES);
        let bytes = unhex(fields[3], op::MAX_RESULT_BYTES);
        let guard = battle_action::Guard {
            stamp: Stamp {
                context: app::SelectionContext {
                    key: app::CharacterKey {
                        source_id: i32_at(&expected, 16) as u32,
                        object_id: i64_at(&expected, 20) as u64,
                    },
                    epoch: i64_at(&expected, 28) as u64,
                },
                channel: i32_at(&expected, 36).try_into().unwrap(),
                world: [
                    i64_at(&expected, 40),
                    i64_at(&expected, 48),
                    i64_at(&expected, 56),
                ],
                entity: i64_at(&expected, 64),
                life: i64_at(&expected, 72),
                revision: i64_at(&expected, 80),
            },
            target: None,
        };
        assert_eq!(op::encode(guard, &expected[116..], 38).unwrap(), expected);
        let reply = op::read_reply(&mut bytes.as_slice(), guard, &expected, 38).unwrap();
        assert_eq!(reply.data.personal.equipment.as_ref().unwrap().revision, 1);
        assert!(reply.data.presentation.is_some());
        let rebuilt = encode(&reply.data).unwrap();
        assert_eq!(&bytes[expected.len() + op::TRAILER_BYTES..], rebuilt);
        peer.ack(&bytes);
        replies.push(reply);
    }
    assert_eq!(peer.line(), "CLOSED:3");
    peer.finish();
    assert_eq!(replies[0].disposition, op::Disposition::Applied);
    assert_eq!(replies[0].reason, op::Reason::None);
    assert_eq!(replies[0].lifecycle.phase, world_protocol::Phase::Active);
    assert_eq!(replies[0].lifecycle.settlement, None);
    assert_eq!(replies[1].disposition, op::Disposition::Denied);
    assert_eq!(replies[1].reason, op::Reason::NoMp);
    assert_eq!(replies[1].rejection, battle_action::Rejection::None);
    assert_eq!(replies[2].disposition, op::Disposition::Rejected);
    assert_eq!(replies[2].rejection, battle_action::Rejection::NotActive);
    assert_eq!(replies[2].lifecycle.phase, world_protocol::Phase::Finished);
    assert_eq!(
        replies[2].lifecycle.settlement,
        Some(world_protocol::SettlementState::Saved)
    );
    assert!(replies[2].data.result.is_some());
    assert!(replies[2].lifecycle.revision > replies[2].data.world.revision + 1);
    assert!(
        replies[0].data.result.is_none(),
        "later SAVED cannot rewrite earlier active receipt"
    );
}
