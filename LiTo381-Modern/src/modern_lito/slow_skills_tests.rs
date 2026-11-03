use super::*;

// Literal protocol body, independent of the production decoder.
fn wire() -> Vec<u8> {
    let mut b = app::header(19, SKILLS_RESULT_KIND, 73).unwrap();
    b.extend(7i32.to_be_bytes());
    b.extend(3333i64.to_be_bytes());
    b.extend(u64::MAX.to_be_bytes());
    b.extend([0, 0, 0, 0, 0, 1, 0, 0]);
    b.extend(29u32.to_be_bytes());
    b.extend([9, 0, 0, 0]);
    b.extend(20i64.to_be_bytes());
    b.extend(500i64.to_be_bytes());
    b.extend([0, 3, b's', b'l', b'w']);
    b
}
fn catalog(b: &[u8], version: u16) -> SkillsReply {
    let app::Response::Skills(s) = app::decode_response_version(b, version).unwrap().response
    else {
        panic!("skills");
    };
    s
}
#[test]
fn qualified_slow_catalog_requires_v47_and_preserves_raw_template_costs() {
    let b = wire();
    let s = catalog(&b, 47);
    assert_eq!(s.context.key.source_id, 7);
    assert_eq!(s.context.key.object_id, 3333);
    assert_eq!(s.context.epoch, u64::MAX);
    assert_eq!(
        s.slow(),
        Some(&Skill {
            skill_id: 29,
            support: Support::Slow,
            mp_cost: 20,
            reuse_millis: 500,
            name: "slw".into()
        })
    );
    for version in 4..47 {
        assert!(app::decode_response_version(&b, version).is_err());
        let mut unsupported = b.clone();
        unsupported[48] = 1;
        assert!(catalog(&unsupported, version).slow().is_none());
        unsupported[48] = 2;
        assert!(app::decode_response_version(&unsupported, version).is_err());
    }
    let mut unavailable = b;
    unavailable[48] = 2;
    assert!(catalog(&unavailable, 47).slow().is_none());
}
#[test]
fn malformed_skill_rows_cannot_grant_slow_or_relabel_other_spells() {
    for n in 0..wire().len() {
        assert!(app::decode_response_version(&wire()[..n], 47).is_err());
    }
    for (at, width, value) in [(44, 4, 43u64), (52, 8, 19), (60, 8, 499)] {
        let mut b = wire();
        b[at..at + width].copy_from_slice(&value.to_be_bytes()[8 - width..]);
        assert!(app::decode_response_version(&b, 47).is_err());
    }
    for at in [49, 50, 51] {
        let mut b = wire();
        b[at] = 1;
        assert!(app::decode_response_version(&b, 47).is_err());
    }
}
