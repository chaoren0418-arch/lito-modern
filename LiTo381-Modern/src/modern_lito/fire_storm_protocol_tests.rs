use super::*;
use crate::modern_lito::{protocol, regen_protocol as regen, spell_protocol as spell};

pub(crate) fn initial() -> combat::Reply {
    let mut r = regen::tests::initial();
    let f = r.frame.as_mut().unwrap();
    for v in &mut f.vitals {
        v.hp = 100;
        v.max_hp = 100;
        v.mp = 300;
        v.max_mp = 300;
    }
    f.fires = f
        .world
        .actors
        .iter()
        .map(|a| State::initial(a.entity_id, 48))
        .collect();
    r.validate().unwrap();
    r
}
/// Exact authority fixture; this is never used to predict a production cast.
pub(crate) fn cast(prior: &combat::Reply, caster: u64, targets: &[(u64, i64)]) -> spell::CastReply {
    let mut f = prior.frame.as_ref().unwrap().clone();
    f.world.revision += 1;
    let i = caster as usize - 1;
    let rev = f.world.revision;
    f.world.casts[i].request_sequence += 1;
    let from = f.world.actors[i].position;
    let mut receipts = Vec::new();
    for &(id, damage) in targets {
        let before = honor::target_hp(&f, id).unwrap();
        let position = honor::target_position(&f, id).unwrap();
        let life = life_count(&f, id);
        let after = before - damage;
        if id <= f.world.actors.len() as u64 {
            let j = id as usize - 1;
            f.vitals[j].hp = after;
            f.regen[j].damage_loss += damage;
            if damage > 0 {
                f.vitals[j].hurt_revision = rev;
            }
            if after == 0 {
                f.vitals[j].death_revision = rev;
                if !f.timed.is_empty() {
                    let s = &mut f.timed[j];
                    if s.haste_until_nanos > 0
                        || s.brave_until_nanos > 0
                        || s.wisdom_until_nanos > 0
                    {
                        s.revision = rev;
                        s.haste_until_nanos = 0;
                        s.brave_until_nanos = 0;
                        s.wisdom_until_nanos = 0;
                        s.brave_kind = 0;
                    }
                }
                if let Some(h) = &mut f.honor {
                    let l = &mut h.lives[j];
                    l.death_count += 1;
                    l.last_death_revision = rev;
                    l.pending_due_millis = 5000;
                    l.death_position = position;
                }
            }
        } else {
            let o = f
                .honor
                .as_mut()
                .unwrap()
                .objectives
                .iter_mut()
                .find(|o| o.entity_id == id)
                .unwrap();
            o.hp = after;
            o.damage_loss += damage;
            if damage > 0 {
                o.hurt_revision = rev;
                o.state_revision = rev;
                o.last_actor_entity_id = caster;
                o.last_kind = honor::Interaction::FireStorm;
                o.action = honor::action_for_hp(o.hp, o.max_hp);
            }
        }
        receipts.push(Target {
            target_entity_id: id,
            hp_before: before,
            hp_after: after,
            applied_damage: damage,
            damage_loss_after: damage_loss(&f, id).unwrap(),
            respawn_count: life,
            position,
        });
    }
    let s = &mut f.fires[i];
    s.count += 1;
    s.cast_revision = rev;
    s.mp_spent += i64::from(s.mp_cost);
    s.last_cast_sequence = f.world.casts[i].request_sequence;
    s.from = from;
    s.targets = receipts;
    f.vitals[i].mp -= i64::from(s.mp_cost);
    let a = &f.world.actors[i];
    let r = spell::CastReply {
        echo: spell::CastRequest {
            context: app::SelectionContext {
                key: a.identity().character().unwrap(),
                epoch: prior.echo.context.epoch,
            },
            channel_id: prior.echo.channel_id,
            match_id: f.world.match_id,
            generation: f.world.generation,
            sequence: s.last_cast_sequence,
            skill_id: 70,
            target_entity_id: 0,
        },
        status: spell::Status::Ok,
        reason: spell::Reason::None,
        frame: Some(f),
    };
    r.validate().unwrap();
    assert!(spell::confirms_transition(
        prior.frame.as_ref().unwrap(),
        &r
    ));
    r
}
fn read(prior: &combat::Reply, r: &spell::CastReply) -> combat::Reply {
    combat::Reply {
        frame: r.frame.clone(),
        ..prior.clone()
    }
}
fn hex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn independent_64_56_big_endian_literal_and_all_prefixes() {
    let bytes=hex(concat!("0001000000000000",
        "00000000000000010000000000000002000000000000000900000000000000600000000000000007",
        "000080000000800100000030000007d00001000000000000",
        "00000000000000020000000000000064000000000000005a000000000000000a00000000000000280000000000000000",
        "0000800100008001"));
    assert_eq!(bytes.len(), 8 + STATE_BYTES + TARGET_BYTES);
    let (states, used) = decode(&bytes, 1).unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(states[0].mp_cost, 48);
    assert_eq!(states[0].targets[0].applied_damage, 10);
    let mut encoded = Vec::new();
    encode(&states, &mut encoded);
    assert_eq!(encoded, bytes);
    for n in 0..bytes.len() {
        assert!(decode(&bytes[..n], 1).is_err(), "prefix {n}");
    }
    for n in [2, 7, 66, 67, 68, 69, 70, 71] {
        let mut bad = bytes.clone();
        bad[n] = 1;
        assert!(decode(&bad, 1).is_err(), "reserved {n}");
    }
}
#[test]
fn zero_targets_success_spends_mp_and_advances_only_fire_and_global_sequence() {
    let a = initial();
    let cast = cast(&a, 1, &[]);
    let b = read(&a, &cast);
    assert!(combat::follows(Some(&a), &b));
    let x = a.frame.as_ref().unwrap();
    let y = b.frame.as_ref().unwrap();
    assert_eq!(y.fires[0].count, 1);
    assert!(y.fires[0].targets.is_empty());
    assert_eq!(y.vitals[0].mp, x.vitals[0].mp - 48);
    assert_eq!(y.vitals[1], x.vitals[1]);
    assert_eq!(y.damages, x.damages);
    assert_eq!(y.items, x.items);
    assert_eq!(y.regen, x.regen);
    assert_eq!(y.world.casts[0].cast_count, 0);
    assert!(combat::follows(Some(&b), &b));
    let mut forged = cast.clone();
    forged.frame.as_mut().unwrap().world.casts[0].request_sequence += 1;
    assert!(forged.validate().is_err());
}
#[test]
fn hit_and_immune_receipts_are_atomic_and_history_does_not_replay_hurt() {
    let a = initial();
    let hit = cast(&a, 1, &[(2, 10)]);
    let b = read(&a, &hit);
    assert!(combat::follows(Some(&a), &b));
    let immune = cast(&b, 1, &[(2, 0)]);
    let c = read(&b, &immune);
    assert!(combat::follows(Some(&b), &c));
    assert_eq!(
        b.frame.as_ref().unwrap().vitals[1].hurt_revision,
        c.frame.as_ref().unwrap().vitals[1].hurt_revision
    );
    let mut bad = c.clone();
    let rev = bad.frame.as_ref().unwrap().world.revision;
    bad.frame.as_mut().unwrap().vitals[1].hurt_revision = rev;
    assert!(bad.validate().is_err());
    let mut historical = b.clone();
    historical.frame.as_mut().unwrap().world.revision += 1;
    assert!(historical.validate().is_ok());
    assert!(combat::follows(Some(&b), &historical));
}
#[test]
fn forged_receipts_cannot_spend_old_damage_or_change_same_count_history() {
    let a = initial();
    let b = read(&a, &cast(&a, 1, &[(2, 10)]));
    for case in 0..9 {
        let mut bad = b.clone();
        let f = bad.frame.as_mut().unwrap();
        match case {
            0 => f.fires[0].targets[0].damage_loss_after = 11,
            1 => f.fires[0].targets[0].applied_damage = 9,
            2 => f.fires[0].targets[0].target_entity_id = 1,
            3 => f.fires[0].targets[0].position.x += 4,
            4 => f.fires[0].targets[0].respawn_count = 1,
            5 => f.fires[0].last_cast_sequence = 0,
            6 => f.fires[0].mp_spent = 47,
            7 => f.fires[0].reuse_millis = 1999,
            _ => f.fires[0].from.x += 1,
        }
        assert!(bad.validate().is_err(), "case {case}");
    }
    let c = read(&b, &cast(&b, 1, &[(2, 10)]));
    let mut bad = c.clone();
    let f = bad.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.fires[0].targets[0].damage_loss_after = 10;
    assert!(bad.validate().is_err());
    assert!(!combat::follows(Some(&b), &bad));
    let mut rewrite = b.clone();
    rewrite.frame.as_mut().unwrap().world.revision += 1;
    rewrite.frame.as_mut().unwrap().fires[0].targets[0]
        .position
        .x += 1;
    assert!(!combat::follows(Some(&b), &rewrite));
}
#[test]
fn hp_regeneration_and_mp_income_preserve_receipts_and_exact_ledger() {
    let a = initial();
    let b = read(&a, &cast(&a, 1, &[(2, 10)]));
    let c = regen::tests::recovered(&b, 2, 5, 0);
    assert!(combat::follows(Some(&b), &c));
    let d = regen::tests::recovered(&c, 1, 0, 48);
    assert!(combat::follows(Some(&c), &d));
    assert_eq!(
        d.frame.as_ref().unwrap().fires,
        b.frame.as_ref().unwrap().fires
    );
    let mut forged = d.clone();
    forged.frame.as_mut().unwrap().vitals[0].mp -= 1;
    assert!(forged.validate().is_err());
}
#[test]
fn historical_hp_anchor_and_global_cast_revisions_cannot_be_forged() {
    let a = initial();
    let mut b = read(&a, &cast(&a, 1, &[(2, 10)]));
    let f = b.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.fires[0].targets[0].hp_before = 90;
    f.fires[0].targets[0].hp_after = 80;
    // Current HP90 and loss10 are otherwise coherent; the historical receipt
    // cannot invent ten additional HP missing before its own ten damage.
    assert!(b.validate().is_err());
    let b = read(&a, &cast(&a, 1, &[]));
    let mut c = read(&b, &cast(&b, 2, &[]));
    let f = c.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.fires[1].cast_revision = f.fires[0].cast_revision;
    assert!(c.validate().is_err());
}
#[test]
fn coalesced_two_casters_must_pay_two_distinct_damage_checkpoints() {
    let mut a = initial();
    let f = a.frame.as_mut().unwrap();
    let mut actor = f.world.actors[1].clone();
    actor.entity_id = 3;
    actor.key.source_id = 43;
    actor.position = WorldCell {
        x: f.world.actors[0].position.x + 1,
        y: f.world.actors[0].position.y + 1,
    };
    let mut member = f.world.room.members[1].clone();
    member.key = actor.key;
    member.team = 1;
    f.world.room.members[1].team = 0;
    f.world.room.summary.settings.capacity = 3;
    f.world.room.summary.settings.attributes &= !32;
    f.world.room.summary.users += 1;
    f.world.room.summary.ready += u16::from(member.ready);
    f.world.room.members.push(member);
    f.world.actors.push(actor);
    let mut v = f.vitals[1].clone();
    v.entity_id = 3;
    f.vitals.push(v);
    let mut c = f.world.casts[1];
    c.entity_id = 3;
    f.world.casts.push(c);
    if let Some(mut d) = f.damages.get(1).copied() {
        d.entity_id = 3;
        f.damages.push(d);
    }
    let mut item = f.items[1].clone();
    item.entity_id = 3;
    f.items.push(item);
    let mut w = f.weights[1].clone();
    w.entity_id = 3;
    f.weights.push(w);
    f.timed
        .push(crate::modern_lito::timed_protocol::State::initial(3));
    f.regen.push(regen::State::initial(3));
    f.fires.push(State::initial(3, 48));
    a.validate().unwrap();
    let b = read(&a, &cast(&a, 1, &[(3, 10)]));
    let c = read(&b, &cast(&b, 2, &[(3, 10)]));
    assert!(combat::follows(Some(&a), &c));
    let mut forged = c.clone();
    let f = forged.frame.as_mut().unwrap();
    f.world.revision += 1;
    let t = &mut f.fires[1].targets[0];
    t.damage_loss_after = 10;
    t.hp_before = 100;
    t.hp_after = 90;
    forged.validate().unwrap(); // Each historical row alone is coherent.
    assert!(!combat::follows(Some(&a), &forged));
    assert!(!combat::follows(Some(&b), &forged));
}
#[test]
fn honor_multi_target_receipts_include_enemy_tower_and_not_a_fake_participant() {
    let mut a = honor::tests::initial();
    let f = a.frame.as_mut().unwrap();
    for v in &mut f.vitals {
        v.hp = 100;
        v.max_hp = 100;
        v.mp = 300;
        v.max_mp = 300;
    }
    f.fires = f
        .world
        .actors
        .iter()
        .map(|a| State::initial(a.entity_id, 48))
        .collect();
    let at = f.world.actors[0].position;
    f.honor.as_mut().unwrap().objectives[1].position = WorldCell {
        x: at.x + 2,
        y: at.y + 2,
    };
    a.validate().unwrap();
    let b = read(&a, &cast(&a, 1, &[(2, 10), (4, 10)]));
    assert!(combat::follows(Some(&a), &b));
    assert_eq!(b.frame.as_ref().unwrap().world.actors.len(), 2);
    assert_eq!(
        b.frame.as_ref().unwrap().honor.as_ref().unwrap().objectives[1].last_kind,
        honor::Interaction::FireStorm
    );
    let mut same_team = b.clone();
    same_team.frame.as_mut().unwrap().fires[0].targets[1].target_entity_id = 3;
    assert!(same_team.validate().is_err());
}
#[test]
fn honor_death_and_respawn_do_not_reset_caster_history_or_accept_an_old_life_receipt() {
    let mut a = honor::tests::initial();
    let f = a.frame.as_mut().unwrap();
    for v in &mut f.vitals {
        v.hp = 100;
        v.max_hp = 100;
        v.mp = 300;
        v.max_mp = 300;
    }
    f.fires = f
        .world
        .actors
        .iter()
        .map(|a| State::initial(a.entity_id, 48))
        .collect();
    let killed = read(&a, &cast(&a, 1, &[(2, 100)]));
    assert!(combat::follows(Some(&a), &killed));
    let respawn = honor::tests::respawn(&killed, 2);
    let next = read(&respawn, &cast(&respawn, 1, &[(2, 10)]));
    assert!(combat::follows(Some(&respawn), &next));
    assert_eq!(
        next.frame.as_ref().unwrap().fires[0].targets[0].respawn_count,
        1
    );
    let mut bad = next.clone();
    bad.frame.as_mut().unwrap().fires[0].targets[0].respawn_count = 0;
    assert!(bad.validate().is_err());
}
#[test]
fn denied_and_terminal_cast_do_not_mutate_fire_history() {
    let a = initial();
    let cast = cast(&a, 1, &[]);
    let b = read(&a, &cast);
    let mut denied = cast.clone();
    let f = denied.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.world.casts[0].request_sequence += 1;
    denied.echo.sequence += 1;
    denied.status = spell::Status::Denied;
    denied.reason = spell::Reason::TooFast;
    denied.validate().unwrap();
    assert!(spell::confirms_transition(
        b.frame.as_ref().unwrap(),
        &denied
    ));
    denied.frame.as_mut().unwrap().fires[0].count += 1;
    assert!(!spell::confirms_transition(
        b.frame.as_ref().unwrap(),
        &denied
    ));
    let mut terminal = b.clone();
    terminal.frame.as_mut().unwrap().world.phase = Phase::Aborted;
    terminal.frame.as_mut().unwrap().world.abort_reason =
        crate::modern_lito::world_protocol::AbortReason::ServerAbort;
    terminal.frame.as_mut().unwrap().world.revision += 1;
    terminal.validate().unwrap();
    let mut mutated = terminal.clone();
    mutated.frame.as_mut().unwrap().world.revision += 1;
    mutated.frame.as_mut().unwrap().fires[0].last_cast_sequence += 1;
    assert!(!follows(
        terminal.frame.as_ref().unwrap(),
        mutated.frame.as_ref().unwrap()
    ));
}
#[test]
fn v14_full_frame_roundtrip_preserves_old_v13_and_rejects_missing_extra_tail() {
    let a = initial();
    let b = read(&a, &cast(&a, 1, &[(2, 10)]));
    let wire = regen::tests::frame_wire(&b, 14);
    assert_eq!(
        app::decode_response_version(&wire, 14).unwrap().response,
        app::Response::Combat(b.clone())
    );
    assert!(app::decode_response_version(&wire, 13).is_err());
    for n in 0..wire.len() {
        assert!(app::decode_response_version(&wire[..n], 14).is_err());
    }
    let mut extra = wire.clone();
    extra.push(0);
    let size = extra.len() as u32;
    extra[4..8].copy_from_slice(&size.to_be_bytes());
    assert!(app::decode_response_version(&extra, 14).is_err());
    let mut off = a.clone();
    off.frame.as_mut().unwrap().fires.clear();
    let old = regen::tests::frame_wire(&off, 13);
    let new = regen::tests::frame_wire(&off, 14);
    assert_eq!(new.len(), old.len() + 8);
    assert_eq!(&new[8..old.len()], &old[8..]);
    assert_eq!(&new[old.len()..], &[0; 8]);
    assert!(app::decode_response_version(&old, 13).is_ok());
    assert!(app::decode_response_version(&old, 14).is_err());
}
#[test]
fn grammar_bound_is_checked_before_target_allocation_and_v14_limit_is_battle_only() {
    let mut states = Vec::new();
    for id in 1..=100 {
        let mut s = State::initial(id, 48);
        s.count = 1;
        s.cast_revision = id as i64;
        s.mp_spent = 48;
        s.last_cast_sequence = 1;
        s.from = WorldCell { x: 32768, y: 32768 };
        s.targets = (1..=102)
            .filter(|target| *target != id)
            .map(|target_entity_id| Target {
                target_entity_id,
                hp_before: 100,
                hp_after: 90,
                applied_damage: 10,
                damage_loss_after: 10,
                respawn_count: 0,
                position: WorldCell { x: 32769, y: 32768 },
            })
            .collect();
        states.push(s);
    }
    // This is the bounded grammar maximum, not a claim that fixed teams and
    // map occupancy permit every one of these targets in a real cast.
    let mut bytes = Vec::new();
    encode(&states, &mut bytes);
    assert_eq!(bytes.len(), 8 + 100 * (64 + 101 * 56));
    assert!(bytes.len() > 256 * 1024);
    assert_eq!(decode(&bytes, 100).unwrap().0, states);
    let mut bad = bytes.clone();
    bad[64..66].copy_from_slice(&102u16.to_be_bytes());
    assert!(decode(&bad, 100).is_err());
    assert!(decode(&[0, 101, 0, 0, 0, 0, 0, 0], 100).is_err());
    for kind in [23, 27, 35, 37] {
        assert!(app::header(1, kind, 1024 * 1024).is_ok());
        assert!(app::header(1, kind, 1024 * 1024 + 1).is_err());
    }
    assert!(app::header(1, 25, 64 * 1024 + 1).is_err());
    assert!(app::header(1, 33, 256 * 1024 + 1).is_err());
    assert_eq!(combat::frame_limit(13), 256 * 1024);
    assert_eq!(combat::frame_limit(14), 1024 * 1024);
}
#[test]
fn capability_and_learned_support_are_explicit_not_a_world_grant() {
    let h = protocol::Hello::current(1);
    assert!(h.max_version >= 14);
    assert_eq!(h.supported_capabilities, 262143);
    for version in 1..=13 {
        let bad = protocol::Message::Ack(protocol::Ack {
            request_id: 1,
            version,
            numeric_schema: 1,
            capabilities: 65535,
        });
        assert!(protocol::encode(bad).is_err());
    }
    let full = initial();
    assert!(!negotiated(&app::Response::Combat(full.clone()), false));
    assert!(negotiated(&app::Response::Combat(full.clone()), true));
    let mut off = full.clone();
    off.frame.as_mut().unwrap().fires.clear();
    assert!(negotiated(&app::Response::Combat(off), true));
    let mut skills = spell::SkillsReply {
        context: full.echo.context,
        status: spell::Status::Ok,
        skills: vec![spell::Skill {
            skill_id: 70,
            support: spell::Support::FireStorm,
            mp_cost: 48,
            reuse_millis: 2000,
            name: "火風暴".into(),
        }],
    };
    skills.validate().unwrap();
    assert!(skills.fire_storm().is_some());
    skills.skills[0].reuse_millis = 500;
    assert!(skills.validate().is_err());
}
