// Included in transport::tests: these launch only an owned ephemeral Java TLS
// peer. No production DB, client resources, native bin or player UI is exercised.
fn chat_login(peer: &JavaPeer, source: u32) -> (ModernTlsConnection, SelectionContext) {
    chat_login_caps(peer,source,0)
}
fn chat_login_caps(peer: &JavaPeer, source: u32, extra:u32) -> (ModernTlsConnection, SelectionContext) {
    let mut hello = Hello::current(100);
    hello.supported_capabilities |= super::super::protocol::CAP_CHAT | extra;
    let mut c = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(&private_root()),
        hello,
        integration_limits(),
        Cancellation::new(),
    )
    .unwrap();
    c.set_exchange_timeout(Duration::from_secs(3)).unwrap();
    c.authenticate(fixture_credentials("fixture-password"))
        .unwrap();
    let roster = c.characters(source, u64::MAX).unwrap();
    let context = SelectionContext {
        key: roster.characters[0].key,
        epoch: u64::MAX,
    };
    assert_eq!(
        c.select_character(context).unwrap().status,
        NavigationStatus::Ok
    );
    assert_eq!(c.enter_channel(0).unwrap().status, NavigationStatus::Ok);
    (c, context)
}
fn chat_boot(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    channel: u8,
    room: u64,
) -> chat::Reply {
    c.chat(chat::Request::read(context, channel, room)).unwrap()
}
fn chat_room(c: &mut ModernTlsConnection, context: SelectionContext) -> rooms::Reply {
    c.rooms(rooms::Request::read(context, 0).unwrap()).unwrap()
}
fn chat_room_command(r: &rooms::Reply, action: rooms::Action) -> rooms::Common {
    let s = r.joined.as_ref().map(|r| &r.summary).unwrap_or(&r.rooms[0]);
    rooms::Common {
        context: r.common.context,
        channel_id: 0,
        action,
        value: 0,
        room_id: s.room_id,
        expected_revision: s.revision,
    }
}
fn chat_create(c: &mut ModernTlsConnection, context: SelectionContext) -> u64 {
    let mut common = rooms::Common::read(context, 0);
    common.action = rooms::Action::Create;
    let public = rooms::PublicDraft {
        capacity: 4,
        mode: 1,
        attributes: 20,
        map_id: 564,
        name: "TLS 聊天房".into(),
    };
    let r = c
        .rooms(
            rooms::Request::draft(
                common,
                rooms::Draft::new(public, "fixture-room-password".into()).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(r.status, rooms::Status::Ok);
    r.joined.unwrap().summary.room_id
}
fn chat_join(c: &mut ModernTlsConnection, context: SelectionContext) {
    let r = chat_room(c, context);
    let common = chat_room_command(&r, rooms::Action::Join);
    assert_eq!(
        c.rooms(rooms::Request::join(common, "fixture-room-password".into()).unwrap())
            .unwrap()
            .status,
        rooms::Status::Ok
    );
}
fn chat_leave(c: &mut ModernTlsConnection, context: SelectionContext) {
    let r = chat_room(c, context);
    let common = chat_room_command(&r, rooms::Action::Leave);
    assert_eq!(
        c.rooms(rooms::Request::simple(common).unwrap())
            .unwrap()
            .status,
        rooms::Status::Ok
    );
}
#[test]
#[ignore = "requires prepared Java 17 chat application TLS peer"]
fn java_chat_four_channels_cross_source_identity_and_late_join_isolation() {
    let mut peer =
        JavaPeer::start_class("local.lito381.server.ModernChatTestPeer", "normal", "30000");
    let (mut a, ca) = chat_login(&peer, 7);
    let (mut b, cb) = chat_login(&peer, 42);
    assert!(a.supports_chat() && b.supports_chat());
    let mut ra = chat_boot(&mut a, ca, 0, 0);
    let mut rb = chat_boot(&mut b, cb, 0, 0);
    for channel in 0..4 {
        if channel > 0 {
            a.leave_lobby().unwrap();
            a.enter_channel(channel).unwrap();
            ra = chat_boot(&mut a, ca, channel, 0);
        }
        let sent = a
            .chat(
                ra.next_send(format!("頻道 {channel} 👩\u{200d}💻"))
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(sent.status, chat::Status::Ok);
        rb = b.chat(rb.next_read()).unwrap();
        if channel == 0 {
            assert_eq!(rb.messages.len(), 1);
            assert_eq!(rb.messages[0].display_name(), "Sun Knight(太陽神)");
        } else {
            assert!(
                rb.messages.is_empty(),
                "a different channel must not receive this message"
            );
            b.leave_lobby().unwrap();
            b.enter_channel(channel).unwrap();
            rb = chat_boot(&mut b, cb, channel, 0);
            assert!(
                rb.messages.is_empty(),
                "joining cannot recover older audience history"
            );
        }
        rb = b
            .chat(rb.next_send(format!("月亮頻道 {channel}")).unwrap())
            .unwrap();
        assert_eq!(rb.status, chat::Status::Ok);
        ra = a.chat(sent.next_read()).unwrap();
        assert_eq!(ra.messages.len(), 1);
        assert_eq!(ra.messages[0].sender, cb.key);
        assert_eq!(ra.messages[0].display_name(), "Moon Mage(月亮)");
        assert_eq!(ra.sequence, (channel + 1) as u64);
    }
    let denied = a.chat(ra.next_send("第五筆".into()).unwrap()).unwrap();
    assert_eq!(denied.status, chat::Status::RateLimited);
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit();
}
#[test]
#[ignore = "requires prepared Java 17 chat application TLS peer"]
fn java_chat_private_room_membership_ready_gm_and_source_reentry_keep_correct_scope() {
    let mut peer =
        JavaPeer::start_class("local.lito381.server.ModernChatTestPeer", "normal", "30000");
    let (mut a, ca) = chat_login(&peer, 7);
    let (mut b, cb) = chat_login(&peer, 42);
    let room = chat_create(&mut a, ca);
    let mut ra = chat_boot(&mut a, ca, 0, room);
    let lobby = chat_boot(&mut b, cb, 0, 0);
    ra = a
        .chat(ra.next_send("入房前的私訊範圍".into()).unwrap())
        .unwrap();
    assert_eq!(ra.status, chat::Status::Ok);
    assert!(b.chat(lobby.next_read()).unwrap().messages.is_empty());
    chat_join(&mut b, cb);
    let mut rb = chat_boot(&mut b, cb, 0, room);
    assert!(rb.messages.is_empty());
    ra = a.chat(ra.next_send("同房可見".into()).unwrap()).unwrap();
    rb = b.chat(rb.next_read()).unwrap();
    assert_eq!(rb.messages[0].text, "同房可見");
    rb = b.chat(rb.next_send("同房回覆".into()).unwrap()).unwrap();
    assert_eq!(rb.status, chat::Status::Ok);
    ra = a.chat(ra.next_read()).unwrap();
    assert_eq!(ra.messages[0].sender, cb.key);
    chat_leave(&mut b, cb);
    let lb = chat_boot(&mut b, cb, 0, 0);
    assert!(lb.messages.is_empty());
    let lb = b
        .chat(lb.next_send("離房後僅大廳".into()).unwrap())
        .unwrap();
    assert_eq!(lb.status, chat::Status::Ok);
    assert!(a.chat(ra.next_read()).unwrap().messages.is_empty());
    ra = a
        .chat(ra.next_send("離房者不可見".into()).unwrap())
        .unwrap();
    assert!(b.chat(lb.next_read()).unwrap().messages.is_empty());
    chat_join(&mut b, cb);
    let rb = chat_boot(&mut b, cb, 0, room);
    assert!(rb.messages.is_empty());
    let mut ready = chat_room_command(&chat_room(&mut b, cb), rooms::Action::Ready);
    ready.value = 1;
    let before = b.rooms(rooms::Request::simple(ready).unwrap()).unwrap();
    assert_eq!(before.status, rooms::Status::Ok);
    assert_eq!(
        b.chat(rb.next_send("準備狀態不被聊天改動".into()).unwrap())
            .unwrap()
            .status,
        chat::Status::Ok
    );
    let after = chat_room(&mut b, cb);
    assert_eq!(before.joined, after.joined);
    ra = a.chat(ra.next_send("第四筆".into()).unwrap()).unwrap();
    assert_eq!(ra.status, chat::Status::Ok);
    ra = a.chat(ra.next_send("第五筆".into()).unwrap()).unwrap();
    assert_eq!(ra.status, chat::Status::RateLimited);
    ra = a.chat(ra.next_send(".de 20".into()).unwrap()).unwrap();
    assert_eq!(ra.status, chat::Status::CommandRequired);
    assert_eq!(chat_room(&mut a, ca).joined.unwrap().members.len(), 2);
    chat_leave(&mut a, ca);
    chat_leave(&mut b, cb);
    a.return_characters().unwrap();
    b.return_characters().unwrap();
    let list = a.characters(42, 2).unwrap();
    let next = SelectionContext {
        key: list.characters[0].key,
        epoch: 2,
    };
    a.select_character(next).unwrap();
    a.enter_channel(0).unwrap();
    let reset = chat_boot(&mut a, next, 0, 0);
    assert!(reset.membership > ra.membership);
    assert_eq!(reset.sequence, ra.sequence);
    assert!(reset.messages.is_empty());
    let denied = a
        .chat(reset.next_send("切換來源不能清空速率".into()).unwrap())
        .unwrap();
    assert_eq!(denied.status, chat::Status::RateLimited);
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit();
}
#[test]
#[ignore = "requires prepared Java 17 chat application TLS peer"]
fn java_chat_stale_membership_is_rejected_without_redirecting_to_same_rejoined_room() {
    let mut peer =
        JavaPeer::start_class("local.lito381.server.ModernChatTestPeer", "stale", "30000");
    let (mut a, ca) = chat_login(&peer, 7);
    let (mut b, cb) = chat_login(&peer, 42);
    let room = chat_create(&mut a, ca);
    chat_join(&mut b, cb);
    let old = chat_boot(&mut b, cb, 0, room);
    chat_leave(&mut b, cb);
    chat_join(&mut b, cb);
    let fresh = chat_boot(&mut b, cb, 0, room);
    assert!(fresh.membership > old.membership);
    assert!(b
        .chat(old.next_send("過期不可重導".into()).unwrap())
        .is_err());
    assert_eq!(b.application_state(), ApplicationState::Closed);
    assert!(chat_boot(&mut a, ca, 0, room).messages.is_empty());
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit();
}
#[test]
#[ignore = "requires prepared Java 17 chat application TLS peer"]
fn java_chat_unnegotiated_capability_does_not_send_and_existing_ping_remains_usable() {
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernChatTestPeer",
        "unnegotiated",
        "30000",
    );
    let (mut a, ca) = chat_login(&peer, 7);
    let (mut b, cb) = chat_login(&peer, 42);
    assert!(!a.supports_chat() && !b.supports_chat());
    assert!(a.chat(chat::Request::read(ca, 0, 0)).is_err());
    assert!(b.chat(chat::Request::read(cb, 0, 0)).is_err());
    a.ping().unwrap();
    b.ping().unwrap();
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit();
}

#[test]
#[ignore = "requires prepared Java 17 chat application TLS peer"]
fn java_chat_peer_departure_revision_conflict_requires_a_confirmed_room_leave() {
    let mut peer =
        JavaPeer::start_class("local.lito381.server.ModernChatTestPeer", "normal", "30000");
    let (mut a, ca) = chat_login(&peer, 7);
    let (mut b, cb) = chat_login(&peer, 42);
    chat_create(&mut a, ca);
    chat_join(&mut b, cb);
    let before = chat_room(&mut a, ca);
    let stale = chat_room_command(&before, rooms::Action::Leave);
    b.close().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let current = chat_room(&mut a, ca);
        if current.joined.as_ref().unwrap().member(cb.key).is_none() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "server did not retire the guest room membership"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let refused = a.rooms(rooms::Request::simple(stale).unwrap()).unwrap();
    assert_eq!(refused.status, rooms::Status::Conflict);
    assert_eq!(refused.reason, rooms::Reason::RevisionMismatch);
    assert!(refused.joined.is_some());
    assert!(a.leave_lobby().is_err());
    let current = chat_room_command(&refused, rooms::Action::Leave);
    let departed = a.rooms(rooms::Request::simple(current).unwrap()).unwrap();
    assert_eq!(departed.status, rooms::Status::Ok);
    assert!(departed.joined.is_none());
    a.leave_lobby().unwrap();
    a.return_characters().unwrap();
    a.close().unwrap();
    peer.expect_successful_exit();
}
