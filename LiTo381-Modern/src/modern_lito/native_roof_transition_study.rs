//! Conditional 48AFC0 multi-group transitions over original resident pointers.
//! Timers retain callback+group identities and FIFO equal-time ordering.
use super::{Model as Resident,State,Inputs as ResidentInputs};
use crate::modern_lito::{native_appearance_inspection::Binding,native_resources::{NativeStaticAssets,NativeAssetPlan},world_map::{WorldCell,ViewProjection,PixelPoint},world_roofs::Shade,world_raster::Raster,compositor::Canvas};
use anyhow::{ensure,Context,Result};use std::{collections::BTreeSet,sync::{Arc,atomic::AtomicBool}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) enum Callback{Down,Up}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) struct Timer{pub serial:u64,pub group:usize,pub callback:Callback,pub at:u32}
#[derive(Clone)]struct Draft{states:Vec<State>,group_active:Vec<bool>,cell:WorldCell,timers:Vec<Timer>,invalid:BTreeSet<usize>,last:u32,serial:u64,left:usize}
pub(crate) struct MoveInputs{pub actor:ResidentInputs,pub fresh_schedule_samples:Option<Vec<u32>>}
pub(crate) struct TickInputs{pub timer_removed_before_callback:Option<bool>,pub callback_group_matches:Option<bool>,pub dispatch_ms:Option<u32>,pub fresh_schedule_samples:Option<Vec<u32>>}
pub(crate) struct Ticket{identity:Arc<()>,pub timer:Timer}
pub(crate) struct Prepared{identity:Arc<()>,revision:u64,draft:Draft}
pub(crate) struct Model<'a>{base:Resident<'a>,identity:Arc<()>,revision:u64,draft:Draft}
struct Samples{values:std::vec::IntoIter<u32>,minimum:u32}
impl Samples{fn new(values:Option<Vec<u32>>,minimum:u32)->Result<Self>{Ok(Self{values:values.context("Fresh clock reads unknown")?.into_iter(),minimum})}fn at(&mut self)->Result<u32>{let v=self.values.next().context("Missing source 4D65F0 fresh sample")?;ensure!(v>=self.minimum,"Backward source clock");self.minimum=v;v.checked_add(50).context("Roof timer clock wraps")?;Ok(v)}fn finish(self)->Result<()>{ensure!(self.values.len()==0,"Unused source clock reads");Ok(())}}
fn cancel(d:&mut Draft,g:usize,c:Callback){d.timers.retain(|t|t.group!=g||t.callback!=c);}
fn insert(d:&mut Draft,g:usize,c:Callback,samples:&mut Samples)->Result<()>{ensure!(d.timers.len()<64&&!d.invalid.contains(&g),"Timer bound/retired group");let sample=samples.at()?;d.last=sample;d.serial=d.serial.checked_add(1).context("Timer serial overflow")?;let t=Timer{serial:d.serial,group:g,callback:c,at:sample+50};let pos=d.timers.partition_point(|e|e.at<=t.at);d.timers.insert(pos,t);Ok(())}
impl<'a> Model<'a>{
 pub fn new(enabled:bool,b:&Binding,now:i64,assets:&'a NativeStaticAssets,plan:&NativeAssetPlan,initial:WorldCell,input:&ResidentInputs,budget:usize)->Result<Self>{ensure!(budget>0&&budget<=64,"Finite scene callback budget required");let base=Resident::new(enabled,b,b,now,assets,plan,initial,133,16,input)?;ensure!(base.triggers.values().flatten().all(|(_,kind)|*kind==0),"Only actual all-kind0 Native564 transition source qualified");let draft=Draft{states:base.states.clone(),group_active:vec![false;base.groups.len()],cell:initial,timers:Vec::new(),invalid:BTreeSet::new(),last:base.last_clock,serial:0,left:budget};Ok(Self{base,identity:Arc::new(()),revision:0,draft})}
 fn active(&self,b:&Binding,now:i64)->Result<()>{self.base.active(b,now)?;ensure!(self.draft.left>0,"Finite roof event budget exhausted");Ok(())}
 pub fn timers(&self,b:&Binding,now:i64)->Result<Vec<Timer>>{self.active(b,now)?;Ok(self.draft.timers.clone())}
 pub fn next(&self,b:&Binding,now:i64)->Result<Ticket>{self.active(b,now)?;Ok(Ticket{identity:self.identity.clone(),timer:*self.draft.timers.first().context("No source roof timer")?})}
 pub fn cell(&self,b:&Binding,now:i64)->Result<WorldCell>{self.active(b,now)?;Ok(self.draft.cell)}
 pub fn group(&self,b:&Binding,now:i64,g:usize)->Result<Vec<(usize,State)>>{self.active(b,now)?;ensure!(!self.draft.invalid.contains(&g),"Group invalidated");Ok(self.base.groups.get(g).context("Unknown source group")?.iter().map(|i|(*i,self.draft.states[*i])).collect())}
 pub fn controlled_alias_count(&self)->usize{let controlled:BTreeSet<_>=self.base.triggers.values().flatten().map(|(g,_)|*g).collect();let mut count=0;for g in controlled{for id in &self.base.groups[g]{if self.base.groups.iter().enumerate().any(|(other,v)|other!=g&&v.contains(id)){count+=1;}}}count}
 fn down(&self,d:&mut Draft,g:usize,samples:&mut Samples)->Result<()>{let mut reduce=false;let mut restore=false;d.group_active[g]=false;for id in &self.base.groups[g]{let s=&mut d.states[*id];ensure!(s.weight<=31&&s.flags&!0xd8b==0,"Unknown resident flags/weight");if s.flags&0x403==0{if s.flags&0x800==0{s.weight=if s.flags&0x100!=0{16}else{31};}s.flags=(s.flags&!0x800)|0x400;s.target=1;reduce=true;}
  else if s.flags&0x800!=0{s.flags=(s.flags&!0x800)|0x400;s.target=1;reduce=true;}
  else if s.flags&0x400!=0{s.flags&=!0x800;s.target=1;reduce=true;}
  else if s.flags&2!=0{s.flags=(s.flags&!0x400)|0x800;s.target=1;restore=true;}
  else{s.weight=0;s.flags=(s.flags&0xffff_f3fc)|1;}d.group_active[g]=true;}
  if reduce{cancel(d,g,Callback::Up);insert(d,g,Callback::Down,samples)?;}else if restore{cancel(d,g,Callback::Up);cancel(d,g,Callback::Down);insert(d,g,Callback::Up,samples)?;}Ok(())
 }
 fn up(&self,d:&mut Draft,g:usize,samples:&mut Samples)->Result<()>{let mut restore=false;for id in &self.base.groups[g]{let s=&mut d.states[*id];if s.flags&2!=0&&s.flags&0x800==0{if s.flags&0x400==0{s.weight=1;}s.target=0;s.flags=(s.flags&!0x400)|0x800;restore=true;}
  else if s.flags&1!=0&&s.flags&0x800==0{if s.flags&0x400==0{s.weight=17;}s.target=0;s.flags=(s.flags&!0x400)|0x800;restore=true;}
  else if s.flags&0x400!=0{s.target=0;s.flags=(s.flags&!0x400)|0x800;restore=true;}
  else if s.flags&0x800!=0{s.target=0;}
  else{s.flags&=0xffff_fbfc;s.weight=0;s.target=0;}}
  if restore{cancel(d,g,Callback::Down);insert(d,g,Callback::Up,samples)?;}Ok(())
 }
 pub fn prepare_move(&self,b:&Binding,now:i64,input:&MoveInputs)->Result<Prepared>{self.active(b,now)?;let q=&input.actor;ensure!(q.local_actor_pointer_matches==Some(true)&&q.old_cell==Some(self.draft.cell)&&q.no_dynamic_cell_entries==Some(true)&&q.source_initialized_tiles==Some(true),"Actor/local pointer/old cell/resident source unknown");let new=q.new_cell.context("New actor cell unknown")?;let clock=q.scheduler_sample_ms.context("Move observation clock unknown")?;ensure!(clock>=self.draft.last&&self.base.assets.geometry().contains(new),"Backward/foreign roof move");let old=self.base.records(self.draft.cell)?;let target=self.base.records(new)?;ensure!(old.iter().chain(&target).all(|(g,k)|*k==0&&!self.draft.invalid.contains(g)),"Unsupported/invalidated source record");let mut d=self.draft.clone();let mut samples=Samples::new(input.fresh_schedule_samples.clone(),clock)?;d.last=clock;
  for(g,_)in &old{d.group_active[*g]=false;}for(g,_)in target{self.down(&mut d,g,&mut samples)?;if !d.group_active[g]{self.up(&mut d,g,&mut samples)?;}}for(g,_)in old{if !d.group_active[g]{self.up(&mut d,g,&mut samples)?;}}samples.finish()?;d.cell=new;d.left-=1;Ok(Prepared{identity:self.identity.clone(),revision:self.revision,draft:d})
 }
 pub fn prepare_tick(&self,b:&Binding,now:i64,ticket:&Ticket,input:&TickInputs)->Result<Prepared>{self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&ticket.identity)&&self.draft.timers.first()==Some(&ticket.timer)&&!self.draft.invalid.contains(&ticket.timer.group)&&input.timer_removed_before_callback==Some(true)&&input.callback_group_matches==Some(true),"Stale/foreign roof callback/owner/removal");let dispatch=input.dispatch_ms.context("Dispatch loop clock unknown")?;ensure!(dispatch>=ticket.timer.at&&dispatch>=self.draft.last,"Early/backward dispatch");let mut samples=Samples::new(input.fresh_schedule_samples.clone(),dispatch)?;let mut d=self.draft.clone();let timer=d.timers.remove(0);cancel(&mut d,timer.group,timer.callback);d.last=dispatch;let mut more=false;
  for id in &self.base.groups[timer.group]{let s=&mut d.states[*id];match timer.callback{Callback::Down=>{if s.flags&0x400==0{continue;}if s.target&2!=0&&s.weight>0{if let Some(v)=s.weight.checked_sub(4){s.weight=v;more=true;continue;}}
   else if s.target&1!=0&&s.weight>15{let v=s.weight-4;if v>=17{s.weight=v;more=true;continue;}}s.weight=0;s.flags=(s.flags&0xffff_fbfc)|u32::from(s.target);},Callback::Up=>{if s.flags&0x800==0{continue;}s.weight=s.weight.checked_add(4).context("Roof weight overflow")?;let limit=if s.flags&0x100!=0||s.target&1!=0{16}else{31};if s.weight<=limit{more=true;}else{s.weight=0;s.target=0;s.flags=if limit==16{(s.flags&0xffff_f7fd)|1}else{s.flags&0xffff_f7fc};}}}}
  if more{insert(&mut d,timer.group,timer.callback,&mut samples)?;}samples.finish()?;d.left-=1;Ok(Prepared{identity:self.identity.clone(),revision:self.revision,draft:d})
 }
 pub fn commit(&mut self,b:&Binding,now:i64,prepared:Prepared)->Result<()>{self.active(b,now)?;ensure!(Arc::ptr_eq(&self.identity,&prepared.identity)&&prepared.revision==self.revision,"Foreign/stale prepared transition");self.revision=self.revision.checked_add(1).context("Roof revision overflow")?;self.draft=prepared.draft;Ok(())}
 pub fn invalidate_group(&mut self,b:&Binding,now:i64,g:usize,source_unload:Option<bool>)->Result<()>{self.active(b,now)?;ensure!(source_unload==Some(true)&&g<self.base.groups.len(),"Group unload writer unknown");self.draft.timers.retain(|t|t.group!=g);self.draft.invalid.insert(g);self.revision+=1;Ok(())}
 fn submit_draft<'r>(&'r self,b:&Binding,now:i64,d:&Draft,raster:&mut Raster<'r>)->Result<()>{self.active(b,now)?;for(i,(cell,depth,key))in self.base.draws.iter().enumerate(){let s=d.states[i];let shade=if s.flags&0xc00!=0{Shade::Weight(s.weight)}else if s.flags&1!=0||s.flags&0x100!=0{Shade::Half}else{Shade::Native};raster.tile_roof(*cell,*depth,&self.base.tiles[key],shade)?;}Ok(())}
 pub fn submit<'r>(&'r self,b:&Binding,now:i64,raster:&mut Raster<'r>)->Result<()>{self.submit_draft(b,now,&self.draft,raster)}
 pub fn submit_prepared<'r>(&'r self,b:&Binding,now:i64,p:&Prepared,raster:&mut Raster<'r>)->Result<()>{ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&p.revision==self.revision,"Foreign/stale roof paint draft");self.submit_draft(b,now,&p.draft,raster)}
 pub fn render(&self,b:&Binding,now:i64,anchor:WorldCell)->Result<Canvas>{let mut canvas=Canvas::new(512,384)?;let mut raster=Raster::new(ViewProjection{anchor_cell:anchor,anchor_pixel:PixelPoint{x:256,y:250}},&canvas);self.submit(b,now,&mut raster)?;raster.paint(&mut canvas,&AtomicBool::new(false),0)?;Ok(canvas)}
 pub fn retire(&mut self){self.base.retire();self.draft.timers.clear();self.revision+=1;}
}
impl Drop for Model<'_>{fn drop(&mut self){self.retire();}}
#[cfg(test)]#[path="native_roof_transition_study_tests.rs"]mod tests;
