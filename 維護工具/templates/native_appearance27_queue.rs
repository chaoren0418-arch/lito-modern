//! S27 and ordered body operations, before integration into the native animator.
//! Native4A70B0 asks4E6AF0, queues tag27 through4E6860 while busy, otherwise
//! calls4E0490. The caller owns the real body clock/busy guard and life state.
//! This component never advances time, changes HP, resolves LT, or plays cues.
use anyhow::{ensure,Context,Result};
use std::collections::{BTreeMap,VecDeque};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) struct Packet {pub actor:u32,pub graphics:u16,pub family:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum GraphicsNamespace {Shared(u16),LtRelative(u16),SignedNegative(i16)}
impl Packet {
    pub(crate) fn decode(bytes:&[u8])->Result<Option<Self>> {
        if bytes.first()!=Some(&0x27){return Ok(None);}
        ensure!(bytes.len()==8,"S27 requires exactly eight bytes");
        let actor=u32::from_le_bytes(bytes[1..5].try_into()?);ensure!(actor!=0,"zero S27 actor");
        Ok(Some(Self{actor,graphics:u16::from_le_bytes(bytes[5..7].try_into()?),family:bytes[7]}))
    }
    /// The original comparison is a signed WORD against30000. Its LT base is
    /// a runtime value, not the raw header count or a guessed shared offset.
    pub(crate) fn namespace(self)->GraphicsNamespace {
        if self.graphics>=32768{GraphicsNamespace::SignedNegative(self.graphics as i16)}
        else if self.graphics>=30000{GraphicsNamespace::LtRelative(self.graphics-30000)}
        else{GraphicsNamespace::Shared(self.graphics)}
    }
    pub(crate) fn shared_graphics(self)->Result<u16> {
        match self.namespace(){GraphicsNamespace::Shared(id)=>Ok(id),_=>anyhow::bail!("S27 graphics namespace is not bound to reviewed assets")}
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Owner {pub connection:u64,pub generation:u64,pub map:u16,pub source_revision:u64,pub actor:u32,pub incarnation:u64}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) struct Body {pub graphics:u16,pub family:u8}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum Operation {Action(u8),Appearance(Packet)}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Entry {pub sequence:u64,pub operation:Operation}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum Dispatch {
    Action{sequence:u64,action:u8,body:Body},
    Appearance{sequence:u64,previous:Body,current:Body},
}
const QUEUE_LIMIT:usize=16;
const ADMISSION_LIMIT:usize=4096;
/// Numerical metadata only; no pixels, clips, sound, timers, or server leases.
#[derive(Clone,Debug)]
pub(crate) struct History {
    owner:Owner,body:Body,pending:VecDeque<Entry>,seen:BTreeMap<u64,Operation>,last:u64,
    admitted:u64,dispatched:u64,source_retired:bool,
}
pub(crate) struct Queue {history:History}
impl Queue {
    pub(crate) fn new(owner:Owner,body:Body)->Result<Self> {
        ensure!(owner.actor!=0,"zero presentation owner");
        Ok(Self{history:History{owner,body,pending:VecDeque::new(),seen:BTreeMap::new(),last:0,admitted:0,dispatched:0,source_retired:false}})
    }
    pub(crate) fn history(&self)->History{self.history.clone()}
    pub(crate) fn restore(mut history:History,same_source:bool)->Self {
        if !same_source{history.source_retired=true;history.pending.clear();}
        Self{history}
    }
    pub(crate) fn retire_source(&mut self){self.history.source_retired=true;self.history.pending.clear();}
    pub(crate) fn body(&self)->Option<Body>{(!self.history.source_retired).then_some(self.history.body)}
    pub(crate) fn pending(&self)->usize{self.history.pending.len()}
    pub(crate) fn admissions(&self)->u64{self.history.admitted}
    /// Caller supplies the real event order, including S27 and S20/S23 body
    /// operations. Never fabricate an S20 or cast acknowledgment for S27.
    pub(crate) fn admit(&mut self,owner:&Owner,entry:Entry)->Result<bool> {
        let h=&mut self.history;ensure!(&h.owner==owner&&!h.source_retired,"retired or different body owner");
        ensure!(entry.sequence!=0,"zero body event sequence");
        if let Operation::Appearance(packet)=&entry.operation{ensure!(packet.actor==owner.actor,"appearance targets another object");}
        if let Some(prior)=h.seen.get(&entry.sequence){ensure!(prior==&entry.operation,"body sequence reused for a different operation");return Ok(false);}
        ensure!(entry.sequence>h.last,"out-of-order body operation");
        ensure!(h.pending.len()<QUEUE_LIMIT&&h.seen.len()<ADMISSION_LIMIT,"body operation/admission budget");
        let admitted=h.admitted.checked_add(1).context("body admission counter")?;
        h.seen.insert(entry.sequence,entry.operation.clone());h.last=entry.sequence;h.admitted=admitted;h.pending.push_back(entry);Ok(true)
    }
    /// At most ONE original operation per explicit caller boundary. A busy
    /// body (including held death) never changes graphics or consumes a step.
    /// The caller must start an Action using its returned appearance snapshot
    /// and then drive the real busy guard; this is not a clock implementation.
    pub(crate) fn dispatch(&mut self,owner:&Owner,body_busy:bool)->Result<Option<Dispatch>> {
        let h=&mut self.history;ensure!(&h.owner==owner,"different body owner");
        if h.source_retired||body_busy||h.pending.is_empty(){return Ok(None);}
        let count=h.dispatched.checked_add(1).context("body dispatch counter")?;
        let entry=h.pending.pop_front().unwrap();h.dispatched=count;
        Ok(Some(match entry.operation{
            Operation::Action(action)=>Dispatch::Action{sequence:entry.sequence,action,body:h.body},
            Operation::Appearance(packet)=>{let previous=h.body;h.body=Body{graphics:packet.graphics,family:packet.family};Dispatch::Appearance{sequence:entry.sequence,previous,current:h.body}},
        }))
    }
    /// A late worker may add admission facts only. It cannot import queued
    /// work, body state or resources, nor reactivate a source retired by ABA.
    pub(crate) fn merge_admission(&mut self,other:&History)->Result<()> {
        let h=&mut self.history;ensure!(h.owner==other.owner,"different admission owner");
        let mut merged=h.seen.clone();for(sequence,operation)in &other.seen{
            if let Some(prior)=merged.get(sequence){ensure!(prior==operation,"conflicting body admission");}
            else{merged.insert(*sequence,operation.clone());}
        }
        ensure!(merged.len()<=ADMISSION_LIMIT,"body admission union budget");h.seen=merged;h.last=h.last.max(other.last);Ok(())
    }
}

#[cfg(test)]mod tests {
 use super::*;
 fn owner()->Owner{Owner{connection:7,generation:4,map:6,source_revision:10,actor:0xf1234567,incarnation:9}}
 fn body()->Body{Body{graphics:240,family:0}}
 fn action(sequence:u64,value:u8)->Entry{Entry{sequence,operation:Operation::Action(value)}}
 fn appearance(sequence:u64,graphics:u16)->Entry{Entry{sequence,operation:Operation::Appearance(Packet{actor:owner().actor,graphics,family:4})}}
 #[test]fn exact_unsigned_s27_does_not_guess_lt_or_accept_tail(){let mut b=vec![39];b.extend(owner().actor.to_le_bytes());b.extend(8859_u16.to_le_bytes());b.push(83);let p=Packet::decode(&b).unwrap().unwrap();assert_eq!(p.actor,owner().actor);assert_eq!(p.shared_graphics().unwrap(),8859);assert_eq!(p.family,83);for raw in [30000_u16,32767,32768,65535]{let mut bytes=b.clone();bytes[5..7].copy_from_slice(&raw.to_le_bytes());let p=Packet::decode(&bytes).unwrap().unwrap();assert!(p.shared_graphics().is_err());assert_eq!(p.namespace(),if raw<32768{GraphicsNamespace::LtRelative(raw-30000)}else{GraphicsNamespace::SignedNegative(raw as i16)});}assert!(Packet::decode(&b[..7]).is_err());b.push(0);assert!(Packet::decode(&b).is_err());assert!(Packet::decode(&[38]).unwrap().is_none());}
 #[test]fn active_body_keeps_old_appearance_then_action_after27_uses_new_snapshot(){let o=owner();let mut q=Queue::new(o.clone(),body()).unwrap();q.admit(&o,action(1,18)).unwrap();assert_eq!(q.dispatch(&o,false).unwrap(),Some(Dispatch::Action{sequence:1,action:18,body:body()}));q.admit(&o,appearance(2,8859)).unwrap();q.admit(&o,action(3,19)).unwrap();assert!(q.dispatch(&o,true).unwrap().is_none());assert_eq!(q.body(),Some(body()));assert_eq!(q.pending(),2);assert_eq!(q.dispatch(&o,false).unwrap(),Some(Dispatch::Appearance{sequence:2,previous:body(),current:Body{graphics:8859,family:4}}));assert_eq!(q.pending(),1);assert_eq!(q.dispatch(&o,false).unwrap(),Some(Dispatch::Action{sequence:3,action:19,body:Body{graphics:8859,family:4}}));}
 #[test]fn multiple_changes_are_fifo_and_one_boundary_never_drains_the_whole_queue(){let o=owner();let mut q=Queue::new(o.clone(),body()).unwrap();for(s,gfx)in[(1,8859),(2,3632),(3,240)]{q.admit(&o,appearance(s,gfx)).unwrap();}for(s,gfx)in[(1,8859),(2,3632),(3,240)]{assert!(matches!(q.dispatch(&o,false).unwrap(),Some(Dispatch::Appearance{sequence,current:Body{graphics,..},..})if sequence==s&&graphics==gfx));assert_eq!(q.pending(),3-s as usize);}assert!(q.dispatch(&o,false).unwrap().is_none());}
 #[test]fn duplicate_collision_wrong_target_and_full_queue_are_atomic(){let o=owner();let mut q=Queue::new(o.clone(),body()).unwrap();q.admit(&o,action(1,18)).unwrap();assert!(!q.admit(&o,action(1,18)).unwrap());assert!(q.admit(&o,action(1,19)).is_err());let mut e=appearance(2,8859);if let Operation::Appearance(ref mut p)=e.operation{p.actor+=1;}assert!(q.admit(&o,e).is_err());for n in 2..=16{q.admit(&o,action(n,2)).unwrap();}assert!(q.admit(&o,action(17,2)).is_err());assert_eq!(q.pending(),16);assert_eq!(q.admissions(),16);}
 #[test]fn ownership_and_held_body_never_advance_a_new_life_or_old_source(){let o=owner();let mut q=Queue::new(o.clone(),body()).unwrap();q.admit(&o,appearance(1,8859)).unwrap();for n in 0..6{let mut changed=o.clone();match n{0=>changed.connection+=1,1=>changed.generation+=1,2=>changed.map+=1,3=>changed.source_revision+=1,4=>changed.actor+=1,_=>changed.incarnation+=1};assert!(q.dispatch(&changed,false).is_err());assert!(q.admit(&changed,action(2,19)).is_err());}for _ in 0..20{assert!(q.dispatch(&o,true).unwrap().is_none());}assert_eq!(q.pending(),1);assert_eq!(q.body(),Some(body()));}
 #[test]fn source_retirement_is_sticky_across_a_b_a_and_same_source_restore(){let o=owner();let mut q=Queue::new(o.clone(),body()).unwrap();q.admit(&o,appearance(1,8859)).unwrap();let mut q=Queue::restore(q.history(),false);assert!(q.body().is_none());assert_eq!(q.pending(),0);q=Queue::restore(q.history(),true);assert!(q.dispatch(&o,false).unwrap().is_none());assert!(q.admit(&o,action(2,19)).is_err());let mut fresh=Queue::new(o.clone(),body()).unwrap();fresh.admit(&o,appearance(1,8859)).unwrap();fresh.retire_source();assert!(Queue::restore(fresh.history(),true).body().is_none());}
 #[test]fn late_admission_imports_neither_pending_body_nor_worker_graph(){let o=owner();let mut current=Queue::new(o.clone(),body()).unwrap();let mut worker=Queue::new(o.clone(),body()).unwrap();worker.admit(&o,appearance(1,8859)).unwrap();worker.dispatch(&o,false).unwrap();worker.admit(&o,action(2,19)).unwrap();current.merge_admission(&worker.history()).unwrap();assert_eq!(current.body(),Some(body()));assert_eq!(current.pending(),0);assert!(!current.admit(&o,appearance(1,8859)).unwrap());assert!(current.dispatch(&o,false).unwrap().is_none());let mut retired=Queue::restore(current.history(),false);retired.merge_admission(&worker.history()).unwrap();assert!(retired.body().is_none());assert_eq!(retired.pending(),0);}
}
