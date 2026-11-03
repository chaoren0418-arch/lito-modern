//! Conditional #2941 lifecycle core; deliberately absent from the normal graph.
//!
//! PE4271D92F: 44EF80 loads static source frame zero and queues 44F330 with
//! delay zero. 4D65F0 inserts after equal deadlines; 4D6570 samples once,
//! removes a due entry before calling it, then inspects the queue again.
//! Drawing is an independent observation between callbacks, never a timer.
//! This module does not bind the original clock, all global callbacks, assets,
//! actor admission, S37 ingress, History, or actual pixels. No runtime claim.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    WrongOwner,
    Capacity,
    ClockRegressed,
    ClockWrapUnqualified,
    InvalidHeading,
    DuplicateEffect,
    Retired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub object_id: u32,
    pub x: u16,
    pub y: u16,
    pub heading: u8,
}

/// Only a lookup request. The loader must independently qualify the canonical
/// member and original fallback branch before turning it into a draw command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameZero {
    pub sequence: u64,
    pub placement: Placement,
}
impl FrameZero {
    pub const GRAPHICS: u16 = 2941;
    pub const SUBFRAME: u32 = 0;
    pub const ORIGINAL_MISSING_SPR_FALLBACK: u16 = 1120;
}

enum Callback {
    Retire97(u64),
    External(u64),
}
struct Due {
    at: u32,
    callback: Callback,
}

/// External callbacks return control to the caller before another due entry
/// is removed. A reentrant redraw may observe frame zero in that interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dispatch {
    Retired97 { sequence: u64 },
    External { id: u64 },
}

/// The owner must carry the caller's complete connection/publication identity.
/// Equality here is a guard, not proof of whole-GAME atomic admission or ABA.
/// No Clone/restore API can resurrect a dispatched retirement from a worker.
pub struct ConditionalModel<Owner> {
    owner: Owner,
    capacity: usize,
    queue: Vec<Due>,
    seen: BTreeSet<u64>,
    active: BTreeMap<u64, FrameZero>,
    last_sample: u32,
    retired: bool,
}

/// A nontransferable drain borrows its model. Its fixed sample cannot be
/// replayed on another owner/model or reused after this drain has ended.
/// Nested timer drains remain outside this bounded draft's qualification.
pub struct Drain<'a, Owner> {
    model: &'a mut ConditionalModel<Owner>,
    owner: &'a Owner,
    sample: u32,
}

impl<Owner: Eq> ConditionalModel<Owner> {
    pub fn new(owner: Owner, explicit_initial_tick: u32, capacity: NonZeroUsize) -> Self {
        Self {
            owner,
            capacity: capacity.get(),
            queue: Vec::new(),
            seen: BTreeSet::new(),
            active: BTreeMap::new(),
            last_sample: explicit_initial_tick,
            retired: false,
        }
    }

    fn guard(&self, owner: &Owner) -> Result<(), Error> {
        if self.retired { return Err(Error::Retired); }
        if owner != &self.owner { return Err(Error::WrongOwner); }
        Ok(())
    }

    fn insertion(&self, fresh_tick: u32, delay: u32) -> Result<(usize, u32), Error> {
        if self.queue.len() >= self.capacity { return Err(Error::Capacity); }
        if fresh_tick < self.last_sample { return Err(Error::ClockRegressed); }
        // Native x86 addition wraps. This bounded draft refuses that domain;
        // it must not silently reinterpret wrapping deadlines as u64 time.
        let at = fresh_tick.checked_add(delay).ok_or(Error::ClockWrapUnqualified)?;
        let position = self.queue.partition_point(|entry| entry.at <= at);
        Ok((position, at))
    }

    /// The caller binds a real admitted S37 sequence and captured placement.
    /// An invalid/duplicate/full admission changes neither queue nor history.
    pub fn admit97(
        &mut self, owner: &Owner, sequence: u64, placement: Placement, fresh_tick: u32,
    ) -> Result<(), Error> {
        self.guard(owner)?;
        if placement.heading > 7 { return Err(Error::InvalidHeading); }
        if self.seen.contains(&sequence) { return Err(Error::DuplicateEffect); }
        if self.seen.len() >= self.capacity { return Err(Error::Capacity); }
        let (position, at) = self.insertion(fresh_tick, 0)?;
        self.queue.insert(position, Due { at, callback: Callback::Retire97(sequence) });
        self.active.insert(sequence, FrameZero { sequence, placement });
        self.seen.insert(sequence);
        self.last_sample = fresh_tick;
        Ok(())
    }

    pub fn enqueue_external(
        &mut self, owner: &Owner, id: u64, fresh_tick: u32, delay: u32,
    ) -> Result<(), Error> {
        self.guard(owner)?;
        let (position, at) = self.insertion(fresh_tick, delay)?;
        self.queue.insert(position, Due { at, callback: Callback::External(id) });
        self.last_sample = fresh_tick;
        Ok(())
    }

    /// Capture once for a drain. Fresh enqueue clocks during its callbacks may
    /// exceed this sample; their entries then remain pending for a later drain.
    pub fn begin_drain<'a>(
        &'a mut self, owner: &'a Owner, tick: u32,
    ) -> Result<Drain<'a, Owner>, Error> {
        self.guard(owner)?;
        if tick < self.last_sample { return Err(Error::ClockRegressed); }
        self.last_sample = tick;
        Ok(Drain { model: self, owner, sample: tick })
    }

    /// Remove before handing control back. Do not loop through external
    /// callbacks and concatenate their outputs after all retirements.
    fn dispatch_one(
        &mut self, owner: &Owner, sample: u32,
    ) -> Result<Option<Dispatch>, Error> {
        self.guard(owner)?;
        if !self.queue.first().is_some_and(|entry| entry.at <= sample) {
            return Ok(None);
        }
        let due = self.queue.remove(0);
        Ok(Some(match due.callback {
            Callback::External(id) => Dispatch::External { id },
            Callback::Retire97(sequence) => {
                self.active.remove(&sequence);
                Dispatch::Retired97 { sequence }
            }
        }))
    }

    /// Does not advance the queue or promise that an actual frame was painted.
    pub fn pending_frame_zero(&self, owner: &Owner) -> Result<Vec<FrameZero>, Error> {
        self.guard(owner)?;
        Ok(self.active.values().copied().collect())
    }

    /// Owner retirement clears work but retains admission tombstones in this
    /// retired instance. A new owner must get a distinct model, never reset it.
    pub fn retire(&mut self) {
        self.retired = true;
        self.queue.clear();
        self.active.clear();
    }
}

impl<Owner: Eq> Drain<'_, Owner> {
    pub fn dispatch_one(&mut self) -> Result<Option<Dispatch>, Error> {
        self.model.dispatch_one(self.owner, self.sample)
    }

    pub fn enqueue_external(&mut self, id: u64, fresh_tick: u32, delay: u32) -> Result<(), Error> {
        self.model.enqueue_external(self.owner, id, fresh_tick, delay)
    }

    pub fn admit97(
        &mut self, sequence: u64, placement: Placement, fresh_tick: u32,
    ) -> Result<(), Error> {
        self.model.admit97(self.owner, sequence, placement, fresh_tick)
    }

    pub fn pending_frame_zero(&self) -> Result<Vec<FrameZero>, Error> {
        self.model.pending_frame_zero(self.owner)
    }

    pub fn retire(&mut self) { self.model.retire(); }
}
