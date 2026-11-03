use super::*;
const S: Scope = Scope {
    room_id: 11,
    match_id: 12,
    generation: 13,
};
const FROM: WorldCell = WorldCell { x: 32778, y: 32778 };
const TO: WorldCell = WorldCell { x: 32779, y: 32778 };
fn outcome(hp: i64, raw: i64) -> Outcome {
    let applied = hp.min(raw);
    Outcome {
        raw_damage: raw,
        applied_damage: applied,
        remaining_hp: hp - applied,
        death: hp == applied,
    }
}
fn target(serial: u64, hp: i64) -> Target {
    Target {
        id: Id { scope: S, serial },
        life: 1,
        lane: 0,
        cell: TO,
        hp_before: hp,
        outcome: outcome(hp, 35),
    }
}
fn structure(kind: u8, hp: i64) -> Structure {
    let id = if kind == 4 {
        StructureId::Tower(time_attack_protocol::TowerId { scope: S })
    } else {
        StructureId::Quest {
            id: quest_protocol::Id {
                scope: S,
                slot: if kind == 1 { 1 } else { 4 },
            },
            kind: match kind {
                1 => QuestKind::RoomSymbol,
                2 => QuestKind::FinalSymbol,
                _ => QuestKind::FinalGuardian,
            },
            born_revision: 3,
        }
    };
    Structure {
        count: 1,
        revision: 9,
        sequence: 5,
        caster_life: 1,
        origin: FROM,
        id,
        cell: WorldCell { x: 32778, y: 32779 },
        maximum_hp: hp,
        hp_before: hp,
        outcome: outcome(hp, 35),
    }
}
fn state(targets: Vec<Target>, structure: Option<Structure>) -> State {
    State {
        entity_id: 1,
        count: 1,
        revision: 9,
        last_sequence: 5,
        mp_spent: 48,
        mp_cost: 48,
        caster_life: 1,
        scope: Some(S),
        origin: Some(FROM),
        lane: Some(0),
        targets,
        structure,
    }
}
fn next(a: &State, targets: Vec<Target>, structure: Option<Structure>) -> State {
    State {
        count: 2,
        revision: 12,
        last_sequence: 8,
        mp_spent: 96,
        targets,
        structure,
        ..a.clone()
    }
}
pub(super) fn bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|n| u8::from_str_radix(&hex[n..n + 2], 16).unwrap())
        .collect()
}
fn roundtrip(states: &[State]) -> Vec<u8> {
    let b = encode_states(states, S, states.len().max(1), VERSION).unwrap();
    assert_eq!(
        encode_states(states, S, states.len().max(1), 32).unwrap(),
        b
    );
    let (v32, used32) = decode_states(&b, S, states.len().max(1), 32).unwrap();
    assert_eq!(v32, states);
    assert_eq!(used32, b.len());
    let (got, used) = decode_states(&b, S, states.len().max(1), VERSION).unwrap();
    assert_eq!(states, got);
    assert_eq!(b.len(), used);
    b
}
fn forward(a: &State, b: &State) -> bool {
    follows(
        S,
        std::slice::from_ref(a),
        9,
        S,
        std::slice::from_ref(b),
        12,
    )
}

