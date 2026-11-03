//! Mode-specific timelines. Engine, NPC-roster and Quest-progress revisions
//! are deliberately separate. Polling may skip complete intermediate waves.
use super::*;
fn life(a: [i64; 5], b: [i64; 5], rev: i64, delta: i64) -> bool {
    let [deaths, death, respawns, respawn, gain] = a;
    let [nd, ndr, nr, nrr, ng] = b;
    advances(deaths, nd, delta)
        && advances(respawns, nr, delta)
        && ng >= gain
        && if nd == deaths {
            ndr == death
        } else {
            ndr > rev
        }
        && if nr == respawns {
            nrr == respawn && ng == gain
        } else {
            nrr > rev && ng > gain
        }
}
fn exchanges<T: PartialEq>(
    a: &[T],
    b: &[T],
    revision: impl Fn(&T) -> i64,
    same: impl Fn(&T, &T) -> bool,
    rev: i64,
    retire: impl Fn(&T) -> bool,
    advance: impl Fn(&T, &T) -> bool,
) -> bool {
    a.iter().all(|x| {
        b.iter().find(|y| same(x, y)).map_or_else(
            || retire(x),
            |y| {
                event(x, y, revision(x), revision(y), rev)
                    && (revision(x) == revision(y) || advance(x, y))
            },
        )
    }) && b
        .iter()
        .all(|y| a.iter().any(|x| same(x, y)) || revision(y) > rev)
}
fn honor(a: &UnvalidatedData, b: &UnvalidatedData) -> bool {
    let x = &a.honor;
    let y = &b.honor;
    let rev = a.world.revision;
    let delta = b.world.revision - rev;
    x.objectives.0.len() == y.objectives.0.len()
        && x.lives.0.len() == y.lives.0.len()
        && x.objectives.0.iter().zip(&y.objectives.0).all(|(p, q)| {
            p.entity == q.entity
                && p.team == q.team
                && p.graphic == q.graphic
                && p.cell == q.cell
                && p.maximum_hp == q.maximum_hp
                && event(p, q, p.revision, q.revision, rev)
                && q.damage_loss >= p.damage_loss
                && q.heal_gain >= p.heal_gain
                && q.hurt_revision >= p.hurt_revision
                && q.death_revision >= p.death_revision
                && (p.hp != 0 || p == q)
                && (p.damage_loss == q.damage_loss) == (p.hurt_revision == q.hurt_revision)
        })
        && x.lives.0.iter().zip(&y.lives.0).all(|(p, q)| {
            p.entity == q.entity
                && life(
                    [
                        p.death_count,
                        p.last_death_revision,
                        p.respawn_count,
                        p.respawn_revision,
                        p.respawn_hp_gain,
                    ],
                    [
                        q.death_count,
                        q.last_death_revision,
                        q.respawn_count,
                        q.respawn_revision,
                        q.respawn_hp_gain,
                    ],
                    rev,
                    delta,
                )
                && (p.death_count != q.death_count || p.death_cell == q.death_cell)
                && (p.death_count != q.death_count
                    || p.respawn_count != q.respawn_count
                    || p.pending_due_millis == q.pending_due_millis
                    || terminal(b.world.phase) && q.pending_due_millis == 0)
        })
}
fn siege(a: &UnvalidatedData, b: &UnvalidatedData) -> bool {
    optional(&a.siege, &b.siege, |x, y| {
        let rev = a.world.revision;
        let delta = b.world.revision - rev;
        let cells = |v: &List<Cell, 64>| {
            v.0.iter()
                .map(|c| (c.x, c.y))
                .collect::<std::collections::BTreeSet<_>>()
        };
        if x.scope != y.scope
            || x.mode != y.mode
            || x.logical_map != y.logical_map
            || x.towers != y.towers
            || cells(&x.gate) != cells(&y.gate)
            || x.attack_camp != y.attack_camp
            || x.defense_camp != y.defense_camp
            || x.portal_inside != y.portal_inside
            || x.portal_outside != y.portal_outside
            || x.duration_millis != y.duration_millis
            || x.respawn_millis != y.respawn_millis
            || x.attacker != y.attacker
            || y.revision < x.revision
            || (y.phase as u8) < x.phase as u8
            || x.gate_revision > 0 && x.gate_revision != y.gate_revision
        {
            return false;
        }
        let mut copy = x.clone();
        copy.lives = y.lives.clone();
        copy.gate = y.gate.clone();
        if !event(&copy, y, x.revision, y.revision, rev) {
            return false;
        }
        x.objectives.0.iter().zip(&y.objectives.0).all(|(p, q)| {
            q.hp <= p.hp
                && advances(p.hit_count, q.hit_count, delta)
                && event(p, q, p.revision, q.revision, rev)
                && (p.hp != 0 || p == q)
                && (p.hit_count != q.hit_count || p == q)
        }) && x.lives.0.iter().zip(&y.lives.0).all(|(p, q)| {
            p.entity == q.entity
                && life(
                    [
                        p.deaths,
                        p.death_revision,
                        p.respawns,
                        p.respawn_revision,
                        p.hp_gain,
                    ],
                    [
                        q.deaths,
                        q.death_revision,
                        q.respawns,
                        q.respawn_revision,
                        q.hp_gain,
                    ],
                    rev,
                    delta,
                )
                && (p.deaths != q.deaths
                    || p.respawns != q.respawns
                    || p.due_millis == q.due_millis
                    || terminal(b.world.phase) && q.due_millis == 0)
        })
    })
}
fn quest(a: &UnvalidatedData, b: &UnvalidatedData) -> bool {
    optional(&a.quest, &b.quest, |x, y| {
        let rev = a.world.revision;
        let delta = b.world.revision - rev;
        if x.flow != y.flow
            || x.targets != y.targets
            || y.revision < x.revision
            || x.lanes.0.len() != y.lanes.0.len()
            || x.revision == y.revision && (x.lanes != y.lanes || x.status != y.status)
        {
            return false;
        }
        if !x
            .lanes
            .0
            .iter()
            .zip(&y.lanes.0)
            .all(|(p, q)| p.lane == q.lane && q.stage >= p.stage)
        {
            return false;
        }
        if !x.objectives.0.iter().all(|p| {
            y.objectives.0.iter().any(|q| {
                q.id.slot > p.id.slot
                    || q.id == p.id
                        && q.born_revision == p.born_revision
                        && q.target == p.target
                        && q.kind == p.kind
                        && q.hp <= p.hp
                        && event(p, q, p.revision, q.revision, rev)
            })
        }) {
            return false;
        }
        if !y
            .objectives
            .0
            .iter()
            .all(|q| x.objectives.0.iter().any(|p| p.id == q.id) || q.born_revision > rev)
        {
            return false;
        }
        exchanges(
            &x.exchanges.0,
            &y.exchanges.0,
            |e| e.revision,
            |p, q| p.participant == q.participant,
            rev,
            |_| false,
            |p, q| q.request_sequence > p.request_sequence,
        ) && x.lives.0.len() == y.lives.0.len()
            && x.lives.0.iter().zip(&y.lives.0).all(|(p, q)| {
                p.entity == q.entity
                    && p.lane == q.lane
                    && p.spawn == q.spawn
                    && life(
                        [
                            p.deaths,
                            p.death_revision,
                            p.respawns,
                            p.respawn_revision,
                            p.hp_gain,
                        ],
                        [
                            q.deaths,
                            q.death_revision,
                            q.respawns,
                            q.respawn_revision,
                            q.hp_gain,
                        ],
                        rev,
                        delta,
                    )
                    && (p.deaths != q.deaths || p.death_cell == q.death_cell)
            })
    })
}
fn time(a: &UnvalidatedData, b: &UnvalidatedData) -> bool {
    optional(&a.time_attack, &b.time_attack, |x, y| {
        if x.waves != y.waves
            || x.duration_millis != y.duration_millis
            || x.plan_sha256 != y.plan_sha256
            || x.tower != y.tower
            || y.revision < x.revision
            || y.observed_millis < x.observed_millis
            || y.wave < x.wave
            || y.tower_hp > x.tower_hp
            || x.started_millis >= 0 && x.started_millis != y.started_millis
            || x.tower_death_revision > 0 && x.tower_death_revision != y.tower_death_revision
            || y.tower_hurt_revision < x.tower_hurt_revision
            || !x.departed.0.iter().all(|id| y.departed.0.contains(id))
        {
            return false;
        }
        let mut copy = x.clone();
        copy.observed_millis = y.observed_millis;
        copy.exchanges = y.exchanges.clone();
        // Tower attack attempts and their latest per-owner exchange can change
        // without a mode transition, but still use engine event revisions.
        event(&copy, y, x.revision, y.revision, a.world.revision)
            && exchanges(
                &x.exchanges.0,
                &y.exchanges.0,
                |e| e.revision,
                |p, q| p.entity == q.entity,
                a.world.revision,
                |_| false,
                |p, q| q.sequence > p.sequence,
            )
    })
}
fn race(a: &UnvalidatedData, b: &UnvalidatedData) -> bool {
    optional(&a.race, &b.race, |x, y| {
        if x.course_sha256 != y.course_sha256
            || x.gates != y.gates
            || x.active_limit_nanos != y.active_limit_nanos
            || y.revision < x.revision
            || (y.phase as u8) < x.phase as u8
            || y.observed_elapsed_nanos < x.observed_elapsed_nanos
            || x.first_finisher_entity > 0
                && (x.first_finisher_entity != y.first_finisher_entity
                    || x.finish_window_deadline_elapsed_nanos
                        != y.finish_window_deadline_elapsed_nanos)
        {
            return false;
        }
        let mut copy = x.clone();
        copy.observed_elapsed_nanos = y.observed_elapsed_nanos;
        if !event(&copy, y, x.revision, y.revision, a.world.revision) {
            return false;
        }
        x.runners
            .0
            .iter()
            .zip(&y.runners.0)
            .zip(a.actors.0.iter().zip(&b.actors.0))
            .all(|((p, q), (u, v))| {
                let progress = (q.completed_laps - p.completed_laps) * x.gates.0.len() as i64
                    + q.next_checkpoint
                    - p.next_checkpoint;
                if p.entity != q.entity
                    || p.base_graphics != q.base_graphics
                    || progress < 0
                    || progress > v.last_move_sequence - u.last_move_sequence
                    || !event(p, q, p.revision, q.revision, a.world.revision)
                    || q.progress_move_sequence < p.progress_move_sequence
                    || progress == 0 && q.progress_move_sequence != p.progress_move_sequence
                    || progress > 0 && q.progress_move_sequence <= u.last_move_sequence
                    || (p.placement > 0 || p.dnf != Dnf::None) && (p != q || u != v)
                {
                    return false;
                }
                if progress == 1 && v.last_move_sequence - u.last_move_sequence == 1 {
                    let g = &x.gates.0[p.next_checkpoint as usize];
                    let inside = |c: &Cell| {
                        (g.min_x..=g.max_x).contains(&c.x) && (g.min_y..=g.max_y).contains(&c.y)
                    };
                    if inside(&u.position) || !inside(&v.position) {
                        return false;
                    }
                }
                true
            })
    })
}
fn npcs(a: &UnvalidatedData, b: &UnvalidatedData) -> bool {
    optional(&a.npcs, &b.npcs, |x, y| {
        if x.names != y.names
            || x.graphics != y.graphics
            || y.revision < x.revision
            || y.elapsed_millis < x.elapsed_millis
            || y.spawned < x.spawned
            || x.closed && x != y
        {
            return false;
        }
        let mut copy = x.clone();
        copy.exchanges = y.exchanges.clone();
        if !event(&copy, y, x.revision, y.revision, x.revision) {
            return false;
        }
        for p in &x.actors.0 {
            let Some(q) = y.actors.0.iter().find(|q| q.serial == p.serial) else {
                // An observation may skip both death and corpse retirement.
                // Never require that the previous poll already saw it dead.
                if !y.closed
                    && (y.revision == x.revision
                        || y.elapsed_millis <= x.elapsed_millis
                        || p.hp == 0 && y.elapsed_millis < p.corpse_until_millis)
                {
                    return false;
                }
                continue;
            };
            if p.source != q.source
                || p.npc_id != q.npc_id
                || p.graphic != q.graphic
                || p.lane != q.lane
                || p.stage != q.stage
                || p.maximum_hp != q.maximum_hp
                || p.maximum_mp != q.maximum_mp
                || p.born_revision != q.born_revision
                || q.life < p.life
                || !event(p, q, p.changed_revision, q.changed_revision, x.revision)
                || p.life == q.life && p.hp == 0 && q.hp > 0
            {
                return false;
            }
        }
        if !y.actors.0.iter().all(|q| {
            x.actors.0.iter().any(|p| p.serial == q.serial)
                || q.serial > x.spawned && q.born_revision > x.revision
        }) {
            return false;
        }
        exchanges(
            &x.exchanges.0,
            &y.exchanges.0,
            |e| e.revision,
            |p, q| {
                p.direction == q.direction
                    && if p.direction == Direction::PlayerToNpc {
                        p.participant == q.participant
                    } else {
                        p.serial == q.serial
                    }
            },
            a.world.revision,
            |p| {
                p.direction == Direction::NpcToParticipant
                    && !y.actors.0.iter().any(|n| n.serial == p.serial)
            },
            |p, q| {
                if p.direction == Direction::PlayerToNpc {
                    q.request_sequence > p.request_sequence
                        && q.participant_life >= p.participant_life
                } else {
                    q.npc_life >= p.npc_life
                }
            },
        )
    })
}
pub(super) fn follows(
    a: &UnvalidatedData,
    b: &UnvalidatedData,
) -> Result<(), app::ApplicationError> {
    ensure(
        a.result
            .as_ref()
            .is_none_or(|r| b.result.as_ref() == Some(r))
            && honor(a, b)
            && siege(a, b)
            && quest(a, b)
            && time(a, b)
            && race(a, b)
            && npcs(a, b),
    )
}

#[cfg(test)]
mod tests {
    use super::exchanges;

    #[test]
    fn retained_attack_exchange_requires_a_new_request_not_only_a_new_revision() {
        // (participant, request sequence, engine revision): mode call sites
        // supply the appropriate per-owner request domain to this shared gate.
        let old = [(1_i64, 7_i64, 20_i64)];
        let accepts = |next: &[(i64, i64, i64)]| {
            exchanges(
                &old,
                next,
                |v| v.2,
                |a, b| a.0 == b.0,
                23,
                |_| false,
                |a, b| b.1 > a.1,
            )
        };
        assert!(accepts(&old));
        assert!(accepts(&[(1, 8, 24)]));
        assert!(accepts(&[(1, 20, 36)]));
        assert!(!accepts(&[(1, 7, 24)]));
        assert!(!accepts(&[(1, 6, 24)]));
        assert!(!accepts(&[(1, 8, 22)]));
        assert!(!accepts(&[]));
    }
}
