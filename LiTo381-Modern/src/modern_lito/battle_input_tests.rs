use super::*;
use crate::modern_lito::{
    battle_payload::data_tests::fixture, battle_scene::tests::observation, battle_window::Stamp,
    world_map::WorldCell,
};
use std::collections::{BTreeMap, BTreeSet};

fn active() -> Observation {
    let mut d = fixture();
    d.world.phase = d::Phase::Active;
    d.world.revision = 2;
    d.world.loading_elapsed_nanos = 1;
    d.world.active_elapsed_nanos = 0;
    for a in &mut d.actors.0 {
        a.loaded = true;
    }
    observation(d)
}
fn intent(o: &Observation, kind: Kind) -> Intent {
    Intent {
        stamp: Stamp::of(o),
        kind,
    }
}
#[test]
fn independent_sequence_overflow_is_rejected_without_saturating_or_wrapping() {
    assert_eq!(next(0).unwrap(), 1);
    assert_eq!(next(i64::MAX - 1).unwrap(), i64::MAX as u64);
    assert!(next(-1).is_err());
    assert!(next(i64::MIN).is_err());
    assert!(next(i64::MAX).is_err());
}
#[test]
fn canonical_steps_keep_opaque_epoch_pose_and_request_sequence_without_a_frame() {
    let o = active();
    let a = actor(&o).unwrap();
    for h in 0..8 {
        let intent = intent(&o, Kind::Local(Local::Step(h)));
        validate_observation(intent, &o).unwrap();
        let op = operation(intent, &o).unwrap();
        let Operation::Move(q) = &op else {
            panic!("not MOVE")
        };
        assert_eq!((q.from_x, q.from_y), (a.position.x, a.position.y));
        assert_eq!(q.sequence, 1);
        assert_eq!(q.context.epoch, 1_u64 << 63);
        assert_eq!(q.heading, h);
        Command::operation(
            Guard {
                stamp: intent.stamp,
                target: None,
            },
            op,
        )
        .validate()
        .unwrap();
    }
}
#[test]
fn loading_foreign_attachment_and_local_catalog_cannot_be_upgraded_into_an_action() {
    let o = active();
    let good = intent(&o, Kind::Local(Local::Step(0)));
    for field in 0..10 {
        let mut bad = good;
        match field {
            0 => bad.stamp.context.key.source_id += 1,
            1 => bad.stamp.context.key.object_id += 1,
            2 => bad.stamp.context.epoch += 1,
            3 => bad.stamp.channel ^= 1,
            4..=6 => bad.stamp.world[field - 4] += 1,
            7 => bad.stamp.entity += 1,
            8 => bad.stamp.life += 1,
            _ => bad.stamp.revision += 1,
        }
        assert!(validate_observation(bad, &o).is_err(), "field {field}");
    }
    let loading = observation(fixture());
    assert!(validate_observation(intent(&loading, good.kind), &loading).is_err());
    for local in [
        Local::Step(8),
        Local::UseItem(3333),
        Local::SelectAmmo(40010),
    ] {
        assert!(validate_observation(intent(&o, Kind::Local(local)), &o).is_err());
    }
    for kind in [
        Kind::Cancel,
        Kind::Navigate(WorldCell { x: 32768, y: 32769 }),
    ] {
        assert!(operation(intent(&o, kind), &o).is_err());
    }
}
#[test]
fn scene_only_lane_does_not_invent_combat_skills_supplies_or_ammo() {
    let o = active();
    for kind in [
        Kind::Local(Local::Teleport),
        Kind::Local(Local::FireStorm),
        Kind::Local(Local::UseItem(40010)),
        Kind::Local(Local::SelectAmmo(40743)),
        Kind::Disintegrate(Target::Player { entity: 2, life: 1 }),
    ] {
        assert!(eligibility(kind, &o, &Default::default()).is_err());
    }
}

