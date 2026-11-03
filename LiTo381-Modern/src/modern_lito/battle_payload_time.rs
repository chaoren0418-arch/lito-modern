//! Cooperative tower/waves/departures and terminal facts, not an SQL receipt.
use super::{
    combat::require,
    data::*,
    invalid,
    scene::{distance, initial, ordered, outcome},
    ApplicationError,
};
use std::collections::BTreeSet;
impl UnvalidatedData {
    pub(super) fn check_time_attack(&self) -> Result<(), ApplicationError> {
        let Some(t) = &self.time_attack else {
            return Ok(());
        };
        let done = t.phase == TimePhase::Finished;
        let aborted = t.phase == TimePhase::Aborted;
        require(
            self.room.mode == 4
                && self.room.map == 540
                && self.actors.0.len() <= 60
                && self.npcs.is_some()
                && t.revision >= 1
                && t.revision <= self.world.revision
                && (2..=32).contains(&t.waves.0.len())
                && t.wave >= 0
                && (t.wave as usize) < t.waves.0.len()
                && (1..=self.actors.0.len() as i64).contains(&t.leader)
                && ordered(&t.departed.0, self.actors.0.len() as i64)
                && (50..=1_800_000).contains(&t.duration_millis)
                && (-1..60_000).contains(&t.started_millis)
                && (0..=1_860_000).contains(&t.observed_millis)
                && t.started_millis <= t.observed_millis
                && t.observed_millis <= self.world.active_elapsed_nanos.max(0) / 1_000_000
                && (self.world.phase == Phase::Finished) == done
                && (self.world.phase == Phase::Aborted) == aborted
                && done == (t.end != TimeEnd::None)
                && done == self.result.is_some()
                && (!done || t.revision == self.world.revision)
                && matches!(t.tower.graphic, 2665 | 2667 | 2669 | 2671)
                && t.tower.maximum_hp >= 1
                && (0..=t.tower.maximum_hp).contains(&t.tower_hp)
                && self.manifest.contains_cell(&t.tower.cell)
                && (0..=t.revision).contains(&t.tower_hurt_revision)
                && (0..=t.revision).contains(&t.tower_death_revision)
                && (t.tower_hp == 0) == (t.tower_death_revision > 0)
                && (t.tower_hp < t.tower.maximum_hp) == (t.tower_hurt_revision > 0)
                && (t.started_millis >= 0) == (t.tower_hp == 0)
                && (t.tower_death_revision == 0 || t.tower_death_revision == t.tower_hurt_revision)
                && t.plan_sha256 != [0; 32],
        )?;
        let mut finals = 0;
        let mut dragons = 0;
        for (i, k) in t.waves.0.iter().enumerate() {
            match k {
                TimeKind::Ordinary => require(finals == 0 && dragons == 0)?,
                TimeKind::FinalBosses => {
                    finals += 1;
                    require(finals == 1 && dragons == 0)?;
                }
                TimeKind::Dragon => {
                    dragons += 1;
                    require(finals == 1 && dragons == 1 && i + 1 == t.waves.0.len())?;
                }
            }
        }
        require(t.waves.0[0] == TimeKind::Ordinary && finals == 1)?;
        match t.phase {
            TimePhase::WaitingForTower => {
                require(t.started_millis == -1 && t.wave == 0 && t.living.0.is_empty())?
            }
            TimePhase::SpawnPending => require(t.started_millis >= 0 && t.living.0.is_empty())?,
            TimePhase::Fighting => require(t.started_millis >= 0 && !t.living.0.is_empty())?,
            _ => {}
        }
        require(t.started_millis >= 0 || t.wave == 0 && t.living.0.is_empty())?;
        if initial(self.world.phase) {
            require(
                t.phase == TimePhase::WaitingForTower
                    && t.observed_millis == 0
                    && t.exchanges.0.is_empty()
                    && t.departed.0.is_empty(),
            )?;
        }
        let mut eligible = BTreeSet::new();
        let mut alive_mask = 0i64;
        let mut departed_mask = 0i64;
        for a in &self.actors.0 {
            let m = self.scene_member(a)?;
            let v = a.vital.as_ref().ok_or_else(invalid)?;
            require(
                m.identity.kind == MemberKind::Human && m.team == 0 && (1..=60).contains(&a.entity),
            )?;
            let departed = t.departed.0.contains(&a.entity);
            require(t.tower_hp == 0 || departed || a.position != t.tower.cell)?;
            if v.values.hp > 0 {
                alive_mask |= 1i64 << (a.entity - 1);
                if !departed {
                    eligible.insert(a.entity);
                }
            }
            if departed {
                departed_mask |= 1i64 << (a.entity - 1);
            }
        }
        let leader = self
            .scene_actor(t.leader)?
            .vital
            .as_ref()
            .ok_or_else(invalid)?;
        require(
            (done || aborted || eligible.contains(&t.leader))
                && (t.end != TimeEnd::LeaderDefeated
                    || leader.values.hp == 0 && !t.departed.0.contains(&t.leader))
                && (t.end != TimeEnd::PartyDefeated
                    || eligible.is_empty() && t.departed.0.contains(&t.leader))
                && (!matches!(t.end, TimeEnd::Cleared | TimeEnd::Timeout)
                    || eligible.contains(&t.leader))
                && (t.end != TimeEnd::Cleared
                    || t.wave as usize + 1 == t.waves.0.len()
                        && t.living.0.is_empty()
                        && t.started_millis >= 0)
                && (t.end != TimeEnd::Timeout || t.started_millis >= 0),
        )?;
        if t.started_millis >= 0 && !aborted {
            require(if t.end == TimeEnd::Timeout {
                t.observed_millis - t.started_millis == i64::from(t.duration_millis)
            } else {
                t.observed_millis - t.started_millis < i64::from(t.duration_millis)
            })?;
        }
        let mut previous = 0;
        let mut seen = BTreeSet::new();
        let mut fatal = false;
        for e in &t.exchanges.0 {
            let a = self.scene_actor(e.entity)?;
            let v = a.vital.as_ref().ok_or_else(invalid)?;
            outcome(&e.outcome, t.tower.maximum_hp)?;
            require(
                seen.insert(e.entity)
                    && e.sequence > 0
                    && e.sequence <= v.attack_sequence
                    && e.revision > previous
                    && e.revision <= t.revision
                    && e.outcome.remaining_hp >= t.tower_hp
                    && (!e.outcome.death || e.revision == t.tower_death_revision),
            )?;
            fatal |= e.outcome.death;
            previous = e.revision;
        }
        for a in &self.actors.0 {
            let Some(s) = &a.spells.structure else {
                continue;
            };
            require(s.quest_kind.is_none() && s.impact.usage.revision <= t.revision)?;
            let mut count = 0;
            for effect in &s.impact.targets.0 {
                if !matches!(effect.target, Target::TimeTower(_)) {
                    continue;
                };
                count += 1;
                require(
                    effect.cell == t.tower.cell
                        && effect.maximum_hp == t.tower.maximum_hp
                        && effect.hp_after >= t.tower_hp
                        && distance(&s.impact.usage.before.cell, &t.tower.cell) <= 3
                        && (effect.hp_after != 0
                            || s.impact.usage.revision == t.tower_death_revision),
                )?;
                fatal |= effect.hp_after == 0;
            }
            require(count == 1)?;
        }
        require((t.tower_death_revision > 0) == fatal)?;
        if let Some(r) = &self.result {
            let p = r.time_attack.as_ref().ok_or_else(invalid)?;
            require(
                done && r.mode == 4
                    && p.plan_sha256 == t.plan_sha256
                    && p.end == t.end
                    && p.wave == t.wave
                    && p.wave_count as usize == t.waves.0.len()
                    && i64::from(p.leader) == t.leader
                    && p.duration_millis == t.duration_millis
                    && p.started_millis == t.started_millis
                    && p.observed_millis == t.observed_millis
                    && p.alive_mask == alive_mask
                    && p.departed_mask == departed_mask
                    && p.tower_death_revision == t.tower_death_revision
                    && r.participants.0.iter().all(|p| {
                        (p.verdict == Verdict::Win)
                            == (t.end == TimeEnd::Cleared && eligible.contains(&p.entity))
                    }),
            )?;
        }
        Ok(())
    }
}
