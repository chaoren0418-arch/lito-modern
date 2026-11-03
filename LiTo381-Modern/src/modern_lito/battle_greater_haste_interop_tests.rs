//! v42 opt-in TLS service/transport test, not a player UI or SQL-source claim.
use super::*;
use crate::modern_lito::{battle_greater_haste as haste, battle_window::Stamp};

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
    }
}

#[test]
#[ignore = "v42 actual two-source TLS with shared514 preflight, synthetic catalogs, no SQL or HWND"]
fn java_v42_greater_haste_tls_shared_load_cost_source_isolation_move_and_leave() {
    flow(42);
}
#[test]
#[ignore = "v43 actual two-source TLS, schema4 no-NPC state, shared514 load/cast/move/leave; no SQL or HWND"]
fn java_v43_greater_haste_tls_schema4_load_cast_move_and_leave() {
    flow(43);
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
    } = ready_pair_mode(version, version, Some("greater-haste"));
    assert!(a.supports_greater_haste());
    assert!(b.supports_greater_haste());
    assert!(a.skills().unwrap().greater_haste().is_some());
    assert!(b.skills().unwrap().greater_haste().is_none());
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
        deadline.pause("v42 preparation");
    }
    assert!(observation(&a).data().greater_haste());
    let early = a.greater_haste(intent(&a)).unwrap();
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
                BattleStatus::Failed(reason) => panic!("v42 shared preflight: {reason}"),
                _ => {}
            }
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            a.ping().unwrap();
            b.ping().unwrap();
            keepalive = Instant::now();
        }
        deadline.pause("v42 canonical LOAD qualification");
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
        deadline.pause("v42 countdown");
    }
    let read_before = a
        .battle_world(battle_world::Request::read(sun, CHANNEL, Some(&w)))
        .unwrap();
    let ack = a.ack().unwrap();
    let mut tracker = crate::modern_lito::battle_state::Tracker::default();
    tracker.accept_world(&read_before, ack).unwrap();
    let before = observation(&a);
    let q = intent(&a);
    let retained = a.battle_snapshot().unwrap().clone();
    let mut wrong = q;
    wrong.stamp.context.key.source_id = 42;
    assert!(a.greater_haste(wrong).is_err());
    assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
    a.ping().unwrap();
    let success = a.greater_haste(q).unwrap();
    assert_eq!(success.reason, haste::Reason::None);
    let retained_tracker = tracker.snapshot().unwrap().clone();
    let mut wrong_reply = success.clone();
    wrong_reply.echo.sequence += 1;
    assert!(tracker
        .accept_greater_haste(&wrong_reply, &success.echo, ack)
        .is_err());
    assert!(Arc::ptr_eq(&retained_tracker, tracker.snapshot().unwrap()));
    assert!(tracker
        .accept_greater_haste(
            &success,
            &success.echo,
            protocol::Ack {
                capabilities: ack.capabilities & !protocol::CAP_GREATER_HASTE,
                ..ack
            }
        )
        .is_err());
    assert!(Arc::ptr_eq(&retained_tracker, tracker.snapshot().unwrap()));
    tracker
        .accept_greater_haste(&success, &success.echo, ack)
        .unwrap();
    assert_eq!(
        tracker.snapshot().unwrap().observation().unwrap().data(),
        &success.data
    );
    assert_eq!(observation(&a).data(), &success.data);
    let o = observation(&a);
    let own = self_actor(&o);
    assert_eq!(own.vital.as_ref().unwrap().values.mp, 240);
    assert_eq!(self_actor(&before).vital.as_ref().unwrap().values.mp, 300);
    assert_eq!(
        own.buffs.as_ref().unwrap().haste_until_nanos,
        success.data.world.active_elapsed_nanos + 2_400_000_000_000
    );
    assert_eq!(
        a.greater_haste(intent(&a)).unwrap().reason,
        haste::Reason::TooFast
    );
    assert_eq!(
        self_actor(&observation(&a))
            .vital
            .as_ref()
            .unwrap()
            .values
            .mp,
        240
    );
    assert_eq!(
        b.greater_haste(intent(&b)).unwrap().reason,
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
    // Passive reads keep the accepted schema3 buff and cannot replace legacy caches.
    loop {
        refresh(&mut a, sun, &w);
        refresh(&mut b, moon, &w);
        if observation(&a).data().world.active_elapsed_nanos
            - success.data.world.active_elapsed_nanos
            >= 2_000_000_000
        {
            break;
        }
        deadline.pause("v42 authoritative cast recovery before MOVE");
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
    assert!(observation(&a).data().greater_haste());
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
        a.greater_haste(intent(&a)).unwrap().reason,
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
    assert!(a.greater_haste(old).is_err());
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
