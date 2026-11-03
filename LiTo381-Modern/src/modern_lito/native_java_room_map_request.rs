//! Explicit ordinary Java C_D1/3 writer. Intent and complete transport write
//! are separate from map receipts, server owner admission, ACK or commit.
use anyhow::{ensure,Context,Result};
use super::{Runtime,SourceEpoch,java_room_settings as settings,source_send};
pub const NETWORK_SHA:&str="3BBAA920A2B379B7562A38C3FF2CADC8842B98F7033C3A5C103174F4A28A8545";
pub const SELECTION_SHA:&str="3D0B5BAA46238ED5589C193FBDE22B966B945EFDEAB70D2A2C3933A62CA8E15F";
pub const C7_SHA:&str="C7B169C00955B444BCDB357E6A63625A87AABC5B23F8F3A21E75988D8270D43A";
pub const MAX_REQUESTS:usize=32;
#[derive(Clone)]pub struct Adapter;
impl Adapter {pub fn parse(raw:&[u8])->Result<Self>{ensure!(raw.len()<=1024,"map writer adapter bound");let v:serde_json::Value=serde_json::from_slice(raw)?;ensure!(v["kind"]=="modern_synthetic_C7_ordinary_map_request_v1"&&v["producer_sha256"]==C7_SHA&&v["network_sha256"]==NETWORK_SHA&&v["selection_sha256"]==SELECTION_SHA,"explicit pinned ordinary map writer required");Ok(Self)}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Choice {Map509,Map514,RejectedFFFFProbe}
impl Choice {pub fn word(self)->u16{match self{Self::Map509=>509,Self::Map514=>514,Self::RejectedFFFFProbe=>0xffff}}}
pub fn encode(choice:Choice)->Vec<u8>{let mut b=vec![209,3];b.extend(choice.word().to_le_bytes());b}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct ContextKey {pub room_id:u32,pub joined:settings::JoinedContext}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Phase {Queued,CompletedWrite,Retired}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Request {pub sequence:u64,pub write_serial:u64,pub map_word:u16,pub context:ContextKey,pub phase:Phase,pub written_bytes:usize,pub raw_body:Vec<u8>,pub frame_bytes:usize,pub retirement:Option<&'static str>}
impl Request {
 fn report(&mut self,serial:u64,progress:&source_send::Progress,current:bool){
  if serial!=self.write_serial||self.phase!=Phase::Queued{return;}
  if !current{self.phase=Phase::Retired;self.retirement=Some("map context retired");return;}
  match progress {source_send::Progress::Partial{offset,..}=>self.written_bytes=*offset,source_send::Progress::Complete=>{self.written_bytes=self.frame_bytes;self.phase=Phase::CompletedWrite;},source_send::Progress::Retired{..}=>{self.phase=Phase::Retired;self.retirement=Some("source write retired");}}
 }
}
#[derive(Clone,Debug,Default,PartialEq,Eq)]pub struct Snapshot {pub revision:u64,pub requests:Vec<Request>}
#[derive(Clone)]pub struct Control {epoch:SourceEpoch,revision:u64,context:ContextKey,choice:Choice}
#[derive(Clone)]struct OwnedRequest {epoch:SourceEpoch,request:Request}
#[derive(Default)]pub(crate) struct State {pub(crate) adapter:Option<Adapter>,revision:u64,requests:Vec<OwnedRequest>}
impl State {
 fn snapshot(&self)->Snapshot{Snapshot{revision:self.revision,requests:self.requests.iter().map(|v|v.request.clone()).collect()}}
 fn invalidate(&mut self,reason:&'static str){for v in &mut self.requests{if v.request.phase==Phase::Queued{v.request.phase=Phase::Retired;v.request.retirement=Some(reason);}}}
 fn retire(&mut self){self.requests.clear();self.revision=0;}
}
impl Runtime {
 pub fn configure_java_room_map_request_adapter(&mut self,adapter:Adapter)->Result<()>{ensure!(self.source_connection_epoch().is_none()&&self.java_room.key.phase==super::java_room_entry::Phase::Idle,"map writer adapter must precede normal entry");self.java_map_request.adapter=Some(adapter);Ok(())}
 pub fn java_room_map_request_snapshot(&self)->Snapshot{self.java_map_request.snapshot()}
 pub fn last_java_room_map_request_checkpoint(&self)->Option<Snapshot>{self.last_map_request_checkpoint.clone()}
 fn current_map_request_context(&self)->Result<ContextKey>{
  ensure!(self.java_map_request.adapter.is_some()&&self.java_settings.mode_map_adapter.is_some()&&self.page==super::UiPage::Menu&&self.java_return.key.phase==super::java_return::Phase::Idle&&!self.entry_transfer.key.timer_active&&self.entry_navigation.queue.is_empty()&&self.java_room_self_snapshot().pending.is_none(),"ordinary map request unavailable");
  let joined=self.admit_java_room_settings_context()?;let room=self.java_room.key.room.as_ref().context("map joined room absent")?;ensure!(room.mode==1,"only CE-bound ordinary mode1 map candidate");
  // The CE/source context establishes a candidate request path. Raw CE/D5
  // owner strings never grant server membership or owner authorization.
  Ok(ContextKey{room_id:room.id,joined})
 }
 pub fn java_room_map_request_control(&self,choice:Choice)->Option<Control>{
  if self.source_send.snapshot().pending.is_some()||self.java_map_request.requests.len()>=MAX_REQUESTS{return None;}
  Some(Control{epoch:self.source_scope.current_epoch().ok()?,revision:self.java_map_request.revision,context:self.current_map_request_context().ok()?,choice})
 }
 pub fn handle_java_room_map_request_control(&mut self,c:&Control)->Result<()>{
  self.source_scope.admit_epoch(&c.epoch)?;ensure!(c.revision==self.java_map_request.revision&&c.context==self.current_map_request_context()?&&self.source_send.snapshot().pending.is_none()&&self.java_map_request.requests.len()<MAX_REQUESTS,"retired, consumed or unavailable map control");
  let revision=self.java_map_request.revision.checked_add(1).context("map intent revision exhausted")?;let body=encode(c.choice);let prepared=self.source_send.prepare_ordinary_room_map(c.choice.word())?;ensure!(prepared.reservation.frame_bytes==6,"map frame must be six bytes");let serial=prepared.reservation.ticket.serial();
  self.source_send.commit(prepared);self.java_map_request.revision=revision;self.java_map_request.requests.push(OwnedRequest{epoch:c.epoch.clone(),request:Request{sequence:revision,write_serial:serial,map_word:c.choice.word(),context:c.context.clone(),phase:Phase::Queued,written_bytes:0,raw_body:body,frame_bytes:6,retirement:None}});self.status="正在送出地圖要求；結果尚未確認".into();Ok(())
 }
 pub(crate) fn note_java_room_map_write(&mut self,serial:u64,progress:&source_send::Progress){
  let context=self.current_map_request_context().ok();let current=self.java_map_request.requests.iter().find(|v|v.request.write_serial==serial).is_some_and(|v|self.source_scope.admit_epoch(&v.epoch).is_ok()&&context.as_ref()==Some(&v.request.context));
  if let Some(v)=self.java_map_request.requests.iter_mut().find(|v|v.request.write_serial==serial){v.request.report(serial,progress,current);}
 }
 pub(crate) fn invalidate_java_room_map_request(&mut self){self.java_map_request.invalidate("joined room invalidated");}
 pub(crate) fn archive_retire_java_room_map_request(&mut self){if !self.java_map_request.requests.is_empty(){self.java_map_request.invalidate("source context retired");self.last_map_request_checkpoint=Some(self.java_map_request.snapshot());}self.java_map_request.retire();}
}
#[cfg(test)]mod tests {
 use super::*;
 fn request(serial:u64)->Request{Request{sequence:1,write_serial:serial,map_word:514,context:ContextKey{room_id:1,joined:settings::JoinedContext{ce_revision:1,source:b"MapNetMoon".to_vec(),source_revision:1,roster_revision:2,selected_slot:0,selected_alias:b"MapNetHost(MapNetMoon)".to_vec(),navigation_revision:3,channel:2}},phase:Phase::Queued,written_bytes:0,raw_body:encode(Choice::Map514),frame_bytes:6,retirement:None}}
 #[test]fn real_sender_six_byte_partial_complete_and_once(){let mut sender=source_send::Sender::default();let seed=0x87546ca110203040;sender.seed_from_source(seed).unwrap();let prepared=sender.prepare_ordinary_room_map(514).unwrap();assert_eq!(prepared.reservation.frame_bytes,6);let ticket=sender.commit(prepared).ticket;let raw=sender.view(&ticket).unwrap().bytes().unwrap().to_vec();assert_eq!(&raw[..2],&[6,0]);let k=seed.to_le_bytes();let mut prev=0;let body=raw[2..].iter().enumerate().map(|(i,b)|{let out=*b^k[i&7]^prev;prev=*b;out}).collect::<Vec<_>>();assert_eq!(body,[209,3,2,2]);let mut intent=request(ticket.serial());for offset in 0..6{let view=sender.view(&ticket).unwrap();assert_eq!(view.offset(),offset);let progress=sender.report(&ticket,offset,source_send::WriteOutcome::Written(1)).unwrap();intent.report(ticket.serial(),&progress,true);assert_eq!(intent.written_bytes,offset+1);if offset<5{assert_eq!(intent.phase,Phase::Queued);}else{assert_eq!(intent.phase,Phase::CompletedWrite);}}assert!(sender.report(&ticket,6,source_send::WriteOutcome::Written(1)).is_err());let before=intent.clone();intent.report(ticket.serial(),&source_send::Progress::Complete,false);assert_eq!(intent,before);}
 #[test]fn real_sender_failures_foreign_stale_and_retired_context(){for failure in [source_send::WriteOutcome::Failed,source_send::WriteOutcome::Written(0),source_send::WriteOutcome::WouldBlock]{let mut a=source_send::Sender::default();let mut b=source_send::Sender::default();a.seed_from_source(1).unwrap();b.seed_from_source(1).unwrap();let ta=a.commit(a.prepare_ordinary_room_map(509).unwrap()).ticket;let tb=b.commit(b.prepare_ordinary_room_map(509).unwrap()).ticket;assert_eq!(ta.serial(),tb.serial());let before=a.snapshot();assert!(a.report(&tb,0,source_send::WriteOutcome::Written(1)).is_err());assert_eq!(a.snapshot(),before);let mut intent=request(ta.serial());let p=a.report(&ta,0,source_send::WriteOutcome::Written(1)).unwrap();intent.report(ta.serial(),&p,true);assert_eq!(intent.phase,Phase::Queued);let before=a.snapshot();assert!(a.report(&ta,0,source_send::WriteOutcome::Written(1)).is_err());assert_eq!(a.snapshot(),before);let p=a.report(&ta,1,failure).unwrap();intent.report(ta.serial(),&p,true);assert_eq!(intent.phase,Phase::Retired);assert_eq!(intent.written_bytes,1);}let mut intent=request(2);intent.report(1,&source_send::Progress::Complete,true);assert_eq!(intent.phase,Phase::Queued);intent.report(2,&source_send::Progress::Complete,false);assert_eq!(intent.phase,Phase::Retired);assert_eq!(intent.written_bytes,0);}
}
