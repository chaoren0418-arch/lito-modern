//! Direct canonical body clocks. Presentation never invents movement, damage,
//! readiness or authoritative revisions. First attachments suppress historical
//! one-shots; skipped observations produce at most one current action per body.
use super::{
    battle_follow::Observation,
    battle_payload::data as d,
    npc_assets::{NpcAction, NpcAppearance, NpcAssets},
    paced_animation::PacedClock,
    world_scene::{ActorState, BaseAction, SceneSnapshot},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

#[path = "player_body_finish.rs"]
pub(crate) mod finish;

#[cfg(test)]
#[path = "battle_npc_pacing_tests.rs"]
pub(crate) mod npc_pacing_tests;
#[cfg(test)]
#[path = "battle_motion_tests.rs"]
pub(crate) mod tests;

#[derive(Clone, Debug)]
struct Body {
    action: BaseAction,
    clock: PacedClock,
    walk_until: Option<Instant>,
    walk_from: Option<super::world_map::WorldCell>,
    heading: Option<u8>,
    historical_death: bool,
    cue_revision: Option<i64>,
    idle_return: Option<finish::IdleReturn>,
}
impl Body {
    fn initial(actor: &d::Actor, now: Instant) -> Self {
        let dead = dead(actor);
        Self {
            action: if dead {
                BaseAction::Death
            } else {
                BaseAction::Idle
            },
            clock: PacedClock::new(now),
            walk_until: None,
            walk_from: None,
            heading: None,
            historical_death: dead,
            cue_revision: None,
            idle_return: None,
        }
    }
    fn settle(&mut self, now: Instant) -> Result<()> {
        if let Some(end) = self
            .idle_return
            .as_ref()
            .map(|tail| tail.deadline())
            .filter(|end| now >= *end)
        {
            ensure!(
                matches!(self.action, BaseAction::Attack | BaseAction::Damage),
                "invalid returning player action"
            );
            self.idle_return = None;
            self.action = BaseAction::Idle;
            self.cue_revision = None;
            self.clock.restart(end);
            self.clock.retime(end, 1, 1)?;
            // Keep the last confirmed attack-facing direction while breathing.
        }
        if let Some(end) = self.walk_until.filter(|end| now >= *end) {
            self.walk_until = None;
            self.walk_from = None;
            self.action = BaseAction::Idle;
            self.heading = None;
            self.clock.restart(end);
            self.clock.retime(end, 1, 1)?;
        }
        Ok(())
    }
    fn start(&mut self, action: BaseAction, now: Instant, heading: Option<u8>) {
        self.action = action;
        self.clock.restart(now);
        self.walk_until = None;
        self.walk_from = None;
        self.heading = heading;
        self.historical_death = false;
        self.cue_revision = None;
        self.idle_return = None;
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pose {
    pub action: BaseAction,
    pub elapsed: Duration,
    pub heading: Option<u8>,
    pub walk_from: Option<super::world_map::WorldCell>,
}
#[derive(Clone)]
struct NpcBody {
    action: NpcAction,
    started: Instant,
    historical_death: bool,
    cue_revision: Option<i64>,
    duration: Option<Duration>,
    walk_from: Option<super::world_map::WorldCell>,
}
#[derive(Clone, Copy)]
struct NpcPace<'a> {
    attack_millis: i32,
    last_walk: Option<&'a super::battle_payload::npc_motion::Walk>,
}
impl<'a> From<&'a super::battle_payload::npc_motion::Actor> for NpcPace<'a> {
    fn from(a: &'a super::battle_payload::npc_motion::Actor) -> Self {
        Self {
            attack_millis: a.attack_millis,
            last_walk: a.last_walk.as_ref(),
        }
    }
}
impl<'a> From<&'a super::battle_payload::haste::Motion> for NpcPace<'a> {
    fn from(a: &'a super::battle_payload::haste::Motion) -> Self {
        Self {
            attack_millis: if a.speed.slow_until_millis != 0 {
                a.attack_millis * 2
            } else if a.speed.haste_until_millis == 0 {
                a.attack_millis
            } else {
                ((f64::from(a.attack_millis) * 0.755) as i32).max(1)
            },
            // Duration belongs to the speed at commit, NOT current speed.
            last_walk: a.last_walk.as_ref(),
        }
    }
}
fn npc_pace<'a>(
    data: &'a d::UnvalidatedData,
    actor: &d::Npc,
) -> Result<Option<(NpcPace<'a>, i64)>> {
    let targeted = data
        .speed_state()
        .map(|e| &e.npcs)
        .or_else(|| data.haste_state().map(|e| &e.npcs));
    if let Some(state) = targeted {
        let npcs = state.as_ref().context("targeted speed NPC motion absent")?;
        let a = npcs
            .actors
            .0
            .iter()
            .find(|a| {
                a.id.world == data.world.identity
                    && a.id.serial == actor.serial
                    && a.life == actor.life
            })
            .context("haste NPC motion identity absent")?;
        return Ok(Some((a.into(), npcs.elapsed_millis)));
    }
    if let Some(motion) = data.motion_state() {
        let npcs = motion.as_ref().context("NPC motion state absent")?;
        let a = npcs
            .actors
            .0
            .iter()
            .find(|a| a.serial == actor.serial && a.life == actor.life)
            .context("NPC motion actor absent")?;
        return Ok(Some((a.into(), npcs.elapsed_millis)));
    }
    Ok(None)
}
impl NpcBody {
    fn initial(actor: &d::Npc, now: Instant) -> Self {
        Self {
            action: if actor.hp == 0 {
                NpcAction::Death
            } else {
                NpcAction::Breath
            },
            started: now,
            historical_death: actor.hp == 0,
            cue_revision: None,
            duration: None,
            walk_from: None,
        }
    }
    fn change(
        &mut self,
        before: &d::Npc,
        after: &d::Npc,
        action: NpcAction,
        facts: Option<(NpcPace<'_>, NpcPace<'_>, i64)>,
        now: Instant,
    ) -> Result<()> {
        self.action = action;
        self.started = now;
        self.historical_death = false;
        self.cue_revision = Some(after.changed_revision);
        self.duration = None;
        self.walk_from = None;
        let Some((old, next, elapsed)) = facts else {
            return Ok(());
        };
        if action == NpcAction::Attack {
            self.duration = Some(Duration::from_millis(next.attack_millis.try_into()?));
        } else if action == NpcAction::Walk {
            let w = next
                .last_walk
                .as_ref()
                .context("confirmed NPC walk absent")?;
            let previous = old.last_walk.as_ref().map_or(0, |w| w.sequence);
            let age = elapsed
                .checked_sub(w.elapsed_millis)
                .context("NPC walk age overflow")?;
            ensure!(age >= 0, "NPC walk precedes authority clock");
            // A validated coalesced update still owns the last confirmed step.
            // Animate that step's remaining time from its published origin;
            // never interpolate/replay the missing route from before.cell.
            // Initial attachment and new lives are handled by initial().
            if w.sequence <= previous
                || (previous.checked_add(1) == Some(w.sequence) && before.cell != w.from)
                || after.cell != w.to
                || w.life != after.life
                || age >= i64::from(w.duration_millis)
            {
                self.action = NpcAction::Breath;
                self.cue_revision = None;
            } else {
                self.started = now
                    .checked_sub(Duration::from_millis(age.try_into()?))
                    .context("NPC walk timestamp underflow")?;
                self.duration = Some(Duration::from_millis(w.duration_millis.try_into()?));
                self.walk_from = Some(super::world_map::WorldCell {
                    x: w.from.x,
                    y: w.from.y,
                });
            }
        }
        Ok(())
    }
    fn sample(&self, actor: &d::Npc, bank: &NpcAssets, now: Instant) -> Result<NpcSample> {
        ensure!(now >= self.started, "NPC sample clock reversed");
        let mut action = self.action;
        let mut elapsed = if self.historical_death {
            Duration::MAX
        } else {
            now.checked_duration_since(self.started)
                .context("NPC sample clock reversed")?
        };
        let form = NpcAppearance::new(actor.graphic.try_into()?)?;
        let mut source_duration = bank.action(form, action)?.duration();
        let mut real_duration = self.duration.unwrap_or(source_duration);
        let mut started = self.started;
        if !matches!(action, NpcAction::Death | NpcAction::Breath) && elapsed >= real_duration {
            action = NpcAction::Breath;
            elapsed = elapsed.saturating_sub(real_duration);
            started = started
                .checked_add(real_duration)
                .context("NPC idle timestamp overflow")?;
            source_duration = bank.action(form, action)?.duration();
            real_duration = source_duration;
        }
        let phase = scaled_npc_time(elapsed, real_duration, source_duration, false)?;
        let (source, cycle) = bank
            .action(form, action)?
            .sample(phase, actor.heading.try_into()?)?;
        Ok(NpcSample {
            action,
            source,
            cycle,
            started,
            source_duration,
            real_duration,
            // Shared walk SPRs contain displacement. Render source1..N-1 at
            // the old cell and terminal0 at the confirmed destination. Never
            // add camera pulses/linear displacement a second time to the body.
            walk_from: self
                .walk_from
                .filter(|_| action == NpcAction::Walk && source != 0),
            stamp: NpcStamp {
                action: self.action,
                started: self.started,
                revision: self.cue_revision,
                historical_death: self.historical_death,
            },
        })
    }
}
fn scaled_npc_time(value: Duration, from: Duration, to: Duration, ceil: bool) -> Result<Duration> {
    ensure!(
        !from.is_zero() && !to.is_zero(),
        "NPC animation duration is zero"
    );
    if from == to {
        return Ok(value);
    }
    let n = value
        .as_nanos()
        .checked_mul(to.as_nanos())
        .context("NPC phase overflow")?;
    let divisor = from.as_nanos();
    let nanos = n / divisor + u128::from(ceil && n % divisor != 0);
    Ok(Duration::from_nanos(nanos.try_into()?))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NpcStamp {
    pub action: NpcAction,
    pub started: Instant,
    pub revision: Option<i64>,
    pub historical_death: bool,
}
pub(crate) struct NpcSample {
    pub action: NpcAction,
    pub source: usize,
    pub cycle: u128,
    pub started: Instant,
    pub stamp: NpcStamp,
    pub walk_from: Option<super::world_map::WorldCell>,
    source_duration: Duration,
    real_duration: Duration,
}
impl NpcSample {
    pub(crate) fn cue_time(&self, source_phase: Duration) -> Result<Instant> {
        self.started
            .checked_add(scaled_npc_time(
                source_phase,
                self.source_duration,
                self.real_duration,
                true,
            )?)
            .context("NPC cue timestamp overflow")
    }
}
#[derive(Clone)]
pub(crate) struct Timeline {
    observation: Observation,
    observed: Instant,
    bodies: BTreeMap<i64, Body>,
    npcs: BTreeMap<i64, NpcBody>,
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "canonical motion cancelled"
    );
    Ok(())
}
fn dead(a: &d::Actor) -> bool {
    a.vital.as_ref().is_some_and(|v| v.values.hp == 0)
}
/// Fit a complete qualified shared walk cycle into one authority step. Without
/// this mapping a 200ms step repeatedly resets a 667ms source cycle before most
/// frames can be seen. This changes body phase only, never position/cooldowns.
/// Ceil the nanosecond boundary so an exact source tick is not lost to rounding.
fn walk_elapsed(elapsed: Duration, step: u64, ticks: u64, fps: u32) -> Result<Duration> {
    ensure!(
        step > 0 && ticks > 0 && fps > 0,
        "invalid canonical walk cycle"
    );
    let numerator = elapsed
        .as_nanos()
        .min(u128::from(step))
        .checked_mul(u128::from(ticks))
        .and_then(|n| n.checked_mul(1_000_000_000))
        .context("canonical walk phase overflow")?;
    let denominator = u128::from(step) * u128::from(fps);
    let nanos = numerator / denominator + u128::from(numerator % denominator != 0);
    Ok(Duration::from_nanos(u64::try_from(nanos)?))
}
fn cast(data: &d::UnvalidatedData, a: &d::Actor) -> i64 {
    a.spells
        .histories
        .0
        .iter()
        .map(|s| s.ledger.last_cast_revision)
        .chain(a.spells.structure.as_ref().map(|s| s.impact.usage.revision))
        .chain(
            data.healing_actors()
                .and_then(|h| h.0.iter().find(|h| h.entity == a.entity))
                .and_then(|h| h.outgoing.as_ref())
                .map(|h| h.ledger.last_cast_revision),
        )
        .max()
        .unwrap_or(0)
}
/// Resource `life` counts completed respawns from zero. UseHistory anchors and
/// NPC exchanges carry the one-based incarnation (Engine.useAnchor /
/// participantLife). This is a domain conversion, never an Engine revision.
pub(crate) fn life(a: &d::Actor) -> Option<i64> {
    a.resources
        .as_ref()
        .map_or(Some(1), |r| r.life.checked_add(1))
}
fn weapon(a: &d::Actor) -> i32 {
    a.vital.as_ref().map_or(0, |v| v.weapon_type)
}
fn shape(data: &d::UnvalidatedData, entity: i64) -> i64 {
    data.vip
        .as_ref()
        .and_then(|v| v.appearances.as_ref())
        .and_then(|v| v.0.iter().find(|a| a.entity == entity))
        .map_or(0, |a| a.graphics)
}
fn heading(from: &d::Cell, to: &d::Cell) -> Option<u8> {
    let delta = (
        (i64::from(to.x) - i64::from(from.x)).signum() as i32,
        (i64::from(to.y) - i64::from(from.y)).signum() as i32,
    );
    super::world_map::DIRECTIONS
        .iter()
        .position(|p| *p == delta)
        .map(|i| i as u8)
}
fn portal(before: &d::Actor, after: &d::Actor, data: &d::UnvalidatedData) -> bool {
    data.siege.as_ref().is_some_and(|s| {
        after.last_move_sequence > before.last_move_sequence
            && ((before.position == s.portal_inside && after.position == s.portal_outside)
                || (before.position == s.portal_outside && after.position == s.portal_inside))
    })
}
fn teleported(a: &d::Actor, previous_revision: i64) -> bool {
    a.spells
        .histories
        .0
        .iter()
        .any(|s| s.lane == d::Lane::Teleport && s.ledger.last_cast_revision > previous_revision)
}
fn single_step(before: &d::Actor, after: &d::Actor) -> bool {
    before.last_move_sequence.checked_add(1) == Some(after.last_move_sequence)
        && (i64::from(after.position.x) - i64::from(before.position.x)).abs() <= 1
        && (i64::from(after.position.y) - i64::from(before.position.y)).abs() <= 1
        && heading(&before.position, &after.position) == u8::try_from(after.heading).ok()
}
/// Keep the exact binary64 multiply/truncate stages used by the server and
/// existing timed protocol. This uses canonical values, not a legacy Frame.
fn pace(a: &d::Actor, action: BaseAction) -> (u64, u64) {
    let base = match action {
        BaseAction::Walk => a.step_interval_nanos as u64,
        BaseAction::Attack => a
            .vital
            .as_ref()
            .map_or(1, |v| v.attack_interval_nanos as u64),
        _ => return (1, 1),
    };
    (base, paced_nanos(a, base, action == BaseAction::Attack))
}
pub(crate) fn paced_nanos(a: &d::Actor, base: u64, attack: bool) -> u64 {
    let Some(b) = &a.buffs else {
        return base;
    };
    let haste = b.haste_until_nanos > b.observed_nanos;
    let brave = b.brave_until_nanos > b.observed_nanos;
    let slow = b.slow_until_nanos > b.observed_nanos;
    if !haste && !brave && !slow {
        return base;
    }
    let mut millis = (base / 1_000_000).max(1);
    if slow {
        millis *= 2;
    } else if haste {
        millis = (millis as f64 * 0.755) as u64;
    }
    if brave {
        millis = (millis as f64 * 0.755) as u64;
    }
    if brave && b.brave_kind == 3 {
        millis = (millis as f64 * 0.855) as u64;
        if attack {
            millis = (millis as f64 * 0.9) as u64;
        }
    }
    millis.max(1) * 1_000_000
}
pub(crate) fn latest_body_event(
    data: &d::UnvalidatedData,
    actor: &d::Actor,
    member: &d::Member,
) -> i64 {
    latest_deliberate_event(data, actor, member).max(
        actor
            .vital
            .as_ref()
            .map_or(0, |v| v.hurt_revision.max(v.death_revision)),
    )
}
/// Hurt owns the visible body but does not erase a committed pending spell.
pub(crate) fn latest_deliberate_event(
    data: &d::UnvalidatedData,
    actor: &d::Actor,
    member: &d::Member,
) -> i64 {
    attack(data, actor, member).revision.max(cast(data, actor))
}
struct Attack {
    revision: i64,
    cell: Option<d::Cell>,
}
fn attack(data: &d::UnvalidatedData, actor: &d::Actor, member: &d::Member) -> Attack {
    let mut result = Attack {
        revision: 0,
        cell: None,
    };
    let mut retain = |revision, cell: Option<d::Cell>| {
        if revision > result.revision {
            result = Attack { revision, cell };
        }
    };
    if let Some(v) = &actor.vital {
        let target = data
            .actors
            .0
            .iter()
            .find(|a| a.entity == v.target)
            .map(|a| a.position.clone())
            .or_else(|| {
                data.honor
                    .objectives
                    .0
                    .iter()
                    .find(|t| t.entity == v.target)
                    .map(|t| t.cell.clone())
            });
        retain(v.attack_revision, target);
    }
    if let Some(n) = &data.npcs {
        for e in &n.exchanges.0 {
            if e.direction == d::Direction::PlayerToNpc
                && e.participant == member.identity
                && Some(e.participant_life) == life(actor)
            {
                retain(
                    e.revision,
                    n.actors
                        .0
                        .iter()
                        .find(|a| a.serial == e.serial && a.life == e.npc_life)
                        .map(|a| a.cell.clone()),
                );
            }
        }
    }
    if let Some(q) = &data.quest {
        for e in &q.exchanges.0 {
            if e.participant == member.identity {
                retain(
                    e.revision,
                    q.targets
                        .0
                        .iter()
                        .find(|t| t.lane * 4 + t.stage == e.target.slot)
                        .map(|t| t.cell.clone()),
                );
            }
        }
    }
    if let Some(t) = &data.time_attack {
        for e in &t.exchanges.0 {
            if e.entity == actor.entity {
                retain(e.revision, Some(t.tower.cell.clone()));
            }
        }
    }
    if let Some(s) = &data.siege {
        for t in &s.objectives.0 {
            if t.last_actor == actor.entity && t.hit_count > 0 {
                retain(t.revision, Some(t.tower.cell.clone()));
            }
        }
    }
    result
}
impl Timeline {
    pub(crate) fn new(observation: Observation, now: Instant) -> Result<Self> {
        ensure!(
            observation.data().world.phase != d::Phase::Aborted,
            "aborted motion must retire"
        );
        let bodies = observation
            .data()
            .actors
            .0
            .iter()
            .map(|a| (a.entity, Body::initial(a, now)))
            .collect();
        let npcs = observation
            .data()
            .npcs
            .as_ref()
            .into_iter()
            .flat_map(|n| &n.actors.0)
            .map(|a| (a.serial, NpcBody::initial(a, now)))
            .collect();
        Ok(Self {
            observation,
            observed: now,
            bodies,
            npcs,
        })
    }
    /// Build a candidate, never partially mutate the accepted timeline.
    pub(crate) fn advance(
        &self,
        next: Observation,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(now >= self.observed, "canonical motion clock reversed");
        self.observation.check_successor(&next)?;
        ensure!(
            next.data().world.phase != d::Phase::Aborted,
            "aborted motion must retire"
        );
        let old = self.observation.data();
        let data = next.data();
        let mut candidate = self.clone();
        for (before, after) in old.actors.0.iter().zip(&data.actors.0) {
            active(cancel)?;
            let member = data.scene_member(after)?;
            let clock = candidate
                .bodies
                .get_mut(&after.entity)
                .context("canonical body absent")?;
            clock.settle(now)?;
            // A newly observed life/weapon is not evidence that an earlier
            // retained attack was performed by this life or with this weapon.
            if life(before) != life(after)
                || weapon(before) != weapon(after)
                || shape(old, before.entity) != shape(data, after.entity)
            {
                *clock = Body::initial(after, now);
            } else if data.world.revision > old.world.revision {
                let movement = before.position != after.position;
                let cast = cast(data, after);
                let attack = attack(data, after, member);
                let hurt = after.vital.as_ref().map_or(0, |v| v.hurt_revision);
                let death = after.vital.as_ref().map_or(0, |v| v.death_revision);
                let new_cast = cast > old.world.revision;
                let discontinuity =
                    teleported(after, old.world.revision) || portal(before, after, data);
                if movement || new_cast || discontinuity {
                    clock.start(
                        if movement && !new_cast && !discontinuity && single_step(before, after) {
                            BaseAction::Walk
                        } else {
                            BaseAction::Idle
                        },
                        now,
                        None,
                    );
                    if clock.action == BaseAction::Walk {
                        clock.walk_from = Some(super::world_map::WorldCell {
                            x: before.position.x,
                            y: before.position.y,
                        });
                    }
                }
                if dead(after) {
                    if !dead(before) || death > old.world.revision {
                        clock.start(BaseAction::Death, now, None);
                        clock.cue_revision = Some(death);
                    }
                    clock.action = BaseAction::Death;
                    clock.walk_until = None;
                } else if hurt > old.world.revision && hurt >= attack.revision && hurt >= cast {
                    clock.start(BaseAction::Damage, now, None);
                    clock.cue_revision = Some(hurt);
                } else if attack.revision > old.world.revision
                    && attack.revision >= cast
                    && !(movement && data.world.revision > attack.revision)
                {
                    clock.start(
                        BaseAction::Attack,
                        now,
                        attack
                            .cell
                            .as_ref()
                            .and_then(|c| heading(&after.position, c)),
                    );
                    clock.cue_revision = Some(attack.revision);
                }
            }
            let (base, effective) = pace(after, clock.action);
            clock.clock.retime(now, base, effective)?;
            if let Some(tail) = &mut clock.idle_return {
                tail.retime(&clock.clock, now)?;
            }
            if clock.action == BaseAction::Walk {
                clock.walk_until = Some(clock.clock.until(now, Duration::from_nanos(base))?);
            }
        }
        let actors = data
            .npcs
            .as_ref()
            .map(|n| n.actors.0.as_slice())
            .unwrap_or(&[]);
        candidate
            .npcs
            .retain(|serial, _| actors.iter().any(|a| a.serial == *serial));
        for actor in actors {
            active(cancel)?;
            let before = old
                .npcs
                .as_ref()
                .and_then(|n| n.actors.0.iter().find(|a| a.serial == actor.serial));
            let Some(before) =
                before.filter(|a| a.life == actor.life && a.graphic == actor.graphic)
            else {
                candidate
                    .npcs
                    .insert(actor.serial, NpcBody::initial(actor, now));
                continue;
            };
            let body = candidate
                .npcs
                .get_mut(&actor.serial)
                .context("canonical NPC body absent")?;
            if actor.changed_revision <= before.changed_revision {
                continue;
            }
            let new_attack = data.npcs.as_ref().is_some_and(|n| {
                n.exchanges.0.iter().any(|e| {
                    e.direction == d::Direction::NpcToParticipant
                        && e.serial == actor.serial
                        && e.npc_life == actor.life
                        && e.revision > old.world.revision
                })
            });
            let facts = match (npc_pace(old, before)?, npc_pace(data, actor)?) {
                (Some((a, _)), Some((b, elapsed))) => Some((a, b, elapsed)),
                (None, None) => None,
                _ => anyhow::bail!("NPC motion schema changed"),
            };
            let moved = facts.map_or(actor.cell != before.cell, |(a, b, _)| {
                a.last_walk.as_ref().map(|w| w.sequence) != b.last_walk.as_ref().map(|w| w.sequence)
            });
            let action = if actor.hp == 0 && before.hp > 0 {
                Some(NpcAction::Death)
            } else if actor.action == d::NpcAction::Hurt && actor.hp < before.hp {
                Some(NpcAction::Damage)
            } else if actor.action == d::NpcAction::Attack && new_attack {
                Some(NpcAction::Attack)
            } else if actor.action == d::NpcAction::Walk && moved {
                Some(NpcAction::Walk)
            } else if actor.action == d::NpcAction::Idle && before.action != actor.action {
                Some(NpcAction::Breath)
            } else {
                None
            };
            if let Some(action) = action {
                body.change(before, actor, action, facts, now)?;
            }
        }
        active(cancel)?;
        candidate.observation = next;
        candidate.observed = now;
        Ok(candidate)
    }
    /// Resource-qualified presentation policy, attached at the same commit as
    /// a newly observed action. Unqualified/legacy routes remain unchanged.
    pub(crate) fn with_player_returns(
        &self,
        proofs: &BTreeMap<i64, finish::QualifiedReturn>,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(now == self.observed, "player completion commit mismatch");
        let mut candidate = self.clone();
        for (entity, proof) in proofs {
            active(cancel)?;
            let body = candidate
                .bodies
                .get_mut(entity)
                .context("player completion body absent")?;
            ensure!(
                body.action == proof.action()
                    && body.cue_revision.is_some()
                    && !body.historical_death,
                "player completion owner/action mismatch"
            );
            if let Some(tail) = &body.idle_return {
                ensure!(
                    tail.matches(*proof),
                    "player completion resource changed without action reset"
                );
            } else {
                body.idle_return = Some(finish::IdleReturn::start(*proof, &body.clock, now)?);
            }
        }
        active(cancel)?;
        Ok(candidate)
    }
    pub(crate) fn with_base_player_returns(
        &self,
        scene: &SceneSnapshot,
        assets: &super::world_scene::WorldSceneAssets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let mut proofs = BTreeMap::new();
        // Race bodies use a separate qualified NPC action/pixel clock. Their
        // completion policy must not be inferred from the player's base class.
        if scene.pet().is_none() {
            for actor in scene.actors() {
                active(cancel)?;
                let entity = i64::try_from(actor.entity_id)?;
                let action = self.pose(entity, now)?.action;
                if matches!(action, BaseAction::Attack | BaseAction::Damage) {
                    let animation = assets
                        .combat_animation_armed(actor.appearance, actor.weapon_type, action)
                        .context("base player completion animation absent")?;
                    proofs.insert(entity, finish::QualifiedReturn::base(action, animation)?);
                }
            }
        }
        self.with_player_returns(&proofs, now, cancel)
    }
    pub(crate) fn pose(&self, entity: i64, now: Instant) -> Result<Pose> {
        ensure!(now >= self.observed, "canonical body sample clock reversed");
        let mut body = self
            .bodies
            .get(&entity)
            .context("canonical body absent")?
            .clone();
        body.settle(now)?;
        Ok(Pose {
            action: body.action,
            heading: body.heading,
            walk_from: body.walk_from,
            elapsed: if body.historical_death {
                Duration::MAX
            } else {
                body.clock.peek(now)?
            },
        })
    }
    /// Sample the very same body accumulator used by scene pixels. A renderer
    /// keeps an already crossed deadline; this only predicts a future marker.
    pub(crate) fn attack_deadline(
        &self,
        entity: i64,
        phase: Duration,
        now: Instant,
    ) -> Result<Instant> {
        ensure!(now >= self.observed, "canonical bow clock reversed");
        let body = self
            .bodies
            .get(&entity)
            .context("canonical bow body absent")?;
        ensure!(
            body.action == BaseAction::Attack,
            "canonical bow body replaced"
        );
        body.clock.clone().until(now, phase)
    }
    pub(crate) fn matches_observation(&self, observation: &Observation) -> bool {
        self.observation.data() == observation.data()
    }
    pub(crate) fn player_audio(
        &self,
        entity: i64,
        now: Instant,
    ) -> Result<Option<(i64, BaseAction, &PacedClock)>> {
        ensure!(now >= self.observed, "canonical body audio clock reversed");
        let body = self.bodies.get(&entity).context("canonical body absent")?;
        if body
            .idle_return
            .as_ref()
            .is_some_and(|tail| now >= tail.deadline())
        {
            return Ok(None);
        }
        Ok(body
            .cue_revision
            .map(|revision| (revision, body.action, &body.clock)))
    }
    pub(crate) fn walk_audio(&self, entity: i64, now: Instant) -> Result<Option<&PacedClock>> {
        let pose = self.pose(entity, now)?;
        Ok(
            (pose.action == BaseAction::Walk && pose.walk_from.is_some())
                .then(|| &self.bodies[&entity].clock),
        )
    }
    pub(crate) fn scene(
        &self,
        template: &SceneSnapshot,
        now: Instant,
        mut walk_cycle: impl FnMut(&ActorState) -> Result<(u64, u32)>,
    ) -> Result<SceneSnapshot> {
        let mut actors = template.actors().to_vec();
        for a in &mut actors {
            let pose = self.pose(a.entity_id as i64, now)?;
            a.action = pose.action;
            a.action_elapsed = pose.elapsed;
            if a.action == BaseAction::Walk && template.pet().is_none() {
                let actor = self
                    .observation
                    .data()
                    .actors
                    .0
                    .iter()
                    .find(|v| v.entity == a.entity_id as i64)
                    .context("canonical walker absent")?;
                let (ticks, fps) = walk_cycle(a)?;
                a.action_elapsed =
                    walk_elapsed(pose.elapsed, actor.step_interval_nanos as u64, ticks, fps)?;
            }
            if let Some(heading) = pose.heading {
                a.heading = heading;
            }
        }
        template.with_actors(actors)
    }

