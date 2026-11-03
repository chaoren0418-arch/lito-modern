//! v14 Fire Storm through two real TLS owners, the shared map/SPR renderer and
//! owned isolated SQL. Fixture camps/stats are synthetic, not recovered balance.
//! This is not a native-window, device-audio or production-database test.
use super::super::{
    fire_storm_protocol as fire, gm_protocol as gm, honor_protocol as honor, shop_protocol as shop,
    transport::ModernTlsConnection,
};
use super::*;

fn connect(
    peer: &JavaPeer,
    ca: &[u8],
    key: CharacterKey,
    honor_mode: bool,
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
            (14, if honor_mode { 65535 } else { 49151 })
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
fn buy_match_stocks(c: &mut ModernTlsConnection, context: SelectionContext) {
    let mut wallet = c
        .shop(shop::Request::read(context, CHANNEL, None))
        .unwrap()
        .snapshot
        .unwrap();
    assert_eq!(wallet.offers.len(), 9);
    for id in [
        40010, 40011, 40012, 40013, 40014, 40016, 40068, 40744, 40746,
    ] {
        let reply = c
            .shop(shop::Request::read(context, CHANNEL, None).mutation(
                &wallet,
                shop::Action::Buy,
                id,
                1,
            ))
            .unwrap();
        assert_eq!(reply.status, shop::Status::Ok);
        wallet = reply.snapshot.unwrap();
    }
    assert_eq!(
        c.shop(shop::Request::read(context, CHANNEL, None))
            .unwrap()
            .snapshot
            .unwrap(),
        wallet
    );
}
fn track(old: &combat::Frame, next: combat::Frame, context: SelectionContext) -> combat::Frame {
    let reply = |f: &combat::Frame| combat::Reply {
        echo: combat::Request::read(context, CHANNEL, &f.world),
        status: combat::Status::Ok,
        reason: combat::Reason::None,
        frame: Some(f.clone()),
    };
    assert!(
        combat::follows(Some(&reply(old)), &reply(&next)),
        "complete Fire Storm proof {} -> {}",
        old.world.revision,
        next.world.revision
    );
    next
}
fn pause(d: &Deadline, duration: Duration) {
    let end = Instant::now() + duration;
    while Instant::now() < end {
        d.pause("Fire Storm retained cadence");
    }
}
fn request(f: &combat::Frame, context: SelectionContext) -> spell::CastRequest {
    let id = actor(&f.world, context.key).entity_id;
    spell::CastRequest {
        context,
        channel_id: CHANNEL,
        match_id: f.world.match_id,
        generation: f.world.generation,
        sequence: f.world.cast(id).unwrap().request_sequence + 1,
        skill_id: fire::SKILL_ID,
        target_entity_id: 0,
    }
}
fn cast(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    old: &combat::Frame,
) -> combat::Frame {
    let current = track(old, read(c, context, &old.world), context);
    super::ranged_interop_tests::arm_action(c, context, &current.world, None);
    let r = c.cast(request(&current, context)).unwrap();
    assert_eq!(
        (r.status, r.reason),
        (spell::Status::Ok, spell::Reason::None)
    );
    track(&current, r.frame.unwrap(), context)
}
fn first_images(
    sr: &mut WorldRuntime,
    mr: &mut WorldRuntime,
    f: &Arc<combat::Frame>,
    d: &Deadline,
    sun: &mut ModernTlsConnection,
    moon: &mut ModernTlsConnection,
) -> Arc<RenderedWorld> {
    sr.update_combat(
        Arc::new(f.world.clone()),
        actor(&f.world, SUN).entity_id,
        Some(f.clone()),
    )
    .unwrap();
    mr.update_combat(
        Arc::new(f.world.clone()),
        actor(&f.world, MOON).entity_id,
        Some(f.clone()),
    )
    .unwrap();
    let started = Instant::now();
    let mut ping = Instant::now();
    loop {
        d.check("Fire Storm full shared first frame");
        assert!(
            started.elapsed() < Duration::from_secs(28),
            "Fire Storm shared first render: sun={:?};moon={:?}",
            sr.poll(),
            mr.poll()
        );
        if ping.elapsed() >= Duration::from_secs(1) {
            sun.ping().unwrap();
            moon.ping().unwrap();
            ping = Instant::now();
        }
        if let (Some(s), Some(_)) = (rendered(sr, &f.world, SUN), rendered(mr, &f.world, MOON)) {
            eprintln!(
                "Fire Storm complete dual shared first render {:.3}s",
                started.elapsed().as_secs_f64()
            );
            return s;
        }
        d.pause("Fire Storm dual renderer loading");
    }
}
fn inspect_image(image: &RenderedWorld, label: &str) {
    assert_eq!(
        image.combat.as_ref().unwrap().world.revision,
        image.revision
    );
    assert!(!image.combat.as_ref().unwrap().fires.is_empty());
    if std::env::var("LITO_WORLD_MAP_WRITE_REFERENCE").as_deref() == Ok("1") {
        let directory =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/modern-lito-render-tests");
        std::fs::create_dir_all(&directory).unwrap();
        let output = directory.join(format!("shared-fire-storm-{label}-diagnostic.png"));
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
        eprintln!("Fire Storm protocol-render diagnostic {}", output.display());
    }
}

/// Candidate selection only, using the server's bounded modern grid-ray policy.
/// This is not an independent LOS oracle: the real CAST reply must still prove
/// that every intended target was included in one atomic batch.
fn fixture_fire_ray(map: &WorldMap, from: WorldCell, target: WorldCell) -> bool {
    let dx = (i64::from(target.x) - i64::from(from.x)).abs();
    let dy = (i64::from(target.y) - i64::from(from.y)).abs();
    if dx.max(dy) == 0 || dx.max(dy) > 3 {
        return false;
    }
    let sx = (target.x - from.x).signum();
    let sy = (target.y - from.y).signum();
    let mut error = dx - dy;
    let mut previous = from;
    for _ in 0..3 {
        let mut next = previous;
        let twice = error * 2;
        if twice > -dy {
            error -= dy;
            next.x += sx;
        }
        if twice < dx {
            error += dx;
            next.y += sy;
        }
        if !map.can_step(previous, next) {
            return false;
        }
        if next == target {
            return true;
        }
        previous = next;
    }
    false
}

/// Find a short legal path over the ACTUAL rendered geometry to a cell with
/// radius and static LOS to every enemy, or outside all their radii. The server
/// still decides movement occupancy and the actual damage target set.
fn route_radius(
    map: &WorldMap,
    f: &combat::Frame,
    targets: &[WorldCell],
    inside: bool,
    key: CharacterKey,
) -> world_path::PlannedPath {
    let selected = actor(&f.world, key);
    let occupied: BTreeSet<_> = f
        .world
        .actors
        .iter()
        .filter(|a| a.entity_id != selected.entity_id)
        .map(|a| a.position)
        .chain(
            f.honor
                .iter()
                .flat_map(|h| h.objectives.iter().map(|o| o.position)),
        )
        .collect();
    let mut visited = BTreeSet::from([selected.position]);
    let mut queue = VecDeque::from([(selected.position, 0usize)]);
    while let Some((from, depth)) = queue.pop_front() {
        if depth > 0
            && targets.iter().all(|t| {
                if inside {
                    fire::in_radius(from, *t) && fixture_fire_ray(map, from, *t)
                } else {
                    !fire::in_radius(from, *t)
                }
            })
        {
            return world_path::plan(
                map,
                selected.position,
                from,
                &occupied,
                world_path::PathLimits {
                    max_expanded: 4096,
                    max_steps: 12,
                },
                &AtomicBool::new(false),
            )
            .unwrap();
        }
        if depth == 12 {
            continue;
        }
        for (h, (dx, dy)) in DIRECTIONS.iter().copied().enumerate() {
            if !world_path::can_step(map, from, h as u8, &occupied) {
                continue;
            }
            let to = WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            };
            if visited.insert(to) {
                queue.push_back((to, depth + 1));
            }
        }
        assert!(
            visited.len() <= 625,
            "bounded actual spawn-neighborhood search"
        );
    }
    panic!("qualified fixture must have reachable Fire Storm position: inside={inside}, selected={:?}, targets={targets:?}, occupied={occupied:?}, visited={visited:?}",selected.position);
}
fn move_path(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    old: &combat::Frame,
    path: &world_path::PlannedPath,
    d: &Deadline,
) -> combat::Frame {
    let mut current = old.clone();
    for (&heading, &expected) in path.headings().iter().zip(path.cells().iter().skip(1)) {
        let a = actor(&current.world, context.key);
        pause(
            d,
            Duration::from_nanos(a.step_interval_nanos).max(Duration::from_millis(10)),
        );
        super::ranged_interop_tests::arm_action(c, context, &current.world, None);
        let r = c
            .world(world::Request {
                heading,
                sequence: a.sequence + 1,
                from_x: a.position.x,
                from_y: a.position.y,
                ..world_request(context, &current.world, world::Action::Move)
            })
            .unwrap();
        assert_eq!(r.status, world::Status::Ok);
        assert_eq!(
            actor(r.frame.as_ref().unwrap(), context.key).position,
            expected
        );
        current = track(
            &current,
            read(c, context, r.frame.as_ref().unwrap()),
            context,
        );
    }
    current
}
fn assert_batch(old: &combat::Frame, next: &combat::Frame, caster: u64, expected: &BTreeSet<u64>) {
    let a = old.fire(caster).unwrap();
    let b = next.fire(caster).unwrap();
    assert_eq!(b.count, a.count + 1);
    assert_eq!(b.mp_spent - a.mp_spent, i64::from(b.mp_cost));
    assert_eq!(b.reuse_millis, 2000);
    assert_eq!(
        b.last_cast_sequence,
        next.world.cast(caster).unwrap().request_sequence
    );
    assert_eq!(
        b.from,
        next.world
            .actors
            .iter()
            .find(|p| p.entity_id == caster)
            .unwrap()
            .position
    );
    assert_eq!(
        b.targets
            .iter()
            .map(|t| t.target_entity_id)
            .collect::<BTreeSet<_>>(),
        *expected
    );
    for t in &b.targets {
        assert!(t.applied_damage > 0);
        assert_eq!(t.hp_before - t.hp_after, t.applied_damage);
        assert_eq!(honor::target_hp(next, t.target_entity_id), Some(t.hp_after));
        assert_eq!(
            honor::target_hurt_revision(next, t.target_entity_id),
            Some(b.cast_revision)
        );
        assert_eq!(
            fire::damage_loss(next, t.target_entity_id),
            Some(t.damage_loss_after)
        );
        assert_ne!(
            honor::target_team(next, caster),
            honor::target_team(next, t.target_entity_id)
        );
    }
    let regen_a = old.regen(caster).unwrap();
    let regen_b = next.regen(caster).unwrap();
    assert_eq!(
        next.vital(caster).unwrap().mp,
        old.vital(caster).unwrap().mp - i64::from(b.mp_cost)
            + (regen_b.mp_gain - regen_a.mp_gain) as i64
            - (regen_b.mp_loss - regen_a.mp_loss) as i64
    );
    for v in &old.vitals {
        if honor::target_team(old, v.entity_id) == honor::target_team(old, caster) {
            assert_eq!(
                next.regen(v.entity_id).unwrap().damage_loss,
                old.regen(v.entity_id).unwrap().damage_loss,
                "friendly fire must be absent"
            );
        }
    }
}

