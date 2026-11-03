//! Complete candidate timelines. No negotiated/live Observation is created.
use super::*;
use crate::modern_lito::battle_payload::data;
use std::collections::BTreeMap;

fn revision(g: &Grant) -> i64 {
    g.receipt.impact.usage.revision
}
fn detail(g: &Grant) -> Result<&Detail, ApplicationError> {
    if let data::Detail::Haste(d) = &g.receipt.detail {
        Ok(d)
    } else {
        Err(invalid())
    }
}
fn grants(c: &View<'_>) -> Result<BTreeMap<i64, Grant>, ApplicationError> {
    let mut all = BTreeMap::new();
    let mut add = |g: Grant| {
        if let Some(old) = all.insert(revision(&g), g.clone()) {
            require(old == g)?;
        }
        Ok(())
    };
    for e in &c.effects.targets.0 {
        for g in e.active.iter().chain(e.walk.iter()) {
            add(g.clone())?;
        }
    }
    for a in &c.data.actors.0 {
        for s in &a.spells.histories.0 {
            if s.lane == data::Lane::Haste {
                if let Some(r) = &s.receipt {
                    add(Grant {
                        caster: Participant {
                            entity: a.entity,
                            identity: c.data.scene_member(a)?.identity.clone(),
                        },
                        receipt: r.clone(),
                    })?;
                }
            }
        }
    }
    Ok(all)
}
fn terminal(p: data::Phase) -> bool {
    matches!(p, data::Phase::Finished | data::Phase::Aborted)
}

impl View<'_> {
    /// Validates both complete observations against the original owner/world,
    /// then reuses all existing actor/mode timelines and proves the extension.
    /// A caller must still perform session/capability/ACK admission separately.
    pub fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        let rev = self.data.world.revision;
        if terminal(self.data.world.phase) || rev == next.data.world.revision {
            return require(self.effects == next.effects);
        }
        let old = grants(self)?;
        for (r, g) in grants(next)? {
            // A newly observed old event must already have been present as an
            // active effect, old walk source, or caster's latest receipt.
            if r <= rev {
                require(old.get(&r) == Some(&g))?;
            }
        }
        for e in &next.effects.targets.0 {
            if let Some(g) = &e.active {
                let previous = self
                    .effects
                    .targets
                    .0
                    .iter()
                    .find(|p| p.target == e.target)
                    .and_then(|p| p.active.as_ref());
                require(if let Some(p) = previous {
                    g == p || revision(g) > rev && revision(g) > revision(p)
                } else {
                    revision(g) > rev
                })?;
            }
        }
        for e in &self.effects.targets.0 {
            if let Some(g) = &e.active {
                let active = next
                    .effects
                    .targets
                    .0
                    .iter()
                    .find(|p| p.target == e.target)
                    .and_then(|p| p.active.as_ref());
                if active.is_none() {
                    self.check_clear(next, g)?;
                }
            }
        }
        self.motion_successor(next)
    }
    fn check_clear(&self, next: &Self, g: &Grant) -> Result<(), ApplicationError> {
        if terminal(next.data.world.phase) {
            return Ok(());
        }
        match &detail(g)?.target {
            StatusTarget::Player(p) => {
                let a = next
                    .data
                    .actors
                    .0
                    .iter()
                    .find(|a| a.entity == p.participant.entity)
                    .ok_or_else(invalid)?;
                let r = a.resources.as_ref().ok_or_else(invalid)?;
                let b = a.buffs.as_ref().ok_or_else(invalid)?;
                require(
                    r.vitals.hp == 0
                        || next
                            .data
                            .time_attack
                            .as_ref()
                            .is_some_and(|t| t.departed.0.contains(&p.participant.entity))
                        || r.life.checked_add(1).ok_or_else(invalid)? > p.life
                        || next.data.world.active_elapsed_nanos >= p.after.haste_until_nanos
                        || matches!(
                            b.haste_origin,
                            data::HasteOrigin::Potion | data::HasteOrigin::GreaterHaste
                        ),
                )
            }
            StatusTarget::Npc(p) => {
                let n = next.data.npcs.as_ref().ok_or_else(invalid)?;
                match n.actors.0.iter().find(|n| n.serial == p.id.serial) {
                    None => Ok(()), // Existing roster timeline already proves retirement.
                    Some(a) => require(
                        a.hp == 0
                            || a.life > p.life
                            || n.elapsed_millis >= p.after.haste_until_millis,
                    ),
                }
            }
        }
    }
    fn motion_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        let (Some(a), Some(b)) = (&self.effects.npcs, &next.effects.npcs) else {
            return require(self.effects.npcs.is_none() && next.effects.npcs.is_none());
        };
        require(b.revision >= a.revision && b.elapsed_millis >= a.elapsed_millis)?;
        if a.revision == b.revision {
            return require(a == b);
        }
        for p in &a.actors.0 {
            let Some(q) = b.actors.0.iter().find(|n| n.id == p.id) else {
                continue;
            };
            require(
                p.walk_millis == q.walk_millis
                    && p.attack_millis == q.attack_millis
                    && q.speed.revision >= p.speed.revision
                    && q.speed.observed_millis >= p.speed.observed_millis
                    && if p.speed.revision == q.speed.revision {
                        p.speed == q.speed
                    } else {
                        q.speed.revision > a.revision
                    },
            )?;
            let before = p.last_walk.as_ref().map_or(0, |w| w.sequence);
            let after = q.last_walk.as_ref().map_or(0, |w| w.sequence);
            require(after >= before && after - before <= b.revision - a.revision)?;
            let old = self
                .data
                .npcs
                .as_ref()
                .and_then(|n| n.actors.0.iter().find(|n| n.serial == p.id.serial))
                .ok_or_else(invalid)?;
            if before == after {
                require(p.last_walk == q.last_walk && p.walk_speed == q.walk_speed)?;
                let new = next
                    .data
                    .npcs
                    .as_ref()
                    .and_then(|n| n.actors.0.iter().find(|n| n.serial == p.id.serial))
                    .ok_or_else(invalid)?;
                require(old.cell == new.cell)?;
            } else {
                let w = q.last_walk.as_ref().ok_or_else(invalid)?;
                require(
                    w.roster_revision > a.revision
                        && w.life >= p.life
                        && w.elapsed_millis >= a.elapsed_millis
                        && (q.walk_speed.revision > a.revision || q.walk_speed == p.speed)
                        && (after - before != 1 || w.from == old.cell),
                )?;
                if let Some(previous) = &p.last_walk {
                    require(
                        w.elapsed_millis
                            >= previous.elapsed_millis + i64::from(previous.duration_millis),
                    )?;
                }
            }
        }
        Ok(())
    }
}
