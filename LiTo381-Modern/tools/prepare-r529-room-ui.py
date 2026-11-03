import pathlib,json,hashlib,shutil
P=pathlib.Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';T=P/'target/r529-player-entry-candidate';B=L/'r529-source-before'
def pin(p):
 p=pathlib.Path(p);raw=p.read_bytes();return dict(path=str(p),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest().upper())
def save(p,v):
 with pathlib.Path(p).open('x',encoding='utf-8',newline='\n')as f:json.dump(v,f,ensure_ascii=False,indent=2);f.write('\n')
T.mkdir();B.mkdir()
old=json.loads((L/'r527-production-source-snapshot.json').read_text('utf-8-sig'))
for name in ['native_city_runtime.rs','native_city_window.rs']:
 p=P/'src/modern_lito'/name;previous=next(v for v in old['source_pins']if pathlib.Path(v['path'])==p);assert pin(p)['sha256']==previous['sha256'];shutil.copyfile(p,B/name)
r=P/'src/modern_lito/native_city_runtime.rs';s=r.read_text('utf-8');a='#[path="native_java_room_ready.rs"]pub mod java_room_ready;';assert s.count(a)==1;s=s.replace(a,a+'\n#[path="native_java_room_player.rs"]pub mod room_player;');r.write_text(s,'utf-8',newline='\n')
w=P/'src/modern_lito/native_city_window.rs';s=w.read_text('utf-8')
def replace(a,b):
 global s
 assert s.count(a)==1,(a,s.count(a));s=s.replace(a,b)
replace('login_events::FixtureProfile};','login_events::FixtureProfile,room_player::{self,Panel as RoomPanel,Operation as RoomOperation}};')
replace('const COUNT:usize=38;','const ROOM_LIST:usize=38;const ROOM_JOIN:usize=39;const ROOM_READY:usize=40;const ROOM_CANCEL:usize=41;const ROOM_DETAILS:usize=42;const ROOM_COUNTDOWN:usize=43;const ROOM_REFRESH:usize=44;const COUNT:usize=45;')
replace('struct State {','struct State {room_panel:RefCell<RoomPanel>,room_list_key:Cell<Option<(u64,u8,u64,u64)>>,')
replace(' let entry=s.screen.get()==Screen::Entry;', ''' let room_panel={let r=s.runtime.borrow();RoomPanel::capture(r.as_ref().unwrap())};
 let in_room=room_panel.display.room_id.is_some();
 let room_catalog=!in_room&&room_panel.catalog_key.is_some()&&lobby;
 let selected=SendMessageW(s.controls.get()[ROOM_LIST],CB_GETCURSEL,None,None).0;
 let selected_id=usize::try_from(selected).ok().and_then(|i|s.room_panel.borrow().rooms.get(i).map(|r|r.room_id));
 if s.room_list_key.replace(room_panel.catalog_key)!=room_panel.catalog_key {
  SendMessageW(s.controls.get()[ROOM_LIST],CB_RESETCONTENT,None,None);
  for row in &room_panel.rooms {let t:Vec<u16>=row.label.encode_utf16().chain(Some(0)).collect();SendMessageW(s.controls.get()[ROOM_LIST],CB_ADDSTRING,None,Some(LPARAM(t.as_ptr() as isize)));}
  let index=selected_id.and_then(|id|room_panel.rooms.iter().position(|row|row.room_id==id)).unwrap_or(usize::MAX);
  SendMessageW(s.controls.get()[ROOM_LIST],CB_SETCURSEL,Some(WPARAM(index)),None);
 }
 let room_details=if in_room{room_panel.member_text()}else{String::new()};label(s.controls.get()[ROOM_DETAILS],&room_details);
 label(s.controls.get()[ROOM_COUNTDOWN],&room_panel.countdown_text());
 *s.room_panel.borrow_mut()=room_panel;
 let entry=s.screen.get()==Screen::Entry;''')
replace('label(c[TITLE],if entry&&channel', 'label(c[TITLE],if entry&&in_room{"LiTo Modern · 房間"}else if entry&&channel')
replace('label(c[INFO],if entry&&restart', 'label(c[INFO],if entry&&in_room{"可按準備或取消準備；狀態依伺服器回包更新。"}else if entry&&room_catalog{"選擇已收到的房間，再按加入房間。"}else if entry&&restart')
replace('SERVER_FIRST..=32=>entry&&lobby&&', 'SERVER_FIRST..=32=>entry&&lobby&&!in_room&&')
replace('let visible=match i{', 'let visible=match i{ROOM_LIST|ROOM_JOIN=>entry&&room_catalog,ROOM_REFRESH=>entry&&s.room_panel.borrow().enabled(RoomOperation::Refresh),ROOM_READY|ROOM_CANCEL|ROOM_DETAILS|ROOM_COUNTDOWN=>entry&&in_room,')
replace('match i{CHARACTER_DIRECTORY=>directory', 'match i{ROOM_LIST=>room_catalog,ROOM_JOIN=>usize::try_from(SendMessageW(c[ROOM_LIST],CB_GETCURSEL,None,None).0).ok().is_some_and(|slot|s.room_panel.borrow().enabled(RoomOperation::Join(slot))),ROOM_READY=>s.room_panel.borrow().enabled(RoomOperation::Ready),ROOM_CANCEL=>s.room_panel.borrow().enabled(RoomOperation::Cancel),ROOM_REFRESH=>s.room_panel.borrow().enabled(RoomOperation::Refresh),CHARACTER_DIRECTORY=>directory')
replace(' for slot in 0..4{let text=', ' for slot in 0..4{let text=') # Assert the existing branch without changing it.
replace('unsafe fn resize(hwnd:HWND,s:&State)', '''unsafe fn room_operation(hwnd:HWND,s:&State,operation:RoomOperation){
 let panel=s.room_panel.borrow().clone();
 let result=panel.invoke(s.runtime.borrow_mut().as_mut().unwrap(),operation);
 if let Err(e)=result{label(s.controls.get()[STATUS],&e.to_string());}
 refresh(hwnd,s);
}
unsafe fn resize(hwnd:HWND,s:&State)''')
replace(' for(i,(x,y,w,h))in boxes.into_iter().enumerate()', ''' for(i,x,y,w,h)in [(ROOM_LIST,m,370,400,185),(ROOM_JOIN,m+scale(416,dpi),370,140,32),(ROOM_REFRESH,m+scale(416,dpi),412,140,32),(ROOM_DETAILS,m,130,650,260),(ROOM_READY,m,420,150,36),(ROOM_CANCEL,m+scale(166,dpi),420,150,36),(ROOM_COUNTDOWN,m,472,400,32)]{let _=MoveWindow(c[i],x,scale(y,dpi),scale(w,dpi),scale(h,dpi),true);}
 for(i,(x,y,w,h))in boxes.into_iter().enumerate()''')
