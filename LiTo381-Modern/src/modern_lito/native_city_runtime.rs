//! UI thread owns only presentation. One cancellable worker owns all resource IO/render.
use super::{native_multisector_resources::{self,Inputs,Entry},native_city_flow::{OfflineMapFlow,MapChoice,Page,PreviewFrame},compositor::{Canvas,Viewport},scene::CameraRect};
use anyhow::{ensure,Context,Result};
use super::native_mode_menu::{Menu,Filter,Row};
#[path="native_mode_selection.rs"]pub mod mode_selection;
#[path="native_source_connection.rs"]pub mod source_connection;
#[path="native_source_identity.rs"]pub mod source_identity;
#[path="native_source_stream.rs"]pub mod source_stream;
#[path="native_source_send.rs"]pub mod source_send;
#[path="native_source_actor.rs"]pub mod source_actor;
#[path="native_character_selection.rs"]pub mod character_selection;
#[path="native_login_form.rs"]pub mod login_form;
#[path="native_login_wire.rs"]pub mod login_wire;
#[path="native_login_bootstrap.rs"]pub mod login_bootstrap;
#[path="native_login_transport.rs"]pub mod login_transport;
#[path="native_login_events.rs"]pub mod login_events;
#[path="native_player_entry_commands.rs"]pub mod player_entry_commands;
#[path="native_java_character_flow.rs"]pub mod java_character_flow;
#[path="native_status_receipt.rs"]pub mod status_receipt;
#[path="native_status_family.rs"]pub mod status_family;
#[path="native_java_return.rs"]pub mod java_return;
#[path="native_java_room_entry.rs"]pub mod java_room_entry;
#[path="native_java_room_ready.rs"]pub mod java_room_ready;
#[path="native_java_room_player.rs"]pub mod room_player;
#[path="native_java_room_mode_request.rs"]pub mod java_room_mode_request;
#[path="native_java_room_settings_request.rs"]pub mod java_room_settings_request;
#[path="native_java_room_map_request.rs"]pub mod java_room_map_request;
#[path="native_java_room_settings.rs"]pub mod java_room_settings;
#[path="native_java_room_tail.rs"]pub mod java_room_tail;
#[path="native_java_room_catalog.rs"]pub mod java_room_catalog;
#[path="native_java_room_self.rs"]pub mod java_room_self;
#[path="native_java_room_departure.rs"]pub mod java_room_departure;
#[path="native_java_room_participants.rs"]pub mod java_room_participants;
#[path="native_entry_navigation.rs"]pub mod entry_navigation;
#[path="native_entry_transfer.rs"]pub mod entry_transfer;
#[path="native_startup_host.rs"]pub mod startup_host;
#[path="native_city_presentation.rs"]pub mod presentation;
use source_connection::{Scope as SourceScope,Epoch as SourceEpoch,DecodedPacket as SourceDecodedPacket};
use mode_selection::{ModeSelection,Outcome,SnapshotOutcome,snapshot::{Snapshot,Context as NativeStateContext}};
use mode_selection::state_flow::{Flow as SourceFlow,Transition as SourceTransition};
use std::{sync::{Arc,mpsc::{self,Receiver,SyncSender},atomic::{AtomicBool,AtomicU64,Ordering}},thread::{self,JoinHandle}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum UiPage {Booting,Menu,Loading,Preview,Closing,Closed}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Action {Enter,Pan(i32,i32),Back,Close,Select(u8,u32),Filter(Filter),RowSelect(usize),OpenSourceMenu}
/// Modern local player command. Native46B7D0 receiver2 opens the map list;
/// this ID is a Modern binding, not a recovered Native resource/control ID.
pub const SOURCE_MENU_COMMAND_ID:u16=0x4910;
pub const SOURCE_CHARACTER_COMMAND_FIRST:u16=0x4960;
pub fn source_character_command_slot(command:u16)->Option<usize>{(SOURCE_CHARACTER_COMMAND_FIRST..SOURCE_CHARACTER_COMMAND_FIRST+4).contains(&command).then(||usize::from(command-SOURCE_CHARACTER_COMMAND_FIRST))}
pub fn player_command_action(command:u16)->Option<Action>{match command{SOURCE_MENU_COMMAND_ID=>Some(Action::OpenSourceMenu),_=>None}}
#[derive(Clone)]pub struct SourceCharacterControl {epoch:SourceEpoch,revision:u64,scene_revision:u64,slot:usize}
#[derive(Clone)]pub struct SourceCharacterBack {epoch:SourceEpoch,revision:u64,scene_revision:u64}
#[derive(Clone)]pub struct SourceCharacterSubmit {control:SourceCharacterControl,selection_revision:u64}
#[derive(Clone)]pub struct SourceLobbyBack {epoch:SourceEpoch,revision:u64,roster_revision:u64}
#[derive(Clone)]pub struct SourceChannelBack {epoch:SourceEpoch,revision:u64}
#[derive(Clone)]pub struct SourceChannelControl {epoch:SourceEpoch,revision:u64,slot:usize}
#[derive(Clone)]pub struct SourceTransferServerControl {epoch:SourceEpoch,revision:u64,navigation_revision:u64,slot:usize}
impl SourceChannelControl {pub fn slot(&self)->usize{self.slot}}
impl SourceTransferServerControl {pub fn slot(&self)->usize{self.slot}}
impl SourceCharacterControl {pub fn slot(&self)->usize{self.slot}}
#[derive(Clone,Debug)]pub enum StreamIngress {JavaCountdown(java_room_ready::CountdownChange),JavaRoomModeMap(java_room_settings::Change),JavaRoomSettings(java_room_settings::Change),JavaRoomTail(java_room_tail::Change),JavaParticipants(java_room_participants::Change),JavaRoom(java_room_entry::Change),JavaReturn(java_return::Change),JavaCharacter(java_character_flow::Change),Bootstrap(login_bootstrap::Change),SeedAccepted{counter:u64},Roster(source_identity::Change),Mode(SourceTransition),Actor(source_actor::Change),Navigation(entry_navigation::Change),Transfer(entry_transfer::Change) }
#[derive(Clone,Debug)]pub struct SourceRejection {pub epoch:u64,pub opcode:Option<u8>,pub body_bytes:usize,pub reason:String}
#[derive(Clone,Debug)]pub struct StreamEvent {pub ingress:StreamIngress,pub counter_step:Option<source_stream::CounterStep>}
#[derive(Clone,Debug)]pub struct StreamBatch {pub consumed:usize,pub events:Vec<StreamEvent>,pub terminal:Option<source_stream::End>}
pub fn key_action(page:UiPage,key:u32,control:bool,alt:bool,repeat:bool)->Option<Action>{
 if control||alt||repeat||matches!(page,UiPage::Closing|UiPage::Closed){return None;}
 match key {0x1b=>Some(Action::Back),0x25 if page==UiPage::Preview=>Some(Action::Pan(-48,0)),0x27 if page==UiPage::Preview=>Some(Action::Pan(48,0)),0x26 if page==UiPage::Preview=>Some(Action::Pan(0,-24)),0x28 if page==UiPage::Preview=>Some(Action::Pan(0,24)),_=>None}
}
#[derive(Default)]pub struct Progress {pub initialized:AtomicBool,pub frames_produced:AtomicU64,pub retired:AtomicBool,pub load_active:AtomicBool}
enum Command {Enter{mode:u8,map_id:u32,generation:u64,cancel:Arc<AtomicBool>},Pan{generation:u64,dx:i32,dy:i32},Back}
struct Pixels {mode:u8,map_id:u32,bgra:Arc<[u8]>,camera:CameraRect}
enum Event {Ready(Vec<Entry>,Option<Arc<super::native_mode_menu_skin::Skin>>),InitFailed(String),Frame{generation:u64,pixels:Pixels},Failed{generation:u64,error:String}}
fn pixels(frame:PreviewFrame,mode:u8,map_id:u32)->Result<Pixels>{let mut c=Canvas::new(frame.camera.width,frame.camera.height)?;c.blit_rgba(frame.rgba()?,frame.camera.width,frame.camera.height,0,0)?;let mut bgra=Vec::new();c.copy_bgra(&mut bgra)?;Ok(Pixels{mode,map_id,bgra:bgra.into(),camera:frame.camera})}
#[derive(Clone)]pub struct UiFrame {pixels:Arc<Pixels>,guard:Arc<AtomicU64>,revision:u64}
impl UiFrame {
 pub fn bgra(&self)->Result<&[u8]>{ensure!(self.guard.load(Ordering::Acquire)==self.revision,"UI frame retired");Ok(&self.pixels.bgra)}
 pub fn identity(&self)->(u8,u32){(self.pixels.mode,self.pixels.map_id)}
 pub fn camera(&self)->CameraRect{self.pixels.camera}
 pub fn viewport(&self,width:i32,height:i32)->Option<Viewport>{Viewport::fit(width,height,self.pixels.camera.width,self.pixels.camera.height)}
}
pub struct Runtime {java_ready:java_room_ready::State,last_ready_checkpoint:Option<java_room_ready::Snapshot>,java_mode_request:java_room_mode_request::State,last_mode_request_checkpoint:Option<java_room_mode_request::Snapshot>,java_settings_request:java_room_settings_request::State,last_settings_request_checkpoint:Option<java_room_settings_request::Snapshot>,java_map_request:java_room_map_request::State,last_map_request_checkpoint:Option<java_room_map_request::Snapshot>,java_settings:java_room_settings::State,last_settings_checkpoint:Option<java_room_settings::Snapshot>,java_self:java_room_self::State,last_self_checkpoint:Option<java_room_self::Snapshot>,java_tail:java_room_tail::State,last_tail_checkpoint:Option<java_room_tail::CatalogSnapshot>,java_participants:java_room_participants::State,last_participants_checkpoint:Option<java_room_participants::Snapshot>,java_room:java_room_entry::State,last_room_checkpoint:Option<java_room_entry::Key>,java_return:java_return::State,java_character:java_character_flow::State,last_java_checkpoint:Option<java_character_flow::Key>,bootstrap_adapter:Option<login_bootstrap::JavaFixtureAdapter>,login_bootstrap:login_bootstrap::State,last_source_rejection:Option<SourceRejection>,startup_host:Option<startup_host::FixtureStartupHost>,entry_transfer:entry_transfer::State,entry_navigation:entry_navigation::State,login_events:login_events::Session,login_transport:Option<login_transport::Transport>,login_form:login_form::Session,awaiting_source_roster:bool,character_scene:character_selection::Scene,source_roster_revision:u64,source_actor:source_actor::Session,source_send:source_send::Sender,source_stream:source_stream::Receiver,source_identity:source_identity::State,source_scope:SourceScope,source_flow:SourceFlow,native_snapshot:Option<Arc<Snapshot>>,native_mode:ModeSelection,skin:Option<Arc<super::native_mode_menu_skin::Skin>>,page:UiPage,status:String,rows:Vec<Entry>,selected:Option<usize>,menu:Option<Menu>,generation:u64,revision:u64,frame:Option<UiFrame>,guard:Option<Arc<AtomicU64>>,cancel:Option<Arc<AtomicBool>>,pan_pending:bool,commands:Option<SyncSender<Command>>,events:Receiver<Event>,worker:Option<JoinHandle<()>>,shutdown:Arc<AtomicBool>,progress:Arc<Progress>,pub stale_results_rejected:u64}
struct Retirement(Arc<Progress>);impl Drop for Retirement {fn drop(&mut self){self.0.load_active.store(false,Ordering::Release);self.0.retired.store(true,Ordering::Release);}}
fn worker(inputs:Inputs,skin_inputs:Option<Inputs>,commands:Receiver<Command>,events:mpsc::Sender<Event>,shutdown:Arc<AtomicBool>,progress:Arc<Progress>){
 let _retirement=Retirement(progress.clone());
 let initialized=(||{let skin=skin_inputs.as_ref().map(super::native_mode_menu_skin::Skin::load).transpose()?.map(Arc::new);let(c,b,rows)=native_multisector_resources::load(&inputs)?;ensure!(rows.iter().any(|r|r.available),"no qualified menu row");let f=OfflineMapFlow::new(true,c,b)?;Ok::<_,anyhow::Error>((f,rows,skin))})();
 let (mut flow,rows,skin)=match initialized{Ok(f)=>f,Err(e)=>{let _=events.send(Event::InitFailed(format!("{e:#}")));return;}};
 if shutdown.load(Ordering::Acquire){return;}
 let _=events.send(Event::Ready(rows,skin));progress.initialized.store(true,Ordering::Release);
 let mut active:Option<(u64,Arc<AtomicBool>)>=None;
 while let Ok(command)=commands.recv(){
  if shutdown.load(Ordering::Acquire){break;}
  if active.as_ref().is_some_and(|(_,c)|c.load(Ordering::Acquire)){if flow.page()!=Page::Menu{let _=flow.back();}active=None;}
  match command {
   Command::Back=>{if flow.page()!=Page::Menu{let _=flow.back();}active=None;},
   Command::Enter{mode,map_id,generation,cancel}=>{
    if cancel.load(Ordering::Acquire){continue;}
    if flow.page()!=Page::Menu{let _=flow.back();}progress.load_active.store(true,Ordering::Release);
    let result=(||{flow.select(mode,map_id)?;let ready=flow.begin_entry()?.load(&cancel)?;let frame=flow.complete_entry(ready)?;pixels(frame,mode,map_id)})();
    progress.load_active.store(false,Ordering::Release);
    if cancel.load(Ordering::Acquire)||shutdown.load(Ordering::Acquire){if flow.page()!=Page::Menu{let _=flow.back();}active=None;continue;}
    match result{Ok(pixels)=>{active=Some((generation,cancel));let _=events.send(Event::Frame{generation,pixels});progress.frames_produced.fetch_add(1,Ordering::Release);},Err(e)=>{if flow.page()!=Page::Menu{let _=flow.back();}let _=events.send(Event::Failed{generation,error:format!("{e:#}")});active=None;}}
   },
   Command::Pan{generation,dx,dy}=>{
    let Some((g,cancel))=&active else{continue;};if *g!=generation||cancel.load(Ordering::Acquire){continue;}
    let selected=flow.selected().unwrap();let(mode,map_id)=(selected.mode,selected.map_id);let result=flow.pan(dx,dy,cancel).and_then(|f|pixels(f,mode,map_id));
    if cancel.load(Ordering::Acquire)||shutdown.load(Ordering::Acquire){if flow.page()!=Page::Menu{let _=flow.back();}active=None;continue;}
    match result{Ok(pixels)=>{let _=events.send(Event::Frame{generation,pixels});progress.frames_produced.fetch_add(1,Ordering::Release);},Err(e)=>{let _=flow.back();let _=events.send(Event::Failed{generation,error:format!("{e:#}")});active=None;}}
   }
  }
 }
 // flow/assets drop before the retirement guard publishes actual worker retirement.
}
impl Runtime {
 pub fn start(inputs:Inputs)->Result<Self>{Self::start_inner(inputs,None)}
 pub fn start_with_menu_skin(inputs:Inputs,skin_inputs:Inputs)->Result<Self>{Self::start_inner(inputs,Some(skin_inputs))}
 pub fn skin(&self)->Option<Arc<super::native_mode_menu_skin::Skin>>{self.skin.clone()}
 fn start_inner(inputs:Inputs,skin_inputs:Option<Inputs>)->Result<Self>{let(tx,rx)=mpsc::sync_channel(4);let(out_tx,out_rx)=mpsc::channel();let shutdown=Arc::new(AtomicBool::new(false));let progress=Arc::new(Progress::default());let(s,p)=(shutdown.clone(),progress.clone());let worker=thread::Builder::new().name("modern-offline-preview".into()).spawn(move||worker(inputs,skin_inputs,rx,out_tx,s,p))?;Ok(Self{java_ready:java_room_ready::State::default(),last_ready_checkpoint:None,java_mode_request:java_room_mode_request::State::default(),last_mode_request_checkpoint:None,java_settings_request:java_room_settings_request::State::default(),last_settings_request_checkpoint:None,java_map_request:java_room_map_request::State::default(),last_map_request_checkpoint:None,java_settings:java_room_settings::State::default(),last_settings_checkpoint:None,java_self:java_room_self::State::default(),last_self_checkpoint:None,java_tail:java_room_tail::State::default(),last_tail_checkpoint:None,java_participants:java_room_participants::State::default(),last_participants_checkpoint:None,java_room:java_room_entry::State::default(),last_room_checkpoint:None,java_return:java_return::State::default(),java_character:java_character_flow::State::default(),last_java_checkpoint:None,bootstrap_adapter:None,login_bootstrap:login_bootstrap::State::default(),last_source_rejection:None,startup_host:None,entry_transfer:entry_transfer::State::default(),entry_navigation:entry_navigation::State::default(),login_events:login_events::Session::default(),login_transport:None,login_form:login_form::Session::default(),awaiting_source_roster:false,character_scene:character_selection::Scene::default(),source_roster_revision:0,source_actor:source_actor::Session::default(),source_send:source_send::Sender::default(),source_stream:source_stream::Receiver::default(),source_identity:source_identity::State::default(),source_scope:SourceScope::default(),source_flow:SourceFlow::default(),native_snapshot:None,native_mode:ModeSelection::default(),skin:None,page:UiPage::Booting,status:"正在讀取場地清單…".into(),rows:Vec::new(),selected:None,menu:None,generation:0,revision:0,frame:None,guard:None,cancel:None,pan_pending:false,commands:Some(tx),events:out_rx,worker:Some(worker),shutdown,progress,stale_results_rejected:0})}
 pub fn last_source_rejection(&self)->Option<SourceRejection>{self.last_source_rejection.clone()}
 pub fn source_flow(&self)->&SourceFlow{&self.source_flow}
 pub fn generation(&self)->u64{self.generation}
 pub fn source_connection_epoch(&self)->Option<u64>{self.source_scope.active_serial()}
 pub fn source_connection_candidate_token(&self)->Option<SourceEpoch>{self.source_scope.current_epoch().ok()}
 /// Explicit Native decoded-ingress candidate. No Modern TLS/profile or
 /// authentication result is converted into a Native connection assertion.
 pub fn begin_source_connection_candidate(&mut self)->Result<SourceEpoch>{
  let (next,epoch)=self.source_scope.prepare_begin()?;self.commit_source_boundary(next)?;self.last_source_rejection=None;self.last_java_checkpoint=None;self.last_room_checkpoint=None;self.last_participants_checkpoint=None;self.last_tail_checkpoint=None;self.last_self_checkpoint=None;self.last_settings_checkpoint=None;self.last_map_request_checkpoint=None;self.last_settings_request_checkpoint=None;self.last_mode_request_checkpoint=None;self.last_ready_checkpoint=None;Ok(epoch)
 }
 pub fn disconnect_source_connection_candidate(&mut self,epoch:&SourceEpoch)->Result<()>{
  let next=self.source_scope.prepare_disconnect(epoch)?;self.commit_source_boundary(next)
 }
 fn commit_source_boundary(&mut self,next:SourceScope)->Result<()>{
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"source connection boundary unavailable");
  self.next_generation()?;self.invalidate();self.selected=None;
  if let Some(menu)=self.menu.as_mut(){menu.clear_items();}
  if self.bootstrap_adapter.is_some()&&self.java_character.key.stage!=java_character_flow::Stage::Idle{self.last_java_checkpoint=Some(self.java_character.key.clone());}if let Some(book)=&self.java_participants.book{self.last_participants_checkpoint=Some(book.snapshot());}if self.java_self.java_snapshot_present(){self.last_self_checkpoint=Some(self.java_room_self_snapshot());}if let Some(book)=&self.java_settings.book{self.last_settings_checkpoint=Some(book.snapshot());}self.java_settings.retire();self.archive_retire_java_room_map_request();self.archive_retire_java_room_settings_request();self.archive_retire_java_room_mode_request();self.archive_retire_java_room_ready_request();self.java_self.retire();self.java_participants.retire();if let Some(catalog)=&self.java_tail.catalog{self.last_tail_checkpoint=Some(catalog.snapshot());}self.java_tail.retire();if self.java_room.key.phase!=java_room_entry::Phase::Idle{self.last_room_checkpoint=Some(self.java_room.key.clone());}self.java_room.retire()?;self.java_return.clear();self.java_character=java_character_flow::State::default();self.login_bootstrap.retire();self.entry_transfer.retire()?;self.entry_navigation.retire()?;self.login_events.retire();self.login_transport=None;self.login_form.retire()?;self.awaiting_source_roster=false;self.character_scene.retire()?;self.source_roster_revision=0;self.source_actor=source_actor::Session::default();self.source_send=source_send::Sender::default();self.source_stream=source_stream::Receiver::default();self.source_identity=source_identity::State::default();self.source_flow=SourceFlow::default();self.source_scope=next;
  self.page=UiPage::Menu;self.status="場地來源已更新，請等待場地狀態".into();
  if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}Ok(())
 }
 pub fn consume_source_event(&mut self,event:&SourceDecodedPacket)->Result<SourceTransition>{
  self.source_scope.admit(event)?;self.apply_source_packet(event.body())
 }
 /// Future Native decoded dispatcher routes opcode3/203 here;206/213 use
 /// consume_source_event. This route itself parses the source roster body.
 /// No socket/profile producer or AFBC/AFBD alternate branch is claimed.
 pub fn consume_source_identity_event(&mut self,event:&SourceDecodedPacket)->Result<source_identity::Change>{
  self.source_scope.admit(event)?;
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"source identity unavailable");
  ensure!(!self.entry_navigation.active(),"roster replacement during navigation requires source retirement");
  ensure!(event.body().first()!=Some(&3)||self.source_send.snapshot().pending.is_none(),"roster replacement while selected write pending;retire source epoch");
  if event.body().first()==Some(&3)&&self.bootstrap_adapter.is_none(){ensure!(usize::from(event.body()[1])<=source_identity::SLOT_COUNT,"strict Native four-row UI boundary");}
  if self.bootstrap_adapter.is_some()&&event.body().first()==Some(&3){ensure!(matches!(self.login_bootstrap.key.stage,login_bootstrap::Stage::AwaitRoster|login_bootstrap::Stage::Roster),"GAME roster requires completed NEWS continuation");self.login_bootstrap.credentials=None;self.login_bootstrap.key.credentials_pending=false;self.login_bootstrap.key.stage=login_bootstrap::Stage::Roster;self.login_bootstrap.key.roster_count=Some(event.body()[1]);self.java_character.key=java_character_flow::Key{stage:java_character_flow::Stage::Directory,..Default::default()};}
  let revision=self.source_roster_revision.checked_add(1).context("source roster control revision exhausted")?;let(next,change)=self.source_identity.prepare_packet(event.body())?;self.login_form.retire()?;self.character_scene.retire()?;if self.bootstrap_adapter.is_some(){for r in next.records(){r.fields().validate_java_projection()?;ensure!(!r.name().is_empty(),"AD368 empty character name");r.display_name()?;source_identity::fields::display_ms950(r.clan_name())?;}}self.source_identity=next;self.source_roster_revision=revision;if matches!(change,source_identity::Change::Started{..}){self.java_self.retire();}if matches!(change,source_identity::Change::Started{..}){self.awaiting_source_roster=false;}if self.source_identity.complete()&&!self.awaiting_source_roster{self.character_scene.enter();}self.finish_java_return_roster()?;Ok(change)
 }
 ///LocalModern form lifecycle for631E38/492270;no sourcephase/roster setters.
 pub fn source_login_snapshot(&self)->login_form::Snapshot{self.login_form.snapshot()}
 pub fn source_login_control(&self)->Option<login_form::Control>{self.login_form.control()}
 pub fn open_source_login_form(&mut self)->Result<()>{ensure!(!self.entry_navigation.active()&&self.page==UiPage::Menu&&!self.character_scene.key().active&&self.source_send.snapshot().pending.is_none()&&!matches!(self.login_form.snapshot().stage,login_form::Stage::Sending|login_form::Stage::AwaitingReply),"登入畫面目前無法開啟");self.login_form.open()?;self.status="輸入帳號與密碼。連線登入尚未開放。".into();Ok(())}
 pub fn edit_source_login_form(&mut self,c:&login_form::Control,account:&[u8],password:&[u8])->Result<()>{ensure!(self.page==UiPage::Menu,"login form unavailable");self.login_form.edit(c,account,password)}
 ///Onlyvalidate/normalize/stageownedinput. AwaitingAuthoritiesisLOCAL UI,notNativephase/authACK.
 pub fn stage_source_login_form(&mut self,c:&login_form::Control)->Result<()>{ensure!(self.page==UiPage::Menu,"login form unavailable");self.login_form.stage(c)?;self.status="登入資料已準備；連線登入尚未開放。可按返回清除。".into();Ok(())}
 pub fn cancel_source_login_form(&mut self,c:&login_form::Control)->Result<()>{ensure!(self.page==UiPage::Menu,"login form unavailable");self.login_form.admit(c)?;ensure!(!matches!(self.login_form.snapshot().stage,login_form::Stage::Sending|login_form::Stage::AwaitingReply),"login write or reply pending; disconnect or Close required");self.login_form.retire()?;self.status="登入資料已清除".into();Ok(())}
 pub fn source_identity(&self)->&source_identity::State{&self.source_identity}
 pub fn source_stream_snapshot(&self)->source_stream::Snapshot{self.source_stream.snapshot()}
 pub fn source_send_snapshot(&self)->source_send::Snapshot{self.source_send.snapshot()}
 pub fn source_actor_snapshot(&self)->source_actor::Snapshot{self.source_actor.snapshot()}
 pub fn presentation_key(&self)->presentation::Key{presentation::Key{java_ready:self.java_room_ready_snapshot(),java_mode_request:self.java_room_mode_request_snapshot(),java_settings_request:self.java_room_settings_request_snapshot(),java_map_request:self.java_room_map_request_snapshot(),java_settings:self.java_room_settings_snapshot(),java_self:self.java_room_self_snapshot(),java_catalog:self.java_room_catalog_snapshot(),java_participants:self.java_room_participants_snapshot(),java_room:self.java_room.key.clone(),java_return:self.java_return.key.clone(),java_character:self.java_character.key.clone(),bootstrap:self.login_bootstrap.key.clone(),login_form:self.login_form.snapshot(),awaiting_roster:self.awaiting_source_roster,character_ui:self.character_scene.key(),navigation:self.entry_navigation.key.clone(),transfer:self.entry_transfer.key.clone(),startup_host_fixture:self.source_startup_host_fixture_target(),epoch:self.source_scope.active_serial(),roster_revision:self.source_roster_revision,pending:self.source_send.snapshot(),page:self.page,generation:self.generation,frame_revision:self.revision,skin:self.skin.is_some(),status:self.status.clone(),phase:self.source_flow.phase(),mode:self.source_flow.mode().raw_mode(),identifier:self.source_flow.mode().identifier(),version:self.source_actor.snapshot().version.map(|v|v.value()),selected_slot:self.source_identity.selected().map(|s|s.slot()),selected_map:self.selected,pan_pending:self.pan_pending,menu:self.menu.as_ref().map(|m|(m.filter,m.count(),m.top,m.selected,m.pressed))}}
 pub fn source_menu_invoker_enabled(&self)->bool{!self.entry_navigation.active()&&self.login_form.snapshot().stage==login_form::Stage::Hidden&&!self.awaiting_source_roster&&!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed)&&self.source_scope.active_serial().is_some()&&self.skin.is_some()&&self.source_flow.phase()==2&&self.source_flow.snapshot().is_some()&&self.source_flow.mode().catalog_mode().is_ok()}
 pub fn consume_source_actor_event(&mut self,event:&SourceDecodedPacket)->Result<source_actor::Change>{
  self.source_scope.admit(event)?;
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"source actor event unavailable");
  let(next,change)=self.source_actor.prepare(event.body(),self.source_identity.selected().is_some(),self.source_flow.phase())?;
  if change==source_actor::Change::LoginRefused{let epoch=self.source_scope.current_epoch()?;self.disconnect_source_connection_candidate(&epoch)?;}else{self.source_actor=next;}Ok(change)
 }
 pub fn source_character_control_enabled(&self,slot:usize)->bool{!self.entry_navigation.active()&&!self.awaiting_source_roster&&self.character_scene.key().active&&self.page==UiPage::Menu&&self.source_scope.active_serial().is_some()&&self.source_send.snapshot().counter.is_some()&&self.source_send.snapshot().pending.is_none()&&self.source_identity.prepare_select_bounded(slot,if self.bootstrap_adapter.is_some(){source_identity::JAVA_ROW_BOUND}else{source_identity::SLOT_COUNT}).is_ok()}
 pub fn source_character_control(&self,slot:usize)->Option<SourceCharacterControl>{if !self.source_character_control_enabled(slot){return None;}Some(SourceCharacterControl{epoch:self.source_scope.current_epoch().ok()?,revision:self.source_roster_revision,scene_revision:self.character_scene.key().scene_revision,slot})}
 pub fn handle_source_character_control(&mut self,control:&SourceCharacterControl)->Result<()>{self.source_scope.admit_epoch(&control.epoch)?;ensure!(control.revision==self.source_roster_revision&&control.scene_revision==self.character_scene.key().scene_revision&&self.source_character_control_enabled(control.slot),"retired source character control");self.prepare_source_send_candidate(&control.epoch,source_send::Intent::SelectCharacter{slot:control.slot})?;self.status="角色已選取，等待傳送".into();Ok(())}
 ///415C40(+34) normal release ->4934A0:select only;414A80 WM203
 /// increments +18 and415C90(+40)->4934A0->4928C0 for double click.
 pub fn choose_source_character_control(&mut self,control:&SourceCharacterControl,double_click:bool)->Result<()>{
  self.source_scope.admit_epoch(&control.epoch)?;
  ensure!(control.revision==self.source_roster_revision&&control.scene_revision==self.character_scene.key().scene_revision&&self.source_character_control_enabled(control.slot),"retired character selection callback");
  self.character_scene.select(control.slot)?;self.status="角色已選取，可按進入角色或雙擊".into();
  if double_click{let submit=self.source_character_submit().context("character submit unavailable")?;self.handle_source_character_submit(&submit)?;}Ok(())
 }
 pub fn source_character_submit(&self)->Option<SourceCharacterSubmit>{let key=self.character_scene.key();Some(SourceCharacterSubmit{control:self.source_character_control(key.selected?)?,selection_revision:key.selection_revision})}
 pub fn handle_source_character_submit(&mut self,submit:&SourceCharacterSubmit)->Result<()>{let key=self.character_scene.key();ensure!(submit.selection_revision==key.selection_revision&&key.selected==Some(submit.control.slot),"retired selected character submit");self.handle_source_character_control(&submit.control)}
 pub fn source_character_press(&mut self,control:&SourceCharacterControl)->Result<()>{self.source_scope.admit_epoch(&control.epoch)?;ensure!(control.revision==self.source_roster_revision&&control.scene_revision==self.character_scene.key().scene_revision&&self.source_character_control_enabled(control.slot),"retired character press");self.character_scene.press(control.slot);Ok(())}
 pub fn cancel_source_character_press(&mut self){self.character_scene.cancel_press();}
 pub fn enter_source_character_scene(&mut self)->Result<()>{ensure!(!self.entry_navigation.active()&&!self.awaiting_source_roster&&self.page==UiPage::Menu&&self.source_scope.active_serial().is_some()&&self.source_identity.complete(),"character scene unavailable");self.character_scene.enter();Ok(())}
 pub fn leave_source_character_scene(&mut self)->Result<()>{ensure!(!self.entry_navigation.active(),"navigation scene cannot open offline gallery");ensure!(!matches!(self.login_form.snapshot().stage,login_form::Stage::Sending|login_form::Stage::AwaitingReply),"login write or reply pending");self.login_form.retire()?;self.character_scene.retire()}
 ///Native493150 receiver6 ->496E30 ->46B670 reserves c15 before local
 ///cleanup and64BBD4=5. Preserve this connection's seed/version/RX/TX.
 ///Nativedynamicdestructors/authentication/loginpanel remain unqualified.
 pub fn source_character_back(&self)->Option<SourceCharacterBack>{if self.entry_navigation.active()||self.awaiting_source_roster||!self.character_scene.key().active||self.page!=UiPage::Menu||!self.source_identity.complete()||self.source_send.snapshot().counter.is_none()||self.source_send.snapshot().pending.is_some(){return None;}Some(SourceCharacterBack{epoch:self.source_scope.current_epoch().ok()?,revision:self.source_roster_revision,scene_revision:self.character_scene.key().scene_revision})}
 pub fn handle_source_character_back(&mut self,control:&SourceCharacterBack)->Result<()>{
  self.source_scope.admit_epoch(&control.epoch)?;
  ensure!(control.revision==self.source_roster_revision&&control.scene_revision==self.character_scene.key().scene_revision&&self.source_character_back().is_some(),"retired source character Back");
  let prepared=self.source_send.prepare_return_login()?;
  let revision=self.source_roster_revision.checked_add(1).context("source roster revision exhausted")?;let mut scene=self.character_scene.clone();scene.retire()?;
  let identity=self.source_identity.without_selected();let flow=self.source_flow.returned_to_login();
  self.next_generation()?;self.invalidate();self.selected=None;if let Some(menu)=self.menu.as_mut(){menu.clear_items();}
  self.login_form.retire()?;self.source_send.commit(prepared);self.character_scene=scene;self.source_identity=identity;self.source_flow=flow;self.source_roster_revision=revision;self.java_self.retire();self.awaiting_source_roster=true;
  self.page=UiPage::Menu;self.status="已返回入口，返回要求等待傳送；等待新角色名單".into();if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}Ok(())
 }
 pub fn return_source_character_scene(&mut self)->Result<()>{let control=self.source_character_back().context("source character Back unavailable")?;self.handle_source_character_back(&control)}
 pub fn source_pending_write_ticket(&self,epoch:&SourceEpoch)->Result<Option<source_send::Ticket>>{self.source_scope.admit_epoch(epoch)?;Ok(self.source_send.pending_ticket())}
 /// Typed selected source record ->4928C0 cs/201 candidate. Reserving a
 /// frame advances sendcounter BEFORE any reported write, like4C6770.
 /// Complete is a local write report;no authentication/ACK/phase transition.
 pub fn prepare_source_send_candidate(&mut self,epoch:&SourceEpoch,intent:source_send::Intent)->Result<source_send::Reservation>{
  self.source_scope.admit_epoch(epoch)?;
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"source send unavailable");
  ensure!(!matches!(intent,source_send::Intent::ReturnLogin),"ReturnLogin requirescurrentopaqueBack control");
  ensure!(!self.entry_navigation.active(),"source navigation reply pending or lobby active");
  ensure!(!self.awaiting_source_roster,"new source roster required after Back");
  let source_send::Intent::SelectCharacter{slot}=intent else{anyhow::bail!("Native480560 FromCD/version/registry/auth sideeffects unqualified");};
  let next=self.source_identity.prepare_select_bounded(slot,if self.bootstrap_adapter.is_some(){source_identity::JAVA_ROW_BOUND}else{source_identity::SLOT_COUNT})?;
  let prepared=self.source_send.prepare_character(next.selected().unwrap().actor_alias())?;
  let serial=prepared.reservation.ticket.serial();let mut navigation=self.entry_navigation.clone();navigation.await_character(serial)?;let mut scene=self.character_scene.clone();scene.retire()?;
  if self.bootstrap_adapter.is_some(){ensure!(self.java_character.key.stage==java_character_flow::Stage::Directory,"AD368 directory not available for C9");self.java_character.key=java_character_flow::Key{stage:java_character_flow::Stage::AwaitCharacterResult,selected_name:Some(next.selected().unwrap().actor_alias().to_vec()),..Default::default()};}
  let reservation=self.source_send.commit(prepared);self.java_self.selection_requested(serial);self.source_identity=next;self.entry_navigation=navigation;self.character_scene=scene;Ok(reservation)
 }
 pub fn source_write_view(&self,epoch:&SourceEpoch,ticket:&source_send::Ticket)->Result<source_send::WriteView>{self.source_scope.admit_epoch(epoch)?;self.source_send.view(ticket)}
 /// Explicit offline/provider-report boundary. No provider is called here.
 /// Unknownspecial2733 buffering policy,zero/oversend orfailure retires the
 /// wholecandidate epoch through existingdisconnect;not Nativeerror CFG emulation.
 pub fn report_source_write_candidate(&mut self,epoch:&SourceEpoch,ticket:&source_send::Ticket,offset:usize,outcome:source_send::WriteOutcome)->Result<source_send::Progress>{
  self.source_scope.admit_epoch(epoch)?;let progress=self.source_send.report(ticket,offset,outcome)?;self.note_self_exit_write(ticket.serial(),&progress);self.note_java_room_map_write(ticket.serial(),&progress);self.note_java_room_settings_write(ticket.serial(),&progress);self.note_java_room_mode_write(ticket.serial(),&progress);self.note_java_room_ready_write(ticket.serial(),&progress);
  if matches!(progress,source_send::Progress::Complete){self.entry_navigation.written(ticket.serial());self.java_room.written(ticket.serial());self.login_bootstrap.written(ticket.serial());self.finish_java_return_write(ticket.serial())?;}
  if let Some(t)=&mut self.login_transport{t.write_report(ticket.serial(),&progress);}
  if matches!(progress,source_send::Progress::Retired{..}){self.disconnect_source_connection_candidate(epoch)?;}else{let login_write=self.login_form.snapshot().write_serial==Some(ticket.serial());self.login_form.write_report(ticket.serial(),&progress)?;if login_write&&matches!(progress,source_send::Progress::Complete){self.status="登入要求已傳送，等待伺服器回包".into();}}if matches!(progress,source_send::Progress::Complete){self.finish_source_navigation_write()?;}Ok(progress)
 }
 /// Owned candidate input after an eventual qualified Native socket reader.
 /// A bounded batch synchronously uses the SAME epoch decoder and Runtime
 /// roster/mode entrypoints. The caller retains/resumes unconsumed input.
 /// SeedAccepted proves only source counter setup, not480560/login success.
 pub fn feed_source_stream_candidate(&mut self,epoch:&SourceEpoch,input:&[u8])->Result<StreamBatch>{
  self.source_scope.admit_epoch(epoch)?;
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"source stream unavailable");
  let mut batch=StreamBatch{consumed:0,events:Vec::new(),terminal:None};
  while batch.consumed<input.len() && batch.events.len()<source_stream::MAX_BATCH_EVENTS {
   let(n,frame)=self.source_stream.next(&input[batch.consumed..]);batch.consumed+=n;
   let frame=match frame {Ok(Some(f))=>f,Ok(None)=>break,Err(reason)=>{self.retire_source_stream(epoch,reason,&mut batch)?;break;}};
   let event=match frame {
    source_stream::Frame::Seed{counter}=>{self.source_send.seed_from_source(counter)?;if self.login_transport.is_some(){self.prepare_source_compatibility_fixture(epoch)?;}StreamEvent{ingress:StreamIngress::SeedAccepted{counter},counter_step:None}},
    source_stream::Frame::Body{bytes,step}=>{
     if bytes.first()==Some(&1){self.retire_source_stream(epoch,source_stream::Terminal::RepeatedSeed,&mut batch)?;break;}
     let packet=match if self.java_room_countdown_dispatch(&bytes){epoch.decoded_java_room_countdown(&bytes)}else if self.java_room_mode_map_dispatch(&bytes){epoch.decoded_java_room_mode_map(&bytes)}else if self.java_room_settings_dispatch(&bytes){epoch.decoded_java_room_settings(&bytes)}else if bytes.first()==Some(&204)&&self.java_room_catalog_full_row_route_enabled(){epoch.decoded_java_room_catalog(&bytes)}else if (bytes.first()==Some(&213)&&self.java_room_owner_route_enabled())||(bytes.first()==Some(&207)&&self.java_room_catalog_route_enabled()){epoch.decoded_java_room_tail(&bytes)}else if (matches!(bytes.first(),Some(205|209))&&self.java_room_participants_route_enabled())||(bytes.first()==Some(&208)&&self.java_room_departure_route_enabled()){epoch.decoded_java_room_participant(&bytes)}else if bytes.first()==Some(&203)&&self.entry_transfer.roster_alternate(){epoch.decoded_transfer_roster_trigger(&bytes)}else{epoch.decoded_packet(&bytes)}{Ok(p)=>p,Err(error)=>{self.last_source_rejection=Some(SourceRejection{epoch:epoch.serial(),opcode:bytes.first().copied(),body_bytes:bytes.len(),reason:error.to_string()});let unsupported59=bytes.first()==Some(&59);self.retire_source_stream(epoch,source_stream::Terminal::UnsupportedBody,&mut batch)?;if unsupported59{self.status="收到尚未支援的 59 清單回包；連線已結束".into();}break;}};
     let before=self.source_stream.snapshot();
     let ingress=if self.java_room_countdown_dispatch(&bytes){self.consume_java_room_countdown_candidate(&packet).map(StreamIngress::JavaCountdown)}else if self.java_room_mode_map_dispatch(&bytes){self.consume_java_room_mode_map_candidate(&packet).map(StreamIngress::JavaRoomModeMap)}else if self.java_room_settings_dispatch(&bytes){self.consume_java_room_settings_candidate(&packet).map(StreamIngress::JavaRoomSettings)}else if bytes.first()==Some(&204)&&self.java_room_catalog_full_row_route_enabled(){self.consume_java_room_tail_candidate(&packet).map(StreamIngress::JavaRoomTail)}else if (bytes.first()==Some(&213)&&self.java_room_owner_route_enabled())||(bytes.first()==Some(&207)&&self.java_room_catalog_route_enabled()){self.consume_java_room_tail_candidate(&packet).map(StreamIngress::JavaRoomTail)}else if (matches!(bytes.first(),Some(205|209))&&self.java_room_participants_route_enabled())||(bytes.first()==Some(&208)&&self.java_room_departure_route_enabled()){self.consume_java_room_participants_candidate(&packet).map(StreamIngress::JavaParticipants)}else if bytes.first()==Some(&206)&&self.bootstrap_adapter.is_some()&&self.java_room.adapter.is_some(){self.consume_java_room_entry_candidate(&packet).map(StreamIngress::JavaRoom)}else if bytes.first()==Some(&245){self.consume_java_return_event(&packet).map(StreamIngress::JavaReturn)}else if (self.bootstrap_adapter.is_some()&&matches!(bytes.first(),Some(200|201)))||matches!(bytes.first(),Some(12|77|80|81|82|86|124|202)){self.consume_java_character_event(&packet).map(StreamIngress::JavaCharacter)}else if matches!(bytes.first(),Some(0|59|230)){self.consume_source_bootstrap_event(&packet).map(StreamIngress::Bootstrap)}else if bytes.first()==Some(&244)&&self.bootstrap_adapter.is_some()&&self.login_bootstrap.key.stage==login_bootstrap::Stage::AwaitRedirect{let entry_transfer::Packet::Request{slot,address_word,port_word}=entry_transfer::decode(&bytes)? else{unreachable!()};self.redirect_login_bootstrap(slot,address_word,port_word).map(StreamIngress::Bootstrap)}else if matches!(bytes.first(),Some(243|244|251)){self.consume_source_transfer_event(&packet).map(StreamIngress::Transfer)}else if matches!(bytes.first(),Some(200|201)){self.consume_source_navigation_event(&packet).map(StreamIngress::Navigation)}else if bytes.first()==Some(&205)&&self.java_room.key.phase==java_room_entry::Phase::RoomReceipts{Err(anyhow::anyhow!("AD368 incremental room S_CD receipt adapter unqualified"))}else if matches!(bytes.first(),Some(58|205|209)){self.consume_source_actor_event(&packet).map(StreamIngress::Actor)}else if matches!(bytes.first(),Some(3|203)){if self.entry_transfer.roster_alternate(){self.consume_source_transfer_roster(&packet).map(StreamIngress::Transfer)}else{self.consume_source_identity_event(&packet).map(StreamIngress::Roster)}}else{self.consume_source_event(&packet).map(StreamIngress::Mode)};
     let ingress=ingress.and_then(|i|{self.note_java_self_stream(epoch,&bytes,&i)?;Ok(i)});
     match ingress {Ok(StreamIngress::JavaCharacter(change@java_character_flow::Change::Refused{..}))=>{batch.events.push(StreamEvent{ingress:StreamIngress::JavaCharacter(change),counter_step:Some(step)});batch.terminal=Some(source_stream::End{terminal:source_stream::Terminal::DispatchRefused,before});break;},Ok(StreamIngress::Actor(source_actor::Change::LoginRefused))=>{batch.events.push(StreamEvent{ingress:StreamIngress::Actor(source_actor::Change::LoginRefused),counter_step:Some(step)});batch.terminal=Some(source_stream::End{terminal:source_stream::Terminal::DispatchRefused,before});break;},Ok(StreamIngress::Navigation(change@entry_navigation::Change::RestartAfterNativeAlert{..}))=>{batch.events.push(StreamEvent{ingress:StreamIngress::Navigation(change),counter_step:Some(step)});batch.terminal=Some(source_stream::End{terminal:source_stream::Terminal::DispatchRefused,before});break;},Ok(ingress)=>StreamEvent{ingress,counter_step:Some(step)},Err(error)=>{self.last_source_rejection=Some(SourceRejection{epoch:epoch.serial(),opcode:bytes.first().copied(),body_bytes:bytes.len(),reason:error.to_string()});self.retire_source_stream(epoch,source_stream::Terminal::DispatchRefused,&mut batch)?;break;}}
    }
   };if let StreamIngress::JavaParticipants(change)=&event.ingress{self.java_participants.note_stream(change);}
   match &event.ingress {StreamIngress::JavaRoomSettings(_)|StreamIngress::JavaRoomModeMap(_)=>self.java_settings.note_stream(),StreamIngress::JavaRoomTail(java_room_tail::Change::Owner(change))=>self.java_participants.note_stream(change),StreamIngress::JavaRoomTail(java_room_tail::Change::CatalogDelete{..}|java_room_tail::Change::CatalogFullRow{..})=>self.java_tail.note_stream(),_=>{}}
   batch.events.push(event);if self.source_scope.admit_epoch(epoch).is_err(){break;}
  }
  Ok(batch)
 }
 fn retire_source_stream(&mut self,epoch:&SourceEpoch,reason:source_stream::Terminal,batch:&mut StreamBatch)->Result<()>{
  let before=self.source_stream.snapshot();self.disconnect_source_connection_candidate(epoch)?;
  batch.terminal=Some(source_stream::End{terminal:reason,before});Ok(())
 }
 pub fn end_source_stream_candidate(&mut self,epoch:&SourceEpoch)->Result<source_stream::End>{
  self.source_scope.admit_epoch(epoch)?;let end=self.source_stream.end();self.disconnect_source_connection_candidate(epoch)?;Ok(end)
 }
 /// Explicit selected Native roster control candidate, matching494120's
 /// slot0..3 and4928C0 record guards. A visible invoker remains OPEN.
 pub fn select_source_character_candidate(&mut self,epoch:&SourceEpoch,slot:usize)->Result<()>{
  self.source_scope.admit_epoch(epoch)?;ensure!(!self.entry_navigation.active(),"selected source character frozen during navigation");
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"source selection unavailable");
  self.source_identity=self.source_identity.prepare_select(slot)?;Ok(())
 }
 /// Shared command route used by the production window and finite consumer.
 /// The command carries no phase, identifier, mode, or inspection context.
 pub fn handle_player_command(&mut self,command:u16)->Result<()>{
  if command==SOURCE_MENU_COMMAND_ID{ensure!(self.source_menu_invoker_enabled(),"場地入口尚未可用");}
  let action=player_command_action(command).context("unknown player command")?;
  self.handle(action)
 }
 /// Persistent source-event entry, separate from the context-supplied
 /// inspection APIs below.206 clears the proven64A968 map-list alias;the
 /// Modern preview generation must retire with its invalid selection.
 pub fn consume_source_packet(&mut self,packet:&[u8])->Result<SourceTransition>{
  ensure!(!self.source_scope.bound(),"raw source entry unavailable for a connection-owned candidate");
  self.apply_source_packet(packet)
 }
 fn apply_source_packet(&mut self,packet:&[u8])->Result<SourceTransition>{
  ensure!(!self.entry_navigation.active(),"navigation mode side effects unqualified; retire source epoch");
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"source event flow unavailable");
  let prepared=self.source_flow.prepare_packet(packet)?;let transition=prepared.transition;
  if transition.clear_map_list{
   self.next_generation()?;self.invalidate();self.selected=None;
   if let Some(menu)=self.menu.as_mut(){menu.clear_items();}
   self.page=UiPage::Menu;self.status="場地清單已更新，請重新選擇".into();
   if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}
  }
  self.source_flow=prepared.next;Ok(transition)
 }
 /// A real explicit list-open boundary for the persistent event state.
 /// Qualified ordinary catalog only;Native toggle/physical capture not wired.
 pub fn open_source_state_menu(&mut self)->Result<()>{
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"source menu unavailable");
  ensure!(self.source_flow.phase()==2&&self.source_flow.snapshot().is_some(),"尚未收到可用的場地狀態");
  let mode=self.source_flow.mode().catalog_mode()?;
  let menu=Menu::for_filter(&self.rows,Filter::Mode{mode,alternate:false})?;
  self.next_generation()?;self.invalidate();self.selected=None;self.menu=Some(menu);self.page=UiPage::Menu;self.status="請選擇場地".into();
  if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}Ok(())
 }
 pub fn native_mode(&self)->ModeSelection{self.native_mode}
 pub fn native_snapshot(&self)->Option<Arc<Snapshot>>{self.native_snapshot.clone()}
 /// Context-supplied inspection only;does not advance source_flow or apply
 /// its source206 list-clear effect. Fully validate before committing scoped
 /// mode/identifier state. Rejection/ignored identity preserves the last
 /// accepted snapshot and every presentation/worker field.
 pub fn apply_native_state_packet(&mut self,packet:&[u8],context:NativeStateContext)->Result<SnapshotOutcome>{
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"native state context unavailable");
  let snapshot=Snapshot::decode(packet)?;
  let result=self.native_mode.apply_snapshot(&snapshot,context)?;
  if matches!(result,SnapshotOutcome::Applied{..}){self.native_snapshot=Some(Arc::new(snapshot));}
  Ok(result)
 }
 /// Offline inspection only: an accepted packet changes context, not the
 /// current menu, selected map, worker generation, or rendered frame.
 pub fn apply_native_mode_packet(&mut self,packet:&[u8],phase:u32)->Result<Outcome>{
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"native mode context unavailable");
  Ok(self.native_mode.apply(packet,phase)?)
 }
 /// Explicit fresh construction modeled on 46B7D0. The native button/network
 /// route is not connected here. Even the same mode clears selection/top.
 pub fn open_native_mode_menu(&mut self)->Result<()>{
  ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"native mode menu unavailable");
  let mode=self.native_mode.catalog_mode()?;
  let menu=Menu::for_filter(&self.rows,Filter::Mode{mode,alternate:false})?;
  self.next_generation()?;self.invalidate();self.selected=None;self.menu=Some(menu);self.page=UiPage::Menu;
  self.status="請選擇場地".into();
  if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}Ok(())
 }
 pub fn menu(&self)->Option<&Menu>{self.menu.as_ref()}pub fn menu_mut(&mut self)->Option<&mut Menu>{self.menu.as_mut()}
 pub fn page(&self)->UiPage{self.page}pub fn status(&self)->&str{&self.status}pub fn choice(&self)->Option<&MapChoice>{self.selected.map(|i|&self.rows[i].choice)}pub fn rows(&self)->&[Entry]{&self.rows}pub fn selected_index(&self)->Option<usize>{self.selected}pub fn frame(&self)->Option<UiFrame>{self.frame.clone()}pub fn pan_pending(&self)->bool{self.pan_pending}pub fn progress(&self)->Arc<Progress>{self.progress.clone()}
 fn invalidate(&mut self){if let Some(m)=self.menu.as_mut(){m.cancel_press();}if let Some(c)=self.cancel.take(){c.store(true,Ordering::Release);}if let Some(g)=self.guard.take(){g.store(0,Ordering::Release);}self.frame=None;self.pan_pending=false;}
 fn next_generation(&mut self)->Result<u64>{self.generation=self.generation.checked_add(1).context("UI generation exhausted")?;Ok(self.generation)}
 pub fn handle(&mut self,action:Action)->Result<()>{
  ensure!(!self.entry_navigation.active()||action==Action::Close,"navigation scene requires its own callback");
  match action {
   Action::OpenSourceMenu=>self.open_source_state_menu()?,
   Action::Enter=>{ensure!(self.page==UiPage::Menu&&self.selected.is_some(),"場地選單尚未準備好");let row=&self.rows[self.selected.unwrap()];ensure!(row.available,"{} [{}:{}] 資源未合格，不能預覽",row.choice.name,row.choice.mode,row.choice.map_id);let choice=self.choice().context("selection absent")?.clone();let generation=self.next_generation()?;let cancel=Arc::new(AtomicBool::new(false));self.commands.as_ref().context("worker retired")?.try_send(Command::Enter{mode:choice.mode,map_id:choice.map_id,generation,cancel:cancel.clone()}).map_err(|_|anyhow::anyhow!("背景工作尚未可接受新載入，請稍後重試"))?;self.cancel=Some(cancel);self.page=UiPage::Loading;self.status=format!("正在載入 {} [{}:{}]…可返回或切換",choice.name,choice.mode,choice.map_id);},
   Action::Pan(dx,dy)=>{ensure!(self.page==UiPage::Preview&&!self.pan_pending,"視角更新尚未完成");self.commands.as_ref().context("worker retired")?.try_send(Command::Pan{generation:self.generation,dx,dy}).map_err(|_|anyhow::anyhow!("視角工作佇列已滿"))?;self.pan_pending=true;},
   Action::Back=>{ensure!(matches!(self.page,UiPage::Loading|UiPage::Preview),"已在場地選單");self.next_generation()?;self.invalidate();self.page=UiPage::Menu;self.status="已返回場地選單".into();if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}},
   Action::Filter(filter)=>{ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"選單未可切換模式");if self.menu.as_ref().is_some_and(|m|m.filter==filter){return Ok(());}let menu=Menu::for_filter(&self.rows,filter)?;self.next_generation()?;self.invalidate();self.selected=None;self.menu=Some(menu);self.page=UiPage::Menu;self.status="模式已切換：選取已清除，捲動已歸零，請選擇場地".into();if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}},
   Action::RowSelect(row)=>{ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"選單尚未準備");let item=self.menu.as_ref().and_then(|m|m.row(row)).context("列不在目前模式")?;match item{Row::Map(i)=>{let c=self.rows[i].choice.clone();self.handle(Action::Select(c.mode,c.map_id))?;},Row::Header{text,..}=>{self.next_generation()?;self.invalidate();self.selected=None;self.menu.as_mut().unwrap().choose(row)?;self.page=UiPage::Menu;self.status=format!("{text}：原版特殊項目，離線預覽未提供隨機配對");if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}}}},
   Action::Select(mode,map_id)=>{ensure!(!matches!(self.page,UiPage::Booting|UiPage::Closing|UiPage::Closed),"選單未可切換");let i=self.rows.iter().position(|r|r.choice.mode==mode&&r.choice.map_id==map_id).context("未知 mode/map；不替換場地")?;let row_index=self.menu.as_ref().context("menu absent")?.rows().iter().position(|r|*r==Row::Map(i)).context("場地不在目前模式；不替換場地")?;self.next_generation()?;self.invalidate();self.menu.as_mut().unwrap().choose(row_index)?;self.selected=Some(i);self.page=UiPage::Menu;let row=&self.rows[i];self.status=format!("{} [{}:{}] · {}",row.choice.name,mode,map_id,if row.available{"資源已合格，可預覽"}else{"資源未合格，不可預覽"});if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}},
   Action::Close=>{if matches!(self.page,UiPage::Closing|UiPage::Closed){return Ok(());}self.startup_host=None;self.next_generation()?;self.invalidate();if self.bootstrap_adapter.is_some()&&self.java_character.key.stage!=java_character_flow::Stage::Idle{self.last_java_checkpoint=Some(self.java_character.key.clone());}self.bootstrap_adapter=None;if let Some(book)=&self.java_participants.book{self.last_participants_checkpoint=Some(book.snapshot());}if self.java_self.java_snapshot_present(){self.last_self_checkpoint=Some(self.java_room_self_snapshot());}if let Some(book)=&self.java_settings.book{self.last_settings_checkpoint=Some(book.snapshot());}self.java_settings.retire();self.archive_retire_java_room_map_request();self.archive_retire_java_room_settings_request();self.archive_retire_java_room_mode_request();self.archive_retire_java_room_ready_request();self.java_self.retire();self.java_participants.retire();if let Some(catalog)=&self.java_tail.catalog{self.last_tail_checkpoint=Some(catalog.snapshot());}self.java_tail.retire();if self.java_room.key.phase!=java_room_entry::Phase::Idle{self.last_room_checkpoint=Some(self.java_room.key.clone());}self.java_room.retire()?;self.java_return.clear();self.java_character=java_character_flow::State::default();self.login_bootstrap.retire();self.entry_transfer.retire()?;self.entry_navigation.retire()?;self.login_events.close();self.login_transport=None;self.login_form.retire()?;if self.source_scope.bound(){self.source_scope.retire();self.awaiting_source_roster=false;self.character_scene.retire()?;self.source_roster_revision=0;self.source_actor=source_actor::Session::default();self.source_send=source_send::Sender::default();self.source_stream=source_stream::Receiver::default();self.source_identity=source_identity::State::default();self.source_flow=SourceFlow::default();}self.shutdown.store(true,Ordering::Release);self.commands.take();self.skin=None;self.page=if self.worker.is_none()&&self.login_events.idle(){UiPage::Closed}else{UiPage::Closing};self.status=if self.page==UiPage::Closed{"已結束"}else{"正在結束背景工作…"}.into();}
  }Ok(())
 }
 pub fn poll(&mut self)->bool {let mut changed=self.poll_source_transfer_timer();changed|=self.poll_player_fixture_events();
  for _ in 0..16{let Ok(event)=self.events.try_recv() else{break;};changed=true;
   if matches!(self.page,UiPage::Closing|UiPage::Closed){self.stale_results_rejected+=1;continue;}
   match event {
    Event::Ready(rows,skin)=>{if self.page==UiPage::Booting{self.skin=skin;self.selected=None;self.menu=Some(Menu::for_filter(&rows,Filter::Mode{mode:1,alternate:false}).expect("validated catalog mode"));self.rows=rows;self.page=UiPage::Menu;self.status="選擇模式與場地後進入離線預覽；特殊項目不提供離線配對".into();}},
    Event::InitFailed(error)=>{self.page=UiPage::Menu;self.status=format!("無法讀取場地：{}",error.chars().take(500).collect::<String>());},
    Event::Frame{generation,pixels}=>{if generation!=self.generation||self.choice().is_none_or(|c|c.mode!=pixels.mode||c.map_id!=pixels.map_id)||!matches!(self.page,UiPage::Loading|UiPage::Preview)||self.cancel.as_ref().is_none_or(|c|c.load(Ordering::Acquire)){self.stale_results_rejected+=1;continue;}self.revision+=1;let guard=self.guard.get_or_insert_with(||Arc::new(AtomicU64::new(0))).clone();guard.store(self.revision,Ordering::Release);self.frame=Some(UiFrame{pixels:Arc::new(pixels),guard,revision:self.revision});self.page=UiPage::Preview;self.pan_pending=false;self.status="離線預覽 · 方向鍵移動視角，Esc 返回".into();},
    Event::Failed{generation,error}=>{if generation!=self.generation{self.stale_results_rejected+=1;continue;}self.invalidate();self.page=UiPage::Menu;self.status=format!("載入失敗，可重試或返回：{}",error.chars().take(500).collect::<String>());}
   }
  }
  if self.worker.as_ref().is_some_and(JoinHandle::is_finished){let result=self.worker.take().unwrap().join();changed=true;if self.page==UiPage::Closing&&self.login_events.idle(){self.page=UiPage::Closed;self.status="已結束".into();}else if result.is_err(){self.invalidate();self.page=UiPage::Menu;self.status="背景工作已停止，請退出後重新開啟預覽".into();}}
  if self.page==UiPage::Closing&&self.worker.is_none()&&self.login_events.idle(){self.page=UiPage::Closed;self.status="已結束".into();changed=true;}
  changed
 }
}
impl Drop for Runtime {fn drop(&mut self){self.login_events.close();self.invalidate();self.shutdown.store(true,Ordering::Release);self.commands.take();/* normal UI waits for Closed; emergency drop only requests cooperative cleanup */}}

