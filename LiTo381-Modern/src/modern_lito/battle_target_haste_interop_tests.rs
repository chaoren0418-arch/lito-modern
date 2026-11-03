//! v44 public TLS service/transport command and shared LOAD. Synthetic sources;
//! neither source-SQL qualification nor skillbook/controller/visual acceptance.
use super::*;
use crate::modern_lito::{battle_target_haste as haste, battle_window::Stamp};

fn observation(c: &ModernTlsConnection) -> crate::modern_lito::battle_follow::Observation {
    c.battle_snapshot().unwrap().observation().unwrap().clone()
}
fn refresh(c: &mut ModernTlsConnection, context: SelectionContext, w: &data::WorldId) {
    assert_eq!(
        c.battle_world(battle_world::Request::read(context, CHANNEL, Some(w)))
            .unwrap()
            .status,
        world::Status::Ok
    );
    assert!(c.world_snapshot().is_none());
}
fn self_actor(o: &crate::modern_lito::battle_follow::Observation) -> &data::Actor {
    o.data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .unwrap()
}
fn intent(c: &ModernTlsConnection) -> haste::Intent {
    let o = observation(c);
    let a = self_actor(&o);
    haste::Intent {
        request_id: 1,
        stamp: Stamp::of(&o),
        sequence: a.spells.sequence + 1,
        equipment_revision: a.resources.as_ref().unwrap().equipment_revision,
        x: a.position.x,
        y: a.position.y,
        buff_revision: a.buffs.as_ref().unwrap().revision,
        target: haste::Target::from_data(
            o.data(),
            crate::modern_lito::battle_window::Target::Player {
                entity: a.entity,
                life: a.resources.as_ref().unwrap().life + 1,
            },
        )
        .unwrap(),
    }
}

