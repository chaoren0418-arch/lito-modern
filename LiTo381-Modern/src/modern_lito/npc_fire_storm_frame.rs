//! Candidate atomic NPC70 authority. A payload receipt alone proves neither
//! mana, caster life, target HP nor a mode's result. Not application admission.
use super::{
    application_protocol::{self as app, ApplicationError},
    combat_protocol as combat,
    npc_fire_storm_wire::{self as wire, QuestKind, State, Structure, StructureId},
    npc_protocol as npc, quest_protocol as quest,
    world_map::WorldCell,
    world_protocol::{self as world, MatchDescriptor as D},
};
use std::collections::{BTreeMap, BTreeSet};

fn bad() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid NPC70 atomic frame")
}
fn wire_revision(revision: i64) -> Result<i64, ApplicationError> {
    if revision == 0 {
        Ok(0)
    } else {
        revision.checked_add(1).ok_or_else(bad)
    }
}
fn event_revision(f: &combat::Frame) -> Result<i64, ApplicationError> {
    if let Some(r) = f
        .world
        .time_attack
        .as_ref()
        .and_then(|d| d.receipt.as_ref())
    {
        wire_revision(r.revision)
    } else if let Some(c) = &f.world.completion {
        wire_revision(c.event.engine_revision)
    } else {
        Ok(f.world.revision)
    }
}
fn casts(f: &combat::Frame, entity: u64) -> i128 {
    i128::from(f.world.cast(entity).map_or(0, |s| s.cast_count))
        + i128::from(f.npc_spell(entity).map_or(0, |s| s.cast_count))
        + i128::from(f.npc_disintegrate(entity).map_or(0, |s| s.count))
        + i128::from(f.npc_fire(entity).map_or(0, |s| s.count))
}
pub(crate) fn validate_frame(f: &combat::Frame) -> Result<(), ApplicationError> {
    if f.npc_fires.is_empty() {
        return Ok(());
    }
    let w = &f.world;
    let n = w.actors.len();
    if n == 0
        || n > wire::MAX_ACTORS
        || f.npc_fires.len() != n
        || f.vitals.len() != n
        || w.casts.len() != n
        || f.items.len() != n
        || f.regen.len() != n
        || f.weights.len() != n
        || !f.npc_spells.is_empty() && f.npc_spells.len() != n
        || !f.npc_disintegrations.is_empty() && f.npc_disintegrations.len() != n
        || w.descriptor
            .is_none_or(|d| !d.has(D::NPC_FIRE_STORM | D::NPC | D::SPELLS | D::REGEN))
        || !f.disintegrations.is_empty()
        || !f.damages.is_empty()
        || !f.timed.is_empty()
        || !f.ammo.is_empty()
        || !f.fires.is_empty()
        || f.honor.is_some()
        || w.siege.is_some()
        || w.room.summary.settings.mode != if w.time_attack.is_some() { 4 } else { 2 }
    {
        return Err(bad());
    }
    let mut occupied = BTreeSet::new();
    for s in &w.casts {
        occupied.insert(s.cast_revision);
    }
    for s in &f.npc_spells {
        occupied.insert(s.revision);
    }
    for s in &f.npc_disintegrations {
        occupied.insert(s.revision);
    }
    for s in &f.items {
        occupied.insert(s.use_revision);
    }
    for s in &f.vitals {
        occupied.extend([s.attack_revision, s.hurt_revision, s.death_revision]);
    }
    for s in &f.regen {
        occupied.extend([s.hp_revision, s.mp_revision]);
    }
    if let Some(q) = &w.quest {
        for l in &q.lives {
            occupied.extend([l.death_revision, l.respawn_revision]);
        }
        for e in &q.exchanges {
            occupied.insert(e.revision);
        }
    }
    if let Some(t) = &w.time_attack {
        for e in &t.exchanges {
            occupied.insert(wire_revision(e.revision)?);
        }
    }
    let end = event_revision(f)?;
    let scope = quest::scope(w);
    let mut owners = BTreeMap::new();
    for (s, a) in f.npc_fires.iter().zip(&w.actors) {
        s.validate()?;
        let c = w.cast(a.entity_id).ok_or_else(bad)?;
        if s.entity_id != a.entity_id
            || s.revision > end
            || s.last_sequence > c.request_sequence
            || casts(f, s.entity_id) > i128::from(c.request_sequence)
            || if a.identity().character().is_some() {
                s.mp_cost == 0
            } else {
                *s != State::initial(s.entity_id, 0)
            }
            || matches!(w.phase, world::Phase::Loading | world::Phase::Countdown) && s.count != 0
        {
            return Err(bad());
        }
        if s.count == 0 {
            continue;
        }
        let origin = s.origin.ok_or_else(bad)?;
        if s.scope != Some(scope)
            || s.lane != w.member(s.entity_id).map(|m| m.team)
            || occupied.contains(&s.revision)
            || owners.insert(s.revision, s.entity_id).is_some()
            || !w.manifest.contains(origin)
            || s.targets.iter().any(|h| !w.manifest.contains(h.cell))
        {
            return Err(bad());
        }
        caster(
            f,
            s.entity_id,
            s.revision,
            s.last_sequence,
            s.caster_life,
            origin,
            end,
        )?;
    }
    for s in &f.npc_fires {
        let Some(e) = &s.structure else { continue };
        if owners
            .insert(e.revision, s.entity_id)
            .is_some_and(|id| id != s.entity_id)
            || occupied.contains(&e.revision)
            || e.id.scope() != scope
            || !w.manifest.contains(e.origin)
            || !w.manifest.contains(e.cell)
        {
            return Err(bad());
        }
        caster(
            f,
            s.entity_id,
            e.revision,
            e.sequence,
            e.caster_life,
            e.origin,
            end,
        )?;
        match e.id {
            StructureId::Quest {
                id,
                kind,
                born_revision,
            } => quest_hit(w, e, id, kind, born_revision)?,
            StructureId::Tower(_) => tower_hit(w, e)?,
        }
    }
    if let Some(q) = &w.quest {
        for a in &q.actors {
            for revision in [a.hurt_revision, a.death_revision] {
                if revision == 0 {
                    continue;
                }
                let Some(owner) = owners.get(&revision) else {
                    continue;
                };
                let receipt = f
                    .npc_fire(*owner)
                    .and_then(|s| s.structure.as_ref())
                    .ok_or_else(bad)?;
                if receipt.revision != revision
                    || !matches!(receipt.id, StructureId::Quest { id, .. } if id.slot == a.slot)
                {
                    return Err(bad());
                }
            }
        }
        if let Some(winner) = q
            .winning_lane
            .filter(|lane| w.audience_lane.is_none_or(|v| v == *lane))
        {
            let slot = winner * 4 + 4;
            if !q
                .exchanges
                .iter()
                .any(|e| e.slot == slot && e.revision == q.death_revision && e.outcome.death)
                && !f
                    .npc_fires
                    .iter()
                    .filter_map(|s| s.structure.as_ref())
                    .any(|e| {
                        e.revision == q.death_revision
                            && e.outcome.death
                            && matches!(e.id, StructureId::Quest { id, .. } if id.slot == slot)
                    })
            {
                return Err(bad());
            }
        }
    }
    if let Some(t) = &w.time_attack {
        let hurt = wire_revision(t.tower_hurt_revision)?;
        let death = wire_revision(t.tower_death_revision)?;
        for s in &f.npc_fires {
            if s.count > 0
                && (s.revision == hurt || s.revision == death)
                && s.structure.as_ref().is_none_or(|e| {
                    e.revision != s.revision || !matches!(e.id, StructureId::Tower(_))
                })
            {
                return Err(bad());
            }
        }
        if death > 0
            && !t
                .exchanges
                .iter()
                .any(|e| e.revision == t.tower_death_revision && e.flags & 2 != 0)
            && !f
                .npc_fires
                .iter()
                .filter_map(|s| s.structure.as_ref())
                .any(|e| {
                    e.revision == death && e.outcome.death && matches!(e.id, StructureId::Tower(_))
                })
        {
            return Err(bad());
        }
    }
    Ok(())
}
fn caster(
    f: &combat::Frame,
    entity: u64,
    revision: i64,
    sequence: u64,
    life: u64,
    origin: WorldCell,
    end: i64,
) -> Result<(), ApplicationError> {
    let w = &f.world;
    let a = w
        .actors
        .iter()
        .find(|a| a.entity_id == entity)
        .ok_or_else(bad)?;
    let v = f.vital(entity).ok_or_else(bad)?;
    let c = w.cast(entity).ok_or_else(bad)?;
    let state = super::quest_life::life(w, entity);
    let current = state.map_or(1, |l| l.incarnation());
    if a.identity().character().is_none()
        || life > current
        || state.is_some_and(|l| {
            if life == current {
                revision <= l.respawn_revision
            } else {
                revision > l.respawn_revision
            }
        })
        || life == current && v.hp == 0 && revision >= v.death_revision
        || revision == end
            && (v.hp == 0
                || sequence != c.request_sequence
                || life != current
                || a.position != origin
                || w.time_attack.as_ref().is_some_and(|t| t.departed(entity)))
    {
        return Err(bad());
    }
    Ok(())
}
fn quest_hit(
    w: &world::Frame,
    e: &Structure,
    id: quest::Id,
    kind: QuestKind,
    born: i64,
) -> Result<(), ApplicationError> {
    let q = w.quest.as_ref().ok_or_else(bad)?;
    if w.time_attack.is_some() {
        return Err(bad());
    }
    let lane = (id.slot - 1) / 4;
    let stage = (id.slot - 1) % 4 + 1;
    let target = q.target(id.slot).ok_or_else(bad)?;
    let actor = q.actor(id.slot);
    let current = q.actors.iter().find(|a| a.lane() == lane);
    let progress = q.lanes.iter().find(|l| l.lane == lane).ok_or_else(bad)?;
    let expected = if q.flow == quest::Flow::FinalGuardian {
        QuestKind::FinalGuardian
    } else if stage == 4 {
        QuestKind::FinalSymbol
    } else {
        QuestKind::RoomSymbol
    };
    if kind != expected
        || e.cell != target.cell
        || e.maximum_hp != target.max_hp
        || stage > progress.stage
        || actor.is_none() && stage == progress.stage
        || current.is_some_and(|a| stage < a.stage() && e.revision >= a.born_revision)
        || actor.is_some_and(|a| {
            born != a.born_revision
                || e.revision > a.revision
                || e.outcome.remaining_hp < a.hp
                || e.outcome.death && a.death_revision != e.revision
                || e.revision == a.revision
                    && (a.hp != e.outcome.remaining_hp
                        || e.outcome.applied_damage > 0 && a.hurt_revision != e.revision)
        })
    {
        return Err(bad());
    }
    Ok(())
}
fn tower_hit(w: &world::Frame, e: &Structure) -> Result<(), ApplicationError> {
    let t = w.time_attack.as_ref().ok_or_else(bad)?;
    let revision = wire_revision(t.revision)?;
    let hurt = wire_revision(t.tower_hurt_revision)?;
    let death = wire_revision(t.tower_death_revision)?;
    if w.quest.is_some()
        || e.cell != t.tower_cell
        || e.maximum_hp != t.tower_max_hp
        || e.revision > revision
        || e.outcome.remaining_hp < t.tower_hp
        || e.outcome.death && death != e.revision
        || e.revision == hurt && e.outcome.remaining_hp != t.tower_hp
        || e.revision == revision
            && (e.outcome.remaining_hp != t.tower_hp
                || e.outcome.applied_damage > 0 && hurt != e.revision)
    {
        return Err(bad());
    }
    Ok(())
}
pub(crate) fn validate_npcs(f: &npc::Frame) -> Result<(), ApplicationError> {
    let end = event_revision(&f.combat)?;
    for s in &f.combat.npc_fires {
        if f.exchanges.iter().any(|e| {
            e.revision == s.revision
                || s.structure
                    .as_ref()
                    .is_some_and(|h| h.revision == e.revision)
        }) {
            return Err(bad());
        }
        for h in &s.targets {
            if h.id.scope != f.scope || h.id.serial > f.spawned {
                return Err(bad());
            }
            let target = f.actor(h.id);
            if target.is_some_and(|n| {
                n.lane != h.lane
                    || n.life < h.life
                    || h.hp_before > n.max_hp
                    || n.life == h.life && h.outcome.remaining_hp < n.hp
            }) || s.revision == end
                && target.is_none_or(|n| {
                    n.life != h.life
                        || n.cell != h.cell
                        || n.hp != h.outcome.remaining_hp
                        || h.outcome.applied_damage > 0
                            && n.action
                                != if h.outcome.death {
                                    npc::NpcAction::Death
                                } else {
                                    npc::NpcAction::Hurt
                                }
                })
            {
                return Err(bad());
            }
        }
    }
    Ok(())
}
pub(crate) fn frames_follow(a: &combat::Frame, b: &combat::Frame) -> bool {
    wire::follows(
        quest::scope(&a.world),
        &a.npc_fires,
        a.world.revision,
        quest::scope(&b.world),
        &b.npc_fires,
        b.world.revision,
    ) && a.npc_fires.iter().all(|s| {
        a.world
            .cast(s.entity_id)
            .zip(b.world.cast(s.entity_id))
            .is_some_and(|(x, y)| {
                y.request_sequence
                    .checked_sub(x.request_sequence)
                    .is_some_and(|budget| {
                        casts(b, s.entity_id) - casts(a, s.entity_id) <= i128::from(budget)
                    })
            })
    })
}

#[cfg(test)]
#[path = "npc_fire_storm_frame_tests.rs"]
mod tests;
