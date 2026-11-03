from r491_common import *
def edit(name,pairs):
 p=P/'src/modern_lito'/name;s=p.read_text(encoding='utf-8')
 for a,b in pairs:
  assert a in s,(name,a[:90]);s=s.replace(a,b)
 p.write_text(s,encoding='utf-8')
edit('native_city_presentation.rs',[('pub struct Key {','pub struct Key {\n pub character_ui:super::character_selection::Key,')])
edit('native_city_runtime.rs',[
 ('#[path="native_city_presentation.rs"]','#[path="native_character_selection.rs"]pub mod character_selection;\n#[path="native_city_presentation.rs"]'),
 ('pub struct SourceCharacterControl {epoch:SourceEpoch,revision:u64,slot:usize}','pub struct SourceCharacterControl {epoch:SourceEpoch,revision:u64,scene_revision:u64,slot:usize}\n#[derive(Clone)]pub struct SourceCharacterSubmit {control:SourceCharacterControl,selection_revision:u64}'),
 ('pub struct Runtime {source_roster_revision:','pub struct Runtime {character_scene:character_selection::Scene,source_roster_revision:'),
 ('Ok(Self{source_roster_revision:0','Ok(Self{character_scene:character_selection::Scene::default(),source_roster_revision:0'),
 ('self.source_roster_revision=0;self.source_actor=','self.character_scene.retire()?;self.source_roster_revision=0;self.source_actor='),
 ('self.source_identity=next;self.source_roster_revision=revision;Ok(change)','self.character_scene.retire()?;self.source_identity=next;self.source_roster_revision=revision;Ok(change)'),
 ('presentation::Key{epoch:','presentation::Key{character_ui:self.character_scene.key(),epoch:'),
 ('revision:self.source_roster_revision,slot})','revision:self.source_roster_revision,scene_revision:self.character_scene.key().scene_revision,slot})'),
 ('control.revision==self.source_roster_revision&&self.source_character_control_enabled','control.revision==self.source_roster_revision&&control.scene_revision==self.character_scene.key().scene_revision&&self.source_character_control_enabled'),
 (' pub fn source_pending_write_ticket', ''' ///415C40(+34) normal release ->4934A0:select only;414A80 WM203
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
 pub fn leave_source_character_scene(&mut self)->Result<()>{self.character_scene.retire()}
 ///Native493150 receiver6 calls496E30/46B670;full producer teardown OPEN.
 ///Modern candidate Back retires this owned epoch, never issues Native logout.
 pub fn return_source_character_scene(&mut self)->Result<()>{ensure!(self.source_send.snapshot().pending.is_none(),"selected character write pending");let epoch=self.source_scope.current_epoch()?;self.disconnect_source_connection_candidate(&epoch)}
 pub fn source_pending_write_ticket''')])
edit('native_mode_menu_skin.rs',[
 ('pub entry_art:Option<Arc<[u8]>>,','pub entry_art:Option<Arc<[u8]>>,pub character_buttons:Vec<Img>,'),
 ('  ensure!(table.len()==1623,','  let mut character_buttons=Vec::new();if let Some(rows)=m.get("character_buttons"){let rows=rows.as_array().context("character button rows")?;ensure!(rows.len()==4,"four original character button images");for(row,id)in rows.iter().zip([6303,6304,6201,6202]){ensure!(row["id"]==id,"character button identity");let range=table.get(&format!("{id}.img")).context("character button absent")?;let raw=&pak[range.clone()];ensure!(raw.len()as u64==row["bytes"].as_u64().context("button size")?&&format!("{:X}",Sha256::digest(raw))==row["sha256"].as_str().context("button SHA")?,"character button pin drift");let image=decode_img(raw,CopyMode::ColorKey)?;ensure!((image.width,image.height)==(51,23),"character button geometry");character_buttons.push(image);}}\n  ensure!(table.len()==1623,'),
 ('down_pressed_bgra,entry_art,thumb','down_pressed_bgra,entry_art,character_buttons,thumb')])
