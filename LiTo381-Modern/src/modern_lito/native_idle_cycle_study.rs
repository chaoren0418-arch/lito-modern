//! Explicit continuation consumes the private completed first-idle seed once.
//! Catalog/binding JSON cannot produce the seed or a live appearance owner.
use super::{a,clock,Model as First};use anyhow::{ensure,Context,Result};use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
struct Identity{binding:a::Binding,retired:AtomicBool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) enum Kind{Ordinary,QueuedRepeat}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) struct Timer{pub serial:u64,pub at:u32,pub kind:Kind}
pub(crate) struct Inputs{pub callback_actor_matches:Option<bool>,pub timer_removed_before_callback:Option<bool>,pub queue_before_pop:Option<usize>,pub actor68_absent:Option<bool>,pub actor4:Option<u8>,pub actor50:Option<u32>,pub render_player_pointer_equal:Option<bool>,pub prior_submissions_unlinked:Option<bool>,pub post_load_actor_globals_match:Option<bool>,pub source_special_branches_absent:Option<bool>,pub idle_table_pointer_present:Option<bool>,pub queue_append_succeeds:Option<bool>,pub ordinary:clock::OrdinaryInputs,pub scheduler_sample_ms:Option<u32>}
#[derive(Clone)]struct State{frame:usize,timer:Timer,expected:u32,last:u32,left:usize}
pub(crate) struct Model{identity:Arc<Identity>,frames:Vec<a::CatalogFrame>,heading:u8,state:State,revision:u64}
pub(crate) struct Prepared{identity:Arc<Identity>,revision:u64,state:State,pub step:Step}
#[derive(Clone,Debug)]pub(crate) struct Step{pub clock:u32,pub sampled:u32,pub frame:usize,pub bank:u32,pub sub:usize,pub ended:bool,pub queued_repeat:bool,pub next:Timer,pub expected:u32,pub delay:u32}
impl Model{
 pub fn from_completed_first(enabled:bool,first:&mut First<'_>,b:&a::Binding,now:i64)->Result<Self>{
  ensure!(enabled&&first.binding==*b&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff),"Idle continuation default off/full binding/cutoff");
  let seed=first.completed_first.context("No private completed first-idle seed; retired/consumed/unexecuted")?;
  ensure!(first.stopped&&first.queued_due.is_none()&&seed.bank==80+u32::from(first.idle.heading)&&seed.subframe==0&&seed.duration_ms==166,"First idle ownership/provenance differs");
  let frames=first.idle.frames.iter().map(|(f,_)|(*f).clone()).collect::<Vec<_>>();ensure!(frames.len()==9&&frames.iter().enumerate().all(|(i,f)|f.image==80&&f.subframe==if i==0{8}else{i as u32-1}&&f.ticks==4&&f.events.is_empty()),"All actual idle49 frames/events required");
  let m=Self{identity:Arc::new(Identity{binding:b.clone(),retired:AtomicBool::new(false)}),frames,heading:first.idle.heading,state:State{frame:1,timer:Timer{serial:1,at:seed.next_due_ms,kind:Kind::Ordinary},expected:seed.next_due_ms,last:seed.dispatch_ms,left:12},revision:0};first.completed_first=None;Ok(m)
 }
 fn active(&self,b:&a::Binding,now:i64)->Result<()>{ensure!(!self.identity.retired.load(Ordering::Acquire)&&self.identity.binding==*b&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff)&&self.state.left>0,"Idle cycle retired/full private owner/cutoff/budget");Ok(())}
 pub fn next(&self,b:&a::Binding,now:i64)->Result<Timer>{self.active(b,now)?;Ok(self.state.timer)}
 pub fn prepare(&self,b:&a::Binding,now:i64,t:Timer,q:&Inputs)->Result<Prepared>{
  self.active(b,now)?;ensure!(t==self.state.timer&&q.callback_actor_matches==Some(true)&&q.timer_removed_before_callback==Some(true)&&q.actor68_absent==Some(true),"Idle timer removal/owner/pop branch unknown");
  ensure!(q.actor4==Some(0)&&q.actor50==Some(0)&&q.render_player_pointer_equal==Some(false)&&q.post_load_actor_globals_match==Some(false)&&q.prior_submissions_unlinked==Some(true)&&q.source_special_branches_absent==Some(true)&&q.idle_table_pointer_present==Some(true)&&q.queue_append_succeeds==Some(true),"Idle action/global/counter/end queue branches unknown");
  let o=&q.ordinary;ensure!(o.ordinary_branch_reached==Some(true)&&o.frozen_external_state==Some(true)&&i64::from(o.player_global.context("Idle clock global unknown")?)!=b.object&&o.queue_after_pop==Some(0)&&o.modifier_1c==Some(0)&&o.modifier_21==Some(0)&&o.action21_counter==Some(0)&&o.catalog_scaling_disabled==Some(true),"Only explicit neutral nonplayer ordinary clock qualified");
  let at=o.sampled_clock_ms.context("Idle dispatch clock missing")?;let sampled=q.scheduler_sample_ms.context("Idle fresh source clock missing")?;ensure!(at==t.at&&at>=self.state.last&&sampled>=at,"Early/nonmonotonic idle dispatch/fresh clock");
  let mut s=self.state.clone();let(ended,queued,delay,next_kind)=match t.kind{
   Kind::Ordinary=>{ensure!(q.queue_before_pop==Some(0),"Ordinary idle actor queue unknown");s.frame=(s.frame+1)%9;if s.frame==0{(true,true,166,Kind::QueuedRepeat)}else{ensure!(sampled>=s.expected,"Idle fresh sample precedes expected anchor");let late=sampled-s.expected;let delay=166u32.saturating_sub(late).max(1)+2;s.expected=s.expected.checked_add(168).context("Idle expected wraps")?;(false,false,delay,Kind::Ordinary)}},
   Kind::QueuedRepeat=>{ensure!(q.queue_before_pop==Some(1)&&s.frame==0,"End0 owned opcode32 missing");s.frame=1;s.expected=sampled.checked_add(168).context("Repeat expected wraps")?;(false,false,168,Kind::Ordinary)},
  };
  let f=&self.frames[s.frame];let next=Timer{serial:t.serial.checked_add(1).context("Timer serial wraps")?,at:sampled.checked_add(delay).context("Idle next wraps")?,kind:next_kind};s.last=at;s.timer=next;s.left-=1;
  let step=Step{clock:at,sampled,frame:s.frame,bank:f.image+u32::from(self.heading),sub:f.subframe as usize,ended,queued_repeat:queued,next,expected:s.expected,delay};Ok(Prepared{identity:self.identity.clone(),revision:self.revision,state:s,step})
 }
 pub fn commit(&mut self,b:&a::Binding,now:i64,p:Prepared)->Result<Step>{self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&self.revision==p.revision,"Foreign/stale private idle preparation");self.state=p.state;self.revision+=1;Ok(p.step)}
 pub fn retire(&mut self){self.identity.retired.store(true,Ordering::Release);}
}
impl Drop for Model{fn drop(&mut self){self.retire();}}
