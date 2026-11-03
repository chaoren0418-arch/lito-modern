//! Static2941: receive -> pending node -> verified SPR -> fresh queue sample.
//! Controllers own retirement. Workers can produce resource proofs, not clocks
//! or timer authority. Full original global callback equivalence is unqualified.
use super::{native_connection::IngressReceipt,native_world::World,tower_assets::Static2941Bank};
use anyhow::{ensure,Context,Result};
use std::{path::Path,sync::{Arc,OnceLock,atomic::{AtomicBool,AtomicU8,Ordering}},time::Instant};
const WAITING:u8=0;const QUEUED:u8=1;const RETIRED:u8=2;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]struct Owner{connection:u64,generation:u64}
#[derive(Debug)]struct Schedule{resource_ready_at:Instant,enqueued_at:Instant}
#[derive(Debug)]struct Live{
    owner:Owner,sequence:u64,decoded_at:Instant,dispatched_at:Instant,constructed_at:Instant,
    schedule:OnceLock<Schedule>,phase:AtomicU8,
}
#[derive(Clone,Debug)]pub struct Lease(Arc<Live>);
impl PartialEq for Lease{fn eq(&self,other:&Self)->bool{Arc::ptr_eq(&self.0,&other.0)}}
impl Eq for Lease{}
impl Lease{
    pub fn active(&self)->bool{self.0.phase.load(Ordering::Acquire)!=RETIRED}
    pub fn waiting(&self)->bool{self.0.phase.load(Ordering::Acquire)==WAITING}
    pub fn drawable(&self)->bool{self.0.phase.load(Ordering::Acquire)==QUEUED}
    pub fn sequence(&self)->u64{self.0.sequence}
    pub fn connection(&self)->u64{self.0.owner.connection}
    pub fn generation(&self)->u64{self.0.owner.generation}
    pub fn decoded_at(&self)->Instant{self.0.decoded_at}
    pub fn dispatched_at(&self)->Instant{self.0.dispatched_at}
    pub fn constructed_at(&self)->Instant{self.0.constructed_at}
    pub fn resource_ready_at(&self)->Option<Instant>{self.0.schedule.get().map(|s|s.resource_ready_at)}
    pub fn enqueued_at(&self)->Option<Instant>{self.0.schedule.get().map(|s|s.enqueued_at)}
    pub fn deadline(&self)->Option<Instant>{self.enqueued_at()}
    pub fn current(&self,world:&World,connection:u64)->bool{
        self.active()&&self.connection()==connection&&self.generation()==world.generation
        &&world.effects.iter().find(|e|e.sequence==self.sequence()).is_some_and(|cue|
            world.static_effect_receipts.get(&cue.sequence).is_some_and(|r|
                r.matches(world.generation,connection,cue)&&r.scheduled_lease().is_some_and(|v|v==self)))
    }
    fn retire(&self){self.0.phase.store(RETIRED,Ordering::Release);}
}

