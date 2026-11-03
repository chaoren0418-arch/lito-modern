//! One conditional queued idle callback, owned by a completed finite attack.
//! Every pointer/clock assumption is explicit; this is not a live actor handle.
use super::{native_appearance_inspection as a,native_appearance_clock_study as clock,native_appearance_continuation_study as continuation,native_appearance_replay_study as replay,native_appearance_layer_study as layer,native_actor_inspection::catalog_frame_ms};
use anyhow::{Result,ensure,Context};
pub(crate) struct Inputs{
 pub callback_actor_matches:Option<bool>,pub timer_removed_before_callback:Option<bool>,
 pub queue_before_pop:Option<usize>,pub actor_68_absent:Option<bool>,
 pub actor_4:Option<u8>,pub render_player_pointer_equal:Option<bool>,
 pub prior_submissions_unlinked:Option<bool>,pub post_load_actor_globals_match:Option<bool>,
 pub ordinary:clock::OrdinaryInputs,pub scheduler_sample_ms:Option<u32>,
}
#[derive(Clone,Copy)]pub(crate) struct Pending{pub bank:u32,pub subframe:u32,pub dispatch_ms:u32,pub scheduler_sample_ms:u32,pub duration_ms:u32,pub next_due_ms:u32}
pub(crate) struct Step{pub pending:Pending,pub sample:layer::Sample}
pub(crate) struct Model<'a>{binding:a::Binding,attack:replay::Replay<'a>,idle:a::InspectionPlan<'a>,layers:layer::Model,queued_due:Option<u32>,stopped:bool,completed_first:Option<Pending>}
impl<'a> Model<'a>{
 pub fn new(enabled:bool,d:&'a a::Dependencies,b:&a::Binding,expected:&a::Binding,now:i64,heading:u8,ordinary:clock::OrdinaryInputs,end:continuation::EndInputs)->Result<Self>{
  let attack=replay::Replay::new(enabled,d,b,expected,now,heading,ordinary,end)?;
  let idle=a::plan(enabled,d,b,expected,now,a::Kind::Idle,heading,128)?;
  ensure!(idle.action==49&&idle.fps==24&&idle.frames.len()==9&&idle.frames[0].0.image==80&&idle.frames[0].0.subframe==8&&idle.frames[1].0.image==80&&idle.frames[1].0.subframe==0&&idle.frames[1].0.events.is_empty(),"Audited first idle49 catalog required; event dispatch unsupported");
  Ok(Self{binding:b.clone(),attack,idle,layers:layer::Model::new(enabled,b,expected,now,1)?,queued_due:None,stopped:false,completed_first:None})
 }
 fn active(&self,b:&a::Binding,now:i64)->Result<()>{ensure!(!self.stopped&&self.binding==*b&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff),"Idle owner retired/full binding/cutoff mismatch");Ok(())}
 pub fn attack_next(&self,b:&a::Binding,now:i64)->Result<(usize,u32,u32,u32)>{self.active(b,now)?;ensure!(self.queued_due.is_none(),"Attack already queued idle");self.attack.next(b,now)}
 pub fn attack_sample(&mut self,b:&a::Binding,now:i64,sampled:u32,bank:&replay::Bank,input:layer::Inputs)->Result<replay::Frame>{
  self.active(b,now)?;ensure!(self.queued_due.is_none(),"Attack already ended");let f=self.attack.sample(b,now,sampled,bank,input)?;
  if f.ended{ensure!(f.queued_idle==Some(49),"Unexpected queued action");self.queued_due=Some(f.next_due_ms);}Ok(f)
 }
 pub fn queued_due(&self,b:&a::Binding,now:i64)->Result<u32>{self.active(b,now)?;self.queued_due.context("Idle message not created by this finite attack")}
 pub fn initial_catalog(&self)->&a::CatalogFrame{self.idle.frames[0].0}
 pub fn preview(&self,b:&a::Binding,now:i64,input:&Inputs)->Result<Pending>{
  let due=self.queued_due(b,now)?;
  ensure!(input.callback_actor_matches==Some(true)&&input.timer_removed_before_callback==Some(true)&&input.queue_before_pop==Some(1)&&input.actor_68_absent==Some(true),"Callback owner/timer/pop/free branch unknown");
  ensure!(input.actor_4==Some(0)&&input.render_player_pointer_equal==Some(false)&&input.prior_submissions_unlinked==Some(true)&&input.post_load_actor_globals_match==Some(false),"Action override/render-global/prior draw removal branches unknown");
  let o=&input.ordinary;ensure!(o.ordinary_branch_reached==Some(true)&&o.frozen_external_state==Some(true)&&i64::from(o.player_global.context("Clock player global unknown")?)!=b.object&&o.queue_after_pop==Some(0)&&o.modifier_1c==Some(0)&&o.modifier_21==Some(0)&&o.action21_counter==Some(0)&&o.catalog_scaling_disabled==Some(true),"Only explicit neutral nonplayer idle clock qualified");
  let dispatch=o.sampled_clock_ms.context("Callback clock unknown")?;let sampled=input.scheduler_sample_ms.context("4D65F0 fresh clock unknown")?;
  ensure!(dispatch>=due&&sampled>=dispatch&&dispatch-due<=i32::MAX as u32,"Early/nonmonotonic/wrapping idle clock unsupported");
  let catalog=self.idle.frames[1].0;let duration=catalog_frame_ms(u32::from(catalog.ticks),u32::from(self.idle.fps))?;
  let next=sampled.checked_add(duration.checked_add(2).context("Idle delay overflow")?).context("Idle next clock wraps")?;
  Ok(Pending{bank:catalog.image+u32::from(self.idle.heading),subframe:catalog.subframe,dispatch_ms:dispatch,scheduler_sample_ms:sampled,duration_ms:duration,next_due_ms:next})
 }
 pub fn sample(&mut self,b:&a::Binding,now:i64,bank:&replay::Bank,input:&Inputs,draw:layer::Inputs)->Result<Step>{
  let pending=self.preview(b,now,input)?;
  let list=self.layers.idle_submissions(b,now,pending.bank,pending.subframe as usize,&bank.body,&bank.shadow,&bank.layer,draw)?;
  let sample=self.layers.render(b,now,&list)?;
  // Commit only after all required inputs/resources and CPU paint succeed.
  self.queued_due=None;self.stopped=true;self.completed_first=Some(pending);Ok(Step{pending,sample})
 }
 pub fn retire(&mut self){self.attack.retire();self.layers.retire();self.queued_due=None;self.completed_first=None;self.stopped=true;}
}
impl Drop for Model<'_>{fn drop(&mut self){self.retire();}}
#[path="native_idle_cycle_study.rs"]pub(crate) mod cycle;
