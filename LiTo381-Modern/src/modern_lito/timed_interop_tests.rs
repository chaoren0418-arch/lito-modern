//! v10 four timed supplies over owned TLS, actual shared rendering and isolated result SQL.
//! Source rows are explicitly synthetic mage/elf fixtures. No GUI or production writes.
use super::super::{
    equipment, item_protocol as item, shop_protocol as shop, timed_protocol as timed,
    transport::ModernTlsConnection,
};
use super::*;

fn connect(
    peer: &JavaPeer,
    ca: &[u8],
    key: CharacterKey,
) -> (ModernTlsConnection, SelectionContext) {
    let mut client = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(ca),
        Hello::current(100),
        TransportLimits {
            connect_timeout: Duration::from_millis(500),
            total_timeout: Duration::from_secs(2),
            io_poll_interval: Duration::from_millis(25),
        },
        Cancellation::new(),
    )
    .unwrap();
    let ack = client.ack().unwrap();
    assert_eq!((ack.version, ack.capabilities), (10, 4095));
    client
        .authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    let roster = client.characters(key.source_id, 1).unwrap();
    assert_eq!(roster.characters[0].key, key);
    let context = SelectionContext { key, epoch: 1 };
    client.select_character(context).unwrap();
    assert_eq!(client.record().unwrap().totals.unwrap().revision, 0);
    let skills = client.skills().unwrap();
    if key == SUN {
        assert!(skills.teleport().is_some() && skills.light_arrow().is_some());
    }
    let gear = client
        .equipment(equipment::Request::read(context))
        .unwrap()
        .snapshot
        .unwrap();
    assert_eq!(gear.weight.as_ref().unwrap().source_gear_weight, 2575);
    client.enter_channel(CHANNEL).unwrap();
    (client, context)
}
fn shop_read(client: &mut ModernTlsConnection, context: SelectionContext) -> Arc<shop::Snapshot> {
    let room = client
        .rooms(rooms::Request::read(context, CHANNEL).unwrap())
        .unwrap()
        .joined;
    client
        .shop(shop::Request::read(context, CHANNEL, room.as_ref()))
        .unwrap()
        .snapshot
        .unwrap()
}
fn combat_read(
    client: &mut ModernTlsConnection,
    context: SelectionContext,
    world: &world::Frame,
) -> combat::Frame {
    client
        .combat(combat::Request::read(context, CHANNEL, world))
        .unwrap()
        .frame
        .unwrap()
}
fn track(
    previous: &combat::Frame,
    next: combat::Frame,
    context: SelectionContext,
) -> combat::Frame {
    let reply = |frame: &combat::Frame| combat::Reply {
        echo: combat::Request::read(context, CHANNEL, &frame.world),
        status: combat::Status::Ok,
        reason: combat::Reason::None,
        frame: Some(frame.clone()),
    };
    assert!(
        combat::follows(Some(&reply(previous)), &reply(&next)),
        "complete client history must accept genuine timed transition"
    );
    next
}
fn use_item(
    client: &mut ModernTlsConnection,
    context: SelectionContext,
    previous: &combat::Frame,
    id: i64,
    expected: item::Reason,
) -> combat::Frame {
    // Each real connection must confirm its own atomic world/combat view. A
    // frame seen by the other TLS owner cannot stand in for this owner's ACK.
    let confirmed = track(
        previous,
        combat_read(client, context, &previous.world),
        context,
    );
    let previous = &confirmed;
    let entity = actor(&previous.world, context.key).entity_id;
    let before = previous.item(entity).unwrap();
    let request = item::Request {
        action: item::Action::Use,
        sequence: before.request_sequence + 1,
        item_id: id,
        ..item::Request::read(context, CHANNEL, &previous.world)
    };
    let reply = client
        .use_item(request)
        .unwrap_or_else(|error| panic!("item {id} source {}: {error}", context.key.source_id));
    assert_eq!(reply.reason, expected);
    assert_eq!(
        reply.status,
        if expected == item::Reason::None {
            item::Status::Ok
        } else {
            item::Status::Denied
        }
    );
    let next = track(previous, reply.frame.unwrap(), context);
    let after = next.item(entity).unwrap();
    assert_eq!(after.request_sequence, before.request_sequence + 1);
    if expected == item::Reason::None {
        assert_eq!(after.use_count, before.use_count + 1);
        assert_eq!(
            after.stock(id).unwrap().remaining + 1,
            before.stock(id).unwrap().remaining
        );
        if timed::Kind::from_item(id).is_some() {
            assert_eq!(after.hp_before, after.hp_after);
            assert_eq!(after.restored, 0);
        }
    } else {
        assert_eq!(after.stocks, before.stocks);
        assert_eq!(after.use_count, before.use_count);
        assert_eq!(next.weights, previous.weights);
        assert_eq!(next.timed, previous.timed);
    }
    next
}
fn pause(deadline: &Deadline, duration: Duration, stage: &'static str) {
    let until = Instant::now() + duration;
    while Instant::now() < until {
        deadline.pause(stage);
    }
}