/// A real bank is required: no booleans or fabricated frame-ready timestamps.
/// Carries the bank across the worker/controller handoff and until retirement.
pub(crate) struct Prepared{lease:Lease,bank:Arc<Static2941Bank>,ready_at:Instant}
impl Prepared{
    pub(crate) fn from_bank(bank:Arc<Static2941Bank>,lease:&Lease,cancel:&AtomicBool)->Result<Option<Self>>{
        super::tower_assets::active(cancel)?;
        if !lease.waiting(){return Ok(None);}
        ensure!(bank.frame_zero().fragments.is_some()&&bank.native_primary_flags()==0,"static frame not qualified");
        let ready_at=Instant::now();ensure!(lease.constructed_at()<=ready_at,"static resource clock regressed");
        super::tower_assets::active(cancel)?;
        Ok(lease.waiting().then(||Self{lease:lease.clone(),bank,ready_at}))
    }
    pub(crate) fn same_ticket(&self,other:&Self)->bool{self.lease==other.lease}
    pub(crate) fn current(&self,world:&World,connection:u64,root:&Path)->bool{
        self.lease.waiting()&&self.lease.current(world,connection)
            &&(root==self.bank.root()||root.canonicalize().ok().as_deref()==Some(self.bank.root()))
    }
}
struct Due{lease:Lease,at:Instant,_bank:Arc<Static2941Bank>}
#[derive(Default,Debug)]pub(crate) struct DrainReport{pub sampled_at:Option<Instant>,pub retired_sequences:Vec<u64>}
#[derive(Default)]pub(crate) struct Queue{owner:Option<Owner>,admission_floor:u64,pending:Vec<Lease>,due:Vec<Due>}
impl Queue{
    pub(crate) fn synchronize(&mut self,world:&World,connection:u64){
        let owner=(connection!=0&&world.generation!=0&&world.map_id.is_some()).then_some(Owner{connection,generation:world.generation});
        if self.owner!=owner{self.retire();self.owner=owner;}
        self.pending.retain(|lease|{let current=lease.current(world,connection);if !current{lease.retire();}current});
        self.due.retain(|entry|entry.lease.current(world,connection));
    }
    /// Receiving a cue creates a pending observer. It creates NO deadline.
    pub(crate) fn admit_received(&mut self,world:&mut World,connection:u64,ingress:IngressReceipt,dispatched_at:Instant)->Result<bool>{
        ensure!(connection!=0&&ingress.connection_epoch()==connection,"static callback connection mismatch");
        ensure!(ingress.decoded_at()<=dispatched_at,"static dispatch precedes packet decode");
        self.synchronize(world,connection);let Some(owner)=self.owner else{return Ok(false)};
        let Some(cue)=world.effects.last().filter(|cue|cue.graphics==2941).cloned()else{return Ok(false)};
        let Some(receipt)=world.static_effect_receipts.get(&cue.sequence)else{return Ok(false)};
        if receipt.ingress()!=ingress||!receipt.matches(world.generation,connection,&cue){return Ok(false);}
        if cue.sequence<=self.admission_floor{return Ok(false);}
        if receipt.scheduled_lease().is_some(){self.admission_floor=cue.sequence;return Ok(false);}
        ensure!(self.pending.len()<128,"static pending nodes exceed effect ring");
        let constructed_at=Instant::now();ensure!(dispatched_at<=constructed_at,"static constructor clock regressed");
        let lease=Lease(Arc::new(Live{owner,sequence:cue.sequence,decoded_at:ingress.decoded_at(),dispatched_at,
            constructed_at,schedule:OnceLock::new(),phase:AtomicU8::new(WAITING)}));
        world.static_effect_receipts.get_mut(&cue.sequence).expect("validated receipt").bind_scheduled(lease.clone());
        self.pending.push(lease);self.admission_floor=cue.sequence;Ok(true)
    }
    /// Only accepted, still-current worker results may reach this controller
    /// boundary. The qualified SPR already exists before taking a fresh clock.
    pub(crate) fn enqueue_prepared(&mut self,world:&mut World,connection:u64,root:&Path,prepared:Prepared)->Result<bool>{
        self.synchronize(world,connection);
        if !prepared.current(world,connection,root)||!self.pending.iter().any(|p|*p==prepared.lease){return Ok(false);}
        let enqueued_at=Instant::now();ensure!(prepared.ready_at<=enqueued_at,"static enqueue precedes verified SPR");
        if prepared.lease.0.schedule.get().is_some(){return Ok(false);}
        let revision=world.visual_revision.checked_add(1).context("static ready visual revision")?;
        let schedule=Schedule{resource_ready_at:prepared.ready_at,enqueued_at};
        ensure!(prepared.lease.0.schedule.set(schedule).is_ok(),"static queue scheduled twice");
        prepared.lease.0.phase.store(QUEUED,Ordering::Release);
        let at=self.due.partition_point(|entry|entry.at<=enqueued_at);
        self.due.insert(at,Due{lease:prepared.lease,at:enqueued_at,_bank:prepared.bank});
        world.visual_revision=revision;Ok(true)
    }
    pub(crate) fn drain_due(&mut self,world:&mut World,connection:u64,sampled_at:Instant)->Result<DrainReport>{
        self.synchronize(world,connection);
        let count=self.due.iter().take_while(|entry|entry.at<=sampled_at).count();
        let revision=if count!=0{Some(world.visual_revision.checked_add(1).context("static retirement visual revision")?)}else{None};
        let mut report=DrainReport{sampled_at:Some(sampled_at),..Default::default()};
        while self.due.first().is_some_and(|entry|entry.at<=sampled_at){
            let entry=self.due.remove(0);entry.lease.retire();report.retired_sequences.push(entry.lease.sequence());
        }
        self.pending.retain(Lease::active);if let Some(revision)=revision{world.visual_revision=revision;}Ok(report)
    }
    pub(crate) fn retire(&mut self){for lease in self.pending.drain(..){lease.retire();}self.due.clear();self.owner=None;self.admission_floor=0;}
}
impl Drop for Queue{fn drop(&mut self){self.retire();}}

