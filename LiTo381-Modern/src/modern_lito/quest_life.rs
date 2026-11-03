//! v22 Quest participant life. Confirmation restores a server-qualified spawn,
//! never the Honor timer or an optimistic client HP/position update.
use super::{
    application_protocol::{self as app, ApplicationError},
    combat_protocol as combat,
    world_map::WorldCell,
    world_protocol as world,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub entity_id: u64,
    pub lane: u8,
    pub spawn: WorldCell,
    pub deaths: i64,
    pub death_revision: i64,
    pub respawns: i64,
    pub respawn_revision: i64,
    pub hp_gain: i64,
    pub death_cell: WorldCell,
}
impl State {
    pub fn incarnation(&self) -> u64 {
        self.respawns
            .checked_add(1)
            .and_then(|v| u64::try_from(v).ok())
            .unwrap_or(0)
    }
    pub fn dead(&self) -> bool {
        self.deaths > self.respawns
    }
    fn valid(&self) -> bool {
        (1..=100).contains(&self.entity_id)
            && self.lane < 2
            && self.deaths >= 0
            && self.respawns >= 0
            && self.respawns < i64::MAX
            && self.respawns <= self.deaths
            && self.deaths - self.respawns <= 1
            && self.death_revision >= 0
            && self.respawn_revision >= 0
            && self.hp_gain >= 0
            && (self.deaths == 0) == (self.death_revision == 0)
            && (self.respawns == 0) == (self.respawn_revision == 0)
            && (self.respawns == 0) == (self.hp_gain == 0)
            && (self.deaths != 0 || self.death_cell == WorldCell { x: 0, y: 0 })
            && (self.deaths == 0
                || if self.dead() {
                    self.death_revision > self.respawn_revision
                } else {
                    self.respawn_revision > self.death_revision
                })
    }
}
pub fn life(world: &world::Frame, entity: u64) -> Option<&State> {
    world
        .quest
        .as_ref()?
        .lives
        .iter()
        .find(|l| l.entity_id == entity)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Confirmation {
    pub scope: super::npc_protocol::Scope,
    pub entity_id: u64,
    pub incarnation: u64,
    pub death_revision: i64,
}
pub fn confirmation(frame: &combat::Frame, entity: u64) -> Option<Confirmation> {
    let l = life(&frame.world, entity)?;
    (frame.world.phase == world::Phase::Active && l.dead() && frame.vital(entity)?.hp == 0)
        .then_some(Confirmation {
            scope: super::quest_protocol::scope(&frame.world),
            entity_id: entity,
            incarnation: l.incarnation(),
            death_revision: l.death_revision,
        })
}
pub(crate) fn validate(world: &world::Frame) -> Result<(), ApplicationError> {
    let rows = world.quest.as_ref().map_or(&[][..], |q| q.lives.as_slice());
    if !rows.is_empty()
        != world
            .descriptor
            .is_some_and(|d| d.has(world::MatchDescriptor::QUEST_LIVES))
        || !rows.is_empty() && rows.len() != world.actors.len()
        || rows.len() > 100
    {
        return Err(invalid());
    }
    for (l, a) in rows.iter().zip(&world.actors) {
        if !l.valid()
            || l.entity_id != a.entity_id
            || world.member(a.entity_id).is_none_or(|m| l.lane != m.team)
            || l.death_revision > world.revision
            || l.respawn_revision > world.revision
            || !world.manifest.contains(l.spawn)
            || l.deaths > 0 && !world.manifest.contains(l.death_cell)
            || l.respawn_revision == world.revision && a.position != l.spawn
            || l.death_revision == world.revision && a.position != l.death_cell
            || matches!(
                world.phase,
                world::Phase::Preparing | world::Phase::Loading | world::Phase::Countdown
            ) && (l.deaths != 0 || l.spawn != a.position)
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub(crate) fn validate_combat(f: &combat::Frame) -> Result<(), ApplicationError> {
    for l in f
        .world
        .quest
        .as_ref()
        .map_or(&[][..], |q| q.lives.as_slice())
    {
        let v = f.vital(l.entity_id).ok_or_else(invalid)?;
        if l.dead() != (v.hp == 0)
            || l.death_revision != v.death_revision
            || i128::from(v.max_hp) * i128::from(l.respawns) != i128::from(l.hp_gain)
            || l.respawn_revision == f.world.revision && v.hp != v.max_hp
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub(crate) fn follows(a: &world::Frame, b: &world::Frame) -> bool {
    let old = a.quest.as_ref().map_or(&[][..], |q| q.lives.as_slice());
    let new = b.quest.as_ref().map_or(&[][..], |q| q.lives.as_slice());
    if old.len() != new.len() {
        return false;
    }
    old.iter().zip(new).all(|(x, y)| {
        if x.entity_id != y.entity_id
            || x.lane != y.lane
            || x.spawn != y.spawn
            || y.deaths < x.deaths
            || y.respawns < x.respawns
            || y.hp_gain < x.hp_gain
        {
            return false;
        }
        if y.deaths == x.deaths {
            if y.death_revision != x.death_revision || y.death_cell != x.death_cell {
                return false;
            }
        } else if y.death_revision <= a.revision {
            return false;
        }
        if y.respawns == x.respawns {
            if y.respawn_revision != x.respawn_revision || y.hp_gain != x.hp_gain {
                return false;
            }
        } else if y.respawn_revision <= a.revision || y.hp_gain == x.hp_gain {
            return false;
        }
        let Some(old_actor) = a.actors.iter().find(|v| v.entity_id == x.entity_id) else {
            return false;
        };
        let Some(new_actor) = b.actors.iter().find(|v| v.entity_id == y.entity_id) else {
            return false;
        };
        old_actor
            .sequence
            .checked_add((y.respawns - x.respawns) as u64)
            .is_some_and(|minimum| new_actor.sequence >= minimum)
            && (!x.dead() || y.respawns != x.respawns || new_actor.position == old_actor.position)
    })
}
// Common read-only presentation/ledger queries. The authorities remain separate.
pub fn respawn_gain(f: &combat::Frame, entity: u64) -> i64 {
    if let Some(l) = super::siege_protocol::life(&f.world, entity) {
        return l.hp_gain;
    }
    life(&f.world, entity).map_or_else(
        || super::honor_protocol::respawn_gain(f, entity),
        |l| l.hp_gain,
    )
}
pub fn respawn_revision(f: &combat::Frame, entity: u64) -> i64 {
    if let Some(l) = super::siege_protocol::life(&f.world, entity) {
        return if l.respawn_revision == 0 {
            0
        } else {
            l.respawn_revision.checked_add(1).unwrap_or(i64::MAX)
        };
    }
    life(&f.world, entity).map_or_else(
        || super::honor_protocol::respawn_revision(f, entity),
        |l| l.respawn_revision,
    )
}
pub fn respawned(a: &combat::Frame, b: &combat::Frame, entity: u64) -> bool {
    if a.world.siege.is_some() || b.world.siege.is_some() {
        return super::siege_protocol::respawned(&a.world, &b.world, entity);
    }
    life(&a.world, entity)
        .zip(life(&b.world, entity))
        .map_or_else(
            || super::honor_protocol::respawned(a, b, entity),
            |(x, y)| y.respawns > x.respawns,
        )
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid Quest participant life",
    )
}
#[cfg(test)]
#[path = "quest_life_tests.rs"]
pub(crate) mod tests;
