//! v12 signed natural recovery with true retained seconds, TLS and isolated SQL.
//! Source facts/RNG are explicit fixtures; shared resources are real. No GUI.
use super::super::{
    ammo_protocol as ammo, equipment, item_protocol as item, shop_protocol as shop,
    transport::ModernTlsConnection,
};
use super::*;

fn connect(
    peer: &JavaPeer,
    ca: &[u8],
    key: CharacterKey,
) -> (ModernTlsConnection, SelectionContext) {
    let mut c = connect_and_negotiate(
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
        (c.ack().unwrap().version, c.ack().unwrap().capabilities),
        (12, 16383)
    );
    c.authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    let roster = c.characters(key.source_id, 1).unwrap();
    assert_eq!(roster.characters[0].key, key);
    let context = SelectionContext { key, epoch: 1 };
    c.select_character(context).unwrap();
    assert_eq!(c.record().unwrap().totals.unwrap().revision, 0);
    c.skills().unwrap();
    let e = c
        .equipment(equipment::Request::read(context))
        .unwrap()
        .snapshot
        .unwrap();
    assert_eq!(
        e.weight.as_ref().unwrap().source_gear_weight,
        if key == SUN { 2576 } else { 2575 }
    );
    c.enter_channel(CHANNEL).unwrap();
    (c, context)
}
fn shop_read(c: &mut ModernTlsConnection, context: SelectionContext) -> Arc<shop::Snapshot> {
    let room = c
        .rooms(rooms::Request::read(context, CHANNEL).unwrap())
        .unwrap()
        .joined;
    c.shop(shop::Request::read(context, CHANNEL, room.as_ref()))
        .unwrap()
        .snapshot
        .unwrap()
}
fn read(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    world: &world::Frame,
) -> combat::Frame {
    c.combat(combat::Request::read(context, CHANNEL, world))
        .unwrap()
        .frame
        .unwrap()
}
fn track(old: &combat::Frame, new: combat::Frame, context: SelectionContext) -> combat::Frame {
    let reply = |f: &combat::Frame| combat::Reply {
        echo: combat::Request::read(context, CHANNEL, &f.world),
        status: combat::Status::Ok,
        reason: combat::Reason::None,
        frame: Some(f.clone()),
    };
    assert!(
        combat::follows(Some(&reply(old)), &reply(&new)),
        "genuine complete natural-recovery transition"
    );
    new
}
fn pause(deadline: &Deadline, duration: Duration) {
    let until = Instant::now() + duration;
    while Instant::now() < until {
        deadline.pause("bow fixture server cadence");
    }
}

