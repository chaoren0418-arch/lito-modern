// Reuses the owned TLS/controller fixture helpers; no HWND, production SQL or BIN.
use super::super::super::matchmaking_protocol as matching;
fn matching_start(context: SelectionContext) -> matching::Request {
    matching::Request {
        context,
        channel_id: 0,
        action: matching::Action::Start,
        ticket_id: 0,
        criteria: Some(matching::Criteria {
            mode: 1,
            map_id: 514,
            players: 2,
        }),
    }
}
fn matching_send(cs: &mut [SessionController; 2], i: usize, r: matching::Request) {
    let before = cs[i].snapshot().matchmaking;
    cs[i].command(LoginCommand::Matchmaking(r)).unwrap();
    assert!(cs[i].snapshot().room_pending);
    assert_eq!(
        cs[i].snapshot().matchmaking,
        before,
        "intent is not a server confirmation"
    );
    wait(cs, "matchmaking confirmation", |cs| {
        !cs[i].snapshot().room_pending
    });
}
#[test]
#[ignore = "owned Java v34 TLS/navigation/queue authority; synthetic map, no battle or SQL"]
fn java_matchmaking_controllers_queue_cancel_match_balanced_sources_manual_room_and_reentry() {
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernMatchmakingTestPeer",
        "normal",
        "30000",
    );
    let mut cs = clients(&peer);
    let contexts = login(&mut cs, false);
    for c in &cs {
        assert!(c.snapshot().matchmaking.supported());
    }
    matching_send(&mut cs, 0, matching_start(contexts[0]));
    let old = cs[0].snapshot().matchmaking.waiting().unwrap().ticket_id;
    assert!(cs[0].snapshot().rooms.joined().is_none());
    matching_send(
        &mut cs,
        0,
        matching::Request {
            action: matching::Action::Cancel,
            ticket_id: old,
            criteria: None,
            ..matching_start(contexts[0])
        },
    );
    assert_eq!(
        cs[0].snapshot().matchmaking.snapshot().unwrap().phase,
        matching::Phase::Idle
    );
    matching_send(&mut cs, 0, matching_start(contexts[0]));
    assert!(cs[0].snapshot().matchmaking.waiting().unwrap().ticket_id > old);
    assert!(cs[0]
        .command(LoginCommand::Matchmaking(matching::Request {
            action: matching::Action::Cancel,
            ticket_id: old,
            criteria: None,
            ..matching_start(contexts[0])
        }))
        .is_err());
    matching_send(&mut cs, 1, matching_start(contexts[1]));
    wait(&mut cs, "both confirmed members", |cs| {
        cs.iter().all(|c| {
            c.snapshot()
                .matchmaking
                .snapshot()
                .is_some_and(|s| s.phase == matching::Phase::Matched)
        })
    });
    let a = cs[0].snapshot();
    let joined = a.rooms.joined().unwrap();
    assert_eq!(joined.members.len(), 2);
    assert_eq!(joined.owner, contexts[0].key);
    assert!(joined.members.iter().all(|m| !m.ready));
    assert!(!joined.summary.satisfied);
    let mut sources = joined
        .members
        .iter()
        .map(|m| m.source_name.as_str())
        .collect::<Vec<_>>();
    sources.sort();
    assert_eq!(sources, vec!["太陽神", "月亮"]);
    let mut teams = joined.members.iter().map(|m| m.team).collect::<Vec<_>>();
    teams.sort();
    assert_eq!(teams, vec![0, 1]);
    assert_eq!(
        cs[1].snapshot().rooms.joined().unwrap().summary.room_id,
        joined.summary.room_id
    );
    for i in 0..2 {
        let directory = read_room(&mut cs, i, contexts[i]);
        room(
            &mut cs,
            i,
            rooms::Request::simple(command(&directory, rooms::Action::Leave)).unwrap(),
        );
        assert!(matches!(
            cs[i].snapshot().matchmaking,
            matching::State::NotRequested
        ));
    }
    matching_send(&mut cs, 0, matching_start(contexts[0]));
    // Existing manual CREATE owns retirement; a later queue poll cannot restore it.
    let mut common = rooms::Common::read(contexts[0], 0);
    common.action = rooms::Action::Create;
    room(
        &mut cs,
        0,
        rooms::Request::draft(
            common,
            rooms::Draft::new(
                rooms::PublicDraft {
                    capacity: 2,
                    mode: 1,
                    map_id: 514,
                    attributes: 0,
                    name: "Manual after queue".into(),
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    assert!(matches!(
        cs[0].snapshot().matchmaking,
        matching::State::NotRequested
    ));
    let directory = read_room(&mut cs, 0, contexts[0]);
    room(
        &mut cs,
        0,
        rooms::Request::simple(command(&directory, rooms::Action::Leave)).unwrap(),
    );
    matching_send(&mut cs, 0, matching_start(contexts[0]));
    leave_lobby(&mut cs, 0, contexts[0]);
    enter(&mut cs, 0, contexts[0], 0, false);
    assert!(cs[0].snapshot().matchmaking.waiting().is_none());
    assert!(cs[0].snapshot().rooms.joined().is_none());
    matching_send(&mut cs, 0, matching_start(contexts[0]));
    shutdown(&mut cs, &mut peer);
}
fn matching_fault(matched: bool, cancel: bool) {
    let mode = match (matched, cancel) {
        (false, false) => "lost-start",
        (true, false) => "lost-match",
        (false, true) => "cancel-start",
        (true, true) => "cancel-match",
    };
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernMatchmakingTestPeer",
        mode,
        "15000",
    );
    let mut cs = clients_with_timeout(&peer, if cancel { 10000 } else { 1200 });
    cs[0].command(LoginCommand::Connect).unwrap();
    phase(&mut cs, 0, LoginPhase::LoginReady);
    fixture_login(&mut cs, 0);
    phase(&mut cs, 0, LoginPhase::ChoosingSource);
    let context = select(&mut cs, 0, 7);
    enter(&mut cs, 0, context, 0, false);
    cs[0]
        .command(LoginCommand::Matchmaking(matching_start(context)))
        .unwrap();
    if matched {
        wait(&mut cs, "queue ACK before dropped match commit", |cs| {
            !cs[0].snapshot().room_pending
        });
        assert!(cs[0].snapshot().matchmaking.waiting().is_some());
    }
    assert_eq!(
        peer.read_bounded_line("COMMITTED MATCHMAKING").trim_end(),
        if matched {
            "COMMITTED MATCH"
        } else {
            "COMMITTED START"
        }
    );
    cs[0].poll();
    assert!(cs[0].snapshot().rooms.joined().is_none());
    if !matched {
        assert!(cs[0].snapshot().room_pending);
        assert!(cs[0].snapshot().matchmaking.waiting().is_none());
    }
    assert!(cs[0]
        .command(LoginCommand::Matchmaking(matching_start(context)))
        .is_err());
    let started = Instant::now();
    if cancel {
        cs[0].command(LoginCommand::Shutdown).unwrap();
    }
    let expected = if cancel {
        LoginPhase::Disconnected
    } else {
        LoginPhase::Failed
    };
    loop {
        cs[0].poll();
        let v = cs[0].snapshot();
        assert!(
            v.rooms.joined().is_none(),
            "lost room ACK must never invent membership"
        );
        if v.phase == expected && cs[0].is_quiescent() {
            break;
        }
        assert!(started.elapsed() < Duration::from_secs(4), "{}", v.status);
        thread::sleep(Duration::from_millis(5));
    }
    let terminal = cs[0].snapshot();
    if cancel {
        assert!(started.elapsed() < Duration::from_secs(2));
    } else {
        assert!(
            terminal.status.contains("不自動重送"),
            "{}",
            terminal.status
        );
    }
    assert!(!terminal.matchmaking.supported());
    assert!(!terminal.room_pending && terminal.navigation.is_none());
    peer.expect_successful_exit(); // Peer verifies one START, no replay/reconnect, all owners/queues/rooms retired.
}
#[test]
#[ignore = "real queue commit with withheld START ACK"]
fn java_matchmaking_lost_start_reply_never_replays_or_invents_queue() {
    matching_fault(false, false);
}
#[test]
#[ignore = "real room commit with withheld READ ACK"]
fn java_matchmaking_lost_match_reply_never_replays_or_invents_room() {
    matching_fault(true, false);
}
#[test]
#[ignore = "real queue commit then prompt shutdown before long timeout"]
fn java_matchmaking_shutdown_after_queue_commit_retires_without_replay() {
    matching_fault(false, true);
}
#[test]
#[ignore = "real room commit then prompt shutdown before long timeout"]
fn java_matchmaking_shutdown_after_room_commit_retires_without_replay() {
    matching_fault(true, true);
}
