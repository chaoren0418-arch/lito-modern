from r491_common import *
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8');s=s.replace(' pub fn leave_source_character_scene(&mut self)->Result<()>{self.character_scene.retire()}',' pub fn leave_source_character_scene(&mut self)->Result<()>{self.login_form.retire()?;self.character_scene.retire()}')
s=s.replace('pub fn source_menu_invoker_enabled(&self)->bool{!self.awaiting_source_roster','pub fn source_menu_invoker_enabled(&self)->bool{self.login_form.snapshot().stage==login_form::Stage::Hidden&&!self.awaiting_source_roster')
a='self.next_generation()?;self.invalidate();if self.source_scope.bound(){self.source_scope.retire();self.login_form.retire()?;';new='self.next_generation()?;self.invalidate();self.login_form.retire()?;if self.source_scope.bound(){self.source_scope.retire();'
if a in s:s=s.replace(a,new)
else:assert new in s
p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_city_window.rs';s=p.read_text(encoding='utf-8')
def rep(a,b):
 global s
 assert a in s,a[:90];s=s.replace(a,b,1)
rep('source_character_command_slot,presentation::Gate','source_character_command_slot,login_form::{Control as LoginControl,Stage as LoginStage},presentation::Gate')
rep('const COUNT:usize=19;','const LOGIN_ACCOUNT_LABEL:usize=19;const LOGIN_ACCOUNT:usize=20;const LOGIN_PASSWORD_LABEL:usize=21;const LOGIN_PASSWORD:usize=22;const LOGIN_SUBMIT:usize=23;const COUNT:usize=24;')
rep('struct State {character_back:','struct State {login_control:RefCell<Option<LoginControl>>,login_stage:Cell<LoginStage>,login_publishing:Cell<bool>,character_back:')
rep(' *s.character_back.borrow_mut()=', ''' let login=s.runtime.borrow().as_ref().unwrap().source_login_snapshot();let login_active=login.stage!=LoginStage::Hidden;
 *s.login_control.borrow_mut()=s.runtime.borrow().as_ref().and_then(Runtime::source_login_control);
 if s.login_stage.replace(login.stage)!=login.stage{s.login_publishing.set(true);if login.stage!=LoginStage::AwaitingAuthorities{label(s.controls.get()[LOGIN_ACCOUNT],"");}label(s.controls.get()[LOGIN_PASSWORD],"");s.login_publishing.set(false);}
 *s.character_back.borrow_mut()=''')
