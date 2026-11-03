from pathlib import Path
p=Path('src/modern_lito/native_city_window.rs');s=p.read_text(encoding='utf-8')
def edit(a,b):
 global s
 assert a in s,a;s=s.replace(a,b)
edit('const COUNT:usize=33;', 'const SOURCE_LIST:usize=33;const SOURCE_SUBMIT:usize=34;const NEWS_CONTINUE:usize=35;const COUNT:usize=36;')
edit('struct State {channel_back:', 'struct State {source_controls:RefCell<Vec<Option<super::native_city_runtime::login_bootstrap::SourceControl>>>,source_list_revision:Cell<u64>,news_control:RefCell<Option<super::native_city_runtime::login_bootstrap::NewsControl>>,channel_back:')
edit('let entry=s.screen.get()==Screen::Entry;', '''let bootstrap=s.runtime.borrow().as_ref().unwrap().source_bootstrap_snapshot();
 let source_choice=bootstrap.stage==super::native_city_runtime::login_bootstrap::Stage::SourceChoice;
 let news_choice=bootstrap.stage==super::native_city_runtime::login_bootstrap::Stage::NewsChoice;
 let bootstrap_busy=!matches!(bootstrap.stage,super::native_city_runtime::login_bootstrap::Stage::Idle|super::native_city_runtime::login_bootstrap::Stage::Roster);
 *s.source_controls.borrow_mut()=(0..bootstrap.sources.len()).map(|slot|s.runtime.borrow().as_ref().unwrap().source_choice_control(slot)).collect();
 *s.news_control.borrow_mut()=s.runtime.borrow().as_ref().and_then(Runtime::source_news_control);
 if source_choice&&s.source_list_revision.get()!=bootstrap.revision {
  SendMessageW(s.controls.get()[SOURCE_LIST],CB_RESETCONTENT,None,None);
  for(slot,row)in bootstrap.sources.iter().enumerate(){let name=if row.name.is_ascii(){String::from_utf8_lossy(&row.name).into_owned()}else{format!("來源 {}",slot+1)};let text:Vec<u16>=name.encode_utf16().chain(Some(0)).collect();SendMessageW(s.controls.get()[SOURCE_LIST],CB_ADDSTRING,None,Some(LPARAM(text.as_ptr() as isize)));}
  SendMessageW(s.controls.get()[SOURCE_LIST],CB_SETCURSEL,Some(WPARAM(0)),None);s.source_list_revision.set(bootstrap.revision);
 }
 let entry=s.screen.get()==Screen::Entry;''')
edit('let visible=match i{CHANNEL_FIRST', 'let visible=match i{SOURCE_LIST|SOURCE_SUBMIT=>entry&&source_choice,NEWS_CONTINUE=>entry&&news_choice,CHANNEL_FIRST')
edit('let enabled=!closing&&match i{CHANNEL_FIRST', 'let enabled=!closing&&match i{SOURCE_LIST|SOURCE_SUBMIT=>source_choice&&s.source_controls.borrow().iter().any(Option::is_some),NEWS_CONTINUE=>s.news_control.borrow().is_some(),CHANNEL_FIRST')
edit('NETWORK=>entry&&!login_active&&(!navigation_active||restart)', 'NETWORK=>entry&&!bootstrap_busy&&!login_active&&(!navigation_active||restart)')
edit('OFFLINE=>entry&&!login_active&&!navigation_active', 'OFFLINE=>entry&&!bootstrap_busy&&!login_active&&!navigation_active')
edit(' for slot in 0..5{let rows=', ' if source_choice{label(c[TITLE],"LiTo Modern · 選擇登入來源");label(c[INFO],"請選擇伺服器提供的登入來源。");}else if news_choice{label(c[INFO],"按繼續取得角色名單。");}\n for slot in 0..5{let rows=')
edit('let _=MoveWindow(c[SOURCE_ENTRY],', 'let _=MoveWindow(c[SOURCE_LIST],m,scale(120,dpi),scale(350,dpi),scale(260,dpi),true);let _=MoveWindow(c[SOURCE_SUBMIT],m+scale(370,dpi),scale(120,dpi),scale(140,dpi),scale(32,dpi),true);let _=MoveWindow(c[NEWS_CONTINUE],m,scale(120,dpi),scale(140,dpi),scale(32,dpi),true);let _=MoveWindow(c[SOURCE_ENTRY],')
edit('Some(NETWORK)=>{', '''Some(SOURCE_SUBMIT)=>{let slot=SendMessageW(s.controls.get()[SOURCE_LIST],CB_GETCURSEL,None,None).0;let token=usize::try_from(slot).ok().and_then(|i|s.source_controls.borrow().get(i).cloned().flatten());let result=token.as_ref().ok_or_else(||anyhow::anyhow!("請選擇有效來源")).and_then(|c|s.runtime.borrow_mut().as_mut().unwrap().handle_player_entry_command(EntryCommand::ChooseSource(c)));if let Err(e)=result{label(s.controls.get()[STATUS],&e.to_string());}else{refresh(hwnd,s);}},Some(NEWS_CONTINUE)=>{let token=s.news_control.borrow().clone();let result=token.as_ref().ok_or_else(||anyhow::anyhow!("繼續目前尚未可用")).and_then(|c|s.runtime.borrow_mut().as_mut().unwrap().handle_player_entry_command(EntryCommand::ContinueNews(c)));if let Err(e)=result{label(s.controls.get()[STATUS],&e.to_string());}else{refresh(hwnd,s);}},Some(NETWORK)=>{''')
edit('State{channel_back:', 'State{source_controls:RefCell::new(Vec::new()),source_list_revision:Cell::new(0),news_control:RefCell::new(None),channel_back:')
edit('"伺服器 4","伺服器 5"]', '"伺服器 4","伺服器 5","","選擇來源","繼續"]')
edit('if i==MAP{', 'if i==MAP||i==SOURCE_LIST{')
edit('run_inner(inputs,skin_inputs,start_at_maps,None,None)', 'run_inner(inputs,skin_inputs,start_at_maps,None,None,None)')
edit('run_inner(inputs,skin_inputs,false,Some(profile),None)', 'run_inner(inputs,skin_inputs,false,Some(profile),None,None)')
edit('run_inner(inputs,skin_inputs,false,Some(profile),Some(host))', 'run_inner(inputs,skin_inputs,false,Some(profile),Some(host),None)')
edit('fn run_inner(inputs:', 'pub fn run_synthetic_java_bootstrap(inputs:Inputs,skin_inputs:Inputs,profile:FixtureProfile,adapter:super::native_city_runtime::login_bootstrap::JavaFixtureAdapter)->Result<Outcome>{run_inner(inputs,skin_inputs,false,Some(profile),None,Some(adapter))}\nfn run_inner(inputs:')
edit('host:Option<super::native_city_runtime::startup_host::FixtureStartupHost>)->Result<Outcome>', 'host:Option<super::native_city_runtime::startup_host::FixtureStartupHost>,adapter:Option<super::native_city_runtime::login_bootstrap::JavaFixtureAdapter>)->Result<Outcome>')
edit('if let Some(host)=host{runtime.configure_source_startup_host_fixture(host)?;}', 'if let Some(host)=host{runtime.configure_source_startup_host_fixture(host)?;}if let Some(adapter)=adapter{runtime.configure_login_producer_fixture(adapter)?;}')
p.write_text(s,encoding='utf-8')
