//! Independent bounded AD368 D5/0..3 receipts. No edit acknowledgement,
//! coherent pair, readiness, physical owner or Native globals are inferred.
use anyhow::{ensure,Context,Result};
use super::{Runtime,SourceEpoch,SourceDecodedPacket};
pub const EDIT_SHA:&str="A13ECBFE9DC9BFCBA1D32649C829FB960DA995591C59C8613BB403FA4D62C168";
pub const LAYOUT_SHA:&str="7774EA7546EEF33CB5240A58A36FA9FFF7A465303049C3C3E238081887279BEF";
pub const NATIVE_SHA:&str="8A6C74A34CFAC1BC1AFF04215F80E347853069ACFA611C076269443B4671DFD8";
pub const MODE_MAP_SHA:&str="9F1B9863C7F0316A4C580BA83DCD14CA92A366418555073FB4FB1181F544ABAE";
pub const SCALAR_BOUNDARY_SHA:&str="4BBBEEB521ECF20E6B27D78F03B25F503A5766412DA9E3FBEE3DC11A7C389BAA";
pub const MAX_RECEIPTS:usize=1024;
#[derive(Clone)]pub struct Adapter {producer:&'static str}
impl Adapter {
 pub fn producer_sha256(&self)->&'static str{self.producer}
 pub fn parse(bytes:&[u8])->Result<Self>{ensure!(bytes.len()<=1024,"settings adapter exceeds bound");let v:serde_json::Value=serde_json::from_slice(bytes)?;ensure!(v["layout_sha256"]==LAYOUT_SHA&&v["native_sha256"]==NATIVE_SHA,"pinned independent settings layout required");let producer=if v["kind"]=="modern_synthetic_AD368_room_settings_v1"&&v["producer_sha256"]==super::login_bootstrap::PRODUCER_SHA&&v["edit_sha256"]==EDIT_SHA{super::login_bootstrap::PRODUCER_SHA}else{ensure!(v["kind"]=="modern_synthetic_485F_room_settings_v1"&&v["producer_sha256"]==super::java_room_settings_request::SETTINGS_485F_SHA&&v["network_sha256"]==super::java_room_settings_request::SETTINGS_485F_NETWORK_SHA,"explicit pinned485F independent0/1 adapter required");super::java_room_settings_request::SETTINGS_485F_SHA};Ok(Self{producer})}
}
/// Mode is a structural observation; its production trigger is unqualified.
/// The scalar boundary pin grants no behavior for D5/4 or D5/6..9.
#[derive(Clone)]pub struct ModeMapAdapter {producer:&'static str}
impl ModeMapAdapter {
 pub fn producer_sha256(&self)->&'static str{self.producer}
 pub fn parse(bytes:&[u8])->Result<Self>{ensure!(bytes.len()<=1024,"mode/map adapter exceeds bound");let v:serde_json::Value=serde_json::from_slice(bytes)?;ensure!(v["layout_sha256"]==LAYOUT_SHA&&v["native_sha256"]==MODE_MAP_SHA&&v["scalar_boundary_sha256"]==SCALAR_BOUNDARY_SHA,"pinned independent2/3 layout required");let producer=if v["kind"]=="modern_synthetic_AD368_room_mode_map_v1"&&v["producer_sha256"]==super::login_bootstrap::PRODUCER_SHA{super::login_bootstrap::PRODUCER_SHA}else{ensure!(v["kind"]=="modern_synthetic_485F_room_mode_map_v1"&&v["producer_sha256"]==super::java_room_mode_request::PRODUCER_SHA&&v["network_sha256"]==super::java_room_mode_request::NETWORK_SHA&&v["mode_step_sha256"]==super::java_room_mode_request::NATIVE_SHA,"explicit pinned485F mode/map receipt adapter required");super::java_room_mode_request::PRODUCER_SHA};Ok(Self{producer})}
}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Summary {
 pub room_id:u32,pub name:Vec<u8>,pub option0:u8,
 /// Full wire WORD; no Native BYTE arithmetic, index or resource binding.
 pub map_word:u16,pub summary:[u8;5],pub owner_alias:Vec<u8>,pub raw_body:Vec<u8>,
}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Configuration {pub room_id:u32,pub name:Vec<u8>,pub password:Vec<u8>,pub attributes:u8,pub capacity:u8,pub options:[u8;4],pub selections:Vec<u8>,pub raw_body:Vec<u8>}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct ModeReceipt {pub room_id:u32,pub raw_mode:u8,pub raw_body:Vec<u8>}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct MapReceipt {pub room_id:u32,pub raw_map_word:u16,pub raw_body:Vec<u8>}
#[derive(Clone,Debug,PartialEq,Eq)]pub enum Packet {Summary(Summary),Configuration(Configuration),Mode(ModeReceipt),Map(MapReceipt)}
impl Packet {pub fn room_id(&self)->u32{match self{Self::Summary(v)=>v.room_id,Self::Configuration(v)=>v.room_id,Self::Mode(v)=>v.room_id,Self::Map(v)=>v.room_id}}}
/// Strict cd+c or cd+h LE only. Values remain raw, without domain masks,
/// Native mode high-BYTE assumptions, map projection or effective CE writes.
pub fn decode_mode_map(body:&[u8])->Result<Packet>{
 ensure!(matches!(body.len(),7|8)&&body[0]==213&&matches!(body[1],2|3),"only exact D5/2/3 mode/map qualified");
 ensure!(body.len()==if body[1]==2{7}else{8},"mode/map field width or trailing bytes");let room_id=u32::from_le_bytes(body[2..6].try_into().unwrap());ensure!(room_id!=0,"mode/map room ID absent");
 Ok(if body[1]==2{Packet::Mode(ModeReceipt{room_id,raw_mode:body[6],raw_body:body.to_vec()})}else{Packet::Map(MapReceipt{room_id,raw_map_word:u16::from_le_bytes([body[6],body[7]]),raw_body:body.to_vec()})})
}
struct Cursor<'a>{body:&'a[u8],at:usize}
impl Cursor<'_>{
 fn take(&mut self,n:usize)->Result<&[u8]>{let end=self.at.checked_add(n).context("settings offset overflow")?;let s=self.body.get(self.at..end).context("short settings field")?;self.at=end;Ok(s)}
 fn c(&mut self)->Result<u8>{Ok(self.take(1)?[0])}
 fn z(&mut self,max:usize,empty:bool)->Result<Vec<u8>>{let n=self.body[self.at..].iter().take(max+1).position(|b|*b==0).context("bounded settings CString absent")?;ensure!(empty||n>0,"empty settings identity string");let v=self.take(n+1)?[..n].to_vec();ensure!(!v.iter().any(|b|*b<32||*b==127),"settings control bytes outside Modern subset");if !v.is_empty(){validate_cp950(&v)?;}Ok(v)}
}
#[cfg(windows)]fn validate_cp950(raw:&[u8])->Result<()>{use windows::Win32::Globalization::{MultiByteToWideChar,MB_ERR_INVALID_CHARS};let mut wide=[0u16;89];ensure!(unsafe{MultiByteToWideChar(950,MB_ERR_INVALID_CHARS,raw,Some(&mut wide))}>0,"invalid settings CP950 bytes");Ok(())}
#[cfg(not(windows))]fn validate_cp950(raw:&[u8])->Result<()>{ensure!(raw.is_ascii(),"CP950 validation unavailable");Ok(())}
/// Exact Java layout, with an explicit bounded Modern string subset. This
/// does not emulate Native string externals or its malformed signed-K loop.
pub fn decode(body:&[u8])->Result<Packet>{
 ensure!((16..=132).contains(&body.len())&&body[0]==213&&matches!(body[1],0|1),"only bounded D5/0/1 settings qualified");
 let mut c=Cursor{body,at:2};let room_id=u32::from_le_bytes(c.take(4)?.try_into().unwrap());ensure!(room_id!=0,"settings room ID absent");let name=c.z(89,false)?;
 let packet=if body[1]==0 {let option0=c.c()?;let map_word=u16::from_le_bytes(c.take(2)?.try_into().unwrap());let first=c.c()?;let owner_alias=c.z(27,false)?;let mut summary=[0;5];summary[0]=first;summary[1..].copy_from_slice(c.take(4)?);Packet::Summary(Summary{room_id,name,option0,map_word,summary,owner_alias,raw_body:body.to_vec()})}
 else {ensure!(body.len()<=116,"configuration envelope exceeded");let password=c.z(4,true)?;let attributes=c.c()?;let capacity=c.c()?;ensure!((2..=200).contains(&capacity),"configuration BYTE capacity outside producer domain");let options:[u8;4]=c.take(4)?.try_into().unwrap();let count=c.c()?;ensure!(count<=8,"configuration selection count exceeds eight");let selections=c.take(count as usize)?.to_vec();ensure!(selections.iter().all(|v|*v!=0),"zero selection outside producer domain");ensure!((options==[0;4]&&selections.is_empty())||(options[0]<=7&&options[1]<=2&&(1..=127).contains(&options[2])&&options[3]<=31),"configuration option domain mismatch");Packet::Configuration(Configuration{room_id,name,password,attributes,capacity,options,selections,raw_body:body.to_vec()})};
 ensure!(c.at==body.len(),"settings trailing bytes");Ok(packet)
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Origin {ExplicitObservation,JoinedCeReceipt}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct JoinedContext {pub ce_revision:u64,pub source:Vec<u8>,pub source_revision:u64,pub roster_revision:u64,pub selected_slot:usize,pub selected_alias:Vec<u8>,pub navigation_revision:u64,pub channel:u8}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Receipt {pub sequence:u64,pub subtype_revision:u64,pub packet:Packet}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Snapshot {
 pub epoch:u64,pub room_id:u32,pub origin:Origin,pub context:Option<JoinedContext>,pub revision:u64,
 pub summary_revision:u64,pub configuration_revision:u64,pub summary:Option<Summary>,pub configuration:Option<Configuration>,
 pub mode_revision:u64,pub map_revision:u64,pub mode:Option<ModeReceipt>,pub map:Option<MapReceipt>,
 pub receipts:Vec<Receipt>,pub stream_arrivals:u64,pub joined_room_invalidated:bool,
}
#[derive(Clone)]pub struct Book {epoch:SourceEpoch,key:Snapshot}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Change {SummaryReceived{revision:u64},ConfigurationReceived{revision:u64},ModeReceived{revision:u64},MapReceived{revision:u64}}
impl Book {
 pub fn for_observation(epoch:&SourceEpoch,room_id:u32)->Result<Self>{Self::new(epoch,room_id,None)}
 fn new(epoch:&SourceEpoch,room_id:u32,context:Option<JoinedContext>)->Result<Self>{ensure!(room_id!=0,"settings scoped room absent");Ok(Self{epoch:epoch.clone(),key:Snapshot{epoch:epoch.serial(),room_id,origin:if context.is_some(){Origin::JoinedCeReceipt}else{Origin::ExplicitObservation},context,revision:0,summary_revision:0,configuration_revision:0,summary:None,configuration:None,mode_revision:0,map_revision:0,mode:None,map:None,receipts:vec![],stream_arrivals:0,joined_room_invalidated:false}})}
 pub fn snapshot(&self)->Snapshot{self.key.clone()}
 pub fn observe(&mut self,runtime:&Runtime,event:&SourceDecodedPacket)->Result<Change>{
  runtime.source_scope.admit_epoch(&self.epoch)?;runtime.source_scope.admit(event)?;ensure!(!self.key.joined_room_invalidated,"settings joined room invalidated");
  let packet=if matches!(event.body().get(1),Some(2|3)){decode_mode_map(event.body())?}else{decode(event.body())?};ensure!(packet.room_id()==self.key.room_id,"settings room differs from owned scope");ensure!(self.key.receipts.len()<MAX_RECEIPTS,"settings receipt budget reached");let seq=self.key.revision.checked_add(1).context("settings sequence exhausted")?;
  let (change,revision)=match &packet {Packet::Summary(v)=>{let n=self.key.summary_revision.checked_add(1).context("summary revision exhausted")?;self.key.summary=Some(v.clone());self.key.summary_revision=n;(Change::SummaryReceived{revision:n},n)},Packet::Configuration(v)=>{let n=self.key.configuration_revision.checked_add(1).context("configuration revision exhausted")?;self.key.configuration=Some(v.clone());self.key.configuration_revision=n;(Change::ConfigurationReceived{revision:n},n)},Packet::Mode(v)=>{let n=self.key.mode_revision.checked_add(1).context("mode revision exhausted")?;self.key.mode=Some(v.clone());self.key.mode_revision=n;(Change::ModeReceived{revision:n},n)},Packet::Map(v)=>{let n=self.key.map_revision.checked_add(1).context("map revision exhausted")?;self.key.map=Some(v.clone());self.key.map_revision=n;(Change::MapReceived{revision:n},n)}};
  self.key.receipts.push(Receipt{sequence:seq,subtype_revision:revision,packet});self.key.revision=seq;Ok(change)
 }
 pub(crate) fn invalidate_joined(&mut self,runtime:&Runtime)->Result<()>{runtime.source_scope.admit_epoch(&self.epoch)?;ensure!(self.key.origin==Origin::JoinedCeReceipt&&self.key.context.as_ref().is_some_and(|c|c.ce_revision==runtime.java_room.key.revision)&&runtime.java_room.key.room.as_ref().is_some_and(|r|r.id==self.key.room_id),"retired settings invalidation");self.key.joined_room_invalidated=true;Ok(())}
}
#[derive(Default)]pub(crate) struct State {pub(crate) adapter:Option<Adapter>,pub(crate) mode_map_adapter:Option<ModeMapAdapter>,pub(crate) book:Option<Book>}
impl State {pub(crate) fn retire(&mut self){self.book=None;}pub(crate) fn note_stream(&mut self){if let Some(b)=self.book.as_mut(){b.key.stream_arrivals+=1;}}}
impl Runtime {
 pub fn configure_java_room_settings_adapter(&mut self,adapter:Adapter)->Result<()>{ensure!(self.source_connection_epoch().is_none()&&self.java_room.key.phase==super::java_room_entry::Phase::Idle,"settings adapter must precede normal entry");self.java_settings.adapter=Some(adapter);Ok(())}
 pub fn configure_java_room_mode_map_adapter(&mut self,adapter:ModeMapAdapter)->Result<()>{ensure!(self.source_connection_epoch().is_none()&&self.java_room.key.phase==super::java_room_entry::Phase::Idle,"mode/map adapter must precede normal entry");self.java_settings.mode_map_adapter=Some(adapter);Ok(())}
 pub fn java_room_settings_snapshot(&self)->Option<Snapshot>{self.java_settings.book.as_ref().map(Book::snapshot)}
 pub fn last_java_room_settings_checkpoint(&self)->Option<Snapshot>{self.last_settings_checkpoint.clone()}
 /// Dispatch is intercepted even when context is missing or invalidated, so
 /// explicit D5/0/1 never falls through into generic Native side effects.
 pub(crate) fn java_room_settings_dispatch(&self,body:&[u8])->bool{self.java_settings.adapter.is_some()&&body.first()==Some(&213)&&matches!(body.get(1),Some(0|1))}
 pub(crate) fn java_room_mode_map_dispatch(&self,body:&[u8])->bool{self.java_settings.mode_map_adapter.is_some()&&body.first()==Some(&213)&&matches!(body.get(1),Some(2|3))}
 pub(crate) fn prepare_settings_joined(&self,room_id:u32,ce_revision:u64,navigation_revision:u64)->Result<Option<Book>>{
  if self.java_settings.adapter.is_none()&&self.java_settings.mode_map_adapter.is_none(){return Ok(None);}let selected=self.source_identity.selected().context("settings selected source row absent")?;
  ensure!(self.bootstrap_adapter.is_some()&&self.login_bootstrap.key.stage==super::login_bootstrap::Stage::Roster&&self.source_identity.complete()&&self.java_character.key.character_result==Some(0)&&self.java_character.key.selected_name.as_deref()==Some(selected.actor_alias()),"settings normal source/selection not qualified");
  let context=JoinedContext{ce_revision,source:self.login_bootstrap.key.selected_name.clone().context("settings selected source absent")?,source_revision:self.login_bootstrap.key.revision,roster_revision:self.source_roster_revision,selected_slot:selected.slot(),selected_alias:selected.actor_alias().to_vec(),navigation_revision,channel:self.java_character.key.channel_info.context("settings channel absent")?.channel};Ok(Some(Book::new(&self.source_scope.current_epoch()?,room_id,Some(context))?))
 }
 pub fn consume_java_room_settings_candidate(&mut self,event:&SourceDecodedPacket)->Result<Change>{
  self.source_scope.admit(event)?;ensure!(self.java_settings.adapter.is_some(),"D5/0/1 adapter absent");decode(event.body())?;self.consume_scoped_settings(event)
 }
 pub fn consume_java_room_mode_map_candidate(&mut self,event:&SourceDecodedPacket)->Result<Change>{
  self.source_scope.admit(event)?;ensure!(self.java_settings.mode_map_adapter.is_some(),"D5/2/3 adapter absent");decode_mode_map(event.body())?;self.consume_scoped_settings(event)
 }
 fn consume_scoped_settings(&mut self,event:&SourceDecodedPacket)->Result<Change>{
  self.source_scope.admit(event)?;ensure!(self.bootstrap_adapter.is_some()&&self.java_room.adapter.is_some()&&self.java_room.key.phase==super::java_room_entry::Phase::RoomReceipts&&self.entry_navigation.key.scene==super::entry_navigation::Scene::RoomReceipts,"settings outside explicit normal room scope");
  self.admit_java_room_settings_context()?;let book=self.java_settings.book.as_ref().unwrap();
  let mut next=book.clone();let change=next.observe(self,event)?;self.java_settings.book=Some(next);self.status="已收到房間設定資料；修改結果仍未確認".into();Ok(change)
 }
 pub(crate) fn admit_java_room_settings_context(&self)->Result<JoinedContext>{
  ensure!(self.bootstrap_adapter.is_some()&&self.java_room.adapter.is_some()&&self.java_room.key.phase==super::java_room_entry::Phase::RoomReceipts&&self.entry_navigation.key.scene==super::entry_navigation::Scene::RoomReceipts,"settings outside explicit normal room scope");
  let book=self.java_settings.book.as_ref().context("private CE settings book absent")?;self.source_scope.admit_epoch(&book.epoch)?;let c=book.key.context.as_ref().context("joined settings context absent")?;
  ensure!(book.key.origin==Origin::JoinedCeReceipt&&!book.key.joined_room_invalidated&&c.ce_revision==self.java_room.key.revision&&self.java_room.key.room.as_ref().is_some_and(|r|r.id==book.key.room_id)
   &&self.login_bootstrap.key.stage==super::login_bootstrap::Stage::Roster&&self.login_bootstrap.key.revision==c.source_revision&&self.login_bootstrap.key.selected_name.as_ref()==Some(&c.source)&&self.source_roster_revision==c.roster_revision&&self.source_identity.complete()
   &&self.source_identity.selected().is_some_and(|s|s.slot()==c.selected_slot&&s.actor_alias()==c.selected_alias)&&self.java_character.key.character_result==Some(0)&&self.java_character.key.selected_name.as_ref()==Some(&c.selected_alias)&&self.entry_navigation.key.revision==c.navigation_revision&&self.java_character.key.channel_info.is_some_and(|i|i.channel==c.channel),"retired settings source/selection/room context");
  Ok(c.clone())
 }
}
