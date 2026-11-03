use super::*;
use crate::modern_lito::{combat_protocol as combat, item_protocol as items};

fn sample() -> Snapshot {
    Snapshot {
        source_gear_weight: 94,
        supplies_weight: 6,
        total: 100,
        maximum_bits: 240f64.to_bits(),
        stage: 100,
        hit_penalty: 1,
        units: vec![UnitWeight {
            item_id: 40010,
            unit_weight: 2000,
        }],
    }
}
pub(crate) fn encode(s: &Snapshot) -> Vec<u8> {
    let mut out = Vec::new();
    for n in [s.source_gear_weight, s.supplies_weight, s.total] {
        out.extend(n.to_be_bytes());
    }
    out.extend(s.maximum_bits.to_be_bytes());
    out.extend(s.stage.to_be_bytes());
    out.extend([s.hit_penalty, 0]);
    out.extend((s.units.len() as u16).to_be_bytes());
    out.extend([0, 0]);
    for u in &s.units {
        out.extend(u.item_id.to_be_bytes());
        out.extend(u.unit_weight.to_be_bytes());
    }
    out
}
pub(crate) fn weighted_reply() -> combat::Reply {
    let mut reply = items::tests::initial();
    let frame = reply.frame.as_mut().unwrap();
    frame.weights = frame
        .items
        .iter()
        .map(|items| {
            let units: Vec<_> = items
                .stocks
                .iter()
                .map(|s| UnitWeight {
                    item_id: s.item_id,
                    unit_weight: 1000,
                })
                .collect();
            let supplies = items.stocks.iter().map(|s| s.remaining).sum();
            let source = if items.entity_id == 1 { 95 } else { 0 };
            let total = source + supplies;
            let stage = stage(total, 120.0).unwrap();
            ActorWeight {
                entity_id: items.entity_id,
                weight: Snapshot {
                    source_gear_weight: source,
                    supplies_weight: supplies,
                    total,
                    maximum_bits: 120f64.to_bits(),
                    stage,
                    hit_penalty: penalty(stage),
                    units,
                },
            }
        })
        .collect();
    reply.validate().unwrap();
    reply
}
#[test]
fn snapshot_has_independent_exact_binary64_40_byte_header_and_units_golden() {
    let literal = concat!(
        "000000000000005e",
        "0000000000000006",
        "0000000000000064",
        "406e000000000000",
        "00640100",
        "00010000",
        "0000000000009c4a",
        "00000000000007d0"
    );
    let bytes: Vec<u8> = literal
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect();
    assert_eq!(bytes.len(), 56);
    assert_eq!(encode(&sample()), bytes);
    assert_eq!(decode(&bytes).unwrap(), (sample(), 56));
    for n in 0..bytes.len() {
        assert!(decode(&bytes[..n]).is_err(), "truncated {n}");
    }
    for offset in [35, 38, 39] {
        let mut bad = bytes.clone();
        bad[offset] = 1;
        assert!(decode(&bad).is_err());
    }
}
#[test]
fn native_rounding_stage_and_penalty_edges_do_not_use_integer_percent() {
    for (raw, want) in [
        (0, 0),
        (1, 1),
        (2, 1),
        (3, 2),
        (4, 2),
        (480, 240),
        (481, 240),
    ] {
        assert_eq!(stage(raw, 480.0).unwrap(), want);
    }
    assert_eq!(stage(1, 480.0).unwrap(), 1); // exactly half rounds up
    for (s, p) in [
        (80, 0),
        (81, 1),
        (120, 1),
        (121, 3),
        (160, 3),
        (161, 5),
        (196, 5),
        (197, 5),
        (200, 5),
        (201, 0),
        (240, 0),
    ] {
        assert_eq!(penalty(s), p);
    }
    assert_eq!(stage(i64::MAX, 1.0).unwrap(), 240);
    assert_eq!(stage(i64::MAX, f64::MAX).unwrap(), 0);
}
#[test]
fn stack_math_floors_each_stack_and_handles_exact_i64_without_product_overflow() {
    assert_eq!(stack_weight(0, i64::MAX).unwrap(), 0);
    assert_eq!(stack_weight(1, 0).unwrap(), 0);
    assert_eq!(stack_weight(i64::MAX, 0).unwrap(), 0);
    assert_eq!(stack_weight(1, 999).unwrap(), 1);
    assert_eq!(stack_weight(3, 999).unwrap(), 2);
    assert_eq!(stack_weight(1000, i64::MAX).unwrap(), i64::MAX);
    assert!(stack_weight(1001, i64::MAX).is_err());
    assert!(stack_weight(-1, 1).is_err());
    assert!(stack_weight(1, -1).is_err());
}
#[test]
fn nonfinite_negative_zero_inconsistent_total_stage_penalty_and_units_are_rejected() {
    for bits in [
        (-0.0f64).to_bits(),
        (-1.0f64).to_bits(),
        f64::NAN.to_bits(),
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
    ] {
        let mut s = sample();
        s.maximum_bits = bits;
        assert!(s.validate().is_err());
    }
    for case in 0..7 {
        let mut s = sample();
        match case {
            0 => s.total += 1,
            1 => s.stage += 1,
            2 => s.hit_penalty = 0,
            3 => s.units[0].unit_weight = -1,
            4 => s.units.push(s.units[0]),
            5 => s.source_gear_weight = -1,
            _ => {
                s.source_gear_weight = i64::MAX;
                s.supplies_weight = 1;
            }
        }
        assert!(s.validate().is_err(), "case {case}");
    }
    assert!(sample().validate_stocks([(40010, 2)]).is_err());
    assert!(sample().validate_stocks([(40011, 3)]).is_err());
    assert!(sample().validate_stocks([(40010, 3), (40011, 1)]).is_err());
    sample().validate_stocks([(40010, 3)]).unwrap();
}
#[test]
fn bot_is_exact_canonical_zero_and_cannot_be_used_for_a_human() {
    let (zero, n) = decode(&[0; 40]).unwrap();
    assert_eq!(n, 40);
    assert!(!zero.applicable());
    zero.validate_stocks([]).unwrap();
    let mut f = weighted_reply().frame.unwrap();
    f.weights[0].weight = zero.clone();
    assert!(f.validate().is_err());
    let mut f = weighted_reply().frame.unwrap();
    let human = f.world.actors[0].key;
    f.world.actors[1].kind = crate::modern_lito::participant::MemberKind::ServerBot;
    f.world.actors[1].key = human;
    f.world.room.members[1].kind = crate::modern_lito::participant::MemberKind::ServerBot;
    f.world.room.members[1].key = human;
    f.weights[1].weight = zero.clone();
    f.validate().unwrap();
    assert!(f.world.actors[0].matches_character(human));
    assert!(!f.world.actors[1].matches_character(human));
    let mut bad = zero;
    bad.units.push(UnitWeight {
        item_id: 40010,
        unit_weight: 0,
    });
    assert!(bad.validate().is_err());
}
#[test]
fn actors_require_full_order_coverage_and_weight_capability_is_not_optional_data_loss() {
    let reply = weighted_reply();
    let f = reply.frame.as_ref().unwrap();
    let mut bytes = vec![0, 2, 0, 0, 0, 0, 0, 0];
    for row in &f.weights {
        bytes.extend(row.entity_id.to_be_bytes());
        bytes.extend(encode(&row.weight));
    }
    assert_eq!(decode_actors(&bytes, 2, false).unwrap().0, f.weights);
    for n in 0..bytes.len() {
        assert!(decode_actors(&bytes[..n], 2, false).is_err(), "prefix{n}");
    }
    let mut bad = bytes.clone();
    bad[15] = 2;
    assert!(decode_actors(&bad, 2, false).is_err());
    assert!(decode_actors(&bad, 2, true).is_err()); // duplicate global IDs stay invalid
    let mut sparse = vec![0, 2, 0, 0, 0, 0, 0, 0];
    for row in &f.weights {
        sparse.extend((row.entity_id + 2).to_be_bytes());
        sparse.extend(encode(&row.weight));
    }
    assert!(decode_actors(&sparse, 2, false).is_err());
    let decoded = decode_actors(&sparse, 2, true).unwrap().0;
    assert_eq!(
        decoded.iter().map(|s| s.entity_id).collect::<Vec<_>>(),
        vec![3, 4]
    );
    sparse[8..16].copy_from_slice(&101u64.to_be_bytes());
    assert!(decode_actors(&sparse, 2, true).is_err());
    assert!(!negotiated(&app::Response::Combat(reply.clone()), false));
    assert!(negotiated(&app::Response::Combat(reply.clone()), true));
    let mut missing = reply;
    missing.frame.as_mut().unwrap().weights.clear();
    assert!(!negotiated(&app::Response::Combat(missing.clone()), true));
    assert!(negotiated(&app::Response::Combat(missing), false));
}
#[test]
fn potion_ack_reduces_exact_weight_and_crosses_attack_threshold_without_changing_pace() {
    let old = weighted_reply();
    assert!(!old.frame.as_ref().unwrap().weight(1).unwrap().can_attack());
    let mut next = items::tests::used(&old);
    let f = next.frame.as_mut().unwrap();
    let w = &mut f.weights[0].weight;
    w.supplies_weight -= 1;
    w.total -= 1;
    w.stage = stage(w.total, w.maximum()).unwrap();
    w.hit_penalty = penalty(w.stage);
    next.validate().unwrap();
    assert!(next.confirms(&next.echo, Some(&old)));
    assert!(next.frame.as_ref().unwrap().weight(1).unwrap().can_attack());
    assert_eq!(
        old.frame.as_ref().unwrap().world.actors[0].step_interval_nanos,
        next.frame.as_ref().unwrap().world.actors[0].step_interval_nanos
    );
    let mut fake = next.clone();
    fake.frame.as_mut().unwrap().weights = old.frame.as_ref().unwrap().weights.clone();
    assert!(fake.validate().is_err());
}
#[test]
fn world_weight_basis_and_purchased_units_are_frozen_but_never_inferred_from_current_offers() {
    let old = weighted_reply();
    for case in 0..4 {
        let mut next = old.clone();
        let f = next.frame.as_mut().unwrap();
        f.world.revision += 1;
        match case {
            0 => {
                f.weights[0].weight.source_gear_weight += 1;
                f.weights[0].weight.total += 1;
            }
            1 => f.weights[0].weight.maximum_bits = 121f64.to_bits(),
            2 => f.weights[0].weight.units[0].unit_weight += 1,
            _ => f.weights.clear(),
        }
        if let Some(row) = f.weights.first_mut() {
            row.weight.stage = stage(row.weight.total, row.weight.maximum()).unwrap();
            row.weight.hit_penalty = penalty(row.weight.stage);
        }
        next.validate().unwrap(); // each isolated frame is valid; frozen-basis follows rejects it.
        assert!(!combat::follows(Some(&old), &next));
    }
}
#[test]
fn optional_snapshot_requires_all_reserved_bytes_and_reports_exact_consumed_length() {
    assert_eq!(decode_optional(&[0; 8]).unwrap(), (None, 8));
    let mut wire = vec![1, 0, 0, 0, 0, 0, 0, 0];
    wire.extend(encode(&sample()));
    assert_eq!(decode_optional(&wire).unwrap(), (Some(sample()), 64));
    for i in 1..8 {
        let mut bad = wire.clone();
        bad[i] = 1;
        assert!(decode_optional(&bad).is_err());
    }
    let mut bad = wire;
    bad[0] = 2;
    assert!(decode_optional(&bad).is_err());
}

