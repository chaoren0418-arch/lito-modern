from r491_common import *
p=P/'src/modern_lito/native_city_window.rs';s=p.read_text(encoding='utf-8')
changes=[
 ('source_character_command_slot}}','source_character_command_slot,presentation::Gate}}'),
 ('const COUNT:usize=17;','const SOURCE_ENTRY:usize=17;const COUNT:usize=18;'),
 ('struct State {character_controls:','struct State {refresh_gate:RefCell<Gate>,character_controls:'),
 ('unsafe fn refresh(hwnd:HWND,s:&State){','unsafe fn refresh(hwnd:HWND,s:&State){\n let key={let r=s.runtime.borrow();let Some(r)=r.as_ref()else{return;};r.presentation_key()};if !s.refresh_gate.borrow_mut().changed(key,s.screen.get()==Screen::Entry){return;}'),
 ('let visible=match i{CHARACTER_FIRST..=16=>entry&&roster,','let visible=match i{SOURCE_ENTRY=>entry&&s.runtime.borrow().as_ref().is_some_and(|r|r.source_connection_epoch().is_some()&&r.skin().is_some()&&r.source_flow().phase()==2),CHARACTER_FIRST..=16=>entry&&roster,'),
 ('let enabled=!closing&&match i{CHARACTER_FIRST..=16=>','let enabled=!closing&&match i{SOURCE_ENTRY=>s.runtime.borrow().as_ref().is_some_and(Runtime::source_menu_invoker_enabled),CHARACTER_FIRST..=16=>'),
 ('for slot in 0..4{let w=(width-scale(12,dpi))/2;', 'let _=MoveWindow(c[SOURCE_ENTRY],m+scale(18,dpi),scale(338,dpi),scale(220,dpi),scale(32,dpi),true);for slot in 0..4{let w=(width-scale(12,dpi))/2;'),
 ('paint_menu(hwnd,s,dc);','paint_menu(hwnd,s,dc);paint_source_entry(hwnd,s,dc);'),
 ('Box::new(State{character_controls:','Box::new(State{refresh_gate:RefCell::new(Gate::default()),character_controls:'),
 ('"角色 1","角色 2","角色 3","角色 4"]','"角色 1","角色 2","角色 3","角色 4","場地選單"]'),
 ('if i>=CHARACTER_FIRST{usize::from(SOURCE_CHARACTER_COMMAND_FIRST)+i-CHARACTER_FIRST}','if i==SOURCE_ENTRY{usize::from(SOURCE_MENU_COMMAND_ID)}else if i>=CHARACTER_FIRST{usize::from(SOURCE_CHARACTER_COMMAND_FIRST)+i-CHARACTER_FIRST}')]
for a,b in changes:assert s.count(a)==1,(a,s.count(a));s=s.replace(a,b)
anchor='unsafe fn paint(hwnd:HWND,s:&State)'
draw='''unsafe fn paint_source_entry(hwnd:HWND,s:&State,dc:windows::Win32::Graphics::Gdi::HDC){
 if s.screen.get()!=Screen::Entry{return;}let r=s.runtime.borrow();let Some(r)=r.as_ref()else{return;};if r.source_connection_epoch().is_none()||r.source_flow().phase()!=2{return;}let Some(skin)=r.skin()else{return;};let Some(bgra)=skin.entry_art.as_ref()else{return;};let dpi=GetDpiForWindow(hwnd);let info=BITMAPINFO{bmiHeader:BITMAPINFOHEADER{biSize:std::mem::size_of::<BITMAPINFOHEADER>()as u32,biWidth:11,biHeight:-11,biPlanes:1,biBitCount:32,biCompression:BI_RGB.0,..Default::default()},..Default::default()};StretchDIBits(dc,scale(24,dpi),scale(348,dpi),scale(11,dpi),scale(11,dpi),0,0,11,11,Some(bgra.as_ptr().cast()),&info,DIB_RGB_COLORS,SRCCOPY);
}
'''
assert s.count(anchor)==1;s=s.replace(anchor,draw+anchor);p.write_text(s,encoding='utf-8')
print('R497 actual4910 button+source6279 badge;allrefresh callers share gate beforelabels/tokens/Invalidate')
