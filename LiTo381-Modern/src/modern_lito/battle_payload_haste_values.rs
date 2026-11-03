//! Skill43 effects and full values shared by candidates and v44 attachments.
//! Session negotiation and atomic Follower admission remain separate gates.
use super::*;
use crate::modern_lito::{battle_payload::data, timed_protocol, use_history, vital_balance};
use std::collections::{BTreeMap, BTreeSet};
const DURATION: i64 = 1_200_000_000_000;
type Key = (u8, i64);
fn key(t: &Target) -> Key {
    match t {
        Target::Player(p) => (0, p.entity),
        Target::Npc(n) => (1, n.serial),
    }
}
fn detail(g: &Grant) -> Result<&Detail, ApplicationError> {
    match &g.receipt.detail {
        data::Detail::Haste(d) => Ok(d),
        _ => Err(invalid()),
    }
}
fn target(g: &Grant) -> Result<Target, ApplicationError> {
    Ok(match &detail(g)?.target {
        StatusTarget::Player(p) => Target::Player(p.participant.clone()),
        StatusTarget::Npc(n) => Target::Npc(n.id.clone()),
    })
}
fn rev(g: &Grant) -> i64 {
    g.receipt.impact.usage.revision
}
fn life(g: &Grant) -> Result<i64, ApplicationError> {
    Ok(match &detail(g)?.target {
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
fn follows(a: &SpellReceipt, b: &SpellReceipt) -> Result<(), ApplicationError> {
    let count = i128::from(b.casts) - i128::from(a.casts);
    let spent = i128::from(b.spent) - i128::from(a.spent);
    let x = &a.impact.usage;
    let y = &b.impact.usage;
    require(
        count >= 0
            && spent >= 0
            && (if count == 0 {
                a == b
            } else {
                x.revision < y.revision
                    && x.sequence < y.sequence
                    && a.loading_elapsed_nanos <= b.loading_elapsed_nanos
                    && x.before.life <= y.before.life
                    && x.before.equipment_revision <= y.before.equipment_revision
                    && spent >= count * 15
                    && spent <= count * 40
            }),
    )
}
pub(super) fn buff(b: &Buffs) -> Result<(), ApplicationError> {
    require(b.slow_until_nanos == 0 && b.slow_revision == 0)?;
    let s = timed_protocol::State {
        entity_id: b.entity.try_into().map_err(|_| invalid())?,
        revision: b.revision,
        observed_nanos: b.observed_nanos,
        haste_until_nanos: b.haste_until_nanos,
        brave_until_nanos: b.brave_until_nanos,
        wisdom_until_nanos: b.wisdom_until_nanos,
        brave_kind: b.brave_kind.try_into().map_err(|_| invalid())?,
    };
    require((b.haste_origin == data::HasteOrigin::None) == (b.haste_until_nanos == 0))?;
    match b.haste_origin {
        data::HasteOrigin::None | data::HasteOrigin::Potion => s.validate(),
        data::HasteOrigin::GreaterHaste => s.validate_greater_haste(),
        data::HasteOrigin::Haste => {
            s.validate_greater_haste()?;
            require(
                i128::from(b.haste_until_nanos) - i128::from(b.observed_nanos)
                    <= i128::from(DURATION),
            )
        }
    }
}
fn speed(s: &Speed) -> Result<(), ApplicationError> {
    require(
        s.slow_until_millis == 0
            && s.slow_revision == 0
            && s.revision >= 0
            && (0..=1_860_000).contains(&s.observed_millis)
            && s.haste_until_millis >= 0
            && (s.haste_until_millis == 0
                || s.haste_until_millis > s.observed_millis
                    && s.haste_until_millis - s.observed_millis <= DURATION / 1_000_000)
            && (s.revision != 0 || s.observed_millis == 0 && s.haste_until_millis == 0),
    )
}
fn at(s: &Speed, time: i64) -> Result<(), ApplicationError> {
    speed(s)?;
    require(
        time >= s.observed_millis
            && time <= 1_860_000
            && (s.haste_until_millis == 0 || time < s.haste_until_millis),
    )
}
fn paced(base: i32, s: &Speed) -> i32 {
    if s.haste_until_millis == 0 {
        base
    } else {
        ((f64::from(base) * 0.755) as i32).max(1)
    }
}
fn matches(g: &Grant, life: i64, lane: i32, speed: &Speed) -> Result<(), ApplicationError> {
    require(
        matches!(&detail(g)?.target,StatusTarget::Npc(n) if n.life==life&&n.lane==lane&&n.after==*speed),
    )
}
impl View<'_> {
    /// Full single-observation proof, including caller-selected ownership.
    /// This is still a candidate: no session capability, command ACK or UI
    /// Observation is created, and all legacy entry points remain closed.
    pub fn check_values(
        &self,
        player: &data::Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        self.data.check_haste_context(player, world)?;
        self.data.check_combat_body()?;
        self.data.personal_facts(player, world)?;
        self.data.check_scene_body()?;
        crate::modern_lito::battle_follow::clocks(&self.data)?;
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
            p.entity > 0
                && p.entity <= 100
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
        let d = detail(g)?;
        let w = &self.data.world;
        let resources = a.resources.as_ref().ok_or_else(invalid)?;
        require(
            g.caster.identity.kind == data::MemberKind::Human
                && r.impact.lane == data::Lane::Haste
                && r.impact.targets.0.is_empty()
                && vital_balance::State::from(resources).valid()
                && use_history::Cast::from(u).valid()
                && u.revision <= w.revision
                && u.sequence < u.revision
                && r.casts > 0
                && r.casts <= u.sequence
                && (15..=40).contains(&u.cost)
                && i128::from(r.spent) >= i128::from(r.casts) * 15
                && i128::from(r.spent) <= i128::from(r.casts) * 40
                && (r.casts != 1 || r.spent == i64::from(u.cost))
                && u.before.life <= resources.life.checked_add(1).ok_or_else(invalid)?
                && u.before.equipment_revision <= resources.equipment_revision
                && (0..=w.loading_elapsed_nanos).contains(&r.loading_elapsed_nanos)
                && (0..=timed_protocol::MAX_OBSERVED_NANOS).contains(&d.active_elapsed_nanos)
                && d.active_elapsed_nanos <= w.active_elapsed_nanos
                && i128::from(r.loading_elapsed_nanos) - i128::from(d.active_elapsed_nanos)
                    == i128::from(w.loading_elapsed_nanos) - i128::from(w.active_elapsed_nanos)
                && self.data.manifest.contains_cell(&u.before.cell),
        )?;
        let (cell, lane) = match &d.target {
            StatusTarget::Player(p) => {
                let t = self.actor(&p.participant)?;
                let tr = t.resources.as_ref().ok_or_else(invalid)?;
                buff(&p.before)?;
                buff(&p.after)?;
                require(
                    p.life > 0
                        && p.life <= tr.life.checked_add(1).ok_or_else(invalid)?
                        && p.equipment_revision >= 0
                        && p.equipment_revision <= tr.equipment_revision
                        && p.before.entity == p.participant.entity
                        && p.after.entity == p.participant.entity
                        && p.before.revision < p.after.revision
                        && p.before.observed_nanos <= d.active_elapsed_nanos
                        && p.after.revision == u.revision
                        && p.after.observed_nanos == d.active_elapsed_nanos
                        && p.after.haste_until_nanos
                            == d.active_elapsed_nanos
                                .checked_add(DURATION)
                                .ok_or_else(invalid)?
                        && p.after.haste_origin == data::HasteOrigin::Haste,
                )?;
                let alive = |time: i64| {
                    if time > d.active_elapsed_nanos {
                        time
                    } else {
                        0
                    }
                };
                require(
                    p.after.brave_until_nanos == alive(p.before.brave_until_nanos)
                        && p.after.wisdom_until_nanos == alive(p.before.wisdom_until_nanos)
                        && p.after.brave_kind
                            == if p.after.brave_until_nanos == 0 {
                                0
                            } else {
                                p.before.brave_kind
                            },
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
                speed(&n.before)?;
                speed(&n.after)?;
                require(
                    n.id.world == w.identity
                        && n.id.serial > 0
                        && n.id.serial <= raw.spawned
                        && n.life > 0
                        && (0..=1).contains(&n.lane)
                        && n.roster_revision >= 1
                        && n.before.revision <= n.roster_revision
                        && n.after.revision
                            == n.roster_revision.checked_add(1).ok_or_else(invalid)?
                        && n.after.revision < u.revision
                        && n.after.revision <= roster.revision
                        && n.before.observed_millis <= d.active_elapsed_nanos / 1_000_000
                        && n.after.observed_millis == d.active_elapsed_nanos / 1_000_000
                        && n.after.haste_until_millis
                            == n.after.observed_millis + DURATION / 1_000_000,
                )?;
                if let Some(t) = raw.actors.0.iter().find(|a| a.serial == n.id.serial) {
                    require(n.life <= t.life && n.lane == t.lane)?;
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
                && (self.data.world.lane < 0 || lane == self.data.scene_member(a)?.team),
        )?;
        let dx = i64::from(cell.x) - i64::from(u.before.cell.x);
        let dy = i64::from(cell.y) - i64::from(u.before.cell.y);
        let distance = dx.abs() + dy.abs();
        let edge = (dx + 18).abs() + (dy + 18).abs();
        require(distance <= 18 || distance == 19 && (19..=52).contains(&edge))
    }
    /// Cross-field status proof, NOT full scene/sequence/owner admission.
    /// Callers cannot convert this result into an existing Observation.
    pub fn check_effects(&self) -> Result<(), ApplicationError> {
        let d = &self.data;
        let s = &self.effects;
        let w = &d.world;
        require(
            d.world.descriptor.features & FEATURE != 0
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
            let histories = a
                .spells
                .histories
                .0
                .iter()
                .filter(|h| h.lane == data::Lane::Haste)
                .collect::<Vec<_>>();
            require(histories.len() == 1)?;
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
                let k = key(&target(&g)?);
                latest_targets
                    .entry(k)
                    .and_modify(|v: &mut i64| *v = (*v).max(rev(&g)))
                    .or_insert(rev(&g));
                add(&mut events, &g)?;
            }
            let b = a.buffs.as_ref().ok_or_else(invalid)?;
            buff(b)?;
            if b.haste_origin != data::HasteOrigin::Haste {
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
                    && t.after.haste_until_nanos == b.haste_until_nanos
                    && b.revision >= t.after.revision
                    && b.revision <= s.revision
                    && b.observed_nanos >= detail(g)?.active_elapsed_nanos
                    && b.observed_nanos <= w.active_elapsed_nanos
                    && b.haste_until_nanos > w.active_elapsed_nanos,
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
                    .find(|h| h.lane == data::Lane::Haste)
                    .and_then(|h| h.receipt.as_ref())
                    .ok_or_else(invalid)?;
                follows(&g.receipt, latest)?;
            }
        }
        let mut prior: BTreeMap<i64, &Grant> = BTreeMap::new();
        for g in events.values() {
            if let Some(p) = prior.insert(g.caster.entity, g) {
                follows(&p.receipt, &g.receipt)?;
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
            if m.speed.haste_until_millis != 0 {
                require(w.phase == data::Phase::Active && a.hp > 0)?;
                matches(active.ok_or_else(invalid)?, m.life, a.lane, &m.speed)?;
            } else {
                require(active.is_none())?;
            }
            if m.walk_speed.haste_until_millis != 0 {
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
