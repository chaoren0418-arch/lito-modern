//! Controller-owned zero-delay retirement. Workers hold observers, never timers.
//! This is the modern static2941 queue, not a certificate that every original
//! native actor callback/reentrant redraw has joined one shared global queue.
use super::{native_connection::IngressReceipt, native_world::World};
use anyhow::{ensure, Context, Result};
use std::{sync::{Arc,atomic::{AtomicBool,Ordering}},time::Instant};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct Owner { connection:u64, generation:u64 }

#[derive(Debug)]
struct Live {
    owner:Owner,
    sequence:u64,
    decoded_at:Instant,
    dispatched_at:Instant,
    constructed_at:Instant,
    enqueued_at:Instant,
    active:AtomicBool,
}

/// Cloneable observation only. Releasing or cancelling a worker cannot retire
/// the controller's callback; an old snapshot cannot turn this token back on.
#[derive(Clone,Debug)]
pub struct Lease(Arc<Live>);
impl PartialEq for Lease {fn eq(&self,other:&Self)->bool{Arc::ptr_eq(&self.0,&other.0)}}
impl Eq for Lease {}
impl Lease {
    pub fn active(&self)->bool {self.0.active.load(Ordering::Acquire)}
    pub fn sequence(&self)->u64 {self.0.sequence}
    pub fn connection(&self)->u64 {self.0.owner.connection}
    pub fn generation(&self)->u64 {self.0.owner.generation}
    pub fn decoded_at(&self)->Instant {self.0.decoded_at}
    pub fn dispatched_at(&self)->Instant {self.0.dispatched_at}
    pub fn constructed_at(&self)->Instant {self.0.constructed_at}
    pub fn enqueued_at(&self)->Instant {self.0.enqueued_at}
    /// Native delay is zero: the deadline is the FRESH enqueue sample, not
    /// decode, controller dispatch, constructor, asset-load or paint time.
    pub fn deadline(&self)->Instant {self.0.enqueued_at}
    pub fn current(&self,world:&World,connection:u64)->bool {
        self.active() && self.connection()==connection && self.generation()==world.generation
            && world.effects.iter().find(|e|e.sequence==self.sequence()).is_some_and(|cue|
                world.static_effect_receipts.get(&cue.sequence).is_some_and(|receipt|
                    receipt.matches(world.generation,connection,cue)
                    && receipt.scheduled_lease().is_some_and(|lease|lease==self)))
    }
    fn retire(&self) {self.0.active.store(false,Ordering::Release);}
}

#[derive(Debug)]
struct Due { lease:Lease }
#[derive(Default,Debug)]
pub(crate) struct DrainReport {
    pub sampled_at:Option<Instant>,
    pub retired_sequences:Vec<u64>,
}

/// Never cloned into World/Model/History. A scalar admission watermark and the
/// World's 128-entry ring bound memory without reviving an evicted sequence.
#[derive(Default)]
pub(crate) struct Queue {
    owner:Option<Owner>,
    admission_floor:u64,
    due:Vec<Due>,
}
impl Queue {
    pub(crate) fn synchronize(&mut self,world:&World,connection:u64) {
        let owner=(connection!=0 && world.generation!=0 && world.map_id.is_some())
            .then_some(Owner{connection,generation:world.generation});
        if self.owner!=owner {
            self.retire();self.owner=owner;
        }
        self.due.retain(|entry| {
            let current=entry.lease.current(world,connection);
            if !current {entry.lease.retire();}
            current
        });
    }

    /// Called immediately after this exact controller packet's World admission.
    /// It never scans old unbound cues and invents a new dispatch for them.
    pub(crate) fn admit_received(
        &mut self,world:&mut World,connection:u64,ingress:IngressReceipt,dispatched_at:Instant,
    )->Result<bool> {
        ensure!(connection!=0 && ingress.connection_epoch()==connection,"static callback connection mismatch");
        ensure!(ingress.decoded_at()<=dispatched_at,"static dispatch precedes packet decode");
        self.synchronize(world,connection);
        let Some(owner)=self.owner else{return Ok(false)};
        let Some(cue)=world.effects.last().filter(|cue|cue.graphics==2941).cloned()else{return Ok(false)};
        let Some(receipt)=world.static_effect_receipts.get(&cue.sequence)else{return Ok(false)};
        if receipt.ingress()!=ingress || !receipt.matches(world.generation,connection,&cue) {return Ok(false);}
        if cue.sequence<=self.admission_floor {return Ok(false);}
        if receipt.scheduled_lease().is_some() {
            self.admission_floor=cue.sequence;return Ok(false);
        }
        ensure!(self.due.len()<128,"static callback queue exceeds effect ring");
        let constructed_at=Instant::now();
        ensure!(dispatched_at<=constructed_at,"static constructor clock regressed");
        let enqueued_at=Instant::now();
        ensure!(constructed_at<=enqueued_at,"static enqueue clock regressed");
        let lease=Lease(Arc::new(Live {owner,sequence:cue.sequence,decoded_at:ingress.decoded_at(),
            dispatched_at,constructed_at,enqueued_at,active:AtomicBool::new(true)}));
        // 4D65F0 inserts AFTER all earlier/equal deadlines. Zero delay remains
        // queued; it is never a synchronous constructor call or a one-frame TTL.
        let at=self.due.partition_point(|entry|entry.lease.deadline()<=lease.deadline());
        world.static_effect_receipts.get_mut(&cue.sequence).expect("validated receipt")
            .bind_scheduled(lease.clone());
        self.due.insert(at,Due{lease});self.admission_floor=cue.sequence;Ok(true)
    }

    /// Caller samples once at the actual controller drain boundary. Removing a
    /// due entry precedes its callback. Rendering never calls this operation.
    pub(crate) fn drain_due(&mut self,world:&mut World,connection:u64,sampled_at:Instant)->Result<DrainReport> {
        self.synchronize(world,connection);
        let count=self.due.iter().take_while(|entry|entry.lease.deadline()<=sampled_at).count();
        let revision=if count!=0 {Some(world.visual_revision.checked_add(1).context("static retirement visual revision")?)}else{None};
        let mut report=DrainReport{sampled_at:Some(sampled_at),..Default::default()};
        while self.due.first().is_some_and(|entry|entry.lease.deadline()<=sampled_at) {
            let entry=self.due.remove(0);
            entry.lease.retire();report.retired_sequences.push(entry.lease.sequence());
        }
        if let Some(revision)=revision {world.visual_revision=revision;}
        Ok(report)
    }

    pub(crate) fn retire(&mut self) {
        for entry in self.due.drain(..) {entry.lease.retire();}
        self.owner=None;self.admission_floor=0;
    }
}
impl Drop for Queue {fn drop(&mut self){self.retire();}}
