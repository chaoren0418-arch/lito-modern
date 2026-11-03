//! Candidate immutable worker intents and single-UI environment application.
use super::{native_ambient_override::{Context,Entry,Finish,Identity,Ledger,Lease,Retirement},native_environment::State};
use anyhow::{ensure,Result};
use std::{collections::{BTreeMap,BTreeSet},sync::{Arc,OnceLock,atomic::{AtomicBool,Ordering}}};
const MAX_FINISHED:usize=4096;

#[derive(Clone,Debug)]
pub struct Scene {connection:u64,generation:u64,map:u16,publication:Arc<()>}
impl Scene {
    pub fn new(connection:u64,generation:u64,map:u16,publication:Arc<()>)->Result<Self> {
        ensure!(connection!=0 && generation!=0,"ambient scene absent");
        Ok(Self{connection,generation,map,publication})
    }
    fn same(&self,other:&Self)->bool {
        self.connection==other.connection && self.generation==other.generation && self.map==other.map && Arc::ptr_eq(&self.publication,&other.publication)
    }
    fn context(&self)->Result<Context> {Context::new(self.connection,self.generation,self.map,self.publication.clone())}
    fn environment_matches(&self,state:&State)->bool {state.epoch==Some(self.connection) && state.map==Some(self.map)}
}
#[derive(Clone,Copy,Debug)]
pub enum Transition {EnterFrameTwo,Finish}
#[derive(Debug)]
struct Payload {identity:Identity,sequence:u64,transition:Transition,scene:OnceLock<Scene>,consumed:AtomicBool}
/// Cloned frames share binding and the claim latch. No worker gets mutable State.
#[derive(Clone,Debug)]
pub struct Prepared(Arc<Payload>);
impl Prepared {
    pub fn new(actor:u32,incarnation:u64,sequence:u64,transition:Transition)->Result<Self> {
        Ok(Self(Arc::new(Payload{identity:Identity::call_lightning(actor,incarnation,sequence)?,sequence,transition,scene:OnceLock::new(),consumed:AtomicBool::new(false)})))
    }
    /// Bind only after the frame's source, connection and generation are admitted.
    /// A retired frame cannot rebind an already bound intent to another scene.
    pub fn bind(&self,scene:&Scene)->bool {self.0.scene.get_or_init(||scene.clone()).same(scene)}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Outcome {Entered,Released,AlreadyConsumed,AlreadyFinished,Stale,Unbound,AtCapacity}
#[derive(Debug,Default)]
pub struct Bridge {
    scene:Option<Scene>,context:Option<Context>,ledger:Ledger,
    leases:BTreeMap<Identity,Lease>,finished:BTreeSet<Identity>,finished_sequences:BTreeMap<Identity,u64>,floor:u64,
}
impl Bridge {
    pub fn bind(&mut self,scene:Scene,state:&mut State)->Result<Retirement> {
        if self.scene.as_ref().is_some_and(|old|old.same(&scene)) {
            ensure!(scene.environment_matches(state),"ambient scene/environment mismatch");
            return self.ledger.bind(self.context.as_ref().expect("bound context").clone(),state);
        }
        let context=scene.context()?;let retired=self.ledger.bind(context.clone(),state)?;
        self.leases.clear();self.finished.clear();self.finished_sequences.clear();self.floor=1;
        self.scene=Some(scene);self.context=Some(context);Ok(retired)
    }
    pub fn retire(&mut self,state:&mut State)->Result<Retirement> {
        let retired=self.ledger.retire(state)?;
        self.scene=None;self.context=None;self.leases.clear();self.finished.clear();self.finished_sequences.clear();self.floor=1;Ok(retired)
    }
    pub fn active_count(&self)->usize {self.leases.len()}
    pub fn prune_before(&mut self,scene:&Scene,sequence:u64)->Result<()> {
        ensure!(self.scene.as_ref().is_some_and(|current|current.same(scene)),"stale ambient prune");
        ensure!(sequence>=self.floor && sequence!=0,"ambient prune floor regressed");
        self.ledger.prune_before(self.context.as_ref().expect("bound context"),sequence)?;
        self.floor=sequence;self.finished_sequences.retain(|_,n|*n>=sequence);
        self.finished.retain(|id|self.finished_sequences.contains_key(id));Ok(())
    }
    pub fn apply(&mut self,scene:&Scene,state:&mut State,prepared:&Prepared)->Result<Outcome> {
        let Some(bound)=prepared.0.scene.get() else{return Ok(Outcome::Unbound)};
        if !bound.same(scene) || !self.scene.as_ref().is_some_and(|current|current.same(scene)) || !scene.environment_matches(state) {
            return Ok(Outcome::Stale);
        }
        let old=prepared.0.sequence<self.floor;
        // An active lease may finish after its source history was pruned.
        if old && !matches!(prepared.0.transition,Transition::Finish) {return Ok(Outcome::Stale)}
        if old && !self.leases.contains_key(&prepared.0.identity) {return Ok(Outcome::Stale)}
        if prepared.0.consumed.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err() {return Ok(Outcome::AlreadyConsumed)}
        let result=self.apply_claimed(state,&prepared.0);
        if result.is_err() || matches!(result,Ok(Outcome::AtCapacity | Outcome::Stale)) {prepared.0.consumed.store(false,Ordering::Release);}
        result
    }
    fn apply_claimed(&mut self,state:&mut State,event:&Payload)->Result<Outcome> {
        let context=self.context.as_ref().expect("validated bound context");
        match event.transition {
            Transition::EnterFrameTwo=> {
                if self.finished.contains(&event.identity) {return Ok(Outcome::AlreadyFinished)}
                match self.ledger.enter_frame(context,state,event.identity,2)? {
                    Entry::Admitted(lease)=>{self.leases.insert(event.identity,lease);Ok(Outcome::Entered)}
                    Entry::Duplicate=>Ok(Outcome::AlreadyConsumed),
                    Entry::AtCapacity=>Ok(Outcome::AtCapacity),
                    Entry::Stale=>Ok(Outcome::Stale),
                    Entry::IgnoredFrame=>unreachable!("fixed class2 source frame"),
                }
            }
            Transition::Finish=> {
                if self.finished.contains(&event.identity) {return Ok(Outcome::AlreadyFinished)}
                // History pressure must never prevent an active override restoring.
                if self.finished.len()>=MAX_FINISHED && !self.leases.contains_key(&event.identity) {return Ok(Outcome::AtCapacity)}
                if let Some(lease)=self.leases.get(&event.identity) {
                    match self.ledger.finish(context,state,lease)? {
                        Finish::Stale=>return Ok(Outcome::Stale),
                        Finish::Released | Finish::AlreadyConsumed=>{},
                    }
                    self.leases.remove(&event.identity);
                }
                // A finish observed before a delayed entry prevents resurrection.
                if self.finished.len()<MAX_FINISHED {
                    self.finished.insert(event.identity);self.finished_sequences.insert(event.identity,event.sequence);
                }
                Ok(Outcome::Released)
            }
        }
    }
}
