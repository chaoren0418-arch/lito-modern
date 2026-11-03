//! Canonical shared-map/mode proof and immutable navigation inputs. This is
//! CPU work for the owned resource/route worker, never the UI/TLS thread.
//! No legacy Frame, predicted movement, command ACK, LOAD ACK or receipt.
use super::{
    battle_follow::Observation,
    battle_payload::data as d,
    siege_space::{Geometry, Regions},
    world_map::{WorldCell, WorldMap},
    world_path::{self, PathLimits, PlannedPath},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

fn cell(c: &d::Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "canonical map qualification cancelled"
    );
    Ok(())
}
fn geometry(s: &d::Siege) -> Geometry {
    Geometry {
        towers: s.towers.0.iter().map(|t| cell(&t.cell)).collect(),
        gate: s.gate.0.iter().map(cell).collect(),
        attack: s.attack_camp.0.iter().map(cell).collect(),
        defense: s.defense_camp.0.iter().map(cell).collect(),
        portal_outside: cell(&s.portal_outside),
        portal_inside: cell(&s.portal_inside),
    }
}
pub(crate) fn occupies(data: &d::UnvalidatedData, entity: i64) -> bool {
    data.time_attack
        .as_ref()
        .is_none_or(|t| !t.departed.0.contains(&entity))
        && data.race.as_ref().is_none_or(|r| {
            r.runners
                .0
                .iter()
                .any(|v| v.entity == entity && v.placement == 0 && v.dnf == d::Dnf::None)
        })
}
fn blocked(data: &d::UnvalidatedData) -> BTreeSet<WorldCell> {
    let mut result = data
        .actors
        .0
        .iter()
        .filter(|a| a.entity != data.personal.entity && occupies(data, a.entity))
        .map(|a| cell(&a.position))
        .collect::<BTreeSet<_>>();
    // Dead player bodies retain the server's occupancy rule; only explicit
    // race/TimeAttack retirement removes them. NPC corpses do not occupy.
    result.extend(data.npcs.iter().flat_map(|n| {
        n.actors
            .0
            .iter()
            .filter(|a| a.hp > 0)
            .map(|a| cell(&a.cell))
    }));
    result.extend(data.honor.objectives.0.iter().map(|t| cell(&t.cell)));
    result.extend(data.quest.iter().flat_map(|q| {
        q.objectives
            .0
            .iter()
            .filter(|t| t.hp > 0)
            .map(|t| cell(&t.target.cell))
    }));
    result.extend(
        data.time_attack
            .iter()
            .filter(|t| t.tower_hp > 0)
            .map(|t| cell(&t.tower.cell)),
    );
    if let Some(s) = &data.siege {
        result.extend(s.towers.0.iter().map(|t| cell(&t.cell)));
        if s.gate_revision == 0 {
            result.extend(s.gate.0.iter().map(cell));
        }
    }
    result
}
fn standing(map: &WorldMap, data: &d::UnvalidatedData, cancel: &AtomicBool) -> Result<()> {
    let mut points = data
        .actors
        .0
        .iter()
        .map(|a| cell(&a.position))
        .collect::<Vec<_>>();
    points.extend(data.honor.objectives.0.iter().map(|t| cell(&t.cell)));
    points.extend(
        data.quest
            .iter()
            .flat_map(|q| q.targets.0.iter().map(|t| cell(&t.cell))),
    );
    points.extend(
        data.quest
            .iter()
            .flat_map(|q| q.lives.0.iter().map(|l| cell(&l.spawn))),
    );
    points.extend(data.time_attack.iter().map(|t| cell(&t.tower.cell)));
    points.extend(
        data.npcs
            .iter()
            .flat_map(|n| n.actors.0.iter().map(|a| cell(&a.cell))),
    );
    for p in points {
        active(cancel)?;
        ensure!(
            map.standing(p),
            "canonical participant/objective/spawn outside shared terrain"
        );
    }
    Ok(())
}
pub(crate) struct Space {
    baseline: Observation,
    map: Arc<WorldMap>,
    siege: Option<Regions>,
}
impl Space {
    pub(crate) fn qualify(
        map: Arc<WorldMap>,
        observation: Observation,
        cancel: &AtomicBool,
    ) -> Result<Arc<Self>> {
        active(cancel)?;
        let data = observation.data();
        ensure!(
            data.world.phase != d::Phase::Aborted,
            "aborted world cannot qualify navigation"
        );
        ensure!(
            i32::try_from(map.map_id())? == data.manifest.map
                && map.canonical_digest() == data.manifest.digest,
            "canonical geometry binding mismatch"
        );
        let bindings = map.bindings();
        ensure!(
            bindings.len() == data.manifest.sectors.0.len()
                && bindings
                    .iter()
                    .zip(&data.manifest.sectors.0)
                    .all(|(b, s)| i32::from(b.key.sector_x) == s.x
                        && i32::from(b.key.sector_y) == s.y
                        && i32::try_from(b.byte_length).ok() == Some(s.byte_length)
                        && b.sha256 == s.sha256),
            "canonical raw-sector manifest mismatch"
        );
        standing(&map, data, cancel)?;
        let initial = matches!(data.world.phase, d::Phase::Loading | d::Phase::Countdown);
        let starts = data
            .actors
            .0
            .iter()
            .map(|a| cell(&a.position))
            .collect::<Vec<_>>();
        if initial {
            map.validate_spawns(&starts, cancel)?;
        }
        let siege = data
            .siege
            .as_ref()
            .map(|s| Regions::qualify(&map, &geometry(s), cancel))
            .transpose()?;
        if let Some(race) = &data.race {
            let gates = race
                .gates
                .0
                .iter()
                .map(|g| super::race_protocol::Gate {
                    min_x: g.min_x,
                    max_x: g.max_x,
                    min_y: g.min_y,
                    max_y: g.max_y,
                })
                .collect::<Vec<_>>();
            super::race_space::qualify(&map, &gates, &starts, initial, cancel)?;
        }
        active(cancel)?;
        Ok(Arc::new(Self {
            baseline: observation,
            map,
            siege,
        }))
    }
    /// The follower proves the static policy, owner, manifest and generation
    /// are unchanged. Recheck dynamic cells; never flood-fill on each frame.
    pub(crate) fn snapshot(
        self: &Arc<Self>,
        observation: Observation,
        cancel: &AtomicBool,
    ) -> Result<Arc<Navigation>> {
        active(cancel)?;
        self.baseline.check_successor(&observation)?;
        ensure!(
            observation.data().world.phase != d::Phase::Aborted,
            "aborted navigation must retire"
        );
        standing(&self.map, observation.data(), cancel)?;
        let blocked = blocked(observation.data());
        active(cancel)?;
        Ok(Arc::new(Navigation {
            space: self.clone(),
            observation,
            blocked,
        }))
    }
}
/// An exact immutable observation + qualified map + current occupancy. A
/// route is intent only: Session must still bind its attachment/owner, reject
/// stale observations and wait for each server MOVE ACK before advancing.
pub struct Navigation {
    space: Arc<Space>,
    observation: Observation,
    blocked: BTreeSet<WorldCell>,
}
impl Navigation {
    pub fn observation(&self) -> &Observation {
        &self.observation
    }
    pub fn matches(&self, observation: &Observation) -> bool {
        std::ptr::eq(self.observation.data(), observation.data())
    }
    pub fn map(&self) -> &WorldMap {
        &self.space.map
    }
    /// Share the already-qualified immutable map with an owned route worker.
    pub fn shared_map(&self) -> Arc<WorldMap> {
        self.space.map.clone()
    }
    pub fn blocked(&self) -> &BTreeSet<WorldCell> {
        &self.blocked
    }
    /// A rendered map may share the owner's pose while its gate, objective or
    /// other occupants have already changed. Time/revision-only updates do not
    /// invalidate it, but commands must wait for changed blocker geometry.
    /// This is a bounded set comparison, with no I/O or terrain qualification.
    pub(crate) fn blockers_current(&self, current: &Observation) -> bool {
        let before = self.observation.data();
        let after = current.data();
        before.world.identity == after.world.identity
            && before.personal.player == after.personal.player
            && before.personal.entity == after.personal.entity
            && before.manifest == after.manifest
            && self.blocked == blocked(after)
    }
    fn mover(&self) -> Result<&d::Actor> {
        let data = self.observation.data();
        ensure!(
            data.world.phase == d::Phase::Active && occupies(data, data.personal.entity),
            "canonical owner is not active"
        );
        let a = data
            .actors
            .0
            .iter()
            .find(|a| a.entity == data.personal.entity)
            .context("canonical owner absent")?;
        ensure!(
            a.vital.as_ref().is_none_or(|v| v.values.hp > 0),
            "dead canonical owner cannot move"
        );
        Ok(a)
    }
    /// CPU-only bounded path search; callers must use an owned route worker.
    pub fn plan(
        &self,
        goal: WorldCell,
        limits: PathLimits,
        cancel: &AtomicBool,
    ) -> Result<PlannedPath> {
        active(cancel)?;
        let actor = self.mover()?;
        Ok(world_path::plan(
            &self.space.map,
            cell(&actor.position),
            goal,
            &self.blocked,
            limits,
            cancel,
        )?)
    }
    pub fn can_step(&self, from: WorldCell, to: WorldCell) -> bool {
        self.mover().is_ok() && world_path::step_open(&self.space.map, from, to, &self.blocked)
    }
    /// Static/local candidate check, never a teleport destination or spell
    /// permission. Only the server's receipt may change the actor position.
    pub fn teleport_allowed(&self, from: WorldCell, to: WorldCell) -> bool {
        self.mover().is_ok()
            && self.space.map.standing(from)
            && self.space.map.standing(to)
            && !self.blocked.contains(&from)
            && !self.blocked.contains(&to)
            && self.space.siege.as_ref().is_none_or(|s| {
                s.allows(
                    from,
                    to,
                    self.observation
                        .data()
                        .siege
                        .as_ref()
                        .is_some_and(|s| s.gate_revision > 0),
                )
            })
    }
}
impl std::fmt::Debug for Navigation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CanonicalNavigation[redacted]")
    }
}

#[cfg(test)]
#[path = "battle_space_tests.rs"]
pub(crate) mod tests;