#[test]
#[ignore = "v44 actual two-source TLS, shared514 LOAD/target43/move/leave; no SQL, skill UI or HWND"]
fn java_v44_target_haste_tls_shared_load_source_isolation_target_stamp_and_leave() {
    flow(44);
}
fn flow(version: u16) {
    let ReadyPair {
        deadline,
        shared,
        mut peer,
        mut a,
        mut b,
        sun,
        moon,
        ready,
    } = ready_pair_mode(version, version, Some("target-haste"));
    assert!(a.supports_target_haste());
    assert!(b.supports_target_haste());
    // Source qualification is separate from the world catalog: same objid in
    // a different source must never inherit skill43.
    let sun_skills = a.skills().unwrap();
    let moon_skills = b.skills().unwrap();
    assert_eq!(sun_skills.context, sun);
    assert_eq!(moon_skills.context, moon);
    assert!(sun_skills.greater_haste().is_none());
    assert!(moon_skills.greater_haste().is_none());
    assert_eq!(sun_skills.skills.len(), 1);
    let row = &sun_skills.skills[0];
    assert_eq!((row.skill_id, row.mp_cost, row.reuse_millis), (43, 40, 10));
    assert_eq!(
        row.support,
        crate::modern_lito::spell_protocol::Support::Haste
    );
    assert!(sun_skills.haste().is_some());
    assert!(moon_skills.haste().is_none());
    assert!(moon_skills.skills.is_empty());
    let prepared = a
        .battle_world(battle_world::Request {
            action: battle_world::Action::Prepare,
            room: ready.summary.room_id as i64,
            room_revision: ready.summary.revision,
            ..battle_world::Request::read(sun, CHANNEL, None)
        })
        .unwrap();
    assert_eq!(prepared.state, battle_world::State::BeforeEngine);
    let w = prepared.attachment.unwrap().world;
    loop {
        refresh(&mut a, sun, &w);
        refresh(&mut b, moon, &w);
        if a.battle_snapshot().unwrap().observation().is_some()
            && b.battle_snapshot().unwrap().observation().is_some()
        {
            break;
        }
        deadline.pause("v44 preparation");
    }
    assert!(observation(&a).data().haste.is_some());
    let early = a.target_haste(intent(&a)).unwrap();
    assert_eq!(early.reason, haste::Reason::NotActive);
    // Loading denies without a charge/effect, but the live engine consumes
    // this request. A terminal publication may instead keep its old sequence.
    assert_eq!(self_actor(&observation(&a)).spells.sequence, 1);
    assert_eq!(
        self_actor(&observation(&a))
            .vital
            .as_ref()
            .unwrap()
            .values
            .mp,
        300
    );
    assert_eq!(
        self_actor(&observation(&a))
            .buffs
            .as_ref()
            .unwrap()
            .haste_until_nanos,
        0
    );
    let mut runtimes = [
        BattleRuntime::new(shared.clone()).unwrap(),
        BattleRuntime::new(shared).unwrap(),
    ];
    runtimes[0].attach(observation(&a)).unwrap();
    runtimes[1].attach(observation(&b)).unwrap();
    let mut loads = [None, None];
    let mut keepalive = Instant::now();
    while loads.iter().any(Option::is_none) {
        for (i, c) in [&a, &b].iter().enumerate() {
            match runtimes[i].poll() {
                BattleStatus::Ready(surface) => {
                    loads[i] = runtimes[i].loaded_request(&surface, &observation(c))
                }
                BattleStatus::Failed(reason) => panic!("v44 shared preflight: {reason}"),
                _ => {}
            }
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            a.ping().unwrap();
            b.ping().unwrap();
            keepalive = Instant::now();
        }
        deadline.pause("v44 canonical LOAD qualification");
    }
    assert_eq!(
        a.battle_world(loads[0].take().unwrap()).unwrap().status,
        world::Status::Ok
    );
    assert_eq!(
        b.battle_world(loads[1].take().unwrap()).unwrap().status,
        world::Status::Ok
    );
    loop {
        refresh(&mut a, sun, &w);
        refresh(&mut b, moon, &w);
        if observation(&a).data().world.phase == data::Phase::Active {
            break;
        }
        deadline.pause("v44 countdown");
    }
    let read_before = a
        .battle_world(battle_world::Request::read(sun, CHANNEL, Some(&w)))
        .unwrap();
    let ack = a.ack().unwrap();
    let mut tracker = crate::modern_lito::battle_state::Tracker::default();
    tracker.accept_world(&read_before, ack).unwrap();
    let before = observation(&a);
    let mut q = intent(&a);
    let other = observation(&b);
    let other_actor = self_actor(&other);
    q.target = haste::Target::from_data(
        before.data(),
        crate::modern_lito::battle_window::Target::Player {
            entity: other_actor.entity,
            life: other_actor.resources.as_ref().unwrap().life + 1,
        },
    )
    .unwrap();
    assert_eq!(
        haste::Intent::from_observation(&before, q.target.selection()).unwrap(),
        q
    );
    let retained = a.battle_snapshot().unwrap().clone();
    // A future proof was never observed and must be a confirmed non-send.
    // Older proofs with unchanged fields are covered by the live Quest test;
    // do not require global revision equality when no actor fact changed.
    let mut stale_revision = q;
    stale_revision.stamp.revision += 1;
    let mut controller_state = crate::modern_lito::battle_control::State::default();
    controller_state
        .execute(
            &mut a,
            crate::modern_lito::battle_control::Command::target_haste(stale_revision),
            1,
        )
        .unwrap();
    assert_eq!(
        controller_state.outcome(),
        Some(crate::modern_lito::battle_control::Outcome::NotSent)
    );
    assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
    assert_eq!(
        self_actor(&observation(&a)).spells,
        self_actor(&before).spells
    );
    let mut wrong = q;
    wrong.stamp.context.key.source_id = 42;
    assert!(a.target_haste(wrong).is_err());
    assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
    a.ping().unwrap();
    // All local stale-target/source/life/cell/sequence failures retain the
    // accepted Arc and do not send a packet or consume the pending sequence.
    let mut foreign_target = q;
    if let haste::Target::Player { source, .. } = &mut foreign_target.target {
        *source = 7;
    }
    assert!(a.target_haste(foreign_target).is_err());
    let mut stale_target = q;
    if let haste::Target::Player { life, .. } = &mut stale_target.target {
        *life += 1;
    }
    assert!(a.target_haste(stale_target).is_err());
    let mut stale_cell = q;
    if let haste::Target::Player { x, .. } = &mut stale_cell.target {
        *x += 1;
    }
    assert!(a.target_haste(stale_cell).is_err());
    let mut skipped = q;
    skipped.sequence += 1;
    assert!(a.target_haste(skipped).is_err());
    let mut old_feature = before.data().clone();
    old_feature.haste = None;
    assert!(q.validate_before(&old_feature).is_err());
    assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
    let success = a.target_haste(q).unwrap();
    assert_eq!(success.reason, haste::Reason::None);
    let retained_tracker = tracker.snapshot().unwrap().clone();
    let mut wrong_reply = success.clone();
    wrong_reply.echo.sequence += 1;
    assert!(tracker
        .accept_target_haste(&wrong_reply, &success.echo, ack)
        .is_err());
    assert!(Arc::ptr_eq(&retained_tracker, tracker.snapshot().unwrap()));
    assert!(tracker
        .accept_target_haste(
            &success,
            &success.echo,
            protocol::Ack {
                capabilities: ack.capabilities & !protocol::CAP_HASTE,
                ..ack
            }
        )
        .is_err());
    assert!(Arc::ptr_eq(&retained_tracker, tracker.snapshot().unwrap()));
    let mut wrong_target_reply = success.clone();
    if let haste::Target::Player { source, .. } = &mut wrong_target_reply.echo.target {
        *source = 7;
    }
    assert!(tracker
        .accept_target_haste(&wrong_target_reply, &success.echo, ack)
        .is_err());
    assert!(Arc::ptr_eq(&retained_tracker, tracker.snapshot().unwrap()));
    tracker
        .accept_target_haste(&success, &success.echo, ack)
        .unwrap();
    assert_eq!(
        tracker.snapshot().unwrap().observation().unwrap().data(),
        &success.data
    );
    assert_eq!(observation(&a).data(), &success.data);
    let o = observation(&a);
    let own = self_actor(&o);
    assert_eq!(own.vital.as_ref().unwrap().values.mp, 260);
    assert_eq!(self_actor(&before).vital.as_ref().unwrap().values.mp, 300);
    assert_eq!(own.buffs.as_ref().unwrap().haste_until_nanos, 0);
    let recipient = success
        .data
        .actors
        .0
        .iter()
        .find(|actor| actor.entity == other_actor.entity)
        .unwrap();
    assert_eq!(recipient.vital.as_ref().unwrap().values.mp, 300);
    assert_eq!(
        recipient.buffs.as_ref().unwrap().haste_origin,
        data::HasteOrigin::Haste
    );
    assert_eq!(
        recipient.buffs.as_ref().unwrap().haste_until_nanos,
        success.data.world.active_elapsed_nanos + 1_200_000_000_000
    );
    assert!(!q.current(&observation(&a)));
    let fresh = intent(&a);
    let stale = haste::Intent {
        target: q.target,
        ..fresh
    };
    let retained_after = a.battle_snapshot().unwrap().clone();
    assert!(a.target_haste(stale).is_err());
    assert!(Arc::ptr_eq(&retained_after, a.battle_snapshot().unwrap()));
    assert_eq!(
        a.target_haste(intent(&a)).unwrap().reason,
        haste::Reason::TooFast
    );
    assert_eq!(
        self_actor(&observation(&a))
            .vital
            .as_ref()
            .unwrap()
            .values
            .mp,
        260
    );
    assert_eq!(
        b.target_haste(intent(&b)).unwrap().reason,
        haste::Reason::Unlearned
    );
    assert_eq!(
        self_actor(&observation(&b))
            .vital
            .as_ref()
            .unwrap()
            .values
            .mp,
        300
    );
    // Passive reads keep the schema5 target buff; legacy caches stay absent.
    loop {
        refresh(&mut a, sun, &w);
        refresh(&mut b, moon, &w);
        if observation(&a).data().world.active_elapsed_nanos
            - success.data.world.active_elapsed_nanos
            >= 2_000_000_000
        {
            break;
        }
        deadline.pause("v44 authoritative cast recovery before MOVE");
    }
    refresh(&mut a, sun, &w);
    refresh(&mut b, moon, &w);
    let active = observation(&a);
    runtimes[0].update(active.clone()).unwrap();
    let surface = worker_surface(&mut runtimes[0], &active, &deadline);
    let own = self_actor(&active);
    let from = WorldCell {
        x: own.position.x,
        y: own.position.y,
    };
    let heading = DIRECTIONS
        .iter()
        .enumerate()
        .find_map(|(h, (dx, dy))| {
            let to = WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            };
            (surface
                .navigation
                .as_ref()
                .unwrap()
                .map()
                .can_step(from, to)
                && active
                    .data()
                    .actors
                    .0
                    .iter()
                    .all(|a| a.position.x != to.x || a.position.y != to.y))
            .then_some(h as u8)
        })
        .unwrap();
    let command = world::Request {
        action: world::Action::Move,
        heading,
        sequence: own.last_move_sequence as u64 + 1,
        from_x: from.x,
        from_y: from.y,
        match_id: w.match_id as u64,
        generation: w.generation as u64,
        ..world::Request::read(sun, CHANNEL)
    };
    let moved = a
        .battle_operation(
            crate::modern_lito::battle_action::Guard {
                stamp: Stamp::of(&active),
                target: None,
            },
            |id, _| world::encode_request(id, &command),
        )
        .unwrap();
    assert_eq!(
        moved.disposition,
        crate::modern_lito::battle_operation::Disposition::Applied
    );
    let moved_observation = observation(&a);
    let moved_actor = self_actor(&moved_observation);
    let (dx, dy) = DIRECTIONS[usize::from(heading)];
    assert_eq!(
        (moved_actor.position.x, moved_actor.position.y),
        (from.x + dx, from.y + dy)
    );
    assert!(observation(&a).data().haste.is_some());
    for runtime in &mut runtimes {
        runtime.shutdown();
        join_worker(runtime, &deadline);
    }
    b.battle_world(battle_world::Request {
        action: battle_world::Action::Leave,
        ..battle_world::Request::read(moon, CHANNEL, Some(&w))
    })
    .unwrap();
    refresh(&mut a, sun, &w);
    assert_eq!(
        a.target_haste(intent(&a)).unwrap().reason,
        haste::Reason::NotActive
    );
    assert!(a
        .battle_snapshot()
        .unwrap()
        .presentation_observation()
        .is_none());
    let old = intent(&a);
    a.battle_world(battle_world::Request {
        action: battle_world::Action::Leave,
        ..battle_world::Request::read(sun, CHANNEL, Some(&w))
    })
    .unwrap();
    assert!(a.target_haste(old).is_err());
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
