//! Public schema-2 ordinary bows: qualified action21 Hit launches shared SPR66.
//! Uses the scene's body clock, not a second animation clock or legacy Frame.
//! Native class3 fixes a hit target at launch; misses keep the receipt cell.
//! No homing, impact sprite, guessed sound, inventory or combat authority.
use super::{
    battle_follow::Observation,
    battle_motion,
    battle_payload::data as d,
    compositor::Canvas,
    ranged_effect::{RangedAssets, RangedError, RangedPath},
    shared_action_catalog::SharedActionCatalog,
    sprite_actions::ActionKey,
    world_map::{ViewProjection, WorldCell, WorldMap},
    world_protocol::MatchDescriptor,
    world_scene::{BaseAction, BaseAppearance, CombatAppearance, WorldSceneAssets},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const MAX_FLIGHTS: usize = 256;
const MAX_AGE: Duration = Duration::from_millis(500);
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "canonical bows cancelled");
    Ok(())
}
fn cell(c: &d::Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}
fn recent(receipt: i64, observed: i64) -> bool {
    observed
        .checked_sub(receipt)
        .is_some_and(|n| (0..=500_000_000).contains(&n))
}
fn events(data: &d::UnvalidatedData) -> impl Iterator<Item = (i64, &d::PublicShot)> {
    data.presentation
        .iter()
        .flat_map(|p| &p.0)
        .filter_map(|p| p.shot.as_ref().map(|s| (p.entity, s)))
}
fn target(data: &d::UnvalidatedData, shot: &d::PublicShot) -> Result<WorldCell> {
    if !shot.hit {
        return Ok(cell(&shot.to));
    }
    match &shot.target {
        d::Target::Participant { entity, identity } => {
            let actor = data
                .actors
                .0
                .iter()
                .find(|a| a.entity == *entity)
                .context("bow target absent")?;
            ensure!(
                &data.scene_member(actor)?.identity == identity,
                "bow target identity changed"
            );
            Ok(cell(
                if battle_motion::life(actor) == Some(shot.target_life)
                    && data
                        .time_attack
                        .as_ref()
                        .is_none_or(|t| !t.departed.0.contains(entity))
                {
                    &actor.position
                } else {
                    &shot.to
                },
            ))
        }
        d::Target::Honor { entity, team } => Ok(cell(
            &data
                .honor
                .objectives
                .0
                .iter()
                .find(|t| t.entity == *entity && t.team == *team)
                .context("bow Honor target absent")?
                .cell,
        )),
        _ => anyhow::bail!("unqualified bow target domain"),
    }
}
pub(crate) fn check_map(observation: &Observation, map: &WorldMap) -> Result<()> {
    for (_, s) in events(observation.data()) {
        ensure!(
            map.standing(cell(&s.from))
                && map.standing(cell(&s.to))
                && map.standing(target(observation.data(), s)?),
            "bow event outside qualified terrain"
        );
    }
    Ok(())
}
fn form(data: &d::UnvalidatedData, actor: &d::Actor) -> Result<CombatAppearance> {
    let member = data.scene_member(actor)?;
    Ok(CombatAppearance {
        appearance: BaseAppearance {
            class: member.class.try_into()?,
            sex: member.sex.try_into()?,
        },
        weapon_type: actor
            .vital
            .as_ref()
            .context("bow vital absent")?
            .weapon_type
            .try_into()?,
    })
}
pub(crate) struct Assets {
    arrow: Option<RangedAssets>,
    delays: BTreeMap<CombatAppearance, Duration>,
}
impl Assets {
    pub(crate) fn load(
        catalog: &SharedActionCatalog,
        observation: &Observation,
        scene: &WorldSceneAssets,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let enabled = observation.data().presentation.is_some()
            && observation.data().world.descriptor.features as u32 & MatchDescriptor::AMMO != 0;
        let mut delays = BTreeMap::new();
        if enabled {
            for actor in &observation.data().actors.0 {
                let form = form(observation.data(), actor)?;
                if !super::ammo_protocol::bow(form.weapon_type) {
                    continue;
                }
                let animation = scene
                    .combat_animation_armed(form.appearance, form.weapon_type, BaseAction::Attack)
                    .context("bow attack bank absent")?;
                ensure!(
                    animation.provenance().action_key == ActionKey::new(21),
                    "bow action is not21"
                );
                // Pure shared-animation qualification, no legacy frame input.
                delays.insert(
                    form,
                    super::world_runtime::bow_first_hit_delay(animation.animation())?,
                );
            }
        }
        Ok(Self {
            arrow: enabled
                .then(|| RangedAssets::load_with_catalog(catalog, cancel))
                .transpose()?,
            delays,
        })
    }
    pub(crate) fn compatible(&self, old: &Self) -> bool {
        self.arrow.is_some() == old.arrow.is_some()
    }
    fn delay(&self, form: CombatAppearance) -> Result<Duration> {
        self.delays
            .get(&form)
            .copied()
            .context("qualified bow Hit absent")
    }
}
#[derive(Clone)]
struct Pending {
    shot: d::PublicShot,
    sequence: i64,
    form: CombatAppearance,
    deadline: Instant,
}
#[derive(Clone)]
struct Flight {
    shot: d::PublicShot,
    path: RangedPath,
    started: Instant,
}
#[derive(Clone)]
pub(crate) struct Timeline {
    observation: Observation,
    observed: Instant,
    pending: BTreeMap<i64, Pending>,
    flights: Vec<Flight>,
    pub(crate) skipped: bool,
}
fn current(data: &d::UnvalidatedData, actor: &d::Actor, pending: &Pending) -> Result<bool> {
    let shot = &pending.shot;
    Ok(actor
        .vital
        .as_ref()
        .is_some_and(|v| v.values.hp > 0 && v.attack_revision == shot.revision)
        && battle_motion::life(actor) == Some(shot.life)
        && actor
            .resources
            .as_ref()
            .is_some_and(|r| r.equipment_revision == shot.equipment_revision)
        && actor.last_move_sequence == pending.sequence
        && actor.position == shot.from
        && data
            .time_attack
            .as_ref()
            .is_none_or(|t| !t.departed.0.contains(&actor.entity))
        && form(data, actor)? == pending.form
        && battle_motion::latest_body_event(data, actor, data.scene_member(actor)?)
            <= shot.revision)
}
impl Timeline {
    pub(crate) fn new(observation: Observation, now: Instant) -> Self {
        Self {
            observation,
            observed: now,
            pending: BTreeMap::new(),
            flights: Vec::new(),
            skipped: false,
        }
    }
    pub(crate) fn advance(
        &self,
        next: Observation,
        now: Instant,
        assets: &Assets,
        motion: &battle_motion::Timeline,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(now >= self.observed, "canonical bow clock reversed");
        self.observation.check_successor(&next)?;
        ensure!(
            next.data().world.phase != d::Phase::Aborted,
            "aborted bows must retire"
        );
        ensure!(
            motion.matches_observation(&next),
            "bow body observation mismatch"
        );
        let data = next.data();
        let mut out = self.clone();
        out.flights
            .retain(|f| !f.path.sample(now.duration_since(f.started)).finished);
        for (entity, shot) in events(data) {
            active(cancel)?;
            if shot.revision <= self.observation.data().world.revision {
                continue;
            }
            let old = events(self.observation.data())
                .find(|(id, _)| *id == entity)
                .map_or(0, |(_, s)| s.count);
            out.skipped |= shot.count.checked_sub(old) != Some(1);
            let actor = data
                .actors
                .0
                .iter()
                .find(|a| a.entity == entity)
                .context("bow actor absent")?;
            let before = self
                .observation
                .data()
                .actors
                .0
                .iter()
                .find(|a| a.entity == entity)
                .context("bow baseline absent")?;
            // The public receipt has no move sequence. If a coalesced interval
            // contains moves and a later revision, ordering is not reconstructible.
            // Drop this visual explicitly instead of inventing an old bow action.
            if !recent(shot.loading_elapsed_nanos, data.world.loading_elapsed_nanos)
                || actor.last_move_sequence != before.last_move_sequence
                    && shot.revision < data.world.revision
                || motion.pose(entity, now)?.action != BaseAction::Attack
            {
                out.pending.remove(&entity);
                out.skipped = true;
                continue;
            }
            let form = form(data, actor)?;
            if !super::ammo_protocol::bow(form.weapon_type) {
                out.skipped = true;
                out.pending.remove(&entity);
                continue;
            }
            let delay = assets.delay(form)?;
            let pending = Pending {
                shot: shot.clone(),
                sequence: actor.last_move_sequence,
                form,
                deadline: motion.attack_deadline(entity, delay, now)?,
            };
            if current(data, actor, &pending)? {
                out.pending.insert(entity, pending);
            } else {
                out.pending.remove(&entity);
                out.skipped = true;
            }
        }
        let mut retired = Vec::new();
        for (&entity, pending) in &mut out.pending {
            active(cancel)?;
            let actor = data
                .actors
                .0
                .iter()
                .find(|a| a.entity == entity)
                .context("pending bow actor absent")?;
            let pose = motion.pose(entity, now)?;
            if !current(data, actor, pending)? || pose.action != BaseAction::Attack {
                retired.push(entity);
                out.skipped = true;
                continue;
            }
            let delay = assets.delay(pending.form)?;
            if pose.elapsed < delay {
                pending.deadline = motion.attack_deadline(entity, delay, now)?;
                continue;
            }
            retired.push(entity);
            if now.saturating_duration_since(pending.deadline) > MAX_AGE {
                out.skipped = true;
                continue;
            }
            let path = match RangedPath::new(cell(&pending.shot.from), target(data, &pending.shot)?)
            {
                Ok(path) => path,
                Err(RangedError::Bound(_)) => {
                    out.skipped = true;
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            if path.sample(Duration::ZERO).finished {
                continue;
            }
            if out.flights.len() >= MAX_FLIGHTS {
                out.skipped = true;
                continue;
            }
            out.flights.push(Flight {
                shot: pending.shot.clone(),
                path,
                started: now,
            });
        }
        for entity in retired {
            out.pending.remove(&entity);
        }
        active(cancel)?;
        out.observation = next;
        out.observed = now;
        Ok(out)
    }
    pub(crate) fn draw(
        &mut self,
        canvas: &mut Canvas,
        projection: ViewProjection,
        assets: &Assets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<()> {
        ensure!(now >= self.observed, "bow draw clock reversed");
        for flight in &self.flights {
            active(cancel)?;
            ensure!(flight.shot.sprite == 66, "unqualified bow flight sprite");
            match assets
                .arrow
                .as_ref()
                .context("bow SPR66 absent")?
                .draw_effect(
                    canvas,
                    &flight.path,
                    projection,
                    now.duration_since(flight.started),
                ) {
                Ok(_) => {}
                Err(RangedError::Bound(_)) => self.skipped = true,
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "battle_bows_tests.rs"]
pub(crate) mod tests;
