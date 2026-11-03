use super::{data::*, *};
#[test]
fn sparse_visible_actor_uses_global_entity_not_visible_list_index() {
    let mut data = data_tests::fixture();
    data.actors.0.remove(0);
    assert_eq!(data.actors.0[0].entity, 2);
    assert_ne!(
        data.room.members.0[0].identity,
        data.room.members.0[1].identity
    );
    assert_eq!(
        data.scene_member(&data.actors.0[0]).unwrap().identity,
        data.room.members.0[1].identity
    );
    for entity in [0, -1, i64::MIN, i64::MAX, 3] {
        let mut invalid = data.actors.0[0].clone();
        invalid.entity = entity;
        assert!(data.scene_member(&invalid).is_err());
    }
}
fn reject(
    d: &UnvalidatedData,
    label: &str,
    case: &str,
    index: usize,
    edit: impl FnOnce(&mut UnvalidatedData),
) -> usize {
    let mut bad = d.clone();
    edit(&mut bad);
    let bad =
        decode(&encode(&bad).unwrap_or_else(|_| panic!("{label}/{case}/{index}: retained syntax")))
            .unwrap();
    bad.personal_view(&d.personal.player, &d.world.identity)
        .unwrap_or_else(|_| panic!("{label}/{case}/{index}: must reach new scene checks"));
    assert!(
        bad.check_scene_values(&d.personal.player, &d.world.identity)
            .is_err(),
        "{label}/{case}/{index}: corrupt scene accepted"
    );
    1
}
pub(super) fn real_payload_mutations(label: &str, d: &UnvalidatedData) -> usize {
    let mut count = 0;
    if let Some(n) = &d.npcs {
        for i in 0..8 {
            count += reject(d, label, "NPC metadata", i, |d| {
                let n = d.npcs.as_mut().unwrap();
                match i {
                    0 => n.revision = 0,
                    1 => n.revision = d.world.revision + 1,
                    2 => n.elapsed_millis = -1,
                    3 => n.elapsed_millis = 1_860_001,
                    4 => n.spawned = -1,
                    5 => n.closed = !n.closed,
                    6 => n.names.0[0].name = Text(" ".into()),
                    _ => n.names.0[0].name_id = Text(" ".into()),
                }
            });
        }
        if n.names.0.len() > 1 {
            count += reject(d, label, "unsorted names", 0, |d| {
                d.npcs.as_mut().unwrap().names.0.swap(0, 1);
            });
        }
        if !n.actors.0.is_empty() {
            for i in 0..17 {
                count += reject(d, label, "NPC resident", i, |d| {
                    let a = &mut d.npcs.as_mut().unwrap().actors.0[0];
                    match i {
                        0 => a.hp = -1,
                        1 => a.maximum_hp = 0,
                        2 => a.hp = a.maximum_hp + 1,
                        3 => a.mp = -1,
                        4 => a.maximum_mp = -1,
                        5 => a.life = 0,
                        6 => a.born_revision = 0,
                        7 => a.changed_revision = d.world.revision + 1,
                        8 => a.corpse_until_millis = -1,
                        9 => a.next_move_millis = 1_920_001,
                        10 => a.next_attack_millis = -1,
                        11 => a.heading = 8,
                        12 => a.stage = 0,
                        13 => a.serial = 0,
                        14 => a.cell.x = i32::MIN,
                        15 => {
                            a.action = if a.hp == 0 {
                                NpcAction::Idle
                            } else {
                                NpcAction::Death
                            }
                        }
                        _ => a.corpse_until_millis = if a.hp == 0 { 0 } else { 1 },
                    }
                });
            }
        }
        if !n.exchanges.0.is_empty() {
            for i in 0..11 {
                count += reject(d, label, "NPC exchange", i, |d| {
                    let e = &mut d.npcs.as_mut().unwrap().exchanges.0[0];
                    match i {
                        0 => e.revision = d.world.revision + 1,
                        1 => e.request_sequence = -1,
                        2 => e.participant_life = 0,
                        3 => e.npc_life = 0,
                        4 => e.outcome.damage = -1,
                        5 => e.outcome.remaining_hp = -1,
                        6 => e.outcome.material_bonus = 21,
                        7 => e.outcome.blessing_bonus = 5,
                        8 => e.outcome.death = !e.outcome.death,
                        9 => {
                            e.outcome.hit = false;
                            e.outcome.damage = 1;
                        }
                        _ => {
                            e.outcome.hit = true;
                            e.outcome.remaining_hp = i64::MAX;
                            e.outcome.applied_damage = 1;
                            e.outcome.damage = 1;
                            e.outcome.death = false;
                        }
                    }
                });
            }
        }
    }
    if !d.honor.lives.0.is_empty() {
        for i in 0..6 {
            count += reject(d, label, "Honor life", i, |d| {
                let l = &mut d.honor.lives.0[0];
                match i {
                    0 => l.death_count = -1,
                    1 => l.respawn_count = i64::MAX,
                    2 => l.last_death_revision = d.world.revision + 1,
                    3 => l.respawn_hp_gain = -1,
                    4 => l.pending_due_millis = -1,
                    _ => l.death_cell.x = i32::MIN,
                }
            });
        }
    }
    if let Some(q) = &d.quest {
        count += reject(d, label, "missing Quest state", 0, |d| d.quest = None);
        for i in 0..6 {
            count += reject(d, label, "Quest progress", i, |d| {
                let q = d.quest.as_mut().unwrap();
                match i {
                    0 => q.revision = 0,
                    1 => {
                        q.flow = if q.flow == QuestFlow::RoomSymbols {
                            QuestFlow::FinalGuardian
                        } else {
                            QuestFlow::RoomSymbols
                        }
                    }
                    2 => {
                        q.status = if q.status == QuestStatus::Finished {
                            QuestStatus::Running
                        } else {
                            QuestStatus::Finished
                        }
                    }
                    3 => q.lanes.0[0].stage = 0,
                    4 => q.winning_lane = 2,
                    _ => q.death_revision = -1,
                }
            });
        }
        if !q.targets.0.is_empty() {
            for i in 0..3 {
                count += reject(d, label, "Quest target policy", i, |d| {
                    let t = &mut d.quest.as_mut().unwrap().targets.0[0];
                    match i {
                        0 => t.maximum_hp = 0,
                        1 => t.graphic = -1,
                        _ => t.cell.x = i32::MIN,
                    }
                });
            }
        }
        if !q.objectives.0.is_empty() {
            for i in 0..7 {
                count += reject(d, label, "Quest objective", i, |d| {
                    let a = &mut d.quest.as_mut().unwrap().objectives.0[0];
                    match i {
                        0 => a.hp = -1,
                        1 => a.born_revision = 0,
                        2 => a.revision = d.world.revision + 1,
                        3 => a.hurt_revision = -1,
                        4 => a.death_revision = -1,
                        5 => a.target.maximum_hp += 1,
                        _ => {
                            a.kind = if a.kind == QuestKind::RoomSymbol {
                                QuestKind::FinalSymbol
                            } else {
                                QuestKind::RoomSymbol
                            }
                        }
                    }
                });
            }
        }
        if !q.exchanges.0.is_empty() {
            for i in 0..5 {
                count += reject(d, label, "Quest physical event", i, |d| {
                    let e = &mut d.quest.as_mut().unwrap().exchanges.0[0];
                    match i {
                        0 => e.revision = d.world.revision + 1,
                        1 => e.request_sequence = 0,
                        2 => e.outcome.raw_damage = -1,
                        3 => e.outcome.applied_damage = -1,
                        _ => e.outcome.death = !e.outcome.death,
                    }
                });
            }
        }
        if !q.lives.0.is_empty() {
            for i in 0..6 {
                count += reject(d, label, "Quest life", i, |d| {
                    let l = &mut d.quest.as_mut().unwrap().lives.0[0];
                    match i {
                        0 => l.deaths = -1,
                        1 => l.respawns = i64::MAX,
                        2 => l.hp_gain = -1,
                        3 => l.death_revision = d.world.revision + 1,
                        4 => l.spawn.x = i32::MIN,
                        _ => l.death_cell.x = i32::MIN,
                    }
                });
            }
        }
    }
    if d.siege.is_some() {
        for i in 0..5 {
            count += reject(d, label, "Siege life", i, |d| {
                let l = &mut d.siege.as_mut().unwrap().lives.0[0];
                match i {
                    0 => l.deaths = -1,
                    1 => l.respawns = i64::MAX,
                    2 => l.hp_gain = -1,
                    3 => l.death_revision = d.world.revision + 1,
                    _ => l.due_millis = -1,
                }
            });
        }
    }
    if let Some(t) = &d.time_attack {
        for i in 0..17 {
            count += reject(d, label, "Time state", i, |d| {
                let t = d.time_attack.as_mut().unwrap();
                match i {
                    0 => t.revision = 0,
                    1 => t.revision = d.world.revision + 1,
                    2 => t.wave = -1,
                    3 => t.leader = 0,
                    4 => t.duration_millis = 49,
                    5 => t.started_millis = -2,
                    6 => t.observed_millis = -1,
                    7 => t.tower.cell.x = i32::MIN,
                    8 => t.tower.graphic = -1,
                    9 => t.tower.maximum_hp = 0,
                    10 => t.tower_hp = -1,
                    11 => t.tower_hurt_revision = -1,
                    12 => t.tower_death_revision = -1,
                    13 => t.plan_sha256 = [0; 32],
                    14 => t.waves.0.clear(),
                    15 => t.waves.0[0] = TimeKind::Dragon,
                    _ => t.departed.0.push(61),
                }
            });
        }
        if !t.exchanges.0.is_empty() {
            for i in 0..5 {
                count += reject(d, label, "Time physical event", i, |d| {
                    let e = &mut d.time_attack.as_mut().unwrap().exchanges.0[0];
                    match i {
                        0 => e.revision = d.world.revision + 1,
                        1 => e.sequence = 0,
                        2 => e.outcome.raw_damage = -1,
                        3 => e.outcome.applied_damage = -1,
                        _ => e.outcome.death = !e.outcome.death,
                    }
                });
            }
        }
    }
    if d.result.as_ref().is_some_and(|r| r.time_attack.is_some()) {
        for i in 0..7 {
            count += reject(d, label, "Time terminal proof", i, |d| {
                let p = d.result.as_mut().unwrap().time_attack.as_mut().unwrap();
                match i {
                    0 => p.plan_sha256[0] ^= 1,
                    1 => p.wave_count += 1,
                    2 => p.leader = 0,
                    3 => p.duration_millis += 1,
                    4 => p.alive_mask ^= 1,
                    5 => p.departed_mask ^= 1,
                    _ => p.tower_death_revision = -1,
                }
            });
        }
    }
    if d.result.is_some() {
        for i in 0..3 {
            count += reject(d, label, "terminal identity/clock", i, |d| {
                let r = d.result.as_mut().unwrap();
                match i {
                    0 => r.result_id = [0; 16],
                    1 => r.finished_at_millis = -1,
                    _ => r.engine_revision = d.world.revision - 1,
                }
            });
        }
    }
    if let Some(q) = &d.quest {
        if !matches!(d.world.phase, Phase::Loading | Phase::Countdown) {
            // Mode-local progression is not an engine revision. It must not be
            // narrowed or accidentally compared to the unrelated engine clock.
            let mut separate = d.clone();
            separate.quest.as_mut().unwrap().revision = i64::MAX;
            separate
                .check_scene_values(&d.personal.player, &d.world.identity)
                .unwrap();
        }
        if label == "quest-loss" {
            assert!(q
                .objectives
                .0
                .iter()
                .all(|a| a.target.lane != q.winning_lane));
        }
    }
    count
}

