//! Same private scene raster and root FIFO for source movement, roofs and fire.
//! Construction/clock/global/collision inputs remain offline conditional.
use super::*;
use crate::modern_lito::native_locomotion_study as movement;
pub(crate) struct Model<'a>{pub(crate) scene:super::Model<'a>,pub(crate) movement:movement::Model<'a>}
fn projection(c:movement::Camera)->Result<ViewProjection>{Ok(ViewProjection{anchor_cell:c.center,anchor_pixel:crate::modern_lito::world_map::PixelPoint{x:c.offset.x,y:c.offset.y.checked_add(12).context("Source camera projection overflow")?}})}
impl<'a> Model<'a>{
 pub fn new(mut scene:super::Model<'a>,movement:movement::Model<'a>,b:&Binding,now:i64)->Result<Self>{
  scene.active(b,now)?;ensure!(scene.pose.is_none()&&scene.generation==0&&scene.roof.cell(b,now)?==movement.cell(b,now)?,"Locomotion requires untouched matching private owner/initial cell");
  scene.player.retire();scene.events.retain(|e|!matches!(e.kind,Kind::PlayerStart|Kind::PlayerAdvance|Kind::PlayerIdle));scene.player_hold_until=u32::MAX;
  Ok(Self{scene,movement})
 }
 pub fn start(&mut self,b:&Binding,now:i64,q:&movement::Start,draw:&DrawInputs)->Result<(Frame,movement::Step)>{
  self.scene.active(b,now)?;draw_inputs(draw)?;let p=self.movement.prepare_start(b,now,q)?;self.start_prepared(b,now,p)
 }
 pub fn click_start(&mut self,b:&Binding,now:i64,q:&movement::Start,draw:&DrawInputs)->Result<(Frame,movement::Step)>{
  self.scene.active(b,now)?;draw_inputs(draw)?;let p=self.movement.prepare_click_start(b,now,q)?;self.start_prepared(b,now,p)
 }
 pub fn restart(&mut self,b:&Binding,now:i64,r:&movement::restart::ReleaseReceipt,q:&movement::Start,draw:&DrawInputs)->Result<(Frame,movement::Step)>{self.scene.active(b,now)?;draw_inputs(draw)?;let p=self.movement.prepare_restart(b,now,r,q)?;self.start_prepared(b,now,p)}
 pub fn idle_reclick(&mut self,b:&Binding,now:i64,q:&movement::idle_reclick::Inputs,draw:&DrawInputs)->Result<(Frame,movement::Step)>{self.scene.active(b,now)?;draw_inputs(draw)?;let p=self.movement.prepare_idle_reclick(b,now,q)?;ensure!(self.scene.events.iter().filter(|e|e.kind==Kind::Locomotion(p.cancelled.serial)&&e.at==p.cancelled.at).count()==1,"Idle cancellation scene timer mismatch");let cancelled=p.cancelled;let result=self.start_prepared(b,now,p.movement)?;self.scene.events.retain(|e|e.kind!=Kind::Locomotion(cancelled.serial));Ok(result)}
 fn start_prepared(&mut self,b:&Binding,now:i64,p:movement::Prepared)->Result<(Frame,movement::Step)>{
  let s=&p.step;ensure!(s.at>=self.scene.last&&self.scene.events.first().is_none_or(|e|s.at<e.at)&&self.scene.fire.assets.geometry().contains(s.cell),"Move startup crosses source timer/map");
  let pose=Pose{bank:s.bank,sub:s.sub,idle:false,cell:s.cell,redraw:self.scene.generation+1};let view=projection(s.camera)?;
  let canvas=paint(&self.scene.roof,None,&self.scene.layers,self.scene.banks,b,now,view,Some(pose),self.scene.fire_index,self.scene.fire_redraw,self.scene.effect,self.scene.fire_cell,&self.scene.identity.retired)?;
  let step=self.movement.commit(b,now,p)?;self.scene.pose=Some(pose);self.scene.view=view;for t in &step.added{self.scene.insert(t.at,Kind::Locomotion(t.serial));}self.scene.player_hold_until=self.movement.next(b,now)?.at;
  Ok((self.scene.frame(canvas,step.at,"SOURCE_CONDITIONAL_4E2150_START".into()),step))
 }
 pub fn input(&mut self,b:&Binding,now:i64,q:&movement::input::Inputs)->Result<movement::input::Fact>{
  self.scene.active(b,now)?;let p=self.movement.prepare_input(b,now,q)?;
  ensure!(p.fact.at>=self.scene.last&&self.scene.events.first().is_some_and(|e|p.fact.at<e.at)&&self.scene.fire.assets.geometry().contains(p.fact.target),"Input crosses root FIFO or pinned finite map");
  // The selected input producer neither draws nor cancels/replaces timers.
  let fact=self.movement.commit_input(b,now,p)?;self.scene.last=fact.at;Ok(fact)
 }
 pub fn action(&mut self,b:&Binding,now:i64,q:&movement::action::Inputs,draw:&DrawInputs)->Result<(Frame,movement::action::Fact)>{
  self.scene.active(b,now)?;draw_inputs(draw)?;let p=self.movement.prepare_action(b,now,q)?;let s=&p.step;
  let first=self.scene.events.first().context("Action scene has no pending source timer")?;
  ensure!(s.at>=self.scene.last&&s.at<=first.at&&(s.at<first.at||p.cancelled.iter().any(|t|first.kind==Kind::Locomotion(t.serial)&&first.at==t.at)),"Action overtakes root callback or unspecified equal-clock order");
  ensure!(self.scene.roof.cell(b,now)?==s.cell,"Action must keep completed source cell/roof");
  let pose=Pose{bank:s.bank,sub:s.sub,idle:false,cell:s.cell,redraw:self.scene.generation+1};let view=projection(s.camera)?;
  let canvas=paint(&self.scene.roof,None,&self.scene.layers,self.scene.banks,b,now,view,Some(pose),self.scene.fire_index,self.scene.fire_redraw,self.scene.effect,self.scene.fire_cell,&self.scene.identity.retired)?;
  let fact=self.movement.commit_action(b,now,p)?;
  self.scene.events.retain(|e|!fact.cancelled.iter().any(|t|e.kind==Kind::Locomotion(t.serial)));
  for t in &fact.step.added{self.scene.insert(t.at,Kind::Locomotion(t.serial));}self.scene.pose=Some(pose);self.scene.view=view;self.scene.player_hold_until=self.movement.next(b,now)?.at;
  Ok((self.scene.frame(canvas,fact.step.at,"SOURCE_CONDITIONAL_PACKET32_4D6870_CANCEL_CARRY_ATTACK47".into()),fact))
 }
 pub fn queue(&mut self,b:&Binding,now:i64,q:&movement::queue::Inputs)->Result<movement::queue::Fact>{
  self.scene.active(b,now)?;let p=self.movement.prepare_queue(b,now,q)?;ensure!(p.fact.at>=self.scene.last&&self.scene.events.first().is_some_and(|e|p.fact.at<e.at),"Queue input crosses root FIFO/unknown equal-clock order");
  let fact=self.movement.commit_queue(b,now,p)?;self.scene.last=fact.at;Ok(fact)
 }
 pub fn remove_queue_owner(&mut self,b:&Binding,now:i64,q:&movement::queue::Remove)->Result<movement::queue::Retired>{self.scene.active(b,now)?;let retired=self.movement.remove_queue_owner(b,now,q)?;self.scene.retire();Ok(retired)}
 pub fn dispatch(&mut self,b:&Binding,now:i64,ticket:&Ticket,q:&movement::Callback,roof_input:Option<&roof::MoveInputs>,draw:&DrawInputs)->Result<(Frame,movement::Step)>{
  self.scene.active(b,now)?;draw_inputs(draw)?;ensure!(Arc::ptr_eq(&self.scene.identity,&ticket.identity)&&self.scene.events.first()==Some(&ticket.event),"Foreign/stale root locomotion callback");
  let t=self.movement.next(b,now)?;ensure!(ticket.event.kind==Kind::Locomotion(t.serial)&&ticket.event.at==t.at,"Root/owned movement FIFO differs");let p=self.movement.prepare_callback(b,now,t,q)?;self.dispatch_prepared(b,now,t,p,roof_input)
 }
 pub fn held_arrival(&mut self,b:&Binding,now:i64,ticket:&Ticket,q:&movement::held::Inputs,draw:&DrawInputs)->Result<(Frame,movement::Step,movement::held::Fact)>{
  self.scene.active(b,now)?;draw_inputs(draw)?;ensure!(Arc::ptr_eq(&self.scene.identity,&ticket.identity)&&self.scene.events.first()==Some(&ticket.event),"Foreign/stale held arrival root");let t=self.movement.next(b,now)?;ensure!(ticket.event.kind==Kind::Locomotion(t.serial)&&ticket.event.at==t.at,"Held root/owned movement FIFO differs");let p=self.movement.prepare_held_arrival(b,now,t,q)?;let(f,s)=self.dispatch_prepared(b,now,t,p.movement,None)?;Ok((f,s,p.fact))
 }
 pub fn released_stop(&mut self,b:&Binding,now:i64,r:&movement::release_stop::Receipt,ticket:&Ticket,q:&movement::release_stop::StopInputs,draw:&DrawInputs)->Result<(Frame,movement::Step,movement::release_stop::StopFact)>{
  self.scene.active(b,now)?;draw_inputs(draw)?;ensure!(Arc::ptr_eq(&self.scene.identity,&ticket.identity)&&self.scene.events.first()==Some(&ticket.event),"Released stop root stale/foreign");let t=self.movement.next(b,now)?;ensure!(ticket.event.kind==Kind::Locomotion(t.serial)&&ticket.event.at==t.at,"Released stop root/movement mismatch");let p=self.movement.prepare_released_stop(b,now,r,t,q)?;let(f,s)=self.dispatch_prepared(b,now,t,p.movement,None)?;Ok((f,s,p.fact))
 }
 pub fn held_current_mouse(&mut self,b:&Binding,now:i64,q:&movement::input::Inputs)->Result<(movement::current_target::Fact,movement::current_target::Receipt)>{self.scene.active(b,now)?;let p=self.movement.prepare_held_current_mouse(b,now,q)?;ensure!(p.fact.at>=self.scene.last&&self.scene.events.first().is_some_and(|e|p.fact.at<e.at),"Held512 scene FIFO crossing");let result=self.movement.commit_held_current_mouse(b,now,p)?;self.scene.last=result.0.at;Ok(result)}
 pub fn inactive_held_stop(&mut self,b:&Binding,now:i64,r:&movement::current_target::Receipt,root:&Ticket,q:&movement::current_target::Stop,draw:&DrawInputs)->Result<(Frame,movement::Step,movement::current_target::StopFact)>{self.scene.active(b,now)?;draw_inputs(draw)?;ensure!(Arc::ptr_eq(&self.scene.identity,&root.identity)&&self.scene.events.first()==Some(&root.event),"Inactive held scene root mismatch");let t=self.movement.next(b,now)?;ensure!(root.event.kind==Kind::Locomotion(t.serial)&&root.event.at==t.at,"Inactive held source/root timer mismatch");let p=self.movement.prepare_inactive_held_stop(b,now,r,t,q)?;let(f,s)=self.dispatch_prepared(b,now,t,p.movement,None)?;Ok((f,s,p.fact))}
 pub fn queued_idle(&mut self,b:&Binding,now:i64,root:&Ticket,q:&movement::queued_idle::Inputs,draw:&DrawInputs)->Result<(Frame,movement::Step)>{self.scene.active(b,now)?;draw_inputs(draw)?;ensure!(Arc::ptr_eq(&self.scene.identity,&root.identity)&&self.scene.events.first()==Some(&root.event),"Idle scene root stale/foreign");let t=self.movement.next(b,now)?;ensure!(root.event.kind==Kind::Locomotion(t.serial)&&root.event.at==t.at,"Idle movement/root mismatch");let p=self.movement.prepare_queued_idle(b,now,t,q)?;self.dispatch_prepared(b,now,t,p,None)}
 fn dispatch_prepared(&mut self,b:&Binding,now:i64,t:movement::Timer,p:movement::Prepared,roof_input:Option<&roof::MoveInputs>)->Result<(Frame,movement::Step)>{
  let s=&p.step;
  ensure!(self.scene.fire.assets.geometry().contains(s.cell),"Move leaves pinned Native geometry");let roof=if s.cell_committed{let input=roof_input.context("End0 requires original48AFC0 roof input")?;ensure!(input.actor.old_cell==Some(s.old_cell)&&input.actor.new_cell==Some(s.cell)&&input.actor.scheduler_sample_ms==Some(s.at),"Roof write differs from source committed cell");Some(self.scene.roof.prepare_move(b,now,input)?)}else{ensure!(roof_input.is_none(),"Intermediate step must not write roof/cell");None};
  let redraw=if t.kind==movement::Kind::Arrival&&s.wire_move.is_none(){self.scene.pose.context("Arrival needs retained source pose")?.redraw}else{self.scene.generation+1};
  let pose=Pose{bank:s.bank,sub:s.sub,idle:s.action==49,cell:s.cell,redraw};let view=projection(s.camera)?;
  let canvas=paint(&self.scene.roof,roof.as_ref(),&self.scene.layers,self.scene.banks,b,now,view,Some(pose),self.scene.fire_index,self.scene.fire_redraw,self.scene.effect,self.scene.fire_cell,&self.scene.identity.retired)?;
  // All inputs, pointer-qualified resources and CPU output preflight above.
  if let Some(p)=roof{self.scene.roof.commit(b,now,p)?;}let step=self.movement.commit(b,now,p)?;self.scene.events.remove(0);self.scene.pose=Some(pose);self.scene.view=view;
  // End0 calls48AFC0 before scheduling the arrival carry in4E2150.
  self.scene.sync_roof(b,now)?;for t in &step.added{self.scene.insert(t.at,Kind::Locomotion(t.serial));}self.scene.player_hold_until=self.movement.next_optional(b,now)?.map_or(u32::MAX,|t|t.at);
  // No native idle timer is inferred after current-cell stop. The private
  // finite CPU scene retains final pose until its explicitly bounded retirement.
  Ok((self.scene.frame(canvas,step.at,format!("SOURCE_CONDITIONAL_{:?}",t.kind)),step))
 }
 pub fn retire(&mut self){self.movement.retire();self.scene.retire();}
}
impl Drop for Model<'_>{fn drop(&mut self){self.retire();}}
#[cfg(test)]#[path="native_locomotion_scene_study_tests.rs"]mod tests;
#[cfg(test)]#[path="native_locomotion_sequence_study_tests.rs"]mod sequence_tests;
#[cfg(test)]#[path="native_locomotion_input_study_tests.rs"]mod input_tests;
#[cfg(test)]#[path="native_locomotion_action_study_tests.rs"]mod action_tests;
#[cfg(test)]#[path="native_locomotion_queue_study_tests.rs"]mod queue_tests;
#[path="native_reload_scene_study.rs"]pub(crate) mod reload;
