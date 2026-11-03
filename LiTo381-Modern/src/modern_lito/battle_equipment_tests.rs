use super::*;
use std::{
    io::{Cursor, Read},
    sync::Arc,
};

fn request() -> Request {
    let context = app::SelectionContext {
        key: app::CharacterKey {
            source_id: 7,
            object_id: 3333,
        },
        epoch: 1_u64 << 63,
    };
    Request {
        stamp: Stamp {
            context,
            channel: 3,
            world: [11, 12, 13],
            entity: 1,
            life: 2,
            revision: 7,
        },
        equipment: equipment::Request {
            context,
            action: equipment::Action::Set,
            desired: false,
            item_id: 123,
            expected_revision: 4,
            expected_view_token: [0; 32],
        },
    }
}
#[test]
fn exact_wire_widths_opaque_epoch_and_nested_cas_roundtrip() {
    let q = request();
    let b = encode_request(9, &q, 41).unwrap();
    assert_eq!(b.len(), 160);
    assert_eq!(
        &b[..16],
        &[76, 84, 77, 49, 0, 0, 0, 160, 0, 0, 0, 9, 0, 70, 0, 0]
    );
    assert_eq!(&b[28..36], &(1_u64 << 63).to_be_bytes());
    assert_eq!(&b[100..108], &(1_u64 << 63).to_be_bytes());
    assert_eq!(&b[120..128], &4_i64.to_be_bytes());
    assert_eq!(expected_request(&b, 41).unwrap(), q);
    assert_eq!(format!("{q:?}"), "BattleEquipmentRequest[redacted]");
    for version in [0, 1, 37, 38, 39, 40, protocol::MAX_VERSION + 1, u16::MAX] {
        assert!(encode_request(9, &q, version).is_err());
    }
    assert!(encode_request(0, &q, 41).is_err());
    for n in 0..b.len() {
        assert!(expected_request(&b[..n], 41).is_err());
    }
    let mut extra = b.clone();
    extra.push(0);
    assert!(expected_request(&extra, 41).is_err());
    for at in [
        14, 36, 40, 48, 56, 64, 72, 80, 88, 92, 100, 108, 109, 110, 112, 120,
    ] {
        let mut bad = b.clone();
        bad[at] = 255;
        assert!(expected_request(&bad, 41).is_err(), "offset {at}");
    }
}
#[test]
fn request_requires_exact_context_world_life_and_set_intent() {
    for case in 0..9 {
        let mut q = request();
        match case {
            0 => q.stamp.context.epoch = 0,
            1 => q.stamp.channel = 4,
            2 => q.stamp.world[2] = 0,
            3 => q.stamp.entity = 101,
            4 => q.stamp.life = 0,
            5 => q.stamp.revision = 0,
            6 => q.equipment.context.key.source_id = 42,
            7 => q.equipment = equipment::Request::read(q.stamp.context),
            _ => q.equipment.expected_revision = -1,
        }
        assert!(q.validate().is_err());
    }
}
struct HeaderOnly(Cursor<Vec<u8>>);
impl Read for HeaderOnly {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        assert!(self.0.position() < 16, "invalid frame read past header");
        self.0.read(b)
    }
}
#[test]
fn rejects_wrong_header_before_body_allocation_or_read() {
    let expected = encode_request(9, &request(), 41).unwrap();
    for case in 0..6 {
        let mut header = expected[..16].to_vec();
        header[4..8].copy_from_slice(&204_u32.to_be_bytes());
        header[12..14].copy_from_slice(&71_u16.to_be_bytes());
        match case {
            0 => header[4..8].copy_from_slice(&203_u32.to_be_bytes()),
            1 => header[4..8].copy_from_slice(&u32::MAX.to_be_bytes()),
            2 => header[12] = 1,
            3 => header[11] = 8,
            4 => header[15] = 1,
            _ => header[0] = 0,
        }
        assert!(read_reply(&mut HeaderOnly(Cursor::new(header)), &expected, 41).is_err());
    }
}
fn ack() -> protocol::Ack {
    protocol::Ack {
        request_id: 1,
        version: 41,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    }
}
#[test]
fn requires_new_version_and_complete_equipment_world_capabilities() {
    let a = ack();
    assert!(negotiated(&a));
    assert!(!negotiated(&protocol::Ack { version: 40, ..a }));
    assert!(!negotiated(&protocol::Ack {
        version: protocol::MAX_VERSION + 1,
        ..a
    }));
    for flag in [
        protocol::CAP_EQUIPMENT,
        protocol::CAP_BATTLE_DATA,
        protocol::CAP_ACTION_LIFE,
    ] {
        assert!(!negotiated(&protocol::Ack {
            capabilities: a.capabilities & !flag,
            ..a
        }));
    }
    assert_eq!(protocol::RUNTIME_WORLD_VERSION, 37);
}
fn fixtures() -> std::path::PathBuf {
    let p = std::path::PathBuf::from(
        std::env::var_os("LITO_MODERN_TEST_SERVER_ROOT").expect("explicit server root"),
    );
    assert!(p.is_absolute());
    p.join("target/modern-equipment-test-fixtures")
}
fn fixture(root: &std::path::Path, name: &str) -> (Vec<u8>, Vec<u8>) {
    (
        std::fs::read(root.join(format!("{name}.request"))).unwrap(),
        std::fs::read(root.join(format!("{name}.reply"))).unwrap(),
    )
}
fn initial(root: &std::path::Path, name: &str) -> super::super::battle_state::Tracker {
    let (q, b) = fixture(root, name);
    let r = super::super::battle_world::read_reply(&mut b.as_slice(), &q, 41).unwrap();
    let mut tracker = super::super::battle_state::Tracker::default();
    tracker.accept_world(&r, ack()).unwrap();
    tracker
}
#[test]
#[ignore = "requires freshly generated Java v41 equipment fixtures; no TLS or HWND"]
fn java_v41_equipment_receipts_follow_atomically_without_inventing_live_success() {
    let root = fixtures();
    let mut tracker = initial(&root, "before");
    for (name, disposition, revision) in [
        ("applied", Disposition::Applied, 1),
        ("unchanged", Disposition::Unchanged, 1),
        ("conflict", Disposition::NotCommitted, 1),
        ("reequipped", Disposition::Applied, 2),
    ] {
        let (q, b) = fixture(&root, name);
        let r = read_reply(&mut b.as_slice(), &q, 41).unwrap();
        assert_eq!(r.disposition, disposition);
        assert!(reply_negotiated(&r, &ack()));
        assert_eq!(encode_request(u32_at(&q, 8), &r.echo, 41).unwrap(), q);
        let previous = tracker.snapshot().unwrap().clone();
        for case in 0..5 {
            let mut bad = r.clone();
            match case {
                0 => bad.echo.stamp.entity += 1,
                1 => bad.durable_revision += 1,
                2 => bad.echo.stamp.world[2] += 1,
                3 => bad.data.personal.equipment.as_mut().unwrap().source = 99,
                _ => bad.data.personal.equipment.as_mut().unwrap().revision += 1,
            }
            assert!(
                tracker.accept_equipment(&bad, ack()).is_err(),
                "{name} case {case}"
            );
            assert!(Arc::ptr_eq(&previous, tracker.snapshot().unwrap()));
        }
        tracker.accept_equipment(&r, ack()).unwrap();
        let o = tracker.snapshot().unwrap().observation().unwrap();
        assert_eq!(
            o.data().personal.equipment.as_ref().unwrap().revision,
            revision
        );
        if name != "conflict" {
            let next = Request::from_observation(
                o,
                r.equipment.as_ref().unwrap().snapshot.as_ref().unwrap(),
                1,
                true,
            )
            .unwrap();
            assert_eq!(next.equipment.expected_revision, revision);
            let mut stale = next;
            stale.equipment.expected_revision -= 1;
            assert!(!stale.current(o));
        }
        assert_eq!(
            previous
                .observation()
                .unwrap()
                .data()
                .personal
                .equipment
                .as_ref()
                .unwrap()
                .revision,
            if name == "applied" { 0 } else { 1 }
        );
    }
    let mut retired = initial(&root, "retired-before");
    let (q, b) = fixture(&root, "retired");
    let r = read_reply(&mut b.as_slice(), &q, 41).unwrap();
    assert_eq!(r.disposition, Disposition::PersistedRetired);
    assert_eq!(r.durable_revision, 1);
    assert!(r.equipment.as_ref().unwrap().snapshot.is_none());
    retired.accept_equipment(&r, ack()).unwrap();
    let s = retired.snapshot().unwrap();
    assert!(s.presentation_observation().is_none());
    assert_eq!(
        s.observation()
            .unwrap()
            .data()
            .personal
            .equipment
            .as_ref()
            .unwrap()
            .revision,
        0
    );
}
#[test]
#[ignore = "requires freshly generated Java v41 equipment fixtures; malformed bodies and nested source checks"]
fn java_v41_equipment_rejects_truncation_lengths_echo_and_nested_identity() {
    let root = fixtures();
    for name in [
        "applied",
        "unchanged",
        "conflict",
        "reequipped",
        "stale",
        "busy",
        "terminal",
        "retired",
        "honor-stale",
        "honor-dead",
        "siege-stale",
    ] {
        let (q, b) = fixture(&root, name);
        let r = read_reply(&mut b.as_slice(), &q, 41).unwrap();
        if name.starts_with("honor-") || name.starts_with("siege-") {
            let actor = r
                .data
                .actors
                .0
                .iter()
                .find(|a| a.entity == r.data.personal.entity)
                .unwrap();
            assert_eq!(super::super::battle_motion::life(actor), Some(2));
            assert_eq!(
                r.reason,
                if name == "honor-dead" {
                    Reason::Dead
                } else {
                    Reason::StaleLife
                }
            );
        }
        assert!(read_reply(&mut b.as_slice(), &q, 40).is_err());
        for n in [0, 15, 16, 159, 160, 203, 204, b.len() - 1] {
            assert!(read_reply(&mut &b[..n], &q, 41).is_err());
        }
        for at in [
            16, 28, 36, 40, 64, 72, 80, 88, 108, 128, 160, 161, 162, 196, 200,
        ] {
            let mut bad = b.clone();
            bad[at] = 255;
            assert!(
                read_reply(&mut bad.as_slice(), &q, 41).is_err(),
                "{name} offset {at}"
            );
        }
        if r.equipment.is_some() {
            for at in [204 + 8, 204 + 12, 204 + 14, 204 + 16, 204 + 28] {
                let mut bad = b.clone();
                bad[at] ^= 1;
                assert!(
                    read_reply(&mut bad.as_slice(), &q, 41).is_err(),
                    "{name} nested {at}"
                );
            }
        } else {
            assert_ne!(r.reason, Reason::None);
            assert_eq!(r.durable_revision, -1);
        }
    }
}
