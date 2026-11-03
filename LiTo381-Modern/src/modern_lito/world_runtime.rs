//! Owned world resource/render worker. The UI only submits immutable confirmed
//! frames and polls pointers; no archive I/O, decoding or pixel conversion runs
//! there. Normal shutdown must poll until quiescent. Drop cancels and joins as
//! a last-resort ownership guarantee, not a nonblocking shutdown substitute.
//!
//! At most one active and one retiring worker plus one latest pending input.
//! Filesystem reads remain synchronous but bounded; cancellation is checked
//! between those reads and CPU stages, not advertised as a hard I/O deadline.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, TryLockError,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{ensure, Context, Result};

use super::{
    combat_protocol,
    compositor::Canvas,
    light_arrow_effect::{LightArrowAssets, LightArrowError, LightArrowPath, LightArrowPlayback},
    ranged_effect::{RangedAssets, RangedError, RangedPath},
    resources::SharedMapSectorKey,
    spell_protocol::CastState,
    teleport_effect::{TeleportAssets, TeleportPlayback},
    world_audio::{AudioFrame, AudioSequencer},
    world_map::{PixelPoint, SectorBinding, ViewProjection, WorldCell, WorldMap},
    world_protocol::{self, Frame, Phase},
    world_scene::{
        ActorOverride, ActorPlacement, ActorState, BaseAction, BaseAppearance, CombatAppearance,
        SceneLimitations, SceneLimits, SceneSnapshot, WorldSceneAssets,
    },
};

pub const WIDTH: u32 = 800;
pub const HEIGHT: u32 = 600;
const PIXEL_BYTES: usize = WIDTH as usize * HEIGHT as usize * 4;
const FRAME_INTERVAL: Duration = Duration::from_millis(50);

#[cfg(test)]
#[path = "honor_runtime_tests.rs"]
mod honor_runtime_tests;

#[cfg(test)]
#[path = "fire_storm_runtime_tests.rs"]
mod fire_storm_runtime_tests;

#[cfg(test)]
#[path = "disintegrate_runtime_tests.rs"]
mod disintegrate_runtime_tests;

#[cfg(test)]
#[path = "npc_disintegrate_runtime_tests.rs"]
mod npc_disintegrate_runtime_tests;

#[cfg(test)]
#[path = "npc_fire_storm_runtime_tests.rs"]
mod npc_fire_storm_runtime_tests;

#[cfg(test)]
#[path = "world_runtime_live_skill_tests.rs"]
pub(crate) mod live_skill_tests;
#[cfg(test)]
#[path = "spell_body_arbitration_tests.rs"]
mod spell_body_arbitration_tests;

#[cfg(test)]
#[path = "attr8_runtime_tests.rs"]
mod attr8_runtime_tests;
#[cfg(test)]
#[path = "npc_runtime_tests.rs"]
mod npc_runtime_tests;
#[cfg(test)]
#[path = "quest_runtime_tests.rs"]
mod quest_runtime_tests;
#[cfg(test)]
#[path = "shared_catalog_integration_tests.rs"]
mod shared_catalog_integration_tests;
#[cfg(test)]
#[path = "siege_runtime_tests.rs"]
mod siege_runtime_tests;
#[cfg(test)]
#[path = "time_attack_runtime_tests.rs"]
mod time_attack_runtime_tests;

#[derive(Clone)]
pub struct RenderedWorld {
    pub match_id: u64,
    pub generation: u64,
    pub revision: i64,
    pub selected_entity_id: u64,
    pub manifest_digest: [u8; 32],
    pub width: u32,
    pub height: u32,
    /// Immutable, top-down BGRA. Alpha is opaque, ready for a Win32 DIB blit.
    pub bgra: Arc<[u8]>,
    /// True only after exact manifest/resource validation AND a successful
    /// render. The session still owns when/once to send the matching LOAD ACK.
    pub ready_to_ack: bool,
    pub limitations: SceneLimitations,
    pub presentation_notice: Option<&'static str>,
    /// The same immutable, SHA-qualified geometry used to draw these pixels.
    /// Test renderers may have no geometry; the UI must then disable routing.
    navigation_map: Option<Arc<WorldMap>>,
    /// Tagged with this rendered revision, never inferred from a newer camera.
    pub projection: ViewProjection,
    /// Exact immutable combat/world revision used for these pixels, never a
    /// newer HP overlay on an older actor position. None means unconfirmed.
    pub combat: Option<Arc<combat_protocol::Frame>>,
    /// Retained even in non-combat/race worlds. Presentation must never label
    /// old pixels using a newer room roster or a different source selection.
    pub world: Arc<Frame>,
    pub actors: Arc<[ActorPlacement]>,
    pub objectives: Arc<[super::world_scene::ObjectivePlacement]>,
    pub quests: Arc<[super::quest_presentation::Placement]>,
    pub time_attack_tower: Option<super::time_attack_presentation::Placement>,
    pub siege_towers: Arc<[super::siege_presentation::Placement]>,
    pub npcs: Arc<[super::world_scene::NpcPlacement]>,
    /// Exact NPC authority behind these pixels; never merged into PC vitals.
    pub npc: Option<Arc<super::npc_protocol::Reply>>,
    /// Bounded sound history from these qualified animations; no UI archive I/O.
    pub audio: Option<AudioFrame>,
    /// Exact worker-side software gain. None means attenuation failed and the
    /// unscaled cue history must be consumed silently, never played at full gain.
    pub audio_gain: Option<u8>,
}

impl RenderedWorld {
    pub fn navigation_map(&self) -> Option<Arc<WorldMap>> {
        self.navigation_map.clone()
    }
}
impl std::fmt::Debug for RenderedWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderedWorld")
            .field("match_id", &self.match_id)
            .field("generation", &self.generation)
            .field("revision", &self.revision)
            .field("selected_entity_id", &self.selected_entity_id)
            .field("projection", &self.projection)
            .field("has_navigation_map", &self.navigation_map.is_some())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub enum RuntimeStatus {
    Idle,
    Loading {
        match_id: u64,
        generation: u64,
    },
    Ready(Arc<RenderedWorld>),
    /// Deliberately static; no raw filesystem exceptions/paths reach the UI.
    Failed(&'static str),
    Stopping,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Identity {
    match_id: u64,
    generation: u64,
    selected_entity_id: u64,
    combat_art: bool,
}

struct Input {
    identity: Identity,
    frame: Arc<Frame>,
    received: Instant,
    combat: Option<Arc<combat_protocol::Frame>>,
    npc: Option<NpcInput>,
}
#[derive(Clone)]
struct NpcInput {
    reply: Arc<super::npc_protocol::Reply>,
    plan: Arc<super::npc_presentation::Plan>,
}

impl Input {
    #[cfg(test)]
    fn new(frame: Arc<Frame>, selected_entity_id: u64) -> Result<Self> {
        Self::with_combat(frame, selected_entity_id, None)
    }
    fn with_combat(
        frame: Arc<Frame>,
        selected_entity_id: u64,
        combat: Option<Arc<combat_protocol::Frame>>,
    ) -> Result<Self> {
        Self::with_npc(frame, selected_entity_id, combat, None)
    }
    fn with_npc(
        frame: Arc<Frame>,
        selected_entity_id: u64,
        combat: Option<Arc<combat_protocol::Frame>>,
        npc: Option<NpcInput>,
    ) -> Result<Self> {
        // SharedRenderer additionally qualifies the siege cutset and all three
        // shared towers before its first pixels can authorize a LOAD ACK.
        ensure!(
            !frame.actors.is_empty() && frame.actors.len() <= super::world_scene::MAX_ACTORS,
            "world actor presentation count exceeds its bound"
        );
        ensure!(
            matches!(
                frame.phase,
                Phase::Loading | Phase::Countdown | Phase::Active | Phase::Finished
            ),
            "world presentation requires a qualified loading/active frame"
        );
        let selected = frame
            .actors
            .iter()
            .find(|a| a.entity_id == selected_entity_id)
            .ok_or_else(|| anyhow::anyhow!("selected world entity is absent"))?;
        frame.validate(
            selected
                .identity()
                .character()
                .ok_or_else(|| anyhow::anyhow!("selected actor is not a human"))?,
        )?;
        ensure!(
            frame
                .descriptor
                .as_ref()
                .is_none_or(|descriptor| descriptor.combat() == combat.is_some()),
            "match descriptor requires the exact world/combat presentation contract"
        );
        if let Some(value) = &combat {
            value.validate()?;
            ensure!(
                value.world == *frame && value.vitals.len() == frame.actors.len(),
                "combat and world rendering revisions differ"
            );
            for (actor, vital) in frame.actors.iter().zip(&value.vitals) {
                ensure!(
                    actor.entity_id == vital.entity_id
                        && vital.max_hp > 0
                        && vital.hp >= 0
                        && vital.hp <= vital.max_hp,
                    "invalid render combat identity/vitals"
                );
            }
        }
        ensure!(
            frame
                .descriptor
                .is_some_and(|d| d.has(world_protocol::MatchDescriptor::NPC))
                == npc.is_some(),
            "NPC descriptor requires its snapshot AND immutable resource plan"
        );
        if let Some(npc) = &npc {
            npc.plan.validate(&npc.reply)?;
            ensure!(
                npc.reply
                    .frame
                    .as_ref()
                    .is_some_and(|n| Some(&n.combat) == combat.as_deref())
                    && selected.matches_character(npc.reply.echo.context.key),
                "NPC render context/revision mismatch"
            );
        }
        ensure!(
            frame
                .time_attack
                .as_ref()
                .is_none_or(|d| !d.departed(selected_entity_id)),
            "departed cooperative owner cannot render an active surface"
        );
        // Room map is a logical mode/map option. A server-owned binding may
        // deliberately map it to a different physical shared manifest map.
        Ok(Self {
            identity: Identity {
                match_id: frame.match_id,
                generation: frame.generation,
                selected_entity_id,
                combat_art: combat.is_some(),
            },
            frame,
            received: Instant::now(),
            combat,
            npc,
        })
    }
}

/// Redundant presentation guard. Transport remains the wire authority, but a
/// mutable public Frame cannot silently change the loaded resource binding.
fn follows(old: &Input, new: &Input) -> bool {
    if match (&old.npc, &new.npc) {
        (None, None) => false,
        (Some(a), Some(b)) => {
            a.plan != b.plan || !super::npc_protocol::follows(Some(&a.reply), &b.reply)
        }
        _ => true,
    } {
        return false;
    }
    if old.identity != new.identity
        || old.frame.manifest != new.frame.manifest
        || old.frame.room != new.frame.room
        || old.frame.descriptor != new.frame.descriptor
        || old.frame.audience_lane != new.frame.audience_lane
        || old.frame.actors.len() != new.frame.actors.len()
        || new.frame.revision < old.frame.revision
        || !super::spell_protocol::casts_follow(&old.frame, &new.frame)
        || !super::race_protocol::follows(&old.frame, &new.frame)
        || !super::siege_protocol::follows(&old.frame, &new.frame)
    {
        return false;
    }
    if old.frame.revision == new.frame.revision {
        return old.frame == new.frame && old.combat == new.combat;
    }
    if !world_protocol::terminal_follows(&old.frame, &new.frame) {
        return false;
    }
    if let (Some(a), Some(b)) = (&old.combat, &new.combat) {
        if !combat_protocol::vitals_follow(a, b) {
            return false;
        }
    }
    old.frame
        .actors
        .iter()
        .zip(&new.frame.actors)
        .all(|(a, b)| {
            a.entity_id == b.entity_id
                && a.identity() == b.identity()
                && b.sequence >= a.sequence
                && (!a.loaded || b.loaded)
                && a.step_interval_nanos == b.step_interval_nanos
                && (a.sequence != b.sequence
                    || a.heading == b.heading
                        && if super::siege_protocol::respawn_is_latest_relocation(
                            &old.frame,
                            &new.frame,
                            a.entity_id,
                        ) {
                            true // Verified camp, life revision and HP ledger above.
                        } else if super::spell_protocol::teleported(
                            &old.frame,
                            &new.frame,
                            a.entity_id,
                        ) {
                            new.frame
                                .cast(b.entity_id)
                                .is_some_and(|cast| b.position == cast.to)
                        } else {
                            a.position == b.position
                        })
        })
}

/// First publication must prove the latest qualified input. After that, slow
/// rendering must not starve behind a continuously advancing authority stream.
/// Publish only monotonic completed snapshots from this immutable resource
/// binding; pixels, hit targets, combat and NPC authority stay tagged together.
/// Equal revisions are allowed so idle animation continues without new packets.
fn publishable(image: &RenderedWorld, desired: &Input, previous: Option<&RenderedWorld>) -> bool {
    image.match_id == desired.identity.match_id
        && image.generation == desired.identity.generation
        && image.selected_entity_id == desired.identity.selected_entity_id
        && image.manifest_digest == desired.frame.manifest.digest
        && image.combat.is_some() == desired.combat.is_some()
        && image.npc.is_some() == desired.npc.is_some()
        && image.revision <= desired.frame.revision
        && previous.map_or(image.revision == desired.frame.revision, |old| {
            old.match_id == image.match_id
                && old.generation == image.generation
                && old.selected_entity_id == image.selected_entity_id
                && old.manifest_digest == image.manifest_digest
                && image.revision >= old.revision
        })
}

/// The newest server event wins; death is terminal. The resource loader must
/// qualify the exact weapon stance before rendering or acknowledging LOADED.
fn combat_pose(
    before: &combat_protocol::Vital,
    after: &combat_protocol::Vital,
) -> Option<BaseAction> {
    if after.death_revision > before.death_revision && after.hp == 0 {
        return Some(BaseAction::Death);
    }
    let attacked = after.attack_revision > before.attack_revision;
    let hurt = after.hurt_revision > before.hurt_revision;
    if !attacked && !hurt {
        return None;
    }
    Some(
        if hurt && (!attacked || after.hurt_revision >= after.attack_revision) {
            BaseAction::Damage
        } else {
            BaseAction::Attack
        },
    )
}

fn facing(from: super::world_map::WorldCell, target: super::world_map::WorldCell) -> Result<u8> {
    let delta = (
        (i64::from(target.x) - i64::from(from.x)).signum() as i32,
        (i64::from(target.y) - i64::from(from.y)).signum() as i32,
    );
    super::world_map::DIRECTIONS
        .iter()
        .position(|d| *d == delta)
        .map(|i| i as u8)
        .context("attack target shares actor cell")
}

fn draw_health_bars(
    canvas: &mut Canvas,
    placements: &[ActorPlacement],
    vitals: &[combat_protocol::Vital],
) -> Result<()> {
    for placement in placements {
        let vital = vitals
            .iter()
            .find(|v| v.entity_id == placement.entity_id)
            .ok_or_else(|| anyhow::anyhow!("health bar actor absent"))?;
        draw_compact_health_bar(
            canvas,
            vital.hp,
            vital.max_hp,
            placement.anchor,
            placement.top_left,
        )?;
    }
    Ok(())
}
fn draw_compact_health_bar(
    canvas: &mut Canvas,
    hp: i64,
    max_hp: i64,
    anchor: PixelPoint,
    top_left: PixelPoint,
) -> Result<()> {
    ensure!(
        max_hp > 0 && hp >= 0 && hp <= max_hp,
        "invalid health bar values"
    );
    let width = (u128::try_from(hp)? * 48 / u128::try_from(max_hp)?) as usize;
    let mut rgba = [0u8; 50 * 5 * 4];
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[28, 28, 28, 255]);
    }
    for y in 1..4 {
        for x in 1..=48 {
            let i = (y * 50 + x) * 4;
            rgba[i..i + 4].copy_from_slice(if x <= width {
                &[35, 210, 75, 255]
            } else {
                &[110, 30, 30, 255]
            });
        }
    }
    let x = anchor.x.checked_sub(25).context("health bar X overflow")?;
    let y = top_left.y.checked_sub(7).context("health bar Y overflow")?;
    canvas.blit_rgba(&rgba, 50, 5, x, y)?;
    Ok(())
}
fn draw_objective_health_bar(
    canvas: &mut Canvas,
    placement: &ActorPlacement,
    o: &super::honor_protocol::Objective,
) -> Result<()> {
    ensure!(
        o.max_hp > 0 && (0..=o.max_hp).contains(&o.hp),
        "invalid objective HP"
    );
    let filled = (i128::from(o.hp) * 96 / i128::from(o.max_hp)) as usize;
    let mut rgba = [0u8; 98 * 7 * 4];
    for y in 0..7 {
        for x in 0..98 {
            let at = (y * 98 + x) * 4;
            let color = if y == 0 || y == 6 || x == 0 || x == 97 {
                [24, 24, 24, 255]
            } else if x <= filled {
                if o.team == 0 {
                    [60, 145, 240, 255]
                } else {
                    [230, 95, 65, 255]
                }
            } else {
                [80, 30, 30, 255]
            };
            rgba[at..at + 4].copy_from_slice(&color);
        }
    }
    canvas.blit_rgba(
        &rgba,
        98,
        7,
        placement
            .anchor
            .x
            .checked_sub(49)
            .context("tower bar X overflow")?,
        placement
            .top_left
            .y
            .checked_sub(9)
            .context("tower bar Y overflow")?,
    )?;
    Ok(())
}

struct ActorTime {
    action_started: Instant,
    body: super::paced_animation::PacedClock,
    walk_until: Option<Instant>,
    pose: Option<BaseAction>,
    combat_heading: Option<u8>,
}

/// Repeated polling of the same revision never restarts animation. A confirmed
/// changed position starts one bounded walking interval, with no interpolation
/// or speculative position. A skipped network update never queues catch-up steps.
struct Timeline {
    input: Arc<Input>,
    actors: BTreeMap<u64, ActorTime>,
}
impl Timeline {
    fn new(input: Arc<Input>) -> Self {
        let actors = input
            .frame
            .actors
            .iter()
            .map(|a| {
                (
                    a.entity_id,
                    ActorTime {
                        action_started: input.received,
                        body: super::paced_animation::PacedClock::new(input.received),
                        walk_until: None,
                        pose: input
                            .combat
                            .as_ref()
                            .and_then(|f| f.vitals.iter().find(|v| v.entity_id == a.entity_id))
                            .filter(|v| v.death_revision > 0 && v.hp == 0)
                            .map(|_| BaseAction::Death),
                        combat_heading: None,
                    },
                )
            })
            .collect();
        Self { input, actors }
    }

    fn update(&mut self, input: Arc<Input>) -> Result<()> {
        ensure!(
            follows(&self.input, &input),
            "world presentation frame changed binding/order"
        );
        if self.input.frame.revision == input.frame.revision {
            return Ok(());
        }
        for (old, new) in self.input.frame.actors.iter().zip(&input.frame.actors) {
            let clock = self.actors.get_mut(&new.entity_id).unwrap();
            if let Some(end) = clock.walk_until.filter(|end| *end <= input.received) {
                clock.walk_until = None;
                clock.action_started = end;
                clock.body.restart(end);
                clock.body.retime(end, 1, 1)?;
            }
            // Accrue the old authority speed first. The new state only changes
            // future body time, and never changes an event/animation identity.
            let action = clock.pose.unwrap_or(if clock.walk_until.is_some() {
                BaseAction::Walk
            } else {
                BaseAction::Idle
            });
            let (base, effective) = body_pace(input.combat.as_deref(), new, action);
            clock.body.retime(input.received, base, effective)?;
            if clock.walk_until.is_some() {
                clock.walk_until = Some(clock.body.until(
                    input.received,
                    Duration::from_nanos(new.step_interval_nanos),
                )?);
            }
            let teleported =
                super::spell_protocol::teleported(&self.input.frame, &input.frame, new.entity_id)
                    || super::siege_presentation::portalled(
                        &self.input.frame,
                        &input.frame,
                        new.entity_id,
                    );
            let damage_cast = self
                .input
                .combat
                .as_ref()
                .and_then(|c| c.damage(new.entity_id))
                .zip(input.combat.as_ref().and_then(|c| c.damage(new.entity_id)))
                .is_some_and(|(a, b)| b.cast_revision > a.cast_revision)
                || self
                    .input
                    .combat
                    .as_ref()
                    .and_then(|c| c.npc_spell(new.entity_id))
                    .zip(
                        input
                            .combat
                            .as_ref()
                            .and_then(|c| c.npc_spell(new.entity_id)),
                    )
                    .is_some_and(|(a, b)| b.revision > a.revision);
            let (healed, respawned) = self
                .input
                .combat
                .as_ref()
                .zip(input.combat.as_ref())
                .map_or((false, false), |(a, b)| {
                    (
                        super::honor_protocol::heal_revision(b, new.entity_id)
                            > super::honor_protocol::heal_revision(a, new.entity_id),
                        super::quest_life::respawned(a, b, new.entity_id),
                    )
                });
            let fire_cast = self
                .input
                .combat
                .as_ref()
                .zip(input.combat.as_ref())
                .is_some_and(|(a, b)| {
                    super::fire_storm_presentation::revision(b, new.entity_id)
                        > super::fire_storm_presentation::revision(a, new.entity_id)
                });
            let disintegrate_cast = self
                .input
                .combat
                .as_ref()
                .zip(input.combat.as_ref())
                .is_some_and(|(a, b)| {
                    super::disintegrate_presentation::revision(b, new.entity_id)
                        > super::disintegrate_presentation::revision(a, new.entity_id)
                });
            if old.position != new.position
                || teleported
                || damage_cast
                || healed
                || respawned
                || fire_cast
                || disintegrate_cast
            {
                clock.pose = None;
                clock.combat_heading = None;
                clock.action_started = input.received;
                clock.body.restart(input.received);
                clock.walk_until = if teleported
                    || damage_cast
                    || healed
                    || respawned
                    || fire_cast
                    || disintegrate_cast
                {
                    None
                } else {
                    Some(
                        input
                            .received
                            .checked_add(Duration::from_nanos(
                                body_pace(input.combat.as_deref(), new, BaseAction::Walk).1,
                            ))
                            .ok_or_else(|| anyhow::anyhow!("world animation time overflow"))?,
                    )
                };
            }
            if let (Some(before), Some(after)) = (&self.input.combat, &input.combat) {
                let a = before
                    .vitals
                    .iter()
                    .find(|v| v.entity_id == new.entity_id)
                    .unwrap();
                let b = after
                    .vitals
                    .iter()
                    .find(|v| v.entity_id == new.entity_id)
                    .unwrap();
                let cast_revision = input
                    .frame
                    .cast(new.entity_id)
                    .map_or(0, |cast| cast.cast_revision)
                    .max(
                        after
                            .damage(new.entity_id)
                            .map_or(0, |cast| cast.cast_revision as i64),
                    )
                    .max(after.npc_spell(new.entity_id).map_or(0, |c| c.revision))
                    .max(super::honor_protocol::heal_revision(after, new.entity_id))
                    .max(super::fire_storm_presentation::revision(
                        after,
                        new.entity_id,
                    ))
                    .max(super::disintegrate_presentation::revision(
                        after,
                        new.entity_id,
                    ))
                    .max(super::quest_life::respawn_revision(after, new.entity_id));
                // A poll can coalesce an attack/hurt followed by a later cast.
                // The later one-shot wins, including when multiple teleports
                // return to the original cell. Don't play the stale melee cue.
                let npc_attack = input
                    .npc
                    .as_ref()
                    .and_then(|n| n.reply.frame.as_ref())
                    .and_then(|n| super::npc_presentation::player_attack(n, new.entity_id))
                    .or_else(|| {
                        super::siege_presentation::player_attack(&input.frame, new.entity_id)
                    });
                let npc_attack_revision = npc_attack.map_or(0, |e| e.revision);
                let new_npc_attack = npc_attack_revision > self.input.frame.revision;
                let pose = if new_npc_attack && b.hp > 0 && npc_attack_revision > b.hurt_revision {
                    Some(BaseAction::Attack)
                } else {
                    combat_pose(a, b)
                }
                .filter(|pose| {
                    let event = match pose {
                        BaseAction::Attack => b.attack_revision.max(npc_attack_revision),
                        BaseAction::Damage => b.hurt_revision,
                        BaseAction::Death => b.death_revision,
                        _ => 0,
                    };
                    event >= cast_revision
                        && !(new_npc_attack
                            && old.position != new.position
                            && input.frame.revision > event)
                });
                if let Some(pose) = pose {
                    clock.pose = Some(pose);
                    clock.action_started = input.received;
                    clock.body.restart(input.received);
                    clock.walk_until = None;
                    if pose == BaseAction::Attack {
                        if new_npc_attack {
                            // A coalesced snapshot can retire a killed corpse.
                            // Keep confirmed actor heading when no current target
                            // remains; never turn a NPC serial into a player ID.
                            clock.combat_heading = npc_attack
                                .and_then(|e| e.cell)
                                .map(|cell| facing(new.position, cell))
                                .transpose()?;
                        } else {
                            let target = super::honor_protocol::target_position(
                                after,
                                b.last_target_entity_id,
                            )
                            .context("combat target absent")?;
                            clock.combat_heading = Some(facing(new.position, target)?);
                        }
                    }
                }
                if b.hp == 0 && b.death_revision > 0 {
                    clock.pose = Some(BaseAction::Death);
                    clock.walk_until = None;
                }
            }
            let action = clock.pose.unwrap_or(if clock.walk_until.is_some() {
                BaseAction::Walk
            } else {
                BaseAction::Idle
            });
            let (base, effective) = body_pace(input.combat.as_deref(), new, action);
            clock.body.retime(input.received, base, effective)?;
        }
        self.input = input;
        Ok(())
    }

