//! Independent singleton GAME roster provenance and one-shot ordinary C_D0
//! completed-write correlation. No committed leave/scene/physical identity.
use anyhow::{ensure,Context,Result};
use super::{Runtime,SourceEpoch,StreamIngress,source_send,source_identity,java_character_flow,login_bootstrap,java_room_entry,entry_navigation};
pub const CONTRACT_SHA:&str="423A1EA9E4578E735FF3CC9717AADA57CA0434C2F1129E2F696EE372CAE6507F";
fn complete_singleton_alias(identity:&source_identity::State)->Result<Option<Vec<u8>>>{
 if identity.expected()!=Some(1)||!identity.complete()||identity.records().len()!=1{return Ok(None);}
 let raw=identity.records()[0].name();ensure!((1..=27).contains(&raw.len())&&!raw.iter().any(|b|*b<32||*b==127),"singleton alias outside bounded ordinary bytes");super::java_room_departure::validate_cp950(raw)?;Ok(Some(raw.to_vec()))
}
#[derive(Clone)]pub struct Adapter;
impl Adapter {
 pub fn parse(bytes:&[u8])->Result<Self>{
  ensure!(bytes.len()<=512,"self correlation adapter exceeds bound");let v:serde_json::Value=serde_json::from_slice(bytes)?;
  ensure!(v["kind"]=="modern_synthetic_AD368_singleton_self_v1"&&v["producer_sha256"]==login_bootstrap::PRODUCER_SHA&&v["contract_sha256"]==CONTRACT_SHA,"pinned singleton self correlation adapter required");Ok(Self)
 }
}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct QualifiedAlias {
 pub epoch:u64,pub source:Vec<u8>,pub source_revision:u64,pub roster_revision:u64,pub selection_write_serial:u64,pub slot:usize,
 pub raw_alias:Vec<u8>,pub raw_count:Vec<u8>,pub raw_row:Vec<u8>,
}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct JoinedContext {pub room_id:u32,pub ce_revision:u64,pub navigation_revision:u64,pub channel:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum IntentPhase {Queued,CompletedWrite,NotificationConsumed}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct PendingIntent {pub write_serial:u64,pub context:JoinedContext,pub phase:IntentPhase}
impl PendingIntent {
 fn report(&mut self,serial:u64,progress:&source_send::Progress,current:bool)->bool{
  if self.write_serial!=serial{return true;}
  if !current||matches!(progress,source_send::Progress::Retired{..}){return false;}
  if self.phase==IntentPhase::Queued&&matches!(progress,source_send::Progress::Complete){self.phase=IntentPhase::CompletedWrite;}true
 }
 fn correlate(&mut self,context:&JoinedContext,alias_matches:bool)->Option<u64>{
  if self.phase!=IntentPhase::CompletedWrite||&self.context!=context||!alias_matches{return None;}
  self.phase=IntentPhase::NotificationConsumed;Some(self.write_serial)
 }
}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Notification {pub context:JoinedContext,pub write_serial:u64,pub receipt:super::java_room_departure::Receipt}
#[derive(Clone,Debug,Default,PartialEq,Eq)]pub struct Snapshot {
 pub alias:Option<QualifiedAlias>,pub current_alias_qualified:bool,pub joined:Option<JoinedContext>,pub pending:Option<PendingIntent>,pub notification:Option<Notification>,pub last_invalidation:Option<&'static str>,
}
#[derive(Clone)]struct IssuedRoster {epoch:SourceEpoch,source:Vec<u8>,source_revision:u64,roster_revision:u64,raw_count:Vec<u8>,raw_row:Option<Vec<u8>>,raw_alias:Option<Vec<u8>>}
#[derive(Clone,Default)]pub(crate) struct State {pub(crate) adapter:Option<Adapter>,issued:Option<IssuedRoster>,binding_epoch:Option<SourceEpoch>,selection_serial:Option<u64>,key:Snapshot}
impl State {
 pub(crate) fn java_snapshot_present(&self)->bool{self.key.alias.is_some()||self.key.pending.is_some()||self.key.notification.is_some()}
 pub(crate) fn retire(&mut self){let adapter=self.adapter.clone();*self=Self{adapter,..Default::default()};}
 pub(crate) fn selection_requested(&mut self,serial:u64){self.binding_epoch=None;self.key=Snapshot::default();self.selection_serial=Some(serial);}
 pub(crate) fn invalidate_joined(&mut self){self.key.joined=None;self.key.pending=None;self.key.last_invalidation=Some("joined room invalidated");}
}
impl Runtime {
 pub fn configure_java_room_self_adapter(&mut self,adapter:Adapter)->Result<()>{
  ensure!(self.source_connection_epoch().is_none()&&self.java_room.key.phase==java_room_entry::Phase::Idle,"self adapter must precede normal entry");self.java_self.adapter=Some(adapter);Ok(())
 }
 fn singleton_binding_current(&self)->bool{
  let Some(a)=&self.java_self.key.alias else{return false;};let Some(epoch)=&self.java_self.binding_epoch else{return false;};
  self.java_self.adapter.is_some()&&self.source_scope.admit_epoch(epoch).is_ok()&&self.bootstrap_adapter.is_some()
   &&self.login_bootstrap.key.stage==login_bootstrap::Stage::Roster&&self.login_bootstrap.key.revision==a.source_revision
   &&self.login_bootstrap.key.selected_name.as_deref()==Some(a.source.as_slice())&&self.login_bootstrap.key.roster_count==Some(1)
   &&self.source_roster_revision==a.roster_revision&&complete_singleton_alias(&self.source_identity).is_ok_and(|raw|raw.as_ref()==Some(&a.raw_alias))
   &&self.source_identity.selected().is_some_and(|s|s.slot()==a.slot&&s.actor_alias()==a.raw_alias)
   &&self.java_character.key.character_result==Some(0)&&self.java_character.key.selected_name.as_deref()==Some(a.raw_alias.as_slice())
 }
 pub fn java_room_self_snapshot(&self)->Snapshot{let mut key=self.java_self.key.clone();key.current_alias_qualified=self.singleton_binding_current();key}
 pub fn last_java_room_self_checkpoint(&self)->Option<Snapshot>{self.last_self_checkpoint.clone()}
 pub(crate) fn qualified_singleton_self_alias(&self)->Option<Vec<u8>>{self.singleton_binding_current().then(||self.java_self.key.alias.as_ref().unwrap().raw_alias.clone())}
 fn self_join_current(&self)->bool{
  let Some(c)=&self.java_self.key.joined else{return false;};
  self.singleton_binding_current()&&self.java_room_departure_route_enabled()&&self.java_room.key.revision==c.ce_revision
   &&self.java_room.key.room.as_ref().is_some_and(|r|r.id==c.room_id)&&self.entry_navigation.key.revision==c.navigation_revision
   &&self.java_character.key.channel_info.is_some_and(|i|i.channel==c.channel)
 }
 pub(crate) fn prepare_self_joined(&self,room_id:u32,ce_revision:u64,navigation_revision:u64)->State{
  let mut next=self.java_self.clone();next.key.pending=None;next.key.notification=None;next.key.last_invalidation=None;
  next.key.joined=if self.singleton_binding_current(){self.java_character.key.channel_info.map(|i|JoinedContext{room_id,ce_revision,navigation_revision,channel:i.channel})}else{None};next
 }
 pub(crate) fn note_java_self_stream(&mut self,epoch:&SourceEpoch,body:&[u8],ingress:&StreamIngress)->Result<()>{
  if self.java_self.adapter.is_none(){return Ok(());}
  match ingress {
   StreamIngress::Roster(source_identity::Change::Started{count})=>{
    self.java_self.retire();
    if *count==1&&self.login_bootstrap.key.stage==login_bootstrap::Stage::Roster&&self.bootstrap_adapter.is_some(){
     let source=self.login_bootstrap.key.selected_name.clone().context("singleton selected source absent")?;
     self.java_self.issued=Some(IssuedRoster{epoch:epoch.clone(),source,source_revision:self.login_bootstrap.key.revision,roster_revision:self.source_roster_revision,raw_count:body.to_vec(),raw_row:None,raw_alias:None});
    }
   },
   StreamIngress::Roster(source_identity::Change::Appended{slot:0,complete:true})=>{
    if let Some(issued)=self.java_self.issued.as_mut(){
     self.source_scope.admit_epoch(&issued.epoch)?;
     ensure!(issued.roster_revision.checked_add(1)==Some(self.source_roster_revision)&&self.source_identity.expected()==Some(1)&&self.source_identity.records().len()==1,"singleton roster revision mismatch");
     let raw=complete_singleton_alias(&self.source_identity)?.context("complete singleton row absent")?;
     issued.roster_revision=self.source_roster_revision;issued.raw_alias=Some(raw);issued.raw_row=Some(body.to_vec());
    }
   },
   StreamIngress::JavaCharacter(java_character_flow::Change::CharacterAccepted)=>{
    if let Some(issued)=&self.java_self.issued{
     self.source_scope.admit_epoch(&issued.epoch)?;
     if issued.roster_revision==self.source_roster_revision&&issued.source_revision==self.login_bootstrap.key.revision&&self.login_bootstrap.key.selected_name.as_ref()==Some(&issued.source)
      &&self.source_identity.complete()&&self.source_identity.expected()==Some(1)&&self.source_identity.records().len()==1{
      let raw=issued.raw_alias.clone().context("complete singleton raw alias absent")?;
      let selected=self.source_identity.selected().context("singleton selector absent")?;
      ensure!(selected.slot()==0&&selected.actor_alias()==raw&&self.java_character.key.selected_name.as_ref()==Some(&raw)&&self.entry_navigation.key.write_complete&&self.source_send.snapshot().pending.is_none(),"singleton C9/C8 not matched");
      let serial=self.java_self.selection_serial.context("singleton owned C9 write absent")?;
      ensure!(self.entry_navigation.key.pending_write==Some(serial),"singleton accepted C8 belongs to another selection write");
      self.java_self.binding_epoch=Some(issued.epoch.clone());self.java_self.key.alias=Some(QualifiedAlias{epoch:epoch.serial(),source:issued.source.clone(),source_revision:issued.source_revision,roster_revision:issued.roster_revision,selection_write_serial:serial,slot:0,raw_alias:raw,raw_count:issued.raw_count.clone(),raw_row:issued.raw_row.clone().context("singleton raw row absent")?});
     }
    }
   },_=>{},
  }Ok(())
 }
 pub(crate) fn start_qualified_self_exit_notification(&mut self)->Result<()>{
  ensure!(self.page==super::UiPage::Menu&&self.self_join_current()&&self.java_self.key.pending.is_none()
   &&self.source_send.snapshot().pending.is_none()&&self.entry_navigation.queue.is_empty()&&self.java_return.key.phase==super::java_return::Phase::Idle,"ordinary self notification requires independently qualified singleton join and unused intent");
  let prepared=self.source_send.prepare_exit_d0()?;let serial=prepared.reservation.ticket.serial();let context=self.java_self.key.joined.clone().unwrap();
  self.source_send.commit(prepared);self.java_self.key.pending=Some(PendingIntent{write_serial:serial,context,phase:IntentPhase::Queued});
  self.status="正在送出離房要求；仍待自身通知，未確認完成離房".into();Ok(())
 }
 pub(crate) fn note_self_exit_write(&mut self,serial:u64,progress:&source_send::Progress){
  let current=self.self_join_current();if let Some(intent)=self.java_self.key.pending.as_mut(){if !intent.report(serial,progress,current){self.java_self.key.pending=None;self.java_self.key.last_invalidation=Some("write failed or context retired");}}
 }
 pub(crate) fn prepare_self_notification(&self,receipt:&super::java_room_departure::Receipt)->(State,Option<u64>){
  let mut next=self.java_self.clone();if !self.self_join_current(){return(next,None);}
  let context=next.key.joined.clone().unwrap();let alias_matches=receipt.room_id==context.room_id&&self.java_self.key.alias.as_ref().is_some_and(|a|a.raw_alias==receipt.display_alias);
  let serial=next.key.pending.as_mut().and_then(|p|p.correlate(&context,alias_matches));
  if let Some(write_serial)=serial{next.key.notification=Some(Notification{context,write_serial,receipt:receipt.clone()});}(next,serial)
 }
}
#[cfg(test)]mod tests {
 use super::*;
 fn context()->JoinedContext{JoinedContext{room_id:1,ce_revision:9,navigation_revision:10,channel:2}}
 #[test]fn count_or_multirow_cannot_supply_singleton_alias(){
  let p=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("logs/continuation-20261003/r519-test-inputs.json");let v:serde_json::Value=serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();let hex=|s:&str|s.as_bytes().chunks_exact(2).map(|c|u8::from_str_radix(std::str::from_utf8(c).unwrap(),16).unwrap()).collect::<Vec<_>>();let row=hex(v["row_body"].as_str().unwrap());
  let empty=source_identity::State::default();assert_eq!(complete_singleton_alias(&empty).unwrap(),None);
  let(single,_)=empty.prepare_packet(&[3,1]).unwrap();assert_eq!(complete_singleton_alias(&single).unwrap(),None);let(single,_)=single.prepare_packet(&row).unwrap();assert_eq!(complete_singleton_alias(&single).unwrap(),Some(hex(v["self_alias_raw"].as_str().unwrap())));
  let(multi,_)=empty.prepare_packet(&[3,2]).unwrap();let(multi,_)=multi.prepare_packet(&row).unwrap();assert_eq!(complete_singleton_alias(&multi).unwrap(),None);let(multi,_)=multi.prepare_packet(&row).unwrap();assert!(multi.complete());assert_eq!(complete_singleton_alias(&multi).unwrap(),None);
  // This helper proves only bounded byte provenance shape. Accepted current
  // C9/C8/source/epoch and ordinary CE binding are still required in Runtime.
 }
 #[test]fn complete_full_frame_required_once(){
  let mut sender=source_send::Sender::default();sender.seed_from_source(0x87546ca110203040).unwrap();let prepared=sender.prepare_exit_d0().unwrap();assert_eq!(prepared.reservation.frame_bytes,3);let ticket=sender.commit(prepared).ticket;
  let mut intent=PendingIntent{write_serial:ticket.serial(),context:context(),phase:IntentPhase::Queued};
  for offset in 0..3{let view=sender.view(&ticket).unwrap();assert_eq!(view.bytes().unwrap().len(),3-offset);let progress=sender.report(&ticket,offset,source_send::WriteOutcome::Written(1)).unwrap();assert!(intent.report(ticket.serial(),&progress,true));if offset<2{assert_eq!(intent.phase,IntentPhase::Queued);assert_eq!(intent.correlate(&context(),true),None);}}
  assert_eq!(intent.phase,IntentPhase::CompletedWrite);assert_eq!(intent.correlate(&context(),false),None);let mut other=context();other.ce_revision+=1;assert_eq!(intent.correlate(&other,true),None);assert_eq!(intent.correlate(&context(),true),Some(ticket.serial()));assert_eq!(intent.correlate(&context(),true),None);
 }
 #[test]fn failed_partial_or_changed_context_cannot_complete(){
  for failure in [source_send::WriteOutcome::Failed,source_send::WriteOutcome::Written(0),source_send::WriteOutcome::WouldBlock]{
   let mut sender=source_send::Sender::default();sender.seed_from_source(1).unwrap();let prepared=sender.prepare_exit_d0().unwrap();let ticket=sender.commit(prepared).ticket;let mut intent=PendingIntent{write_serial:ticket.serial(),context:context(),phase:IntentPhase::Queued};
   let partial=sender.report(&ticket,0,source_send::WriteOutcome::Written(1)).unwrap();assert!(intent.report(ticket.serial(),&partial,true));let failed=sender.report(&ticket,1,failure).unwrap();assert!(!intent.report(ticket.serial(),&failed,true));assert_eq!(intent.correlate(&context(),true),None);
  }
  let mut intent=PendingIntent{write_serial:2,context:context(),phase:IntentPhase::Queued};assert!(intent.report(1,&source_send::Progress::Complete,true));assert_eq!(intent.phase,IntentPhase::Queued);assert!(!intent.report(2,&source_send::Progress::Complete,false));assert_eq!(intent.correlate(&context(),true),None);
 }
}