edit('native_city_window.rs',[
 ('SourceCharacterControl,source_character_command_slot','SourceCharacterControl,SourceCharacterSubmit,source_character_command_slot'),
 ('const COUNT:usize=18;','const CHARACTER_SUBMIT:usize=18;const COUNT:usize=19;'),
 ('struct State {refresh_gate:','struct State {character_submit:RefCell<Option<SourceCharacterSubmit>>,refresh_gate:'),
 (' *s.character_controls.borrow_mut()=',' *s.character_submit.borrow_mut()=s.runtime.borrow().as_ref().and_then(Runtime::source_character_submit);\n *s.character_controls.borrow_mut()='),
 ('match i{SOURCE_ENTRY=>entry','match i{CHARACTER_SUBMIT=>entry&&roster,SOURCE_ENTRY=>entry'),
 ('BACK=>!entry','BACK=>!entry||roster'),
 ('match i{SOURCE_ENTRY=>s.runtime','match i{CHARACTER_SUBMIT=>s.character_submit.borrow().is_some(),SOURCE_ENTRY=>s.runtime'),
 ('let text={let r=s.runtime.borrow();r.as_ref().and_then','let text={let r=s.runtime.borrow();let selected=r.as_ref().is_some_and(|r|r.presentation_key().character_ui.selected==Some(slot));let prefix=if selected{"● "}else{""};r.as_ref().and_then'),
 ('format!("角色 {} · {}",slot+1','format!("{prefix}角色 {} · {}",slot+1'),
 ('format!("角色 {}",slot+1)}).unwrap','format!("{prefix}角色 {}",slot+1)}).unwrap'),
 ('for slot in 0..4{let w=','let _=MoveWindow(c[CHARACTER_SUBMIT],m+scale(310,dpi),scale(338,dpi),scale(220,dpi),scale(32,dpi),true);for slot in 0..4{let w='),
 ('unsafe fn back(hwnd:HWND,s:&State){','unsafe fn back(hwnd:HWND,s:&State){if s.screen.get()==Screen::Entry&&s.runtime.borrow().as_ref().is_some_and(|r|r.source_identity().complete()&&!r.source_identity().records().is_empty()){let result=s.runtime.borrow_mut().as_mut().unwrap().return_source_character_scene();if let Err(e)=result{label(s.controls.get()[STATUS],&e.to_string());}else{refresh(hwnd,s);}return;}'),
 ('else if s.screen.get()==Screen::Maps{s.screen.set','else if s.screen.get()==Screen::Maps{if let Some(r)=s.runtime.borrow_mut().as_mut(){let _=r.leave_source_character_scene();}s.screen.set'),
 ('&&(('+'wparam.0>>16)&0xffff)==BN_CLICKED as usize=>{let slot=','&&matches!((wparam.0>>16)&0xffff,n if n==BN_CLICKED as usize||n==BN_DOUBLECLICKED as usize)=>{let slot='),
 ('r.handle_source_character_control(control)','r.choose_source_character_control(control,((wparam.0>>16)&0xffff)==BN_DOUBLECLICKED as usize)'),
 ('  WM_COMMAND if (wparam.0&0xffff)==usize::from(SOURCE_MENU_COMMAND_ID)', '  WM_COMMAND if (wparam.0&0xffff)==FIRST+CHARACTER_SUBMIT&&((wparam.0>>16)&0xffff)==BN_CLICKED as usize=>{let token=s.character_submit.borrow().clone();let result={let mut r=s.runtime.borrow_mut();r.as_mut().map(|r|token.as_ref().ok_or_else(||anyhow::anyhow!("請先選擇角色")).and_then(|t|r.handle_source_character_submit(t)))};if let Some(Err(e))=result{label(s.controls.get()[STATUS],&e.to_string());}else{refresh(hwnd,s);}LRESULT(0)},\n  WM_COMMAND if (wparam.0&0xffff)==usize::from(SOURCE_MENU_COMMAND_ID)'),
 ('Some(Ok(()))=>{s.screen.set','Some(Ok(()))=>{if let Some(r)=s.runtime.borrow_mut().as_mut(){let _=r.leave_source_character_scene();}s.screen.set'),
 ('Some(OFFLINE)=>{s.screen.set','Some(OFFLINE)=>{if let Some(r)=s.runtime.borrow_mut().as_mut(){let _=r.leave_source_character_scene();}s.screen.set'),
 ('WM_CAPTURECHANGED|WM_KILLFOCUS=>{s.menu_mut','WM_CAPTURECHANGED|WM_KILLFOCUS=>{if let Some(r)=s.runtime.borrow_mut().as_mut(){r.cancel_source_character_press();}s.menu_mut'),
 ('State{refresh_gate:','State{character_submit:RefCell::new(None),refresh_gate:'),
 ('"角色 4","場地選單"];','"角色 4","場地選單","進入角色"];'),
 ('WS_CHILD|WS_VISIBLE|WS_TABSTOP)};','WS_CHILD|WS_VISIBLE|WS_TABSTOP|WINDOW_STYLE(if (CHARACTER_FIRST..CHARACTER_FIRST+4).contains(&i){BS_NOTIFY as u32}else{0}))};'),
 ('else if i>=CHARACTER_FIRST{','else if (CHARACTER_FIRST..CHARACTER_FIRST+4).contains(&i){')])
print('R498 source edits complete;mapping/card geometry gaps retained')