#[test]
fn selector_request_golden_uses_exact_i64_object_and_opaque_epoch() {
    let r = Request {
        context: SelectionContext {
            key: app::CharacterKey {
                source_id: 7,
                object_id: 9007199254740993,
            },
            epoch: u64::MAX,
        },
        channel_id: 3,
        scope: S,
        sequence: 17,
        caster_life: 19,
    };
    let b = encode_request(&r, VERSION).unwrap();
    assert_eq!(encode_request(&r, 32).unwrap(), b);
    assert_eq!(decode_request(&b, 32).unwrap(), r);
    assert_eq!(b,bytes("000000070020000000000001ffffffffffffffff03000000000000000000000b000000000000000c000000000000000d00000000000000110000000000000013"));
    assert_eq!(r, decode_request(&b, VERSION).unwrap());
    for n in 0..b.len() {
        assert!(decode_request(&b[..n], VERSION).is_err());
    }
    let mut tail = b.clone();
    tail.push(0);
    assert!(decode_request(&tail, VERSION).is_err());
    for at in [21, 22, 23] {
        let mut bad = b.clone();
        bad[at] = 1;
        assert!(decode_request(&bad, VERSION).is_err());
    }
    for at in [4, 12, 24, 32, 40, 48, 56] {
        let mut bad = b.clone();
        bad[at..at + 8].fill(0);
        assert!(decode_request(&bad, VERSION).is_err());
    }
    for at in [4, 24, 32, 40, 48, 56] {
        let mut bad = b.clone();
        bad[at..at + 8].copy_from_slice(&u64::MAX.to_be_bytes());
        assert!(decode_request(&bad, VERSION).is_err());
    }
}
#[test]
fn initial_bot_sparse_and_hundred_actor_sections_are_canonical() {
    assert_eq!(roundtrip(&[]), [0; 8]);
    roundtrip(&[State::initial(1, 39), State::initial(2, 0)]);
    roundtrip(&[State::initial(100, 48)]);
    let many = (1..=100).map(|i| State::initial(i, 0)).collect::<Vec<_>>();
    assert_eq!(roundtrip(&many).len(), 7208);
}
#[test]
fn empty_zero_partial_death_huge_hp_and_all_structure_kinds() {
    roundtrip(&[state(vec![], None)]);
    for hp in [1, 100, 9007199254740993, i64::MAX] {
        roundtrip(&[state(vec![target(1, hp)], None)]);
    }
    let mut zero = target(1, i64::MAX);
    zero.outcome = outcome(i64::MAX, 0);
    roundtrip(&[state(vec![zero], None)]);
    for kind in 1..=4 {
        for hp in [1, 9007199254740993] {
            let a = state(vec![target(1, 100)], Some(structure(kind, hp)));
            assert_eq!(roundtrip(&[a.clone()]).len(), 248);
            assert!(a.current_structure().is_some());
            let b = next(&a, vec![target(1, 65)], a.structure.clone());
            roundtrip(&[b.clone()]);
            assert!(forward(&a, &b));
            assert!(b.current_structure().is_none());
        }
    }
}
#[test]
fn history_can_accompany_48_npc_hits_but_current_structure_counts_toward_limit() {
    let a = state(vec![], Some(structure(4, 1)));
    let hits = (1..=48).map(|i| target(i, 100)).collect::<Vec<_>>();
    assert_eq!(
        roundtrip(&[next(&a, hits.clone(), a.structure.clone())]).len(),
        8 + 72 + 48 * 64 + 104
    );
    assert!(state(hits, a.structure).validate().is_err());
}
#[test]
fn versions_scope_actor_counts_and_order_fail_closed() {
    assert_eq!(super::super::protocol::MAX_VERSION, 47);
    let a = state(vec![target(1, 100)], None);
    let b = roundtrip(&[a.clone()]);
    for v in [
        0,
        1,
        29,
        30,
        super::super::protocol::MAX_VERSION + 1,
        u16::MAX,
    ] {
        assert!(decode_states(&b, S, 1, v).is_err());
        assert!(encode_states(&[a.clone()], S, 1, v).is_err());
    }
    for n in [0, 2, 101] {
        assert!(decode_states(&b, S, n, VERSION).is_err());
        assert!(encode_states(&[a.clone()], S, n, VERSION).is_err());
    }
    assert!(encode_states(
        &[a.clone()],
        Scope {
            generation: 14,
            ..S
        },
        1,
        VERSION
    )
    .is_err());
    assert!(encode_states(&[a.clone(), a], S, 2, VERSION).is_err());
    assert!(encode_states(&[State::initial(101, 0)], S, 1, VERSION).is_err());
    assert!(decode_states(&b, Scope { room_id: 0, ..S }, 1, VERSION).is_err());
}
#[test]
fn all_truncations_reserved_flags_and_invalid_lengths_rejected() {
    let good = roundtrip(&[state(vec![target(1, 100)], Some(structure(4, 1)))]);
    for n in 0..good.len() {
        assert!(
            decode_states(&good[..n], S, 1, VERSION).is_err(),
            "truncated {n}"
        );
    }
    for at in [2, 7, 75, 76, 77, 78, 79, 137, 143, 187, 191] {
        let mut b = good.clone();
        b[at] = 1;
        assert!(decode_states(&b, S, 1, VERSION).is_err(), "reserved {at}");
    }
    for at in [74, 136, 186] {
        let mut b = good.clone();
        b[at] = 2;
        assert!(decode_states(&b, S, 1, VERSION).is_err(), "flag {at}");
    }
    for at in [73, 184, 185] {
        let mut b = good.clone();
        b[at] = 255;
        assert!(decode_states(&b, S, 1, VERSION).is_err());
    }
    let mut tail = good.clone();
    tail.extend(123u64.to_be_bytes());
    assert_eq!(decode_states(&tail, S, 1, VERSION).unwrap().1, good.len());
}
#[test]
fn initial_payload_cost_spend_overflow_and_noncanonical_tower_rejected() {
    let initial = roundtrip(&[State::initial(1, 48)]);
    for at in [24, 32, 40, 56, 64, 68, 72] {
        let mut b = initial.clone();
        b[at] = 1;
        assert!(decode_states(&b, S, 1, VERSION).is_err(), "initial {at}");
    }
    let mut overflow = roundtrip(&[state(vec![target(1, 100)], None)]);
    for at in [16, 32] {
        overflow[at..at + 8].copy_from_slice(&i64::MAX.to_be_bytes());
    }
    assert!(decode_states(&overflow, S, 1, VERSION).is_err());
    let tower = roundtrip(&[state(vec![target(1, 100)], Some(structure(4, 1)))]);
    let mut born = tower.clone();
    born[216..224].copy_from_slice(&1i64.to_be_bytes());
    assert!(decode_states(&born, S, 1, VERSION).is_err());
}
#[test]
fn follows_cannot_drop_invent_rewrite_or_replay_history() {
    let a = state(vec![target(1, 100)], Some(structure(4, 1)));
    let b = next(&a, vec![target(1, 65)], a.structure.clone());
    assert!(forward(&a, &b));
    assert!(!forward(&a, &next(&a, vec![], None)));
    assert!(!forward(&state(vec![], None), &b));
    assert!(!forward(&a, &next(&a, vec![], Some(structure(3, 1)))));
    assert!(!forward(
        &a,
        &state(vec![target(1, 99)], a.structure.clone())
    ));
    assert!(!follows(S, &[a.clone()], 9, S, &[b.clone()], 11));
    assert!(!follows(
        S,
        &[a],
        9,
        Scope {
            generation: 14,
            ..S
        },
        &[b],
        12
    ));
}
#[test]
fn same_npc_life_cannot_heal_or_resurrect_but_new_life_is_distinct() {
    let a = state(vec![target(1, 1)], None);
    assert!(!forward(&a, &next(&a, vec![target(1, 1)], None)));
    let mut born = target(1, 100);
    born.life = 2;
    assert!(forward(&a, &next(&a, vec![born], None)));
}
#[test]
fn later_structure_hit_must_not_heal_change_identity_or_follow_destruction() {
    let a = state(vec![], Some(structure(4, 100)));
    let mut proof = structure(4, 100);
    proof.count = 2;
    proof.revision = 12;
    proof.sequence = 8;
    proof.hp_before = 65;
    proof.outcome = outcome(65, 35);
    assert!(forward(&a, &next(&a, vec![], Some(proof.clone()))));
    assert!(!forward(
        &state(vec![], Some(structure(4, 1))),
        &next(&a, vec![], Some(proof.clone()))
    ));
    proof.hp_before = 66;
    proof.outcome = outcome(66, 35);
    assert!(!forward(&a, &next(&a, vec![], Some(proof))));
}
#[test]
fn quest_structure_stage_may_advance_but_cannot_switch_flow_or_reuse_old_birth() {
    let a = state(vec![], Some(structure(1, 1)));
    let mut e = structure(1, 1);
    e.count = 2;
    e.revision = 12;
    e.sequence = 8;
    e.id = StructureId::Quest {
        id: quest_protocol::Id { scope: S, slot: 2 },
        kind: QuestKind::RoomSymbol,
        born_revision: 10,
    };
    assert!(forward(&a, &next(&a, vec![], Some(e.clone()))));
    e.id = StructureId::Quest {
        id: quest_protocol::Id { scope: S, slot: 2 },
        kind: QuestKind::RoomSymbol,
        born_revision: 3,
    };
    assert!(!forward(&a, &next(&a, vec![], Some(e))));
}
#[test]
fn malformed_arbitrary_payloads_never_panic() {
    let base = roundtrip(&[state(
        vec![target(1, 9007199254740993)],
        Some(structure(1, 100)),
    )]);
    for at in 0..base.len() {
        for byte in [0, 1, 127, 128, 255] {
            let mut b = base.clone();
            b[at] = byte;
            let result = std::panic::catch_unwind(|| decode_states(&b, S, 1, VERSION));
            assert!(result.is_ok(), "panic at {at}:{byte}");
            if let Ok(Ok((states, used))) = result {
                assert!(used <= b.len());
                for s in &states {
                    s.validate().unwrap();
                }
            }
        }
    }
}

