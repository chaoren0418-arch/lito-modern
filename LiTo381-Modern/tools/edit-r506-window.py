from r491_common import *
p=P/'src/modern_lito/native_city_window.rs';s=p.read_text(encoding='utf-8')
s=s.replace('SourceLobbyBack,entry_navigation','SourceLobbyBack,SourceChannelControl,SourceTransferServerControl,entry_navigation')
s=s.replace('const COUNT:usize=24;','const CHANNEL_FIRST:usize=24;const SERVER_FIRST:usize=28;const COUNT:usize=33;')
s=s.replace('struct State {lobby_back:', 'struct State {channel_controls:RefCell<[Option<SourceChannelControl>;4]>,server_controls:RefCell<[Option<SourceTransferServerControl>;5]>,lobby_back:')
s=s.replace('let lobby=navigation.scene==NavigationScene::Lobby;', 'let lobby=navigation.scene==NavigationScene::Lobby;let channel=navigation.scene==NavigationScene::Channel;')
mark=' *s.character_back.borrow_mut()='
a=s.index(mark);s=s[:a]+''' *s.channel_controls.borrow_mut()=std::array::from_fn(|slot|s.runtime.borrow().as_ref().and_then(|r|r.source_channel_control(slot)));
 *s.server_controls.borrow_mut()=std::array::from_fn(|slot|s.runtime.borrow().as_ref().and_then(|r|r.source_transfer_server_control(slot)));
'''+s[a:]
s=s.replace('if entry&&lobby{"LiTo Modern · 大廳"}', 'if entry&&channel{"LiTo Modern · 選擇頻道"}else if entry&&lobby{"LiTo Modern · 大廳"}')
s=s.replace('if lobby{"返回頻道（尚未開放）"}else{"返回"}', 'if lobby&&s.lobby_back.borrow().is_some(){"返回"}else if lobby||channel{"返回尚未可用"}else{"返回"}')
s=s.replace('else if entry&&lobby{"已收到大廳回包。大廳功能與返回頻道尚未開放。"}', 'else if entry&&channel{"請選擇頻道。切換期間請稍候。"}else if entry&&lobby{"已收到大廳回包。可選取已提供的伺服器項目。"}')
s=s.replace('let visible=match i{LOGIN_', 'let visible=match i{CHANNEL_FIRST..=27=>entry&&channel,SERVER_FIRST..=32=>entry&&lobby&&s.runtime.borrow().as_ref().is_some_and(|r|r.source_transfer_snapshot().server_rows.len()>i-SERVER_FIRST),LOGIN_')
s=s.replace('let enabled=!closing&&match i{LOGIN_', 'let enabled=!closing&&match i{CHANNEL_FIRST..=27=>s.channel_controls.borrow()[i-CHANNEL_FIRST].is_some(),SERVER_FIRST..=32=>s.server_controls.borrow()[i-SERVER_FIRST].is_some(),LOGIN_')
s=s.replace('BACK=>!navigation_active&&((login_active', 'BACK=>s.lobby_back.borrow().is_some()||!navigation_active&&((login_active')
mark=' resize(hwnd,s);let _=InvalidateRect'
a=s.index(mark);s=s[:a]+''' for slot in 0..5{let rows=s.runtime.borrow().as_ref().unwrap().source_transfer_snapshot().server_rows;let name=rows.get(slot).filter(|r|r.name.is_ascii()).map(|r|String::from_utf8_lossy(&r.name).into_owned()).unwrap_or_else(||format!("伺服器 {}",slot+1));label(c[SERVER_FIRST+slot],&name);}
'''+s[a:]
mark='let _=InvalidateRect(Some(hwnd),None,false);'
a=s.index(mark,s.index('unsafe fn resize'));s=s[:a]+'''if let Some(v)=channel_layout(hwnd){for(slot,y)in [53,162,269,381].into_iter().enumerate(){let x=v.x+286*v.width/640;let y=v.y+y*v.height/480;let _=MoveWindow(c[CHANNEL_FIRST+slot],x,y,72*v.width/640,69*v.height/480,true);}}for slot in 0..5{let _=MoveWindow(c[SERVER_FIRST+slot],m,scale(120+slot as i32*44,dpi),width.min(scale(400,dpi)),scale(36,dpi),true);}
'''+s[a:]
#Keep the unqualified source-host Back command explicit and unavailable.
s=s.replace('if s.screen.get()==Screen::Entry&&s.runtime.borrow().as_ref().is_some_and(|r|r.source_navigation_snapshot().scene!=NavigationScene::Dormant){', 'if s.screen.get()==Screen::Entry&&s.runtime.borrow().as_ref().is_some_and(|r|r.source_navigation_snapshot().scene==NavigationScene::Channel){let result=s.runtime.borrow_mut().as_mut().unwrap().handle_player_entry_command(EntryCommand::BackChannel);if let Err(e)=result{label(s.controls.get()[STATUS],&e.to_string());}return;}if s.screen.get()==Screen::Entry&&s.runtime.borrow().as_ref().is_some_and(|r|r.source_navigation_snapshot().scene!=NavigationScene::Dormant){')
mark='unsafe fn paint_source_entry('
a=s.index(mark);s=s[:a]+'''unsafe fn channel_layout(hwnd:HWND)->Option<super::compositor::Viewport>{let mut r=RECT::default();GetClientRect(hwnd,&mut r).ok()?;let dpi=GetDpiForWindow(hwnd);let top=scale(104,dpi);let m=scale(24,dpi);let mut v=super::compositor::Viewport::fit((r.right-2*m).max(0),(r.bottom-top-scale(114,dpi)).max(0),640,480)?;v.x+=m;v.y+=top;Some(v)}
unsafe fn paint_channel(hwnd:HWND,s:&State,dc:windows::Win32::Graphics::Gdi::HDC){if s.screen.get()!=Screen::Entry{return;}let r=s.runtime.borrow();let Some(r)=r.as_ref()else{return;};if r.source_navigation_snapshot().scene!=NavigationScene::Channel{return;}let(Some(skin),Some(v))=(r.skin(),channel_layout(hwnd))else{return;};let Some(img)=&skin.channel_background else{return;};let b:Vec<u8>=img.rgba.chunks_exact(4).flat_map(|p|[p[2],p[1],p[0],p[3]]).collect();let info=BITMAPINFO{bmiHeader:BITMAPINFOHEADER{biSize:std::mem::size_of::<BITMAPINFOHEADER>()as u32,biWidth:640,biHeight:-480,biPlanes:1,biBitCount:32,biCompression:BI_RGB.0,..Default::default()},..Default::default()};StretchDIBits(dc,v.x,v.y,v.width,v.height,0,0,640,480,Some(b.as_ptr().cast()),&info,DIB_RGB_COLORS,SRCCOPY);}
'''+s[a:]
s=s.replace('paint_menu(hwnd,s,dc);paint_source_entry', 'paint_menu(hwnd,s,dc);paint_channel(hwnd,s,dc);paint_source_entry')
s=s.replace('let submit=draw.CtlID as usize==FIRST+CHARACTER_SUBMIT;', 'let submit=draw.CtlID as usize==FIRST+CHARACTER_SUBMIT;let channel_slot=(draw.CtlID as usize).checked_sub(FIRST+CHANNEL_FIRST).filter(|i|*i<4);')
old='let index=if submit{0}else{2}+if draw.itemState.0&ODS_SELECTED.0!=0{1}else{0};skin.character_buttons.get(index).map'
new='let pressed=usize::from(draw.itemState.0&ODS_SELECTED.0!=0);let image=if let Some(slot)=channel_slot{skin.channel_buttons.get(slot*2+pressed)}else{skin.character_buttons.get(if submit{0}else{2}+pressed)};image.map'
assert old in s;s=s.replace(old,new)
s=s.replace('r.presentation_key().character_ui.active){if let Some((w,h,rgba))=image', '(r.presentation_key().character_ui.active||channel_slot.is_some()&&r.source_navigation_snapshot().scene==NavigationScene::Channel)){if let Some((w,h,rgba))=image')
s=s.replace('if draw.CtlID as usize==FIRST+CHARACTER_SUBMIT||draw.CtlID as usize==FIRST+BACK{', 'if draw.CtlID as usize==FIRST+CHARACTER_SUBMIT||draw.CtlID as usize==FIRST+BACK||(FIRST+CHANNEL_FIRST..FIRST+CHANNEL_FIRST+4).contains(&(draw.CtlID as usize)){')
mark='  WM_COMMAND if (usize::from(SOURCE_CHARACTER_COMMAND_FIRST)'
a=s.index(mark);s=s[:a]+'''  WM_COMMAND if (FIRST+CHANNEL_FIRST..FIRST+CHANNEL_FIRST+4).contains(&(wparam.0&0xffff))&&((wparam.0>>16)&0xffff)==BN_CLICKED as usize=>{let slot=(wparam.0&0xffff)-FIRST-CHANNEL_FIRST;let token=s.channel_controls.borrow()[slot].clone();let result=token.as_ref().ok_or_else(||anyhow::anyhow!("頻道目前尚未可操作")).and_then(|c|s.runtime.borrow_mut().as_mut().unwrap().handle_player_entry_command(EntryCommand::ChooseChannel(c)));if let Err(e)=result{label(s.controls.get()[STATUS],&e.to_string());}else{refresh(hwnd,s);}LRESULT(0)},
  WM_COMMAND if (FIRST+SERVER_FIRST..FIRST+SERVER_FIRST+5).contains(&(wparam.0&0xffff))&&((wparam.0>>16)&0xffff)==BN_CLICKED as usize=>{let slot=(wparam.0&0xffff)-FIRST-SERVER_FIRST;let token=s.server_controls.borrow()[slot].clone();let result=token.as_ref().ok_or_else(||anyhow::anyhow!("伺服器項目目前尚未可操作")).and_then(|c|s.runtime.borrow_mut().as_mut().unwrap().handle_player_entry_command(EntryCommand::ChooseTransferServer(c)));if let Err(e)=result{label(s.controls.get()[STATUS],&e.to_string());}else{refresh(hwnd,s);}LRESULT(0)},
'''+s[a:]
s=s.replace('State{lobby_back:','State{channel_controls:RefCell::new(std::array::from_fn(|_|None)),server_controls:RefCell::new(std::array::from_fn(|_|None)),lobby_back:')
s=s.replace('"","登入"];','"","登入","頻道 1","頻道 2","頻道 3","頻道 4","伺服器 1","伺服器 2","伺服器 3","伺服器 4","伺服器 5"];')
s=s.replace('i==CHARACTER_SUBMIT||i==BACK{BS_OWNERDRAW', 'i==CHARACTER_SUBMIT||i==BACK||(CHANNEL_FIRST..CHANNEL_FIRST+4).contains(&i){BS_OWNERDRAW')
p.write_text(s,encoding='utf-8');print('R506 Window four channel and five source-row controls wired to shared opaque commands')
