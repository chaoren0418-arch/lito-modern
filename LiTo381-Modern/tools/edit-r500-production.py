from r491_common import *
def edit(name,pairs):
 p=P/'src/modern_lito'/name;s=p.read_text(encoding='utf-8')
 for a,b in pairs:assert a in s,(name,a[:70]);s=s.replace(a,b)
 p.write_text(s,encoding='utf-8')
edit('native_city_presentation.rs',[(' pub awaiting_roster:bool,',' pub login_form:super::login_form::Snapshot,\n pub awaiting_roster:bool,')])
edit('native_city_runtime.rs',[
 ('#[path="native_character_selection.rs"]pub mod character_selection;','#[path="native_character_selection.rs"]pub mod character_selection;\n#[path="native_login_form.rs"]pub mod login_form;'),
 ('pub struct Runtime {awaiting_source_roster:','pub struct Runtime {login_form:login_form::Session,awaiting_source_roster:'),
 ('Ok(Self{awaiting_source_roster:','Ok(Self{login_form:login_form::Session::default(),awaiting_source_roster:'),
 ('presentation::Key{awaiting_roster:','presentation::Key{login_form:self.login_form.snapshot(),awaiting_roster:'),
 ('self.awaiting_source_roster=false;self.character_scene.retire()?;','self.login_form.retire()?;self.awaiting_source_roster=false;self.character_scene.retire()?;'),
 ('self.character_scene.retire()?;self.source_identity=next;','self.login_form.retire()?;self.character_scene.retire()?;self.source_identity=next;'),
 ('self.source_send.commit(prepared);self.character_scene=scene;','self.login_form.retire()?;self.source_send.commit(prepared);self.character_scene=scene;'),
 (' pub fn source_identity(&self)',' ///LocalModern form lifecycle for631E38/492270;no sourcephase/roster setters.\n pub fn source_login_snapshot(&self)->login_form::Snapshot{self.login_form.snapshot()}\n pub fn source_login_control(&self)->Option<login_form::Control>{self.login_form.control()}\n pub fn open_source_login_form(&mut self)->Result<()>{ensure!(self.page==UiPage::Menu&&!self.character_scene.key().active&&self.source_send.snapshot().pending.is_none(),"登入畫面目前無法開啟");self.login_form.open()?;self.status="輸入帳號與密碼。連線登入尚未開放。".into();Ok(())}\n pub fn edit_source_login_form(&mut self,c:&login_form::Control,account:&[u8],password:&[u8])->Result<()>{ensure!(self.page==UiPage::Menu,"login form unavailable");self.login_form.edit(c,account,password)}\n ///Onlyvalidate/normalize/stageownedinput. AwaitingAuthoritiesisLOCAL UI,notNativephase/authACK.\n pub fn stage_source_login_form(&mut self,c:&login_form::Control)->Result<()>{ensure!(self.page==UiPage::Menu,"login form unavailable");self.login_form.stage(c)?;self.status="登入資料已準備；連線登入尚未開放。可按返回清除。".into();Ok(())}\n pub fn cancel_source_login_form(&mut self,c:&login_form::Control)->Result<()>{ensure!(self.page==UiPage::Menu,"login form unavailable");self.login_form.admit(c)?;self.login_form.retire()?;self.status="登入資料已清除".into();Ok(())}\n pub fn source_identity(&self)')])
print('R500 localownedloginSession added;sourceboundary/roster/Back/Close retire secrets')
