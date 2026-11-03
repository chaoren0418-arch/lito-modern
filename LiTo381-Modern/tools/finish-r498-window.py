from r491_common import *
p=P/'src/modern_lito/native_city_window.rs';s=p.read_text(encoding='utf-8')
s=s.replace('GetSysColorBrush,','GetSysColorBrush,GetSysColor,')
s=s.replace('let _=MoveWindow(c[CHARACTER_SUBMIT],m+scale(310,dpi),scale(338,dpi),scale(220,dpi),scale(32,dpi),true);','let native=super::native_city_runtime::character_selection::SUBMIT;let _=MoveWindow(c[CHARACTER_SUBMIT],m+scale(native.x,dpi),scale(232+native.y,dpi),scale(native.width,dpi),scale(native.height,dpi),true);if s.screen.get()==Screen::Entry&&s.runtime.borrow().as_ref().is_some_and(|r|r.source_identity().complete()&&!r.source_identity().records().is_empty()){let native=super::native_city_runtime::character_selection::BACK;let _=MoveWindow(c[BACK],m+scale(native.x,dpi),scale(232+native.y,dpi),scale(native.width,dpi),scale(native.height,dpi),true);}')
s=s.replace('let _=InvalidateRect(Some(hwnd),None,false);\n}\nunsafe fn resize','resize(hwnd,s);let _=InvalidateRect(Some(hwnd),None,false);\n}\nunsafe fn resize',1)
s=s.replace('BS_NOTIFY as u32}else{0}','BS_NOTIFY as u32}else if i==CHARACTER_SUBMIT||i==BACK{BS_OWNERDRAW as u32}else{0}')
marker='unsafe extern "system" fn proc'
fn='''unsafe fn draw_character_button(s:&State,draw:&DRAWITEMSTRUCT){
 FillRect(draw.hDC,&draw.rcItem,GetSysColorBrush(COLOR_BTNFACE));
 let submit=draw.CtlID as usize==FIRST+CHARACTER_SUBMIT;
 let r=s.runtime.borrow();let image=r.as_ref().and_then(Runtime::skin).and_then(|skin|{let index=if submit{0}else{2}+if draw.itemState.0&ODS_SELECTED.0!=0{1}else{0};skin.character_buttons.get(index).map(|i|(i.width,i.height,i.rgba.clone()))});
 if s.screen.get()==Screen::Entry{if let Some((w,h,rgba))=image{let color=GetSysColor(COLOR_BTNFACE);let bg=[(color>>16)as u8,(color>>8)as u8,color as u8,255];let mut b=Vec::with_capacity(rgba.len());for pixel in rgba.chunks_exact(4){b.extend_from_slice(if pixel[3]==0{&bg}else{&[pixel[2],pixel[1],pixel[0],255]});}let info=BITMAPINFO{bmiHeader:BITMAPINFOHEADER{biSize:std::mem::size_of::<BITMAPINFOHEADER>()as u32,biWidth:w as i32,biHeight:-(h as i32),biPlanes:1,biBitCount:32,biCompression:BI_RGB.0,..Default::default()},..Default::default()};StretchDIBits(draw.hDC,draw.rcItem.left,draw.rcItem.top,draw.rcItem.right-draw.rcItem.left,draw.rcItem.bottom-draw.rcItem.top,0,0,w as i32,h as i32,Some(b.as_ptr().cast()),&info,DIB_RGB_COLORS,SRCCOPY);return;}}
 let mut text:Vec<u16>=if submit{"進入角色"}else{"返回"}.encode_utf16().collect();let mut rect=draw.rcItem;DrawTextW(draw.hDC,&mut text,&mut rect,DT_SINGLELINE|DT_VCENTER|DT_CENTER);
}
'''
assert marker in s;s=s.replace(marker,fn+marker)
marker='  WM_COMMAND if (usize::from(SOURCE_CHARACTER_COMMAND_FIRST)'
assert marker in s;s=s.replace(marker,'  WM_DRAWITEM=>{let draw=&*(lparam.0 as *const DRAWITEMSTRUCT);if draw.CtlID as usize==FIRST+CHARACTER_SUBMIT||draw.CtlID as usize==FIRST+BACK{draw_character_button(s,draw);LRESULT(1)}else{DefWindowProcW(hwnd,msg,wparam,lparam)}},\n'+marker)
p.write_text(s,encoding='utf-8')
print('R498 original51x23 submit/back IMG drawn;slot art not substituted')
