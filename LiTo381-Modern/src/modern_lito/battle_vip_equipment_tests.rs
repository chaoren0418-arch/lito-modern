use super::*;
use crate::modern_lito::protocol;
use std::io::Cursor;
fn selected(ext: u32) -> Selected {
    Selected::new(
        protocol::Ack {
            request_id: 1,
            version: 47,
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES,
        },
        ext,
    )
    .unwrap()
}
fn request() -> Request {
    Request {
        stamp: Stamp {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: 0xfedcba98,
            },
            channel: 3,
            world: [1, 2, 3],
            entity: 1,
            life: 2,
            revision: 50,
        },
        versions: [3, 9, 20],
        action: Action::Inspect,
        item: 0,
        desired: false,
        token: [0; 32],
    }
}
#[test]
fn full_vip_equipment_requests_pin_all_three_versions_and_exact_bytes() {
    let p = selected(48);
    let q = request();
    let b = encode_request(0xfedcba98, &q, p).unwrap();
    assert_eq!(b.len(), REQUEST_BYTES);
    assert_eq!(decode_request(&b, p).unwrap(), q);
    assert_eq!(i64_at(&b, 88), 3);
    assert_eq!(i64_at(&b, 96), 9);
    assert_eq!(i64_at(&b, 104), 20);
    let set = Request {
        action: Action::Set,
        item: i64::MAX,
        desired: true,
        token: [17; 32],
        ..q
    };
    assert_eq!(
        decode_request(&encode_request(1, &set, p).unwrap(), p).unwrap(),
        set
    );
    assert_eq!(format!("{set:?}"), "VipEquipmentIntent[redacted]");
}
#[test]
fn vip_data_only_and_legacy_selection_never_grant_equipment_commands() {
    let q = request();
    let b = encode_request(1, &q, selected(48)).unwrap();
    for ext in [0, 1, 3, 4, 7, 12, 15, 16, 17, 28, 29, 31] {
        let p = selected(ext);
        assert!(!p.allows_vip_equipment());
        assert!(encode_request(1, &q, p).is_err());
        assert!(decode_request(&b, p).is_err());
        let mut input = Cursor::new(vec![0; RESULT_PREFIX_BYTES]);
        assert!(read_reply(&mut input, &b, p).is_err());
        assert_eq!(input.position(), 0);
    }
    for ext in [48, 49, 60, 61, 63] {
        assert!(selected(ext).allows_vip_equipment());
    }
    for ext in [32, 64, u32::MAX] {
        assert!(Selected::new(selected(0).base(), ext).is_err());
    }
    assert_eq!(
        [
            protocol::extensions::HEALING.required,
            protocol::extensions::ARMOR.required,
            protocol::extensions::HEALING_ARMOR_PRESENTATION.required
        ],
        [3, 4, 15]
    );
}
#[test]
fn malformed_inspection_cannot_hide_set_fields_or_invalid_versions() {
    let p = selected(48);
    let q = request();
    let b = encode_request(1, &q, p).unwrap();
    for len in 0..b.len() {
        assert!(decode_request(&b[..len], p).is_err());
    }
    for at in [14, 15, 112, 113, 114, 115, 123, 124] {
        let mut bad = b.clone();
        bad[at] = 2;
        assert!(decode_request(&bad, p).is_err(), "offset {at}");
    }
    for versions in [[-1, 9, 20], [10, 9, 20], [3, 9, -1]] {
        assert!(encode_request(1, &Request { versions, ..q }, p).is_err());
    }
    assert!(encode_request(0, &q, p).is_err());
    assert!(encode_request(
        1,
        &Request {
            action: Action::Set,
            ..q
        },
        p
    )
    .is_err());
}
#[test]
fn oversized_or_foreign_header_is_rejected_before_allocating_body() {
    let p = selected(48);
    let b = encode_request(1, &request(), p).unwrap();
    for size in [
        0,
        15,
        RESULT_PREFIX_BYTES - 1,
        MAX_RESULT_BYTES + 1,
        u32::MAX as usize,
    ] {
        let mut h = vec![0; 32];
        h[..4].copy_from_slice(b"LTM1");
        h[4..8].copy_from_slice(&(size as u32).to_be_bytes());
        h[8..12].copy_from_slice(&1u32.to_be_bytes());
        h[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
        let mut input = Cursor::new(h);
        assert!(read_reply(&mut input, &b, p).is_err());
        assert_eq!(input.position(), 16);
    }
}
