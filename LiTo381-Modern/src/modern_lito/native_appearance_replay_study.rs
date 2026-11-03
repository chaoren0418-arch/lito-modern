//! One finite conditional ordinary attack47, including bank88->bank72 end0.
use super::{native_appearance_inspection as a,native_appearance_clock_study as clock,native_appearance_continuation_study as continuation,native_appearance_layer_study as layers};
use anyhow::{Result,ensure,Context};
pub(crate) struct Bank{pub body:layers::Asset,pub shadow:layers::Asset,pub layer:layers::Asset}
pub(crate) struct Frame{pub catalog_frame:usize,pub bank:u32,pub subframe:u32,pub dispatch_ms:u32,pub next_due_ms:u32,pub ended:bool,pub queued_idle:Option<u8>,pub delay_ms:u32,pub sample:layers::Sample}
pub(crate) struct Replay<'a>{plan:a::InspectionPlan<'a>,clock:continuation::Model<'a>,layers:layers::Model,first:bool,index:usize,anchor:u32,stopped:bool}
impl<'a> Replay<'a>{
 pub fn new(enabled:bool,d:&'a a::Dependencies,b:&a::Binding,expected:&a::Binding,now:i64,heading:u8,input:clock::OrdinaryInputs,end:continuation::EndInputs)->Result<Self>{
  let anchor=input.sampled_clock_ms.context("Initial explicit clock unknown")?;
  let clock=continuation::Model::new(enabled,d,b,expected,now,heading,input,end)?;let layers=layers::Model::new(enabled,b,expected,now,7)?;
  let plan=a::plan(enabled,d,b,expected,now,a::Kind::Attack,heading,128)?;
  ensure!(plan.action==47&&plan.frames.len()==7&&plan.frames[0].0.image==72&&plan.frames[1..].iter().all(|(f,_)|f.image==88),"Complete audited attack47 bank switch required");
  Ok(Self{plan,clock,layers,first:true,index:1,anchor,stopped:false})
 }
 pub fn next(&self,b:&a::Binding,now:i64)->Result<(usize,u32,u32,u32)>{
  ensure!(!self.stopped,"Finite replay ended; idle callback unqualified");let due=self.clock.due(b,now)?;let index=if self.first{1}else if self.index+1==7{0}else{self.index+1};let catalog=self.plan.frames[index].0;
  Ok((index,catalog.image+u32::from(self.plan.heading),catalog.subframe,if self.first{self.anchor}else{due}))
 }
 pub fn sample(&mut self,b:&a::Binding,now:i64,sampled:u32,bank:&Bank,input:layers::Inputs)->Result<Frame>{
  let(index,number,subframe,due)=self.next(b,now)?;
  // Validate all three pointers, flags, resources and subframes before clock
  // mutation. Failed preflight leaves the same pending callback available.
  let list=self.layers.submissions(b,now,number,subframe as usize,&bank.body,&bank.shadow,&bank.layer,input)?;
  let(ended,queued_idle,delay_ms,next_due_ms)=if self.first{ensure!(sampled==due,"Initial conditional clock differs");let next=self.clock.due(b,now)?;(false,None,next-self.anchor,next)}else{let s=self.clock.advance(b,now,sampled)?;ensure!(s.frame==index&&s.catalog.subframe==subframe,"Clock/layer catalog sequence differs");(s.ended,s.next_action_queued,s.next_delay_ms,s.next_due_ms)};
  let sample=self.layers.render(b,now,&list)?;self.first=false;self.index=index;self.stopped=ended;
  Ok(Frame{catalog_frame:index,bank:number,subframe,dispatch_ms:sampled,next_due_ms,ended,queued_idle,delay_ms,sample})
 }
 pub fn retire(&mut self){self.clock.retire();self.layers.retire();self.stopped=true;}
}
impl Drop for Replay<'_>{fn drop(&mut self){self.retire();}}
