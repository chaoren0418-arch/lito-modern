//! Source4E63C0 opcode32 queue. Only bounded private nodes and explicit native
//! allocation/registry assumptions; no receive, ACK or native pointer authority.
use super::*;
#[derive(Clone,Debug,PartialEq,Eq)]pub(crate) struct Node{pub serial:u64,pub arg:u8}
#[derive(Clone)]pub(super) struct StateQueue{pub nodes:Vec<Node>,pub capacity:usize,pub serial:u64}
#[derive(Clone,Debug)]pub(crate) struct Enable{pub initial_capacity:Option<usize>,pub array_pointer_valid:Option<bool>,pub source_count_zero:Option<bool>,pub actor12c134_absent:Option<bool>,pub class_type_not11:Option<bool>}
#[derive(Clone,Debug)]pub(crate) struct Inputs{
 pub at:Option<u32>,pub packet:Option<Vec<u8>>,pub dispatcher32_branch:Option<bool>,pub registry_pointer_matches:Option<bool>,
 pub queue_count:Option<usize>,pub queue_capacity:Option<usize>,pub array_pointer_valid:Option<bool>,pub node_allocation_succeeds:Option<bool>,pub growth_allocation_succeeds:Option<bool>,pub runtime:Runtime,
}
#[derive(Clone,Debug)]pub(crate) struct Fact{pub at:u32,pub arg:u8,pub appended:Option<Node>,pub suppressed:bool,pub grew:bool,pub capacity:usize,pub nodes:Vec<Node>}
pub(crate) struct PreparedQueue{identity:Arc<Identity>,revision:u64,state:State,pub fact:Fact}
#[derive(Clone,Debug)]pub(crate) struct Remove{pub registry_actor_matches:Option<bool>,pub class68_absent:Option<bool>,pub opcode11_absent:Option<bool>,pub source_unlink_reached:Option<bool>,pub owner_all_cancel_reached:Option<bool>,pub virtual_destructor_reached:Option<bool>}
#[derive(Clone,Debug)]pub(crate) struct Retired{pub nodes:Vec<Node>,pub timers:Vec<Timer>}
#[derive(Clone,Debug)]pub(crate) struct Registration{pub at:Option<u32>,pub producer4e7a40_reached:Option<bool>,pub registry_empty_before:Option<bool>,pub allocation13c_succeeds:Option<bool>,pub constructor_remaining_initialization_succeeds:Option<bool>,pub queue16byte_allocation_succeeds:Option<bool>,pub registry_insertion_reached:Option<bool>,pub new_local_pointer_matches:Option<bool>,pub source_camera_initialized:Option<bool>,pub source_roof_initialized:Option<bool>,pub cell:Option<WorldCell>,pub heading:Option<u8>,pub runtime:Runtime}
impl Model<'_>{
 pub fn owned_timers(&self,b:&a::Binding,now:i64)->Result<Vec<Timer>>{self.active(b,now)?;Ok(self.state.events.clone())}
 pub fn prepare_registration(&self,b:&a::Binding,now:i64,q:&Registration)->Result<Prepared>{
  self.active(b,now)?;q.runtime.validate(b)?;ensure!(!self.state.started&&self.state.events.is_empty()&&self.state.action==49&&self.state.frame==0&&self.state.queue.as_ref().is_some_and(|v|v.nodes.is_empty()&&v.capacity==4),"New registration requires untouched capacity4 private constructor prefix");
  ensure!(q.producer4e7a40_reached==Some(true)&&q.registry_empty_before==Some(true)&&q.allocation13c_succeeds==Some(true)&&q.constructor_remaining_initialization_succeeds==Some(true)&&q.queue16byte_allocation_succeeds==Some(true)&&q.registry_insertion_reached==Some(true)&&q.new_local_pointer_matches==Some(true)&&q.source_camera_initialized==Some(true)&&q.source_roof_initialized==Some(true)&&q.cell==Some(self.state.cell),"New native registration/allocation/constructor/camera/roof producer unknown");
  let at=q.at.context("New registration clock unknown")?;ensure!(at>=self.state.last,"New registration clock backwards");let h=q.heading.context("New source heading unknown")?;ensure!(h<8,"New heading outside source range");let plan=a::plan(true,self.d,b,b,now,a::Kind::Walk,h,16)?;ensure!(plan.action==46&&plan.fps==24&&plan.frames[0].0.image==72&&plan.frames[0].0.subframe==0&&plan.frames[0].0.ticks==4,"New base46/frame0 draw catalog differs");let mut s=self.state.clone();s.action=46;s.frame=0;s.heading=h;self.prepared(s,at,self.state.cell,false,vec![],None)
 }
 pub fn enable_queue(&mut self,b:&a::Binding,now:i64,q:&Enable)->Result<()>{
  self.active(b,now)?;ensure!(!self.state.started&&self.state.queue.is_none(),"Queue profile requires untouched private owner");
  let cap=q.initial_capacity.context("Native initial queue capacity unknown")?;
  ensure!((1..=8).contains(&cap)&&q.array_pointer_valid==Some(true)&&q.source_count_zero==Some(true)&&q.actor12c134_absent==Some(true)&&q.class_type_not11==Some(true),"Queue pointer/count/optional effects/class branch unknown");
  self.state.queue=Some(StateQueue{nodes:vec![],capacity:cap,serial:0});self.state.left=32;self.revision+=1;Ok(())
 }
 pub fn queued(&self,b:&a::Binding,now:i64)->Result<Vec<Node>>{self.active(b,now)?;Ok(self.state.queue.as_ref().context("No qualified queue profile")?.nodes.clone())}
 pub fn queue_capacity(&self,b:&a::Binding,now:i64)->Result<usize>{self.active(b,now)?;Ok(self.state.queue.as_ref().context("No qualified queue profile")?.capacity)}
 pub fn prepare_queue(&self,b:&a::Binding,now:i64,q:&Inputs)->Result<PreparedQueue>{
  self.active(b,now)?;q.runtime.validate(b)?;let packet=q.packet.as_ref().context("Queue packet32/DC absent")?;
  ensure!(packet.len()==6&&packet[0]==32&&i64::from(u32::from_le_bytes(packet[1..5].try_into()?))==b.object&&matches!(packet[5],1|2),"Only bounded same-object packet32 args1/2 qualified; action8 excluded");
  ensure!(q.dispatcher32_branch==Some(true)&&q.registry_pointer_matches==Some(true)&&q.array_pointer_valid==Some(true),"Queue receive/registry/pointer unknown");
  ensure!(self.state.started&&self.state.action==46&&self.state.frame>0&&self.state.events.iter().all(|t|t.kind==Kind::Step),"Selected busy moving frame required");
  let at=q.at.context("Queue input clock unknown")?;ensure!(at>=self.state.last&&self.state.events.first().is_some_and(|t|at<t.at),"Input after timer or unspecified equal-clock order");
  let mut s=self.state.clone();let queue=s.queue.as_mut().context("Queue profile not enabled")?;
  ensure!(q.queue_count==Some(queue.nodes.len())&&q.queue_capacity==Some(queue.capacity),"Native queue count/capacity mismatch");
  let arg=packet[5];let suppressed=queue.nodes.len()>3&&arg==2&&queue.nodes.last().is_some_and(|n|n.arg==arg);let mut grew=false;let mut appended=None;
  if !suppressed{
   ensure!(queue.nodes.len()<8&&q.node_allocation_succeeds==Some(true),"Private bound or node allocation failure/unknown");
   if queue.nodes.len()>=queue.capacity{let cap=queue.capacity.checked_mul(2).context("Capacity overflow")?;ensure!(cap<=8&&q.growth_allocation_succeeds==Some(true),"Growth allocation unknown/failed/outside private bound");queue.capacity=cap;grew=true;}
   queue.serial+=1;let node=Node{serial:queue.serial,arg};queue.nodes.push(node.clone());appended=Some(node);
  }
  let fact=Fact{at,arg,appended,suppressed,grew,capacity:queue.capacity,nodes:queue.nodes.clone()};s.last=at;s.left-=1;
  Ok(PreparedQueue{identity:self.identity.clone(),revision:self.revision,state:s,fact})
 }
 pub fn commit_queue(&mut self,b:&a::Binding,now:i64,p:PreparedQueue)->Result<Fact>{self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&p.revision==self.revision,"Foreign/stale private queue preparation");self.state=p.state;self.revision+=1;Ok(p.fact)}
 pub fn remove_queue_owner(&mut self,b:&a::Binding,now:i64,q:&Remove)->Result<Retired>{
  self.active(b,now)?;ensure!(q.registry_actor_matches==Some(true)&&q.class68_absent==Some(true)&&q.opcode11_absent==Some(true)&&q.source_unlink_reached==Some(true)&&q.owner_all_cancel_reached==Some(true)&&q.virtual_destructor_reached==Some(true),"4E82F0 no-opcode11 removal/cancel/destructor producer unknown");
  let nodes=self.state.queue.as_ref().context("No qualified queued owner")?.nodes.clone();
  ensure!(self.state.events.iter().all(|t|matches!(t.kind,Kind::QueuedActionPop|Kind::AttackAdvance)),"Actor-owner cancel does not remove owner0 movement timers");
  let retired=Retired{nodes,timers:self.state.events.clone()};self.retire();Ok(retired)
 }
}
