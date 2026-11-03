use super::*;
use crate::modern_lito::{protocol, spell_protocol as spell, timed_protocol};

/// Pure fixture; production obtains the full-match initial values from TLS.
pub(crate) fn initial() -> combat::Reply {
    let mut r = timed_protocol::tests::initial();
    let f = r.frame.as_mut().unwrap();
    for v in &mut f.vitals {
        v.hp = v.max_hp;
        v.mp = v.max_mp;
    }
    f.regen = f
        .world
        .actors
        .iter()
        .map(|a| State::initial(a.entity_id))
        .collect();
    r.validate().unwrap();
    r
}
/// Applied (already clamped) signed recovery; never a USE/hurt/cast event.
pub(crate) fn recovered(prior: &combat::Reply, entity: u64, hp: i64, mp: i64) -> combat::Reply {
    assert!(hp != 0 || mp != 0);
    let mut r = prior.clone();
    let f = r.frame.as_mut().unwrap();
    f.world.revision += 1;
    let i = f.vitals.iter().position(|v| v.entity_id == entity).unwrap();
    let s = &mut f.regen[i];
    let v = &mut f.vitals[i];
    if hp != 0 {
        s.hp_before = v.hp;
        v.hp = v.hp.checked_add(hp).unwrap();
        s.hp_after = v.hp;
        s.hp_revision = f.world.revision;
        if hp > 0 {
            s.hp_gain += hp;
        } else {
            s.hp_loss -= hp;
        }
    }
    if mp != 0 {
        s.mp_before = v.mp;
        v.mp = v.mp.checked_add(mp).unwrap();
        s.mp_after = v.mp;
        s.mp_revision = f.world.revision;
        if mp > 0 {
            s.mp_gain += mp;
        } else {
            s.mp_loss -= mp;
        }
    }
    r.validate().unwrap();
    assert!(combat::follows(Some(prior), &r));
    r
}
fn damaged(prior: &combat::Reply, entity: u64, amount: i64) -> combat::Reply {
    let mut r = prior.clone();
    let f = r.frame.as_mut().unwrap();
    f.world.revision += 1;
    let i = f.vitals.iter().position(|v| v.entity_id == entity).unwrap();
    let v = &mut f.vitals[i];
    v.hp -= amount;
    v.hurt_revision = f.world.revision;
    if v.hp == 0 {
        v.death_revision = f.world.revision;
    }
    f.regen[i].damage_loss += amount;
    r.validate().unwrap();
    assert!(combat::follows(Some(prior), &r));
    r
}
fn used(prior: &combat::Reply) -> item::Reply {
    let mut r = item::tests::used(prior);
    let f = r.frame.as_mut().unwrap();
    f.regen[0].item_gain += f.items[0].restored;
    let w = &mut f.weights[0].weight;
    w.supplies_weight -= 1;
    w.total -= 1;
    w.stage = crate::modern_lito::weight_protocol::stage(w.total, w.maximum()).unwrap();
    w.hit_penalty = crate::modern_lito::weight_protocol::penalty(w.stage);
    r.validate().unwrap();
    assert!(r.confirms(&r.echo, Some(prior)));
    r
}
fn hex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
pub(crate) fn frame_wire(reply: &combat::Reply, version: u16) -> Vec<u8> {
    let f = reply.frame.as_ref().unwrap();
    assert!(f.damages.is_empty() && f.ammo.is_empty());
    let mut world_reply = reply.world_reply().unwrap();
    world_reply.frame.as_mut().unwrap().casts.clear();
    let mut body = world::tests::wire_complete_version(&world_reply, version)[124..].to_vec();
    let descriptor = if version >= 15 {
        let size =
            8 + usize::from(f.world.descriptor.is_some()) * world::MatchDescriptor::WIRE_BYTES;
        body.split_off(body.len() - size)
    } else {
        Vec::new()
    };
    // The legacy test helper deliberately supports only empty cast vectors.
    // Replace that empty appendix explicitly, retaining the real fixture casts.
    body.truncate(body.len() - 4);
    body.extend((f.world.casts.len() as u16).to_be_bytes());
    body.extend([0; 2]);
    for c in &f.world.casts {
        for n in [
            c.entity_id,
            c.request_sequence,
            c.cast_count,
            c.cast_revision as u64,
            c.mp_spent as u64,
        ] {
            body.extend(n.to_be_bytes());
        }
        body.extend(c.skill_id.to_be_bytes());
        for n in [c.from.x, c.from.y, c.to.x, c.to.y] {
            body.extend(n.to_be_bytes());
        }
        body.extend(c.cast_interval_millis.to_be_bytes());
    }
    body.extend(descriptor);
    body.extend((f.vitals.len() as u16).to_be_bytes());
    body.extend([0; 2]);
    for v in &f.vitals {
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
    body.extend([0; 4]);
    body.extend(item::tests::state_bytes(&f.items));
    body.extend((f.weights.len() as u16).to_be_bytes());
    body.extend([0; 6]);
    for row in &f.weights {
        body.extend(row.entity_id.to_be_bytes());
        let w = &row.weight;
        for n in [
            w.source_gear_weight as u64,
            w.supplies_weight as u64,
            w.total as u64,
            w.maximum_bits,
        ] {
            body.extend(n.to_be_bytes());
        }
        body.extend(w.stage.to_be_bytes());
        body.extend([w.hit_penalty, 0]);
        body.extend((w.units.len() as u16).to_be_bytes());
        body.extend([0; 2]);
        for u in &w.units {
            body.extend(u.item_id.to_be_bytes());
            body.extend(u.unit_weight.to_be_bytes());
        }
    }
    body.extend((f.timed.len() as u16).to_be_bytes());
    body.extend([0; 6]);
    for s in &f.timed {
        for n in [
            s.entity_id as i64,
            s.revision,
            s.observed_nanos,
            s.haste_until_nanos,
            s.brave_until_nanos,
            s.wisdom_until_nanos,
        ] {
            body.extend(n.to_be_bytes());
        }
        body.extend([s.brave_kind, 0, 0, 0, 0, 0, 0, 0]);
    }
    body.extend([0; 8]);
    if version >= 12 {
        body.extend((f.regen.len() as u16).to_be_bytes());
        body.extend([0; 6]);
        for s in &f.regen {
            for n in [
                s.entity_id as i64,
                s.hp_gain,
                s.hp_loss,
                s.hp_revision,
                s.hp_before,
                s.hp_after,
                s.mp_gain,
                s.mp_loss,
                s.mp_revision,
                s.mp_before,
                s.mp_after,
                s.item_gain,
                s.damage_loss,
            ] {
                body.extend(n.to_be_bytes());
            }
        }
    }
    if version >= 13 {
        super::super::honor_protocol::encode(&mut body, f.honor.as_ref());
    }
    if version >= 14 {
        super::super::fire_storm_protocol::encode(&f.fires, &mut body);
    }
    let mut wire = combat::encode_request(0xfedcba98, &reply.echo).unwrap();
    wire[12..14].copy_from_slice(&combat::RESULT_KIND.to_be_bytes());
    wire.extend([reply.status as u8, reply.reason as u8, 1, 0]);
    wire.extend(body);
    let len = wire.len() as u32;
    wire[4..8].copy_from_slice(&len.to_be_bytes());
    wire
}
#[test]
fn full_frame_v12_appendix_is_atomic_and_old_v11_bytes_and_limits_are_unchanged() {
    let a = initial();
    let mut disabled = a.clone();
    disabled.frame.as_mut().unwrap().regen.clear();
    let v11 = frame_wire(&disabled, 11);
    let decoded = app::decode_response_version(&v11, 11).unwrap();
    assert_eq!(decoded.response, app::Response::Combat(disabled.clone()));
    assert!(app::decode_response_version(&v11, 12).is_err());
    let v12 = frame_wire(&disabled, 12);
    assert_eq!(v12.len(), v11.len() + 8);
    assert_eq!(&v12[8..v11.len()], &v11[8..]);
    assert_eq!(
        app::decode_response_version(&v12, 12).unwrap().response,
        app::Response::Combat(disabled)
    );
    let full = frame_wire(&a, 12);
    assert_eq!(full.len(), v11.len() + 8 + 2 * 104);
    assert_eq!(
        app::decode_response_version(&full, 12).unwrap().response,
        app::Response::Combat(a)
    );
    assert!(app::decode_response_version(&full, 11).is_err());
    for end in 0..full.len() {
        assert!(
            app::decode_response_version(&full[..end], 12).is_err(),
            "truncate {end}"
        );
    }
    let mut tail = full;
    tail.push(0);
    let n = tail.len() as u32;
    tail[4..8].copy_from_slice(&n.to_be_bytes());
    assert!(app::decode_response_version(&tail, 12).is_err());
}
#[test]
fn stale_regen_attack_and_item_require_the_correct_changed_axis_and_exact_denial_sequence() {
    let a = initial();
    let b = recovered(&a, 2, -1, 0);
    let mut reply = b.clone();
    let f = reply.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.vitals[0].attack_sequence = 1;
    reply.echo.action = combat::Action::Attack;
    reply.echo.sequence = 1;
    reply.echo.target_entity_id = 2;
    reply.status = combat::Status::Denied;
    reply.reason = combat::Reason::StaleRegen;
    reply.validate().unwrap();
    assert!(combat::follows(Some(&a), &reply));
    let wire = frame_wire(&reply, 12);
    assert!(app::decode_response_version(&wire, 12).is_ok());
    assert!(app::decode_response_version(&wire, 11).is_err());
    for case in 0..4 {
        let mut bad = reply.clone();
        match case {
            0 => bad.echo.sequence = 2,
            1 => bad.echo.target_entity_id = 1,
            2 => bad.status = combat::Status::Busy,
            _ => bad.frame.as_mut().unwrap().regen.clear(),
        }
        assert!(bad.validate().is_err());
    }
    let base = recovered(&a, 1, -10, 0);
    let mut f = base.frame.unwrap();
    f.world.revision += 1;
    f.items[0].request_sequence = 1;
    let use_reply = item::Reply {
        echo: item::Request {
            action: item::Action::Use,
            sequence: 1,
            item_id: 40010,
            ..item::Request::read(a.echo.context, a.echo.channel_id, &f.world)
        },
        status: item::Status::Denied,
        reason: item::Reason::StaleRegen,
        frame: Some(f),
    };
    use_reply.validate().unwrap();
    assert!(use_reply.confirms(&use_reply.echo, Some(&a)));
    let mut bad = use_reply.clone();
    bad.echo.item_id = 40013;
    assert!(bad.validate().is_err());
    let mut bad = use_reply;
    bad.echo.sequence = 2;
    assert!(bad.validate().is_err());
}
#[test]
fn stale_regen_cast_does_not_spend_mp_and_cannot_use_unknown_target_or_wrong_axis() {
    let a = initial();
    let b = recovered(&a, 1, 0, -1);
    let mut f = b.frame.unwrap();
    f.world.revision += 1;
    f.world.casts[0].request_sequence = 1;
    let reply = spell::CastReply {
        echo: spell::CastRequest {
            context: a.echo.context,
            channel_id: a.echo.channel_id,
            match_id: f.world.match_id,
            generation: f.world.generation,
            sequence: 1,
            skill_id: 5,
            target_entity_id: 0,
        },
        status: spell::Status::Denied,
        reason: spell::Reason::StaleRegen,
        frame: Some(f),
    };
    reply.validate().unwrap();
    assert!(spell::confirms_transition(
        a.frame.as_ref().unwrap(),
        &reply
    ));
    assert_eq!(reply.frame.as_ref().unwrap().world.casts[0].mp_spent, 0);
    let mut bad = reply.clone();
    bad.echo.skill_id = 4;
    bad.echo.target_entity_id = 999;
    assert!(bad.validate().is_err());
    let mut bad = reply.clone();
    bad.echo.target_entity_id = 1;
    assert!(bad.validate().is_err());
    let mut bad = reply;
    bad.frame.as_mut().unwrap().regen[0].mp_revision = 0;
    assert!(bad.validate().is_err());
}
#[test]
fn independent_104_byte_state_golden_and_every_truncation() {
    let bytes = hex(concat!(
        "0001000000000000",
        "0000000000000001",
        "000000000000000a",
        "0000000000000028",
        "0000000000000004",
        "000000000000003c",
        "0000000000000046",
        "0000000000000005",
        "0000000000000032",
        "0000000000000004",
        "0000000000000032",
        "0000000000000037",
        "0000000000000007",
        "0000000000000003"
    ));
    assert_eq!(bytes.len(), 112);
    let (rows, n) = decode_states(&bytes, 1, false).unwrap();
    assert_eq!(n, 112);
    assert_eq!(
        rows[0],
        State {
            entity_id: 1,
            hp_gain: 10,
            hp_loss: 40,
            hp_revision: 4,
            hp_before: 60,
            hp_after: 70,
            mp_gain: 5,
            mp_loss: 50,
            mp_revision: 4,
            mp_before: 50,
            mp_after: 55,
            item_gain: 7,
            damage_loss: 3
        }
    );
    for end in 0..bytes.len() {
        assert!(
            decode_states(&bytes[..end], 1, false).is_err(),
            "truncate {end}"
        );
    }
    for at in [2, 3, 4, 5, 6, 7] {
        let mut b = bytes.clone();
        b[at] = 1;
        assert!(decode_states(&b, 1, false).is_err());
    }
    let mut bad = bytes.clone();
    bad[15] = 2;
    assert!(decode_states(&bad, 1, false).is_err());
    let sparse = decode_states(&bad, 1, true).unwrap().0;
    assert_eq!(sparse[0].entity_id, 2);
    assert_eq!(sparse[0].hp_gain, 10);
    bad[8..16].copy_from_slice(&101u64.to_be_bytes());
    assert!(decode_states(&bad, 1, true).is_err());
    let mut bad = bytes.clone();
    bad[0..2].copy_from_slice(&101u16.to_be_bytes());
    assert!(decode_states(&bad, 101, false).is_err());
    assert!(decode_states(&bytes, 2, false).is_err());
    for at in (16..112).step_by(8) {
        let mut bad = bytes.clone();
        bad[at..at + 8].copy_from_slice(&(-1i64).to_be_bytes());
        assert!(decode_states(&bad, 1, false).is_err(), "negative {at}");
    }
}
#[test]
fn positive_negative_recovery_and_full_i64_balance_do_not_overflow_intermediate() {
    let a = initial();
    let b = recovered(&a, 1, -40, -50);
    let c = recovered(&b, 1, 10, 5);
    assert_eq!(
        (
            c.frame.as_ref().unwrap().vitals[0].hp,
            c.frame.as_ref().unwrap().vitals[0].mp
        ),
        (70, 55)
    );
    assert_eq!(c.frame.as_ref().unwrap().vitals[0].hurt_revision, 0);
    assert_eq!(
        c.frame.as_ref().unwrap().items,
        a.frame.as_ref().unwrap().items
    );
    assert!(combat::follows(Some(&a), &c));
    let mut high = initial();
    let f = high.frame.as_mut().unwrap();
    f.vitals[0].max_hp = i64::MAX;
    f.vitals[0].hp = i64::MAX;
    f.vitals[0].max_mp = i64::MAX;
    f.vitals[0].mp = i64::MAX;
    let low = recovered(&high, 1, -100, -100);
    let high = recovered(&low, 1, 100, 100);
    assert_eq!(high.frame.unwrap().vitals[0].hp, i64::MAX);
}
#[test]
fn no_event_or_same_revision_cannot_change_hp_mp_or_ledger() {
    let a = initial();
    for axis in 0..2 {
        let mut b = a.clone();
        b.frame.as_mut().unwrap().world.revision += 1;
        if axis == 0 {
            b.frame.as_mut().unwrap().vitals[0].hp -= 1;
        } else {
            b.frame.as_mut().unwrap().vitals[0].mp -= 1;
        }
        assert!(b.validate().is_err());
        assert!(!combat::follows(Some(&a), &b));
    }
    let mut b = recovered(&a, 1, -1, 0);
    b.frame.as_mut().unwrap().world.revision = a.frame.as_ref().unwrap().world.revision;
    assert!(!combat::follows(Some(&a), &b));
}
#[test]
fn gain_loss_cancellation_retains_both_legs_and_cannot_rewrite_history() {
    let a = recovered(&initial(), 1, -20, -20);
    let b = recovered(&a, 1, 10, 10);
    let c = recovered(&b, 1, -10, -10);
    assert!(combat::follows(Some(&a), &c));
    assert_eq!(
        c.frame.as_ref().unwrap().vitals,
        a.frame.as_ref().unwrap().vitals
    );
    let mut bad = c.clone();
    let s = &mut bad.frame.as_mut().unwrap().regen[0];
    s.hp_gain -= 10;
    s.hp_loss -= 10;
    s.mp_gain -= 10;
    s.mp_loss -= 10;
    assert!(!combat::follows(Some(&b), &bad));
    let mut bad = c.clone();
    bad.frame.as_mut().unwrap().regen[0].hp_revision = a.frame.as_ref().unwrap().world.revision;
    assert!(!combat::follows(Some(&a), &bad));
}
#[test]
fn latest_use_then_regen_then_damage_and_coalesced_history_are_exact() {
    let a = damaged(&initial(), 1, 60);
    let b = used(&a).combat_reply();
    let c = recovered(&b, 1, 7, -5);
    let d = damaged(&c, 1, 3);
    assert!(combat::follows(Some(&a), &d));
    assert_eq!(d.frame.as_ref().unwrap().vitals[0].hp, 54);
    let mut forged = d.clone();
    forged.frame.as_mut().unwrap().vitals[0].hp += 1;
    assert!(forged.validate().is_err());
    let e = used(&d).combat_reply();
    assert!(combat::follows(Some(&a), &e));
    assert_eq!(e.frame.as_ref().unwrap().regen[0].item_gain, 20);
}
#[test]
fn single_use_exact_gain_and_multiple_use_bounds_cannot_be_forged() {
    let a = damaged(&initial(), 1, 70);
    let b = used(&a).combat_reply();
    let mut bad = b.clone();
    let f = bad.frame.as_mut().unwrap();
    f.regen[0].item_gain += 1;
    f.regen[0].damage_loss += 1;
    assert!(!combat::follows(Some(&a), &bad));
    let c = used(&b).combat_reply();
    assert!(combat::follows(Some(&a), &c));
    let mut bad = c.clone();
    let f = bad.frame.as_mut().unwrap();
    f.regen[0].item_gain = 41;
    f.regen[0].damage_loss += 21;
    f.vitals[0].hurt_revision = f.world.revision - 1;
    assert!(!combat::follows(Some(&a), &bad));
}
#[test]
fn damage_loss_requires_new_hurt_and_zero_damage_cannot_fake_a_hurt() {
    let a = initial();
    let b = damaged(&a, 1, 10);
    let mut bad = b.clone();
    bad.frame.as_mut().unwrap().vitals[0].hurt_revision = 0;
    assert!(bad.validate().is_err());
    let mut bad = b.clone();
    let f = bad.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.vitals[0].hurt_revision = f.world.revision;
    assert!(!combat::follows(Some(&b), &bad));
}
#[test]
fn recovery_cannot_kill_resurrect_or_cross_same_revision_use_hurt_cast() {
    let a = initial();
    let mut bad = a.clone();
    let f = bad.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.regen[0].hp_loss = 100;
    f.regen[0].hp_before = 100;
    f.regen[0].hp_revision = f.world.revision;
    f.vitals[0].hp = 0;
    f.vitals[0].death_revision = f.world.revision;
    assert!(bad.validate().is_err());
    let dead = damaged(&a, 1, 100);
    let mut bad = dead.clone();
    let f = bad.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.regen[0].mp_loss = 1;
    f.regen[0].mp_before = 100;
    f.regen[0].mp_after = 99;
    f.regen[0].mp_revision = f.world.revision;
    f.vitals[0].mp = 99;
    assert!(bad.validate().is_err());
    assert!(!combat::follows(Some(&dead), &bad));
    let a = damaged(&initial(), 1, 50);
    let b = recovered(&a, 1, 5, 0);
    let mut bad = b.clone();
    bad.frame.as_mut().unwrap().regen[0].hp_revision =
        a.frame.as_ref().unwrap().vitals[0].hurt_revision;
    assert!(bad.validate().is_err());
}
#[test]
fn mp_refill_allows_cumulative_spell_cost_above_initial_max_without_resetting_cost() {
    fn tp(prior: &combat::Reply, count: u64) -> combat::Reply {
        let mut r = prior.clone();
        let f = r.frame.as_mut().unwrap();
        f.world.revision += 1;
        let actor = &mut f.world.actors[0];
        let from = actor.position;
        actor.position.x += 2;
        let c = &mut f.world.casts[0];
        c.cast_count += count;
        c.request_sequence += count;
        c.cast_revision = f.world.revision;
        c.mp_spent += (count * 10) as i64;
        c.skill_id = 5;
        c.from = from;
        c.to = actor.position;
        f.vitals[0].mp -= (count * 10) as i64;
        r.validate().unwrap();
        assert!(combat::follows(Some(prior), &r));
        r
    }
    let a = tp(&initial(), 9);
    let b = recovered(&a, 1, 0, 90);
    let c = tp(&b, 2);
    assert_eq!(c.frame.as_ref().unwrap().vitals[0].mp, 80);
    assert_eq!(c.frame.as_ref().unwrap().world.casts[0].mp_spent, 110);
    assert!(combat::follows(Some(&a), &c));
}
#[test]
fn loading_and_terminal_states_preserve_history_and_no_missing_cap_is_accepted() {
    let a = initial();
    let mut loading = a.clone();
    loading.frame.as_mut().unwrap().world.phase = world::Phase::Loading;
    loading.validate().unwrap();
    let b = recovered(&a, 1, -10, 0);
    let mut bad = b.clone();
    bad.frame.as_mut().unwrap().world.phase = world::Phase::Loading;
    assert!(bad.validate().is_err());
    let mut terminal = b.clone();
    let f = terminal.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.world.phase = world::Phase::Aborted;
    f.world.abort_reason = world::AbortReason::ServerAbort;
    terminal.validate().unwrap();
    assert!(combat::follows(Some(&b), &terminal));
    assert!(combat::follows(Some(&terminal), &terminal));
    let response = app::Response::Combat(terminal.clone());
    assert!(negotiated(&response, true));
    assert!(!negotiated(&response, false));
    let mut missing = terminal;
    missing.frame.as_mut().unwrap().regen.clear();
    assert!(!negotiated(&app::Response::Combat(missing), true));
}
#[test]
fn v12_capability_dependencies_and_old_masks_are_explicit() {
    for version in 1..=11 {
        assert!(protocol::encode(protocol::Message::Ack(protocol::Ack {
            request_id: 1,
            version,
            numeric_schema: 1,
            capabilities: 16383
        }))
        .is_err());
    }
    for missing in [
        protocol::CAP_COMBAT,
        protocol::CAP_WEIGHT,
        protocol::CAP_SHOP,
    ] {
        assert!(protocol::encode(protocol::Message::Ack(protocol::Ack {
            request_id: 1,
            version: 12,
            numeric_schema: 1,
            capabilities: 16383 & !missing
        }))
        .is_err());
    }
    assert!(protocol::encode(protocol::Message::Ack(protocol::Ack {
        request_id: 1,
        version: 12,
        numeric_schema: 1,
        capabilities: 16383
    }))
    .is_ok());
    for (version, capabilities) in [
        (1, 15),
        (8, 1023),
        (9, 2047),
        (10, 4095),
        (11, 8191),
        (12, 8191),
    ] {
        assert!(protocol::encode(protocol::Message::Ack(protocol::Ack {
            request_id: 1,
            version,
            numeric_schema: 1,
            capabilities
        }))
        .is_ok());
    }
}
#[test]
fn tagged_bot_only_accumulates_damage_even_with_same_numeric_human_key() {
    use crate::modern_lito::{participant::MemberKind, weight_protocol::Snapshot};
    let mut r = initial();
    let f = r.frame.as_mut().unwrap();
    f.world.actors[1].kind = MemberKind::ServerBot;
    f.world.actors[1].key = f.world.actors[0].key;
    f.world.room.members[1].kind = MemberKind::ServerBot;
    f.world.room.members[1].key = f.world.room.members[0].key;
    f.world.room.members[1].source_name = f.world.room.members[0].source_name.clone();
    f.world.room.members[1].name = "BOT01".into();
    f.weights[1].weight = Snapshot {
        source_gear_weight: 0,
        supplies_weight: 0,
        total: 0,
        maximum_bits: 0,
        stage: 0,
        hit_penalty: 0,
        units: vec![],
    };
    r.validate().unwrap();
    let hurt = damaged(&r, 2, 1);
    assert_eq!(hurt.frame.as_ref().unwrap().regen[1].damage_loss, 1);
    let mut bad = hurt;
    let f = bad.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.regen[1].hp_gain = 1;
    f.regen[1].hp_revision = f.world.revision;
    f.regen[1].hp_before = 99;
    f.regen[1].hp_after = 100;
    f.vitals[1].hp = 100;
    assert!(bad.validate().is_err());
}
#[test]
fn finished_receipt_preserves_all_recovery_and_damage_history() {
    let mut pending = combat::tests::finished_reply();
    let f = pending.frame.as_mut().unwrap();
    f.vitals[0].hp = f.vitals[0].max_hp;
    f.regen = f
        .vitals
        .iter()
        .map(|v| {
            let mut s = State::initial(v.entity_id);
            s.damage_loss = v.max_hp - v.hp;
            s
        })
        .collect();
    pending.validate().unwrap();
    let mut saved = pending.clone();
    let f = saved.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.world.completion.as_mut().unwrap().state = world::SettlementState::Saved;
    saved.echo.action = combat::Action::Read;
    saved.echo.sequence = 0;
    saved.echo.target_entity_id = 0;
    saved.validate().unwrap();
    assert!(combat::follows(Some(&pending), &saved));
    let mut bad = saved.clone();
    let f = bad.frame.as_mut().unwrap();
    f.regen[0].hp_loss = 1;
    f.regen[0].hp_revision = f.world.revision;
    f.regen[0].hp_before = f.vitals[0].hp;
    f.regen[0].hp_after = f.vitals[0].hp - 1;
    f.vitals[0].hp -= 1;
    assert!(bad.validate().is_err());
    assert!(!combat::follows(Some(&pending), &bad));
    assert_eq!(pending.frame.unwrap().regen, saved.frame.unwrap().regen);
}
