use super::*;

#[path = "npc_disintegrate_wire_golden.rs"]
mod golden;

pub(super) fn bytes(s: &str) -> Vec<u8> {
    assert_eq!(s.len() % 2, 0);
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn scope() -> Scope {
    Scope {
        room_id: 11,
        match_id: 12,
        generation: 13,
    }
}
fn vector(name: &str) -> (Scope, Vec<State>, Vec<u8>) {
    let (_, r, m, g, count, data) = golden::VECTORS.iter().find(|(n, ..)| *n == name).unwrap();
    let scope = Scope {
        room_id: *r,
        match_id: *m,
        generation: *g,
    };
    let data = bytes(data);
    let (states, used) = decode_states(&data, scope, (*count).max(1), VERSION).unwrap();
    assert_eq!(used, data.len());
    (scope, states, data)
}

#[test]
fn java_request_has_exact_i64_identity_unsigned_epoch_and_independent_lives() {
    let data = bytes(golden::REQUEST);
    let r = decode_request(&data, VERSION).unwrap();
    assert_eq!(data.len(), 80);
    assert_eq!(r.context.key.source_id, 7);
    assert_eq!(r.context.key.object_id, 9_007_199_254_740_993);
    assert_eq!(r.context.epoch, u64::MAX);
    assert_eq!(r.channel_id, 3);
    assert_eq!(r.target.scope, scope());
    assert_eq!(r.target.serial, 23);
    assert_eq!((r.sequence, r.caster_life, r.target_life), (17, 19, 29));
    assert_eq!(encode_request(&r, VERSION).unwrap(), data);
    for n in 0..data.len() {
        assert!(decode_request(&data[..n], VERSION).is_err());
    }
    let mut trailing = data.clone();
    trailing.push(0);
    assert!(decode_request(&trailing, VERSION).is_err());
    for offset in [21, 22, 23] {
        let mut bad = data.clone();
        bad[offset] = 1;
        assert!(decode_request(&bad, VERSION).is_err());
    }
    for offset in [4, 12, 24, 32, 40, 48, 56, 64, 72] {
        let mut bad = data.clone();
        bad[offset..offset + 8].fill(0);
        assert!(decode_request(&bad, VERSION).is_err());
    }
    let mut source = data.clone();
    source[..4].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(decode_request(&source, VERSION).is_err());
    let mut invalid = r;
    invalid.target.serial = 0;
    assert!(encode_request(&invalid, VERSION).is_err());
    invalid = r;
    invalid.channel_id = 4;
    assert!(encode_request(&invalid, VERSION).is_err());
    invalid = r;
    invalid.caster_life = i64::MAX as u64 + 1;
    assert!(encode_request(&invalid, VERSION).is_err());
    for version in [
        0,
        1,
        24,
        27,
        28,
        crate::modern_lito::world_protocol::CODEC_MAX_VERSION + 1,
        u16::MAX,
    ] {
        assert!(encode_request(&r, version).is_err());
        assert!(decode_request(&data, version).is_err());
    }
    assert_eq!(
        super::super::protocol::RUNTIME_CAPABILITIES & super::super::protocol::CAP_NPC_DISINTEGRATE,
        0,
        "request and renderer admission remain pending"
    );
}

#[test]
fn every_java_vector_reencodes_byte_for_byte_including_real_engine_sections() {
    assert_eq!(golden::VECTORS.len(), 14);
    for (name, r, m, g, count, data) in golden::VECTORS {
        let data = bytes(data);
        let scope = Scope {
            room_id: *r,
            match_id: *m,
            generation: *g,
        };
        let (states, used) = decode_states(&data, scope, (*count).max(1), VERSION)
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(used, 8 + states.len() * 144, "{name}");
        assert_eq!(used, data.len());
        assert_eq!(
            encode_states(&states, scope, (*count).max(1), VERSION).unwrap(),
            data,
            "{name}"
        );
    }
}

#[test]
fn huge_hp_zero_damage_bot_sparse_and_critical_meanings_are_preserved() {
    let (_, initial, _) = vector("initial");
    assert_eq!(initial[0].initial_lawful, i64::MIN);
    assert_eq!(initial[0].mp_cost, 59);
    assert_eq!(initial[1], State::initial(2, 0, 0));
    let (_, sparse, _) = vector("sparse");
    assert_eq!(sparse[0], State::initial(100, 70, i64::MAX));
    let (_, huge, _) = vector("huge");
    let h = huge[0].hit.as_ref().unwrap();
    assert_eq!(h.hp_before, 9_007_199_254_740_993);
    assert_eq!(h.remaining_hp, 9_007_199_254_740_777);
    assert_eq!(h.effect(), 11746);
    assert_eq!(h.target.serial, huge[0].entity_id);
    let (_, death, _) = vector("critical-death");
    let h = death[0].hit.as_ref().unwrap();
    assert!(h.death && h.critical);
    assert_eq!(h.effect(), 11748);
    assert_eq!(
        (h.hp_before, h.raw_damage, h.applied_damage, h.remaining_hp),
        (1, 432, 1, 0)
    );
    let (_, zero, _) = vector("zero");
    let h = zero[0].hit.as_ref().unwrap();
    assert_eq!((h.hp_before, h.remaining_hp), (i64::MAX, i64::MAX));
    assert_eq!(h.applied_damage, 0);
    assert!(!h.death);
    assert_eq!(zero[0].mp_spent, 70);
    assert_eq!(zero[0].lawful, 501);
    let (_, quest, _) = vector("quest-sparse");
    assert_eq!(quest.len(), 1);
    assert_eq!(quest[0].entity_id, 2);
    assert_eq!(quest[0].hit.as_ref().unwrap().lane, 1);
    let (_, terminal, _) = vector("time-attack-finished");
    assert!(terminal[0].count >= 3);
    assert!(terminal[0].hit.as_ref().unwrap().death);
    assert_eq!(terminal[1].count, 0);
}

#[test]
fn every_truncated_section_and_noncanonical_empty_payload_is_rejected() {
    for name in [
        "initial",
        "huge",
        "critical-death",
        "engine-normal-hit",
        "quest-sparse",
    ] {
        let (scope, states, data) = vector(name);
        for n in 0..data.len() {
            assert!(
                decode_states(&data[..n], scope, states.len(), VERSION).is_err(),
                "{name}:{n}"
            );
        }
        for offset in [2, 3, 4, 5, 6, 7, 75, 76, 77, 78, 79] {
            let mut bad = data.clone();
            bad[offset] = 1;
            assert!(
                decode_states(&bad, scope, states.len(), VERSION).is_err(),
                "{name}:{offset}"
            );
        }
        let mut trailing = data.clone();
        trailing.extend([1, 2, 3]);
        let (actual, used) = decode_states(&trailing, scope, states.len(), VERSION).unwrap();
        assert_eq!(actual, states);
        assert_eq!(used, data.len());
    }
    let (s, states, data) = vector("sparse");
    for offset in [72, 74, 80, 87, 88, 95, 96, 103, 104, 119, 120, 151] {
        let mut bad = data.clone();
        bad[offset] = 1;
        assert!(decode_states(&bad, s, 1, VERSION).is_err(), "{offset}");
    }
    for count in [0, 2, 101, usize::MAX] {
        assert!(decode_states(&data, s, count, VERSION).is_err());
        assert!(encode_states(&states, s, count, VERSION).is_err());
    }
    let (_, _, hit) = vector("huge");
    for offset in [
        8, 16, 24, 32, 40, 48, 56, 64, 68, 72, 73, 74, 80, 88, 96, 120, 128, 136, 144,
    ] {
        let mut bad = hit.clone();
        let n = if (64..=74).contains(&offset) { 1 } else { 8 };
        bad[offset..offset + n].fill(255);
        assert!(
            decode_states(&bad, s, 1, VERSION).is_err(),
            "field {offset}"
        );
    }
}

#[test]
fn maximum_count_order_scope_versions_and_overflows_are_strict() {
    let states = (1..=100)
        .map(|i| State::initial(i, 0, 0))
        .collect::<Vec<_>>();
    let data = encode_states(&states, scope(), 100, VERSION).unwrap();
    assert_eq!(data.len(), 14408);
    assert_eq!(
        decode_states(&data, scope(), 100, VERSION).unwrap().0,
        states
    );
    let mut duplicate = states.clone();
    duplicate[99] = duplicate[98].clone();
    assert!(encode_states(&duplicate, scope(), 100, VERSION).is_err());
    let (_, hit, _) = vector("huge");
    let mut wrong = scope();
    wrong.generation += 1;
    assert!(encode_states(&hit, wrong, 1, VERSION).is_err());
    wrong = scope();
    wrong.room_id = 0;
    assert!(decode_states(&data, wrong, 100, VERSION).is_err());
    for version in [
        0,
        1,
        24,
        27,
        28,
        crate::modern_lito::world_protocol::CODEC_MAX_VERSION + 1,
        u16::MAX,
    ] {
        assert!(decode_states(&data, scope(), 100, version).is_err());
        assert!(encode_states(&states, scope(), 100, version).is_err());
    }
    for cost in [1, 58, 71, u32::MAX] {
        assert!(State::initial(1, cost, 500).validate().is_err());
    }
    let mut overflow = hit[0].clone();
    overflow.count = i64::MAX;
    overflow.last_sequence = i64::MAX as u64;
    assert!(overflow.validate().is_err());
    overflow = hit[0].clone();
    overflow.initial_lawful = i64::MAX;
    assert!(overflow.validate().is_err());
    assert!(State::initial(101, 70, 500).validate().is_err());
    assert!(State::initial(1, 0, 500).validate().is_err());
}

#[test]
fn actual_engine_initial_hit_mixed_arrow_recast_is_monotonic() {
    for prefix in ["engine-normal", "engine-critical"] {
        let (s, a, _) = vector(&format!("{prefix}-before"));
        let (_, b, _) = vector(&format!("{prefix}-hit"));
        let (_, c, _) = vector(&format!("{prefix}-second"));
        assert!(follows(s, &a, 1, s, &b, b[0].revision));
        assert!(follows(s, &b, b[0].revision, s, &c, c[0].revision));
        assert_eq!((b[0].count, b[0].mp_spent, b[0].lawful), (1, 70, 501));
        assert_eq!(
            (c[0].count, c[0].last_sequence, c[0].mp_spent, c[0].lawful),
            (2, 4, 140, 502)
        );
        assert_eq!(
            b[0].hit.as_ref().unwrap().critical,
            prefix == "engine-critical"
        );
        assert!(follows(s, &b, b[0].revision, s, &b, b[0].revision + 1));
        assert!(!follows(s, &c, c[0].revision, s, &b, c[0].revision + 1));
    }
}

#[test]
fn changed_history_scope_identity_initial_values_or_regressed_lives_do_not_follow() {
    let (s, a, _) = vector("huge");
    let r = a[0].revision;
    let changed = |edit: fn(&mut State)| {
        let mut b = a.clone();
        edit(&mut b[0]);
        assert!(!follows(s, &a, r, s, &b, r + 2));
    };
    changed(|b| b.entity_id = 2);
    changed(|b| b.mp_cost = 69);
    changed(|b| b.initial_lawful = 501);
    changed(|b| b.hit.as_mut().unwrap().critical = true);
    changed(|b| b.revision += 1);
    changed(|b| b.last_sequence += 1);
    changed(|b| b.hit.as_mut().unwrap().target.serial += 1);
    let mut wrong = s;
    wrong.match_id += 1;
    assert!(!follows(s, &a, r, wrong, &a, r));
    assert!(!follows(s, &a, r, s, &a, r - 1));
    assert!(!follows(s, &a, r, s, &[], r + 1));
    let mut b = a.clone();
    b[0].count += 1;
    b[0].revision += 2;
    b[0].last_sequence += 1;
    b[0].mp_spent += 70;
    b[0].lawful += 1;
    assert!(follows(s, &a, r, s, &b, r + 2));
    let mut life = b.clone();
    life[0].hit.as_mut().unwrap().caster_life -= 1;
    assert!(!follows(s, &a, r, s, &life, r + 2));
    life = b.clone();
    life[0].hit.as_mut().unwrap().target_life -= 1;
    assert!(!follows(s, &a, r, s, &life, r + 2));
    let mut skip = b.clone();
    skip[0].count += 1;
    skip[0].mp_spent += 70;
    skip[0].lawful += 1;
    assert!(!follows(s, &a, r, s, &skip, r + 2));
    skip[0].last_sequence += 1;
    assert!(follows(s, &a, r, s, &skip, r + 2));
    skip[0].revision = r + 1;
    assert!(!follows(s, &a, r, s, &skip, r + 1));
}
