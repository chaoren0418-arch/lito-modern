use super::*;
use std::{
    io::{Cursor, Read},
    sync::Arc,
};
fn request() -> Request {
    Request {
        stamp: Stamp {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: 1_u64 << 63,
            },
            channel: 3,
            world: [11, 12, 13],
            entity: 1,
            life: 2,
            revision: 7,
        },
        death_revision: 6,
    }
}
#[test]
fn independent_widths_keep_opaque_epoch_and_canonical_death_revision() {
    let q = request();
    let b = encode_request(9, &q, 40).unwrap();
    assert_eq!(b.len(), 96);
    assert_eq!(
        &b[..16],
        &[76, 84, 77, 49, 0, 0, 0, 96, 0, 0, 0, 9, 0, 68, 0, 0]
    );
    assert_eq!(&b[28..36], &(1_u64 << 63).to_be_bytes());
    assert_eq!(
        &b[40..64],
        &[
            11_i64.to_be_bytes(),
            12_i64.to_be_bytes(),
            13_i64.to_be_bytes()
        ]
        .concat()
    );
    assert_eq!(&b[88..], &6_i64.to_be_bytes());
    assert_eq!(expected_request(&b, 40).unwrap(), q);
    assert_eq!(format!("{q:?}"), "BattleRevivalRequest[redacted]");
    assert!(encode_request(0, &q, 40).is_err());
    for v in [0, 1, 37, 38, 39, protocol::MAX_VERSION + 1, u16::MAX] {
        assert!(encode_request(9, &q, v).is_err());
    }
}
#[test]
fn source_scope_life_revision_and_request_tail_are_strict() {
    for case in 0..9 {
        let mut q = request();
        match case {
            0 => q.stamp.context.key.source_id = 0,
            1 => q.stamp.context.key.object_id = 0,
            2 => q.stamp.context.epoch = 0,
            3 => q.stamp.channel = 4,
            4 => q.stamp.world[1] = 0,
            5 => q.stamp.entity = 101,
            6 => q.stamp.life = 0,
            7 => q.death_revision = 0,
            _ => q.death_revision = 8,
        }
        assert!(q.validate().is_err());
    }
    let b = encode_request(9, &request(), 40).unwrap();
    for n in 0..96 {
        assert!(expected_request(&b[..n], 40).is_err());
    }
    let mut extra = b;
    extra.push(0);
    assert!(expected_request(&extra, 40).is_err());
}
struct HeaderOnly(Cursor<Vec<u8>>);
impl Read for HeaderOnly {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        assert!(self.0.position() < 16, "bad frame read past its header");
        self.0.read(b)
    }
}
#[test]
fn invalid_response_kind_id_size_and_flags_reject_before_body_allocation() {
    let expected = encode_request(9, &request(), 40).unwrap();
    for case in 0..5 {
        let mut header = expected[..16].to_vec();
        header[4..8].copy_from_slice(&128_u32.to_be_bytes());
        header[12..14].copy_from_slice(&69_u16.to_be_bytes());
        match case {
            0 => header[4..8].copy_from_slice(&127_u32.to_be_bytes()),
            1 => header[4..8].copy_from_slice(&u32::MAX.to_be_bytes()),
            2 => header[12] = 1,
            3 => header[11] = 8,
            _ => header[15] = 1,
        }
        assert!(read_reply(&mut HeaderOnly(Cursor::new(header)), &expected, 40).is_err());
    }
}
#[test]
fn negotiation_requires_v40_and_the_complete_existing_canonical_capabilities() {
    let ack = protocol::Ack {
        request_id: 1,
        version: 40,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    };
    assert!(negotiated(&ack));
    assert!(!negotiated(&protocol::Ack { version: 39, ..ack }));
    for flag in [protocol::CAP_BATTLE_DATA, protocol::CAP_ACTION_LIFE] {
        assert!(!negotiated(&protocol::Ack {
            capabilities: ack.capabilities & !flag,
            ..ack
        }));
    }
    assert_eq!(protocol::RUNTIME_WORLD_VERSION, 37);
}
#[test]
#[ignore = "requires freshly generated Java v40 canonical death-confirmation fixtures; no HWND"]
fn java_v40_revival_wire_and_atomic_follower_preserve_exact_death_and_new_life() {
    let server = std::path::PathBuf::from(
        std::env::var_os("LITO_MODERN_TEST_SERVER_ROOT").expect("explicit server root"),
    );
    assert!(server.is_absolute());
    let root = server.join("target/modern-revival-test-fixtures");
    let dead_request = std::fs::read(root.join("dead.request")).unwrap();
    let dead_bytes = std::fs::read(root.join("dead.reply")).unwrap();
    let dead =
        crate::modern_lito::battle_world::read_reply(&mut dead_bytes.as_slice(), &dead_request, 40)
            .unwrap();
    let ack = protocol::Ack {
        request_id: 1,
        version: 40,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    };
    let mut tracker = crate::modern_lito::battle_state::Tracker::default();
    tracker.accept_world(&dead, ack).unwrap();
    let observed =
        Request::from_observation(tracker.snapshot().unwrap().observation().unwrap()).unwrap();
    let mut old_life = observed.stamp.life;
    for (name, reason) in [
        ("revived", Reason::Revived),
        ("stale", Reason::StaleLife),
        ("not-dead", Reason::NotDead),
        ("aborted", Reason::NotActive),
    ] {
        let q = std::fs::read(root.join(format!("{name}.request"))).unwrap();
        let bytes = std::fs::read(root.join(format!("{name}.reply"))).unwrap();
        let r = read_reply(&mut bytes.as_slice(), &q, 40).unwrap();
        assert_eq!(r.reason, reason);
        assert!(reply_negotiated(&r, &ack));
        if name == "revived" {
            assert_eq!(r.echo, observed);
            old_life = r.echo.stamp.life;
        }
        for n in [0, 15, 16, 95, 96, 127, 128, bytes.len() - 1] {
            assert!(read_reply(&mut &bytes[..n], &q, 40).is_err());
        }
        let mut wrong = q.clone();
        wrong[35] ^= 1;
        assert!(read_reply(&mut bytes.as_slice(), &wrong, 40).is_err());
        let mut wrong = bytes.clone();
        wrong[97] = 1;
        assert!(read_reply(&mut wrong.as_slice(), &q, 40).is_err());
        let mut wrong = bytes.clone();
        wrong[96] = 255;
        assert!(read_reply(&mut wrong.as_slice(), &q, 40).is_err());
        let previous = tracker.snapshot().unwrap().clone();
        let mut bad = r.clone();
        bad.echo.stamp.entity += 1;
        assert!(tracker.accept_revival(&bad, ack).is_err());
        assert!(Arc::ptr_eq(&previous, tracker.snapshot().unwrap()));
        tracker.accept_revival(&r, ack).unwrap();
        let snapshot = tracker.snapshot().unwrap();
        let o = snapshot.observation().unwrap();
        assert_eq!(Stamp::of(o).life, old_life + 1);
        assert!(Request::from_observation(o).is_none());
        if name == "aborted" {
            assert!(snapshot.presentation_observation().is_none());
        }
    }
    let q = std::fs::read(root.join("revived-retired.request")).unwrap();
    let bytes = std::fs::read(root.join("revived-retired.reply")).unwrap();
    let retired = read_reply(&mut bytes.as_slice(), &q, 40).unwrap();
    assert_eq!(retired.reason, Reason::Revived);
    assert_eq!(retired.lifecycle.phase, world::Phase::Aborted);
    assert!(read_reply(&mut bytes.as_slice(), &q, 39).is_err());
}