/// Metadata only. A source switch can reload resources, never reset a token.
#[derive(Clone,Default)]pub(crate) struct History{leases:Vec<Lease>,error:Option<String>}
impl History{
    pub(crate) fn for_world(world:&World)->Self{Self{leases:world.static_effect_receipts.values().filter_map(|r|r.scheduled_lease()).filter(|l|l.current(world,l.connection())).cloned().collect(),error:None}}
    pub(crate) fn active_sequences(&self)->std::collections::BTreeSet<u64>{self.leases.iter().filter(|l|l.active()).map(Lease::sequence).collect()}
    pub(crate) fn for_source(&self,same:bool)->Self{let mut h=self.clone();h.leases.retain(Lease::active);if !same{h.error=None;}h}
    pub(crate) fn merge(&mut self,other:&Self){self.leases.retain(Lease::active);for l in &other.leases{if l.active()&&!self.leases.contains(l)&&self.leases.len()<128{self.leases.push(l.clone());}}}
}
#[derive(Default)]pub(crate) struct Stage{
    bank:Option<Arc<Static2941Bank>>,error:Option<String>,tickets:Vec<(super::native_skills::Effect,Lease)>,history:History,
}
impl Stage{
    pub(crate) fn history(&self)->History{let mut h=self.history.clone();h.error=self.error.clone();h}
    pub(crate) fn restore_history(&mut self,h:History){self.error=h.error.clone();self.history=h;}
    pub(crate) fn prepare(&mut self,root:&Path,world:&World,cancel:&AtomicBool,shared:&mut Option<super::shared_action_catalog::SharedActionCatalog>)->Result<Vec<Prepared>>{
        super::tower_assets::active(cancel)?;
        self.tickets=world.effects.iter().filter(|e|e.graphics==2941).filter_map(|cue|
            world.static_effect_receipts.get(&cue.sequence).and_then(|r|r.scheduled_lease())
                .filter(|l|l.current(world,l.connection())).map(|l|(cue.clone(),l.clone()))).collect();
        self.history.leases=self.tickets.iter().map(|(_,l)|l.clone()).collect();
        if self.tickets.is_empty(){return Ok(vec![]);}
        if self.bank.is_none()&&self.error.is_none(){
            let loaded=(||->Result<Static2941Bank>{let c=super::shared_action_catalog::SharedActionCatalog::reuse_for_model(root,cancel,shared)?;Ok(Static2941Bank::load(c,cancel)?)})();
            match loaded{Ok(bank)=>self.bank=Some(Arc::new(bank)),Err(e)=>{super::tower_assets::active(cancel)?;self.error=Some(e.to_string());}}
        }
        let Some(bank)=&self.bank else{return Ok(vec![])};
        let mut ready=vec![];
        for(_,lease)in &self.tickets{if lease.current(world,lease.connection()){if let Some(p)=Prepared::from_bank(bank.clone(),lease,cancel)?{ready.push(p);}}}
        Ok(ready)
    }
    pub(crate) fn draw_to<'s>(&'s self,world:&World,target:&mut super::native_world_effects::Target<'_,'s>,view:super::world_map::ViewProjection,cancel:&AtomicBool)->Result<Vec<Lease>>{
        let mut used=vec![];let Some(bank)=&self.bank else{return Ok(used)};
        for(cue,lease)in &self.tickets{
            if lease.drawable()&&lease.current(world,lease.connection()){
                bank.submit_frame_zero(target,super::world_map::WorldCell{x:i32::from(cue.position[0]),y:i32::from(cue.position[1])},view,cancel)?;
                used.push(lease.clone());
            }
        }Ok(used)
    }
    pub(crate) fn receipt(&self)->serde_json::Value{serde_json::json!({"graphics":2941,"pending":self.tickets.iter().filter(|(_,l)|l.waiting()).count(),"queued":self.tickets.iter().filter(|(_,l)|l.drawable()).count(),"resource_loaded":self.bank.is_some(),"error":self.error,"paint_advances_clock":false})}
}
