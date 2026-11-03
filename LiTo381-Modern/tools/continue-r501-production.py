from r491_common import *
import shutil
root=P/'target/r501a-failed-source';root.mkdir(exist_ok=False)
sources=[]
for name in ['native_city_runtime.rs','native_login_form.rs','native_source_send.rs','native_login_transport.rs','native_login_wire.rs']:
 p=P/'src/modern_lito'/name;d=root/name;shutil.copyfile(p,d);sources.append(pin(d))
save(L/'r501a-build-failure.json',dict(state='R501A_BUILD_FAILED_PRESERVED_BEFORE_REPAIR',exit_code=101,reason='E0599 finish_source_stream_candidate; actual method is end_source_stream_candidate',sources=sources,logs=[pin(L/'r501a-library-build.jsonl'),pin(L/'r501a-library-build.stderr.log')],tools_read_failure='Two initial read commands omitted workdir and failed at cwd D:/lineage開發; no writes; corrected workdir on next call'))
p=P/'src/modern_lito/native_login_transport.rs';s=p.read_text(encoding='utf-8');assert 'finish_source_stream_candidate' in s;s=s.replace('finish_source_stream_candidate','end_source_stream_candidate');p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8');s=s.replace('self.page==UiPage::Menu&&!self.character_scene.key().active&&self.source_send.snapshot().pending.is_none()','self.page==UiPage::Menu&&!self.character_scene.key().active&&self.source_send.snapshot().pending.is_none()&&!matches!(self.login_form.snapshot().stage,login_form::Stage::Sending|login_form::Stage::AwaitingReply)')
old='self.login_form.admit(c)?;self.login_form.retire()?;self.status="登入資料已清除"';new='self.login_form.admit(c)?;ensure!(!matches!(self.login_form.snapshot().stage,login_form::Stage::Sending|login_form::Stage::AwaitingReply),"login write or reply pending; disconnect or Close required");self.login_form.retire()?;self.status="登入資料已清除"';assert old in s;s=s.replace(old,new)
s=s.replace('pub fn leave_source_character_scene(&mut self)->Result<()>{self.login_form.retire()?','pub fn leave_source_character_scene(&mut self)->Result<()>{ensure!(!matches!(self.login_form.snapshot().stage,login_form::Stage::Sending|login_form::Stage::AwaitingReply),"login write or reply pending");self.login_form.retire()?')
p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_city_window.rs';s=p.read_text(encoding='utf-8');s=s.replace('BACK=>login_active||!entry||s.character_back.borrow().is_some()','BACK=>(login_active&&!matches!(login.stage,LoginStage::Sending|LoginStage::AwaitingReply))||!entry||s.character_back.borrow().is_some()');p.write_text(s,encoding='utf-8')
print('R501a failure preserved; EOF method and pending-login lifecycle guards repaired')