replace('  WM_COMMAND if (FIRST+CHANNEL_FIRST', '''  WM_COMMAND if (wparam.0&0xffff)==FIRST+ROOM_LIST&&((wparam.0>>16)&0xffff)==CBN_SELCHANGE as usize=>{
   let slot=usize::try_from(SendMessageW(s.controls.get()[ROOM_LIST],CB_GETCURSEL,None,None).0).ok();
   let enabled=s.runtime.borrow().as_ref().is_some_and(|r|r.page()==UiPage::Menu)&&slot.is_some_and(|slot|s.room_panel.borrow().enabled(RoomOperation::Join(slot)));
   let _=EnableWindow(s.controls.get()[ROOM_JOIN],enabled);LRESULT(0)
  },
  WM_COMMAND if (wparam.0>>16)&0xffff==BN_CLICKED as usize&&matches!((wparam.0&0xffff).checked_sub(FIRST),Some(ROOM_READY|ROOM_CANCEL|ROOM_JOIN|ROOM_REFRESH))=>{
   let operation=match (wparam.0&0xffff)-FIRST{ROOM_READY=>Some(RoomOperation::Ready),ROOM_CANCEL=>Some(RoomOperation::Cancel),ROOM_REFRESH=>Some(RoomOperation::Refresh),_=>usize::try_from(SendMessageW(s.controls.get()[ROOM_LIST],CB_GETCURSEL,None,None).0).ok().map(RoomOperation::Join)};
   if let Some(operation)=operation{room_operation(hwnd,s,operation);}LRESULT(0)
  },
  WM_COMMAND if (FIRST+CHANNEL_FIRST''')
replace('State{directory_controls:', 'State{room_panel:RefCell::new(RoomPanel::default()),room_list_key:Cell::new(None),directory_controls:')
replace('"LiTo Modern · 玩家入口候選 R483"','"LiTo Modern · 玩家入口候選 R529"')
replace('"","請選擇角色"];', '"","請選擇角色","","加入房間","準備","取消準備","","","重新取得清單"];')
replace('if i==MAP||i==SOURCE_LIST||i==CHARACTER_DIRECTORY','if i==MAP||i==SOURCE_LIST||i==CHARACTER_DIRECTORY||i==ROOM_LIST')
replace('LOGIN_PASSWORD_LABEL|CHARACTER_DETAILS)', 'LOGIN_PASSWORD_LABEL|CHARACTER_DETAILS|ROOM_DETAILS|ROOM_COUNTDOWN)')
replace('runtime.configure_login_producer_fixture(adapter)?;}', 'runtime.configure_login_producer_fixture(adapter)?;room_player::configure(&mut runtime)?;}')
w.write_text(s,'utf-8',newline='\n')
entry=json.loads((L/'r526-entry-source-contract.json').read_text('utf-8-sig'))
main=pathlib.Path(entry['candidate_main']['path']);assert pin(main)==entry['candidate_main']
c=main.read_text('utf-8').replace('R526_','R529_').replace('"candidate_revision": "r526"','"candidate_revision": "r529"')
start=c.index('fn candidate_java_entry(');c=c[:start]+c[start:].replace('native_city_window::{self, Outcome}','native_city_window',1)
candidate=T/'LiToModern-R529.rs';candidate.write_text(c,'utf-8',newline='\n')
save(L/'r529-entry-source-contract.json',dict(state='R529_EXPLICIT_JAVA_BOOTSTRAP_ROOM_CALLBACK_CANDIDATE_GUI_UNVERIFIED',original_main=pin(P/'src/main.rs'),previous_candidate_main=entry['candidate_main'],candidate_main=pin(candidate),binding=entry['binding'],skin_binding=entry['skin_binding'],default_route_unchanged=True,flag='--candidate-java-bootstrap <absolute synthetic loopback fixture profile> <absolute pinned AD368 layout adapter>',profile_scope='Legacy explicit SYNTHETIC profile only; not supervisor READY consumption or qualified real-backend GUI entry.',room_callbacks='public observed room Join; private typed Ready1/NotReady0; observed D1 status, E1 count only; no local timer/WORLD',production_source_changes=['src/modern_lito/native_city_runtime.rs','src/modern_lito/native_city_window.rs','src/modern_lito/native_java_room_player.rs'],new_assets=0,GUI=0))
print('R529 actual candidate window/room callbacks and separate entry prepared; src main/frozen evidence untouched')
