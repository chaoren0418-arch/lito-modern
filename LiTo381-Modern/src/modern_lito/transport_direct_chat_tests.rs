// Real TLS with production authority; fixture authentication only, no native bin/SQL/UI.
use super::super::direct_chat_protocol as direct;

#[test]
#[ignore = "requires prepared Java 17 direct-chat TLS authority"]
fn java_direct_chat_forged_identity_epoch_channel_cursor_and_sequence_fail_closed() {
    for case in 0..5 {
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernChatTestPeer",
            "direct-stale",
            "30000",
        );
        let (mut a, ca) = direct_login(&peer, 7);
        let (mut b, cb) = direct_login(&peer, 42);
        let initial = direct_boot(&mut a, ca, 0, 0);
        let mut request = initial.next_read();
        match case {
            0 => request.context.key = cb.key,
            1 => request.context.epoch = 2,
            2 => request.channel_id = 1,
            3 => request.after = 1,
            4 => {
                request = initial
                    .next_command(
                        direct::Action::Send,
                        direct_target(42, "Moon Mage"),
                        None,
                        "bad sequence".into(),
                    )
                    .unwrap();
                request.sequence = 2;
            }
            _ => unreachable!(),
        }
        let id = a.next_request_id().unwrap();
        let bytes = direct::encode_request(id, &request, 32).unwrap();
        assert!(
            a.exchange(&bytes, id, Stage::DirectChat).is_err(),
            "forgery {case}"
        );
        assert_eq!(a.application_state(), ApplicationState::Closed);
        assert!(direct_boot(&mut b, cb, 0, 0).messages.is_empty());
        b.ping().unwrap();
        a.close().unwrap();
        b.close().unwrap();
        peer.expect_successful_exit();
    }
}
fn direct_login(peer: &JavaPeer, source: u32) -> (ModernTlsConnection, SelectionContext) {
    chat_login_caps(peer, source, super::super::protocol::CAP_DIRECT_CHAT)
}
fn direct_boot(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    channel: u8,
    room: u64,
) -> direct::Reply {
    let public = chat_boot(c, context, channel, room);
    c.direct_chat(direct::Request::read(
        context,
        channel,
        room,
        public.membership,
    ))
    .unwrap()
}
fn direct_target(source: u32, name: &str) -> Option<direct::Address> {
    Some(direct::Address {
        source_id: source,
        name: name.into(),
    })
}
fn direct_send(
    c: &mut ModernTlsConnection,
    r: &direct::Reply,
    source: u32,
    name: &str,
    text: &str,
) -> direct::Reply {
    c.direct_chat(
        r.next_command(
            direct::Action::Send,
            direct_target(source, name),
            None,
            text.into(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn direct_control(
    c: &mut ModernTlsConnection,
    r: &direct::Reply,
    action: direct::Action,
    target: Option<direct::Address>,
    prefs: Option<direct::Preferences>,
) -> direct::Reply {
    c.direct_chat(
        r.next_command(action, target, prefs, String::new())
            .unwrap(),
    )
    .unwrap()
}
#[test]
#[ignore = "requires prepared Java 17 direct-chat TLS authority"]
fn java_direct_chat_cross_channel_room_private_identity_and_navigation_lifecycle() {
    let mut peer =
        JavaPeer::start_class("local.lito381.server.ModernChatTestPeer", "normal", "30000");
    let (mut a, ca) = direct_login(&peer, 7);
    let (mut b, cb) = direct_login(&peer, 42);
    assert!(a.supports_direct_chat() && b.supports_direct_chat());
    b.leave_lobby().unwrap();
    b.enter_channel(1).unwrap();
    let room = chat_create(&mut a, ca);
    let mut ra = direct_boot(&mut a, ca, 0, room);
    let mut rb = direct_boot(&mut b, cb, 1, 0);
    ra = direct_send(&mut a, &ra, 42, "  MOON Mage  ", ".de 20");
    assert_eq!(ra.status, direct::Status::Ok);
    assert_eq!(ra.messages[0].sender.display_name(), "Sun Knight(太陽神)");
    assert_eq!(ra.messages[0].recipient.display_name(), "Moon Mage(月亮)");
    assert_eq!(ra.messages[0].sender.key, ca.key);
    assert_eq!(ra.messages[0].recipient.key, cb.key);
    assert_eq!(ra.echo.target.as_ref().unwrap().name, "  MOON Mage  ");
    rb = b.direct_chat(rb.next_read()).unwrap();
    assert_eq!(rb.messages.len(), 1);
    assert_eq!(rb.messages[0].text, ".de 20");
    assert!(chat_boot(&mut a, ca, 0, room).messages.is_empty());
    assert!(chat_boot(&mut b, cb, 1, 0).messages.is_empty());
    assert_eq!(
        chat_room(&mut a, ca).joined.unwrap().members.len(),
        1,
        "private text must not invoke GM"
    );
    rb = direct_send(&mut b, &rb, 7, "SUN KNIGHT", "跨頻道收到 👋");
    assert_eq!(rb.status, direct::Status::Ok);
    ra = a.direct_chat(ra.next_read()).unwrap();
    assert_eq!(ra.messages.len(), 1);
    assert_eq!(ra.messages[0].sender.key, cb.key);
    ra = direct_send(&mut a, &ra, 7, "Sun Knight", "自己私訊只存一份");
    assert_eq!(ra.cursor, 3);
    assert_eq!(ra.messages.len(), 1);
    rb = b.direct_chat(rb.next_read()).unwrap();
    assert!(rb.messages.is_empty());
    chat_leave(&mut a, ca);
    let moved = direct_boot(&mut a, ca, 0, 0);
    assert_eq!(
        (moved.sequence, moved.cursor, moved.messages.len()),
        (2, 3, 3)
    );
    a.leave_lobby().unwrap();
    assert!(a.direct_chat(moved.next_read()).is_err());
    a.enter_channel(0).unwrap();
    let reset = direct_boot(&mut a, ca, 0, 0);
    assert_eq!((reset.sequence, reset.cursor), (2, 0));
    assert!(reset.messages.is_empty());
    a.return_characters().unwrap();
    b.return_characters().unwrap();
    let list = a.characters(42, 2).unwrap();
    let next = SelectionContext {
        key: list.characters[0].key,
        epoch: 2,
    };
    a.select_character(next).unwrap();
    a.enter_channel(0).unwrap();
    let changed = direct_boot(&mut a, next, 0, 0);
    assert_eq!(changed.owner.key, next.key);
    assert_eq!(changed.sequence, 2);
    assert!(changed.blocked.is_empty() && changed.messages.is_empty());
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit();
}
#[test]
#[ignore = "requires prepared Java 17 direct-chat TLS authority"]
fn java_direct_chat_offline_blocks_preferences_limits_and_shared_public_budget() {
    let mut peer =
        JavaPeer::start_class("local.lito381.server.ModernChatTestPeer", "normal", "30000");
    let (mut a, ca) = direct_login(&peer, 7);
    let (mut b, cb) = direct_login(&peer, 42);
    let mut ra = direct_boot(&mut a, ca, 0, 0);
    let mut rb = direct_boot(&mut b, cb, 0, 0);
    rb = direct_control(
        &mut b,
        &rb,
        direct::Action::Block,
        direct_target(7, "  SUN KNIGHT  "),
        None,
    );
    assert_eq!(rb.blocked[0].name, "sun knight");
    ra = direct_send(&mut a, &ra, 42, "Moon Mage", "拒收不洩漏原因");
    assert_eq!(ra.status, direct::Status::Unreachable);
    assert!(ra.messages.is_empty());
    rb = direct_control(
        &mut b,
        &rb,
        direct::Action::Unblock,
        direct_target(7, "Sun Knight"),
        None,
    );
    assert!(rb.blocked.is_empty());
    rb = direct_control(
        &mut b,
        &rb,
        direct::Action::Preferences,
        None,
        Some(direct::Preferences {
            whispers: false,
            world: false,
            trade: false,
        }),
    );
    ra = direct_send(&mut a, &ra, 42, "Moon Mage", "關閉私訊");
    assert_eq!(ra.status, direct::Status::Unreachable);
    rb = direct_control(
        &mut b,
        &rb,
        direct::Action::Preferences,
        None,
        Some(direct::Preferences::default()),
    );
    ra = direct_send(&mut a, &ra, 42, "Moon Mage", "開啟私訊");
    assert_eq!(ra.status, direct::Status::Ok);
    let public = chat_boot(&mut a, ca, 0, 0);
    assert_eq!(
        a.chat(public.next_send("第四筆共用額度".into()).unwrap())
            .unwrap()
            .status,
        chat::Status::Ok
    );
    ra = direct_send(&mut a, &ra, 42, "Moon Mage", "不得取得第五筆");
    assert_eq!(ra.status, direct::Status::RateLimited);
    assert_eq!(ra.sequence, 4);
    rb = b.direct_chat(rb.next_read()).unwrap();
    assert_eq!(rb.messages.len(), 1);
    assert_eq!(rb.messages[0].text, "開啟私訊");
    for i in 0..16 {
        rb = direct_control(
            &mut b,
            &rb,
            direct::Action::Block,
            direct_target(7, &format!("Offline{i}")),
            None,
        );
        assert_eq!(rb.status, direct::Status::Ok);
    }
    rb = direct_control(
        &mut b,
        &rb,
        direct::Action::Block,
        direct_target(7, "Offline17"),
        None,
    );
    assert_eq!(rb.status, direct::Status::BlockListFull);
    assert_eq!(rb.blocked.len(), 16);
    rb = direct_control(&mut b, &rb, direct::Action::ClearBlocks, None, None);
    assert!(rb.blocked.is_empty());
    let stale = rb
        .next_command(
            direct::Action::Preferences,
            None,
            Some(direct::Preferences {
                whispers: false,
                world: true,
                trade: false,
            }),
            String::new(),
        )
        .unwrap();
    rb = b.direct_chat(stale.clone()).unwrap();
    assert!(b.direct_chat(stale).is_err());
    b.ping().unwrap();
    b.leave_lobby().unwrap();
    b.enter_channel(1).unwrap();
    let reentered = direct_boot(&mut b, cb, 1, 0);
    assert_eq!(reentered.sequence, rb.sequence);
    assert_eq!(reentered.preferences, rb.preferences);
    assert_eq!(reentered.cursor, 0);
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit();
}
#[test]
#[ignore = "requires prepared Java 17 direct-chat TLS authority"]
fn java_direct_chat_non_opted_in_recipient_is_unreachable_and_old_chat_stays_usable() {
    let mut peer =
        JavaPeer::start_class("local.lito381.server.ModernChatTestPeer", "normal", "30000");
    let (mut a, ca) = direct_login(&peer, 7);
    let (mut b, cb) = chat_login(&peer, 42);
    assert!(!b.supports_direct_chat());
    let ra = direct_boot(&mut a, ca, 0, 0);
    let rb = chat_boot(&mut b, cb, 0, 0);
    assert!(b
        .direct_chat(direct::Request::read(cb, 0, 0, rb.membership))
        .is_err());
    b.ping().unwrap();
    let denied = direct_send(&mut a, &ra, 42, "Moon Mage", "未協商者不可收件");
    assert_eq!(denied.status, direct::Status::Unreachable);
    assert!(b.chat(rb.next_read()).unwrap().messages.is_empty());
    assert_eq!(
        b.chat(rb.next_send("原有一般聊天".into()).unwrap())
            .unwrap()
            .status,
        chat::Status::Ok
    );
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit();
}
#[test]
#[ignore = "requires prepared Java 17 direct-chat TLS authority"]
fn java_direct_chat_stale_same_room_token_is_rejected_by_authority_without_delivery() {
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernChatTestPeer",
        "direct-stale",
        "30000",
    );
    let (mut a, ca) = direct_login(&peer, 7);
    let (mut b, cb) = direct_login(&peer, 42);
    let room = chat_create(&mut a, ca);
    chat_join(&mut b, cb);
    let before = direct_boot(&mut b, cb, 0, room);
    chat_leave(&mut b, cb);
    chat_join(&mut b, cb);
    let fresh = direct_boot(&mut b, cb, 0, room);
    assert!(fresh.echo.membership > before.echo.membership);
    let stale = before
        .next_command(
            direct::Action::Send,
            direct_target(7, "Sun Knight"),
            None,
            "不可重導".into(),
        )
        .unwrap();
    assert!(b.direct_chat(stale).is_err());
    assert_eq!(b.application_state(), ApplicationState::Closed);
    assert!(direct_boot(&mut a, ca, 0, room).messages.is_empty());
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit();
}
#[test]
#[ignore = "requires prepared Java 17 direct-chat TLS authority"]
fn java_direct_chat_unnegotiated_wire_is_rejected_even_when_encoded_manually() {
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernChatTestPeer",
        "direct-unnegotiated",
        "30000",
    );
    let (mut a, ca) = direct_login(&peer, 7);
    let (mut b, cb) = direct_login(&peer, 42);
    assert!(!a.supports_direct_chat() && !b.supports_direct_chat());
    let token = chat_boot(&mut a, ca, 0, 0).membership;
    let request = direct::Request::read(ca, 0, 0, token);
    assert!(a.direct_chat(request.clone()).is_err());
    a.ping().unwrap();
    let id = a.next_request_id().unwrap();
    let bytes = direct::encode_request(id, &request, 32).unwrap();
    assert!(a.exchange(&bytes, id, Stage::DirectChat).is_err());
    assert_eq!(a.application_state(), ApplicationState::Closed);
    assert!(chat_boot(&mut b, cb, 0, 0).messages.is_empty());
    b.ping().unwrap();
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit();
}
