//! Canonical skill4/70/77 presentation. Confirmed receipts own typed targets;
//! caster Hit launches pixels/cues only, never damage or command authority.
//! Assets are shared Lineage resources. No legacy Frame or target pointer.
use super::{
    battle_follow::Observation,
    battle_payload::data as d,
    combat_animation::CrossedCue,
    combat_audio::SoundBank,
    compositor::Canvas,
    disintegrate_assets::DisintegrateAssets,
    disintegrate_effect::{Binding, DisintegrateDraw, DisintegrateEffect, Variant},
    fire_storm_effect::{FireStormAssets, FireStormPlayback},
    light_arrow_effect::{LightArrowAssets, LightArrowError, LightArrowPath, LightArrowPlayback},
    paced_animation::PacedClock,
    shared_action_catalog::SharedActionCatalog,
    world_audio::{AudioFrame, EventLog},
    world_map::{ViewProjection, WorldCell, WorldMap},
    world_scene::{ActorOverride, BaseAppearance, SceneSnapshot},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
const MAX_EFFECTS: usize = 64;
const MAX_FLIGHTS: usize = 256;
const MAX_AGE: Duration = Duration::from_millis(500);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CasterForm {
    Base(BaseAppearance),
    Polymorph(super::polymorph_assets::PolymorphForm),
}
fn caster_form(
    data: &d::UnvalidatedData,
    actor: &d::Actor,
    member: &d::Member,
) -> Result<CasterForm> {
    let graphics = match &data.vip {
        None => 0,
        Some(vip) => {
            vip.appearances
                .as_ref()
                .context("spell public appearance absent")?
                .0
                .iter()
                .find(|a| a.entity == actor.entity)
                .context("spell appearance owner absent")?
                .graphics
        }
    };
    Ok(
        match super::polymorph_assets::PolymorphForm::from_wire(graphics)? {
            Some(form) => CasterForm::Polymorph(form),
            None => CasterForm::Base(BaseAppearance {
                class: member.class.try_into()?,
                sex: member.sex.try_into()?,
            }),
        },
    )
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Arrow,
    Fire,
    Disintegrate,
}
fn kind(lane: d::Lane) -> Option<Kind> {
    match lane {
        d::Lane::LightArrow | d::Lane::NpcLightArrow => Some(Kind::Arrow),
        d::Lane::FireStorm | d::Lane::NpcFireStorm => Some(Kind::Fire),
        d::Lane::Disintegrate | d::Lane::NpcDisintegrate => Some(Kind::Disintegrate),
        _ => None,
    }
}
fn cell(c: &d::Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "canonical spell cancelled");
    Ok(())
}
fn recent(receipt: i64, observed: i64) -> bool {
    observed
        .checked_sub(receipt)
        .is_some_and(|n| (0..=500_000_000).contains(&n))
}
/// Native class4 snapshots the latest confirmed cell at launch, then keeps an
/// immutable flight. Retired or different-life targets keep the receipt cell.
fn arrow_target(data: &d::UnvalidatedData, impact: &d::Impact) -> Result<WorldCell> {
    let hit = impact.targets.0.first().context("arrow target absent")?;
    resolved_arrow_target(data, hit)
}
fn resolved_arrow_target(data: &d::UnvalidatedData, hit: &d::Effect) -> Result<WorldCell> {
    let current = match &hit.target {
        d::Target::Participant { entity, identity } => {
            let actor = data
                .actors
                .0
                .iter()
                .find(|a| a.entity == *entity)
                .context("arrow participant absent")?;
            let member = data.scene_member(actor)?;
            ensure!(
                &member.identity == identity,
                "arrow participant identity changed"
            );
            (super::battle_motion::life(actor) == Some(hit.life)
                && data
                    .time_attack
                    .as_ref()
                    .is_none_or(|t| !t.departed.0.contains(entity)))
            .then_some(&actor.position)
        }
        d::Target::Npc { scope, serial } => {
            ensure!(scope == &data.world.identity, "arrow NPC scope changed");
            data.npcs
                .as_ref()
                .context("arrow NPC state absent")?
                .actors
                .0
                .iter()
                .find(|n| n.serial == *serial && n.life == hit.life)
                .map(|n| &n.cell)
        }
        d::Target::Honor { entity, team } => Some(
            &data
                .honor
                .objectives
                .0
                .iter()
                .find(|t| t.entity == *entity && t.team == *team)
                .context("arrow Honor target absent")?
                .cell,
        ),
        _ => anyhow::bail!("unsupported arrow target domain"),
    };
    Ok(cell(current.unwrap_or(&hit.cell)))
}
pub(crate) fn check_map(observation: &Observation, map: &WorldMap) -> Result<()> {
    for a in &observation.data().actors.0 {
        for r in a
            .spells
            .histories
            .0
            .iter()
            .filter(|s| kind(s.lane).is_some())
            .filter_map(|s| s.receipt.as_ref())
        {
            ensure!(
                map.standing(cell(&r.impact.usage.before.cell))
                    && r.impact
                        .targets
                        .0
                        .iter()
                        .all(|t| map.standing(cell(&t.cell))),
                "canonical spell receipt outside qualified map"
            );
            if kind(r.impact.lane) == Some(Kind::Arrow) {
                ensure!(
                    map.standing(arrow_target(observation.data(), &r.impact)?),
                    "arrow launch target outside qualified map"
                );
            }
        }
    }
    Ok(())
}
pub(crate) struct Assets {
    fire: Option<FireStormAssets>,
    light: Option<LightArrowAssets>,
    disintegrate: Option<Arc<DisintegrateAssets>>,
    polymorph: Option<super::polymorph_assets::cast::Assets>,
}
impl Assets {
    pub(crate) fn load(
        root: &Path,
        catalog: &SharedActionCatalog,
        observation: &Observation,
        forms: &[BaseAppearance],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let enabled = |k| {
            observation
                .data()
                .actors
                .0
                .iter()
                .any(|a| a.spells.histories.0.iter().any(|s| kind(s.lane) == Some(k)))
        };
        let fire = if enabled(Kind::Fire) {
            Some(FireStormAssets::load_with_catalog(catalog, forms, cancel)?)
        } else {
            None
        };
        let light = if enabled(Kind::Arrow) || enabled(Kind::Disintegrate) {
            Some(LightArrowAssets::load_with_catalog(catalog, forms, cancel)?)
        } else {
            None
        };
        let disintegrate = if enabled(Kind::Disintegrate) {
            Some(Arc::new(DisintegrateAssets::load(root, cancel)?))
        } else {
            None
        };
        let mut polymorph = None;
        if [Kind::Arrow, Kind::Fire, Kind::Disintegrate]
            .into_iter()
            .any(enabled)
        {
            for actor in &observation.data().actors.0 {
                if let CasterForm::Polymorph(form) = caster_form(
                    observation.data(),
                    actor,
                    observation.data().scene_member(actor)?,
                )? {
                    if polymorph.is_none() {
                        polymorph = Some(super::polymorph_assets::cast::Assets::load(
                            catalog, form, cancel,
                        )?);
                    }
                }
            }
        }
        let bank = Self {
            fire,
            light,
            disintegrate,
            polymorph,
        };
        for form in forms {
            for k in [Kind::Arrow, Kind::Fire, Kind::Disintegrate]
                .into_iter()
                .filter(|k| enabled(*k))
            {
                let (delay, duration) = bank.timing(k, CasterForm::Base(*form))?;
                ensure!(delay < duration, "canonical caster18 lacks launch window");
            }
        }
        active(cancel)?;
        Ok(bank)
    }
    pub(crate) fn compatible(&self, old: &Self) -> bool {
        self.fire.is_some() == old.fire.is_some()
            && self.light.is_some() == old.light.is_some()
            && self.disintegrate.is_some() == old.disintegrate.is_some()
    }
    pub(crate) fn polymorph_cache_bytes(&self) -> (usize, usize) {
        self.polymorph
            .as_ref()
            .map_or((0, 0), |p| (p.encoded_bytes(), p.rgba_bytes()))
    }
    fn timing(&self, kind: Kind, form: CasterForm) -> Result<(Duration, Duration)> {
        let form = match form {
            CasterForm::Base(base) => base,
            CasterForm::Polymorph(_) => {
                let q = self
                    .polymorph
                    .as_ref()
                    .context("polymorph caster not loaded")?
                    .action();
                return Ok((q.launch(), q.terminal()));
            }
        };
        match kind {
            Kind::Fire => {
                let b = self.fire.as_ref().context("fire bank absent")?;
                Ok((b.cast_launch_delay(form)?, b.cast_duration(form)?))
            }
            Kind::Arrow | Kind::Disintegrate => {
                let b = &self.light.as_ref().context("caster18 bank absent")?;
                Ok((b.cast_launch_delay(form)?, b.cast_duration(form)?))
            }
        }
    }
    fn sound(&self) -> Option<Arc<SoundBank>> {
        self.fire
            .as_ref()
            .map(FireStormAssets::sound_bank)
            .or_else(|| self.light.as_ref().map(LightArrowAssets::sound_bank))
            .or_else(|| self.disintegrate.as_ref().map(|b| b.sound_bank()))
    }
}
#[derive(Clone)]
struct Cast {
    kind: Kind,
    receipt: d::SpellReceipt,
    form: CasterForm,
    sequence: i64,
    heading: u8,
    body: PacedClock,
    launch_at: Instant,
    launched: bool,
    visible: bool,
    tail: Option<super::battle_motion::finish::IdleReturn>,
}
#[derive(Clone)]
enum Playback {
    Arrow {
        path: LightArrowPath,
        playback: LightArrowPlayback,
    },
    Fire {
        origin: WorldCell,
        playback: FireStormPlayback,
    },
    Disintegrate(DisintegrateEffect),
}
#[derive(Clone)]
struct Effect {
    started: Instant,
    duration: Duration,
    playback: Playback,
}
#[derive(Clone)]
pub(crate) struct Timeline {
    observation: Observation,
    observed: Instant,
    casts: BTreeMap<i64, Cast>,
    effects: Vec<Effect>,
    next_event: u64,
    audio: EventLog,
    pub(crate) skipped: bool,
}
fn current(data: &d::UnvalidatedData, a: &d::Actor, m: &d::Member, cast: &Cast) -> bool {
    let use_ = &cast.receipt.impact.usage;
    caster_form(data, a, m).ok() == Some(cast.form)
        && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
        && data
            .time_attack
            .as_ref()
            .is_none_or(|t| !t.departed.0.contains(&a.entity))
        && super::battle_motion::life(a) == Some(use_.before.life)
        && a.resources
            .as_ref()
            .is_some_and(|r| r.equipment_revision == use_.before.equipment_revision)
        && a.last_move_sequence == cast.sequence
        && a.position == use_.before.cell
        && super::battle_motion::latest_deliberate_event(data, a, m) <= use_.revision
}
impl Timeline {
    pub(crate) fn new(observation: Observation, now: Instant) -> Self {
        Self {
            observation,
            observed: now,
            casts: BTreeMap::new(),
            effects: vec![],
            next_event: 0,
            audio: EventLog::default(),
            skipped: false,
        }
    }
    /// Clone/validate/commit at the scene boundary: cancellation or a bad bank
    /// cannot partially consume cues or restart a live presentation.
    pub(crate) fn advance(
        &self,
        next: Observation,
        now: Instant,
        assets: &Assets,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(now >= self.observed, "canonical spell clock reversed");
        self.observation.check_successor(&next)?;
        ensure!(
            next.data().world.phase != d::Phase::Aborted,
            "aborted spells must retire"
        );
        let mut out = self.clone();
        let data = next.data();
        out.effects
            .retain(|e| now.saturating_duration_since(e.started) < e.duration);
        for a in &data.actors.0 {
            active(cancel)?;
            let m = data.scene_member(a)?;
            // Retain at most the newest supported cast per body, not a burst of
            // missed lanes. An even newer unsupported action cancels below.
            let receipt = a
                .spells
                .histories
                .0
                .iter()
                .filter(|s| kind(s.lane).is_some())
                .filter_map(|s| s.receipt.as_ref())
                .max_by_key(|r| r.impact.usage.revision);
            if let Some(r) = receipt.filter(|r| {
                r.impact.usage.revision > self.observation.data().world.revision
                    && recent(r.loading_elapsed_nanos, data.world.loading_elapsed_nanos)
            }) {
                let k = kind(r.impact.lane).context("unsupported spell lane")?;
                let form = caster_form(data, a, m)?;
                let (delay, _) = assets.timing(k, form)?;
                let heading = if matches!(k, Kind::Arrow | Kind::Disintegrate) {
                    let target = r.impact.targets.0.first().context("skill77 lacks target")?;
                    let from = &r.impact.usage.before.cell;
                    let delta = (
                        (i64::from(target.cell.x) - i64::from(from.x)).signum() as i32,
                        (i64::from(target.cell.y) - i64::from(from.y)).signum() as i32,
                    );
                    let direction = super::world_map::DIRECTIONS
                        .iter()
                        .position(|d| *d == delta);
                    if k == Kind::Arrow {
                        direction.map_or(a.heading as u8, |h| h as u8)
                    } else {
                        direction.context("skill77 target shares caster cell")? as u8
                    }
                } else {
                    a.heading.try_into()?
                };
                let mut body = PacedClock::new(now);
                let base = a.cast_interval_nanos.try_into()?;
                body.retime(now, base, super::battle_motion::paced_nanos(a, base, false))?;
                let launch_at = body.until(now, delay)?;
                let tail = match form {
                    CasterForm::Base(_) => None,
                    CasterForm::Polymorph(_) => {
                        Some(super::battle_motion::finish::IdleReturn::start(
                            super::battle_motion::finish::QualifiedReturn::polymorph_cast(
                                assets
                                    .polymorph
                                    .as_ref()
                                    .context("polymorph cast tail bank absent")?
                                    .action(),
                            ),
                            &body,
                            now,
                        )?)
                    }
                };
                out.casts.insert(
                    a.entity,
                    Cast {
                        kind: k,
                        receipt: r.clone(),
                        form,
                        sequence: a.last_move_sequence,
                        heading,
                        body,
                        launch_at,
                        launched: false,
                        visible: true,
                        tail,
                    },
                );
            }
            if out
                .casts
                .get(&a.entity)
                .is_some_and(|c| !current(data, a, m, c))
            {
                out.casts.remove(&a.entity);
            }
            let Some(c) = out.casts.get_mut(&a.entity) else {
                continue;
            };
            c.visible &= a
                .vital
                .as_ref()
                .is_some_and(|v| v.hurt_revision <= c.receipt.impact.usage.revision);
            let base = a.cast_interval_nanos.try_into()?;
            c.body
                .retime(now, base, super::battle_motion::paced_nanos(a, base, false))?;
            if let Some(tail) = &mut c.tail {
                tail.retime(&c.body, now)?;
            }
            let (delay, _) = assets.timing(c.kind, c.form)?;
            let elapsed = c.body.elapsed(now)?;
            if !c.launched && elapsed < delay {
                c.launch_at = c.body.until(now, delay)?;
            }
            if !c.launched && elapsed >= delay {
                c.launched = true;
                // Never invent a moved target's historical position at a
                // missed deadline. Class4 launches using this snapshot now,
                // within a bounded grace window; older pending visuals retire.
                let flight = if c.kind == Kind::Arrow {
                    if now.saturating_duration_since(c.launch_at) > MAX_AGE {
                        out.skipped = true;
                        continue;
                    }
                    match LightArrowPath::new(
                        cell(&c.receipt.impact.usage.before.cell),
                        arrow_target(data, &c.receipt.impact)?,
                    ) {
                        Ok(path) => Some(path),
                        Err(LightArrowError::Bound(_)) => {
                            out.skipped = true;
                            continue;
                        }
                        Err(error) => return Err(error.into()),
                    }
                } else {
                    None
                };
                let duration = match c.kind {
                    Kind::Arrow => assets
                        .light
                        .as_ref()
                        .context("arrow bank absent")?
                        .effect_duration(flight.as_ref().context("arrow path absent")?),
                    Kind::Fire => assets
                        .fire
                        .as_ref()
                        .context("fire bank absent")?
                        .effect_duration(),
                    Kind::Disintegrate => Duration::from_millis(2500),
                };
                // Keep the original wall deadline after a stalled render.
                let started = if c.kind == Kind::Arrow {
                    now
                } else {
                    c.launch_at
                };
                if now.saturating_duration_since(started) >= duration {
                    continue;
                }
                let flights = out
                    .effects
                    .iter()
                    .filter(|e| matches!(e.playback, Playback::Arrow { .. }))
                    .count();
                if if c.kind == Kind::Arrow {
                    flights >= MAX_FLIGHTS
                } else {
                    out.effects.len() - flights >= MAX_EFFECTS
                } {
                    out.skipped = true;
                    continue;
                }
                out.next_event = out
                    .next_event
                    .checked_add(1)
                    .filter(|n| *n <= i64::MAX as u64)
                    .context("spell presentation ID exhausted")?;
                let playback = match c.kind {
                    Kind::Arrow => {
                        let path = flight.context("arrow path absent")?;
                        let playback = assets
                            .light
                            .as_ref()
                            .context("arrow bank absent")?
                            .start_effect(&path);
                        Playback::Arrow { path, playback }
                    }
                    Kind::Fire => Playback::Fire {
                        origin: cell(&c.receipt.impact.usage.before.cell),
                        playback: assets
                            .fire
                            .as_ref()
                            .context("fire bank absent")?
                            .start_effect(),
                    },
                    Kind::Disintegrate => {
                        let target = c
                            .receipt
                            .impact
                            .targets
                            .0
                            .first()
                            .context("skill77 target absent")?;
                        let d::Detail::Disintegrate { critical, .. } = c.receipt.detail else {
                            anyhow::bail!("skill77 detail mismatch");
                        };
                        Playback::Disintegrate(DisintegrateEffect::start(
                            assets
                                .disintegrate
                                .as_ref()
                                .context("skill77 bank absent")?
                                .clone(),
                            Binding {
                                world_id: data.world.identity.match_id.try_into()?,
                                map_digest: data.manifest.digest,
                                event_id: out.next_event,
                                start_revision: c.receipt.impact.usage.revision.try_into()?,
                                target: cell(&target.cell),
                            },
                            if critical {
                                Variant::Critical
                            } else {
                                Variant::Normal
                            },
                        )?)
                    }
                };
                out.effects.push(Effect {
                    started,
                    duration,
                    playback,
                });
            }
        }
        let mut finished = Vec::new();
        for (id, c) in &out.casts {
            // A completed polymorph keeps a bounded idle origin/facing until
            // a newer confirmed body event replaces it. No effect is replayed.
            if c.tail.is_none() && c.body.peek(now)? >= assets.timing(c.kind, c.form)?.1 {
                finished.push(*id);
            }
        }
        for id in finished {
            out.casts.remove(&id);
        }
        for e in &mut out.effects {
            active(cancel)?;
            let elapsed = now.duration_since(e.started);
            let (cues, bank) = match &mut e.playback {
                Playback::Arrow { playback, .. } => (
                    playback.advance(elapsed)?.crossed_cues,
                    assets
                        .light
                        .as_ref()
                        .context("arrow bank absent")?
                        .sound_bank(),
                ),
                Playback::Fire { playback, .. } => (
                    playback.advance(elapsed)?.crossed_cues,
                    assets
                        .fire
                        .as_ref()
                        .context("fire bank absent")?
                        .sound_bank(),
                ),
                Playback::Disintegrate(playback) => (
                    playback.advance(elapsed)?.crossed_cues,
                    playback.assets().sound_bank(),
                ),
            };
            queue_cues(&mut out.audio, &cues, bank, e.started, now)?;
        }
        active(cancel)?;
        out.observation = next;
        out.observed = now;
        Ok(out)
    }
    pub(crate) fn overrides<'a>(
        &self,
        scene: &SceneSnapshot,
        assets: &'a Assets,
        now: Instant,
    ) -> Result<Vec<ActorOverride<'a>>> {
        ensure!(now >= self.observed, "spell sample reversed");
        let mut draws = Vec::new();
        for (id, cast) in &self.casts {
            let CasterForm::Base(form) = cast.form else {
                continue;
            };
            if !cast.visible || scene.hidden((*id).try_into()?) {
                continue;
            }
            let elapsed = cast.body.peek(now)?;
            if elapsed >= assets.timing(cast.kind, cast.form)?.1 {
                continue;
            }
            let (frame, graphics_id, source_frame, sprite_name, subframe, finished) =
                match cast.kind {
                    Kind::Fire => {
                        let v = assets
                            .fire
                            .as_ref()
                            .context("fire bank absent")?
                            .caster_frame(form, cast.heading, elapsed)?;
                        (
                            v.frame,
                            v.graphics_id,
                            v.source_frame,
                            v.sprite_name,
                            v.subframe,
                            v.finished,
                        )
                    }
                    Kind::Arrow | Kind::Disintegrate => {
                        let v = assets
                            .light
                            .as_ref()
                            .context("caster18 bank absent")?
                            .caster_frame(form, cast.heading, elapsed)?;
                        (
                            v.frame,
                            v.graphics_id,
                            v.source_frame,
                            v.sprite_name,
                            v.subframe,
                            v.finished,
                        )
                    }
                };
            if !finished {
                draws.push(ActorOverride {
                    entity_id: (*id).try_into()?,
                    frame,
                    graphics_id,
                    source_frame,
                    sprite_name,
                    subframe,
                });
            }
        }
        Ok(draws)
    }
    pub(crate) fn polymorph_frames<'a>(
        &self,
        scene: &SceneSnapshot,
        assets: &'a Assets,
        now: Instant,
    ) -> Result<BTreeMap<u64, super::polymorph_assets::cast::Frame<'a>>> {
        ensure!(now >= self.observed, "polymorph spell sample reversed");
        let mut frames = BTreeMap::new();
        for (entity, cast) in &self.casts {
            let CasterForm::Polymorph(_) = cast.form else {
                continue;
            };
            let id = (*entity).try_into()?;
            if !cast.visible
                || scene.hidden(id)
                || now
                    >= cast
                        .tail
                        .as_ref()
                        .context("polymorph cast tail absent")?
                        .deadline()
            {
                continue;
            }
            frames.insert(
                id,
                assets
                    .polymorph
                    .as_ref()
                    .context("polymorph caster absent")?
                    .frame(cast.body.peek(now)?, cast.heading)?,
            );
        }
        Ok(frames)
    }
    pub(crate) fn disintegrations(&self, now: Instant) -> Result<Vec<DisintegrateDraw<'_>>> {
        ensure!(now >= self.observed, "spell sample reversed");
        self.effects
            .iter()
            .filter_map(|e| match &e.playback {
                Playback::Disintegrate(p) => p.draw(now.duration_since(e.started)).transpose(),
                _ => None,
            })
            .collect()
    }
    pub(crate) fn polymorph_idle(
        &self,
        scene: &SceneSnapshot,
        now: Instant,
    ) -> Result<Vec<(u64, super::polymorph_assets::PolymorphForm, u8, Duration)>> {
        ensure!(now >= self.observed, "polymorph idle sample reversed");
        let mut poses = Vec::new();
        for (entity, cast) in &self.casts {
            let CasterForm::Polymorph(form) = cast.form else {
                continue;
            };
            let id = (*entity).try_into()?;
            let end = cast
                .tail
                .as_ref()
                .context("polymorph idle origin absent")?
                .deadline();
            if cast.visible && !scene.hidden(id) && now >= end {
                poses.push((id, form, cast.heading, now.duration_since(end)));
            }
        }
        Ok(poses)
    }
    pub(crate) fn draw_projected(
        &self,
        canvas: &mut Canvas,
        projection: ViewProjection,
        assets: &Assets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<()> {
        ensure!(now >= self.observed, "spell sample reversed");
        for e in &self.effects {
            active(cancel)?;
            if let Playback::Arrow { path, .. } = &e.playback {
                assets
                    .light
                    .as_ref()
                    .context("arrow bank absent")?
                    .draw_effect(canvas, path, projection, now.duration_since(e.started))?;
            } else if let Playback::Fire { origin, .. } = &e.playback {
                assets
                    .fire
                    .as_ref()
                    .context("fire bank absent")?
                    .draw_effect(
                        canvas,
                        projection
                            .cell_to_pixel(*origin)
                            .context("fire projection overflow")?,
                        now.duration_since(e.started),
                    )?;
            }
        }
        Ok(())
    }
    pub(crate) fn audio(&mut self, assets: &Assets, now: Instant) -> Option<AudioFrame> {
        assets.sound().map(|bank| AudioFrame {
            bank,
            events: self.audio.snapshot(now),
        })
    }
}
/// Qualified effects run at 24fps; arrow's only cue is launch Sound97 at tick0,
/// and its separately timed impact has no cue. Age each source cue against its
/// original wall timestamp, not the render wakeup; consume stale cues silently.
fn queue_cues(
    log: &mut EventLog,
    cues: &[CrossedCue],
    bank: Arc<SoundBank>,
    started: Instant,
    now: Instant,
) -> Result<()> {
    for (tick, group) in super::world_audio::crossed_audio_groups(cues)? {
        let created = cue_created(started, tick)?;
        ensure!(created <= now, "future spell cue");
        if now.duration_since(created) > MAX_AGE {
            continue;
        }
        ensure!(
            group
                .candidates()
                .iter()
                .all(|id| *id == 0 || bank.get(*id).is_some()),
            "spell sound absent from qualified bank"
        );
        log.push_with_bank(group, created, Some(bank.clone()))?;
    }
    Ok(())
}
fn cue_created(started: Instant, tick: u64) -> Result<Instant> {
    let nanos = u64::try_from((u128::from(tick) * 1_000_000_000).div_ceil(24))?;
    started
        .checked_add(Duration::from_nanos(nanos))
        .context("cue timestamp overflow")
}

#[cfg(test)]
#[path = "battle_spells_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "battle_arrow_tests.rs"]
pub(crate) mod arrow_tests;
