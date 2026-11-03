//! Explicit AD368 return adapter; independent of Native B8/AFBD owners.
//! Retained raw names are expectations, never physical object/skill authority.
use super::{Runtime, SourceEpoch, SourceDecodedPacket, entry_navigation::{Scene,Request}};
use anyhow::{ensure,Context,Result};
use std::time::{Duration,Instant};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Intent { Refresh, Resume, Logout }
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum Phase {#[default] Idle, Confirming, RefreshWrites, AwaitRefresh, SourceWrite, Roster, QuitWrites, Retained, Authenticating, ResumeRoster, ResumeChoice, ResumeChannel }
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct Key {pub phase:Phase,pub intent:Option<Intent>,pub source:Option<Vec<u8>>,pub character:Option<Vec<u8>>,pub channel:Option<u8>,pub one_shot_consumed:bool}
#[derive(Clone)] pub struct Control {epoch:SourceEpoch,revision:u64,roster:u64,intent:Intent}
impl Control {pub(crate) fn epoch(&self)->SourceEpoch{self.epoch.clone()}}
#[derive(Clone)] pub struct Confirmation {control:Control,nonce:u64}
#[derive(Default)] pub(crate) struct State {
 pub(crate) key:Key, nonce:u64, confirmation:Option<u64>, arm_serial:Option<u64>,
 arm_deadline:Option<Instant>, pub(crate) expires:Option<Instant>,
 pub(crate) account:Option<[u8;32]>, pub(crate) endpoint:Option<std::net::SocketAddrV4>,
}
impl State {
 pub(crate) fn unexpired(&self)->bool {self.expires.is_some_and(|e|Instant::now()<e)}
 pub(crate) fn clear(&mut self){let n=self.nonce;*self=Self::default();self.nonce=n;}
}
#[derive(Clone,Debug)] pub enum Change {RefreshAllowed}
impl Runtime {
 pub fn java_return_snapshot(&self)->Key {self.java_return.key.clone()}
 pub fn java_return_control(&self,intent:Intent)->Option<Control>{
  if self.bootstrap_adapter.is_none()||self.page!=super::UiPage::Menu||self.java_return.key.phase!=Phase::Idle||self.source_send.snapshot().pending.is_some()||!self.entry_navigation.queue.is_empty(){return None;}
  if intent!=Intent::Logout && (self.java_room.key.phase!=super::java_room_entry::Phase::Idle||self.java_character.key.stage!=super::java_character_flow::Stage::ChannelReceipts||self.login_bootstrap.key.selected_name.is_none()||self.java_character.key.selected_name.is_none()){return None;}
  if intent==Intent::Resume && (self.java_return.account.is_none()||self.source_current_fixture_endpoint_v4().is_none()){return None;}
  Some(Control{epoch:self.source_scope.current_epoch().ok()?,revision:self.entry_navigation.key.revision,roster:self.source_roster_revision,intent})
 }
 fn admit_java_return_control(&self,c:&Control)->Result<()> {
  self.source_scope.admit_epoch(&c.epoch)?;
  ensure!(c.revision==self.entry_navigation.key.revision&&c.roster==self.source_roster_revision&&self.java_return_control(c.intent).is_some(),"retired or unavailable AD368 return control");Ok(())
 }
 /// Confirmation itself sends nothing. Tokens cannot be forged by UI callers.
 pub fn request_java_return_confirmation(&mut self,c:&Control)->Result<Confirmation>{
  self.admit_java_return_control(c)?;let n=self.java_return.nonce.checked_add(1).context("confirmation nonce exhausted")?;
  self.java_return.nonce=n;self.java_return.confirmation=Some(n);self.java_return.key.phase=Phase::Confirming;
  Ok(Confirmation{control:c.clone(),nonce:n})
 }
 pub fn resolve_java_return_confirmation(&mut self,c:&Confirmation,accepted:bool)->Result<()> {
  self.source_scope.admit_epoch(&c.control.epoch)?;
  ensure!(self.java_return.confirmation==Some(c.nonce)&&self.java_return.key.phase==Phase::Confirming&&c.control.revision==self.entry_navigation.key.revision&&c.control.roster==self.source_roster_revision,"retired confirmation");
  self.java_return.confirmation=None;self.java_return.key.phase=Phase::Idle;
  if accepted {self.start_java_return(&c.control)?;}Ok(())
 }
 pub(crate) fn start_java_return(&mut self,c:&Control)->Result<()> {
  self.admit_java_return_control(c)?;
  let requests=match c.intent {Intent::Refresh=>vec![Request::RosterCB,Request::Channel(100)],Intent::Resume=>vec![Request::ExitD0,Request::Return15],Intent::Logout=>vec![Request::Return15]};
  let source=self.login_bootstrap.key.selected_name.clone();let character=self.java_character.key.selected_name.clone();let channel=self.java_character.key.channel_info.map(|i|i.channel);
  self.begin_source_navigation_sequence(Scene::AwaitTransferReply,requests)?;
  self.java_return.key=Key{phase:if c.intent==Intent::Refresh{Phase::RefreshWrites}else{Phase::QuitWrites},intent:Some(c.intent),source,character,channel,one_shot_consumed:false};
  self.java_return.endpoint=self.source_current_fixture_endpoint_v4();
  self.java_return.arm_serial=if c.intent==Intent::Resume{self.entry_navigation.key.pending_write}else{None};
  self.status="正在返回，等待傳送完成…".into();Ok(())
 }
 pub(crate) fn finish_java_return_write(&mut self,serial:u64)->Result<()> {
  if self.java_return.key.phase==Phase::QuitWrites&&self.java_return.arm_serial==Some(serial){self.java_return.arm_deadline=Some(Instant::now()+Duration::from_secs(30));return Ok(());}
  if !self.entry_navigation.queue.is_empty()||self.source_send.snapshot().pending.is_some(){return Ok(());}
  match self.java_return.key.phase {
   Phase::RefreshWrites=>self.java_return.key.phase=Phase::AwaitRefresh,
   Phase::SourceWrite=>{self.entry_navigation.retire()?;self.java_return.key.phase=Phase::Roster;self.login_bootstrap.key.stage=super::login_bootstrap::Stage::AwaitRoster;self.awaiting_source_roster=true;},
   Phase::QuitWrites=>{
    if self.java_return.key.intent==Some(Intent::Logout){self.handle(super::Action::Close)?;return Ok(());}
    ensure!(self.java_return.arm_deadline.is_some_and(|e|Instant::now()<e),"D0 quit window expired; no retained handoff");
    let mut retained=std::mem::take(&mut self.java_return);retained.key.phase=Phase::Retained;retained.expires=Some(Instant::now()+Duration::from_secs(30));
    let e=self.source_scope.current_epoch()?;self.disconnect_source_connection_candidate(&e)?;self.java_return=retained;
    self.entry_navigation.key.scene=Scene::ReconnectRequired;self.status="舊連線已結束；重新登入後可驗證返回資料".into();
   },_=>{},
  }Ok(())
 }
 pub(crate) fn consume_java_return_event(&mut self,event:&SourceDecodedPacket)->Result<Change>{
  self.source_scope.admit(event)?;ensure!(self.bootstrap_adapter.is_some()&&event.body()==[245]&&self.java_return.key.phase==Phase::AwaitRefresh&&self.source_send.snapshot().pending.is_none()&&self.entry_navigation.key.write_complete,"S_F5 outside completed CB/CA100 return");
  let name=self.java_return.key.source.as_ref().context("retained source absent")?.clone();
  self.begin_source_navigation_sequence(Scene::AwaitTransferRoster,vec![Request::Source(name)])?;
  self.java_return.key.phase=Phase::SourceWrite;self.status="重新取得角色名單…".into();Ok(Change::RefreshAllowed)
 }
 pub(crate) fn finish_java_return_roster(&mut self)->Result<()> {
  if !self.source_identity.complete(){return Ok(());}
  match self.java_return.key.phase {
   Phase::Roster=>{self.java_return.key=Key::default();},
   Phase::ResumeRoster=>{
    ensure!(self.java_return.unexpired(),"retained channel handoff expired");
    let name=self.java_return.key.character.as_ref().context("retained character absent")?;
    ensure!(self.source_identity.records().iter().any(|r|r.name()==name.as_slice()),"retained character absent from fresh roster");
    // NEWS/roster have no silent-selection ack. This remains an explicit
    // adapter expectation until matching S_C9/S_CA, with no C9/S7C replay.
    self.java_return.key.phase=Phase::ResumeChoice;
    self.java_character.key.stage=super::java_character_flow::Stage::ResumeChooseChannel;
    self.java_character.key.selected_name=Some(name.clone());
    self.entry_navigation.key.scene=Scene::Channel;self.character_scene.retire()?;
   },_=>{},
  }Ok(())
 }
}
