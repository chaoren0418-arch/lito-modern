//! Explicit ordinary Java participant C_D2 writer. Intent and complete transport write
//! are separate from D1/E1 receipts, server membership/owner admission, ACK or commit.
use anyhow::{ensure,Context,Result};
use super::{Runtime,SourceEpoch,java_room_settings as settings,source_send};
pub const NETWORK_SHA:&str="3F31E26FF006B132C4531BA53811597D3D8EF8581531AD04E8599B0C2B170947";
pub const NATIVE_SHA:&str="F3867EC1B86DA3DF66A7493E92CD239E40AEC59B658EC0B588F463EFF13A2667";
pub const PRODUCER_SHA:&str="485F599DA9BA1A13504CB48DDCA3D53C6CAD91D7BAFDF738D104E65A05E1DFD5";
pub const MAX_REQUESTS:usize=32;
#[derive(Clone)]pub struct Adapter;
impl Adapter {pub fn parse(raw:&[u8])->Result<Self>{ensure!(raw.len()<=1024,"ready writer adapter bound");let v:serde_json::Value=serde_json::from_slice(raw)?;ensure!(v["kind"]=="modern_synthetic_485F_participant_ready_v1"&&v["producer_sha256"]==PRODUCER_SHA&&v["network_sha256"]==NETWORK_SHA&&v["native_sha256"]==NATIVE_SHA,"explicit pinned participant ready writer required");Ok(Self)}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ParticipantIntent {NotReady,Ready,ShopList}
impl ParticipantIntent {pub fn byte(self)->u8{match self{Self::NotReady=>0,Self::Ready=>1,Self::ShopList=>2}}pub fn from_wire(b:u8)->Result<Self>{match b{0=>Ok(Self::NotReady),1=>Ok(Self::Ready),2=>Ok(Self::ShopList),_=>anyhow::bail!("participant state must be0/1/2")}}}
pub fn encode(intent:ParticipantIntent)->Vec<u8>{vec![210,intent.byte()]}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct ContextKey {pub room_id:u32,pub joined:settings::JoinedContext,pub participants_revision:u64}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Phase {Queued,CompletedWrite,Retired}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Request {pub sequence:u64,pub write_serial:u64,pub intent:ParticipantIntent,pub producer_sha256:&'static str,pub context:ContextKey,pub phase:Phase,pub written_bytes:usize,pub raw_body:Vec<u8>,pub frame_bytes:usize,pub retirement:Option<&'static str>}
impl Request {
 fn report(&mut self,serial:u64,progress:&source_send::Progress,current:bool){
  if serial!=self.write_serial||self.phase!=Phase::Queued{return;}
  if !current{self.phase=Phase::Retired;self.retirement=Some("ready context retired");return;}
  match progress {source_send::Progress::Partial{offset,..}=>self.written_bytes=*offset,source_send::Progress::Complete=>{self.written_bytes=self.frame_bytes;self.phase=Phase::CompletedWrite;},source_send::Progress::Retired{..}=>{self.phase=Phase::Retired;self.retirement=Some("source write retired");}}
 }
}
#[derive(Clone,Debug,Default,PartialEq,Eq)]pub struct Snapshot {pub revision:u64,pub requests:Vec<Request>,pub countdown:Option<Countdown>}
#[derive(Clone)]pub struct Control {epoch:SourceEpoch,revision:u64,context:ContextKey,intent:ParticipantIntent}
#[derive(Clone)]struct OwnedRequest {epoch:SourceEpoch,request:Request}
#[derive(Default)]pub(crate) struct State {pub(crate) adapter:Option<Adapter>,revision:u64,requests:Vec<OwnedRequest>,countdown:Option<Countdown>}
impl State {
 fn snapshot(&self)->Snapshot{Snapshot{revision:self.revision,requests:self.requests.iter().map(|v|v.request.clone()).collect(),countdown:self.countdown.clone()}}
 fn invalidate(&mut self,reason:&'static str){for v in &mut self.requests{if v.request.phase==Phase::Queued{v.request.phase=Phase::Retired;v.request.retirement=Some(reason);}}}
 fn retire(&mut self){self.requests.clear();self.revision=0;self.countdown=None;}
}
impl Runtime {
 pub fn configure_java_room_ready_adapter(&mut self,adapter:Adapter)->Result<()>{ensure!(self.source_connection_epoch().is_none()&&self.java_room.key.phase==super::java_room_entry::Phase::Idle,"ready writer adapter must precede normal entry");self.java_ready.adapter=Some(adapter);Ok(())}
 pub fn java_room_ready_snapshot(&self)->Snapshot{self.java_ready.snapshot()}
 pub fn last_java_room_ready_checkpoint(&self)->Option<Snapshot>{self.last_ready_checkpoint.clone()}
 fn current_ready_request_context(&self)->Result<ContextKey>{
  ensure!(self.java_ready.adapter.is_some()&&self.java_settings.adapter.as_ref().is_some_and(|a|a.producer_sha256()==PRODUCER_SHA)&&self.page==super::UiPage::Menu&&self.java_return.key.phase==super::java_return::Phase::Idle&&!self.entry_transfer.key.timer_active&&self.entry_navigation.queue.is_empty()&&self.java_room_self_snapshot().pending.is_none(),"ordinary ready request unavailable");
  let joined=self.admit_java_room_settings_context()?;let room=self.java_room.key.room.as_ref().context("ready joined room absent")?;ensure!(room.mode<=5,"only CE-bound ordinary battle modes0..5; town excluded");
  // The CE/source context establishes a candidate request path. Raw CE/D5
  // owner strings never grant server membership or owner authorization.
  let own=self.java_room_self_snapshot();let alias=self.qualified_singleton_self_alias().context("ready current singleton self absent")?;
  ensure!(own.current_alias_qualified&&own.joined.as_ref().is_some_and(|c|c.room_id==room.id&&c.ce_revision==joined.ce_revision&&c.navigation_revision==joined.navigation_revision)&&self.java_room_participants_route_enabled(),"ready current self/ROOM context absent");
  let members=self.java_room_participants_snapshot().context("ready private participants absent")?;
  ensure!(members.authenticated_singleton_self_qualified&&members.room_id==room.id&&members.ce_revision==Some(joined.ce_revision)&&!members.joined_room_invalidated&&members.rows.iter().filter(|r|r.participant.display_alias==alias).count()==1,"ready requires unique observed current self; not hub membership proof");
  Ok(ContextKey{room_id:room.id,joined,participants_revision:members.revision})
 }
 pub fn java_room_ready_control(&self,intent:ParticipantIntent)->Option<Control>{
  if self.source_send.snapshot().pending.is_some()||self.java_ready.requests.len()>=MAX_REQUESTS{return None;}
  Some(Control{epoch:self.source_scope.current_epoch().ok()?,revision:self.java_ready.revision,context:self.current_ready_request_context().ok()?,intent})
 }
 pub fn handle_java_room_ready_control(&mut self,c:&Control)->Result<()>{
  self.source_scope.admit_epoch(&c.epoch)?;ensure!(c.revision==self.java_ready.revision&&c.context==self.current_ready_request_context()?&&self.source_send.snapshot().pending.is_none()&&self.java_ready.requests.len()<MAX_REQUESTS,"retired, consumed or unavailable ready control");
  let revision=self.java_ready.revision.checked_add(1).context("ready intent revision exhausted")?;let body=encode(c.intent);let prepared=self.source_send.prepare_participant_ready(c.intent.byte())?;ensure!(prepared.reservation.frame_bytes==4,"participant ready frame must be four bytes");let serial=prepared.reservation.ticket.serial();
  self.source_send.commit(prepared);self.java_ready.revision=revision;self.java_ready.requests.push(OwnedRequest{epoch:c.epoch.clone(),request:Request{sequence:revision,write_serial:serial,intent:c.intent,producer_sha256:PRODUCER_SHA,context:c.context.clone(),phase:Phase::Queued,written_bytes:0,raw_body:body,frame_bytes:4,retirement:None}});self.status="正在送出準備狀態要求；結果尚未確認".into();Ok(())
 }
 pub(crate) fn note_java_room_ready_write(&mut self,serial:u64,progress:&source_send::Progress){
  let context=self.current_ready_request_context().ok();let current=self.java_ready.requests.iter().find(|v|v.request.write_serial==serial).is_some_and(|v|self.source_scope.admit_epoch(&v.epoch).is_ok()&&context.as_ref()==Some(&v.request.context));
  if let Some(v)=self.java_ready.requests.iter_mut().find(|v|v.request.write_serial==serial){v.request.report(serial,progress,current);}
 }
 pub(crate) fn invalidate_java_room_ready_request(&mut self){self.java_ready.invalidate("joined room invalidated");if let Some(c)=self.java_ready.countdown.as_mut(){c.count=None;c.invalidated=true;}}
 pub(crate) fn archive_retire_java_room_ready_request(&mut self){if !self.java_ready.requests.is_empty()||self.java_ready.countdown.is_some(){self.invalidate_java_room_ready_request();self.last_ready_checkpoint=Some(self.java_ready.snapshot());}self.java_ready.retire();}
}

/// E1 is an unsigned BYTE receipt with no room, generation or sequence on wire.
/// The local context labels where it was observed, never proves wire freshness.
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Countdown {pub epoch:u64,pub room_id:u32,pub joined:settings::JoinedContext,pub revision:u64,pub count:Option<u8>,pub receipts:Vec<u8>,pub invalidated:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct CountdownChange {pub count:u8,pub cancelled:bool}
pub fn decode_countdown(body:&[u8])->Result<u8>{ensure!(body.len()==2&&body[0]==225,"E1 requires exact opcode+unsignedBYTE");Ok(body[1])}
impl Runtime {
 /// No independent owner authority exists in this adapter. CE/D5 display,
 /// selected alias, Native flags and D1 state cannot mint an owner command.
 pub fn java_room_owner_start_unavailable_reason(&self)->&'static str{"independent current owner qualification absent; opcode-only C_D2 refused"}
 pub fn request_java_room_owner_start(&mut self)->Result<()>{anyhow::bail!(self.java_room_owner_start_unavailable_reason())}
 pub(crate) fn java_room_countdown_dispatch(&self,body:&[u8])->bool{self.java_ready.adapter.is_some()&&body.first()==Some(&225)}
 pub fn consume_java_room_countdown_candidate(&mut self,event:&super::SourceDecodedPacket)->Result<CountdownChange>{
  self.source_scope.admit(event)?;ensure!(self.java_ready.adapter.is_some(),"E1 explicit485F adapter absent");let count=decode_countdown(event.body())?;
  let context=self.current_ready_request_context()?;
  let mut next=self.java_ready.countdown.clone().filter(|c|!c.invalidated&&c.epoch==event.epoch_serial()&&c.room_id==context.room_id&&c.joined==context.joined).unwrap_or(Countdown{epoch:event.epoch_serial(),room_id:context.room_id,joined:context.joined,revision:0,count:None,receipts:vec![],invalidated:false});
  ensure!(next.receipts.len()<1024,"E1 receipt budget reached");next.revision=next.revision.checked_add(1).context("E1 revision exhausted")?;next.count=if count==0{None}else{Some(count)};next.receipts.push(count);self.java_ready.countdown=Some(next);
  self.status=if count==0{"已收到倒數取消通知"}else{"已收到倒數資料；等待伺服器後續通知"}.into();
  // E100 never changes D1 readiness. No local clock or WORLD grant exists.
  Ok(CountdownChange{count,cancelled:count==0})
 }
}

#[cfg(test)]mod r526_tests {
 use super::*;
 fn request(serial:u64)->Request{Request{sequence:1,write_serial:serial,intent:ParticipantIntent::NotReady,producer_sha256:PRODUCER_SHA,context:ContextKey{room_id:1,joined:settings::JoinedContext{ce_revision:1,source:b"GateNetMoon".to_vec(),source_revision:1,roster_revision:2,selected_slot:0,selected_alias:b"GateNetHost(GateNetMoon)".to_vec(),navigation_revision:3,channel:2},participants_revision:3},phase:Phase::Queued,written_bytes:0,raw_body:encode(ParticipantIntent::NotReady),frame_bytes:4,retirement:None}}
 #[test]fn participant_intent_pins_real_sender_four_byte_partial_full(){let spec=serde_json::json!({"kind":"modern_synthetic_485F_participant_ready_v1","producer_sha256":PRODUCER_SHA,"network_sha256":NETWORK_SHA,"native_sha256":NATIVE_SHA});Adapter::parse(&serde_json::to_vec(&spec).unwrap()).unwrap();for key in ["producer_sha256","network_sha256","native_sha256"]{let mut bad=spec.clone();bad[key]=serde_json::json!("unpinned");assert!(Adapter::parse(&serde_json::to_vec(&bad).unwrap()).is_err());}for b in [3,5,6,255]{assert!(ParticipantIntent::from_wire(b).is_err());let mut sender=source_send::Sender::default();sender.seed_from_source(1).unwrap();let before=sender.snapshot();assert!(sender.prepare_participant_ready(b).is_err());assert_eq!(sender.snapshot(),before);}assert_eq!(encode(ParticipantIntent::NotReady),[210,0]);assert_eq!(encode(ParticipantIntent::ShopList),[210,2]);for intent in [ParticipantIntent::NotReady,ParticipantIntent::Ready,ParticipantIntent::ShopList]{let seed=0x87546ca110203040u64;let mut sender=source_send::Sender::default();sender.seed_from_source(seed).unwrap();let ticket=sender.commit(sender.prepare_participant_ready(intent.byte()).unwrap()).ticket;let bytes=sender.view(&ticket).unwrap().bytes().unwrap().to_vec();assert_eq!(&bytes[..2],&[4,0]);let key=seed.to_le_bytes();let mut previous=0;let body=bytes[2..].iter().enumerate().map(|(i,b)|{let plain=*b^key[i&7]^previous;previous=*b;plain}).collect::<Vec<_>>();assert_eq!(body,encode(intent));let mut record=request(ticket.serial());record.intent=intent;record.raw_body=body;for offset in 0..4{let progress=sender.report(&ticket,offset,source_send::WriteOutcome::Written(1)).unwrap();record.report(ticket.serial(),&progress,true);assert_eq!(record.written_bytes,offset+1);assert_eq!(record.phase,if offset==3{Phase::CompletedWrite}else{Phase::Queued});}assert!(sender.report(&ticket,4,source_send::WriteOutcome::Written(1)).is_err());}}
 #[test]fn real_sender_foreign_stale_failed_zero_wouldblock_and_context_retired(){for failure in [source_send::WriteOutcome::Failed,source_send::WriteOutcome::Written(0),source_send::WriteOutcome::WouldBlock]{let mut a=source_send::Sender::default();let mut b=source_send::Sender::default();a.seed_from_source(1).unwrap();b.seed_from_source(1).unwrap();let ta=a.commit(a.prepare_participant_ready(1).unwrap()).ticket;let tb=b.commit(b.prepare_participant_ready(1).unwrap()).ticket;assert_eq!(ta.serial(),tb.serial());let before=a.snapshot();assert!(a.report(&tb,0,source_send::WriteOutcome::Written(1)).is_err());assert_eq!(a.snapshot(),before);let mut record=request(ta.serial());let p=a.report(&ta,0,source_send::WriteOutcome::Written(1)).unwrap();record.report(ta.serial(),&p,true);let before=a.snapshot();assert!(a.report(&ta,0,source_send::WriteOutcome::Written(1)).is_err());assert_eq!(a.snapshot(),before);let p=a.report(&ta,1,failure).unwrap();record.report(ta.serial(),&p,true);assert_eq!((record.phase,record.written_bytes),(Phase::Retired,1));}let mut record=request(2);record.report(1,&source_send::Progress::Complete,true);assert_eq!(record.phase,Phase::Queued);record.report(2,&source_send::Progress::Complete,false);assert_eq!((record.phase,record.written_bytes),(Phase::Retired,0));}
}
