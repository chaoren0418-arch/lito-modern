//! Complete single-observation speed target/motion provenance. Candidate only;
//! no negotiated authority, command ACK or live Observation is created.
use super::super::data::{self, SpellReceipt};
use super::values::{buff, npc as speed};
use super::*;
use crate::modern_lito::{use_history, vital_balance};
use std::collections::{BTreeMap, BTreeSet};
type Key = (u8, i64);
pub(super) struct Effect<'a> {
    pub active_elapsed_nanos: i64,
    pub target: &'a StatusTarget,
    pub changes: bool,
    pub missing: Option<&'a MissingEffect>,
}
pub(super) fn detail(g: &Grant) -> Result<Effect<'_>, ApplicationError> {
    match &g.receipt.detail {
        data::Detail::Haste(d) => Ok(Effect {
            active_elapsed_nanos: d.active_elapsed_nanos,
            target: &d.target,
            changes: true,
            missing: None,
        }),
        data::Detail::Slow(d) => Ok(Effect {
            active_elapsed_nanos: d.active_elapsed_nanos,
            target: &d.target,
            changes: d.missing_effect.is_none()
                && d.roll <= d.probability
                && d.equipment_haste_items == 0,
            missing: d.missing_effect.as_ref(),
        }),
        _ => Err(invalid()),
    }
}
fn key(t: &Target) -> Key {
    match t {
        Target::Player(p) => (0, p.entity),
        Target::Npc(n) => (1, n.serial),
    }
}
pub(super) fn target(g: &Grant) -> Result<Target, ApplicationError> {
    Ok(match detail(g)?.target {
        StatusTarget::Player(p) => Target::Player(p.participant.clone()),
        StatusTarget::Npc(n) => Target::Npc(n.id.clone()),
    })
}
fn rev(g: &Grant) -> i64 {
    g.receipt.impact.usage.revision
}
fn life(g: &Grant) -> Result<i64, ApplicationError> {
    Ok(match detail(g)?.target {
        StatusTarget::Player(p) => p.life,
        StatusTarget::Npc(n) => n.life,
    })
}
fn add(events: &mut BTreeMap<i64, Grant>, g: &Grant) -> Result<(), ApplicationError> {
    if let Some(old) = events.insert(rev(g), g.clone()) {
        require(old == *g)?;
    }
    Ok(())
}
fn event_follows(a: &SpellReceipt, b: &SpellReceipt) -> Result<(), ApplicationError> {
    let x = &a.impact.usage;
    let y = &b.impact.usage;
    require(
        x.revision < y.revision
            && x.sequence < y.sequence
            && a.loading_elapsed_nanos <= b.loading_elapsed_nanos
            && x.before.life <= y.before.life
            && x.before.equipment_revision <= y.before.equipment_revision,
    )
}
fn follows(a: &SpellReceipt, b: &SpellReceipt) -> Result<(), ApplicationError> {
    require(a.impact.lane == b.impact.lane)?;
    let count = i128::from(b.casts) - i128::from(a.casts);
    let spent = i128::from(b.spent) - i128::from(a.spent);
    require(count >= 0 && spent >= 0)?;
    if count == 0 {
        return require(a == b);
    }
    event_follows(a, b)?;
    let last = i128::from(b.impact.usage.cost);
    require(
        count <= i128::from(b.impact.usage.sequence) - i128::from(a.impact.usage.sequence)
            && count <= i128::from(b.impact.usage.revision) - i128::from(a.impact.usage.revision)
            && spent >= last
            && spent - last >= (count - 1) * 15
            && spent - last
                <= (count - 1)
                    * if a.impact.lane == data::Lane::Slow {
                        20
                    } else {
                        40
                    },
    )
}
fn at(s: &Speed, time: i64) -> Result<(), ApplicationError> {
    speed(s)?;
    require(
        time >= s.observed_millis
            && time <= 1_860_000
            && (s.haste_until_millis == 0 || time < s.haste_until_millis)
            && (s.slow_until_millis == 0 || time < s.slow_until_millis),
    )
}
fn paced(base: i32, s: &Speed) -> i32 {
    if s.slow_until_millis != 0 {
        base * 2
    } else if s.haste_until_millis != 0 {
        ((f64::from(base) * 0.755) as i32).max(1)
    } else {
        base
    }
}
fn matches(g: &Grant, life: i64, lane: i32, s: &Speed) -> Result<(), ApplicationError> {
    require(
        detail(g)?.changes
            && g.receipt.impact.lane
                == if s.slow_until_millis != 0 {
                    data::Lane::Slow
                } else {
                    data::Lane::Haste
                }
            && matches!(detail(g)?.target,StatusTarget::Npc(n)if n.life==life&&n.lane==lane&&n.after==*s),
    )
}
impl View<'_> {
    pub fn check_values(
        &self,
        player: &data::Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        self.data.check_speed_context(player, world)?;
        self.data.check_combat_body()?;
        self.data.personal_facts(player, world)?;
        self.data.check_scene_body()?;
        crate::modern_lito::battle_follow::clocks(&self.data)?;
        self.check_components()?;
        self.check_effects()
    }
    fn actor(&self, p: &Participant) -> Result<&data::Actor, ApplicationError> {
        let a = self
            .data
            .actors
            .0
            .iter()
            .find(|a| a.entity == p.entity)
            .ok_or_else(invalid)?;
        require(
            (1..=100).contains(&p.entity)
                && p.identity.source > 0
                && p.identity.id > 0
                && self.data.scene_member(a)?.identity == p.identity,
        )?;
        Ok(a)
    }
    fn participant(&self, a: &data::Actor) -> Result<Participant, ApplicationError> {
        Ok(Participant {
            entity: a.entity,
            identity: self.data.scene_member(a)?.identity.clone(),
        })
    }
    fn grant(&self, g: &Grant) -> Result<(), ApplicationError> {
        let a = self.actor(&g.caster)?;
        let r = &g.receipt;
        let u = &r.impact.usage;
        let e = detail(g)?;
        let w = &self.data.world;
        let resources = a.resources.as_ref().ok_or_else(invalid)?;
        values::receipt(r)?;
        require(
            g.caster.identity.kind == data::MemberKind::Human
                && vital_balance::State::from(resources).valid()
                && a.spells.histories.0.iter().any(|h| h.lane == r.impact.lane)
                && u.revision <= w.revision
                && u.before.life <= resources.life.checked_add(1).ok_or_else(invalid)?
                && u.before.equipment_revision <= resources.equipment_revision
                && (0..=w.loading_elapsed_nanos).contains(&r.loading_elapsed_nanos)
                && e.active_elapsed_nanos <= w.active_elapsed_nanos
                && i128::from(r.loading_elapsed_nanos) - i128::from(e.active_elapsed_nanos)
                    == i128::from(w.loading_elapsed_nanos) - i128::from(w.active_elapsed_nanos)
                && self.data.manifest.contains_cell(&u.before.cell),
        )?;
        let (cell, lane) = match e.target {
            StatusTarget::Player(p) => {
                let t = self.actor(&p.participant)?;
                let tr = t.resources.as_ref().ok_or_else(invalid)?;
                require(
                    p.life <= tr.life.checked_add(1).ok_or_else(invalid)?
                        && p.equipment_revision <= tr.equipment_revision,
                )?;
                if p.participant.entity == g.caster.entity
                    || p.participant.identity == g.caster.identity
                {
                    require(
                        p.participant == g.caster
                            && p.life == u.before.life
                            && p.equipment_revision == u.before.equipment_revision
                            && p.cell == u.before.cell,
                    )?;
                }
                if u.revision == w.revision {
                    require(
                        w.phase == data::Phase::Active
                            && tr.vitals.hp > 0
                            && p.life == tr.life.checked_add(1).ok_or_else(invalid)?
                            && p.equipment_revision == tr.equipment_revision
                            && p.cell == t.position
                            && t.buffs.as_ref() == Some(&p.after),
                    )?;
                }
                (&p.cell, self.data.scene_member(t)?.team)
            }
            StatusTarget::Npc(n) => {
                let roster = self.effects.npcs.as_ref().ok_or_else(invalid)?;
                let raw = self.data.npcs.as_ref().ok_or_else(invalid)?;
                require(
                    n.id.world == w.identity
                        && n.id.serial <= raw.spawned
                        && n.roster_revision <= roster.revision
                        && n.after.revision <= roster.revision,
                )?;
                if let Some(t) = raw.actors.0.iter().find(|a| a.serial == n.id.serial) {
                    require(
                        n.life <= t.life
                            && n.lane == t.lane
                            && e.missing.is_none_or(|m| m.npc_template == t.npc_id),
                    )?;
                }
                if u.revision == w.revision {
                    let t = raw
                        .actors
                        .0
                        .iter()
                        .find(|a| a.serial == n.id.serial)
                        .ok_or_else(invalid)?;
                    let m = roster
                        .actors
                        .0
                        .iter()
                        .find(|a| a.id == n.id)
                        .ok_or_else(invalid)?;
                    require(
                        w.phase == data::Phase::Active
                            && t.hp > 0
                            && n.life == t.life
                            && n.cell == t.cell
                            && m.speed == n.after,
                    )?;
                }
                (&n.cell, n.lane)
            }
        };
        require(
            self.data.manifest.contains_cell(cell)
                && (w.lane < 0 || lane == self.data.scene_member(a)?.team),
        )
    }
    /// Cross-field status proof, NOT full scene/sequence/owner admission.
    /// Callers cannot convert this result into an existing Observation.
    pub fn check_effects(&self) -> Result<(), ApplicationError> {
        let d = &self.data;
        let s = &self.effects;
        let w = &d.world;
        require(
            d.world.descriptor.features & FEATURE != 0
                && d.haste.is_none()
                && d.presentation.is_some()
                && d.npc_motion.is_none()
                && s.world == w.identity
                && s.revision == w.revision
                && s.revision > 0
                && s.world.room > 0
                && s.world.match_id > 0
                && s.world.generation > 0
                && s.targets.0.len() <= 228
                && s.npcs.is_some() == d.npcs.is_some()
                && d.actors.0.len() <= 100
                && !d.actors.0.is_empty()
                && (-1..=1).contains(&w.lane),
        )?;
        let mut entries = BTreeMap::new();
        let mut previous = None;
        let mut players = 0;
        let mut monsters = 0;
        let mut events = BTreeMap::new();
        let mut latest_targets = BTreeMap::new();
        let mut used = BTreeSet::new();
        let mut actor_ids = BTreeSet::new();
        for e in &s.targets.0 {
            let k = key(&e.target);
            require(
                k.1 > 0
                    && previous.is_none_or(|p| p < k)
                    && (e.active.is_some() || e.walk.is_some()),
            )?;
            previous = Some(k);
            match &e.target {
                Target::Player(p) => {
                    self.actor(p)?;
                    players += 1;
                    require(e.walk.is_none())?;
                }
                Target::Npc(n) => {
                    monsters += 1;
                    require(n.world == w.identity)?;
                }
            }
            if let (Some(a), Some(b)) = (&e.active, &e.walk) {
                require(rev(a) >= rev(b) && (rev(a) != rev(b) || a == b) && life(a)? >= life(b)?)?;
            }
            for g in e.active.iter().chain(e.walk.iter()) {
                require(target(g)? == e.target)?;
                self.grant(g)?;
                add(&mut events, g)?;
            }
            entries.insert(k, e);
        }
        require(players <= 100 && monsters <= 128)?;
        for a in &d.actors.0 {
            require(actor_ids.insert(a.entity))?;
            let p = self.participant(a)?;
            self.actor(&p)?;
            require(w.lane < 0 || self.data.scene_member(a)?.team == w.lane)?;
            for lane in [data::Lane::Haste, data::Lane::Slow] {
                let wanted = lane == data::Lane::Slow
                    || w.descriptor.features & super::super::haste::FEATURE != 0;
                let histories = a
                    .spells
                    .histories
                    .0
                    .iter()
                    .filter(|h| h.lane == lane)
                    .collect::<Vec<_>>();
                require(histories.len() == usize::from(wanted))?;
                if !wanted {
                    continue;
                }
                let h = histories[0];
                require(
                    use_history::Spell::from(h).valid() && h.last.is_some() == h.receipt.is_some(),
                )?;
                if let Some(r) = &h.receipt {
                    require(
                        h.last.as_ref() == Some(&r.impact.usage)
                            && h.ledger.casts == r.casts
                            && h.ledger.spent == r.spent,
                    )?;
                    let g = Grant {
                        caster: p.clone(),
                        receipt: r.clone(),
                    };
                    self.grant(&g)?;
                    if detail(&g)?.changes {
                        latest_targets
                            .entry(key(&target(&g)?))
                            .and_modify(|v: &mut i64| *v = (*v).max(rev(&g)))
                            .or_insert(rev(&g));
                    }
                    add(&mut events, &g)?;
                }
            }
            let b = a.buffs.as_ref().ok_or_else(invalid)?;
            buff(b)?;
            if b.slow_until_nanos == 0 && b.haste_origin != data::HasteOrigin::Haste {
                continue;
            }
            let k = (0, a.entity);
            let e = entries.get(&k).ok_or_else(invalid)?;
            let g = e.active.as_ref().ok_or_else(invalid)?;
            let StatusTarget::Player(t) = &detail(g)?.target else {
                return Err(invalid());
            };
            let r = a.resources.as_ref().ok_or_else(invalid)?;
            require(
                w.phase == data::Phase::Active
                    && r.vitals.hp > 0
                    && !d
                        .time_attack
                        .as_ref()
                        .is_some_and(|t| t.departed.0.contains(&a.entity))
                    && b.entity == a.entity
                    && t.participant == p
                    && t.life == r.life.checked_add(1).ok_or_else(invalid)?
                    && t.equipment_revision <= r.equipment_revision
                    && detail(g)?.changes
                    && g.receipt.impact.lane
                        == if b.slow_until_nanos != 0 {
                            data::Lane::Slow
                        } else {
                            data::Lane::Haste
                        }
                    && (if b.slow_until_nanos != 0 {
                        t.after.slow_until_nanos == b.slow_until_nanos
                            && t.after.slow_revision == b.slow_revision
                    } else {
                        t.after.haste_until_nanos == b.haste_until_nanos
                    })
                    && b.revision >= t.after.revision
                    && b.revision <= s.revision
                    && b.observed_nanos >= detail(g)?.active_elapsed_nanos
                    && b.observed_nanos <= w.active_elapsed_nanos
                    && b.slow_until_nanos.max(b.haste_until_nanos) > w.active_elapsed_nanos,
            )?;
            used.insert(k);
        }
        self.motion(&entries, &mut used)?;
        require(used.len() == s.targets.0.len())?;
        for e in &s.targets.0 {
            if let Some(a) = &e.active {
                require(
                    latest_targets
                        .get(&key(&e.target))
                        .is_none_or(|r| *r <= rev(a)),
                )?;
            }
            for g in e.active.iter().chain(e.walk.iter()) {
                let latest = self
                    .actor(&g.caster)?
                    .spells
                    .histories
                    .0
                    .iter()
                    .find(|h| h.lane == g.receipt.impact.lane)
                    .and_then(|h| h.receipt.as_ref())
                    .ok_or_else(invalid)?;
                follows(&g.receipt, latest)?;
            }
        }
        let mut prior: BTreeMap<i64, &Grant> = BTreeMap::new();
        let mut lanes: BTreeMap<(i64, u8), &Grant> = BTreeMap::new();
        for g in events.values() {
            if let Some(p) = lanes.insert((g.caster.entity, g.receipt.impact.lane as u8), g) {
                follows(&p.receipt, &g.receipt)?;
            }
            if let Some(p) = prior.insert(g.caster.entity, g) {
                event_follows(&p.receipt, &g.receipt)?;
            }
        }
        Ok(())
    }
    fn motion(
        &self,
        entries: &BTreeMap<Key, &Entry>,
        used: &mut BTreeSet<Key>,
    ) -> Result<(), ApplicationError> {
        let (Some(n), Some(body)) = (&self.effects.npcs, &self.data.npcs) else {
            return Ok(());
        };
        let w = &self.data.world;
        require(
            n.world == w.identity
                && n.revision == body.revision
                && n.revision >= 1
                && n.revision <= w.revision
                && n.elapsed_millis == body.elapsed_millis
                && (0..=1_860_000).contains(&n.elapsed_millis)
                && n.elapsed_millis <= w.active_elapsed_nanos.max(0) / 1_000_000
                && n.actors.0.len() == body.actors.0.len()
                && n.actors.0.len() <= 128,
        )?;
        let mut previous = 0;
        for (m, a) in n.actors.0.iter().zip(&body.actors.0) {
            require(
                m.id.world == w.identity
                    && m.id.serial == a.serial
                    && a.serial > previous
                    && a.serial <= body.spawned
                    && m.life == a.life
                    && m.life > 0
                    && (1..=60_000).contains(&m.walk_millis)
                    && (1..=60_000).contains(&m.attack_millis)
                    && a.born_revision >= 1
                    && a.changed_revision >= a.born_revision
                    && m.speed.revision <= a.changed_revision
                    && a.changed_revision <= n.revision
                    && (w.lane < 0 || a.lane == w.lane),
            )?;
            previous = a.serial;
            at(&m.speed, n.elapsed_millis)?;
            speed(&m.walk_speed)?;
            if let Some(step) = &m.last_walk {
                at(&m.walk_speed, step.elapsed_millis)?;
                require(
                    step.life > 0
                        && step.life <= m.life
                        && step.sequence > 0
                        && step.roster_revision > a.born_revision
                        && step.roster_revision <= a.changed_revision
                        && step.sequence <= step.roster_revision - a.born_revision
                        && step.elapsed_millis <= n.elapsed_millis
                        && step.duration_millis == paced(m.walk_millis, &m.walk_speed)
                        && m.walk_speed.revision < step.roster_revision
                        && self.data.manifest.contains_cell(&step.from)
                        && step.to == a.cell
                        && (0..=7).contains(&step.heading),
                )?;
                let (dx, dy) = [
                    (0, -1),
                    (1, -1),
                    (1, 0),
                    (1, 1),
                    (0, 1),
                    (-1, 1),
                    (-1, 0),
                    (-1, -1),
                ][step.heading as usize];
                require(
                    i64::from(step.to.x) - i64::from(step.from.x) == dx
                        && i64::from(step.to.y) - i64::from(step.from.y) == dy,
                )?;
                if a.action == data::NpcAction::Walk {
                    require(
                        step.life == m.life
                            && step.heading == a.heading
                            && a.next_move_millis
                                == step.elapsed_millis + i64::from(step.duration_millis),
                    )?;
                }
            } else {
                require(
                    m.walk_speed
                        == Speed {
                            revision: 0,
                            observed_millis: 0,
                            haste_until_millis: 0,
                            slow_until_millis: 0,
                            slow_revision: 0,
                        }
                        && a.action != data::NpcAction::Walk,
                )?;
            }
            let k = (1, a.serial);
            let e = entries.get(&k);
            let active = e.and_then(|e| e.active.as_ref());
            let walk = e.and_then(|e| e.walk.as_ref());
            if m.speed.haste_until_millis != 0 || m.speed.slow_until_millis != 0 {
                require(w.phase == data::Phase::Active && a.hp > 0)?;
                matches(active.ok_or_else(invalid)?, m.life, a.lane, &m.speed)?;
            } else {
                require(active.is_none())?;
            }
            if m.walk_speed.haste_until_millis != 0 || m.walk_speed.slow_until_millis != 0 {
                matches(
                    walk.ok_or_else(invalid)?,
                    m.last_walk.as_ref().ok_or_else(invalid)?.life,
                    a.lane,
                    &m.walk_speed,
                )?;
            } else {
                require(walk.is_none())?;
            }
            if e.is_some() {
                used.insert(k);
            }
        }
        Ok(())
    }
}
