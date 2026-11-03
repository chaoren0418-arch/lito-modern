//! Selected stopped46/frame0 release then ordinary re-click, same actor state.
use super::*;
pub(crate) struct ReleaseReceipt{identity:Arc<Identity>,pub at:u32}
#[derive(Clone,Debug)]pub(crate) struct ReleaseFact{pub at:u32,pub fresh:u32,pub cell:WorldCell,pub target:WorldCell,pub action:u16,pub frame:usize,pub held_after:bool}
pub(crate) struct PreparedRelease{identity:Arc<Identity>,revision:u64,state:State,pub fact:ReleaseFact}
impl Model<'_>{
 pub fn prepare_stopped_release(&self,b:&a::Binding,now:i64,q:&release_stop::ReleaseInputs)->Result<PreparedRelease>{
  self.active(b,now)?;q.runtime.validate_key(b,true)?;ensure!(self.state.started&&self.state.continuous_click&&!self.state.active_move&&self.state.action==46&&self.state.frame==0&&self.state.events.is_empty()&&self.state.queue.as_ref().is_some_and(|v|v.nodes.is_empty()),"Stopped release requires existing held46frame0 stopped scene; noidle reset");ensure!(q.at>=self.state.last&&q.fresh>=q.at&&q.source48c57b_reached==Some(true)&&q.source48c604_reached==Some(true),"Stopped release selectedsource/fresh clock unknown");let mut s=self.state.clone();s.continuous_click=false;s.last=q.fresh;let fact=ReleaseFact{at:q.at,fresh:q.fresh,cell:s.cell,target:s.target,action:s.action,frame:s.frame,held_after:false};Ok(PreparedRelease{identity:self.identity.clone(),revision:self.revision,state:s,fact})
 }
 pub fn commit_stopped_release(&mut self,b:&a::Binding,now:i64,p:PreparedRelease)->Result<(ReleaseFact,ReleaseReceipt)>{self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&p.revision==self.revision,"Foreign/stale stopped release");let receipt=ReleaseReceipt{identity:self.identity.clone(),at:p.fact.fresh};self.state=p.state;self.revision+=1;Ok((p.fact,receipt))}
 pub fn prepare_restart(&self,b:&a::Binding,now:i64,r:&ReleaseReceipt,q:&Start)->Result<Prepared>{
  self.active(b,now)?;q.runtime.validate_key(b,true)?;ensure!(Arc::ptr_eq(&self.identity,&r.identity)&&r.at<=self.state.last&&self.state.started&&!self.state.continuous_click&&!self.state.active_move&&self.state.action==46&&self.state.frame==0&&self.state.events.is_empty()&&self.state.queue.as_ref().is_some_and(|v|v.nodes.is_empty()&&v.capacity==4),"Restart needs same existing stopped46frame0 owner and explicitreleased latch/emptyqueue");let at=q.at.context("Restart clock absent")?;ensure!(at>r.at&&at>=self.state.last,"Restart before release/backward");let clocks=samples(&q.fresh_schedule_samples,at,3)?;let mut s=self.state.clone();let old=s.cell;s.target=mouse_cell(q.mouse.context("Restart source mouse absent")?,s.camera.center)?;s.continuous_click=true;let added=self.begin_step(&mut s,at,q.source_collision_blocked,q.source_final_passable,&clocks)?;let heading=s.heading;self.prepared(s,at,old,false,added,Some((old,heading)))
 }
}
