//! v13 Honor, two TLS owners, real shared532 resources, isolated SQL.
//! Compact camps/source stats/RNG are explicit fixtures, not recovered balance.
use super::super::{
    honor_protocol as honor, shop_protocol as shop, transport::ModernTlsConnection,
};
use super::*;
fn connect(
    peer: &JavaPeer,
    ca: &[u8],
    key: CharacterKey,
    life_actions: bool,
) -> (ModernTlsConnection, SelectionContext) {
    let mut hello = Hello::current(100);
    if life_actions {
        hello.max_version = 37;
        hello.supported_capabilities |= crate::modern_lito::protocol::CAP_BATTLE_DATA
            | crate::modern_lito::protocol::CAP_ACTION_LIFE;
    }
    let mut c = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(ca),
        hello,
        TransportLimits {
            connect_timeout: Duration::from_millis(500),
            total_timeout: Duration::from_secs(2),
            io_poll_interval: Duration::from_millis(25),
        },
        Cancellation::new(),
    )
    .unwrap();
    if life_actions {
        assert_eq!(c.ack().unwrap().version, 37);
        assert!(c.supports_battle_actions());
    } else {
        assert_eq!(
            (c.ack().unwrap().version, c.ack().unwrap().capabilities),
            (13, 32767)
        );
    }
    c.authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    assert_eq!(
        c.characters(key.source_id, 1).unwrap().characters[0].key,
        key
    );
    let context = SelectionContext { key, epoch: 1 };
    c.select_character(context).unwrap();
    assert_eq!(c.record().unwrap().totals.unwrap().revision, 0);
    c.skills().unwrap();
    c.enter_channel(CHANNEL).unwrap();
    (c, context)
}
fn read(c: &mut ModernTlsConnection, context: SelectionContext, w: &world::Frame) -> combat::Frame {
    c.combat(combat::Request::read(context, CHANNEL, w))
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
        "complete Honor frame proof at {} -> {}",
        old.world.revision,
        new.world.revision
    );
    new
}
fn shop_read(c: &mut ModernTlsConnection, context: SelectionContext) -> Arc<shop::Snapshot> {
    c.shop(shop::Request::read(context, CHANNEL, None))
        .unwrap()
        .snapshot
        .unwrap()
}
fn pause(d: &Deadline, millis: u64) {
    let end = Instant::now() + Duration::from_millis(millis);
    while Instant::now() < end {
        d.pause("Honor retained cadence");
    }
}
fn cast(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    old: &combat::Frame,
    skill: u32,
    target: u64,
) -> combat::Frame {
    let current = track(old, read(c, context, &old.world), context);
    let id = actor(&current.world, context.key).entity_id;
    super::ranged_interop_tests::arm_action(c, context, &current.world, Some(target));
    let r = c
        .cast(spell::CastRequest {
            context,
            channel_id: CHANNEL,
            match_id: current.world.match_id,
            generation: current.world.generation,
            sequence: current.world.cast(id).unwrap().request_sequence + 1,
            skill_id: skill,
            target_entity_id: target,
        })
        .unwrap();
    assert_eq!(
        (r.status, r.reason),
        (spell::Status::Ok, spell::Reason::None)
    );
    track(&current, r.frame.unwrap(), context)
}
fn attack(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    old: &combat::Frame,
    target: u64,
) -> combat::Frame {
    let current = track(old, read(c, context, &old.world), context);
    let id = actor(&current.world, context.key).entity_id;
    super::ranged_interop_tests::arm_action(c, context, &current.world, Some(target));
    let r = c
        .combat(combat::Request {
            action: combat::Action::Attack,
            sequence: current.vital(id).unwrap().attack_sequence + 1,
            target_entity_id: target,
            ..combat::Request::read(context, CHANNEL, &current.world)
        })
        .unwrap();
    assert_eq!(
        (r.status, r.reason),
        (combat::Status::Ok, combat::Reason::None)
    );
    track(&current, r.frame.unwrap(), context)
}
fn objective(f: &combat::Frame, id: u64) -> &honor::Objective {
    f.honor.as_ref().unwrap().objective(id).unwrap()
}
fn life(f: &combat::Frame, id: u64) -> &honor::Life {
    f.honor.as_ref().unwrap().life(id).unwrap()
}
fn healing(f: &combat::Frame, id: u64) -> &honor::HealCast {
    f.honor.as_ref().unwrap().heal(id).unwrap()
}
// Complete six-sector debug qualification measured past the older single-sector
// 12s stage. Only this Honor first-load test uses the existing allowed 30s gate;
// old peers, native attack/respawn cadence and all other tests stay unchanged.
fn first_images(
    sun: &mut WorldRuntime,
    moon: &mut WorldRuntime,
    frame: &Arc<combat::Frame>,
    deadline: &Deadline,
    mut progress: impl FnMut(),
) -> Arc<RenderedWorld> {
    sun.update_combat(
        Arc::new(frame.world.clone()),
        actor(&frame.world, SUN).entity_id,
        Some(frame.clone()),
    )
    .unwrap();
    moon.update_combat(
        Arc::new(frame.world.clone()),
        actor(&frame.world, MOON).entity_id,
        Some(frame.clone()),
    )
    .unwrap();
    let started = Instant::now();
    loop {
        assert!(
            started.elapsed() < Duration::from_secs(28),
            "Honor first render 28s stage: sun={:?};moon={:?}",
            sun.poll(),
            moon.poll()
        );
        deadline.check("Honor complete shared first frame");
        progress();
        if let (Some(s), Some(m)) = (
            rendered(sun, &frame.world, SUN),
            rendered(moon, &frame.world, MOON),
        ) {
            assert_eq!(s.objectives.len(), 2);
            assert_eq!(m.objectives.len(), 2);
            eprintln!(
                "Honor complete dual first render: {:.3}s",
                started.elapsed().as_secs_f64()
            );
            return s;
        }
        deadline.pause("Honor actual dual first render");
    }
}
fn inspect_image(image: &RenderedWorld, label: &str) {
    assert_eq!(image.actors.len(), 2);
    assert_eq!(image.objectives.len(), 2);
    assert_eq!(
        image
            .objectives
            .iter()
            .map(|p| p.entity_id)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([3, 4])
    );
    assert!(image.objectives.iter().all(|p| p.width > 0 && p.height > 0));
    assert_eq!(
        image.combat.as_ref().unwrap().world.revision,
        image.revision
    );
    if std::env::var("LITO_WORLD_MAP_WRITE_REFERENCE").as_deref() == Ok("1") {
        let directory =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/modern-lito-render-tests");
        std::fs::create_dir_all(&directory).unwrap();
        let output = directory.join(format!("shared-honor-{label}-diagnostic.png"));
        let mut rgba = image.bgra.to_vec();
        for p in rgba.chunks_exact_mut(4) {
            p.swap(0, 2);
        }
        let mut encoder = png::Encoder::new(
            std::fs::File::create(&output).unwrap(),
            image.width,
            image.height,
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&rgba)
            .unwrap();
        eprintln!("Honor protocol-render diagnostic {}", output.display());
    }
}
#[test]
#[ignore = "v13 Java Honor peer + isolated MySQL + true shared532/SPR; no GUI"]
fn java_honor_tower_healing_five_second_respawn_destroy_and_saved_sql() {
    run_honor(false);
}
#[test]
#[ignore = "v37 real Honor target respawn rejection, same-sequence recovery, shared532 and isolated SQL; no HWND"]
fn java_v37_honor_delayed_target_respawn_rejection_then_fresh_attack_heal_and_saved_sql() {
    run_honor(true);
}
fn run_honor(life_actions: bool) {
    let deadline = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernHonorTestPeer",
        if life_actions { "life-honor" } else { "honor" },
    );
    assert!(peer.process_job.is_some());
    let (mut sun, sc) = connect(&peer, &ca, SUN, life_actions);
    let (mut moon, mc) = connect(&peer, &ca, MOON, life_actions);
    for (c, context) in [(&mut sun, sc), (&mut moon, mc)] {
        let mut wallet = shop_read(c, context);
        assert_eq!(wallet.offers.len(), 9);
        for id in [
            40010, 40011, 40012, 40013, 40014, 40016, 40068, 40744, 40746,
        ] {
            let qty = if context.key == SUN && id == 40744 {
                80
            } else {
                1
            };
            let r = c
                .shop(shop::Request::read(context, CHANNEL, None).mutation(
                    &wallet,
                    shop::Action::Buy,
                    id,
                    qty,
                ))
                .unwrap();
            assert_eq!(r.status, shop::Status::Ok);
            wallet = r.snapshot.unwrap();
        }
        assert_eq!(shop_read(c, context), wallet);
    }
    let room = sun
        .rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(sc, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 5,
                        attributes: 16,
                        map_id: 532,
                        name: "Shared532 Honor integration".into(),
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
                action: rooms::Action::Join,
                room_id,
                expected_revision: summary.revision,
                ..rooms::Common::read(mc, CHANNEL)
            },
            String::new(),
        )
        .unwrap(),
    )
    .unwrap();
    let r = sun
        .rooms(rooms::Request::read(sc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    sun.rooms(room_command(sc, &r, rooms::Action::Ready, 1))
        .unwrap();
    let r = moon
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    let ready = moon
        .rooms(room_command(mc, &r, rooms::Action::Ready, 1))
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
        deadline.pause("Honor source preparation");
        wf = sun
            .world(world_request(sc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    assert_eq!(wf.phase, world::Phase::Loading);
    assert_eq!(wf.manifest.map_id, 532);
    assert_eq!(wf.manifest.sectors.len(), 6);
    assert_eq!(wf.actors.len(), 2);
    assert_eq!(actor(&wf, SUN).position, WorldCell { x: 32792, y: 32843 });
    assert_eq!(actor(&wf, MOON).position, WorldCell { x: 32798, y: 32843 });
    moon.world(world_request(mc, &wf, world::Action::Read))
        .unwrap();
    let initial = Arc::new(read(&mut sun, sc, &wf));
    let sid = actor(&wf, SUN).entity_id;
    let mid = actor(&wf, MOON).entity_id;
    assert_eq!((sid, mid), (1, 2));
    assert_eq!(
        initial
            .honor
            .as_ref()
            .unwrap()
            .objectives
            .iter()
            .map(|o| o.entity_id)
            .collect::<Vec<_>>(),
        [3, 4]
    );
    assert_eq!(initial.vitals.len(), 2);
    assert!(initial.vital(3).is_none());
    assert_eq!(objective(&initial, 3).hp, 3000);
    assert_eq!(initial.ammo(sid).unwrap().selected_item_id, 40744);
    let mut sr = WorldRuntime::new(shared.clone()).unwrap();
    let mut mr = WorldRuntime::new(shared).unwrap();
    let mut keepalive = Instant::now();
    let image = first_images(&mut sr, &mut mr, &initial, &deadline, || {
        if keepalive.elapsed() >= Duration::from_secs(1) {
            sun.ping().unwrap();
            moon.ping().unwrap();
            keepalive = Instant::now();
        }
    });
    assert!(image.ready_to_ack);
    assert_eq!(image.navigation_map().unwrap().map_id(), 532);
    inspect_image(&image, "loading");
    // Renderer-specific objective placements/hits are also asserted below once
    // the world has accepted these two owners' actual first-frame acknowledgments.
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
        deadline.pause("Honor three-second countdown");
        wf = sun
            .world(world_request(sc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    let mut current = track(&initial, read(&mut sun, sc, &wf), sc);
    // Same-team full-HP heal is a successful interaction, not a forged HP gain.
    current = track(&current, read(&mut moon, mc, &current.world), mc);
    let full = current.clone();
    current = cast(&mut moon, mc, &current, honor::BASIC_HEAL, 4);
    assert_eq!(healing(&current, mid).count, 1);
    assert_eq!(healing(&current, mid).restored, 0);
    assert_eq!(objective(&current, 4).hp, 3000);
    assert!(objective(&current, 4).state_revision > objective(&full, 4).state_revision);
    assert_eq!(objective(&current, 4).hurt_revision, 0);
    assert_eq!(objective(&current, 4).heal_gain, 0);
    pause(&deadline, 750);
    current = cast(&mut moon, mc, &current, spell::LIGHT_ARROW, 3);
    assert!(objective(&current, 3).hp < 3000);
    let damage = objective(&current, 3).damage_loss;
    assert!(damage > 0);
    for _ in 0..20 {
        if objective(&current, 3).hp == 3000 {
            break;
        }
        current = cast(&mut sun, sc, &current, honor::BASIC_HEAL, 3);
        pause(&deadline, 750);
    }
    assert_eq!(objective(&current, 3).hp, 3000);
    assert_eq!(objective(&current, 3).heal_gain, damage);
    assert!(healing(&current, sid).mp_spent > 0);
    inspect_image(
        &combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline),
        "healed",
    );
    // An ordinary ranged PC kill must not finish an objective-based match.
    let living = current.clone();
    let deferred = life_actions.then(|| {
        (
            super::ranged_interop_tests::action_guard(&mut sun, sc, &current.world, Some(mid)),
            combat::Request {
                action: combat::Action::Attack,
                sequence: current.vital(sid).unwrap().attack_sequence + 2,
                target_entity_id: mid,
                ..combat::Request::read(sc, CHANNEL, &current.world)
            },
        )
    });
    current = attack(&mut sun, sc, &current, mid);
    assert_eq!(current.vital(mid).unwrap().hp, 0);
    assert_eq!(current.world.phase, world::Phase::Active);
    assert_eq!(life(&current, mid).death_count, 1);
    assert_eq!(life(&current, mid).respawn_count, 0);
    assert!(life(&current, mid).pending_due_millis >= 5000);
    let corpse = current.clone();
    let died_at = Instant::now();
    assert!(current.ammo(sid).unwrap().stocks != living.ammo(sid).unwrap().stocks);
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    while life(&current, mid).respawn_count == 0 {
        pause(&deadline, 100);
        current = track(&current, read(&mut sun, sc, &current.world), sc);
        moon.ping().unwrap();
        assert_eq!(current.world.phase, world::Phase::Active);
    }
    // Polling/render latency can precede died_at; require the server's exact due
    // contract and a conservative real elapsed lower bound, not an accelerated clock.
    assert!(died_at.elapsed() >= Duration::from_millis(4500));
    assert_eq!(life(&current, mid).respawn_count, 1);
    assert_eq!(life(&current, mid).respawn_hp_gain, 40);
    assert_eq!(life(&current, mid).pending_due_millis, 0);
    assert_eq!(current.vital(mid).unwrap().hp, 40);
    assert_eq!(
        current.vital(mid).unwrap().death_revision,
        corpse.vital(mid).unwrap().death_revision
    );
    assert_eq!(
        actor(&current.world, MOON).position,
        actor(&corpse.world, MOON).position
    );
    assert_eq!(
        current.item(mid).unwrap().stocks,
        corpse.item(mid).unwrap().stocks
    );
    assert_eq!(
        current.ammo(mid).unwrap().stocks,
        corpse.ammo(mid).unwrap().stocks
    );
    assert_eq!(
        current.vital(mid).unwrap().mp,
        corpse.vital(mid).unwrap().mp
            + (current.regen(mid).unwrap().mp_gain - corpse.regen(mid).unwrap().mp_gain) as i64
            - (current.regen(mid).unwrap().mp_loss - corpse.regen(mid).unwrap().mp_loss) as i64
    );
    if let Some((guard, request)) = deferred {
        use crate::modern_lito::{
            application_protocol::Response, battle_action::Rejection, transport::TransportError,
        };
        let before = current.clone();
        assert_eq!(
            guard.target,
            Some(crate::modern_lito::battle_window::Target::Player {
                entity: mid as i64,
                life: 1
            })
        );
        sun.arm_battle_action(guard).unwrap();
        assert!(matches!(
            sun.combat(request),
            Err(TransportError::GuardRejected(Rejection::StaleTarget))
        ));
        let Response::Combat(recovery) = sun.take_action_recovery().unwrap() else {
            panic!("Honor rejection must be READ")
        };
        assert_eq!(recovery.echo.action, combat::Action::Read);
        current = track(&current, recovery.frame.unwrap(), sc);
        assert_eq!(
            current.vital(sid).unwrap().attack_sequence,
            before.vital(sid).unwrap().attack_sequence
        );
        assert_eq!(
            current.ammo, before.ammo,
            "stale target cannot consume arrows"
        );
        assert_eq!(
            current.vital(mid).unwrap().hp,
            before.vital(mid).unwrap().hp
        );
        assert_eq!(life(&current, mid).respawn_count, 1);
        current = attack(&mut sun, sc, &current, mid);
        assert_eq!(
            current.vital(sid).unwrap().attack_sequence,
            request.sequence,
            "fresh target reuses unconsumed sequence"
        );
        assert_eq!(current.vital(mid).unwrap().hp, 0);
        assert_eq!(life(&current, mid).death_count, 2);
        assert!(life(&current, mid).pending_due_millis >= 5000);
        while life(&current, mid).respawn_count < 2 {
            pause(&deadline, 100);
            current = track(&current, read(&mut sun, sc, &current.world), sc);
            moon.ping().unwrap();
        }
        assert_eq!(current.vital(mid).unwrap().hp, 40);
        assert_eq!(life(&current, mid).respawn_count, 2);
    }
    current = track(&current, read(&mut moon, mc, &current.world), mc);
    current = cast(&mut moon, mc, &current, honor::BASIC_HEAL, 4);
    assert_eq!(healing(&current, mid).count, 2);
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    current = cast(&mut sun, sc, &current, spell::LIGHT_ARROW, 4);
    assert_eq!(
        objective(&current, 4).last_kind,
        honor::Interaction::LightArrow
    );
    let mut states = BTreeSet::from([0]);
    let mut hits = 0;
    while current.world.phase == world::Phase::Active {
        pause(&deadline, 1050);
        current = attack(&mut sun, sc, &current, 4);
        hits += 1;
        assert!(hits <= 60, "bounded qualified +100 ranged fixture");
        assert_eq!(
            objective(&current, 4).last_kind,
            honor::Interaction::Physical
        );
        states.insert(objective(&current, 4).action);
        moon.ping().unwrap();
        if hits % 5 == 0 || current.world.phase == world::Phase::Finished {
            combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
        }
    }
    assert_eq!(states, BTreeSet::from([0, 32, 33, 34, 35]));
    assert_eq!(current.world.phase, world::Phase::Finished);
    assert_eq!(objective(&current, 4).hp, 0);
    assert!(objective(&current, 3).hp > 0);
    assert!(
        current.vital(mid).unwrap().hp > 0,
        "losing PC need not die for objective victory"
    );
    let terminal_honor = current.honor.clone();
    let terminal_items = current.items.clone();
    let terminal_ammo = current.ammo.clone();
    while current.world.completion.as_ref().unwrap().state != world::SettlementState::Saved {
        deadline.pause("Honor durable SQL");
        current = track(&current, read(&mut sun, sc, &current.world), sc);
    }
    assert_eq!(current.honor, terminal_honor);
    assert_eq!(current.items, terminal_items);
    assert_eq!(current.ammo, terminal_ammo);
    assert_eq!(current.world.completion.as_ref().unwrap().event.mode, 5);
    assert_eq!(current.world.completion.as_ref().unwrap().event.map_id, 532);
    // Keep the authoritative terminal frame while the real shared 68-tick
    // collapse finishes. SAVED publication must not restart the frozen death.
    pause(&deadline, 3100);
    sun.ping().unwrap();
    moon.ping().unwrap();
    let final_image = combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    inspect_image(&final_image, "saved");
    assert_eq!(
        final_image
            .objectives
            .iter()
            .find(|o| o.entity_id == 4)
            .unwrap()
            .subframe,
        20
    );
    let won = sun.record().unwrap().totals.unwrap();
    let lost = moon.record().unwrap().totals.unwrap();
    assert_eq!((won.wins, won.losses, won.revision), (1, 0, 1));
    assert_eq!((lost.wins, lost.losses, lost.revision), (0, 1, 1));
    assert_eq!(sun.record().unwrap().totals, Some(won));
    sun.world(world_request(sc, &current.world, world::Action::Leave))
        .unwrap();
    moon.world(world_request(mc, &current.world, world::Action::Leave))
        .unwrap();
    assert_eq!(
        sun.rooms(rooms::Request::read(sc, CHANNEL).unwrap())
            .unwrap()
            .joined
            .unwrap()
            .summary
            .ready,
        0
    );
    sr.shutdown();
    mr.shutdown();
    while !sr.is_quiescent() || !mr.is_quiescent() {
        sr.poll();
        mr.poll();
        deadline.pause("Honor render workers retirement");
    }
    sun.close().unwrap();
    moon.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
