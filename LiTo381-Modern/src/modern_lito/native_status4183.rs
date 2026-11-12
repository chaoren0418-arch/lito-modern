//! S0B 4183 owns a controller clock, not an ordinary actor idle animation.
//! Source: 4E7A40, 4E02C0, 4E9300, 4E45F0, 4E34C0 and 4D6570.
//! Only the empty-name, stationary, ordinary-speed server object is qualified.
use super::{
    native_audio_delivery::{FiniteEventKey, PreparedCue},
    native_world::{Actor, World},
};
use anyhow::{ensure, Result};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, OnceLock, Weak,
    },
    time::Instant,
};

fn origin() -> Instant {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    *ORIGIN.get_or_init(Instant::now)
}
pub(crate) fn now_ms() -> u64 {
    ms_at(Instant::now())
}
/// Same process-relative millisecond axis as `now_ms`, for a recorded instant
/// (packet decode). Instants before the origin saturate to zero.
pub(crate) fn ms_at(at: Instant) -> u64 {
    at.saturating_duration_since(origin())
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

pub(crate) fn is_object(world: &World, actor: &Actor) -> bool {
    actor.graphics == 4183
        && world.status.is_none_or(|s| s.object_id != actor.id)
        && actor.name.is_empty()
        && actor.title.is_empty()
        && canonical(actor)
}
fn canonical(actor: &Actor) -> bool {
    actor.incarnation != 0
        && actor.walk_action == 0
        && actor.heading == 0
        && actor.speed == 0
        && actor.brave == 0
        && actor.light == 0
        && actor.status == 0
        && !actor.dead
        && actor.motion_from.is_none()
        && !actor.position_predicted
        && actor.draw_state == super::native_actor_states::State::default()
        && actor.actions.is_empty()
        && actor.action_sequence == 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Callback {
    Queue,
    Frame,
}
#[derive(Clone, Debug)]
struct Live {
    incarnation: u64,
    received_ms: u64,
    dispatched_ms: u64,
    active_action: bool,
    frame: usize,
    idle_queued: bool,
    ideal: u64,
    // Serial preserves insertion order at equal deadlines (4D65F0).
    callbacks: BTreeMap<(u64, u64), Callback>,
    serial: Arc<AtomicU64>,
    lease: Arc<AtomicBool>,
    presentation: Arc<OnceLock<Weak<()>>>,
}
impl Live {
    fn schedule(&mut self, now: u64, delay: u64, kind: Callback) -> Result<()> {
        ensure!(self.callbacks.len() < 64, "4183 callback budget");
        let serial = self
            .serial
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| anyhow::anyhow!("4183 serial overflow"))?;
        let due = now
            .checked_add(delay)
            .ok_or_else(|| anyhow::anyhow!("4183 deadline overflow"))?;
        self.callbacks.insert((due, serial), kind);
        Ok(())
    }
    fn idle(&mut self, now: u64, pending_delay: u64) -> Result<()> {
        // 4E02C8: an existing action queue is neither cleared nor appended to.
        let delay = if self.idle_queued {
            pending_delay
        } else {
            self.idle_queued = true;
            if self.active_action {
                166
            } else {
                584
            }
        };
        self.schedule(now, delay, Callback::Queue)
    }
    fn refresh(&mut self, actor: &Actor, received: u64, dispatched: u64) -> Result<()> {
        self.incarnation = actor.incarnation;
        self.received_ms = received;
        self.dispatched_ms = dispatched;
        // 4E7C1D/24/3F reset action, stance and frame on both new and reused IDs.
        self.active_action = false;
        self.frame = 0;
        self.idle(dispatched, 1)
    }
    fn callback(&mut self, kind: Callback, now: u64) -> Result<bool> {
        match kind {
            Callback::Queue if !self.idle_queued => Ok(false),
            Callback::Queue => {
                self.idle_queued = false; // 4E85F0 pops before entering action3.
                self.active_action = true;
                self.frame = 1; // 4E4913/17 increments before entry.
                self.schedule(now, 168, Callback::Frame)?;
                self.ideal = now + 168;
                Ok(true)
            }
            Callback::Frame if !self.active_action => {
                // A refresh can leave an old frame callback. Absent action0
                // reaches the static path (4E62A1/4E63AA), returning delay0.
                self.frame = 0;
                self.idle(now, 0)?;
                Ok(false)
            }
            Callback::Frame => {
                self.frame = (self.frame + 1) % 6;
                if self.frame == 0 {
                    self.idle(now, 166)?;
                } else {
                    // Signed remaining time also handles an early callback
                    // from a previous S0B, not just the usual overdue case.
                    let remaining = (166i128 - (i128::from(now) - i128::from(self.ideal))).max(1);
                    let delay = u64::try_from(remaining + 2)?;
                    self.schedule(now, delay, Callback::Frame)?;
                    self.ideal = self
                        .ideal
                        .checked_add(168)
                        .ok_or_else(|| anyhow::anyhow!("4183 ideal overflow"))?;
                }
                Ok(self.frame == 1)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Sound {
    sequence: u64,
    lease: Arc<AtomicBool>,
    prepared: Arc<OnceLock<PreparedCue>>,
    presentation: Arc<OnceLock<Weak<()>>>,
}
/// A per-object scheduler failure. The object is isolated (removed from the
/// live set, its lease retired) so the rest of the scene and every other 4183
/// keep running; the fault stays visible in the frame receipt until the same
/// ID is rebuilt by a new S0B, the actor leaves, or the world is replaced.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct Fault {
    pub(crate) incarnation: u64,
    pub(crate) at_ms: u64,
    pub(crate) error: String,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Runtime {
    generation: Option<u64>,
    live: BTreeMap<u32, Live>,
    sounds: Vec<Sound>,
    last_ms: Option<u64>,
    serial: Arc<AtomicU64>,
    pub(crate) faults: BTreeMap<u32, Fault>,
    /// Count of faulted IDs that a later S0B rebuilt successfully.
    pub(crate) recovered: u64,
}
impl Runtime {
    fn isolate(&mut self, id: u32, at_ms: u64, error: anyhow::Error) {
        if let Some(live) = self.live.remove(&id) {
            live.lease.store(false, Ordering::Release);
            self.faults.insert(
                id,
                Fault {
                    incarnation: live.incarnation,
                    at_ms,
                    error: format!("{error:#}"),
                },
            );
        }
        self.sounds.retain(|s| s.lease.load(Ordering::Acquire));
    }
    pub(crate) fn sequences(&self) -> impl Iterator<Item = u64> + '_ {
        self.sounds.iter().map(|s| s.sequence)
    }
    pub(crate) fn leases(&self) -> Vec<Arc<AtomicBool>> {
        self.live.values().map(|v| v.lease.clone()).collect()
    }
    pub(crate) fn synchronize(
        &mut self,
        world: &World,
        put: Option<u32>,
        received: u64,
        dispatched: u64,
    ) {
        self.live.retain(|id, live| {
            let keep = self.generation == Some(world.generation)
                && world.actors.get(id).is_some_and(|a| {
                    is_object(world, a)
                        && canonical(a)
                        && (a.incarnation == live.incarnation || put == Some(*id))
                });
            if !keep {
                live.lease.store(false, Ordering::Release);
            }
            keep
        });
        if self.generation != Some(world.generation) {
            self.generation = Some(world.generation);
            self.last_ms = None;
            self.faults.clear();
        }
        // A fault describes an object that is still present but isolated.
        self.faults
            .retain(|id, _| world.actors.get(id).is_some_and(|a| a.graphics == 4183));
        if let Some(actor) = put
            .and_then(|id| world.actors.get(&id))
            .filter(|a| is_object(world, a) && canonical(a))
        {
            let rebuilding = self.faults.contains_key(&actor.id);
            let live = self.live.entry(actor.id).or_insert_with(|| Live {
                incarnation: actor.incarnation,
                received_ms: received,
                dispatched_ms: dispatched,
                active_action: false,
                frame: 0,
                idle_queued: false,
                ideal: 0,
                callbacks: BTreeMap::new(),
                serial: self.serial.clone(),
                lease: Arc::new(AtomicBool::new(true)),
                presentation: Arc::new(OnceLock::new()),
            });
            // Retire old render/audio snapshots, while preserving native queues.
            live.lease.store(false, Ordering::Release);
            live.lease = Arc::new(AtomicBool::new(true));
            live.presentation = Arc::new(OnceLock::new());
            match live.refresh(actor, received, dispatched) {
                Ok(()) => {
                    if rebuilding {
                        self.faults.remove(&actor.id);
                        self.recovered = self.recovered.saturating_add(1);
                    }
                }
                Err(error) => self.isolate(actor.id, dispatched, error),
            }
        }
        self.sounds.retain(|s| s.lease.load(Ordering::Acquire));
    }
    /// Errors returned here are runtime-wide (clock regression, dispatch or
    /// revision budget) and go to the controller error channel. Per-object
    /// failures never stop other objects; they are isolated into `faults`.
    pub(crate) fn poll(&mut self, now: u64, revision: &mut u64) -> Result<()> {
        ensure!(
            self.last_ms.is_none_or(|old| now >= old),
            "4183 controller clock regressed"
        );
        self.last_ms = Some(now);
        // Fixed sample, earliest deadline first; callbacks enqueue against this
        // dispatch sample. There is no modulo catch-up and no paint-side tick.
        for _ in 0..4096 {
            let next = self
                .live
                .iter()
                .filter_map(|(id, v)| {
                    v.callbacks
                        .first_key_value()
                        .filter(|(key, _)| key.0 <= now)
                        .map(|(key, _)| (*key, *id))
                })
                .min();
            let Some((key, id)) = next else {
                return Ok(());
            };
            let live = self.live.get_mut(&id).unwrap();
            let kind = live.callbacks.remove(&key).unwrap();
            let result = live.callback(kind, now);
            *revision = revision
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("4183 revision overflow"))?;
            let sound = match result {
                Ok(sound) => sound,
                Err(error) => {
                    // The bumped revision redraws without the isolated object.
                    self.isolate(id, now, error);
                    continue;
                }
            };
            let live = &self.live[&id];
            if sound {
                if self.sounds.len() == 32 {
                    self.sounds.remove(0);
                }
                self.sounds.push(Sound {
                    sequence: *revision,
                    lease: live.lease.clone(),
                    prepared: Arc::new(OnceLock::new()),
                    presentation: live.presentation.clone(),
                });
            }
        }
        anyhow::bail!("4183 dispatch budget")
    }
}

pub(crate) fn qualify(
    catalog: &super::sprite_actions::SpriteActionCatalog,
) -> std::result::Result<
    Vec<(u32, u8, super::sprite_actions::Animation)>,
    super::tower_assets::HonorAssetError,
> {
    use super::{
        sprite_actions::{ActionKey, Directionality, FrameEvent},
        tower_assets::HonorAssetError,
    };
    let row = catalog
        .sprites
        .get(&4183)
        .ok_or(HonorAssetError::Unsupported("4183 absent"))?;
    let resolved = catalog.resolve_action(4183, ActionKey::new(3))?;
    let a = resolved.animation;
    if row.graphics_id != 4183
        || row.header_issue.is_some()
        || row.image_count != Some(1)
        || resolved.definition_sprite_id != 4183
        || row.actions.len() != 1
        || row.attributes.len() != 2
        || [(102, b"0".as_slice()), (104, b"8".as_slice())]
            .iter()
            .any(|(key, body)| {
                row.attributes
                    .iter()
                    .filter(|v| v.key == ActionKey::new(*key) && v.body_bytes.trim_ascii() == *body)
                    .count()
                    != 1
            })
        || a.frames_per_second != 24
        || a.directionality != Directionality::Undirected
        || a.frames.len() != 6
        || a.frames.iter().enumerate().any(|(i, f)| {
            f.image_id != 0
                || f.subframe != if i == 0 { 5 } else { i as u32 - 1 }
                || f.duration_ticks != 4
                || f.events
                    != if i == 1 {
                        vec![FrameEvent::Effect(1453)]
                    } else {
                        vec![]
                    }
        })
    {
        return Err(HonorAssetError::Unsupported(
            "4183 action3 Screen/overlap contract changed",
        ));
    }
    Ok(vec![(4183, 3, a.clone())])
}

#[derive(Default)]
pub(crate) struct Stage {
    bank: Option<super::tower_assets::SharedEffectBank>,
    error: Option<String>,
    owner: Arc<()>,
}
fn admit(presentation: &OnceLock<Weak<()>>, owner: &Arc<()>) -> bool {
    let admitted = presentation.get_or_init(|| Arc::downgrade(owner));
    Weak::ptr_eq(admitted, &Arc::downgrade(owner))
}
impl Stage {
    pub(crate) fn prepare(
        &mut self,
        root: &std::path::Path,
        world: &World,
        cancel: &AtomicBool,
        catalog: &mut Option<super::shared_action_catalog::SharedActionCatalog>,
    ) {
        if self.bank.is_some() || self.error.is_some() || world.status4183.live.is_empty() {
            return;
        }
        let result = (|| -> Result<_> {
            let shared = super::shared_action_catalog::SharedActionCatalog::reuse_for_model(
                root, cancel, catalog,
            )?;
            Ok(super::tower_assets::SharedEffectBank::load_directional(
                &shared, cancel, qualify,
            )?)
        })();
        match result {
            Ok(bank) => self.bank = Some(bank),
            Err(error) => self.error = Some(format!("{error:#}")),
        }
    }
    pub(crate) fn draw_to<'s>(
        &'s self,
        world: &World,
        target: &mut super::native_world_effects::Target<'_, 's>,
        view: super::world_map::ViewProjection,
    ) -> Result<(serde_json::Value, Vec<PreparedCue>)> {
        let mut rows = Vec::new();
        let mut audio = Vec::new();
        if let Some(bank) = &self.bank {
            for (id, live) in &world.status4183.live {
                if !live.lease.load(Ordering::Acquire) || !admit(&live.presentation, &self.owner) {
                    continue;
                }
                let actor = &world.actors[id];
                // Static subframe0 before action3 remains a source inference.
                let frame = if live.active_action { live.frame } else { 1 };
                let cell = super::world_map::WorldCell {
                    x: i32::from(actor.x),
                    y: i32::from(actor.y),
                };
                if view
                    .cell_to_pixel(cell)
                    .is_some_and(|p| p.x.abs() < 1000 && p.y.abs() < 800)
                {
                    target.submit(
                        bank.frame(4183, 3, frame, false)?.frame,
                        cell,
                        view,
                        super::native_world_effects::Mix::Screen,
                    )?;
                }
                rows.push(serde_json::json!({"id":id,"incarnation":live.incarnation,"received_ms":live.received_ms,
                    "dispatched_ms":live.dispatched_ms,"source_frame":frame,"initial_static_inferred":!live.active_action,
                    "callbacks":live.callbacks.len(),"next_callback_ms":live.callbacks.first_key_value().map(|(k,_)|k.0)}));
            }
            for sound in &world.status4183.sounds {
                if !sound.lease.load(Ordering::Acquire) || !admit(&sound.presentation, &self.owner)
                {
                    continue;
                }
                if sound.prepared.get().is_none() {
                    let animation = bank.animation(4183, 3)?;
                    let group =
                        super::combat_animation::classify_native_cues(&animation.frames[1].events)?
                            .audio
                            .ok_or_else(|| anyhow::anyhow!("4183 overlap absent"))?;
                    let cue = PreparedCue::from_status_event(
                        bank.sound_bank(),
                        group,
                        FiniteEventKey {
                            sequence: sound.sequence,
                            graphics: 4183,
                            action: 3,
                            source_frame: 1,
                        },
                        sound.lease.clone(),
                    )?;
                    let _ = sound.prepared.set(cue);
                }
                if let Some(cue) = sound.prepared.get().filter(|cue| !cue.consumed()) {
                    audio.push(cue.clone());
                }
            }
        }
        Ok((
            serde_json::json!({"objects":rows,"load_error":self.error,"object_faults":world.status4183.faults,
            "recovered":world.status4183.recovered,
            "local_expiry":false,"original_clock_execution_qualified":false}),
            audio,
        ))
    }
}

#[cfg(test)]
#[path = "native_status4183_tests.rs"]
mod tests;