#[test]
fn full_v9_combat_appends_weight_after_v8_items_and_old_versions_stay_byte_exact() {
    let reply = weighted_reply();
    let frame = reply.frame.as_ref().unwrap();
    let world = crate::modern_lito::world_protocol::tests::wire_complete_version(
        &reply.world_reply().unwrap(),
        8,
    );
    let mut body = world[124..].to_vec();
    body.extend((frame.vitals.len() as u16).to_be_bytes());
    body.extend([0, 0]);
    for v in &frame.vitals {
        for n in [
            v.entity_id,
            v.hp as u64,
            v.max_hp as u64,
            v.mp as u64,
            v.max_mp as u64,
            v.ac as u64,
        ] {
            body.extend(n.to_be_bytes());
        }
        body.extend([v.weapon_type, 0, 0, 0, 0, 0, 0, 0]);
        for n in [
            v.attack_sequence,
            v.attack_revision as u64,
            v.hurt_revision as u64,
            v.death_revision as u64,
            v.last_target_entity_id,
            v.attack_interval_nanos,
        ] {
            body.extend(n.to_be_bytes());
        }
    }
    body.extend([0; 4]); // no damage spells in this fixture
    body.extend(items::tests::state_bytes(&frame.items));
    let v8 = body.clone();
    let (old, used) = combat::decode_frame_version(&v8, reply.echo.context.key, 8).unwrap();
    assert_eq!(used, v8.len());
    assert!(old.weights.is_empty());
    body.extend([0, 2, 0, 0, 0, 0, 0, 0]);
    for row in &frame.weights {
        body.extend(row.entity_id.to_be_bytes());
        body.extend(encode(&row.weight));
    }
    assert_eq!(
        combat::decode_frame_version(&body, reply.echo.context.key, 9).unwrap(),
        (frame.clone(), body.len())
    );
    assert!(combat::decode_frame_version(&v8, reply.echo.context.key, 9).is_err());
    // The enclosing application grammar rejects this old-version trailing appendix.
    let mut wire = combat::encode_request(19, &reply.echo).unwrap();
    wire[12..14].copy_from_slice(&combat::RESULT_KIND.to_be_bytes());
    wire.extend([0, 0, 1, 0]);
    wire.extend(&body);
    let n = wire.len() as u32;
    wire[4..8].copy_from_slice(&n.to_be_bytes());
    assert!(app::decode_response_version(&wire, 9).is_ok());
    assert!(app::decode_response_version(&wire, 8).is_err());
    for prefix in 0..wire.len() {
        assert!(
            app::decode_response_version(&wire[..prefix], 9).is_err(),
            "prefix {prefix}"
        );
    }
    wire.push(0);
    let n = wire.len() as u32;
    wire[4..8].copy_from_slice(&n.to_be_bytes());
    assert!(app::decode_response_version(&wire, 9).is_err());
    let mut disabled = v8;
    disabled.extend([0; 8]);
    assert!(
        combat::decode_frame_version(&disabled, reply.echo.context.key, 9)
            .unwrap()
            .0
            .weights
            .is_empty()
    );
}

