from r491_common import *
p=P/'src/modern_lito/native_entry_navigation.rs';s=p.read_text(encoding='utf-8')
s=s.replace('Dormant, AwaitCharacterReply, AwaitAutomaticLobbyReply, Lobby, ReconnectRequired','Dormant, AwaitCharacterReply, AwaitAutomaticLobbyReply, AwaitTransferReply, AwaitTransferRoster, Channel, AwaitChannelReply, Lobby, ReconnectRequired')
s=s.replace('AutomaticLobbyRequest, LobbyEntered','AutomaticLobbyRequest, ChannelEntered, LobbyEntered')
s=s[:s.index('#[derive(Clone, Default)]')]+r'''
#[derive(Clone)]pub(crate) enum Request {NoticeD9,RosterCB,Channel(u8),Return15,Character(Vec<u8>)}
impl Request {
 fn prepare(&self,s:&super::source_send::Sender)->Result<super::source_send::Prepared>{match self{Self::NoticeD9=>s.prepare_notice_d9(),Self::RosterCB=>s.prepare_roster_cb(),Self::Channel(c)=>s.prepare_channel(*c),Self::Return15=>s.prepare_return_login(),Self::Character(name)=>s.prepare_character(name)}}
}
#[derive(Clone, Default)]
pub(crate) struct State { pub key: Key,pub(crate) queue:std::collections::VecDeque<Request> }
impl State {
 pub fn retire(&mut self)->Result<()> {let revision=self.key.revision.checked_add(1).context("navigation revision exhausted")?;self.key=Key {revision,..Key::default()};self.queue.clear();Ok(())}
 pub fn await_character(&mut self,serial:u64)->Result<()> {ensure!(self.key.scene==Scene::Dormant,"navigation request already active");self.retire()?;self.key.scene=Scene::AwaitCharacterReply;self.key.pending_write=Some(serial);Ok(())}
 pub fn written(&mut self,serial:u64){if self.key.pending_write==Some(serial){self.key.write_complete=self.queue.is_empty();}}
 pub fn active(&self)->bool {self.key.scene!=Scene::Dormant}
}
impl super::Runtime {
 pub fn source_navigation_snapshot(&self)->Key {self.entry_navigation.key.clone()}
 pub(crate) fn begin_source_navigation_sequence(&mut self,scene:Scene,requests:Vec<Request>)->Result<()> {
  ensure!((1..=3).contains(&requests.len())&&self.entry_navigation.queue.is_empty()&&self.source_send.snapshot().pending.is_none(),"bounded navigation sequence unavailable");
  let prepared=requests[0].prepare(&self.source_send)?;let serial=prepared.reservation.ticket.serial();let mut nav=self.entry_navigation.clone();let (state,mode)=(nav.key.return_state,nav.key.return_mode);nav.retire()?;
  nav.key.scene=scene;nav.key.return_state=state;nav.key.return_mode=mode;nav.key.pending_write=Some(serial);nav.queue=requests.into_iter().skip(1).collect();let mut character=self.character_scene.clone();character.retire()?;
  self.source_send.commit(prepared);self.entry_navigation=nav;self.character_scene=character;Ok(())
 }
 pub(crate) fn drive_source_navigation_sequence(&mut self)->Result<()> {
  if self.source_send.snapshot().pending.is_some(){return Ok(());}
  if let Some(request)=self.entry_navigation.queue.front(){let prepared=request.prepare(&self.source_send)?;let serial=prepared.reservation.ticket.serial();self.source_send.commit(prepared);self.entry_navigation.queue.pop_front();self.entry_navigation.key.pending_write=Some(serial);self.entry_navigation.key.write_complete=false;}
  Ok(())
 }
 pub(crate) fn finish_source_navigation_write(&mut self)->Result<()> {
  self.drive_source_navigation_sequence()?;
  if self.source_send.snapshot().pending.is_none()&&self.entry_navigation.queue.is_empty()&&self.entry_navigation.key.write_complete {
   if let Some(target)=self.entry_transfer.reconnect_after_write {self.reconnect_source_transfer_fixture(target)?;}
  }
  Ok(())
 }
 pub fn source_channel_control(&self,slot:usize)->Option<super::SourceChannelControl> {
  if slot>=4||self.page!=super::UiPage::Menu||self.entry_navigation.key.scene!=Scene::Channel||self.entry_transfer.key.timer_active||self.source_send.snapshot().pending.is_some(){return None;}
  if self.skin.as_ref()?.channel_buttons.len()!=8{return None;}
  Some(super::SourceChannelControl{epoch:self.source_scope.current_epoch().ok()?,revision:self.entry_navigation.key.revision,slot})
 }
 pub fn handle_source_channel_control(&mut self,c:&super::SourceChannelControl)->Result<()> {
  self.source_scope.admit_epoch(&c.epoch)?;ensure!(c.revision==self.entry_navigation.key.revision&&self.source_channel_control(c.slot).is_some(),"retired or unavailable channel callback");
  //630B20 four receiver indices ->492FA0 cc202,index.
  self.begin_source_navigation_sequence(Scene::AwaitChannelReply,vec![Request::Channel(c.slot as u8)])?;self.entry_navigation.key.channel=Some(c.slot as u8);self.status="正在進入所選頻道…".into();Ok(())
 }
 pub fn source_lobby_back(&self)->Option<super::SourceLobbyBack> {
  if self.page!=super::UiPage::Menu||self.entry_navigation.key.scene!=Scene::Lobby||self.entry_navigation.key.return_state!=1||self.entry_transfer.key.timer_active||self.source_send.snapshot().pending.is_some(){return None;}
  Some(super::SourceLobbyBack {epoch:self.source_scope.current_epoch().ok()?,revision:self.entry_navigation.key.revision,roster_revision:self.source_roster_revision})
 }
 pub fn handle_source_lobby_back(&mut self,c:&super::SourceLobbyBack)->Result<()> {
  self.source_scope.admit_epoch(&c.epoch)?;ensure!(c.revision==self.entry_navigation.key.revision&&c.roster_revision==self.source_roster_revision&&self.source_lobby_back().is_some(),"retired lobby Back callback");
  //455330 ->46F010 B8=1: cCB then ccCA100; AFBD=1,B8=0; BE unchanged.
  self.begin_source_navigation_sequence(Scene::AwaitAutomaticLobbyReply,vec![Request::RosterCB,Request::Channel(100)])?;self.entry_transfer.return_one();self.entry_navigation.key.return_state=0;self.status="正在返回，等待伺服器回覆…".into();Ok(())
 }
 pub fn consume_source_navigation_event(&mut self,event:&super::SourceDecodedPacket)->Result<Change> {
  self.source_scope.admit(event)?;ensure!(self.page==super::UiPage::Menu,"navigation scene unavailable");
  let reply=decode(event.body())?;let k=&self.entry_navigation.key;
  let matching=matches!((k.scene,reply),(Scene::AwaitCharacterReply|Scene::AwaitTransferReply,Reply::Character{..}|Reply::Lobby{..})|(Scene::AwaitAutomaticLobbyReply|Scene::AwaitChannelReply,Reply::Lobby{..}));
  if !matching||!k.write_complete||!self.entry_navigation.queue.is_empty()||self.source_send.snapshot().pending.is_some(){self.entry_navigation.key.ignored_replies=self.entry_navigation.key.ignored_replies.checked_add(1).context("ignored navigation replies overflow")?;return Ok(Change::Ignored);}
  ensure!(self.source_identity.selected().is_some()||self.entry_transfer.alias().is_some(),"navigation selected source character absent");
  let status=match reply {Reply::Character {status}|Reply::Lobby {status,..}=>status};let opcode=event.body()[0];
  match status_effect(reply) {
   StatusEffect::NativeIgnore=>{self.entry_navigation.key.ignored_replies=self.entry_navigation.key.ignored_replies.checked_add(1).context("ignored navigation replies overflow")?;return Ok(Change::IgnoredStatus {opcode,status});},
   StatusEffect::NativeAlert=>{let epoch=self.source_scope.current_epoch()?;self.disconnect_source_connection_candidate(&epoch)?;self.entry_navigation.key.scene=Scene::ReconnectRequired;self.entry_navigation.key.last_alert=Some((opcode,status));self.status="進入未成功。上一連線已結束，請重新連線後再選擇角色。".into();return Ok(Change::RestartAfterNativeAlert {opcode,status});},
   StatusEffect::Success=>{},
  }
  match reply {
   Reply::Character {status:0}=>{
    if let Some(state)=self.entry_transfer.character_success(){self.entry_navigation.key.return_state=state;if state==2 {
     self.character_scene.retire()?;self.entry_navigation.key.revision=self.entry_navigation.key.revision.checked_add(1).context("navigation revision exhausted")?;self.entry_navigation.key.scene=Scene::Channel;self.entry_navigation.key.pending_write=None;self.entry_navigation.key.write_complete=false;self.entry_navigation.key.return_mode=0;self.source_flow=self.source_flow.channel_reply_received();self.entry_transfer.complete();self.status="請選擇頻道；切換完成後可操作。".into();return Ok(Change::ChannelEntered);
    }}
    self.begin_source_navigation_sequence(Scene::AwaitAutomaticLobbyReply,vec![Request::Channel(100)])?;self.status="等待大廳回包…".into();Ok(Change::AutomaticLobbyRequest)
   },
   Reply::Lobby {status:0,channel,fields:Some(fields)}=>{
    if let Some(state)=self.entry_transfer.lobby_success(){self.entry_navigation.key.return_state=state;}
    self.character_scene.retire()?;self.entry_navigation.key.revision=self.entry_navigation.key.revision.checked_add(1).context("navigation revision exhausted")?;self.entry_navigation.key.scene=Scene::Lobby;self.entry_navigation.key.return_mode=1;self.entry_navigation.key.channel=Some(channel);self.entry_navigation.key.fields=Some(fields);self.entry_navigation.key.pending_write=None;self.entry_navigation.key.write_complete=false;self.source_flow=self.source_flow.lobby_reply_received();self.status="已進入大廳。".into();Ok(Change::LobbyEntered)
   },
   _=>unreachable!("nonzero status classified"),
  }
 }
}
'''
s=s.replace('//! R503 ordinary BSS return-state subset. No AFBC/AFBD/provider flag setter.\n//! 4928C0 cs201 -> S_C9 success ->46BA10 phase3; S_C8/0 with B8=0\n//! instead reserves cc202,100 (497020+0), then awaits S_C9. Page40 requires\n//! another, currently unqualified AFBC/B8 producer; it is never fabricated.','//! Source-owned character/transfer/channel/lobby navigation.\n//! F4 and S_FB producers own flags; opaque controls own user commands.\n//! Startup host fallback and room paths remain unavailable.')
p.write_text(s,encoding='utf-8')
print('R506 navigation queue/channel callbacks connected')
