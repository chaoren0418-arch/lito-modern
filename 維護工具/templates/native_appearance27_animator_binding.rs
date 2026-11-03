#[derive(Clone,Default)]
struct AppearanceHistory {context:Option<(u64,u64,u16)>,source_revision:u64,retired:bool,actors:BTreeMap<u32,super::native_appearance27::Presentation>}
impl AppearanceHistory {
 fn retire(&mut self){self.retired=true;for p in self.actors.values_mut(){p.retire();}}
 fn merge_late(&mut self,other:&Self)->Result<()>{
  if self.context!=other.context{return Ok(());}self.source_revision=self.source_revision.max(other.source_revision);
  for(id,p)in &other.actors{if let Some(current)=self.actors.get_mut(id){if current.owner.incarnation==p.owner.incarnation{current.merge_late(p)?;}}else{let mut metadata=p.clone();metadata.retire();self.actors.insert(*id,metadata);}}
  Ok(())
 }
}
impl History {
 pub(crate) fn for_source(&self,same_source:bool)->Self{let mut h=self.clone();if !same_source{h.appearance.retire();}h}
 pub(crate) fn merge_late(&mut self,other:&Self){let mut keep=self.appearance.clone();if keep.merge_late(&other.appearance).is_err(){keep.retire();}*self=other.clone();self.appearance=keep;}
}
impl Animator {
 pub(crate) fn select_appearance_context(&mut self,connection:u64,generation:u64,map:u16){let key=Some((connection,generation,map));if self.appearance.context.is_some()&&self.appearance.context!=key{self.appearance=AppearanceHistory::default();}self.appearance.context=key;}
 pub fn pose(&mut self,catalog:&SpriteActionCatalog,actor:&Actor,now:u64)->Result<Pose>{
  use super::native_appearance27::{Owner,Presentation,Dispatch};
  let Some(tape)=&actor.appearance else{self.appearance.actors.remove(&actor.id);return self.pose_body(catalog,actor,now);};
  let(connection,generation,map)=self.appearance.context.context("S27 requires a selected render owner")?;ensure!(connection!=0,"S27 render epoch absent");
  let mut state=match self.appearance.actors.remove(&actor.id){Some(p)if p.owner.incarnation==actor.incarnation=>p,_=>Presentation::new(Owner{connection,generation,map,source_revision:self.appearance.source_revision,actor:actor.id,incarnation:actor.incarnation},tape,self.combat.seen_for(actor))?};
  let result=(||->Result<Pose>{
   if state.needs_rebase{let floor=state.admission_floor.max(state.queue.last_admitted()).max(self.combat.seen_for(actor));let mut snapshot=tape.snapshot_at(floor)?;snapshot.motion.from=None;snapshot.motion.revision=floor;let mut owner=state.owner.clone();owner.source_revision=self.appearance.source_revision.max(owner.source_revision.checked_add(1).context("body source revision")?);state.queue.rebase(owner.clone(),snapshot.body,floor)?;state.owner=owner;state.current=snapshot;state.ceiling=floor;state.needs_rebase=false;state.deferred=false;state.callback_due=None;state.released.clear();self.combat.clear_body(actor,floor,now);self.tracks.remove(&actor.id);}
   state.ingest(tape)?;let mut all_events=Vec::new();let mut result=None;
   for _ in 0..=16 {
    let view=state.view(actor)?;let mut pose=self.pose_body(catalog,&view,now)?;all_events.append(&mut pose.entered_events);
    let busy=self.combat.body_busy(&view)||(pose.phase=="walk"&&!pose.action_complete)||(pose.phase=="death");
    if busy {if state.queue.pending()>0{state.deferred=true;}result=Some(pose);break;}
    if state.callback_due.is_some_and(|due|now/1_000_000<due){result=Some(pose);break;}
    state.callback_due=None;let Some(step)=state.queue.dispatch(&state.owner,false)?else{result=Some(pose);break;};
    let queued=state.deferred;let reschedules=matches!(step,Dispatch::Appearance{..}|Dispatch::Heading{..});state.apply(step);state.deferred=false;
    if queued&&reschedules&&state.queue.pending()>0 {
     // Original tag39/tag28 pass0x248 to4E02C0, then4D65F0.
     // This follows the existing integer-ms render clock; native import/time
     // initialization and exact cross-packet arrival skew remain unqualified.
     state.callback_due=Some((now/1_000_000).checked_add(584).context("body callback deadline")?);
     let view=state.view(actor)?;let mut next=self.pose_body(catalog,&view,now)?;all_events.append(&mut next.entered_events);result=Some(next);break;
    }
   }
   let mut pose=result.context("body operation dispatch budget")?;pose.entered_events=all_events;pose.body_pending=state.queue.pending();pose.body_callback_millis=state.callback_due;Ok(pose)
  })();
  self.appearance.actors.insert(actor.id,state);result
 }
}
