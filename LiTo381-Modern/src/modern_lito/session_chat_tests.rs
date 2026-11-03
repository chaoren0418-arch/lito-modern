// Included in session::tests. These are deterministic mailbox tests, not TLS/UI proof.
fn chat_view_fixture() -> LoginSnapshot {
    let mut view = navigation_view(
        &principal(),
        navigation_reply(
            context(11),
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        ),
    )
    .unwrap();
    let mut room = rooms::tests::sample(context(11), true);
    room.common.channel_id = 2;
    view.rooms = rooms::State::Ready(Arc::new(room));
    view.chat.align(true, current_chat_scope(&view));
    view.chat
        .receive(chat::Reply {
            echo: view.chat.read().unwrap(),
            status: chat::Status::Ok,
            membership: 7,
            sequence: 0,
            cursor: 0,
            available: true,
            truncated: false,
            messages: vec![],
        })
        .unwrap();
    view
}
#[test]
fn chat_command_has_one_owner_and_background_read_cannot_ack_send_or_navigate() {
    let mut view = chat_view_fixture();
    let token = view.chat.token().unwrap();
    let (mut c, release) = controller_with(view.clone());
    let status = c.snapshot().status;
    c.command(LoginCommand::ChatSend {
        token,
        text: "不可預先送達".into(),
    })
    .unwrap();
    let worker = c.active.as_ref().unwrap();
    let inbox = worker.inbox.clone();
    let (request, epoch) = inbox.lock().unwrap().chat.take().unwrap();
    assert_eq!(worker.pending_chat, Some(epoch));
    assert!(c.snapshot().chat_pending && c.snapshot().chat.messages().is_empty());
    assert_eq!(c.snapshot().status, status);
    for command in [
        LoginCommand::ChatSend {
            token,
            text: "重複".into(),
        },
        LoginCommand::Rooms(
            rooms::Request::read(token.scope.context, token.scope.channel_id).unwrap(),
        ),
        LoginCommand::LeaveLobby(token.scope.context),
        LoginCommand::ReadRecord,
        LoginCommand::ReadSkills,
        LoginCommand::World(world::Request::read(
            token.scope.context,
            token.scope.channel_id,
        )),
    ] {
        assert!(c.command(command).is_err());
    }
    publish_navigation(
        &c.active.as_ref().unwrap().published,
        view.clone(),
        token.scope.context.epoch,
        epoch - 1,
    );
    c.poll();
    assert!(
        c.snapshot().chat_pending,
        "periodic read cannot confirm a queued send"
    );
    view.chat
        .receive(chat::Reply {
            echo: request,
            status: chat::Status::RateLimited,
            membership: 7,
            sequence: 1,
            cursor: 0,
            available: true,
            truncated: false,
            messages: vec![],
        })
        .unwrap();
    publish_navigation(
        &c.active.as_ref().unwrap().published,
        view,
        token.scope.context.epoch,
        epoch,
    );
    c.poll();
    assert!(!c.snapshot().chat_pending);
    assert_eq!(
        c.snapshot().chat.delivery(),
        chat_state::Delivery::RateLimited
    );
    assert_eq!(c.snapshot().status, status);
    c.command(LoginCommand::ChatSend {
        token,
        text: "使用者另行送出".into(),
    })
    .unwrap();
    assert_eq!(inbox.lock().unwrap().chat.as_ref().unwrap().0.sequence, 2);
    c.command(LoginCommand::Shutdown).unwrap();
    assert!(c.snapshot().chat.messages().is_empty() && c.snapshot().chat.token().is_none());
    assert!(inbox.lock().unwrap().chat.is_none());
    release.send(()).unwrap();
}
#[test]
fn chat_send_rejects_stale_token_and_other_owned_work_without_reserving_sequence() {
    let view = chat_view_fixture();
    let token = view.chat.token().unwrap();
    let (cmut, release) = controller_with(view);
    let mut c = cmut;
    for field in 0..5 {
        let mut old = token;
        match field {
            0 => old.scope.context.key.source_id = 42,
            1 => old.scope.context.epoch += 1,
            2 => old.scope.channel_id = 1,
            3 => old.scope.room_id += 1,
            _ => old.membership += 1,
        }
        assert!(c
            .command(LoginCommand::ChatSend {
                token: old,
                text: "過期輸入".into()
            })
            .is_err());
    }
    let before = c.next_selection;
    let inbox = c.active.as_ref().unwrap().inbox.clone();
    inbox.lock().unwrap().auto_world = true;
    assert!(c
        .command(LoginCommand::ChatSend {
            token,
            text: "進場期間".into()
        })
        .is_err());
    inbox.lock().unwrap().auto_world = false;
    c.active.as_mut().unwrap().pending_record = Some((token.scope.context, before));
    assert!(c
        .command(LoginCommand::ChatSend {
            token,
            text: "尚未確認".into()
        })
        .is_err());
    assert_eq!(c.next_selection, before);
    assert!(inbox.lock().unwrap().chat.is_none());
    release.send(()).unwrap();
}
#[test]
fn chat_publication_clears_room_or_source_history_before_exposing_changed_page() {
    let mut view = chat_view_fixture();
    let token = view.chat.token().unwrap();
    let mut reply = chat::Reply {
        echo: view.chat.read().unwrap(),
        status: chat::Status::Ok,
        membership: 7,
        sequence: 0,
        cursor: 1,
        available: true,
        truncated: false,
        messages: vec![],
    };
    reply.messages.push(chat::Message {
        id: 1,
        sender: token.scope.context.key,
        character_name: "3333".into(),
        source_name: "太陽神".into(),
        text: "私房內容".into(),
    });
    view.chat.receive(reply).unwrap();
    let shared = Mutex::new(Published {
        view: view.clone(),
        source_epoch: 11,
        navigation_epoch: 0,
    });
    let mut lobby = view.clone();
    let mut rooms = lobby.rooms.clone();
    if let rooms::State::Ready(r) = &mut rooms {
        Arc::make_mut(r).joined = None;
    }
    lobby.rooms = rooms;
    publish_navigation(&shared, lobby, 11, 1);
    let published = shared.lock().unwrap().view.clone();
    assert!(published.chat.messages().is_empty());
    assert!(published.chat.token().is_none());
    let mut returned = view;
    returned.phase = LoginPhase::ChoosingChannel;
    returned.navigation.as_mut().unwrap().snapshot.phase = NavigationPhase::Channels;
    publish_navigation(&shared, returned, 11, 2);
    assert!(shared.lock().unwrap().view.chat.messages().is_empty());
    assert!(shared.lock().unwrap().view.chat.scope().is_none());
}
#[test]
fn chat_busy_mailbox_never_blocks_ui_and_shutdown_drops_queued_text() {
    let view = chat_view_fixture();
    let token = view.chat.token().unwrap();
    let (mut c, release) = controller_with(view);
    let inbox = c.active.as_ref().unwrap().inbox.clone();
    let guard = inbox.lock().unwrap();
    let start = Instant::now();
    assert!(c
        .command(LoginCommand::ChatSend {
            token,
            text: "busy".into()
        })
        .is_err());
    assert!(start.elapsed() < Duration::from_millis(250));
    drop(guard);
    c.next_selection = u64::MAX;
    assert!(c
        .command(LoginCommand::ChatSend {
            token,
            text: "overflow".into()
        })
        .is_err());
    assert!(inbox.lock().unwrap().chat.is_none());
    release.send(()).unwrap();
}
