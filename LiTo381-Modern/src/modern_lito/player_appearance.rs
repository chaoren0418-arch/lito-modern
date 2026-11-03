//! Source-bound public player appearances and transactional shared pixel sets.
//! This prepares player bodies, not a complete VIP scene/LOAD certificate.
//! Inventory/icons/effects and complete world admission remain separate requirements.
use super::{
    battle_follow::Observation,
    battle_payload::data as d,
    combat_audio::{BankLimits, SoundBank},
    compositor::Canvas,
    participant::MemberKind,
    polymorph_assets::{PolymorphAssets, PolymorphForm, PolymorphFrame},
    shared_action_catalog::SharedActionCatalog,
    world_map::{WorldCell, WorldMap},
    world_scene::{
        ActorState, BaseAction, BaseAppearance, CombatAppearance, PlayerWalkPlacement, RenderInfo,
        SceneLimits, SceneSnapshot, WorldSceneAssets,
    },
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

#[path = "player_utility_spells.rs"]
mod utility;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Form {
    Base(CombatAppearance),
    Polymorph {
        form: PolymorphForm,
        weapon_type: u8,
    },
}
#[derive(Clone)]
pub struct Plan {
    observation: Observation,
    forms: BTreeMap<u64, Form>,
    snapshot: SceneSnapshot,
}
impl Plan {
    /// Only a followed schema11 observation can supply a nonzero override.
    /// It is never inferred from the owner's private inventory or another
    /// source's same-numbered character. Zero retains the class/sex/weapon.
    pub fn from_observation(observation: Observation) -> Result<Self> {
        let data = observation.data();
        ensure!(
            data.world.phase != d::Phase::Aborted && data.race.is_none(),
            "retired/pet appearance plan"
        );
        let appearances = if let Some(vip) = &data.vip {
            Some(
                &vip.appearances
                    .as_ref()
                    .context("VIP appearance section required")?
                    .0,
            )
        } else {
            None
        };
        let mut forms = BTreeMap::new();
        let mut actors = Vec::with_capacity(data.actors.0.len());
        for (index, a) in data.actors.0.iter().enumerate() {
            let member = data.scene_member(a)?;
            let base = BaseAppearance {
                class: member.class.try_into()?,
                sex: member.sex.try_into()?,
            };
            let weapon_type = a.vital.as_ref().map_or(0, |v| v.weapon_type).try_into()?;
            let entity = a.entity.try_into()?;
            let graphics = appearances.map_or(0, |v| v[index].graphics);
            let form = match PolymorphForm::from_wire(graphics)? {
                Some(form) => Form::Polymorph { form, weapon_type },
                None => Form::Base(CombatAppearance {
                    appearance: base,
                    weapon_type,
                }),
            };
            ensure!(
                forms.insert(entity, form).is_none(),
                "duplicate player appearance"
            );
            let dead = a.resources.as_ref().is_some_and(|r| r.vitals.hp == 0)
                || a.vital.as_ref().is_some_and(|v| v.values.hp == 0);
            actors.push(ActorState {
                kind: match member.identity.kind {
                    d::MemberKind::Human => MemberKind::Human,
                    d::MemberKind::ServerBot => MemberKind::ServerBot,
                },
                entity_id: entity,
                source_id: member.identity.source.try_into()?,
                object_id: member.identity.id.try_into()?,
                appearance: base,
                weapon_type,
                cell: WorldCell {
                    x: a.position.x,
                    y: a.position.y,
                },
                heading: a.heading.try_into()?,
                action: if dead {
                    BaseAction::Death
                } else {
                    BaseAction::Idle
                },
                action_elapsed: if dead { Duration::MAX } else { Duration::ZERO },
            });
        }
        let snapshot = SceneSnapshot::new(
            data.world.identity.match_id.try_into()?,
            data.world.revision.try_into()?,
            data.manifest.digest,
            data.personal.entity.try_into()?,
            actors,
        )?;
        Ok(Self {
            observation,
            forms,
            snapshot,
        })
    }
    pub fn observation(&self) -> &Observation {
        &self.observation
    }
    pub fn snapshot(&self) -> &SceneSnapshot {
        &self.snapshot
    }
    pub fn forms(&self) -> &BTreeMap<u64, Form> {
        &self.forms
    }
    pub fn check_successor(&self, next: &Self) -> Result<()> {
        Ok(self.observation.check_successor(&next.observation)?)
    }
    fn matches(&self, snapshot: &SceneSnapshot) -> bool {
        let original = &self.snapshot;
        snapshot.world_id() == original.world_id()
            && snapshot.revision() == original.revision()
            && snapshot.map_digest() == original.map_digest()
            && snapshot.selected_entity_id() == original.selected_entity_id()
            && snapshot.actors().len() == original.actors().len()
            && snapshot.pet().is_none()
            && snapshot
                .actors()
                .iter()
                .zip(original.actors())
                .all(|(a, b)| {
                    a.kind == b.kind
                        && a.entity_id == b.entity_id
                        && a.source_id == b.source_id
                        && a.object_id == b.object_id
                        && a.appearance == b.appearance
                        && a.weapon_type == b.weapon_type
                        && a.cell == b.cell
                        && !(b.action == BaseAction::Death && a.action != BaseAction::Death)
                })
    }
    fn bases(&self) -> BTreeSet<CombatAppearance> {
        self.forms
            .values()
            .filter_map(|f| match f {
                Form::Base(b) => Some(*b),
                _ => None,
            })
            .collect()
    }
}

/// Follows confirmed movement and body events; never predicts a MOVE or
/// invents the missing portion of a skipped route. Resource publication is
/// separate: callers may render a candidate before committing it and its assets.
#[derive(Clone)]
pub struct Motion {
    plan: Plan,
    timeline: super::battle_motion::Timeline,
}
impl Motion {
    pub fn new(plan: Plan, now: Instant) -> Result<Self> {
        let timeline = super::battle_motion::Timeline::new(plan.observation.clone(), now)?;
        Ok(Self { plan, timeline })
    }
    pub fn plan(&self) -> &Plan {
        &self.plan
    }
    pub fn advance(&self, next: Plan, now: Instant, cancel: &AtomicBool) -> Result<Self> {
        let timeline = self
            .timeline
            .advance(next.observation.clone(), now, cancel)?;
        Ok(Self {
            plan: next,
            timeline,
        })
    }
    /// Body time is the canonical paced accumulator, not a raw SPR clock.
    /// render_motion converts confirmed walk phases using the exact native
    /// integer-duration certificate (including source0 camera pulse).
    pub fn scene(&self, now: Instant) -> Result<SceneSnapshot> {
        let mut actors = self.plan.snapshot.actors().to_vec();
        for a in &mut actors {
            let pose = self.timeline.pose(a.entity_id.try_into()?, now)?;
            a.action = pose.action;
            a.action_elapsed = pose.elapsed;
            if let Some(heading) = pose.heading {
                a.heading = heading;
            }
        }
        self.plan.snapshot.with_actors(actors)
    }
}

/// Loading a replacement returns a new owned set. Failure never mutates the
/// old plan, pixels or published canvas; the caller commits only on success.
pub struct Assets {
    world: WorldSceneAssets,
    forms: BTreeSet<Form>,
    polymorphs: BTreeMap<PolymorphForm, PolymorphAssets>,
    body_audio: Arc<SoundBank>,
    spells: super::battle_spells::Assets,
    utility: utility::Assets,
}
impl Assets {
    pub fn limitations(&self) -> &super::world_scene::SceneLimitations {
        self.world.limitations()
    }
    pub fn load_with_catalog(
        context: &SharedActionCatalog,
        map: Arc<WorldMap>,
        plan: &Plan,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        ensure!(
            !cancel.load(Ordering::Acquire),
            "player appearance load cancelled"
        );
        ensure!(
            map.canonical_digest() == plan.snapshot.map_digest(),
            "appearance geometry mismatch"
        );
        let forms: BTreeSet<_> = plan.forms.values().copied().collect();
        let mut polymorphs = BTreeMap::new();
        for f in &forms {
            if let Form::Polymorph { form, weapon_type } = f {
                form.qualify_weapon(context.catalog(), *weapon_type)?;
                if !polymorphs.contains_key(form) {
                    polymorphs.insert(
                        *form,
                        PolymorphAssets::load_with_catalog(context, *form, cancel)?,
                    );
                }
            }
        }
        let spell_forms: Vec<_> = plan
            .snapshot
            .actors()
            .iter()
            .map(|a| a.appearance)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        super::battle_spells::check_map(&plan.observation, &map)?;
        utility::check_map(&plan.observation, &map)?;
        let utility = utility::Assets::load(context, plan, cancel)?;
        let spells = super::battle_spells::Assets::load(
            context.root(),
            context,
            &plan.observation,
            &spell_forms,
            cancel,
        )?;
        // Reserve the independently bounded bank from the ordinary scene's
        // total cache allowance. Current reviewed set contains at most one bank.
        let mut limits = SceneLimits::default();
        let (utility_encoded, utility_rgba) = utility.cache_bytes();
        limits.max_encoded_bytes = limits
            .max_encoded_bytes
            .checked_sub(utility_encoded)
            .context("utility encoded budget")?;
        limits.max_texture_bytes = limits
            .max_texture_bytes
            .checked_sub(utility_rgba)
            .context("utility pixel budget")?;
        let (cast_encoded, cast_rgba) = spells.polymorph_cache_bytes();
        limits.max_encoded_bytes = limits
            .max_encoded_bytes
            .checked_sub(cast_encoded)
            .context("cast encoded budget")?;
        limits.max_texture_bytes = limits
            .max_texture_bytes
            .checked_sub(cast_rgba)
            .context("cast pixel budget")?;
        for bank in polymorphs.values() {
            limits.max_encoded_bytes = limits
                .max_encoded_bytes
                .checked_sub(bank.encoded_bytes())
                .context("appearance encoded budget")?;
            limits.max_texture_bytes = limits
                .max_texture_bytes
                .checked_sub(bank.rgba_bytes())
                .context("appearance pixel budget")?;
        }
        let bases: Vec<_> = plan.bases().into_iter().collect();
        let world = if bases.is_empty() {
            WorldSceneAssets::load_terrain_with_catalog(context, map, limits, cancel)?
        } else {
            WorldSceneAssets::load_combat_armed_with_catalog(context, map, &bases, limits, cancel)?
        };
        let body_audio =
            super::world_audio::load_canonical_body_bank(context.root(), &world, cancel)?;
        let mut file_bytes = body_audio.file_bytes();
        let mut pcm_bytes = body_audio.pcm_bytes();
        for bank in polymorphs.values() {
            file_bytes = file_bytes
                .checked_add(bank.sound_bank().file_bytes())
                .context("player WAV budget overflow")?;
            pcm_bytes = pcm_bytes
                .checked_add(bank.sound_bank().pcm_bytes())
                .context("player PCM budget overflow")?;
        }
        let budget = BankLimits::default();
        ensure!(
            file_bytes <= budget.max_bytes && pcm_bytes <= budget.max_bytes,
            "combined player audio budget exceeded"
        );
        ensure!(
            !cancel.load(Ordering::Acquire),
            "player appearance load cancelled"
        );
        Ok(Self {
            world,
            forms,
            polymorphs,
            body_audio,
            spells,
            utility,
        })
    }
    pub fn supports(&self, plan: &Plan) -> bool {
        self.world.map().canonical_digest() == plan.snapshot.map_digest()
            && plan.forms.values().all(|f| self.forms.contains(f))
    }
    pub fn render(
        &self,
        plan: &Plan,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
    ) -> Result<RenderInfo> {
        self.render_with_walks(plan, snapshot, canvas, cancel, &[])
    }
    pub fn render_motion(
        &self,
        motion: &Motion,
        now: Instant,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
    ) -> Result<RenderInfo> {
        self.render_motion_effects(motion, now, canvas, cancel, None, None)
    }
    fn render_motion_effects(
        &self,
        motion: &Motion,
        now: Instant,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        spells: Option<&super::battle_spells::Timeline>,
        utility: Option<&utility::Timeline>,
    ) -> Result<RenderInfo> {
        ensure!(
            !cancel.load(Ordering::Acquire) && self.supports(&motion.plan),
            "cancelled or unloaded moving player appearances"
        );
        let snapshot = motion.scene(now)?;
        let mut walks = Vec::new();
        for (actor, wire) in snapshot
            .actors()
            .iter()
            .zip(&motion.plan.observation.data().actors.0)
        {
            let pose = motion.timeline.pose(actor.entity_id.try_into()?, now)?;
            if actor.action != BaseAction::Walk {
                continue;
            }
            let from = pose.walk_from.context("unconfirmed player walk origin")?;
            let qualified = match motion.plan.forms[&actor.entity_id] {
                Form::Base(base) => self
                    .world
                    .player_walk_animation(base.appearance, base.weapon_type)
                    .context("base player walk absent")?,
                Form::Polymorph { form, .. } => self.polymorphs[&form]
                    .action(BaseAction::Walk)?
                    .walk()
                    .context("polymorph player walk absent")?,
            };
            walks.push(self.world.player_walk_qualified(
                &snapshot,
                actor.entity_id,
                from,
                pose.elapsed,
                wire.step_interval_nanos.try_into()?,
                cancel,
                qualified,
            )?);
        }
        self.render_with_effects(
            &motion.plan,
            &snapshot,
            canvas,
            cancel,
            &walks,
            spells,
            utility,
            now,
        )
    }
    fn render_with_walks(
        &self,
        plan: &Plan,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        walks: &[PlayerWalkPlacement],
    ) -> Result<RenderInfo> {
        self.render_with_effects(
            plan,
            snapshot,
            canvas,
            cancel,
            walks,
            None,
            None,
            Instant::now(),
        )
    }
    fn render_with_effects(
        &self,
        plan: &Plan,
        snapshot: &SceneSnapshot,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        walks: &[PlayerWalkPlacement],
        spells: Option<&super::battle_spells::Timeline>,
        utility: Option<&utility::Timeline>,
        now: Instant,
    ) -> Result<RenderInfo> {
        ensure!(
            !cancel.load(Ordering::Acquire),
            "player appearance render cancelled"
        );
        ensure!(
            plan.matches(snapshot) && self.supports(plan),
            "stale or unloaded player appearances"
        );
        let mut frames = BTreeMap::new();
        for actor in snapshot.actors() {
            if let Form::Polymorph { form, .. } = plan.forms[&actor.entity_id] {
                let bank = &self.polymorphs[&form];
                let source =
                    if let Some(walk) = walks.iter().find(|w| w.entity() == actor.entity_id) {
                        walk.source()
                    } else {
                        bank.action(actor.action)?
                            .sample(actor.action_elapsed, actor.heading)?
                            .0
                    };
                frames.insert(
                    actor.entity_id,
                    PlayerFrame::from(bank.frame(actor.action, actor.heading, source)?),
                );
            }
        }
        let mut overrides = spells
            .map(|s| s.overrides(snapshot, &self.spells, now))
            .transpose()?
            .unwrap_or_default();
        let effects = spells
            .map(|s| s.disintegrations(now))
            .transpose()?
            .unwrap_or_default();
        if let Some(spells) = spells {
            for (id, form, heading, elapsed) in spells.polymorph_idle(snapshot, now)? {
                ensure!(
                    matches!(plan.forms.get(&id), Some(Form::Polymorph { form: actual, .. }) if *actual == form),
                    "cast idle appearance owner mismatch"
                );
                let bank = &self.polymorphs[&form];
                let source = bank.action(BaseAction::Idle)?.sample(elapsed, heading)?.0;
                frames.insert(
                    id,
                    PlayerFrame::from(bank.frame(BaseAction::Idle, heading, source)?),
                );
            }
            for (id, frame) in spells.polymorph_frames(snapshot, &self.spells, now)? {
                ensure!(
                    matches!(plan.forms.get(&id), Some(Form::Polymorph { form, .. }) if *form == frame.form),
                    "caster appearance owner mismatch"
                );
                ensure!(
                    !walks.iter().any(|w| w.entity() == id),
                    "walking caster overlay"
                );
                frames.insert(
                    id,
                    PlayerFrame {
                        form: frame.form,
                        action: None,
                        source: frame.source,
                        body: frame.body,
                        shadow: frame.shadow,
                        attachment: frame.attachment,
                    },
                );
            }
        }
        if let Some(utility) = utility {
            overrides.extend(utility.overrides(snapshot, &self.utility, now)?);
            for (id, frame) in utility.frames(snapshot, &self.utility, &self.polymorphs, now)? {
                ensure!(
                    !walks.iter().any(|w| w.entity() == id),
                    "walking utility caster overlay"
                );
                frames.insert(id, frame);
            }
        }
        let draws = Draws { snapshot, frames };
        let info = if spells.is_none() && utility.is_none() {
            self.world
                .render_player_appearances(snapshot, canvas, cancel, &draws, walks)?
        } else {
            self.world.render_player_appearances_effects(
                snapshot, canvas, cancel, &draws, walks, &overrides, &effects,
            )?
        };
        if let Some(spells) = spells {
            spells.draw_projected(canvas, info.projection, &self.spells, now, cancel)?;
        }
        if let Some(utility) = utility {
            utility.draw(canvas, info.projection, &self.utility, now, cancel)?;
        }
        Ok(info)
    }
    /// Transactional player body + supported spell presentation. No partial
    /// canvas/audio publication on a failed or cancelled candidate.
    pub fn render_playback(
        &self,
        playback: &Playback,
        now: Instant,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
    ) -> Result<(Playback, RenderInfo, Option<super::world_audio::AudioFrame>)> {
        ensure!(
            playback.last_render.is_none_or(|old| now >= old),
            "player render clock reversed"
        );
        let spells = playback.spells.advance(
            playback.motion.plan.observation.clone(),
            now,
            &self.spells,
            cancel,
        )?;
        let mut surface = Canvas::new(canvas.width(), canvas.height())?;
        let utility =
            playback
                .utility
                .advance(playback.motion.plan.clone(), now, &self.utility, cancel)?;
        let info = self.render_motion_effects(
            &playback.motion,
            now,
            &mut surface,
            cancel,
            Some(&spells),
            Some(&utility),
        )?;
        let (mut candidate, body_audio) = playback.sample_audio(self, now, cancel)?;
        candidate.spells = spells;
        candidate.utility = utility;
        let [teleport_audio, healing_audio] = candidate.utility.audio(&self.utility, now)?;
        let audio = candidate.mixer.merge(
            [
                candidate.spells.audio(&self.spells, now),
                body_audio,
                teleport_audio,
                healing_audio,
            ],
            now,
        )?;
        ensure!(
            !cancel.load(Ordering::Acquire),
            "player presentation cancelled"
        );
        candidate.last_render = Some(now);
        *canvas = surface;
        Ok((candidate, info, audio))
    }
    /// Shared player presentation with the explicitly loaded effect-ID status
    /// bank. Missing status art cancels body/audio publication as well. This
    /// remains a scene component, not the full world/LOAD or native UI skin.
    pub fn render_playback_with_status(
        &self,
        playback: &Playback,
        status: &super::vip_status::Assets,
        now: Instant,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        origin: (u32, u32),
        columns: u32,
    ) -> Result<(Playback, RenderInfo, Option<super::world_audio::AudioFrame>)> {
        let mut surface = Canvas::new(canvas.width(), canvas.height())?;
        let result = self.render_playback(playback, now, &mut surface, cancel)?;
        status.draw(
            playback.motion.plan.observation(),
            &mut surface,
            origin,
            columns,
            cancel,
        )?;
        ensure!(
            !cancel.load(Ordering::Acquire),
            "player status composition cancelled"
        );
        *canvas = surface;
        Ok(result)
    }
}

/// Player body and reviewed directed/utility spell presentation with owned cue history.
/// Construction suppresses historical attacks/deaths. Each advance/sample is
/// an immutable candidate: failure cannot consume the published audio history.
/// This is not complete VIP/NPC/all-spell admission and never grants LOAD authority.
#[derive(Clone)]
pub struct Playback {
    motion: Motion,
    audio: super::battle_body_audio::Timeline,
    spells: super::battle_spells::Timeline,
    utility: utility::Timeline,
    mixer: super::battle_audio::Mixer,
    last_render: Option<Instant>,
}
impl Playback {
    pub fn new(plan: Plan, now: Instant) -> Result<Self> {
        Self::player_only(&plan)?;
        let audio = super::battle_body_audio::Timeline::new(plan.observation.clone(), now);
        let spells = super::battle_spells::Timeline::new(plan.observation.clone(), now);
        let utility = utility::Timeline::new(plan.clone(), now);
        Ok(Self {
            motion: Motion::new(plan, now)?,
            audio,
            spells,
            utility,
            mixer: super::battle_audio::Mixer::default(),
            last_render: None,
        })
    }
    fn player_only(plan: &Plan) -> Result<()> {
        ensure!(
            plan.observation
                .data()
                .npcs
                .as_ref()
                .is_none_or(|n| n.actors.0.is_empty()),
            "player body playback does not qualify NPCs"
        );
        Ok(())
    }
    pub fn motion(&self) -> &Motion {
        &self.motion
    }
    pub fn advance(
        &self,
        plan: Plan,
        assets: &Assets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::player_only(&plan)?;
        ensure!(assets.supports(&plan), "unloaded player audio appearances");
        ensure!(
            self.last_render.is_none_or(|old| now >= old),
            "player observation predates paint"
        );
        super::battle_spells::check_map(&plan.observation, assets.world.map())?;
        utility::check_map(&plan.observation, assets.world.map())?;
        let utility = self
            .utility
            .advance(plan.clone(), now, &assets.utility, cancel)?;
        let spells = self
            .spells
            .advance(plan.observation.clone(), now, &assets.spells, cancel)?;
        let mut motion = self.motion.advance(plan, now, cancel)?;
        let mut returns = BTreeMap::new();
        for (entity, form) in &motion.plan.forms {
            let actor = i64::try_from(*entity)?;
            let action = motion.timeline.pose(actor, now)?.action;
            if !matches!(action, BaseAction::Attack | BaseAction::Damage) {
                continue;
            }
            let proof = match form {
                Form::Base(base) => super::battle_motion::finish::QualifiedReturn::base(
                    action,
                    assets
                        .world
                        .combat_animation_armed(base.appearance, base.weapon_type, action)
                        .context("player completion animation absent")?,
                )?,
                Form::Polymorph { form, .. } => {
                    super::battle_motion::finish::QualifiedReturn::polymorph(
                        assets
                            .polymorphs
                            .get(form)
                            .context("player completion polymorph absent")?
                            .action(action)?,
                    )?
                }
            };
            returns.insert(actor, proof);
        }
        motion.timeline = motion.timeline.with_player_returns(&returns, now, cancel)?;
        let scene = motion.scene(now)?;
        let mut polymorphs = BTreeMap::new();
        for (entity, form) in &motion.plan.forms {
            if let Form::Polymorph { form, .. } = form {
                polymorphs.insert(
                    *entity,
                    assets
                        .polymorphs
                        .get(form)
                        .context("polymorph audio bank absent")?,
                );
            }
        }
        let audio = self.audio.advance_players(
            motion.plan.observation.clone(),
            &motion.timeline,
            &scene,
            &assets.world,
            &polymorphs,
            None,
            now,
            cancel,
        )?;
        Ok(Self {
            motion,
            audio,
            spells,
            utility,
            mixer: self.mixer.clone(),
            last_render: self.last_render,
        })
    }
    pub fn sample_audio(
        &self,
        assets: &Assets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<(Self, Option<super::world_audio::AudioFrame>)> {
        ensure!(
            assets.supports(&self.motion.plan),
            "unloaded player audio appearances"
        );
        let (audio, frame) =
            self.audio
                .sample(&self.motion.timeline, &assets.body_audio, None, now, cancel)?;
        Ok((
            Self {
                motion: self.motion.clone(),
                audio,
                spells: self.spells.clone(),
                utility: self.utility.clone(),
                mixer: self.mixer.clone(),
                last_render: self.last_render,
            },
            frame,
        ))
    }
}

/// Opaque frame set bound to this exact immutable snapshot, not just an entity
/// number. The renderer cannot accidentally use an earlier world's body bank.
pub(crate) struct Draws<'a> {
    snapshot: &'a SceneSnapshot,
    frames: BTreeMap<u64, PlayerFrame<'a>>,
}
pub(crate) struct PlayerFrame<'a> {
    pub(crate) form: PolymorphForm,
    pub(crate) action: Option<BaseAction>,
    pub(crate) source: usize,
    pub(crate) body: super::npc_assets::NpcLayer<'a>,
    pub(crate) shadow: super::npc_assets::NpcLayer<'a>,
    pub(crate) attachment: super::npc_assets::NpcLayer<'a>,
}
impl<'a> From<PolymorphFrame<'a>> for PlayerFrame<'a> {
    fn from(f: PolymorphFrame<'a>) -> Self {
        Self {
            form: f.form,
            action: Some(f.action),
            source: f.source,
            body: f.body,
            shadow: f.shadow,
            attachment: f.attachment,
        }
    }
}
impl<'a> Draws<'a> {
    pub(crate) fn matches(&self, snapshot: &SceneSnapshot) -> bool {
        std::ptr::eq(self.snapshot, snapshot)
    }
    pub(crate) fn frame(&self, entity: u64) -> Option<&PlayerFrame<'a>> {
        self.frames.get(&entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn active_plan() -> Plan {
        let mut data = super::super::battle_payload::data_tests::fixture();
        data.world.revision = 2;
        data.world.phase = d::Phase::Active;
        data.world.loading_elapsed_nanos = 100_000_000;
        data.world.active_elapsed_nanos = 0;
        for a in &mut data.actors.0 {
            a.loaded = true;
            a.step_interval_nanos = 200_000_000;
            // Keep all eight adjacent directions inside the declared sector.
            a.position.x += 10;
            a.position.y += 10;
        }
        Plan::from_observation(super::super::battle_scene::tests::observation(data)).unwrap()
    }
    #[test]
    fn motion_uses_confirmed_single_steps_and_suppresses_missing_routes() {
        let t = Instant::now();
        let plan = active_plan();
        let motion = Motion::new(plan.clone(), t).unwrap();
        for (heading, (dx, dy)) in super::super::world_map::DIRECTIONS.into_iter().enumerate() {
            let mut data = plan.observation.data().clone();
            data.world.revision += 1;
            data.actors.0[0].last_move_sequence = 1;
            data.actors.0[0].position.x += dx;
            data.actors.0[0].position.y += dy;
            data.actors.0[0].heading = heading as i32;
            let next = Plan::from_observation(super::super::battle_scene::tests::observation(
                data.clone(),
            ))
            .unwrap();
            let at = t + Duration::from_millis(10);
            let moved = motion
                .advance(next.clone(), at, &AtomicBool::new(false))
                .unwrap();
            assert_eq!(
                moved.scene(at).unwrap().actors()[0].action,
                BaseAction::Walk
            );
            assert_eq!(
                moved
                    .scene(at + Duration::from_millis(50))
                    .unwrap()
                    .actors()[0]
                    .action_elapsed,
                Duration::from_millis(50)
            );
            assert_eq!(
                moved
                    .scene(at + Duration::from_millis(200))
                    .unwrap()
                    .actors()[0]
                    .action,
                BaseAction::Idle
            );
            assert_eq!(
                Motion::new(next, at).unwrap().scene(at).unwrap().actors()[0].action,
                BaseAction::Idle
            );
            data.actors.0[0].last_move_sequence = 2;
            // Two consumed movement sequences require at least two world
            // revisions even when the consumer skipped the intermediate one.
            data.world.revision += 1;
            let skipped =
                Plan::from_observation(super::super::battle_scene::tests::observation(data))
                    .unwrap();
            assert_eq!(
                motion
                    .advance(skipped, at, &AtomicBool::new(false))
                    .unwrap()
                    .scene(at)
                    .unwrap()
                    .actors()[0]
                    .action,
                BaseAction::Idle
            );
        }
    }
    #[test]
    fn motion_rejects_cancel_and_reverse_time_without_changing_previous_pose() {
        let t = Instant::now();
        let plan = active_plan();
        let motion = Motion::new(plan.clone(), t).unwrap();
        let before = motion.scene(t).unwrap();
        assert!(motion
            .advance(plan.clone(), t, &AtomicBool::new(true))
            .is_err());
        assert!(motion
            .advance(plan, t - Duration::from_nanos(1), &AtomicBool::new(false))
            .is_err());
        assert!(motion.scene(t - Duration::from_nanos(1)).is_err());
        assert_eq!(motion.scene(t).unwrap().actors(), before.actors());
    }
    fn plan() -> Plan {
        Plan::from_observation(super::super::battle_scene::tests::observation(
            super::super::battle_payload::data_tests::fixture(),
        ))
        .unwrap()
    }
    #[test]
    fn ordinary_players_keep_source_keys_class_sex_and_weapon() {
        let plan = plan();
        let d = plan.observation().data();
        assert!(plan.matches(plan.snapshot()));
        assert_eq!(plan.forms.len(), d.actors.0.len());
        for (actor, wire) in plan.snapshot.actors().iter().zip(&d.actors.0) {
            let member = d.scene_member(wire).unwrap();
            assert_eq!(
                (actor.source_id, actor.object_id),
                (member.identity.source as u32, member.identity.id as u64)
            );
            assert_eq!(
                plan.forms[&actor.entity_id],
                Form::Base(CombatAppearance {
                    appearance: actor.appearance,
                    weapon_type: actor.weapon_type
                })
            );
        }
    }
    #[test]
    fn presentation_pose_does_not_authorize_a_different_actor_cell_or_attachment() {
        let plan = plan();
        let base = plan.snapshot();
        let mut actors = base.actors().to_vec();
        actors[0].action = BaseAction::Attack;
        actors[0].action_elapsed = Duration::from_millis(50);
        assert!(plan.matches(&base.with_actors(actors).unwrap()));
        for mutation in 0..7 {
            let mut actors = base.actors().to_vec();
            let mut world = base.world_id();
            let mut revision = base.revision();
            let mut digest = base.map_digest();
            match mutation {
                0 => actors[0].source_id = 987,
                1 => actors[0].object_id += 1000,
                2 => actors[0].appearance.sex ^= 1,
                3 => actors[0].cell.x += 1,
                4 => world += 1,
                5 => revision += 1,
                _ => digest[0] ^= 1,
            }
            let other =
                SceneSnapshot::new(world, revision, digest, base.selected_entity_id(), actors)
                    .unwrap();
            assert!(!plan.matches(&other), "mutation{mutation}");
        }
    }
    #[test]
    fn drawn_frame_set_is_bound_to_the_exact_snapshot_not_a_reused_number() {
        let plan = plan();
        let cloned = plan.snapshot.clone();
        let frames = Draws {
            snapshot: plan.snapshot(),
            frames: BTreeMap::new(),
        };
        assert!(frames.matches(plan.snapshot()));
        assert!(!frames.matches(&cloned));
    }
}
