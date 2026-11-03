//! Conditional SD2/wait/viewport producer, bounded through first cell end0.
//! Per-source input identity is independent of actor/HP and process timer identity.
use super::*;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) enum Handlers{Uninstalled,Waiting,Restored}
struct InputIdentity;
pub(crate) struct Token{identity:Arc<InputIdentity>,incarnation:u64,revision:u64}
#[derive(Clone,Debug)]pub(crate) struct Install{pub at:Option<u32>,pub reached4a98a0:Option<bool>,pub phase4_resources_complete:Option<bool>,pub waiting64afc7:Option<bool>,pub wait_start64aac8:Option<u32>,pub counters:Option<(u32,u32)>,pub fresh_schedule:Option<u32>,pub provider_success:Option<bool>}
#[derive(Clone,Debug)]pub(crate) struct Restore{pub at:Option<u32>,pub completed64afc8:Option<bool>,pub saved_handlers:Option<[u32;3]>,pub spinner_provider_success:Option<bool>,pub restore_provider_success:Option<bool>,pub wait_branch469bd0:Option<bool>}
#[derive(Clone,Debug)]pub(crate) struct Click{
 pub at:Option<u32>,pub message:Option<u32>,pub flags:Option<u32>,pub prior_throttle64b474:Option<u32>,pub source_dispatcher_reached:Option<bool>,pub ui_nonconsuming:Option<bool>,pub viewport_list_member:Option<bool>,pub viewport_vtable:Option<u32>,pub viewport:Option<[i32;4]>,pub image1048_height:Option<i32>,pub layout64bbf0:Option<i32>,pub actor4:Option<u8>,pub disabled64b8ec:Option<bool>,pub decoded831708:Option<i32>,pub spell75a82c_absent:Option<bool>,pub selected75a98c_absent:Option<bool>,pub popup75a838_absent:Option<bool>,pub random64bc2b:Option<bool>,pub busy4e6af0:Option<bool>,pub pending834eb3:Option<bool>,pub target_conversion_source_reached:Option<bool>,pub start:movement::Start,
}
pub(crate) struct PreparedClick{identity:Arc<InputIdentity>,revision:u64,incarnation:u64,process_revision:u64,table:Vec<NativeTimer>,scene_generation:u64,scene_last:u32,q:Click}
pub(crate) struct PreparedHeld{identity:Arc<InputIdentity>,revision:u64,incarnation:u64,process_revision:u64,table:Vec<NativeTimer>,scene_generation:u64,scene_last:u32,event:Event,q:movement::held::Inputs}
pub(crate) struct Control<'a>{pub process:super::Model<'a>,identity:Arc<InputIdentity>,incarnation:u64,revision:u64,pub handlers:Handlers,pub counters:(u32,u32),wait_start:Option<u32>,last:u32,first_mouse:Option<crate::modern_lito::world_map::PixelPoint>,last_press:Option<u32>}
impl<'a> Control<'a>{
 pub fn new(process:super::Model<'a>)->Self{let incarnation=process.source.private_incarnation;let last=process.last;Self{process,identity:Arc::new(InputIdentity),incarnation,revision:0,handlers:Handlers::Uninstalled,counters:(0,0),wait_start:None,last,first_mouse:None,last_press:None}}
 fn active(&self,b:&Binding,now:i64)->Result<()>{self.process.current.scene.active(b,now)?;ensure!(self.process.registry==Some(self.incarnation)&&self.process.source.private_incarnation==self.incarnation&&self.process.source.binding==*b,"Input source changed or registry absent");Ok(())}
 fn before(&self,at:u32)->Result<()>{ensure!(at>=self.last&&at>=self.process.current.scene.last&&self.process.timers.first().is_none_or(|t|at<t.at),"Input/restore overtakes process callback or clock backward");Ok(())}
 pub fn reset_new_source(&mut self,b:&Binding,now:i64)->Result<()>{self.process.current.scene.active(b,now)?;ensure!(self.process.registry==Some(self.process.source.private_incarnation)&&self.process.source.private_incarnation!=self.incarnation,"No distinct registered input source");self.incarnation=self.process.source.private_incarnation;self.identity=Arc::new(InputIdentity);self.revision+=1;self.handlers=Handlers::Uninstalled;self.wait_start=None;self.first_mouse=None;self.last_press=None;self.last=self.process.last;Ok(())}
 pub fn token(&self,b:&Binding,now:i64)->Result<Token>{self.active(b,now)?;Ok(Token{identity:self.identity.clone(),incarnation:self.incarnation,revision:self.revision})}
 fn token_valid(&self,t:&Token)->Result<()>{ensure!(Arc::ptr_eq(&self.identity,&t.identity)&&t.incarnation==self.incarnation&&t.revision==self.revision,"Old/foreign/stale input token");Ok(())}
 pub fn install(&mut self,b:&Binding,now:i64,q:&Install)->Result<NativeTimer>{
  self.active(b,now)?;let at=q.at.context("SD2 clock unknown")?;self.before(at)?;ensure!(self.handlers==Handlers::Uninstalled&&q.reached4a98a0==Some(true)&&q.phase4_resources_complete==Some(true)&&q.provider_success==Some(true),"SD2 branch/phase4/provider unknown or installed");
  let waiting=q.waiting64afc7.context("Wait flag unknown")?;ensure!(waiting,"Immediate SD2 mouse512 side effects unimplemented");let s=q.wait_start64aac8.context("Wait start unknown")?;ensure!(s<=at&&s.checked_add(10000).is_some(),"Wait clock wrap/backward");let start=Some(s);
  let fresh=q.fresh_schedule.context("Keeper fresh clock unknown")?;let due=fresh.checked_add(64).context("Keeper clock wrap")?;ensure!(fresh>=at&&self.process.timers.iter().filter(|t|!(t.callback==0x4878c0&&t.owner==Owner::Zero)).all(|t|fresh<t.at),"SD2 clock crosses other callbacks");let(c,d)=q.counters.context("Keeper counters unknown")?;let counters=(c.checked_add(1).context("Counter wrap")?,d.checked_add(1).context("Counter wrap")?);
  // 4A98A0 cancels function+owner before immediate4878C0; neither is a draw.
  self.process.timers.retain(|t|!(t.callback==0x4878c0&&t.owner==Owner::Zero));let t=insert(&mut self.process.timers,&mut self.process.serial,due,0x4878c0,Owner::Zero);
  self.counters=counters;self.wait_start=start;self.handlers=if waiting{Handlers::Waiting}else{Handlers::Restored};self.last=fresh;self.process.last=fresh;self.process.revision+=1;self.revision+=1;Ok(t)
 }
 pub fn keeper(&mut self,b:&Binding,now:i64,t:&GlobalTicket,dispatch:Option<u32>,fresh:Option<u32>)->Result<NativeTimer>{
  self.active(b,now)?;ensure!(self.handlers!=Handlers::Uninstalled,"Keeper not installed");let r=self.process.retained_record(t)?;ensure!(self.process.timers.first()==Some(&r)&&r.callback==0x4878c0&&r.owner==Owner::Zero&&dispatch==Some(r.at)&&r.at>=self.last,"Keeper is not current process root");
  let fresh=fresh.context("Keeper fresh clock unknown")?;ensure!(fresh>=r.at&&self.process.timers.iter().skip(1).all(|t|fresh<t.at),"Keeper schedule crosses another root");let due=fresh.checked_add(64).context("Keeper clock wraps")?;let counters=(self.counters.0.checked_add(1).context("Counter wraps")?,self.counters.1.checked_add(1).context("Counter wraps")?);
  self.process.timers.remove(0);let next=insert(&mut self.process.timers,&mut self.process.serial,due,0x4878c0,Owner::Zero);self.counters=counters;self.last=fresh;self.process.last=fresh;self.process.revision+=1;Ok(next)
 }
 pub fn restore(&mut self,b:&Binding,now:i64,q:&Restore)->Result<()>{
  self.active(b,now)?;let at=q.at.context("Wait dispatch clock unknown")?;self.before(at)?;let start=self.wait_start.context("No wait start")?;
  ensure!(self.handlers==Handlers::Waiting&&q.wait_branch469bd0==Some(true)&&q.completed64afc8==Some(true)&&q.saved_handlers==Some([0x47bb10,0x47c4d0,0x47cdb0])&&q.spinner_provider_success==Some(true)&&q.restore_provider_success==Some(true),"Wait completion/handler/provider/producer unknown");
  ensure!(start.checked_add(3000).is_some_and(|v|v<at),"Strict wait threshold not passed");self.handlers=Handlers::Restored;self.last=at;self.process.last=at;self.revision+=1;self.process.revision+=1;Ok(())
 }
 // An observed active tick body is a separate producer. Only its source
 // no-op gate is projected; never claim the raw488860 body executed.
 pub fn restored_tick(&self,b:&Binding,now:i64,flag641498:Option<bool>,flag834eb4:Option<bool>)->Result<()>{self.active(b,now)?;ensure!(self.handlers==Handlers::Restored&&flag641498==Some(false)&&flag834eb4==Some(true),"Restored active tick488860 effects unqualified");Ok(())}
 pub fn prepare_click(&self,b:&Binding,now:i64,t:&Token,q:&Click)->Result<PreparedClick>{
  self.active(b,now)?;self.token_valid(t)?;ensure!(self.handlers==Handlers::Restored,"Actor/HP/source owner is not restored input");let at=q.at.context("Click clock unknown")?;self.before(at)?;
  ensure!(q.message==Some(513)&&q.flags==Some(1)&&q.source_dispatcher_reached==Some(true)&&q.ui_nonconsuming==Some(true)&&q.viewport_list_member==Some(true)&&q.viewport_vtable==Some(0x4f96d8)&&q.target_conversion_source_reached==Some(true),"Dispatcher/UI/list/vtable/first-click source unknown");
  let throttle=q.prior_throttle64b474.context("Mouse throttle clock unknown")?;ensure!(at<=i32::MAX as u32&&throttle.checked_add(102).is_some_and(|v|v<=at&&v<=i32::MAX as u32),"Signed mouse throttle/wrap refuses");
  let mouse=q.start.mouse.context("Click coordinates unknown")?;ensure!((0..=65535).contains(&mouse.x)&&(0..=65535).contains(&mouse.y)&&q.start.at==Some(at),"Native unsigned packed coordinate/start clock differs");let[l,top,r,bottom]=q.viewport.context("Viewport bounds unknown")?;ensure!(l<r&&top<bottom&&mouse.x>=l&&mouse.x<r&&mouse.y>=top&&mouse.y<bottom,"Click outside source viewport");
  let h=q.image1048_height.context("HUD image1048 height unknown")?;let layout=q.layout64bbf0.context("HUD layout unknown")?;ensure!((0..=480).contains(&h)&&(-2..=2).contains(&layout),"HUD arithmetic bound");let hud=620-34*layout-h;ensure!(!(mouse.x>=136&&mouse.x<485&&mouse.y>=hud&&mouse.y<480),"Click consumed by source HUD rectangle");
  ensure!(q.actor4==Some(0)&&q.disabled64b8ec==Some(false)&&q.decoded831708==Some(0)&&q.spell75a82c_absent==Some(true)&&q.selected75a98c_absent==Some(true)&&q.popup75a838_absent==Some(true)&&q.random64bc2b==Some(false)&&q.busy4e6af0==Some(false)&&q.pending834eb3==Some(false),"First click optional/state/busy/pending branch unqualified");
  self.process.current.movement.prepare_click_start(b,now,&q.start)?;
  Ok(PreparedClick{identity:self.identity.clone(),revision:self.revision,incarnation:self.incarnation,process_revision:self.process.revision,table:self.process.timers.clone(),scene_generation:self.process.current.scene.generation,scene_last:self.process.current.scene.last,q:q.clone()})
 }
 pub fn commit_click(&mut self,b:&Binding,now:i64,p:PreparedClick,draw:&DrawInputs)->Result<(Frame,movement::Step)>{
  self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&self.revision==p.revision&&self.incarnation==p.incarnation&&self.process.revision==p.process_revision&&self.process.timers==p.table&&self.process.current.scene.generation==p.scene_generation&&self.process.current.scene.last==p.scene_last,"Stale/foreign prepared click");
  let at=p.q.at.unwrap();self.before(at)?;let result=self.process.current.click_start(b,now,&p.q.start,draw)?;self.sync_owned(b,now)?;self.first_mouse=p.q.start.mouse;self.last_press=Some(at);self.last=at;self.process.last=at;self.process.revision+=1;self.revision+=1;Ok(result)
 }
 fn sync_owned(&mut self,b:&Binding,now:i64)->Result<()>{
  let mut remaining=owned(&self.process.current,b,now,self.incarnation)?;
  // Preserve unchanged native records and their FIFO serial/order. Only the
  // removed callback disappears; genuinely new schedules append at equal due.
  self.process.timers.retain(|t|{let owned=matches!(t.owner,Owner::Actor(v)|Owner::Fire(v)|Owner::Roof(v,_) if v==self.incarnation)||t.callback==0x4e2150;if !owned{return true;}if let Some(i)=remaining.iter().position(|v|*v==(t.at,t.callback,t.owner)){remaining.remove(i);true}else{false}});
  for(at,c,o)in remaining{insert(&mut self.process.timers,&mut self.process.serial,at,c,o);}Ok(())
 }
 pub fn scene_root(&mut self,b:&Binding,now:i64,t:&Ticket,q:Option<&movement::Callback>,other:Option<super::super::Inputs>,roof:Option<&roof::MoveInputs>,draw:&DrawInputs)->Result<(Frame,Option<movement::Step>)>{
  self.active(b,now)?;let e=t.event;let expected=owned(&self.process.current,b,now,self.incarnation)?;let first=expected.first().context("No owned root")?;ensure!(self.process.timers.first().is_some_and(|r|(r.at,r.callback,r.owner)==*first)&&e.at==first.0&&e.at>=self.last,"Scene bypasses process/global callback FIFO");
  let result=if matches!(e.kind,Kind::Locomotion(_)){ensure!(other.is_none(),"Unexpected fire input");let(f,s)=self.process.current.dispatch(b,now,t,q.context("Movement input missing")?,roof,draw)?;(f,Some(s))}else{ensure!(q.is_none()&&roof.is_none(),"Unexpected movement input");(self.process.current.scene.dispatch(b,now,t,other.context("Scene callback input missing")?,draw)?,None)};
  self.sync_owned(b,now)?;self.last=e.at;self.process.last=e.at;self.process.revision+=1;Ok(result)
 }
 pub fn prepare_held(&self,b:&Binding,now:i64,token:&Token,t:&Ticket,q:&movement::held::Inputs)->Result<PreparedHeld>{
  self.active(b,now)?;self.token_valid(token)?;ensure!(self.handlers==Handlers::Restored&&self.first_mouse.is_some()&&q.global_mouse==self.first_mouse,"Held source mouse changed/unknown or input not restored");ensure!(Arc::ptr_eq(&self.process.current.scene.identity,&t.identity)&&self.process.current.scene.events.first()==Some(&t.event),"Old/foreign held root");
  let root=owned(&self.process.current,b,now,self.incarnation)?.into_iter().next().context("Held root absent")?;ensure!(self.process.timers.first().is_some_and(|r|(r.at,r.callback,r.owner)==root)&&t.event.at==root.0&&t.event.at>=self.last,"Held arrival bypasses process/global FIFO");let timer=self.process.current.movement.next(b,now)?;ensure!(t.event.kind==Kind::Locomotion(timer.serial)&&t.event.at==timer.at,"Held root does not select movement timer");self.process.current.movement.prepare_held_arrival(b,now,timer,q)?;
  let last=q.callback.fresh_schedule_samples.as_ref().and_then(|v|v.last()).context("Held fresh clocks absent")?;ensure!(self.process.timers.iter().skip(1).all(|r|*last<r.at),"Held source fresh schedule crosses another due process callback");
  Ok(PreparedHeld{identity:self.identity.clone(),revision:self.revision,incarnation:self.incarnation,process_revision:self.process.revision,table:self.process.timers.clone(),scene_generation:self.process.current.scene.generation,scene_last:self.process.current.scene.last,event:t.event,q:q.clone()})
 }
 pub fn commit_held(&mut self,b:&Binding,now:i64,p:PreparedHeld,t:&Ticket,draw:&DrawInputs)->Result<(Frame,movement::Step,movement::held::Fact)>{
  self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&self.revision==p.revision&&self.incarnation==p.incarnation&&self.process.revision==p.process_revision&&self.process.timers==p.table&&self.process.current.scene.generation==p.scene_generation&&self.process.current.scene.last==p.scene_last&&t.event==p.event,"Stale/foreign prepared held arrival");
  let result=self.process.current.held_arrival(b,now,t,&p.q,draw)?;self.sync_owned(b,now)?;self.last=p.event.at;self.process.last=p.event.at;self.process.revision+=1;self.revision+=1;Ok(result)
 }
}
#[cfg(test)]#[path="native_input_restore_study_tests.rs"]mod tests;
#[path="native_active_tick_study.rs"]pub(crate) mod tick;
#[cfg(test)]#[path="native_held_arrival_study_tests.rs"]mod held_tests;
#[path="native_release_stop_study.rs"]pub(crate) mod release_stop;
#[cfg(test)]#[path="native_release_stop_study_tests.rs"]mod release_stop_tests;
#[path="native_held_current_input_study.rs"]pub(crate) mod held_current;
#[cfg(test)]#[path="native_held_current_input_study_tests.rs"]mod held_current_tests;
#[path="native_stopped_restart_input_study.rs"]pub(crate) mod restart;
#[cfg(test)]#[path="native_stopped_restart_input_study_tests.rs"]mod restart_tests;

#[path="native_send_idle_input_study.rs"]pub(crate) mod send_idle;
#[cfg(test)]#[path="native_send_idle_input_study_tests.rs"]mod send_idle_tests;

#[path="native_idle_reclick_input_study.rs"]pub(crate) mod idle_reclick;
#[cfg(test)]#[path="native_idle_reclick_input_study_tests.rs"]mod idle_reclick_tests;
