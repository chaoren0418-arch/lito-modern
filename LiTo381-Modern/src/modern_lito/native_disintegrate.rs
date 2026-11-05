//! Current TW S23 presentation core. No request permission or local HP/MP writes.
//! The caller supplies the resource-owner identity, not a per-frame token.
use super::{
    compositor::Canvas,
    disintegrate_assets::DisintegrateAssets,
    native_combat::ProjectileCue,
    native_combat_animation::EnteredEvent,
    native_effect_timing::{Clip, Playback},
    native_projectiles::Impact,
    native_world::World,
    sprite::SpriteFrame,
    sprite_actions::{ActionKey, FrameEvent},
    world_map::{ViewProjection, WorldCell},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
pub fn supports_graphics(graphics: u16) -> bool {
    matches!(graphics, 11746 | 11748)
}
type Key = (u32, u64, u64);
fn key(c: &ProjectileCue) -> Key {
    (c.attacker, c.attacker_incarnation, c.action_sequence)
}
/// Capture synchronously when native actor playback emits the marker. Retaining
/// this stamp across a source/world/life change cannot rebind it to a new cast.
pub struct Launch {
    owner: Arc<()>,
    publication: Arc<()>,
    generation: u64,
    key: Key,
}
impl Launch {
    pub fn capture(
        effects: &Effects,
        owner: &Arc<()>,
        world: &World,
        event: &EnteredEvent,
    ) -> Option<Self> {
        if effects.require_owner(owner).is_err()
            || effects.generation != Some(world.generation)
            || world.generation == 0
            || event.action != 18
            || event.event != "Hit"
        {
            return None;
        }
        let actor = world.actors.get(&event.actor).filter(|a| !a.dead)?;
        let cue = world.projectiles.iter().find(|c| {
            c.attacker == event.actor
                && c.attacker_incarnation == actor.incarnation
                && c.action_sequence == event.sequence
                && c.projectile.flight_type == 0
                && supports_graphics(c.projectile.graphics)
        })?;
        Some(Self {
            owner: owner.clone(),
            publication: effects.publication.clone(),
            generation: world.generation,
            key: key(cue),
        })
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Cue {
    pub actor: u32,
    pub sequence: u64,
    pub graphics: u16,
    pub source_frame: usize,
    pub event_index: usize,
    pub sound: u32,
    #[serde(skip)]
    pub prepared_audio: super::native_audio_delivery::PreparedCue,
}
/// Retired models keep admission, never their old pixels or pending callbacks.
#[derive(Clone, Default)]
pub(crate) struct History {
    generation: Option<u64>,
    seen: BTreeSet<Key>,
    // Preserve a failed presentation load across same-world scene replacement.
    unavailable: Option<(PathBuf, String)>,
}
impl History {
    pub(crate) fn merge(&mut self, other: &Self) {
        if self.generation == other.generation {
            self.seen.extend(other.seen.iter().copied());
            if self.unavailable.is_none() {
                self.unavailable = other.unavailable.clone();
            }
        }
    }
}
struct Active {
    cue: ProjectileCue,
    position: WorldCell,
    start: u64,
    playback: Playback,
    frame: usize,
    finished: bool,
}
/// Presentation availability never grants or revokes permission to cast.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum PresentationStatus {
    #[default]
    Pending,
    Ready,
    Unavailable { reason: String },
}

#[derive(Default)]
pub struct Effects {
    publication: Arc<()>,
    owner: Option<Arc<()>>,
    root: Option<PathBuf>,
    generation: Option<u64>,
    assets: Option<Arc<DisintegrateAssets>>,
    presentation: PresentationStatus,
    clips: BTreeMap<u16, Clip>,
    seen: BTreeSet<Key>,
    active: Vec<Active>,
    events: Vec<Cue>,
    last_now: Option<u64>,
    started: u64,
    hits: u64,
    omitted: u64,
    pending_history: Option<History>,
}
pub struct Draw<'a> {
    main: &'a SpriteFrame,
    layer: &'a SpriteFrame,
    position: WorldCell,
    pub graphics: u16,
    pub sequence: u64,
    pub source_frame: usize,
}
impl<'a> Draw<'a> {
    pub fn position(&self) -> WorldCell {
        self.position
    }
    pub fn frames(&self) -> (&SpriteFrame, &SpriteFrame) {
        (self.main, self.layer)
    }
    pub fn submit_to(
        &self,
        target: &mut super::native_world_effects::Target<'_, 'a>,
        view: ViewProjection,
    ) -> Result<()> {
        use super::native_world_effects::Mix;
        target.submit_batch(
            &[(self.main, Mix::Screen), (self.layer, Mix::Multiply)],
            self.position,
            view,
        )
    }
    /// Same qualified Screen/Multiply555 primitives as the shared raster.
    /// Scene interleaving integration is separate from this canvas operation.
    pub fn draw_canvas(&self, canvas: &mut Canvas, view: ViewProjection) -> Result<()> {
        let p = view
            .cell_to_pixel(self.position)
            .context("disintegrate projection")?;
        let main = (
            p.x.checked_add(self.main.x_offset).context("main X")?,
            p.y.checked_add(self.main.y_offset).context("main Y")?,
        );
        let layer = (
            p.x.checked_add(self.layer.x_offset).context("layer X")?,
            p.y.checked_add(self.layer.y_offset).context("layer Y")?,
        );
        canvas.blit_screen_rgb555(
            &self.main.rgba,
            self.main.width,
            self.main.height,
            main.0,
            main.1,
        )?;
        canvas.blit_multiply_rgb555(
            &self.layer.rgba,
            self.layer.width,
            self.layer.height,
            layer.0,
            layer.1,
        )?;
        Ok(())
    }
}
impl Effects {
    pub(crate) fn history(&self) -> History {
        History {
            generation: self.generation,
            seen: self.seen.clone(),
            unavailable: match (&self.root, &self.presentation) {
                (Some(root), PresentationStatus::Unavailable { reason }) => {
                    Some((root.clone(), reason.clone()))
                }
                _ => None,
            },
        }
    }
    pub(crate) fn restore_history(&mut self, history: History) {
        self.pending_history = Some(history);
    }
    fn clear_presentation(&mut self) {
        self.active.clear();
        self.seen.clear();
        self.events.clear();
        self.last_now = None;
        self.started = 0;
        self.hits = 0;
        self.omitted = 0;
    }
    pub fn retire(&mut self) {
        self.publication = Arc::new(());
        self.clear_presentation();
        self.generation = None;
        self.owner = None;
        self.root = None;
        self.assets = None;
        self.presentation = PresentationStatus::Pending;
        self.clips.clear();
        self.pending_history = None;
    }
    pub fn prepare(
        &mut self,
        root: &Path,
        owner: Arc<()>,
        world: &World,
        cancel: &AtomicBool,
    ) -> Result<()> {
        self.prepare_with_loader(root, owner, world, cancel, DisintegrateAssets::load_current_tw)
    }
    fn prepare_with_loader(
        &mut self,
        root: &Path,
        owner: Arc<()>,
        world: &World,
        cancel: &AtomicBool,
        load: impl FnOnce(&Path, &AtomicBool) -> Result<DisintegrateAssets>,
    ) -> Result<()> {
        if cancel.load(Ordering::Acquire) {
            self.retire();
            anyhow::bail!("disintegrate preparation cancelled");
        }
        let root = match root.canonicalize() {
            Ok(root) => root,
            Err(error) => {
                self.retire();
                return Err(error.into());
            }
        };
        let same = self
            .owner
            .as_ref()
            .is_some_and(|old| Arc::ptr_eq(old, &owner));
        if same && self.root.as_ref() != Some(&root) {
            self.retire();
            anyhow::bail!("resource owner reused with another root");
        }
        if !same {
            let restored = self.pending_history.take().unwrap_or_else(|| self.history());
            let retire_existing = self.owner.is_some() && self.generation == Some(world.generation);
            self.retire();
            self.owner = Some(owner);
            self.root = Some(root.clone());
            self.generation = Some(world.generation);
            // An old cast cannot be relaunched merely by swapping A -> B -> A.
            if retire_existing {
                self.seen.extend(world.projectiles.iter().map(key));
            }
            if restored.generation == Some(world.generation) {
                let history = restored;
                ensure!(
                    history.seen.len() <= 4096,
                    "disintegrate admission history budget"
                );
                self.seen.extend(history.seen);
                if let Some((failed_root, reason)) = history.unavailable {
                    if failed_root == root {
                        self.presentation = PresentationStatus::Unavailable { reason };
                    }
                }
            }
        }
        if self.generation != Some(world.generation) {
            self.clear_presentation();
            self.generation = Some(world.generation);
            if matches!(self.presentation, PresentationStatus::Unavailable { .. }) {
                self.presentation = PresentationStatus::Pending;
            }
        }
        if self.assets.is_some()
            || matches!(self.presentation, PresentationStatus::Unavailable { .. })
            || !(world.learned_skills.contains(&77)
                || world
                    .projectiles
                    .iter()
                    .any(|c| supports_graphics(c.projectile.graphics)))
        {
            return Ok(());
        }
        let loaded = load(&root, cancel);
        // Cancellation is lifecycle control, not a resource qualification failure.
        if cancel.load(Ordering::Acquire) {
            self.retire();
            anyhow::bail!("disintegrate preparation cancelled");
        }
        let assets = match loaded {
            Ok(assets) => Arc::new(assets),
            Err(error) => {
                let reason = format!("{error:#}");
                eprintln!(
                    "disintegrate presentation unavailable (generation {}): {reason}",
                    world.generation
                );
                self.presentation = PresentationStatus::Unavailable { reason };
                return Ok(());
            }
        };
        let mut clips = BTreeMap::new();
        for gfx in [11746u16, 11748] {
            let a = assets
                .catalog()
                .catalog()
                .resolve_action(u32::from(gfx), ActionKey::new(0))?
                .animation;
            clips.insert(gfx, Clip::new(Arc::new(a.clone()))?);
        }
        if cancel.load(Ordering::Acquire) {
            self.retire();
            anyhow::bail!("disintegrate preparation cancelled");
        }
        self.assets = Some(assets);
        self.clips = clips;
        self.presentation = PresentationStatus::Ready;
        Ok(())
    }
    fn require_owner(&self, owner: &Arc<()>) -> Result<()> {
        ensure!(
            self.owner
                .as_ref()
                .is_some_and(|old| Arc::ptr_eq(old, owner)),
            "stale disintegrate source owner"
        );
        Ok(())
    }
    pub fn update<'a>(
        &mut self,
        owner: &Arc<()>,
        world: &World,
        events: impl Iterator<Item = &'a Launch>,
        now: u64,
    ) -> Result<Vec<Impact>> {
        self.require_owner(owner)?;
        ensure!(world.projectiles.len() <= 4096, "disintegrate cue budget");
        if self.generation != Some(world.generation) {
            self.clear_presentation();
            self.generation = Some(world.generation);
            if matches!(self.presentation, PresentationStatus::Unavailable { .. }) {
                self.presentation = PresentationStatus::Pending;
            }
        }
        ensure!(
            self.last_now.is_none_or(|last| now >= last),
            "disintegrate clock regressed"
        );
        self.last_now = Some(now);
        self.events.clear();
        self.active.retain(|a| !a.finished);
        if world.generation == 0 {
            self.clear_presentation();
            return Ok(Vec::new());
        }
        let live: BTreeSet<_> = world.projectiles.iter().map(key).collect();
        self.seen.retain(|k| live.contains(k));
        for event in events.filter(|e| {
            e.generation == world.generation
                && Arc::ptr_eq(&e.owner, owner)
                && Arc::ptr_eq(&e.publication, &self.publication)
        }) {
            let Some(cue) = world.projectiles.iter().find(|c| {
                key(c) == event.key
                    && c.projectile.flight_type == 0
                    && supports_graphics(c.projectile.graphics)
            }) else {
                continue;
            };
            if !self.seen.insert(key(cue)) {
                continue;
            }
            if cue.action_sequence == 0
                || cue.attacker_incarnation == 0
                || self.active.len() >= 64
                || world
                    .actors
                    .get(&cue.attacker)
                    .is_none_or(|a| a.dead || a.incarnation != cue.attacker_incarnation)
            {
                self.omitted += 1;
                continue;
            }
            let Some(clip) = self.clips.get(&cue.projectile.graphics) else {
                self.omitted += 1;
                continue;
            };
            self.active.push(Active {
                cue: cue.clone(),
                position: WorldCell {
                    x: i32::from(cue.projectile.to[0]),
                    y: i32::from(cue.projectile.to[1]),
                },
                start: now,
                playback: clip.start(),
                frame: 1,
                finished: false,
            });
            self.started += 1;
        }
        let mut impacts = Vec::new();
        for a in &mut self.active {
            let sample = a
                .playback
                .advance((now / 1_000_000).saturating_sub(a.start / 1_000_000) * 1_000_000)?;
            a.frame = sample.source_frame;
            a.finished = sample.finished;
            for cue in sample.crossed_cues {
                match cue.event {
                    FrameEvent::Sound(717) => self.events.push(Cue {
                        actor: a.cue.attacker,
                        sequence: a.cue.action_sequence,
                        graphics: a.cue.projectile.graphics,
                        source_frame: cue.source_frame,
                        event_index: cue.event_index,
                        sound: 717,
                        prepared_audio: {
                            let assets = self
                                .assets
                                .as_ref()
                                .context("disintegrate sound assets absent")?;
                            super::native_audio_delivery::PreparedCue::from_finite_event(
                                assets.sound_bank(),
                                assets.sound_group().clone(),
                                0,
                                super::native_audio_delivery::FiniteEventKey {
                                    sequence: a.cue.action_sequence,
                                    graphics: a.cue.projectile.graphics,
                                    action: 0,
                                    source_frame: cue.source_frame,
                                },
                            )?
                        },
                    }),
                    FrameEvent::Hit => {
                        self.hits += 1;
                        if a.cue.damage_marker > 0
                            && world.actors.get(&a.cue.target).is_some_and(|t| {
                                !t.dead && Some(t.incarnation) == a.cue.target_incarnation
                            })
                        {
                            impacts.push(Impact {
                                target: a.cue.target,
                                sequence: a.cue.action_sequence,
                                projectile: a.cue.projectile.id,
                            });
                        }
                    }
                    _ => {
                        self.omitted += 1;
                        a.finished = true;
                    }
                }
            }
        }
        Ok(impacts)
    }
    pub fn events(&self) -> &[Cue] {
        &self.events
    }
    pub fn entered_events(&self) -> Vec<EnteredEvent> {
        self.events
            .iter()
            .map(|e| EnteredEvent {
                actor: e.actor,
                sequence: e.sequence,
                action: 0,
                frame: e.source_frame,
                event: "DisintegrateSound(717)".into(),
                sound: Some(e.sound),
                prepared_audio: Some(e.prepared_audio.clone()),
            })
            .collect()
    }
    pub fn draws(&self, owner: &Arc<()>) -> Result<Vec<Draw<'_>>> {
        self.require_owner(owner)?;
        let Some(assets) = &self.assets else {
            return Ok(Vec::new());
        };
        let mut draws = Vec::new();
        for active in self.active.iter().filter(|a| !a.finished) {
            let main = usize::from(active.cue.projectile.graphics == 11748) * 2;
            let layer = main + 1;
            let animation = assets
                .catalog()
                .catalog()
                .resolve_action(u32::from(active.cue.projectile.graphics), ActionKey::new(0))?
                .animation;
            let subframe = animation.frames[active.frame].subframe as usize;
            draws.push(Draw {
                main: &assets.sprites()[main].decoded().frames[subframe],
                layer: &assets.sprites()[layer].decoded().frames[subframe],
                position: active.position,
                graphics: active.cue.projectile.graphics,
                sequence: active.cue.action_sequence,
                source_frame: active.frame,
            });
        }
        Ok(draws)
    }
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"generation":self.generation,"presentation":self.presentation,"assets_ready":self.assets.is_some(),"active":self.active.iter().filter(|a|!a.finished).count(),"seen":self.seen.len(),"started":self.started,"hit_entries":self.hits,"omitted":self.omitted,
        "prepared_sound_events":self.events.len(),"authority_mutated":false})
    }
}

#[cfg(test)]
mod availability_tests {
    use super::*;
    type Assets = DisintegrateAssets;
    const SKILL: u16 = 77;
    const GRAPHICS: u16 = 11746;
    const FEATURE: &str = "disintegrate";
    include!("native_tw_presentation_tests.rs");
}
