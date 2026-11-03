//! Owned skill77 presentation. Authority supplies the receipt; caster18's Hit
//! launches an independently owned, fixed-cell class1 effect. No target pointer,
//! local HP change, resource I/O in the UI, or replay of retained cast history.
use super::{
    combat_protocol::Frame,
    disintegrate_assets::DisintegrateAssets,
    disintegrate_effect::{Binding, DisintegrateDraw, DisintegrateEffect, Variant},
    disintegrate_protocol as dis,
    light_arrow_effect::LightArrowAssets,
    paced_animation::PacedClock,
    world_audio::AudioSequencer,
    world_map::WorldMap,
    world_protocol::Phase,
    world_scene::{ActorOverride, BaseAppearance},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

const MAX_EFFECTS: usize = 64; // The same bound enforced by the native-depth queue.
const EFFECT_DURATION: Duration = Duration::from_millis(2500);
type Identity = (u64, u64, [u8; 32]);
fn identity(f: &Frame) -> Identity {
    (
        f.world.match_id,
        f.world.generation,
        f.world.manifest.digest,
    )
}
/// Rendering receipts retain target type and life, but no mutable world object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Player {
        entity_id: u64,
        life: i64,
    },
    Npc {
        id: super::npc_protocol::Id,
        life: u64,
    },
}
#[derive(Clone, Copy)]
pub(crate) struct ReceiptHit {
    target: Target,
    caster_life: i64,
    pub(crate) from: super::world_map::WorldCell,
    to: super::world_map::WorldCell,
    critical: bool,
}
#[derive(Clone)]
pub(crate) struct Receipt {
    pub(crate) entity_id: u64,
    pub(crate) revision: i64,
    pub(crate) hit: Option<ReceiptHit>,
}
pub(crate) fn receipts(f: &Frame) -> impl Iterator<Item = Receipt> + '_ {
    f.disintegrations
        .iter()
        .map(|s| Receipt {
            entity_id: s.entity_id,
            revision: s.revision,
            hit: s.hit.as_ref().map(|h| ReceiptHit {
                target: Target::Player {
                    entity_id: h.target_entity_id,
                    life: h.target_life,
                },
                caster_life: h.caster_life,
                from: h.from,
                to: h.to,
                critical: h.critical,
            }),
        })
        .chain(f.npc_disintegrations.iter().map(|s| Receipt {
            entity_id: s.entity_id,
            revision: s.revision,
            hit: s.hit.as_ref().map(|h| ReceiptHit {
                target: Target::Npc {
                    id: h.target,
                    life: h.target_life,
                },
                caster_life: h.caster_life as i64,
                from: h.from,
                to: h.to,
                critical: h.critical,
            }),
        }))
}
pub(crate) fn enabled(f: &Frame) -> bool {
    !f.disintegrations.is_empty() || !f.npc_disintegrations.is_empty()
}
pub(crate) fn revision(f: &Frame, id: u64) -> i64 {
    dis::revision(f, id).max(f.npc_disintegrate(id).map_or(0, |s| s.revision))
}
struct Cast {
    event: Receipt,
    body: PacedClock,
    appearance: BaseAppearance,
    move_sequence: u64,
    launched: bool,
    launch_at: Option<Instant>,
    duration: Duration,
    body_visible: bool,
}
fn pending_current(f: &Frame, npc: Option<&super::npc_protocol::Frame>, cast: &Cast) -> bool {
    let id = cast.event.entity_id;
    let (Some(v), Some(a), Some(hit)) = (f.vital(id), f.world.actor(id), cast.event.hit.as_ref())
    else {
        return false;
    };
    let life = match hit.target {
        Target::Player { .. } => f
            .honor
            .as_ref()
            .map_or(Some(1), |h| h.life(id)?.respawn_count.checked_add(1)),
        Target::Npc { .. } => {
            Some(super::quest_life::life(&f.world, id).map_or(1, |l| l.incarnation()) as i64)
        }
    };
    v.hp > 0
        // Time Attack preserves departed actors in the result roster, while
        // SceneSnapshot removes their drawable bodies. Roster presence alone
        // must not keep a cast override alive in another player's renderer.
        && f.world.time_attack.as_ref().is_none_or(|t| !t.departed(id))
        && life == Some(hit.caster_life)
        && a.sequence == cast.move_sequence
        && a.position == hit.from
        && v.attack_revision
            .max(npc.and_then(|n| super::npc_presentation::player_attack(n, id))
                .map_or(0, |a| a.revision))
            .max(v.death_revision)
            .max(super::quest_life::respawn_revision(f, id))
            .max(super::honor_protocol::heal_revision(f, id))
            .max(f.world.cast(id).map_or(0, |c| c.cast_revision))
            .max(f.damage(id).map_or(0, |c| c.cast_revision as i64))
            .max(super::fire_storm_presentation::revision(f, id))
            .max(f.npc_spell(id).map_or(0, |s| s.revision))
            .max(revision(f, id))
            <= cast.event.revision
}
struct Launch {
    binding: Binding,
    variant: Variant,
    started: Instant,
}
/// Pure event/clock ownership, separated from immutable decoded resources so
/// cancellation, coalescing and life changes can be exercised without I/O.
struct Tracker {
    identity: Identity,
    baseline: BTreeMap<u64, i64>,
    active: BTreeMap<u64, Cast>,
    next_event: u64,
    npc: bool,
}
impl Tracker {
    fn new(f: &Frame) -> Self {
        Self {
            identity: identity(f),
            baseline: receipts(f).map(|s| (s.entity_id, s.revision)).collect(),
            active: BTreeMap::new(),
            next_event: 0,
            npc: !f.npc_disintegrations.is_empty(),
        }
    }
    fn resets(&self, f: &Frame) -> bool {
        self.identity != identity(f) || matches!(f.world.phase, Phase::Preparing | Phase::Aborted)
    }
    #[cfg(test)]
    fn update(
        &mut self,
        f: &Frame,
        now: Instant,
        timing: impl Fn(BaseAppearance) -> Result<(Duration, Duration)>,
    ) -> Result<Vec<Launch>> {
        self.update_atomic(f, None, now, timing)
    }
    fn update_atomic(
        &mut self,
        f: &Frame,
        npc: Option<&super::npc_protocol::Frame>,
        now: Instant,
        timing: impl Fn(BaseAppearance) -> Result<(Duration, Duration)>,
    ) -> Result<Vec<Launch>> {
        ensure!(
            npc.is_none_or(|n| n.combat == *f),
            "Disintegrate NPC snapshot differs"
        );
        if self.resets(f) {
            *self = Self::new(f);
            return Ok(Vec::new());
        }
        ensure!(
            f.disintegrations.is_empty() || f.npc_disintegrations.is_empty(),
            "Disintegrate target domains cannot alias"
        );
        ensure!(
            self.npc == !f.npc_disintegrations.is_empty(),
            "Disintegrate target domain changed"
        );
        ensure!(
            self.baseline.len() == f.disintegrations.len() + f.npc_disintegrations.len(),
            "Disintegrate roster changed"
        );
        for event in receipts(f) {
            let previous = self
                .baseline
                .get_mut(&event.entity_id)
                .context("Disintegrate actor missing")?;
            ensure!(
                event.revision >= *previous,
                "Disintegrate revision regressed"
            );
            if event.revision == *previous {
                continue;
            }
            *previous = event.revision;
            let actor = f
                .world
                .actor(event.entity_id)
                .context("Disintegrate actor absent")?;
            let member = f
                .world
                .member(event.entity_id)
                .context("Disintegrate member absent")?;
            let appearance = BaseAppearance {
                class: u8::try_from(member.class)?,
                sex: member.sex,
            };
            let (delay, duration) = timing(appearance)?;
            ensure!(delay < duration, "Disintegrate caster lacks launch window");
            self.active.insert(
                event.entity_id,
                Cast {
                    event: event.clone(),
                    body: PacedClock::new(now),
                    appearance,
                    move_sequence: actor.sequence,
                    launched: false,
                    // Also supports a qualified first-source Hit at zero.
                    launch_at: Some(now),
                    duration,
                    body_visible: true,
                },
            );
        }
        // A newer hurt pose wins the body, but cannot erase a committed hit.
        // Native 4E0EF0 owns the pending effect at +80 until launch/replacement;
        // a hurt action in 4E45F0 does not destroy that ownership. Keep our
        // bounded, paced launch clock independently of the visible hurt body.
        // Departure, death/life changes, movement and a superseding deliberate
        // action still cancel the pending cast. Already launched FX are owned.
        self.active.retain(|_, cast| pending_current(f, npc, cast));
        for cast in self.active.values_mut() {
            cast.body_visible &= f
                .vital(cast.event.entity_id)
                .is_some_and(|v| v.hurt_revision <= cast.event.revision);
        }
        let mut launches = Vec::new();
        for cast in self.active.values_mut() {
            let id = cast.event.entity_id;
            let base = u64::from(
                f.world
                    .cast(id)
                    .context("Disintegrate pace absent")?
                    .cast_interval_millis,
            ) * 1_000_000;
            cast.body.retime(
                now,
                base,
                super::timed_protocol::effective_nanos(
                    f,
                    id,
                    base,
                    super::timed_protocol::Pace::Spell,
                ),
            )?;
            let (delay, _) = timing(cast.appearance)?;
            let elapsed = cast.body.elapsed(now)?;
            if !cast.launched && elapsed < delay {
                cast.launch_at = Some(cast.body.until(now, delay)?);
            }
            if !cast.launched && elapsed >= delay {
                cast.launched = true;
                // Preserve the pre-Hit wall deadline. A stalled render must not
                // restart a 2.5-second effect (or its sound) when it wakes.
                let started = cast
                    .launch_at
                    .context("Disintegrate launch deadline absent")?;
                if now.saturating_duration_since(started) >= EFFECT_DURATION {
                    continue;
                }
                let hit = cast
                    .event
                    .hit
                    .as_ref()
                    .context("Disintegrate receipt absent")?;
                self.next_event = self
                    .next_event
                    .checked_add(1)
                    .filter(|n| *n <= i64::MAX as u64)
                    .context("Disintegrate event ID exhausted")?;
                launches.push(Launch {
                    binding: Binding {
                        world_id: f.world.match_id,
                        map_digest: f.world.manifest.digest,
                        event_id: self.next_event,
                        start_revision: u64::try_from(cast.event.revision)?,
                        target: hit.to,
                    },
                    variant: if hit.critical {
                        Variant::Critical
                    } else {
                        Variant::Normal
                    },
                    started,
                });
            }
        }
        self.active
            .retain(|_, c| c.body.peek(now).is_ok_and(|elapsed| elapsed < c.duration));
        Ok(launches)
    }
}
struct Effect {
    started: Instant,
    playback: DisintegrateEffect,
}
pub(crate) struct Presentation {
    assets: Arc<DisintegrateAssets>,
    tracker: Tracker,
    effects: Vec<Effect>,
    pub(crate) skipped: bool,
}
pub(crate) fn validate_geometry(f: &Frame, map: &WorldMap) -> Result<()> {
    ensure!(
        f.world.manifest.digest == map.canonical_digest(),
        "Disintegrate map binding changed"
    );
    ensure!(
        f.disintegrations.is_empty() || f.npc_disintegrations.is_empty(),
        "Disintegrate target domains cannot alias"
    );
    for hit in receipts(f).filter_map(|s| s.hit) {
        if let Target::Npc { id, life } = hit.target {
            ensure!(
                life > 0
                    && id.scope.room_id == f.world.room.summary.room_id
                    && id.scope.match_id == f.world.match_id
                    && id.scope.generation == f.world.generation,
                "NPC77 receipt belongs to another encounter"
            );
        }
        ensure!(
            map.standing(hit.from) && map.standing(hit.to),
            "Disintegrate receipt outside standing geometry"
        );
    }
    Ok(())
}
impl Presentation {
    pub(crate) fn load(
        root: &Path,
        f: &Frame,
        map: &WorldMap,
        bodies: &LightArrowAssets,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        validate_geometry(f, map)?;
        for a in &f.world.actors {
            let m = f
                .world
                .member(a.entity_id)
                .context("Disintegrate member absent")?;
            let appearance = BaseAppearance {
                class: u8::try_from(m.class)?,
                sex: m.sex,
            };
            ensure!(
                bodies.cast_launch_delay(appearance)? < bodies.cast_duration(appearance)?,
                "Disintegrate caster18 not qualified"
            );
        }
        Ok(Self {
            assets: Arc::new(DisintegrateAssets::load(root, cancel)?),
            tracker: Tracker::new(f),
            effects: Vec::new(),
            skipped: false,
        })
    }
    pub(crate) fn update(
        &mut self,
        f: &Frame,
        map: &WorldMap,
        bodies: &LightArrowAssets,
        npc: Option<&super::npc_protocol::Frame>,
        now: Instant,
    ) -> Result<()> {
        validate_geometry(f, map)?;
        if self.tracker.resets(f) {
            self.effects.clear();
        }
        self.effects
            .retain(|e| now.saturating_duration_since(e.started) < EFFECT_DURATION);
        let launches = self.tracker.update_atomic(f, npc, now, |a| {
            Ok((bodies.cast_launch_delay(a)?, bodies.cast_duration(a)?))
        })?;
        for launch in launches {
            if self.effects.len() == MAX_EFFECTS {
                self.skipped = true;
                continue;
            }
            self.effects.push(Effect {
                started: launch.started,
                playback: DisintegrateEffect::start(
                    self.assets.clone(),
                    launch.binding,
                    launch.variant,
                )?,
            });
        }
        Ok(())
    }
    pub(crate) fn overrides<'a>(
        &self,
        bodies: &'a LightArrowAssets,
        now: Instant,
    ) -> Result<Vec<ActorOverride<'a>>> {
        let mut draws = Vec::new();
        for cast in self.tracker.active.values() {
            if !cast.body_visible {
                continue;
            }
            let hit = cast
                .event
                .hit
                .as_ref()
                .context("Disintegrate body receipt absent")?;
            let delta = (
                (i64::from(hit.to.x) - i64::from(hit.from.x)).signum() as i32,
                (i64::from(hit.to.y) - i64::from(hit.from.y)).signum() as i32,
            );
            let heading = super::world_map::DIRECTIONS
                .iter()
                .position(|d| *d == delta)
                .context("Disintegrate target shares caster cell")? as u8;
            let v = bodies.caster_frame(cast.appearance, heading, cast.body.peek(now)?)?;
            if !v.finished {
                draws.push(ActorOverride {
                    entity_id: cast.event.entity_id,
                    frame: v.frame,
                    graphics_id: v.graphics_id,
                    source_frame: v.source_frame,
                    sprite_name: v.sprite_name,
                    subframe: v.subframe,
                });
            }
        }
        Ok(draws)
    }
    pub(crate) fn draws(&self, now: Instant) -> Result<Vec<DisintegrateDraw<'_>>> {
        self.effects
            .iter()
            .filter_map(|e| {
                e.playback
                    .draw(now.saturating_duration_since(e.started))
                    .transpose()
            })
            .collect()
    }
    pub(crate) fn audio(&mut self, audio: &mut AudioSequencer, now: Instant) -> Result<()> {
        for effect in &mut self.effects {
            let elapsed = now.saturating_duration_since(effect.started);
            let sample = effect.playback.advance(elapsed)?;
            if elapsed <= Duration::from_millis(500) && !sample.crossed_cues.is_empty() {
                audio.push_preloaded_cues(
                    &sample.crossed_cues,
                    self.assets.sound_bank(),
                    effect.started,
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "disintegrate_presentation_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "npc_disintegrate_presentation_tests.rs"]
pub(crate) mod npc_tests;
