//! Held512 plain equal-current input, retained step, then BBC3-false arrival.
//! Also a pure selected equal-refresh tail preview, never producer authority.
use super::*;
pub(crate) struct Receipt{identity:Arc<Identity>,pub at:u32,pub target:WorldCell}
#[derive(Clone,Debug)]pub(crate) struct Fact{pub at:u32,pub old_target:WorldCell,pub target:WorldCell,pub cell:WorldCell,pub frame:usize,pub heading:u8,pub unchanged_timers:Vec<Timer>,pub flag834eb5:bool,pub interaction_mode:u32,pub active64bbc3:bool}
pub(crate) struct PreparedMouse{identity:Arc<Identity>,revision:u64,state:State,pub fact:Fact}
#[derive(Clone,Debug)]pub(crate) struct Stop{pub callback:Callback,pub source4e2455_reached:Option<bool>,pub active64bbc3:Option<bool>,pub pending834eb3:Option<bool>}
#[derive(Clone,Debug)]pub(crate) struct StopFact{pub at:u32,pub cell:WorldCell,pub target:WorldCell,pub held760098:bool,pub target_refreshed:bool,pub queued_idle:bool,pub required_native_draw_called:bool}
pub(crate) struct PreparedStop{pub movement:Prepared,pub fact:StopFact}
#[derive(Clone,Debug)]pub(crate) struct EqualTail{pub equality_reached:Option<bool>,pub source48b9f0_plain_reached:Option<bool>,pub flag834eb5:Option<bool>,pub interaction_mode:Option<u32>,pub actor139:Option<u8>,pub actor12c134_absent:Option<bool>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) struct EqualTailFact{pub sentinel:u8,pub collision_called:bool,pub heading_written:bool,pub queue_changed:bool,pub native_draw_called:bool}
pub(crate) fn equal_tail_preview(q:&EqualTail)->Result<EqualTailFact>{
 ensure!(q.equality_reached==Some(true)&&q.source48b9f0_plain_reached==Some(true)&&q.flag834eb5==Some(true)&&q.interaction_mode==Some(0)&&q.actor139==Some(255)&&q.actor12c134_absent==Some(true),"Equal-refresh nonplain/optional/producer branch unqualified");
 //4E1DB0 equality returns8 without writing out-varF. EB5true skips
 //reversal guard; mode0 makes both varF possibilities join4E2876.
 //48B7B0(8) returnsfalse before48B640, then4E2592 actor139FF exits.
 Ok(EqualTailFact{sentinel:8,collision_called:false,heading_written:false,queue_changed:false,native_draw_called:false})
}
impl Model<'_>{
 pub fn prepare_held_current_mouse(&self,b:&a::Binding,now:i64,q:&input::Inputs)->Result<PreparedMouse>{
  self.active(b,now)?;q.runtime.validate_key(b,true)?;
  ensure!(self.state.continuous_click&&self.state.started&&self.state.action==46&&(1..=3).contains(&self.state.frame)&&self.state.events.iter().all(|t|t.kind==Kind::Step),"Held equal-current input needs in-flight source step");
  ensure!(q.event==Some(512)&&q.flags==Some(1)&&q.hud_intercept_absent==Some(true)&&q.viewport_bounds_passed==Some(true)&&q.plain64b8f6==Some(false),"Only selected plain held512 source input");let at=q.at.context("Held512 clock unknown")?;ensure!(at>=self.state.last&&self.state.events.first().is_some_and(|t|at<t.at),"Held512 crosses timer");let point=q.point.context("Held512 point unknown")?;ensure!((0..=65535).contains(&point.x)&&(0..=65535).contains(&point.y),"Unsigned packed point bound");
  let mut s=self.state.clone();let old_target=s.target;s.target=mouse_cell(point,s.camera.center)?;ensure!(s.target==s.cell,"Only equal-current held input qualified");s.active_move=false;s.last=at;s.left-=1;
  let fact=Fact{at,old_target,target:s.target,cell:s.cell,frame:s.frame,heading:s.heading,unchanged_timers:s.events.clone(),flag834eb5:true,interaction_mode:0,active64bbc3:false};Ok(PreparedMouse{identity:self.identity.clone(),revision:self.revision,state:s,fact})
 }
 pub fn commit_held_current_mouse(&mut self,b:&a::Binding,now:i64,p:PreparedMouse)->Result<(Fact,Receipt)>{self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&p.revision==self.revision,"Stale/foreign held512 preparation");let r=Receipt{identity:self.identity.clone(),at:p.fact.at,target:p.fact.target};self.state=p.state;self.revision+=1;Ok((p.fact,r))}
 pub fn prepare_inactive_held_stop(&self,b:&a::Binding,now:i64,r:&Receipt,t:Timer,q:&Stop)->Result<PreparedStop>{
  self.active(b,now)?;q.callback.runtime.validate_key(b,true)?;ensure!(Arc::ptr_eq(&self.identity,&r.identity)&&r.at<=self.state.last&&r.target==self.state.target&&self.state.continuous_click&&!self.state.active_move&&self.state.action==46&&self.state.frame==0,"BBC3false stop needs same held-current receipt/completed step");
  ensure!(self.state.events.first()==Some(&t)&&t.kind==Kind::Arrival&&q.callback.dispatch==Some(t.at)&&q.callback.timer_removed==Some(true)&&q.callback.callback_owner_matches==Some(true)&&t.at>=self.state.last&&q.callback.queue_before_pop==Some(0)&&self.state.queue.as_ref().is_some_and(|v|v.nodes.is_empty()),"BBC3false stop owner/removal/queue unknown");ensure!(q.source4e2455_reached==Some(true)&&q.active64bbc3==Some(false)&&q.pending834eb3==Some(true),"BBC3false stop flags unknown");samples(&q.callback.fresh_schedule_samples,t.at,0)?;let mut s=self.state.clone();s.events.remove(0);let cell=s.cell;
  let fact=StopFact{at:t.at,cell,target:s.target,held760098:true,target_refreshed:false,queued_idle:false,required_native_draw_called:false};Ok(PreparedStop{movement:self.prepared(s,t.at,cell,false,vec![],None)?,fact})
 }
}
