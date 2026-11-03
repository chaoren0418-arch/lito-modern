//! Worker-owned, preloaded Quest objectives. The private painter key never
//! leaves the renderer as a player/NPC identity. Destruction is server-owned;
//! retained ruins have no click or occupancy authority.
use super::{
    quest_protocol::{self as wire, Id, Target},
    shared_action_catalog::SharedActionCatalog,
    tower_assets::{HonorPlayback, TowerAssets},
    world_audio::AudioSequencer,
    world_map::WorldMap,
    world_protocol::Frame,
    world_scene::{ObjectiveDraw, ObjectivePlacement},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub id: Id,
    pub alive: bool,
    pub geometry: ObjectivePlacement,
}
/// Private sorting key, disjoint from the bounded public participant IDs.
pub(crate) fn draw_id(slot: u8) -> u64 {
    100 + u64::from(slot)
}
pub(crate) fn valid_placement(world: &Frame, p: &Placement) -> bool {
    wire::target(world, p.id).is_some_and(|(t, a)| {
        p.geometry.entity_id == draw_id(t.slot)
            && p.geometry.cell == t.cell
            && p.geometry.graphics_id == t.graphic
            && p.alive == (a.hp > 0)
    })
}
struct Visual {
    action: u8,
    started: Option<Instant>,
    playback: HonorPlayback,
}
impl Visual {
    fn elapsed(&self, now: Instant) -> Duration {
        self.started
            .map_or(Duration::MAX, |start| now.saturating_duration_since(start))
    }
}
pub(crate) struct Presentation {
    assets: TowerAssets,
    scope: super::npc_protocol::Scope,
    targets: Vec<Target>,
    visuals: BTreeMap<u8, Visual>,
}
impl Presentation {
    pub(crate) fn load(
        catalog: &SharedActionCatalog,
        frame: &Frame,
        map: &WorldMap,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        wire::validate(frame)?;
        let state = frame.quest.as_ref().context("Quest preflight absent")?;
        // Validate ALL future stage cells and graphics before permitting LOADED.
        for t in &state.targets {
            ensure!(
                map.standing(t.cell),
                "Quest objective outside standable shared geometry"
            );
        }
        let assets = TowerAssets::load_with_catalog(catalog, cancel)?;
        for t in &state.targets {
            assets.state_frame(t.graphic, 0, Duration::ZERO)?;
        }
        let mut presentation = Self {
            assets,
            scope: wire::scope(frame),
            targets: state.targets.clone(),
            visuals: BTreeMap::new(),
        };
        presentation.update(frame, Instant::now(), true)?;
        Ok(presentation)
    }
    pub(crate) fn update(&mut self, frame: &Frame, now: Instant, baseline: bool) -> Result<()> {
        let state = frame.quest.as_ref().context("Quest presentation absent")?;
        ensure!(
            wire::scope(frame) == self.scope && state.targets == self.targets,
            "Quest preflight contract changed"
        );
        self.visuals.retain(|slot, _| state.actor(*slot).is_some());
        for actor in &state.actors {
            let target = state.target(actor.slot).context("Quest target absent")?;
            let action = actor.action(target);
            if self
                .visuals
                .get(&actor.slot)
                .is_none_or(|v| v.action != action)
            {
                self.visuals.insert(
                    actor.slot,
                    Visual {
                        action,
                        started: (!baseline).then_some(now),
                        playback: self.assets.start_state(target.graphic, action)?,
                    },
                );
            }
        }
        Ok(())
    }
    pub(crate) fn draws<'a>(
        &'a self,
        frame: &Frame,
        now: Instant,
    ) -> Result<Vec<ObjectiveDraw<'a>>> {
        let state = frame.quest.as_ref().context("Quest state absent")?;
        state
            .actors
            .iter()
            .map(|actor| {
                let target = state.target(actor.slot).context("Quest target absent")?;
                let visual = self
                    .visuals
                    .get(&actor.slot)
                    .context("Quest visual absent")?;
                let elapsed = visual.elapsed(now);
                let body = self
                    .assets
                    .state_frame(target.graphic, visual.action, elapsed)?;
                let shadow = self
                    .assets
                    .shadow_frame(target.graphic, visual.action, elapsed)?;
                Ok(ObjectiveDraw {
                    entity_id: draw_id(actor.slot),
                    blocking: actor.hp > 0,
                    cell: target.cell,
                    frame: body.frame,
                    shadow: Some(shadow.frame),
                    graphics_id: body.graphics_id,
                    source_frame: body.source_frame,
                    sprite_name: body.sprite_name,
                    subframe: body.subframe,
                })
            })
            .collect()
    }
    pub(crate) fn placements(
        &self,
        frame: &Frame,
        geometry: Vec<ObjectivePlacement>,
    ) -> Result<Vec<Placement>> {
        let state = frame.quest.as_ref().context("Quest state absent")?;
        ensure!(
            geometry.len() == state.actors.len(),
            "Quest paint count mismatch"
        );
        geometry
            .into_iter()
            .map(|geometry| {
                let actor = state
                    .actors
                    .iter()
                    .find(|a| draw_id(a.slot) == geometry.entity_id)
                    .context("Quest paint identity mismatch")?;
                let p = Placement {
                    id: Id {
                        scope: self.scope,
                        slot: actor.slot,
                    },
                    alive: actor.hp > 0,
                    geometry,
                };
                ensure!(valid_placement(frame, &p), "Quest paint geometry mismatch");
                Ok(p)
            })
            .collect()
    }
    pub(crate) fn audio(&mut self, sequencer: &mut AudioSequencer, now: Instant) -> Result<()> {
        for visual in self.visuals.values_mut() {
            if let Some(started) = visual.started {
                let elapsed = now.saturating_duration_since(started);
                let sample = visual.playback.advance(elapsed)?;
                if elapsed <= Duration::from_millis(500) && !sample.crossed_cues.is_empty() {
                    sequencer.push_preloaded_cues(
                        &sample.crossed_cues,
                        self.assets.sound_bank(),
                        started,
                    )?;
                }
            }
        }
        Ok(())
    }
}
