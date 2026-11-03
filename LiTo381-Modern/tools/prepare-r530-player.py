import pathlib,json,shutil
P=pathlib.Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';T=P/'target/r530-player-entry-candidate'
T.mkdir()
def write(p,s):
 with pathlib.Path(p).open('x',encoding='utf-8',newline='\n')as f:f.write(s)
s=(P/'tools/r528-status-io-provider.rs').read_text('utf-8').split('#[cfg(test)]')[0]
for n in ['now_ms','claim_status_poll','same_live_identity','fresh_status','sync_common','armed_getter']:
 s=s.replace('fn '+n+'(','pub(super) fn '+n+'(').replace("fn "+n+"<'a>(","pub(super) fn "+n+"<'a>(")
s=s.replace('"retired R527 READY prohibited"','"retired R527 READY prohibited"')
s=s.replace('let c:ReadyContract=serde_json::from_slice(raw)?;', 'ensure!(accepted_sha.to_ascii_uppercase()!="2B45D78545ADF12A0CF2FAD02FB1ABF1E9899E3F2AE5B40A8A9F68BBF900D5F4","retired R528 READY prohibited");\n let c:ReadyContract=serde_json::from_slice(raw)?;')
s=s.replace('"nw-975e97ed0f74480c9c92edd7d6b7d51e")','"nw-975e97ed0f74480c9c92edd7d6b7d51e"|"nw-c9afae2f2cd14da08208599b1bf066fb")')
s=s.replace('status_poll_started:std::cell::Cell::new(None)};', 'status_poll_started:std::cell::Cell::new(None),identities:rows.iter().map(|row|(u32::try_from(row["pid"].as_u64().unwrap()).unwrap(),row["creation_FILETIME_100ns"].as_str().unwrap().to_owned())).collect()};')
s=s[:s.index('pub fn synchronized_host')]
s+='''#[cfg(test)]mod r530_entry_guards {
 use super::*;
 #[test]fn digest_and_retired_supply_refuse_before_io(){
  assert!(static_ready(b"{}","bad").is_err());
  assert!(static_ready(b"{}","2B45D78545ADF12A0CF2FAD02FB1ABF1E9899E3F2AE5B40A8A9F68BBF900D5F4").is_err());
  assert!(static_ready(b"{}",&digest(b"{}")).is_err());
 }
 #[test]fn health_poll_budget_is_shared_not_callback_wait(){let last=Cell::new(None);let now=Instant::now();assert!(claim_status_poll(&last,now));assert!(!claim_status_poll(&last,now+Duration::from_millis(199)));assert!(claim_status_poll(&last,now+Duration::from_millis(200)));}
}
'''
write(P/'src/modern_lito/native_backend_485f_provider.rs',s)
write(P/'src/modern_lito/native_backend_485f_status_reader.rs',(P/'tools/r528-status-io-reader.rs').read_text('utf-8').split('#[cfg(test)]')[0])
mod=P/'src/modern_lito/mod.rs';s=mod.read_text('utf-8');assert s.count('pub mod native_city_window;')==1;s=s.replace('pub mod native_city_window;','pub mod native_city_window;\npub mod native_backend_485f_entry;');mod.write_text(s,'utf-8',newline='\n')
w=P/'src/modern_lito/native_city_window.rs';s=w.read_text('utf-8')
def rep(a,b):
 global s
 assert s.count(a)==1,(a,s.count(a));s=s.replace(a,b)
