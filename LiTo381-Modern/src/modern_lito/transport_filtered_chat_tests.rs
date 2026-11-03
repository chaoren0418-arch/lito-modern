// Real production listener + TLS/application/navigation/Rooms, synthetic accounts only.
use super::super::filtered_chat_protocol as filtered;
fn filtered_peer(mode:&str)->JavaPeer {
    JavaPeer::start_class("local.lito381.server.ModernFilteredChatTestPeer",mode,"30000")
}
fn filtered_login(peer:&JavaPeer,source:u32)->(ModernTlsConnection,SelectionContext){
    chat_login_caps(peer,source,super::super::protocol::CAP_DIRECT_CHAT|super::super::protocol::CAP_FILTERED_CHAT)
}
fn filtered_boot(c:&mut ModernTlsConnection,context:SelectionContext,channel:u8,room:u64)->filtered::Reply {
    c.filtered_chat(filtered::Request::read(context,channel,room)).unwrap()
}
fn filtered_private(c:&mut ModernTlsConnection,r:&filtered::Reply)->direct::Reply {
    c.direct_chat(direct::Request::read(r.echo.context,r.echo.channel_id,r.echo.room_id,r.membership)).unwrap()
}
#[test]
#[ignore="requires prepared Java 17 filtered-chat TLS authority"]
fn java_filtered_chat_receipt_preferences_blocks_and_qualified_private_delivery(){
    let mut peer=filtered_peer("normal");let (mut a,ca)=filtered_login(&peer,7);let (mut b,cb)=filtered_login(&peer,42);
    assert!(a.supports_filtered_chat()&&b.supports_filtered_chat());
    let mut ra=filtered_boot(&mut a,ca,0,0);let rb=filtered_boot(&mut b,cb,0,0);
    let mut da=filtered_private(&mut a,&ra);let mut db=filtered_private(&mut b,&rb);
    da=direct_control(&mut a,&da,direct::Action::Preferences,None,Some(direct::Preferences{whispers:true,world:false,trade:true}));
    ra=a.filtered_chat(ra.next_send(filtered::Kind::World,"世界訊息🙂".into()).unwrap()).unwrap();
    assert_eq!(ra.status,filtered::Status::Ok);assert!(ra.messages.is_empty());assert_eq!(ra.cursor,0);
    assert_eq!(ra.receipt.as_ref().unwrap().sender.key,ca.key);
    let rb=b.filtered_chat(rb.next_read()).unwrap();assert_eq!(rb.messages.len(),1);assert_eq!(rb.messages[0].sender.key,ca.key);
    db=direct_control(&mut b,&db,direct::Action::Preferences,None,Some(direct::Preferences{whispers:true,world:true,trade:false}));
    ra=a.filtered_chat(ra.next_send(filtered::Kind::Trade,"交易訊息".into()).unwrap()).unwrap();
    let rb=b.filtered_chat(rb.next_read()).unwrap();assert!(rb.messages.is_empty());assert_eq!(rb.cursor,1);
    db=direct_control(&mut b,&db,direct::Action::Block,direct_target(7,"Sun Knight"),None);
    ra=a.filtered_chat(ra.next_send(filtered::Kind::Normal,"已拒收".into()).unwrap()).unwrap();
    assert!(b.filtered_chat(rb.next_read()).unwrap().messages.is_empty());
    da=direct_send(&mut a,&da,42,"Moon Mage","拒收私訊");assert_eq!(da.status,direct::Status::Unreachable);
    db=direct_send(&mut b,&db,7,"Sun Knight","反向私訊可收");assert_eq!(db.status,direct::Status::Ok);
    let delivered=a.direct_chat(da.next_read()).unwrap();assert_eq!(delivered.messages.len(),1);
    assert_eq!(delivered.messages[0].sender.key,cb.key);assert_eq!(delivered.messages[0].recipient.key,ca.key);
    assert_eq!(ra.sequence,3);a.close().unwrap();b.close().unwrap();peer.expect_successful_exit();
}
#[test]
#[ignore="requires prepared Java 17 filtered-chat TLS authority"]
fn java_filtered_chat_legacy_normal_bridge_and_room_admission_reset(){
    let mut peer=filtered_peer("normal");let (mut a,ca)=filtered_login(&peer,7);let (mut b,cb)=direct_login(&peer,42);
    let mut ra=filtered_boot(&mut a,ca,0,0);let rb=chat_boot(&mut b,cb,0,0);
    ra=a.filtered_chat(ra.next_send(filtered::Kind::World,"不降級為一般".into()).unwrap()).unwrap();
    assert!(b.chat(rb.next_read()).unwrap().messages.is_empty());
    let rb=b.chat(rb.next_send("舊一般可互通".into()).unwrap()).unwrap();
    ra=a.filtered_chat(ra.next_read()).unwrap();assert_eq!(ra.messages.len(),1);assert_eq!(ra.messages[0].kind,filtered::Kind::Normal);
    let prior=ra.clone();let room=chat_create(&mut a,ca);
    assert!(a.filtered_chat(prior.next_send(filtered::Kind::Normal,"舊大廳".into()).unwrap()).is_err());
    ra=filtered_boot(&mut a,ca,0,room);assert!(ra.membership>prior.membership);assert_eq!(ra.cursor,0);assert_eq!(ra.sequence,prior.sequence);
    b.chat(rb.next_send("仍在大廳".into()).unwrap()).unwrap();assert!(a.filtered_chat(ra.next_read()).unwrap().messages.is_empty());
    chat_join(&mut b,cb);let rb=chat_boot(&mut b,cb,0,room);
    b.chat(rb.next_send("同房間".into()).unwrap()).unwrap();ra=a.filtered_chat(ra.next_read()).unwrap();assert_eq!(ra.messages.len(),1);
    chat_leave(&mut a,ca);let fresh=filtered_boot(&mut a,ca,0,0);assert!(fresh.membership>ra.membership);assert_eq!(fresh.cursor,0);
    a.close().unwrap();b.close().unwrap();peer.expect_successful_exit();
}
#[test]
#[ignore="requires prepared Java 17 filtered-chat TLS authority"]
fn java_filtered_chat_navigation_keeps_sequences_and_clears_retired_history(){
    let mut peer=filtered_peer("normal");let (mut a,ca)=filtered_login(&peer,7);let (mut b,cb)=filtered_login(&peer,42);
    let ra=filtered_boot(&mut a,ca,0,0);let rb=filtered_boot(&mut b,cb,0,0);
    let da=filtered_private(&mut a,&ra);filtered_private(&mut b,&rb);
    let da=direct_send(&mut a,&da,42,"Moon Mage","切頁前私訊");
    let ra=a.filtered_chat(ra.next_send(filtered::Kind::Normal,"切頁前公聊".into()).unwrap()).unwrap();
    a.leave_lobby().unwrap();assert!(a.filtered_chat(ra.next_read()).is_err());a.enter_channel(1).unwrap();
    let fresh=filtered_boot(&mut a,ca,1,0);assert!(fresh.membership>ra.membership);assert_eq!((fresh.sequence,fresh.cursor),(1,0));
    let private=filtered_private(&mut a,&fresh);assert_eq!((private.sequence,private.cursor),(da.sequence,0));
    a.leave_lobby().unwrap();a.return_characters().unwrap();let rows=a.characters(7,2).unwrap();
    let next=SelectionContext{key:rows.characters[0].key,epoch:2};a.select_character(next).unwrap();a.enter_channel(0).unwrap();
    let final_read=filtered_boot(&mut a,next,0,0);assert!(final_read.membership>fresh.membership);assert_eq!(final_read.sequence,1);
    let final_private=filtered_private(&mut a,&final_read);assert_eq!(final_private.sequence,1);assert_eq!(final_private.preferences,direct::Preferences::default());
    a.close().unwrap();b.close().unwrap();peer.expect_successful_exit();
}
#[test]
#[ignore="requires prepared Java 17 filtered-chat TLS authority"]
fn java_filtered_chat_old_endpoints_cannot_bypass_negotiated_profile(){
    for private in [false,true] {
        let mut peer=filtered_peer("reject");let (mut a,ca)=filtered_login(&peer,7);let (mut b,cb)=filtered_login(&peer,42);
        let ra=filtered_boot(&mut a,ca,0,0);filtered_boot(&mut b,cb,0,0);
        assert!(a.chat(chat::Request::read(ca,0,0)).is_err());a.ping().unwrap();
        let id=a.next_request_id().unwrap();
        let bytes=if private {direct::encode_request(id,&direct::Request::read(ca,0,0,ra.membership),33).unwrap()}
            else {chat::encode_request(id,&chat::Request::read(ca,0,0),33).unwrap()};
        assert!(a.exchange(&bytes,id,Stage::Chat).is_err());assert_eq!(a.application_state(),ApplicationState::Closed);
        b.ping().unwrap();a.close().unwrap();b.close().unwrap();peer.expect_successful_exit();
    }
}
#[test]
#[ignore="requires prepared Java 17 filtered-chat TLS authority"]
fn java_filtered_chat_forged_scope_membership_cursor_and_sequence_fail_closed(){
    for case in 0..6 {
        let mut peer=filtered_peer("reject");let (mut a,ca)=filtered_login(&peer,7);let (mut b,cb)=filtered_login(&peer,42);
        let ra=filtered_boot(&mut a,ca,0,0);let rb=filtered_boot(&mut b,cb,0,0);
        let mut request=ra.next_send(filtered::Kind::Normal,"不能送達".into()).unwrap();
        match case {0=>request.context.key=cb.key,1=>request.context.epoch=2,2=>request.channel_id=1,
            3=>request.membership+=1,4=>request.after=1,5=>request.sequence=2,_=>unreachable!()}
        let id=a.next_request_id().unwrap();let bytes=filtered::encode_request(id,&request,33).unwrap();
        assert!(a.exchange(&bytes,id,Stage::Chat).is_err());assert_eq!(a.application_state(),ApplicationState::Closed);
        assert!(b.filtered_chat(rb.next_read()).unwrap().messages.is_empty());
        a.close().unwrap();b.close().unwrap();peer.expect_successful_exit();
    }
}
#[test]
#[ignore="requires prepared Java 17 filtered-chat TLS authority"]
fn java_filtered_chat_unnegotiated_new_frames_fail_and_missing_provider_stays_legacy(){
    for private in [false,true] {
        let mut peer=filtered_peer("reject");let (mut a,ca)=direct_login(&peer,7);let (mut b,cb)=filtered_login(&peer,42);
        assert!(!a.supports_filtered_chat());let ra=direct_boot(&mut a,ca,0,0);filtered_boot(&mut b,cb,0,0);
        let id=a.next_request_id().unwrap();let bytes=if private {direct::encode_moderated_request(id,&ra.next_read(),33).unwrap()}
            else {filtered::encode_request(id,&filtered::Request::read(ca,0,0),33).unwrap()};
        assert!(a.exchange(&bytes,id,Stage::Chat).is_err());b.ping().unwrap();
        a.close().unwrap();b.close().unwrap();peer.expect_successful_exit();
    }
    let mut peer=filtered_peer("unavailable");let (mut a,ca)=filtered_login(&peer,7);let (mut b,cb)=filtered_login(&peer,42);
    assert!(!a.supports_filtered_chat()&&!b.supports_filtered_chat());
    let ra=direct_boot(&mut a,ca,0,0);direct_boot(&mut b,cb,0,0);
    assert_eq!(direct_send(&mut a,&ra,42,"Moon Mage","舊私訊仍正常").status,direct::Status::Ok);
    assert!(a.filtered_chat(filtered::Request::read(ca,0,0)).is_err());a.ping().unwrap();
    a.close().unwrap();b.close().unwrap();peer.expect_successful_exit();
}
#[test]
#[ignore="v33 actual moderated-private commit then reply loss or cancellation"]
fn java_filtered_chat_committed_private_loss_cancel_and_controls_never_replay(){
    for (mode,cancel) in [("silent-lobby",false),("silent-room",false),("cancel-lobby",true),("cancel-room",true),
        ("prefs-lobby",false),("block-room",false)] {
        committed_direct_profile_fault(mode,cancel,true);
    }
}
