//! Explicit AD368 D5/05 owner display and S_CF catalog deletion. Neither is
//! own leave, session ownership, Native table/UI or atomic commit authority.
use anyhow::{ensure,Context,Result};
use super::{Runtime,SourceEpoch,SourceDecodedPacket};
pub const SUPPLEMENT_SHA:&str="7774EA7546EEF33CB5240A58A36FA9FFF7A465303049C3C3E238081887279BEF";
pub const DYNAMIC_SHA:&str="4CB29A6DDCB95B4EEE9389AA1B36BF179F3415878F7361E7A7149ABE7F066517";
pub const NATIVE_SHA:&str="C8E20CD020392C0D26E21F8FB1B830B9009998B942CABF65EBDF59AA355EDA2C";
pub const MAX_CATALOG_ROWS:usize=256;
pub const MAX_DELETE_RECEIPTS:usize=1024;
pub const MAX_CATALOG_RECEIPTS:usize=1024;
#[derive(Clone)] pub struct Adapter;
impl Adapter {
 pub fn parse(bytes:&[u8])->Result<Self>{
  ensure!(bytes.len()<=1024,"room tail adapter exceeds bound");let v:serde_json::Value=serde_json::from_slice(bytes)?;
  ensure!(v["kind"]=="modern_synthetic_AD368_room_tail_v1"&&v["producer_sha256"]==super::login_bootstrap::PRODUCER_SHA
   &&v["supplement_sha256"]==SUPPLEMENT_SHA&&v["dynamic_sha256"]==DYNAMIC_SHA&&v["native_sha256"]==NATIVE_SHA,"pinned AD368 room tail adapter required");Ok(Self)
 }
}
#[derive(Clone,Debug,PartialEq,Eq)] pub struct OwnerReceipt {pub room_id:u32,pub display_alias:Vec<u8>,pub raw_body:Vec<u8>}
pub fn decode_owner(body:&[u8])->Result<OwnerReceipt>{
 ensure!((8..=34).contains(&body.len())&&body[0]==213&&body[1]==5,"only explicit AD368 S_D5 subtype5 qualified");
 let room_id=u32::from_le_bytes(body[2..6].try_into().unwrap());ensure!(room_id!=0,"owner room ID absent");
 let end=body[6..].iter().position(|b|*b==0).context("owner bounded CString absent")?+6;
 ensure!(end>6&&end==body.len()-1,"owner alias empty or trailing bytes");let display_alias=body[6..end].to_vec();
 super::java_room_departure::validate_cp950(&display_alias)?;Ok(OwnerReceipt{room_id,display_alias,raw_body:body.to_vec()})
}
#[derive(Clone,Debug,PartialEq,Eq)] pub enum OwnerLink {Absent,Unique{force:u8},Ambiguous{rows:usize}}
#[derive(Clone,Debug,PartialEq,Eq)] pub struct DeleteReceipt {pub room_id:u32,pub raw_body:Vec<u8>}
/// Server direction only. C_CF join includes its password CString and is
/// rejected here, even with the same numeric opcode.
pub fn decode_delete(body:&[u8])->Result<DeleteReceipt>{
 ensure!(body.len()==5&&body[0]==207,"S_CF requires exact server DWORD body5");
 let room_id=u32::from_le_bytes(body[1..5].try_into().unwrap());ensure!(room_id!=0,"catalog room ID absent");Ok(DeleteReceipt{room_id,raw_body:body.to_vec()})
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum CatalogOrigin {ExplicitObservation,ChannelReceipt}
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Completeness {PossiblyIncomplete}
#[derive(Clone,Debug,PartialEq,Eq)] pub enum CatalogReceipt {
 FullRow{row:super::java_room_catalog::Row,added:bool},Delete{receipt:DeleteReceipt,removed:usize},
}
#[derive(Clone,Debug,PartialEq,Eq)] pub struct CatalogSnapshot {
 pub epoch:u64,pub channel:u8,pub admission_revision:u64,pub origin:CatalogOrigin,
 pub revision:u64,pub room_ids:Vec<u32>,pub receipts:Vec<DeleteReceipt>,pub stream_arrivals:u64,
 pub rows:Vec<super::java_room_catalog::Row>,pub ordered_receipts:Vec<CatalogReceipt>,pub completeness:Completeness,
}
/// Standalone observations cannot be installed into Runtime. The live catalog
/// is created EMPTY by normal S_CA. CC0 adds absent IDs; duplicates preserve
/// the first full row. No implicit refresh/clear/upsert or completion inference.
#[derive(Clone)] pub struct Catalog {epoch:SourceEpoch,key:CatalogSnapshot}
impl Catalog {
 pub fn for_observation(epoch:&SourceEpoch,channel:u8,room_ids:Vec<u32>)->Result<Self>{Self::new(epoch,channel,0,CatalogOrigin::ExplicitObservation,room_ids)}
 fn new(epoch:&SourceEpoch,channel:u8,admission_revision:u64,origin:CatalogOrigin,room_ids:Vec<u32>)->Result<Self>{
  ensure!(channel<4&&room_ids.len()<=MAX_CATALOG_ROWS&&room_ids.iter().all(|id|*id!=0),"bounded channel/catalog IDs required");
  Ok(Self{epoch:epoch.clone(),key:CatalogSnapshot{epoch:epoch.serial(),channel,admission_revision,origin,revision:0,room_ids,receipts:vec![],stream_arrivals:0,rows:vec![],ordered_receipts:vec![],completeness:Completeness::PossiblyIncomplete}})
 }
 pub fn snapshot(&self)->CatalogSnapshot{self.key.clone()}
 pub fn observe_delete(&mut self,runtime:&Runtime,event:&SourceDecodedPacket)->Result<usize>{
  runtime.source_scope.admit_epoch(&self.epoch)?;runtime.source_scope.admit(event)?;
  let receipt=decode_delete(event.body())?;ensure!(self.key.ordered_receipts.len()<MAX_CATALOG_RECEIPTS,"catalog receipt budget reached");
  let revision=self.key.revision.checked_add(1).context("catalog receipt sequence exhausted")?;
  let count=self.key.room_ids.iter().filter(|id|**id==receipt.room_id).count();self.key.room_ids.retain(|id|*id!=receipt.room_id);
  self.key.rows.retain(|r|r.room_id!=receipt.room_id);
  self.key.ordered_receipts.push(CatalogReceipt::Delete{receipt:receipt.clone(),removed:count});
  self.key.receipts.push(receipt);self.key.revision=revision;Ok(count)
 }
 pub fn observe_full_row(&mut self,runtime:&Runtime,event:&SourceDecodedPacket)->Result<bool>{
  runtime.source_scope.admit_epoch(&self.epoch)?;runtime.source_scope.admit(event)?;
  let row=super::java_room_catalog::decode(event.body())?;
  ensure!(self.key.ordered_receipts.len()<MAX_CATALOG_RECEIPTS,"catalog receipt budget reached");
  let revision=self.key.revision.checked_add(1).context("catalog receipt sequence exhausted")?;
  let added=!self.key.room_ids.contains(&row.room_id);
  ensure!(!added||self.key.room_ids.len()<MAX_CATALOG_ROWS,"catalog row budget reached; view remains incomplete");
  if added{self.key.room_ids.push(row.room_id);self.key.rows.push(row.clone());}
  self.key.ordered_receipts.push(CatalogReceipt::FullRow{row,added});self.key.revision=revision;Ok(added)
 }
}
#[derive(Default)] pub(crate) struct State {pub(crate) adapter:Option<Adapter>,pub(crate) catalog_adapter:Option<super::java_room_catalog::Adapter>,pub(crate) catalog:Option<Catalog>}
impl State {
 pub(crate) fn retire(&mut self){self.catalog=None;}
 pub(crate) fn prepare_channel(&self,epoch:&SourceEpoch,channel:u8,revision:u64)->Result<Option<Catalog>>{
  if self.adapter.is_some(){Ok(Some(Catalog::new(epoch,channel,revision,CatalogOrigin::ChannelReceipt,vec![])?))}else{Ok(None)}
 }
 pub(crate) fn note_stream(&mut self){if let Some(c)=self.catalog.as_mut(){c.key.stream_arrivals+=1;}}
}
#[derive(Clone,Debug,PartialEq,Eq)] pub enum Change {Owner(super::java_room_participants::Change),CatalogDelete{room_id:u32,removed:usize,joined_room_invalidated:bool},CatalogFullRow{room_id:u32,added:bool}}
impl Runtime {
 pub fn configure_java_room_catalog_adapter(&mut self,adapter:super::java_room_catalog::Adapter)->Result<()>{
  ensure!(self.source_connection_epoch().is_none()&&self.java_tail.catalog.is_none()&&self.java_room.key.phase==super::java_room_entry::Phase::Idle,"catalog adapter must precede owned entry");self.java_tail.catalog_adapter=Some(adapter);Ok(())
 }
 pub fn configure_java_room_tail_adapter(&mut self,adapter:Adapter)->Result<()>{
  ensure!(self.source_connection_epoch().is_none()&&self.java_tail.catalog.is_none()&&self.java_room.key.phase==super::java_room_entry::Phase::Idle,"tail adapter must precede owned entry");self.java_tail.adapter=Some(adapter);Ok(())
 }
 pub fn java_room_catalog_snapshot(&self)->Option<CatalogSnapshot>{self.java_tail.catalog.as_ref().map(Catalog::snapshot)}
 pub fn last_java_room_catalog_checkpoint(&self)->Option<CatalogSnapshot>{self.last_tail_checkpoint.clone()}
 pub(crate) fn java_room_owner_route_enabled(&self)->bool{self.java_tail.adapter.is_some()&&self.java_room_participants_route_enabled()}
 pub(crate) fn java_room_catalog_route_enabled(&self)->bool{
  self.bootstrap_adapter.is_some()&&self.java_tail.adapter.is_some()&&self.java_tail.catalog.is_some()
   &&self.java_character.key.stage==super::java_character_flow::Stage::ChannelReceipts
 }
 pub(crate) fn java_room_catalog_full_row_route_enabled(&self)->bool{self.java_tail.catalog_adapter.is_some()&&self.java_room_catalog_route_enabled()}
 pub fn consume_java_room_tail_candidate(&mut self,event:&SourceDecodedPacket)->Result<Change>{
  self.source_scope.admit(event)?;
  if event.body().first()==Some(&213){ensure!(self.java_room_owner_route_enabled(),"owner display outside current private CE context");return self.consume_java_room_participants_candidate(event).map(Change::Owner);}
  ensure!(self.java_room_catalog_route_enabled(),"catalog receipt outside current explicit channel catalog");
  let info=self.java_character.key.channel_info.context("channel receipt absent")?;
  let catalog=self.java_tail.catalog.as_ref().context("channel catalog absent")?;
  ensure!(catalog.key.channel==info.channel&&catalog.key.origin==CatalogOrigin::ChannelReceipt,"retired channel catalog");
  if event.body().first()==Some(&204){
   ensure!(self.java_room_catalog_full_row_route_enabled(),"CC0 explicit adapter absent");
   let room_id=super::java_room_catalog::decode(event.body())?.room_id;let mut next=catalog.clone();let added=next.observe_full_row(self,event)?;
   self.java_tail.catalog=Some(next);self.status="已收到房間清單資料；清單可能尚不完整".into();return Ok(Change::CatalogFullRow{room_id,added});
  }
  let receipt=decode_delete(event.body())?;
  let mut next=catalog.clone();let removed=next.observe_delete(self,event)?;
  let mut book=self.java_participants.book.clone();let mut invalidated=false;
  if let Some(b)=book.as_mut(){if self.java_room.key.room.as_ref().is_some_and(|room|room.id==receipt.room_id){b.invalidate_joined(self)?;invalidated=true;}}
  let mut settings=self.java_settings.book.clone();
  if self.java_room.key.room.as_ref().is_some_and(|room|room.id==receipt.room_id){if let Some(b)=settings.as_mut(){b.invalidate_joined(self)?;invalidated=true;}}
  // Clone-before-commit: row deletion/diagnostic and joined invalidation are
  // installed together, after all scope/budget/revision checks succeeded.
  self.java_tail.catalog=Some(next);self.java_participants.book=book;self.java_settings.book=settings;
  if invalidated{self.java_self.invalidate_joined();self.invalidate_java_room_map_request();self.invalidate_java_room_settings_request();self.invalidate_java_room_mode_request();self.invalidate_java_room_ready_request();}
  self.status=if invalidated{"目前房間資料已失效；仍未確認離房"}else{"已收到房間清單刪除通知"}.into();
  Ok(Change::CatalogDelete{room_id:receipt.room_id,removed,joined_room_invalidated:invalidated})
 }
}
