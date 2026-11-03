use super::super::application_protocol::CharacterKey;
use super::*;
fn token() -> Token {
    Token {
        enrollment: Enrollment {
            context: SelectionContext {
                key: CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: u64::MAX,
            },
            channel_id: 0,
            generation: 1,
        },
        room_id: 0,
        membership: 1,
    }
}
fn owner() -> wire::Peer {
    wire::Peer {
        key: token().enrollment.context.key,
        character_name: "角色3333".into(),
        source_name: "太陽神".into(),
    }
}
fn other() -> wire::Peer {
    wire::Peer {
        key: CharacterKey {
            source_id: 42,
            object_id: 3333,
        },
        character_name: "角色3333".into(),
        source_name: "月亮神".into(),
    }
}
fn reply(s: &State) -> wire::Reply {
    wire::Reply {
        echo: s.read().unwrap().1,
        status: wire::Status::Ok,
        sequence: s.sequence,
        cursor: s.cursor,
        available: true,
        truncated: false,
        owner: owner(),
        preferences: wire::Preferences::default(),
        blocked: vec![],
        messages: vec![],
    }
}
fn ready() -> State {
    let mut s = State::default();
    s.bind(true, Some(token().enrollment), Some(token()))
        .unwrap();
    s.receive(token(), reply(&s)).unwrap();
    s
}
fn incoming(id: u64) -> wire::Message {
    wire::Message {
        id,
        sender_sequence: id,
        sender: other(),
        recipient: owner(),
        text: format!("秘密{id}"),
    }
}
fn inbox() -> State {
    let mut s = ready();
    let mut r = reply(&s);
    r.cursor = 1;
    r.messages.push(incoming(1));
    s.receive(token(), r).unwrap();
    s
}
fn send(s: &State) -> wire::Request {
    s.command(
        token(),
        wire::Action::Send,
        Some(wire::Address {
            source_id: 42,
            name: "  角色3333  ".into(),
        }),
        None,
        ".de 20".into(),
    )
    .unwrap()
}
#[test]
fn requires_confirmed_private_bootstrap_and_never_optimistically_applies_command() {
    let mut s = State::default();
    s.bind(true, Some(token().enrollment), Some(token()))
        .unwrap();
    assert!(s.waiting());
    assert!(s.token().is_none());
    assert!(s
        .command(token(), wire::Action::Send, None, None, "hello".into())
        .is_err());
    let mut r = reply(&s);
    r.sequence = 17;
    s.receive(token(), r).unwrap();
    assert!(!s.waiting());
    let before = s.clone();
    let request = send(&s);
    assert_eq!(request.sequence, 18);
    assert_eq!(request.text, ".de 20");
    assert_eq!(s, before);
    assert!(s.messages().is_empty());
    assert_eq!(s.outcome(), Outcome::None);
    let mut r = reply(&s);
    r.echo = request;
    r.sequence = 18;
    r.cursor = 1;
    r.messages.push(wire::Message {
        id: 1,
        sender_sequence: 18,
        sender: owner(),
        recipient: other(),
        text: ".de 20".into(),
    });
    s.receive(token(), r).unwrap();
    assert_eq!(s.messages().len(), 1);
    assert_eq!(s.outcome(), Outcome::Confirmed(wire::Action::Send));
    assert!(!format!("{s:?}").contains("3333"));
    assert!(!format!("{s:?}").contains(".de"));
}
#[test]
fn semantic_denial_consumes_sequence_but_does_not_create_message() {
    let mut s = ready();
    let mut r = reply(&s);
    r.echo = send(&s);
    r.sequence = 1;
    r.status = wire::Status::Unreachable;
    s.receive(token(), r).unwrap();
    assert_eq!(send(&s).sequence, 2);
    assert!(s.messages().is_empty());
    assert_eq!(s.outcome(), Outcome::Unreachable);
    let mut r = reply(&s);
    r.echo = send(&s);
    r.sequence = 2;
    r.status = wire::Status::RateLimited;
    s.receive(token(), r).unwrap();
    assert_eq!(s.outcome(), Outcome::RateLimited);
}
#[test]
fn control_changes_only_after_matching_ack_and_read_preserves_last_outcome() {
    let mut s = ready();
    let prefs = wire::Preferences {
        whispers: false,
        world: false,
        trade: true,
    };
    let request = s
        .command(
            token(),
            wire::Action::Preferences,
            None,
            Some(prefs),
            String::new(),
        )
        .unwrap();
    assert_eq!(s.preferences(), Some(wire::Preferences::default()));
    let mut r = reply(&s);
    r.echo = request;
    r.sequence = 1;
    r.preferences = prefs;
    s.receive(token(), r).unwrap();
    assert_eq!(s.preferences(), Some(prefs));
    assert_eq!(s.outcome(), Outcome::Confirmed(wire::Action::Preferences));
    let mut r = reply(&s);
    r.preferences = prefs;
    s.receive(token(), r).unwrap();
    assert_eq!(s.outcome(), Outcome::Confirmed(wire::Action::Preferences));
    let target = wire::Address {
        source_id: 42,
        name: "離線角色".into(),
    };
    let request = s
        .command(
            token(),
            wire::Action::Block,
            Some(target.clone()),
            None,
            String::new(),
        )
        .unwrap();
    assert!(s.blocked().is_empty());
    let mut r = reply(&s);
    r.echo = request;
    r.sequence = 2;
    r.preferences = prefs;
    r.blocked.push(target.clone());
    s.receive(token(), r).unwrap();
    assert_eq!(s.blocked(), [target]);
}
#[test]
fn room_change_suspends_controls_but_retains_history_preferences_and_sequence() {
    let mut s = inbox();
    let old = s.read().unwrap();
    let before = s.clone();
    s.bind(true, Some(token().enrollment), None).unwrap();
    assert!(s.token().is_none() && s.read().is_none());
    assert_eq!(s.messages(), before.messages());
    assert_eq!(s.preferences(), before.preferences());
    assert!(s.receive(old.0, reply(&before)).is_err());
    let next = Token {
        room_id: 9,
        membership: 2,
        ..token()
    };
    s.bind(true, Some(next.enrollment), Some(next)).unwrap();
    let (t, r) = s.read().unwrap();
    assert_eq!(t, next);
    assert_eq!((r.after, r.room_id, r.membership), (1, 9, 2));
    assert!(s
        .command(
            token(),
            wire::Action::ClearBlocks,
            None,
            None,
            String::new()
        )
        .is_err());
    let mut r = reply(&s);
    r.cursor = 2;
    r.messages.push(incoming(2));
    s.receive(next, r).unwrap();
    assert_eq!(s.messages().len(), 2);
}
#[test]
fn new_channel_enrollment_including_same_channel_clears_private_data_and_rejects_old_job() {
    let mut s = inbox();
    let old = s.read().unwrap();
    let stale = reply(&s);
    let next = Token {
        enrollment: Enrollment {
            generation: 2,
            ..token().enrollment
        },
        ..token()
    };
    s.bind(true, Some(next.enrollment), Some(next)).unwrap();
    assert!(s.messages().is_empty() && s.blocked().is_empty());
    assert!(s.preferences().is_none() && s.owner().is_none());
    assert!(
        s.receive(old.0, stale).is_err(),
        "same wire fields cannot authorize a retired local job"
    );
    let mut r = reply(&s);
    r.sequence = 4;
    s.receive(next, r).unwrap();
    assert_eq!(
        s.command(next, wire::Action::ClearBlocks, None, None, String::new())
            .unwrap()
            .sequence,
        5
    );
}
#[test]
fn source_and_epoch_change_clear_old_role_and_validate_new_canonical_owner() {
    let mut s = inbox();
    let next = Token {
        enrollment: Enrollment {
            context: SelectionContext {
                key: other().key,
                epoch: 2,
            },
            channel_id: 1,
            generation: 2,
        },
        room_id: 0,
        membership: 2,
    };
    s.bind(true, Some(next.enrollment), Some(next)).unwrap();
    assert!(s.messages().is_empty());
    let mut r = reply(&s);
    assert!(s.receive(next, r.clone()).is_err());
    r.owner = other();
    r.sequence = 8;
    s.receive(next, r).unwrap();
    assert_eq!(s.owner(), Some(&other()));
}
#[test]
fn replay_owner_rewrite_cursor_rollback_and_duplicate_content_reject_atomically() {
    let s = inbox();
    let mut candidates = vec![];
    let mut r = reply(&s);
    r.sequence = 1;
    candidates.push(r);
    let mut r = reply(&s);
    r.echo.after = 0;
    r.cursor = 0;
    candidates.push(r);
    let mut r = reply(&s);
    r.owner.character_name = "偽名".into();
    candidates.push(r);
    let mut r = reply(&s);
    r.echo.after = 0;
    r.messages.push(incoming(1));
    r.messages[0].text = "rewrite".into();
    candidates.push(r);
    for r in candidates {
        let mut state = s.clone();
        assert!(state.receive(token(), r).is_err());
        assert_eq!(state, s);
    }
    let mut state = s.clone();
    let mut r = reply(&state);
    r.echo.after = 0;
    r.messages.push(incoming(1));
    state.receive(token(), r).unwrap();
    assert_eq!(state.messages().len(), 1);
}
#[test]
fn bounded_history_marks_actual_truncation_and_never_rewrites_arc_snapshot() {
    let mut s = ready();
    let mut r = reply(&s);
    r.cursor = 64;
    r.messages = (1..=64).map(incoming).collect();
    s.receive(token(), r).unwrap();
    let old = s.clone();
    let mut r = reply(&s);
    r.cursor = 65;
    r.messages.push(incoming(65));
    s.receive(token(), r).unwrap();
    assert_eq!(s.messages().len(), 64);
    assert_eq!(s.messages()[0].id, 2);
    assert!(s.history_gap());
    assert_eq!(old.messages()[0].id, 1);
    assert_eq!(old.messages().last().unwrap().id, 64);
    let mut r = reply(&s);
    r.cursor = 130;
    r.truncated = true;
    r.messages = (67..=130).map(incoming).collect();
    s.receive(token(), r).unwrap();
    assert_eq!(s.messages()[0].id, 67);
    assert!(s.history_gap());
}
#[test]
fn unavailable_and_unknown_retire_all_private_data_without_reporting_success_or_rollback() {
    let mut s = inbox();
    let mut r = reply(&s);
    r.available = false;
    r.status = wire::Status::Unavailable;
    s.receive(token(), r).unwrap();
    assert!(s.token().is_none() && s.read().is_none());
    assert_eq!(s.outcome(), Outcome::Unavailable);
    for action in [
        wire::Action::Send,
        wire::Action::Preferences,
        wire::Action::Block,
        wire::Action::Unblock,
        wire::Action::ClearBlocks,
    ] {
        let mut s = inbox();
        s.retire_unknown(action).unwrap();
        assert_eq!(s.outcome(), Outcome::Unknown(action));
        assert!(
            s.messages().is_empty()
                && s.blocked().is_empty()
                && s.owner().is_none()
                && s.preferences().is_none()
        );
        assert!(!s.supported() && s.token().is_none() && s.read().is_none());
        assert!(s.receive(token(), reply(&ready())).is_err());
        s.clear();
        assert_eq!(s, State::default());
    }
    let mut s = inbox();
    let before = s.clone();
    assert!(s.retire_unknown(wire::Action::Read).is_err());
    assert_eq!(s, before);
}
#[test]
fn invalid_or_unnegotiated_bind_never_retains_partial_qualification() {
    let mut s = inbox();
    let before = s.clone();
    assert!(s
        .bind(false, Some(token().enrollment), Some(token()))
        .is_err());
    assert_eq!(s, before);
    let mut bad = token();
    bad.membership = 0;
    assert!(s.bind(true, Some(bad.enrollment), Some(bad)).is_err());
    assert_eq!(s, before);
    bad = token();
    bad.enrollment.generation = 0;
    assert!(s.bind(true, Some(bad.enrollment), Some(bad)).is_err());
    assert_eq!(s, before);
    s.bind(false, None, None).unwrap();
    assert_eq!(s, State::default());
    assert!(s.read().is_none());
}

