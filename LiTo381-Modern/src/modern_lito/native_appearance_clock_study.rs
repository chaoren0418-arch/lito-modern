//! Default-off, single catalog interval inspection. No runtime event/layer grant.
use super::{native_appearance_inspection as a,native_actor_inspection::{catalog_frame_ms,relative_frame}};
use anyhow::{Result,ensure,Context};
use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
struct Identity{binding:a::Binding,retired:AtomicBool}
pub(crate) struct Study<'a>{identity:Arc<Identity>,plan:a::InspectionPlan<'a>,durations:Vec<u32>,total:u64}
pub(crate) struct Sample<'a>{identity:Arc<Identity>,plan:&'a a::InspectionPlan<'a>,frame:usize,elapsed:u64,start:u64,end:u64}
pub(crate) struct View<'a>{pub graphics:u32,pub action:u16,pub heading:u8,pub fps:u8,pub frame:usize,pub elapsed_ms:u64,pub start_ms:u64,pub end_ms:u64,pub catalog:&'a a::CatalogFrame,pub member:&'a a::Member,pub renderer_ready:bool,pub runtime_qualified:bool,pub game_wire_profile:u32}
impl<'a> Study<'a>{
 pub fn new(enabled:bool,d:&'a a::Dependencies,binding:&a::Binding,expected:&a::Binding,now:i64,kind:a::Kind,heading:u8,budget:usize)->Result<Self>{
 let plan=a::plan(enabled,d,binding,expected,now,kind,heading,budget)?;ensure!(!plan.renderer_ready&&!plan.runtime_qualified&&plan.game_wire_profile==0,"Offline inspection scope only");
 ensure!(!plan.frames.is_empty()&&plan.frames.len()<=128,"Single interval frame bound");let durations=plan.frames.iter().map(|(f,_)|catalog_frame_ms(u32::from(f.ticks),u32::from(plan.fps))).collect::<Result<Vec<_>>>()?;
 let total=durations.iter().map(|d|u64::from(*d)).sum::<u64>();ensure!(total>0&&total<=60_000,"Finite offline study time budget");Ok(Self{identity:Arc::new(Identity{binding:binding.clone(),retired:AtomicBool::new(false)}),plan,durations,total})
 }
 fn active(&self,expected:&a::Binding,now:i64)->Result<()>{self.identity.active(expected,now)}
 pub fn duration_ms(&self)->&[u32]{&self.durations}
 pub fn total_ms(&self)->u64{self.total}
 pub fn sample(&'a self,expected:&a::Binding,now:i64,elapsed:u64)->Result<Sample<'a>>{self.active(expected,now)?;ensure!(elapsed<self.total,"Natural loop/start phase beyond explicit interval unqualified");let frame=relative_frame(&self.durations,elapsed)?;let start=self.durations[..frame].iter().map(|v|u64::from(*v)).sum();let end=start+u64::from(self.durations[frame]);Ok(Sample{identity:self.identity.clone(),plan:&self.plan,frame,elapsed,start,end})}
 pub fn retire(&self){self.identity.retired.store(true,Ordering::Release);}
}
impl Drop for Study<'_>{fn drop(&mut self){self.retire();}}
impl Identity{fn active(&self,expected:&a::Binding,now:i64)->Result<()>{ensure!(!self.retired.load(Ordering::Acquire),"Appearance study retired");ensure!(self.binding==*expected&&now>=self.binding.observed&&(self.binding.cutoff==-1||now<self.binding.cutoff),"Study full binding/revision/lifetime mismatch");Ok(())}}
impl<'a> Sample<'a>{pub fn view(&self,study:&Study<'a>,expected:&a::Binding,now:i64)->Result<View<'a>>{ensure!(Arc::ptr_eq(&self.identity,&study.identity),"Foreign clock-study identity");self.identity.active(expected,now)?;let(catalog,member)=self.plan.frames.get(self.frame).copied().context("Catalog frame absent")?;ensure!(catalog.subframe<u32::from(member.frames),"Actual SPR member subframe bound");Ok(View{graphics:self.plan.graphics,action:self.plan.action,heading:self.plan.heading,fps:self.plan.fps,frame:self.frame,elapsed_ms:self.elapsed,start_ms:self.start,end_ms:self.end,catalog,member,renderer_ready:false,runtime_qualified:false,game_wire_profile:0})}}

/// Explicit conditional state at the ordinary 4E45F0 -> 4E5B50 branch.
/// These are study inputs, never observations of the original running client.
pub(crate) struct OrdinaryInputs {
 pub ordinary_branch_reached:Option<bool>,pub player_global:Option<u32>,
 pub queue_after_pop:Option<usize>,pub modifier_1c:Option<i8>,pub modifier_21:Option<i8>,
 pub action21_counter:Option<i32>,pub catalog_scaling_disabled:Option<bool>,
 pub frozen_external_state:Option<bool>,pub sampled_clock_ms:Option<u32>,
}
pub(crate) struct OrdinaryFirstAdvance<'a>{identity:Arc<Identity>,plan:a::InspectionPlan<'a>,anchor:u32}
pub(crate) struct FirstAdvanceView<'a>{
 pub action:u16,pub frame:usize,pub initial_catalog:&'a a::CatalogFrame,
 pub selected_catalog:&'a a::CatalogFrame,pub selected_member:&'a a::Member,
 pub returned_duration_ms:u32,pub next_delay_ms:u32,pub next_due_ms:u32,
 pub frame0_visible_residency_qualified:bool,pub events_dispatched:bool,pub renderer_ready:bool,
}
impl<'a> OrdinaryFirstAdvance<'a>{
 pub fn new(enabled:bool,d:&'a a::Dependencies,binding:&a::Binding,expected:&a::Binding,now:i64,heading:u8,inputs:OrdinaryInputs)->Result<Self>{
 let plan=a::plan(enabled,d,binding,expected,now,a::Kind::Attack,heading,128)?;
 ensure!(matches!(plan.action,1|51|47)&&plan.frames.len()>1&&plan.frames.len()<=128,"Audited ordinary attack table required");
 ensure!(inputs.ordinary_branch_reached==Some(true)&&inputs.frozen_external_state==Some(true),"Original ordinary branch/frozen state unknown");
 ensure!(i64::from(inputs.player_global.context("Original player global unknown")?)!=binding.object,"Player clock branch unqualified");
 ensure!(inputs.queue_after_pop==Some(0)&&inputs.modifier_1c==Some(0)&&inputs.modifier_21==Some(0)&&inputs.action21_counter==Some(0)&&inputs.catalog_scaling_disabled==Some(true),"Only explicit neutral nonplayer clock qualified");
 let anchor=inputs.sampled_clock_ms.context("Explicit sampled clock required")?;
 Ok(Self{identity:Arc::new(Identity{binding:binding.clone(),retired:AtomicBool::new(false)}),plan,anchor})
 }
 pub fn first(&self,expected:&a::Binding,now:i64)->Result<FirstAdvanceView<'_>>{
 self.identity.active(expected,now)?;
 // 4E45F0 resets +13 to zero; 4E5B50 increments before any table access.
 // This is independent of frame0 tick value, not a generic zero-duration skip.
 let initial=self.plan.frames[0].0;let(selected,member)=self.plan.frames[1];
 ensure!(selected.subframe<u32::from(member.frames),"Actual first-advance member subframe bound");
 let duration=catalog_frame_ms(u32::from(selected.ticks),u32::from(self.plan.fps))?;
 let delay=duration.checked_add(2).context("Original delay overflow")?;
 let due=self.anchor.checked_add(delay).context("Original wrapping clock unqualified")?;
 Ok(FirstAdvanceView{action:self.plan.action,frame:1,initial_catalog:initial,selected_catalog:selected,selected_member:member,returned_duration_ms:duration,next_delay_ms:delay,next_due_ms:due,frame0_visible_residency_qualified:false,events_dispatched:false,renderer_ready:false})
 }
 pub fn retire(&self){self.identity.retired.store(true,Ordering::Release);}
}
impl Drop for OrdinaryFirstAdvance<'_>{fn drop(&mut self){self.retire();}}
