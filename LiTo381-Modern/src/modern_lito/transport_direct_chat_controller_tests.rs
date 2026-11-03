// Included in chat_controller_interop. Real controller + production intent
// adapter + TLS authority. No HWND/resource pack or production SQL is opened.
use super::super::super::{
    direct_chat_protocol as dc, direct_chat_state as ds, direct_chat_window as dw,
};
fn private_ready(cs: &mut [SessionController; 2], i: usize) {
    wait(cs, "private admission", |cs| {
        cs[i].snapshot().direct_chat.token().is_some()
    });
}
fn private_intent(
    cs: &[SessionController; 2],
    i: usize,
    action: dc::Action,
    target: Option<dc::Address>,
    preferences: Option<dc::Preferences>,
    text: &str,
) -> dw::Intent {
    dw::Intent {
        token: cs[i].snapshot().direct_chat.token().unwrap(),
        action,
        target,
        preferences,
        text: text.into(),
    }
}
fn private_dispatch(cs: &mut [SessionController; 2], i: usize, intent: dw::Intent) {
    let before = cs[i].snapshot();
    let command = super::super::super::login_window::direct_chat_command(&before, intent).unwrap();
    cs[i].command(command).unwrap();
    assert!(cs[i].snapshot().direct_chat_pending);
    assert_eq!(
        cs[i].snapshot().direct_chat,
        before.direct_chat,
        "no optimistic private mutation"
    );
}
fn private_control(
    cs: &mut [SessionController; 2],
    i: usize,
    action: dc::Action,
    target: Option<dc::Address>,
    preferences: Option<dc::Preferences>,
    text: &str,
    outcome: ds::Outcome,
) {
    let intent = private_intent(cs, i, action, target, preferences, text);
    private_dispatch(cs, i, intent);
    wait(cs, "private receipt", |cs| {
        !cs[i].snapshot().direct_chat_pending
    });
    assert_eq!(cs[i].snapshot().direct_chat.outcome(), outcome);
}
fn private_target(source_id: u32, name: &str) -> Option<dc::Address> {
    Some(dc::Address {
        source_id,
        name: name.into(),
    })
}
fn private_create_room(cs: &mut [SessionController; 2], i: usize, context: SelectionContext) {
    let mut common = rooms::Common::read(context, 0);
    common.action = rooms::Action::Create;
    room(
        cs,
        i,
        rooms::Request::draft(
            common,
            rooms::Draft::new(
                rooms::PublicDraft {
                    capacity: 4,
                    mode: 1,
                    attributes: 16,
                    map_id: 564,
                    name: "Private controller room".into(),
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    let id = cs[i].snapshot().rooms.joined().unwrap().summary.room_id;
    wait(cs, "private room admission", |cs| {
        cs[i]
            .snapshot()
            .direct_chat
            .token()
            .is_some_and(|t| t.room_id == id)
    });
}
#[test]
#[ignore = "actual two controllers/intent adapter/Java TLS; no HWND or production SQL"]
fn java_direct_chat_controllers_qualified_delivery_controls_room_and_navigation() {
    let mut peer =
        JavaPeer::start_class("local.lito381.server.ModernChatTestPeer", "normal", "30000");
    let mut cs = clients(&peer);
    let [a, _b] = login(&mut cs, true);
    for i in 0..2 {
        private_ready(&mut cs, i);
    }
    let original = cs[0].snapshot().direct_chat.token().unwrap();
    private_control(
        &mut cs,
        0,
        dc::Action::Send,
        private_target(42, "Moon Mage"),
        None,
        ".de 20",
        ds::Outcome::Confirmed(dc::Action::Send),
    );
    wait(&mut cs, "qualified private recipient", |cs| {
        cs[1].snapshot().direct_chat.messages().len() == 1
    });
    let row = cs[1].snapshot().direct_chat.messages()[0].clone();
    assert_eq!(row.sender.display_name(), "Sun Knight(太陽神)");
    assert_eq!(row.recipient.display_name(), "Moon Mage(月亮)");
    assert_eq!(row.text, ".de 20");
    assert!(cs.iter().all(|c| c.snapshot().chat.messages().is_empty()));
    private_control(
        &mut cs,
        1,
        dc::Action::Block,
        private_target(7, "  SUN KNIGHT  "),
        None,
        "",
        ds::Outcome::Confirmed(dc::Action::Block),
    );
    assert_eq!(cs[1].snapshot().direct_chat.blocked()[0].name, "sun knight");
    private_control(
        &mut cs,
        0,
        dc::Action::Send,
        private_target(42, "Moon Mage"),
        None,
        "拒收",
        ds::Outcome::Unreachable,
    );
    private_control(
        &mut cs,
        1,
        dc::Action::Unblock,
        private_target(7, "sun knight"),
        None,
        "",
        ds::Outcome::Confirmed(dc::Action::Unblock),
    );
    private_control(
        &mut cs,
        1,
        dc::Action::Preferences,
        None,
        Some(dc::Preferences {
            whispers: false,
            ..Default::default()
        }),
        "",
        ds::Outcome::Confirmed(dc::Action::Preferences),
    );
    private_control(
        &mut cs,
        0,
        dc::Action::Send,
        private_target(42, "Moon Mage"),
        None,
        "不接收",
        ds::Outcome::Unreachable,
    );
    private_control(
        &mut cs,
        1,
        dc::Action::Preferences,
        None,
        Some(dc::Preferences::default()),
        "",
        ds::Outcome::Confirmed(dc::Action::Preferences),
    );
    private_create_room(&mut cs, 0, a);
    let in_room = cs[0].snapshot().direct_chat.token().unwrap();
    assert_eq!(in_room.enrollment, original.enrollment);
    assert_ne!(in_room, original);
    assert_eq!(cs[0].snapshot().direct_chat.messages().len(), 1);
    let mut stale = private_intent(
        &cs,
        0,
        dc::Action::Send,
        private_target(42, "Moon Mage"),
        None,
        "舊房間",
    );
    stale.token = original;
    assert!(
        super::super::super::login_window::direct_chat_command(&cs[0].snapshot(), stale).is_err()
    );
    private_control(
        &mut cs,
        0,
        dc::Action::Send,
        private_target(42, "Moon Mage"),
        None,
        "戰鬥室對大廳",
        ds::Outcome::Confirmed(dc::Action::Send),
    );
    wait(&mut cs, "cross-room private recipient", |cs| {
        cs[1].snapshot().direct_chat.messages().len() == 2
    });
    private_control(
        &mut cs,
        1,
        dc::Action::Block,
        private_target(7, "offline"),
        None,
        "",
        ds::Outcome::Confirmed(dc::Action::Block),
    );
    private_control(
        &mut cs,
        1,
        dc::Action::ClearBlocks,
        None,
        None,
        "",
        ds::Outcome::Confirmed(dc::Action::ClearBlocks),
    );
    assert!(cs[1].snapshot().direct_chat.blocked().is_empty());
    leave_room(&mut cs, 0, a);
    private_ready(&mut cs, 0);
    assert_eq!(cs[0].snapshot().direct_chat.messages().len(), 2);
    leave_lobby(&mut cs, 0, a);
    assert!(cs[0].snapshot().direct_chat.owner().is_none());
    enter(&mut cs, 0, a, 3, true);
    private_ready(&mut cs, 0);
    let reentered = cs[0].snapshot().direct_chat.token().unwrap();
    assert_ne!(
        reentered.enrollment.generation,
        original.enrollment.generation
    );
    assert!(cs[0].snapshot().direct_chat.messages().is_empty());
    assert_eq!(
        cs[0]
            .snapshot()
            .direct_chat
            .command(
                reentered,
                dc::Action::ClearBlocks,
                None,
                None,
                String::new()
            )
            .unwrap()
            .sequence,
        5
    );
    leave_lobby(&mut cs, 0, a);
    cs[0].command(LoginCommand::ReturnCharacters(a)).unwrap();
    phase(&mut cs, 0, LoginPhase::CharactersReady);
    assert!(cs[0].snapshot().direct_chat.owner().is_none());
    cs[1].command(LoginCommand::Disconnect).unwrap();
    wait(&mut cs, "second owner retired", |cs| cs[1].is_quiescent());
    let switched = select(&mut cs, 0, 42);
    enter(&mut cs, 0, switched, 0, true);
    private_ready(&mut cs, 0);
    assert_eq!(
        cs[0].snapshot().direct_chat.owner().unwrap().key.source_id,
        42
    );
    assert!(cs[0].snapshot().direct_chat.messages().is_empty());
    shutdown(&mut cs, &mut peer);
    assert!(cs
        .iter()
        .all(|c| c.snapshot().direct_chat.owner().is_none()));
}
fn private_controller_fault(mode: &str, cancel: bool) {
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernDirectChatFaultTestPeer",
        mode,
        "15000",
    );
    let mut cs = clients_with_timeout(&peer, if cancel { 10000 } else { 1200 });
    cs[0].command(LoginCommand::Connect).unwrap();
    phase(&mut cs, 0, LoginPhase::LoginReady);
    fixture_login(&mut cs, 0);
    phase(&mut cs, 0, LoginPhase::ChoosingSource);
    let context = select(&mut cs, 0, 7);
    enter(&mut cs, 0, context, 0, true);
    private_ready(&mut cs, 0);
    if mode.ends_with("-room") {
        private_create_room(&mut cs, 0, context);
    }
    private_control(
        &mut cs,
        0,
        dc::Action::Send,
        private_target(7, "Sun Knight"),
        None,
        "先前已確認的私訊",
        ds::Outcome::Confirmed(dc::Action::Send),
    );
    let before = cs[0].snapshot();
    let (action, target, prefs, text) = if mode.starts_with("prefs-") {
        (
            dc::Action::Preferences,
            None,
            Some(dc::Preferences {
                whispers: false,
                ..Default::default()
            }),
            "",
        )
    } else if mode.starts_with("block-") {
        (dc::Action::Block, private_target(42, "離線角色"), None, "")
    } else {
        (
            dc::Action::Send,
            private_target(7, "Sun Knight"),
            None,
            "权威已接受但回覆遺失",
        )
    };
    let intent = private_intent(&cs, 0, action, target, prefs, text);
    let token = intent.token;
    private_dispatch(&mut cs, 0, intent);
    assert_eq!(
        peer.read_bounded_line("COMMITTED DIRECT").trim_end(),
        "COMMITTED DIRECT 2"
    );
    cs[0].poll();
    assert!(cs[0].snapshot().direct_chat_pending);
    assert_eq!(cs[0].snapshot().direct_chat, before.direct_chat);
    assert!(cs[0].command(LoginCommand::LeaveLobby(context)).is_err());
    assert!(cs[0]
        .command(LoginCommand::ChatSend {
            token: before.chat.token().unwrap(),
            text: "不能插入廣播".into()
        })
        .is_err());
    let closing = Instant::now();
    if cancel {
        cs[0].command(LoginCommand::Shutdown).unwrap();
    }
    let expected = if cancel {
        LoginPhase::Disconnected
    } else {
        LoginPhase::Failed
    };
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        cs[0].poll();
        let v = cs[0].snapshot();
        assert!(v.direct_chat.messages().len() <= 1);
        if v.phase == expected && cs[0].is_quiescent() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "private retirement: {}",
            v.status
        );
        thread::sleep(Duration::from_millis(5));
    }
    let v = cs[0].snapshot();
    if cancel {
        assert!(closing.elapsed() < Duration::from_secs(2));
    } else {
        assert!(
            v.status.contains("私訊操作結果尚未確認") && v.status.contains("不自動重送"),
            "{}",
            v.status
        );
    }
    assert!(!v.direct_chat_pending);
    assert!(
        v.direct_chat.owner().is_none()
            && v.direct_chat.messages().is_empty()
            && v.direct_chat.blocked().is_empty()
            && v.direct_chat.preferences().is_none()
            && v.direct_chat.token().is_none()
    );
    assert!(
        v.sources.is_empty()
            && v.characters.is_empty()
            && v.navigation.is_none()
            && v.rooms.joined().is_none()
    );
    assert!(cs[0]
        .command(LoginCommand::DirectChat {
            token,
            action: dc::Action::ClearBlocks,
            target: None,
            preferences: None,
            text: String::new()
        })
        .is_err());
    peer.expect_successful_exit();
    cs[0].poll();
    assert_eq!(cs[0].snapshot().phase, expected);
}
#[test]
#[ignore = "real private controller TLS commit then lost ACK"]
fn java_direct_chat_controller_lobby_reply_loss() {
    private_controller_fault("silent-lobby", false);
}
#[test]
#[ignore = "real private controller TLS commit then lost ACK"]
fn java_direct_chat_controller_room_reply_loss() {
    private_controller_fault("silent-room", false);
}
#[test]
#[ignore = "real private controller TLS commit then Shutdown"]
fn java_direct_chat_controller_lobby_shutdown() {
    private_controller_fault("cancel-lobby", true);
}
#[test]
#[ignore = "real private controller TLS commit then Shutdown"]
fn java_direct_chat_controller_room_shutdown() {
    private_controller_fault("cancel-room", true);
}
#[test]
#[ignore = "real private controller preference commit then lost ACK"]
fn java_direct_chat_controller_preference_reply_loss() {
    private_controller_fault("prefs-lobby", false);
}
#[test]
#[ignore = "real private controller block commit then Shutdown"]
fn java_direct_chat_controller_block_shutdown() {
    private_controller_fault("block-room", true);
}