#[test]
#[ignore = "v10 Java timed peer, explicit owned MySQL, real shared 191/750/751/121/142 assets; no GUI or production SQL"]
fn java_four_timed_supplies_class_refresh_pace_magic_render_and_sql() {
    let deadline = Deadline(Instant::now() + Duration::from_secs(60));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernCombatTestPeer",
        "timed",
    );
    assert!(peer.process_job.is_some());
    let (mut sun, sc) = connect(&peer, &ca, SUN);
    let (mut moon, mc) = connect(&peer, &ca, MOON);
    let mut sw = shop_read(&mut sun, sc);
    let mut mw = shop_read(&mut moon, mc);
    assert_eq!(sw.offers.len(), 7);
    assert_eq!(mw.offers.len(), 7);
    for id in [40010, 40011, 40012, 40013, 40014, 40016, 40068] {
        for (client, context, wallet) in [(&mut sun, sc, &mut sw), (&mut moon, mc, &mut mw)] {
            let offer = wallet.offer(id).unwrap();
            if let Some(kind) = timed::Kind::from_item(id) {
                assert_eq!(offer.support, shop::Support::Timed);
                assert_eq!(
                    wallet.timed_offer(id).unwrap().eligible,
                    kind.eligible(if context.key == SUN { 3 } else { 2 })
                );
            }
            let request = shop::Request::read(context, CHANNEL, None).mutation(
                wallet,
                shop::Action::Buy,
                id,
                2,
            );
            let bought = client.shop(request).unwrap();
            assert_eq!(bought.status, shop::Status::Ok);
            *wallet = bought.snapshot.unwrap();
        }
    }
    assert_eq!(sw.weight.as_ref().unwrap().units.len(), 7);
    for _ in 0..4 {
        assert_eq!(shop_read(&mut sun, sc), sw);
    }
    assert_ne!(
        sw.balance, mw.balance,
        "source-specific healing prices remain isolated"
    );
    let created = sun
        .rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(sc, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "Timed supplies TLS".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let room = created.joined.unwrap();
    let room_id = room.summary.room_id;
    let directory = moon
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap();
    let summary = directory
        .rooms
        .iter()
        .find(|r| r.room_id == room_id)
        .unwrap();
    moon.rooms(
        rooms::Request::join(
            rooms::Common {
                context: mc,
                channel_id: CHANNEL,
                action: rooms::Action::Join,
                value: 0,
                room_id,
                expected_revision: summary.revision,
            },
            String::new(),
        )
        .unwrap(),
    )
    .unwrap();
    let room = sun
        .rooms(rooms::Request::read(sc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    sun.rooms(room_command(sc, &room, rooms::Action::Ready, 1))
        .unwrap();
    let room = moon
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    let ready = moon
        .rooms(room_command(mc, &room, rooms::Action::Ready, 1))
        .unwrap()
        .joined
        .unwrap();
    assert!(ready.summary.satisfied);
    let mut wf = sun
        .world(world::Request {
            context: sc,
            channel_id: CHANNEL,
            action: world::Action::Prepare,
            heading: 0,
            room_id,
            expected_room_revision: ready.summary.revision,
            match_id: 0,
            generation: 0,
            sequence: 0,
            from_x: 0,
            from_y: 0,
            manifest_digest: [0; 32],
        })
        .unwrap()
        .frame
        .unwrap();
    while wf.phase == world::Phase::Preparing {
        deadline.pause("timed fresh preparation");
        wf = sun
            .world(world_request(sc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    assert_eq!(wf.phase, world::Phase::Loading);
    moon.world(world_request(mc, &wf, world::Action::Read))
        .unwrap();
    let initial = Arc::new(combat_read(&mut sun, sc, &wf));
    let sid = actor(&wf, SUN).entity_id;
    let mid = actor(&wf, MOON).entity_id;
    assert_eq!(
        initial.timed,
        vec![timed::State::initial(sid), timed::State::initial(mid)]
    );
    assert_eq!(initial.weight(sid), sw.weight.as_ref());
    assert_eq!(initial.weight(mid), mw.weight.as_ref());
    let mut sr = WorldRuntime::new(shared.clone()).unwrap();
    let mut mr = WorldRuntime::new(shared).unwrap();
    let mut keepalive = Instant::now();
    combat_images_with_progress(&mut sr, &mut mr, &initial, &deadline, || {
        if keepalive.elapsed() >= Duration::from_secs(1) {
            sun.ping().unwrap();
            moon.ping().unwrap();
            keepalive = Instant::now();
        }
    });
    sun.world(world::Request {
        manifest_digest: wf.manifest.digest,
        ..world_request(sc, &wf, world::Action::Loaded)
    })
    .unwrap();
    let countdown = Instant::now();
    wf = moon
        .world(world::Request {
            manifest_digest: wf.manifest.digest,
            ..world_request(mc, &wf, world::Action::Loaded)
        })
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(wf.phase, world::Phase::Countdown);
    while wf.phase != world::Phase::Active {
        deadline.pause("timed full countdown");
        wf = sun
            .world(world_request(sc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    assert!(countdown.elapsed() >= Duration::from_secs(3));
    let mut current = track(&initial, combat_read(&mut sun, sc, &wf), sc);
    // A zero-delay timed use cannot reset the separate previous healing-group clock.
    current = use_item(&mut sun, sc, &current, 40010, item::Reason::None);
    current = use_item(&mut sun, sc, &current, 40013, item::Reason::None);
    current = use_item(&mut sun, sc, &current, 40010, item::Reason::TooFast);
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    assert_eq!(
        current
            .timed(sid)
            .unwrap()
            .effective_nanos(200_000_000, timed::Pace::Move),
        151_000_000
    );
    current = use_item(&mut sun, sc, &current, 40014, item::Reason::None);
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    current = use_item(&mut sun, sc, &current, 40016, item::Reason::None);
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    let first_wisdom = current.timed(sid).unwrap().wisdom_until_nanos;
    current = use_item(&mut sun, sc, &current, 40016, item::Reason::None);
    assert!(current.timed(sid).unwrap().wisdom_until_nanos >= first_wisdom);
    assert_eq!(current.timed(sid).unwrap().spell_power_bonus(), 2);
    assert_eq!(
        current
            .timed(sid)
            .unwrap()
            .effective_nanos(200_000_000, timed::Pace::Move),
        114_000_000
    );
    current = use_item(&mut sun, sc, &current, 40068, item::Reason::ClassRestricted);
    current = use_item(
        &mut moon,
        mc,
        &current,
        40016,
        item::Reason::ClassRestricted,
    );
    current = use_item(&mut moon, mc, &current, 40013, item::Reason::None);
    current = use_item(&mut moon, mc, &current, 40068, item::Reason::None);
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    assert_eq!(current.timed(mid).unwrap().brave_kind, 3);
    assert_eq!(
        current
            .timed(mid)
            .unwrap()
            .effective_nanos(1_000_000_000, timed::Pace::Attack),
        438_000_000
    );
    current = use_item(&mut moon, mc, &current, 40014, item::Reason::None);
    assert_eq!(current.timed(mid).unwrap().brave_kind, 1);
    current = use_item(&mut moon, mc, &current, 40068, item::Reason::None);
    assert_eq!(current.timed(mid).unwrap().brave_kind, 3);
    assert_eq!(
        current.timed(mid).unwrap().brave_until_nanos - current.timed(mid).unwrap().observed_nanos,
        480_000_000_000
    );
    assert_eq!(
        actor(&current.world, SUN).step_interval_nanos,
        200_000_000,
        "WORLD keeps the immutable base"
    );
    let origin = actor(&current.world, SUN).position;
    let mut direction = None;
    for heading in 0..8 {
        let reply = sun
            .world(world::Request {
                heading,
                sequence: actor(&current.world, SUN).sequence + 1,
                from_x: origin.x,
                from_y: origin.y,
                ..world_request(sc, &current.world, world::Action::Move)
            })
            .unwrap();
        current = track(
            &current,
            combat_read(&mut sun, sc, &reply.frame.unwrap()),
            sc,
        );
        if reply.status == world::Status::Ok {
            direction = Some(heading);
            break;
        }
    }
    let heading = direction.expect("qualified shared map permits adjacent step");
    pause(
        &deadline,
        Duration::from_millis(125),
        "timed shorter confirmed movement interval",
    );
    let position = actor(&current.world, SUN).position;
    let back = sun
        .world(world::Request {
            heading: (heading + 4) % 8,
            sequence: actor(&current.world, SUN).sequence + 1,
            from_x: position.x,
            from_y: position.y,
            ..world_request(sc, &current.world, world::Action::Move)
        })
        .unwrap();
    assert_eq!(back.status, world::Status::Ok);
    current = track(
        &current,
        combat_read(&mut sun, sc, &back.frame.unwrap()),
        sc,
    );
    assert_eq!(actor(&current.world, SUN).position, origin);
    let mut shots = 0;
    while current.world.phase == world::Phase::Active {
        assert!(shots < 24);
        pause(
            &deadline,
            Duration::from_millis(420),
            "timed authoritative cast interval",
        );
        let request = spell::CastRequest {
            context: sc,
            channel_id: CHANNEL,
            match_id: current.world.match_id,
            generation: current.world.generation,
            sequence: current.world.cast(sid).unwrap().request_sequence + 1,
            skill_id: spell::LIGHT_ARROW,
            target_entity_id: mid,
        };
        let reply = sun.cast(request).unwrap();
        assert_eq!(reply.status, spell::Status::Ok);
        current = track(&current, reply.frame.unwrap(), sc);
        shots += 1;
        let other = combat_read(&mut moon, mc, &current.world);
        assert_eq!(other.vitals, current.vitals);
        assert_eq!(other.timed, current.timed);
    }
    assert!(shots > 0);
    assert_eq!(current.world.phase, world::Phase::Finished);
    assert!(
        current
            .timed
            .iter()
            .all(|s| !s.haste() && !s.brave() && !s.wisdom()),
        "terminal clears every timed lease"
    );
    while current.world.completion.as_ref().unwrap().state != world::SettlementState::Saved {
        deadline.pause("timed durable result receipt");
        current = track(&current, combat_read(&mut sun, sc, &current.world), sc);
    }
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    let won = sun.record().unwrap().totals.unwrap();
    let lost = moon.record().unwrap().totals.unwrap();
    assert_eq!((won.wins, won.losses, won.revision), (1, 0, 1));
    assert_eq!((lost.wins, lost.losses, lost.revision), (0, 1, 1));
    assert_eq!(sun.record().unwrap().totals, Some(won));
    sun.world(world_request(sc, &current.world, world::Action::Leave))
        .unwrap();
    moon.world(world_request(mc, &current.world, world::Action::Read))
        .unwrap();
    moon.world(world_request(mc, &current.world, world::Action::Leave))
        .unwrap();
    let reset = shop_read(&mut sun, sc);
    assert_eq!((reset.round_id, reset.balance), (2, 20000));
    assert!(reset.stacks.is_empty());
    for _ in 0..2 {
        assert_eq!(shop_read(&mut sun, sc), reset);
    }
    sr.shutdown();
    mr.shutdown();
    while !sr.is_quiescent() || !mr.is_quiescent() {
        sr.poll();
        mr.poll();
        deadline.pause("timed render retirement");
    }
    sun.close().unwrap();
    moon.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
