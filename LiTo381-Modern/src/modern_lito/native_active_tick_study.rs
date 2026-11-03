//! Source active-tick gate and pure normal-tail preview. Never execute or
//! pretend to complete488860's native surface/list/music/provider body.
use super::*;
#[derive(Clone,Debug)]pub(crate) struct TailInputs{pub normal_tail_reached:Option<bool>,pub catalog_pointer_valid:Option<bool>,pub catalog_frame_count:Option<usize>,pub previous75a834:Option<u32>,pub fresh4d69f0:Option<u32>,pub prior75ac34:Option<u32>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) struct TailFact{pub raw_elapsed:u32,pub capped75ac18:u32,pub new75a834:u32,pub new75ac34:u32,pub flag641498:bool}
pub(crate) fn tail_preview(q:&TailInputs)->Result<TailFact>{
 ensure!(q.normal_tail_reached==Some(true)&&q.catalog_pointer_valid==Some(true),"Native normal-tail reach/catalog pointer unknown");let count=q.catalog_frame_count.context("Native catalog count unknown")?;ensure!((1..=127).contains(&count),"Signed catalog byte outside selected bound");let prior=q.previous75a834.context("Prior native render clock unknown")?;let clock=q.fresh4d69f0.context("Fresh native render clock unknown")?;let elapsed=clock.checked_sub(prior).context("Native render clock wrap/backward unqualified")?;let frames=q.prior75ac34.context("Native rendered count unknown")?.checked_add(1).context("Native rendered counter wrap unqualified")?;
 Ok(TailFact{raw_elapsed:elapsed,capped75ac18:elapsed.min(500),new75a834:clock,new75ac34:frames,flag641498:count==12&&elapsed>40})
}
#[derive(Clone,Debug)]pub(crate) struct Gate{pub at:Option<u32>,pub flag641498:Option<bool>,pub flag834eb4:Option<bool>,pub local_actor_pointer_matches:Option<bool>,pub main_loop_drained_due_callbacks:Option<bool>}
impl Control<'_>{
 pub fn active_tick_gate(&self,b:&Binding,now:i64,t:&Token,q:&Gate)->Result<&'static str>{
  self.active(b,now)?;self.token_valid(t)?;ensure!(self.handlers==Handlers::Restored&&q.local_actor_pointer_matches==Some(true)&&q.main_loop_drained_due_callbacks==Some(true),"Restored C0/local pointer/native loop unknown");let at=q.at.context("Native active tick clock unknown")?;self.before(at)?;let flag=q.flag641498.context("Native641498 producer unknown")?;let moving=q.flag834eb4.context("Native834EB4 producer unknown")?;
  ensure!(!flag&&moving,"Native488860 surface/list/music/provider body unimplemented; finite CPU movement does not execute it");Ok("SOURCE_48AF60_C0_SKIPPED_834EB4_TRUE_641498_FALSE")
 }
}