fn run(mode: &'static str, honor_mode: bool) {
    let life_actions = mode == "life-fire";
    let deadline = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernFireStormTestPeer",
        mode,
    );
    assert!(peer.process_job.is_some());
    let (mut sun, sc) = connect(&peer, &ca, SUN, honor_mode, life_actions);
    let (mut moon, mc) = connect(&peer, &ca, MOON, honor_mode, life_actions);
    buy_match_stocks(&mut sun, sc);
    buy_match_stocks(&mut moon, mc);
    let created = sun
        .rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(sc, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: if honor_mode { 2 } else { 4 },
                        mode: if honor_mode { 5 } else { 1 },
                        attributes: 16,
                        map_id: if honor_mode { 532 } else { 514 },
                        name: format!("Shared Fire Storm {mode}"),
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
    let room_id = created.summary.room_id;
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
    if !honor_mode {
        let r = sun
            .rooms(rooms::Request::read(sc, CHANNEL).unwrap())
            .unwrap()
            .joined
            .unwrap();
        let bots = sun
            .gm_bots(gm::Request {
                context: sc,
                channel_id: CHANNEL,
                room_id,
                expected_revision: r.summary.revision,
                count: 2,
            })
            .unwrap();
        assert_eq!((bots.added, bots.removed), (2, 0));
        let members = &bots.room.as_ref().unwrap().joined.as_ref().unwrap().members;
        assert_eq!(
            members
                .iter()
                .filter(|m| m.kind == rooms::MemberKind::ServerBot)
                .count(),
            2
        );
        assert_eq!(members.iter().filter(|m| m.team == 0).count(), 2);
        assert_eq!(members.iter().filter(|m| m.team == 1).count(), 2);
    }
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
        deadline.pause("Fire Storm source preparation");
        wf = sun
            .world(world_request(sc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    assert_eq!(wf.phase, world::Phase::Loading);
    assert_eq!(wf.manifest.map_id, if honor_mode { 532 } else { 514 });
    assert_eq!(wf.manifest.sectors.len(), if honor_mode { 6 } else { 1 });
    assert_eq!(wf.actors.len(), if honor_mode { 2 } else { 4 });
    moon.world(world_request(mc, &wf, world::Action::Read))
        .unwrap();
    let initial = Arc::new(read(&mut sun, sc, &wf));
    let sid = actor(&wf, SUN).entity_id;
    let mid = actor(&wf, MOON).entity_id;
    assert_eq!(initial.fires.len(), wf.actors.len());
    for a in &wf.actors {
        let f = initial.fire(a.entity_id).unwrap();
        assert_eq!(f.count, 0);
        assert_eq!(f.mp_cost == 0, a.identity().character().is_none());
    }
    assert_eq!(initial.honor.is_some(), honor_mode);
    let (mut sr, mut mr) = (
        WorldRuntime::new(shared.clone()).unwrap(),
        WorldRuntime::new(shared).unwrap(),
    );
    let image = first_images(&mut sr, &mut mr, &initial, &deadline, &mut sun, &mut moon);
    inspect_image(&image, &format!("{mode}-loading"));
    let map = image.navigation_map().unwrap();
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
        deadline.pause("Fire Storm native countdown");
        wf = sun
            .world(world_request(sc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    let mut current = track(&initial, read(&mut sun, sc, &wf), sc);
    if !honor_mode {
        // Move Moon outside every enemy radius to establish the zero-target
        // Fire Storm case, using actual allocated cells and real MOVE/ACK.
        // Independent lone-human tests prove egress without another actor moving.
        let enemy_positions: Vec<_> = current
            .world
            .actors
            .iter()
            .filter(|a| {
                honor::target_team(&current, a.entity_id) != honor::target_team(&current, mid)
            })
            .map(|a| a.position)
            .collect();
        let away = route_radius(&map, &current, &enemy_positions, false, MOON);
        current = move_path(&mut moon, mc, &current, &away, &deadline);
        assert!(enemy_positions
            .iter()
            .all(|p| !fire::in_radius(actor(&current.world, MOON).position, *p)));
        pause(
            &deadline,
            Duration::from_nanos(actor(&current.world, MOON).step_interval_nanos)
                + Duration::from_millis(10),
        );
    }
    let zero_id = if honor_mode { sid } else { mid };
    let zero_context = if honor_mode { sc } else { mc };
    let zero_connection = if honor_mode { &mut sun } else { &mut moon };
    let before = current.clone();
    current = cast(zero_connection, zero_context, &current);
    assert_batch(&before, &current, zero_id, &BTreeSet::new());
    assert!(
        current.fire(zero_id).unwrap().targets.is_empty(),
        "empty target set is a successful paid cast"
    );
    let accepted = current.clone();
    super::ranged_interop_tests::arm_action(zero_connection, zero_context, &current.world, None);
    let denied = zero_connection
        .cast(request(&current, zero_context))
        .unwrap();
    assert_eq!(
        (denied.status, denied.reason),
        (spell::Status::Denied, spell::Reason::TooFast)
    );
    current = track(&current, denied.frame.unwrap(), zero_context);
    assert_eq!(current.fires, accepted.fires);
    assert_eq!(
        current.regen(zero_id).unwrap().damage_loss,
        accepted.regen(zero_id).unwrap().damage_loss
    );
    assert_eq!(
        current.vital(zero_id).unwrap().mp,
        accepted.vital(zero_id).unwrap().mp
            + (current.regen(zero_id).unwrap().mp_gain - accepted.regen(zero_id).unwrap().mp_gain)
                as i64
            - (current.regen(zero_id).unwrap().mp_loss - accepted.regen(zero_id).unwrap().mp_loss)
                as i64,
        "cooldown denial must not spend another Fire Storm cost"
    );
    // Keep both actual renderers ticking through the qualified caster handoff,
    // including a zero-target effect. No audio device is opened by this test.
    let first = combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    let visual_end = Instant::now() + Duration::from_secs(4);
    // The device consumer drops cues older than 500ms. An event present in a
    // slow render's mailbox is not sufficient evidence of playable audio.
    let fresh710 = |image: &RenderedWorld| {
        image.audio.as_ref().is_some_and(|a| {
            a.events.iter().any(|e| {
                e.group.candidates().contains(&710)
                    && Instant::now().saturating_duration_since(e.created)
                        <= Duration::from_millis(500)
            })
        })
    };
    let mut sound710 = fresh710(&first);
    let mut image710 = sound710;
    let mut ping = Instant::now();
    if sound710 {
        inspect_image(&first, &format!("{mode}-empty-effect"));
    }
    while Instant::now() < visual_end {
        if let Some(i) = rendered(&mut sr, &current.world, SUN) {
            if fresh710(&i) {
                sound710 = true;
                if !image710 {
                    inspect_image(&i, &format!("{mode}-empty-effect"));
                    image710 = true;
                }
            }
        }
        mr.poll();
        if ping.elapsed() >= Duration::from_secs(1) {
            sun.ping().unwrap();
            moon.ping().unwrap();
            ping = Instant::now();
        }
        pause(&deadline, Duration::from_millis(20));
    }
    assert!(
        sound710,
        "shared1819 must emit qualified sound710 for an empty-target cast"
    );
    current = track(&current, read(&mut sun, sc, &current.world), sc);
    assert_eq!(current.fire(zero_id), accepted.fire(zero_id));
    let enemies: BTreeSet<_> = current
        .world
        .actors
        .iter()
        .filter(|a| honor::target_team(&current, a.entity_id) != honor::target_team(&current, sid))
        .map(|a| a.entity_id)
        .chain(current.honor.iter().flat_map(|h| {
            h.objectives
                .iter()
                .filter(|o| Some(o.team) != honor::target_team(&current, sid))
                .map(|o| o.entity_id)
        }))
        .collect();
    assert_eq!(enemies.len(), 2);
    let positions: Vec<_> = enemies
        .iter()
        .map(|id| honor::target_position(&current, *id).unwrap())
        .collect();
    let path = route_radius(&map, &current, &positions, true, SUN);
    current = move_path(&mut sun, sc, &current, &path, &deadline);
    pause(&deadline, Duration::from_millis(2100));
    let before = current.clone();
    current = cast(&mut sun, sc, &current);
    let died_at = Instant::now();
    assert_batch(&before, &current, sid, &enemies);
    assert_eq!(current.vital(mid).unwrap().hp, 0);
    let corpse = current.clone();
    combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    if honor_mode {
        let tid = *enemies
            .iter()
            .find(|id| current.vital(**id).is_none())
            .unwrap();
        assert_eq!(current.world.phase, world::Phase::Active);
        assert!(honor::target_hp(&current, tid).unwrap() > 0);
        assert_eq!(
            current
                .honor
                .as_ref()
                .unwrap()
                .life(mid)
                .unwrap()
                .death_count,
            1
        );
        assert!(
            current
                .honor
                .as_ref()
                .unwrap()
                .life(mid)
                .unwrap()
                .pending_due_millis
                >= 5000
        );
        while fire::life_count(&current, mid) == 0 {
            pause(&deadline, Duration::from_millis(100));
            current = track(&current, read(&mut sun, sc, &current.world), sc);
            moon.ping().unwrap();
        }
        assert!(died_at.elapsed() >= Duration::from_millis(4500));
        assert_eq!(fire::life_count(&current, mid), 1);
        assert!(current.vital(mid).unwrap().hp > 0);
        assert_eq!(current.fires, corpse.fires);
        assert_eq!(current.items, corpse.items);
        assert_eq!(current.ammo, corpse.ammo);
        assert_eq!(
            current.vital(mid).unwrap().mp,
            corpse.vital(mid).unwrap().mp
                + (current.regen(mid).unwrap().mp_gain - corpse.regen(mid).unwrap().mp_gain) as i64
                - (current.regen(mid).unwrap().mp_loss - corpse.regen(mid).unwrap().mp_loss) as i64
        );
        let mut casts = 0;
        while current.world.phase == world::Phase::Active {
            pause(&deadline, Duration::from_millis(2100));
            current = cast(&mut sun, sc, &current);
            casts += 1;
            assert!(casts <= 5, "bounded synthetic fire bonus, real 2s reuse");
            moon.ping().unwrap();
            assert!(current
                .fire(sid)
                .unwrap()
                .targets
                .iter()
                .any(|t| t.target_entity_id == tid));
        }
        assert_eq!(honor::target_hp(&current, tid), Some(0));
        assert_eq!(
            current
                .honor
                .as_ref()
                .unwrap()
                .objective(tid)
                .unwrap()
                .last_kind,
            honor::Interaction::FireStorm
        );
    } else {
        assert!(enemies.iter().all(|id| current.vital(*id).unwrap().hp == 0));
        assert_eq!(current.world.phase, world::Phase::Finished);
    }
    let frozen = current.fires.clone();
    while current.world.completion.as_ref().unwrap().state != world::SettlementState::Saved {
        deadline.pause("Fire Storm durable source-qualified SQL");
        current = track(&current, read(&mut sun, sc, &current.world), sc);
    }
    assert_eq!(current.fires, frozen);
    assert_eq!(
        current.world.completion.as_ref().unwrap().event.mode,
        if honor_mode { 5 } else { 1 }
    );
    let final_image = combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
    inspect_image(&final_image, &format!("{mode}-saved"));
    let won = sun.record().unwrap().totals.unwrap();
    let lost = moon.record().unwrap().totals.unwrap();
    assert_eq!((won.wins, won.losses, won.revision), (1, 0, 1));
    assert_eq!((lost.wins, lost.losses, lost.revision), (0, 1, 1));
    assert_eq!(sun.record().unwrap().totals, Some(won));
    assert_eq!(moon.record().unwrap().totals, Some(lost));
    sun.world(world_request(sc, &current.world, world::Action::Leave))
        .unwrap();
    moon.world(world_request(mc, &current.world, world::Action::Leave))
        .unwrap();
    let returned = sun
        .rooms(rooms::Request::read(sc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    // Existing room policy un-readies every human after a match. Server-owned
    // test BOTs remain ready; they do not consent on behalf of either human.
    assert_eq!(returned.summary.ready, if honor_mode { 0 } else { 2 });
    assert!(!returned.summary.satisfied);
    for member in &returned.members {
        assert_eq!(member.ready, member.identity().character().is_none());
    }
    sr.shutdown();
    mr.shutdown();
    while !sr.is_quiescent() || !mr.is_quiescent() {
        sr.poll();
        mr.poll();
        deadline.pause("Fire Storm worker retirement");
    }
    sun.close().unwrap();
    moon.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
#[test]
#[ignore = "v14 Java Fire Storm + isolated MySQL + real shared514/SPR; no GUI"]
fn java_fire_storm_empty_cast_pc_bot_atomic_multikill_saved_sql() {
    run("fire", false);
}
#[test]
#[ignore = "v37 life-bound MOVE and fire storm, empty/cooldown/PC+BOT kill, shared514 and isolated SQL; no HWND"]
fn java_v37_fire_storm_guarded_move_empty_cast_cooldown_pc_bot_multikill_and_saved_sql() {
    run("life-fire", false);
}
#[test]
#[ignore = "v14 Java Fire Storm Honor + isolated MySQL + real shared532/SPR; no GUI"]
fn java_fire_storm_honor_pc_tower_respawn_destroy_saved_sql() {
    run("fire-honor", true);
}
