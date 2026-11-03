//! Actual source packet32/DC producer, qualified only at the movement frame0
//! gap. Bounded offline bytes do not represent received/live native authority.
use super::*;
#[derive(Clone,Debug)]pub(crate) struct Inputs{
 pub at:Option<u32>,pub packet:Option<Vec<u8>>,pub dispatcher32_branch:Option<bool>,pub registry_pointer_matches:Option<bool>,
 pub actor_queue_zero:Option<bool>,pub byte834eb3:Option<bool>,pub byte834eb4:Option<bool>,pub actor12c134_absent:Option<bool>,pub class_type_not11:Option<bool>,
 pub timer_dispatch_not_started:Option<bool>,pub same_clock_packet_first:Option<bool>,pub cancel_callback:Option<u32>,pub cancel_owner_zero:Option<bool>,
 pub fresh_schedule_sample:Option<u32>,pub runtime:Runtime,
}
pub(crate) struct PreparedAction{prepared:Prepared,pub step:Step,pub cancelled:Vec<Timer>}
pub(crate) struct Fact{pub step:Step,pub cancelled:Vec<Timer>}
impl Model<'_>{
 pub fn prepare_action(&self,b:&a::Binding,now:i64,q:&Inputs)->Result<PreparedAction>{
  self.active(b,now)?;q.runtime.validate(b)?;
  let packet=q.packet.as_ref().context("Action packet32/DC bytes absent")?;
  ensure!(packet.len()==6&&packet[0]==32&&i64::from(u32::from_le_bytes(packet[1..5].try_into()?))==b.object&&packet[5]==1,"Only exact bounded packet32 same-object action1 qualified");
  ensure!(q.dispatcher32_branch==Some(true)&&q.registry_pointer_matches==Some(true)&&q.actor_queue_zero==Some(true)&&q.byte834eb3==Some(true)&&q.byte834eb4==Some(true)&&q.actor12c134_absent==Some(true)&&q.class_type_not11==Some(true),"Original producer/registry/queue/flags/class/effect branch unknown");
  ensure!(self.state.started&&self.state.action==46&&self.state.frame==0&&self.state.events.len()==1&&self.state.events[0].kind==Kind::Arrival&&self.state.queue.as_ref().is_none_or(|q|q.nodes.is_empty()),"Midstep4E6AF0 busy must queue action; immediate gap profile unavailable");
  let at=q.at.context("Action producer clock unknown")?;let t=self.state.events[0];
  ensure!(at>=self.state.last&&at<=t.at&&q.timer_dispatch_not_started==Some(true)&&q.cancel_callback==Some(0x4e2150)&&q.cancel_owner_zero==Some(true),"Carry already dispatched/foreign cancellation signature/order unknown");
  if at==t.at{ensure!(q.same_clock_packet_first==Some(true),"Same clock does not prove packet-before-carry order");}
  let fresh=q.fresh_schedule_sample.context("Action fresh4D65F0 clock unknown")?;ensure!(fresh>=at,"Action scheduling clock precedes input");
  let plan=a::plan(true,self.d,b,b,now,a::Kind::Attack,self.state.heading,16)?;
  ensure!(plan.action==47&&plan.fps==24&&plan.frames.len()==7&&plan.frames[0].0.image==72&&plan.frames[1].0.image==88&&plan.frames[1].0.subframe==0&&plan.frames[1].0.ticks==4,"Exact attack47/source end0 catalog required");
  let mut s=self.state.clone();let old=s.cell;let cancelled=s.events.clone();s.events.clear();s.action=47;s.frame=1;
  // 4E45F0 cancels the owner0 movement callback, not64BBC3/destination.
  let next=fresh.checked_add(168).context("Attack first timer wraps")?;s.expected=Some(next);let mut added=vec![];schedule(&mut s,next,Kind::AttackAdvance,&mut added);
  let p=self.prepared(s,at,old,false,added,None)?;let step=p.step.clone();Ok(PreparedAction{prepared:p,step,cancelled})
 }
 pub fn commit_action(&mut self,b:&a::Binding,now:i64,p:PreparedAction)->Result<Fact>{let step=self.commit(b,now,p.prepared)?;Ok(Fact{step,cancelled:p.cancelled})}
}
