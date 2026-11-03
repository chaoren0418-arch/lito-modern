//! Candidate ingress AFTER a qualified Native transport has decoded a body.
//! Modern LTM1/TLS frames are a separate namespace and are never translated.
//! No normal Native socket/decoder producer exists yet. Epochs below protect
//! the production Runtime candidate boundary, not authentication or success.
use super::mode_selection::snapshot::Snapshot;
use anyhow::{ensure,Context,Result};
use std::sync::Arc;
/// LE16 total frame length minus its two-byte header. This is a Modern
/// bounded allocation policy; the historical Native receive buffer is unproved.
pub const MAX_DECODED_BODY:usize=u16::MAX as usize-2;
#[derive(Clone)]pub struct Epoch {owner:Arc<()>,serial:u64}
#[derive(Clone)]pub struct DecodedPacket {epoch:Epoch,body:Arc<[u8]>}
impl Epoch {
 pub fn serial(&self)->u64{self.serial}
 /// Copies only a completely validated known Native body. This is a bounded
 /// decoded-event constructor, not a wire/TLS decoder or server assertion.
 /// Ownership is checked again by Runtime at consumption, including after IO.
 pub fn decoded_packet(&self,body:&[u8])->Result<DecodedPacket>{
  ensure!(body.len()<=MAX_DECODED_BODY,"decoded Native body exceeds bound");
  if body.first()==Some(&206){
   // Generic Native success keeps its existing structural contract. AD368
   // room-ID/name admission belongs to the explicit Runtime adapter only.
   if body.get(1).is_some_and(|result|*result!=0){super::java_room_entry::decode(body)?;}else{Snapshot::decode(body)?;}
  }else if matches!(body.first(),Some(80|81|82|86)){super::status_family::decode(body)?;}else if body.first()==Some(&77){super::status_receipt::decode_mana(body)?;}else if body.first()==Some(&12){super::status_receipt::decode(body)?;}else if body.first()==Some(&245){ensure!(body.len()==1,"S_F5 exact opcode-only required");}else if matches!(body.first(),Some(124|202)){super::java_character_flow::decode(body)?;}else if matches!(body.first(),Some(0|59|230)){super::login_bootstrap::decode(body)?;}else if matches!(body.first(),Some(243|244|251)){super::entry_transfer::decode(body)?;}else if matches!(body.first(),Some(200|201)){super::entry_navigation::decode(body)?;}else if matches!(body.first(),Some(58|205|209)){super::source_actor::validate(body)?;}else if matches!(body.first(),Some(3|203)){super::source_identity::decode(body)?;}else if body.starts_with(&[213,2]){ensure!(body.len()==7,"mode delta must be exactly seven bytes");}else{Snapshot::decode(body)?;}
 Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})
 }
 /// Explicit AD368 full CD/D1/D0 validation; does not change generic Native version policy.
 pub fn decoded_java_room_participant(&self,body:&[u8])->Result<DecodedPacket>{super::java_room_participants::decode(body)?;Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})}
 /// Explicit server-direction AD368 owner/delete body; generic Native stays separate.
 pub fn decoded_java_room_countdown(&self,body:&[u8])->Result<DecodedPacket>{super::java_room_ready::decode_countdown(body)?;Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})}
 pub fn decoded_java_room_mode_map(&self,body:&[u8])->Result<DecodedPacket>{super::java_room_settings::decode_mode_map(body)?;Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})}
 pub fn decoded_java_room_settings(&self,body:&[u8])->Result<DecodedPacket>{super::java_room_settings::decode(body)?;Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})}
 pub fn decoded_java_room_tail(&self,body:&[u8])->Result<DecodedPacket>{match body.first(){Some(213)=>{super::java_room_tail::decode_owner(body)?;},Some(207)=>{super::java_room_tail::decode_delete(body)?;},_=>anyhow::bail!("not explicit AD368 D5/05 or S_CF")};Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})}
 pub fn decoded_java_room_catalog(&self,body:&[u8])->Result<DecodedPacket>{super::java_room_catalog::decode(body)?;Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})}
 ///4A7550 alternate branch checks flags before reading ordinary roster fields.
 ///Only Runtime's flag-qualified stream route can construct this trigger.
 pub(crate) fn decoded_transfer_roster_trigger(&self,body:&[u8])->Result<DecodedPacket>{ensure!(body.first()==Some(&203)&&body.len()<=MAX_DECODED_BODY,"bounded alternate203 required");Ok(DecodedPacket{epoch:self.clone(),body:Arc::from(body)})}
}
impl DecodedPacket {pub fn body(&self)->&[u8]{&self.body}pub fn epoch_serial(&self)->u64{self.epoch.serial}}
#[derive(Clone)]pub(crate) struct Scope {owner:Arc<()>,next:u64,active:Option<u64>,bound:bool}
impl Default for Scope {fn default()->Self{Self{owner:Arc::new(()),next:1,active:None,bound:false}}}
impl Scope {
 pub(crate) fn prepare_begin(&self)->Result<(Self,Epoch)>{let next_serial=self.next.checked_add(1).context("source connection epoch exhausted")?;let mut next=self.clone();next.bound=true;next.active=Some(self.next);next.next=next_serial;Ok((next,Epoch{owner:self.owner.clone(),serial:self.next}))}
 pub(crate) fn prepare_disconnect(&self,epoch:&Epoch)->Result<Self>{self.admit_epoch(epoch)?;let mut next=self.clone();next.active=None;Ok(next)}
 pub(crate) fn admit(&self,packet:&DecodedPacket)->Result<()>{self.admit_epoch(&packet.epoch)}
 pub(crate) fn admit_epoch(&self,epoch:&Epoch)->Result<()>{ensure!(Arc::ptr_eq(&self.owner,&epoch.owner)&&self.active==Some(epoch.serial),"retired or foreign source connection event");Ok(())}
 pub(crate) fn bound(&self)->bool{self.bound}
 pub(crate) fn retire(&mut self){self.active=None;}
 pub(crate) fn active_serial(&self)->Option<u64>{self.active}
 pub(crate) fn current_epoch(&self)->Result<Epoch>{Ok(Epoch{owner:self.owner.clone(),serial:self.active.context("source connection unavailable")?})}
}
