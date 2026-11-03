//! Respawn accounting uses actual grants across equipment eras, never current
//! maxHP multiplied by respawn count. Due timers do not grant HP by themselves.
use super::{combat::require, data::*, invalid, scene::initial, ApplicationError};
use crate::modern_lito::world_protocol::MatchDescriptor as F;
struct Life {
    entity: i64,
    deaths: i64,
    death: i64,
    respawns: i64,
    respawn: i64,
    granted: i64,
}
impl UnvalidatedData {
    pub(super) fn check_lives(&self) -> Result<(), ApplicationError> {
        let honor = !self.honor.objectives.0.is_empty();
        require(honor || self.honor.lives.0.is_empty())?;
        let quest = self.world.descriptor.features as u32 & F::QUEST_LIVES != 0;
        let siege = self.siege.is_some();
        require(u8::from(honor) + u8::from(quest) + u8::from(siege) <= 1)?;
        if honor {
            require(self.honor.lives.0.len() == self.actors.0.len())?;
            for (l, a) in self.honor.lives.0.iter().zip(&self.actors.0) {
                self.life(
                    a,
                    Life {
                        entity: l.entity,
                        deaths: l.death_count,
                        death: l.last_death_revision,
                        respawns: l.respawn_count,
                        respawn: l.respawn_revision,
                        granted: l.respawn_hp_gain,
                    },
                )?;
                self.pending(
                    l.death_count > l.respawn_count,
                    l.pending_due_millis,
                    5000,
                    1_805_000,
                )?;
                require(if l.death_count == 0 {
                    l.death_cell == Cell { x: 0, y: 0 }
                } else {
                    self.manifest.contains_cell(&l.death_cell)
                        && (!(l.death_count > l.respawn_count
                            || l.last_death_revision == self.world.revision
                            || l.respawn_revision == self.world.revision)
                            || a.position == l.death_cell)
                })?;
            }
        }
        if let Some(q) = &self.quest {
            require(quest == !q.lives.0.is_empty())?;
            if quest {
                require(q.lives.0.len() == self.actors.0.len())?;
                for (l, a) in q.lives.0.iter().zip(&self.actors.0) {
                    self.life(
                        a,
                        Life {
                            entity: l.entity,
                            deaths: l.deaths,
                            death: l.death_revision,
                            respawns: l.respawns,
                            respawn: l.respawn_revision,
                            granted: l.hp_gain,
                        },
                    )?;
                    require(
                        l.lane == self.scene_member(a)?.team
                            && self.manifest.contains_cell(&l.spawn)
                            && (l.deaths != 0 || l.death_cell == Cell { x: 0, y: 0 })
                            && (l.deaths == 0 || self.manifest.contains_cell(&l.death_cell))
                            && (!(l.deaths > l.respawns
                                || l.death_revision == self.world.revision)
                                || a.position == l.death_cell)
                            && (!(initial(self.world.phase)
                                || l.respawn_revision == self.world.revision)
                                || a.position == l.spawn),
                    )?;
                }
            }
        } else {
            require(!quest)?;
        }
        if let Some(s) = &self.siege {
            require(
                s.lives.0.len() == self.actors.0.len()
                    && (0..=1).contains(&s.attacker)
                    && (1..=60_000).contains(&s.respawn_millis),
            )?;
            for (l, a) in s.lives.0.iter().zip(&self.actors.0) {
                self.life(
                    a,
                    Life {
                        entity: l.entity,
                        deaths: l.deaths,
                        death: l.death_revision,
                        respawns: l.respawns,
                        respawn: l.respawn_revision,
                        granted: l.hp_gain,
                    },
                )?;
                self.pending(
                    l.deaths > l.respawns,
                    l.due_millis,
                    s.respawn_millis,
                    1_860_000,
                )?;
                let camp = if self.scene_member(a)?.team == s.attacker {
                    &s.attack_camp.0
                } else {
                    &s.defense_camp.0
                };
                require(
                    !camp.is_empty()
                        && camp.iter().all(|c| self.manifest.contains_cell(c))
                        && (!(initial(self.world.phase)
                            || l.respawn_revision == self.world.revision)
                            || camp.contains(&a.position)),
                )?;
            }
        }
        if !honor && !quest && !siege {
            for a in &self.actors.0 {
                if let Some(r) = &a.resources {
                    require(r.life == 0 && r.flows.hp_granted == 0 && r.flows.mp_granted == 0)?;
                }
            }
        }
        Ok(())
    }
    fn life(&self, a: &Actor, l: Life) -> Result<(), ApplicationError> {
        let v = a.vital.as_ref().ok_or_else(invalid)?;
        let r = a.resources.as_ref().ok_or_else(invalid)?;
        require(
            l.entity == a.entity
                && l.deaths >= 0
                && l.respawns >= 0
                && l.respawns < i64::MAX
                && l.respawns <= l.deaths
                && l.deaths - l.respawns <= 1
                && l.death >= 0
                && l.respawn >= 0
                && l.granted >= 0
                && (l.deaths == 0) == (l.death == 0)
                && (l.respawns == 0) == (l.respawn == 0)
                && (l.respawns == 0) == (l.granted == 0)
                && (l.deaths == 0
                    || if l.deaths > l.respawns {
                        l.death > l.respawn
                    } else {
                        l.respawn > l.death
                    })
                && l.death == v.death_revision
                && l.death <= self.world.revision
                && l.respawn <= self.world.revision
                && l.deaths <= l.death
                && l.respawns <= l.respawn
                && l.respawns <= self.world.revision - 1
                && l.deaths <= self.world.revision - 1 - l.respawns
                && (v.values.hp == 0) == (l.deaths > l.respawns)
                && l.respawns == r.life
                && l.granted == r.flows.hp_granted
                && r.flows.mp_granted == 0
                && l.death <= r.revision
                && l.respawn <= r.revision
                && (l.deaths == 0 || self.world.active_elapsed_nanos >= 0)
                && (!initial(self.world.phase) || l.deaths == 0)
                && (l.respawn != self.world.revision
                    || v.values.hp == v.values.maximum_hp && r.revision == self.world.revision),
        )
    }
    fn pending(
        &self,
        dead: bool,
        due: i64,
        delay: i64,
        maximum: i64,
    ) -> Result<(), ApplicationError> {
        let pending = self.world.phase == Phase::Active && dead;
        require(
            (0..=maximum).contains(&due)
                && (due > 0) == pending
                && (!pending
                    || self.world.active_elapsed_nanos >= 0
                        && i128::from(due)
                            <= i128::from(self.world.active_elapsed_nanos / 1_000_000)
                                + i128::from(delay)),
        )
    }
}
