from r491_common import *
p=P/'src/modern_lito/native_city_runtime.rs';s=p.read_text(encoding='utf-8');a='Ok(ingress)=>StreamEvent{ingress,counter_step:Some(step)},Err(_)';b='Ok(StreamIngress::Navigation(change@entry_navigation::Change::RestartAfterNativeAlert{..}))=>{batch.events.push(StreamEvent{ingress:StreamIngress::Navigation(change),counter_step:Some(step)});batch.terminal=Some(source_stream::End{terminal:source_stream::Terminal::DispatchRefused,before});break;},Ok(ingress)=>StreamEvent{ingress,counter_step:Some(step)},Err(_)';assert s.count(a)==1;s=s.replace(a,b);p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_city_window.rs';s=p.read_text(encoding='utf-8')
def sub(a,b):
 global s
 assert s.count(a)==1,(a,s.count(a));s=s.replace(a,b)
sub('let lobby=navigation.scene==NavigationScene::Lobby;','let lobby=navigation.scene==NavigationScene::Lobby;let restart=navigation.scene==NavigationScene::ReconnectRequired;')
sub('label(c[NETWORK],if s.runtime','label(c[NETWORK],if restart{"重新連線"}else if s.runtime')
sub('label(c[INFO],if entry&&lobby','label(c[INFO],if entry&&restart{"上一連線已結束。請重新連線後再選擇角色。"}else if entry&&lobby')
sub('NETWORK|OFFLINE=>entry&&!login_active&&!navigation_active','NETWORK=>entry&&!login_active&&(!navigation_active||restart),OFFLINE=>entry&&!login_active&&!navigation_active')
sub('NETWORK=>page==UiPage::Menu&&!roster&&!navigation_active','NETWORK=>page==UiPage::Menu&&!roster&&(!navigation_active||restart)')
sub('BACK=>!entry||roster||login_active||navigation_active','BACK=>!entry||roster||login_active||(navigation_active&&!restart)')
sub('let mut text:Vec<u16>=if submit{"進入角色"}else{"返回"}.encode_utf16().collect();let mut rect=draw.rcItem;DrawTextW(draw.hDC,&mut text,&mut rect,DT_SINGLELINE|DT_VCENTER|DT_CENTER);','let mut text=[0u16;128];let n=GetWindowTextW(draw.hwndItem,&mut text)as usize;let mut rect=draw.rcItem;DrawTextW(draw.hDC,&mut text[..n],&mut rect,DT_SINGLELINE|DT_VCENTER|DT_CENTER);')
p.write_text(s,encoding='utf-8')
