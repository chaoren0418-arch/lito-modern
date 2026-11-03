//! NPC namespace, resident roster and retained attack facts. Retired corpses
//! may remain in historical target receipts, but never as live draw objects.
use super::{
    combat::require,
    data::*,
    invalid,
    scene::{initial, ordered},
    ApplicationError,
};
use crate::modern_lito::npc_assets::NpcAppearance;
use std::collections::BTreeSet;
fn label(s: &str, empty: bool) -> bool {
    s.len() <= 256
        && (empty && s.is_empty() || !s.trim().is_empty())
        && !s.chars().any(char::is_control)
}
impl UnvalidatedData {
    pub(super) fn check_npcs(&self) -> Result<(), ApplicationError> {
        let Some(n) = &self.npcs else {
            return Ok(());
        };
        require(
            n.revision >= 1
                && n.revision <= self.world.revision
                && (0..=1_860_000).contains(&n.elapsed_millis)
                && n.spawned >= 0
                && n.closed == (self.world.phase == Phase::Aborted)
                && (!n.closed || n.actors.0.is_empty())
                && (!initial(self.world.phase)
                    || n.spawned == 0 && n.elapsed_millis == 0 && n.exchanges.0.is_empty())
                && (self.world.phase != Phase::Finished
                    || self.quest.is_some()
                    || self.time_attack.is_some())
                && !n.names.0.is_empty()
                && !n.graphics.0.is_empty()
                && n.names
                    .0
                    .windows(2)
                    .all(|p| (p[0].source, p[0].npc_id) < (p[1].source, p[1].npc_id)),
        )?;
        for name in &n.names.0 {
            require(
                name.source > 0
                    && name.npc_id > 0
                    && label(&name.name.0, false)
                    && label(&name.name_id.0, true)
                    && u32::try_from(name.graphic)
                        .ok()
                        .is_some_and(|g| NpcAppearance::new(g).is_ok()),
            )?;
        }
        let isolated = self.world.lane >= 0;
        let cell_key = |lane: i32, c: &Cell| (if isolated { lane } else { 0 }, c.x, c.y);
        let mut occupied = BTreeSet::new();
        for a in &self.actors.0 {
            if !self
                .time_attack
                .as_ref()
                .is_some_and(|t| t.departed.0.contains(&a.entity))
            {
                occupied.insert(cell_key(self.scene_member(a)?.team, &a.position));
            }
        }
        if let Some(t) = &self.time_attack {
            if t.tower_hp > 0 {
                occupied.insert(cell_key(0, &t.tower.cell));
            }
        }
        if let Some(q) = &self.quest {
            for o in &q.objectives.0 {
                if o.hp > 0 {
                    occupied.insert(cell_key(o.target.lane, &o.target.cell));
                }
            }
        }
        let mut previous = 0;
        for a in &n.actors.0 {
            require(
                a.serial > previous
                    && a.serial <= n.spawned
                    && (0..=1).contains(&a.lane)
                    && (1..=4).contains(&a.stage)
                    && (0..=7).contains(&a.heading)
                    && a.maximum_hp >= 1
                    && (0..=a.maximum_hp).contains(&a.hp)
                    && a.maximum_mp >= 0
                    && (0..=a.maximum_mp).contains(&a.mp)
                    && a.life > 0
                    && a.born_revision > 0
                    && a.changed_revision >= a.born_revision
                    && a.changed_revision <= n.revision
                    && [
                        a.corpse_until_millis,
                        a.next_move_millis,
                        a.next_attack_millis,
                    ]
                    .iter()
                    .all(|v| (0..=1_920_000).contains(v))
                    && (a.hp == 0) == (a.action == NpcAction::Death)
                    && (a.hp == 0) == (a.corpse_until_millis > 0)
                    && (a.hp != 0
                        || a.next_move_millis == 0
                            && a.next_attack_millis == 0
                            && a.corpse_until_millis > n.elapsed_millis)
                    && self.manifest.contains_cell(&a.cell)
                    && (a.hp == 0 || occupied.insert(cell_key(a.lane, &a.cell))),
            )?;
            previous = a.serial;
        }
        let mut humans = BTreeSet::new();
        let mut attackers = BTreeSet::new();
        let mut previous = 0;
        for e in &n.exchanges.0 {
            let a = self.identity_actor(&e.participant)?;
            let v = a.vital.as_ref().ok_or_else(invalid)?;
            let o = &e.outcome;
            require(
                e.serial > 0
                    && e.serial <= n.spawned
                    && e.npc_life > 0
                    && e.participant_life > 0
                    && e.revision > previous
                    && e.revision <= self.world.revision
                    && o.damage >= 0
                    && o.applied_damage >= 0
                    && o.applied_damage <= o.damage
                    && o.remaining_hp >= 0
                    && (1..=i128::from(i64::MAX))
                        .contains(&(i128::from(o.remaining_hp) + i128::from(o.applied_damage)))
                    && (0..=20).contains(&o.material_bonus)
                    && (0..=4).contains(&o.blessing_bonus)
                    && o.death == (o.remaining_hp == 0 && o.applied_damage > 0)
                    && (o.hit
                        || o.raw_damage == 0
                            && o.damage == 0
                            && o.applied_damage == 0
                            && !o.death
                            && !o.maximum_triggered
                            && !o.critical_triggered
                            && o.material_bonus == 0
                            && o.blessing_bonus == 0),
            )?;
            if let Some(l) = self
                .quest
                .as_ref()
                .and_then(|q| q.lives.0.iter().find(|l| l.entity == a.entity))
            {
                let incarnation = l.respawns.checked_add(1).ok_or_else(invalid)?;
                require(
                    e.participant_life <= incarnation
                        && if e.participant_life == incarnation {
                            e.revision > l.respawn_revision
                        } else {
                            e.revision <= l.respawn_revision
                        },
                )?;
            } else {
                require(e.participant_life == 1)?;
            }
            let target = n.actors.0.iter().find(|a| a.serial == e.serial);
            let team = self.scene_member(a)?.team;
            require(target.is_none_or(|a| e.npc_life <= a.life && (!isolated || a.lane == team)))?;
            match e.direction {
                Direction::PlayerToNpc => {
                    require(
                        e.participant.kind == MemberKind::Human
                            && e.request_sequence > 0
                            && e.request_sequence <= v.attack_sequence
                            && humans.insert(a.entity),
                    )?;
                    if e.revision == self.world.revision {
                        require(
                            target.is_some_and(|n| n.life == e.npc_life && n.hp == o.remaining_hp)
                                && v.values.hp > 0
                                && e.request_sequence == v.attack_sequence,
                        )?;
                    }
                }
                Direction::NpcToParticipant => {
                    require(
                        e.request_sequence == 0 && attackers.insert(e.serial) && target.is_some(),
                    )?;
                    if e.revision == self.world.revision {
                        require(
                            v.values.hp == o.remaining_hp
                                && target.is_some_and(|n| {
                                    n.life == e.npc_life && n.action == NpcAction::Attack
                                })
                                && (o.applied_damage == 0 || v.hurt_revision == e.revision),
                        )?;
                    }
                }
            }
            previous = e.revision;
        }
        for caster in &self.actors.0 {
            let impacts = caster
                .spells
                .histories
                .0
                .iter()
                .filter_map(|s| s.receipt.as_ref().map(|r| &r.impact))
                .chain(caster.spells.structure.as_ref().map(|s| &s.impact));
            for impact in impacts {
                for effect in &impact.targets.0 {
                    let Target::Npc { serial, .. } = &effect.target else {
                        continue;
                    };
                    require(*serial > 0 && *serial <= n.spawned)?;
                    let current = n.actors.0.iter().find(|n| n.serial == *serial);
                    if let Some(target) = current {
                        require(
                            effect.life <= target.life
                                && (effect.life != target.life
                                    || effect.maximum_hp == target.maximum_hp)
                                && (!isolated || target.lane == self.scene_member(caster)?.team),
                        )?;
                    }
                    if impact.usage.revision == self.world.revision {
                        require(
                            current
                                .is_some_and(|n| n.life == effect.life && n.hp == effect.hp_after)
                                && caster.vital.as_ref().is_some_and(|v| v.values.hp > 0),
                        )?;
                    }
                }
            }
        }
        // A living list is an exact set of current NPC identities, not a count.
        if let Some(q) = &self.quest {
            for lane in &q.lanes.0 {
                require(ordered(&lane.living.0, n.spawned))?;
                if !n.closed {
                    let living = n
                        .actors
                        .0
                        .iter()
                        .filter(|n| n.hp > 0 && n.lane == lane.lane)
                        .collect::<Vec<_>>();
                    require(
                        living.iter().map(|n| n.serial).collect::<Vec<_>>() == lane.living.0
                            && living.iter().all(|n| n.stage == lane.stage),
                    )?;
                }
            }
        }
        if let Some(t) = &self.time_attack {
            require(
                ordered(&t.living.0, n.spawned)
                    && n.actors.0.iter().all(|n| n.lane == 0 && n.life == 1),
            )?;
            if !n.closed {
                let stage = match t
                    .waves
                    .0
                    .get(usize::try_from(t.wave).map_err(|_| invalid())?)
                    .ok_or_else(invalid)?
                {
                    TimeKind::Ordinary => 1,
                    TimeKind::FinalBosses => 2,
                    TimeKind::Dragon => 3,
                };
                let living = n.actors.0.iter().filter(|n| n.hp > 0).collect::<Vec<_>>();
                require(
                    living.iter().map(|n| n.serial).collect::<Vec<_>>() == t.living.0
                        && living.iter().all(|n| n.stage == stage),
                )?;
            }
        }
        Ok(())
    }
}
