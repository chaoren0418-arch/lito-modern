// Deterministic mailbox/publication checks, not live UI or TLS evidence.
fn direct_view_fixture() -> LoginSnapshot {
    let mut v = chat_view_fixture();
    let p = v.chat.token().unwrap();
    let e = direct_state::Enrollment {
        context: p.scope.context,
        channel_id: p.scope.channel_id,
        generation: 9,
    };
    bind_direct(&mut v.direct_chat, true, Some(e), Some(p)).unwrap();
    let (t, echo) = v.direct_chat.read().unwrap();
    v.direct_chat
        .receive(
            t,
            direct::Reply {
                echo,
                status: direct::Status::Ok,
                sequence: 0,
                cursor: 0,
                available: true,
                truncated: false,
                owner: direct::Peer {
                    key: e.context.key,
                    character_name: "私訊角色".into(),
                    source_name: "來源".into(),
                },
                preferences: direct::Preferences::default(),
                blocked: vec![],
                messages: vec![],
            },
        )
        .unwrap();
    v
}
#[test]
fn private_command_single_flight_and_background_read_cannot_confirm_mutation() {
    let mut v = direct_view_fixture();
    let token = v.direct_chat.token().unwrap();
    let (mut c, release) = controller_with(v.clone());
    let command = || LoginCommand::DirectChat {
        token,
        action: direct::Action::Preferences,
        target: None,
        preferences: Some(direct::Preferences {
            whispers: false,
            ..Default::default()
        }),
        text: String::new(),
    };
    c.command(command()).unwrap();
    let active = c.active.as_ref().unwrap();
    let (captured, request, epoch) = active.inbox.lock().unwrap().direct_chat.take().unwrap();
    assert_eq!(captured, token);
    assert_eq!(active.pending_direct_chat, Some(epoch));
    assert!(c.snapshot().direct_chat_pending);
    assert_eq!(c.snapshot().direct_chat, v.direct_chat);
    assert!(c.command(command()).is_err());
    assert!(c
        .command(LoginCommand::ChatSend {
            token: v.chat.token().unwrap(),
            text: "廣播".into()
        })
        .is_err());
    assert!(c
        .command(LoginCommand::LeaveLobby(token.enrollment.context))
        .is_err());
    assert!(c.command(LoginCommand::ReadSkills).is_err());
    publish_navigation(
        &c.active.as_ref().unwrap().published,
        v.clone(),
        token.enrollment.context.epoch,
        epoch - 1,
    );
    c.poll();
    assert!(c.snapshot().direct_chat_pending);
    let prefs = request.preferences.unwrap();
    v.direct_chat
        .receive(
            token,
            direct::Reply {
                echo: request,
                status: direct::Status::Ok,
                sequence: 1,
                cursor: 0,
                available: true,
                truncated: false,
                owner: v.direct_chat.owner().unwrap().clone(),
                preferences: prefs,
                blocked: vec![],
                messages: vec![],
            },
        )
        .unwrap();
    publish_navigation(
        &c.active.as_ref().unwrap().published,
        v,
        token.enrollment.context.epoch,
        epoch,
    );
    c.poll();
    assert!(!c.snapshot().direct_chat_pending);
    assert_eq!(c.snapshot().direct_chat.preferences(), Some(prefs));
    c.command(LoginCommand::Shutdown).unwrap();
    assert!(
        c.snapshot().direct_chat.messages().is_empty()
            && c.snapshot().direct_chat.token().is_none()
    );
    release.send(()).unwrap();
}
#[test]
fn publication_retires_private_admission_on_room_change_and_history_on_channel_leave() {
    let v = direct_view_fixture();
    let t = v.direct_chat.token().unwrap();
    let shared = Mutex::new(Published {
        view: v.clone(),
        source_epoch: 1,
        navigation_epoch: 1,
    });
    let mut changed = v.clone();
    changed.rooms = rooms::State::NotRequested;
    publish_navigation(&shared, changed, 1, 2);
    let published = shared.lock().unwrap().view.clone();
    assert!(published.direct_chat.token().is_none());
    assert_eq!(published.direct_chat.owner(), v.direct_chat.owner());
    assert_eq!(published.direct_chat.enrollment(), Some(t.enrollment));
    let mut left = v;
    left.phase = LoginPhase::ChoosingChannel;
    left.navigation.as_mut().unwrap().snapshot.phase = NavigationPhase::Channels;
    publish_navigation(&shared, left, 1, 3);
    assert!(shared.lock().unwrap().view.direct_chat.owner().is_none());
    assert!(shared
        .lock()
        .unwrap()
        .view
        .direct_chat
        .enrollment()
        .is_none());
}
#[test]
fn private_ui_adapter_uses_exact_qualified_target_and_rejects_old_token_or_hidden_flags() {
    let mut v = direct_view_fixture();
    let token = v.direct_chat.token().unwrap();
    v.sources.push(SourceChoice {
        id: 42,
        label: "月亮".into(),
    });
    let intent = || super::super::direct_chat_window::Intent {
        token,
        action: direct::Action::Send,
        target: Some(direct::Address {
            source_id: 42,
            name: "  3333  ".into(),
        }),
        preferences: None,
        text: ".de 20".into(),
    };
    let c = super::super::login_window::direct_chat_command(&v, intent()).unwrap();
    assert!(matches!(
        c,
        LoginCommand::DirectChat {
            action: direct::Action::Send,
            ..
        }
    ));
    let mut old = intent();
    old.token.enrollment.generation += 1;
    assert!(super::super::login_window::direct_chat_command(&v, old).is_err());
    let mut hidden = intent();
    hidden.action = direct::Action::Preferences;
    hidden.target = None;
    hidden.text.clear();
    hidden.preferences = Some(direct::Preferences {
        whispers: false,
        world: false,
        trade: true,
    });
    assert!(super::super::login_window::direct_chat_command(&v, hidden).is_err());
    v.chat_pending = true;
    assert!(super::super::login_window::direct_chat_command(&v, intent()).is_err());
}
