//! Finite48C080 case512 -> plain48B9F0. Branch facts are conditional;
//! this never creates observed Windows input or native player authority.
use super::*;
#[derive(Clone,Debug)]pub(crate) struct Inputs{
 pub at:Option<u32>,pub event:Option<u32>,pub flags:Option<u32>,pub point:Option<PixelPoint>,
 pub hud_intercept_absent:Option<bool>,pub viewport_bounds_passed:Option<bool>,
 pub plain64b8f6:Option<bool>,pub runtime:Runtime,
}
pub(crate) struct PreparedInput{identity:Arc<Identity>,revision:u64,state:State,pub fact:Fact}
#[derive(Clone,Debug)]pub(crate) struct Fact{
 pub at:u32,pub old_target:WorldCell,pub target:WorldCell,pub cell:WorldCell,pub active_move:bool,
 pub heading:u8,pub frame:usize,pub camera:Camera,pub unchanged_timers:Vec<Timer>,
}
impl Model<'_>{
 pub fn prepare_input(&self,b:&a::Binding,now:i64,q:&Inputs)->Result<PreparedInput>{
  self.active(b,now)?;q.runtime.validate(b)?;
  ensure!(q.event==Some(512)&&q.flags.is_some_and(|f|f&1!=0)&&q.hud_intercept_absent==Some(true)&&q.viewport_bounds_passed==Some(true)&&q.plain64b8f6==Some(false),"Selected48C080/48B9F0 input branch unknown");
  ensure!(self.state.started&&self.state.action==46&&self.state.frame>0&&self.state.events.iter().all(|t|t.kind==Kind::Step),"Finite input requires in-flight prequeued movement");
  let at=q.at.context("Input clock unknown")?;ensure!(at>=self.state.last&&self.state.events.first().is_some_and(|t|at<t.at),"Input crosses prequeued source callback");
  let mut s=self.state.clone();let old_target=s.target;
  s.target=mouse_cell(q.point.context("Handler point unknown")?,s.camera.center)?;
  s.active_move=s.target!=s.cell;s.last=at;s.left-=1;
  let fact=Fact{at,old_target,target:s.target,cell:s.cell,active_move:s.active_move,heading:s.heading,frame:s.frame,camera:s.camera,unchanged_timers:s.events.clone()};
  Ok(PreparedInput{identity:self.identity.clone(),revision:self.revision,state:s,fact})
 }
 pub fn commit_input(&mut self,b:&a::Binding,now:i64,p:PreparedInput)->Result<Fact>{
  self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&p.revision==self.revision,"Foreign/stale private input preparation");
  self.state=p.state;self.revision+=1;Ok(p.fact)
 }
}
