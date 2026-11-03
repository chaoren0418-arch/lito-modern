//! Canonical skill1 / participant and Honor healing. Explicit SharedClassicBasicHealV1 (744),
//! not a missing11722 fallback. Confirmed fixed target effect starts immediately;
//! action19 is presentation only, never a delayed heal or command/LOAD authority.
use super::{
    battle_follow::Observation,
    battle_payload::data as d,
    compositor::Canvas,
    honor_healing_effect::HonorHealingAssets,
    paced_animation::PacedClock,
    shared_action_catalog::SharedActionCatalog,
    teleport_effect::TeleportAssets,
    world_audio::{AudioFrame, EventLog},
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
    ensure!(
        !cancel.load(Ordering::Acquire),
        "canonical healing cancelled"
    );
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
        .find(|s| s.lane == d::Lane::Healing)?
        .receipt
        .as_ref()
}
pub(crate) fn enabled(observation: &Observation) -> bool {
    observation.data().healing_actors().is_some()
        || observation.data().actors.0.iter().any(|a| {
            a.spells
                .histories
                .0
                .iter()
                .any(|s| s.lane == d::Lane::Healing)
        })
}
/// Presentation-only projection from two distinct, already-validated receipt
/// domains. Never turn a participant receipt into a legacy wire/history lane.
struct VisualReceipt {
    player: bool,
    usage: d::Cast,
    casts: i64,
    elapsed: i64,
    position: WorldCell,
}
fn visuals(data: &d::UnvalidatedData, entity: i64) -> Result<Vec<VisualReceipt>> {
    let actor = data
        .actors
        .0
        .iter()
        .find(|a| a.entity == entity)
        .context("healing actor absent")?;
    let mut result = Vec::with_capacity(2);
    if let Some(r) = receipt(actor) {
        result.push(VisualReceipt {
            player: false,
            usage: r.impact.usage.clone(),
            casts: r.casts,
            elapsed: r.loading_elapsed_nanos,
            position: target(r)?,
        });
    }
    if let Some(r) = data
        .healing_actors()
        .and_then(|h| h.0.iter().find(|a| a.entity == entity))
        .and_then(|a| a.outgoing.as_ref())
        .and_then(|o| o.receipt.as_ref())
    {
        ensure!(
            r.impact.caster.entity == entity
                && r.impact.target.kind == d::EffectKind::Heal
                && matches!(r.impact.target.target, d::Target::Participant { .. }),
            "unsupported participant healing presentation"
        );
        result.push(VisualReceipt {
            player: true,
            usage: r.impact.usage.clone(),
            casts: r.casts,
            elapsed: r.loading_elapsed_nanos,
            position: cell(&r.impact.target.cell),
        });
    }
    result.sort_by_key(|r| r.usage.revision);
    Ok(result)
}
fn target(r: &d::SpellReceipt) -> Result<WorldCell> {
    ensure!(
        r.impact.lane == d::Lane::Healing
            && r.detail == d::Detail::Ordinary
            && r.impact.targets.0.len() == 1,
        "unqualified healing receipt"
    );
    let target = &r.impact.targets.0[0];
    ensure!(
        target.kind == d::EffectKind::Heal && matches!(target.target, d::Target::Honor { .. }),
        "unsupported healing target domain"
    );
    Ok(cell(&target.cell))
}
pub(crate) fn check_map(observation: &Observation, map: &WorldMap) -> Result<()> {
    for a in &observation.data().actors.0 {
        for r in visuals(observation.data(), a.entity)? {
            ensure!(
                map.standing(cell(&r.usage.before.cell)) && map.standing(r.position),
                "canonical healing outside shared terrain"
            );
        }
    }
    Ok(())
}
pub(crate) struct Assets {
    effect: Option<HonorHealingAssets>,
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
                .then(|| HonorHealingAssets::load_with_catalog(catalog, cancel))
                .transpose()?,
        })
    }
    pub(crate) fn compatible(&self, old: &Self) -> bool {
        self.effect.is_some() == old.effect.is_some()
    }
}
#[derive(Clone)]
struct Cast {
    usage: d::Cast,
    sequence: i64,
    body: PacedClock,
    heading: u8,
}
#[derive(Clone)]
struct Effect {
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
    audio: EventLog,
    pub(crate) skipped: bool,
}
fn current(data: &d::UnvalidatedData, actor: &d::Actor, cast: &Cast) -> Result<bool> {
    let usage = &cast.usage;
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
            audio: EventLog::default(),
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
        ensure!(now >= self.observed, "canonical healing clock reversed");
        self.observation.check_successor(&next)?;
        ensure!(
            next.data().world.phase != d::Phase::Aborted,
            "aborted healing must retire"
        );
        let mut out = self.clone();
        out.effects
            .retain(|e| now.duration_since(e.started) < e.duration);
        let data = next.data();
        for a in &data.actors.0 {
            active(cancel)?;
            for r in visuals(data, a.entity)?
                .into_iter()
                .filter(|r| r.usage.revision > self.observation.data().world.revision)
            {
                let old = self
                    .observation
                    .data()
                    .actors
                    .0
                    .iter()
                    .find(|old| old.entity == a.entity)
                    .context("healing actor absent")?;
                let count = visuals(self.observation.data(), a.entity)?
                    .iter()
                    .find(|previous| previous.player == r.player)
                    .map_or(0, |r| r.casts);
                out.skipped |= r.casts.checked_sub(count) != Some(1);
                if let Some(age) = age(r.elapsed, data.world.loading_elapsed_nanos) {
                    let bank = assets.effect.as_ref().context("healing bank absent")?;
                    let position = r.position;
                    // The receipt has no move-sequence anchor. Do not fabricate
                    // body order if a coalesced move may follow this cast.
                    if old.last_move_sequence == a.last_move_sequence
                        || r.usage.revision == data.world.revision
                    {
                        out.casts.insert(
                            a.entity,
                            Cast {
                                usage: r.usage.clone(),
                                sequence: a.last_move_sequence,
                                body: PacedClock::new(now),
                                heading: a.heading.try_into()?,
                            },
                        );
                    } else {
                        out.skipped = true;
                    }
                    if out.effects.len() < MAX_EFFECTS {
                        let started = now
                            .checked_sub(age)
                            .context("healing timestamp underflow")?;
                        let sample = bank.start_effect().advance(age)?;
                        for (tick, group) in
                            super::world_audio::crossed_audio_groups(&sample.crossed_cues)?
                        {
                            ensure!(
                                tick == 0 && group.candidates() == [109],
                                "unqualified healing cue"
                            );
                            out.audio
                                .push_with_bank(group, started, Some(bank.sound_bank()))?;
                        }
                        out.effects.push(Effect {
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
        ensure!(now >= self.observed, "healing body sample reversed");
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
        ensure!(now >= self.observed, "healing sample reversed");
        for e in &self.effects {
            active(cancel)?;
            assets
                .effect
                .as_ref()
                .context("healing bank absent")?
                .draw_effect(
                    canvas,
                    projection
                        .cell_to_pixel(e.position)
                        .context("healing projection overflow")?,
                    now.duration_since(e.started),
                )?;
        }
        Ok(())
    }
    pub(crate) fn audio(&mut self, assets: &Assets, now: Instant) -> Option<AudioFrame> {
        assets.effect.as_ref().map(|bank| AudioFrame {
            bank: bank.sound_bank(),
            events: self.audio.snapshot(now),
        })
    }
}

#[cfg(test)]
#[path = "battle_healing_tests.rs"]
pub(crate) mod tests;
