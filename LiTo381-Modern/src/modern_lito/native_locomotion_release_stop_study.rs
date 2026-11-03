//! Selected514 release changes the held key, preserving in-flight movement.
//! Completed-cell held-false arrival enqueues49 without another movement step.
use super::*;
#[derive(Clone,Debug)]pub(crate) struct ReleaseInputs{pub at:u32,pub fresh:u32,pub source48c57b_reached:Option<bool>,pub source48c604_reached:Option<bool>,pub runtime:Runtime}
pub(crate) struct Receipt{identity:Arc<Identity>,pub at:u32}
#[derive(Clone,Debug)]pub(crate) struct ReleaseFact{pub at:u32,pub fresh:u32,pub cell:WorldCell,pub frame:usize,pub target:WorldCell,pub timers:Vec<Timer>}
pub(crate) struct PreparedRelease{identity:Arc<Identity>,revision:u64,state:State,pub fact:ReleaseFact}
#[derive(Clone,Debug)]pub(crate) struct StopInputs{pub callback:Callback,pub source4e2614_reached:Option<bool>,pub source4e02c0_reached:Option<bool>,pub active64bbc3:Option<bool>,pub pending834eb3:Option<bool>,pub queue_count:Option<usize>,pub queue_capacity:Option<usize>,pub array_pointer_valid:Option<bool>,pub node_allocation_succeeds:Option<bool>,pub actor74:Option<i32>,pub class_type_not11:Option<bool>,pub idle49_pointer_valid:Option<bool>,pub actor12c134_absent:Option<bool>}
#[derive(Clone,Debug)]pub(crate) struct StopFact{pub at:u32,pub fresh:u32,pub cell:WorldCell,pub idle_at:u32,pub queued:queue::Node,pub target_refresh_called:bool,pub required_native_draw_called:bool}
pub(crate) struct PreparedStop{pub movement:Prepared,pub fact:StopFact}
impl Model<'_>{
 pub fn prepare_release(&self,b:&a::Binding,now:i64,q:&ReleaseInputs)->Result<PreparedRelease>{
  self.active(b,now)?;q.runtime.validate_key(b,true)?;
  ensure!(self.state.continuous_click&&self.state.started&&self.state.action==46&&(1..=3).contains(&self.state.frame)&&self.state.active_move&&self.state.events.first().is_some_and(|t|q.fresh<t.at),"Release requires held in-flight source step before timer");
  ensure!(q.at>=self.state.last&&q.fresh>=q.at&&q.source48c57b_reached==Some(true)&&q.source48c604_reached==Some(true),"Release source path/fresh clock unknown");
  let mut s=self.state.clone();s.continuous_click=false;s.last=q.fresh;
  let fact=ReleaseFact{at:q.at,fresh:q.fresh,cell:s.cell,frame:s.frame,target:s.target,timers:s.events.clone()};
  Ok(PreparedRelease{identity:self.identity.clone(),revision:self.revision,state:s,fact})
 }
 pub fn commit_release(&mut self,b:&a::Binding,now:i64,p:PreparedRelease)->Result<(ReleaseFact,Receipt)>{
  self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&p.revision==self.revision,"Stale/foreign movement release");
  let at=p.fact.fresh;self.state=p.state;self.revision+=1;Ok((p.fact,Receipt{identity:self.identity.clone(),at}))
 }
 pub fn prepare_released_stop(&self,b:&a::Binding,now:i64,r:&Receipt,t:Timer,q:&StopInputs)->Result<PreparedStop>{
  self.active(b,now)?;q.callback.runtime.validate_key(b,false)?;
  ensure!(Arc::ptr_eq(&self.identity,&r.identity)&&r.at<=self.state.last&&!self.state.continuous_click&&self.state.started&&self.state.action==46&&self.state.frame==0&&self.state.active_move&&self.state.target==self.state.cell,"Released stop requires same source receipt and completed target equality");
  ensure!(self.state.events.first()==Some(&t)&&t.kind==Kind::Arrival&&q.callback.dispatch==Some(t.at)&&q.callback.timer_removed==Some(true)&&q.callback.callback_owner_matches==Some(true)&&t.at>=self.state.last&&q.callback.queue_before_pop==Some(0),"Released stop timer/removal/owner mismatch");
  ensure!(q.source4e2614_reached==Some(true)&&q.source4e02c0_reached==Some(true)&&q.active64bbc3==Some(true)&&q.pending834eb3==Some(true)&&q.actor74==Some(0)&&q.class_type_not11==Some(true)&&q.idle49_pointer_valid==Some(true)&&q.actor12c134_absent==Some(true),"Held-false stop source flags unknown");
  let clock=samples(&q.callback.fresh_schedule_samples,t.at,1)?[0];let mut s=self.state.clone();s.events.remove(0);s.active_move=false;
  let queue=s.queue.as_mut().context("Stop private queue absent")?;
  ensure!(queue.nodes.is_empty()&&queue.capacity==4&&q.queue_count==Some(0)&&q.queue_capacity==Some(4)&&q.array_pointer_valid==Some(true)&&q.node_allocation_succeeds==Some(true),"Idle enqueue count/capacity/pointer/allocation unqualified");
  // base46+3=49, current46 differs ->4E0399 fixed248h delay;
  // class734 is not type11, actor74 is zero, optional139 isFF.
  queue.serial+=1;let node=queue::Node{serial:queue.serial,arg:49};queue.nodes.push(node.clone());let idle_at=clock.checked_add(584).context("Stop idle clock wraps")?;
  let mut added=vec![];schedule(&mut s,idle_at,Kind::IdlePop,&mut added);let cell=s.cell;
  let fact=StopFact{at:t.at,fresh:clock,cell,idle_at,queued:node,target_refresh_called:false,required_native_draw_called:false};
  Ok(PreparedStop{movement:self.prepared(s,t.at,cell,false,added,None)?,fact})
 }
}
