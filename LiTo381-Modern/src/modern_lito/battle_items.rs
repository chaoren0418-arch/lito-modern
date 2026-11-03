//! Confirmed schema-2 potion overlays. Public event anchors, never private
//! inventory guesses; no healing, buff timers or command authority in pixels.
use super::{
    battle_follow::Observation,
    battle_payload::data as d,
    compositor::Canvas,
    healing_effect::HealingAssets,
    shared_action_catalog::SharedActionCatalog,
    timed_effect::TimedAssets,
    world_audio::{AudioFrame, EventLog},
    world_map::{ViewProjection, WorldCell, WorldMap},
    world_protocol::MatchDescriptor,
};
use anyhow::{ensure, Context, Result};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const MAX_EFFECTS: usize = 128;
const MAX_AGE: Duration = Duration::from_millis(500);
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "canonical items cancelled");
    Ok(())
}
fn cell(c: &d::Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}
fn age(receipt: i64, observed: i64) -> Option<Duration> {
    observed
        .checked_sub(receipt)
        .filter(|n| (0..=500_000_000).contains(n))
        .map(|n| Duration::from_nanos(n as u64))
}
fn events(data: &d::UnvalidatedData) -> impl Iterator<Item = (i64, &d::PublicItem)> {
    data.presentation
        .iter()
        .flat_map(|p| &p.0)
        .filter_map(|p| p.item.as_ref().map(|i| (p.entity, i)))
}
pub(crate) fn check_map(observation: &Observation, map: &WorldMap) -> Result<()> {
    for (_, item) in events(observation.data()) {
        ensure!(
            map.standing(cell(&item.cell)),
            "canonical item origin outside shared terrain"
        );
    }
    Ok(())
}
pub(crate) struct Assets {
    healing: Option<HealingAssets>,
    timed: Option<TimedAssets>,
}
impl Assets {
    pub(crate) fn load(
        catalog: &SharedActionCatalog,
        observation: &Observation,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let data = observation.data();
        let enabled = |feature| {
            data.presentation.is_some() && data.world.descriptor.features as u32 & feature != 0
        };
        Ok(Self {
            healing: enabled(MatchDescriptor::ITEMS)
                .then(|| HealingAssets::load_with_catalog(catalog, cancel))
                .transpose()?,
            timed: enabled(MatchDescriptor::TIMED)
                .then(|| TimedAssets::load_with_catalog(catalog, cancel))
                .transpose()?,
        })
    }
    pub(crate) fn compatible(&self, old: &Self) -> bool {
        self.healing.is_some() == old.healing.is_some()
            && self.timed.is_some() == old.timed.is_some()
    }
    fn duration(&self, effect: i32) -> Result<Duration> {
        if super::healing_effect::EFFECTS.contains(&effect) {
            Ok(self
                .healing
                .as_ref()
                .context("potion bank absent")?
                .duration(effect)?)
        } else {
            Ok(self
                .timed
                .as_ref()
                .context("timed item bank absent")?
                .duration(effect)?)
        }
    }
}
#[derive(Clone)]
struct Effect {
    receipt: d::PublicItem,
    started: Instant,
    duration: Duration,
}
#[derive(Clone)]
pub(crate) struct Timeline {
    observation: Observation,
    observed: Instant,
    effects: Vec<Effect>,
    audio: EventLog,
    pub(crate) skipped: bool,
}
impl Timeline {
    pub(crate) fn new(observation: Observation, now: Instant) -> Self {
        // A first attachment is a baseline, not permission to replay history.
        Self {
            observation,
            observed: now,
            effects: Vec::new(),
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
        ensure!(now >= self.observed, "canonical item clock reversed");
        self.observation.check_successor(&next)?;
        ensure!(
            next.data().world.phase != d::Phase::Aborted,
            "aborted item effects must retire"
        );
        let mut out = self.clone();
        out.effects
            .retain(|e| now.duration_since(e.started) < e.duration);
        for (entity, receipt) in events(next.data()) {
            active(cancel)?;
            if receipt.revision <= self.observation.data().world.revision {
                continue;
            }
            let old_count = events(self.observation.data())
                .find(|(id, _)| *id == entity)
                .map_or(0, |(_, r)| r.count);
            // Latest-receipt protocol cannot reconstruct coalesced intermediate uses.
            out.skipped |= receipt.count.checked_sub(old_count) != Some(1);
            let Some(age) = age(
                receipt.loading_elapsed_nanos,
                next.data().world.loading_elapsed_nanos,
            ) else {
                out.skipped = true;
                continue;
            };
            let started = now.checked_sub(age).context("item timestamp underflow")?;
            let duration = assets.duration(receipt.effect)?;
            if out.effects.len() >= MAX_EFFECTS {
                out.skipped = true;
                continue;
            }
            if let Some(bank) = &assets.timed {
                if super::timed_effect::EFFECTS.contains(&receipt.effect) {
                    let cues = bank
                        .start_effect(receipt.effect)?
                        .advance(age)?
                        .crossed_cues;
                    for (tick, group) in super::world_audio::crossed_audio_groups(&cues)? {
                        ensure!(tick == 0 && age <= MAX_AGE, "unqualified item cue time");
                        out.audio
                            .push_with_bank(group, started, Some(bank.sound_bank()))?;
                    }
                }
            }
            out.effects.push(Effect {
                receipt: receipt.clone(),
                started,
                duration,
            });
        }
        active(cancel)?;
        out.observation = next;
        out.observed = now;
        Ok(out)
    }
    pub(crate) fn draw(
        &self,
        canvas: &mut Canvas,
        projection: ViewProjection,
        assets: &Assets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<()> {
        ensure!(now >= self.observed, "item sample reversed");
        for effect in &self.effects {
            active(cancel)?;
            let anchor = projection
                .cell_to_pixel(cell(&effect.receipt.cell))
                .context("item projection overflow")?;
            let elapsed = now.duration_since(effect.started);
            if super::healing_effect::EFFECTS.contains(&effect.receipt.effect) {
                assets
                    .healing
                    .as_ref()
                    .context("potion bank absent")?
                    .draw_effect(canvas, effect.receipt.effect, anchor, elapsed)?;
            } else {
                assets
                    .timed
                    .as_ref()
                    .context("timed item bank absent")?
                    .draw_effect(canvas, effect.receipt.effect, anchor, elapsed)?;
            }
        }
        Ok(())
    }
    pub(crate) fn audio(&mut self, assets: &Assets, now: Instant) -> Option<AudioFrame> {
        assets.timed.as_ref().map(|bank| AudioFrame {
            bank: bank.sound_bank(),
            events: self.audio.snapshot(now),
        })
    }
}

#[cfg(test)]
#[path = "battle_items_tests.rs"]
pub(crate) mod tests;
