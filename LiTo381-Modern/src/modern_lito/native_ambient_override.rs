//! Candidate class2 ambient lease ledger. Not registered in the product.
//! 4271D92F: 44EBD0 enters override15 at native source frame2; overlapping
//! effects share648EFC and restore saved648EF8 when the last effect finishes.
//! The UI/controller bridge must provide the current published scene context.
//! No World/HP/MP, network, device, file, or playback clock mutation lives here.
use super::native_environment::State;
use anyhow::{ensure, Context as _, Result};
use std::{collections::{BTreeMap, BTreeSet}, sync::{Arc, atomic::{AtomicBool, Ordering}}};

const MAX_ACTIVE: usize = 64;
const MAX_SEEN: usize = 4096;

#[derive(Clone, Debug)]
pub struct Context {
    connection: u64,
    generation: u64,
    map: u16,
    publication: Arc<()>,
}
impl Context {
    pub fn new(connection: u64, generation: u64, map: u16, publication: Arc<()>) -> Result<Self> {
        ensure!(connection != 0 && generation != 0, "ambient scene identity absent");
        Ok(Self {connection, generation, map, publication})
    }
    fn same(&self, other: &Self) -> bool {
        self.connection == other.connection && self.generation == other.generation
            && self.map == other.map && Arc::ptr_eq(&self.publication, &other.publication)
    }
    fn environment_matches(&self, state: &State) -> bool {
        state.epoch == Some(self.connection) && state.map == Some(self.map)
    }
}

/// Received S23 owner/incarnation/action sequence, scoped to canonical11736.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identity { actor: u32, incarnation: u64, sequence: u64 }
impl Identity {
    pub fn call_lightning(actor: u32, incarnation: u64, sequence: u64) -> Result<Self> {
        ensure!(actor != 0 && incarnation != 0 && sequence != 0, "ambient effect identity absent");
        Ok(Self {actor, incarnation, sequence})
    }
}
#[derive(Debug)]
struct LeaseData {
    manager: Arc<()>,
    context: Context,
    identity: Identity,
    consumed: AtomicBool,
}
#[derive(Clone, Debug)]
pub struct Lease(Arc<LeaseData>);
#[derive(Debug)]
pub enum Entry { Admitted(Lease), IgnoredFrame, Duplicate, Stale, AtCapacity }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finish { Released, AlreadyConsumed, Stale }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Retirement { Empty, Restored, PreservedNewMap, ReplacedEnvironment }

