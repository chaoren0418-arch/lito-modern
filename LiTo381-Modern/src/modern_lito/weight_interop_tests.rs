//! v9 authority weight over actual TLS, shared assets and isolated result SQL.
//! Synthetic source gear/rates; no production DB/store, native process or GUI.
use super::super::{
    equipment, item_protocol as item, shop_protocol as shop, transport::ModernTlsConnection,
};
use super::*;

const RED: i64 = 40010;

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
    assert_eq!(
        (
            client.ack().unwrap().version,
            client.ack().unwrap().capabilities
        ),
        (9, 2047)
    );
    client
        .authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    let roster = client.characters(key.source_id, 1).unwrap();
    assert_eq!(roster.characters[0].key, key);
    let context = SelectionContext { key, epoch: 1 };
    client.select_character(context).unwrap();
    assert_eq!(client.record().unwrap().totals.unwrap().revision, 0);
    let skills = client.skills().unwrap();
    assert_eq!(skills.status, spell::Status::Ok);
    if key == SUN {
        assert!(skills.teleport().is_some() && skills.light_arrow().is_some());
    }
    let gear = client
        .equipment(equipment::Request::read(context))
        .unwrap()
        .snapshot
        .unwrap();
    let w = gear.weight.as_ref().unwrap();
    assert_eq!(
        (w.source_gear_weight, w.supplies_weight, w.total, w.stage),
        (2575, 0, 2575, 196)
    );
    assert_eq!(w.maximum(), 3150.0);
    assert!(w.units.is_empty());
    client.enter_channel(CHANNEL).unwrap();
    (client, context)
}
fn read_shop(client: &mut ModernTlsConnection, context: SelectionContext) -> shop::Reply {
    let room = client
        .rooms(rooms::Request::read(context, CHANNEL).unwrap())
        .unwrap()
        .joined;
    client
        .shop(shop::Request::read(context, CHANNEL, room.as_ref()))
        .unwrap()
}
fn read_combat(
    client: &mut ModernTlsConnection,
    context: SelectionContext,
    frame: &world::Frame,
) -> combat::Frame {
    client
        .combat(combat::Request::read(context, CHANNEL, frame))
        .unwrap()
        .frame
        .unwrap()
}
fn cast_request(
    context: SelectionContext,
    frame: &combat::Frame,
    skill: u32,
    target: u64,
) -> spell::CastRequest {
    let id = actor(&frame.world, context.key).entity_id;
    spell::CastRequest {
        context,
        channel_id: CHANNEL,
        match_id: frame.world.match_id,
        generation: frame.world.generation,
        sequence: frame.world.cast(id).unwrap().request_sequence + 1,
        skill_id: skill,
        target_entity_id: target,
    }
}
fn pause_for(deadline: &Deadline, duration: Duration, stage: &'static str) {
    let until = Instant::now() + duration;
    while Instant::now() < until {
        deadline.pause(stage);
    }
}