fn cast(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    old: &combat::Frame,
    target: u64,
    reason: spell::Reason,
) -> combat::Frame {
    let current = track(old, read(c, context, &old.world), context);
    let entity = actor(&current.world, context.key).entity_id;
    let reply = c
        .cast(spell::CastRequest {
            context,
            channel_id: CHANNEL,
            match_id: current.world.match_id,
            generation: current.world.generation,
            sequence: current.world.cast(entity).unwrap().request_sequence + 1,
            skill_id: spell::LIGHT_ARROW,
            target_entity_id: target,
        })
        .unwrap();
    assert_eq!(reply.reason, reason);
    assert_eq!(
        reply.status,
        if reason == spell::Reason::None {
            spell::Status::Ok
        } else {
            spell::Status::Denied
        }
    );
    let next = track(&current, reply.frame.unwrap(), context);
    if reason != spell::Reason::None {
        assert_eq!(next.damages, current.damages);
        // A denied action spends nothing; independently due HP recovery can
        // still arrive in its authoritative reply and is checked by track().
        assert_eq!(
            next.vital(entity).unwrap().mp,
            current.vital(entity).unwrap().mp
        );
        assert_eq!(next.items, current.items);
        assert_eq!(next.ammo, current.ammo);
    }
    next
}
#[test]
#[ignore = "v12 Java regen peer + isolated MySQL + real shared resources; true 16-second MPR, no GUI"]
fn java_natural_hp_signed_mp_healing_spent_above_max_render_and_sql() {
    let deadline = Deadline(Instant::now() + Duration::from_secs(60));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernCombatTestPeer",
        "regen",
    );
    assert!(peer.process_job.is_some());
    let (mut sun, sc) = connect(&peer, &ca, SUN);
    let (mut moon, mc) = connect(&peer, &ca, MOON);
    let mut sw = shop_read(&mut sun, sc);
    let mut mw = shop_read(&mut moon, mc);
    for (c, context, wallet) in [(&mut sun, sc, &mut sw), (&mut moon, mc, &mut mw)] {
        assert_eq!(wallet.offers.len(), 9);
        for id in [
            40010, 40011, 40012, 40013, 40014, 40016, 40068, 40744, 40746,
        ] {
            if ammo::supported(id) {
                assert_eq!(wallet.offer(id).unwrap().support, shop::Support::Ammo);
            }
            let qty = if context.key == SUN {
                if id == 40744 {
                    2
                } else if id == 40746 {
                    8
                } else {
                    1
                }
            } else {
                1
            };
            let request = shop::Request::read(context, CHANNEL, None).mutation(
                wallet,
                shop::Action::Buy,
                id,
                qty,
            );
            let bought = c.shop(request).unwrap();
            assert_eq!(bought.status, shop::Status::Ok);
            *wallet = bought.snapshot.unwrap();
        }
        assert_eq!(wallet.weight.as_ref().unwrap().units.len(), 9);
    }
    for _ in 0..3 {
        assert_eq!(shop_read(&mut sun, sc), sw);
    }
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
                        name: "Natural regen TLS".into(),
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
        deadline.pause("bow qualified preparation");
        wf = sun
            .world(world_request(sc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    assert_eq!(wf.phase, world::Phase::Loading);
    moon.world(world_request(mc, &wf, world::Action::Read))
        .unwrap();
    let initial = Arc::new(read(&mut sun, sc, &wf));
    let sid = actor(&wf, SUN).entity_id;
    let mid = actor(&wf, MOON).entity_id;
    let early = sun
        .select_ammo(ammo::Request {
            action: ammo::Action::Select,
            sequence: 1,
            item_id: 40746,
            ..ammo::Request::read(sc, CHANNEL, &wf)
        })
        .unwrap();
    assert_eq!(early.reason, ammo::Reason::NotActive);
    assert_eq!(early.status, ammo::Status::Denied);
    assert_eq!(early.frame.as_ref().unwrap().ammo, initial.ammo);
    assert_eq!(
        early.frame.as_ref().unwrap().world.revision,
        initial.world.revision
    );
    sun.ping().unwrap();
    assert_eq!(initial.vital(sid).unwrap().weapon_type, 4);
    assert_eq!(initial.ammo(sid).unwrap().selected_item_id, 40744);
    assert_eq!(initial.ammo(sid).unwrap().stocks.len(), 2);
    assert_eq!(initial.item(sid).unwrap().stocks.len(), 7);
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
    wf = moon
        .world(world::Request {
            manifest_digest: wf.manifest.digest,
            ..world_request(mc, &wf, world::Action::Loaded)
        })
        .unwrap()
        .frame
        .unwrap();
    while wf.phase != world::Phase::Active {
        deadline.pause("bow countdown");
        wf = sun
            .world(world_request(sc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    let mut current = track(&initial, read(&mut sun, sc, &wf), sc);

    assert_eq!(current.regen.len(), 2);
    assert!(current
        .regen
        .iter()
        .all(|s| *s == super::super::regen_protocol::State::initial(s.entity_id)));
    assert_eq!(current.vital(sid).unwrap().max_mp, 6);
    assert_eq!(current.vital(mid).unwrap().max_mp, 6);
    let active_at = Instant::now();

    // The opposing owner's real qualified unarmed attack damages SUN.
    current = track(&current, read(&mut moon, mc, &current.world), mc);
    let hit = moon
        .combat(combat::Request {
            action: combat::Action::Attack,
            sequence: current.vital(mid).unwrap().attack_sequence + 1,
            target_entity_id: sid,
            ..combat::Request::read(mc, CHANNEL, &current.world)
        })
        .unwrap();
    assert_eq!(
        (hit.status, hit.reason),
        (combat::Status::Ok, combat::Reason::None)
    );
    current = track(&current, hit.frame.unwrap(), mc);
    // Retained unarmed damage replaces the earlier equipment subtotal: the
    // fixture armor+30 is not added. Mage level80 adds its later 80/20 class
    // bonus4 to max roll2; no AC reduction applies at the target's AC10.
    assert_eq!(current.regen(sid).unwrap().damage_loss, 6);
    pause(&deadline, Duration::from_millis(1050));
    current = cast(&mut moon, mc, &current, sid, spell::Reason::None);

    // Positive HP recovery is a real server event, never an invented USE/hurt.
    while current.regen(sid).unwrap().hp_gain == 0 {
        pause(&deadline, Duration::from_millis(200));
        current = track(&current, read(&mut sun, sc, &current.world), sc);
        moon.ping().unwrap();
    }
    assert!(active_at.elapsed() >= Duration::from_secs(2));
    assert_eq!(current.item(sid).unwrap().use_count, 0);
    let before_heal = current.clone();
    let healed = sun
        .use_item(item::Request {
            action: item::Action::Use,
            sequence: current.item(sid).unwrap().request_sequence + 1,
            item_id: 40010,
            ..item::Request::read(sc, CHANNEL, &current.world)
        })
        .unwrap();
    assert_eq!(
        (healed.status, healed.reason),
        (item::Status::Ok, item::Reason::None)
    );
    current = track(&current, healed.frame.unwrap(), sc);
    assert!(current.regen(sid).unwrap().item_gain > 0);
    assert_eq!(
        current.regen(sid).unwrap().damage_loss,
        before_heal.regen(sid).unwrap().damage_loss
    );
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);

    // Consume the entire six-MP pool, then require a typed, cost-free denial.
    current = cast(&mut sun, sc, &current, mid, spell::Reason::None);
    pause(&deadline, Duration::from_millis(750));
    current = cast(&mut sun, sc, &current, mid, spell::Reason::None);
    assert_eq!(current.vital(sid).unwrap().mp, 0);
    assert!(current.vital(mid).unwrap().hp > 0);
    pause(&deadline, Duration::from_millis(750));
    current = cast(&mut sun, sc, &current, mid, spell::Reason::NoMp);

    // No shortened fixture clock: Knight MPR is 64 counter units / idle4.
    // MOON's signed equipment MPR=-2 makes its ordinary base1 a loss of one.
    while current.regen(sid).unwrap().mp_gain < 6 || current.regen(mid).unwrap().mp_loss < 1 {
        pause(&deadline, Duration::from_millis(200));
        current = track(&current, read(&mut sun, sc, &current.world), sc);
        current = track(&current, read(&mut moon, mc, &current.world), mc);
        assert_eq!(current.world.phase, world::Phase::Active);
        assert_eq!(current.regen(mid).unwrap().hp_gain, 0);
        assert_eq!(current.regen(mid).unwrap().hp_loss, 0);
    }
    assert!(active_at.elapsed() >= Duration::from_secs(16));
    assert_eq!(current.vital(sid).unwrap().mp, 6);
    assert_eq!(
        current
            .damages
            .iter()
            .find(|s| s.entity_id == sid)
            .unwrap()
            .mp_spent,
        6
    );
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    current = cast(&mut sun, sc, &current, mid, spell::Reason::None);
    pause(&deadline, Duration::from_millis(750));
    current = cast(&mut sun, sc, &current, mid, spell::Reason::None);
    assert_eq!(current.world.phase, world::Phase::Finished);
    let spent = current
        .damages
        .iter()
        .find(|s| s.entity_id == sid)
        .unwrap()
        .mp_spent;
    assert_eq!(spent, 12);
    assert!(spent > current.vital(sid).unwrap().max_mp as u64);
    assert_eq!(current.vital(mid).unwrap().hp, 0);
    assert_eq!(current.regen(mid).unwrap().damage_loss, 40);
    let terminal = current.regen.clone();
    while current.world.completion.as_ref().unwrap().state != world::SettlementState::Saved {
        deadline.pause("natural regen durable result");
        current = track(&current, read(&mut sun, sc, &current.world), sc);
        assert_eq!(current.regen, terminal);
    }
    pause(&deadline, Duration::from_millis(1200));
    current = track(&current, read(&mut sun, sc, &current.world), sc);
    assert_eq!(
        current.regen, terminal,
        "no terminal recovery or corpse revival"
    );
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
    for (c, context) in [(&mut sun, sc), (&mut moon, mc)] {
        let reset = shop_read(c, context);
        assert_eq!((reset.round_id, reset.balance), (2, 20000));
        assert!(reset.stacks.is_empty());
    }
    sr.shutdown();
    mr.shutdown();
    while !sr.is_quiescent() || !mr.is_quiescent() {
        sr.poll();
        mr.poll();
        deadline.pause("natural regen resource retirement");
    }
    sun.close().unwrap();
    moon.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
