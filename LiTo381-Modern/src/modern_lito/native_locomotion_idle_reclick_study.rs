//! Same started actor: source busy-query idle cancellation then var_E frame reset.
use super::*;
#[derive(Clone,Debug)]pub(crate) struct Inputs{pub start:Start,pub busy_query_idle_branch:Option<bool>,pub idle_gap_branch:Option<bool>,pub owner_timer_cancel_qualified:Option<bool>,pub late_var_e_frame_reset:Option<bool>,pub action49_pointer_valid:Option<bool>}
pub(crate) struct PreparedReclick{pub movement:Prepared,pub cancelled:Timer,pub expected_before:u32}
impl Model<'_>{
 pub fn prepare_idle_reclick(&self,b:&a::Binding,now:i64,q:&Inputs)->Result<PreparedReclick>{
  self.active(b,now)?;q.start.runtime.validate_key(b,true)?;ensure!(q.busy_query_idle_branch==Some(true)&&q.idle_gap_branch==Some(true)&&q.owner_timer_cancel_qualified==Some(true)&&q.late_var_e_frame_reset==Some(true)&&q.action49_pointer_valid==Some(true),"Idle busy cancellation/gap/late reset/pointer source unknown");
  ensure!(self.state.started&&!self.state.continuous_click&&!self.state.active_move&&self.state.action==49&&self.state.frame==1&&self.state.queue.as_ref().is_some_and(|v|v.nodes.is_empty()&&v.capacity==4)&&self.state.events.len()==1,"Reclick requires same started idle49frame1 emptycap4 queue");let cancelled=self.state.events[0];ensure!(cancelled.kind==Kind::IdleAdvance,"Must cancel current actor idle timer only");let at=q.start.at.context("Reclick clock absent")?;ensure!(at>=self.state.last&&at<cancelled.at,"Reclick cannot dispatch or overtake idle callback");let clocks=samples(&q.start.fresh_schedule_samples,at,3)?;ensure!(clocks.iter().all(|v|*v<cancelled.at),"Reclick fresh clocks cross idle callback");let expected_before=self.state.expected.context("Idle expected clock absent")?;
  let mut s=self.state.clone();let old=s.cell;s.events.clear();s.target=mouse_cell(q.start.mouse.context("Reclick mouse absent")?,s.camera.center)?;s.continuous_click=true;
  // 4E6AF0/4E2438 cancel without frame reset. Selected var_E=1 tail resets
  // frame0 only after carry setup, then4E5B50 increments1; no constructor.
  let added=self.begin_step(&mut s,at,q.start.source_collision_blocked,q.start.source_final_passable,&clocks)?;ensure!(s.expected==Some(expected_before),"Cancellation must retain actor expected clock memory");let heading=s.heading;let movement=self.prepared(s,at,old,false,added,Some((old,heading)))?;Ok(PreparedReclick{movement,cancelled,expected_before})
 }
}