    fn snapshot(&mut self, now: Instant) -> Result<SceneSnapshot> {
        let mut actors = Vec::with_capacity(self.input.frame.actors.len());
        for actor in &self.input.frame.actors {
            if self
                .input
                .frame
                .time_attack
                .as_ref()
                .is_some_and(|d| d.departed(actor.entity_id))
            {
                continue;
            }
            let member = self
                .input
                .frame
                .member(actor.entity_id)
                .context("visible actor member absent")?;
            let clock = self.actors.get_mut(&actor.entity_id).unwrap();
            if let Some(end) = clock.walk_until {
                if now >= end {
                    clock.walk_until = None;
                    clock.action_started = end;
                    clock.body.restart(end);
                    clock.body.retime(end, 1, 1)?;
                }
            }
            actors.push(ActorState {
                kind: actor.kind,
                entity_id: actor.entity_id,
                source_id: actor.key.source_id,
                object_id: actor.key.object_id,
                appearance: BaseAppearance {
                    class: u8::try_from(member.class)?,
                    sex: member.sex,
                },
                weapon_type: self
                    .input
                    .combat
                    .as_ref()
                    .and_then(|combat| {
                        combat
                            .vitals
                            .iter()
                            .find(|v| v.entity_id == actor.entity_id)
                    })
                    .map_or(0, |v| v.weapon_type),
                cell: actor.position,
                heading: clock.combat_heading.unwrap_or(actor.heading),
                action: if let Some(pose) = clock.pose {
                    pose
                } else if clock.walk_until.is_some() {
                    BaseAction::Walk
                } else {
                    BaseAction::Idle
                },
                action_elapsed: if clock.pose == Some(BaseAction::Attack)
                    || clock.walk_until.is_some()
                {
                    clock.body.elapsed(now)?
                } else {
                    now.saturating_duration_since(clock.action_started)
                },
            });
        }
        if let Some(race) = &self.input.frame.race {
            let runners = self
                .input
                .frame
                .actors
                .iter()
                .map(|actor| {
                    let runner = race.runner(actor.entity_id).context("race runner absent")?;
                    Ok(super::world_scene::PetRunner {
                        entity_id: actor.entity_id,
                        appearance: super::pet_animation::PetAppearance::new(u32::try_from(
                            runner.base_graphics_id,
                        )?)?,
                        sequence: actor.sequence,
                        step_interval_nanos: actor.step_interval_nanos,
                        competing: runner.competing(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            return SceneSnapshot::new_pet(
                self.input.frame.match_id,
                self.input.frame.revision as u64,
                self.input.frame.manifest.digest,
                self.input.identity.selected_entity_id,
                actors,
                super::world_scene::PetScene {
                    generation: self.input.frame.generation,
                    course: race.course_sha256,
                    runners,
                },
            );
        }
        SceneSnapshot::new(
            self.input.frame.match_id,
            self.input.frame.revision as u64,
            self.input.frame.manifest.digest,
            self.input.identity.selected_entity_id,
            actors,
        )
    }
}

fn body_pace(
    combat: Option<&combat_protocol::Frame>,
    actor: &world_protocol::Actor,
    action: BaseAction,
) -> (u64, u64) {
    let Some(combat) = combat else {
        // The pair is also the authoritative walk interval, not merely a
        // playback ratio. Movement-only worlds still walk for one full step.
        return if action == BaseAction::Walk {
            (actor.step_interval_nanos, actor.step_interval_nanos)
        } else {
            (1, 1)
        };
    };
    let (base, kind) = match action {
        BaseAction::Walk => (actor.step_interval_nanos, super::timed_protocol::Pace::Move),
        BaseAction::Attack => (
            combat
                .vital(actor.entity_id)
                .map_or(1, |v| v.attack_interval_nanos),
            super::timed_protocol::Pace::Attack,
        ),
        _ => return (1, 1),
    };
    (
        base,
        super::timed_protocol::effective_nanos(combat, actor.entity_id, base, kind),
    )
}

struct Pixels {
    bgra: Arc<[u8]>,
    limitations: SceneLimitations,
    actors: Vec<ActorPlacement>,
    objectives: Vec<super::world_scene::ObjectivePlacement>,
    quests: Vec<super::quest_presentation::Placement>,
    time_attack_tower: Option<super::time_attack_presentation::Placement>,
    siege_towers: Vec<super::siege_presentation::Placement>,
    npcs: Vec<super::world_scene::NpcPlacement>,
    audio: Option<AudioFrame>,
    presentation_notice: Option<&'static str>,
}
trait Renderer: Send {
    fn render(
        &mut self,
        snapshot: &SceneSnapshot,
        combat: Option<&combat_protocol::Frame>,
        cancel: &AtomicBool,
    ) -> Result<Pixels>;
    fn render_npcs(
        &mut self,
        snapshot: &SceneSnapshot,
        combat: Option<&combat_protocol::Frame>,
        npc: Option<&NpcInput>,
        cancel: &AtomicBool,
    ) -> Result<Pixels> {
        ensure!(npc.is_none(), "renderer has not qualified NPC resources");
        self.render(snapshot, combat, cancel)
    }
    fn navigation_map(&self) -> Option<Arc<WorldMap>> {
        None
    }
}
type Loader = Arc<
    dyn Fn(
            &Path,
            &Frame,
            Option<&combat_protocol::Frame>,
            Option<&NpcInput>,
            &AtomicBool,
        ) -> Result<Box<dyn Renderer>>
        + Send
        + Sync
        + 'static,
>;

struct SharedRenderer {
    assets: WorldSceneAssets,
    roofs: super::world_roofs::Animation,
    map: Arc<WorldMap>,
    canvas: Canvas,
    bgra: Vec<u8>,
    audio: Option<AudioSequencer>,
    teleport: Option<TeleportPresentation>,
    light_arrow: Option<LightArrowPresentation>,
    healing: Option<HealingPresentation>,
    timed: Option<TimedPresentation>,
    ranged: Option<RangedPresentation>,
    honor: Option<HonorPresentation>,
    quest: Option<super::quest_presentation::Presentation>,
    time_attack: Option<super::time_attack_presentation::Presentation>,
    siege: Option<super::siege_presentation::Presentation>,
    fire_storm: Option<FirePresentation>,
    disintegrate: Option<super::disintegrate_presentation::Presentation>,
    pet: Option<PetPresentation>,
    npc: Option<super::npc_presentation::Presentation>,
}

#[derive(Clone)]
struct PetMotion {
    playback: super::fixed_walk::WalkPlayback,
    from: WorldCell,
    to: WorldCell,
    started: Instant,
    interval: Duration,
    unpaced_cycle: Duration,
    camera_offset: (i32, i32),
}
#[derive(Clone)]
struct PetVisual {
    sequence: u64,
    position: WorldCell,
    appearance: super::pet_animation::PetAppearance,
    competing: bool,
    walk: Option<PetMotion>,
    idle: super::pet_animation::PetIdlePlayback,
    idle_started: Instant,
    idle_marker: Option<(u128, usize)>,
    last_sample: Instant,
}
pub(crate) struct PetPose {
    pub(crate) entity_id: u64,
    pub(crate) appearance: super::pet_animation::PetAppearance,
    pub(crate) action: super::pet_animation::PetAction,
    pub(crate) heading: u8,
    pub(crate) source: usize,
    pub(crate) anchor: WorldCell,
}
#[derive(Clone)]
pub(crate) struct PetPresentation {
    pub(crate) assets: Arc<super::pet_animation::PetAssets>,
    identity: (u64, u64, [u8; 32]),
    forms: BTreeMap<u64, super::pet_animation::PetAppearance>,
    actors: BTreeMap<u64, PetVisual>,
    sequence: u64,
    // Actor ownership lets MOVE/finish/DNF retire queued old idle sounds.
    sounds: std::collections::VecDeque<(u64, super::world_audio::SoundEvent)>,
}

/// Explicit modern pacing: one native base cycle fits one authority interval.
/// The literal camera pulses are used only by the selected actor's camera;
/// body and shadow retain their original SPR offsets without interpolation.
fn pet_walk_elapsed(elapsed: Duration, interval: Duration, cycle: Duration) -> Result<Duration> {
    ensure!(
        !interval.is_zero() && !cycle.is_zero(),
        "invalid pet walk interval"
    );
    let nanos = elapsed
        .min(interval)
        .as_nanos()
        .checked_mul(cycle.as_nanos())
        .context("pet paced time overflow")?
        / interval.as_nanos();
    Ok(Duration::from_nanos(u64::try_from(nanos)?))
}
fn pet_camera_projection(from: WorldCell, offset: (i32, i32)) -> Result<ViewProjection> {
    Ok(ViewProjection {
        anchor_cell: from,
        anchor_pixel: PixelPoint {
            x: ((WIDTH / 2) as i32)
                .checked_sub(offset.0)
                .context("pet camera X overflow")?,
            y: ((HEIGHT / 2) as i32)
                .checked_sub(offset.1)
                .context("pet camera Y overflow")?,
        },
    })
}
impl PetPresentation {
    /// Shared-resource presentation only. Both wire paths can supply their
    /// already-validated scene directly; this does not construct a legacy Frame
    /// or confer a load/command/settlement permission.
    pub(crate) fn from_scene(
        assets: super::pet_animation::PetAssets,
        snapshot: &SceneSnapshot,
    ) -> Result<Self> {
        let race = snapshot.pet().context("pet scene absent")?;
        Ok(Self {
            assets: Arc::new(assets),
            identity: (snapshot.world_id(), race.generation, race.course),
            forms: race
                .runners
                .iter()
                .map(|r| (r.entity_id, r.appearance))
                .collect(),
            actors: BTreeMap::new(),
            sequence: 0,
            sounds: std::collections::VecDeque::new(),
        })
    }
    pub(crate) fn update(
        &mut self,
        snapshot: &SceneSnapshot,
        map: &WorldMap,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<(Vec<PetPose>, Option<ViewProjection>, AudioFrame)> {
        let race = snapshot
            .pet()
            .context("pet renderer requires race metadata")?;
        ensure!(
            self.identity == (snapshot.world_id(), race.generation, race.course)
                && race.runners.len() == self.forms.len(),
            "pet resource identity/course changed"
        );
        let mut poses = Vec::new();
        let mut camera = None;
        for runner in &race.runners {
            ensure!(
                self.forms.get(&runner.entity_id) == Some(&runner.appearance),
                "pet form changed"
            );
            let actor = snapshot
                .actors()
                .iter()
                .find(|a| a.entity_id == runner.entity_id)
                .context("pet actor absent")?;
            ensure!(map.standing(actor.cell), "pet position outside geometry");
            if !self.actors.contains_key(&actor.entity_id) {
                self.actors.insert(
                    actor.entity_id,
                    PetVisual {
                        sequence: runner.sequence,
                        position: actor.cell,
                        appearance: runner.appearance,
                        competing: runner.competing,
                        walk: None,
                        idle: self
                            .assets
                            .start_remote_idle(runner.appearance, actor.heading)?,
                        idle_started: now,
                        idle_marker: None,
                        last_sample: now,
                    },
                );
            }
            let visual = self.actors.get_mut(&actor.entity_id).unwrap();
            ensure!(
                now >= visual.last_sample
                    && runner.sequence >= visual.sequence
                    && visual.appearance == runner.appearance,
                "pet presentation order changed"
            );
            let changed = runner.sequence != visual.sequence;
            if changed || !runner.competing {
                self.sounds.retain(|(id, _)| *id != actor.entity_id);
                let single_step = runner.competing
                    && visual.competing
                    && visual.sequence.checked_add(1) == Some(runner.sequence)
                    && map.can_step(visual.position, actor.cell)
                    && facing(visual.position, actor.cell)? == actor.heading;
                visual.walk = if single_step {
                    Some(PetMotion {
                        playback: self.assets.start_walk(runner.appearance, actor.heading)?,
                        from: visual.position,
                        to: actor.cell,
                        started: now,
                        interval: Duration::from_nanos(runner.step_interval_nanos),
                        unpaced_cycle: self
                            .assets
                            .action(runner.appearance, super::pet_animation::PetAction::Walk)?
                            .unpaced_cycle_duration(),
                        camera_offset: (0, 0),
                    })
                } else {
                    None
                };
                visual.idle = self
                    .assets
                    .start_remote_idle(runner.appearance, actor.heading)?;
                visual.idle_started = now;
                visual.idle_marker = None;
                visual.sequence = runner.sequence;
                visual.position = actor.cell;
                visual.competing = runner.competing;
            }
            if !runner.competing {
                visual.last_sample = now;
                continue;
            }
            let mut action = super::pet_animation::PetAction::Walk;
            let mut source = 0;
            let mut anchor = actor.cell;
            if let Some(walk) = &mut visual.walk {
                let elapsed = now.duration_since(walk.started);
                let sample = walk.playback.advance(
                    pet_walk_elapsed(elapsed, walk.interval, walk.unpaced_cycle)?,
                    cancel,
                )?;
                for pulse in sample.local_camera_pulses {
                    walk.camera_offset.0 = walk
                        .camera_offset
                        .0
                        .checked_add(pulse.displacement.0)
                        .context("pet camera X overflow")?;
                    walk.camera_offset.1 = walk
                        .camera_offset
                        .1
                        .checked_add(pulse.displacement.1)
                        .context("pet camera Y overflow")?;
                }
                source = sample.source_frame;
                anchor = if sample.terminal { walk.to } else { walk.from };
                if actor.entity_id == snapshot.selected_entity_id() && !sample.terminal {
                    camera = Some(pet_camera_projection(walk.from, walk.camera_offset)?);
                }
                if sample.finished {
                    visual.idle_started = walk
                        .started
                        .checked_add(walk.interval)
                        .context("pet idle deadline overflow")?;
                    visual.walk = None;
                }
            }
            if visual.walk.is_none() {
                // Explicit modern stopped-pet policy uses the proven remote
                // 584 ms idle schedule, not an assertion about native mouse-up.
                let sample = visual
                    .idle
                    .advance(now.duration_since(visual.idle_started), cancel)?;
                action = sample.action;
                source = sample.source_frame;
                let marker = sample.cycle.map(|cycle| (cycle, source));
                // Deliver only a newly observed current frame, never stale
                // crossed-cycle history after a slow render or resumed worker.
                if marker.is_some()
                    && marker != visual.idle_marker
                    && now.duration_since(visual.last_sample) <= Duration::from_millis(500)
                {
                    if let Some(group) = self
                        .assets
                        .action(runner.appearance, action)?
                        .sound_group(source)?
                    {
                        self.sequence = self
                            .sequence
                            .checked_add(1)
                            .context("pet audio sequence exhausted")?;
                        while self.sounds.len() >= 256 {
                            self.sounds.pop_front();
                        }
                        self.sounds.push_back((
                            actor.entity_id,
                            super::world_audio::SoundEvent {
                                sequence: self.sequence,
                                created: now,
                                group: group.clone(),
                                bank: None,
                            },
                        ));
                    }
                }
                visual.idle_marker = marker;
            }
            visual.last_sample = now;
            poses.push(PetPose {
                entity_id: actor.entity_id,
                appearance: runner.appearance,
                action,
                heading: actor.heading,
                source,
                anchor,
            });
        }
        self.sounds.retain(|(_, e)| {
            now.saturating_duration_since(e.created) <= Duration::from_millis(500)
        });
        let audio = AudioFrame {
            bank: self.assets.sound_bank(),
            events: self
                .sounds
                .iter()
                .map(|(_, e)| e.clone())
                .collect::<Vec<_>>()
                .into(),
        };
        Ok((poses, camera, audio))
    }
}
struct FireVisual {
    event: super::fire_storm_presentation::Event,
    body: super::paced_animation::PacedClock,
    appearance: BaseAppearance,
    move_sequence: u64,
    launched: bool,
    launch_at: Option<Instant>,
    body_visible: bool,
}
struct FireEffect {
    position: WorldCell,
    started: Instant,
    playback: super::fire_storm_effect::FireStormPlayback,
    finished: bool,
}
struct FirePresentation {
    assets: super::fire_storm_effect::FireStormAssets,
    identity: (u64, u64),
    baseline: BTreeMap<u64, i64>,
    active: BTreeMap<u64, FireVisual>,
    effects: Vec<FireEffect>,
    skipped: bool,
}
fn fire_body_current(frame: &combat_protocol::Frame, cast: &FireVisual) -> bool {
    let id = cast.event.entity_id;
    let Some(v) = frame.vital(id) else {
        return false;
    };
    let Some(a) = frame.world.actors.iter().find(|a| a.entity_id == id) else {
        return false;
    };
    v.hp > 0
        && frame
            .world
            .time_attack
            .as_ref()
            .is_none_or(|t| !t.departed(id))
        && cast.event.caster_life.is_none_or(|life| {
            super::quest_life::life(&frame.world, id).map_or(1, |l| l.incarnation()) == life
        })
        && a.sequence == cast.move_sequence
        && a.position == cast.event.from
        && v.attack_revision
            .max(v.death_revision)
            .max(super::quest_life::respawn_revision(frame, id))
            .max(super::honor_protocol::heal_revision(frame, id))
            .max(frame.world.cast(id).map_or(0, |c| c.cast_revision))
            .max(frame.damage(id).map_or(0, |c| c.cast_revision as i64))
            .max(super::disintegrate_presentation::revision(frame, id))
            .max(frame.npc_spell(id).map_or(0, |s| s.revision))
            <= cast.event.cast_revision
}
fn validate_fire_geometry(frame: &combat_protocol::Frame, map: &WorldMap) -> Result<()> {
    for s in frame.npc_fires.iter().filter(|s| s.count > 0) {
        ensure!(
            s.origin.is_some_and(|p| map.standing(p)),
            "NPC70 origin outside standing geometry"
        );
        for t in &s.targets {
            ensure!(
                map.standing(t.cell),
                "NPC70 target outside standing geometry"
            );
        }
        if let Some(t) = &s.structure {
            ensure!(
                map.standing(t.cell) && map.standing(t.origin),
                "NPC70 retained structure outside standing geometry"
            );
        }
    }
    for s in frame.fires.iter().filter(|s| s.count > 0) {
        ensure!(
            map.standing(s.from),
            "Fire Storm origin outside standing geometry"
        );
        for t in &s.targets {
            ensure!(
                map.standing(t.position),
                "Fire Storm receipt outside standing geometry"
            );
        }
    }
    Ok(())
}
impl FirePresentation {
    fn update(
        &mut self,
        frame: &combat_protocol::Frame,
        map: &WorldMap,
        npc: Option<&super::npc_protocol::Frame>,
        now: Instant,
    ) -> Result<()> {
        validate_fire_geometry(frame, map)?;
        let events = super::fire_storm_presentation::events(frame);
        let identity = (frame.world.match_id, frame.world.generation);
        if self.identity != identity
            || matches!(frame.world.phase, Phase::Preparing | Phase::Aborted)
        {
            self.identity = identity;
            self.active.clear();
            self.effects.clear();
            self.baseline = events
                .iter()
                .map(|s| (s.entity_id, s.cast_revision))
                .collect();
            return Ok(());
        }
        ensure!(
            self.baseline.len() == events.len(),
            "Fire Storm roster changed"
        );
        for event in &events {
            let previous = self
                .baseline
                .get_mut(&event.entity_id)
                .context("Fire Storm actor missing")?;
            ensure!(
                event.cast_revision >= *previous,
                "Fire Storm revision regressed"
            );
            if event.cast_revision > *previous {
                *previous = event.cast_revision;
                let a = frame
                    .world
                    .actor(event.entity_id)
                    .context("Fire Storm actor absent")?;
                let m = frame
                    .world
                    .member(event.entity_id)
                    .context("Fire Storm member absent")?;
                self.active.insert(
                    event.entity_id,
                    FireVisual {
                        event: event.clone(),
                        body: super::paced_animation::PacedClock::new(now),
                        appearance: BaseAppearance {
                            class: u8::try_from(m.class)?,
                            sex: m.sex,
                        },
                        move_sequence: a.sequence,
                        launched: false,
                        // Some qualified caster actions have Hit at elapsed zero.
                        launch_at: Some(now),
                        body_visible: true,
                    },
                );
            }
        }
        self.active.retain(|id, cast| {
            fire_body_current(frame, cast)
                && npc
                    .and_then(|f| super::npc_presentation::player_attack(f, *id))
                    .is_none_or(|attack| attack.revision <= cast.event.cast_revision)
        });
        for cast in self.active.values_mut() {
            // Native actor+80h owns the pending effect until the caster Hit
            // calls sub_4E45C0. Hurt replaces the visible body, not ownership.
            // Keep the confirmed cast's paced clock (as skill77 does); a new
            // deliberate action, death/life change or movement still cancels it.
            cast.body_visible &= frame
                .vital(cast.event.entity_id)
                .is_some_and(|v| v.hurt_revision <= cast.event.cast_revision);
            let base = u64::from(
                frame
                    .world
                    .cast(cast.event.entity_id)
                    .context("Fire Storm pace absent")?
                    .cast_interval_millis,
            ) * 1_000_000;
            cast.body.retime(
                now,
                base,
                super::timed_protocol::effective_nanos(
                    frame,
                    cast.event.entity_id,
                    base,
                    super::timed_protocol::Pace::Spell,
                ),
            )?;
            let delay = self.assets.cast_launch_delay(cast.appearance)?;
            if !cast.launched && cast.body.elapsed(now)? < delay {
                // Recompute only before crossing Hit. Once crossed, keep the
                // last known wall deadline so a stalled render cannot reset age.
                cast.launch_at = Some(cast.body.until(now, delay)?);
            }
            if !cast.launched && cast.body.elapsed(now)? >= delay {
                cast.launched = true;
                let started = cast
                    .launch_at
                    .context("Fire Storm launch deadline absent")?;
                if now.saturating_duration_since(started) >= self.assets.effect_duration() {
                    continue;
                }
                if !admit_flight(
                    &mut self.effects,
                    FireEffect {
                        position: cast.event.from,
                        started,
                        playback: self.assets.start_effect(),
                        finished: false,
                    },
                ) {
                    self.skipped = true;
                }
            }
        }
        Ok(())
    }
}
struct TowerVisual {
    action: u8,
    started: Option<Instant>,
    playback: super::tower_assets::HonorPlayback,
}
impl TowerVisual {
    fn elapsed(&self, now: Instant) -> Duration {
        self.started
            .map_or(Duration::MAX, |start| now.saturating_duration_since(start))
    }
}
struct HonorPresentation {
    assets: super::tower_assets::TowerAssets,
    towers: BTreeMap<u64, TowerVisual>,
    healing: super::honor_healing_effect::HonorHealingAssets,
    baseline: BTreeMap<u64, i64>,
    casts: BTreeMap<u64, HonorHealVisual>,
    effects: Vec<HonorHealEffect>,
    skipped: bool,
}
struct HonorHealVisual {
    event: super::honor_protocol::HealCast,
    body: super::paced_animation::PacedClock,
    move_sequence: u64,
}
struct HonorHealEffect {
    position: WorldCell,
    started: Instant,
    playback: super::tower_assets::HonorPlayback,
    finished: bool,
}
impl HonorPresentation {
    fn load_with_catalog(
        catalog: &super::shared_action_catalog::SharedActionCatalog,
        frame: &combat_protocol::Frame,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let state = frame.honor.as_ref().context("Honor state absent")?;
        let assets = super::tower_assets::TowerAssets::load_with_catalog(catalog, cancel)?;
        let towers = state
            .objectives
            .iter()
            .map(|o| {
                Ok((
                    o.entity_id,
                    TowerVisual {
                        action: o.action,
                        started: None,
                        playback: assets.start_state(o.graphics_id as u32, o.action)?,
                    },
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            assets,
            towers,
            healing: super::honor_healing_effect::HonorHealingAssets::load_with_catalog(
                catalog, cancel,
            )?,
            baseline: state
                .heals
                .iter()
                .map(|h| (h.entity_id, h.revision))
                .collect(),
            casts: BTreeMap::new(),
            effects: Vec::new(),
            skipped: false,
        })
    }
    fn update(
        &mut self,
        frame: &combat_protocol::Frame,
        map: &WorldMap,
        now: Instant,
    ) -> Result<()> {
        let state = frame.honor.as_ref().context("Honor state absent")?;
        ensure!(
            state.objectives.len() == self.towers.len(),
            "Honor tower identities changed"
        );
        for o in &state.objectives {
            ensure!(
                map.standing(o.position),
                "Honor tower outside standable geometry"
            );
            let visual = self
                .towers
                .get_mut(&o.entity_id)
                .context("Honor tower absent")?;
            if visual.action != o.action {
                visual.action = o.action;
                visual.started = Some(now);
                visual.playback = self.assets.start_state(o.graphics_id as u32, o.action)?;
            }
        }
        for event in &state.heals {
            ensure!(
                event.count == 0 || map.standing(event.from) && map.standing(event.to),
                "Honor heal geometry invalid"
            );
            let baseline = self
                .baseline
                .get_mut(&event.entity_id)
                .context("Honor healing identity absent")?;
            if event.revision > *baseline {
                *baseline = event.revision;
                let actor = frame
                    .world
                    .actors
                    .iter()
                    .find(|a| a.entity_id == event.entity_id)
                    .context("Honor caster absent")?;
                self.casts.insert(
                    event.entity_id,
                    HonorHealVisual {
                        event: event.clone(),
                        body: super::paced_animation::PacedClock::new(now),
                        move_sequence: actor.sequence,
                    },
                );
                if !admit_flight(
                    &mut self.effects,
                    HonorHealEffect {
                        position: event.to,
                        started: now,
                        playback: self.healing.start_effect(),
                        finished: false,
                    },
                ) {
                    self.skipped = true;
                }
            }
        }
        for cast in self.casts.values_mut() {
            let base = u64::from(
                frame
                    .world
                    .cast(cast.event.entity_id)
                    .context("Honor cast pace absent")?
                    .cast_interval_millis,
            ) * 1_000_000;
            cast.body.retime(
                now,
                base,
                super::timed_protocol::effective_nanos(
                    frame,
                    cast.event.entity_id,
                    base,
                    super::timed_protocol::Pace::Spell,
                ),
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct RangedEvent {
    entity: u64,
    revision: i64,
    target: u64,
    from: WorldCell,
    to: WorldCell,
    hit: bool,
}
struct RangedFlight {
    entity: u64,
    target: u64,
    path: RangedPath,
    started: Instant,
}
/// Per-match presentation only. The body is already timed by Timeline's
/// PacedClock; sampling that exact SceneSnapshot avoids a second launch clock.
struct RangedTracker {
    identity: (u64, u64),
    baseline: BTreeMap<u64, i64>,
    pending: BTreeMap<u64, RangedEvent>,
    flights: Vec<RangedFlight>,
    launch_delays: BTreeMap<u64, Duration>,
    skipped: bool,
}
impl RangedTracker {
    fn new(frame: &combat_protocol::Frame, launch_delays: BTreeMap<u64, Duration>) -> Self {
        Self {
            identity: (frame.world.match_id, frame.world.generation),
            baseline: frame
                .ammo
                .iter()
                .map(|s| (s.entity_id, s.shot_revision))
                .collect(),
            pending: BTreeMap::new(),
            flights: Vec::new(),
            launch_delays,
            skipped: false,
        }
    }
    fn update(
        &mut self,
        snapshot: &SceneSnapshot,
        frame: &combat_protocol::Frame,
        now: Instant,
    ) -> Result<()> {
        let identity = (frame.world.match_id, frame.world.generation);
        if identity != self.identity {
            // Normally a different owned worker is used. Still never carry
            // one old shot/endpoint into a reused presentation object's world.
            self.identity = identity;
            self.pending.clear();
            self.flights.clear();
            self.baseline = frame
                .ammo
                .iter()
                .map(|s| (s.entity_id, s.shot_revision))
                .collect();
            return Ok(());
        }
        if matches!(frame.world.phase, Phase::Preparing | Phase::Aborted) {
            self.pending.clear();
            self.flights.clear();
            self.baseline = frame
                .ammo
                .iter()
                .map(|s| (s.entity_id, s.shot_revision))
                .collect();
            return Ok(());
        }
        ensure!(
            snapshot.revision() == frame.world.revision as u64
                && snapshot.map_digest() == frame.world.manifest.digest,
            "ranged body and authoritative frame differ"
        );
        let present = |entity| super::honor_protocol::target(frame, entity).is_some();
        self.pending
            .retain(|_, event| present(event.entity) && present(event.target));
        self.flights.retain(|flight| {
            present(flight.entity)
                && present(flight.target)
                && !flight
                    .path
                    .sample(now.saturating_duration_since(flight.started))
                    .finished
        });
        self.baseline.retain(|entity, _| present(*entity));
        for state in &frame.ammo {
            let previous = self
                .baseline
                .get_mut(&state.entity_id)
                .context("ranged actor absent from baseline")?;
            ensure!(
                state.shot_revision >= *previous,
                "ranged event revision regressed"
            );
            if state.shot_revision > *previous {
                *previous = state.shot_revision;
                self.pending.insert(
                    state.entity_id,
                    RangedEvent {
                        entity: state.entity_id,
                        revision: state.shot_revision,
                        target: state.last_target_entity_id,
                        from: state.from,
                        to: state.to,
                        hit: state.hit,
                    },
                );
            }
        }
        let mut retired = Vec::new();
        for (&entity, event) in &self.pending {
            let Some(actor) = snapshot.actors().iter().find(|a| a.entity_id == entity) else {
                retired.push(entity);
                continue;
            };
            let vital = frame.vital(entity).context("ranged vital absent")?;
            // A later movement/cast/hurt/death may replace the body before its
            // Hit. Do not replay a stale attack solely to manufacture an arrow.
            // Already launched flights are independent and are not removed here.
            if actor.action != BaseAction::Attack
                || vital.hp == 0
                || vital.attack_revision != event.revision
                || vital.hurt_revision > event.revision
                || vital.death_revision > event.revision
            {
                self.skipped = true;
                retired.push(entity);
                continue;
            }
            let delay = self
                .launch_delays
                .get(&entity)
                .context("qualified bow launch marker absent")?;
            if actor.action_elapsed < *delay {
                continue;
            }
            retired.push(entity);
            let Some(target) = super::honor_protocol::target_position(frame, event.target) else {
                continue;
            };
            // Native class3 chooses a hit target's current position only here;
            // a miss retains the packet endpoint. Corpse positions remain valid.
            // The origin is the shot event, never this actor's later position.
            let endpoint = if event.hit { target } else { event.to };
            match RangedPath::new(event.from, endpoint) {
                Ok(path) => {
                    if !path.sample(Duration::ZERO).finished
                        && !admit_flight(
                            &mut self.flights,
                            RangedFlight {
                                entity,
                                target: event.target,
                                path,
                                started: now,
                            },
                        )
                    {
                        self.skipped = true;
                    }
                }
                Err(RangedError::Bound(_)) => self.skipped = true,
                Err(_) => self.skipped = true,
            }
        }
        for entity in retired {
            self.pending.remove(&entity);
        }
        Ok(())
    }
}
struct RangedPresentation {
    assets: RangedAssets,
    tracker: RangedTracker,
}

/// The wire manifest only establishes sector containment. Qualify retained
/// shot coordinates against the already loaded geometry before observing any
/// revision, including a historical event that will not be replayed.
fn validate_ranged_geometry(frame: &combat_protocol::Frame, map: &WorldMap) -> Result<()> {
    for state in frame.ammo.iter().filter(|state| state.shot_count > 0) {
        ensure!(
            map.standing(state.from) && map.standing(state.to),
            "ranged event contains a non-standable source or endpoint"
        );
    }
    Ok(())
}

/// Only inspect a prequalified bow21 body. Sound/Hit events are not executed
/// here; AudioSequencer remains the single body audio owner. Terminal source0
/// is excluded from the active timeline, including its possible nonzero ticks.
pub(crate) fn bow_first_hit_delay(
    animation: &super::sprite_actions::Animation,
) -> Result<Duration> {
    use super::sprite_actions::{Directionality, FrameEvent};
    ensure!(
        animation.directionality == Directionality::EightWay
            && animation.frames_per_second == 24
            && animation.frames.len() >= 2
            && !animation.frames[0].events.contains(&FrameEvent::Hit),
        "unsupported qualified bow launch definition"
    );
    let mut ticks = 0u64;
    let mut launch = None;
    for frame in &animation.frames[1..] {
        for cue in &frame.events {
            if *cue == FrameEvent::Hit {
                ensure!(launch.is_none(), "ambiguous bow launch markers");
                launch = Some(ticks);
            }
        }
        ticks = ticks
            .checked_add(u64::from(frame.duration_ticks))
            .context("bow launch ticks overflow")?;
    }
    let nanos = (u128::from(launch.context("bow launch marker missing")?) * 1_000_000_000)
        .div_ceil(u128::from(animation.frames_per_second));
    ensure!(
        nanos <= 10_000_000_000,
        "bow launch timeline exceeds presentation bound"
    );
    Ok(Duration::from_nanos(u64::try_from(nanos)?))
}
struct HealingVisual {
    effect: i32,
    position: WorldCell,
    started: Instant,
    playback: super::healing_effect::HealingPlayback,
    finished: bool,
}
struct HealingPresentation {
    assets: super::healing_effect::HealingAssets,
    baseline: BTreeMap<u64, i64>,
    active: Vec<HealingVisual>,
    skipped: bool,
}
fn advance_item_watermark(
    baseline: &mut BTreeMap<u64, i64>,
    entity: u64,
    revision: i64,
) -> Result<bool> {
    let previous = baseline.get_mut(&entity).context("supply actor absent")?;
    ensure!(revision >= *previous, "supply event revision regressed");
    let changed = revision > *previous;
    *previous = revision;
    Ok(changed)
}
impl HealingPresentation {
    fn update(
        &mut self,
        frame: &combat_protocol::Frame,
        map: &WorldMap,
        now: Instant,
    ) -> Result<()> {
        ensure!(
            frame.items.len() == self.baseline.len(),
            "supply identities changed"
        );
        for event in &frame.items {
            ensure!(
                event.use_count == 0 || map.standing(event.position),
                "healing event outside bound geometry"
            );
            if advance_item_watermark(&mut self.baseline, event.entity_id, event.use_revision)? {
                if super::item_protocol::healing(event.item_id).is_none() {
                    continue;
                }
                let visual = HealingVisual {
                    effect: event.effect,
                    position: event.position,
                    started: now,
                    playback: self.assets.start_effect(event.effect)?,
                    finished: false,
                };
                if !admit_flight(&mut self.active, visual) {
                    self.skipped = true;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LightArrowTarget {
    Player(u64),
    Npc {
        id: super::npc_protocol::Id,
        life: u64,
    },
}
#[derive(Clone, Copy)]
struct LightArrowEvent {
    entity_id: u64,
    cast_count: u64,
    cast_revision: u64,
    from: WorldCell,
    to: WorldCell,
    target: Option<LightArrowTarget>,
    caster_life: Option<u64>,
}
fn light_arrow_events(f: &combat_protocol::Frame) -> Result<Vec<LightArrowEvent>> {
    ensure!(
        f.damages.is_empty() || f.npc_spells.is_empty(),
        "NPC and player magic ledgers cannot alias"
    );
    Ok(f.damages
        .iter()
        .map(|c| LightArrowEvent {
            entity_id: c.entity_id,
            cast_count: c.cast_count,
            cast_revision: c.cast_revision,
            from: c.from,
            to: c.to,
            target: Some(LightArrowTarget::Player(c.target_entity_id)),
            caster_life: None,
        })
        .chain(f.npc_spells.iter().map(|c| {
            LightArrowEvent {
                entity_id: c.entity_id,
                cast_count: c.cast_count,
                cast_revision: c.revision as u64,
                from: c
                    .receipt
                    .as_ref()
                    .map_or(WorldCell { x: 0, y: 0 }, |r| r.from),
                to: c
                    .receipt
                    .as_ref()
                    .map_or(WorldCell { x: 0, y: 0 }, |r| r.to),
                target: c.receipt.as_ref().map(|r| LightArrowTarget::Npc {
                    id: r.target,
                    life: r.target_life,
                }),
                caster_life: Some(c.caster_life),
            }
        }))
        .collect())
}
fn light_arrow_target(
    event: &LightArrowEvent,
    frame: &combat_protocol::Frame,
    npc: Option<&super::npc_protocol::Frame>,
) -> Result<WorldCell> {
    match event.target.context("light-arrow receipt has no target")? {
        LightArrowTarget::Player(id) => super::honor_protocol::target_position(frame, id)
            .context("spell target retired without world retirement"),
        LightArrowTarget::Npc { id, life } => {
            let npc = npc.context("NPC spell lacks atomic NPC target state")?;
            ensure!(
                npc.scope == id.scope && npc.combat == *frame,
                "NPC spell target scope changed"
            );
            // A corpse may retire before a delayed render. Preserve the last
            // confirmed impact cell, never retarget a new life or player ID.
            Ok(npc
                .actor(id)
                .filter(|a| a.life == life)
                .map_or(event.to, |a| a.cell))
        }
    }
}
struct DamageVisual {
    event: LightArrowEvent,
    body: super::paced_animation::PacedClock,
    move_sequence: u64,
    appearance: BaseAppearance,
    /// Once launched, the effect has its own lifetime, independent of body replacement.
    launched: bool,
    body_visible: bool,
}

// A newer hurt pose hides the body, but does not discard actor+80h's pending
// effect. A deliberate action, relocation, death/life change or departure does.
// Already launched projectiles live in a separate owned collection.
fn light_arrow_body_current(
    frame: &combat_protocol::Frame,
    npc: Option<&super::npc_protocol::Frame>,
    cast: &DamageVisual,
) -> bool {
    let id = cast.event.entity_id;
    let (Some(v), Some(a)) = (frame.vital(id), frame.world.actor(id)) else {
        return false;
    };
    v.hp > 0
        && frame
            .world
            .time_attack
            .as_ref()
            .is_none_or(|t| !t.departed(id))
        && cast.event.caster_life.is_none_or(|life| {
            super::quest_life::life(&frame.world, id).map_or(1, |l| l.incarnation()) == life
        })
        && a.sequence == cast.move_sequence
        && a.position == cast.event.from
        && v.attack_revision
            .max(
                npc.and_then(|n| super::npc_presentation::player_attack(n, id))
                    .map_or(0, |a| a.revision),
            )
            .max(v.death_revision)
            .max(super::quest_life::respawn_revision(frame, id))
            .max(super::honor_protocol::heal_revision(frame, id))
            .max(frame.world.cast(id).map_or(0, |c| c.cast_revision))
            .max(super::fire_storm_presentation::revision(frame, id))
            .max(super::disintegrate_presentation::revision(frame, id))
            <= cast.event.cast_revision as i64
}
struct FlightVisual {
    path: LightArrowPath,
    playback: LightArrowPlayback,
    started: Instant,
    finished: bool,
}
const MAX_DAMAGE_FLIGHTS: usize = 256;
/// Admission never evicts an already launched effect. New excess visuals are
/// explicitly omitted; this is not an authority, damage, or movement budget.
fn admit_flight<T>(flights: &mut Vec<T>, flight: T) -> bool {
    if flights.len() >= MAX_DAMAGE_FLIGHTS {
        false
    } else {
        flights.push(flight);
        true
    }
}
struct LightArrowPresentation {
    assets: LightArrowAssets,
    baseline: BTreeMap<u64, u64>,
    active: BTreeMap<u64, DamageVisual>,
    flights: Vec<FlightVisual>,
    skipped_visual: bool,
}
impl LightArrowPresentation {
    fn update(
        &mut self,
        frame: &combat_protocol::Frame,
        npc: Option<&super::npc_protocol::Frame>,
        map: &WorldMap,
        now: Instant,
    ) -> Result<()> {
        let events = light_arrow_events(frame)?;
        ensure!(
            npc.is_none_or(|n| n.combat == *frame),
            "Light Arrow NPC snapshot differs"
        );
        ensure!(
            events.len() == self.baseline.len(),
            "damage spell identities changed"
        );
        for event in &events {
            ensure!(
                event.cast_count == 0 || map.standing(event.from) && map.standing(event.to),
                "damage event outside bound geometry"
            );
            let baseline = self
                .baseline
                .get_mut(&event.entity_id)
                .context("damage caster absent")?;
            if event.cast_revision > *baseline {
                let index = frame
                    .world
                    .actors
                    .iter()
                    .position(|a| a.entity_id == event.entity_id)
                    .context("damage caster absent")?;
                let member = frame
                    .world
                    .member(event.entity_id)
                    .context("damage member absent")?;
                self.active.insert(
                    event.entity_id,
                    DamageVisual {
                        event: *event,
                        body: super::paced_animation::PacedClock::new(now),
                        move_sequence: frame.world.actors[index].sequence,
                        appearance: BaseAppearance {
                            class: u8::try_from(member.class)?,
                            sex: member.sex,
                        },
                        launched: false,
                        body_visible: true,
                    },
                );
                *baseline = event.cast_revision;
            }
        }
        // A departed cooperative member remains in the authority/result roster,
        // but cannot start a pending projectile or retain a body override. A
        // projectile already in flight has its own lifetime and is not removed.
        self.active
            .retain(|_, cast| light_arrow_body_current(frame, npc, cast));
        for cast in self.active.values_mut() {
            cast.body_visible &= frame
                .vital(cast.event.entity_id)
                .is_some_and(|v| v.hurt_revision <= cast.event.cast_revision as i64);
            let base = u64::from(
                frame
                    .world
                    .cast(cast.event.entity_id)
                    .context("spell pace absent")?
                    .cast_interval_millis,
            ) * 1_000_000;
            cast.body.retime(
                now,
                base,
                super::timed_protocol::effective_nanos(
                    frame,
                    cast.event.entity_id,
                    base,
                    super::timed_protocol::Pace::Spell,
                ),
            )?;
            let delay = self.assets.cast_launch_delay(cast.appearance)?;
            if !cast.launched && cast.body.elapsed(now)? >= delay {
                cast.launched = true;
                let target = light_arrow_target(&cast.event, frame, npc)?;
                ensure!(map.standing(target), "spell launch target outside geometry");
                let path = match LightArrowPath::new(cast.event.from, target) {
                    Ok(path) => path,
                    Err(LightArrowError::Bound(_)) => {
                        self.skipped_visual = true;
                        continue;
                    }
                    Err(error) => return Err(error.into()),
                };
                let playback = self.assets.start_effect(&path);
                // On a delayed render, launch now using the latest confirmed target,
                // rather than inventing its historical position at a missed deadline.
                if !admit_flight(
                    &mut self.flights,
                    FlightVisual {
                        path,
                        playback,
                        started: now,
                        finished: false,
                    },
                ) {
                    self.skipped_visual = true;
                }
            }
        }
        Ok(())
    }
}

struct TimedVisual {
    effect: i32,
    position: WorldCell,
    started: Instant,
    playback: super::timed_effect::TimedPlayback,
    finished: bool,
}
struct TimedPresentation {
    assets: super::timed_effect::TimedAssets,
    baseline: BTreeMap<u64, i64>,
    active: Vec<TimedVisual>,
    skipped: bool,
}
impl TimedPresentation {
    fn update(
        &mut self,
        frame: &combat_protocol::Frame,
        map: &WorldMap,
        now: Instant,
    ) -> Result<()> {
        ensure!(
            frame.items.len() == self.baseline.len() && !frame.timed.is_empty(),
            "timed presentation identity changed"
        );
        for event in &frame.items {
            if advance_item_watermark(&mut self.baseline, event.entity_id, event.use_revision)? {
                if let Some(kind) = super::timed_protocol::Kind::from_item(event.item_id) {
                    ensure!(
                        map.standing(event.position) && event.effect == kind.effect(),
                        "timed event outside bound geometry"
                    );
                    let visual = TimedVisual {
                        effect: event.effect,
                        position: event.position,
                        started: now,
                        playback: self.assets.start_effect(event.effect)?,
                        finished: false,
                    };
                    if !admit_flight(&mut self.active, visual) {
                        self.skipped = true;
                    }
                }
            }
        }
        Ok(())
    }
}

struct CastVisual {
    event: CastState,
    started: Instant,
    body: super::paced_animation::PacedClock,
    move_sequence: u64,
    effect: TeleportPlayback,
}
struct TeleportPresentation {
    assets: TeleportAssets,
    baseline: BTreeMap<u64, i64>,
    active: BTreeMap<u64, CastVisual>,
}
fn teleport_body_current(
    frame: &combat_protocol::Frame,
    npc: Option<&super::npc_protocol::Frame>,
    event: &CastState,
    move_sequence: u64,
) -> bool {
    let id = event.entity_id;
    let (Some(v), Some(a)) = (frame.vital(id), frame.world.actor(id)) else {
        return false;
    };
    v.hp > 0
        && frame
            .world
            .time_attack
            .as_ref()
            .is_none_or(|t| !t.departed(id))
        && a.sequence == move_sequence
        && a.position == event.to
        && v.attack_revision
            .max(
                npc.and_then(|n| super::npc_presentation::player_attack(n, id))
                    .map_or(0, |a| a.revision),
            )
            .max(v.hurt_revision)
            .max(v.death_revision)
            .max(
                super::siege_presentation::player_attack(&frame.world, id)
                    .map_or(0, |a| a.revision),
            )
            .max(super::quest_life::respawn_revision(frame, id))
            .max(super::honor_protocol::heal_revision(frame, id))
            .max(super::disintegrate_presentation::revision(frame, id))
            .max(super::fire_storm_presentation::revision(frame, id))
            .max(frame.damage(id).map_or(0, |c| c.cast_revision as i64))
            .max(frame.npc_spell(id).map_or(0, |c| c.revision))
            <= event.cast_revision
}
impl TeleportPresentation {
    fn update(
        &mut self,
        frame: &combat_protocol::Frame,
        map: &WorldMap,
        now: Instant,
    ) -> Result<()> {
        ensure!(
            frame.world.casts.len() == self.baseline.len(),
            "spell presentation identity changed"
        );
        for event in &frame.world.casts {
            if event.cast_count > 0 {
                ensure!(
                    map.standing(event.from) && map.standing(event.to),
                    "teleport event outside authoritative standable geometry"
                );
            }
            let baseline = self
                .baseline
                .get_mut(&event.entity_id)
                .context("spell actor absent")?;
            if event.cast_revision > *baseline {
                let actor = frame
                    .world
                    .actors
                    .iter()
                    .find(|a| a.entity_id == event.entity_id)
                    .context("spell actor absent")?;
                self.active.insert(
                    event.entity_id,
                    CastVisual {
                        event: *event,
                        started: now,
                        body: super::paced_animation::PacedClock::new(now),
                        move_sequence: actor.sequence,
                        effect: self.assets.start_effect(),
                    },
                );
                *baseline = event.cast_revision;
            }
        }
        for cast in self.active.values_mut() {
            let base = u64::from(cast.event.cast_interval_millis) * 1_000_000;
            cast.body.retime(
                now,
                base,
                super::timed_protocol::effective_nanos(
                    frame,
                    cast.event.entity_id,
                    base,
                    super::timed_protocol::Pace::Spell,
                ),
            )?;
        }
        Ok(())
    }
}
impl Renderer for SharedRenderer {
    fn render_npcs(
        &mut self,
        snapshot: &SceneSnapshot,
        combat: Option<&combat_protocol::Frame>,
        npc: Option<&NpcInput>,
        cancel: &AtomicBool,
    ) -> Result<Pixels> {
        self.render_at_npcs(snapshot, combat, npc, cancel, Instant::now())
    }
    fn navigation_map(&self) -> Option<Arc<WorldMap>> {
        Some(self.map.clone())
    }
    fn render(
        &mut self,
        snapshot: &SceneSnapshot,
        combat: Option<&combat_protocol::Frame>,
        cancel: &AtomicBool,
    ) -> Result<Pixels> {
        self.render_at(snapshot, combat, cancel, Instant::now())
    }
}

impl SharedRenderer {
    /// Keep every presentation layer on one clock. Production supplies the
    /// current instant above; deterministic compositor tests inject that same
    /// instant without changing actor elapsed times or authoritative vitals.
    fn render_at(
        &mut self,
        snapshot: &SceneSnapshot,
        combat: Option<&combat_protocol::Frame>,
        cancel: &AtomicBool,
        now: Instant,
    ) -> Result<Pixels> {
        self.render_at_npcs(snapshot, combat, None, cancel, now)
    }
    fn render_at_npcs(
        &mut self,
        snapshot: &SceneSnapshot,
        combat: Option<&combat_protocol::Frame>,
        npc: Option<&NpcInput>,
        cancel: &AtomicBool,
        now: Instant,
    ) -> Result<Pixels> {
        let roof_frame = self.assets.sample_roofs(&mut self.roofs, snapshot, now)?;
        ensure!(
            npc.is_none()
                || self.fire_storm.is_some()
                    == combat.is_some_and(super::fire_storm_presentation::enabled),
            "Fire Storm presentation capability changed"
        );
        ensure!(
            self.npc.is_some() == npc.is_some(),
            "NPC presentation capability changed"
        );
        ensure!(
            self.quest.is_some() == combat.is_some_and(|f| f.world.quest.is_some()),
            "Quest presentation capability changed"
        );
        ensure!(
            self.time_attack.is_some() == combat.is_some_and(|f| f.world.time_attack.is_some()),
            "cooperative tower presentation capability changed"
        );
        ensure!(
            self.siege.is_some() == combat.is_some_and(|f| f.world.siege.is_some()),
            "siege presentation capability changed"
        );
        ensure!(
            self.disintegrate.is_some()
                == combat.is_some_and(super::disintegrate_presentation::enabled),
            "Disintegrate presentation capability changed"
        );
        if let (Some(presentation), Some(input)) = (&mut self.npc, npc) {
            let f = input.reply.frame.as_ref().context("NPC snapshot absent")?;
            ensure!(
                Some(&f.combat) == combat
                    && f.combat.world.revision as u64 == snapshot.revision()
                    && f.scope.match_id == snapshot.world_id(),
                "NPC/world draw revision mismatch"
            );
            presentation.update(input.reply.clone(), now, cancel)?;
            let cues = presentation.sample(now, cancel)?;
            self.audio
                .as_mut()
                .context("NPC audio owner absent")?
                .push_preloaded_cues(&cues, presentation.assets.sound_bank(), now)?;
        }
        if let Some(pet) = &mut self.pet {
            ensure!(
                combat.is_none()
                    && self.audio.is_none()
                    && self.teleport.is_none()
                    && self.light_arrow.is_none()
                    && self.healing.is_none()
                    && self.timed.is_none()
                    && self.ranged.is_none()
                    && self.honor.is_none()
                    && self.quest.is_none()
                    && self.time_attack.is_none()
                    && self.siege.is_none()
                    && self.fire_storm.is_none()
                    && self.disintegrate.is_none(),
                "race renderer cannot carry combat/player layers"
            );
            let (poses, camera, audio) = pet.update(snapshot, &self.map, now, cancel)?;
            let draws = poses
                .iter()
                .map(|pose| {
                    Ok(super::world_scene::PetDraw {
                        entity_id: pose.entity_id,
                        anchor_cell: pose.anchor,
                        sample: pet.assets.frame(
                            pose.appearance,
                            pose.action,
                            pose.heading,
                            pose.source,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let info = self.assets.render_into_pets_roofs(
                snapshot,
                &mut self.canvas,
                cancel,
                &draws,
                camera,
                Some(&roof_frame),
            )?;
            ensure!(!cancel.load(Ordering::Acquire), "pet rendering cancelled");
            self.canvas.copy_bgra(&mut self.bgra)?;
            ensure!(!cancel.load(Ordering::Acquire), "pet rendering cancelled");
            return Ok(Pixels {
                bgra: Arc::from(self.bgra.as_slice()), limitations: self.assets.limitations().clone(),
                actors: info.actors, objectives: Vec::new(), quests: Vec::new(), time_attack_tower: None, siege_towers: Vec::new(), npcs: Vec::new(), audio: Some(audio),
                presentation_notice: Some("寵物使用已資格共享素材及權威步速；停止後採明示 idle 排程，非原生 mouse-up／RGB565 逐像素重製。"),
            });
        }
        ensure!(
            snapshot.pet().is_none(),
            "race metadata requires qualified pet resources"
        );
        if let Some(fire) = &mut self.fire_storm {
            fire.update(
                combat.context("Fire Storm lacks atomic combat state")?,
                &self.map,
                npc.and_then(|n| n.reply.frame.as_ref()),
                now,
            )?;
        }
        if let Some(dis) = &mut self.disintegrate {
            dis.update(
                combat.context("Disintegrate lacks atomic combat state")?,
                &self.map,
                &self
                    .light_arrow
                    .as_ref()
                    .context("Disintegrate caster18 absent")?
                    .assets,
                npc.and_then(|n| n.reply.frame.as_ref()),
                now,
            )?;
        }
        if let Some(honor) = &mut self.honor {
            honor.update(
                combat.context("Honor lacks atomic combat state")?,
                &self.map,
                now,
            )?;
        }
        if let Some(quest) = &mut self.quest {
            quest.update(
                &combat.context("Quest lacks atomic combat state")?.world,
                now,
                false,
            )?;
        }
        if let Some(tower) = &mut self.time_attack {
            tower.update(
                &combat
                    .context("cooperative tower lacks atomic combat state")?
                    .world,
                now,
            )?;
        }
        if let Some(siege) = &mut self.siege {
            siege.update(&combat.context("siege combat absent")?.world, now)?;
        }
        if let Some(teleport) = &mut self.teleport {
            teleport.update(
                combat.context("spell world lacks atomic combat state")?,
                &self.map,
                now,
            )?;
        }
        if let Some(light) = &mut self.light_arrow {
            light.update(
                combat.context("damage spell lacks atomic combat state")?,
                npc.and_then(|n| n.reply.frame.as_ref()),
                &self.map,
                now,
            )?;
        }
        if let Some(ranged) = &mut self.ranged {
            let combat = combat.context("ranged world lacks atomic combat state")?;
            validate_ranged_geometry(combat, &self.map)?;
            ranged.tracker.update(snapshot, combat, now)?;
        }
        let mut replacements = Vec::new();
        if let (Some(teleport), Some(combat)) = (&self.teleport, combat) {
            for actor in snapshot.actors() {
                if let Some(cast) = teleport.active.get(&actor.entity_id) {
                    let elapsed = cast.body.peek(now)?;
                    let authoritative = combat
                        .world
                        .actor(actor.entity_id)
                        .context("spell actor absent")?;
                    if teleport_body_current(
                        combat,
                        npc.and_then(|n| n.reply.frame.as_ref()),
                        &cast.event,
                        cast.move_sequence,
                    ) && elapsed < teleport.assets.cast_duration(actor.appearance)?
                    {
                        let visual = teleport.assets.caster_frame(
                            actor.appearance,
                            authoritative.heading,
                            elapsed,
                        )?;
                        if !visual.finished {
                            replacements.push(ActorOverride {
                                entity_id: actor.entity_id,
                                frame: visual.frame,
                                graphics_id: visual.graphics_id,
                                source_frame: visual.source_frame,
                                sprite_name: visual.sprite_name,
                                subframe: visual.subframe,
                            });
                        }
                    }
                }
            }
        }
        if let (Some(light), Some(combat)) = (&self.light_arrow, combat) {
            for actor in snapshot.actors() {
                if let Some(cast) = light.active.get(&actor.entity_id) {
                    let elapsed = cast.body.peek(now)?;
                    if cast.body_visible
                        && light_arrow_body_current(
                            combat,
                            npc.and_then(|n| n.reply.frame.as_ref()),
                            cast,
                        )
                        && elapsed < light.assets.cast_duration(actor.appearance)?
                    {
                        let visual = light.assets.caster_frame(
                            actor.appearance,
                            facing(cast.event.from, cast.event.to)?,
                            elapsed,
                        )?;
                        if !visual.finished {
                            replacements.push(ActorOverride {
                                entity_id: actor.entity_id,
                                frame: visual.frame,
                                graphics_id: visual.graphics_id,
                                source_frame: visual.source_frame,
                                sprite_name: visual.sprite_name,
                                subframe: visual.subframe,
                            });
                        }
                    }
                }
            }
        }
        if let (Some(honor), Some(teleport), Some(combat)) = (&self.honor, &self.teleport, combat) {
            for actor in snapshot.actors() {
                if let Some(cast) = honor.casts.get(&actor.entity_id) {
                    let elapsed = cast.body.peek(now)?;
                    let vital = combat
                        .vital(actor.entity_id)
                        .context("Honor vital absent")?;
                    let current = combat
                        .world
                        .actor(actor.entity_id)
                        .context("Honor actor absent")?;
                    let last = vital
                        .attack_revision
                        .max(vital.hurt_revision)
                        .max(vital.death_revision)
                        .max(super::disintegrate_presentation::revision(
                            combat,
                            actor.entity_id,
                        ))
                        .max(super::fire_storm_presentation::revision(
                            combat,
                            actor.entity_id,
                        ))
                        .max(super::quest_life::respawn_revision(combat, actor.entity_id))
                        .max(
                            combat
                                .world
                                .cast(actor.entity_id)
                                .map_or(0, |c| c.cast_revision),
                        )
                        .max(
                            combat
                                .damage(actor.entity_id)
                                .map_or(0, |d| d.cast_revision as i64),
                        );
                    if vital.hp > 0
                        && last <= cast.event.revision
                        && current.sequence == cast.move_sequence
                        && current.position == cast.event.from
                        && elapsed < teleport.assets.cast_duration(actor.appearance)?
                    {
                        let visual = teleport.assets.caster_frame(
                            actor.appearance,
                            facing(cast.event.from, cast.event.to)?,
                            elapsed,
                        )?;
                        if !visual.finished {
                            replacements.push(ActorOverride {
                                entity_id: actor.entity_id,
                                frame: visual.frame,
                                graphics_id: visual.graphics_id,
                                source_frame: visual.source_frame,
                                sprite_name: visual.sprite_name,
                                subframe: visual.subframe,
                            });
                        }
                    }
                }
            }
        }
        if let (Some(fire), Some(combat)) = (&self.fire_storm, combat) {
            for actor in snapshot.actors() {
                if let Some(cast) = fire.active.get(&actor.entity_id) {
                    let elapsed = cast.body.peek(now)?;
                    if cast.body_visible
                        && fire_body_current(combat, cast)
                        && elapsed < fire.assets.cast_duration(actor.appearance)?
                    {
                        let heading = combat
                            .world
                            .actor(actor.entity_id)
                            .context("Fire Storm actor absent")?
                            .heading;
                        let visual =
                            fire.assets
                                .caster_frame(actor.appearance, heading, elapsed)?;
                        if !visual.finished {
                            replacements.push(ActorOverride {
                                entity_id: actor.entity_id,
                                frame: visual.frame,
                                graphics_id: visual.graphics_id,
                                source_frame: visual.source_frame,
                                sprite_name: visual.sprite_name,
                                subframe: visual.subframe,
                            });
                        }
                    }
                }
            }
        }
        if let Some(dis) = &self.disintegrate {
            replacements.extend(
                dis.overrides(
                    &self
                        .light_arrow
                        .as_ref()
                        .context("Disintegrate caster18 absent")?
                        .assets,
                    now,
                )?,
            );
        }
        let mut objective_draws = Vec::new();
        if let (Some(honor), Some(state)) = (&self.honor, combat.and_then(|f| f.honor.as_ref())) {
            for o in &state.objectives {
                let elapsed = honor.towers[&o.entity_id].elapsed(now);
                let body = honor
                    .assets
                    .state_frame(o.graphics_id as u32, o.action, elapsed)?;
                let shadow = honor
                    .assets
                    .shadow_frame(o.graphics_id as u32, o.action, elapsed)?;
                objective_draws.push(super::world_scene::ObjectiveDraw {
                    entity_id: o.entity_id,
                    blocking: true,
                    cell: o.position,
                    frame: body.frame,
                    shadow: Some(shadow.frame),
                    graphics_id: body.graphics_id,
                    source_frame: body.source_frame,
                    sprite_name: body.sprite_name,
                    subframe: body.subframe,
                });
            }
        }
        if let Some(quest) = &self.quest {
            objective_draws.extend(quest.draws(&combat.context("Quest state absent")?.world, now)?);
        }
        if let Some(tower) = &self.time_attack {
            objective_draws
                .push(tower.draw(&combat.context("cooperative tower absent")?.world, now)?);
        }
        if let Some(siege) = &self.siege {
            objective_draws
                .extend(siege.draws(&combat.context("siege combat absent")?.world, now)?);
        }
        let npc_draws = self
            .npc
            .as_ref()
            .map(|n| n.draws())
            .transpose()?
            .unwrap_or_default();
        let dis_draws = self
            .disintegrate
            .as_ref()
            .map(|d| d.draws(now))
            .transpose()?
            .unwrap_or_default();
        let info = self.assets.render_into_npcs_roofs_effects(
            snapshot,
            &mut self.canvas,
            cancel,
            &replacements,
            &objective_draws,
            &npc_draws,
            Some(&roof_frame),
            &dis_draws,
        )?;
        drop(dis_draws);
        drop(npc_draws);
        drop(objective_draws);
        // Release frame borrows before advancing/removing owned playback clocks.
        drop(replacements);
        if let Some(dis) = &mut self.disintegrate {
            dis.audio(
                self.audio.as_mut().context("Disintegrate audio absent")?,
                now,
            )?;
        }
        if let Some(siege) = &mut self.siege {
            siege.audio(self.audio.as_mut().context("siege audio absent")?, now)?;
            siege.markers(
                &mut self.canvas,
                &combat.context("siege combat absent")?.world,
                info.projection,
            )?;
        }
        if let Some(quest) = &mut self.quest {
            quest.audio(self.audio.as_mut().context("Quest audio absent")?, now)?;
        }
        if let Some(tower) = &mut self.time_attack {
            tower.audio(
                self.audio
                    .as_mut()
                    .context("cooperative tower audio absent")?,
                now,
            )?;
        }
        if let Some(honor) = &mut self.honor {
            for visual in honor.towers.values_mut() {
                if let Some(started) = visual.started {
                    let elapsed = now.saturating_duration_since(started);
                    let sample = visual.playback.advance(elapsed)?;
                    if elapsed <= Duration::from_millis(500) && !sample.crossed_cues.is_empty() {
                        self.audio
                            .as_mut()
                            .context("Honor audio absent")?
                            .push_preloaded_cues(
                                &sample.crossed_cues,
                                honor.assets.sound_bank(),
                                started,
                            )?;
                    }
                }
            }
            for effect in &mut honor.effects {
                let elapsed = now.saturating_duration_since(effect.started);
                let sample = effect.playback.advance(elapsed)?;
                if elapsed <= Duration::from_millis(500) && !sample.crossed_cues.is_empty() {
                    self.audio
                        .as_mut()
                        .context("Honor heal audio absent")?
                        .push_preloaded_cues(
                            &sample.crossed_cues,
                            honor.healing.sound_bank(),
                            effect.started,
                        )?;
                }
                let anchor = info
                    .projection
                    .cell_to_pixel(effect.position)
                    .context("Honor heal projection overflow")?;
                honor
                    .healing
                    .draw_effect(&mut self.canvas, anchor, elapsed)?;
                effect.finished = sample.finished;
            }
            honor.effects.retain(|e| !e.finished);
            honor
                .casts
                .retain(|_, c| c.body.peek(now).unwrap_or(Duration::MAX) < Duration::from_secs(10));
        }
        if let Some(teleport) = &mut self.teleport {
            for cast in teleport.active.values_mut() {
                let elapsed = now.saturating_duration_since(cast.started);
                let sample = cast.effect.advance(elapsed)?;
                if !sample.crossed_cues.is_empty() && elapsed <= Duration::from_millis(500) {
                    self.audio
                        .as_mut()
                        .context("spell audio absent")?
                        .push_preloaded_cues(
                            &sample.crossed_cues,
                            teleport.assets.sound_bank(),
                            cast.started,
                        )?;
                }
                let anchor = info
                    .projection
                    .cell_to_pixel(cast.event.from)
                    .context("teleport effect projection overflow")?;
                teleport
                    .assets
                    .draw_effect(&mut self.canvas, anchor, elapsed)?;
            }
            // All qualified caster actions are bounded below ten seconds. Keep
            // the independent revision watermark after retiring their clocks.
            teleport.active.retain(|_, cast| {
                now.saturating_duration_since(cast.started) < Duration::from_secs(10)
            });
        }
        if let Some(light) = &mut self.light_arrow {
            for flight in &mut light.flights {
                let elapsed = now.saturating_duration_since(flight.started);
                let sample = flight.playback.advance(elapsed)?;
                if !sample.crossed_cues.is_empty() && elapsed <= Duration::from_millis(500) {
                    self.audio
                        .as_mut()
                        .context("damage spell audio absent")?
                        .push_preloaded_cues(
                            &sample.crossed_cues,
                            light.assets.sound_bank(),
                            flight.started,
                        )?;
                }
                light.assets.draw_effect(
                    &mut self.canvas,
                    &flight.path,
                    info.projection,
                    elapsed,
                )?;
                flight.finished = sample.finished;
            }
            light.flights.retain(|flight| !flight.finished);
            let assets = &light.assets;
            light.active.retain(|_, cast| {
                cast.body.peek(now).unwrap_or(Duration::MAX)
                    < assets
                        .cast_duration(cast.appearance)
                        .unwrap_or(Duration::ZERO)
            });
        }
        if let Some(fire) = &mut self.fire_storm {
            for effect in &mut fire.effects {
                let elapsed = now.saturating_duration_since(effect.started);
                let sample = effect.playback.advance(elapsed)?;
                // One group cursor owns Sound710. The later native Hit remains
                // metadata: target HP/hurt already arrived atomically in the ACK.
                if elapsed <= Duration::from_millis(500) && !sample.crossed_cues.is_empty() {
                    self.audio
                        .as_mut()
                        .context("Fire Storm audio absent")?
                        .push_preloaded_cues(
                            &sample.crossed_cues,
                            fire.assets.sound_bank(),
                            effect.started,
                        )?;
                }
                let anchor = info
                    .projection
                    .cell_to_pixel(effect.position)
                    .context("Fire Storm projection overflow")?;
                fire.assets.draw_effect(&mut self.canvas, anchor, elapsed)?;
                effect.finished = sample.finished;
            }
            fire.effects.retain(|e| !e.finished);
            let assets = &fire.assets;
            fire.active.retain(|_, c| {
                c.body.peek(now).unwrap_or(Duration::MAX)
                    < assets.cast_duration(c.appearance).unwrap_or(Duration::ZERO)
            });
        }
        if let Some(ranged) = &mut self.ranged {
            // Flights use their wall-clock class3 cadence, not actor haste or
            // the current target. A per-flight projection/draw bound can omit
            // only that presentation; missing required art already failed load.
            let mut skipped = false;
            let assets = &ranged.assets;
            let canvas = &mut self.canvas;
            ranged.tracker.flights.retain(|flight| {
                if cancel.load(Ordering::Acquire) {
                    return false;
                }
                let elapsed = now.saturating_duration_since(flight.started);
                match assets.draw_effect(canvas, &flight.path, info.projection, elapsed) {
                    Ok(visible) => visible,
                    Err(_) => {
                        skipped = true;
                        false
                    }
                }
            });
            ranged.tracker.skipped |= skipped;
        }
        if let (Some(healing), Some(combat)) = (&mut self.healing, combat) {
            healing.update(combat, &self.map, now)?;
            for visual in &mut healing.active {
                let elapsed = now.saturating_duration_since(visual.started);
                let sample = visual.playback.advance(elapsed)?;
                ensure!(
                    sample.crossed_cues.is_empty(),
                    "qualified healing has no sound or hit cue"
                );
                let anchor = info
                    .projection
                    .cell_to_pixel(visual.position)
                    .context("healing projection overflow")?;
                healing
                    .assets
                    .draw_effect(&mut self.canvas, visual.effect, anchor, elapsed)?;
                visual.finished = sample.finished;
            }
            healing.active.retain(|v| !v.finished);
        }
        if let (Some(timed), Some(combat)) = (&mut self.timed, combat) {
            timed.update(combat, &self.map, now)?;
            for visual in &mut timed.active {
                let elapsed = now.saturating_duration_since(visual.started);
                let sample = visual.playback.advance(elapsed)?;
                if !sample.crossed_cues.is_empty() && elapsed <= Duration::from_millis(500) {
                    self.audio
                        .as_mut()
                        .context("timed audio absent")?
                        .push_preloaded_cues(
                            &sample.crossed_cues,
                            timed.assets.sound_bank(),
                            visual.started,
                        )?;
                }
                let anchor = info
                    .projection
                    .cell_to_pixel(visual.position)
                    .context("timed projection overflow")?;
                timed
                    .assets
                    .draw_effect(&mut self.canvas, visual.effect, anchor, elapsed)?;
                visual.finished = sample.finished;
            }
            timed.active.retain(|v| !v.finished);
        }
        if let Some(combat) = combat {
            draw_health_bars(&mut self.canvas, &info.actors, &combat.vitals)?;
            if let Some(d) = &combat.world.siege {
                for slot in super::siege_protocol::Slot::ALL {
                    let t = &d.objectives[slot as usize];
                    if t.hp == 0 {
                        continue;
                    }
                    let p = info
                        .objectives
                        .iter()
                        .find(|p| p.entity_id == super::siege_presentation::draw_id(slot))
                        .context("siege tower HP placement absent")?;
                    draw_compact_health_bar(
                        &mut self.canvas,
                        t.hp,
                        t.max_hp,
                        p.anchor,
                        p.top_left,
                    )?;
                }
            }
            if let Some(d) = combat.world.time_attack.as_ref().filter(|d| d.tower_hp > 0) {
                let p = info
                    .objectives
                    .iter()
                    .find(|p| p.entity_id == super::time_attack_presentation::DRAW_ID)
                    .context("cooperative tower HP placement absent")?;
                draw_compact_health_bar(
                    &mut self.canvas,
                    d.tower_hp,
                    d.tower_max_hp,
                    p.anchor,
                    p.top_left,
                )?;
            }
            if let Some(q) = &combat.world.quest {
                for actor in q.actors.iter().filter(|a| a.hp > 0) {
                    let target = q.target(actor.slot).context("Quest HP target absent")?;
                    let p = info
                        .objectives
                        .iter()
                        .find(|p| p.entity_id == super::quest_presentation::draw_id(actor.slot))
                        .context("Quest HP placement absent")?;
                    draw_compact_health_bar(
                        &mut self.canvas,
                        actor.hp,
                        target.max_hp,
                        p.anchor,
                        p.top_left,
                    )?;
                }
            }
            if let Some(h) = &combat.honor {
                for o in &h.objectives {
                    let placement = info
                        .objectives
                        .iter()
                        .find(|p| p.entity_id == o.entity_id)
                        .context("Honor placement absent")?;
                    draw_objective_health_bar(&mut self.canvas, placement, o)?;
                }
            }
        }
        if let Some(input) = npc {
            let frame = input
                .reply
                .frame
                .as_ref()
                .context("NPC HP snapshot absent")?;
            for placement in &info.npcs {
                let actor = frame
                    .actor(placement.id)
                    .context("NPC HP identity absent")?;
                ensure!(
                    actor.life == placement.life && actor.alive() == placement.alive,
                    "NPC HP life mismatch"
                );
                if actor.alive() {
                    draw_compact_health_bar(
                        &mut self.canvas,
                        actor.hp,
                        actor.max_hp,
                        placement.anchor,
                        placement.top_left,
                    )?;
                }
            }
        }
        ensure!(!cancel.load(Ordering::Acquire), "world rendering cancelled");
        self.canvas.copy_bgra(&mut self.bgra)?;
        ensure!(!cancel.load(Ordering::Acquire), "world rendering cancelled");
        let audio = match (&mut self.audio, combat) {
            (Some(audio), Some(combat)) => Some(audio.advance_npcs(
                &self.assets,
                snapshot,
                combat,
                npc.and_then(|n| n.reply.frame.as_ref()),
                now,
            )?),
            (None, None) => None,
            _ => anyhow::bail!("audio/world capability changed during rendering"),
        };
        let (objectives, siege_towers) = match &self.siege {
            Some(siege) => (
                Vec::new(),
                siege.placements(
                    &combat.context("siege combat absent")?.world,
                    info.objectives,
                )?,
            ),
            None => (info.objectives, Vec::new()),
        };
        let (objectives, time_attack_tower) = match &self.time_attack {
            Some(tower) => (
                Vec::new(),
                Some(tower.placement(
                    &combat.context("cooperative tower absent")?.world,
                    objectives,
                )?),
            ),
            None => (objectives, None),
        };
        let (objectives, quests) = match &self.quest {
            Some(quest) => (
                Vec::new(),
                quest.placements(&combat.context("Quest state absent")?.world, objectives)?,
            ),
            None => (objectives, Vec::new()),
        };
        Ok(Pixels {
            bgra: Arc::from(self.bgra.as_slice()),
            limitations: self.assets.limitations().clone(),
            actors: info.actors,
            objectives,
            quests,
            time_attack_tower,
            siege_towers,
            npcs: info.npcs,
            audio,
            presentation_notice: self
                .light_arrow
                .as_ref()
                .filter(|l| l.skipped_visual)
                .map(|_| {
                    "光箭超出有界視覺範圍／並行數量；該次飛行／音效略過，伺服器傷害與魔力不變。"
                })
                .or_else(|| {
                    self.healing
                        .as_ref()
                        .filter(|h| h.skipped)
                        .map(|_| "補血特效超出有界並行數；該次畫面略過，伺服器回血與補給數量不變。")
                })
                .or_else(|| {
                    self.timed.as_ref().filter(|t| t.skipped).map(|_| {
                        "限時用品特效超出有界並行數；該次畫面／聲音略過，權威效果與數量不變。"
                    })
                })
                .or_else(|| {
                    self.ranged.as_ref().filter(|r| r.tracker.skipped).map(|_| {
                        "箭飛行因動作被取代或超出有界視覺範圍略過；伺服器命中與彈藥消耗不變。"
                    })
                })
                .or_else(|| {
                    self.fire_storm
                        .as_ref()
                        .filter(|f| f.skipped)
                        .map(|_| "火風暴呈現超出有界並行數；該次視覺略過，權威傷害與魔力不變。")
                })
                .or_else(|| {
                    self.honor
                        .as_ref()
                        .filter(|h| h.skipped)
                        .map(|_| "守護塔治癒呈現超過並行上限；該次視覺略過，伺服器HP與MP不變。")
                })
                .or_else(|| {
                    self.disintegrate.as_ref().filter(|d| d.skipped).map(|_| {
                        "究極光裂術呈現超過並行上限；該次畫面／聲音略過，伺服器傷害與魔力不變。"
                    })
                }),
        })
    }
}

#[cfg(test)]
fn load_shared(
    root: &Path,
    frame: &Frame,
    combat: Option<&combat_protocol::Frame>,
    cancel: &AtomicBool,
) -> Result<Box<dyn Renderer>> {
    Ok(Box::new(load_shared_renderer(root, frame, combat, cancel)?))
}

#[cfg(test)]
fn load_shared_renderer(
    root: &Path,
    frame: &Frame,
    combat: Option<&combat_protocol::Frame>,
    cancel: &AtomicBool,
) -> Result<SharedRenderer> {
    load_shared_renderer_npcs(root, frame, combat, None, cancel)
}
fn load_shared_renderer_npcs(
    root: &Path,
    frame: &Frame,
    combat: Option<&combat_protocol::Frame>,
    npc: Option<&NpcInput>,
    cancel: &AtomicBool,
) -> Result<SharedRenderer> {
    ensure!(
        frame
            .descriptor
            .is_some_and(|d| d.has(world_protocol::MatchDescriptor::NPC))
            == npc.is_some(),
        "NPC resources need explicit preflight plan"
    );
    if let Some(input) = npc {
        input.plan.validate(&input.reply)?;
        ensure!(
            input
                .reply
                .frame
                .as_ref()
                .is_some_and(|n| n.combat.world == *frame && Some(&n.combat) == combat),
            "NPC preflight/world mismatch"
        );
    }
    let bindings: Vec<_> = frame
        .manifest
        .sectors
        .iter()
        .map(|sector| SectorBinding {
            key: SharedMapSectorKey {
                map_id: frame.manifest.map_id,
                sector_x: sector.x,
                sector_y: sector.y,
            },
            byte_length: sector.byte_length,
            sha256: sector.sha256,
        })
        .collect();
    // This loader never rebuilds expected hashes from the local bytes. Both
    // per-file SHA/length and the canonical full geometry digest come from TLS.
    let map = Arc::new(WorldMap::load_shared(root, &bindings, cancel)?);
    load_qualified_map_renderer(root, frame, combat, npc, cancel, map)
}
fn load_qualified_map_renderer(
    root: &Path,
    frame: &Frame,
    combat: Option<&combat_protocol::Frame>,
    npc: Option<&NpcInput>,
    cancel: &AtomicBool,
    map: Arc<WorldMap>,
) -> Result<SharedRenderer> {
    ensure!(
        map.canonical_digest() == frame.manifest.digest && map.map_id() == frame.manifest.map_id,
        "world geometry digest mismatch"
    );
    let appearances: Vec<_> = frame
        .actors
        .iter()
        .map(|actor| {
            let member = frame
                .member(actor.entity_id)
                .context("visible actor appearance absent")?;
            Ok(BaseAppearance {
                class: u8::try_from(member.class)?,
                sex: member.sex,
            })
        })
        .collect::<Result<_>>()?;
    // Exactly one pinned Text/list.spr read and parse per renderer construction.
    // Families borrow it only while qualifying their owned, bounded assets.
    let catalog = super::shared_action_catalog::SharedActionCatalog::load(root, cancel)?;
    let root = catalog.root();
    let pet = if let Some(race) = &frame.race {
        ensure!(
            combat.is_none() && frame.casts.is_empty(),
            "race must not depend on combat resources"
        );
        let forms = race
            .runners
            .iter()
            .map(|runner| {
                Ok((
                    runner.entity_id,
                    super::pet_animation::PetAppearance::new(u32::try_from(
                        runner.base_graphics_id,
                    )?)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let pet_forms = forms
            .values()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        Some(PetPresentation {
            assets: Arc::new(super::pet_animation::PetAssets::load_with_catalog(
                &catalog, &pet_forms, cancel,
            )?),
            identity: (frame.match_id, frame.generation, race.course_sha256),
            forms,
            actors: BTreeMap::new(),
            sequence: 0,
            sounds: std::collections::VecDeque::new(),
        })
    } else {
        None
    };
    let assets = if pet.is_some() {
        WorldSceneAssets::load_terrain_with_catalog(
            &catalog,
            map.clone(),
            SceneLimits::default(),
            cancel,
        )?
    } else if let Some(combat) = combat {
        let armed: Vec<_> = appearances
            .iter()
            .zip(&frame.actors)
            .map(|(appearance, actor)| {
                Ok(CombatAppearance {
                    appearance: *appearance,
                    weapon_type: combat
                        .vitals
                        .iter()
                        .find(|v| v.entity_id == actor.entity_id)
                        .context("combat stance absent")?
                        .weapon_type,
                })
            })
            .collect::<Result<_>>()?;
        WorldSceneAssets::load_combat_armed_with_catalog(
            &catalog,
            map.clone(),
            &armed,
            SceneLimits::default(),
            cancel,
        )?
    } else {
        WorldSceneAssets::load_with_catalog(
            &catalog,
            map.clone(),
            &appearances,
            SceneLimits::default(),
            cancel,
        )?
    };
    let mut audio = combat
        .map(|combat| AudioSequencer::load_shared(root, &assets, combat, cancel))
        .transpose()?;
    let teleport = if frame.casts.is_empty() {
        None
    } else {
        ensure!(combat.is_some(), "spells require qualified combat");
        for cast in &frame.casts {
            ensure!(
                cast.cast_count == 0 || map.standing(cast.from) && map.standing(cast.to),
                "initial teleport event outside bound geometry"
            );
        }
        Some(TeleportPresentation {
            assets: TeleportAssets::load_with_catalog(&catalog, &appearances, cancel)?,
            baseline: frame
                .casts
                .iter()
                .map(|cast| (cast.entity_id, cast.cast_revision))
                .collect(),
            active: BTreeMap::new(),
        })
    };
    let light_arrow = if let Some(combat) = combat.filter(|c| {
        !c.damages.is_empty()
            || !c.npc_spells.is_empty()
            || super::disintegrate_presentation::enabled(c)
    }) {
        let events = light_arrow_events(combat)?;
        for cast in &events {
            ensure!(
                cast.cast_count == 0 || map.standing(cast.from) && map.standing(cast.to),
                "initial damage spell outside bound geometry"
            );
        }
        Some(LightArrowPresentation {
            assets: LightArrowAssets::load_with_catalog(&catalog, &appearances, cancel)?,
            baseline: events
                .iter()
                .map(|c| (c.entity_id, c.cast_revision))
                .collect(),
            active: BTreeMap::new(),
            flights: Vec::new(),
            skipped_visual: false,
        })
    } else {
        None
    };
    ensure!(
        !cancel.load(Ordering::Acquire),
        "world resource loading cancelled"
    );
    let disintegrate = combat
        .filter(|f| super::disintegrate_presentation::enabled(f))
        .map(|f| {
            super::disintegrate_presentation::Presentation::load(
                root,
                f,
                &map,
                &light_arrow
                    .as_ref()
                    .context("Disintegrate requires qualified caster18")?
                    .assets,
                cancel,
            )
        })
        .transpose()?;
    let healing = if let Some(combat) = combat.filter(|c| !c.items.is_empty()) {
        for event in &combat.items {
            ensure!(
                event.use_count == 0 || map.standing(event.position),
                "initial healing outside geometry"
            );
        }
        Some(HealingPresentation {
            assets: super::healing_effect::HealingAssets::load_with_catalog(&catalog, cancel)?,
            baseline: combat
                .items
                .iter()
                .map(|s| (s.entity_id, s.use_revision))
                .collect(),
            active: Vec::new(),
            skipped: false,
        })
    } else {
        None
    };
    let timed = if let Some(combat) = combat.filter(|c| !c.timed.is_empty()) {
        Some(TimedPresentation {
            assets: super::timed_effect::TimedAssets::load_with_catalog(&catalog, cancel)?,
            baseline: combat
                .items
                .iter()
                .map(|s| (s.entity_id, s.use_revision))
                .collect(),
            active: Vec::new(),
            skipped: false,
        })
    } else {
        None
    };
    let ranged = if let Some(combat) = combat.filter(|c| !c.ammo.is_empty()) {
        ensure!(
            combat.ammo.len() == frame.actors.len(),
            "ranged world requires a complete ammunition roster"
        );
        let mut delays = BTreeMap::new();
        for (actor, appearance) in frame.actors.iter().zip(&appearances) {
            let vital = combat
                .vital(actor.entity_id)
                .context("ranged actor vital absent")?;
            if super::ammo_protocol::bow(vital.weapon_type) {
                let qualified = assets
                    .combat_animation_armed(*appearance, vital.weapon_type, BaseAction::Attack)
                    .context("required bow attack body missing")?;
                ensure!(
                    qualified.provenance().action_key == super::sprite_actions::ActionKey::new(21),
                    "required bow attack action is not21"
                );
                delays.insert(actor.entity_id, bow_first_hit_delay(qualified.animation())?);
            }
        }
        Some(RangedPresentation {
            assets: RangedAssets::load_with_catalog(&catalog, cancel)?,
            tracker: RangedTracker::new(combat, delays),
        })
    } else {
        None
    };
    ensure!(
        !cancel.load(Ordering::Acquire),
        "world resource loading cancelled"
    );
    let honor = combat
        .filter(|f| f.honor.is_some())
        .map(|f| HonorPresentation::load_with_catalog(&catalog, f, cancel))
        .transpose()?;
    let quest = frame
        .quest
        .as_ref()
        .map(|_| {
            ensure!(
                combat.is_some() && npc.is_some(),
                "Quest requires atomic NPC preflight"
            );
            super::quest_presentation::Presentation::load(&catalog, frame, &map, cancel)
        })
        .transpose()?;
    let fire_storm = if let Some(f) = combat.filter(|f| super::fire_storm_presentation::enabled(f))
    {
        validate_fire_geometry(f, &map)?;
        Some(FirePresentation {
            assets: super::fire_storm_effect::FireStormAssets::load_with_catalog(
                &catalog,
                &appearances,
                cancel,
            )?,
            identity: (f.world.match_id, f.world.generation),
            baseline: super::fire_storm_presentation::events(f)
                .iter()
                .map(|s| (s.entity_id, s.cast_revision))
                .collect(),
            active: BTreeMap::new(),
            effects: Vec::new(),
            skipped: false,
        })
    } else {
        None
    };
    let time_attack = frame
        .time_attack
        .as_ref()
        .map(|_| {
            ensure!(
                combat.is_some() && npc.is_some() && honor.is_none() && quest.is_none(),
                "cooperative tower requires exclusive atomic NPC preflight"
            );
            super::time_attack_presentation::Presentation::load(&catalog, frame, &map, cancel)
        })
        .transpose()?;
    let siege = frame
        .siege
        .as_ref()
        .map(|_| {
            ensure!(
                combat.is_some()
                    && npc.is_none()
                    && honor.is_none()
                    && quest.is_none()
                    && time_attack.is_none()
                    && pet.is_none(),
                "siege requires exclusive atomic preflight"
            );
            super::siege_presentation::Presentation::load(&catalog, frame, map.clone(), cancel)
        })
        .transpose()?;
    // No global/cache lifetime: the full AST and its pinned Text handles retire
    // before first-render ACK. Each completed family retains only used assets.
    if let Some(input) = npc {
        audio
            .as_mut()
            .context("NPC audio resources absent")?
            .pin_npc_baseline(input.reply.frame.as_ref().context("NPC baseline absent")?);
    }
    let npc = npc
        .map(|n| super::npc_presentation::Presentation::load(&catalog, n.plan.clone(), cancel))
        .transpose()?;
    drop(catalog);
    Ok(SharedRenderer {
        assets,
        roofs: super::world_roofs::Animation::default(),
        map,
        canvas: Canvas::new(WIDTH, HEIGHT)?,
        bgra: Vec::new(),
        audio,
        teleport,
        light_arrow,
        healing,
        timed,
        ranged,
        honor,
        quest,
        time_attack,
        siege,
        fire_storm,
        disintegrate,
        pet,
        npc,
    })
}

struct Job {
    identity: Identity,
    cancel: Arc<AtomicBool>,
    input: Arc<Mutex<Arc<Input>>>,
    output: Arc<Mutex<Option<Arc<RenderedWorld>>>>,
    handle: Option<JoinHandle<std::result::Result<(), &'static str>>>,
}

impl Job {
    fn spawn(
        root: PathBuf,
        initial: Arc<Input>,
        loader: Loader,
        preferences: Arc<super::preferences::SharedPreferences>,
    ) -> Result<Self> {
        let identity = initial.identity;
        let cancel = Arc::new(AtomicBool::new(false));
        let input = Arc::new(Mutex::new(initial.clone()));
        let output = Arc::new(Mutex::new(None));
        let worker_cancel = cancel.clone();
        let worker_input = input.clone();
        let worker_output = output.clone();
        let handle = thread::Builder::new()
            .name("lito-world-render".to_owned())
            .spawn(move || {
                let result = run_worker(
                    &root,
                    initial,
                    loader,
                    &worker_cancel,
                    &worker_input,
                    &worker_output,
                    &preferences,
                );
                // Cancellation is intentional retirement, never a resource
                // failure that can overwrite the successor match's status.
                if worker_cancel.load(Ordering::Acquire) {
                    Ok(())
                } else {
                    result
                }
            })?;
        Ok(Self {
            identity,
            cancel,
            input,
            output,
            handle: Some(handle),
        })
    }
    fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(handle) = &self.handle {
            handle.thread().unpark();
        }
    }
    fn finished(&self) -> bool {
        self.handle.as_ref().is_none_or(JoinHandle::is_finished)
    }
    fn finish(&mut self) -> std::result::Result<(), &'static str> {
        self.handle
            .take()
            .map_or(Ok(()), |h| h.join().unwrap_or(Err("world worker panicked")))
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel();
        // This handle is never detached, including panic/unwind/drop paths.
        let _ = self.finish();
    }
}

fn run_worker(
    root: &Path,
    initial: Arc<Input>,
    loader: Loader,
    cancel: &AtomicBool,
    input: &Mutex<Arc<Input>>,
    output: &Mutex<Option<Arc<RenderedWorld>>>,
    preferences: &super::preferences::SharedPreferences,
) -> std::result::Result<(), &'static str> {
    if cancel.load(Ordering::Acquire) {
        return Ok(());
    }
    let mut renderer = loader(
        root,
        &initial.frame,
        initial.combat.as_deref(),
        initial.npc.as_ref(),
        cancel,
    )
    .map_err(|error| {
        #[cfg(test)]
        if std::env::var_os("LITO_MODERN_WORLD_DIAGNOSTICS").is_some() {
            eprintln!("opt-in shared resource test diagnostics: {error:#}");
        }
        #[cfg(not(test))]
        let _ = error;
        "shared world resources could not be validated or decoded"
    })?;
    let navigation_map = renderer.navigation_map();
    let mut timeline = Timeline::new(initial);
    let mut gain_cache = super::audio_gain::GainCache::default();
    loop {
        if cancel.load(Ordering::Acquire) {
            return Ok(());
        }
        let cycle = Instant::now();
        let next = input
            .lock()
            .map_err(|_| "world input mailbox failed")?
            .clone();
        timeline
            .update(next)
            .map_err(|_| "world frame changed during resource loading")?;
        let snapshot = timeline
            .snapshot(cycle)
            .map_err(|_| "world actor presentation is invalid")?;
        let pixels = renderer
            .render_npcs(
                &snapshot,
                timeline.input.combat.as_deref(),
                timeline.input.npc.as_ref(),
                cancel,
            )
            .map_err(|error| {
                #[cfg(test)]
                if std::env::var_os("LITO_MODERN_WORLD_DIAGNOSTICS").is_some() {
                    eprintln!("opt-in shared render test diagnostics: {error:#}");
                }
                #[cfg(not(test))]
                let _ = error;
                "shared world rendering failed"
            })?;
        if cancel.load(Ordering::Acquire) {
            return Ok(());
        }
        if pixels.bgra.len() != PIXEL_BYTES {
            return Err("world renderer returned an invalid surface");
        }
        let current = &timeline.input;
        let gain = preferences.get().audible_gain(true);
        let (audio, audio_gain, audio_notice) = match pixels.audio {
            Some(original) => match gain_cache.apply(original.clone(), gain, cancel) {
                Ok(frame) => (Some(frame), Some(gain), None),
                Err(_) => (
                    Some(original),
                    None,
                    Some("音效音量轉換失敗，本批音效已靜音；遊戲仍可操作。"),
                ),
            },
            None => (None, None, None),
        };
        if cancel.load(Ordering::Acquire) {
            return Ok(());
        }
        let image = Arc::new(RenderedWorld {
            match_id: current.identity.match_id,
            generation: current.identity.generation,
            revision: current.frame.revision,
            selected_entity_id: current.identity.selected_entity_id,
            manifest_digest: current.frame.manifest.digest,
            width: WIDTH,
            height: HEIGHT,
            bgra: pixels.bgra,
            ready_to_ack: true,
            limitations: pixels.limitations,
            presentation_notice: pixels.presentation_notice.or(audio_notice),
            navigation_map: navigation_map.clone(),
            projection: ViewProjection {
                anchor_cell: current
                    .frame
                    .actors
                    .iter()
                    .find(|a| a.entity_id == current.identity.selected_entity_id)
                    .ok_or("selected world actor disappeared")?
                    .position,
                anchor_pixel: PixelPoint {
                    x: (WIDTH / 2) as i32,
                    y: (HEIGHT / 2) as i32,
                },
            },
            combat: current.combat.clone(),
            world: current.frame.clone(),
            actors: Arc::from(pixels.actors),
            objectives: Arc::from(pixels.objectives),
            quests: Arc::from(pixels.quests),
            time_attack_tower: pixels.time_attack_tower,
            siege_towers: Arc::from(pixels.siege_towers),
            npcs: Arc::from(pixels.npcs),
            npc: current.npc.as_ref().map(|n| n.reply.clone()),
            audio,
            audio_gain,
        });
        let old = output
            .lock()
            .map_err(|_| "world output mailbox failed")?
            .replace(image);
        drop(old);
        // <=20Hz, no unbounded catch-up if a render is slower than 50ms.
        if let Some(wait) = FRAME_INTERVAL.checked_sub(cycle.elapsed()) {
            thread::park_timeout(wait);
        }
    }
}

pub struct WorldRuntime {
    preferences: Arc<super::preferences::SharedPreferences>,
    root: PathBuf,
    loader: Loader,
    active: Option<Job>,
    retiring: Option<Job>,
    /// Latest accepted authority frame, also rejects stale updates before IO.
    desired: Option<Arc<Input>>,
    /// One coalescing UI-side mailbox; flushing uses try_lock exclusively.
    pending: Option<Arc<Input>>,
    published: Option<Arc<RenderedWorld>>,
    failed: Option<&'static str>,
    closing: bool,
    // The canonical player owns presentation after preflight. Keep the last
    // immutable proof, but join this renderer instead of drawing unused frames.
    preflight_finished: bool,
}
impl WorldRuntime {
    /// Does not inspect/open the path. The shared reader performs confinement
    /// on its worker; relative roots are rejected at this public boundary.
    pub fn new(root: PathBuf) -> Result<Self> {
        ensure!(root.is_absolute(), "world resource root must be absolute");
        Ok(Self::with_loader(
            root,
            Arc::new(|root, frame, combat, npc, cancel| {
                Ok(Box::new(load_shared_renderer_npcs(
                    root, frame, combat, npc, cancel,
                )?))
            }),
        ))
    }

    pub fn with_preferences(
        root: PathBuf,
        preferences: Arc<super::preferences::SharedPreferences>,
    ) -> Result<Self> {
        let mut runtime = Self::new(root)?;
        runtime.preferences = preferences;
        Ok(runtime)
    }

    fn with_loader(root: PathBuf, loader: Loader) -> Self {
        Self {
            preferences: Arc::new(super::preferences::SharedPreferences::default()),
            root,
            loader,
            active: None,
            retiring: None,
            desired: None,
            pending: None,
            published: None,
            failed: None,
            closing: false,
            preflight_finished: false,
        }
    }

    /// Test-only synthetic S32 arena used by the real Java siege peer. Shared
    /// SPR/Tile/WAV and the complete production compositor remain unchanged.
    #[cfg(test)]
    pub(crate) fn siege_fixture(root: PathBuf) -> Self {
        Self::with_loader(
            root,
            Arc::new(|root, frame, combat, npc, cancel| {
                ensure!(frame.siege.is_some() && npc.is_none(), "not siege fixture");
                let map = super::siege_space::tests::map(frame);
                Ok(Box::new(load_qualified_map_renderer(
                    root, frame, combat, npc, cancel, map,
                )?))
            }),
        )
    }

    pub fn update(&mut self, frame: Arc<Frame>, selected_entity_id: u64) -> Result<()> {
        self.update_combat(frame, selected_entity_id, None)
    }
    pub fn update_combat(
        &mut self,
        frame: Arc<Frame>,
        selected_entity_id: u64,
        combat: Option<Arc<combat_protocol::Frame>>,
    ) -> Result<()> {
        let input = Arc::new(Input::with_combat(frame, selected_entity_id, combat)?);
        self.accept(input)
    }
    /// Encounter owner must pin a plan before LOADED. No fallback to a generic
    /// combat frame: that would acknowledge without qualifying future NPC art.
    pub fn update_npc(
        &mut self,
        reply: Arc<super::npc_protocol::Reply>,
        selected_entity_id: u64,
        plan: Arc<super::npc_presentation::Plan>,
    ) -> Result<()> {
        ensure!(!self.closing, "world runtime is shutting down");
        plan.validate(&reply)?;
        let f = reply
            .frame
            .as_ref()
            .context("NPC rendering needs a snapshot")?;
        if f.closed {
            let old = self
                .desired
                .as_ref()
                .context("no current NPC encounter to retire")?;
            let previous = old
                .npc
                .as_ref()
                .context("NPC abort cannot retire another ruleset")?;
            ensure!(
                old.identity.selected_entity_id == selected_entity_id
                    && previous.plan == plan
                    && super::npc_protocol::follows(Some(&previous.reply), &reply),
                "stale/foreign NPC abort"
            );
            // No terminal corpse overlay or old READY surface survives an
            // authoritative abort. The owned worker is cancelled, not detached.
            self.clear();
            return Ok(());
        }
        let input = Arc::new(Input::with_npc(
            Arc::new(f.combat.world.clone()),
            selected_entity_id,
            Some(Arc::new(f.combat.clone())),
            Some(NpcInput { reply, plan }),
        )?);
        self.accept(input)
    }
    fn accept(&mut self, input: Arc<Input>) -> Result<()> {
        ensure!(!self.closing, "world runtime is shutting down");
        if let Some(old) = &self.desired {
            ensure!(
                old.identity.match_id != input.identity.match_id
                    || old.identity.generation != input.identity.generation
                    || old.frame.descriptor == input.frame.descriptor,
                "confirmed match descriptor cannot change within a world generation"
            );
            ensure!(
                old.identity.match_id != input.identity.match_id
                    || old.identity.generation != input.identity.generation
                    || old.combat.is_none()
                    || input.combat.is_some(),
                "confirmed combat cannot silently downgrade to absent data"
            );
            if old.identity == input.identity {
                ensure!(
                    follows(old, &input),
                    "stale or incompatible world presentation frame"
                );
                if old.frame == input.frame {
                    return Ok(());
                }
                ensure!(
                    self.failed.is_none(),
                    "failed world runtime requires a new match"
                );
            } else {
                self.published = None;
                self.failed = None;
                self.preflight_finished = false;
                if let Some(active) = &self.active {
                    active.cancel();
                }
            }
        }
        self.desired = Some(input.clone());
        // Still validate all authority transitions (including NPC plans).
        // Retirement never turns an old image into a newer command snapshot.
        self.pending = (!self.preflight_finished).then_some(input);
        Ok(())
    }

    /// Only after the server confirms this actor loaded may the canonical
    /// owner retire legacy drawing. This never sends or fabricates LOAD.
    pub(crate) fn finish_preflight(&mut self, expected: &Frame, selected: u64) -> bool {
        if self.closing || self.failed.is_some() {
            return false;
        }
        let (Some(input), Some(image)) = (&self.desired, &self.published) else {
            return false;
        };
        if input.frame.as_ref() != expected
            || input.identity.selected_entity_id != selected
            || !matches!(
                input.frame.phase,
                Phase::Countdown | Phase::Active | Phase::Finished
            )
            || !input
                .frame
                .actors
                .iter()
                .any(|a| a.entity_id == input.identity.selected_entity_id && a.loaded)
            || !image.ready_to_ack
            || !publishable(image, input, Some(image))
        {
            return false;
        }
        self.preflight_finished = true;
        self.pending = None;
        if let Some(job) = &self.active {
            job.cancel();
        }
        if let Some(job) = &self.retiring {
            job.cancel();
        }
        true
    }

    #[cfg(test)]
    pub(crate) fn preflight_finished(&self) -> bool {
        self.preflight_finished
    }

    /// Only short try_lock attempts and joins of already-finished workers.
    /// Call regularly while visible AND during shutdown, independent of focus.
    pub fn poll(&mut self) -> RuntimeStatus {
        self.retire_finished();
        if !self.closing && self.failed.is_none() {
            self.flush_pending();
            self.read_output();
        }
        if self.closing {
            return if self.is_quiescent() {
                RuntimeStatus::Idle
            } else {
                RuntimeStatus::Stopping
            };
        }
        if let Some(error) = self.failed {
            return RuntimeStatus::Failed(error);
        }
        if let Some(image) = &self.published {
            return RuntimeStatus::Ready(image.clone());
        }
        self.desired
            .as_ref()
            .map_or(RuntimeStatus::Idle, |input| RuntimeStatus::Loading {
                match_id: input.identity.match_id,
                generation: input.identity.generation,
            })
    }

    fn retire_finished(&mut self) {
        if self.retiring.as_ref().is_some_and(Job::finished) {
            if let Some(mut job) = self.retiring.take() {
                let _ = job.finish();
            }
        }
        if self.active.as_ref().is_some_and(Job::finished) {
            if let Some(mut job) = self.active.take() {
                let cancelled = job.cancel.load(Ordering::Acquire);
                let outcome = job.finish();
                if !cancelled && !self.closing {
                    self.failed = Some(outcome.err().unwrap_or("world worker exited unexpectedly"));
                    self.published = None;
                    self.pending = None;
                }
            }
        }
    }

    fn flush_pending(&mut self) {
        let Some(pending) = self.pending.clone() else {
            return;
        };
        if let Some(active) = &self.active {
            if active.identity != pending.identity || active.cancel.load(Ordering::Acquire) {
                active.cancel();
                if self.retiring.is_some() {
                    return;
                }
                self.retiring = self.active.take();
            }
        }
        if let Some(active) = &self.active {
            match active.input.try_lock() {
                Ok(mut mailbox) => {
                    let old = std::mem::replace(&mut *mailbox, pending);
                    drop(mailbox);
                    drop(old);
                    self.pending = None;
                }
                Err(TryLockError::WouldBlock) => {}
                Err(TryLockError::Poisoned(_)) => {
                    self.failed = Some("world input mailbox failed");
                    active.cancel();
                    self.published = None;
                }
            }
            return;
        }
        match Job::spawn(
            self.root.clone(),
            pending,
            self.loader.clone(),
            self.preferences.clone(),
        ) {
            Ok(job) => {
                self.active = Some(job);
                self.pending = None;
            }
            Err(_) => {
                self.failed = Some("world worker could not start");
                self.pending = None;
            }
        }
    }

    fn read_output(&mut self) {
        let (Some(active), Some(desired)) = (&self.active, &self.desired) else {
            return;
        };
        if active.identity != desired.identity || active.cancel.load(Ordering::Acquire) {
            return;
        }
        match active.output.try_lock() {
            Ok(output) => {
                if let Some(image) = output.as_ref() {
                    if publishable(image, desired, self.published.as_deref()) {
                        self.published = Some(image.clone());
                    }
                }
            }
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Poisoned(_)) => {
                self.failed = Some("world output mailbox failed");
                active.cancel();
                self.published = None;
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.closing = true;
        self.clear();
    }

    /// Leave/abort clears presentation but keeps this one bounded runtime
    /// reusable. Cancellation is immediate; handles remain owned until poll
    /// joins them. A successor is coalesced while both slots are retiring.
    pub fn clear(&mut self) {
        self.preflight_finished = false;
        self.pending = None;
        self.desired = None;
        self.published = None;
        self.failed = None;
        if let Some(job) = &self.active {
            job.cancel();
        }
        if let Some(job) = &self.retiring {
            job.cancel();
        }
    }

    /// True means all owned handles were joined, not merely cancellation sent.
    pub fn is_quiescent(&self) -> bool {
        self.active.is_none() && self.retiring.is_none() && self.pending.is_none()
    }
}
impl Drop for WorldRuntime {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(mut job) = self.active.take() {
            let _ = job.finish();
        }
        if let Some(mut job) = self.retiring.take() {
            let _ = job.finish();
        }
    }
}

#[cfg(test)]
mod ranged_tests {
    use super::super::{ammo_protocol as ammo, item_protocol, timed_protocol, weight_protocol};
    use super::*;
    #[test]
    fn siege_input_requires_atomic_combat_before_qualified_shared_preload() {
        let frame = super::super::siege_protocol::tests::sample("active")
            .frame
            .unwrap();
        assert!(Input::new(Arc::new(frame.world.clone()), 1).is_err());
        assert!(
            Input::with_combat(Arc::new(frame.world.clone()), 1, Some(Arc::new(frame))).is_ok()
        );
    }

    fn initial() -> combat_protocol::Frame {
        let mut f = tests::combat_frame();
        f.world.actors[1].position.x = f.world.actors[0].position.x + 6;
        f.vitals[0].weapon_type = 4;
        f.vitals[0].attack_interval_nanos = 200_000_000;
        f.ammo = f
            .world
            .actors
            .iter()
            .map(|actor| {
                ammo::State::initial(
                    actor.entity_id,
                    if actor.entity_id == 1 {
                        vec![ammo::Stock {
                            facts: ammo::Facts {
                                item_id: 40744,
                                small: 7,
                                large: 6,
                                raw_arrow_sprite: -1,
                                consume: true,
                                material: 0,
                                blessing: 0,
                            },
                            initial: 16,
                            remaining: 16,
                        }]
                    } else {
                        Vec::new()
                    },
                )
            })
            .collect();
        f.items = f
            .world
            .actors
            .iter()
            .map(|actor| item_protocol::ItemState {
                entity_id: actor.entity_id,
                request_sequence: 0,
                use_count: 0,
                use_revision: 0,
                item_id: 0,
                restored: 0,
                hp_before: 0,
                hp_after: 0,
                effect: 0,
                delay_millis: 0,
                position: WorldCell { x: 0, y: 0 },
                fixed_recovery: 0,
                stocks: Vec::new(),
            })
            .collect();
        f.timed = f
            .world
            .actors
            .iter()
            .map(|a| timed_protocol::State::initial(a.entity_id))
            .collect();
        f.weights = f
            .world
            .actors
            .iter()
            .map(|a| weight_protocol::ActorWeight {
                entity_id: a.entity_id,
                weight: weight_protocol::Snapshot {
                    source_gear_weight: 0,
                    supplies_weight: if a.entity_id == 1 { 1 } else { 0 },
                    total: if a.entity_id == 1 { 1 } else { 0 },
                    maximum_bits: 3150f64.to_bits(),
                    stage: 0,
                    hit_penalty: 0,
                    units: if a.entity_id == 1 {
                        vec![weight_protocol::UnitWeight {
                            item_id: 40744,
                            unit_weight: 47,
                        }]
                    } else {
                        Vec::new()
                    },
                },
            })
            .collect();
        f.validate().unwrap();
        f
    }
    fn input(f: &combat_protocol::Frame, at: Instant) -> Arc<Input> {
        let mut value =
            Input::with_combat(Arc::new(f.world.clone()), 1, Some(Arc::new(f.clone()))).unwrap();
        value.received = at;
        Arc::new(value)
    }
    fn shot(f: &combat_protocol::Frame, hit: bool) -> combat_protocol::Frame {
        let mut f = f.clone();
        f.world.revision += 1;
        let from = f.world.actors[0].position;
        let to = f.world.actors[1].position;
        let v = &mut f.vitals[0];
        v.attack_sequence += 1;
        v.attack_revision = f.world.revision;
        v.last_target_entity_id = 2;
        let a = &mut f.ammo[0];
        a.revision = f.world.revision;
        a.shot_count += 1;
        a.shot_revision = f.world.revision;
        a.shot_item_id = a.selected_item_id;
        a.attack_sequence = v.attack_sequence;
        a.last_target_entity_id = 2;
        a.from = from;
        a.to = to;
        a.projectile_sprite = 66;
        a.hit = hit;
        a.stocks[0].remaining -= 1;
        if hit {
            f.vitals[1].hp -= 3;
            f.vitals[1].hurt_revision = f.world.revision;
        }
        f.validate().unwrap();
        f
    }
    fn moved(f: &combat_protocol::Frame, index: usize, dx: i32) -> combat_protocol::Frame {
        let mut f = f.clone();
        f.world.revision += i64::from(dx.unsigned_abs().max(1));
        f.world.actors[index].sequence += u64::from(dx.unsigned_abs().max(1));
        f.world.actors[index].position.x += dx;
        f.validate().unwrap();
        f
    }
    fn tracker(f: &combat_protocol::Frame, delay: Duration) -> RangedTracker {
        RangedTracker::new(f, BTreeMap::from([(1, delay)]))
    }
    fn observe(
        timeline: &mut Timeline,
        tracker: &mut RangedTracker,
        f: &combat_protocol::Frame,
        at: Instant,
    ) {
        timeline.update(input(f, at)).unwrap();
        tracker
            .update(&timeline.snapshot(at).unwrap(), f, at)
            .unwrap();
    }
    #[test]
    fn historical_shot_requires_standable_geometry_even_inside_manifest() {
        // A complete, otherwise open synthetic S32 with the four cardinal
        // boundaries of local cell (24,16) closed. No asset or filesystem I/O.
        let mut bytes = vec![0; 49_170];
        for (x, y, edge) in [(24, 16, 0), (24, 16, 1), (23, 16, 1), (24, 17, 0)] {
            let offset = 32_770 + (y * 64 + x) * 4 + edge * 2;
            bytes[offset..offset + 2].copy_from_slice(&1i16.to_le_bytes());
        }
        let map = WorldMap::from_sector_bytes(
            vec![super::super::world_map::SectorInput {
                key: SharedMapSectorKey {
                    map_id: 514,
                    sector_x: 0x7fff,
                    sector_y: 0x8000,
                },
                bytes,
            }],
            &AtomicBool::new(false),
        )
        .unwrap();
        let blocked = WorldCell { x: 32728, y: 32784 };
        assert!(map.contains(blocked));
        assert!(!map.standing(blocked));
        let initial = initial();
        assert!(initial.ammo.iter().all(|state| state.shot_count == 0));
        validate_ranged_geometry(&initial, &map).unwrap();
        let historical = moved(&shot(&initial, false), 1, 1);
        assert!(historical.ammo[0].shot_revision < historical.world.revision);
        assert!(historical
            .world
            .actors
            .iter()
            .all(|actor| map.standing(actor.position)));
        validate_ranged_geometry(&historical, &map).unwrap();
        for replace_source in [false, true] {
            let mut invalid = historical.clone();
            if replace_source {
                invalid.ammo[0].from = blocked;
            } else {
                invalid.ammo[0].to = blocked;
            }
            // Structurally valid/coalesced wire history is not sufficient to
            // qualify a projectile source or endpoint on the shared map.
            invalid.validate().unwrap();
            assert!(validate_ranged_geometry(&invalid, &map).is_err());
        }
    }
    #[test]
    fn first_hit_excludes_terminal_ticks_preserves_cues_and_rejects_missing_or_duplicate_marker() {
        use super::super::sprite_actions::*;
        let frame = |ticks, events| ActionFrame {
            image_id: 216,
            subframe: 0,
            duration_ticks: ticks,
            events,
        };
        let mut animation = Animation {
            label_bytes: b"attack bow".to_vec(),
            directionality: Directionality::EightWay,
            frames_per_second: 24,
            frames: vec![
                frame(99, vec![]),
                frame(4, vec![]),
                frame(3, vec![FrameEvent::Sound(13)]),
                frame(3, vec![]),
                frame(3, vec![]),
                frame(6, vec![]),
                frame(2, vec![FrameEvent::Hit]),
            ],
        };
        assert_eq!(
            bow_first_hit_delay(&animation).unwrap(),
            Duration::from_nanos(791_666_667)
        );
        assert_eq!(animation.frames[2].events, [FrameEvent::Sound(13)]);
        animation.frames[0].events.push(FrameEvent::Hit);
        assert!(bow_first_hit_delay(&animation).is_err());
        animation.frames[0].events.clear();
        animation.frames[1].events.push(FrameEvent::Hit);
        assert!(bow_first_hit_delay(&animation).is_err());
        animation.frames[1].events.clear();
        animation.frames.last_mut().unwrap().events.clear();
        assert!(bow_first_hit_delay(&animation).is_err());
    }
    #[test]
    fn launch_uses_same_paced_body_phase_without_replaying_baseline_or_same_revision() {
        let f = initial();
        let now = Instant::now();
        let delay = Duration::from_millis(600);
        let fired = shot(&f, false);
        let mut late = tracker(&fired, delay);
        let mut late_timeline = Timeline::new(input(&fired, now));
        late.update(
            &late_timeline
                .snapshot(now + Duration::from_secs(2))
                .unwrap(),
            &fired,
            now + Duration::from_secs(2),
        )
        .unwrap();
        assert!(late.pending.is_empty() && late.flights.is_empty());
        let mut tracker = tracker(&f, delay);
        let mut timeline = Timeline::new(input(&f, now));
        observe(&mut timeline, &mut tracker, &fired, now);
        // Same PacedClock consumed by SceneSnapshot/actor art, not a second
        // projectile timer. This synthetic2x pace is not a native buff claim.
        timeline
            .actors
            .get_mut(&1)
            .unwrap()
            .body
            .retime(now, 2, 1)
            .unwrap();
        tracker
            .update(
                &timeline.snapshot(now + Duration::from_millis(299)).unwrap(),
                &fired,
                now + Duration::from_millis(299),
            )
            .unwrap();
        assert!(tracker.flights.is_empty());
        tracker
            .update(
                &timeline.snapshot(now + Duration::from_millis(300)).unwrap(),
                &fired,
                now + Duration::from_millis(300),
            )
            .unwrap();
        assert_eq!(tracker.flights.len(), 1);
        assert!(tracker.pending.is_empty());
        let started = tracker.flights[0].started;
        tracker
            .update(
                &timeline.snapshot(now + Duration::from_millis(310)).unwrap(),
                &fired,
                now + Duration::from_millis(310),
            )
            .unwrap();
        assert_eq!(tracker.flights.len(), 1);
        assert_eq!(tracker.flights[0].started, started);
        assert_eq!(
            tracker.flights[0].path.finished_at(),
            Duration::from_millis(150),
            "haste must not accelerate the native flight"
        );
        tracker
            .update(
                &timeline.snapshot(now + Duration::from_secs(2)).unwrap(),
                &fired,
                now + Duration::from_secs(2),
            )
            .unwrap();
        assert!(tracker.flights.is_empty());
        assert_eq!(tracker.baseline[&1], fired.ammo[0].shot_revision);
    }
    #[test]
    fn hit_snapshots_latest_confirmed_target_once_miss_keeps_event_endpoint_and_origin() {
        for hit in [false, true] {
            let f = initial();
            let now = Instant::now();
            let mut tracker = tracker(&f, Duration::from_millis(600));
            let mut timeline = Timeline::new(input(&f, now));
            let fired = shot(&f, hit);
            observe(&mut timeline, &mut tracker, &fired, now);
            let moved_target = moved(&fired, 1, 1);
            observe(
                &mut timeline,
                &mut tracker,
                &moved_target,
                now + Duration::from_millis(200),
            );
            tracker
                .update(
                    &timeline.snapshot(now + Duration::from_millis(600)).unwrap(),
                    &moved_target,
                    now + Duration::from_millis(600),
                )
                .unwrap();
            assert_eq!(tracker.flights.len(), 1);
            let frozen = tracker.flights[0].path.clone();
            assert_eq!(frozen.from(), fired.ammo[0].from);
            assert_eq!(
                frozen.target(),
                if hit {
                    moved_target.world.actors[1].position
                } else {
                    fired.ammo[0].to
                }
            );
            let moved_again = moved(&moved_target, 1, 1);
            observe(
                &mut timeline,
                &mut tracker,
                &moved_again,
                now + Duration::from_millis(620),
            );
            let moved_shooter = moved(&moved_again, 0, 1);
            observe(
                &mut timeline,
                &mut tracker,
                &moved_shooter,
                now + Duration::from_millis(630),
            );
            assert_eq!(
                tracker.flights[0].path, frozen,
                "neither actor movement can retarget a launched arrow"
            );
        }
    }
    #[test]
    fn next_shot_retains_previous_flight_and_hurt_only_cancels_an_unlaunched_body() {
        let mut f = initial();
        f.world.actors[1].position.x += 14;
        f.validate().unwrap();
        let now = Instant::now();
        let delay = Duration::from_millis(125);
        let mut tracker = tracker(&f, delay);
        let mut timeline = Timeline::new(input(&f, now));
        let first = shot(&f, false);
        observe(&mut timeline, &mut tracker, &first, now);
        tracker
            .update(
                &timeline.snapshot(now + delay).unwrap(),
                &first,
                now + delay,
            )
            .unwrap();
        let started = tracker.flights[0].started;
        let second = shot(&first, false);
        observe(
            &mut timeline,
            &mut tracker,
            &second,
            now + Duration::from_millis(210),
        );
        assert_eq!(tracker.flights.len(), 1);
        tracker
            .update(
                &timeline.snapshot(now + Duration::from_millis(335)).unwrap(),
                &second,
                now + Duration::from_millis(335),
            )
            .unwrap();
        assert_eq!(tracker.flights.len(), 2);
        assert_eq!(tracker.flights[0].started, started);
        let third = shot(&second, false);
        observe(
            &mut timeline,
            &mut tracker,
            &third,
            now + Duration::from_millis(420),
        );
        assert_eq!(tracker.pending.len(), 1);
        let mut hurt = third.clone();
        hurt.world.revision += 1;
        hurt.vitals[0].hp -= 1;
        hurt.vitals[0].hurt_revision = hurt.world.revision;
        observe(
            &mut timeline,
            &mut tracker,
            &hurt,
            now + Duration::from_millis(430),
        );
        assert!(tracker.pending.is_empty());
        assert_eq!(
            tracker.flights.len(),
            2,
            "already launched arrows outlive body interruption"
        );
        tracker
            .update(
                &timeline.snapshot(now + Duration::from_secs(2)).unwrap(),
                &hurt,
                now + Duration::from_secs(2),
            )
            .unwrap();
        assert!(tracker.flights.is_empty() && tracker.pending.is_empty());
    }
    #[test]
    fn latest_target_over_visual_bound_is_omitted_without_failing_or_retrying_world() {
        let mut f = initial();
        let sector = f.world.manifest.sectors[0].clone();
        f.world.manifest.sectors = (0..6)
            .map(|i| {
                let mut s = sector.clone();
                s.x += i;
                s
            })
            .collect();
        f.validate().unwrap();
        let now = Instant::now();
        let delay = Duration::from_millis(600);
        let mut tracker = tracker(&f, delay);
        let mut timeline = Timeline::new(input(&f, now));
        let fired = shot(&f, true);
        observe(&mut timeline, &mut tracker, &fired, now);
        let moved_far = moved(&fired, 1, 300);
        observe(
            &mut timeline,
            &mut tracker,
            &moved_far,
            now + Duration::from_millis(100),
        );
        tracker
            .update(
                &timeline.snapshot(now + delay).unwrap(),
                &moved_far,
                now + delay,
            )
            .unwrap();
        assert!(tracker.skipped && tracker.flights.is_empty() && tracker.pending.is_empty());
        tracker
            .update(
                &timeline.snapshot(now + Duration::from_secs(2)).unwrap(),
                &moved_far,
                now + Duration::from_secs(2),
            )
            .unwrap();
        assert!(tracker.flights.is_empty());
        assert_eq!(moved_far.vitals[1].hp, fired.vitals[1].hp);
        assert_eq!(moved_far.ammo[0].stocks, fired.ammo[0].stocks);
    }
    #[test]
    fn generation_and_actor_retirement_remove_old_pending_and_flights() {
        let f = initial();
        let now = Instant::now();
        let delay = Duration::from_millis(125);
        let mut tracker = tracker(&f, delay);
        let mut timeline = Timeline::new(input(&f, now));
        let fired = shot(&f, true);
        observe(&mut timeline, &mut tracker, &fired, now);
        let scene = timeline.snapshot(now + delay).unwrap();
        tracker.update(&scene, &fired, now + delay).unwrap();
        assert_eq!(tracker.flights.len(), 1);
        // A partial roster is not accepted by Input/transport; this redundant
        // retirement guard independently drops references before any draw.
        let mut retired = fired.clone();
        retired.world.actors.pop();
        retired.ammo.pop();
        tracker.update(&scene, &retired, now + delay).unwrap();
        assert!(tracker.flights.is_empty() && tracker.pending.is_empty());
        let mut next = fired.clone();
        next.world.generation += 1;
        tracker.update(&scene, &next, now + delay).unwrap();
        assert!(tracker.flights.is_empty() && tracker.pending.is_empty());
        assert_eq!(
            tracker.baseline[&1], fired.ammo[0].shot_revision,
            "new generation starts with a non-replaying watermark"
        );
    }
    #[test]
    fn final_hit_uses_confirmed_corpse_and_receipt_update_does_not_restart_arrow() {
        let f = initial();
        let now = Instant::now();
        let delay = Duration::from_millis(125);
        let mut tracker = tracker(&f, delay);
        let mut timeline = Timeline::new(input(&f, now));
        let mut finished = shot(&f, true);
        finished.world.phase = Phase::Finished;
        finished.vitals[1].hp = 0;
        finished.vitals[1].death_revision = finished.world.revision;
        let mut completion = super::super::combat_protocol::tests::finished_reply()
            .frame
            .unwrap()
            .world
            .completion
            .unwrap();
        completion.event.match_id = finished.world.match_id;
        completion.event.generation = finished.world.generation;
        completion.event.engine_revision = finished.world.revision - 1;
        finished.world.completion = Some(completion);
        finished.validate().unwrap();
        observe(&mut timeline, &mut tracker, &finished, now);
        let mut saved = finished.clone();
        saved.world.revision += 1;
        saved.world.completion.as_mut().unwrap().state = world_protocol::SettlementState::Saved;
        observe(
            &mut timeline,
            &mut tracker,
            &saved,
            now + Duration::from_millis(50),
        );
        tracker
            .update(
                &timeline.snapshot(now + delay).unwrap(),
                &saved,
                now + delay,
            )
            .unwrap();
        assert_eq!(tracker.flights.len(), 1);
        assert_eq!(
            tracker.flights[0].path.target(),
            saved.world.actors[1].position
        );
        assert_eq!(saved.vitals[1].hp, 0);
        let started = tracker.flights[0].started;
        tracker
            .update(
                &timeline.snapshot(now + Duration::from_millis(150)).unwrap(),
                &saved,
                now + Duration::from_millis(150),
            )
            .unwrap();
        assert_eq!(tracker.flights.len(), 1);
        assert_eq!(tracker.flights[0].started, started);
        let mut aborted = saved.clone();
        aborted.world.phase = Phase::Aborted;
        tracker
            .update(
                &timeline.snapshot(now + Duration::from_millis(160)).unwrap(),
                &aborted,
                now + Duration::from_millis(160),
            )
            .unwrap();
        assert!(tracker.flights.is_empty() && tracker.pending.is_empty());
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; real shared renderer arrow-layer isolation at fixed times, no output/audio device/DB/UI"]
    fn real_shared_arrow_layer_starts_at_bow_hit_then_retires_without_replay() {
        use std::collections::{BTreeSet, VecDeque};

        fn digest(hex: &str) -> [u8; 32] {
            std::array::from_fn(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap())
        }
        fn without_arrow(
            renderer: &mut SharedRenderer,
            scene: &SceneSnapshot,
            combat: &combat_protocol::Frame,
            cancel: &AtomicBool,
            now: Instant,
        ) -> Pixels {
            // Keep assets, pending Hit, immutable flights and watermark intact.
            // Only this layer is absent; HP/body/camera/frame/time are identical.
            let ranged = renderer.ranged.take().expect("qualified ranged layer");
            let result = renderer.render_at(scene, Some(combat), cancel, now);
            renderer.ranged = Some(ranged);
            result.unwrap()
        }
        fn audio_sequences(pixels: &Pixels) -> Vec<u64> {
            pixels
                .audio
                .as_ref()
                .unwrap()
                .events
                .iter()
                .map(|e| e.sequence)
                .collect()
        }

        let root =
            PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let cancel = AtomicBool::new(false);
        let mut source = initial();
        source.world.manifest.digest =
            digest("2feecc83de156a03c62e44effb7106067b24098a9ef7250cc471f7bf826da7a8");
        source.world.manifest.sectors[0].sha256 =
            digest("ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271");
        let origin = source.world.actors[0].position;
        source.world.actors[1].position = WorldCell {
            x: origin.x + 1,
            y: origin.y,
        };
        source.validate().unwrap();
        // Exact production loader: manifest/raw hashes, bow21 bodies/WAV and
        // all eight shared66 banks. No directly injected sprite or flight.
        let mut renderer =
            load_shared_renderer(&root, &source.world, Some(&source), &cancel).unwrap();
        assert_eq!(
            renderer.ranged.as_ref().unwrap().assets.encoded_bytes(),
            8_666
        );
        assert_eq!(
            renderer
                .ranged
                .as_ref()
                .unwrap()
                .assets
                .sprite_sources()
                .len(),
            8
        );

        // Derive a visible, connected target from this SHA-bound shared map,
        // not an assumed neighboring terrain cell. Four cells avoids native
        // adjacent/same-cell immediate retirement, which has no visible arrow.
        let mut queue = VecDeque::from([origin]);
        let mut visited = BTreeSet::from([origin]);
        let mut endpoint = None;
        while let Some(cell) = queue.pop_front() {
            let distance = (cell.x - origin.x).abs().max((cell.y - origin.y).abs());
            if distance == 4 {
                endpoint = Some(cell);
                break;
            }
            for (dx, dy) in super::super::world_map::DIRECTIONS {
                let next = WorldCell {
                    x: cell.x + dx,
                    y: cell.y + dy,
                };
                if (next.x - origin.x).abs().max((next.y - origin.y).abs()) <= 4
                    && renderer.map.can_step(cell, next)
                    && visited.insert(next)
                {
                    queue.push_back(next);
                }
            }
            assert!(visited.len() <= 81, "bounded shared-map fixture search");
        }
        let endpoint =
            endpoint.expect("shared spawn component needs a visible four-cell projectile");
        renderer
            .map
            .validate_spawns(&[origin, endpoint], &cancel)
            .unwrap();
        source.world.actors[1].position = endpoint;
        source.validate().unwrap();
        let expected_path = RangedPath::new(origin, endpoint).unwrap();
        let duration = expected_path.finished_at();
        assert!(duration >= Duration::from_millis(90));
        let hit_delay = renderer.ranged.as_ref().unwrap().tracker.launch_delays[&1];
        assert_eq!(
            hit_delay,
            Duration::from_nanos(666_666_667),
            "real class1 male bow21 Hit tick16"
        );
        let start = Instant::now();
        let mut timeline = Timeline::new(input(&source, start));
        let scene = timeline.snapshot(start).unwrap();
        renderer
            .render_at(&scene, Some(&source), &cancel, start)
            .unwrap();
        assert!(renderer.ranged.as_ref().unwrap().tracker.flights.is_empty());

        for (index, hit) in [false, true].into_iter().enumerate() {
            let event_start = start + Duration::from_secs(index as u64 * 3 + 1);
            source = shot(&source, hit);
            let confirmed = source.clone();
            timeline.update(input(&source, event_start)).unwrap();
            // This remains a genuine Timeline Attack triggered by a new
            // authoritative shot; never manufacture a flight in the test.
            for elapsed in [Duration::ZERO, hit_delay - Duration::from_nanos(1)] {
                let now = event_start + elapsed;
                let scene = timeline.snapshot(now).unwrap();
                assert_eq!(scene.actors()[0].action, BaseAction::Attack);
                assert!(scene.actors()[0].action_elapsed < hit_delay);
                let control = without_arrow(&mut renderer, &scene, &source, &cancel, now);
                let actual = renderer
                    .render_at(&scene, Some(&source), &cancel, now)
                    .unwrap();
                assert_eq!(
                    actual.bgra.as_ref(),
                    control.bgra.as_ref(),
                    "no arrow before qualified Hit"
                );
                assert_eq!(audio_sequences(&actual), audio_sequences(&control));
                let ranged = &renderer.ranged.as_ref().unwrap().tracker;
                assert_eq!(ranged.pending.len(), 1);
                assert!(ranged.flights.is_empty());
            }
            let launch = event_start + hit_delay;
            // Freeze this exact immutable scene for every control/actual pair:
            // no HP, hurt/attack frame, actor position or camera can create the
            // pixel difference asserted below.
            let scene = timeline.snapshot(launch).unwrap();
            assert_eq!(scene.actors()[0].action_elapsed, hit_delay);
            let mut reference = None::<Arc<[u8]>>;
            let mut placements = None::<Vec<ActorPlacement>>;
            for elapsed in [
                Duration::ZERO,
                Duration::from_millis(30),
                duration - Duration::from_nanos(1),
                duration,
                duration + Duration::from_nanos(1),
                Duration::from_secs(1),
            ] {
                let now = launch + elapsed;
                let control = without_arrow(&mut renderer, &scene, &source, &cancel, now);
                if let Some(reference) = &reference {
                    assert_eq!(
                        control.bgra.as_ref(),
                        reference.as_ref(),
                        "fixed HP/body/camera control changed"
                    );
                    assert_eq!(&control.actors, placements.as_ref().unwrap());
                } else {
                    reference = Some(control.bgra.clone());
                    placements = Some(control.actors.clone());
                }
                let actual = renderer
                    .render_at(&scene, Some(&source), &cancel, now)
                    .unwrap();
                assert_eq!(actual.bgra.len(), PIXEL_BYTES);
                assert_eq!(actual.actors, control.actors);
                assert!(actual.presentation_notice.is_none());
                assert_eq!(
                    audio_sequences(&actual),
                    audio_sequences(&control),
                    "arrow66 must not add a second audio owner"
                );
                if elapsed < duration {
                    assert_ne!(
                        actual.bgra.as_ref(),
                        control.bgra.as_ref(),
                        "real runtime arrow layer must contribute pixels during flight"
                    );
                } else {
                    assert_eq!(
                        actual.bgra.as_ref(),
                        control.bgra.as_ref(),
                        "native terminal0 must leave no arrow or invented impact"
                    );
                }
                let repeated = renderer
                    .render_at(&scene, Some(&source), &cancel, now)
                    .unwrap();
                assert_eq!(repeated.bgra.as_ref(), actual.bgra.as_ref());
                assert_eq!(audio_sequences(&repeated), audio_sequences(&actual));
                let ranged = &renderer.ranged.as_ref().unwrap().tracker;
                assert_eq!(ranged.baseline[&1], source.ammo[0].shot_revision);
                assert!(ranged.pending.is_empty());
                assert_eq!(ranged.flights.len(), usize::from(elapsed < duration));
                if let Some(flight) = ranged.flights.first() {
                    assert_eq!(
                        flight.started, launch,
                        "same revision must not restart flight"
                    );
                    assert_eq!(flight.path.from(), origin);
                    assert_eq!(flight.path.target(), endpoint);
                }
                assert_eq!(
                    source, confirmed,
                    "rendering never mutates server HP or ammo"
                );
            }
        }
    }
}

#[cfg(test)]
mod pet_tests {
    use super::*;
    use crate::modern_lito::{
        fixed_walk::QualifiedFixedWalk,
        pet_animation::PetAppearance,
        race_protocol as race,
        sprite_actions::{ActionLimits, SpriteActionCatalog},
    };

    fn scene(frame: &Frame, at: Instant) -> SceneSnapshot {
        let mut input = Input::new(Arc::new(frame.clone()), 1).unwrap();
        input.received = at;
        Timeline::new(Arc::new(input)).snapshot(at).unwrap()
    }
    #[test]
    fn pet_pacing_camera_has_one_cell_of_pulses_without_body_translation() {
        let cancel = AtomicBool::new(false);
        let from = WorldCell { x: 32768, y: 32849 };
        for count in [4, 8] {
            let frames = (0..count)
                .map(|i| format!("0.{i}:2>"))
                .collect::<Vec<_>>()
                .join(" ");
            let text = format!("#1 8 test\n0.walk(1 {count},{frames})\n");
            let catalog =
                SpriteActionCatalog::parse_for_inspection(text.as_bytes(), ActionLimits::default())
                    .unwrap();
            let walk = QualifiedFixedWalk::from_catalog(&catalog, 1).unwrap();
            for heading in 0..8 {
                let mut cursor = walk.start_unpaced_step(heading).unwrap();
                let first = cursor.advance(Duration::ZERO, &cancel).unwrap();
                assert_eq!(first.source_frame, 1);
                assert_eq!(first.local_camera_pulses.len(), 1);
                let mut offset = first.local_camera_pulses[0].displacement;
                assert!(cursor
                    .advance(Duration::ZERO, &cancel)
                    .unwrap()
                    .local_camera_pulses
                    .is_empty());
                let terminal = cursor.advance(walk.active_duration(), &cancel).unwrap();
                for p in terminal.local_camera_pulses {
                    offset.0 += p.displacement.0;
                    offset.1 += p.displacement.1;
                }
                let (dx, dy) = super::super::world_map::DIRECTIONS[heading as usize];
                let to = WorldCell {
                    x: from.x + dx,
                    y: from.y + dy,
                };
                let camera = pet_camera_projection(from, offset).unwrap();
                assert_eq!(
                    camera.cell_to_pixel(to),
                    Some(PixelPoint { x: 400, y: 300 })
                );
                assert_eq!(terminal.source_frame, 0);
                assert!(terminal.terminal && !terminal.finished);
                let interval = Duration::from_millis(200);
                assert_eq!(
                    pet_walk_elapsed(interval, interval, walk.step_duration()).unwrap(),
                    walk.step_duration()
                );
                assert_eq!(
                    pet_walk_elapsed(Duration::MAX, interval, walk.step_duration()).unwrap(),
                    walk.step_duration()
                );
                assert!(
                    cursor
                        .advance(walk.step_duration(), &cancel)
                        .unwrap()
                        .finished
                );
            }
        }
        assert!(
            pet_walk_elapsed(Duration::ZERO, Duration::ZERO, Duration::from_millis(1)).is_err()
        );
        assert!(pet_camera_projection(from, (i32::MIN, 0)).is_err());
    }
    #[test]
    fn pet_timeline_preserves_frozen_identity_but_only_competing_cells_occupy() {
        let at = Instant::now();
        let waiting = race::tests::waiting();
        let snapshot = scene(&waiting, at);
        assert_eq!(snapshot.pet().unwrap().runners.len(), 2);
        assert!(snapshot.pet().unwrap().runners.iter().all(|r| r.competing));
        let complete = race::tests::complete();
        let snapshot = scene(&complete, at);
        assert_eq!(snapshot.actors().len(), 2);
        assert_eq!(snapshot.selected_entity_id(), 1);
        assert!(snapshot.pet().unwrap().runners.iter().all(|r| !r.competing));
        let mut actors = snapshot.actors().to_vec();
        actors[1].cell = actors[0].cell;
        let pet = snapshot.pet().unwrap().clone();
        assert!(SceneSnapshot::new_pet(
            snapshot.world_id(),
            snapshot.revision(),
            snapshot.map_digest(),
            1,
            actors.clone(),
            pet.clone()
        )
        .is_ok());
        assert!(SceneSnapshot::new(
            snapshot.world_id(),
            snapshot.revision(),
            snapshot.map_digest(),
            1,
            actors.clone()
        )
        .is_err());
        let mut active = pet;
        for r in &mut active.runners {
            r.competing = true;
        }
        assert!(SceneSnapshot::new_pet(
            snapshot.world_id(),
            snapshot.revision(),
            snapshot.map_digest(),
            1,
            actors,
            active
        )
        .is_err());
    }
    #[test]
    fn pet_timeline_rejects_same_generation_form_or_course_changes() {
        let old = race::tests::running();
        let input = Input::new(Arc::new(old.clone()), 1).unwrap();
        for alter_form in [false, true] {
            let mut new = old.clone();
            new.revision += 1;
            if alter_form {
                for runner in &mut new.race.as_mut().unwrap().runners {
                    runner.base_graphics_id = 938;
                }
            } else {
                new.race.as_mut().unwrap().gates[0].min_x -= 1;
            }
            let hash = new.race.as_ref().unwrap().course_hash(&new);
            new.race.as_mut().unwrap().course_sha256 = hash;
            let next = Input::new(Arc::new(new), 1).unwrap();
            assert!(!follows(&input, &next));
        }
    }

    fn digest(s: &str) -> [u8; 32] {
        std::array::from_fn(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap())
    }
    /// Controlled renderer fixture, not a claim that this two-gate course is
    /// the accepted server race. Geometry is independently pinned shared 5143.
    fn actual_frame(form: PetAppearance, match_id: u64) -> Frame {
        let mut f = race::tests::waiting();
        f.match_id = match_id;
        f.manifest.map_id = 5143;
        f.manifest.digest =
            digest("f6c171c57eeae9e9223af51b12e89a1f0640086150abb41252e5e742c6feffde");
        f.manifest.sectors = [
            (
                0x7fff,
                0x8000,
                127474,
                "72f52a1f3dbc6db65650c4c470f254a6a1d94b4283dc928b78edb4921aacdb15",
            ),
            (
                0x7fff,
                0x8001,
                141309,
                "2712d11716d060cf220d3f33ab2c26a190bb4c333ab78e185eb24baf996effea",
            ),
            (
                0x8000,
                0x8000,
                111184,
                "c8f0ef53c400f2558e7239c21d092be10a42613fd2b74ef0fe9270d05a3e5cb8",
            ),
            (
                0x8000,
                0x8001,
                99133,
                "054ca18e1cb46f4084135fc5409bb692bb8e99b998c50579148f6b2059bcae0d",
            ),
        ]
        .into_iter()
        .map(|(x, y, byte_length, sha)| world_protocol::Sector {
            x,
            y,
            byte_length,
            sha256: digest(sha),
        })
        .collect();
        f.actors[0].position = WorldCell { x: 32768, y: 32849 };
        f.actors[1].position = WorldCell { x: 32768, y: 32848 };
        f.race.as_mut().unwrap().gates = vec![
            race::Gate {
                min_x: 32770,
                max_x: 32770,
                min_y: 32849,
                max_y: 32849,
            },
            race::Gate {
                min_x: 32772,
                max_x: 32772,
                min_y: 32849,
                max_y: 32849,
            },
        ];
        for runner in &mut f.race.as_mut().unwrap().runners {
            runner.base_graphics_id = i64::from(form.graphics_id());
        }
        let hash = f.race.as_ref().unwrap().course_hash(&f);
        f.race.as_mut().unwrap().course_sha256 = hash;
        f.validate(f.actors[0].key).unwrap();
        f
    }
    fn hidden(snapshot: &SceneSnapshot) -> SceneSnapshot {
        let mut pet = snapshot.pet().unwrap().clone();
        for runner in &mut pet.runners {
            runner.competing = false;
        }
        SceneSnapshot::new_pet(
            snapshot.world_id(),
            snapshot.revision(),
            snapshot.map_digest(),
            snapshot.selected_entity_id(),
            snapshot.actors().to_vec(),
            pet,
        )
        .unwrap()
    }
    // Independent binary-coverage reference. It does not call either blitter.
    fn oracle_blit(
        out: &mut [u8],
        frame: &super::super::sprite::SpriteFrame,
        anchor: PixelPoint,
        shadow: bool,
        opaque_shadow: bool,
    ) {
        // Native sprite origin is the upper-left of the actor's half-cell,
        // 24/12 pixels before its foot anchor. This is independent of the
        // production layer_position helper and leaves decoded pixels intact.
        assert!(
            frame.fragments.is_some(),
            "real world SPR must retain native fragments"
        );
        let anchor = PixelPoint {
            x: anchor.x - 24,
            y: anchor.y - 12,
        };
        for y in 0..frame.height as usize {
            for x in 0..frame.width as usize {
                let sx = i64::from(anchor.x) + i64::from(frame.x_offset) + x as i64;
                let sy = i64::from(anchor.y) + i64::from(frame.y_offset) + y as i64;
                if sx < 0 || sy < 0 || sx >= i64::from(WIDTH) || sy >= i64::from(HEIGHT) {
                    continue;
                }
                let src = &frame.rgba[(y * frame.width as usize + x) * 4..][..4];
                assert!(src[3] == 0 || src[3] == 255);
                if src[3] == 0 {
                    continue;
                }
                let dst = &mut out[(sy as usize * WIDTH as usize + sx as usize) * 4..][..4];
                for c in 0..3 {
                    dst[c] = if shadow && !opaque_shadow {
                        let five = ((u16::from(dst[c] / 8) + u16::from(src[c] / 8)) / 2) as u8;
                        five * 8 + five / 4
                    } else {
                        src[c]
                    };
                }
                dst[3] = 255;
            }
        }
    }
    /// Same fixed scene/camera/time. Only the specified selected-pet layer is
    /// removed; other pets and the complete shared static background stay fixed.
    fn assert_pet_layers(renderer: &mut SharedRenderer, snapshot: &SceneSnapshot, now: Instant) {
        let cancel = AtomicBool::new(false);
        let pixels = renderer.render_at(snapshot, None, &cancel, now).unwrap();
        let full = renderer.canvas.rgba().to_vec();
        let pet = renderer.pet.as_mut().unwrap();
        let (poses, camera, audio) = pet.update(snapshot, &renderer.map, now, &cancel).unwrap();
        assert_eq!(
            pixels
                .audio
                .unwrap()
                .events
                .iter()
                .map(|e| e.sequence)
                .collect::<Vec<_>>(),
            audio.events.iter().map(|e| e.sequence).collect::<Vec<_>>()
        );
        let mut background = Canvas::new(WIDTH, HEIGHT).unwrap();
        let info = renderer
            .assets
            .render_into_pets(&hidden(snapshot), &mut background, &cancel, &[], camera)
            .unwrap();
        let mut poses: Vec<_> = poses
            .iter()
            .map(|p| (info.projection.cell_to_pixel(p.anchor).unwrap(), p))
            .collect();
        poses.sort_by_key(|(a, p)| (a.y, a.x, p.entity_id));
        let mut controls = vec![background.rgba().to_vec(); 4];
        for (anchor, pose) in poses {
            let sample = pet
                .assets
                .frame(pose.appearance, pose.action, pose.heading, pose.source)
                .unwrap();
            for (mode, out) in controls.iter_mut().enumerate() {
                if !(mode == 2 && pose.entity_id == 1) {
                    oracle_blit(out, sample.shadow.frame, anchor, true, mode == 3);
                }
                if !(mode == 1 && pose.entity_id == 1) {
                    oracle_blit(out, sample.body.frame, anchor, false, false);
                }
            }
        }
        assert!(
            full == controls[0],
            "same-scene independent body/shadow oracle: differing bytes={}",
            full.iter()
                .zip(&controls[0])
                .filter(|(a, b)| a != b)
                .count()
        );
        assert_ne!(full, controls[1], "body must actually contribute pixels");
        assert_ne!(full, controls[2], "shadow must actually contribute pixels");
        assert_ne!(
            full, controls[3],
            "opaque gray shadow is not the reference blend"
        );
    }
    #[test]
    #[ignore = "requires explicit real shared resource root; renderer only, no GUI/SQL/audio device"]
    fn actual_shared_four_pet_renderers_layers_steps_and_scope_retirement() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT")
            .map(PathBuf::from)
            .expect("explicit shared root");
        let cancel = AtomicBool::new(false);
        for (index, form) in PetAppearance::ALL.into_iter().enumerate() {
            let mut f = actual_frame(form, 100 + index as u64);
            let mut renderer = load_shared_renderer(&root, &f, None, &cancel).unwrap();
            assert!(renderer.pet.is_some() && renderer.audio.is_none());
            assert_eq!(renderer.assets.combat_animations().count(), 0);
            let t = Instant::now();
            let base = scene(&f, t);
            assert_pet_layers(&mut renderer, &base, t);
            assert!(renderer.pet.as_ref().unwrap().sounds.is_empty());
            renderer
                .render_at(&base, None, &cancel, t + Duration::from_millis(550))
                .unwrap();
            assert_pet_layers(&mut renderer, &base, t + Duration::from_millis(600));
            let source_one_sound = renderer
                .pet
                .as_ref()
                .unwrap()
                .assets
                .action(form, super::super::pet_animation::PetAction::Breath)
                .unwrap()
                .sound_group(1)
                .unwrap()
                .is_some();
            assert_eq!(
                renderer.pet.as_ref().unwrap().sounds.len(),
                if source_one_sound { 2 } else { 0 }
            );
            let first_revision = renderer.pet.as_ref().unwrap().sequence;
            renderer
                .render_at(&base, None, &cancel, t + Duration::from_millis(600))
                .unwrap();
            assert_eq!(renderer.pet.as_ref().unwrap().sequence, first_revision);
            f.phase = Phase::Active;
            f.revision = 2;
            f.race.as_mut().unwrap().state = race::RaceState::Running;
            for actor in &mut f.actors {
                actor.loaded = true;
            }
            f.actors[0].sequence = 1;
            f.actors[0].position.x += 1;
            f.actors[0].heading = 2;
            let moved = scene(&f, t);
            let start = t + Duration::from_secs(3);
            assert_pet_layers(&mut renderer, &moved, start);
            let visual = &renderer.pet.as_ref().unwrap().actors[&1];
            assert_eq!(visual.walk.as_ref().unwrap().from, base.actors()[0].cell);
            assert_eq!(renderer.canvas.width(), WIDTH);
            let finish = start + Duration::from_nanos(f.actors[0].step_interval_nanos);
            let done = renderer.render_at(&moved, None, &cancel, finish).unwrap();
            assert!(renderer.pet.as_ref().unwrap().actors[&1].walk.is_none());
            let placement = done.actors.iter().find(|a| a.entity_id == 1).unwrap();
            assert_eq!(placement.source_frame, 0);
            assert_eq!(placement.anchor, PixelPoint { x: 400, y: 300 });
            assert_eq!(placement.graphics_id, form.graphics_id());
            // A jump over multiple acknowledgements never manufactures a route.
            f.revision += 1;
            f.actors[0].sequence += 2;
            f.actors[0].position = base.actors()[0].cell;
            let coalesced = scene(&f, t);
            renderer
                .render_at(&coalesced, None, &cancel, finish + Duration::from_secs(1))
                .unwrap();
            assert!(renderer.pet.as_ref().unwrap().actors[&1].walk.is_none());
            // Inactive roster remains as camera/reference but has no layer.
            let retired = hidden(&coalesced);
            let at = finish + Duration::from_secs(2);
            let pixels = renderer.render_at(&retired, None, &cancel, at).unwrap();
            assert!(pixels.actors.is_empty() && pixels.audio.unwrap().events.is_empty());
            let mut control = Canvas::new(WIDTH, HEIGHT).unwrap();
            renderer
                .assets
                .render_into_pets(&retired, &mut control, &cancel, &[], None)
                .unwrap();
            assert_eq!(renderer.canvas.rgba(), control.rgba());
            let mut foreign = retired.pet().unwrap().clone();
            foreign.generation += 1;
            let foreign = SceneSnapshot::new_pet(
                retired.world_id(),
                retired.revision(),
                retired.map_digest(),
                1,
                retired.actors().to_vec(),
                foreign,
            )
            .unwrap();
            assert!(renderer.render_at(&foreign, None, &cancel, at).is_err());
            let cancelled = AtomicBool::new(true);
            assert!(renderer.render_at(&retired, None, &cancelled, at).is_err());
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    #[test]
    fn item_effect_watermarks_are_per_actor_and_do_not_replay_same_revision_or_hide_regression() {
        use super::*;
        let mut marks = BTreeMap::from([(1, 4), (2, 4)]);
        assert!(advance_item_watermark(&mut marks, 1, 5).unwrap());
        assert!(advance_item_watermark(&mut marks, 2, 5).unwrap());
        assert!(!advance_item_watermark(&mut marks, 1, 5).unwrap());
        assert!(!advance_item_watermark(&mut marks, 2, 5).unwrap());
        assert!(advance_item_watermark(&mut marks, 1, 6).unwrap());
        assert!(advance_item_watermark(&mut marks, 1, 5).is_err());
        assert!(advance_item_watermark(&mut marks, 3, 7).is_err());
        assert_eq!(marks[&1], 6);
        let mut effects = Vec::new();
        for i in 0..MAX_DAMAGE_FLIGHTS {
            assert!(admit_flight(&mut effects, i));
        }
        assert!(!admit_flight(&mut effects, 999));
        assert_eq!(effects[0], 0);
    }
    use super::*;
    #[test]
    fn flight_admission_is_bounded_and_does_not_replace_existing_launches() {
        let mut flights = Vec::new();
        assert!(admit_flight(&mut flights, (1, 1)));
        assert!(
            admit_flight(&mut flights, (1, 2)),
            "same caster's next cast must not replace the first flight"
        );
        assert_eq!(flights, [(1, 1), (1, 2)]);
        for revision in 3..=MAX_DAMAGE_FLIGHTS {
            assert!(admit_flight(&mut flights, (1, revision)));
        }
        let before = flights.clone();
        assert!(!admit_flight(&mut flights, (1, MAX_DAMAGE_FLIGHTS + 1)));
        assert_eq!(flights, before);
        flights.retain(|(_, revision)| *revision != 1);
        assert!(admit_flight(&mut flights, (2, MAX_DAMAGE_FLIGHTS + 1)));
        assert_eq!(flights[0], (1, 2));
    }
    use crate::modern_lito::{
        application_protocol::CharacterKey,
        rooms::{Member, PublicDraft, RoomSnapshot, Summary},
        world_map::WorldCell,
        world_protocol::{AbortReason, Actor, Manifest, Sector},
    };
    use std::sync::atomic::AtomicUsize;

    pub(crate) fn combat_frame() -> combat_protocol::Frame {
        let mut world = frame(1);
        world.phase = Phase::Active;
        for actor in &mut world.actors {
            actor.loaded = true;
        }
        let vitals = world
            .actors
            .iter()
            .map(|a| combat_protocol::Vital {
                entity_id: a.entity_id,
                hp: 5_000_000_000,
                max_hp: 5_000_000_000,
                mp: 300,
                max_mp: 300,
                ac: -100,
                weapon_type: 0,
                attack_sequence: 0,
                attack_revision: 0,
                hurt_revision: 0,
                death_revision: 0,
                last_target_entity_id: 0,
                attack_interval_nanos: 1_000_000_000,
            })
            .collect();
        combat_protocol::Frame {
            world,
            vitals,
            damages: Vec::new(),
            items: Vec::new(),
            weights: Vec::new(),
            timed: Vec::new(),
            ammo: Vec::new(),
            regen: Vec::new(),
            honor: None,
            fires: Vec::new(),
            npc_spells: Vec::new(),
            disintegrations: Vec::new(),
            npc_disintegrations: Vec::new(),
            npc_fires: Vec::new(),
        }
    }
    pub(crate) fn match_descriptor(combat: bool) -> world_protocol::MatchDescriptor {
        use world_protocol::{MatchDescriptor, MatchRuleset};
        MatchDescriptor {
            ruleset: if combat {
                MatchRuleset::PvpSkirmish
            } else {
                MatchRuleset::SceneOnly
            },
            features: if combat { MatchDescriptor::COMBAT } else { 0 },
            preparation_timeout_millis: 15_000,
            load_timeout_millis: 10_000,
            countdown_nanos: 3_000_000_001,
            active_timeout_millis: 150_123,
        }
    }

    #[test]
    fn descriptor_requires_exact_combat_presence_before_starting_presentation() {
        let mut combat = combat_frame();
        combat.world.descriptor = Some(match_descriptor(true));
        assert!(Input::with_combat(Arc::new(combat.world.clone()), 1, None).is_err());
        assert!(Input::with_combat(
            Arc::new(combat.world.clone()),
            1,
            Some(Arc::new(combat.clone()))
        )
        .is_ok());
        combat.world.descriptor = Some(match_descriptor(false));
        assert!(Input::with_combat(
            Arc::new(combat.world.clone()),
            1,
            Some(Arc::new(combat.clone()))
        )
        .is_err());
        assert!(Input::with_combat(Arc::new(combat.world.clone()), 1, None).is_ok());
        combat.world.descriptor = None;
        assert!(
            Input::with_combat(Arc::new(combat.world.clone()), 1, None).is_ok(),
            "legacy absence keeps its prior semantics"
        );
    }

    #[test]
    fn immutable_descriptor_cannot_bypass_guards_by_replacing_the_render_worker() {
        let counts = Arc::new(Counts::default());
        let mut runtime = runtime(fake_loader(counts.clone(), None));
        let mut scene = frame(1);
        scene.descriptor = Some(match_descriptor(false));
        runtime.update(Arc::new(scene.clone()), 1).unwrap();
        let retained = runtime.desired.clone().unwrap();
        for strip in [false, true] {
            let mut bad = scene.clone();
            bad.revision += 1;
            if strip {
                bad.descriptor = None;
            } else {
                bad.descriptor.as_mut().unwrap().active_timeout_millis += 1;
            }
            assert!(!follows(
                &retained,
                &Input::new(Arc::new(bad.clone()), 1).unwrap()
            ));
            assert!(runtime.update(Arc::new(bad), 1).is_err());
            assert!(Arc::ptr_eq(&retained, runtime.desired.as_ref().unwrap()));
        }
        let mut combat = combat_frame();
        combat.world.descriptor = Some(match_descriptor(true));
        assert!(
            runtime
                .update_combat(
                    Arc::new(combat.world.clone()),
                    1,
                    Some(Arc::new(combat.clone()))
                )
                .is_err(),
            "combat_art identity change must not bypass same-match descriptor freeze"
        );
        assert!(Arc::ptr_eq(&retained, runtime.desired.as_ref().unwrap()));
        combat.world.match_id = 2;
        runtime
            .update_combat(
                Arc::new(combat.world.clone()),
                1,
                Some(Arc::new(combat.clone())),
            )
            .unwrap();
        scene.match_id = 3;
        runtime.update(Arc::new(scene), 1).unwrap();
        assert!(runtime.desired.as_ref().unwrap().combat.is_none());
        assert_eq!(
            counts.loaded.load(Ordering::Acquire),
            0,
            "rejected updates never launch a resource read"
        );
        runtime.clear();
        assert!(runtime.is_quiescent());
    }
    #[test]
    fn a_new_match_can_replace_all_binding_clocks_without_reusing_the_old_descriptor() {
        let counts = Arc::new(Counts::default());
        let mut runtime = runtime(fake_loader(counts.clone(), None));
        let mut first = frame(1);
        first.descriptor = Some(match_descriptor(false));
        runtime.update(Arc::new(first.clone()), 1).unwrap();
        let retained = runtime.desired.clone().unwrap();
        let mut next = first.clone();
        next.match_id += 1;
        next.generation += 1;
        next.room.summary.revision += 1;
        let clocks = next.descriptor.as_mut().unwrap();
        clocks.load_timeout_millis = 25_000;
        clocks.countdown_nanos = 1;
        clocks.active_timeout_millis = 6_000;
        next.validate(next.actors[0].key).unwrap();
        runtime.update(Arc::new(next.clone()), 1).unwrap();
        let desired = runtime.desired.as_ref().unwrap();
        assert!(!Arc::ptr_eq(&retained, desired));
        assert_eq!(desired.frame.descriptor, next.descriptor);
        assert_ne!(desired.frame.descriptor, retained.frame.descriptor);
        assert_eq!(retained.frame.descriptor, first.descriptor);
        assert_eq!(
            counts.loaded.load(Ordering::Acquire),
            0,
            "this pure admission test does not read shared assets"
        );
        runtime.clear();
        assert!(runtime.is_quiescent());
    }

    /// Synthetic authority frames for presentation-only regression tests.
    /// Every HP/MP change has an exact v12 ledger, without resource or DB I/O.
    pub(crate) fn natural_regen_frames() -> (
        combat_protocol::Frame,
        combat_protocol::Frame,
        combat_protocol::Frame,
    ) {
        let mut before = combat_frame();
        before.world.revision = 3;
        for vital in &mut before.vitals {
            vital.hp = 100;
            vital.max_hp = 100;
            vital.mp = 20;
            vital.max_mp = 20;
        }
        before.regen = before
            .vitals
            .iter()
            .map(|v| super::super::regen_protocol::State::initial(v.entity_id))
            .collect();
        before.vitals[0].hp = 80;
        before.vitals[0].hurt_revision = 2;
        before.regen[0].damage_loss = 20;
        before.vitals[0].mp = 18;
        before.regen[0].mp_loss = 2;
        before.regen[0].mp_revision = 3;
        before.regen[0].mp_before = 20;
        before.regen[0].mp_after = 18;
        before.vitals[1].attack_sequence = 1;
        before.vitals[1].attack_revision = 2;
        before.vitals[1].last_target_entity_id = 1;
        before.validate().unwrap();
        let mut attacked = before.clone();
        attacked.world.revision = 4;
        attacked.vitals[0].attack_sequence = 1;
        attacked.vitals[0].attack_revision = 4;
        attacked.vitals[0].last_target_entity_id = 2;
        attacked.vitals[1].hp = 90;
        attacked.vitals[1].hurt_revision = 4;
        attacked.regen[1].damage_loss = 10;
        attacked.validate().unwrap();
        let mut recovered = attacked.clone();
        recovered.world.revision = 5;
        recovered.vitals[0].hp = 85;
        recovered.vitals[0].mp = 19;
        recovered.regen[0].hp_gain = 5;
        recovered.regen[0].hp_revision = 5;
        recovered.regen[0].hp_before = 80;
        recovered.regen[0].hp_after = 85;
        recovered.regen[0].mp_gain = 1;
        recovered.regen[0].mp_revision = 5;
        recovered.regen[0].mp_before = 18;
        recovered.regen[0].mp_after = 19;
        recovered.validate().unwrap();
        (before, attacked, recovered)
    }
    #[test]
    fn natural_recovery_updates_health_without_restarting_confirmed_body_phase() {
        let (before, attacked, recovered) = natural_regen_frames();
        let start = Instant::now();
        let attack_at = start + Duration::from_millis(100);
        let regen_at = start + Duration::from_millis(200);
        let mut timeline = Timeline::new(combat_input(before, start));
        timeline
            .update(combat_input(attacked.clone(), attack_at))
            .unwrap();
        let original = timeline
            .snapshot(attack_at + Duration::from_millis(30))
            .unwrap();
        assert_eq!(original.actors()[0].action, BaseAction::Attack);
        assert_eq!(
            original.actors()[0].action_elapsed,
            Duration::from_millis(30)
        );
        let started = timeline.actors[&1].action_started;
        let hp_before = timeline.input.combat.as_ref().unwrap().vitals[0].hp;
        timeline
            .update(combat_input(recovered.clone(), regen_at))
            .unwrap();
        let after = timeline
            .snapshot(regen_at + Duration::from_millis(50))
            .unwrap();
        assert_eq!(timeline.actors[&1].action_started, started);
        assert_eq!(after.actors()[0].action, BaseAction::Attack);
        assert_eq!(after.actors()[0].action_elapsed, Duration::from_millis(150));
        assert_eq!(after.actors()[0].cell, original.actors()[0].cell);
        assert_eq!(after.actors()[0].heading, original.actors()[0].heading);
        assert_eq!(
            timeline.input.combat.as_ref().unwrap().vitals[0].hp,
            hp_before + 5
        );
        assert_eq!(timeline.input.combat.as_ref().unwrap().vitals[0].mp, 19);
        assert_eq!(combat_pose(&attacked.vitals[0], &recovered.vitals[0]), None);
        assert_eq!(combat_pose(&attacked.vitals[1], &recovered.vitals[1]), None);
        assert_eq!(attacked.items, recovered.items);
        assert_eq!(attacked.world.casts, recovered.world.casts);
        assert_eq!(attacked.damages, recovered.damages);
        assert_eq!(attacked.ammo, recovered.ammo);
        // The same new authority revision submitted twice cannot reset time.
        timeline
            .update(combat_input(
                recovered,
                regen_at + Duration::from_millis(60),
            ))
            .unwrap();
        assert_eq!(
            timeline
                .snapshot(regen_at + Duration::from_millis(70))
                .unwrap()
                .actors()[0]
                .action_elapsed,
            Duration::from_millis(170)
        );
    }
    #[test]
    fn healing_changes_atomic_hp_without_inventing_damage_attack_or_movement_pose() {
        let before = super::super::item_protocol::tests::initial();
        let start = Instant::now();
        let mut timeline = Timeline::new(combat_input(before.frame.clone().unwrap(), start));
        let after = super::super::item_protocol::tests::used(&before)
            .frame
            .unwrap();
        timeline
            .update(combat_input(
                after.clone(),
                start + Duration::from_millis(10),
            ))
            .unwrap();
        let first = timeline
            .snapshot(start + Duration::from_millis(20))
            .unwrap();
        assert_eq!(first.actors()[0].action, BaseAction::Idle);
        assert_eq!(first.actors()[0].cell, after.world.actors[0].position);
        assert!(timeline.actors[&1].pose.is_none());
        assert!(timeline.actors[&1].walk_until.is_none());
        let clock = timeline.actors[&1].action_started;
        timeline
            .update(combat_input(after, start + Duration::from_millis(30)))
            .unwrap();
        assert_eq!(timeline.actors[&1].action_started, clock);
    }

    fn combat_input(frame: combat_protocol::Frame, received: Instant) -> Arc<Input> {
        let mut input =
            Input::with_combat(Arc::new(frame.world.clone()), 1, Some(Arc::new(frame))).unwrap();
        input.received = received;
        Arc::new(input)
    }

    #[test]
    fn teleport_revision_changes_position_without_walking_and_preserves_same_event_clock() {
        let start = Instant::now();
        let original = super::super::spell_protocol::tests::initial();
        let mut timeline = Timeline::new(combat_input(original.frame.clone().unwrap(), start));
        let cast = super::super::spell_protocol::tests::success(&original);
        let after = cast.frame.unwrap();
        timeline
            .update(combat_input(
                after.clone(),
                start + Duration::from_millis(10),
            ))
            .unwrap();
        let snapshot = timeline
            .snapshot(start + Duration::from_millis(20))
            .unwrap();
        assert_eq!(snapshot.actors()[0].action, BaseAction::Idle);
        assert_eq!(snapshot.actors()[0].cell, after.world.casts[0].to);
        assert!(timeline.actors[&1].walk_until.is_none());
        timeline
            .update(combat_input(after.clone(), start + Duration::from_secs(1)))
            .unwrap();
        assert_eq!(
            timeline.actors[&1].action_started,
            start + Duration::from_millis(10)
        );
        let mut bad = after;
        bad.world.revision += 1;
        bad.world.actors[0].position.x += 1;
        assert!(timeline
            .update(combat_input(bad, start + Duration::from_secs(2)))
            .is_err());
    }

    #[test]
    fn coalesced_attack_then_cast_clears_old_pose_even_if_two_casts_return_to_same_cell() {
        let original = super::super::spell_protocol::tests::initial();
        let start = Instant::now();
        for return_to_origin in [false, true] {
            let mut after = original.frame.clone().unwrap();
            after.world.revision = 4;
            after.vitals[0].attack_sequence = 1;
            after.vitals[0].attack_revision = 2;
            after.vitals[0].last_target_entity_id = 2;
            let from = after.world.actors[0].position;
            let mut destination = from;
            destination.y += 2;
            after.world.casts[0] = CastState {
                entity_id: 1,
                request_sequence: if return_to_origin { 2 } else { 1 },
                cast_count: if return_to_origin { 2 } else { 1 },
                cast_revision: 4,
                mp_spent: if return_to_origin { 20 } else { 10 },
                skill_id: 5,
                from: if return_to_origin { destination } else { from },
                to: if return_to_origin { from } else { destination },
                cast_interval_millis: 1000,
            };
            after.world.actors[0].position = after.world.casts[0].to;
            after.vitals[0].mp -= after.world.casts[0].mp_spent;
            let mut timeline = Timeline::new(combat_input(original.frame.clone().unwrap(), start));
            timeline
                .update(combat_input(after, start + Duration::from_millis(20)))
                .unwrap();
            assert!(timeline.actors[&1].pose.is_none());
            assert!(timeline.actors[&1].walk_until.is_none());
            assert_eq!(
                timeline
                    .snapshot(start + Duration::from_secs(3))
                    .unwrap()
                    .actors()[0]
                    .action,
                BaseAction::Idle
            );
        }
    }

    #[test]
    fn interim_world_only_ack_cannot_discard_combat_history_or_start_replacement_loader() {
        let counts = Arc::new(Counts::default());
        let mut runtime = runtime(fake_loader(counts.clone(), None));
        let combat = Arc::new(combat_frame());
        runtime
            .update_combat(Arc::new(combat.world.clone()), 1, Some(combat.clone()))
            .unwrap();
        let retained = runtime.desired.clone().unwrap();
        let mut world_only = combat.world.clone();
        world_only.revision += 1;
        assert!(runtime.update(Arc::new(world_only), 1).is_err());
        assert!(Arc::ptr_eq(&retained, runtime.desired.as_ref().unwrap()));
        assert_eq!(counts.loaded.load(Ordering::Acquire), 0);
        runtime.clear();
        assert!(runtime.is_quiescent());
    }
    #[test]
    fn stationary_damage_spell_overrides_older_melee_but_not_newer_death_or_teleport() {
        let initial = super::super::spell_protocol::tests::initial_damage();
        let start = Instant::now();
        let mut prior = initial.clone();
        let f = prior.frame.as_mut().unwrap();
        f.world.revision = 2;
        f.vitals[0].attack_sequence = 1;
        f.vitals[0].attack_revision = 2;
        f.vitals[0].last_target_entity_id = 2;
        let hit = super::super::spell_protocol::tests::damage_success(&prior)
            .combat_reply()
            .unwrap();
        let mut timeline = Timeline::new(combat_input(initial.frame.unwrap(), start));
        timeline
            .update(combat_input(
                hit.frame.clone().unwrap(),
                start + Duration::from_millis(20),
            ))
            .unwrap();
        let image = timeline
            .snapshot(start + Duration::from_millis(40))
            .unwrap();
        assert_eq!(
            image.actors()[0].action,
            BaseAction::Idle,
            "body18 is an independent override; do not replay stale attack audio"
        );
        assert!(timeline.actors[&1].walk_until.is_none());
        assert_eq!(image.actors()[1].action, BaseAction::Damage);
        let tp = super::super::spell_protocol::tests::success(&hit)
            .combat_reply()
            .unwrap();
        timeline
            .update(combat_input(
                tp.frame.clone().unwrap(),
                start + Duration::from_millis(60),
            ))
            .unwrap();
        assert!(timeline.actors[&1].walk_until.is_none());
        assert_eq!(
            timeline
                .snapshot(start + Duration::from_millis(80))
                .unwrap()
                .actors()[0]
                .cell,
            tp.frame.as_ref().unwrap().world.casts[0].to
        );
        let mut killed = tp.frame.unwrap();
        killed.world.revision += 1;
        killed.vitals[0].hp = 0;
        killed.vitals[0].hurt_revision = killed.world.revision;
        killed.vitals[0].death_revision = killed.world.revision;
        timeline
            .update(combat_input(killed, start + Duration::from_millis(100)))
            .unwrap();
        assert_eq!(
            timeline
                .snapshot(start + Duration::from_secs(3))
                .unwrap()
                .actors()[0]
                .action,
            BaseAction::Death
        );
    }

    #[test]
    fn combat_timeline_uses_server_revisions_preserves_same_revision_and_holds_death() {
        let start = Instant::now();
        let original = combat_frame();
        let mut timeline = Timeline::new(combat_input(original.clone(), start));
        assert_eq!(
            timeline.snapshot(start).unwrap().actors()[0].action,
            BaseAction::Idle
        );
        let mut attacked = original;
        attacked.world.revision += 1;
        attacked.vitals[0].attack_revision = attacked.world.revision;
        attacked.vitals[0].attack_sequence = 1;
        attacked.vitals[0].last_target_entity_id = 2;
        attacked.vitals[1].hurt_revision = attacked.world.revision;
        attacked.vitals[1].hp -= 17;
        timeline
            .update(combat_input(
                attacked.clone(),
                start + Duration::from_millis(10),
            ))
            .unwrap();
        let snapshot = timeline
            .snapshot(start + Duration::from_millis(110))
            .unwrap();
        assert_eq!(snapshot.actors()[0].action, BaseAction::Attack);
        assert_eq!(snapshot.actors()[0].heading, 2);
        assert_eq!(snapshot.actors()[1].action, BaseAction::Damage);
        assert_eq!(
            snapshot.actors()[0].action_elapsed,
            Duration::from_millis(100)
        );
        timeline
            .update(combat_input(
                attacked.clone(),
                start + Duration::from_millis(500),
            ))
            .unwrap();
        assert_eq!(
            timeline
                .snapshot(start + Duration::from_millis(610))
                .unwrap()
                .actors()[0]
                .action_elapsed,
            Duration::from_millis(600)
        );
        attacked.world.revision += 1;
        attacked.vitals[1].hp = 0;
        attacked.vitals[1].hurt_revision = attacked.world.revision;
        attacked.vitals[1].death_revision = attacked.world.revision;
        timeline
            .update(combat_input(
                attacked.clone(),
                start + Duration::from_secs(1),
            ))
            .unwrap();
        let snapshot = timeline.snapshot(start + Duration::from_secs(60)).unwrap();
        assert_eq!(snapshot.actors()[1].action, BaseAction::Death);
        assert_eq!(snapshot.actors()[1].action_elapsed, Duration::from_secs(59));
        assert!(timeline
            .update(combat_input(
                combat_frame(),
                start + Duration::from_secs(61)
            ))
            .is_err());
        let mut wrong = attacked.clone();
        wrong.world.revision += 1;
        wrong.vitals[0].hp += 1;
        assert!(
            Input::with_combat(Arc::new(attacked.world.clone()), 1, Some(Arc::new(wrong))).is_err()
        );
    }

    #[test]
    fn npc_light_arrow_visual_keeps_typed_life_and_receipt_cell_after_retirement() {
        let hit = super::super::npc_spell_protocol::tests::success();
        let frame = hit.frame.unwrap();
        let event = light_arrow_events(&frame.combat).unwrap()[0];
        assert!(
            matches!(event.target, Some(LightArrowTarget::Npc { id, life: 1 }) if id == hit.echo.target)
        );
        assert_eq!(
            light_arrow_target(&event, &frame.combat, Some(&frame)).unwrap(),
            event.to
        );
        assert!(light_arrow_target(&event, &frame.combat, None).is_err());
        let mut retired = frame.clone();
        retired.actors.clear();
        assert_eq!(
            light_arrow_target(&event, &retired.combat, Some(&retired)).unwrap(),
            event.to
        );
        let mut next_life = frame.clone();
        next_life.actors[0].life += 1;
        next_life.actors[0].cell.x += 1;
        assert_eq!(
            light_arrow_target(&event, &next_life.combat, Some(&next_life)).unwrap(),
            event.to
        );
        let mut wrong_scope = frame.clone();
        wrong_scope.scope.generation += 1;
        assert!(light_arrow_target(&event, &wrong_scope.combat, Some(&wrong_scope)).is_err());
    }

    #[test]
    fn weapon_events_keep_the_authoritative_pose_and_presentation_facing_does_not_mutate_world() {
        let mut before = combat_frame().vitals.remove(0);
        for kind in [1, 2, 3, 5, 6, 14, 15, 18] {
            before.weapon_type = kind;
            let mut after = before.clone();
            after.attack_revision = 2;
            after.attack_sequence = 1;
            after.last_target_entity_id = 2;
            assert_eq!(combat_pose(&before, &after), Some(BaseAction::Attack));
            after.hp = 0;
            after.hurt_revision = 3;
            after.death_revision = 3;
            assert_eq!(combat_pose(&before, &after), Some(BaseAction::Death));
        }
        for (i, (x, y)) in super::super::world_map::DIRECTIONS.iter().enumerate() {
            assert_eq!(
                facing(WorldCell { x: 0, y: 0 }, WorldCell { x: *x, y: *y }).unwrap(),
                i as u8
            );
        }
        assert!(facing(WorldCell { x: 0, y: 0 }, WorldCell { x: 0, y: 0 }).is_err());
    }

    #[test]
    fn health_bars_preserve_i64_ratio_and_absent_or_invalid_data_is_not_full_health() {
        let mut canvas = Canvas::new(60, 10).unwrap();
        let p = ActorPlacement {
            visibility: None,
            entity_id: 1,
            cell: WorldCell { x: 0, y: 0 },
            anchor: PixelPoint { x: 30, y: 8 },
            top_left: PixelPoint { x: 5, y: 8 },
            width: 20,
            height: 20,
            graphics_id: 3213,
            source_frame: 1,
            sprite_name: "fixture".into(),
            subframe: 0,
        };
        let mut vital = combat_frame().vitals.remove(0);
        vital.max_hp = i64::MAX;
        vital.hp = i64::MAX / 2;
        draw_health_bars(&mut canvas, &[p.clone()], &[vital.clone()]).unwrap();
        let green = canvas
            .rgba()
            .chunks_exact(4)
            .filter(|p| **p == [35, 210, 75, 255])
            .count();
        assert_eq!(green, 23 * 3);
        assert!(draw_health_bars(&mut canvas, &[p.clone()], &[]).is_err());
        vital.hp = -1;
        assert!(draw_health_bars(&mut canvas, &[p], &[vital]).is_err());
    }

    pub(crate) fn frame(id: u64) -> Frame {
        let keys = [
            CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            CharacterKey {
                source_id: 42,
                object_id: 3333,
            },
        ];
        Frame {
            match_id: id,
            descriptor: None,
            race: None,
            audience_lane: None,
            quest: None,
            time_attack: None,
            siege: None,
            generation: 3,
            revision: 1,
            phase: Phase::Loading,
            abort_reason: AbortReason::None,
            completion: None,
            casts: Vec::new(),
            room: RoomSnapshot {
                summary: Summary {
                    room_id: 7,
                    revision: 3,
                    settings: PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 32,
                        map_id: 514,
                        name: "Synthetic world".to_owned(),
                    },
                    users: 2,
                    ready: 2,
                    satisfied: true,
                },
                owner: keys[0],
                members: keys
                    .iter()
                    .enumerate()
                    .map(|(i, key)| Member {
                        kind: super::super::participant::MemberKind::Human,
                        key: *key,
                        level: 80,
                        class: if i == 0 { 1 } else { 3 },
                        sex: i as u8,
                        team: i as u8,
                        ready: true,
                        name: format!("Actor{i}"),
                        source_name: format!("Source{i}"),
                    })
                    .collect(),
            },
            manifest: Manifest {
                version: 1,
                map_id: 514,
                digest: [2; 32],
                sectors: vec![Sector {
                    x: 0x7fff,
                    y: 0x8000,
                    byte_length: 137_784,
                    sha256: [1; 32],
                }],
            },
            actors: keys
                .iter()
                .enumerate()
                .map(|(i, key)| Actor {
                    kind: super::super::participant::MemberKind::Human,
                    entity_id: i as u64 + 1,
                    key: *key,
                    position: WorldCell {
                        x: 32724 + i as i32,
                        y: 32784,
                    },
                    heading: 2,
                    loaded: false,
                    sequence: 0,
                    step_interval_nanos: 200_000_000,
                })
                .collect(),
        }
    }
    #[test]
    fn finished_presentation_retains_world_and_only_allows_exact_receipt_update() {
        let pending = super::super::combat_protocol::tests::finished_reply()
            .frame
            .unwrap();
        let make = |frame: super::super::combat_protocol::Frame| {
            Input::with_combat(Arc::new(frame.world.clone()), 1, Some(Arc::new(frame))).unwrap()
        };
        let first = make(pending.clone());
        let mut saved = pending.clone();
        saved.world.revision += 1;
        saved.world.completion.as_mut().unwrap().state =
            super::super::world_protocol::SettlementState::Saved;
        let second = make(saved.clone());
        assert!(follows(&first, &second));
        assert!(!follows(&second, &first));
        saved.vitals[0].attack_sequence += 1;
        assert!(!follows(&first, &make(saved)));
    }

    fn input(frame: Frame, received: Instant) -> Arc<Input> {
        let mut input = Input::new(Arc::new(frame), 1).unwrap();
        input.received = received;
        Arc::new(input)
    }

    fn limits() -> SceneLimitations {
        SceneLimitations {
            opacity_records: 0,
            sprite_effects: 0,
            portals: 0,
            actor_object_occlusion_unimplemented: true,
            actor_dependent_roofs_unimplemented: true,
            native_roof_timers_and_aliases_unverified: true,
            native_actor_update_order_unverified: true,
            weapon_and_transformation_art_unimplemented: true,
            native_actor_foot_pivot_unverified: true,
            unplayed_sound_ids: vec![],
            unrendered_base_effect_ids: vec![478, 479],
            unrendered_base_effects: vec![],
        }
    }

    #[derive(Default)]
    struct Counts {
        loaded: AtomicUsize,
        active: AtomicUsize,
        high_water: AtomicUsize,
        rendered: AtomicUsize,
    }
    struct FakeRenderer(Arc<Counts>);
    impl Drop for FakeRenderer {
        fn drop(&mut self) {
            self.0.active.fetch_sub(1, Ordering::AcqRel);
        }
    }
    impl Renderer for FakeRenderer {
        fn render(
            &mut self,
            snapshot: &SceneSnapshot,
            _: Option<&combat_protocol::Frame>,
            _: &AtomicBool,
        ) -> Result<Pixels> {
            self.0.rendered.fetch_add(1, Ordering::AcqRel);
            let mut bytes = vec![0; PIXEL_BYTES];
            bytes[0] = snapshot.revision() as u8;
            bytes[1] = snapshot.actors()[0].cell.x as u8;
            Ok(Pixels {
                bgra: bytes.into(),
                limitations: limits(),
                actors: vec![],
                objectives: vec![],
                quests: vec![],
                time_attack_tower: None,
                siege_towers: Vec::new(),
                npcs: vec![],
                audio: None,
                presentation_notice: None,
            })
        }
    }
    fn fake_loader(counts: Arc<Counts>, hold: Option<Arc<AtomicBool>>) -> Loader {
        Arc::new(move |_, _, _, _, _| {
            counts.loaded.fetch_add(1, Ordering::AcqRel);
            let active = counts.active.fetch_add(1, Ordering::AcqRel) + 1;
            counts.high_water.fetch_max(active, Ordering::AcqRel);
            let renderer = FakeRenderer(counts.clone());
            if let Some(hold) = &hold {
                // Deliberately simulates a synchronous filesystem operation
                // that ignores cancellation until it returns. Test releases it.
                while hold.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(1));
                }
            }
            Ok(Box::new(renderer))
        })
    }
    fn runtime(loader: Loader) -> WorldRuntime {
        WorldRuntime::with_loader(PathBuf::from("test-root-never-opened"), loader)
    }
    fn wait(
        runtime: &mut WorldRuntime,
        predicate: impl Fn(&RuntimeStatus) -> bool,
    ) -> RuntimeStatus {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let status = runtime.poll();
            if predicate(&status) {
                return status;
            }
            assert!(Instant::now() < deadline, "world runtime did not converge");
            thread::sleep(Duration::from_millis(1));
        }
    }
    // Separate helper avoids borrowing the runtime from a predicate passed to
    // its mutable polling helper.
    fn retire(runtime: &mut WorldRuntime) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !runtime.is_quiescent() {
            runtime.poll();
            assert!(Instant::now() < deadline, "world worker was not retired");
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn completed_preflight_joins_unused_renderer_but_keeps_authority_validation() {
        let counts = Arc::new(Counts::default());
        let mut runtime = runtime(fake_loader(counts.clone(), None));
        assert!(!runtime.finish_preflight(&frame(1), 1));
        runtime.update(Arc::new(frame(1)), 1).unwrap();
        let RuntimeStatus::Ready(proof) =
            wait(&mut runtime, |s| matches!(s, RuntimeStatus::Ready(_)))
        else {
            unreachable!()
        };
        assert!(
            !runtime.finish_preflight(&frame(1), 1),
            "Loading cannot bypass LOAD ACK"
        );
        let mut active = frame(1);
        active.phase = Phase::Active;
        active.revision = 2;
        for actor in &mut active.actors {
            actor.loaded = true;
        }
        runtime.update(Arc::new(active.clone()), 1).unwrap();
        assert!(
            !runtime.finish_preflight(&active, 2),
            "different actor cannot retire worker"
        );
        let mut wrong_world = active.clone();
        wrong_world.match_id += 1;
        assert!(!runtime.finish_preflight(&wrong_world, 1));
        assert!(runtime.finish_preflight(&active, 1));
        retire(&mut runtime);
        let renders = counts.rendered.load(Ordering::Acquire);
        for _ in 0..32 {
            active.revision += 1;
            runtime.update(Arc::new(active.clone()), 1).unwrap();
            let RuntimeStatus::Ready(retained) = runtime.poll() else {
                panic!("preflight proof lost")
            };
            assert!(Arc::ptr_eq(&proof, &retained));
            assert_eq!(
                retained.world.phase,
                Phase::Loading,
                "never relabel old pixels"
            );
            assert!(runtime.preflight_finished());
            assert!(runtime.is_quiescent());
        }
        assert_eq!(counts.rendered.load(Ordering::Acquire), renders);
        assert_eq!(counts.loaded.load(Ordering::Acquire), 1);
        assert_eq!(counts.active.load(Ordering::Acquire), 0);
        let mut foreign = active;
        foreign.manifest.digest[0] ^= 1;
        assert!(runtime.update(Arc::new(foreign), 1).is_err());
        runtime.clear();
        assert!(!runtime.preflight_finished());
        assert!(matches!(runtime.poll(), RuntimeStatus::Idle));
        runtime.update(Arc::new(frame(1)), 1).unwrap();
        wait(&mut runtime, |s| matches!(s, RuntimeStatus::Ready(_)));
        assert_eq!(
            counts.loaded.load(Ordering::Acquire),
            2,
            "reentry reloads its own proof"
        );
        runtime.shutdown();
        retire(&mut runtime);
        assert!(!runtime.finish_preflight(&frame(1), 1));
    }

    #[test]
    fn qualified_input_preserves_logical_to_physical_binding_and_rejects_other_phases() {
        let mut sample = frame(1);
        assert!(Input::new(Arc::new(sample.clone()), 1).is_ok());
        // Logical 540 is a valid room option, bound by server policy to the
        // different shared physical 514. The client must not re-infer the map.
        sample.room.summary.settings.mode = 4;
        sample.room.summary.settings.map_id = 540;
        assert!(Input::new(Arc::new(sample.clone()), 1).is_ok());
        sample.phase = Phase::Preparing;
        assert!(Input::new(Arc::new(sample.clone()), 1).is_err());
        sample.phase = Phase::Aborted;
        sample.abort_reason = AbortReason::Disconnected;
        assert!(Input::new(Arc::new(sample), 1).is_err());
        assert!(Input::new(Arc::new(frame(1)), 99).is_err());
        assert!(WorldRuntime::new(PathBuf::from("relative-root")).is_err());
    }

    #[test]
    fn unchanged_revision_does_not_reset_animation_and_confirmed_move_has_one_interval() {
        let start = Instant::now();
        let initial = input(frame(1), start);
        let mut timeline = Timeline::new(initial.clone());
        timeline
            .update(input(frame(1), start + Duration::from_millis(50)))
            .unwrap();
        let idle = timeline
            .snapshot(start + Duration::from_millis(80))
            .unwrap();
        assert_eq!(idle.actors()[0].action, BaseAction::Idle);
        assert_eq!(idle.actors()[0].action_elapsed, Duration::from_millis(80));
        let mut moved = frame(1);
        moved.revision = 2;
        moved.actors[0].position = WorldCell { x: 32725, y: 32785 };
        moved.actors[0].sequence = 1;
        moved.actors[0].heading = 3;
        let received = start + Duration::from_millis(100);
        timeline.update(input(moved.clone(), received)).unwrap();
        let walk = timeline
            .snapshot(received + Duration::from_millis(50))
            .unwrap();
        assert_eq!(walk.actors()[0].cell, moved.actors[0].position);
        assert_eq!(walk.actors()[0].action, BaseAction::Walk);
        assert_eq!(walk.actors()[0].action_elapsed, Duration::from_millis(50));
        timeline
            .update(input(moved, received + Duration::from_millis(100)))
            .unwrap();
        let ended = timeline
            .snapshot(received + Duration::from_millis(250))
            .unwrap();
        assert_eq!(ended.actors()[0].action, BaseAction::Idle);
        assert_eq!(ended.actors()[0].action_elapsed, Duration::from_millis(50));
        assert_eq!(ended.actors()[1].action_elapsed, Duration::from_millis(350));
        assert_eq!(ended.actors()[1].cell, initial.frame.actors[1].position);
    }

    #[test]
    fn presentation_rejects_changed_hash_roster_sequence_and_equal_revision_content() {
        let start = Instant::now();
        let old = input(frame(1), start);
        for case in 0..6 {
            let mut changed = frame(1);
            changed.revision = 2;
            match case {
                0 => changed.manifest.digest[0] ^= 1,
                1 => changed.room.members[0].class = 0,
                2 => changed.actors[0].position.x -= 1,
                3 => changed.actors[0].step_interval_nanos += 1,
                4 => {
                    changed.revision = 1;
                    changed.actors[0].loaded = true;
                }
                5 => changed.room.members[0].level += 1,
                _ => unreachable!(),
            }
            assert!(!follows(&old, &input(changed, start)), "case {case}");
        }
        let mut moved = frame(1);
        moved.revision = 3;
        moved.actors[0].sequence = 2;
        moved.actors[0].position.x -= 1;
        let newest = input(moved, start);
        assert!(follows(&old, &newest));
        assert!(!follows(&newest, &old));
    }

    #[test]
    fn live_preferences_reprepare_audio_without_network_revision_or_resource_reload() {
        use crate::modern_lito::{
            audio_gain::tests::wave,
            combat_animation::classify_native_cues,
            combat_audio::{PreparedSound, SoundBank},
            preferences::{Preferences, SharedPreferences},
            sprite_actions::FrameEvent,
            world_audio::{AudioFrame, SoundEvent},
        };
        struct WithAudio(FakeRenderer, AudioFrame);
        impl Renderer for WithAudio {
            fn render(
                &mut self,
                scene: &SceneSnapshot,
                combat: Option<&combat_protocol::Frame>,
                cancel: &AtomicBool,
            ) -> Result<Pixels> {
                let mut pixels = self.0.render(scene, combat, cancel)?;
                pixels.audio = Some(self.1.clone());
                Ok(pixels)
            }
        }
        let sound = PreparedSound::from_wav(97, wave(8, 1, &[0, 128, 255, 127])).unwrap();
        let original_hash = sound.sha256();
        let audio = AudioFrame {
            bank: Arc::new(SoundBank::playback_subset(BTreeMap::from([(97, sound)])).unwrap()),
            events: vec![SoundEvent {
                sequence: 7,
                created: Instant::now(),
                group: classify_native_cues(&[FrameEvent::Sound(97)])
                    .unwrap()
                    .audio
                    .unwrap(),
                bank: None,
            }]
            .into(),
        };
        let counts = Arc::new(Counts::default());
        let base = fake_loader(counts.clone(), None);
        let loader_counts = counts.clone();
        // Use the same resource ownership accounting as the existing worker tests.
        let loader: Loader = Arc::new(move |root, frame, combat, npc, cancel| {
            drop(base(root, frame, combat, npc, cancel)?);
            loader_counts.active.fetch_add(1, Ordering::AcqRel);
            Ok(Box::new(WithAudio(
                FakeRenderer(loader_counts.clone()),
                audio.clone(),
            )))
        });
        let values = Arc::new(SharedPreferences::default());
        let mut runtime = runtime(loader);
        runtime.preferences = values.clone();
        let authoritative = Arc::new(frame(1));
        runtime.update(authoritative.clone(), 1).unwrap();
        for gain in [100, 50, 0, 37, 100] {
            values
                .apply(Preferences {
                    volume_percent: gain,
                    ..Preferences::default()
                })
                .unwrap();
            let status = wait(
                &mut runtime,
                |s| matches!(s, RuntimeStatus::Ready(image) if image.audio_gain == Some(gain)),
            );
            let RuntimeStatus::Ready(image) = status else {
                unreachable!()
            };
            assert!(Arc::ptr_eq(&image.world, &authoritative));
            assert_eq!(
                (image.match_id, image.generation, image.revision),
                (1, 3, 1)
            );
            assert!(image.ready_to_ack && image.presentation_notice.is_none());
            let audio = image.audio.as_ref().unwrap();
            assert_eq!(audio.events[0].sequence, 7);
            assert_eq!(audio.bank.get(97).unwrap().sha256(), original_hash);
            let expected: &[u8] = match gain {
                50 => &[64, 128, 191, 128],
                37 => &[81, 128, 174, 128],
                _ => &[0, 128, 255, 127],
            };
            assert_eq!(audio.bank.get(97).unwrap().pcm_bytes(), expected);
        }
        assert_eq!(counts.loaded.load(Ordering::Acquire), 1);
        runtime.shutdown();
        retire(&mut runtime);
        assert_eq!(counts.active.load(Ordering::Acquire), 0);
    }

    #[test]
    fn one_loaded_cache_renders_repeated_frames_and_new_revision_without_reloading() {
        let counts = Arc::new(Counts::default());
        let mut runtime = runtime(fake_loader(counts.clone(), None));
        runtime.update(Arc::new(frame(1)), 1).unwrap();
        let status = wait(&mut runtime, |s| matches!(s, RuntimeStatus::Ready(_)));
        let RuntimeStatus::Ready(first) = status else {
            unreachable!()
        };
        assert!(first.ready_to_ack);
        assert_eq!(
            (first.width, first.height, first.bgra.len()),
            (800, 600, PIXEL_BYTES)
        );
        assert_eq!(first.manifest_digest, [2; 32]);
        assert_eq!(*first.world, frame(1));
        runtime.update(Arc::new(frame(1)), 1).unwrap();
        let mut newer = frame(1);
        newer.revision = 2;
        newer.actors[0].loaded = true;
        runtime.update(Arc::new(newer), 1).unwrap();
        let status = wait(
            &mut runtime,
            |s| matches!(s, RuntimeStatus::Ready(i) if i.revision == 2),
        );
        let RuntimeStatus::Ready(second) = status else {
            unreachable!()
        };
        assert_eq!(second.bgra[0], 2);
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(first.bgra[0], 1); // immutable previous publication
        assert_eq!(first.world.revision, 1);
        assert_eq!(second.world.revision, 2);
        assert!(!first.world.actors[0].loaded);
        assert!(second.world.actors[0].loaded);
        assert_eq!(counts.loaded.load(Ordering::Acquire), 1);
        runtime.shutdown();
        retire(&mut runtime);
        assert_eq!(counts.active.load(Ordering::Acquire), 0);
        assert!(runtime.update(Arc::new(frame(2)), 1).is_err());
    }

    #[test]
    fn completed_frames_advance_behind_authority_without_first_ack_or_identity_regression() {
        let mut runtime = runtime(fake_loader(Arc::new(Counts::default()), None));
        runtime.update(Arc::new(frame(1)), 1).unwrap();
        let RuntimeStatus::Ready(first) =
            wait(&mut runtime, |s| matches!(s, RuntimeStatus::Ready(_)))
        else {
            unreachable!()
        };
        let mut next = frame(1);
        next.revision = 8;
        let desired = input(next, Instant::now());
        let mut completed = (*first).clone();
        completed.revision = 4;
        assert!(
            !publishable(&completed, &desired, None),
            "stale first frame cannot prove loaded"
        );
        assert!(
            publishable(&completed, &desired, Some(&first)),
            "completed same-world frames must not starve"
        );
        assert!(
            publishable(&completed, &desired, Some(&completed)),
            "animation may advance at equal authority revision"
        );
        assert!(
            !publishable(&first, &desired, Some(&completed)),
            "never rewind displayed authority"
        );
        for case in 0..5 {
            let mut bad = completed.clone();
            match case {
                0 => bad.match_id += 1,
                1 => bad.generation += 1,
                2 => bad.selected_entity_id += 1,
                3 => bad.manifest_digest[0] ^= 1,
                4 => bad.revision = 9,
                _ => unreachable!(),
            }
            assert!(
                !publishable(&bad, &desired, Some(&first)),
                "foreign/future case {case}"
            );
        }
        completed.revision = 8;
        assert!(publishable(&completed, &desired, None));
        runtime.shutdown();
        retire(&mut runtime);
    }

    struct Release(Arc<AtomicBool>);
    impl Drop for Release {
        fn drop(&mut self) {
            self.0.store(false, Ordering::Release);
        }
    }

    #[test]
    fn cancellation_during_noninterruptible_load_never_publishes_or_detaches() {
        let counts = Arc::new(Counts::default());
        let hold = Arc::new(AtomicBool::new(true));
        let mut runtime = runtime(fake_loader(counts.clone(), Some(hold.clone())));
        // Declared after runtime, releases the synthetic IO on unwind BEFORE
        // the runtime's mandatory Drop join, so failing assertions cannot hang.
        let release = Release(hold);
        runtime.update(Arc::new(frame(1)), 1).unwrap();
        wait(&mut runtime, |_| counts.loaded.load(Ordering::Acquire) == 1);
        let started = Instant::now();
        runtime.shutdown();
        assert!(matches!(runtime.poll(), RuntimeStatus::Stopping));
        assert!(started.elapsed() < Duration::from_millis(200));
        assert!(!runtime.is_quiescent());
        drop(release);
        retire(&mut runtime);
        assert_eq!(counts.rendered.load(Ordering::Acquire), 0);
        assert_eq!(counts.active.load(Ordering::Acquire), 0);
        assert!(matches!(runtime.poll(), RuntimeStatus::Idle));
    }

    #[test]
    fn clear_reuses_one_runtime_with_two_owned_workers_and_latest_pending_only() {
        let counts = Arc::new(Counts::default());
        let hold = Arc::new(AtomicBool::new(true));
        let mut runtime = runtime(fake_loader(counts.clone(), Some(hold.clone())));
        let release = Release(hold);
        runtime.update(Arc::new(frame(1)), 1).unwrap();
        wait(&mut runtime, |_| counts.loaded.load(Ordering::Acquire) == 1);
        runtime.clear();
        runtime.update(Arc::new(frame(2)), 1).unwrap();
        wait(&mut runtime, |_| counts.loaded.load(Ordering::Acquire) == 2);
        for id in 3..=10 {
            runtime.update(Arc::new(frame(id)), 1).unwrap();
            runtime.poll();
        }
        assert_eq!(counts.loaded.load(Ordering::Acquire), 2);
        assert_eq!(counts.high_water.load(Ordering::Acquire), 2);
        assert!(runtime.published.is_none());
        drop(release);
        wait(
            &mut runtime,
            |s| matches!(s, RuntimeStatus::Ready(i) if i.match_id == 10),
        );
        assert_eq!(counts.loaded.load(Ordering::Acquire), 3);
        assert_eq!(counts.high_water.load(Ordering::Acquire), 2);
        runtime.clear();
        retire(&mut runtime);
        assert!(matches!(runtime.poll(), RuntimeStatus::Idle));
        runtime.update(Arc::new(frame(11)), 1).unwrap();
        wait(
            &mut runtime,
            |s| matches!(s, RuntimeStatus::Ready(i) if i.match_id == 11),
        );
        runtime.shutdown();
        retire(&mut runtime);
        assert_eq!(counts.active.load(Ordering::Acquire), 0);
    }

    #[test]
    fn ui_poll_and_submission_never_wait_for_locked_worker_mailboxes() {
        let counts = Arc::new(Counts::default());
        let mut runtime = runtime(fake_loader(counts, None));
        runtime.update(Arc::new(frame(1)), 1).unwrap();
        wait(&mut runtime, |s| matches!(s, RuntimeStatus::Ready(_)));
        let input = runtime.active.as_ref().unwrap().input.clone();
        let output = runtime.active.as_ref().unwrap().output.clone();
        let input_lock = input.lock().unwrap();
        let output_lock = output.lock().unwrap();
        let mut next = frame(1);
        next.revision = 2;
        next.actors[0].loaded = true;
        let started = Instant::now();
        runtime.update(Arc::new(next), 1).unwrap();
        runtime.poll();
        assert!(started.elapsed() < Duration::from_millis(200));
        assert!(runtime.pending.is_some());
        drop(output_lock);
        drop(input_lock);
        wait(
            &mut runtime,
            |s| matches!(s, RuntimeStatus::Ready(i) if i.revision == 2),
        );
        runtime.shutdown();
        retire(&mut runtime);
    }

    #[test]
    fn resource_failure_is_static_and_cleared_for_a_new_world_without_auto_retry() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let copy = attempts.clone();
        let mut runtime = runtime(Arc::new(move |_, _, _, _, _| {
            copy.fetch_add(1, Ordering::AcqRel);
            anyhow::bail!("private local path and raw decoder exception must not escape")
        }));
        runtime.update(Arc::new(frame(1)), 1).unwrap();
        let status = wait(&mut runtime, |s| matches!(s, RuntimeStatus::Failed(_)));
        let RuntimeStatus::Failed(message) = status else {
            unreachable!()
        };
        assert_eq!(
            message,
            "shared world resources could not be validated or decoded"
        );
        for _ in 0..5 {
            runtime.poll();
        }
        assert_eq!(attempts.load(Ordering::Acquire), 1);
        assert!(runtime.is_quiescent());
        runtime.clear();
        runtime.update(Arc::new(frame(2)), 1).unwrap();
        wait(&mut runtime, |s| matches!(s, RuntimeStatus::Failed(_)));
        assert_eq!(attempts.load(Ordering::Acquire), 2);
        runtime.shutdown();
        retire(&mut runtime);
    }

    fn digest(hex: &str) -> [u8; 32] {
        let mut bytes = [0; 32];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap();
        }
        bytes
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; actual shared compositor/healing pixels at fixed times, no output/audio device/DB/UI"]
    fn real_shared_healing_layer_changes_pixels_then_retires_without_same_revision_replay() {
        // This is a layer-isolation test, not a comparison of injured/healed
        // health bars. Both renders receive the exact same immutable scene,
        // combat frame, vitals and instant. Only the healing layer is absent
        // from the control; its assets and watermark are restored unchanged.
        fn without_healing(
            renderer: &mut SharedRenderer,
            scene: &SceneSnapshot,
            combat: &combat_protocol::Frame,
            cancel: &AtomicBool,
            now: Instant,
        ) -> Pixels {
            let healing = renderer.healing.take().expect("qualified healing layer");
            let result = renderer.render_at(scene, Some(combat), cancel, now);
            renderer.healing = Some(healing);
            result.unwrap()
        }

        let root =
            PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let mut source = super::super::item_protocol::tests::initial().frame.unwrap();
        source.world.manifest.digest =
            digest("2feecc83de156a03c62e44effb7106067b24098a9ef7250cc471f7bf826da7a8");
        source.world.manifest.sectors[0].sha256 =
            digest("ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271");
        source.vitals[0].max_hp = 1_000;
        source.items[0].stocks = [40010, 40011, 40012]
            .into_iter()
            .map(|item_id| super::super::item_protocol::Stock {
                item_id,
                initial: 1,
                remaining: 1,
            })
            .collect();
        source.validate().unwrap();
        let cancel = AtomicBool::new(false);
        // Use the exact production loader (including expected manifest hashes,
        // actors, WAV qualification and all three real healing SPR/actions).
        let mut renderer =
            load_shared_renderer(&root, &source.world, Some(&source), &cancel).unwrap();
        let start = Instant::now();
        for (index, (item_id, effect, restored, delay_millis)) in [
            (40010, 189, 10, 500),
            (40011, 194, 30, 1_000),
            (40012, 197, 60, 1_000),
        ]
        .into_iter()
        .enumerate()
        {
            source.world.revision += 1;
            let event = &mut source.items[0];
            event.request_sequence += 1;
            event.use_count += 1;
            event.use_revision = source.world.revision;
            event.item_id = item_id;
            event.restored = restored;
            event.hp_before = source.vitals[0].hp;
            event.hp_after = event.hp_before + restored;
            event.effect = effect;
            event.delay_millis = delay_millis;
            event.position = source.world.actors[0].position;
            event.stocks[index].remaining = 0;
            source.vitals[0].hp = event.hp_after;
            source.validate().unwrap();

            let event_start = start + Duration::from_secs(index as u64 * 2);
            let mut timeline = Timeline::new(combat_input(source.clone(), event_start));
            // Freeze actor animation elapsed and camera as well as HP. The
            // only elapsed time that advances below belongs to presentation
            // layers; this fixture has no teleport, projectile or attack cue.
            let scene = timeline.snapshot(event_start).unwrap();
            assert!(scene
                .actors()
                .iter()
                .all(|a| { a.action == BaseAction::Idle && a.action_elapsed == Duration::ZERO }));
            let mut reference = None::<Arc<[u8]>>;
            for millis in [0, 125, 333, 624, 625, 626, 1_250] {
                let now = event_start + Duration::from_millis(millis);
                let control = without_healing(&mut renderer, &scene, &source, &cancel, now);
                if let Some(reference) = &reference {
                    assert_eq!(
                        control.bgra.as_ref(),
                        reference.as_ref(),
                        "control must not change through HP, actor animation or camera"
                    );
                } else {
                    reference = Some(control.bgra.clone());
                }
                let actual = renderer
                    .render_at(&scene, Some(&source), &cancel, now)
                    .unwrap();
                assert_eq!(actual.bgra.len(), PIXEL_BYTES);
                assert!(actual.presentation_notice.is_none());
                if millis < 625 {
                    assert_ne!(
                        actual.bgra.as_ref(),
                        control.bgra.as_ref(),
                        "shared effect {effect} must contribute visible pixels at {millis}ms"
                    );
                } else {
                    assert_eq!(
                        actual.bgra.as_ref(),
                        control.bgra.as_ref(),
                        "shared effect {effect} must leave no layer at {millis}ms"
                    );
                    // A second poll of precisely the same authoritative
                    // revision/time must not restart an already retired use.
                    let repeated = renderer
                        .render_at(&scene, Some(&source), &cancel, now)
                        .unwrap();
                    assert_eq!(repeated.bgra.as_ref(), control.bgra.as_ref());
                }
            }
        }
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; real timed layers/qualified WAV at fixed times, no audio device/DB/UI"]
    fn real_shared_timed_layers_change_only_effect_pixels_and_never_replay_a_revision() {
        use super::super::timed_protocol::{tests as fixture, Kind};

        fn without_timed(
            renderer: &mut SharedRenderer,
            scene: &SceneSnapshot,
            combat: &combat_protocol::Frame,
            cancel: &AtomicBool,
            now: Instant,
        ) -> Pixels {
            // Retain the actual assets, watermark and active playback. Only
            // this presentation layer is absent from the control render.
            let timed = renderer.timed.take().expect("qualified timed layer");
            let result = renderer.render_at(scene, Some(combat), cancel, now);
            renderer.timed = Some(timed);
            result.unwrap()
        }
        fn sequences(pixels: &Pixels) -> Vec<u64> {
            pixels
                .audio
                .as_ref()
                .unwrap()
                .events
                .iter()
                .map(|e| e.sequence)
                .collect()
        }

        let root =
            PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let mut confirmed = fixture::initial();
        let initial = confirmed.frame.as_mut().unwrap();
        initial.world.manifest.digest =
            digest("2feecc83de156a03c62e44effb7106067b24098a9ef7250cc471f7bf826da7a8");
        initial.world.manifest.sectors[0].sha256 =
            digest("ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271");
        initial.validate().unwrap();
        let cancel = AtomicBool::new(false);
        // This is the production loader, including map digest, actor SPRs,
        // all three timed SPRs and the native Sound 121/142 qualification.
        let mut renderer =
            load_shared_renderer(&root, &initial.world, Some(&*initial), &cancel).unwrap();
        let start = Instant::now();
        for (index, (kind, sound)) in [
            (Kind::Haste, None),
            (Kind::Brave, Some(121)),
            (Kind::Wisdom, Some(142)),
        ]
        .into_iter()
        .enumerate()
        {
            confirmed =
                fixture::use_timed(&confirmed, kind, index as i64 * 2_000_000_000).combat_reply();
            confirmed.validate().unwrap();
            let source = confirmed.frame.as_ref().unwrap();
            let event_start = start + Duration::from_secs(index as u64 * 2);
            let mut timeline = Timeline::new(combat_input(source.clone(), event_start));
            // Freeze HP, body animation elapsed, actor placement and camera.
            // Comparing two unrelated network revisions would not isolate
            // these layers, especially once haste affects body animation.
            let scene = timeline.snapshot(event_start).unwrap();
            assert!(scene
                .actors()
                .iter()
                .all(|a| { a.action == BaseAction::Idle && a.action_elapsed == Duration::ZERO }));
            let effect = kind.effect();
            let duration = renderer
                .timed
                .as_ref()
                .unwrap()
                .assets
                .duration(effect)
                .unwrap();
            assert_eq!(
                duration,
                if effect == 191 {
                    Duration::from_millis(625)
                } else {
                    Duration::from_nanos(1_041_666_667)
                }
            );
            let mut reference = None::<Arc<[u8]>>;
            let mut first_sequence = None;
            for elapsed in [
                Duration::ZERO,
                Duration::from_millis(125),
                Duration::from_millis(333),
                duration - Duration::from_nanos(1),
                duration,
                duration + Duration::from_nanos(1),
                Duration::from_secs(2),
            ] {
                let now = event_start + elapsed;
                let control = without_timed(&mut renderer, &scene, source, &cancel, now);
                if let Some(reference) = &reference {
                    assert_eq!(
                        control.bgra.as_ref(),
                        reference.as_ref(),
                        "fixed HP/body/camera control changed for effect {effect}"
                    );
                } else {
                    reference = Some(control.bgra.clone());
                }
                let actual = renderer
                    .render_at(&scene, Some(source), &cancel, now)
                    .unwrap();
                assert_eq!(actual.bgra.len(), PIXEL_BYTES);
                assert!(actual.presentation_notice.is_none());
                if elapsed < duration {
                    assert_ne!(
                        actual.bgra.as_ref(),
                        control.bgra.as_ref(),
                        "shared effect {effect} has no visible isolated layer at {elapsed:?}"
                    );
                } else {
                    assert_eq!(
                        actual.bgra.as_ref(),
                        control.bgra.as_ref(),
                        "shared effect {effect} retained pixels after its terminal frame"
                    );
                }
                let audio = actual.audio.as_ref().unwrap();
                if elapsed == Duration::ZERO {
                    if let Some(sound) = sound {
                        assert_eq!(audio.events.len(), 1, "one native initial cue only");
                        let event = &audio.events[0];
                        assert_eq!(event.group.candidates(), &[sound]);
                        assert!(event.bank.as_ref().unwrap().get(sound).is_some());
                        first_sequence = Some(event.sequence);
                    } else {
                        assert!(audio.events.is_empty(), "native effect 191 is silent");
                    }
                }
                assert!(audio
                    .events
                    .iter()
                    .all(|event| Some(event.sequence) == first_sequence));
                if elapsed >= duration {
                    assert!(
                        audio.events.is_empty(),
                        "expired cue must leave bounded audio history"
                    );
                }
                // Repeating the exact frame and instant must not restart the
                // layer or mint a second cue, before OR after retirement.
                let repeated = renderer
                    .render_at(&scene, Some(source), &cancel, now)
                    .unwrap();
                assert_eq!(repeated.bgra.as_ref(), actual.bgra.as_ref());
                assert_eq!(sequences(&repeated), sequences(&actual));
                let timed = renderer.timed.as_ref().unwrap();
                assert_eq!(timed.baseline[&1], source.items[0].use_revision);
                assert_eq!(timed.active.len(), usize::from(elapsed < duration));
            }
        }
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; memory-only actual shared map and actor worker"]
    fn real_shared_manifest_loader_first_render_ready_then_confirmed_move_and_owned_retirement() {
        let root =
            PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let mut source = frame(1);
        source.manifest.digest =
            digest("2feecc83de156a03c62e44effb7106067b24098a9ef7250cc471f7bf826da7a8");
        source.manifest.sectors[0].sha256 =
            digest("ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271");
        let mut wrong = source.clone();
        wrong.manifest.digest[0] ^= 1;
        assert!(load_shared(&root, &wrong, None, &AtomicBool::new(false)).is_err());
        wrong = source.clone();
        wrong.manifest.sectors[0].sha256[0] ^= 1;
        assert!(load_shared(&root, &wrong, None, &AtomicBool::new(false)).is_err());
        let mut runtime = WorldRuntime::new(root).unwrap();
        runtime.update(Arc::new(source.clone()), 1).unwrap();
        let deadline = Instant::now() + Duration::from_secs(45);
        let first = loop {
            match runtime.poll() {
                RuntimeStatus::Ready(image) => break image,
                RuntimeStatus::Failed(error) => panic!("{error}"),
                _ => {}
            }
            assert!(
                Instant::now() < deadline,
                "real shared worker load deadline"
            );
            thread::sleep(Duration::from_millis(5));
        };
        assert!(first.ready_to_ack);
        assert_eq!(first.manifest_digest, source.manifest.digest);
        assert_eq!(first.limitations.unrendered_base_effect_ids, [478, 479]);
        assert_eq!(first.limitations.opacity_records, 17);
        assert_eq!(first.limitations.sprite_effects, 36);
        source.revision = 2;
        source.phase = Phase::Active;
        for actor in &mut source.actors {
            actor.loaded = true;
        }
        source.actors[0].sequence = 1;
        source.actors[0].heading = 3;
        source.actors[0].position = WorldCell { x: 32725, y: 32785 };
        runtime.update(Arc::new(source), 1).unwrap();
        let next = wait(
            &mut runtime,
            |s| matches!(s, RuntimeStatus::Ready(i) if i.revision == 2),
        );
        let RuntimeStatus::Ready(next) = next else {
            unreachable!()
        };
        assert_ne!(first.bgra, next.bgra);
        runtime.shutdown();
        retire(&mut runtime);
        assert!(runtime.is_quiescent());
    }

    #[test]
    #[ignore = "requires real shared armed SPR/WAV; deterministic render timeline, no audio device/DB/native BIN"]
    fn real_armed_render_sound_sequence_does_not_replay_baseline_revisions_or_stalled_events() {
        let root =
            PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let mut source = combat_frame();
        source.world.manifest.digest =
            digest("2feecc83de156a03c62e44effb7106067b24098a9ef7250cc471f7bf826da7a8");
        source.world.manifest.sectors[0].sha256 =
            digest("ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271");
        for member in &mut source.world.room.members {
            member.class = 1;
        }
        for vital in &mut source.vitals {
            vital.weapon_type = 1;
        }
        let cancel = AtomicBool::new(false);
        let mut renderer = load_shared(&root, &source.world, Some(&source), &cancel).unwrap();
        let start = Instant::now();
        let mut timeline = Timeline::new(combat_input(source.clone(), start));
        let first = renderer
            .render(&timeline.snapshot(start).unwrap(), Some(&source), &cancel)
            .unwrap();
        let first_audio = first.audio.unwrap();
        assert!(!first_audio.bank.is_empty());
        assert!(first_audio.events.is_empty());
        source.world.revision += 1;
        source.vitals[0].attack_sequence = 1;
        source.vitals[0].attack_revision = source.world.revision;
        source.vitals[0].last_target_entity_id = 2;
        source.vitals[1].hurt_revision = source.world.revision;
        source.vitals[1].hp -= 1;
        timeline
            .update(combat_input(source.clone(), start))
            .unwrap();
        let mut last_sequence = 0;
        let mut last_snapshot = None;
        for millis in (0..=900).step_by(100) {
            let snapshot = timeline
                .snapshot(start + Duration::from_millis(millis))
                .unwrap();
            assert!(snapshot.actors().iter().all(|actor| actor.weapon_type == 1));
            let pixels = renderer.render(&snapshot, Some(&source), &cancel).unwrap();
            for event in pixels.audio.unwrap().events.iter() {
                last_sequence = last_sequence.max(event.sequence);
            }
            last_snapshot = Some(snapshot);
        }
        assert!(
            last_sequence > 0,
            "armed shared one-shots must generate qualified WAV cues"
        );
        let repeat = renderer
            .render(last_snapshot.as_ref().unwrap(), Some(&source), &cancel)
            .unwrap();
        assert!(repeat
            .audio
            .unwrap()
            .events
            .iter()
            .all(|event| event.sequence <= last_sequence));
        // A newer attack first observed well after its cues never emits a new
        // batch of stale sound, and initial/late join death never replays it.
        source.world.revision += 1;
        source.vitals[0].attack_sequence += 1;
        source.vitals[0].attack_revision = source.world.revision;
        timeline
            .update(combat_input(source.clone(), start + Duration::from_secs(1)))
            .unwrap();
        let stalled = renderer
            .render(
                &timeline.snapshot(start + Duration::from_secs(10)).unwrap(),
                Some(&source),
                &cancel,
            )
            .unwrap();
        assert!(stalled
            .audio
            .unwrap()
            .events
            .iter()
            .all(|event| event.sequence <= last_sequence));
        let mut late = load_shared(&root, &source.world, Some(&source), &cancel).unwrap();
        let mut late_timeline = Timeline::new(combat_input(source.clone(), start));
        let pixels = late
            .render(
                &late_timeline.snapshot(start).unwrap(),
                Some(&source),
                &cancel,
            )
            .unwrap();
        assert!(pixels.audio.unwrap().events.is_empty());
    }
}
