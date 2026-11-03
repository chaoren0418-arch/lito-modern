from r491_common import *
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8')
items=[
 ('OpenSourceMenu,SourceCharacter(usize)','OpenSourceMenu'),
 ('pub fn player_command_action(command:u16)->Option<Action>{if (SOURCE_CHARACTER_COMMAND_FIRST..SOURCE_CHARACTER_COMMAND_FIRST+4).contains(&command){Some(Action::SourceCharacter(usize::from(command-SOURCE_CHARACTER_COMMAND_FIRST)))}else{match command{SOURCE_MENU_COMMAND_ID=>Some(Action::OpenSourceMenu),_=>None}}}', 'pub fn source_character_command_slot(command:u16)->Option<usize>{(SOURCE_CHARACTER_COMMAND_FIRST..SOURCE_CHARACTER_COMMAND_FIRST+4).contains(&command).then(||usize::from(command-SOURCE_CHARACTER_COMMAND_FIRST))}\npub fn player_command_action(command:u16)->Option<Action>{match command{SOURCE_MENU_COMMAND_ID=>Some(Action::OpenSourceMenu),_=>None}}\n#[derive(Clone)]pub struct SourceCharacterControl {epoch:SourceEpoch,revision:u64,slot:usize}\nimpl SourceCharacterControl {pub fn slot(&self)->usize{self.slot}}'),
 ('pub struct Runtime {source_actor:','pub struct Runtime {source_roster_revision:u64,source_actor:'),
 ('Ok(Self{source_actor:','Ok(Self{source_roster_revision:0,source_actor:'),
 ('self.source_actor=source_actor::Session::default();','self.source_roster_revision=0;self.source_actor=source_actor::Session::default();'),
 ('let(next,change)=self.source_identity.prepare_packet(event.body())?;self.source_identity=next;Ok(change)', 'let revision=self.source_roster_revision.checked_add(1).context("source roster control revision exhausted")?;let(next,change)=self.source_identity.prepare_packet(event.body())?;self.source_identity=next;self.source_roster_revision=revision;Ok(change)'),
 (' pub fn source_pending_write_ticket(', ' pub fn source_character_control(&self,slot:usize)->Option<SourceCharacterControl>{if !self.source_character_control_enabled(slot){return None;}Some(SourceCharacterControl{epoch:self.source_scope.current_epoch().ok()?,revision:self.source_roster_revision,slot})}\n pub fn handle_source_character_control(&mut self,control:&SourceCharacterControl)->Result<()>{self.source_scope.admit_epoch(&control.epoch)?;ensure!(control.revision==self.source_roster_revision&&self.source_character_control_enabled(control.slot),"retired source character control");self.prepare_source_send_candidate(&control.epoch,source_send::Intent::SelectCharacter{slot:control.slot})?;self.status="角色已選取，等待傳送".into();Ok(())}\n pub fn source_pending_write_ticket('),
 ('   Action::SourceCharacter(slot)=>{ensure!(self.source_character_control_enabled(slot),"source character control unavailable");let epoch=self.source_scope.current_epoch()?;self.prepare_source_send_candidate(&epoch,source_send::Intent::SelectCharacter{slot})?;self.status="角色已選取，等待傳送".into();},\n','')]
for a,b in items:
 assert s.count(a)==(2 if a.startswith('self.source_actor=')else 1),(a,s.count(a));s=s.replace(a,b)
p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_city_window.rs';s=p.read_text(encoding='utf-8')
for a,b in [
 ('SOURCE_CHARACTER_COMMAND_FIRST}}','SOURCE_CHARACTER_COMMAND_FIRST,SourceCharacterControl,source_character_command_slot}}'),
 ('struct State {clock:Instant,','struct State {character_controls:RefCell<[Option<SourceCharacterControl>;4]>,clock:Instant,'),
 (' let roster=s.runtime.borrow()', ' *s.character_controls.borrow_mut()=std::array::from_fn(|slot|s.runtime.borrow().as_ref().and_then(|r|r.source_character_control(slot)));\n let roster=s.runtime.borrow()'),
 ('let result={let mut r=s.runtime.borrow_mut();r.as_mut().map(|r|r.handle_player_command((wparam.0&0xffff)as u16))};','let slot=source_character_command_slot((wparam.0&0xffff)as u16).unwrap();let control=s.character_controls.borrow()[slot].clone();let result={let mut r=s.runtime.borrow_mut();r.as_mut().map(|r|control.as_ref().ok_or_else(||anyhow::anyhow!("角色尚未可選取")).and_then(|control|r.handle_source_character_control(control)))};'),
 ('Box::new(State{clock:','Box::new(State{character_controls:RefCell::new(std::array::from_fn(|_|None)),clock:')]:
 assert s.count(a)==1,(a,s.count(a));s=s.replace(a,b)
p.write_text(s,encoding='utf-8')
history=read(L/'r496-preparation-history.json');history['production_library_builds_before_control_refinement']=1;history['reason_for_necessary_second_build']='Opaque displayed control gains epoch+roster-revision guard before first consumer/player link;first library never used for tests or player execution';save(L/'r496-preparation-history.json',history)
print('R496 displayed controls carry opaque epoch/roster revision;necessary second library build,zero tests rerun')
