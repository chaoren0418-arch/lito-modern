// Two real SessionControllers on the production Java TLS listener. No HWND,
// production DB or resource root is opened by this navigation/chat fixture.
mod chat_controller_interop {
    use super::super::super::{
        chat_state,
        login_window::{LoginCommand, LoginController, LoginPhase},
        profile::ModernProfile,
        session::SessionController,
    };
    use super::*;
    use base64::Engine;
    fn clients(peer: &JavaPeer) -> [SessionController; 2] {
        clients_with_timeout(peer, 3000)
    }
    fn clients_with_timeout(peer: &JavaPeer, exchange_timeout_ms: u32) -> [SessionController; 2] {
        let profile=ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema":1,"profile_id":"owned-chat-controller","display_name":"Owned chat controller test",
            "shared_resource_root":"D:\\synthetic-test","endpoint":peer.endpoint.to_string(),"tls_server_name":"localhost",
            "trust":{"kind":"private_ca","certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())},
            "connect_timeout_ms":500,"exchange_timeout_ms":exchange_timeout_ms
        })).unwrap()).unwrap();
        [
            SessionController::new(profile.clone()),
            SessionController::new(profile),
        ]
    }
    #[track_caller]
    fn wait(
        cs: &mut [SessionController; 2],
        label: &str,
        test: impl Fn(&[SessionController; 2]) -> bool,
    ) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            for c in cs.iter_mut() {
                c.poll();
                assert_ne!(
                    c.snapshot().phase,
                    LoginPhase::Failed,
                    "{label}: {}",
                    c.snapshot().status
                );
            }
            if test(cs) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "{label}: {} / {}",
                cs[0].snapshot().status,
                cs[1].snapshot().status
            );
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn phase(cs: &mut [SessionController; 2], i: usize, p: LoginPhase) {
        wait(cs, "page confirmation", |cs| cs[i].snapshot().phase == p);
    }
    fn select(cs: &mut [SessionController; 2], i: usize, source: u32) -> SelectionContext {
        cs[i].command(LoginCommand::SelectSource(source)).unwrap();
        phase(cs, i, LoginPhase::CharactersReady);
        let v = cs[i].snapshot();
        let c = SelectionContext {
            key: application::CharacterKey {
                source_id: source,
                object_id: v.characters[0].object_id,
            },
            epoch: v.roster_epoch,
        };
        cs[i].command(LoginCommand::SelectCharacter(c)).unwrap();
        phase(cs, i, LoginPhase::ChoosingChannel);
        c
    }
    fn enter(
        cs: &mut [SessionController; 2],
        i: usize,
        c: SelectionContext,
        channel: u8,
        chat: bool,
    ) {
        cs[i]
            .command(LoginCommand::EnterChannel {
                context: c,
                channel_id: channel,
            })
            .unwrap();
        phase(cs, i, LoginPhase::InLobby);
        if chat {
            wait(cs, "chat membership bootstrap", |cs| {
                cs[i].snapshot().chat.token().is_some()
            });
        }
    }
    // A nonblocking UI mailbox may reject an immediate click before enqueue.
    // Only this exact pre-enqueue LoginReady error permits a new fixture click;
    // no accepted authentication or gameplay request is ever replayed.
    fn fixture_login(cs: &mut [SessionController; 2], i: usize) {
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            let result = cs[i].command(LoginCommand::Login {
                account: "fixture-user".into(),
                password: "fixture-password".into(),
            });
            match result {
                Ok(()) => {
                    assert_eq!(cs[i].snapshot().phase, LoginPhase::Authenticating);
                    return;
                }
                Err(e)
                    if e == "連線工作忙碌，請稍候再試"
                        && cs[i].snapshot().phase == LoginPhase::LoginReady =>
                {
                    assert!(
                        Instant::now() < deadline,
                        "login mailbox never became available"
                    );
                    thread::sleep(Duration::from_millis(5));
                }
                Err(e) => panic!("fixture login rejected without retry: {e}"),
            }
        }
    }
    fn login(cs: &mut [SessionController; 2], chat: bool) -> [SelectionContext; 2] {
        for c in cs.iter_mut() {
            c.command(LoginCommand::Connect).unwrap();
        }
        for i in 0..2 {
            phase(cs, i, LoginPhase::LoginReady);
            fixture_login(cs, i);
        }
        for i in 0..2 {
            phase(cs, i, LoginPhase::ChoosingSource);
        }
        let contexts = [select(cs, 0, 7), select(cs, 1, 42)];
        for i in 0..2 {
            enter(cs, i, contexts[i], 0, chat);
        }
        contexts
    }
    fn send(cs: &mut [SessionController; 2], i: usize, text: &str, delivery: chat_state::Delivery) {
        let before = cs[i].snapshot();
        let token = before.chat.token().unwrap();
        let command = if text.starts_with('.') {
            // Keep explicit raw-protocol rejection coverage independently of UI routing.
            LoginCommand::ChatSend {
                token,
                text: text.into(),
            }
        } else {
            super::super::super::login_window::chat_command(
                &before,
                super::super::super::chat_window::Intent {
                    token,
                    text: text.into(),
                    kind: super::super::super::chat_window::Kind::Message,
                },
            )
            .unwrap()
        };
        cs[i].command(command).unwrap();
        assert!(cs[i].snapshot().chat_pending);
        assert_eq!(
            cs[i].snapshot().chat.messages(),
            before.chat.messages(),
            "no optimistic delivery"
        );
        wait(cs, "chat send receipt", |cs| !cs[i].snapshot().chat_pending);
        assert_eq!(cs[i].snapshot().chat.delivery(), delivery);
    }
    #[track_caller]
    fn room(cs: &mut [SessionController; 2], i: usize, r: rooms::Request) {
        cs[i].command(LoginCommand::Rooms(r)).unwrap();
        wait(cs, "room receipt", |cs| !cs[i].snapshot().room_pending);
        let rooms::State::Ready(reply) = cs[i].snapshot().rooms else {
            panic!("room missing")
        };
        assert_eq!(
            reply.status,
            rooms::Status::Ok,
            "room action {:?}: {:?}",
            reply.common.action,
            reply.reason
        );
    }
    fn read_room(cs: &mut [SessionController; 2], i: usize, c: SelectionContext) -> rooms::Reply {
        let channel = cs[i]
            .snapshot()
            .navigation
            .unwrap()
            .snapshot
            .channel_id
            .unwrap();
        room(cs, i, rooms::Request::read(c, channel).unwrap());
        let rooms::State::Ready(reply) = cs[i].snapshot().rooms else {
            panic!()
        };
        reply.as_ref().clone()
    }
    fn command(r: &rooms::Reply, action: rooms::Action) -> rooms::Common {
        let s = r
            .joined
            .as_ref()
            .map(|r| &r.summary)
            .unwrap_or_else(|| &r.rooms[0]);
        rooms::Common {
            context: r.common.context,
            channel_id: r.common.channel_id,
            action,
            value: 0,
            room_id: s.room_id,
            expected_revision: s.revision,
        }
    }
    fn leave_room(cs: &mut [SessionController; 2], i: usize, c: SelectionContext) {
        let r = read_room(cs, i, c);
        room(
            cs,
            i,
            rooms::Request::simple(command(&r, rooms::Action::Leave)).unwrap(),
        );
        assert!(cs[i].snapshot().rooms.joined().is_none());
        wait(cs, "new lobby membership", |cs| {
            cs[i]
                .snapshot()
                .chat
                .token()
                .is_some_and(|t| t.scope.room_id == 0)
        });
    }
    fn leave_lobby(cs: &mut [SessionController; 2], i: usize, c: SelectionContext) {
        cs[i].command(LoginCommand::LeaveLobby(c)).unwrap();
        super::super::super::login_window::validate_snapshot(&cs[i].snapshot()).unwrap();
        phase(cs, i, LoginPhase::ChoosingChannel);
        assert!(cs[i].snapshot().chat.messages().is_empty());
        assert!(cs[i].snapshot().chat.token().is_none());
    }
    fn shutdown(cs: &mut [SessionController; 2], peer: &mut JavaPeer) {
        for c in cs.iter_mut() {
            c.command(LoginCommand::Shutdown).unwrap();
        }
        wait(cs, "owned controller retirement", |cs| {
            cs.iter().all(|c| c.is_quiescent())
        });
        for c in cs.iter() {
            assert!(c.snapshot().chat.messages().is_empty() && c.snapshot().chat.token().is_none());
        }
        peer.expect_successful_exit();
    }
    #[test]
    #[ignore = "requires prepared Java 17 chat application TLS peer"]
    fn java_chat_controllers_confirm_scope_room_history_denials_and_source_return() {
        let mut peer =
            JavaPeer::start_class("local.lito381.server.ModernChatTestPeer", "normal", "30000");
        let mut cs = clients(&peer);
        let [a, b] = login(&mut cs, true);
        let old = cs[0].snapshot().chat.token().unwrap();
        send(
            &mut cs,
            0,
            "初次大廳 👩\u{200d}💻",
            chat_state::Delivery::Confirmed,
        );
        wait(&mut cs, "periodic receive", |cs| {
            cs[1].snapshot().chat.messages().len() == 1
        });
        assert_eq!(
            cs[1].snapshot().chat.messages()[0].display_name(),
            "Sun Knight(太陽神)"
        );
        leave_lobby(&mut cs, 0, a);
        enter(&mut cs, 0, a, 1, true);
        assert!(cs[0]
            .command(LoginCommand::ChatSend {
                token: old,
                text: "過期頻道".into()
            })
            .is_err());
        send(&mut cs, 0, "頻道1舊歷史", chat_state::Delivery::Confirmed);
        send(&mut cs, 1, "仍在頻道0", chat_state::Delivery::Confirmed);
        assert!(cs[1]
            .snapshot()
            .chat
            .messages()
            .iter()
            .all(|m| m.text != "頻道1舊歷史"));
        leave_lobby(&mut cs, 1, b);
        enter(&mut cs, 1, b, 1, true);
        assert!(cs[1].snapshot().chat.messages().is_empty());
        send(&mut cs, 1, "頻道1新訊息", chat_state::Delivery::Confirmed);
        wait(&mut cs, "new channel receive", |cs| {
            cs[0]
                .snapshot()
                .chat
                .messages()
                .iter()
                .any(|m| m.text == "頻道1新訊息")
        });
        // Channel entry is allowed, but this fixture character's level does
        // not grant room creation in channel 1. Use its authorized channel 0.
        for (i, context) in [a, b].into_iter().enumerate() {
            leave_lobby(&mut cs, i, context);
            enter(&mut cs, i, context, 0, true);
            assert!(cs[i].snapshot().chat.messages().is_empty());
        }
        let mut common = rooms::Common::read(a, 0);
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
                        attributes: 20,
                        map_id: 564,
                        name: "Controller 私房".into(),
                    },
                    "fixture-room-password".into(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        let room_id = cs[0].snapshot().rooms.joined().unwrap().summary.room_id;
        wait(&mut cs, "private membership", |cs| {
            cs[0]
                .snapshot()
                .chat
                .token()
                .is_some_and(|t| t.scope.room_id == room_id)
        });
        assert!(cs[0].snapshot().chat.messages().is_empty());
        send(
            &mut cs,
            0,
            "尚未加入不可讀",
            chat_state::Delivery::Confirmed,
        );
        send(&mut cs, 1, "留在大廳", chat_state::Delivery::Confirmed);
        assert!(cs[1]
            .snapshot()
            .chat
            .messages()
            .iter()
            .all(|m| m.text != "尚未加入不可讀"));
        let r = read_room(&mut cs, 1, b);
        room(
            &mut cs,
            1,
            rooms::Request::join(
                command(&r, rooms::Action::Join),
                "fixture-room-password".into(),
            )
            .unwrap(),
        );
        wait(&mut cs, "guest private membership", |cs| {
            cs[1]
                .snapshot()
                .chat
                .token()
                .is_some_and(|t| t.scope.room_id == room_id)
        });
        assert!(cs[1].snapshot().chat.messages().is_empty());
        send(&mut cs, 0, "私房共同訊息", chat_state::Delivery::Confirmed);
        wait(&mut cs, "private receive", |cs| {
            cs[1]
                .snapshot()
                .chat
                .messages()
                .iter()
                .any(|m| m.text == "私房共同訊息")
        });
        let private = cs[1].snapshot().chat.token().unwrap();
        leave_room(&mut cs, 1, b);
        assert!(cs[1].snapshot().chat.messages().is_empty());
        assert!(cs[1]
            .command(LoginCommand::ChatSend {
                token: private,
                text: "不能回投私房".into()
            })
            .is_err());
        send(&mut cs, 0, "第五則限速", chat_state::Delivery::RateLimited);
        assert!(cs[0]
            .snapshot()
            .chat
            .messages()
            .iter()
            .all(|m| m.text != "第五則限速"));
        send(&mut cs, 0, ".de 20", chat_state::Delivery::CommandRequired);
        let before_gm = cs[0].snapshot();
        let gm_command = super::super::super::login_window::chat_command(
            &cs[0].snapshot(),
            super::super::super::chat_window::Intent {
                token: cs[0].snapshot().chat.token().unwrap(),
                text: ".de 20".into(),
                kind: super::super::super::chat_window::Kind::GmBots(20),
            },
        )
        .unwrap();
        assert!(matches!(gm_command, LoginCommand::GmBots { count: 20 }));
        cs[0].command(gm_command).unwrap();
        wait(&mut cs, "typed GM authority reply", |cs| {
            !cs[0].snapshot().room_pending
        });
        assert_eq!(
            cs[0].snapshot().status,
            "GM 授權／來源讀取暫不可用，保留已確認的房間。"
        );
        assert_eq!(
            cs[0].snapshot().chat.messages(),
            before_gm.chat.messages(),
            "GM text must not be broadcast or consume chat history"
        );
        let r = read_room(&mut cs, 0, a);
        assert_eq!(r.joined.unwrap().members.len(), 1);
        leave_lobby(&mut cs, 1, b);
        cs[1].command(LoginCommand::ReturnCharacters(b)).unwrap();
        phase(&mut cs, 1, LoginPhase::CharactersReady);
        leave_room(&mut cs, 0, a);
        leave_lobby(&mut cs, 0, a);
        cs[0].command(LoginCommand::ReturnCharacters(a)).unwrap();
        phase(&mut cs, 0, LoginPhase::CharactersReady);
        let changed = select(&mut cs, 0, 42);
        assert_ne!(changed, a);
        enter(&mut cs, 0, changed, 2, true);
        assert!(cs[0].snapshot().chat.messages().is_empty());
        assert!(cs[0]
            .command(LoginCommand::ChatSend {
                token: old,
                text: "舊來源".into()
            })
            .is_err());
        send(
            &mut cs,
            0,
            "換來源不重置限速",
            chat_state::Delivery::RateLimited,
        );
        shutdown(&mut cs, &mut peer);
    }
    #[test]
    #[ignore = "requires prepared Java 17 chat application TLS peer"]
    fn java_chat_controllers_without_capability_keep_navigation_and_shutdown_working() {
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernChatTestPeer",
            "unnegotiated",
            "30000",
        );
        let mut cs = clients(&peer);
        let contexts = login(&mut cs, false);
        for i in 0..2 {
            assert!(!cs[i].snapshot().chat.supported());
            let token = chat_state::Token {
                scope: chat_state::Scope {
                    context: contexts[i],
                    channel_id: 0,
                    room_id: 0,
                },
                membership: 1,
            };
            assert!(cs[i]
                .command(LoginCommand::ChatSend {
                    token,
                    text: "不可送".into()
                })
                .is_err());
            leave_lobby(&mut cs, i, contexts[i]);
            enter(&mut cs, i, contexts[i], 3, false);
        }
        shutdown(&mut cs, &mut peer);
    }
    include!("transport_chat_fault_tests.rs");
    include!("transport_direct_chat_controller_tests.rs");
    include!("transport_filtered_chat_controller_tests.rs");
    include!("transport_filtered_public_fault_tests.rs");
    include!("transport_matchmaking_controller_tests.rs");
}
