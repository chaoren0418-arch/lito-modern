from pathlib import Path
p=Path('src/modern_lito/native_city_window.rs');s=p.read_text(encoding='utf-8')
def edit(a,b):
 global s
 assert a in s,a;s=s.replace(a,b)
edit('const COUNT:usize=36;', 'const CHARACTER_DIRECTORY:usize=36;const CHARACTER_DETAILS:usize=37;const COUNT:usize=38;')
edit('struct State {source_controls:', 'struct State {directory_controls:RefCell<Vec<Option<SourceCharacterControl>>>,directory_revision:Cell<u64>,source_controls:')
edit('let entry=s.screen.get()==Screen::Entry;', '''let directory=s.runtime.borrow().as_ref().unwrap().java_character_snapshot().stage==super::native_city_runtime::java_character_flow::Stage::Directory&&roster;
 if directory {
  let revision=s.runtime.borrow().as_ref().unwrap().presentation_key().roster_revision;
  let rows=s.runtime.borrow().as_ref().unwrap().source_identity().records().to_vec();
  *s.directory_controls.borrow_mut()=(0..rows.len()).map(|slot|s.runtime.borrow().as_ref().unwrap().source_character_control(slot)).collect();
  if s.directory_revision.get()!=revision {SendMessageW(s.controls.get()[CHARACTER_DIRECTORY],CB_RESETCONTENT,None,None);for row in &rows{let text=row.display_name().unwrap_or_else(|_|"名稱無法顯示".into());let text:Vec<u16>=text.encode_utf16().chain(Some(0)).collect();SendMessageW(s.controls.get()[CHARACTER_DIRECTORY],CB_ADDSTRING,None,Some(LPARAM(text.as_ptr() as isize)));}SendMessageW(s.controls.get()[CHARACTER_DIRECTORY],CB_SETCURSEL,Some(WPARAM(usize::MAX)),None);s.directory_revision.set(revision);}
  let selected=s.runtime.borrow().as_ref().unwrap().presentation_key().character_ui.selected;
  let text=selected.and_then(|slot|rows.get(slot)).map(|r|{let f=r.fields();format!("{} · 等級 {} · HP {} · MP {} · AC {} · 正義值 {}\\n排名 {} · 變動 {}",r.display_name().unwrap_or_default(),f.level,f.max_hp,f.max_mp,f.armor_class,f.lawful,f.ranks.overall,f.movements.overall)}).unwrap_or_else(||"請選擇角色，再按進入角色。".into());label(s.controls.get()[CHARACTER_DETAILS],&text);
 }
 let entry=s.screen.get()==Screen::Entry;''')
edit('let visible=match i{SOURCE_LIST', 'let visible=match i{CHARACTER_DIRECTORY|CHARACTER_DETAILS=>entry&&directory,SOURCE_LIST')
edit('CHARACTER_FIRST..=16=>entry&&roster', 'CHARACTER_FIRST..=16=>entry&&roster&&!directory')
edit('let enabled=!closing&&match i{SOURCE_LIST', 'let enabled=!closing&&match i{CHARACTER_DIRECTORY=>directory&&s.directory_controls.borrow().iter().any(Option::is_some),SOURCE_LIST')
edit('let _=MoveWindow(c[SOURCE_LIST],', 'let _=MoveWindow(c[CHARACTER_DIRECTORY],m,scale(120,dpi),scale(400,dpi),scale(280,dpi),true);let _=MoveWindow(c[CHARACTER_DETAILS],m,scale(164,dpi),width,scale(90,dpi),true);let _=MoveWindow(c[SOURCE_LIST],')
edit('WM_COMMAND if (wparam.0&0xffff)==FIRST+MAP', '''WM_COMMAND if (wparam.0&0xffff)==FIRST+CHARACTER_DIRECTORY&&(wparam.0>>16)&0xffff==CBN_SELCHANGE as usize=>{let index=SendMessageW(s.controls.get()[CHARACTER_DIRECTORY],CB_GETCURSEL,None,None).0;let token=usize::try_from(index).ok().and_then(|slot|s.directory_controls.borrow().get(slot).cloned().flatten());let result=token.as_ref().ok_or_else(||anyhow::anyhow!("角色控制已失效")).and_then(|control|s.runtime.borrow_mut().as_mut().unwrap().handle_player_entry_command(EntryCommand::ChooseCharacter{control,double_click:false}));if let Err(e)=result{label(s.controls.get()[STATUS],&e.to_string());}else{refresh(hwnd,s);}LRESULT(0)},
  WM_COMMAND if (wparam.0&0xffff)==FIRST+MAP''')
edit('State{source_controls:', 'State{directory_controls:RefCell::new(Vec::new()),directory_revision:Cell::new(0),source_controls:')
edit('"選擇來源","繼續"]','"選擇來源","繼續","","請選擇角色"]')
edit('if i==MAP||i==SOURCE_LIST{', 'if i==MAP||i==SOURCE_LIST||i==CHARACTER_DIRECTORY{')
edit('matches!(i,TITLE|INFO|STATUS|LOGIN_ACCOUNT_LABEL|LOGIN_PASSWORD_LABEL)', 'matches!(i,TITLE|INFO|STATUS|LOGIN_ACCOUNT_LABEL|LOGIN_PASSWORD_LABEL|CHARACTER_DETAILS)')
p.write_text(s,encoding='utf-8')
