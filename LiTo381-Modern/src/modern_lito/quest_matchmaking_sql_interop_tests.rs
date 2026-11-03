//! Two real controllers, production Main, isolated SQL and shared Quest renderer.
//! Queue confirmations alone are not a gameplay or HWND acceptance test.
use super::*;
use crate::modern_lito::matchmaking_protocol as matching;

#[test]
#[ignore = "v34 production Main/owned SQL/shared map554: matching, ready, countdown, skill70, saved results, return; no HWND"]
fn java_matchmaking_sql_shared_countdown_battle_saved_results_and_return() {
    sql_flow_with_matchmaking(false, false, false, false, false, true, true, true);
}

pub(super) fn matched(cs: &mut [Client; 2], d: &Deadline, contexts: [SelectionContext; 2]) {
    let criteria = matching::Criteria {
        mode: 2,
        map_id: 554,
        players: 2,
    };
    for i in 0..2 {
        assert!(cs[i].view().matchmaking.supported());
        assert!(cs[i].view().rooms.joined().is_none());
        let before = cs[i].view().matchmaking;
        cs[i].command(LoginCommand::Matchmaking(matching::Request {
            context: contexts[i],
            channel_id: CHANNEL,
            action: matching::Action::Start,
            ticket_id: 0,
            criteria: Some(criteria),
        }));
        assert!(cs[i].view().room_pending);
        assert_eq!(
            cs[i].view().matchmaking,
            before,
            "START intent is not membership"
        );
        wait(cs, d, "production SQL-backed queue receipt", |cs| {
            !cs[i].view().room_pending
        });
        if i == 0 {
            let view = cs[0].view();
            let waiting = view
                .matchmaking
                .waiting()
                .expect("first human must wait for another human");
            assert!(waiting.ticket_id > 0);
            assert_eq!(waiting.criteria, Some(criteria));
            assert!(view.rooms.joined().is_none());
        }
    }
    wait(cs, d, "both real humans matched by server", |cs| {
        cs.iter().all(|c| {
            c.view()
                .matchmaking
                .snapshot()
                .is_some_and(|s| s.phase == matching::Phase::Matched)
        })
    });
    let first = cs[0].view();
    let joined = first.rooms.joined().unwrap();
    assert_eq!(joined.owner, contexts[0].key);
    assert_eq!(joined.summary.settings.capacity, 2);
    assert_eq!(
        (joined.summary.settings.mode, joined.summary.settings.map_id),
        (2, 554)
    );
    assert_eq!(joined.members.len(), 2);
    assert!(joined
        .members
        .iter()
        .all(|m| !m.ready && m.kind == rooms::MemberKind::Human));
    assert!(!joined.summary.satisfied);
    let mut teams = joined.members.iter().map(|m| m.team).collect::<Vec<_>>();
    teams.sort();
    assert_eq!(teams, [0, 1]);
    let mut sources = joined
        .members
        .iter()
        .map(|m| m.source_name.as_str())
        .collect::<Vec<_>>();
    sources.sort();
    assert_eq!(sources, ["太陽神", "月亮"]);
    assert_eq!(contexts[0].key.object_id, contexts[1].key.object_id);
    assert_ne!(contexts[0].key.source_id, contexts[1].key.source_id);
    for i in 0..2 {
        let v = cs[i].view();
        let queue = v.matchmaking.snapshot().unwrap();
        assert_eq!(queue.criteria, Some(criteria));
        assert_eq!(
            queue.rooms.common,
            rooms::Common::read(contexts[i], CHANNEL)
        );
        assert_eq!(
            v.rooms.joined().unwrap().summary.room_id,
            joined.summary.room_id
        );
        assert!(v.rooms.joined().unwrap().member(contexts[i].key).is_some());
        assert_eq!(
            v.record.totals(),
            Some(record::Totals {
                stored: false,
                wins: 0,
                losses: 0,
                revision: 0
            })
        );
    }
}

pub(super) fn still_in_room(c: &Client) {
    let v = c.view();
    assert!(v.rooms.joined().is_some(), "world return is not room LEAVE");
    assert_eq!(
        v.matchmaking.snapshot().unwrap().phase,
        matching::Phase::Matched
    );
}
pub(super) fn retired(c: &Client) {
    assert!(matches!(
        c.view().matchmaking,
        matching::State::NotRequested
    ));
    assert!(c.view().rooms.joined().is_none());
}
