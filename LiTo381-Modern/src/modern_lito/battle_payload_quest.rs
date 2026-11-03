//! Visible-lane Quest progress and retained structure events. Progress revision
//! is mode-local; objective/result revisions are ENGINE-domain. The losing
//! audience is never required to contain the hidden winning objective.
use super::{
    combat::require,
    data::*,
    invalid,
    scene::{distance, initial, outcome},
    ApplicationError,
};
use std::collections::BTreeSet;
impl UnvalidatedData {
    pub(super) fn check_quest(&self) -> Result<(), ApplicationError> {
        let Some(q) = &self.quest else {
            return require(
                self.world.descriptor.features as u32
                    & crate::modern_lito::world_protocol::MatchDescriptor::QUEST_OBJECTIVES
                    == 0
                    && !self.result.as_ref().is_some_and(|r| r.mode == 2),
            );
        };
        let done = q.status == QuestStatus::Finished;
        require(
            self.room.mode == 2
                && self.world.lane >= 0
                && self.npcs.is_some()
                && q.revision > 0
                && q.flow
                    == if matches!(self.room.map, 513 | 527) {
                        QuestFlow::RoomSymbols
                    } else {
                        QuestFlow::FinalGuardian
                    }
                && (self.world.phase == Phase::Finished) == done
                && (self.world.phase == Phase::Aborted) == (q.status == QuestStatus::Aborted)
                && (-1..=1).contains(&q.winning_lane)
                && done == (q.winning_lane >= 0)
                && done == self.result.is_some()
                && if done {
                    q.death_revision == self.world.revision
                } else {
                    q.death_revision == 0
                },
        )?;
        require(
            q.lanes.0.len() == 1
                && q.objectives.0.len() <= 1
                && q.exchanges.0.len() <= self.actors.0.len(),
        )?;
        let lane = &q.lanes.0[0];
        require(
            (1..=4).contains(&lane.stage)
                && (lane.phase == QuestPhase::Fighting) == !lane.living.0.is_empty(),
        )?;
        let configured = self.world.descriptor.features as u32
            & crate::modern_lito::world_protocol::MatchDescriptor::QUEST_OBJECTIVES
            != 0;
        require(configured || q.objectives.0.is_empty() && q.exchanges.0.is_empty() && !done)?;
        let per_lane = if !configured {
            0
        } else if q.flow == QuestFlow::RoomSymbols {
            4
        } else {
            1
        };
        require(q.targets.0.len() == per_lane)?;
        for (i, t) in q.targets.0.iter().enumerate() {
            require(
                t.lane == lane.lane
                    && t.stage == if per_lane == 4 { i as i32 + 1 } else { 4 }
                    && matches!(t.graphic, 2665 | 2667 | 2669 | 2671)
                    && t.maximum_hp >= 1
                    && self.manifest.contains_cell(&t.cell),
            )?;
        }
        if initial(self.world.phase) {
            require(
                q.revision == 1
                    && q.objectives.0.is_empty()
                    && q.exchanges.0.is_empty()
                    && lane.stage == 1
                    && lane.phase == QuestPhase::SpawnPending,
            )?;
        }
        for a in &q.objectives.0 {
            require(
                q.targets.0.contains(&a.target)
                    && a.kind == kind(q.flow, a.target.stage)
                    && (0..=a.target.maximum_hp).contains(&a.hp)
                    && a.born_revision >= 1
                    && a.revision >= a.born_revision
                    && a.revision <= self.world.revision
                    && (0..=a.revision).contains(&a.hurt_revision)
                    && (a.hurt_revision == 0 || a.hurt_revision > a.born_revision)
                    && (0..=a.revision).contains(&a.death_revision)
                    && (a.hp == 0) == (a.death_revision > 0)
                    && (a.death_revision == 0 || a.death_revision == a.hurt_revision)
                    && (a.hp < a.target.maximum_hp) == (a.hurt_revision > 0)
                    && (a.revision != a.born_revision
                        || a.hp == a.target.maximum_hp && a.hurt_revision == 0),
            )?;
            if a.hp > 0 {
                require(
                    a.target.stage == lane.stage
                        && lane.phase == QuestPhase::Objective
                        && self.actors.0.iter().all(|p| p.position != a.target.cell),
                )?;
            } else {
                require(if a.target.stage == lane.stage {
                    done && q.winning_lane == lane.lane && a.target.stage == 4
                } else {
                    a.target.stage + 1 == lane.stage && a.kind == QuestKind::RoomSymbol
                })?;
            }
        }
        let mut previous = 0;
        let mut humans = BTreeSet::new();
        let mut revisions = BTreeSet::new();
        let mut lethal = false;
        for e in &q.exchanges.0 {
            let player = self.identity_actor(&e.participant)?;
            let v = player.vital.as_ref().ok_or_else(invalid)?;
            let target = q
                .targets
                .0
                .iter()
                .find(|t| t.lane * 4 + t.stage == e.target.slot)
                .ok_or_else(invalid)?;
            outcome(&e.outcome, target.maximum_hp)?;
            require(
                e.participant.kind == MemberKind::Human
                    && target.lane == self.scene_member(player)?.team
                    && target.stage <= lane.stage
                    && humans.insert(player.entity)
                    && revisions.insert(e.revision)
                    && e.revision > previous
                    && e.revision <= self.world.revision
                    && e.request_sequence > 0
                    && e.request_sequence <= v.attack_sequence,
            )?;
            let current = q.objectives.0.iter().find(|a| a.id == e.target);
            if let Some(a) = current {
                impact(
                    a,
                    e.revision,
                    0,
                    e.outcome.remaining_hp,
                    e.outcome.applied_damage,
                    e.outcome.death,
                )?;
            }
            if e.revision == self.world.revision {
                require(
                    current.is_some() && v.values.hp > 0 && e.request_sequence == v.attack_sequence,
                )?;
                lethal |= e.outcome.death && target.stage == 4;
            }
            previous = e.revision;
        }
        for p in &self.actors.0 {
            let Some(s) = &p.spells.structure else {
                continue;
            };
            let usage = &s.impact.usage;
            let origin = &usage.before.cell;
            require(
                revisions.insert(usage.revision)
                    && !initial(self.world.phase)
                    && self.scene_member(p)?.identity.kind == MemberKind::Human,
            )?;
            let mut count = 0;
            for e in &s.impact.targets.0 {
                let Target::Quest { id, born_revision } = &e.target else {
                    continue;
                };
                count += 1;
                let target = q
                    .targets
                    .0
                    .iter()
                    .find(|t| t.lane * 4 + t.stage == id.slot)
                    .ok_or_else(invalid)?;
                let current = q.objectives.0.first().ok_or_else(invalid)?;
                require(
                    target.lane == self.scene_member(p)?.team
                        && target.stage <= lane.stage
                        && s.quest_kind == Some(kind(q.flow, target.stage))
                        && e.cell == target.cell
                        && e.maximum_hp == target.maximum_hp
                        && (1..=3).contains(&distance(origin, &e.cell))
                        && current.target.stage >= target.stage,
                )?;
                if current.id == *id {
                    impact(
                        current,
                        usage.revision,
                        *born_revision,
                        e.hp_after,
                        e.applied,
                        e.hp_after == 0,
                    )?;
                }
                if usage.revision == self.world.revision {
                    lethal |= e.hp_after == 0 && target.stage == 4;
                }
            }
            require(count == 1)?;
        }
        if done {
            let r = self.result.as_ref().ok_or_else(invalid)?;
            require(
                r.mode == 2
                    && r.engine_revision == self.world.revision
                    && r.participants
                        .0
                        .iter()
                        .all(|p| (p.verdict == Verdict::Win) == (p.team == q.winning_lane)),
            )?;
            if lane.lane == q.winning_lane {
                require(
                    lane.stage == 4
                        && lane.phase == QuestPhase::Objective
                        && lethal
                        && q.objectives.0.first().is_some_and(|a| {
                            a.target.stage == 4
                                && a.hp == 0
                                && a.death_revision == self.world.revision
                        }),
                )?;
            }
        }
        Ok(())
    }
}
fn kind(flow: QuestFlow, stage: i32) -> QuestKind {
    if flow == QuestFlow::FinalGuardian {
        QuestKind::FinalGuardian
    } else if stage == 4 {
        QuestKind::FinalSymbol
    } else {
        QuestKind::RoomSymbol
    }
}
fn impact(
    a: &QuestObjective,
    revision: i64,
    birth: i64,
    hp: i64,
    applied: i64,
    death: bool,
) -> Result<(), ApplicationError> {
    require(
        revision > a.born_revision
            && revision <= a.revision
            && (birth == 0 || birth == a.born_revision)
            && hp >= a.hp
            && (!death || a.death_revision == revision)
            && (revision != a.revision
                || hp == a.hp && (applied == 0 || a.hurt_revision == revision)),
    )
}