#[test]
fn unchanged_scene_needs_no_invented_npc_or_life_tables() {
    let d = data_tests::fixture();
    d.check_scene_values(&d.personal.player, &d.world.identity)
        .unwrap();
    let mut invented = d.clone();
    invented.honor.lives.0.push(HonorLife {
        entity: 1,
        death_count: 0,
        last_death_revision: 0,
        respawn_count: 0,
        respawn_revision: 0,
        respawn_hp_gain: 0,
        pending_due_millis: 0,
        death_cell: Cell { x: 0, y: 0 },
    });
    invented
        .personal_view(&d.personal.player, &d.world.identity)
        .unwrap();
    assert!(invented
        .check_scene_values(&d.personal.player, &d.world.identity)
        .is_err());
}
#[test]
fn physical_outcome_uses_wide_safe_capacity_not_overflowing_sum() {
    let o = Outcome {
        hit: true,
        raw_damage: i64::MAX,
        applied_damage: i64::MAX - 1,
        remaining_hp: 1,
        death: false,
        maximum_triggered: true,
        critical_triggered: false,
    };
    scene::outcome(&o, i64::MAX).unwrap();
    let mut bad = o.clone();
    bad.remaining_hp = 2;
    assert!(scene::outcome(&bad, i64::MAX).is_err());
    assert_eq!(
        scene::distance(&Cell { x: i32::MIN, y: 0 }, &Cell { x: i32::MAX, y: 0 }),
        u32::MAX as i64
    );
    assert!(!scene::ordered(&[1, 1], 10));
    assert!(!scene::ordered(&[i64::MAX], 10));
}