    /// Only confirmed adjacent single steps have a visual origin. A skipped
    /// sequence, teleport, new life/weapon or later body action must snap to
    /// the confirmed cell instead of inventing an intermediate route.
    pub(crate) fn walks(
        &self,
        snapshot: &SceneSnapshot,
        assets: &super::world_scene::WorldSceneAssets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Vec<super::world_scene::PlayerWalkPlacement>> {
        let mut result = Vec::new();
        if snapshot.pet().is_some() {
            return Ok(result);
        }
        for a in snapshot.actors() {
            active(cancel)?;
            let pose = self.pose(a.entity_id as i64, now)?;
            if let Some(from) = pose.walk_from.filter(|_| pose.action == BaseAction::Walk) {
                let actor = self
                    .observation
                    .data()
                    .actors
                    .0
                    .iter()
                    .find(|v| v.entity == a.entity_id as i64)
                    .context("canonical walker absent")?;
                result.push(assets.player_walk(
                    snapshot,
                    a.entity_id,
                    from,
                    pose.elapsed,
                    actor.step_interval_nanos as u64,
                    cancel,
                )?);
            }
        }
        Ok(result)
    }
    #[cfg(test)]
    pub(crate) fn npc_pose(&self, serial: i64, now: Instant) -> Result<(NpcAction, Duration)> {
        ensure!(now >= self.observed, "canonical NPC sample clock reversed");
        let body = self
            .npcs
            .get(&serial)
            .context("canonical NPC body absent")?;
        Ok((
            body.action,
            if body.historical_death {
                Duration::MAX
            } else {
                now.duration_since(body.started)
            },
        ))
    }
    /// One shared pose/sample for pixels and cues, including finite-action
    /// fallback to Breath. No duplicate audio-specific animation schedule.
    pub(crate) fn npc_sample(
        &self,
        actor: &d::Npc,
        bank: &NpcAssets,
        now: Instant,
    ) -> Result<NpcSample> {
        let body = self
            .npcs
            .get(&actor.serial)
            .context("canonical NPC body absent")?;
        ensure!(now >= self.observed, "canonical NPC sample clock reversed");
        body.sample(actor, bank, now)
    }
}
