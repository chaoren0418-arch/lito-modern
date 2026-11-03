//! Finite conditional 4E34C0 continuation and 4E02C0 idle-queue branch.
use super::{native_appearance_inspection as a,native_appearance_clock_study as clock,native_actor_inspection::catalog_frame_ms};
use anyhow::{Result,ensure,Context};
use std::sync::atomic::{AtomicBool,Ordering};
pub(crate) struct EndInputs{pub queue_after_end:Option<usize>,pub render_player_pointer_equal:Option<bool>,pub actor_50_counter:Option<u8>,pub base_action:Option<u8>,pub idle_table_pointer_present:Option<bool>,pub queue_append_succeeds:Option<bool>}
pub(crate) struct Model<'a>{binding:a::Binding,plan:a::InspectionPlan<'a>,frame:usize,expected:u32,scheduled:u32,last_tick:u32,stopped:bool,retired:AtomicBool}
pub(crate) struct Step<'a>{pub action:u16,pub frame:usize,pub catalog:&'a a::CatalogFrame,pub member:&'a a::Member,pub dispatch_ms:u32,pub previous_expected_ms:u32,pub returned_duration_ms:u32,pub next_delay_ms:u32,pub next_due_ms:u32,pub next_action_queued:Option<u8>,pub ended:bool,pub study_stopped:bool,pub visible_residency_qualified:bool,pub events_dispatched:bool,pub renderer_ready:bool}
impl<'a> Model<'a>{
 pub fn new(enabled:bool,d:&'a a::Dependencies,b:&a::Binding,expected:&a::Binding,now:i64,heading:u8,ordinary:clock::OrdinaryInputs,end:EndInputs)->Result<Self>{
 let first=clock::OrdinaryFirstAdvance::new(enabled,d,b,expected,now,heading,ordinary)?;let initial=first.first(expected,now)?;
 ensure!(initial.action==47&&b.class==734&&b.character_type==3&&b.sex==0&&b.retained_type==2&&b.effective_equipped,"Only audited Type3 weapon2 attack47 continuation");
 ensure!(end.queue_after_end==Some(0)&&end.render_player_pointer_equal==Some(false)&&end.actor_50_counter==Some(0)&&end.base_action==Some(46)&&end.idle_table_pointer_present==Some(true)&&end.queue_append_succeeds==Some(true),"Original end-queue/render-player/base/pointer/allocation state unknown");
 let idle=d.rows.get(&734).and_then(|r|r.actions.get(&49)).context("Idle49 table absent")?;ensure!(!idle.frames.is_empty(),"Idle49 pointer not qualified");
 let plan=a::plan(enabled,d,b,expected,now,a::Kind::Attack,heading,128)?;
 ensure!(plan.frames.len()==7,"Audited complete attack47 table required");
 Ok(Self{binding:b.clone(),plan,frame:1,expected:initial.next_due_ms,scheduled:initial.next_due_ms,last_tick:initial.next_due_ms-initial.next_delay_ms,stopped:false,retired:AtomicBool::new(false)})
 }
 fn active(&self,b:&a::Binding,now:i64)->Result<()>{ensure!(!self.retired.load(Ordering::Acquire)&&!self.stopped,"Continuation retired or bounded end reached");ensure!(self.binding==*b&&now>=b.observed&&(b.cutoff==-1||now<b.cutoff),"Continuation owner/revision/cutoff mismatch");Ok(())}
 pub fn due(&self,b:&a::Binding,now:i64)->Result<u32>{self.active(b,now)?;Ok(self.scheduled)}
 pub fn advance(&mut self,b:&a::Binding,now:i64,sampled:u32)->Result<Step<'a>>{
 self.active(b,now)?;ensure!(sampled>=self.scheduled&&sampled>=self.last_tick,"Callback early/nonmonotonic clock unqualified");let late=sampled-self.expected;ensure!(late<=i32::MAX as u32,"Original signed lateness range unqualified");
 let next=self.frame+1;let ended=next>=self.plan.frames.len();let frame=if ended{0}else{next};let(catalog,member)=self.plan.frames[frame];ensure!(catalog.subframe<u32::from(member.frames),"Actual continuation subframe bound");
 let duration=catalog_frame_ms(u32::from(catalog.ticks),u32::from(self.plan.fps))?;
 // End resets frame0 and calls 4E02C0. Under the explicit branch inputs,
 // current47 != base46+3, so queue opcode32/action49 with source delay0x248.
 let delay=if ended{584}else{duration.saturating_sub(late).max(1).checked_add(2).context("Callback delay overflow")?};
 let due=sampled.checked_add(delay).context("Original wrapping callback clock unqualified")?;let old=self.expected;
 let expected=if ended{self.expected}else{self.expected.checked_add(duration+2).context("Original expected clock wraps")?};
 self.frame=frame;self.expected=expected;self.scheduled=due;self.last_tick=sampled;self.stopped=ended;
 Ok(Step{action:self.plan.action,frame,catalog,member,dispatch_ms:sampled,previous_expected_ms:old,returned_duration_ms:duration,next_delay_ms:delay,next_due_ms:due,next_action_queued:if ended{Some(49)}else{None},ended,study_stopped:ended,visible_residency_qualified:false,events_dispatched:false,renderer_ready:false})
 }
 pub fn retire(&self){self.retired.store(true,Ordering::Release);}
}
impl Drop for Model<'_>{fn drop(&mut self){self.retire();}}
