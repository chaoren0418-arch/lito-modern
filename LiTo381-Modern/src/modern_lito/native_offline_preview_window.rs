//! Candidate-only same-process player entry. Standard Modern controls and GDI DIB.
//! Painting never reads resources; normal close pumps until the worker retired.
use super::{native_offline_preview_resources::Inputs,native_offline_preview_runtime::{Runtime,UiPage,Action,key_action,UiFrame}};
use anyhow::{ensure,Result};
use std::cell::{Cell,RefCell};
use windows::{core::{w,PCWSTR},Win32::{Foundation::{HWND,HINSTANCE,LPARAM,WPARAM,LRESULT,RECT},System::LibraryLoader::GetModuleHandleW,Graphics::Gdi::{BeginPaint,EndPaint,FillRect,GetStockObject,GetSysColorBrush,InvalidateRect,StretchDIBits,PAINTSTRUCT,HBRUSH,BLACK_BRUSH,DEFAULT_GUI_FONT,COLOR_BTNFACE,BITMAPINFO,BITMAPINFOHEADER,BI_RGB,DIB_RGB_COLORS,SRCCOPY},UI::{WindowsAndMessaging::*,Input::KeyboardAndMouse::{EnableWindow,GetKeyState,SetFocus},HiDpi::{SetThreadDpiAwarenessContext,DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,GetDpiForWindow}}}};
const CLASS:PCWSTR=w!("LiToModernR475OfflineEntry");const TIMER:usize=0x475;const FIRST:usize=8000;
const TITLE:usize=0;const INFO:usize=1;const NETWORK:usize=2;const OFFLINE:usize=3;const MAP:usize=4;const ENTER:usize=5;const BACK:usize=6;const CLOSE:usize=7;const STATUS:usize=8;const LEFT:usize=9;const RIGHT:usize=10;const UP:usize=11;const DOWN:usize=12;const COUNT:usize=13;
#[derive(Clone,Copy,PartialEq,Eq)]enum Screen{Entry,Maps}
#[derive(Clone,Copy,PartialEq,Eq)]pub enum Outcome{Exit,Network}
struct State {runtime:RefCell<Option<Runtime>>,controls:Cell<[HWND;COUNT]>,screen:Cell<Screen>,outcome:Cell<Outcome>,catalog_painted:Cell<bool>,fault:Cell<bool>}
fn scale(n:i32,dpi:u32)->i32{((i64::from(n)*i64::from(dpi.clamp(48,768))+48)/96) as i32}
unsafe fn label(hwnd:HWND,text:&str){let text:Vec<u16>=text.encode_utf16().chain(Some(0)).collect();let _=SetWindowTextW(hwnd,PCWSTR(text.as_ptr()));}
unsafe fn refresh(hwnd:HWND,s:&State){
 let(page,choice,status,pending)={let r=s.runtime.borrow();let Some(r)=r.as_ref() else{return;};(r.page(),r.choice().cloned(),r.status().to_owned(),r.pan_pending())};
 let entry=s.screen.get()==Screen::Entry;let preview=page==UiPage::Preview;let closing=matches!(page,UiPage::Closing|UiPage::Closed);let c=s.controls.get();
 if !s.catalog_painted.get(){if let Some(choice)=choice.as_ref(){let text:Vec<u16>=format!("{} · {}",choice.name,choice.map_id).encode_utf16().chain(Some(0)).collect();SendMessageW(c[MAP],CB_ADDSTRING,Some(WPARAM(0)),Some(LPARAM(text.as_ptr() as isize)));SendMessageW(c[MAP],CB_SETCURSEL,Some(WPARAM(0)),None);s.catalog_painted.set(true);}}
 label(c[TITLE],if entry{"LiTo Modern"}else{"離線場地預覽 · Death Match"});
 label(c[INFO],if entry{"選擇登入方式。離線預覽可查看 Battle Ground 的原版地圖。"}else if preview{"方向鍵移動視角；Esc 或返回按鈕回到場地選單。"}else{"選擇 Battle Ground，查看原版場地。載入中也可以返回。"});label(c[STATUS],&status);
 for i in 0..COUNT {let visible=match i{TITLE|INFO|STATUS|CLOSE=>true,NETWORK|OFFLINE=>entry,MAP|ENTER=>!entry&&!preview,BACK=>!entry,LEFT|RIGHT|UP|DOWN=>!entry&&preview,_=>false};let _=ShowWindow(c[i],if visible{SW_SHOW}else{SW_HIDE});let enabled=!closing&&match i{OFFLINE=>choice.is_some(),ENTER|MAP=>page==UiPage::Menu&&choice.is_some(),LEFT|RIGHT|UP|DOWN=>preview&&!pending,BACK=>true,_=>true};let _=EnableWindow(c[i],enabled);}
 let _=InvalidateRect(Some(hwnd),None,false);
}
unsafe fn resize(hwnd:HWND,s:&State){let mut r=RECT::default();if GetClientRect(hwnd,&mut r).is_err(){return;}let dpi=GetDpiForWindow(hwnd);let m=scale(24,dpi);let width=(r.right-2*m).max(1);let bottom=(r.bottom-scale(50,dpi)).max(scale(200,dpi));let c=s.controls.get();
 let boxes=[(m,scale(16,dpi),width,scale(30,dpi)),(m,scale(52,dpi),width,scale(45,dpi)),(m,scale(120,dpi),scale(250,dpi),scale(42,dpi)),(m,scale(176,dpi),scale(250,dpi),scale(42,dpi)),(m,scale(110,dpi),scale(420,dpi),scale(240,dpi)),(m+scale(436,dpi),scale(110,dpi),scale(150,dpi),scale(32,dpi)),(m,bottom,scale(170,dpi),scale(32,dpi)),(r.right-m-scale(100,dpi),bottom,scale(100,dpi),scale(32,dpi)),(m,bottom-scale(45,dpi),width,scale(40,dpi)),(m+scale(186,dpi),bottom,scale(60,dpi),scale(32,dpi)),(m+scale(252,dpi),bottom,scale(60,dpi),scale(32,dpi)),(m+scale(318,dpi),bottom,scale(60,dpi),scale(32,dpi)),(m+scale(384,dpi),bottom,scale(60,dpi),scale(32,dpi))];
 for(i,(x,y,w,h))in boxes.into_iter().enumerate(){let _=MoveWindow(c[i],x,y,w,h,true);}let _=InvalidateRect(Some(hwnd),None,false);
}
unsafe fn action(hwnd:HWND,s:&State,a:Action){
 let result={let mut r=s.runtime.borrow_mut();let Some(r)=r.as_mut() else{return;};r.handle(a)};
 if let Err(e)=result{label(s.controls.get()[STATUS],&e.to_string());}else{refresh(hwnd,s);}
}
unsafe fn back(hwnd:HWND,s:&State){let page=s.runtime.borrow().as_ref().map(Runtime::page);if matches!(page,Some(UiPage::Loading|UiPage::Preview)){action(hwnd,s,Action::Back);}else if s.screen.get()==Screen::Maps{s.screen.set(Screen::Entry);refresh(hwnd,s);let _=SetFocus(Some(s.controls.get()[OFFLINE]));}else{action(hwnd,s,Action::Close);}}
unsafe fn paint(hwnd:HWND,s:&State){let mut ps=PAINTSTRUCT::default();let dc=BeginPaint(hwnd,&mut ps);let mut whole=RECT::default();if GetClientRect(hwnd,&mut whole).is_ok(){FillRect(dc,&whole,GetSysColorBrush(COLOR_BTNFACE));}let frame:Option<UiFrame>=if s.screen.get()==Screen::Maps{s.runtime.borrow().as_ref().and_then(Runtime::frame)}else{None};if let Some(frame)=frame{let mut r=RECT::default();if GetClientRect(hwnd,&mut r).is_ok(){let dpi=GetDpiForWindow(hwnd);let m=scale(24,dpi);let top=scale(110,dpi);let footer=scale(104,dpi);if let Some(mut vp)=frame.viewport((r.right-2*m).max(0),(r.bottom-top-footer).max(0)){vp.x+=m;vp.y+=top;let rect=RECT{left:m,top,right:r.right-m,bottom:r.bottom-footer};FillRect(dc,&rect,HBRUSH(GetStockObject(BLACK_BRUSH).0));if let Ok(bgra)=frame.bgra(){let camera=frame.camera();let info=BITMAPINFO{bmiHeader:BITMAPINFOHEADER{biSize:std::mem::size_of::<BITMAPINFOHEADER>() as u32,biWidth:camera.width as i32,biHeight:-(camera.height as i32),biPlanes:1,biBitCount:32,biCompression:BI_RGB.0,..Default::default()},..Default::default()};StretchDIBits(dc,vp.x,vp.y,vp.width,vp.height,0,0,camera.width as i32,camera.height as i32,Some(bgra.as_ptr().cast()),&info,DIB_RGB_COLORS,SRCCOPY);}}}}let _=EndPaint(hwnd,&ps);}
unsafe extern "system" fn proc(hwnd:HWND,msg:u32,wparam:WPARAM,lparam:LPARAM)->LRESULT {
 if msg==WM_NCCREATE{let cs=&*(lparam.0 as *const CREATESTRUCTW);SetWindowLongPtrW(hwnd,GWLP_USERDATA,cs.lpCreateParams as _);}
 let ptr=GetWindowLongPtrW(hwnd,GWLP_USERDATA) as *const State;if ptr.is_null(){return DefWindowProcW(hwnd,msg,wparam,lparam);}let s=&*ptr;
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||match msg{
  WM_COMMAND if (wparam.0>>16)&0xffff==BN_CLICKED as usize=>{match (wparam.0&0xffff).checked_sub(FIRST){Some(NETWORK)=>{s.outcome.set(Outcome::Network);action(hwnd,s,Action::Close);},Some(OFFLINE)=>{s.screen.set(Screen::Maps);refresh(hwnd,s);let _=SetFocus(Some(s.controls.get()[ENTER]));},Some(ENTER)=>action(hwnd,s,Action::Enter),Some(BACK)=>back(hwnd,s),Some(CLOSE)=>action(hwnd,s,Action::Close),Some(LEFT)=>action(hwnd,s,Action::Pan(-48,0)),Some(RIGHT)=>action(hwnd,s,Action::Pan(48,0)),Some(UP)=>action(hwnd,s,Action::Pan(0,-24)),Some(DOWN)=>action(hwnd,s,Action::Pan(0,24)),_=>{}}LRESULT(0)},
  WM_TIMER if wparam.0==TIMER=>{let(changed,closed)={let mut r=s.runtime.borrow_mut();if let Some(r)=r.as_mut(){(r.poll(),r.page()==UiPage::Closed)}else{(false,false)}};if closed{let _=DestroyWindow(hwnd);}else if changed{refresh(hwnd,s);}LRESULT(0)},
  WM_CLOSE=>{action(hwnd,s,Action::Close);LRESULT(0)},WM_SIZE=>{resize(hwnd,s);LRESULT(0)},WM_PAINT=>{paint(hwnd,s);LRESULT(0)},
  WM_DPICHANGED=>{let r=&*(lparam.0 as *const RECT);let _=SetWindowPos(hwnd,None,r.left,r.top,r.right-r.left,r.bottom-r.top,SWP_NOZORDER|SWP_NOACTIVATE);resize(hwnd,s);LRESULT(0)},
  WM_DESTROY=>{let _=KillTimer(Some(hwnd),TIMER);PostQuitMessage(0);LRESULT(0)},WM_NCDESTROY=>{SetWindowLongPtrW(hwnd,GWLP_USERDATA,0);DefWindowProcW(hwnd,msg,wparam,lparam)},_=>DefWindowProcW(hwnd,msg,wparam,lparam)
 }));match result{Ok(r)=>r,Err(_)=>{s.fault.set(true);if let Ok(mut r)=s.runtime.try_borrow_mut(){if let Some(r)=r.as_mut(){let _=r.handle(Action::Close);}}LRESULT(0)}}
}
pub fn run(inputs:Inputs,start_at_maps:bool)->Result<Outcome>{unsafe{
 let old=SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);struct Dpi(windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT);impl Drop for Dpi{fn drop(&mut self){unsafe{let _=SetThreadDpiAwarenessContext(self.0);}}}let _dpi=Dpi(old);
 let instance:HINSTANCE=GetModuleHandleW(None)?.into();let class=WNDCLASSW{lpfnWndProc:Some(proc),hInstance:instance,lpszClassName:CLASS,hbrBackground:GetSysColorBrush(COLOR_BTNFACE),hCursor:LoadCursorW(None,IDC_ARROW)?,..Default::default()};ensure!(RegisterClassW(&class)!=0,"register candidate offline class");
 struct Class(HINSTANCE);impl Drop for Class{fn drop(&mut self){unsafe{let _=UnregisterClassW(CLASS,Some(self.0));}}}let _class=Class(instance);
 let state=Box::new(State{runtime:RefCell::new(None),controls:Cell::new([HWND::default();COUNT]),screen:Cell::new(if start_at_maps{Screen::Maps}else{Screen::Entry}),outcome:Cell::new(Outcome::Exit),catalog_painted:Cell::new(false),fault:Cell::new(false)});
 let hwnd=CreateWindowExW(WS_EX_CONTROLPARENT,CLASS,w!("LiTo Modern · 玩家入口候選 R475"),WS_OVERLAPPEDWINDOW|WS_CLIPCHILDREN,CW_USEDEFAULT,CW_USEDEFAULT,900,850,None,None,Some(instance),Some((&*state as *const State).cast()))?;
 struct Window(HWND);impl Drop for Window{fn drop(&mut self){unsafe{if IsWindow(Some(self.0)).as_bool(){let _=DestroyWindow(self.0);}}}}let _window=Window(hwnd);
 let mut controls=[HWND::default();COUNT];let texts=["LiTo Modern","選擇登入方式","連線登入","離線場地預覽","","進入預覽","返回","退出","正在讀取場地清單…","←","→","↑","↓"];
 for(i,text)in texts.iter().enumerate(){let text:Vec<u16>=text.encode_utf16().chain(Some(0)).collect();let (name,style)=if i==MAP{(w!("COMBOBOX"),WS_CHILD|WS_TABSTOP|WS_VSCROLL|WINDOW_STYLE(CBS_DROPDOWNLIST as u32))}else if matches!(i,TITLE|INFO|STATUS){(w!("STATIC"),WS_CHILD|WS_VISIBLE)}else{(w!("BUTTON"),WS_CHILD|WS_VISIBLE|WS_TABSTOP)};
  controls[i]=CreateWindowExW(WINDOW_EX_STYLE::default(),name,PCWSTR(text.as_ptr()),style,0,0,1,1,Some(hwnd),Some(HMENU((FIRST+i) as *mut _)),Some(instance),None)?;SendMessageW(controls[i],WM_SETFONT,Some(WPARAM(GetStockObject(DEFAULT_GUI_FONT).0 as usize)),Some(LPARAM(1)));
 }state.controls.set(controls);*state.runtime.borrow_mut()=Some(Runtime::start(inputs)?);ensure!(SetTimer(Some(hwnd),TIMER,40,None)!=0,"UI poll timer unavailable");resize(hwnd,&state);refresh(hwnd,&state);let _=ShowWindow(hwnd,SW_SHOW);let _=SetFocus(Some(controls[if start_at_maps{ENTER}else{NETWORK}]));
 let mut msg=MSG::default();loop{let got=GetMessageW(&mut msg,None,0,0).0;ensure!(got>=0,"GetMessage failed");if got==0{break;}if msg.message==WM_KEYDOWN&&(msg.hwnd==hwnd||IsChild(hwnd,msg.hwnd).as_bool()){let page=state.runtime.borrow().as_ref().unwrap().page();if let Some(a)=key_action(page,msg.wParam.0 as u32,GetKeyState(0x11)<0,GetKeyState(0x12)<0,(msg.lParam.0&(1<<30))!=0){if a==Action::Back{back(hwnd,&state);}else{action(hwnd,&state,a);}continue;}}
  if !IsDialogMessageW(hwnd,&msg).as_bool(){let _=TranslateMessage(&msg);DispatchMessageW(&msg);}
 }
 ensure!(!state.fault.get(),"candidate offline UI state error");Ok(state.outcome.get())
}}