#[test]
#[ignore = "requires v9 Java weight peer, explicit owned MySQL and real shared resources; no production SQL or GUI"]
fn java_weight_authority_buy_gate_full_hp_consumption_move_cast_result_and_sql() {
    let deadline = Deadline(Instant::now() + Duration::from_secs(60));
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernCombatTestPeer",
        "weight",
    );
    assert!(peer.process_job.is_some());
    let (mut sun, sun_context) = connect(&peer, &ca, SUN);
    let (mut moon, moon_context) = connect(&peer, &ca, MOON);
    let first = read_shop(&mut sun, sun_context);
    assert_eq!(first.status, shop::Status::Ok);
    let empty = first.snapshot.unwrap();
    assert_eq!(empty.balance, 20000);
    assert_eq!(empty.weight.as_ref().unwrap().total, 2575);
    let buy =
        shop::Request::read(sun_context, CHANNEL, None).mutation(&empty, shop::Action::Buy, RED, 2);
    let bought = sun.shop(buy).unwrap();
    assert_eq!(bought.status, shop::Status::Ok);
    let wallet = bought.snapshot.unwrap();
    let w = wallet.weight.as_ref().unwrap();
    assert_eq!((w.total, w.supplies_weight, w.stage), (2582, 7, 197));
    assert_eq!(w.units[0].unit_weight, 3900);
    assert_eq!(wallet.balance, 19920);
    let denied = sun
        .shop(shop::Request::read(sun_context, CHANNEL, None).mutation(
            &wallet,
            shop::Action::Buy,
            RED,
            400,
        ))
        .unwrap();
    assert_eq!(
        (denied.status, denied.reason),
        (shop::Status::Denied, shop::Reason::TooHeavy)
    );
    assert_eq!(
        denied.snapshot.as_ref(),
        Some(&wallet),
        "overweight admission cannot debit/revise/alter stacks"
    );
    for _ in 0..4 {
        assert_eq!(
            read_shop(&mut sun, sun_context).snapshot.as_ref(),
            Some(&wallet)
        );
    }
    assert_eq!(
        read_shop(&mut moon, moon_context).snapshot.unwrap().balance,
        20000
    );
    let created = sun
        .rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(sun_context, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "Weight authority TLS".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let room = created.joined.unwrap();
    assert_eq!(room.owner, SUN);
    let room_id = room.summary.room_id;
    let directory = moon
        .rooms(rooms::Request::read(moon_context, CHANNEL).unwrap())
        .unwrap();
    let summary = directory
        .rooms
        .iter()
        .find(|r| r.room_id == room_id)
        .unwrap();
    moon.rooms(
        rooms::Request::join(
            rooms::Common {
                context: moon_context,
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
        .rooms(rooms::Request::read(sun_context, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    sun.rooms(room_command(sun_context, &room, rooms::Action::Ready, 1))
        .unwrap();
    let room = moon
        .rooms(rooms::Request::read(moon_context, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    let ready = moon
        .rooms(room_command(moon_context, &room, rooms::Action::Ready, 1))
        .unwrap()
        .joined
        .unwrap();
    assert!(ready.summary.satisfied);
    let prepare = world::Request {
        context: sun_context,
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
    };
    let prepared = sun.world(prepare).unwrap();
    assert_eq!(prepared.status, world::Status::Ok);
    let mut wf = prepared.frame.unwrap();
    while wf.phase == world::Phase::Preparing {
        deadline.pause("weight fresh world qualification");
        wf = sun
            .world(world_request(sun_context, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    assert_eq!(wf.phase, world::Phase::Loading);
    moon.world(world_request(moon_context, &wf, world::Action::Read))
        .unwrap();
    let loading = Arc::new(read_combat(&mut sun, sun_context, &wf));
    assert_eq!(read_combat(&mut moon, moon_context, &wf), *loading);
    let sun_id = actor(&wf, SUN).entity_id;
    let moon_id = actor(&wf, MOON).entity_id;
    assert_eq!(loading.weight(sun_id), wallet.weight.as_ref());
    assert_eq!(loading.weights.len(), 2);
    assert_eq!(loading.vital(sun_id).unwrap().hp, 100);
    let mut sr = WorldRuntime::new(shared.clone()).unwrap();
    let mut mr = WorldRuntime::new(shared).unwrap();
    let mut keepalive = Instant::now();
    combat_images_with_progress(&mut sr, &mut mr, &loading, &deadline, || {
        if keepalive.elapsed() >= Duration::from_secs(1) {
            sun.ping().unwrap();
            moon.ping().unwrap();
            keepalive = Instant::now();
        }
    });
    sun.world(world::Request {
        manifest_digest: wf.manifest.digest,
        ..world_request(sun_context, &wf, world::Action::Loaded)
    })
    .unwrap();
    let started = Instant::now();
    wf = moon
        .world(world::Request {
            manifest_digest: wf.manifest.digest,
            ..world_request(moon_context, &wf, world::Action::Loaded)
        })
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(wf.phase, world::Phase::Countdown);
    while wf.phase != world::Phase::Active {
        deadline.pause("weight three-second countdown");
        wf = sun
            .world(world_request(sun_context, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    assert!(started.elapsed() >= Duration::from_secs(3));
    let mut current = read_combat(&mut sun, sun_context, &wf);
    let before = current.clone();
    let attack = sun
        .combat(combat::Request {
            action: combat::Action::Attack,
            sequence: 1,
            target_entity_id: moon_id,
            ..combat::Request::read(sun_context, CHANNEL, &current.world)
        })
        .unwrap();
    assert_eq!(
        (attack.status, attack.reason),
        (combat::Status::Denied, combat::Reason::TooHeavy)
    );
    current = attack.frame.unwrap();
    assert_eq!(current.vital(sun_id).unwrap().attack_sequence, 1);
    assert_eq!(current.vital(sun_id).unwrap().attack_revision, 0);
    assert_eq!(current.vital(moon_id), before.vital(moon_id));
    assert_eq!(current.weights, before.weights);
    for (skill, target) in [(spell::TELEPORT, 0), (spell::LIGHT_ARROW, moon_id)] {
        let reply = sun
            .cast(cast_request(sun_context, &current, skill, target))
            .unwrap();
        assert_eq!(
            (reply.status, reply.reason),
            (spell::Status::Denied, spell::Reason::TooHeavy)
        );
        current = reply.frame.unwrap();
        assert_eq!(current.vital(sun_id).unwrap().mp, 100);
        assert_eq!(current.world.cast(sun_id).unwrap().cast_count, 0);
        assert_eq!(current.damages, before.damages);
        assert_eq!(current.items, before.items);
        assert_eq!(current.weights, before.weights);
    }
    // Prove actual server movement is not gated by >=197. Failed geometry
    // attempts never become visual movement and always use the next confirmed sequence.
    let origin = actor(&current.world, SUN).position;
    let mut moved = None;
    for heading in 0..8 {
        let request = world::Request {
            heading,
            sequence: actor(&current.world, SUN).sequence + 1,
            from_x: origin.x,
            from_y: origin.y,
            ..world_request(sun_context, &current.world, world::Action::Move)
        };
        let reply = sun.world(request).unwrap();
        let next = reply.frame.unwrap();
        current = read_combat(&mut sun, sun_context, &next);
        if reply.status == world::Status::Ok {
            moved = Some(heading);
            break;
        }
    }
    let heading = moved.expect("real shared spawn has legal adjacent movement");
    assert_eq!(current.weight(sun_id).unwrap().stage, 197);
    pause_for(&deadline, Duration::from_millis(210), "weight MOVE pace");
    let position = actor(&current.world, SUN).position;
    let returned = sun
        .world(world::Request {
            heading: (heading + 4) % 8,
            sequence: actor(&current.world, SUN).sequence + 1,
            from_x: position.x,
            from_y: position.y,
            ..world_request(sun_context, &current.world, world::Action::Move)
        })
        .unwrap();
    assert_eq!(returned.status, world::Status::Ok);
    current = read_combat(&mut sun, sun_context, &returned.frame.unwrap());
    assert_eq!(actor(&current.world, SUN).position, origin);
    let healed = sun
        .use_item(item::Request {
            action: item::Action::Use,
            sequence: 1,
            item_id: RED,
            ..item::Request::read(sun_context, CHANNEL, &current.world)
        })
        .unwrap();
    assert_eq!(healed.status, item::Status::Ok);
    current = healed.frame.unwrap();
    let used = current.item(sun_id).unwrap();
    assert_eq!(
        (used.hp_before, used.hp_after, used.restored),
        (100, 100, 0)
    );
    assert_eq!(used.stock(RED).unwrap().remaining, 1);
    let w = current.weight(sun_id).unwrap();
    assert_eq!((w.total, w.supplies_weight, w.stage), (2578, 3, 196));
    assert_eq!(w.units, wallet.weight.as_ref().unwrap().units);
    pause_for(&deadline, Duration::from_millis(210), "post-move cast pace");
    let tp = sun
        .cast(cast_request(sun_context, &current, spell::TELEPORT, 0))
        .unwrap();
    assert_eq!(tp.status, spell::Status::Ok);
    current = tp.frame.unwrap();
    assert_eq!(current.vital(sun_id).unwrap().mp, 90);
    let tp_pos = actor(&current.world, SUN).position;
    let heading = DIRECTIONS
        .iter()
        .position(|&(dx, dy)| tp_pos.x + dx == origin.x && tp_pos.y + dy == origin.y)
        .expect("server fixture chooses adjacent teleport") as u8;
    pause_for(
        &deadline,
        Duration::from_millis(710),
        "original cast MOVE pace",
    );
    let moved_back = sun
        .world(world::Request {
            heading,
            sequence: actor(&current.world, SUN).sequence + 1,
            from_x: tp_pos.x,
            from_y: tp_pos.y,
            ..world_request(sun_context, &current.world, world::Action::Move)
        })
        .unwrap();
    assert_eq!(moved_back.status, world::Status::Ok);
    current = read_combat(&mut sun, sun_context, &moved_back.frame.unwrap());
    pause_for(
        &deadline,
        Duration::from_millis(210),
        "weight attack after MOVE",
    );
    let attack = sun
        .combat(combat::Request {
            action: combat::Action::Attack,
            sequence: 2,
            target_entity_id: moon_id,
            ..combat::Request::read(sun_context, CHANNEL, &current.world)
        })
        .unwrap();
    assert_eq!(attack.status, combat::Status::Ok);
    current = attack.frame.unwrap();
    assert_eq!(current.vital(sun_id).unwrap().attack_sequence, 2);
    assert!(current.vital(sun_id).unwrap().attack_revision > 0);
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    let mut casts = 0;
    while current.world.phase == world::Phase::Active {
        assert!(casts < 24);
        pause_for(
            &deadline,
            Duration::from_millis(1010),
            "weight original combat cast pace",
        );
        let reply = sun
            .cast(cast_request(
                sun_context,
                &current,
                spell::LIGHT_ARROW,
                moon_id,
            ))
            .unwrap();
        assert_eq!(reply.status, spell::Status::Ok);
        current = reply.frame.unwrap();
        assert_eq!(current.weight(sun_id).unwrap().total, 2578);
        casts += 1;
        let other = read_combat(&mut moon, moon_context, &current.world);
        assert_eq!(other.weights, current.weights);
        assert_eq!(other.vitals, current.vitals);
    }
    assert_eq!(current.world.phase, world::Phase::Finished);
    assert!(casts > 0);
    while current.world.completion.as_ref().unwrap().state != world::SettlementState::Saved {
        deadline.pause("weight durable SQL receipt");
        current = read_combat(&mut sun, sun_context, &current.world);
    }
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    let won = sun.record().unwrap().totals.unwrap();
    let lost = moon.record().unwrap().totals.unwrap();
    assert_eq!((won.wins, won.losses, won.revision), (1, 0, 1));
    assert_eq!((lost.wins, lost.losses, lost.revision), (0, 1, 1));
    assert_eq!(sun.record().unwrap().totals, Some(won));
    sun.world(world_request(
        sun_context,
        &current.world,
        world::Action::Leave,
    ))
    .unwrap();
    moon.world(world_request(
        moon_context,
        &current.world,
        world::Action::Read,
    ))
    .unwrap();
    moon.world(world_request(
        moon_context,
        &current.world,
        world::Action::Leave,
    ))
    .unwrap();
    let reset = read_shop(&mut sun, sun_context).snapshot.unwrap();
    assert_eq!((reset.round_id, reset.balance), (2, 20000));
    assert!(reset.stacks.is_empty());
    assert_eq!(reset.weight.as_ref().unwrap().total, 2575);
    for _ in 0..2 {
        assert_eq!(
            read_shop(&mut sun, sun_context).snapshot.as_ref(),
            Some(&reset)
        );
    }
    sr.shutdown();
    mr.shutdown();
    while !sr.is_quiescent() || !mr.is_quiescent() {
        sr.poll();
        mr.poll();
        deadline.pause("weight owned render retirement");
    }
    sun.close().unwrap();
    moon.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