rep('struct State {','struct State {backend:RefCell<Option<super::native_backend_485f_entry::Accepted>>,')
rep('let result=panel.invoke(s.runtime.borrow_mut().as_mut().unwrap(),operation);','let result={let mut r=s.runtime.borrow_mut();let r=r.as_mut().unwrap();if let Some(b)=s.backend.borrow_mut().as_mut(){b.invoke(r,&panel,operation)}else{panel.invoke(r,operation)}};')
rep('label(s.controls.get()[LOGIN_ACCOUNT],"");','label(s.controls.get()[LOGIN_ACCOUNT],if s.backend.borrow().is_some(){"Fx485Guest"}else{""});')
rep('*s.channel_controls.borrow_mut()=std::array::from_fn(|slot|s.runtime.borrow().as_ref().and_then(|r|r.source_channel_control(slot)));','*s.channel_controls.borrow_mut()=std::array::from_fn(|slot|if s.backend.borrow().is_some()&&slot!=0{None}else{s.runtime.borrow().as_ref().and_then(|r|r.source_channel_control(slot))});')
rep('*s.server_controls.borrow_mut()=std::array::from_fn(|slot|s.runtime.borrow().as_ref().and_then(|r|r.source_transfer_server_control(slot)));','*s.server_controls.borrow_mut()=std::array::from_fn(|slot|if s.backend.borrow().is_some(){None}else{s.runtime.borrow().as_ref().and_then(|r|r.source_transfer_server_control(slot))});')
rep('let roster=s.runtime.borrow().as_ref().is_some_and(|r|r.presentation_key()', 'let backend_roster=s.backend.borrow().as_ref().is_none_or(|b|b.roster_allowed(s.runtime.borrow().as_ref().unwrap()));\n if !backend_roster{*s.character_controls.borrow_mut()=std::array::from_fn(|_|None);*s.character_submit.borrow_mut()=None;}\n let roster=backend_roster&&s.runtime.borrow().as_ref().is_some_and(|r|r.presentation_key()')
rep('(0..bootstrap.sources.len()).map(|slot|s.runtime.borrow().as_ref().unwrap().source_choice_control(slot))','(0..bootstrap.sources.len()).map(|slot|if s.backend.borrow().as_ref().is_none_or(|b|b.source_allowed(&bootstrap.sources[slot])){s.runtime.borrow().as_ref().unwrap().source_choice_control(slot)}else{None})')
rep('label(c[NETWORK],if restart', 'label(c[NETWORK],if s.backend.borrow().is_some(){"Backend485F 驗收登入"}else if restart')
rep('if entry&&in_room{"可按準備或取消準備；狀態依伺服器回包更新。"}', 'if entry&&in_room&&s.backend.borrow().is_some(){"先取消準備送出基線；收到 Host 準備後可按一次準備，再依實收倒數取消。"}else if entry&&in_room{"可按準備或取消準備；狀態依伺服器回包更新。"}')
rep('else if entry&&s.runtime.borrow().as_ref().is_some_and(Runtime::synthetic_player_fixture_configured){"合成 loopback', 'else if entry&&s.backend.borrow().is_some(){"使用 Fx485Guest 與供應者提供的密碼；限這次供應的一次連線。"}else if entry&&s.runtime.borrow().as_ref().is_some_and(Runtime::synthetic_player_fixture_configured){"合成 loopback')
rep('let enabled=!closing&&match i{','let backend_room_op=match i{ROOM_READY=>Some(RoomOperation::Ready),ROOM_CANCEL=>Some(RoomOperation::Cancel),ROOM_REFRESH=>Some(RoomOperation::Refresh),ROOM_JOIN=>usize::try_from(SendMessageW(c[ROOM_LIST],CB_GETCURSEL,None,None).0).ok().map(RoomOperation::Join),_=>None};let backend_ok=backend_room_op.is_none_or(|op|s.backend.borrow().as_ref().is_none_or(|b|b.permits(s.runtime.borrow().as_ref().unwrap(),&s.room_panel.borrow(),op).is_ok()));let enabled=!closing&&backend_ok&&match i{')
rep('CHARACTER_FIRST..=16=>s.runtime.borrow().as_ref().is_some_and(|r|r.source_character_control_enabled','CHARACTER_FIRST..=16=>backend_roster&&s.runtime.borrow().as_ref().is_some_and(|r|r.source_character_control_enabled')
rep('unsafe fn login_submit(hwnd:HWND,s:&State){let result=login_edit(s)', '''unsafe fn login_submit(hwnd:HWND,s:&State){let result=(||{if let Some(b)=s.backend.borrow().as_ref(){let mut account=[0u16;28];let n=GetWindowTextW(s.controls.get()[LOGIN_ACCOUNT],&mut account)as usize;let expected:Vec<u16>="Fx485Guest".encode_utf16().collect();ensure!(account[..n]==expected,"accepted Guest account required");b.login_account(b"Fx485Guest")?;}Ok(())})().and_then(|_|login_edit(s))''')
rep('let result=s.runtime.borrow_mut().as_mut().unwrap().handle_player_entry_command(EntryCommand::OpenLogin);','let result=s.backend.borrow_mut().as_mut().map_or(Ok(()),|b|b.begin()).and_then(|_|s.runtime.borrow_mut().as_mut().unwrap().handle_player_entry_command(EntryCommand::OpenLogin));')
rep('WM_TIMER if wparam.0==TIMER=>{let(changed,closed)', 'WM_TIMER if wparam.0==TIMER=>{if let(Some(b),Some(r))=(s.backend.borrow_mut().as_mut(),s.runtime.borrow_mut().as_mut()){if b.poll_health(r){*s.refresh_gate.borrow_mut()=Gate::default();}}let(changed,closed)')
s=s.replace('run_inner(inputs,skin_inputs,start_at_maps,None,None,None)','run_inner(inputs,skin_inputs,start_at_maps,None,None,None,None)').replace('run_inner(inputs,skin_inputs,false,Some(profile),None,None)','run_inner(inputs,skin_inputs,false,Some(profile),None,None,None)').replace('run_inner(inputs,skin_inputs,false,Some(profile),Some(host),None)','run_inner(inputs,skin_inputs,false,Some(profile),Some(host),None,None)').replace('run_inner(inputs,skin_inputs,false,Some(profile),None,Some(adapter))','run_inner(inputs,skin_inputs,false,Some(profile),None,Some(adapter),None)')
rep('fn run_inner(inputs:Inputs', 'pub fn run_backend_485f(inputs:Inputs,skin_inputs:Inputs,accepted:super::native_backend_485f_entry::Accepted)->Result<Outcome>{run_inner(inputs,skin_inputs,false,None,None,None,Some(accepted))}\nfn run_inner(inputs:Inputs')
rep('JavaFixtureAdapter>)->Result<Outcome>{unsafe{','JavaFixtureAdapter>,backend:Option<super::native_backend_485f_entry::Accepted>)->Result<Outcome>{unsafe{')
rep('State{room_panel:', 'State{backend:RefCell::new(backend),room_panel:')
rep('"LiTo Modern · 玩家入口候選 R529"','"LiTo Modern · 玩家入口候選 R530"')
rep('*state.runtime.borrow_mut()=Some(runtime);','if let Some(b)=state.backend.borrow().as_ref(){b.configure(&mut runtime)?;}*state.runtime.borrow_mut()=Some(runtime);')
rep('ensure!(!state.fault.get(),"candidate offline UI state error");','if let(Some(b),Some(r))=(state.backend.borrow().as_ref(),state.runtime.borrow().as_ref()){b.finish(r)?;}ensure!(!state.fault.get(),"candidate offline UI state error");')
w.write_text(s,'utf-8',newline='\n')
p=P/'src/modern_lito/native_java_room_player.rs';s=p.read_text('utf-8');s=s.replace('last_D1','last_d1').replace('received_BYTE','received_byte').replace('and_Close','and_close');p.write_text(s,'utf-8',newline='\n')
s=(P/'target/r529-player-entry-candidate/LiToModern-R529.rs').read_text('utf-8').replace('R529_','R530_').replace('"r529"','"r530"')
s=s.replace('    CandidateJavaBootstrap(PathBuf,PathBuf),','    CandidateJavaBootstrap(PathBuf,PathBuf),\n    CandidateBackend485f(PathBuf,String),')
s=s.replace('        None => Ok(Route::Pick),','        None => Ok(Route::Pick),\n        Some(x) if x==OsStr::new("--candidate-backend-485f")&&args.len()==4=>{let p=PathBuf::from(&args[2]);let sha=args[3].to_str().ok_or_else(||anyhow::anyhow!("ASCII SHA required"))?;ensure!(p.is_absolute()&&sha.len()==64&&sha.bytes().all(|b|b.is_ascii_hexdigit()),"absolute fresh READY plus trusted SHA256 required");Ok(Route::CandidateBackend485f(p,sha.to_ascii_uppercase()))},')
s=s.replace('        Route::CandidateJavaBootstrap(path,adapter_path)', '        Route::CandidateBackend485f(path,sha)=>{let accepted=modern_lito::native_backend_485f_entry::Accepted::load(&path,&sha)?;candidate_backend_entry(accepted)?},\n        Route::CandidateJavaBootstrap(path,adapter_path)')
start=s.index('fn candidate_java_entry(');end=s.index('\nfn ',start+4);fn=s[start:end];fn=fn.replace('fn candidate_java_entry(profile: modern_lito::native_city_runtime::login_events::FixtureProfile,adapter:modern_lito::native_city_runtime::login_bootstrap::JavaFixtureAdapter)', 'fn candidate_backend_entry(accepted:modern_lito::native_backend_485f_entry::Accepted)').replace('run_synthetic_java_bootstrap(inputs,skin_inputs,profile,adapter)','run_backend_485f(inputs,skin_inputs,accepted)');s=s[:end]+'\n'+fn+s[end:]
s+='''\n#[cfg(test)]mod r530_backend_route {
 use super::*;
 #[test]fn explicit_ready_route_has_no_synthetic_profile_escape(){
  let args=|a:&[&str]|a.iter().map(OsString::from).collect::<Vec<_>>();
  assert!(route(&args(&["candidate","--candidate-backend-485f","relative","bad"])).is_err());
  let sha="A".repeat(64);assert!(matches!(route(&args(&["candidate","--candidate-backend-485f","D:/new/ready-contract.json",&sha])).unwrap(),Route::CandidateBackend485f(_,s)if s==sha));
  assert!(route(&args(&["candidate","--candidate-backend-485f"])).is_err());
 }
}
'''
write(T/'LiToModern-R530.rs',s)
print('R530 entry/provider/status reader/window/main prepared; no provider/PID/socket read executed')
