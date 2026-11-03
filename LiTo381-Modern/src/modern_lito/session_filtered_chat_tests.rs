fn filtered_view_fixture()->LoginSnapshot {
    let mut v=chat_view_fixture();let scope=current_chat_scope(&v);
    v.chat.align_profile(true,true,scope);
    let echo=v.chat.filtered_read().unwrap();
    let owner=direct::Peer{key:echo.context.key,character_name:"3333".into(),source_name:"太陽神".into()};
    v.chat.receive_filtered(filtered_chat::Reply{echo,status:filtered_chat::Status::Ok,membership:7,sequence:0,cursor:0,
        available:true,truncated:false,owner,messages:vec![],receipt:None}).unwrap();v
}
#[test]
fn filtered_commands_have_one_owner_and_never_use_old_public_mailbox(){
    for kind in [filtered_chat::Kind::Normal,filtered_chat::Kind::World,filtered_chat::Kind::Trade] {
        let view=filtered_view_fixture();let token=view.chat.token().unwrap();
        let (mut c,release)=controller_with(view.clone());
        let command=if kind==filtered_chat::Kind::Normal {LoginCommand::ChatSend{token,text:"分類測試".into()}}
            else {LoginCommand::FilteredChatSend{token,kind,text:"分類測試".into()}};
        c.command(command).unwrap();let worker=c.active.as_ref().unwrap();let inbox=worker.inbox.clone();
        assert!(inbox.lock().unwrap().chat.is_none());
        let (request,epoch)=inbox.lock().unwrap().filtered_chat.take().unwrap();
        assert_eq!(request.kind,kind);assert_eq!(request.membership,7);assert_eq!(worker.pending_chat,Some(epoch));
        assert!(c.snapshot().chat_pending&&c.snapshot().chat.filtered_messages().is_empty());
        assert!(c.command(LoginCommand::FilteredChatSend{token,kind,text:"重複".into()}).is_err());
        publish_navigation(&c.active.as_ref().unwrap().published,view.clone(),token.scope.context.epoch,epoch-1);
        c.poll();assert!(c.snapshot().chat_pending,"periodic READ must not acknowledge queued SEND");
        c.command(LoginCommand::Shutdown).unwrap();release.send(()).unwrap();
    }
}
#[test]
fn filtered_category_requires_new_profile_and_rejects_stale_scope_before_enqueue(){
    let view=chat_view_fixture();let old=view.chat.token().unwrap();let (mut legacy,release)=controller_with(view);
    assert!(legacy.command(LoginCommand::FilteredChatSend{token:old,kind:filtered_chat::Kind::World,text:"不可降級".into()}).is_err());
    assert!(legacy.active.as_ref().unwrap().inbox.lock().unwrap().filtered_chat.is_none());legacy.command(LoginCommand::Shutdown).unwrap();release.send(()).unwrap();
    let view=filtered_view_fixture();let mut token=view.chat.token().unwrap();token.membership+=1;
    let (mut c,release)=controller_with(view);
    assert!(c.command(LoginCommand::FilteredChatSend{token,kind:filtered_chat::Kind::Trade,text:"過期".into()}).is_err());
    assert!(!c.snapshot().chat_pending);c.command(LoginCommand::Shutdown).unwrap();release.send(()).unwrap();
}

#[test]
fn filtered_shutdown_clears_queued_text_before_worker_retirement() {
    let view = filtered_view_fixture();
    let token = view.chat.token().unwrap();
    let (mut controller, release) = controller_with(view);
    let inbox = controller.active.as_ref().unwrap().inbox.clone();
    controller.command(LoginCommand::FilteredChatSend {
        token, kind: filtered_chat::Kind::World, text: "不可留在已取消的信箱".into(),
    }).unwrap();
    assert!(inbox.lock().unwrap().busy());
    controller.command(LoginCommand::Shutdown).unwrap();
    let cleared = inbox.lock().unwrap().filtered_chat.is_none();
    let shutdown = inbox.lock().unwrap().shutdown;
    // Release before assertions so a failing regression cannot retain our worker.
    release.send(()).unwrap();
    assert!(shutdown);
    assert!(cleared, "queued category text must retire with every other command");
}
