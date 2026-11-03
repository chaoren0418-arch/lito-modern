//! Explicit ordinary Java C_D1/2 writer. Intent and complete transport write
//! are separate from independent mode/map receipts, server owner admission, ACK or commit.
use anyhow::{ensure,Context,Result};
use super::{Runtime,SourceEpoch,java_room_settings as settings,source_send};
pub const NETWORK_SHA:&str="14B6DA5301356A26D038A97A7D0D42FD0F8C2DA3D68DD7ED0A0D13EB9409945E";
pub const NATIVE_SHA:&str="1BB37D61C8E25EC195EA2084941EFDC89D2E6F2760119583357A088A04C3EC8F";
pub const PRODUCER_SHA:&str="485F599DA9BA1A13504CB48DDCA3D53C6CAD91D7BAFDF738D104E65A05E1DFD5";
pub const MAX_REQUESTS:usize=32;
#[derive(Clone)]pub struct Adapter;
impl Adapter {pub fn parse(raw:&[u8])->Result<Self>{ensure!(raw.len()<=1024,"mode writer adapter bound");let v:serde_json::Value=serde_json::from_slice(raw)?;ensure!(v["kind"]=="modern_synthetic_485F_relative_mode_request_v1"&&v["producer_sha256"]==PRODUCER_SHA&&v["network_sha256"]==NETWORK_SHA&&v["native_sha256"]==NATIVE_SHA,"explicit pinned relative mode writer required");Ok(Self)}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Direction {Next,Previous}
impl Direction {pub fn byte(self)->u8{match self{Self::Next=>1,Self::Previous=>2}}pub fn from_wire(b:u8)->Result<Self>{match b{1=>Ok(Self::Next),2=>Ok(Self::Previous),_=>anyhow::bail!("relative direction must be NEXT1 or PREVIOUS2")}}}
pub fn encode(direction:Direction)->Vec<u8>{vec![209,2,direction.byte()]}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct ContextKey {pub room_id:u32,pub joined:settings::JoinedContext}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Phase {Queued,CompletedWrite,Retired}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Request {pub sequence:u64,pub write_serial:u64,pub direction:Direction,pub producer_sha256:&'static str,pub context:ContextKey,pub phase:Phase,pub written_bytes:usize,pub raw_body:Vec<u8>,pub frame_bytes:usize,pub retirement:Option<&'static str>}
impl Request {
 fn report(&mut self,serial:u64,progress:&source_send::Progress,current:bool){
  if serial!=self.write_serial||self.phase!=Phase::Queued{return;}
  if !current{self.phase=Phase::Retired;self.retirement=Some("mode context retired");return;}
  match progress {source_send::Progress::Partial{offset,..}=>self.written_bytes=*offset,source_send::Progress::Complete=>{self.written_bytes=self.frame_bytes;self.phase=Phase::CompletedWrite;},source_send::Progress::Retired{..}=>{self.phase=Phase::Retired;self.retirement=Some("source write retired");}}
 }
}
#[cfg(test)]mod tests {
 use super::*;
 fn request(serial:u64)->Request{Request{sequence:1,write_serial:serial,direction:Direction::Next,producer_sha256:PRODUCER_SHA,context:ContextKey{room_id:1,joined:settings::JoinedContext{ce_revision:1,source:b"GateNetMoon".to_vec(),source_revision:1,roster_revision:2,selected_slot:0,selected_alias:b"GateNetHost(GateNetMoon)".to_vec(),navigation_revision:3,channel:2}},phase:Phase::Queued,written_bytes:0,raw_body:encode(Direction::Next),frame_bytes:5,retirement:None}}
 #[test]fn relative_direction_pins_real_sender_five_byte_partial_full(){let spec=serde_json::json!({"kind":"modern_synthetic_485F_relative_mode_request_v1","producer_sha256":PRODUCER_SHA,"network_sha256":NETWORK_SHA,"native_sha256":NATIVE_SHA});Adapter::parse(&serde_json::to_vec(&spec).unwrap()).unwrap();for key in ["producer_sha256","network_sha256","native_sha256"]{let mut bad=spec.clone();bad[key]=serde_json::json!("unpinned");assert!(Adapter::parse(&serde_json::to_vec(&bad).unwrap()).is_err());}for b in [0,3,5,6,255]{assert!(Direction::from_wire(b).is_err());let mut sender=source_send::Sender::default();sender.seed_from_source(1).unwrap();let before=sender.snapshot();assert!(sender.prepare_ordinary_room_mode_step(b).is_err());assert_eq!(sender.snapshot(),before);}assert_eq!(encode(Direction::Next),[209,2,1]);assert_eq!(encode(Direction::Previous),[209,2,2]);for direction in [Direction::Next,Direction::Previous]{let seed=0x87546ca110203040u64;let mut sender=source_send::Sender::default();sender.seed_from_source(seed).unwrap();let ticket=sender.commit(sender.prepare_ordinary_room_mode_step(direction.byte()).unwrap()).ticket;let bytes=sender.view(&ticket).unwrap().bytes().unwrap().to_vec();assert_eq!(&bytes[..2],&[5,0]);let key=seed.to_le_bytes();let mut previous=0;let body=bytes[2..].iter().enumerate().map(|(i,b)|{let plain=*b^key[i&7]^previous;previous=*b;plain}).collect::<Vec<_>>();assert_eq!(body,encode(direction));let mut record=request(ticket.serial());record.direction=direction;record.raw_body=body;for offset in 0..5{let progress=sender.report(&ticket,offset,source_send::WriteOutcome::Written(1)).unwrap();record.report(ticket.serial(),&progress,true);assert_eq!(record.written_bytes,offset+1);assert_eq!(record.phase,if offset==4{Phase::CompletedWrite}else{Phase::Queued});}assert!(sender.report(&ticket,5,source_send::WriteOutcome::Written(1)).is_err());}}
 #[test]fn real_sender_foreign_stale_failed_zero_wouldblock_and_context_retired(){for failure in [source_send::WriteOutcome::Failed,source_send::WriteOutcome::Written(0),source_send::WriteOutcome::WouldBlock]{let mut a=source_send::Sender::default();let mut b=source_send::Sender::default();a.seed_from_source(1).unwrap();b.seed_from_source(1).unwrap();let ta=a.commit(a.prepare_ordinary_room_mode_step(1).unwrap()).ticket;let tb=b.commit(b.prepare_ordinary_room_mode_step(1).unwrap()).ticket;assert_eq!(ta.serial(),tb.serial());let before=a.snapshot();assert!(a.report(&tb,0,source_send::WriteOutcome::Written(1)).is_err());assert_eq!(a.snapshot(),before);let mut record=request(ta.serial());let p=a.report(&ta,0,source_send::WriteOutcome::Written(1)).unwrap();record.report(ta.serial(),&p,true);let before=a.snapshot();assert!(a.report(&ta,0,source_send::WriteOutcome::Written(1)).is_err());assert_eq!(a.snapshot(),before);let p=a.report(&ta,1,failure).unwrap();record.report(ta.serial(),&p,true);assert_eq!((record.phase,record.written_bytes),(Phase::Retired,1));}let mut record=request(2);record.report(1,&source_send::Progress::Complete,true);assert_eq!(record.phase,Phase::Queued);record.report(2,&source_send::Progress::Complete,false);assert_eq!((record.phase,record.written_bytes),(Phase::Retired,0));}
}
#[derive(Clone,Debug,Default,PartialEq,Eq)]pub struct Snapshot {pub revision:u64,pub requests:Vec<Request>}
#[derive(Clone)]pub struct Control {epoch:SourceEpoch,revision:u64,context:ContextKey,direction:Direction}
#[derive(Clone)]struct OwnedRequest {epoch:SourceEpoch,request:Request}
#[derive(Default)]pub(crate) struct State {pub(crate) adapter:Option<Adapter>,revision:u64,requests:Vec<OwnedRequest>}
impl State {
 fn snapshot(&self)->Snapshot{Snapshot{revision:self.revision,requests:self.requests.iter().map(|v|v.request.clone()).collect()}}
 fn invalidate(&mut self,reason:&'static str){for v in &mut self.requests{if v.request.phase==Phase::Queued{v.request.phase=Phase::Retired;v.request.retirement=Some(reason);}}}
 fn retire(&mut self){self.requests.clear();self.revision=0;}
}
impl Runtime {
 pub fn configure_java_room_mode_request_adapter(&mut self,adapter:Adapter)->Result<()>{ensure!(self.source_connection_epoch().is_none()&&self.java_room.key.phase==super::java_room_entry::Phase::Idle,"mode writer adapter must precede normal entry");self.java_mode_request.adapter=Some(adapter);Ok(())}
 pub fn java_room_mode_request_snapshot(&self)->Snapshot{self.java_mode_request.snapshot()}
 pub fn last_java_room_mode_request_checkpoint(&self)->Option<Snapshot>{self.last_mode_request_checkpoint.clone()}
 fn current_mode_request_context(&self)->Result<ContextKey>{
  ensure!(self.java_mode_request.adapter.is_some()&&self.java_settings.mode_map_adapter.as_ref().is_some_and(|a|a.producer_sha256()==PRODUCER_SHA)&&self.java_settings.adapter.as_ref().is_some_and(|a|a.producer_sha256()==PRODUCER_SHA)&&self.page==super::UiPage::Menu&&self.java_return.key.phase==super::java_return::Phase::Idle&&!self.entry_transfer.key.timer_active&&self.entry_navigation.queue.is_empty()&&self.java_room_self_snapshot().pending.is_none(),"ordinary mode request unavailable");
  let joined=self.admit_java_room_settings_context()?;let room=self.java_room.key.room.as_ref().context("mode joined room absent")?;ensure!(room.mode<=5,"only CE-bound ordinary battle modes0..5; town excluded");
  // The CE/source context establishes a candidate request path. Raw CE/D5
  // owner strings never grant server membership or owner authorization.
  Ok(ContextKey{room_id:room.id,joined})
 }
 pub fn java_room_mode_request_control(&self,direction:Direction)->Option<Control>{
  if self.source_send.snapshot().pending.is_some()||self.java_mode_request.requests.len()>=MAX_REQUESTS{return None;}
  Some(Control{epoch:self.source_scope.current_epoch().ok()?,revision:self.java_mode_request.revision,context:self.current_mode_request_context().ok()?,direction})
 }
 pub fn handle_java_room_mode_request_control(&mut self,c:&Control)->Result<()>{
  self.source_scope.admit_epoch(&c.epoch)?;ensure!(c.revision==self.java_mode_request.revision&&c.context==self.current_mode_request_context()?&&self.source_send.snapshot().pending.is_none()&&self.java_mode_request.requests.len()<MAX_REQUESTS,"retired, consumed or unavailable mode control");
  let revision=self.java_mode_request.revision.checked_add(1).context("mode intent revision exhausted")?;let body=encode(c.direction);let prepared=self.source_send.prepare_ordinary_room_mode_step(c.direction.byte())?;ensure!(prepared.reservation.frame_bytes==5,"relative mode frame must be five bytes");let serial=prepared.reservation.ticket.serial();
  self.source_send.commit(prepared);self.java_mode_request.revision=revision;self.java_mode_request.requests.push(OwnedRequest{epoch:c.epoch.clone(),request:Request{sequence:revision,write_serial:serial,direction:c.direction,producer_sha256:PRODUCER_SHA,context:c.context.clone(),phase:Phase::Queued,written_bytes:0,raw_body:body,frame_bytes:5,retirement:None}});self.status="正在送出相對模式要求；結果尚未確認".into();Ok(())
 }
 pub(crate) fn note_java_room_mode_write(&mut self,serial:u64,progress:&source_send::Progress){
  let context=self.current_mode_request_context().ok();let current=self.java_mode_request.requests.iter().find(|v|v.request.write_serial==serial).is_some_and(|v|self.source_scope.admit_epoch(&v.epoch).is_ok()&&context.as_ref()==Some(&v.request.context));
  if let Some(v)=self.java_mode_request.requests.iter_mut().find(|v|v.request.write_serial==serial){v.request.report(serial,progress,current);}
 }
 pub(crate) fn invalidate_java_room_mode_request(&mut self){self.java_mode_request.invalidate("joined room invalidated");}
 pub(crate) fn archive_retire_java_room_mode_request(&mut self){if !self.java_mode_request.requests.is_empty(){self.java_mode_request.invalidate("source context retired");self.last_mode_request_checkpoint=Some(self.java_mode_request.snapshot());}self.java_mode_request.retire();}
}
