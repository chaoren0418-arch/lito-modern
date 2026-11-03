//! Scoped offline decoding of opcode 213/subtype 2 from 4A9E50.
//! The native parser writes a single byte at word64ACCA. The nested snapshot
//! reader covers two complete known layouts;live network dispatch is outside
//! this adapter. Strict bounds are Modern's inspection policy.
#[path="native_mode_snapshot.rs"]pub mod snapshot;
#[path="native_mode_state_flow.rs"]pub mod state_flow;
use snapshot::{Snapshot,Kind,Context,SnapshotError};
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct ModeSelection {raw_mode:u16,identifier:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ModeError {Length{actual:usize},Opcode{actual:u8},Subtype{actual:u8},UnqualifiedMode{actual:u16}}
impl std::fmt::Display for ModeError {
 fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"native mode inspection refused: {self:?}")}
}
impl std::error::Error for ModeError {}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Outcome {IgnoredIdentifier,Applied{previous:u16,current:u16,identifier_rebound:bool}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SnapshotOutcome {IgnoredIdentifier,Applied{mode:Outcome,native_phase_after:u32}}
impl ModeSelection {
 /// Explicit already-known context; this does not decode a full-state packet.
 pub fn from_known_state(raw_mode:u16,identifier:u32)->Self{Self{raw_mode,identifier}}
 pub fn raw_mode(&self)->u16{self.raw_mode}
 pub fn identifier(&self)->u32{self.identifier}
 pub fn catalog_mode(&self)->Result<u8,ModeError>{if self.raw_mode<=5{Ok(self.raw_mode as u8)}else{Err(ModeError::UnqualifiedMode{actual:self.raw_mode})}}
 /// Commit only an already fully decoded known snapshot. No other native
 /// globals/UI side effects are simulated. Returned phase is source metadata
 /// for the caller's explicit context, not a live connection state machine.
 pub fn apply_snapshot(&mut self,s:&Snapshot,context:Context)->Result<SnapshotOutcome,SnapshotError>{
  if s.kind()==Kind::State206&&context.dispatch_word==4&&(context.phase as i32)>=3{return Err(SnapshotError::DispatchUnavailable{phase:context.phase,dispatch_word:context.dispatch_word});}
  if s.kind()==Kind::State213&&context.phase!=6&&s.identifier()!=self.identifier{return Ok(SnapshotOutcome::IgnoredIdentifier);}
  let previous=self.raw_mode;
  let bind=s.kind()==Kind::State206||context.phase==6;
  let identifier_rebound=bind&&s.identifier()!=self.identifier;
  if bind{self.identifier=s.identifier();}
  self.raw_mode=(previous&0xff00)|u16::from(s.mode_byte());
  Ok(SnapshotOutcome::Applied{mode:Outcome::Applied{previous,current:self.raw_mode,identifier_rebound},native_phase_after:if s.kind()==Kind::State206{2}else{context.phase}})
 }
 pub fn apply(&mut self,packet:&[u8],phase:u32)->Result<Outcome,ModeError>{
  if packet.len()!=7{return Err(ModeError::Length{actual:packet.len()});}
  if packet[0]!=213{return Err(ModeError::Opcode{actual:packet[0]});}
  if packet[1]!=2{return Err(ModeError::Subtype{actual:packet[1]});}
  let identifier=u32::from_le_bytes([packet[2],packet[3],packet[4],packet[5]]);
  if phase!=6&&identifier!=self.identifier{return Ok(Outcome::IgnoredIdentifier);}
  let previous=self.raw_mode;
  let identifier_rebound=phase==6&&identifier!=self.identifier;
  if phase==6{self.identifier=identifier;}
  self.raw_mode=(previous&0xff00)|u16::from(packet[6]);
  Ok(Outcome::Applied{previous,current:self.raw_mode,identifier_rebound})
 }
}