#[test]
fn native_buy_projection_keeps_plus_one_separate_from_stack_rounding_and_fails_overflow() {
    let mut current = sample();
    assert!(current.permits_add(1, 0));
    assert!(current.permits_add(3, 3900));
    current.maximum_bits = 101f64.to_bits();
    current.stage = 238;
    current.hit_penalty = 0;
    current.validate().unwrap();
    assert_eq!(stack_weight(1, 0).unwrap(), 0);
    assert!(
        !current.permits_add(1, 0),
        "native admission includes +1 even when stored unit weight is zero"
    );
    assert!(!current.permits_add(i64::MAX, i64::MAX));
    assert!(!current.permits_add(0, 1));
    assert!(!current.permits_add(1, -1));
    assert!(!decode(&[0; 40]).unwrap().0.permits_add(1, 1));
}

#[test]
fn overweight_attack_and_cast_denials_need_enabled_capability_and_the_correct_heavy_self() {
    use crate::modern_lito::spell_protocol as spell;
    let mut attack = weighted_reply();
    attack.echo.action = combat::Action::Attack;
    attack.echo.sequence = 1;
    attack.echo.target_entity_id = 2;
    attack.status = combat::Status::Denied;
    attack.reason = combat::Reason::TooHeavy;
    attack.frame.as_mut().unwrap().vitals[0].attack_sequence = 1;
    attack.validate().unwrap();
    assert!(negotiated(&app::Response::Combat(attack.clone()), true));
    assert!(!negotiated(&app::Response::Combat(attack.clone()), false));
    for case in 0..7 {
        let mut bad = attack.clone();
        match case {
            0 => bad.frame.as_mut().unwrap().weights.clear(),
            1 => {
                let w = &mut bad.frame.as_mut().unwrap().weights[0].weight;
                w.source_gear_weight -= 1;
                w.total -= 1;
                w.stage = 196;
            }
            2 => {
                bad.echo.context.key = bad.frame.as_ref().unwrap().world.actors[1].key;
                bad.echo.target_entity_id = 1;
            }
            3 => {
                bad.echo.action = combat::Action::Read;
                bad.echo.sequence = 0;
                bad.echo.target_entity_id = 0;
            }
            4 => {
                bad.frame.as_mut().unwrap().world.phase =
                    crate::modern_lito::world_protocol::Phase::Countdown
            }
            5 => {
                let f = bad.frame.as_mut().unwrap();
                f.vitals[0].hp = 0;
                f.vitals[0].death_revision = f.world.revision;
            }
            _ => bad.frame.as_mut().unwrap().vitals[0].attack_sequence = 0,
        }
        assert!(bad.validate().is_err(), "attack case {case}");
        if case == 0 {
            assert!(!negotiated(&app::Response::Combat(bad), false));
        }
    }
    let mut frame = attack.frame.unwrap();
    frame.world.casts = frame
        .world
        .actors
        .iter()
        .map(|a| spell::CastState::initial(a.entity_id, 1000))
        .collect();
    frame.world.casts[0].request_sequence = 1;
    for v in &mut frame.vitals {
        v.mp = 100;
        v.max_mp = 100;
    }
    let cast = spell::CastReply {
        echo: spell::CastRequest {
            context: attack.echo.context,
            channel_id: attack.echo.channel_id,
            match_id: frame.world.match_id,
            generation: frame.world.generation,
            sequence: 1,
            skill_id: 5,
            target_entity_id: 0,
        },
        status: spell::Status::Denied,
        reason: spell::Reason::TooHeavy,
        frame: Some(frame),
    };
    cast.validate().unwrap();
    assert!(negotiated(&app::Response::Cast(cast.clone()), true));
    assert!(!negotiated(&app::Response::Cast(cast.clone()), false));
    for case in 0..4 {
        let mut bad = cast.clone();
        if case == 0 {
            bad.frame.as_mut().unwrap().weights.clear();
        } else if case == 1 {
            let w = &mut bad.frame.as_mut().unwrap().weights[0].weight;
            w.source_gear_weight -= 1;
            w.total -= 1;
            w.stage = 196;
        } else if case == 2 {
            bad.frame.as_mut().unwrap().world.phase =
                crate::modern_lito::world_protocol::Phase::Countdown;
        } else {
            let f = bad.frame.as_mut().unwrap();
            f.vitals[0].hp = 0;
            f.vitals[0].death_revision = f.world.revision;
        }
        assert!(bad.validate().is_err(), "cast case {case}");
        if case == 0 {
            assert!(!negotiated(&app::Response::Cast(bad), false));
        }
    }
}
