//! Default-off, bounded conditional source study. No OS clock, initial phase,
//! player globals, scene object allocation, repeat policy or LOADED inference.
use super::{native_canonical_draft::RendererJob,native_resources::NativeAssetPlan,
    battle_payload::data::UnvalidatedData,native_action_inspection::{NativeActionSnapshot,NativeActionSlice},
    native_actor_dependency_draft::VerifiedActorBody,native_actor_inspection::{BodyAction,NativeActorSlice}};
use anyhow::{ensure,bail,Context,Result};
use std::{collections::VecDeque,sync::{Arc,atomic::{AtomicBool,Ordering}}};

#[derive(Clone,Copy,Debug)]
pub enum InitialEvent { GeneralIdleTransition, ExplicitOpcode32 { delay_ms:u32 } }
/// Every option is required. Explicit values describe a conditional local
/// experiment; this type never claims a runtime observation of original BIN.
#[derive(Clone,Debug)]
pub struct ConditionalInputs {
    pub object_id:Option<u32>,pub player_global:Option<u32>,pub anchor_ms:Option<u32>,
    pub queue:Option<Vec<u8>>,pub current_action:Option<u8>,pub base_action:Option<u8>,pub frame:Option<u8>,
    pub modifier_1c:Option<i8>,pub modifier_21:Option<i8>,pub action21_counter:Option<i32>,
    pub ordinary_action_path:Option<bool>,pub idle_guards_clear:Option<bool>,pub initial:Option<InitialEvent>,
    pub frozen_external_state_study:Option<bool>,
}
#[derive(Clone,Debug)]
struct Qualified {
    object:u32,anchor:u32,queue:VecDeque<u8>,current:u8,base:u8,frame:usize,m1:i8,m2:i8,counter:i32,initial:InitialEvent,
}
impl ConditionalInputs {
    fn qualify(self)->Result<Qualified>{
        let object=self.object_id.context("Original objectID unknown")?;
        let player=self.player_global.context("Original player global unknown")?;
        ensure!(object!=player,"Player-global branch not qualified");
        let queue=self.queue.context("Original initial queue unknown")?;
        ensure!(queue.len()<=32&&queue.iter().all(|a|matches!(a,0|3)),"Unknown queued opcode/action");
        ensure!(self.ordinary_action_path==Some(true),"Special actor/action path unqualified");
        ensure!(self.idle_guards_clear==Some(true),"Original idle guard selection unqualified");
        ensure!(self.frozen_external_state_study==Some(true),"External queue/modifier writes not qualified for this bounded study");
        let current=self.current_action.context("Initial action unknown")?;
        let base=self.base_action.context("Base action unknown")?;
        let frame=usize::from(self.frame.context("Initial frame unknown")?);
        ensure!(matches!(current,0|3)&&base==0&&frame==0,"Only explicit base0 ordinary initial frame0");
        Ok(Qualified{object,anchor:self.anchor_ms.context("Initial relative clock anchor unknown")?,queue:queue.into(),current,base,frame,
            m1:self.modifier_1c.context("Modifier1C unknown")?,m2:self.modifier_21.context("Modifier21 unknown")?,
            counter:self.action21_counter.context("Action21 counter unknown")?,initial:self.initial.context("Initial event unknown")?})
    }
}
const MOVEMENT_ACTIONS:[u8;11]=[0,4,11,20,24,40,46,50,54,58,62];
/// Literal 4E8750 nonplayer order. Later queue4/8/16 branches are unreachable
/// in the fixed CFG, so >=2 always divides by2. Signed Rust division matches
/// x86 truncate toward zero; bounded positive input avoids arithmetic wrap.
pub fn nonplayer_duration(catalog_ms:u32,action:u8,queue:usize,m1:i8,m2:i8,counter:i32)->Result<u32>{
    ensure!((1..=32767).contains(&catalog_ms)&&queue<=32,"Duration scalar/queue overbound");
    let mut n=i32::try_from(catalog_ms)?;
    if queue>=2{n/=2;}
    if counter>0&&action==21{n=(n+2)/3;}
    match m1{1=>n=n*3/4,2=>n*=2,17 if MOVEMENT_ACTIONS.contains(&action)=>n=n*3/4,18 if MOVEMENT_ACTIONS.contains(&action)=>n*=2,_=>{}}
    match m2{1=>n=n*3/4,3=>n=n*7/8,4 if MOVEMENT_ACTIONS.contains(&action)=>n=n*3/4,5=>n=n*3/8,_=>{}}
    Ok(u32::try_from(n)?)
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Callback { Opcode32, FrameAdvance }
#[derive(Clone,Copy,Debug)]
struct Due {at:u32,callback:Callback}
#[derive(Default)]
struct TimerQueue {items:Vec<Due>}
impl TimerQueue {
    fn insert(&mut self,at:u32,callback:Callback)->Result<()>{
        ensure!(self.items.len()<64,"Native timer queue overbound");
        // 4D65F0/66F0 scan backwards until existing time <= new time.
        // Equal-time items are inserted after existing items (stable FIFO).
        let pos=self.items.partition_point(|e|e.at<=at);
        self.items.insert(pos,Due{at,callback});Ok(())
    }
    fn take_due(&mut self, sampled_tick:u32)->Option<Due>{
        (self.items.first().is_some_and(|e|e.at<=sampled_tick)).then(||self.items.remove(0))
    }
}
#[derive(Clone,Debug,serde::Serialize)]
pub struct Step {
    pub sequence:usize,pub due_ms:u32,pub dispatch_ms:u32,pub object_id:u32,pub action:u8,pub frame:usize,
    pub state:&'static str,pub actor_queue_after_pop:usize,pub timer_queue_after_removal:usize,
    pub returned_duration_ms:u32,pub next_due_ms:Option<u32>,
}
struct Binding {cancel:Arc<AtomicBool>,local_cancel:Arc<AtomicBool>,data:UnvalidatedData,plan:NativeAssetPlan,durations:Vec<u32>}
impl Binding {
    fn active(&self)->Result<()>{ensure!(!self.cancel.load(Ordering::Acquire)&&!self.local_cancel.load(Ordering::Acquire),"Native event owner cancelled/retired");Ok(())}
    fn matches(&self,job:&RendererJob)->bool{Arc::ptr_eq(&self.cancel,&job.cancellation_token())&&self.data==*job.admitted_observation().data()&&self.plan==*job.source_plan()}
}
/// Private binding prevents moving event-selected pixels across worker owner,
/// ticket/fullDATA or retirement. Event timestamps are not asset elapsed time.
pub struct BoundEvent {binding:Arc<Binding>,step:Step}
impl BoundEvent {
    pub fn step(&self)->&Step{&self.step}
    pub fn render_firepot(&self,job:&RendererJob,snapshot:&NativeActionSnapshot)->Result<NativeActionSlice>{
        self.binding.active()?;ensure!(self.binding.matches(job)&&self.step.action==3,"Event worker/action mismatch");
        ensure!(snapshot.duration_ms()?==self.binding.durations,"Event/source duration table mismatch");
        let slice=snapshot.render_frame_index(self.step.frame,&self.binding.cancel)?;
        ensure!(slice.verified_plan()==&self.binding.plan,"Event source plan mismatch");Ok(slice)
    }
    pub fn render_body(&self,job:&RendererJob,body:&VerifiedActorBody)->Result<NativeActorSlice>{
        self.binding.active()?;ensure!(self.binding.matches(job),"Event body worker mismatch");
        ensure!(i64::from(self.step.object_id)==self.binding.data.personal.player.object,"Event objectID differs from selected body owner");
        let action=match self.step.action{0=>BodyAction::Walk,3=>BodyAction::Idle,_=>bail!("Event body action unqualified")};
        ensure!(body.duration_ms(action).map_err(|_|anyhow::anyhow!("Event body clock source absent"))?==self.binding.durations,"Event/body duration table mismatch");
        let slice=body.render_frame_index(action,self.step.frame)?;
        ensure!(slice.matches_job(job),"Event body assets belong to another worker");Ok(slice)
    }
}
struct Session {binding:Arc<Binding>,q:Qualified,durations:Vec<u32>,timers:TimerQueue,last_tick:u32,expected:u32,steps:usize,limit:usize,end:bool}
#[derive(Default)]
pub struct EventModel {session:Option<Session>}
impl EventModel {
    /// Audited action3 FirePot11x83 or prince Idle8x166. Caller obtains the
    /// table from decoded/pinned assets. No implicit table/FPS/clock default.
    pub fn local_experiment(job:&RendererJob,inputs:ConditionalInputs,durations:Vec<u32>,max_steps:usize)->Result<Self>{
        ensure!(max_steps>0&&max_steps<=64,"Explicit event budget missing/overbound");
        ensure!((durations.len()==11&&durations.iter().all(|d|*d==83))||(durations.len()==8&&durations.iter().all(|d|*d==166)),"Audited Native action3 durations required");
        ensure!(!job.cancellation().load(Ordering::Acquire),"Event worker cancelled");
        let mut q=inputs.qualify()?;let mut timers=TimerQueue::default();
        let delay=match q.initial{
            InitialEvent::GeneralIdleTransition=>{ensure!(q.queue.is_empty(),"Idle transition requires explicit empty queue");q.queue.push_back(q.base+3);if q.current==q.base+3{nonplayer_duration(durations[0],3,0,q.m1,q.m2,q.counter)?}else{584}},
            InitialEvent::ExplicitOpcode32{delay_ms}=>{ensure!(!q.queue.is_empty()&&q.queue[0]==3,"Only explicitly queued audited action3 supported");delay_ms}
        };
        let at=q.anchor.checked_add(delay).context("Native clock wrapping unqualified")?;
        timers.insert(at,Callback::Opcode32)?;
        let binding=Arc::new(Binding{cancel:job.cancellation_token(),local_cancel:Arc::new(AtomicBool::new(false)),data:job.admitted_observation().data().clone(),plan:job.source_plan().clone(),durations:durations.clone()});
        Ok(Self{session:Some(Session{binding,q,last_tick:at.saturating_sub(delay),durations,timers,expected:at,steps:0,limit:max_steps,end:false})})
    }
    pub fn next_due(&self)->Result<Option<u32>>{let s=self.session.as_ref().context("Native event model disabled")?;s.binding.active()?;Ok(s.timers.items.first().map(|e|e.at))}
    /// 4D6570 samples once per dispatch. Remove each due timer before its
    /// callback. Future callbacks use the explicit sampled clock value.
    pub fn dispatch_tick(&mut self,sampled_tick:u32)->Result<Vec<BoundEvent>>{
        let s=self.session.as_mut().context("Native event model disabled")?;
        if s.binding.active().is_err(){s.timers.items.clear();s.end=true;bail!("Native event cancellation drained timers");}
        ensure!(sampled_tick>=s.last_tick,"Nonmonotonic/wrapping clock unqualified");s.last_tick=sampled_tick;
        ensure!(sampled_tick-s.q.anchor<=i32::MAX as u32,"Signed original clock-delta range unqualified");
        let mut output=Vec::new();
        while let Some(due)=s.timers.take_due(sampled_tick){
            if s.steps>=s.limit{s.timers.items.clear();s.binding.local_cancel.store(true,Ordering::Release);bail!("Native event budget exhausted; continuation refused");}
            let removed=s.timers.items.len();s.steps+=1;
            match due.callback{Callback::Opcode32=>{s.q.current=s.q.queue.pop_front().context("Opcode32 actor queue absent")?;ensure!(s.q.current==3,"Unknown queued action callback");s.q.frame=0;},Callback::FrameAdvance=>{}}
            s.q.frame+=1;
            let ended=s.q.frame>=s.durations.len();if ended{s.q.frame=0;s.end=true;}
            let returned=nonplayer_duration(s.durations[s.q.frame],s.q.current,s.q.queue.len(),s.q.m1,s.q.m2,s.q.counter)?;
            let next=if ended{None}else{
                let delay=if due.callback==Callback::Opcode32{returned.checked_add(2).context("Duration overflow")?}else{
                    let lateness=sampled_tick.checked_sub(s.expected).context("Clock before expected frame deadline")?;
                    returned.saturating_sub(lateness).max(1).checked_add(2).context("Duration overflow")?
                };
                let at=sampled_tick.checked_add(delay).context("Native clock wrapping unqualified")?;
                s.expected=if due.callback==Callback::Opcode32{at}else{s.expected.checked_add(returned+2).context("Expected clock wrapping unqualified")?};
                s.timers.insert(at,Callback::FrameAdvance)?;Some(at)
            };
            let step=Step{sequence:s.steps,due_ms:due.at,dispatch_ms:sampled_tick,object_id:s.q.object,action:s.q.current,frame:s.q.frame,
                state:if ended{"FIRST_ANIMATION_END_CONTINUATION_UNQUALIFIED"}else if due.callback==Callback::Opcode32{"OPCODE32_ACTION_FRAME_ADVANCED"}else{"FRAME_ADVANCED"},actor_queue_after_pop:s.q.queue.len(),timer_queue_after_removal:removed,returned_duration_ms:returned,next_due_ms:next};
            output.push(BoundEvent{binding:s.binding.clone(),step});
        }Ok(output)
    }
    pub fn retire(&mut self){if let Some(s)=self.session.as_mut(){s.binding.local_cancel.store(true,Ordering::Release);s.timers.items.clear();s.end=true;}}
    pub fn loaded_admission_available(&self)->bool{false}
}
impl Drop for EventModel {fn drop(&mut self){self.retire();}}
#[cfg(test)]
#[path="native_event_scheduler_tests.rs"]
mod tests;