#[test]
fn maximum_section_is_bounded_but_needs_a_new_application_envelope() {
    let a = state(vec![], Some(structure(4, 1)));
    let b = next(
        &a,
        (1..=48).map(|i| target(i, 100)).collect(),
        a.structure.clone(),
    );
    let many = (1..=100)
        .map(|id| State {
            entity_id: id,
            ..b.clone()
        })
        .collect::<Vec<_>>();
    assert_eq!(MAX_SECTION_BYTES, 324808);
    assert_eq!(roundtrip(&many).len(), MAX_SECTION_BYTES);
    assert!(MAX_SECTION_BYTES > 256 * 1024);
}

#[test]
fn retained_structure_has_independent_sequence_and_revision_budgets() {
    let mut a = state(vec![], None);
    a.last_sequence = 100;
    for sequence in [false, true] {
        let mut proof = structure(4, 100);
        proof.count = 3;
        proof.revision = if sequence { 11 } else { 10 };
        proof.sequence = if sequence { 101 } else { 102 };
        let mut b = state(vec![], Some(proof));
        b.count = 4;
        b.revision = 20;
        b.last_sequence = 103;
        b.mp_spent = 192;
        b.validate().unwrap();
        assert!(!follows(S, &[a.clone()], 9, S, &[b], 20));
    }
    for sequence in [false, true] {
        let mut b = state(vec![], Some(structure(4, 100)));
        b.count = 5;
        b.revision = if sequence { 12 } else { 11 };
        b.last_sequence = if sequence { 6 } else { 9 };
        b.mp_spent = 240;
        assert!(b.validate().is_err());
    }
    let mut proof = structure(4, 100);
    proof.count = 10;
    proof.sequence = 10;
    assert!(proof.validate().is_err());
}
