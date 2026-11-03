from r491_common import *
p=P/'src/modern_lito/native_city_window.rs';s=p.read_text(encoding='utf-8')
replacements=[
 ('UiFrame,SOURCE_MENU_COMMAND_ID','UiFrame,SOURCE_MENU_COMMAND_ID,SOURCE_CHARACTER_COMMAND_FIRST'),
 ('const COUNT:usize=13;','const CHARACTER_FIRST:usize=13;const COUNT:usize=17;'),
 (' let entry=s.screen.get()==Screen::Entry;',' let roster=s.runtime.borrow().as_ref().is_some_and(|r|r.source_connection_epoch().is_some()&&r.source_identity().complete()&&!r.source_identity().records().is_empty());\n let entry=s.screen.get()==Screen::Entry;'),
 ('for i in 0..COUNT {let visible=match i{','for i in 0..COUNT {let visible=match i{CHARACTER_FIRST..=16=>entry&&roster,'),
 ('let enabled=!closing&&match i{','let enabled=!closing&&match i{CHARACTER_FIRST..=16=>s.runtime.borrow().as_ref().is_some_and(|r|r.source_character_control_enabled(i-CHARACTER_FIRST)),'),
 (' let _=InvalidateRect(Some(hwnd),None,false);\n}\nunsafe fn resize',' for slot in 0..4{let text={let r=s.runtime.borrow();r.as_ref().and_then(|r|r.source_identity().records().get(slot)).map(|record|if record.name().is_ascii(){format!("角色 {} · {}",slot+1,std::str::from_utf8(record.name()).unwrap())}else{format!("角色 {}",slot+1)}).unwrap_or_else(||format!("角色 {}",slot+1))};label(c[CHARACTER_FIRST+slot],&text);}\n let _=InvalidateRect(Some(hwnd),None,false);\n}\nunsafe fn resize'),
 ('for(i,(x,y,w,h))in boxes.into_iter().enumerate(){let _=MoveWindow(c[i],x,y,w,h,true);}','for(i,(x,y,w,h))in boxes.into_iter().enumerate(){let _=MoveWindow(c[i],x,y,w,h,true);}for slot in 0..4{let w=(width-scale(12,dpi))/2;let _=MoveWindow(c[CHARACTER_FIRST+slot],m+(slot%2)as i32*(w+scale(12,dpi)),scale(232+(slot/2)as i32*50,dpi),w.max(1),scale(42,dpi),true);}'),
 ('  WM_COMMAND if (wparam.0&0xffff)==usize::from(SOURCE_MENU_COMMAND_ID)', '  WM_COMMAND if (usize::from(SOURCE_CHARACTER_COMMAND_FIRST)..usize::from(SOURCE_CHARACTER_COMMAND_FIRST)+4).contains(&(wparam.0&0xffff))&&((wparam.0>>16)&0xffff)==BN_CLICKED as usize=>{let result={let mut r=s.runtime.borrow_mut();r.as_mut().map(|r|r.handle_player_command((wparam.0&0xffff)as u16))};if let Some(Err(e))=result{label(s.controls.get()[STATUS],&e.to_string());}else{refresh(hwnd,s);}LRESULT(0)},\n  WM_COMMAND if (wparam.0&0xffff)==usize::from(SOURCE_MENU_COMMAND_ID)'),
 ('if changed||scrolled{refresh(hwnd,s);}','let _=(changed,scrolled);refresh(hwnd,s);'),
 ('"←","→","↑","↓"]','"←","→","↑","↓","角色 1","角色 2","角色 3","角色 4"]'),
 ('Some(HMENU((FIRST+i) as *mut _))','Some(HMENU((if i>=CHARACTER_FIRST{usize::from(SOURCE_CHARACTER_COMMAND_FIRST)+i-CHARACTER_FIRST}else{FIRST+i}) as *mut _))')]
for a,b in replacements:
 assert s.count(a)==1,(a,s.count(a));s=s.replace(a,b)
p.write_text(s,encoding='utf-8')
print('R496 four real Win32 button definitions/IDs/WM_COMMAND ->same Runtime typedcs201;GUI never executed')