/// Called with actual Java engine publications by the already frozen six-mode
/// navigation regression. This covers field mapping, not GUI availability or
/// skill qualification; real controller tests additionally check those gates.
pub(crate) fn verify_traces(groups: &BTreeMap<String, Vec<d::UnvalidatedData>>) {
    let mut lanes = BTreeSet::new();
    let mut mapped = 0;
    for raw in groups
        .values()
        .flatten()
        .filter(|d| d.world.phase == d::Phase::Active)
    {
        let o = observation(raw.clone());
        let a = actor(&o).unwrap();
        let mut kinds = vec![
            Kind::Local(Local::Step(2)),
            Kind::Local(Local::Teleport),
            Kind::Local(Local::FireStorm),
            Kind::Portal,
        ];
        for p in &raw.actors.0 {
            if p.entity != a.entity {
                let t = Target::Player {
                    entity: p.entity,
                    life: window::life(p),
                };
                kinds.extend([Kind::Attack(t), Kind::LightArrow(t), Kind::Disintegrate(t)]);
            }
        }
        for h in &raw.honor.objectives.0 {
            let t = Target::Honor { entity: h.entity };
            kinds.extend([Kind::Attack(t), Kind::LightArrow(t), Kind::Heal(t)]);
        }
        if let Some(npcs) = &raw.npcs {
            for n in &npcs.actors.0 {
                let t = Target::Npc {
                    serial: n.serial,
                    life: n.life,
                };
                kinds.extend([Kind::Attack(t), Kind::LightArrow(t), Kind::Disintegrate(t)]);
            }
        }
        if let Some(q) = &raw.quest {
            kinds.extend(q.objectives.0.iter().map(|t| {
                Kind::Attack(Target::Quest {
                    slot: t.id.slot,
                    born: t.born_revision,
                })
            }));
        }
        if raw.time_attack.is_some() {
            kinds.push(Kind::Attack(Target::TimeTower));
        }
        if let Some(s) = &raw.siege {
            kinds.extend(s.objectives.0.iter().map(|t| {
                Kind::Attack(Target::Siege {
                    slot: t.tower.slot as u8,
                })
            }));
        }
        if let Some(s) = &raw.personal.supplies {
            kinds.extend(
                s.stocks
                    .0
                    .iter()
                    .map(|s| Kind::Local(Local::UseItem(s.item))),
            );
        }
        if let Some(s) = &raw.personal.ammo {
            kinds.extend(
                s.stocks
                    .0
                    .iter()
                    .map(|s| Kind::Local(Local::SelectAmmo(s.item))),
            );
        }
        for kind in kinds {
            let i = intent(&o, kind);
            if validate_observation(i, &o).is_err() {
                continue;
            }
            let Ok(op) = operation(i, &o) else {
                continue;
            };
            let seq = match &op {
                Operation::Move(q) => {
                    lanes.insert("move");
                    assert_eq!((q.from_x, q.from_y), (a.position.x, a.position.y));
                    assert_eq!(q.sequence, next(a.last_move_sequence).unwrap());
                    q.sequence
                }
                Operation::Attack(q) => {
                    lanes.insert(if matches!(kind, Kind::Attack(Target::Honor { .. })) {
                        "honor-attack"
                    } else {
                        "player-attack"
                    });
                    assert_eq!(
                        q.sequence,
                        next(a.vital.as_ref().unwrap().attack_sequence).unwrap()
                    );
                    q.sequence
                }
                Operation::Spell(q) => {
                    lanes.insert(match q.skill_id {
                        1 => "heal",
                        4 => "arrow",
                        5 => "teleport",
                        70 => "fire",
                        _ => panic!("skill"),
                    });
                    assert_eq!(q.sequence, next(a.spells.sequence).unwrap());
                    q.sequence
                }
                Operation::Npc(q) => {
                    lanes.insert(match q.action {
                        super::super::npc_protocol::Action::Attack => "npc-attack",
                        super::super::npc_protocol::Action::AttackObjective => "quest-attack",
                        super::super::npc_protocol::Action::AttackTower => "time-tower",
                        _ => panic!("read/revival bypass"),
                    });
                    assert_eq!(
                        q.sequence,
                        next(a.vital.as_ref().unwrap().attack_sequence).unwrap()
                    );
                    q.sequence
                }
                Operation::NpcSpell(q) => {
                    lanes.insert("npc-arrow");
                    assert_eq!(q.caster_life, i.stamp.life as u64);
                    assert_eq!(q.sequence, next(a.spells.sequence).unwrap());
                    q.sequence
                }
                Operation::Disintegrate(q) => {
                    lanes.insert("disintegrate");
                    assert_eq!(q.caster_life, i.stamp.life as u64);
                    assert_eq!(q.sequence, next(a.spells.sequence).unwrap());
                    q.sequence
                }
                Operation::NpcDisintegrate(q) => {
                    lanes.insert("npc-disintegrate");
                    assert_eq!(q.caster_life, i.stamp.life as u64);
                    assert_eq!(q.sequence, next(a.spells.sequence).unwrap());
                    q.sequence
                }
                Operation::NpcFireStorm(q) => {
                    lanes.insert("npc-fire");
                    assert_eq!(q.caster_life, i.stamp.life as u64);
                    assert_eq!(q.sequence, next(a.spells.sequence).unwrap());
                    q.sequence
                }
                Operation::Siege(q) => {
                    let portal = q.action == super::super::siege_protocol::Action::Portal;
                    lanes.insert(if portal { "portal" } else { "siege-attack" });
                    assert_eq!(
                        q.sequence,
                        next(if portal {
                            a.last_move_sequence
                        } else {
                            a.vital.as_ref().unwrap().attack_sequence
                        })
                        .unwrap()
                    );
                    q.sequence
                }
                Operation::Item(q) => {
                    lanes.insert("supply");
                    assert_eq!(
                        q.sequence,
                        next(raw.personal.supplies.as_ref().unwrap().request_sequence).unwrap()
                    );
                    q.sequence
                }
                Operation::Ammo(q) => {
                    lanes.insert("ammo");
                    assert_eq!(
                        q.sequence,
                        next(raw.personal.ammo.as_ref().unwrap().request_sequence).unwrap()
                    );
                    q.sequence
                }
            };
            assert!(seq > 0);
            Command::operation(
                Guard {
                    stamp: i.stamp,
                    target: target(kind),
                },
                op,
            )
            .validate()
            .unwrap();
            mapped += 1;
        }
    }
    for required in [
        "move",
        "player-attack",
        "honor-attack",
        "heal",
        "arrow",
        "teleport",
        "fire",
        "npc-attack",
        "quest-attack",
        "time-tower",
        "npc-arrow",
        "disintegrate",
        "npc-disintegrate",
        "npc-fire",
        "siege-attack",
        "portal",
        "supply",
        "ammo",
    ] {
        assert!(
            lanes.contains(required),
            "missing actual canonical input mapping: {required}"
        );
    }
    assert!(mapped >= 100);
    eprintln!(
        "canonical input mapping: {mapped} commands across {} lanes",
        lanes.len()
    );
}
