use super::*;

fn fixture() -> Candidate {
    let mut data = super::super::data_tests::fixture();
    data.presentation = Some(List(
        data.actors
            .0
            .iter()
            .map(|a| PublicPresentation {
                entity: a.entity,
                shot: None,
                item: None,
            })
            .collect(),
    ));
    Candidate { data, motion: None }
}
fn full_state() -> State {
    State {
        world: fixture().data.world.identity,
        revision: 1000,
        elapsed_millis: 100,
        actors: List(
            (1..=128)
                .map(|serial| Actor {
                    serial,
                    life: 1,
                    walk_millis: 100,
                    attack_millis: 750,
                    last_walk: Some(Walk {
                        sequence: 1,
                        life: 1,
                        roster_revision: 999,
                        elapsed_millis: 100,
                        duration_millis: 100,
                        from: Cell { x: 10, y: 10 },
                        to: Cell { x: 11, y: 10 },
                        heading: 2,
                    }),
                })
                .collect(),
        ),
    }
}
#[test]
fn explicit_motion_roundtrip_requires_v43_and_rejects_legacy_or_missing_extension() {
    let p = fixture();
    let old = super::super::encode(&p.data).unwrap();
    let bytes = encode(&p).unwrap();
    assert_eq!(decode(&bytes).unwrap(), p);
    assert_eq!(&bytes[16..old.len()], &old[16..]);
    assert_eq!(&bytes[old.len()..], &[0, 14, 0, 0, 0, 0, 0, 1, 0]);
    assert!(super::super::inspect(&bytes).is_err());
    assert!(super::super::decode(&bytes).is_err());
    assert!(super::super::decode_legacy(&bytes).is_err());
    assert_eq!(
        super::super::decode_versioned(&bytes, 44).unwrap(),
        super::super::decode_versioned(&bytes, 43).unwrap()
    );
    assert_eq!(
        super::super::decode_versioned(&bytes, 45).unwrap(),
        super::super::decode_versioned(&bytes, 43).unwrap()
    );
    for version in [
        0,
        35,
        36,
        37,
        41,
        42,
        crate::modern_lito::protocol::MAX_VERSION + 1,
        u16::MAX,
    ] {
        assert!(super::super::decode_versioned(&bytes, version).is_err());
    }
    assert!(super::super::decode_versioned(&bytes, 43)
        .unwrap()
        .npc_motion
        .is_some());
    assert!(super::super::decode_versioned(&old, 43).is_err());
    assert!(decode(&old).is_err());
    let mut no_presentation = p;
    no_presentation.data.presentation = None;
    assert!(encode(&no_presentation).is_err());
}
#[test]
fn motion_container_rejects_every_truncation_trailing_reserved_duplicate_and_bad_presence() {
    let mut p = fixture();
    p.motion = Some(full_state());
    let bytes = encode(&p).unwrap();
    for end in 0..bytes.len() {
        assert!(decode(&bytes[..end]).is_err(), "{end}");
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    let n = trailing.len() as u32;
    trailing[8..12].copy_from_slice(&n.to_be_bytes());
    assert!(decode(&trailing).is_err());
    let start = super::super::encode(&p.data).unwrap().len();
    for index in [4, 6, 7, 12, 14, 15, start, start + 1, start + 2, start + 3] {
        let mut bad = bytes.clone();
        bad[index] ^= 1;
        assert!(decode(&bad).is_err(), "{index}");
    }
    for len in [u32::MAX, (LIMIT + 1) as u32, 0] {
        let mut bad = bytes.clone();
        bad[start + 4..start + 8].copy_from_slice(&len.to_be_bytes());
        assert!(decode(&bad).is_err());
    }
    let mut bad = bytes;
    bad[start + 8] = 2;
    assert!(decode(&bad).is_err());
}
#[test]
fn complete_128_motion_records_have_exact_independent_section_bound() {
    let mut p = fixture();
    p.motion = Some(full_state());
    let old_len = super::super::encode(&p.data).unwrap().len();
    let bytes = encode(&p).unwrap();
    assert_eq!(bytes.len() - old_len - 8, LIMIT);
    assert_eq!(LIMIT, 10_413);
    assert_eq!(decode(&bytes).unwrap(), p);
    let state = p.motion.as_mut().unwrap();
    state.actors.0.push(state.actors.0[0].clone());
    assert!(encode(&p).is_err());
    for count in [-1i32, 129, i32::MAX] {
        let mut bad = bytes.clone();
        let at = old_len + 8 + 1 + 24 + 8 + 8;
        bad[at..at + 4].copy_from_slice(&count.to_be_bytes());
        assert!(decode(&bad).is_err());
    }
}
#[test]
fn candidate_requires_complete_npc_presence_and_never_supplies_default_motion() {
    let mut p = fixture();
    assert!(p.check_motion().is_ok());
    p.motion = Some(full_state());
    assert!(p.check_motion().is_err());
    p.motion = None;
    p.data.npcs = Some(Npcs {
        revision: 1,
        elapsed_millis: 0,
        spawned: 0,
        closed: false,
        actors: List(vec![]),
        exchanges: List(vec![]),
        graphics: List(vec![]),
        names: List(vec![]),
    });
    assert!(p.check_motion().is_err());
    let mut state = full_state();
    state.revision = 1;
    state.elapsed_millis = 0;
    state.actors.0.clear();
    p.motion = Some(state);
    assert!(p.check_motion().is_ok());
    // These are syntax-level fixtures; complete scene admission is still a separate requirement.
}
