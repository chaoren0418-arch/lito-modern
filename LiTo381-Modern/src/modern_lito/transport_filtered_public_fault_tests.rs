// Included in chat_controller_interop: production UI intent -> controller -> TLS
// -> real Java chat commit, with only the response withheld by a test-owned peer.
fn committed_filtered_public_fault(in_room: bool, cancel: bool, forged:bool) {
    let mode = if forged {"forged-lobby"}else{match (in_room, cancel) {
        (false, false) => "silent-lobby",
        (true, false) => "silent-room",
        (false, true) => "cancel-lobby",
        (true, true) => "cancel-room",
    }};
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernFilteredPublicFaultTestPeer",
        mode,
        "15000",
    );
    let mut cs = clients_with_timeout(&peer, if cancel { 10000 } else { 1200 });
    cs[0].command(LoginCommand::Connect).unwrap();
    phase(&mut cs, 0, LoginPhase::LoginReady);
    fixture_login(&mut cs, 0);
    phase(&mut cs, 0, LoginPhase::ChoosingSource);
    let context = select(&mut cs, 0, 7);
    wait(&mut cs, "initial record reply", |cs| {
        !cs[0].snapshot().record_pending && cs[0].snapshot().record.context() == Some(context)
    });
    enter(&mut cs, 0, context, 0, true);
    if in_room {
        let mut common = rooms::Common::read(context, 0);
        common.action = rooms::Action::Create;
        room(
            &mut cs,
            0,
            rooms::Request::draft(
                common,
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 4,
                        mode: 1,
                        attributes: 16,
                        map_id: 564,
                        name: "Chat fault room".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        let id = cs[0].snapshot().rooms.joined().unwrap().summary.room_id;
        wait(&mut cs, "room chat admission", |cs| {
            cs[0]
                .snapshot()
                .chat
                .token()
                .is_some_and(|t| t.scope.room_id == id)
        });
    }
    send(
        &mut cs,
        0,
        "先前已確認的訊息",
        chat_state::Delivery::Confirmed,
    );
    let before = cs[0].snapshot();
    assert_eq!(before.chat.filtered_messages().len(), 1);
    let token = before.chat.token().unwrap();
    let text = "伺服器已接受但回覆遺失🙂X";
    let command = super::super::super::login_window::chat_command(
        &before,
        super::super::super::chat_window::Intent {
            token,
            text: text.into(),
            kind: super::super::super::chat_window::Kind::Message,
        },
    )
    .unwrap();
    cs[0].command(command).unwrap();
    assert!(cs[0].snapshot().chat_pending);
    assert_eq!(cs[0].snapshot().chat.filtered_messages(), before.chat.filtered_messages());
    // Not a delay or a fake client state: the peer reread both committed messages
    // from real navigation/chat authority before emitting this out-of-band marker.
    assert_eq!(
        peer.read_bounded_line("COMMITTED FILTERED").trim_end(),
        "COMMITTED FILTERED 2"
    );
    cs[0].poll();
    if !forged {assert_eq!(cs[0].snapshot().phase,LoginPhase::InLobby);assert!(cs[0].snapshot().chat_pending);assert_eq!(cs[0].snapshot().chat.filtered_messages(),before.chat.filtered_messages());}
    assert!(cs[0]
        .command(LoginCommand::ChatSend {
            token,
            text: text.into()
        })
        .is_err());
    assert!(cs[0].command(LoginCommand::LeaveLobby(context)).is_err());
    let closing = Instant::now();
    if cancel {
        cs[0].command(LoginCommand::Shutdown).unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(4);
    let expected = if cancel {
        LoginPhase::Disconnected
    } else {
        LoginPhase::Failed
    };
    loop {
        cs[0].poll();
        let view = cs[0].snapshot();
        assert!(
            view.chat.filtered_messages().iter().all(|m| m.text != text),
            "unacknowledged text must never appear as delivered"
        );
        if view.phase == expected && cs[0].is_quiescent() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "chat fault did not retire: {}",
            view.status
        );
        thread::sleep(Duration::from_millis(5));
    }
    let terminal = cs[0].snapshot();
    if cancel {
        assert!(
            closing.elapsed() < Duration::from_secs(2),
            "Shutdown must cancel well before the 10s exchange timeout"
        );
    } else {
        assert!(
            terminal.status.contains("聊天送出結果尚未確認"),
            "{}",
            terminal.status
        );
        assert!(
            terminal.status.contains("不自動重送"),
            "{}",
            terminal.status
        );
    }
    assert!(!terminal.chat_pending);
    assert!(terminal.chat.filtered_messages().is_empty() && terminal.chat.token().is_none());
    assert_ne!(terminal.chat.delivery(), chat_state::Delivery::Confirmed);
    assert!(terminal.navigation.is_none() && terminal.rooms.joined().is_none());
    assert!(terminal.characters.is_empty() && terminal.sources.is_empty());
    assert!(cs[0]
        .command(LoginCommand::ChatSend {
            token,
            text: text.into()
        })
        .is_err());
    peer.expect_successful_exit(); // Independently proves EOF, no replay/reconnect, zero server owners.
    cs[0].poll();
    assert_eq!(
        cs[0].snapshot().phase,
        expected,
        "a late worker result must not reopen the page"
    );
}

#[test]
#[ignore = "real TLS chat commit with dropped reply; no HWND or production SQL"]
fn java_filtered_chat_controller_committed_lobby_reply_loss_never_replays_or_reports_delivery() {
    committed_filtered_public_fault(false,false,false);
}
#[test]
#[ignore = "real TLS chat commit with dropped reply; no HWND or production SQL"]
fn java_filtered_chat_controller_committed_room_reply_loss_retires_membership_without_replay() {
    committed_filtered_public_fault(true,false,false);
}
#[test]
#[ignore = "real TLS chat commit then Shutdown before ACK; no HWND or production SQL"]
fn java_filtered_chat_controller_committed_lobby_shutdown_cancels_before_exchange_timeout() {
    committed_filtered_public_fault(false,true,false);
}
#[test]
#[ignore = "real TLS chat commit then Shutdown before ACK; no HWND or production SQL"]
fn java_filtered_chat_controller_committed_room_shutdown_retires_room_and_chat_owners() {
    committed_filtered_public_fault(true,true,false);
}


#[test]
#[ignore="real committed public SEND with a valid UTF-8 but forged receipt"]
fn java_filtered_chat_controller_committed_forged_receipt_never_reports_delivery(){committed_filtered_public_fault(false,false,true);}
