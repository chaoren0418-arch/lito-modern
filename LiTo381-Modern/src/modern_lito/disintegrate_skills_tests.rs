use super::*;

// Independent literal also asserted by Java; not produced by this decoder.
const GOLDEN: &str = "4c544d3100000049fedcba9800190000000000070000000000000d05ffffffffffffffff00000000000100000000004d06000000000000000000004600000000000013880003646973";
fn bytes() -> Vec<u8> {
    (0..GOLDEN.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&GOLDEN[i..i + 2], 16).unwrap())
        .collect()
}
fn catalog(bytes: &[u8], version: u16) -> SkillsReply {
    let app::Response::Skills(s) = app::decode_response_version(bytes, version)
        .unwrap()
        .response
    else {
        panic!("skills")
    };
    s
}
#[test]
fn v27_disintegrate_catalog_literal_and_all_legacy_versions_are_unambiguous() {
    let raw = bytes();
    let s = catalog(&raw, 27);
    assert_eq!(s.context.key.source_id, 7);
    assert_eq!(s.context.key.object_id, 3333);
    assert_eq!(s.context.epoch, u64::MAX);
    assert_eq!(
        s.skills,
        vec![Skill {
            skill_id: 77,
            support: Support::Disintegrate,
            mp_cost: 70,
            reuse_millis: 5000,
            name: "dis".into()
        }]
    );
    assert!(s.disintegrate().is_some());
    for version in 1..27 {
        assert!(app::decode_response_version(&raw, version).is_err());
        let mut old = raw.clone();
        old[48] = 1;
        if version < 4 {
            assert!(app::decode_response_version(&old, version).is_err());
            continue;
        }
        let s = catalog(&old, version);
        assert!(s.disintegrate().is_none());
        assert_eq!(s.skills[0].support, Support::Unimplemented);
        old[48] = 2;
        assert!(app::decode_response_version(&old, version).is_err());
    }
    let mut unsupported = raw;
    unsupported[48] = 2;
    assert!(catalog(&unsupported, 27).disintegrate().is_none());
}
#[test]
fn catalog_support_cannot_hide_invalid_id_cost_reuse_or_use_legacy_cast() {
    for (offset, width, value) in [(44, 4, 70u64), (52, 8, 69), (60, 8, 4999)] {
        let mut bad = bytes();
        bad[offset..offset + width].copy_from_slice(&value.to_be_bytes()[8 - width..]);
        assert!(app::decode_response_version(&bad, 27).is_err());
    }
    let s = catalog(&bytes(), 27);
    let legacy = CastRequest {
        context: s.context,
        channel_id: 0,
        match_id: 1,
        generation: 1,
        sequence: 1,
        skill_id: 77,
        target_entity_id: 2,
    };
    assert!(legacy.validate().is_err());
    assert!(encode_cast_version(1, &legacy, 27).is_err());
    assert!(SkillsState::NotRequested.disintegrate().is_none());
    assert!(SkillsState::Unsupported.disintegrate().is_none());
}
