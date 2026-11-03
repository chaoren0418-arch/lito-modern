from r491_common import *
def edit(name,pairs):
 p=P/'src/modern_lito'/name;s=p.read_text(encoding='utf-8')
 for a,b in pairs:assert a in s,(name,a[:70]);s=s.replace(a,b)
 p.write_text(s,encoding='utf-8')
edit('native_source_send.rs',[
 ('SelectCharacter{slot:usize},CompatibilityLogin','SelectCharacter{slot:usize},ReturnLogin,CompatibilityLogin'),
 ('  ensure!(self.pending.is_none(),"source write still pending");let before=', '  ensure!(name.len()<28&&!name.contains(&0),"selected source name exceeds NativeNUL capacity");let mut body=Vec::with_capacity(name.len()+2);body.push(201);body.extend_from_slice(name);body.push(0);self.prepare_body(body)\n }\n pub(crate) fn prepare_return_login(&self)->Result<Prepared>{self.prepare_body(vec![15])}\n fn prepare_body(&self,mut body:Vec<u8>)->Result<Prepared>{\n  ensure!(self.pending.is_none(),"source write still pending");let before='),
 ('  ensure!(name.len()<28&&!name.contains(&0),"selected source name exceeds NativeNUL capacity");\n  let mut body=Vec::with_capacity(name.len()+2);body.push(201);body.extend_from_slice(name);body.push(0);\n','')])
edit('native_source_identity.rs',[('    pub(crate) fn prepare_select','    ///NativeBack retains raw62FAA0 roster bytes;Modern retireslogicalselected binding.\n    pub(crate) fn without_selected(&self)->Self{let mut next=self.clone();next.selected=None;next}\n    pub(crate) fn prepare_select')])
edit('native_mode_state_flow.rs',[(' pub fn phase(&self)',' ///Source46B670 writes64BBD4=5 afterc15;retainedmodeglobals are not reset.\n ///This is action-owned, not a caller-supplied phase setter.\n pub(crate) fn returned_to_login(&self)->Self{let mut next=self.clone();next.phase=5;next}\n pub fn phase(&self)')])
edit('native_character_selection.rs',[('#[derive(Default)]pub(crate) struct Scene','#[derive(Clone,Default)]pub(crate) struct Scene')])
edit('native_city_presentation.rs',[(' pub character_ui:',' pub awaiting_roster:bool,\n pub character_ui:')])
edit('native_city_runtime.rs',[
 ('#[derive(Clone)]pub struct SourceCharacterSubmit','#[derive(Clone)]pub struct SourceCharacterBack {epoch:SourceEpoch,revision:u64,scene_revision:u64}\n#[derive(Clone)]pub struct SourceCharacterSubmit'),
 ('pub struct Runtime {character_scene:','pub struct Runtime {awaiting_source_roster:bool,character_scene:'),
 ('Ok(Self{character_scene:','Ok(Self{awaiting_source_roster:false,character_scene:'),
 ('self.character_scene.retire()?;self.source_roster_revision=0;','self.awaiting_source_roster=false;self.character_scene.retire()?;self.source_roster_revision=0;'),
 ('self.source_identity=next;self.source_roster_revision=revision;if self.source_identity.complete()','self.source_identity=next;self.source_roster_revision=revision;if matches!(change,source_identity::Change::Started{..}){self.awaiting_source_roster=false;}if self.source_identity.complete()&&!self.awaiting_source_roster'),
 ('presentation::Key{character_ui:','presentation::Key{awaiting_roster:self.awaiting_source_roster,character_ui:'),
 ('self.character_scene.key().active&&self.page==UiPage::Menu','!self.awaiting_source_roster&&self.character_scene.key().active&&self.page==UiPage::Menu'),
 ('ensure!(self.page==UiPage::Menu&&self.source_scope.active_serial().is_some()&&self.source_identity.complete(),"character scene unavailable")','ensure!(!self.awaiting_source_roster&&self.page==UiPage::Menu&&self.source_scope.active_serial().is_some()&&self.source_identity.complete(),"character scene unavailable")'),
 (''' ///Native493150 receiver6 calls496E30/46B670;full producer teardown OPEN.
 ///Modern candidate Back retires this owned epoch, never issues Native logout.
 pub fn return_source_character_scene(&mut self)->Result<()>{ensure!(self.source_send.snapshot().pending.is_none(),"selected character write pending");let epoch=self.source_scope.current_epoch()?;self.disconnect_source_connection_candidate(&epoch)}''',''' ///Native493150 receiver6 ->496E30 ->46B670 reserves c15 before local
 ///cleanup and64BBD4=5. Preserve this connection's seed/version/RX/TX.
 ///Nativedynamicdestructors/authentication/loginpanel remain unqualified.
 pub fn source_character_back(&self)->Option<SourceCharacterBack>{if self.awaiting_source_roster||!self.character_scene.key().active||self.page!=UiPage::Menu||!self.source_identity.complete()||self.source_send.snapshot().counter.is_none()||self.source_send.snapshot().pending.is_some(){return None;}Some(SourceCharacterBack{epoch:self.source_scope.current_epoch().ok()?,revision:self.source_roster_revision,scene_revision:self.character_scene.key().scene_revision})}
 pub fn handle_source_character_back(&mut self,control:&SourceCharacterBack)->Result<()>{
  self.source_scope.admit_epoch(&control.epoch)?;
  ensure!(control.revision==self.source_roster_revision&&control.scene_revision==self.character_scene.key().scene_revision&&self.source_character_back().is_some(),"retired source character Back");
  let prepared=self.source_send.prepare_return_login()?;
  let revision=self.source_roster_revision.checked_add(1).context("source roster revision exhausted")?;let mut scene=self.character_scene.clone();scene.retire()?;
  let identity=self.source_identity.without_selected();let flow=self.source_flow.returned_to_login();
  self.next_generation()?;self.invalidate();self.selected=None;if let Some(menu)=self.menu.as_mut(){menu.clear_items();}
  self.source_send.commit(prepared);self.character_scene=scene;self.source_identity=identity;self.source_flow=flow;self.source_roster_revision=revision;self.awaiting_source_roster=true;
  self.page=UiPage::Menu;self.status="已返回入口，返回要求等待傳送；等待新角色名單".into();if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}Ok(())
 }
 pub fn return_source_character_scene(&mut self)->Result<()>{let control=self.source_character_back().context("source character Back unavailable")?;self.handle_source_character_back(&control)}'''),
 ('  let source_send::Intent::SelectCharacter{slot}=intent else','  if matches!(intent,source_send::Intent::ReturnLogin){self.return_source_character_scene()?;anyhow::bail!("ReturnLogin reserved throughownedBack; usependingticket view");}\n  let source_send::Intent::SelectCharacter{slot}=intent else')])
