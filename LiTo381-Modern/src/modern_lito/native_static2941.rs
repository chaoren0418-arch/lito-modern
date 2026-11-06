//! #97 receive -> resource proof -> fresh zero-delay enqueue -> retirement.
//! Controller owns timers; workers own resource decoding and may compose
//! speculatively, but only a queued, live lease may publish.
use super::{
    native_clock, native_connection::IngressReceipt, native_skills::Effect, native_world::World,
    shared_action_catalog::SharedActionCatalog, tower_assets::Static2941Bank,
};
use anyhow::{ensure, Context, Result};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc, OnceLock,
    },
    time::Instant,
};

const WAITING: u8 = 0;
const QUEUED: u8 = 1;
const RETIRED: u8 = 2;
const CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Owner {
    connection: u64,
    generation: u64,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Schedule {
    pub ready_at: Instant,
    pub enqueued_at: Instant,
    pub deadline_ms: u32,
}
#[derive(Debug)]
struct Live {
    owner: Owner,
    sequence: u64,
    ingress: IngressReceipt,
    dispatched_at: Instant,
    constructed_at: Instant,
    phase: AtomicU8,
    schedule: OnceLock<Schedule>,
}
/// Cloned World snapshots share this one-way retirement token, never a timer.
#[derive(Clone, Debug)]
pub struct Lease(Arc<Live>);
impl PartialEq for Lease {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Lease {}
impl Lease {
    pub fn active(&self) -> bool {
        self.0.phase.load(Ordering::Acquire) != RETIRED
    }
    pub fn waiting(&self) -> bool {
        self.0.phase.load(Ordering::Acquire) == WAITING
    }
    pub fn drawable(&self) -> bool {
        self.0.phase.load(Ordering::Acquire) == QUEUED && self.0.schedule.get().is_some()
    }
    pub fn sequence(&self) -> u64 {
        self.0.sequence
    }
    pub fn connection(&self) -> u64 {
        self.0.owner.connection
    }
    pub fn generation(&self) -> u64 {
        self.0.owner.generation
    }
    pub fn ingress(&self) -> IngressReceipt {
        self.0.ingress
    }
    pub fn dispatched_at(&self) -> Instant {
        self.0.dispatched_at
    }
    pub fn constructed_at(&self) -> Instant {
        self.0.constructed_at
    }
    pub(crate) fn schedule(&self) -> Option<Schedule> {
        self.0.schedule.get().copied()
    }
    pub(crate) fn retire(&self) {
        self.0.phase.store(RETIRED, Ordering::Release);
    }
    pub(crate) fn current(&self, world: &World, connection: u64) -> bool {
        self.active()
            && self.0.owner
                == Owner {
                    connection,
                    generation: world.generation,
                }
            && world.map_id.is_some()
            && world
                .effects
                .iter()
                .find(|cue| cue.sequence == self.sequence())
                .is_some_and(|cue| {
                    world
                        .static_effect_receipts
                        .get(&cue.sequence)
                        .is_some_and(|receipt| {
                            receipt.matches(world, connection, cue)
                                && receipt.lease.as_ref() == Some(self)
                        })
                })
    }
}

/// Binding is captured only after a validated S37; JSON excludes local clocks.
#[derive(Clone, Debug)]
pub struct Receipt {
    ingress: IngressReceipt,
    generation: u64,
    sequence: u64,
    target: Option<(u32, u64)>,
    position: [u16; 2],
    lease: Option<Lease>,
}
impl Receipt {
    pub(crate) fn received(world: &World, cue: &Effect, ingress: IngressReceipt) -> Self {
        Self {
            ingress,
            generation: world.generation,
            sequence: cue.sequence,
            target: cue.target,
            position: cue.position,
            lease: None,
        }
    }
    pub fn ingress(&self) -> IngressReceipt {
        self.ingress
    }
    pub fn lease(&self) -> Option<&Lease> {
        self.lease.as_ref()
    }
    fn matches(&self, world: &World, connection: u64, cue: &Effect) -> bool {
        self.generation == world.generation
            && self.ingress.connection_epoch() == connection
            && cue.graphics == 2941
            && self.sequence == cue.sequence
            && self.target == cue.target
            && self.position == cue.position
            && self.target.is_some_and(|(id, incarnation)| {
                world
                    .actors
                    .get(&id)
                    .is_some_and(|actor| actor.incarnation == incarnation)
            })
    }
    pub(crate) fn retire(&self) {
        if let Some(lease) = &self.lease {
            lease.retire();
        }
    }
}

/// Not constructible from a boolean, a timestamp, or arbitrary decoded pixels.
/// The actual immutable bank crosses the worker/controller boundary with it.
pub(crate) struct Prepared {
    lease: Lease,
    bank: Arc<Static2941Bank>,
    ready_at: Instant,
}
impl Prepared {
    pub(crate) fn from_bank(
        bank: Arc<Static2941Bank>,
        lease: &Lease,
        cancel: &AtomicBool,
    ) -> Result<Option<Self>> {
        super::tower_assets::active(cancel)?;
        if !lease.waiting() {
            return Ok(None);
        }
        let ready_at = Instant::now();
        ensure!(
            ready_at >= lease.constructed_at(),
            "static resource clock regressed"
        );
        super::tower_assets::active(cancel)?;
        Ok(Some(Self {
            lease: lease.clone(),
            bank,
            ready_at,
        }))
    }
}
struct Due {
    lease: Lease,
    deadline: u32,
    _bank: Arc<Static2941Bank>,
}
type Producer = Box<dyn FnOnce(&mut Queue, &mut World, u64) -> Result<()> + Send>;
enum Callback {
    Retire(Due),
    Shared { owner: Owner, producer: Producer },
}
struct Event {
    deadline: u32,
    callback: Callback,
}
#[derive(Default)]
pub(crate) struct Queue {
    owner: Option<Owner>,
    // Reset identity is separate from numeric connection/generation. A
    // reentrant callback can retire and rebind to the very same Owner.
    scope: Arc<()>,
    floor: u64,
    pending: Vec<Lease>,
    due: Vec<Event>,
}
impl Queue {
    pub(crate) fn synchronize(&mut self, world: &World, connection: u64) {
        let owner =
            (connection != 0 && world.generation != 0 && world.map_id.is_some()).then_some(Owner {
                connection,
                generation: world.generation,
            });
        if owner != self.owner {
            self.retire();
            self.owner = owner;
        }
        self.pending.retain(|lease| {
            if lease.current(world, connection) {
                true
            } else {
                lease.retire();
                false
            }
        });
        self.due.retain(|entry| match &entry.callback {
            Callback::Retire(due) => due.lease.active(),
            Callback::Shared { owner, .. } => Some(*owner) == self.owner,
        });
    }

    pub(crate) fn admit_received(
        &mut self,
        world: &mut World,
        connection: u64,
        ingress: IngressReceipt,
        dispatched_at: Instant,
    ) -> Result<bool> {
        ensure!(
            connection != 0 && ingress.connection_epoch() == connection,
            "static ingress connection mismatch"
        );
        ensure!(
            ingress.decoded_at() <= dispatched_at,
            "static dispatch precedes decode"
        );
        self.synchronize(world, connection);
        let Some(owner) = self.owner else {
            return Ok(false);
        };
        let Some(cue) = world.effects.last() else {
            return Ok(false);
        };
        let Some(receipt) = world.static_effect_receipts.get(&cue.sequence) else {
            return Ok(false);
        };
        if !receipt.matches(world, connection, cue)
            || receipt.ingress != ingress
            || cue.sequence <= self.floor
            || receipt.lease.is_some()
        {
            return Ok(false);
        }
        ensure!(
            self.pending.len() < CAPACITY,
            "static pending queue exceeds world ring"
        );
        let constructed_at = Instant::now();
        ensure!(
            dispatched_at <= constructed_at,
            "static constructor precedes dispatch"
        );
        let lease = Lease(Arc::new(Live {
            owner,
            sequence: cue.sequence,
            ingress,
            dispatched_at,
            constructed_at,
            phase: AtomicU8::new(WAITING),
            schedule: OnceLock::new(),
        }));
        self.floor = cue.sequence;
        world
            .static_effect_receipts
            .get_mut(&cue.sequence)
            .expect("checked receipt")
            .lease = Some(lease.clone());
        self.pending.push(lease);
        Ok(true)
    }

    pub(crate) fn enqueue_prepared(
        &mut self,
        world: &mut World,
        connection: u64,
        root: &Path,
        prepared: Prepared,
        clock: impl FnOnce() -> Result<u32>,
    ) -> Result<bool> {
        self.synchronize(world, connection);
        let lease = &prepared.lease;
        if !lease.waiting() || !lease.current(world, connection) || !self.pending.contains(lease) {
            return Ok(false);
        }
        let same_root = root == prepared.bank.root()
            || root.canonicalize().ok().as_deref() == Some(prepared.bank.root());
        if !same_root {
            return Ok(false);
        }
        let revision = world
            .visual_revision
            .checked_add(1)
            .context("static ready revision overflow")?;
        // All identity/resource checks precede the actual fresh 4D65F0 sample.
        let deadline = clock()?;
        let enqueued_at = Instant::now();
        ensure!(
            prepared.ready_at <= enqueued_at,
            "static enqueue precedes resource ready"
        );
        let schedule = Schedule {
            ready_at: prepared.ready_at,
            enqueued_at,
            deadline_ms: deadline,
        };
        // A snapshot can be retired while an imported clock call is in flight.
        // Publishing readiness must never overwrite that one-way transition.
        if lease
            .0
            .phase
            .compare_exchange(WAITING, QUEUED, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(false);
        }
        if lease.0.schedule.set(schedule).is_err() {
            lease.retire();
            anyhow::bail!("static scheduled twice");
        }
        // Native unsigned order, including rollover; equal deadlines stay FIFO.
        let position = self.due.partition_point(|entry| entry.deadline <= deadline);
        self.due.insert(
            position,
            Event {
                deadline,
                callback: Callback::Retire(Due {
                    lease: prepared.lease,
                    deadline,
                    _bank: prepared.bank,
                }),
            },
        );
        world.visual_revision = revision;
        Ok(true)
    }

    /// Shared producers and retirement use one ordered queue. A callback owns
    /// its arguments, never a borrowed Vec entry, so it may enqueue or nest a
    /// drain. The caller supplies a fresh enqueue sample, not the outer cutoff.
    pub(crate) fn enqueue_callback(
        &mut self,
        world: &World,
        connection: u64,
        delay: u32,
        clock: impl FnOnce() -> Result<u32>,
        producer: Producer,
    ) -> Result<bool> {
        self.synchronize(world, connection);
        let Some(owner) = self.owner else {
            return Ok(false);
        };
        ensure!(self.due.len() < 4096, "shared native callback queue budget");
        let deadline = clock()?.wrapping_add(delay);
        let at = self.due.partition_point(|entry| entry.deadline <= deadline);
        self.due.insert(
            at,
            Event {
                deadline,
                callback: Callback::Shared { owner, producer },
            },
        );
        Ok(true)
    }

    /// 4D6570: one cutoff per invocation, remove before invoking the callback.
    /// A nested drain has its own cutoff; resuming this loop keeps the outer
    /// value, even if a callback sampled a newer time. Painting never drains.
    pub(crate) fn drain_due(
        &mut self,
        world: &mut World,
        connection: u64,
        cutoff: u32,
    ) -> Result<Vec<u64>> {
        self.synchronize(world, connection);
        let scope = self.scope.clone();
        if self
            .due
            .first()
            .is_none_or(|entry| !native_clock::due(entry.deadline, cutoff))
        {
            return Ok(vec![]);
        }
        let mut retired = Vec::new();
        while self
            .due
            .first()
            .is_some_and(|entry| native_clock::due(entry.deadline, cutoff))
        {
            // Validate before removal so overflow cannot silently lose a live
            // retirement. Shared callbacks do not themselves change revision.
            let revision = if matches!(self.due[0].callback, Callback::Retire(_)) {
                Some(
                    world
                        .visual_revision
                        .checked_add(1)
                        .context("static retirement revision overflow")?,
                )
            } else {
                None
            };
            let entry = self.due.remove(0);
            match entry.callback {
                Callback::Retire(due) => {
                    debug_assert_eq!(entry.deadline, due.deadline);
                    due.lease.retire();
                    retired.push(due.lease.sequence());
                    world.visual_revision = revision.expect("retirement revision");
                }
                Callback::Shared { owner, producer } => {
                    if self.owner == Some(owner) {
                        producer(self, world, connection)?;
                    }
                }
            }
            if !Arc::ptr_eq(&scope, &self.scope) {
                // The callback replaced the queue. Its new events require a
                // new invocation/sample, never this retired stack's cutoff.
                break;
            }
            self.synchronize(world, connection);
            if !Arc::ptr_eq(&scope, &self.scope) {
                break;
            }
        }
        self.pending.retain(Lease::active);
        Ok(retired)
    }
    /// 4857B0's timer/draw portion. Each iteration has a fresh 4D6570 sample,
    /// then redraw, then a separate relative-time sample. The native loop is
    /// do/while with signed accumulated dword deltas (including wrap).
    /// The caller must establish the S37/169 own-actor/control-flag branch;
    /// receiving 2941 alone never authorizes entering this pump.
    pub(crate) fn drain_4857b0(
        &mut self,
        world: &mut World,
        connection: u64,
        mut clock: impl FnMut() -> Result<u32>,
        mut draw: impl FnMut(&mut Self, &mut World, u64) -> Result<()>,
    ) -> Result<()> {
        self.synchronize(world, connection);
        if self.owner.is_none() {
            return Ok(());
        }
        let scope = self.scope.clone();
        let mut previous = clock()?;
        let mut elapsed = 0i32;
        loop {
            self.drain_due(world, connection, clock()?)?;
            if !Arc::ptr_eq(&scope, &self.scope) {
                return Ok(());
            }
            draw(self, world, connection)?;
            if !Arc::ptr_eq(&scope, &self.scope) {
                return Ok(());
            }
            self.synchronize(world, connection);
            if !Arc::ptr_eq(&scope, &self.scope) {
                return Ok(());
            }
            let now = clock()?;
            elapsed = elapsed.wrapping_add(now.wrapping_sub(previous) as i32);
            previous = now;
            if elapsed >= 500 {
                return Ok(());
            }
        }
    }
    pub(crate) fn retire(&mut self) {
        self.scope = Arc::new(());
        for lease in self.pending.drain(..) {
            lease.retire();
        }
        self.due.clear();
        self.owner = None;
        self.floor = 0;
    }
}
impl Drop for Queue {
    fn drop(&mut self) {
        self.retire();
    }
}

/// Worker-side resource cache. An error is latched for this root; changing the
/// root can reload resources but can never revive a retired World lease.
#[derive(Default)]
pub(crate) struct Stage {
    root: Option<PathBuf>,
    bank: Option<Arc<Static2941Bank>>,
    error: Option<String>,
    history: History,
}

/// History shares the original one-way tokens. Changing resource roots or
/// merging a late worker may preserve evidence, never manufacture a lifetime.
#[derive(Clone, Default)]
pub(crate) struct History {
    leases: Vec<Lease>,
    source_error: Option<(PathBuf, String)>,
}
impl History {
    pub(crate) fn for_world(world: &World) -> Self {
        Self {
            leases: world
                .static_effect_receipts
                .values()
                .filter_map(Receipt::lease)
                .filter(|lease| lease.current(world, lease.connection()))
                .cloned()
                .collect(),
            source_error: None,
        }
    }
    pub(crate) fn for_source(&self, same: bool) -> Self {
        let mut next = self.clone();
        next.leases.retain(Lease::active);
        if !same {
            next.source_error = None;
        }
        next
    }
    pub(crate) fn merge(&mut self, other: &Self) {
        self.leases.retain(Lease::active);
        for lease in &other.leases {
            if lease.active() && !self.leases.contains(lease) && self.leases.len() < CAPACITY {
                self.leases.push(lease.clone());
            }
        }
    }
}

/// Retained by frames and painted input surfaces, including speculative ones.
#[derive(Clone, Default)]
pub(crate) struct DrawGuard(Vec<Lease>);
impl DrawGuard {
    pub(crate) fn current(&self, world: &World, connection: u64) -> bool {
        self.0
            .iter()
            .all(|lease| lease.drawable() && lease.current(world, connection))
    }
    pub(crate) fn active(&self, world: &World) -> bool {
        self.0
            .iter()
            .all(|lease| lease.current(world, lease.connection()))
    }
    pub(crate) fn publishable(&self, world: &World) -> bool {
        self.0
            .iter()
            .all(|lease| lease.drawable() && lease.current(world, lease.connection()))
    }
    pub(crate) fn sequences(&self) -> Vec<u64> {
        self.0.iter().map(Lease::sequence).collect()
    }
}
impl Stage {
    pub(crate) fn history(&self) -> History {
        let mut history = self.history.clone();
        history.source_error = self.root.clone().zip(self.error.clone());
        history
    }
    pub(crate) fn restore(&mut self, history: &History) {
        self.history.merge(history);
        if let Some((root, error)) = &history.source_error {
            self.root = Some(root.clone());
            self.error = Some(error.clone());
        }
    }

    /// Pure composition may precede enqueue on the worker. The returned guard
    /// prevents those pixels from publication until the real controller sample,
    /// and revokes them immediately after callback, even in copied snapshots.
    pub(crate) fn draw_to<'s>(
        &'s self,
        world: &World,
        target: &mut super::native_world_effects::Target<'_, 's>,
        view: super::world_map::ViewProjection,
        cancel: &AtomicBool,
    ) -> Result<DrawGuard> {
        let mut guard = DrawGuard::default();
        let Some(bank) = &self.bank else {
            return Ok(guard);
        };
        for cue in &world.effects {
            let Some(lease) = world
                .static_effect_receipts
                .get(&cue.sequence)
                .and_then(Receipt::lease)
                .filter(|lease| lease.current(world, lease.connection()))
            else {
                continue;
            };
            bank.submit_frame_zero(
                target,
                super::world_map::WorldCell {
                    x: i32::from(cue.position[0]),
                    y: i32::from(cue.position[1]),
                },
                view,
                cancel,
            )?;
            guard.0.push(lease.clone());
        }
        Ok(guard)
    }
    pub(crate) fn prepare(
        &mut self,
        root: &Path,
        world: &World,
        cancel: &AtomicBool,
        shared: &mut Option<SharedActionCatalog>,
    ) -> Result<Vec<Prepared>> {
        super::tower_assets::active(cancel)?;
        self.history = History::for_world(world);
        if self.root.as_deref() != Some(root) {
            self.root = Some(root.to_owned());
            self.bank = None;
            self.error = None;
        }
        let leases: Vec<_> = world
            .static_effect_receipts
            .values()
            .filter_map(Receipt::lease)
            .filter(|lease| lease.current(world, lease.connection()))
            .collect();
        if leases.is_empty() {
            return Ok(vec![]);
        }
        if self.bank.is_none() && self.error.is_none() {
            let loaded = (|| -> Result<Static2941Bank> {
                let catalog = SharedActionCatalog::reuse_for_model(root, cancel, shared)?;
                Ok(Static2941Bank::load(catalog, cancel)?)
            })();
            match loaded {
                Ok(bank) => self.bank = Some(Arc::new(bank)),
                Err(error) => {
                    super::tower_assets::active(cancel)?;
                    self.error = Some(error.to_string());
                }
            }
        }
        let Some(bank) = &self.bank else {
            return Ok(vec![]);
        };
        let mut proofs = Vec::new();
        for lease in leases {
            if let Some(proof) = Prepared::from_bank(bank.clone(), lease, cancel)? {
                proofs.push(proof);
            }
        }
        Ok(proofs)
    }
    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

#[cfg(test)]
#[path = "native_static2941_tests.rs"]
mod tests;
