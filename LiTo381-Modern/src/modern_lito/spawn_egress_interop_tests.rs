//! A lone human must leave the complete spawn region with every BOT stationary.
//! Actual TLS, shared-map renderer and production Navigation/MOVE acknowledgments;
//! no teleport, attack, potion, BOT movement, native window or source-DB writes.
use super::super::{gm_protocol as gm, transport::ModernTlsConnection};
use super::*;

fn read(c: &mut ModernTlsConnection, context: SelectionContext, w: &world::Frame) -> combat::Frame {
    let reply = c
        .combat(combat::Request::read(context, CHANNEL, w))
        .unwrap();
    assert_eq!(
        (reply.status, reply.reason),
        (combat::Status::Ok, combat::Reason::None)
    );
    reply.frame.unwrap()
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
        "spawn MOVE history"
    );
    next
}

fn outside_spawn_halo(cell: WorldCell, initial: &world::Frame) -> bool {
    initial.actors.iter().all(|a| {
        (i64::from(cell.x) - i64::from(a.position.x))
            .abs()
            .max((i64::from(cell.y) - i64::from(a.position.y)).abs())
            > 1
    })
}

/// Independent target search over the received roster and the ACTUAL decoded
/// map, not a Java allocator helper/returned proof. Production Navigation must
/// subsequently find and execute its own route under the same occupancy.
fn exit_goal(map: &WorldMap, initial: &combat::Frame) -> WorldCell {
    let human = actor(&initial.world, SUN);
    let occupied: BTreeSet<_> = initial
        .world
        .actors
        .iter()
        .filter(|a| a.entity_id != human.entity_id)
        .map(|a| a.position)
        .chain(
            initial
                .honor
                .iter()
                .flat_map(|h| h.objectives.iter().map(|o| o.position)),
        )
        .collect();
    let mut seen = BTreeSet::from([human.position]);
    let mut queue = VecDeque::from([(human.position, 0usize)]);
    while let Some((from, steps)) = queue.pop_front() {
        if steps >= 5 && outside_spawn_halo(from, &initial.world) {
            return from;
        }
        if steps == 64 {
            continue;
        }
        for &(dx, dy) in &DIRECTIONS {
            let (Some(x), Some(y)) = (from.x.checked_add(dx), from.y.checked_add(dy)) else {
                continue;
            };
            let to = WorldCell { x, y };
            if world_path::step_open(map, from, to, &occupied) {
                if seen.insert(to) {
                    assert!(seen.len() <= 65_536, "bounded shared-map search");
                    queue.push_back((to, steps + 1));
                }
            }
        }
    }
    panic!("human cannot leave the complete spawn halo with stationary BOTs");
}

fn no_shortcuts(initial: &combat::Frame, current: &combat::Frame) {
    assert_eq!(current.world.phase, world::Phase::Active);
    assert!(current.world.completion.is_none());
    for a in &current.world.actors {
        let old = initial
            .world
            .actors
            .iter()
            .find(|old| old.entity_id == a.entity_id)
            .unwrap();
        assert_eq!(a.identity(), old.identity());
        if a.kind == rooms::MemberKind::ServerBot {
            assert_eq!(a, old, "BOT must never move");
        }
    }
    // Full-health, zero-MP fixture: no spell/attack can supply an escape.
    for v in &current.vitals {
        let old = initial.vital(v.entity_id).unwrap();
        assert_eq!((v.hp, v.mp), (old.hp, old.mp));
        assert_eq!(
            (
                v.attack_sequence,
                v.attack_revision,
                v.hurt_revision,
                v.death_revision
            ),
            (0, 0, 0, 0)
        );
    }
    assert_eq!(current.world.casts, initial.world.casts);
    assert!(current
        .world
        .casts
        .iter()
        .all(|c| c.request_sequence == 0 && c.cast_count == 0));
    assert_eq!(current.damages, initial.damages);
    assert!(current.damages.iter().all(|c| c.cast_count == 0));
    assert_eq!(current.items, initial.items);
    assert!(current
        .items
        .iter()
        .all(|i| i.request_sequence == 0 && i.use_count == 0));
    assert_eq!(current.ammo, initial.ammo);
    assert!(current
        .ammo
        .iter()
        .all(|a| a.request_sequence == 0 && a.shot_count == 0));
    assert_eq!(current.honor, initial.honor);
    assert_eq!(current.fires, initial.fires);
    assert!(current.fires.iter().all(|f| f.count == 0));
}