# ReturnLogin is deliberately not offered through generic send:onlyopaqueBack.
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8');s=s.replace('  if matches!(intent,source_send::Intent::ReturnLogin){self.return_source_character_scene()?;anyhow::bail!("ReturnLogin reserved throughownedBack; usependingticket view");}\n','');p.write_text(s,encoding='utf-8')
edit('native_city_window.rs',[
 ('SourceCharacterControl,SourceCharacterSubmit,','SourceCharacterControl,SourceCharacterSubmit,SourceCharacterBack,'),
 ('struct State {character_submit:','struct State {character_back:RefCell<Option<SourceCharacterBack>>,character_submit:'),
 (' *s.character_submit.borrow_mut()=',' *s.character_back.borrow_mut()=s.runtime.borrow().as_ref().and_then(Runtime::source_character_back);\n *s.character_submit.borrow_mut()='),
 ('r.source_connection_epoch().is_some()&&r.source_identity().complete()','r.presentation_key().character_ui.active&&r.source_connection_epoch().is_some()&&r.source_identity().complete()'),
 ('r.source_identity().complete()&&!r.source_identity().records().is_empty()','r.presentation_key().character_ui.active&&r.source_identity().complete()&&!r.source_identity().records().is_empty()'),
 ('if s.screen.get()==Screen::Entry&&s.runtime.borrow().as_ref().is_some_and(|r|r.presentation_key().character_ui.active&&r.source_identity().complete()&&!r.source_identity().records().is_empty()){let result=s.runtime.borrow_mut().as_mut().unwrap().return_source_character_scene();','if s.screen.get()==Screen::Entry&&s.runtime.borrow().as_ref().is_some_and(|r|r.source_connection_epoch().is_some()){let control=s.character_back.borrow().clone();let result=control.as_ref().ok_or_else(||anyhow::anyhow!("目前未在角色選取畫面")).and_then(|control|s.runtime.borrow_mut().as_mut().unwrap().handle_source_character_back(control));'),
 ('State{character_submit:','State{character_back:RefCell::new(None),character_submit:')])
print('R499 c15-ownedBack preservesepoch/RX/TX/version;retiresUI/selectedbinding;freshsource3required')
