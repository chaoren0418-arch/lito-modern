use super::{data::*, *};
fn reject(
    d: &UnvalidatedData,
    label: &str,
    case: &str,
    index: usize,
    edit: impl FnOnce(&mut UnvalidatedData),
) -> usize {
    let mut bad = d.clone();
    edit(&mut bad);
    let bad = decode(&encode(&bad).expect("mode mutation remains encodable")).unwrap();
    bad.personal_view(&d.personal.player, &d.world.identity)
        .unwrap_or_else(|_| panic!("{label}/{case}/{index}: must reach mode gate"));
    assert!(
        bad.check_scene_values(&d.personal.player, &d.world.identity)
            .is_err(),
        "{label}/{case}/{index}: accepted corrupt mode"
    );
    1
}
pub(super) fn real_payload_mutations(label: &str, d: &UnvalidatedData) -> usize {
    let mut n = 0;
    if !d.honor.objectives.0.is_empty() {
        for i in 0..13 {
            n += reject(d, label, "Honor tower", i, |d| {
                let o = &mut d.honor.objectives.0[0];
                match i {
                    0 => o.hp = -1,
                    1 => o.maximum_hp = 0,
                    2 => o.action = 99,
                    3 => o.graphic = 0,
                    4 => o.damage_loss = -1,
                    5 => o.heal_gain = -1,
                    6 => o.last_actor = 101,
                    7 => o.last_kind = 5,
                    8 => o.revision = d.world.revision + 1,
                    9 => o.hurt_revision = -1,
                    10 => o.death_revision = -1,
                    11 => o.cell.x = i32::MIN,
                    _ => o.heal_gain += 1,
                }
            });
        }
        n += reject(d, label, "Honor occupied", 0, |d| {
            d.honor.objectives.0[1].cell = d.honor.objectives.0[0].cell.clone()
        });
    }
    if let Some(s) = &d.siege {
        for i in 0..14 {
            n += reject(d, label, "Siege state", i, |d| {
                let s = d.siege.as_mut().unwrap();
                match i {
                    0 => s.revision = 0,
                    1 => s.revision = d.world.revision + 1,
                    2 => s.duration_millis = 49,
                    3 => s.duration_millis += 1,
                    4 => s.respawn_millis = 60_001,
                    5 => s.attacker = 2,
                    6 => s.winner = 2,
                    7 => s.gate_revision = -1,
                    8 => s.finish_revision = -1,
                    9 => s.finished_at_millis = -1,
                    10 => s.portal_inside = s.portal_outside.clone(),
                    11 => s.gate.0[0] = s.towers.0[0].cell.clone(),
                    12 => s.attack_camp.0.clear(),
                    _ => {
                        s.phase = if s.phase == SiegePhase::Waiting {
                            SiegePhase::Outer
                        } else {
                            SiegePhase::Waiting
                        }
                    }
                }
            });
        }
        for i in 0..11 {
            n += reject(d, label, "Siege objective", i, |d| {
                let s = d.siege.as_mut().unwrap();
                let t = &mut s.objectives.0[0];
                match i {
                    0 => t.hp = -1,
                    1 => t.damage_loss += 1,
                    2 => t.hit_count = -1,
                    3 => t.revision = d.world.revision + 1,
                    4 => t.death_revision = -1,
                    5 => t.last_actor = 101,
                    6 => t.tower.maximum_hp += 1,
                    7 => s.towers.0[0].graphic = 0,
                    8 => s.objectives.0.swap(0, 1),
                    9 => s.towers.0[0].cell.x = i32::MIN,
                    _ => t.hit_count = t.revision + 1,
                }
            });
        }
        // Gate sets are hashed in coordinate order; arbitrary wire enumeration
        // must not change policy identity or turn a valid result into an error.
        let mut reordered = d.clone();
        reordered.siege.as_mut().unwrap().gate.0.reverse();
        reordered
            .check_scene_values(&d.personal.player, &d.world.identity)
            .unwrap();
        assert_eq!(
            siege::config_hash(s),
            siege::config_hash(reordered.siege.as_ref().unwrap())
        );
        if d.result.is_some() {
            for i in 0..10 {
                n += reject(d, label, "Siege proof", i, |d| {
                    let p = d.result.as_mut().unwrap().siege.as_mut().unwrap();
                    match i {
                        0 => p.config_sha256[0] ^= 1,
                        1 => p.attacker = 1 - p.attacker,
                        2 => p.winner = 1 - p.winner,
                        3 => {
                            p.end = if p.end == SiegeEnd::Timeout {
                                SiegeEnd::InnerDestroyed
                            } else {
                                SiegeEnd::Timeout
                            }
                        }
                        4 => p.duration_millis += 1,
                        5 => p.respawn_millis += 1,
                        6 => p.elapsed_millis += 1,
                        7 => p.gate_revision += 1,
                        8 => p.towers.0[0].hp += 1,
                        _ => {
                            d.result.as_mut().unwrap().participants.0[0].verdict =
                                if d.result.as_ref().unwrap().participants.0[0].verdict
                                    == Verdict::Win
                                {
                                    Verdict::Loss
                                } else {
                                    Verdict::Win
                                }
                        }
                    }
                });
            }
        }
    }
    if let Some(r) = &d.race {
        for i in 0..13 {
            n += reject(d, label, "Race state", i, |d| {
                let r = d.race.as_mut().unwrap();
                match i {
                    0 => r.revision = -1,
                    1 => r.revision = d.world.revision + 1,
                    2 => r.active_limit_nanos += 1,
                    3 => r.observed_elapsed_nanos = -1,
                    4 => r.first_finisher_entity = 11,
                    5 => r.finish_window_deadline_elapsed_nanos += 1,
                    6 => r.terminal_revision = -1,
                    7 => r.course_sha256[0] ^= 1,
                    8 => r.gates.0[0].min_x = i32::MAX,
                    9 => r.gates.0[0] = r.gates.0[1].clone(),
                    10 => r.result_id = Some([0; 16]),
                    11 => {
                        r.phase = if r.phase == RacePhase::Running {
                            RacePhase::Waiting
                        } else {
                            RacePhase::Running
                        }
                    }
                    _ => r.gates.0.reverse(),
                }
            });
        }
        for i in 0..11 {
            n += reject(d, label, "Race runner", i, |d| {
                let r = &mut d.race.as_mut().unwrap().runners.0[0];
                match i {
                    0 => r.revision = -1,
                    1 => r.revision = d.world.revision + 1,
                    2 => r.progress_move_sequence = -1,
                    3 => r.completed_laps = 5,
                    4 => r.next_checkpoint = 64,
                    5 => r.placement = 11,
                    6 => r.finish_elapsed_nanos = -1,
                    7 => r.base_graphics = 0,
                    8 => r.progress_move_sequence = i64::MAX,
                    9 => r.finish_elapsed_nanos = i64::MAX,
                    _ => r.placement = if r.placement == 0 { 1 } else { 0 },
                }
            });
        }
        if r.phase == RacePhase::Complete {
            n += reject(d, label, "Race premature complete", 0, |d| {
                let r = d.race.as_mut().unwrap();
                r.observed_elapsed_nanos = r.finish_window_deadline_elapsed_nanos - 1;
            });
        }
    }
    if d.result.is_some() && matches!(d.room.mode, 1 | 5) {
        n += reject(d, label, "Team verdict", 0, |d| {
            let p = &mut d.result.as_mut().unwrap().participants.0[0];
            p.verdict = if p.verdict == Verdict::Win {
                Verdict::Loss
            } else {
                Verdict::Win
            };
        });
    }
    n
}
#[test]
fn honor_action_uses_java_integer_thresholds_without_i64_overflow() {
    assert_eq!(honor::tower_action(0, 1), 35);
    assert_eq!(honor::tower_action(1, 1), 0);
    assert_eq!(honor::tower_action(i64::MAX / 2, i64::MAX), 32);
    assert_eq!(honor::tower_action(i64::MAX / 4 - 1, i64::MAX), 34);
    assert_eq!(honor::tower_action(i64::MAX, i64::MAX), 0);
}
#[test]
fn finished_scene_cannot_omit_its_mode_result() {
    let mut d = data_tests::fixture();
    d.world.phase = Phase::Finished;
    for a in &mut d.actors.0 {
        a.loaded = true;
    }
    assert!(d
        .check_scene_values(&d.personal.player, &d.world.identity)
        .is_err());
}
#[test]
fn scene_only_cannot_award_honor_victory_without_any_towers() {
    let mut d = data_tests::fixture();
    d.room.mode = 5;
    d.room.map = 510;
    d.world.phase = Phase::Finished;
    for a in &mut d.actors.0 {
        a.loaded = true;
    }
    d.result = Some(MatchResult {
        result_id: [1; 16],
        match_id: d.world.identity.match_id,
        generation: d.world.identity.generation,
        engine_revision: d.world.revision,
        mode: 5,
        map: 510,
        finished_at_millis: 1,
        participants: List(
            d.room
                .members
                .0
                .iter()
                .enumerate()
                .map(|(i, m)| ResultParticipant {
                    entity: i as i64 + 1,
                    identity: m.identity.clone(),
                    team: m.team,
                    verdict: if i == 0 { Verdict::Win } else { Verdict::Loss },
                })
                .collect(),
        ),
        time_attack: None,
        siege: None,
    });
    let d = decode(&encode(&d).unwrap()).unwrap();
    d.personal_view(&d.personal.player, &d.world.identity)
        .unwrap();
    assert!(d
        .check_scene_values(&d.personal.player, &d.world.identity)
        .is_err());
}