#[test]
fn final_mutation_can_be_confirmed_without_allowing_counter_wrap_or_further_read() {
    let mut s = State::default();
    s.bind(true, Some(token().enrollment), Some(token()))
        .unwrap();
    let mut r = reply(&s);
    r.sequence = i64::MAX as u64 - 1;
    s.receive(token(), r).unwrap();
    let prefs = wire::Preferences {
        whispers: false,
        world: true,
        trade: false,
    };
    let request = s
        .command(
            token(),
            wire::Action::Preferences,
            None,
            Some(prefs),
            String::new(),
        )
        .unwrap();
    assert_eq!(request.sequence, i64::MAX as u64);
    let mut r = reply(&s);
    r.echo = request;
    r.sequence = i64::MAX as u64;
    r.available = false;
    r.preferences = prefs;
    s.receive(token(), r).unwrap();
    assert_eq!(s.outcome(), Outcome::Confirmed(wire::Action::Preferences));
    assert_eq!(s.preferences(), Some(prefs));
    assert!(s.token().is_none() && s.read().is_none());
    s.bind(true, Some(token().enrollment), None).unwrap();
    assert!(!s.waiting());
}

#[test]
fn unavailable_read_does_not_claim_a_confirmed_mutation_and_invalid_controls_leave_state_unchanged()
{
    let mut s = ready();
    let before = s.clone();
    for action in [
        wire::Action::Send,
        wire::Action::Block,
        wire::Action::Unblock,
        wire::Action::Preferences,
        wire::Action::Read,
    ] {
        assert!(s
            .command(token(), action, None, None, String::new())
            .is_err());
        assert_eq!(s, before);
    }
    let mut r = reply(&s);
    r.available = false;
    s.receive(token(), r).unwrap();
    assert_eq!(s.outcome(), Outcome::Unavailable);
    assert!(s.token().is_none());
}
