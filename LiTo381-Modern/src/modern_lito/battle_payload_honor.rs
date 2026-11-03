//! Canonical tower and elimination evidence. No legacy +1 revision and no
//! fixed historical skill price: receipts retain their actual equipment era.
use super::{
    combat::require,
    data::*,
    invalid,
    scene::{distance, initial},
    ApplicationError,
};

pub(super) fn tower_action(hp: i64, maximum: i64) -> i32 {
    if hp == 0 {
        35
    } else if hp < maximum / 4 {
        34
    } else if hp < maximum / 2 {
        33
    } else if i128::from(hp) < i128::from(maximum) * 3 / 4 {
        32
    } else {
        0
    }
}
fn in_screen(a: &Cell, b: &Cell) -> bool {
    let dx = i64::from(b.x) - i64::from(a.x);
    let dy = i64::from(b.y) - i64::from(a.y);
    let d = dx.abs() + dy.abs();
    d <= 18 || d == 19 && (dx + 18).abs() + (dy + 18).abs() <= 52
}
fn matches(e: &Effect, o: &HonorObjective) -> bool {
    matches!(e.target, Target::Honor { entity, team } if entity == o.entity && team == o.team)
}
impl UnvalidatedData {
    pub(super) fn check_honor(&self) -> Result<(), ApplicationError> {
        let towers = &self.honor.objectives.0;
        if towers.is_empty() {
            return Ok(());
        }
        let n = self.actors.0.len();
        require(self.room.mode == 5 && n >= 2 && towers.len() == 2)?;
        let mut occupied = self
            .actors
            .0
            .iter()
            .map(|a| (a.position.x, a.position.y))
            .collect::<std::collections::BTreeSet<_>>();
        for (i, o) in towers.iter().enumerate() {
            require(
                o.entity == (n + i + 1) as i64
                    && o.team == i as i32
                    && [2665, 2667, 2669, 2671].contains(&o.graphic)
                    && o.maximum_hp > 0
                    && (0..=o.maximum_hp).contains(&o.hp)
                    && (0..=self.world.revision).contains(&o.revision)
                    && (0..=o.revision).contains(&o.hurt_revision)
                    && (0..=o.revision).contains(&o.death_revision)
                    && o.damage_loss >= 0
                    && o.heal_gain >= 0
                    && (0..=n as i64).contains(&o.last_actor)
                    && (0..=4).contains(&o.last_kind)
                    && o.action == tower_action(o.hp, o.maximum_hp)
                    && (o.hp == 0) == (o.death_revision > 0)
                    && (o.death_revision == 0 || o.death_revision == o.hurt_revision)
                    && (o.revision == 0) == (o.last_kind == 0)
                    && (o.revision == 0) == (o.last_actor == 0)
                    && (o.revision != 0
                        || o.hp == o.maximum_hp
                            && o.hurt_revision == 0
                            && o.damage_loss == 0
                            && o.heal_gain == 0)
                    && i128::from(o.maximum_hp) + i128::from(o.heal_gain)
                        - i128::from(o.damage_loss)
                        == i128::from(o.hp)
                    && (!initial(self.world.phase) || o.revision == 0)
                    && self.manifest.contains_cell(&o.cell)
                    && occupied.insert((o.cell.x, o.cell.y)),
            )?;
            if o.last_actor == 0 {
                continue;
            }
            let a = self.scene_actor(o.last_actor)?;
            require((o.last_kind == 3) == (self.scene_member(a)?.team == o.team))?;
            if o.last_kind == 1 {
                let v = a.vital.as_ref().ok_or_else(invalid)?;
                require(
                    v.attack_revision >= o.revision
                        && (v.attack_revision != o.revision || v.target == o.entity),
                )?;
            } else {
                let lane = match o.last_kind {
                    2 => Lane::LightArrow,
                    3 => Lane::Healing,
                    4 => Lane::FireStorm,
                    _ => return Err(invalid()),
                };
                let r = a
                    .spells
                    .histories
                    .0
                    .iter()
                    .find(|s| s.lane == lane)
                    .and_then(|s| s.receipt.as_ref())
                    .ok_or_else(invalid)?;
                require(
                    r.impact.usage.revision >= o.revision
                        && (r.impact.usage.revision != o.revision
                            || r.impact.targets.0.iter().any(|e| matches(e, o))),
                )?;
            }
        }
        for a in &self.actors.0 {
            for s in &a.spells.histories.0 {
                let Some(r) = &s.receipt else {
                    continue;
                };
                let kind = match s.lane {
                    Lane::LightArrow => 2,
                    Lane::Healing => 3,
                    Lane::FireStorm => 4,
                    _ => continue,
                };
                let healing = kind == 3;
                for (index, e) in r.impact.targets.0.iter().enumerate() {
                    let Target::Honor { team, .. } = e.target else {
                        require(!healing)?;
                        continue;
                    };
                    let o = usize::try_from(team)
                        .ok()
                        .and_then(|i| towers.get(i))
                        .ok_or_else(invalid)?;
                    let c = &r.impact.usage;
                    require(
                        self.scene_member(a)?.identity.kind == MemberKind::Human
                            && matches(e, o)
                            && e.cell == o.cell
                            && e.maximum_hp == o.maximum_hp
                            && healing == (self.scene_member(a)?.team == o.team)
                            && c.revision <= o.revision
                            && e.applied <= if healing { o.heal_gain } else { o.damage_loss }
                            && if healing {
                                in_screen(&c.before.cell, &o.cell)
                            } else {
                                distance(&c.before.cell, &o.cell) <= if kind == 4 { 3 } else { 10 }
                            },
                    )?;
                    if c.revision == o.revision {
                        require(
                            o.last_actor == a.entity
                                && o.last_kind == kind
                                && o.hp == e.hp_after
                                && (healing || e.applied == 0 || o.hurt_revision == c.revision)
                                && (e.hp_after != 0 || o.death_revision == c.revision),
                        )?;
                    }
                    if let Detail::FireStorm { damage_loss_after } = &r.detail {
                        let loss = *damage_loss_after.0.get(index).ok_or_else(invalid)?;
                        require(
                            loss <= o.damage_loss
                                && (c.revision != o.revision || loss == o.damage_loss),
                        )?;
                    }
                }
            }
        }
        let dead = towers.iter().filter(|t| t.hp == 0).collect::<Vec<_>>();
        match &self.result {
            None => require(dead.is_empty() && self.world.phase != Phase::Finished),
            Some(r) => {
                require(
                    self.world.phase == Phase::Finished
                        && r.mode == 5
                        && dead.len() == 1
                        && dead[0].death_revision == self.world.revision,
                )?;
                require(
                    r.participants
                        .0
                        .iter()
                        .all(|p| (p.verdict == Verdict::Loss) == (p.team == dead[0].team)),
                )
            }
        }
    }
    pub(super) fn check_elimination(&self) -> Result<(), ApplicationError> {
        let enabled = self.world.descriptor.ruleset == Ruleset::DeathmatchElimination;
        if !enabled {
            return require(self.result.as_ref().is_none_or(|r| r.mode != 1));
        }
        let alive = self
            .actors
            .0
            .iter()
            .filter(|a| a.vital.as_ref().is_some_and(|v| v.values.hp > 0))
            .map(|a| self.scene_member(a).map(|m| m.team))
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
        if self.world.phase == Phase::Active {
            require(alive.len() == 2)?;
        }
        if let Some(r) = &self.result {
            require(
                r.mode == 1
                    && alive.len() == 1
                    && self.actors.0.iter().all(|a| a.vital.is_some())
                    && self.actors.0.iter().any(|a| {
                        a.vital.as_ref().is_some_and(|v| {
                            v.values.hp == 0 && v.death_revision == self.world.revision
                        })
                    }),
            )?;
            let winner = *alive.first().ok_or_else(invalid)?;
            require(
                r.participants
                    .0
                    .iter()
                    .all(|p| (p.verdict == Verdict::Win) == (p.team == winner)),
            )?;
        }
        Ok(())
    }
}