rep('if entry{"選擇登入方式。離線場地選單標示資源是否已合格。"}','if entry&&login_active{"輸入帳號與密碼。連線登入尚未開放；返回會清除輸入資料。"}else if entry{"選擇登入方式。離線場地選單標示資源是否已合格。"}')
rep('match i{CHARACTER_SUBMIT=>','match i{LOGIN_ACCOUNT_LABEL|LOGIN_ACCOUNT|LOGIN_PASSWORD_LABEL|LOGIN_PASSWORD|LOGIN_SUBMIT=>entry&&login_active,CHARACTER_SUBMIT=>')
rep('SOURCE_ENTRY=>entry&&','SOURCE_ENTRY=>entry&&!login_active&&')
rep('NETWORK|OFFLINE=>entry,','NETWORK|OFFLINE=>entry&&!login_active,')
rep('BACK=>!entry||roster,','BACK=>!entry||roster||login_active,')
rep('match i{CHARACTER_SUBMIT=>s.character_submit','match i{LOGIN_ACCOUNT|LOGIN_PASSWORD|LOGIN_SUBMIT=>login.stage==LoginStage::Editing,NETWORK=>page==UiPage::Menu&&!roster,CHARACTER_SUBMIT=>s.character_submit')
rep('BACK=>!entry||s.character_back.borrow().is_some()','BACK=>login_active||!entry||s.character_back.borrow().is_some()')
rep('}let _=MoveWindow(c[SOURCE_ENTRY]', '''}for(i,x,y,w,h)in [(LOGIN_ACCOUNT_LABEL,m,120,90,28),(LOGIN_ACCOUNT,m+scale(96,dpi),120,250,28),(LOGIN_PASSWORD_LABEL,m,160,90,28),(LOGIN_PASSWORD,m+scale(96,dpi),160,250,28),(LOGIN_SUBMIT,m+scale(96,dpi),204,120,32)]{let _=MoveWindow(c[i],x,scale(y,dpi),scale(w,dpi),scale(h,dpi),true);}let _=MoveWindow(c[SOURCE_ENTRY]''')
rep('unsafe fn back(hwnd:HWND,s:&State){','''unsafe fn login_edit(s:&State)->Result<()> {
 use zeroize::Zeroizing;let c=s.controls.get();let mut a=Zeroizing::new([0u16;28]);let mut p=Zeroizing::new([0u16;14]);ensure!(GetWindowTextLengthW(c[LOGIN_ACCOUNT])<=27&&GetWindowTextLengthW(c[LOGIN_PASSWORD])<=13,"登入欄位超過來源容量");let an=GetWindowTextW(c[LOGIN_ACCOUNT],&mut a[..])as usize;let pn=GetWindowTextW(c[LOGIN_PASSWORD],&mut p[..])as usize;ensure!(a[..an].iter().chain(&p[..pn]).all(|n|(32..=126).contains(n)),"登入欄位尚未支援此字元編碼");let account=Zeroizing::new(a[..an].iter().map(|n|*n as u8).collect::<Vec<_>>());let password=Zeroizing::new(p[..pn].iter().map(|n|*n as u8).collect::<Vec<_>>());let control=s.login_control.borrow().clone().ok_or_else(||anyhow::anyhow!("登入控制已失效"))?;s.runtime.borrow_mut().as_mut().unwrap().edit_source_login_form(&control,&account,&password)
}
unsafe fn login_submit(hwnd:HWND,s:&State){let result=login_edit(s).and_then(|_|{let control=s.runtime.borrow().as_ref().and_then(Runtime::source_login_control).ok_or_else(||anyhow::anyhow!("登入控制已失效"))?;s.runtime.borrow_mut().as_mut().unwrap().stage_source_login_form(&control)});match result{Ok(())=>refresh(hwnd,s),Err(e)=>label(s.controls.get()[STATUS],&e.to_string())}}
unsafe fn back(hwnd:HWND,s:&State){if s.runtime.borrow().as_ref().is_some_and(|r|r.source_login_snapshot().stage!=LoginStage::Hidden){let control=s.login_control.borrow().clone();let result=control.as_ref().ok_or_else(||anyhow::anyhow!("登入控制已失效")).and_then(|c|s.runtime.borrow_mut().as_mut().unwrap().cancel_source_login_form(c));if let Err(e)=result{label(s.controls.get()[STATUS],&e.to_string());}else{refresh(hwnd,s);}return;}''')
rep('  WM_COMMAND if (wparam.0&0xffff)==usize::from(SOURCE_MENU_COMMAND_ID)', '''  WM_COMMAND if (wparam.0>>16)&0xffff==EN_CHANGE as usize&&matches!((wparam.0&0xffff).checked_sub(FIRST),Some(LOGIN_ACCOUNT|LOGIN_PASSWORD))=>{if !s.login_publishing.get(){match login_edit(s){Ok(())=>refresh(hwnd,s),Err(e)=>label(s.controls.get()[STATUS],&e.to_string())}}LRESULT(0)},
  WM_COMMAND if (wparam.0&0xffff)==usize::from(SOURCE_MENU_COMMAND_ID)''')
rep('Some(NETWORK)=>{s.outcome.set(Outcome::Network);action(hwnd,s,Action::Close);}', 'Some(NETWORK)=>{let result=s.runtime.borrow_mut().as_mut().unwrap().open_source_login_form();match result{Ok(())=>{refresh(hwnd,s);let _=SetFocus(Some(s.controls.get()[LOGIN_ACCOUNT]));},Err(e)=>label(s.controls.get()[STATUS],&e.to_string())}},Some(LOGIN_SUBMIT)=>login_submit(hwnd,s),')
rep('State{character_back:','State{login_control:RefCell::new(None),login_stage:Cell::new(LoginStage::Hidden),login_publishing:Cell::new(false),character_back:')
rep('"場地選單","進入角色"]','"場地選單","進入角色","帳號","","密碼","","登入"]')
rep('else if matches!(i,TITLE|INFO|STATUS)','else if matches!(i,LOGIN_ACCOUNT|LOGIN_PASSWORD){(w!("EDIT"),WS_CHILD|WS_TABSTOP|WS_BORDER|WINDOW_STYLE(ES_AUTOHSCROLL as u32|if i==LOGIN_PASSWORD{ES_PASSWORD as u32}else{0}))}else if matches!(i,TITLE|INFO|STATUS|LOGIN_ACCOUNT_LABEL|LOGIN_PASSWORD_LABEL)')
rep(' }for name in [',' }SendMessageW(controls[LOGIN_ACCOUNT],EM_SETLIMITTEXT,Some(WPARAM(27)),None);SendMessageW(controls[LOGIN_PASSWORD],EM_SETLIMITTEXT,Some(WPARAM(13)),None);for name in [')
rep('let page=state.runtime.borrow().as_ref().unwrap().page();if state.screen', 'let page=state.runtime.borrow().as_ref().unwrap().page();if msg.wParam.0==0x0d&&state.runtime.borrow().as_ref().is_some_and(|r|r.source_login_snapshot().stage==LoginStage::Editing)&&GetKeyState(0x11)>=0&&GetKeyState(0x12)>=0&&(msg.lParam.0&(1<<30))==0{login_submit(hwnd,&state);continue;}if state.screen')
p.write_text(s,encoding='utf-8')
print('R500 sameWindow account/passwordmask/login/cancel/callback bridges finalized;normal authsend unavailable')
