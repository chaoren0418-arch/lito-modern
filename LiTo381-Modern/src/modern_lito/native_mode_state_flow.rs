//! Persistent source subset: BSS zero context ->206 handler ->phase2.
//! Phase6 callback arrival and dispatch-word producers are not connected.
//! There is deliberately no caller-supplied phase/dispatch setter per packet.
use super::{ModeSelection,Outcome,SnapshotOutcome,snapshot::{Snapshot,Kind,Context}};
use std::sync::Arc;
#[derive(Clone,Debug,Default)]pub struct Flow {phase:u32,dispatch_word:u32,mode:ModeSelection,snapshot:Option<Arc<Snapshot>>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum PacketKind {State206,State213,Delta213}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Transition {pub kind:PacketKind,pub applied:bool,pub clear_map_list:bool,pub phase_before:u32,pub phase_after:u32,pub mode:Outcome}
pub struct Prepared {pub(crate) next:Flow,pub transition:Transition}
impl Flow {
 ///Source46B670 writes64BBD4=5 afterc15;retainedmodeglobals are not reset.
 ///This is action-owned, not a caller-supplied phase setter.
 pub(crate) fn returned_to_login(&self)->Self{let mut next=self.clone();next.phase=5;next}
 ///S_C9 success ->46BA10+22 writes64BBD4=3; does not write64AF98/64ACCA.
 pub(crate) fn lobby_reply_received(&self)->Self{let mut next=self.clone();next.phase=3;next}
 ///46EB30/46ED50 named producer writes64BBD4=4 before its transfer sequence.
 pub(crate) fn transfer_started(&self)->Self{let mut next=self.clone();next.phase=4;next}
 ///46ED50 retains transfer phase and installs dispatch62C9C4=3.
 pub(crate) fn transfer_connected(&self)->Self{let mut next=self.clone();next.dispatch_word=3;next}
 ///C8/0 AFBC branch ->496A00 writes64BBD4=2 then497020 page40.
 pub(crate) fn channel_reply_received(&self)->Self{let mut next=self.clone();next.phase=2;next}
 pub fn phase(&self)->u32{self.phase}pub fn dispatch_word(&self)->u32{self.dispatch_word}
 pub fn mode(&self)->ModeSelection{self.mode}pub fn snapshot(&self)->Option<Arc<Snapshot>>{self.snapshot.clone()}
 /// Stage without mutating the current flow. Runtime can validate its own
 /// generation advance before committing either state or presentation.
 pub fn prepare_packet(&self,packet:&[u8])->anyhow::Result<Prepared>{
  let mut next=self.clone();let phase_before=self.phase;
  if packet.starts_with(&[213,2]){
   let mode=next.mode.apply(packet,self.phase)?;let applied=matches!(mode,Outcome::Applied{..});
   return Ok(Prepared{next,transition:Transition{kind:PacketKind::Delta213,applied,clear_map_list:false,phase_before,phase_after:self.phase,mode}});
  }
  let snapshot=Snapshot::decode(packet)?;
  let context=Context{phase:self.phase,dispatch_word:self.dispatch_word};
  let outcome=next.mode.apply_snapshot(&snapshot,context)?;
  let kind=match snapshot.kind(){Kind::State206=>PacketKind::State206,Kind::State213=>PacketKind::State213};
  let (applied,mode)=match outcome {
   SnapshotOutcome::IgnoredIdentifier=>(false,Outcome::IgnoredIdentifier),
   SnapshotOutcome::Applied{mode,native_phase_after}=>{next.phase=native_phase_after;next.snapshot=Some(Arc::new(snapshot));(true,mode)}
  };
  let phase_after=next.phase;
  Ok(Prepared{next,transition:Transition{kind,applied,clear_map_list:applied&&kind==PacketKind::State206,phase_before,phase_after,mode}})
 }
}
// Keep dispatch validation in the known-layout commit. Unsupported packets
// and unknown callback/connection producers return errors;they never become
// an invented phase or dispatch-word transition in this persistent subset.
