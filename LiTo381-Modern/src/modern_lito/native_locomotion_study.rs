//! Conditional source4E2150 player movement. No observed runtime pointers,
//! collision producer, network sender, audio backend or live appearance grant.
use crate::modern_lito::{native_appearance_inspection as a,native_actor_inspection::catalog_frame_ms,world_map::{WorldCell,PixelPoint},world_depth::NativeDrawCell};
use anyhow::{ensure,Context,Result};
use std::sync::{Arc,atomic::{AtomicBool,Ordering}};

#[derive(Clone,Debug)]pub(crate) struct Runtime{
 pub render_player_matches:Option<bool>,pub player_object:Option<i64>,pub decoded_class:Option<i32>,pub decoded_action:Option<u8>,pub catalog_classes:Option<usize>,
 pub decoded_speed:Option<i32>,pub decoded_haste:Option<i32>,pub special_speed_flag:Option<bool>,pub actor_counter11c:Option<i32>,pub movement_variant_count:Option<u8>,
 pub disabled64b8ec:Option<bool>,pub decoded_state831708:Option<i32>,pub continuous_key760098:Option<bool>,pub interaction_mode:Option<u32>,
 pub actor70_absent:Option<bool>,pub actor139:Option<u8>,pub actor68_absent:Option<bool>,pub selected_targets_absent:Option<bool>,pub popup75a838_absent:Option<bool>,
 pub reload834f1d:Option<bool>,pub interaction834eb2:Option<bool>,pub warp834f1c:Option<bool>,pub capture64af9f:Option<bool>,pub aux64bc2d:Option<bool>,
 pub source_draw_pointers_present:Option<bool>,pub prior_submissions_unlinked:Option<bool>,pub idle_table_pointer_present:Option<bool>,pub queue_allocation_succeeds:Option<bool>,pub source_light_list_empty:Option<bool>,
 // These side effects are recorded requirements, never executed/observed facts.
 pub record_unexecuted_wire_audio_ui:Option<bool>,
 pub actor4:Option<u8>,pub actor50:Option<i32>,pub ordinary_branch_reached:Option<bool>,pub source_special_class_branches_absent:Option<bool>,
}
impl Runtime{fn validate(&self,b:&a::Binding)->Result<()>{self.validate_key(b,false)}fn validate_key(&self,b:&a::Binding,key:bool)->Result<()>{
 ensure!(self.render_player_matches==Some(true)&&self.player_object==Some(b.object)&&self.decoded_class==Some(734)&&self.decoded_action==Some(46)&&self.catalog_classes.is_some_and(|v|v>734),"True player/global class/action/catalog unknown");
 ensure!(self.decoded_speed==Some(0)&&self.decoded_haste==Some(0)&&self.special_speed_flag==Some(false)&&self.actor_counter11c==Some(0)&&self.movement_variant_count==Some(0),"Neutral true player4E8750/variant branch unknown");
 ensure!(self.disabled64b8ec==Some(false)&&self.decoded_state831708==Some(0)&&self.continuous_key760098==Some(key)&&self.interaction_mode==Some(0),"Only source plain mouse movement state qualified");
 ensure!(self.actor70_absent==Some(true)&&self.actor139==Some(255)&&self.actor68_absent==Some(true)&&self.selected_targets_absent==Some(true)&&self.popup75a838_absent==Some(true),"Optional actor/selection/popup branches unknown");
 ensure!(self.reload834f1d==Some(false)&&self.interaction834eb2==Some(false)&&self.warp834f1c==Some(false)&&self.capture64af9f==Some(false)&&self.aux64bc2d==Some(false),"Reload/interaction/warp/capture/aux branch unqualified");
 ensure!(self.source_draw_pointers_present==Some(true)&&self.prior_submissions_unlinked==Some(true)&&self.idle_table_pointer_present==Some(true)&&self.queue_allocation_succeeds==Some(true)&&self.source_light_list_empty==Some(true)&&self.record_unexecuted_wire_audio_ui==Some(true),"Draw/table/allocation/light/side-effect boundary unknown");
 ensure!(self.actor4==Some(0)&&self.actor50==Some(0)&&self.ordinary_branch_reached==Some(true)&&self.source_special_class_branches_absent==Some(true),"Action override/counter/special class/ordinary branch unknown");Ok(())
}}
#[derive(Clone,Copy,Debug)]pub(crate) struct Camera{pub center:WorldCell,pub offset:PixelPoint,pub base:PixelPoint,pub min:PixelPoint,pub max:PixelPoint,pub ring:PixelPoint}
#[derive(Clone,Debug)]pub(crate) struct Start{pub at:Option<u32>,pub mouse:Option<PixelPoint>,pub source_collision_blocked:Option<[bool;8]>,pub source_final_passable:Option<bool>,pub fresh_schedule_samples:Option<Vec<u32>>,pub runtime:Runtime}
#[derive(Clone,Debug)]pub(crate) struct Callback{pub dispatch:Option<u32>,pub timer_removed:Option<bool>,pub callback_owner_matches:Option<bool>,pub fresh_schedule_samples:Option<Vec<u32>>,pub queue_before_pop:Option<usize>,pub runtime:Runtime,pub source_collision_blocked:Option<[bool;8]>,pub source_final_passable:Option<bool>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) enum Kind{Step,Arrival,IdlePop,IdleAdvance,AttackAdvance,QueuedActionPop}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) struct Timer{pub serial:u64,pub at:u32,pub kind:Kind}
struct Identity{binding:a::Binding,retired:AtomicBool}
#[derive(Clone)]struct State{cell:WorldCell,target:WorldCell,heading:u8,action:u16,frame:usize,camera:Camera,events:Vec<Timer>,serial:u64,last:u32,started:bool,active_move:bool,expected:Option<u32>,left:usize,queue:Option<queue::StateQueue>,continuous_click:bool}
pub(crate) struct Model<'a>{identity:Arc<Identity>,d:&'a a::Dependencies,state:State,revision:u64}
pub(crate) struct Prepared{identity:Arc<Identity>,revision:u64,state:State,pub step:Step}
#[derive(Clone,Debug)]pub(crate) struct Step{pub at:u32,pub action:u16,pub frame:usize,pub bank:u32,pub sub:usize,pub old_cell:WorldCell,pub cell:WorldCell,pub cell_committed:bool,pub camera:Camera,pub raw_events:Vec<u8>,pub added:Vec<Timer>,pub wire_move:Option<(WorldCell,u8)>,pub active_move:bool,pub popped:Option<queue::Node>,pub queued:Vec<queue::Node>}
const DIRECTIONS:[(i32,i32);8]=[(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)];
const QUARTER:[(i32,i32);8]=[(-6,-3),(0,-6),(6,-3),(12,0),(6,3),(0,6),(-6,3),(-12,0)];
fn neighbor(c:WorldCell,h:u8)->Result<WorldCell>{let(dx,dy)=DIRECTIONS[h as usize];Ok(WorldCell{x:c.x.checked_add(dx).context("Cell X overflow")?,y:c.y.checked_add(dy).context("Cell Y overflow")?})}
// Exact bounded signed arithmetic of4B9110->4B9060->48B9F0 even-X.
pub(crate) fn mouse_cell(mouse:PixelPoint,center:WorldCell)->Result<WorldCell>{
 let x=i64::from(mouse.x)+48;let y=i64::from(mouse.y)+48;ensure!(x.abs()<100000&&y.abs()<100000,"Mouse conversion arithmetic bound");
 let iy=(x/2+y+1)/24;let mut hx=((x-2*y)&!1).checked_add(0x12ee).context("Mouse transform overflow")?/48*2-200;
 if x>=12*(hx+2*iy){hx+=1;}
 let base=NativeDrawCell::from_world(center).context("Mouse center overflow")?;let aligned=(i64::from(base.half_x)+hx)&!1;
 Ok(WorldCell{x:i32::try_from((aligned+32768)/2)?,y:i32::try_from(i64::from(base.y)+iy)?})
}
fn schedule(s:&mut State,at:u32,kind:Kind,added:&mut Vec<Timer>){s.serial+=1;let t=Timer{serial:s.serial,at,kind};let p=s.events.partition_point(|v|v.at<=at);s.events.insert(p,t);added.push(t);}
fn samples(q:&Option<Vec<u32>>,at:u32,n:usize)->Result<Vec<u32>>{let v=q.as_ref().context("Missing source fresh schedule clock reads")?;ensure!(v.len()==n&&v.iter().all(|t|*t>=at)&&v.windows(2).all(|p|p[0]<=p[1]),"Missing/extra/nonmonotonic fresh source clocks");Ok(v.clone())}
impl<'a> Model<'a>{
 pub fn new(enabled:bool,d:&'a a::Dependencies,b:&a::Binding,now:i64,cell:WorldCell,camera:Option<Camera>)->Result<Self>{
  let walk=a::plan(enabled,d,b,b,now,a::Kind::Walk,0,16)?;let idle=a::plan(enabled,d,b,b,now,a::Kind::Idle,0,16)?;
  ensure!(b.class==734&&walk.action==46&&walk.fps==24&&walk.frames.len()==4&&idle.action==49&&idle.fps==24&&idle.frames.len()==9,"Exact source movement/idle catalog required");
  for(i,(f,_))in walk.frames.iter().enumerate(){ensure!(f.image==72&&f.subframe==i as u32&&f.ticks==4&&f.events==match i{0=>b"<478".to_vec(),2=>b"<479".to_vec(),_=>vec![]},"Walk event/frame facts differ");}
  for(i,(f,_))in idle.frames.iter().enumerate(){ensure!(f.image==80&&f.subframe==if i==0{8}else{i as u32-1}&&f.ticks==4&&f.events.is_empty(),"Idle frame/event facts differ");}
  let camera=camera.context("Source camera globals unknown")?;ensure!(camera.offset.x>=camera.min.x&&camera.offset.x<camera.max.x&&camera.offset.y>=camera.min.y&&camera.offset.y<camera.max.y&&(0..640).contains(&camera.ring.x)&&(0..420).contains(&camera.ring.y),"Camera/ring bounds unknown");
  Ok(Self{identity:Arc::new(Identity{binding:b.clone(),retired:AtomicBool::new(false)}),d,state:State{cell,target:cell,heading:0,action:49,frame:0,camera,events:vec![],serial:0,last:0,started:false,active_move:false,expected:None,left:16,queue:None,continuous_click:false},revision:0})
 }
 fn active(&self,b:&a::Binding,now:i64)->Result<()>{ensure!(!self.identity.retired.load(Ordering::Acquire)&&self.identity.binding==*b&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff)&&self.state.left>0,"Locomotion retired/private full owner/cutoff/budget");Ok(())}
 pub fn cell(&self,b:&a::Binding,now:i64)->Result<WorldCell>{self.active(b,now)?;Ok(self.state.cell)}
 pub fn next(&self,b:&a::Binding,now:i64)->Result<Timer>{self.active(b,now)?;self.state.events.first().copied().context("No owned locomotion timer")}
 pub fn next_optional(&self,b:&a::Binding,now:i64)->Result<Option<Timer>>{self.active(b,now)?;Ok(self.state.events.first().copied())}
 fn quarter(s:&mut State)->Result<()>{let(dx,dy)=QUARTER[s.heading as usize];s.camera.offset.x=s.camera.offset.x.checked_add(dx).context("Camera X overflow")?;s.camera.offset.y=s.camera.offset.y.checked_add(dy).context("Camera Y overflow")?;s.camera.ring.x=(s.camera.ring.x-dx+640)%640;s.camera.ring.y=(s.camera.ring.y-dy+420)%420;
  Ok(())}
 fn prepared(&self,mut s:State,at:u32,old:WorldCell,committed:bool,added:Vec<Timer>,wire:Option<(WorldCell,u8)>)->Result<Prepared>{
  let row=self.d.rows.get(&734).context("Missing movement row")?;let action=row.actions.get(&s.action).context("Missing owned action")?;let f=action.frames.get(s.frame).context("Missing owned catalog frame")?;s.last=at;s.left-=1;
  let step=Step{at,action:s.action,frame:s.frame,bank:f.image+u32::from(s.heading),sub:f.subframe as usize,old_cell:old,cell:s.cell,cell_committed:committed,camera:s.camera,raw_events:f.events.clone(),added,wire_move:wire,active_move:s.active_move,popped:None,queued:s.queue.as_ref().map_or_else(Vec::new,|q|q.nodes.clone())};
  Ok(Prepared{identity:self.identity.clone(),revision:self.revision,state:s,step})
 }
 pub fn prepare_start(&self,b:&a::Binding,now:i64,q:&Start)->Result<Prepared>{
  self.active(b,now)?;q.runtime.validate(b)?;ensure!(!self.state.started,"Finite movement already started");let at=q.at.context("Move startup clock unknown")?;let v=samples(&q.fresh_schedule_samples,at,3)?;let mut s=self.state.clone();let old=s.cell;
  s.target=mouse_cell(q.mouse.context("Mouse input unknown")?,s.camera.center)?;let added=self.begin_step(&mut s,at,q.source_collision_blocked,q.source_final_passable,&v)?;
  let heading=s.heading;self.prepared(s,at,old,false,added,Some((old,heading)))
 }
 // Source48C4F5 sets760098 before the direct call. Its first unequal-target
 // step and end0 schedule are equivalent here; arrival target refresh is not.
 pub fn prepare_click_start(&self,b:&a::Binding,now:i64,q:&Start)->Result<Prepared>{
  self.active(b,now)?;q.runtime.validate_key(b,true)?;ensure!(!self.state.started&&self.state.action==46&&self.state.frame==0&&self.state.events.is_empty()&&self.state.queue.as_ref().is_some_and(|q|q.nodes.is_empty()&&q.capacity==4),"First click requires registered base46/frame0 and empty capacity4 queue");
  let at=q.at.context("Click startup clock unknown")?;ensure!(at>=self.state.last,"Click clock backward");let v=samples(&q.fresh_schedule_samples,at,3)?;let mut s=self.state.clone();let old=s.cell;
  s.target=mouse_cell(q.mouse.context("Click mouse unknown")?,s.camera.center)?;s.continuous_click=true;
  let added=self.begin_step(&mut s,at,q.source_collision_blocked,q.source_final_passable,&v)?;let heading=s.heading;self.prepared(s,at,old,false,added,Some((old,heading)))
 }
 fn begin_step(&self,s:&mut State,at:u32,blocked:Option<[bool;8]>,passable:Option<bool>,v:&[u32])->Result<Vec<Timer>>{
  ensure!(s.target!=s.cell&&v.len()==3,"Next step requires remaining target and three clocks");let blocked=blocked.context("Fresh48B640 collision outputs unknown")?;
  let mut best=u64::MAX;let mut heading=0;for h in 0..8{let c=neighbor(s.cell,h)?;let dx=i64::from(c.x)-i64::from(s.target.x);let dy=i64::from(c.y)-i64::from(s.target.y);ensure!(dx.abs()<4096&&dy.abs()<4096,"Distance arithmetic unqualified");let cost=if blocked[h as usize]{99999999}else{(dx*dx+dy*dy)as u64};if cost<best{best=cost;heading=h;}}
  ensure!(!blocked[heading as usize]&&passable==Some(true),"Fresh48B7B0 final passability unknown/blocked");s.heading=heading;s.action=46;s.frame=1;s.active_move=true;s.started=true;Self::quarter(s)?;
  if s.camera.offset.x<s.camera.min.x||s.camera.offset.x>=s.camera.max.x||s.camera.offset.y<s.camera.min.y||s.camera.offset.y>=s.camera.max.y{s.camera.offset=s.camera.base;}
  let mut added=vec![];
  ensure!(v.iter().all(|sample|*sample>=at),"Next step fresh clock precedes source dispatch");
  for(i,sampled)in v.iter().enumerate(){let f=&self.d.rows[&734].actions[&46].frames[i+1];let duration=catalog_frame_ms(u32::from(f.ticks),24)?;let delay=duration.checked_mul(i as u32+1).context("Prequeue delay overflow")?;schedule(s,sampled.checked_add(delay).context("Move timer wraps")?,Kind::Step,&mut added);}
  Ok(added)
 }
 pub fn prepare_callback(&self,b:&a::Binding,now:i64,t:Timer,q:&Callback)->Result<Prepared>{
  self.active(b,now)?;q.runtime.validate_key(b,self.state.continuous_click)?;ensure!(!self.state.continuous_click||t.kind==Kind::Step,"Held-button arrival target refresh unimplemented");ensure!(self.state.events.first()==Some(&t)&&q.timer_removed==Some(true)&&q.callback_owner_matches==Some(true)&&q.dispatch==Some(t.at)&&t.at>=self.state.last,"Early/stale/timer/owner dispatch unknown");
  let mut s=self.state.clone();s.events.remove(0);let old=s.cell;let mut added=vec![];let mut committed=false;let mut wire=None;let mut popped=None;let queued=s.queue.as_ref().map_or(0,|q|q.nodes.len());
  match t.kind{
   Kind::Step=>{ensure!(s.action==46&&s.frame>0,"Owned move action/frame differs");Self::quarter(&mut s)?;s.frame+=1;if s.frame==4{s.frame=0;s.cell=neighbor(s.cell,s.heading)?;s.camera.center=neighbor(s.camera.center,s.heading)?;s.camera.offset=s.camera.base;committed=true;ensure!(q.queue_before_pop==Some(queued),"End0 private/native queue count mismatch");if queued>0||s.active_move||s.continuous_click{let v=samples(&q.fresh_schedule_samples,t.at,1)?;schedule(&mut s,v[0].checked_add(166).context("End0 schedule wraps")?,if queued>0{Kind::QueuedActionPop}else{Kind::Arrival},&mut added);}else{samples(&q.fresh_schedule_samples,t.at,0)?;}}else{samples(&q.fresh_schedule_samples,t.at,0)?;}}
   Kind::Arrival=>{ensure!(s.active_move&&s.frame==0&&s.action==46&&q.queue_before_pop==Some(0),"Carry move/frame/queue state unknown");if s.cell==s.target{s.active_move=false;let v=samples(&q.fresh_schedule_samples,t.at,1)?;schedule(&mut s,v[0].checked_add(584).context("Idle queue wraps")?,Kind::IdlePop,&mut added);}else{let v=samples(&q.fresh_schedule_samples,t.at,3)?;added=self.begin_step(&mut s,t.at,q.source_collision_blocked,q.source_final_passable,&v)?;wire=Some((old,s.heading));}}
   Kind::IdlePop=>{ensure!(q.queue_before_pop==Some(1)&&matches!(s.action,46|47|49)&&s.frame==0&&(s.action==47||!s.active_move),"Idle source opcode32 queue mismatch");let v=samples(&q.fresh_schedule_samples,t.at,1)?;s.action=49;s.frame=1;let next=v[0].checked_add(168).context("First idle schedule wraps")?;s.expected=Some(next);schedule(&mut s,next,Kind::IdleAdvance,&mut added);}
   Kind::QueuedActionPop=>{ensure!(s.frame==0&&matches!(s.action,46|47)&&queued>0&&q.queue_before_pop==Some(queued),"Queued action pop/owner/count mismatch");let v=samples(&q.fresh_schedule_samples,t.at,1)?;let queue=s.queue.as_mut().context("No private source queue")?;ensure!(queue.nodes[0].arg==1,"Only queuedarg1 execution qualified; arg2 tail retained");popped=Some(queue.nodes.remove(0));let plan=a::plan(true,self.d,b,b,now,a::Kind::Attack,s.heading,16)?;ensure!(plan.action==47&&plan.fps==24&&plan.frames.len()==7&&plan.frames[0].0.image==72&&plan.frames[1].0.image==88&&plan.frames[1].0.subframe==0&&plan.frames[1].0.ticks==4,"Queued attack catalog differs");s.action=47;s.frame=1;let next=v[0].checked_add(168).context("Queued attack first wraps")?;s.expected=Some(next);schedule(&mut s,next,Kind::AttackAdvance,&mut added);}
   Kind::AttackAdvance=>{ensure!(s.action==47&&s.frame>0&&q.queue_before_pop==Some(queued),"Ordinary attack/queue state unknown");let v=samples(&q.fresh_schedule_samples,t.at,1)?;s.frame+=1;if s.frame==7{s.frame=0;let next=v[0].checked_add(if queued>0{83}else{584}).context("Attack end queue wraps")?;schedule(&mut s,next,if queued>0{Kind::QueuedActionPop}else{Kind::IdlePop},&mut added);}else{let expected=s.expected.context("Attack expected clock absent")?;ensure!(v[0]>=expected,"Attack clock precedes expected");let f=&self.d.rows[&734].actions[&47].frames[s.frame];let duration=catalog_frame_ms(u32::from(f.ticks),24)?;let delay=duration.saturating_sub(v[0]-expected).max(1)+2;let next=v[0].checked_add(delay).context("Attack ordinary schedule wraps")?;s.expected=Some(expected.checked_add(duration+2).context("Attack expected wraps")?);schedule(&mut s,next,Kind::AttackAdvance,&mut added);}}
   Kind::IdleAdvance=>{ensure!(s.action==49&&q.queue_before_pop==Some(0),"Ordinary idle actor queue mismatch");let v=samples(&q.fresh_schedule_samples,t.at,1)?;s.frame=(s.frame+1)%9;if s.frame==0{schedule(&mut s,v[0].checked_add(166).context("Idle end requeue wraps")?,Kind::IdlePop,&mut added);}else{let expected=s.expected.context("Idle expected clock missing")?;ensure!(v[0]>=expected,"Idle sample precedes expected clock");let late=v[0]-expected;let delay=166u32.saturating_sub(late).max(1)+2;let next=v[0].checked_add(delay).context("Idle ordinary clock wraps")?;s.expected=Some(expected.checked_add(168).context("Idle expected wraps")?);schedule(&mut s,next,Kind::IdleAdvance,&mut added);}}
  }
  let mut p=self.prepared(s,t.at,old,committed,added,wire)?;p.step.popped=popped;Ok(p)
 }
 pub fn commit(&mut self,b:&a::Binding,now:i64,p:Prepared)->Result<Step>{self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&p.revision==self.revision,"Foreign/stale locomotion preparation");self.state=p.state;self.revision+=1;Ok(p.step)}
 pub fn retire(&mut self){self.identity.retired.store(true,Ordering::Release);self.state.events.clear();if let Some(q)=&mut self.state.queue{q.nodes.clear();q.capacity=0;}}
}
impl Drop for Model<'_>{fn drop(&mut self){self.retire();}}

#[path="native_locomotion_input_study.rs"]pub(crate) mod input;
#[path="native_locomotion_action_study.rs"]pub(crate) mod action;
#[path="native_locomotion_queue_study.rs"]pub(crate) mod queue;
#[path="native_locomotion_held_arrival_study.rs"]pub(crate) mod held;
#[path="native_locomotion_release_stop_study.rs"]pub(crate) mod release_stop;
#[path="native_locomotion_current_target_study.rs"]pub(crate) mod current_target;
#[path="native_locomotion_restart_study.rs"]pub(crate) mod restart;

#[path="native_locomotion_queued_idle_study.rs"]pub(crate) mod queued_idle;

#[path="native_locomotion_idle_reclick_study.rs"]pub(crate) mod idle_reclick;
