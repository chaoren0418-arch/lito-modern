//! Source4E2150 completed-cell/held-button refresh. Only unchanged global
//! mouse and a new unequal destination; release/equal-target branches refuse.
use super::*;
#[derive(Clone,Debug)]pub(crate) struct Inputs{
 pub callback:Callback,pub held760098:Option<bool>,pub active64bbc3:Option<bool>,pub pending834eb3:Option<bool>,pub global_mouse:Option<PixelPoint>,pub no_mouse_messages_since_click:Option<bool>,pub refresh48b9f0_reached:Option<bool>,pub plain64b8f6:Option<bool>,pub spell_absent:Option<bool>,pub hover_absent:Option<bool>,pub popup_absent:Option<bool>,
}
#[derive(Clone,Debug)]pub(crate) struct Fact{pub at:u32,pub cell:WorldCell,pub old_target:WorldCell,pub refreshed_target:WorldCell,pub source_mouse:PixelPoint,pub camera_center:WorldCell,pub fresh_clocks:Vec<u32>,pub new_timers:Vec<Timer>}
pub(crate) struct PreparedHeld{pub movement:Prepared,pub fact:Fact}
impl Model<'_>{
 pub fn prepare_held_arrival(&self,b:&a::Binding,now:i64,t:Timer,q:&Inputs)->Result<PreparedHeld>{
  self.active(b,now)?;q.callback.runtime.validate_key(b,true)?;
  ensure!(self.state.continuous_click&&self.state.started&&self.state.action==46&&self.state.frame==0&&self.state.active_move&&self.state.target==self.state.cell&&self.state.events.first()==Some(&t)&&t.kind==Kind::Arrival,"Held arrival requires completed source click cell and exact owner0 callback");
  ensure!(q.callback.dispatch==Some(t.at)&&q.callback.timer_removed==Some(true)&&q.callback.callback_owner_matches==Some(true)&&t.at>=self.state.last&&q.callback.queue_before_pop==Some(0)&&self.state.queue.as_ref().is_some_and(|v|v.nodes.is_empty()),"Held arrival clock/removal/owner/empty queue unknown");
  ensure!(q.held760098==Some(true)&&q.active64bbc3==Some(true)&&q.pending834eb3==Some(true)&&q.no_mouse_messages_since_click==Some(true)&&q.refresh48b9f0_reached==Some(true)&&q.plain64b8f6==Some(false)&&q.spell_absent==Some(true)&&q.hover_absent==Some(true)&&q.popup_absent==Some(true),"Held refresh/release/retarget/optional branch unqualified");
  let mouse=q.global_mouse.context("Held global mouse snapshot unknown")?;ensure!((0..=65535).contains(&mouse.x)&&(0..=65535).contains(&mouse.y),"Held packed mouse outside source range");let clocks=samples(&q.callback.fresh_schedule_samples,t.at,3)?;
  let mut s=self.state.clone();s.events.remove(0);let cell=s.cell;let old_target=s.target;s.active_move=false;s.target=mouse_cell(mouse,s.camera.center)?;
  ensure!(s.target!=cell,"Held refresh same-current-target branch unimplemented");
  let added=self.begin_step(&mut s,t.at,q.callback.source_collision_blocked,q.callback.source_final_passable,&clocks)?;
  let fact=Fact{at:t.at,cell,old_target,refreshed_target:s.target,source_mouse:mouse,camera_center:s.camera.center,fresh_clocks:clocks,new_timers:added.clone()};let heading=s.heading;
  Ok(PreparedHeld{movement:self.prepared(s,t.at,cell,false,added,Some((cell,heading)))?,fact})
 }
}
