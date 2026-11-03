//! Source idle49 continuation in the same owner/clock/depth scene.
use super::*;use crate::modern_lito::native_appearance_idle_study::cycle;
pub(crate) struct Model<'a>{pub(crate) scene:super::Model<'a>,pub(crate) cycle:cycle::Model}
impl<'a> Model<'a>{
 pub fn new(enabled:bool,mut scene:super::Model<'a>,b:&Binding,now:i64)->Result<Self>{
  scene.active(b,now)?;let pose=scene.pose.context("No first-idle source pose")?;ensure!(pose.idle&&pose.sub==0&&scene.player_hold_until>scene.last,"No completed first-idle residency");let cycle=cycle::Model::from_completed_first(enabled,&mut scene.player,b,now)?;let t=cycle.next(b,now)?;ensure!(t.at==scene.player_hold_until&&t.at>scene.last,"First-idle seed/root clock differs");scene.insert(t.at,Kind::IdleCycle(t.serial));Ok(Self{scene,cycle})
 }
 pub fn dispatch(&mut self,b:&Binding,now:i64,ticket:&Ticket,q:&cycle::Inputs,draw:&DrawInputs)->Result<(Frame,cycle::Step)>{
  self.scene.active(b,now)?;draw_inputs(draw)?;ensure!(Arc::ptr_eq(&self.scene.identity,&ticket.identity)&&self.scene.events.first()==Some(&ticket.event),"Foreign/stale scene idle callback");let t=self.cycle.next(b,now)?;ensure!(ticket.event.kind==Kind::IdleCycle(t.serial)&&ticket.event.at==t.at,"Source idle/root FIFO differs");let p=self.cycle.prepare(b,now,t,q)?;let step=&p.step;let pose=Pose{bank:step.bank,sub:step.sub,idle:true,cell:self.scene.roof.cell(b,now)?,redraw:self.scene.generation+1};
  let canvas=paint(&self.scene.roof,None,&self.scene.layers,self.scene.banks,b,now,self.scene.view,Some(pose),self.scene.fire_index,self.scene.fire_redraw,self.scene.effect,self.scene.fire_cell,&self.scene.identity.retired)?;
  let step=self.cycle.commit(b,now,p)?;self.scene.pose=Some(pose);self.scene.events.remove(0);self.scene.insert(step.next.at,Kind::IdleCycle(step.next.serial));self.scene.player_hold_until=step.next.at;Ok((self.scene.frame(canvas,step.clock,format!("SOURCE_CONDITIONAL_IDLE49_{:?}",t.kind)),step))
 }
 pub fn retire(&mut self){self.cycle.retire();self.scene.retire();}
}
impl Drop for Model<'_>{fn drop(&mut self){self.retire();}}
#[cfg(test)]#[path="native_idle_cycle_scene_study_tests.rs"]mod tests;