fn first_image(
    runtime: &mut WorldRuntime,
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    frame: &combat::Frame,
    d: &Deadline,
) -> Arc<RenderedWorld> {
    runtime
        .update_combat(
            Arc::new(frame.world.clone()),
            actor(&frame.world, SUN).entity_id,
            Some(Arc::new(frame.clone())),
        )
        .unwrap();
    let start = Instant::now();
    let mut ping = start;
    loop {
        d.check("spawn shared-resource render");
        assert!(
            start.elapsed() < Duration::from_secs(28),
            "spawn first render: {:?}",
            runtime.poll()
        );
        if ping.elapsed() >= Duration::from_secs(1) {
            c.ping().unwrap();
            ping = Instant::now();
        }
        if let Some(image) = rendered(runtime, &frame.world, context.key) {
            assert_eq!(
                image.projection.anchor_cell,
                actor(&frame.world, SUN).position
            );
            return image;
        }
        d.pause("spawn renderer");
    }
}

fn run(mode: &'static str, count: u16, honor: bool) {
    let deadline = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernSpawnEgressTestPeer",
        mode,
    );
    assert!(peer.process_job.is_some(), "owned SQL process tree");
    let mut c = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(&ca),
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
        (13, if honor { 32767 } else { 16383 })
    );
    c.authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    assert_eq!(
        c.characters(SUN.source_id, 1).unwrap().characters[0].key,
        SUN
    );
    let context = SelectionContext { key: SUN, epoch: 1 };
    c.select_character(context).unwrap();
    let zero = c.record().unwrap().totals.unwrap();
    assert_eq!((zero.wins, zero.losses, zero.revision), (0, 0, 0));
    assert!(!zero.stored);
    assert!(
        c.skills().unwrap().skills.is_empty(),
        "no learned escape skill"
    );
    c.enter_channel(CHANNEL).unwrap();
    let created = c
        .rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(context, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: count + 1,
                        mode: if honor { 5 } else { 1 },
                        attributes: 16,
                        map_id: if honor { 532 } else { 514 },
                        name: format!("Spawn egress {mode}"),
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
    let bots = c
        .gm_bots(gm::Request {
            context,
            channel_id: CHANNEL,
            room_id,
            expected_revision: created.summary.revision,
            count,
        })
        .unwrap();
    assert_eq!((bots.added, bots.removed), (count, 0));
    let room = bots.room.unwrap().joined.unwrap();
    assert_eq!(room.members.len(), usize::from(count) + 1);
    assert_eq!(
        room.members
            .iter()
            .filter(|m| m.kind == rooms::MemberKind::Human)
            .count(),
        1
    );
    assert!(room
        .members
        .iter()
        .filter(|m| m.kind == rooms::MemberKind::ServerBot)
        .all(|m| m.ready));
    assert!(
        room.members
            .iter()
            .all(|m| !m.source_name.trim().is_empty()),
        "BOTs retain server names"
    );
    let team0 = room.members.iter().filter(|m| m.team == 0).count();
    let team1 = room.members.iter().filter(|m| m.team == 1).count();
    assert!(team0.abs_diff(team1) <= 1);
    let ready = c
        .rooms(room_command(context, &room, rooms::Action::Ready, 1))
        .unwrap()
        .joined
        .unwrap();
    assert!(ready.summary.satisfied);
    let prepare_start = Instant::now();
    let reply = c
        .world(world::Request {
            context,
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
        .unwrap();
    assert_eq!(reply.status, world::Status::Ok, "bounded usable allocation");
    let mut wf = reply.frame.unwrap();
    while wf.phase == world::Phase::Preparing {
        deadline.pause("spawn preparation");
        wf = c
            .world(world_request(context, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    eprintln!(
        "{mode} prepare-to-LOADING {:.3}s",
        prepare_start.elapsed().as_secs_f64()
    );
    assert_eq!(wf.phase, world::Phase::Loading);
    assert_eq!(wf.manifest.map_id, if honor { 532 } else { 514 });
    assert_eq!(wf.manifest.sectors.len(), if honor { 6 } else { 1 });
    assert_eq!(wf.actors.len(), usize::from(count) + 1);
    let loading = read(&mut c, context, &wf);
    assert_eq!(loading.honor.is_some(), honor);
    assert!(
        loading.vitals.iter().all(|v| v.hp == v.max_hp),
        "full-health fixture cannot gain an escape through death"
    );
    assert_eq!(loading.vital(actor(&wf, SUN).entity_id).unwrap().mp, 0);
    let mut runtime = WorldRuntime::new(shared).unwrap();
    let image = first_image(&mut runtime, &mut c, context, &loading, &deadline);
    let map = image.navigation_map().unwrap();
    wf = c
        .world(world::Request {
            manifest_digest: wf.manifest.digest,
            ..world_request(context, &wf, world::Action::Loaded)
        })
        .unwrap()
        .frame
        .unwrap();
    while wf.phase != world::Phase::Active {
        assert!(matches!(
            wf.phase,
            world::Phase::Loading | world::Phase::Countdown
        ));
        deadline.pause("spawn countdown");
        wf = c
            .world(world_request(context, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    let initial = track(&loading, read(&mut c, context, &wf), context);
    no_shortcuts(&initial, &initial);
    let goal = exit_goal(&map, &initial);
    let entity = actor(&wf, SUN).entity_id;
    let mut navigation = Navigation::default();
    navigation
        .request_combat(map.clone(), Arc::new(initial.clone()), entity, goal)
        .unwrap();
    let walk_start = Instant::now();
    let mut current = initial.clone();
    let mut moves = 0;
    loop {
        deadline.check("spawn real navigation");
        assert!(
            walk_start.elapsed() < Duration::from_secs(30),
            "spawn route must finish"
        );
        navigation.poll();
        if let Some(intent) = navigation.next_step_combat(&current, entity, Instant::now()) {
            assert_eq!(actor(&current.world, SUN).position, intent.from);
            let occupied = current
                .world
                .actors
                .iter()
                .filter(|a| a.entity_id != entity)
                .map(|a| a.position)
                .chain(
                    current
                        .honor
                        .iter()
                        .flat_map(|h| h.objectives.iter().map(|o| o.position)),
                )
                .collect();
            assert!(world_path::can_step(
                &map,
                intent.from,
                intent.heading,
                &occupied
            ));
            let &(dx, dy) = DIRECTIONS.get(usize::from(intent.heading)).unwrap();
            assert_eq!(
                intent.to,
                WorldCell {
                    x: intent.from.x.checked_add(dx).unwrap(),
                    y: intent.from.y.checked_add(dy).unwrap()
                }
            );
            let reply = c
                .world(world::Request {
                    heading: intent.heading,
                    sequence: intent.sequence,
                    from_x: intent.from.x,
                    from_y: intent.from.y,
                    ..world_request(context, &current.world, world::Action::Move)
                })
                .unwrap();
            navigation.submitted(intent).unwrap();
            assert_eq!(
                (reply.status, reply.reason),
                (world::Status::Ok, world::Reason::None)
            );
            let next_world = reply.frame.unwrap();
            assert_eq!(
                (
                    actor(&next_world, SUN).position,
                    actor(&next_world, SUN).sequence
                ),
                (intent.to, intent.sequence)
            );
            current = track(&current, read(&mut c, context, &next_world), context);
            no_shortcuts(&initial, &current);
            moves += 1;
            assert!(moves <= 64);
        }
        match navigation.status() {
            NavigationStatus::Arrived => break,
            NavigationStatus::Stopped(reason) => panic!("spawn route stopped: {reason}"),
            _ => deadline.pause("spawn MOVE cadence"),
        }
    }
    assert!(moves >= 5);
    assert_eq!(actor(&current.world, SUN).position, goal);
    assert!(outside_spawn_halo(goal, &initial.world));
    first_image(&mut runtime, &mut c, context, &current, &deadline);
    eprintln!("{mode}: {count} stationary BOTs; {moves} actual MOVE ACKs, {:?} -> {:?}; no skills/attacks/items", actor(&initial.world, SUN).position, goal);
    let leave = c
        .world(world_request(context, &current.world, world::Action::Leave))
        .unwrap();
    assert_eq!(leave.status, world::Status::Ok);
    assert!(leave.frame.as_ref().is_none_or(|f| f.completion.is_none()));
    let returned = c
        .rooms(rooms::Request::read(context, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    assert_eq!(returned.summary.ready, count);
    assert!(!returned.summary.satisfied);
    assert_eq!(c.record().unwrap().totals, Some(zero));
    assert_eq!(
        c.rooms(room_command(context, &returned, rooms::Action::Leave, 0))
            .unwrap()
            .status,
        rooms::Status::Ok
    );
    navigation.cancel();
    runtime.shutdown();
    while !navigation.is_quiescent() || !runtime.is_quiescent() {
        navigation.poll();
        runtime.poll();
        deadline.pause("spawn worker retirement");
    }
    c.close().unwrap();
    peer.expect_successful_exit(&deadline);
}

#[test]
#[ignore = "lone GM + 3 static BOTs, actual shared514/TLS/SQL; no GUI"]
fn java_single_human_three_bots_can_walk_out_of_514_spawn() {
    run("spawn3", 3, false);
}
#[test]
#[ignore = "lone GM + 8 static BOTs, actual shared514/TLS/SQL; no GUI"]
fn java_single_human_eight_bots_can_walk_out_of_514_spawn() {
    run("spawn8", 8, false);
}
#[test]
#[ignore = "lone GM + 20 static BOTs, actual shared514/TLS/SQL; no GUI"]
fn java_single_human_twenty_bots_can_walk_out_of_514_spawn() {
    run("spawn20", 20, false);
}
#[test]
#[ignore = "lone GM + 20 static BOTs, actual shared532 camps/TLS/SQL; no GUI"]
fn java_single_human_twenty_bots_can_walk_out_of_532_honor_spawn() {
    run("spawn-honor20", 20, true);
}
