//! Canonical single-observation scene checks. No legacy frame construction,
//! revision translation, command capability or saved receipt is implied.
use super::{combat::require, data::*, invalid, ApplicationError};

pub(super) fn initial(p: Phase) -> bool {
    matches!(p, Phase::Loading | Phase::Countdown)
}
pub(super) fn distance(a: &Cell, b: &Cell) -> i64 {
    (i64::from(a.x) - i64::from(b.x))
        .abs()
        .max((i64::from(a.y) - i64::from(b.y)).abs())
}
pub(super) fn ordered(ids: &[i64], maximum: i64) -> bool {
    ids.iter().all(|n| *n > 0 && *n <= maximum) && ids.windows(2).all(|p| p[0] < p[1])
}
pub(super) fn outcome(o: &Outcome, maximum: i64) -> Result<(), ApplicationError> {
    require(
        o.raw_damage >= 0
            && o.applied_damage >= 0
            && o.applied_damage <= o.raw_damage
            && o.remaining_hp >= 0
            && o.remaining_hp <= maximum
            && o.applied_damage <= maximum - o.remaining_hp
            && (o.applied_damage > 0 || o.remaining_hp > 0)
            && o.death == (o.remaining_hp == 0)
            && (o.hit
                || o.raw_damage == 0
                    && o.applied_damage == 0
                    && !o.death
                    && !o.maximum_triggered
                    && !o.critical_triggered),
    )
}
impl UnvalidatedData {
    /// Extends the private/combat gate with scene and mode checks in the actual
    /// ENGINE revision domain. Temporal following, command acknowledgement and
    /// durable receipts remain separate; this is still UnvalidatedData.
    pub fn check_scene_values(
        &self,
        player: &Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        if self.vip.is_some() {
            return super::vip::attached(self)?.check_values(player, world);
        }
        if self.armor.is_some() {
            return super::armor::attached(self)?.check_values(player, world);
        }
        if self.healing.is_some() {
            return super::healing::attached(self)?.check_values(player, world);
        }
        if self.speed.is_some() {
            return super::speed::attached(self)?.check_values(player, world);
        }
        if self.haste.is_some() {
            return super::haste::attached(self)?.check_values(player, world);
        }
        self.personal_view(player, world)?;
        self.check_scene_body()
    }
    pub(super) fn check_scene_body(&self) -> Result<(), ApplicationError> {
        self.check_lives()?;
        super::presentation::check(self)?;
        // Caster-life admission applies to every mode, including Honor and
        // Siege where no NPC roster exists. Do not hide it behind NPC presence.
        for actor in &self.actors.0 {
            for receipt in actor
                .spells
                .histories
                .0
                .iter()
                .filter_map(|s| s.receipt.as_ref())
            {
                self.scene_cast(actor, &receipt.impact)?;
            }
            if let Some(structure) = &actor.spells.structure {
                self.scene_cast(actor, &structure.impact)?;
            }
        }
        self.check_npcs()?;
        super::npc_motion::check_attached(self)?;
        self.check_quest()?;
        self.check_time_attack()?;
        self.check_honor()?;
        self.check_siege()?;
        self.check_race()?;
        self.check_elimination()?;
        require(
            (self.world.phase == Phase::Finished)
                == (self.result.is_some()
                    || self
                        .race
                        .as_ref()
                        .is_some_and(|r| r.phase == RacePhase::Complete)),
        )?;
        if let Some(r) = &self.result {
            // A syntactically complete result cannot enable a mode that the
            // published world never had. In particular, a SceneOnly payload
            // must not bypass a mode proof simply because its section is None.
            require(match r.mode {
                1 => self.world.descriptor.ruleset == Ruleset::DeathmatchElimination,
                2 => self.quest.is_some(),
                3 => self.siege.is_some(),
                4 => self.time_attack.is_some(),
                5 => !self.honor.objectives.0.is_empty(),
                _ => false,
            })?;
            require(
                self.world.phase == Phase::Finished
                    && r.engine_revision == self.world.revision
                    && r.result_id != [0; 16]
                    && r.finished_at_millis >= 0
                    && (r.mode == 4) == r.time_attack.is_some()
                    && (r.mode == 3) == r.siege.is_some(),
            )?;
        }
        Ok(())
    }
    pub(super) fn scene_actor(&self, id: i64) -> Result<&Actor, ApplicationError> {
        self.actors
            .0
            .iter()
            .find(|a| a.entity == id)
            .ok_or_else(invalid)
    }
    pub(crate) fn scene_member(&self, a: &Actor) -> Result<&Member, ApplicationError> {
        usize::try_from(a.entity.checked_sub(1).ok_or_else(invalid)?)
            .ok()
            .and_then(|i| self.room.members.0.get(i))
            .ok_or_else(invalid)
    }
    pub(super) fn identity_actor(&self, id: &Identity) -> Result<&Actor, ApplicationError> {
        self.actors
            .0
            .iter()
            .find(|a| self.scene_member(a).is_ok_and(|m| m.identity == *id))
            .ok_or_else(invalid)
    }
    pub(super) fn scene_cast(
        &self,
        actor: &Actor,
        impact: &Impact,
    ) -> Result<(), ApplicationError> {
        let c = &impact.usage;
        self.scene_usage(actor, c, impact.lane == Lane::Teleport)
    }
    pub(super) fn scene_usage(
        &self,
        actor: &Actor,
        c: &Cast,
        teleport: bool,
    ) -> Result<(), ApplicationError> {
        self.scene_life(actor, c.before.life, c.revision)?;
        if c.revision == self.world.revision {
            require(
                actor.vital.as_ref().ok_or_else(invalid)?.values.hp > 0
                    && c.sequence == actor.spells.sequence
                    && (teleport || c.before.cell == actor.position),
            )?;
        }
        Ok(())
    }
    pub(super) fn scene_life(
        &self,
        actor: &Actor,
        life: i64,
        revision: i64,
    ) -> Result<(), ApplicationError> {
        let r = actor.resources.as_ref().ok_or_else(invalid)?;
        let v = actor.vital.as_ref().ok_or_else(invalid)?;
        let incarnation = r.life.checked_add(1).ok_or_else(invalid)?;
        require(
            life <= incarnation
                && !(life == incarnation && v.values.hp == 0 && revision >= v.death_revision),
        )?;
        let respawn = self
            .quest
            .as_ref()
            .and_then(|q| q.lives.0.iter().find(|l| l.entity == actor.entity))
            .map(|l| l.respawn_revision)
            .or_else(|| {
                self.honor
                    .lives
                    .0
                    .iter()
                    .find(|l| l.entity == actor.entity)
                    .map(|l| l.respawn_revision)
            })
            .or_else(|| {
                self.siege
                    .as_ref()
                    .and_then(|s| s.lives.0.iter().find(|l| l.entity == actor.entity))
                    .map(|l| l.respawn_revision)
            });
        if let Some(respawn) = respawn {
            require(if life == incarnation {
                revision > respawn
            } else {
                revision <= respawn
            })?;
        }
        Ok(())
    }
}
