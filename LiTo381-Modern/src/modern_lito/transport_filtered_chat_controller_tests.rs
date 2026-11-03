use super::super::super::{filtered_chat_protocol as fc,chat_window as cw};
fn filtered_send(cs:&mut [SessionController;2],i:usize,kind:fc::Kind,text:&str){
    let before=cs[i].snapshot();assert!(before.chat.filtered());
    let intent=cw::Intent{token:before.chat.token().unwrap(),text:text.into(),
        kind:if kind==fc::Kind::Normal {cw::Kind::Message}else{cw::Kind::Public(kind)}};
    let command=super::super::super::login_window::chat_command(&before,intent).unwrap();
    cs[i].command(command).unwrap();assert!(cs[i].snapshot().chat_pending);
    assert_eq!(cs[i].snapshot().chat.filtered_messages(),before.chat.filtered_messages());
    wait(cs,"filtered SEND exact receipt",|cs|!cs[i].snapshot().chat_pending);
    assert_eq!(cs[i].snapshot().chat.delivery(),chat_state::Delivery::Confirmed);
    super::super::super::login_window::validate_snapshot(&cs[i].snapshot()).unwrap();
}
#[test]
#[ignore="real v33 game controllers, category/private intent adapters, production Java TLS; no HWND"]
fn java_filtered_chat_controllers_categories_preferences_receipts_and_qualified_blocks(){
    let mut peer=filtered_peer("normal");let mut cs=clients(&peer);let contexts=login(&mut cs,true);
    for i in 0..2 {assert!(cs[i].snapshot().chat.filtered());private_ready(&mut cs,i);}
    private_control(&mut cs,0,dc::Action::Preferences,None,Some(dc::Preferences{whispers:true,world:false,trade:true}),"",ds::Outcome::Confirmed(dc::Action::Preferences));
    filtered_send(&mut cs,0,fc::Kind::World,"沒有自收也能確認");assert!(cs[0].snapshot().chat.filtered_messages().is_empty());
    wait(&mut cs,"recipient-local world row",|cs|cs[1].snapshot().chat.filtered_messages().len()==1);
    assert_eq!(cs[1].snapshot().chat.filtered_messages()[0].sender.key,contexts[0].key);
    private_control(&mut cs,1,dc::Action::Preferences,None,Some(dc::Preferences{whispers:true,world:true,trade:false}),"",ds::Outcome::Confirmed(dc::Action::Preferences));
    filtered_send(&mut cs,0,fc::Kind::Trade,"交易拒收");
    private_control(&mut cs,1,dc::Action::Block,private_target(7,"Sun Knight"),None,"",ds::Outcome::Confirmed(dc::Action::Block));
    filtered_send(&mut cs,0,fc::Kind::Normal,"一般也被拒收");
    private_control(&mut cs,0,dc::Action::Send,private_target(42,"Moon Mage"),None,"私訊被拒收",ds::Outcome::Unreachable);
    private_control(&mut cs,1,dc::Action::Send,private_target(7,"Sun Knight"),None,"反向可收",ds::Outcome::Confirmed(dc::Action::Send));
    wait(&mut cs,"qualified private delivery",|cs|cs[0].snapshot().direct_chat.messages().len()==1);
    let deadline=Instant::now()+Duration::from_millis(900);
    while Instant::now()<deadline {for c in &mut cs{c.poll();assert_ne!(c.snapshot().phase,LoginPhase::Failed);}thread::sleep(Duration::from_millis(5));}
    assert_eq!(cs[1].snapshot().chat.filtered_messages().len(),1); // Complete periodic read after both excluded rows.
    assert_eq!(cs[0].snapshot().direct_chat.messages()[0].sender.key,contexts[1].key);
    shutdown(&mut cs,&mut peer);
}
#[test]
#[ignore="real v33 game controllers; room/channel/source lifecycle and confirmed private admission"]
fn java_filtered_chat_controllers_room_channel_and_source_return_clear_old_public_admission(){
    let mut peer=filtered_peer("normal");let mut cs=clients(&peer);let contexts=login(&mut cs,true);
    for i in 0..2{private_ready(&mut cs,i);}
    filtered_send(&mut cs,0,fc::Kind::Normal,"大廳歷史");let old=cs[0].snapshot().chat.token().unwrap();
    let mut common=rooms::Common::read(contexts[0],0);common.action=rooms::Action::Create;
    room(&mut cs,0,rooms::Request::draft(common,rooms::Draft::new(rooms::PublicDraft{capacity:4,mode:1,attributes:16,map_id:564,name:"新版分類房".into()},String::new()).unwrap()).unwrap());
    let id=cs[0].snapshot().rooms.joined().unwrap().summary.room_id;
    wait(&mut cs,"room new public bootstrap",|cs|cs[0].snapshot().chat.token().is_some_and(|t|t.scope.room_id==id));private_ready(&mut cs,0);
    assert!(cs[0].snapshot().chat.filtered_messages().is_empty());
    assert!(cs[0].command(LoginCommand::FilteredChatSend{token:old,kind:fc::Kind::World,text:"舊大廳".into()}).is_err());
    let list=read_room(&mut cs,1,contexts[1]);room(&mut cs,1,rooms::Request::join(command(&list,rooms::Action::Join),String::new()).unwrap());
    wait(&mut cs,"guest room bootstrap",|cs|cs[1].snapshot().chat.token().is_some_and(|t|t.scope.room_id==id));private_ready(&mut cs,1);
    filtered_send(&mut cs,0,fc::Kind::Trade,"房間交易");
    wait(&mut cs,"room category receive",|cs|cs[1].snapshot().chat.filtered_messages().len()==1);
    for i in [1,0]{let r=read_room(&mut cs,i,contexts[i]);room(&mut cs,i,rooms::Request::simple(command(&r,rooms::Action::Leave)).unwrap());
        wait(&mut cs,"lobby re-admission",|cs|cs[i].snapshot().chat.token().is_some_and(|t|t.scope.room_id==0));}
    for i in 0..2 {leave_lobby(&mut cs,i,contexts[i]);cs[i].command(LoginCommand::ReturnCharacters(contexts[i])).unwrap();phase(&mut cs,i,LoginPhase::CharactersReady);}
    let moon=select(&mut cs,0,42);enter(&mut cs,0,moon,1,true);private_ready(&mut cs,0);
    let snapshot=cs[0].snapshot();assert!(snapshot.chat.filtered_messages().is_empty());assert_eq!(snapshot.chat.token().unwrap().scope.context,moon);
    assert_eq!(snapshot.direct_chat.owner().unwrap().key,moon.key);assert!(snapshot.direct_chat.messages().is_empty());
    assert!(cs[0].command(LoginCommand::ChatSend{token:old,text:"跨來源舊輸入".into()}).is_err());
    shutdown(&mut cs,&mut peer);
}
