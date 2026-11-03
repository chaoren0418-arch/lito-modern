//! True Rust -> authenticated Main -> isolated SQL; no internal server probes.
//! Shared assets qualify LOAD. Slow effect pixels and native HWND are not claimed.
use super::*;
use crate::modern_lito::{
    battle_control::{Command as GameCommand, Outcome, State as GameState},
    battle_slow as slow,
    battle_state::Tracker,
    battle_window::Target,
    shop_protocol as shop,
};
fn observation(c: &ModernTlsConnection) -> crate::modern_lito::battle_follow::Observation {
    c.battle_snapshot().unwrap().observation().unwrap().clone()
}
fn own(o: &crate::modern_lito::battle_follow::Observation) -> &data::Actor {
    o.data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .unwrap()
}
fn materials(c: &ModernTlsConnection) -> crate::modern_lito::battle_payload::speed::Materials {
    observation(c).data().personal.materials.clone().unwrap()
}
fn refresh(c: &mut ModernTlsConnection, ctx: SelectionContext, w: &data::WorldId) {
    assert_eq!(
        c.battle_world(battle_world::Request::read(ctx, CHANNEL, Some(w)))
            .unwrap()
            .status,
        world::Status::Ok
    );
    assert!(c.world_snapshot().is_none());
}
fn intent(c: &ModernTlsConnection, target_source: i32) -> slow::Intent {
    let o = observation(c);
    let member = o
        .data()
        .room
        .members
        .0
        .iter()
        .position(|m| m.identity.source == target_source)
        .unwrap();
    let target = &o.data().actors.0[member];
    slow::Intent::from_observation(
        &o,
        Target::Player {
            entity: target.entity,
            life: target.resources.as_ref().unwrap().life + 1,
        },
    )
    .unwrap()
}
fn read_shop(c: &mut ModernTlsConnection, ctx: SelectionContext) -> shop::Reply {
    c.shop(shop::Request::read(ctx, CHANNEL, None)).unwrap()
}
#[test]
#[ignore = "v47 real Rust/Main/TLS/owned SQL/shared514 LOAD: gems, Slow receipts, atomic denial, source return; no Slow pixels or HWND"]
fn java_v47_slow_main_sql_materials_cast_atomic_state_and_source_return() {
    let deadline = Deadline(Instant::now() + Duration::from_secs(150));
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernSlowSqlTestPeer",
        "slow-sql",
        Some("150000"),
    );
    let ca = private_root(&fixtures);
    let sun = SelectionContext { key: SUN, epoch: 1 };
    let moon = SelectionContext {
        key: MOON,
        epoch: 1,
    };
    let mut a = connect(&peer, &ca, sun, 47);
    let mut b = connect(&peer, &ca, moon, 47);
    assert!(a.supports_slow() && b.supports_slow());
    assert!(a.skills().unwrap().slow().is_some());
    assert!(b.skills().unwrap().slow().is_none());
    for (c, ctx) in [(&mut a, sun), (&mut b, moon)] {
        let initial = read_shop(c, ctx);
        let s = initial.snapshot.as_ref().unwrap();
        assert!(s.materials_enabled && s.purchasable(40318));
        let gem = s.offer(40318).unwrap();
        assert_eq!(
            (
                gem.name.as_str(),
                gem.inventory_graphic,
                gem.ground_graphic,
                gem.weight
            ),
            ("$508", 235, 773, 2000)
        );
        let q = shop::Request::read(ctx, CHANNEL, None).mutation(s, shop::Action::Buy, 40318, 4);
        let bought = c.shop(q).unwrap();
        assert_eq!(bought.status, shop::Status::Ok);
        let next = bought.snapshot.unwrap();
        assert_eq!(next.balance, s.balance - 400);
        assert_eq!(next.stack(40318).unwrap().quantity, 4);
    }
    let created = a
        .rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(sun, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 0,
                        map_id: 514,
                        name: "Slow SQL".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
        .joined
        .unwrap();
    let joined = b
        .rooms(
            rooms::Request::join(
                rooms::Common {
                    context: moon,
                    channel_id: CHANNEL,
                    action: rooms::Action::Join,
                    value: 0,
                    room_id: created.summary.room_id,
                    expected_revision: created.summary.revision,
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap()
        .joined
        .unwrap();
    let teams = b
        .rooms(room_command(moon, &joined, rooms::Action::Team, 1))
        .unwrap()
        .joined
        .unwrap();
    let ready = a
        .rooms(room_command(sun, &teams, rooms::Action::Ready, 1))
        .unwrap()
        .joined
        .unwrap();
    let ready = b
        .rooms(room_command(moon, &ready, rooms::Action::Ready, 1))
        .unwrap()
        .joined
        .unwrap();
    assert!(ready.summary.satisfied);
    let prepared = a
        .battle_world(battle_world::Request {
            action: battle_world::Action::Prepare,
            room: ready.summary.room_id as i64,
            room_revision: ready.summary.revision,
            ..battle_world::Request::read(sun, CHANNEL, None)
        })
        .unwrap();
    let w = prepared.attachment.unwrap().world;
    loop {
        refresh(&mut a, sun, &w);
        refresh(&mut b, moon, &w);
        if a.battle_snapshot().unwrap().observation().is_some()
            && b.battle_snapshot().unwrap().observation().is_some()
        {
            break;
        }
        deadline.pause("Slow source preparation");
    }
    assert!(observation(&a).data().speed.is_some());
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
                BattleStatus::Failed(reason) => panic!("Slow shared LOAD failed: {reason}"),
                _ => {}
            }
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            a.ping().unwrap();
            b.ping().unwrap();
            keepalive = Instant::now();
        }
        deadline.pause("Slow shared LOAD");
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
        deadline.pause("Slow countdown");
    }
    let baseline = a
        .battle_world(battle_world::Request::read(sun, CHANNEL, Some(&w)))
        .unwrap();
    let before = observation(&a);
    let q = intent(&a, 42);
    let retained = a.battle_snapshot().unwrap().clone();
    let mut state = GameState::default();
    let mut stale = q;
    // A future proof must not be sent, consume a sequence or charge materials.
    stale.stamp.revision += 1;
    state.execute(&mut a, GameCommand::slow(stale), 1).unwrap();
    assert_eq!(state.outcome(), Some(Outcome::NotSent));
    assert!(state.slow_supported());
    assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
    let mut wrong = q;
    wrong.stamp.context.key.source_id = 42;
    assert!(a.slow(wrong).is_err());
    assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
    let success = a.slow(q).unwrap();
    assert_eq!(success.reason, slow::Reason::None);
    let expected = slow::Intent {
        request_id: success.echo.request_id,
        ..q
    };
    assert_eq!(success.echo, expected);
    success.validate_after(&expected, before.data()).unwrap();
    assert_eq!(own(&observation(&a)).vital.as_ref().unwrap().values.mp, 285);
    assert_eq!(
        (materials(&a).uses, materials(&a).stocks.0[0].remaining),
        (1, 3)
    );
    assert_eq!(
        (materials(&b).uses, materials(&b).stocks.0[0].remaining),
        (0, 4)
    );
    let mut tracker = Tracker::default();
    tracker.accept_world(&baseline, a.ack().unwrap()).unwrap();
    let bads = [
        {
            let mut x = success.clone();
            x.reason = slow::Reason::NoMaterial;
            x
        },
        {
            let mut x = success.clone();
            x.data.personal.materials.as_mut().unwrap().stocks.0[0].remaining += 1;
            x
        },
        {
            let mut x = success.clone();
            x.data.personal.materials.as_mut().unwrap().last_sequence += 1;
            x
        },
        {
            let mut x = success.clone();
            x.echo.stamp.context.key.source_id = 42;
            x
        },
        {
            let mut x = success.clone();
            x.echo.sequence += 1;
            x
        },
    ];
    for bad in bads {
        let mut check = tracker.clone();
        let old = check.snapshot().unwrap().clone();
        assert!(check
            .accept_slow(&bad, &expected, a.ack().unwrap())
            .is_err());
        assert!(Arc::ptr_eq(&old, check.snapshot().unwrap()));
    }
    for ack in [
        protocol::Ack {
            version: 46,
            ..a.ack().unwrap()
        },
        protocol::Ack {
            capabilities: a.ack().unwrap().capabilities & !protocol::CAP_SLOW_DATA,
            ..a.ack().unwrap()
        },
    ] {
        let mut check = tracker.clone();
        let old = check.snapshot().unwrap().clone();
        assert!(check.accept_slow(&success, &expected, ack).is_err());
        assert!(Arc::ptr_eq(&old, check.snapshot().unwrap()));
    }
    tracker
        .accept_slow(&success, &expected, a.ack().unwrap())
        .unwrap();
    let after = a.battle_snapshot().unwrap().clone();
    assert!(a.slow(q).is_err());
    assert!(Arc::ptr_eq(&after, a.battle_snapshot().unwrap()));
    refresh(&mut b, moon, &w);
    let denied = b.slow(intent(&b, 7)).unwrap();
    assert_eq!(denied.reason, slow::Reason::Unlearned);
    assert_eq!(
        (materials(&b).uses, materials(&b).stocks.0[0].remaining),
        (0, 4)
    );
    let mut last_elapsed = success.data.world.active_elapsed_nanos;
    for count in 2..=4 {
        loop {
            refresh(&mut a, sun, &w);
            if observation(&a).data().world.active_elapsed_nanos - last_elapsed >= 2_000_000_000 {
                break;
            }
            deadline.pause("Slow cast cadence");
        }
        let q = intent(&a, 42);
        state
            .execute(&mut a, GameCommand::slow(q), count as u64)
            .unwrap();
        assert_eq!(state.outcome(), Some(Outcome::Slow(slow::Reason::None)));
        assert_eq!(
            (materials(&a).uses, materials(&a).stocks.0[0].remaining),
            (count, 4 - count)
        );
        assert_eq!(
            own(&observation(&a)).vital.as_ref().unwrap().values.mp,
            300 - count * 15
        );
        last_elapsed = observation(&a).data().world.active_elapsed_nanos;
    }
    loop {
        refresh(&mut a, sun, &w);
        if observation(&a).data().world.active_elapsed_nanos - last_elapsed >= 2_000_000_000 {
            break;
        }
        deadline.pause("Slow exhausted material cadence");
    }
    let spent = materials(&a);
    assert_eq!(
        a.slow(intent(&a, 42)).unwrap().reason,
        slow::Reason::NoMaterial
    );
    assert_eq!(materials(&a), spent);
    assert_eq!(own(&observation(&a)).vital.as_ref().unwrap().values.mp, 240);
    for runtime in &mut runtimes {
        runtime.shutdown();
        join_worker(runtime, &deadline);
    }
    for (c, ctx) in [(&mut a, sun), (&mut b, moon)] {
        c.battle_world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(ctx, CHANNEL, Some(&w))
        })
        .unwrap();
        let room = c
            .rooms(rooms::Request::read(ctx, CHANNEL).unwrap())
            .unwrap()
            .joined
            .unwrap();
        c.rooms(room_command(ctx, &room, rooms::Action::Leave, 0))
            .unwrap();
        loop {
            let reply = read_shop(c, ctx);
            if reply.status == shop::Status::Ok {
                assert!(reply.snapshot.unwrap().stacks.is_empty());
                break;
            }
            assert_eq!(reply.status, shop::Status::Frozen);
            deadline.pause("Slow shop release");
        }
        c.leave_lobby().unwrap();
        c.return_characters().unwrap();
    }
    for (c, ctx) in [
        (
            &mut a,
            SelectionContext {
                key: MOON,
                epoch: 2,
            },
        ),
        (&mut b, SelectionContext { key: SUN, epoch: 2 }),
    ] {
        assert_eq!(
            c.characters(ctx.key.source_id, ctx.epoch)
                .unwrap()
                .characters[0]
                .key,
            ctx.key
        );
        c.select_character(ctx).unwrap();
        c.enter_channel(CHANNEL).unwrap();
        assert_eq!(c.skills().unwrap().slow().is_some(), ctx.key.source_id == 7);
        assert!(read_shop(c, ctx).snapshot.unwrap().stacks.is_empty());
        c.leave_lobby().unwrap();
        c.return_characters().unwrap();
        c.close().unwrap();
    }
    peer.expect_successful_exit(&deadline);
}
