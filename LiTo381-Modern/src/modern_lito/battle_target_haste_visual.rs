//! Confirmed skill43 targeted presentation: shared755 and the qualified action19 body.
//! No optimistic buff, legacy Frame, invented sound or repeated historical cast.
use super::{
    battle_follow::Observation,
    battle_payload::{data as d, haste::StatusTarget},
    compositor::Canvas,
    haste_effect::HasteAssets,
    paced_animation::PacedClock,
    shared_action_catalog::SharedActionCatalog,
    teleport_effect::TeleportAssets,
    world_map::{ViewProjection, WorldCell, WorldMap},
    world_scene::{ActorOverride, BaseAppearance, SceneSnapshot},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const MAX_EFFECTS: usize = 128;
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "canonical haste cancelled");
    Ok(())
}
fn age(receipt: i64, observed: i64) -> Option<Duration> {
    observed
        .checked_sub(receipt)
        .filter(|n| (0..=500_000_000).contains(n))
        .map(|n| Duration::from_nanos(n as u64))
}
fn cell(c: &d::Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}
fn receipt(a: &d::Actor) -> Option<&d::SpellReceipt> {
    a.spells
        .histories
        .0
        .iter()
        .find(|s| s.lane == d::Lane::Haste)?
        .receipt
        .as_ref()
}
pub(crate) fn enabled(observation: &Observation) -> bool {
    observation.data().actors.0.iter().any(|a| {
        a.spells
            .histories
            .0
            .iter()
            .any(|s| s.lane == d::Lane::Haste)
    })
}
fn status(r: &d::SpellReceipt) -> Result<&StatusTarget> {
    ensure!(
        r.impact.lane == d::Lane::Haste && r.impact.targets.0.is_empty(),
        "unqualified targeted haste receipt"
    );
    match &r.detail {
        d::Detail::Haste(detail) => Ok(&detail.target),
        _ => anyhow::bail!("targeted haste detail absent"),
    }
}
fn target(r: &d::SpellReceipt) -> Result<WorldCell> {
    Ok(cell(match status(r)? {
        StatusTarget::Player(p) => &p.cell,
        StatusTarget::Npc(n) => &n.cell,
    }))
}
fn target_live(data: &d::UnvalidatedData, target: &StatusTarget) -> bool {
    match target {
        StatusTarget::Player(p) => data.actors.0.iter().any(|a| {
            a.entity == p.participant.entity
                && a.loaded
                && super::battle_space::occupies(data, a.entity)
                && data
                    .scene_member(a)
                    .is_ok_and(|m| m.identity == p.participant.identity)
                && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
                && super::battle_motion::life(a) == Some(p.life)
                && a.resources
                    .as_ref()
                    .is_some_and(|r| r.equipment_revision == p.equipment_revision)
        }),
        StatusTarget::Npc(n) => {
            n.id.world == data.world.identity
                && data.npcs.as_ref().is_some_and(|roster| {
                    roster
                        .actors
                        .0
                        .iter()
                        .any(|a| a.serial == n.id.serial && a.life == n.life && a.hp > 0)
                })
        }
    }
}
pub(crate) fn check_map(observation: &Observation, map: &WorldMap) -> Result<()> {
    for r in observation.data().actors.0.iter().filter_map(receipt) {
        ensure!(
            map.standing(cell(&r.impact.usage.before.cell)) && map.standing(target(r)?),
            "canonical haste outside shared terrain"
        );
    }
    Ok(())
}
pub(crate) struct Assets {
    effect: Option<HasteAssets>,
}
impl Assets {
    pub(crate) fn load(
        catalog: &SharedActionCatalog,
        observation: &Observation,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        Ok(Self {
            effect: enabled(observation)
                .then(|| HasteAssets::load_with_catalog(catalog, cancel))
                .transpose()?,
        })
    }
    pub(crate) fn compatible(&self, old: &Self) -> bool {
        self.effect.is_some() == old.effect.is_some()
    }
}
#[derive(Clone)]
struct Cast {
    receipt: d::SpellReceipt,
    sequence: i64,
    body: PacedClock,
    heading: u8,
}
#[derive(Clone)]
struct Effect {
    target: StatusTarget,
    position: WorldCell,
    started: Instant,
    duration: Duration,
}
#[derive(Clone)]
pub(crate) struct Timeline {
    observation: Observation,
    observed: Instant,
    casts: BTreeMap<i64, Cast>,
    effects: Vec<Effect>,
    pub(crate) skipped: bool,
}
fn current(data: &d::UnvalidatedData, actor: &d::Actor, cast: &Cast) -> Result<bool> {
    let usage = &cast.receipt.impact.usage;
    Ok(actor.vital.as_ref().is_some_and(|v| v.values.hp > 0)
        && super::battle_motion::life(actor) == Some(usage.before.life)
        && actor
            .resources
            .as_ref()
            .is_some_and(|r| r.equipment_revision == usage.before.equipment_revision)
        && actor.last_move_sequence == cast.sequence
        && actor.position == usage.before.cell
        && super::battle_motion::latest_body_event(data, actor, data.scene_member(actor)?)
            <= usage.revision)
}
impl Timeline {
    pub(crate) fn new(observation: Observation, now: Instant) -> Self {
        Self {
            observation,
            observed: now,
            casts: BTreeMap::new(),
            effects: vec![],
            skipped: false,
        }
    }
    pub(crate) fn advance(
        &self,
        next: Observation,
        now: Instant,
        assets: &Assets,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(now >= self.observed, "canonical haste clock reversed");
        self.observation.check_successor(&next)?;
        ensure!(
            next.data().world.phase != d::Phase::Aborted,
            "aborted haste must retire"
        );
        let mut out = self.clone();
        out.effects
            .retain(|e| now.duration_since(e.started) < e.duration);
        let data = next.data();
        if data.world.phase != d::Phase::Active {
            out.casts.clear();
            out.effects.clear();
            out.observation = next;
            out.observed = now;
            return Ok(out);
        }
        out.effects.retain(|e| target_live(data, &e.target));
        for a in &data.actors.0 {
            active(cancel)?;
            if let Some(r) = receipt(a)
                .filter(|r| r.impact.usage.revision > self.observation.data().world.revision)
            {
                if !target_live(data, status(r)?) {
                    continue;
                }
                let old = self
                    .observation
                    .data()
                    .actors
                    .0
                    .iter()
                    .find(|old| old.entity == a.entity)
                    .context("haste actor absent")?;
                let count = receipt(old).map_or(0, |r| r.casts);
                out.skipped |= r.casts.checked_sub(count) != Some(1);
                if let Some(age) = age(r.loading_elapsed_nanos, data.world.loading_elapsed_nanos) {
                    let bank = assets.effect.as_ref().context("haste bank absent")?;
                    let position = target(r)?;
                    // The receipt has no move-sequence anchor. Do not fabricate
                    // body order if a coalesced move may follow this cast.
                    if a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
                        && super::battle_motion::life(a) == Some(r.impact.usage.before.life)
                        && a.resources.as_ref().is_some_and(|v| {
                            v.equipment_revision == r.impact.usage.before.equipment_revision
                        })
                        && (old.last_move_sequence == a.last_move_sequence
                            || r.impact.usage.revision == data.world.revision)
                    {
                        out.casts.insert(
                            a.entity,
                            Cast {
                                receipt: r.clone(),
                                sequence: a.last_move_sequence,
                                body: PacedClock::new(now),
                                heading: a.heading.try_into()?,
                            },
                        );
                    } else {
                        out.skipped = true;
                    }
                    if out.effects.len() < MAX_EFFECTS {
                        let started = now.checked_sub(age).context("haste timestamp underflow")?;
                        out.effects.push(Effect {
                            target: status(r)?.clone(),
                            position,
                            started,
                            duration: bank.duration(),
                        });
                    } else {
                        out.skipped = true;
                    }
                } else {
                    out.skipped = true;
                }
            }
            if let Some(cast) = out.casts.get_mut(&a.entity) {
                if !current(data, a, cast)? || cast.body.peek(now)? >= Duration::from_secs(10) {
                    out.casts.remove(&a.entity);
                } else {
                    let base = a.cast_interval_nanos.try_into()?;
                    cast.body.retime(
                        now,
                        base,
                        super::battle_motion::paced_nanos(a, base, false),
                    )?;
                }
            }
        }
        active(cancel)?;
        out.observation = next;
        out.observed = now;
        Ok(out)
    }
    pub(crate) fn overrides<'a>(
        &self,
        scene: &SceneSnapshot,
        bank: &'a TeleportAssets,
        now: Instant,
    ) -> Result<Vec<ActorOverride<'a>>> {
        ensure!(now >= self.observed, "haste body sample reversed");
        let mut result = Vec::new();
        let data = self.observation.data();
        for a in &data.actors.0 {
            let Some(cast) = self.casts.get(&a.entity) else {
                continue;
            };
            if scene.hidden(a.entity as u64) || !current(data, a, cast)? {
                continue;
            }
            let member = data.scene_member(a)?;
            let form = BaseAppearance {
                class: member.class.try_into()?,
                sex: member.sex.try_into()?,
            };
            let elapsed = cast.body.peek(now)?;
            if elapsed >= bank.cast_duration(form)? {
                continue;
            }
            let v = bank.caster_frame(form, cast.heading, elapsed)?;
            if !v.finished {
                result.push(ActorOverride {
                    entity_id: a.entity.try_into()?,
                    frame: v.frame,
                    graphics_id: v.graphics_id,
                    source_frame: v.source_frame,
                    sprite_name: v.sprite_name,
                    subframe: v.subframe,
                });
            }
        }
        Ok(result)
    }
    pub(crate) fn draw(
        &self,
        canvas: &mut Canvas,
        projection: ViewProjection,
        assets: &Assets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<()> {
        ensure!(now >= self.observed, "haste sample reversed");
        for e in &self.effects {
            active(cancel)?;
            assets
                .effect
                .as_ref()
                .context("haste bank absent")?
                .draw_effect(
                    canvas,
                    projection
                        .cell_to_pixel(e.position)
                        .context("haste projection overflow")?,
                    now.duration_since(e.started),
                )?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "battle_target_haste_visual_tests.rs"]
pub(crate) mod tests;
