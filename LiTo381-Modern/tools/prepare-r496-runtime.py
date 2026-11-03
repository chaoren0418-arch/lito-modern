from r491_common import *
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8')
for a,b,n in [
 ('pub struct Runtime {source_send:','pub struct Runtime {source_actor:source_actor::Session,source_send:',1),
 ('Ok(Self{source_send:','Ok(Self{source_actor:source_actor::Session::default(),source_send:',1),
 ('self.source_send=source_send::Sender::default();','self.source_actor=source_actor::Session::default();self.source_send=source_send::Sender::default();',2),
 ('  match action {','  match action {\n   Action::SourceCharacter(slot)=>{ensure!(self.source_character_control_enabled(slot),"source character control unavailable");let epoch=self.source_scope.current_epoch()?;self.prepare_source_send_candidate(&epoch,source_send::Intent::SelectCharacter{slot})?;self.status="角色已選取，等待傳送".into();},',1)]:
 assert s.count(a)==n,(a,s.count(a));s=s.replace(a,b)
p.write_text(s,encoding='utf-8')
print('R496 Runtime actor/session and actual player-command typed control bound')
