use super::*;
use crate::modern_lito::{regen_protocol as regen, spell_protocol as spell};

pub(crate) fn initial() -> combat::Reply {
    let mut r = regen::tests::initial();
    let f = r.frame.as_mut().unwrap();
    f.world.room.summary.settings.mode = MODE;
    f.world.room.summary.settings.map_id = 532;
    let at = f.world.actors[0].position;
    let objectives = (0..2)
        .map(|team| Objective {
            entity_id: 3 + team as u64,
            team,
            action: 0,
            graphics_id: 2665 + 2 * i32::from(team),
            position: crate::modern_lito::world_map::WorldCell {
                x: at.x + 4 + 2 * i32::from(team),
                y: at.y,
            },
            hp: 100,
            max_hp: 100,
            state_revision: 0,
            hurt_revision: 0,
            death_revision: 0,
            damage_loss: 0,
            heal_gain: 0,
            last_actor_entity_id: 0,
            last_kind: Interaction::None,
        })
        .collect();
    f.honor = Some(State {
        objectives,
        lives: vec![Life::initial(1), Life::initial(2)],
        heals: vec![HealCast::initial(1, 4), HealCast::initial(2, 0)],
    });
    r.validate().unwrap();
    r
}
pub(crate) fn died(prior: &combat::Reply, entity: u64) -> combat::Reply {
    let mut r = prior.clone();
    let f = r.frame.as_mut().unwrap();
    let i = entity as usize - 1;
    f.world.revision += 1;
    f.regen[i].damage_loss += f.vitals[i].hp;
    f.vitals[i].hp = 0;
    f.vitals[i].hurt_revision = f.world.revision;
    f.vitals[i].death_revision = f.world.revision;
    let l = &mut f.honor.as_mut().unwrap().lives[i];
    l.death_count += 1;
    l.last_death_revision = f.world.revision;
    l.pending_due_millis = 5000;
    l.death_position = f.world.actors[i].position;
    r.validate().unwrap();
    assert!(combat::follows(Some(prior), &r));
    r
}
pub(crate) fn respawn(prior: &combat::Reply, entity: u64) -> combat::Reply {
    let mut r = prior.clone();
    let f = r.frame.as_mut().unwrap();
    let i = entity as usize - 1;
    f.world.revision += 1;
    f.vitals[i].hp = f.vitals[i].max_hp;
    let l = &mut f.honor.as_mut().unwrap().lives[i];
    l.respawn_count += 1;
    l.respawn_revision = f.world.revision;
    l.respawn_hp_gain += f.vitals[i].max_hp;
    l.pending_due_millis = 0;
    r.validate().unwrap();
    assert!(combat::follows(Some(prior), &r));
    r
}
fn tower_hit(prior: &combat::Reply, amount: i64) -> combat::Reply {
    let mut r = prior.clone();
    let f = r.frame.as_mut().unwrap();
    f.world.revision += 1;
    let v = &mut f.vitals[0];
    v.attack_sequence += 1;
    v.attack_revision = f.world.revision;
    v.last_target_entity_id = 4;
    let o = &mut f.honor.as_mut().unwrap().objectives[1];
    o.hp -= amount;
    o.damage_loss += amount;
    o.state_revision = f.world.revision;
    o.hurt_revision = f.world.revision;
    o.last_actor_entity_id = 1;
    o.last_kind = Interaction::Physical;
    o.action = action_for_hp(o.hp, o.max_hp);
    r.validate().unwrap();
    assert!(combat::follows(Some(prior), &r));
    r
}
pub(crate) fn heal(prior: &combat::Reply, restored: i64) -> spell::CastReply {
    let mut f = prior.frame.as_ref().unwrap().clone();
    f.world.revision += 1;
    f.world.casts[0].request_sequence += 1;
    let h = f.honor.as_mut().unwrap();
    let o = &mut h.objectives[0];
    let before = o.hp;
    o.hp += restored;
    o.heal_gain += restored;
    o.state_revision = f.world.revision;
    o.last_actor_entity_id = 1;
    o.last_kind = Interaction::Heal;
    o.action = action_for_hp(o.hp, o.max_hp);
    let a = &mut h.heals[0];
    a.count += 1;
    a.revision = f.world.revision;
    a.mp_spent += i64::from(a.mp_cost);
    a.target_objective_id = o.entity_id;
    a.hp_before = before;
    a.hp_after = o.hp;
    a.restored = restored;
    a.from = f.world.actors[0].position;
    a.to = o.position;
    a.skill_id = BASIC_HEAL;
    f.vitals[0].mp -= i64::from(a.mp_cost);
    let r = spell::CastReply {
        echo: spell::CastRequest {
            context: prior.echo.context,
            channel_id: prior.echo.channel_id,
            match_id: f.world.match_id,
            generation: f.world.generation,
            sequence: f.world.casts[0].request_sequence,
            skill_id: BASIC_HEAL,
            target_entity_id: 3,
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
    assert!(combat::follows(Some(prior), &r.combat_reply().unwrap()));
    r
}
#[test]
fn initial_is_separate_from_participants_and_all_reserved_bytes_are_strict() {
    let r = initial();
    let h = r.frame.as_ref().unwrap().honor.as_ref().unwrap();
    let mut b = Vec::new();
    encode(&mut b, Some(h));
    assert_eq!(b.len(), 8 + 2 * 96 + 2 * 64 + 2 * 88);
    assert_eq!(&b[..8], &[1, 2, 0, 2, 0, 2, 0, 0]);
    assert_eq!(decode(&b, 2).unwrap(), (Some(h.clone()), b.len()));
    for n in 0..b.len() {
        assert!(decode(&b[..n], 2).is_err(), "prefix {n}");
    }
    for offset in [6, 7, 8 + 10, 8 + 11, 8 + 89, 8 + 95] {
        let mut bad = b.clone();
        bad[offset] = 1;
        assert!(decode(&bad, 2).is_err(), "reserved {offset}");
    }
    assert_eq!(
        target(r.frame.as_ref().unwrap(), 1),
        Some(Target::Participant(1))
    );
    assert_eq!(
        target(r.frame.as_ref().unwrap(), 3),
        Some(Target::Objective(3))
    );
    assert_eq!(target(r.frame.as_ref().unwrap(), 5), None);
    assert_eq!(r.frame.as_ref().unwrap().world.room.members.len(), 2);
}
#[test]
fn v13_full_frame_preserves_v12_and_never_silently_discards_honor() {
    let r = initial();
    let bytes = regen::tests::frame_wire(&r, 13);
    assert_eq!(
        app::decode_response_version(&bytes, 13).unwrap().response,
        app::Response::Combat(r)
    );
    for v in 1..=12 {
        assert!(app::decode_response_version(&bytes, v).is_err());
    }
    let old = regen::tests::initial();
    let oldbytes = regen::tests::frame_wire(&old, 12);
    assert!(app::decode_response_version(&oldbytes, 12).is_ok());
    assert!(app::decode_response_version(&oldbytes, 13).is_err());
    let current = regen::tests::frame_wire(&old, 13);
    assert!(app::decode_response_version(&current, 13).is_ok());
    assert!(app::decode_response_version(&current, 12).is_err());
}
#[test]
fn same_position_respawn_is_an_explicit_full_hp_gain_without_mp_or_inventory_reset() {
    let a = initial();
    let b = died(&a, 1);
    let c = respawn(&b, 1);
    let f = c.frame.as_ref().unwrap();
    assert_eq!(f.vitals[0].hp, f.vitals[0].max_hp);
    assert_eq!(
        f.vitals[0].death_revision,
        b.frame.as_ref().unwrap().vitals[0].death_revision
    );
    assert_eq!(f.vitals[0].mp, a.frame.as_ref().unwrap().vitals[0].mp);
    assert_eq!(f.items, a.frame.as_ref().unwrap().items);
    assert_eq!(f.world.casts, a.frame.as_ref().unwrap().world.casts);
    assert!(combat::follows(Some(&a), &c));
    let again = died(&c, 1);
    let twice = respawn(&again, 1);
    assert_eq!(
        twice.frame.unwrap().honor.unwrap().lives[0].respawn_hp_gain,
        200
    );
}
#[test]
fn respawn_cannot_erase_death_history_or_invent_mp_or_move_the_corpse() {
    let a = initial();
    let b = died(&a, 1);
    let c = respawn(&b, 1);
    for case in 0..5 {
        let mut bad = c.clone();
        let f = bad.frame.as_mut().unwrap();
        match case {
            0 => f.vitals[0].death_revision = 0,
            1 => f.honor.as_mut().unwrap().lives[0].respawn_hp_gain -= 1,
            2 => f.vitals[0].mp -= 1,
            3 => f.world.actors[0].position.x += 1,
            _ => f.honor.as_mut().unwrap().lives[0].pending_due_millis = 5000,
        };
        assert!(bad.validate().is_err(), "{case}");
    }
}
#[test]
fn dead_actor_does_not_regenerate_and_legacy_death_stays_permanent() {
    let a = initial();
    let b = died(&a, 1);
    let mut bad = b.clone();
    let f = bad.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.vitals[0].hp = 1;
    f.regen[0].hp_gain = 1;
    f.regen[0].hp_revision = f.world.revision;
    f.regen[0].hp_before = 1;
    f.regen[0].hp_after = 2;
    assert!(bad.validate().is_err());
    let mut legacy = initial();
    legacy.frame.as_mut().unwrap().honor = None;
    legacy
        .frame
        .as_mut()
        .unwrap()
        .world
        .room
        .summary
        .settings
        .mode = 1;
    legacy
        .frame
        .as_mut()
        .unwrap()
        .world
        .room
        .summary
        .settings
        .map_id = 514;
    let mut dead = legacy.clone();
    let f = dead.frame.as_mut().unwrap();
    f.world.revision += 1;
    f.vitals[0].hp = 0;
    f.vitals[0].hurt_revision = f.world.revision;
    f.vitals[0].death_revision = f.world.revision;
    f.regen[0].damage_loss = 100;
    assert!(combat::follows(Some(&legacy), &dead));
    let mut revive = dead.clone();
    revive.frame.as_mut().unwrap().vitals[0].hp = 100;
    assert!(!combat::follows(Some(&dead), &revive));
}
#[test]
fn full_hp_heal_spends_mp_and_advances_interaction_but_not_tower_animation_or_hurt() {
    let a = initial();
    let b = heal(&a, 0);
    let f = b.frame.as_ref().unwrap();
    let o = &f.honor.as_ref().unwrap().objectives[0];
    assert_eq!(
        (o.hp, o.heal_gain, o.hurt_revision, o.action),
        (100, 0, 0, 0)
    );
    assert_eq!(o.state_revision, f.world.revision);
    assert_eq!(o.last_kind, Interaction::Heal);
    assert_eq!(f.vitals[0].mp, a.frame.as_ref().unwrap().vitals[0].mp - 4);
    assert_eq!(f.honor.as_ref().unwrap().heals[0].count, 1);
    let mut bad = b.clone();
    bad.frame
        .as_mut()
        .unwrap()
        .honor
        .as_mut()
        .unwrap()
        .objectives[0] = a.frame.as_ref().unwrap().honor.as_ref().unwrap().objectives[0].clone();
    assert!(bad.validate().is_err());
}
#[test]
fn tower_damage_never_changes_roster_health_or_regen_damage_ledger() {
    let a = initial();
    let b = tower_hit(&a, 26);
    let f = b.frame.as_ref().unwrap();
    assert_eq!(f.honor.as_ref().unwrap().objectives[1].action, 32);
    assert_eq!(f.vitals[1], a.frame.as_ref().unwrap().vitals[1]);
    assert_eq!(f.regen, a.frame.as_ref().unwrap().regen);
    for case in 0..4 {
        let mut bad = b.clone();
        let o = &mut bad
            .frame
            .as_mut()
            .unwrap()
            .honor
            .as_mut()
            .unwrap()
            .objectives[1];
        match case {
            0 => o.hp += 1,
            1 => o.last_actor_entity_id = 3,
            2 => o.team = 0,
            _ => o.action = 0,
        };
        assert!(bad.validate().is_err());
    }
}
#[test]
fn shape_identity_occupancy_mode_and_frozen_tower_profile_are_required() {
    let a = initial();
    for case in 0..6 {
        let mut b = a.clone();
        let f = b.frame.as_mut().unwrap();
        match case {
            0 => {
                f.honor.as_mut().unwrap().lives.pop();
            }
            1 => f.honor.as_mut().unwrap().objectives[0].entity_id = 1,
            2 => f.honor.as_mut().unwrap().objectives[0].position = f.world.actors[0].position,
            3 => f.world.room.summary.settings.mode = 1,
            4 => f.honor.as_mut().unwrap().heals[0].mp_cost = 5,
            _ => f.regen.clear(),
        };
        assert!(b.validate().is_err());
    }
    let mut b = a.clone();
    b.frame.as_mut().unwrap().world.revision += 1;
    b.frame.as_mut().unwrap().honor.as_mut().unwrap().objectives[0].graphics_id = 2671;
    assert!(b.validate().is_ok());
    assert!(!combat::follows(Some(&a), &b));
}
#[test]
fn old_versions_reject_targeted_basic_heal_and_support4_is_exact() {
    let a = initial();
    let r = heal(&a, 0);
    for v in 4..=12 {
        assert!(spell::encode_cast_version(1, &r.echo, v).is_err());
    }
    assert_eq!(
        spell::encode_cast_version(1, &r.echo, 13).unwrap().len(),
        80
    );
    let mut skill = spell::SkillsReply {
        context: a.echo.context,
        status: spell::Status::Ok,
        skills: vec![spell::Skill {
            skill_id: 1,
            support: spell::Support::BasicHeal,
            mp_cost: 4,
            reuse_millis: 10,
            name: "初級治癒術".into(),
        }],
    };
    assert!(skill.validate().is_ok());
    skill.skills[0].mp_cost = 0;
    assert!(skill.validate().is_err());
}
#[test]
fn negotiated_honor_is_per_mode_without_granting_targets_to_legacy_deathmatch() {
    let a = app::Response::Combat(initial());
    assert!(negotiated(&a, true));
    assert!(!negotiated(&a, false));
    let dm = app::Response::Combat(regen::tests::initial());
    assert!(negotiated(&dm, true));
    assert!(negotiated(&dm, false));
}
#[test]
fn a_respawned_actor_cannot_reactivate_a_lease_from_before_its_death() {
    use crate::modern_lito::timed_protocol as timed;
    let initial = initial();
    let dead = died(&initial, 1);
    let alive = respawn(&dead, 1);
    let used = timed::tests::use_timed(&alive, timed::Kind::Haste, 1_000_000);
    let mut frame = used.frame.unwrap();
    assert!(timed::validate_frame(&frame).is_ok());
    frame.world.revision += 1;
    for revision in [0, frame.vitals[0].death_revision] {
        let mut bad = frame.clone();
        bad.timed[0].revision = revision;
        assert!(timed::validate_frame(&bad).is_err());
    }
}
#[test]
fn skipped_heal_revision_still_requires_objective_interaction_and_applied_gain() {
    let a = initial();
    let mut fake = a.clone();
    let f = fake.frame.as_mut().unwrap();
    f.world.revision += 2;
    f.world.casts[0].request_sequence = 1;
    f.vitals[0].mp -= 4;
    let o = &f.honor.as_ref().unwrap().objectives[0];
    let to = o.position;
    let rev = f.world.revision - 1;
    let h = &mut f.honor.as_mut().unwrap().heals[0];
    h.count = 1;
    h.revision = rev;
    h.mp_spent = 4;
    h.target_objective_id = 3;
    h.hp_before = 90;
    h.hp_after = 100;
    h.restored = 10;
    h.from = f.world.actors[0].position;
    h.to = to;
    h.skill_id = 1;
    assert!(fake.validate().is_err());
    // A genuinely confirmed older gain cannot pay for a second cast. Both
    // endpoints pass single-frame validation; only the new history is false.
    let mut damaged = a.clone();
    let f = damaged.frame.as_mut().unwrap();
    f.world.revision += 1;
    let v = &mut f.vitals[1];
    v.attack_sequence = 1;
    v.attack_revision = f.world.revision;
    v.last_target_entity_id = 3;
    let o = &mut f.honor.as_mut().unwrap().objectives[0];
    o.hp = 90;
    o.damage_loss = 10;
    o.hurt_revision = f.world.revision;
    o.state_revision = f.world.revision;
    o.last_actor_entity_id = 2;
    o.last_kind = Interaction::Physical;
    damaged.validate().unwrap();
    assert!(combat::follows(Some(&a), &damaged));
    let previous = heal(&damaged, 10).combat_reply().unwrap();
    let mut fake = previous.clone();
    let f = fake.frame.as_mut().unwrap();
    f.world.revision += 2;
    f.world.casts[0].request_sequence += 1;
    f.vitals[0].mp -= 4;
    let h = f.honor.as_mut().unwrap();
    h.heals[0].count += 1;
    h.heals[0].mp_spent += 4;
    h.heals[0].revision = f.world.revision - 1;
    h.objectives[0].state_revision = f.world.revision - 1;
    fake.validate().unwrap();
    assert!(!follows(
        previous.frame.as_ref().unwrap(),
        fake.frame.as_ref().unwrap()
    ));
}