/// Deliberately not Clone. One live UI owns the ledger; workers emit immutable
/// frame-entry/finish intents and must not clone or directly apply this state.
#[derive(Debug, Default)]
pub struct Ledger {
    manager: Arc<()>,
    context: Option<Context>,
    active: BTreeMap<Identity, Arc<LeaseData>>,
    seen: BTreeSet<Identity>,
    sequence_floor: u64,
    saved_level: Option<u8>,
}
impl Ledger {
    pub fn active_count(&self) -> usize { self.active.len() }
    pub fn retained_identities(&self) -> usize { self.seen.len() }
    fn current(&self, context: &Context, state: &State) -> bool {
        self.context.as_ref().is_some_and(|c| c.same(context)) && context.environment_matches(state)
    }
    fn counter_owned(&self, state: &State) -> Result<()> {
        ensure!(state.ambient_override_counter == i32::try_from(self.active.len())?,
            "ambient counter belongs to another ledger or replacement state");
        Ok(())
    }
    fn next_revision(state: &State) -> Result<u64> {
        state.revision.checked_add(1).context("ambient revision exhausted")
    }
    fn clear(&mut self) {
        for lease in self.active.values() { lease.consumed.store(true, Ordering::Release); }
        self.active.clear(); self.seen.clear(); self.saved_level = None;
        self.context = None; self.sequence_floor = 1; self.manager = Arc::new(());
    }
    /// Call when publication, world generation, map, or GAME owner changes.
    /// A changed map keeps its newly selected level; a replaced GAME state is
    /// untouched. This prevents the retiring scene from restoring into it.
    pub fn retire(&mut self, state: &mut State) -> Result<Retirement> {
        let Some(context) = self.context.as_ref() else {
            self.clear(); return Ok(Retirement::Empty);
        };
        if self.active.is_empty() { self.clear(); return Ok(Retirement::Empty); }
        if state.epoch != Some(context.connection) || state.ambient_override_counter == 0 {
            self.clear(); return Ok(Retirement::ReplacedEnvironment);
        }
        self.counter_owned(state)?;
        let revision = Self::next_revision(state)?;
        let same_map = state.map == Some(context.map);
        let saved = self.saved_level.context("ambient saved level missing")?;
        state.ambient_override_counter = 0;
        if same_map { state.ambient_level = Some(saved); }
        state.revision = revision;
        self.clear();
        Ok(if same_map {Retirement::Restored} else {Retirement::PreservedNewMap})
    }
    pub fn bind(&mut self, context: Context, state: &mut State) -> Result<Retirement> {
        ensure!(context.environment_matches(state), "ambient context/environment mismatch");
        ensure!(state.ambient_level.is_some_and(|v| v <= 15), "ambient baseline unavailable");
        if self.context.as_ref().is_some_and(|old| old.same(&context)) {
            self.counter_owned(state)?; return Ok(Retirement::Empty);
        }
        // Validate before retiring an old lease: no partially applied bind.
        let expected = if self.context.as_ref().is_some_and(|c| state.epoch == Some(c.connection)) {
            i32::try_from(self.active.len())?
        } else {0};
        ensure!(state.ambient_override_counter == expected || state.ambient_override_counter == 0,
            "foreign ambient counter during scene binding");
        let retired = self.retire(state)?;
        ensure!(state.ambient_override_counter == 0, "ambient counter not retired");
        self.context = Some(context); self.sequence_floor = 1;
        Ok(retired)
    }
    /// The caller supplies the minimum retained, received S23 action sequence.
    /// Pruning cannot make an older deferred frame admissible again.
    pub fn prune_before(&mut self, context: &Context, oldest_retained_sequence: u64) -> Result<()> {
        ensure!(self.context.as_ref().is_some_and(|c| c.same(context)), "stale ambient history prune");
        ensure!(oldest_retained_sequence >= self.sequence_floor && oldest_retained_sequence != 0,
            "ambient source sequence floor regressed");
        self.sequence_floor = oldest_retained_sequence;
        self.seen.retain(|key| key.sequence >= oldest_retained_sequence);
        Ok(())
    }
    pub fn enter_frame(&mut self, context: &Context, state: &mut State,
        identity: Identity, native_source_frame: usize) -> Result<Entry> {
        if !self.current(context, state) || identity.sequence < self.sequence_floor { return Ok(Entry::Stale); }
        if native_source_frame != 2 { return Ok(Entry::IgnoredFrame); }
        if self.seen.contains(&identity) || self.active.contains_key(&identity) { return Ok(Entry::Duplicate); }
        if self.active.len() >= MAX_ACTIVE || self.seen.len() >= MAX_SEEN { return Ok(Entry::AtCapacity); }
        self.counter_owned(state)?;
        let level = state.ambient_level.filter(|v| *v <= 15).context("ambient baseline unavailable")?;
        let revision = Self::next_revision(state)?;
        let lease = Arc::new(LeaseData {manager: self.manager.clone(), context: context.clone(),
            identity, consumed: AtomicBool::new(false)});
        if self.active.is_empty() { self.saved_level = Some(level); state.ambient_level = Some(15); }
        self.active.insert(identity, lease.clone()); self.seen.insert(identity);
        state.ambient_override_counter = i32::try_from(self.active.len())?; state.revision = revision;
        Ok(Entry::Admitted(Lease(lease)))
    }
    pub fn finish(&mut self, context: &Context, state: &mut State, lease: &Lease) -> Result<Finish> {
        if !self.current(context, state) || !lease.0.context.same(context)
            || !Arc::ptr_eq(&self.manager, &lease.0.manager) { return Ok(Finish::Stale); }
        if lease.0.consumed.load(Ordering::Acquire) { return Ok(Finish::AlreadyConsumed); }
        self.counter_owned(state)?;
        ensure!(self.active.get(&lease.0.identity).is_some_and(|item| Arc::ptr_eq(item, &lease.0)),
            "ambient lease missing from its ledger");
        let revision = Self::next_revision(state)?;
        let saved = self.saved_level.context("ambient saved level missing")?;
        if lease.0.consumed.swap(true, Ordering::AcqRel) { return Ok(Finish::AlreadyConsumed); }
        self.active.remove(&lease.0.identity);
        state.ambient_override_counter = i32::try_from(self.active.len())?;
        if self.active.is_empty() { state.ambient_level = Some(saved); self.saved_level = None; }
        state.revision = revision;
        Ok(Finish::Released)
    }
}
