use super::*;
const GOLDEN: &str = "4c544d3100000049fedcba9800190000000000070000000000000d05ffffffffffffffff00000000000100000000003607000000000000000000003c000000000000000a0003686173";
fn bytes() -> Vec<u8> {
    (0..GOLDEN.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&GOLDEN[i..i + 2], 16).unwrap())
        .collect()
}
fn catalog(b: &[u8], v: u16) -> SkillsReply {
    let app::Response::Skills(s) = app::decode_response_version(b, v).unwrap().response else {
        panic!("skills");
    };
    s
}
#[test]
fn v42_qualified_catalog_literal_and_legacy_rejection_preserve_the_raw_row() {
    let b = bytes();
    let s = catalog(&b, 42);
    assert_eq!(s.context.key.source_id, 7);
    assert_eq!(s.context.key.object_id, 3333);
    assert_eq!(s.context.epoch, u64::MAX);
    assert_eq!(
        s.greater_haste(),
        Some(&Skill {
            skill_id: 54,
            support: Support::GreaterHaste,
            mp_cost: 60,
            reuse_millis: 10,
            name: "has".into()
        })
    );
    for v in 1..42 {
        assert!(app::decode_response_version(&b, v).is_err());
        let mut old = b.clone();
        old[48] = 1;
        if v < 4 {
            assert!(app::decode_response_version(&old, v).is_err());
            continue;
        }
        assert!(catalog(&old, v).greater_haste().is_none());
        old[48] = 2;
        assert!(app::decode_response_version(&old, v).is_err());
    }
    let mut off = b;
    off[48] = 2;
    assert!(catalog(&off, 42).greater_haste().is_none());
}
#[test]
fn wrong_id_cost_and_delay_cannot_grant_greater_haste_or_relabel_a_legacy_spell() {
    for (offset, width, value) in [(44, 4, 77u64), (52, 8, 59), (60, 8, 9)] {
        let mut b = bytes();
        b[offset..offset + width].copy_from_slice(&value.to_be_bytes()[8 - width..]);
        assert!(app::decode_response_version(&b, 42).is_err());
    }
    let q = CastRequest {
        context: catalog(&bytes(), 42).context,
        channel_id: 0,
        match_id: 1,
        generation: 1,
        sequence: 1,
        skill_id: 54,
        target_entity_id: 0,
    };
    // The legacy self-cast codec deliberately carries unknown skill IDs for
    // an UNSUPPORTED server reply. Encoding it never creates a v42 intent.
    let legacy = encode_cast_version(1, &q, 42).unwrap();
    assert!(super::super::battle_greater_haste::decode_intent(&legacy, 42).is_err());
}
