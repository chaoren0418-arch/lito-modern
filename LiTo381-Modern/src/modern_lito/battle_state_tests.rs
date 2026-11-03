use super::*;
fn ack() -> protocol::Ack {
    protocol::Ack {
        request_id: 1,
        version: 39,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    }
}
fn data() -> d::UnvalidatedData {
    let mut d = crate::modern_lito::battle_payload::data_tests::fixture();
    d.presentation = Some(d::List(
        d.actors
            .0
            .iter()
            .map(|a| d::PublicPresentation {
                entity: a.entity,
                shot: None,
                item: None,
            })
            .collect(),
    ));
    d
}
fn request(d: &d::UnvalidatedData) -> world::Request {
    world::Request::read(
        app::SelectionContext {
            key: app::CharacterKey {
                source_id: d.personal.player.source as u32,
                object_id: d.personal.player.object as u64,
            },
            epoch: d.personal.player.epoch as u64,
        },
        d.personal.player.channel as u8,
        Some(&d.world.identity),
    )
}
fn reply(d: d::UnvalidatedData) -> world::Reply {
    world::Reply {
        echo: request(&d),
        status: legacy::Status::Ok,
        reason: legacy::Reason::None,
        state: world::State::Data,
        attachment: Some(world::Attachment {
            world: d.world.identity.clone(),
            lifecycle: Lifecycle {
                revision: d.world.revision + 1,
                phase: match d.world.phase {
                    d::Phase::Loading => legacy::Phase::Loading,
                    d::Phase::Countdown => legacy::Phase::Countdown,
                    d::Phase::Active => legacy::Phase::Active,
                    d::Phase::Aborted => legacy::Phase::Aborted,
                    d::Phase::Finished => legacy::Phase::Finished,
                },
                abort: legacy::AbortReason::None,
                settlement: None,
                race_settlement: None,
                race_terminal_revision: 0,
            },
        }),
        data: Some(d),
    }
}
fn preparing() -> world::Reply {
    let mut r = reply(data());
    r.echo.action = world::Action::Prepare;
    r.echo.room_revision = r.echo.generation - 1;
    r.echo.match_id = 0;
    r.echo.generation = 0;
    r.state = world::State::BeforeEngine;
    r.data = None;
    let life = &mut r.attachment.as_mut().unwrap().lifecycle;
    life.revision = 1;
    life.phase = legacy::Phase::Preparing;
    r
}
fn detached(action: world::Action) -> world::Reply {
    let mut r = reply(data());
    r.echo.action = action;
    if action == world::Action::Read {
        r.echo.room = 0;
        r.echo.match_id = 0;
        r.echo.generation = 0;
    }
    r.state = world::State::Detached;
    r.attachment = None;
    r.data = None;
    r
}
fn reject_unchanged(t: &mut Tracker, bad: &world::Reply) {
    let old = t.snapshot().unwrap().clone();
    let high = t.high_water_match;
    assert!(t.accept_world(bad, ack()).is_err());
    assert!(Arc::ptr_eq(&old, t.snapshot().unwrap()));
    assert_eq!(t.high_water_match, high);
}
#[test]
fn preparation_loading_and_retained_observation_are_atomic_without_fake_frames() {
    let mut t = Tracker::default();
    t.accept_world(&detached(world::Action::Read), ack())
        .unwrap();
    t.accept_world(&preparing(), ack()).unwrap();
    let before = t.snapshot().unwrap().clone();
    assert!(before.observation().is_none() && before.presentation_observation().is_none());
    let d = data();
    t.accept_world(&reply(d.clone()), ack()).unwrap();
    let retained = t.snapshot().unwrap().clone();
    assert_eq!(retained.observation().unwrap().data(), &d);
    assert_eq!(retained.presentation_observation().unwrap().data(), &d);
    let mut clock = d.clone();
    clock.world.loading_elapsed_nanos = 10;
    t.accept_world(&reply(clock), ack()).unwrap();
    reject_unchanged(&mut t, &reply(d.clone()));
    reject_unchanged(&mut t, &preparing());
    assert_eq!(retained.observation().unwrap().data(), &d);
    assert_eq!(
        before.attachment().unwrap().lifecycle.phase,
        legacy::Phase::Preparing
    );
    assert!(before.observation().is_none());
    assert_eq!(format!("{retained:?}"), "ConfirmedBattleState[redacted]");
}
#[test]
fn failed_payload_and_same_revision_changes_do_not_advance_any_state() {
    let r = reply(data());
    let mut t = Tracker::default();
    t.accept_world(&r, ack()).unwrap();
    for case in 0..6 {
        let mut bad = r.clone();
        match case {
            0 => bad.data.as_mut().unwrap().actors.0[0].heading = 1,
            1 => bad.attachment.as_mut().unwrap().lifecycle.revision -= 1,
            2 => bad.attachment.as_mut().unwrap().lifecycle.revision += 1,
            3 => bad.data.as_mut().unwrap().world.revision += 1,
            4 => bad.data.as_mut().unwrap().presentation = None,
            _ => bad.reason = legacy::Reason::NotReady,
        }
        // A service-only tick is allowed when not terminal; give case 2 an
        // invalid actor too, so failure happens after candidate construction.
        if case == 2 {
            bad.data.as_mut().unwrap().actors.0[0].heading = 1;
        }
        reject_unchanged(&mut t, &bad);
    }
    t.accept_world(&r, ack()).unwrap();
}
#[test]
fn scope_cannot_change_even_when_object_id_is_equal() {
    let r = reply(data());
    let mut t = Tracker::default();
    t.accept_world(&r, ack()).unwrap();
    for case in 0..5 {
        let mut bad = r.clone();
        match case {
            0 => bad.echo.context.key.source_id += 1,
            1 => bad.echo.context.key.object_id += 1,
            2 => bad.echo.context.epoch += 1,
            3 => bad.echo.channel += 1,
            _ => bad.attachment.as_mut().unwrap().world.match_id += 1,
        }
        reject_unchanged(&mut t, &bad);
    }
    // Each is a valid independent audience, not merely an echo/payload typo.
    for case in 0..3 {
        let mut foreign = data();
        match case {
            0 => {
                foreign.personal.player.source = 8;
                foreign.personal.entity = 2;
            }
            1 => foreign.personal.player.epoch += 1,
            _ => foreign.personal.player.channel += 1,
        }
        let other = reply(foreign);
        Tracker::default().accept_world(&other, ack()).unwrap();
        reject_unchanged(&mut t, &other);
    }
}
#[test]
fn stale_local_lifecycle_requests_are_rejected_without_retiring_the_connection_state() {
    let mut t = Tracker::default();
    let q = request(&data());
    assert!(t.admits_world(&q));
    assert!(!t.admits_world(&world::Request {
        action: world::Action::Leave,
        ..q.clone()
    }));
    t.accept_world(&reply(data()), ack()).unwrap();
    let retained = t.snapshot().unwrap().clone();
    assert!(!t.admits_world(&preparing().echo));
    for case in 0..5 {
        let mut bad = q.clone();
        match case {
            0 => bad.context.epoch += 1,
            1 => bad.room += 1,
            2 => bad.match_id += 1,
            3 => bad.generation += 1,
            _ => bad.channel += 1,
        }
        assert!(!t.admits_world(&bad));
        assert!(Arc::ptr_eq(&retained, t.snapshot().unwrap()));
    }
    assert!(t.admits_world(&world::Request::read(q.context, q.channel, None)));
    t.accept_world(&detached(world::Action::Leave), ack())
        .unwrap();
    assert!(!t.admits_world(&q));
    assert!(t.admits_world(&preparing().echo));
    assert!(t.admits_world(&world::Request {
        match_id: q.match_id + 1,
        ..q.clone()
    }));
    t.retire();
    assert!(!t.admits_world(&q));
}
#[test]
fn explicit_leave_retires_match_and_navigation_keeps_the_retirement_fence() {
    let r = reply(data());
    let mut t = Tracker::default();
    t.accept_world(&r, ack()).unwrap();
    reject_unchanged(&mut t, &detached(world::Action::Read));
    let mut wrong_leave = detached(world::Action::Leave);
    wrong_leave.echo.match_id += 1;
    reject_unchanged(&mut t, &wrong_leave);
    t.accept_world(&detached(world::Action::Leave), ack())
        .unwrap();
    assert!(t.snapshot().unwrap().attachment().is_none());
    t.accept_world(&detached(world::Action::Read), ack())
        .unwrap();
    reject_unchanged(&mut t, &r);
    t.retire();
    assert!(t.snapshot().is_none());
    assert!(t.accept_world(&r, ack()).is_err());
    let mut next = data();
    next.world.identity.match_id += 1;
    t.accept_world(&reply(next), ack()).unwrap();
    assert_eq!(t.high_water_match, 103);
}
#[test]
fn first_failed_attachment_does_not_poison_the_match_fence() {
    let mut t = Tracker::default();
    let mut bad = data();
    bad.world.identity.match_id = 9000;
    bad.world.active_elapsed_nanos = 0; // Loading must have no ACTIVE clock.
    assert!(t.accept_world(&reply(bad), ack()).is_err());
    assert!(t.snapshot().is_none() && t.follower.is_none());
    assert_eq!(t.high_water_match, 0);
    t.accept_world(&reply(data()), ack()).unwrap();
}
#[test]
fn preparing_abort_cannot_resurrect_or_create_an_engine_capture() {
    let mut t = Tracker::default();
    t.accept_world(&preparing(), ack()).unwrap();
    let mut aborted = preparing();
    aborted.echo = request(&data());
    let l = &mut aborted.attachment.as_mut().unwrap().lifecycle;
    l.revision = 2;
    l.phase = legacy::Phase::Aborted;
    l.abort = legacy::AbortReason::PreparationFailed;
    t.accept_world(&aborted, ack()).unwrap();
    t.accept_world(&aborted, ack()).unwrap();
    assert!(!t.admits_world(&preparing().echo));
    reject_unchanged(&mut t, &reply(data()));
    let mut reason = aborted.clone();
    reason.attachment.as_mut().unwrap().lifecycle.revision += 1;
    reason.attachment.as_mut().unwrap().lifecycle.abort = legacy::AbortReason::ServerAbort;
    reject_unchanged(&mut t, &reason);
    assert!(t.snapshot().unwrap().observation().is_none());
    t.accept_world(&detached(world::Action::Leave), ack())
        .unwrap();
}
fn active() -> d::UnvalidatedData {
    let mut d = data();
    d.world.revision = 3;
    d.world.phase = d::Phase::Active;
    d.world.loading_elapsed_nanos = 1000;
    d.world.active_elapsed_nanos = 0;
    for a in &mut d.actors.0 {
        a.loaded = true;
    }
    d
}
#[test]
fn service_abort_overlay_preserves_exact_engine_capture_but_hides_active_presentation() {
    let d = active();
    let mut t = Tracker::default();
    t.accept_world(&reply(d.clone()), ack()).unwrap();
    let mut aborted = reply(d.clone());
    let l = &mut aborted.attachment.as_mut().unwrap().lifecycle;
    l.revision += 1;
    l.phase = legacy::Phase::Aborted;
    l.abort = legacy::AbortReason::ServerAbort;
    t.accept_world(&aborted, ack()).unwrap();
    assert_eq!(t.snapshot().unwrap().observation().unwrap().data(), &d);
    assert!(t.snapshot().unwrap().presentation_observation().is_none());
    let mut clock = aborted.clone();
    clock.data.as_mut().unwrap().world.loading_elapsed_nanos += 1;
    clock.data.as_mut().unwrap().world.active_elapsed_nanos += 1;
    reject_unchanged(&mut t, &clock);
    let mut resurrect = reply(d);
    resurrect.attachment.as_mut().unwrap().lifecycle.revision += 2;
    reject_unchanged(&mut t, &resurrect);
}
#[test]
fn operation_and_read_share_one_atomic_follower_without_acknowledging_stale_life() {
    let mut t = Tracker::default();
    t.accept_world(&reply(active()), ack()).unwrap();
    let guard = Guard {
        stamp: super::super::battle_window::Stamp::of(t.snapshot().unwrap().observation().unwrap()),
        target: None,
    };
    let mut wrong = guard;
    wrong.stamp.world[1] += 1;
    assert!(!t.admits_operation(wrong));
    let mut stale = guard;
    stale.stamp.life += 1;
    assert!(t.admits_operation(stale));
    let mut next = active();
    next.world.revision += 1;
    next.actors.0[0].last_move_sequence += 1;
    next.actors.0[0].position.x += 1;
    let r = reply(next.clone());
    let operation = operation::Reply {
        guard,
        rejection: super::super::battle_action::Rejection::None,
        disposition: operation::Disposition::Applied,
        reason: operation::Reason::None,
        lifecycle: r.attachment.as_ref().unwrap().lifecycle,
        data: next.clone(),
    };
    t.accept_operation(&operation, ack()).unwrap();
    assert_eq!(t.snapshot().unwrap().observation().unwrap().data(), &next);
    reject_unchanged(&mut t, &reply(active()));
    t.accept_world(&r, ack()).unwrap();
    let retained = t.snapshot().unwrap().clone();
    let mut bad = operation;
    bad.data.actors.0[0].heading = 1;
    assert!(t.accept_operation(&bad, ack()).is_err());
    assert!(Arc::ptr_eq(&retained, t.snapshot().unwrap()));
    t.accept_world(&detached(world::Action::Leave), ack())
        .unwrap();
    assert!(!t.admits_operation(guard));
}
#[test]
fn settlement_saved_failed_and_race_terminal_are_sticky() {
    use legacy::SettlementState as S;
    let mut a = reply(active()).attachment.unwrap().lifecycle;
    a.phase = legacy::Phase::Finished;
    a.revision = 20;
    a.settlement = Some(S::Pending);
    a.race_settlement = Some(S::Pending);
    a.race_terminal_revision = 20;
    for final_state in [S::Saved, S::Failed] {
        let b = Lifecycle {
            revision: 21,
            settlement: Some(final_state),
            race_settlement: Some(final_state),
            ..a
        };
        lifecycle_follows(a, b).unwrap();
        lifecycle_follows(b, b).unwrap();
        for state in [None, Some(S::Pending), Some(S::Saved), Some(S::Failed)] {
            let bad = Lifecycle {
                revision: 22,
                settlement: state,
                ..b
            };
            assert_eq!(
                lifecycle_follows(b, bad).is_ok(),
                state == Some(final_state)
            );
            let bad = Lifecycle {
                revision: 22,
                race_settlement: state,
                ..b
            };
            assert_eq!(
                lifecycle_follows(b, bad).is_ok(),
                state == Some(final_state)
            );
        }
        assert!(lifecycle_follows(
            b,
            Lifecycle {
                race_terminal_revision: 21,
                ..b
            }
        )
        .is_err());
        assert!(lifecycle_follows(a, Lifecycle { revision: 20, ..b }).is_err());
        assert!(lifecycle_follows(
            b,
            Lifecycle {
                phase: legacy::Phase::Active,
                revision: 22,
                ..b
            }
        )
        .is_err());
    }
}
#[test]
fn unnegotiated_or_malformed_public_world_dto_never_publishes() {
    for bad_ack in [
        protocol::Ack {
            version: 38,
            ..ack()
        },
        protocol::Ack {
            numeric_schema: 2,
            ..ack()
        },
        protocol::Ack {
            capabilities: protocol::KNOWN_CAPABILITIES & !protocol::CAP_ACTION_LIFE,
            ..ack()
        },
    ] {
        let mut t = Tracker::default();
        assert!(t.accept_world(&reply(data()), bad_ack).is_err());
        assert!(t.snapshot().is_none());
    }
    for case in 0..5 {
        let mut t = Tracker::default();
        let mut bad = reply(data());
        match case {
            0 => bad.state = world::State::Detached,
            1 => bad.state = world::State::BeforeEngine,
            2 => bad.data = None,
            3 => bad.attachment = None,
            _ => bad.echo.action = world::Action::Leave,
        }
        assert!(t.accept_world(&bad, ack()).is_err());
        assert!(t.snapshot().is_none());
    }
}
