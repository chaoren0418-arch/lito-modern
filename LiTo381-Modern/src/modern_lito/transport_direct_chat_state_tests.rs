// Confirmed presentation + real TLS, not a SessionController or HWND claim.
#[test]
#[ignore = "private confirmed state with actual dual TLS authority; no UI or production SQL"]
fn java_direct_chat_state_dual_peer_room_rebind_preserves_inbox_and_channel_reentry_clears_it() {
    use super::super::direct_chat_state as state;
    let mut peer =
        JavaPeer::start_class("local.lito381.server.ModernChatTestPeer", "normal", "30000");
    let (mut a, ca) = direct_login(&peer, 7);
    let (mut b, cb) = direct_login(&peer, 42);
    let initial_a = direct_boot(&mut a, ca, 0, 0);
    let initial_b = direct_boot(&mut b, cb, 0, 0);
    let ta = state::Token {
        enrollment: state::Enrollment {
            context: ca,
            channel_id: 0,
            generation: 1,
        },
        room_id: 0,
        membership: initial_a.echo.membership,
    };
    let mut tb = state::Token {
        enrollment: state::Enrollment {
            context: cb,
            channel_id: 0,
            generation: 1,
        },
        room_id: 0,
        membership: initial_b.echo.membership,
    };
    let mut va = state::State::default();
    let mut vb = state::State::default();
    va.bind(true, Some(ta.enrollment), Some(ta)).unwrap();
    va.receive(ta, initial_a).unwrap();
    vb.bind(true, Some(tb.enrollment), Some(tb)).unwrap();
    vb.receive(tb, initial_b).unwrap();
    let request = va
        .command(
            ta,
            direct::Action::Send,
            direct_target(42, "Moon Mage"),
            None,
            "私訊不受房間切換清空".into(),
        )
        .unwrap();
    assert!(va.messages().is_empty());
    va.receive(ta, a.direct_chat(request).unwrap()).unwrap();
    let (token, read) = vb.read().unwrap();
    vb.receive(token, b.direct_chat(read).unwrap()).unwrap();
    assert_eq!(vb.messages().len(), 1);
    assert_eq!(vb.messages()[0].sender.key, ca.key);
    assert_eq!(vb.messages()[0].recipient.key, cb.key);
    let request = vb
        .command(
            tb,
            direct::Action::Block,
            direct_target(7, "  SUN KNIGHT  "),
            None,
            String::new(),
        )
        .unwrap();
    vb.receive(tb, b.direct_chat(request).unwrap()).unwrap();
    assert_eq!(vb.blocked()[0].name, "sun knight");
    let request = va
        .command(
            ta,
            direct::Action::Send,
            direct_target(42, "Moon Mage"),
            None,
            "遭拒收".into(),
        )
        .unwrap();
    va.receive(ta, a.direct_chat(request).unwrap()).unwrap();
    assert_eq!(va.outcome(), state::Outcome::Unreachable);
    assert_eq!(va.messages().len(), 1);
    let old = tb;
    let kept = vb.clone();
    let room = chat_create(&mut b, cb);
    vb.bind(true, Some(tb.enrollment), None).unwrap();
    assert_eq!(vb.messages(), kept.messages());
    assert_eq!(vb.blocked(), kept.blocked());
    assert!(vb.token().is_none());
    let member = chat_boot(&mut b, cb, 0, room).membership;
    tb = state::Token {
        room_id: room,
        membership: member,
        ..tb
    };
    vb.bind(true, Some(tb.enrollment), Some(tb)).unwrap();
    let (token, read) = vb.read().unwrap();
    assert_eq!(read.after, 1);
    vb.receive(token, b.direct_chat(read).unwrap()).unwrap();
    assert_eq!(vb.messages(), kept.messages());
    assert!(vb
        .command(old, direct::Action::ClearBlocks, None, None, String::new())
        .is_err());
    let request = vb
        .command(
            tb,
            direct::Action::Unblock,
            direct_target(7, "Sun Knight"),
            None,
            String::new(),
        )
        .unwrap();
    vb.receive(tb, b.direct_chat(request).unwrap()).unwrap();
    assert!(vb.blocked().is_empty());
    let request = va
        .command(
            ta,
            direct::Action::Send,
            direct_target(42, "Moon Mage"),
            None,
            "跨房間再收到".into(),
        )
        .unwrap();
    va.receive(ta, a.direct_chat(request).unwrap()).unwrap();
    let (token, read) = vb.read().unwrap();
    let stale = b.direct_chat(read).unwrap();
    vb.receive(token, stale.clone()).unwrap();
    assert_eq!(vb.messages().len(), 2);
    chat_leave(&mut b, cb);
    b.leave_lobby().unwrap();
    b.enter_channel(1).unwrap();
    let member = chat_boot(&mut b, cb, 1, 0).membership;
    let next = state::Token {
        enrollment: state::Enrollment {
            channel_id: 1,
            generation: 2,
            ..tb.enrollment
        },
        room_id: 0,
        membership: member,
    };
    vb.bind(true, Some(next.enrollment), Some(next)).unwrap();
    assert!(vb.messages().is_empty() && vb.preferences().is_none());
    assert!(vb.receive(token, stale).is_err());
    let (token, read) = vb.read().unwrap();
    vb.receive(token, b.direct_chat(read).unwrap()).unwrap();
    assert!(vb.messages().is_empty());
    assert_eq!(
        vb.command(next, direct::Action::ClearBlocks, None, None, String::new())
            .unwrap()
            .sequence,
        3,
        "server sequence survives channel change"
    );
    a.close().unwrap();
    b.close().unwrap();
    va.clear();
    vb.clear();
    assert_eq!(va, state::State::default());
    assert_eq!(vb, state::State::default());
    peer.expect_successful_exit();
}
