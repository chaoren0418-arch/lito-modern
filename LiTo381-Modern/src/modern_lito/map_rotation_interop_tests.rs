//! Three complete rounds on two real shared maps, same TLS owners and runtimes.
//! Synthetic qualified PCs/RNG, isolated SQL; not a GUI or full map-art claim.
use super::super::transport::ModernTlsConnection;
use super::*;

fn connect(
    peer: &JavaPeer,
    ca: &[u8],
    key: CharacterKey,
    version: u16,
) -> (ModernTlsConnection, SelectionContext) {
    let mut hello = Hello::current(100);
    if version == 34 {
        hello.supported_capabilities |= super::super::protocol::CAP_LARGE_MAPS;
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
    assert_eq!(c.ack().unwrap().version, version);
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
fn room(c: &mut ModernTlsConnection, context: SelectionContext) -> rooms::RoomSnapshot {
    c.rooms(rooms::Request::read(context, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap()
}
fn draft(
    context: SelectionContext,
    map: u32,
    previous: Option<&rooms::RoomSnapshot>,
) -> rooms::Request {
    capacity_draft(context, map, previous, 2)
}
fn capacity_draft(
    context: SelectionContext,
    map: u32,
    previous: Option<&rooms::RoomSnapshot>,
    capacity: u16,
) -> rooms::Request {
    rooms::Request::draft(
        rooms::Common {
            action: if previous.is_some() {
                rooms::Action::Configure
            } else {
                rooms::Action::Create
            },
            room_id: previous.map_or(0, |r| r.summary.room_id),
            expected_revision: previous.map_or(0, |r| r.summary.revision),
            ..rooms::Common::read(context, CHANNEL)
        },
        rooms::Draft::new(
            rooms::PublicDraft {
                capacity,
                mode: 1,
                attributes: 16,
                map_id: map,
                name: "Pinned shared map rotation".into(),
            },
            String::new(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn combat_read(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    frame: &world::Frame,
) -> combat::Frame {
    c.combat(combat::Request::read(context, CHANNEL, frame))
        .unwrap()
        .frame
        .unwrap()
}

#[test]
#[ignore = "owned Java two-map TLS peer + isolated MySQL + real shared S32/Tile/SPR; no GUI"]
fn java_shared_509_514_509_three_saved_rounds_same_owners_and_renderers() {
    run_map_rotation("rotation", [509, 514, 509]);
}

#[test]
#[ignore = "owned TLS peer, isolated MySQL and shared 551/552 terrain/sprites; no GUI"]
fn java_shared_552_551_552_three_saved_rounds_same_owners_and_renderers() {
    run_map_rotation("rotation-shared", [552, 551, 552]);
}

#[test]
#[ignore = "owned TLS peer, isolated MySQL and whole shared 508/531 terrain/sprites; no GUI"]
fn java_shared_531_508_531_three_saved_rounds_same_owners_and_renderers() {
    run_map_rotation("rotation-shared-508-531", [531, 508, 531]);
}

#[test]
#[ignore = "owned TLS peer, isolated MySQL and whole shared 531/534 terrain/sprites; no GUI"]
fn java_shared_534_531_534_three_saved_rounds_same_owners_and_renderers() {
    run_map_rotation("rotation-shared-531-534", [534, 531, 534]);
}

#[test]
#[ignore = "whole 20-sector shared516, sparse Tile I/O, same two TLS owners/SQL/renderers; no GUI"]
fn java_shared_516_531_516_three_saved_rounds_same_owners_and_renderers() {
    run_map_rotation("rotation-shared-516-531", [516, 531, 516]);
}

#[test]
#[ignore = "whole shared504, compact lossless tiles, two TLS owners/SQL/renderers; no GUI"]
fn java_shared_504_531_504_three_saved_rounds_same_owners_and_renderers() {
    run_map_rotation("rotation-shared-504-531", [504, 531, 504]);
}
#[test]
#[ignore = "whole shared505, compact lossless tiles, two TLS owners/SQL/renderers; no GUI"]
fn java_shared_505_531_505_three_saved_rounds_same_owners_and_renderers() {
    run_map_rotation("rotation-shared-505-531", [505, 531, 505]);
}
#[test]
#[ignore = "whole shared506, compact lossless tiles, two TLS owners/SQL/renderers; no GUI"]
fn java_shared_506_531_506_three_saved_rounds_same_owners_and_renderers() {
    run_map_rotation("rotation-shared-506-531", [506, 531, 506]);
}

fn run_map_rotation(mode: &'static str, maps: [u32; 3]) {
    let scenario_started = Instant::now();
    // Whole 20-31 sector v34 maps include cold SQL/resource setup and three
    // real countdown/combat/settlement rounds. 504 expired the old 60s total;
    // 505/506 took 59/58s. Keep server spell/movement cadences unchanged.
    let budget = if matches!(mode, "rotation-shared-516-531" | "rotation-shared-504-531"
        | "rotation-shared-505-531" | "rotation-shared-506-531") { 180 } else { 60 };
    let deadline = Deadline(scenario_started + Duration::from_secs(budget));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernMapRotationTestPeer",
        mode,
    );
    assert!(peer.process_job.is_some());
    let ca = private_root(&fixtures);
    let version = if mode == "rotation" {
        6
    } else if matches!(
        mode,
        "rotation-shared-516-531"
            | "rotation-shared-504-531"
            | "rotation-shared-505-531"
            | "rotation-shared-506-531"
    ) {
        34
    } else {
        19
    };
    let (mut sun, sc) = connect(&peer, &ca, SUN, version);
    let (mut moon, mc) = connect(&peer, &ca, MOON, version);
    if mode == "rotation-shared" {
        let rejected = sun.rooms(capacity_draft(sc, maps[0], None, 51)).unwrap();
        assert_eq!(rejected.status, rooms::Status::Denied);
        assert!(rejected.joined.is_none());
        assert!(rejected.rooms.is_empty());
        for id in [551, 552] {
            assert_eq!(
                rejected
                    .options
                    .iter()
                    .find(|o| o.mode == 1 && o.map_id == id)
                    .unwrap()
                    .maximum_players,
                50
            );
        }
    }
    let created = sun.rooms(draft(sc, maps[0], None)).unwrap().joined.unwrap();
    let room_id = created.summary.room_id;
    let directory = moon
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap();
    for map_id in maps {
        let option = directory
            .options
            .iter()
            .find(|o| o.mode == 1 && o.map_id == map_id)
            .unwrap();
        assert!(option.world_available);
        assert_eq!(
            option.maximum_players,
            if mode == "rotation-shared" { 50 } else { 100 }
        );
    }
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
    let mut sr = WorldRuntime::new(shared.clone()).unwrap();
    let mut mr = WorldRuntime::new(shared).unwrap();
    let mut prior: Option<world::Frame> = None;
    let mut matches = BTreeSet::new();
    let mut results = BTreeSet::new();
    let mut map_hashes = BTreeMap::new();
    for (index, map_id) in maps.into_iter().enumerate() {
        let before = room(&mut sun, sc);
        assert_eq!(before.summary.room_id, room_id);
        assert_eq!(before.summary.ready, 0);
        if index > 0 {
            if mode == "rotation-shared" {
                assert!(matches!(
                    sun.rooms(capacity_draft(sc, map_id, Some(&before), 51)),
                    Err(super::super::transport::TransportError::InvalidState(_))
                ));
                assert_eq!(
                    room(&mut sun, sc),
                    before,
                    "rejected local oversized intent never mutates the room"
                );
            }
            let changed = sun.rooms(draft(sc, map_id, Some(&before))).unwrap();
            assert_eq!(changed.status, rooms::Status::Ok);
            assert_eq!(changed.joined.unwrap().summary.ready, 0);
        }
        let r = room(&mut sun, sc);
        sun.rooms(room_command(sc, &r, rooms::Action::Ready, 1))
            .unwrap();
        let r = room(&mut moon, mc);
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
            deadline.pause("new map source qualification");
            wf = sun
                .world(world_request(sc, &wf, world::Action::Read))
                .unwrap()
                .frame
                .unwrap();
        }
        assert_eq!(wf.phase, world::Phase::Loading);
        assert_eq!(wf.manifest.map_id, map_id);
        assert!(matches.insert((wf.match_id, wf.generation)));
        assert_eq!(wf.actors.len(), 2);
        let expected_anchor = match map_id {
            504 | 505 | 506 => WorldCell { x: 32933, y: 32862 },
            508 => WorldCell { x: 32832, y: 33151 },
            509 => WorldCell { x: 32740, y: 32796 },
            514 => WorldCell { x: 32724, y: 32784 },
            516 => WorldCell { x: 33123, y: 32772 },
            531 => WorldCell { x: 32799, y: 32797 },
            534 => WorldCell { x: 32817, y: 32832 },
            551 => WorldCell { x: 32729, y: 32790 },
            552 => WorldCell { x: 32736, y: 32858 },
            _ => panic!("not an explicit pinned arena"),
        };
        assert_eq!(actor(&wf, SUN).position, expected_anchor);
        if let Some(old) = &prior {
            assert_ne!(old.manifest.digest, wf.manifest.digest);
            // Transport rejects a stale combat intent before writing. Illegal
            // cross-generation WORLD packets intentionally fail-close on the
            // server and are covered separately, not injected into these owners.
            let stale = sun.combat(combat::Request {
                action: combat::Action::Attack,
                sequence: 1,
                target_entity_id: actor(old, MOON).entity_id,
                ..combat::Request::read(sc, CHANNEL, old)
            });
            assert!(matches!(
                stale,
                Err(super::super::transport::TransportError::InvalidState(_))
            ));
            sun.ping().unwrap();
        }
        moon.world(world_request(mc, &wf, world::Action::Read))
            .unwrap();
        let initial = Arc::new(combat_read(&mut sun, sc, &wf));
        assert!(initial
            .vitals
            .iter()
            .all(|v| v.hp == v.max_hp && v.attack_sequence == 0));
        assert!(initial.damages.iter().all(|d| d.mp_spent == 0));
        let mut keepalive = Instant::now();
        // Deliberately do not clear/replace the runtimes between matches. This
        // exercises direct Some(old) -> Some(new) and actual worker retirement.
        let image = combat_images_with_progress(&mut sr, &mut mr, &initial, &deadline, || {
            if keepalive.elapsed() >= Duration::from_secs(1) {
                sun.ping().unwrap();
                moon.ping().unwrap();
                keepalive = Instant::now();
            }
        });
        assert_eq!(image.navigation_map().unwrap().map_id(), map_id);
        assert_eq!(image.manifest_digest, wf.manifest.digest);
        assert!(image.ready_to_ack);
        eprintln!("MAP_ROTATION_STAGE mode={mode} physical={map_id} stage=shared-render-ready elapsed_ms={}", scenario_started.elapsed().as_millis());
        let hash = image.navigation_map().unwrap().canonical_digest();
        if let Some(previous) = map_hashes.insert(map_id, hash) {
            assert_eq!(previous, hash);
        }
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
        assert_eq!(wf.phase, world::Phase::Countdown);
        while wf.phase != world::Phase::Active {
            deadline.pause("actual three-second map countdown");
            wf = sun
                .world(world_request(sc, &wf, world::Action::Read))
                .unwrap()
                .frame
                .unwrap();
        }
        let mut current = combat_read(&mut sun, sc, &wf);
        let target = actor(&wf, MOON).entity_id;
        let selected = actor(&wf, SUN).entity_id;
        let mut sequence = 0;
        if mode != "rotation" {
            current = teleport_then_walk(
                &mut sun,
                &mut moon,
                sc,
                mc,
                &mut sr,
                &mut mr,
                current,
                &image.navigation_map().unwrap(),
                &deadline,
            );
            sequence = 1;
        }
        while current.world.phase == world::Phase::Active {
            sequence += 1;
            assert!(
                sequence <= 8 + u64::from(mode != "rotation"),
                "bounded eight light-arrow casts plus the explicit teleport"
            );
            let reply = sun
                .cast(spell::CastRequest {
                    context: sc,
                    channel_id: CHANNEL,
                    match_id: wf.match_id,
                    generation: wf.generation,
                    sequence,
                    skill_id: spell::LIGHT_ARROW,
                    target_entity_id: target,
                })
                .unwrap();
            assert_eq!(
                (reply.status, reply.reason),
                (spell::Status::Ok, spell::Reason::None)
            );
            current = reply.frame.unwrap();
            if current.world.phase == world::Phase::Active {
                let until = Instant::now() + Duration::from_millis(750);
                while Instant::now() < until {
                    deadline.pause("retained spell cadence");
                }
            }
        }
        assert_eq!(current.world.phase, world::Phase::Finished);
        assert_eq!(current.vital(target).unwrap().hp, 0);
        assert!(current.vital(selected).unwrap().hp > 0);
        while current.world.completion.as_ref().unwrap().state != world::SettlementState::Saved {
        deadline.pause("map-specific durable settlement");
            current = combat_read(&mut sun, sc, &current.world);
        }
        let completion = current.world.completion.as_ref().unwrap();
        assert_eq!(completion.event.map_id, map_id);
        assert!(results.insert(completion.event.result_id));
        combat_images(&mut sr, &mut mr, &Arc::new(current.clone()), &deadline);
        let win = sun.record().unwrap().totals.unwrap();
        let loss = moon.record().unwrap().totals.unwrap();
        let rounds = index as i64 + 1;
        assert_eq!((win.wins, win.losses, win.revision), (rounds, 0, rounds));
        assert_eq!((loss.wins, loss.losses, loss.revision), (0, rounds, rounds));
        assert_eq!(sun.record().unwrap().totals, Some(win));
        sun.world(world_request(sc, &current.world, world::Action::Leave))
            .unwrap();
        let still_owned = moon
            .world(world_request(mc, &current.world, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
        assert_eq!(
            still_owned.completion, current.world.completion,
            "first owner cannot erase second owner's result"
        );
        moon.world(world_request(mc, &current.world, world::Action::Leave))
            .unwrap();
        assert_eq!(room(&mut sun, sc).summary.ready, 0);
        assert_eq!(room(&mut moon, mc).summary.ready, 0);
        prior = Some(current.world);
    }
    assert_eq!(matches.len(), 3);
    assert_eq!(results.len(), 3);
    assert_ne!(map_hashes[&maps[0]], map_hashes[&maps[1]]);
    sr.shutdown();
    mr.shutdown();
    while !sr.is_quiescent() || !mr.is_quiescent() {
        sr.poll();
        mr.poll();
        deadline.pause("both map workers retired");
    }
    sun.close().unwrap();
    moon.close().unwrap();
    peer.expect_successful_exit(&deadline);
}

/// Actual transport, two existing render workers and original source MP.
/// The server fixture selects an adjacent destination, never client coordinates.
fn teleport_then_walk(
    sun: &mut ModernTlsConnection,
    moon: &mut ModernTlsConnection,
    sc: SelectionContext,
    mc: SelectionContext,
    sr: &mut WorldRuntime,
    mr: &mut WorldRuntime,
    before: combat::Frame,
    map: &super::super::world_map::WorldMap,
    deadline: &Deadline,
) -> combat::Frame {
    let selected = actor(&before.world, SUN).entity_id;
    let from = actor(&before.world, SUN).position;
    // Rust traversal independently checks the arena against the authoritative
    // destination; it does not read Java's candidate list.
    let mut arena = BTreeSet::from([from]);
    let mut queue = std::collections::VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        for (dx, dy) in DIRECTIONS {
            let q = WorldCell {
                x: p.x + dx,
                y: p.y + dy,
            };
            if map.can_step(p, q) && map.can_step(q, p) && arena.insert(q) {
                queue.push_back(q);
            }
        }
        assert!(arena.len() <= super::super::world_map::MAX_SECTORS * 4096);
    }
    let reply = sun
        .cast(spell::CastRequest {
            context: sc,
            channel_id: CHANNEL,
            match_id: before.world.match_id,
            generation: before.world.generation,
            sequence: 1,
            skill_id: 5,
            target_entity_id: 0,
        })
        .unwrap();
    assert_eq!(
        (reply.status, reply.reason),
        (spell::Status::Ok, spell::Reason::None)
    );
    let mut current = reply.frame.unwrap();
    let cast = *current.world.cast(selected).unwrap();
    assert_eq!(cast.from, from);
    assert_eq!(cast.to, actor(&current.world, SUN).position);
    assert_ne!(cast.to, from);
    assert!(arena.contains(&cast.to));
    assert_eq!(
        (cast.request_sequence, cast.cast_count, cast.mp_spent),
        (1, 1, 10)
    );
    assert_eq!(cast.cast_revision, current.world.revision);
    assert_eq!(
        current.vital(selected).unwrap().mp,
        before.vital(selected).unwrap().mp - 10
    );
    assert_eq!(
        current.vital(selected).unwrap().hp,
        before.vital(selected).unwrap().hp
    );
    assert_eq!(
        actor(&current.world, SUN).sequence,
        actor(&before.world, SUN).sequence
    );
    assert_eq!(combat_read(moon, mc, &current.world), current);
    let image = combat_images(sr, mr, &Arc::new(current.clone()), deadline);
    assert_eq!(image.projection.anchor_cell, cast.to);
    let until =
        Instant::now() + Duration::from_millis(u64::from(cast.cast_interval_millis).max(800));
    let mut sound119 = false;
    while Instant::now() < until {
        if let Some(image) = rendered(sr, &current.world, SUN) {
            sound119 |= image
                .audio
                .as_ref()
                .is_some_and(|a| a.events.iter().any(|e| e.group.candidates().contains(&119)));
        }
        deadline.pause("shared teleport effect and original cast cadence");
    }
    assert!(
        sound119,
        "shared effect169 must emit the qualified sound119 cue"
    );
    let heading = DIRECTIONS
        .iter()
        .position(|&(dx, dy)| cast.to.x + dx == from.x && cast.to.y + dy == from.y)
        .unwrap() as u8;
    let moved = sun
        .world(world::Request {
            heading,
            sequence: actor(&current.world, SUN).sequence + 1,
            from_x: cast.to.x,
            from_y: cast.to.y,
            ..world_request(sc, &current.world, world::Action::Move)
        })
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(actor(&moved, SUN).position, from);
    assert_eq!(moved.cast(selected).unwrap(), &cast);
    current = combat_read(sun, sc, &moved);
    assert_eq!(
        current.vital(selected).unwrap().mp,
        before.vital(selected).unwrap().mp - 10
    );
    assert_eq!(
        combat_images(sr, mr, &Arc::new(current.clone()), deadline)
            .projection
            .anchor_cell,
        from
    );
    let until =
        Instant::now() + Duration::from_nanos(actor(&current.world, SUN).step_interval_nanos);
    while Instant::now() < until {
        deadline.pause("original move-to-spell fence");
    }
    current
}
