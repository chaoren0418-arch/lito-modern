// Transport-level private mutation evidence. No private player controller/UI
// is claimed here; the peer proves the actual authorized commit before loss.
fn committed_direct_fault(mode: &str, cancel: bool) {
    committed_direct_profile_fault(mode,cancel,false)
}
fn committed_direct_profile_fault(mode: &str, cancel: bool, filtered:bool) {
    use super::super::direct_chat_state as state;
    let mut peer = JavaPeer::start_class(
        if filtered {"local.lito381.server.ModernFilteredChatFaultTestPeer"}else{"local.lito381.server.ModernDirectChatFaultTestPeer"},
        mode,
        "15000",
    );
    let (mut c, context) = if filtered {filtered_login(&peer,7)}else{direct_login(&peer, 7)};
    c.set_exchange_timeout(Duration::from_millis(if cancel { 10000 } else { 1200 }))
        .unwrap();
    let room = if mode.ends_with("-room") {
        chat_create(&mut c, context)
    } else {
        0
    };
    let initial = if filtered {let public=filtered_boot(&mut c,context,0,room);filtered_private(&mut c,&public)}else{direct_boot(&mut c, context, 0, room)};
    let token = state::Token {
        enrollment: state::Enrollment {
            context,
            channel_id: 0,
            generation: 1,
        },
        room_id: room,
        membership: initial.echo.membership,
    };
    let mut view = state::State::default();
    view.bind(true, Some(token.enrollment), Some(token))
        .unwrap();
    view.receive(token, initial).unwrap();
    let request = view
        .command(
            token,
            direct::Action::Send,
            direct_target(7, "Sun Knight"),
            None,
            "先前已確認的私訊".into(),
        )
        .unwrap();
    assert!(view.messages().is_empty());
    let confirmed = c.direct_chat(request).unwrap();
    view.receive(token, confirmed.clone()).unwrap();
    assert_eq!(confirmed.status, direct::Status::Ok);
    assert_eq!((confirmed.sequence, confirmed.cursor), (1, 1));
    let request = if mode.starts_with("prefs-") {
        view.command(
            token,
            direct::Action::Preferences,
            None,
            Some(direct::Preferences {
                whispers: false,
                world: true,
                trade: false,
            }),
            String::new(),
        )
        .unwrap()
    } else if mode.starts_with("block-") {
        view.command(
            token,
            direct::Action::Block,
            direct_target(42, "離線角色"),
            None,
            String::new(),
        )
        .unwrap()
    } else {
        view.command(
            token,
            direct::Action::Send,
            direct_target(7, "Sun Knight"),
            None,
            "權威已接受但確認遺失🙂".into(),
        )
        .unwrap()
    };
    let action = request.action;
    let before = view.clone();
    let cancelled = c.cancellation.clone();
    let worker = thread::spawn(move || {
        let result = c.direct_chat(request);
        assert_eq!(c.application_state(), ApplicationState::Closed);
        assert!(c.principal().is_none());
        assert!(c.navigation().is_none());
        assert!(c.room_snapshot().is_none());
        assert!(
            c.ping().is_err(),
            "uncertain connection cannot accept another command"
        );
        c.close().unwrap();
        result
    });
    assert_eq!(
        peer.read_bounded_line("COMMITTED DIRECT").trim_end(),
        "COMMITTED DIRECT 2"
    );
    assert_eq!(
        view, before,
        "server commit without ACK must not mutate private presentation"
    );
    let closing = Instant::now();
    if cancel {
        cancelled.cancel();
    }
    let deadline = Instant::now() + Duration::from_secs(4);
    while !worker.is_finished() {
        assert!(Instant::now() < deadline, "private mutation did not retire");
        thread::sleep(Duration::from_millis(5));
    }
    let result = worker.join().unwrap();
    if cancel {
        assert!(closing.elapsed() < Duration::from_secs(2));
        assert!(matches!(result, Err(TransportError::Cancelled)));
    } else {
        assert!(matches!(
            result,
            Err(TransportError::DeadlineExceeded(Stage::DirectChat))
        ));
    }
    // Last known authority is unchanged; no successful result or invented
    // rollback is manufactured for the mutation whose ACK was never received.
    assert_eq!(
        (
            confirmed.sequence,
            confirmed.cursor,
            confirmed.messages.len()
        ),
        (1, 1, 1)
    );
    assert_eq!(confirmed.preferences, direct::Preferences::default());
    assert!(confirmed.blocked.is_empty());
    view.retire_unknown(action).unwrap();
    assert_eq!(view.outcome(), state::Outcome::Unknown(action));
    assert!(
        view.messages().is_empty()
            && view.blocked().is_empty()
            && view.preferences().is_none()
            && view.owner().is_none()
    );
    assert!(view.token().is_none() && view.read().is_none());
    assert!(view.receive(token, confirmed).is_err());
    peer.expect_successful_exit(); // EOF/no next command/no reconnect/zero private and public owners.
}
#[test]
#[ignore = "actual private TLS commit then dropped reply; no controller/UI or production SQL"]
fn java_direct_chat_committed_lobby_reply_loss_aborts_without_replay() {
    committed_direct_fault("silent-lobby", false);
}
#[test]
#[ignore = "actual private TLS commit then dropped reply; no controller/UI or production SQL"]
fn java_direct_chat_committed_room_reply_loss_aborts_without_replay() {
    committed_direct_fault("silent-room", false);
}
#[test]
#[ignore = "actual private TLS commit then cancellation; no controller/UI or production SQL"]
fn java_direct_chat_committed_lobby_cancellation_retires_before_deadline() {
    committed_direct_fault("cancel-lobby", true);
}
#[test]
#[ignore = "actual private TLS commit then cancellation; no controller/UI or production SQL"]
fn java_direct_chat_committed_room_cancellation_retires_before_deadline() {
    committed_direct_fault("cancel-room", true);
}
#[test]
#[ignore = "actual preferences commit then dropped reply; no controller/UI or production SQL"]
fn java_direct_chat_committed_preferences_reply_loss_is_not_reported_successful() {
    committed_direct_fault("prefs-lobby", false);
}
#[test]
#[ignore = "actual block commit then cancellation; no controller/UI or production SQL"]
fn java_direct_chat_committed_block_cancellation_is_not_reported_successful() {
    committed_direct_fault("block-room", true);
}
