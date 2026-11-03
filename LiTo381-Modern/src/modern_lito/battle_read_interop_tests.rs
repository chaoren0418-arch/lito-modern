//! Actual authenticated canonical TLS reads, operations and lifecycle. LOAD
//! requires full shared-resource qualification, never baseline pixels alone.
use super::*;
#[path = "battle_equipment_interop_tests.rs"]
mod live_equipment;
#[path = "battle_greater_haste_interop_tests.rs"]
mod live_greater_haste;
#[path = "battle_slow_sql_interop_tests.rs"]
mod live_slow_sql;
#[path = "battle_target_haste_interop_tests.rs"]
mod live_target_haste;
#[path = "battle_preflight_interop_tests.rs"]
mod preflight;
use crate::modern_lito::{
    battle_payload::data,
    battle_read,
    battle_runtime::{BattleRuntime, Status as BattleStatus},
    battle_world, protocol,
    transport::ModernTlsConnection,
};

fn worker_surface(
    runtime: &mut BattleRuntime,
    expected: &crate::modern_lito::battle_follow::Observation,
    deadline: &Deadline,
) -> Arc<crate::modern_lito::battle_scene::Surface> {
    loop {
        deadline.check("canonical worker exact surface");
        match runtime.poll() {
            BattleStatus::Ready(surface) if surface.observation.data() == expected.data() => {
                assert_eq!(surface.bgra.len(), 800 * 600 * 4);
                assert_eq!(surface.geometry.actors.len(), 2);
                let navigation = surface
                    .navigation
                    .as_ref()
                    .expect("qualified canonical navigation");
                assert!(navigation.matches(&surface.observation));
                assert_eq!(
                    navigation.map().canonical_digest(),
                    expected.data().manifest.digest
                );
                assert_eq!(
                    surface.geometry.revision as i64,
                    expected.personal().revision()
                );
                return surface;
            }
            BattleStatus::Loading | BattleStatus::Ready(_) => {}
            status => panic!("canonical worker failed: {status:?}"),
        }
        deadline.pause("canonical worker awaiting pixels");
    }
}

fn join_worker(runtime: &mut BattleRuntime, deadline: &Deadline) {
    while !runtime.is_quiescent() {
        deadline.check("canonical worker owned retirement");
        runtime.poll();
        deadline.pause("canonical worker joining");
    }
}

fn connect(
    peer: &JavaPeer,
    ca: &[u8],
    context: SelectionContext,
    version: u16,
) -> ModernTlsConnection {
    let mut hello = Hello::current(context.key.source_id);
    hello.max_version = version;
    hello.supported_capabilities |= protocol::CAP_BATTLE_DATA;
    if version >= 37 {
        hello.supported_capabilities |= protocol::CAP_ACTION_LIFE;
    }
    if version >= 42 {
        hello.supported_capabilities |= protocol::CAP_GREATER_HASTE;
    }
    if version >= 44 {
        hello.supported_capabilities |= protocol::CAP_HASTE;
    }
    if version >= 47 {
        hello.supported_capabilities |= protocol::CAP_SLOW_DATA;
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
    assert!(c.supports_battle_data());
    c.authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    assert_eq!(
        c.characters(context.key.source_id, context.epoch)
            .unwrap()
            .characters[0]
            .key,
        context.key
    );
    c.select_character(context).unwrap();
    c.enter_channel(CHANNEL).unwrap();
    c
}

fn read(
    c: &mut ModernTlsConnection,
    request: &battle_read::Request,
    follower: &mut crate::modern_lito::battle_follow::Follower,
) -> battle_read::UnvalidatedReply {
    let legacy = c.world_snapshot().cloned();
    let result = if c.supports_battle_world() {
        let reply = c
            .battle_world(battle_world::Request::read(
                request.context,
                request.channel_id,
                Some(&request.world),
            ))
            .unwrap();
        assert_eq!(reply.status, world::Status::Ok);
        assert_world_snapshot(c, &reply);
        // Test-only feed of the exact canonical payload to the existing
        // temporal follower. This grants no lifecycle or LOAD authority.
        battle_read::UnvalidatedReply {
            echo: request.clone(),
            state: match reply.state {
                battle_world::State::BeforeEngine => battle_read::State::Preparing,
                battle_world::State::Data => battle_read::State::Data,
                battle_world::State::Detached => panic!("attached read detached"),
            },
            data: reply.data,
        }
    } else {
        c.read_battle(request.clone()).unwrap()
    };
    assert_eq!(result.echo, *request);
    follower
        .accept(result.clone())
        .expect("actual TLS canonical follower");
    let private = result.personal_view().unwrap();
    if let Some(data) = &result.data {
        let private = private.unwrap();
        assert_eq!(private.player(), &request.player());
        assert_eq!(private.world(), &request.world);
        assert_eq!(private.revision(), data.world.revision);
        assert_eq!(
            private.source().is_some(),
            data.personal.equipment.is_some()
        );
    } else {
        assert!(private.is_none());
    }
    assert_eq!(
        legacy.as_ref(),
        c.world_snapshot(),
        "new reads must not replace legacy caches"
    );
    result
}

fn assert_world_snapshot(c: &ModernTlsConnection, reply: &battle_world::Reply) {
    let snapshot = c.battle_snapshot().expect("confirmed v39 connection state");
    assert!(snapshot.attachment() == reply.attachment.as_ref());
    assert_eq!(
        snapshot.observation().map(|o| o.data()),
        reply.data.as_ref()
    );
    if reply.attachment.as_ref().is_none_or(|a| {
        matches!(
            a.lifecycle.phase,
            world::Phase::Preparing | world::Phase::Aborted
        )
    }) {
        assert!(snapshot.presentation_observation().is_none());
    } else {
        assert_eq!(
            snapshot.presentation_observation().map(|o| o.data()),
            reply.data.as_ref()
        );
    }
}

struct CanonicalPair {
    deadline: Deadline,
    shared: PathBuf,
    peer: JavaPeer,
    a: ModernTlsConnection,
    b: ModernTlsConnection,
    sun: SelectionContext,
    moon: SelectionContext,
    ticket: world::Frame,
    request: battle_read::Request,
    guest_request: battle_read::Request,
    owner_follow: crate::modern_lito::battle_follow::Follower,
    guest_follow: crate::modern_lito::battle_follow::Follower,
    received: data::UnvalidatedData,
    guest: data::UnvalidatedData,
}
fn canonical_pair() -> CanonicalPair {
    canonical_pair_versions(35, 35)
}
fn canonical_pair_versions(owner_version: u16, guest_version: u16) -> CanonicalPair {
    let ReadyPair {
        deadline,
        shared,
        peer,
        a,
        b,
        sun,
        moon,
        ready,
    } = ready_pair(owner_version, guest_version);
    canonical_prepared_pair(deadline, shared, peer, a, b, sun, moon, ready)
}
struct ReadyPair {
    deadline: Deadline,
    shared: PathBuf,
    peer: JavaPeer,
    a: ModernTlsConnection,
    b: ModernTlsConnection,
    sun: SelectionContext,
    moon: SelectionContext,
    ready: rooms::RoomSnapshot,
}
fn ready_pair(owner_version: u16, guest_version: u16) -> ReadyPair {
    ready_pair_mode(owner_version, guest_version, None)
}
fn ready_pair_mode(
    owner_version: u16,
    guest_version: u16,
    mode: Option<&'static str>,
) -> ReadyPair {
    let deadline = Deadline::new();
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let ca = private_root(&fixtures);
    let peer = match mode {
        Some("target-haste") => JavaPeer::start_class(
            &fixtures,
            &deadline,
            "local.lito381.server.ModernHasteTestPeer",
        ),
        Some("greater-haste") => JavaPeer::start_class(
            &fixtures,
            &deadline,
            "local.lito381.server.ModernGreaterHasteTestPeer",
        ),
        Some(mode) => JavaPeer::start_mode(
            &fixtures,
            &deadline,
            "local.lito381.server.ModernCombatTestPeer",
            mode,
        ),
        None => JavaPeer::start(&fixtures, &deadline),
    };
    let sun = SelectionContext {
        key: SUN,
        epoch: u64::MAX,
    };
    let moon = SelectionContext {
        key: MOON,
        epoch: 1,
    };
    let mut a = connect(&peer, &ca, sun, owner_version);
    let mut b = connect(&peer, &ca, moon, guest_version);
    let created = a
        .rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(sun, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 4,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "Canonical read".into(),
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
    let ready = a
        .rooms(room_command(sun, &joined, rooms::Action::Ready, 1))
        .unwrap()
        .joined
        .unwrap();
    let ready = b
        .rooms(room_command(moon, &ready, rooms::Action::Ready, 1))
        .unwrap()
        .joined
        .unwrap();
    assert!(ready.summary.satisfied);
    ReadyPair {
        deadline,
        shared,
        peer,
        a,
        b,
        sun,
        moon,
        ready,
    }
}
fn canonical_prepared_pair(
    deadline: Deadline,
    shared: PathBuf,
    peer: JavaPeer,
    mut a: ModernTlsConnection,
    mut b: ModernTlsConnection,
    sun: SelectionContext,
    moon: SelectionContext,
    ready: rooms::RoomSnapshot,
) -> CanonicalPair {
    // Existing explicit command obtains the service ticket, not an LTB1 view.
    let ticket = if a.supports_battle_world() {
        let empty = a
            .battle_world(battle_world::Request::read(sun, CHANNEL, None))
            .unwrap();
        assert_eq!(empty.state, battle_world::State::Detached);
        assert!(empty.data.is_none());
        assert!(a.world(world::Request::read(sun, CHANNEL)).is_err());
        a.ping().unwrap();
        let prepared = a
            .battle_world(battle_world::Request {
                action: battle_world::Action::Prepare,
                room: ready.summary.room_id as i64,
                room_revision: ready.summary.revision as i64,
                ..battle_world::Request::read(sun, CHANNEL, None)
            })
            .unwrap();
        assert_eq!(prepared.status, world::Status::Ok);
        assert_eq!(prepared.state, battle_world::State::BeforeEngine);
        assert!(prepared.data.is_none());
        let attached = prepared.attachment.unwrap();
        assert_eq!(attached.lifecycle.phase, world::Phase::Preparing);
        assert_eq!(attached.lifecycle.revision, 1);
        // Only the OLD guest receives a genuine old frame for its existing
        // complete SceneOnly preflight. No canonical-to-legacy synthesis.
        let frame = b
            .world(world::Request::read(moon, CHANNEL))
            .unwrap()
            .frame
            .unwrap();
        assert_eq!(frame.match_id as i64, attached.world.match_id);
        assert_eq!(frame.generation as i64, attached.world.generation);
        assert!(a.world_snapshot().is_none());
        frame
    } else {
        a.world(world::Request {
            action: world::Action::Prepare,
            room_id: ready.summary.room_id,
            expected_room_revision: ready.summary.revision,
            ..world::Request::read(sun, CHANNEL)
        })
        .unwrap()
        .frame
        .unwrap()
    };
    let request = battle_read::Request {
        context: sun,
        channel_id: CHANNEL,
        world: data::WorldId {
            room: ticket.room.summary.room_id as i64,
            match_id: ticket.match_id as i64,
            generation: ticket.generation as i64,
        },
    };
    let guest_request = battle_read::Request {
        context: moon,
        ..request.clone()
    };
    let mut owner_follow =
        crate::modern_lito::battle_follow::Follower::new(request.clone(), a.ack().unwrap().clone())
            .unwrap();
    let mut guest_follow = crate::modern_lito::battle_follow::Follower::new(
        guest_request.clone(),
        b.ack().unwrap().clone(),
    )
    .unwrap();
    let received = loop {
        deadline.check("canonical publication prepared");
        let r = read(&mut a, &request, &mut owner_follow);
        match r.state {
            battle_read::State::Preparing => {
                assert!(r.data.is_none());
                deadline.pause("canonical preparation");
            }
            battle_read::State::Data => break r.data.unwrap(),
            battle_read::State::Unavailable => panic!("shared preparation unavailable"),
        }
    };
    let guest = read(&mut b, &guest_request, &mut guest_follow)
        .data
        .unwrap();
    assert_eq!(received.world.phase, data::Phase::Loading);
    assert_eq!(guest.world.phase, data::Phase::Loading);
    assert_eq!(received.world.identity, guest.world.identity);
    assert_eq!(received.personal.player, request.player());
    assert_eq!(guest.personal.player, guest_request.player());
    assert_ne!(received.personal.entity, guest.personal.entity);
    assert_eq!(received.actors.0.len(), 2);
    assert!(received.actors.0.iter().all(|actor| !actor.loaded));
    assert!(received.result.is_none());
    assert_eq!(received.manifest, guest.manifest);
    assert_eq!(received.manifest.map, 514);
    CanonicalPair {
        deadline,
        shared,
        peer,
        a,
        b,
        sun,
        moon,
        ticket,
        request,
        guest_request,
        owner_follow,
        guest_follow,
        received,
        guest,
    }
}

#[test]
#[ignore = "real TLS v36/schema2 and v35/schema1 coexistence; shared manifest, no combat effects or GUI"]
fn java_schema_two_tls_coexists_with_schema_one_and_retires_cleanly() {
    let CanonicalPair {
        deadline,
        mut peer,
        mut a,
        mut b,
        sun,
        moon,
        ticket,
        request,
        guest_request,
        mut owner_follow,
        mut guest_follow,
        received,
        guest,
        ..
    } = canonical_pair_versions(36, 35);
    let p = received.presentation.as_ref().unwrap();
    assert_eq!(p.0.len(), 2);
    assert!(p.0.iter().all(|x| x.shot.is_none() && x.item.is_none()));
    assert!(guest.presentation.is_none());
    assert_eq!(received.actors, guest.actors);
    let next = read(&mut a, &request, &mut owner_follow).data.unwrap();
    assert_eq!(next.presentation, received.presentation);
    a.world(world::Request {
        action: world::Action::Leave,
        match_id: ticket.match_id,
        generation: ticket.generation,
        ..world::Request::read(sun, CHANNEL)
    })
    .unwrap();
    assert_eq!(
        read(&mut b, &guest_request, &mut guest_follow)
            .data
            .unwrap()
            .world
            .phase,
        data::Phase::Aborted
    );
    b.world(world::Request {
        action: world::Action::Leave,
        match_id: ticket.match_id,
        generation: ticket.generation,
        ..world::Request::read(moon, CHANNEL)
    })
    .unwrap();
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit(&deadline);
}

#[test]
#[ignore = "requires compiled Java peer and explicit shared Lineage root/TLS fixtures; no DB or GUI"]
fn java_battle_read_tls_two_sources_shared_manifest_without_legacy_conversion_or_load_ack() {
    let CanonicalPair {
        deadline,
        shared,
        mut peer,
        mut a,
        mut b,
        sun,
        moon,
        ticket,
        request,
        guest_request,
        mut owner_follow,
        mut guest_follow,
        received,
        guest: _,
    } = canonical_pair();
    let bindings: Vec<_> = received
        .manifest
        .sectors
        .0
        .iter()
        .map(|s| SectorBinding {
            key: SharedMapSectorKey {
                map_id: received.manifest.map.try_into().unwrap(),
                sector_x: s.x.try_into().unwrap(),
                sector_y: s.y.try_into().unwrap(),
            },
            byte_length: s.byte_length.try_into().unwrap(),
            sha256: s.sha256,
        })
        .collect();
    let map = WorldMap::load_shared(&shared, &bindings, &AtomicBool::new(false)).unwrap();
    assert_eq!(map.canonical_digest(), received.manifest.digest);
    let spawns: Vec<_> = received
        .actors
        .0
        .iter()
        .map(|a| WorldCell {
            x: a.position.x,
            y: a.position.y,
        })
        .collect();
    map.validate_spawns(&spawns, &AtomicBool::new(false))
        .unwrap();
    // Direct canonical path: no synthesized world/combat frame, no revision
    // translation. Pixels and private overlays keep one immutable observation.
    let retained = owner_follow.observation().unwrap().clone();
    crate::modern_lito::shared_action_catalog::test_counts::reset();
    let mut scene = crate::modern_lito::battle_scene::SharedScene::load(
        &shared,
        retained.clone(),
        &AtomicBool::new(false),
    )
    .expect("actual canonical shared scene");
    let loads = crate::modern_lito::shared_action_catalog::test_counts::snapshot();
    assert_eq!((loads.load_attempts, loads.reads, loads.parses), (1, 1, 1));
    let now = Instant::now();
    let surface = scene.render(now, &AtomicBool::new(false)).unwrap();
    assert_eq!(surface.observation.data(), retained.data());
    assert_eq!(
        surface.geometry.revision as i64,
        retained.personal().revision()
    );
    assert_eq!(
        surface.geometry.world_id as i64,
        retained.data().world.identity.match_id
    );
    assert_eq!(surface.geometry.actors.len(), 2);
    assert_eq!(surface.bgra.len(), 800 * 600 * 4);
    assert!(surface.bgra.chunks_exact(4).all(|p| p[3] == 255));
    assert!(surface
        .bgra
        .chunks_exact(4)
        .any(|p| p != &surface.bgra[..4]));
    if let Some(directory) = std::env::var_os("LITO_CANONICAL_CAPTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .canonicalize()
            .unwrap();
        assert!(directory.is_absolute() && directory.canonicalize().unwrap().starts_with(&target));
        let output = directory.join("canonical-tls-baseline.png");
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output)
            .unwrap();
        let mut rgba = surface.bgra.to_vec();
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        let mut encoder = png::Encoder::new(file, surface.width, surface.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&rgba)
            .unwrap();
        eprintln!("canonical TLS baseline pixels: {}", output.display());
    }
    assert!(scene.render(now, &AtomicBool::new(true)).is_err());
    assert_eq!(
        scene.render(now, &AtomicBool::new(false)).unwrap().bgra,
        surface.bgra
    );
    let after = crate::modern_lito::shared_action_catalog::test_counts::snapshot();
    assert_eq!((after.load_attempts, after.reads, after.parses), (1, 1, 1));
    eprintln!(
        "canonical shared pixels={} actors={} engine={} private={} catalogReads=1; no LOAD ACK",
        surface.bgra.len(),
        surface.geometry.actors.len(),
        surface.geometry.revision,
        surface.observation.personal().revision()
    );
    // Wrong local selection is rejected without transmitting/closing the owner.
    assert!(a.read_battle(guest_request.clone()).is_err());
    a.ping().unwrap();
    assert_eq!(
        read(&mut a, &request, &mut owner_follow)
            .data
            .unwrap()
            .world
            .phase,
        data::Phase::Loading
    );
    let next = owner_follow.observation().unwrap().clone();
    assert!(scene.observe(next.clone(), &AtomicBool::new(true)).is_err());
    assert_eq!(scene.observation().data(), retained.data());
    assert!(scene
        .observe(
            guest_follow.observation().unwrap().clone(),
            &AtomicBool::new(false)
        )
        .is_err());
    scene
        .observe(next.clone(), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(
        surface.observation.data(),
        retained.data(),
        "retained pixels keep the old exact data"
    );
    let refreshed = scene
        .render(Instant::now(), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(refreshed.observation.data(), next.data());
    assert_eq!(
        refreshed.geometry.revision as i64,
        next.personal().revision()
    );
    // Clock-only canonical updates must reach the renderer without a fake
    // engine event or rewriting any legacy caches.
    assert_eq!(next.data().world.revision, retained.data().world.revision);
    assert!(next.data().world.loading_elapsed_nanos > retained.data().world.loading_elapsed_nanos);
    let repeat = scene.observe(retained.clone(), &AtomicBool::new(false));
    assert!(repeat.is_err());
    assert_eq!(scene.observation().data(), next.data());
    let mut guest_scene = crate::modern_lito::battle_scene::SharedScene::load(
        &shared,
        guest_follow.observation().unwrap().clone(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let guest_surface = guest_scene
        .render(Instant::now(), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(
        guest_surface.observation.personal().player(),
        &guest_request.player()
    );
    assert_ne!(
        guest_surface.observation.personal().player(),
        surface.observation.personal().player()
    );
    // No LOAD ACK: baseline rendering is not complete gameplay readiness.
    // Leaving the actual service ticket
    // creates a canonical abort, observable by the other authenticated owner.
    a.world(world::Request {
        action: world::Action::Leave,
        match_id: ticket.match_id,
        generation: ticket.generation,
        ..world::Request::read(sun, CHANNEL)
    })
    .unwrap();
    let aborted = read(&mut b, &guest_request, &mut guest_follow)
        .data
        .unwrap();
    assert_eq!(aborted.world.phase, data::Phase::Aborted);
    assert!(aborted.result.is_none());
    // Retire the still-attached guest from its own canonical abort. The owner
    // has left, so it must drop its scene rather than reread a detached ticket.
    drop(scene);
    guest_scene
        .observe(
            guest_follow.observation().unwrap().clone(),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(guest_scene
        .render(Instant::now(), &AtomicBool::new(false))
        .is_err());
    assert!(
        b.world_snapshot().is_none(),
        "guest never requested a legacy world frame"
    );
    b.world(world::Request {
        action: world::Action::Leave,
        match_id: ticket.match_id,
        generation: ticket.generation,
        ..world::Request::read(moon, CHANNEL)
    })
    .unwrap();
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit(&deadline);
}

#[test]
#[ignore = "compiled Java world peer, shared Lineage resources/TLS; real owned workers, no DB/GUI/LOAD ACK"]
fn java_battle_worker_two_sources_clock_update_clear_abort_and_owned_shutdown() {
    let CanonicalPair {
        deadline,
        shared,
        mut peer,
        mut a,
        mut b,
        sun,
        moon,
        ticket,
        request,
        guest_request,
        mut owner_follow,
        mut guest_follow,
        received,
        guest,
    } = canonical_pair();
    assert_eq!(received.world.phase, data::Phase::Loading);
    assert_eq!(guest.world.phase, data::Phase::Loading);
    let retained = owner_follow.observation().unwrap().clone();
    let guest_initial = guest_follow.observation().unwrap().clone();
    let mut owner = BattleRuntime::new(shared.clone()).unwrap();
    let mut guest_runtime = BattleRuntime::new(shared).unwrap();
    owner.attach(retained.clone()).unwrap();
    guest_runtime.attach(guest_initial.clone()).unwrap();
    // Start both resource jobs before waiting, keeping network ownership on
    // this caller. Neither job holds a connection or sends LOAD ACK.
    owner.poll();
    guest_runtime.poll();
    let first = worker_surface(&mut owner, &retained, &deadline);
    let guest_first = worker_surface(&mut guest_runtime, &guest_initial, &deadline);
    assert!(first.bgra.chunks_exact(4).all(|p| p[3] == 255));
    assert!(first.bgra.chunks_exact(4).any(|p| p != &first.bgra[..4]));
    assert_eq!(first.observation.personal().player(), &request.player());
    assert_eq!(
        guest_first.observation.personal().player(),
        &guest_request.player()
    );
    assert_ne!(
        first.observation.personal().player(),
        guest_first.observation.personal().player()
    );
    read(&mut a, &request, &mut owner_follow);
    read(&mut b, &guest_request, &mut guest_follow);
    let next = owner_follow.observation().unwrap().clone();
    let guest_next = guest_follow.observation().unwrap().clone();
    assert_eq!(
        next.data().world.phase,
        data::Phase::Loading,
        "loading deadline must not have expired"
    );
    assert_eq!(guest_next.data().world.phase, data::Phase::Loading);
    assert_eq!(next.data().world.revision, retained.data().world.revision);
    assert!(next.data().world.loading_elapsed_nanos > retained.data().world.loading_elapsed_nanos);
    assert!(next.data().actors.0.iter().all(|actor| !actor.loaded));
    assert!(owner.update(guest_next.clone()).is_err());
    owner.update(next.clone()).unwrap();
    guest_runtime.update(guest_next.clone()).unwrap();
    let refreshed = worker_surface(&mut owner, &next, &deadline);
    worker_surface(&mut guest_runtime, &guest_next, &deadline);
    assert_eq!(first.observation.data(), retained.data());
    assert_eq!(refreshed.observation.data(), next.data());
    assert!(owner.update(retained).is_err());
    a.world(world::Request {
        action: world::Action::Leave,
        match_id: ticket.match_id,
        generation: ticket.generation,
        ..world::Request::read(sun, CHANNEL)
    })
    .unwrap();
    owner.clear();
    assert!(matches!(owner.poll(), BattleStatus::Idle));
    join_worker(&mut owner, &deadline);
    let aborted = read(&mut b, &guest_request, &mut guest_follow)
        .data
        .unwrap();
    assert_eq!(aborted.world.phase, data::Phase::Aborted);
    assert_ne!(
        aborted.world.abort,
        data::Abort::LoadTimeout,
        "actual owner retirement, not accidental timeout"
    );
    guest_runtime
        .update(guest_follow.observation().unwrap().clone())
        .unwrap();
    assert!(matches!(guest_runtime.poll(), BattleStatus::Retired));
    join_worker(&mut guest_runtime, &deadline);
    assert!(guest_runtime.update(guest_next).is_err());
    assert_eq!(
        guest_first.observation.data().world.phase,
        data::Phase::Loading
    );
    assert!(b.world_snapshot().is_none());
    b.world(world::Request {
        action: world::Action::Leave,
        match_id: ticket.match_id,
        generation: ticket.generation,
        ..world::Request::read(moon, CHANNEL)
    })
    .unwrap();
    owner.shutdown();
    guest_runtime.shutdown();
    assert!(owner.is_quiescent() && guest_runtime.is_quiescent());
    a.close().unwrap();
    b.close().unwrap();
    peer.expect_successful_exit(&deadline);
    eprintln!("canonical owned workers: two authenticated sources, exact clock/private pixels, clear/abort joined; no LOAD ACK");
}

#[test]
#[ignore = "real TLS movement plus shared sprites; existing complete scene preflight owns LOAD ACK, canonical renderer never does"]
fn java_canonical_confirmed_move_plays_shared_walk_frames_then_idle_without_replay() {
    run_canonical_move(false, false);
}

#[test]
#[ignore = "v38 canonical operation TLS with v37 peer, shared preflight/walking, denied/stale/aborted receipts; no HWND"]
fn java_v38_operation_tls_exact_receipts_and_v37_peer_shared_movement() {
    run_canonical_move(true, false);
}

#[test]
#[ignore = "v39 lifecycle TLS plus v38 peer, real shared preflight/operation/passive refresh/leave; no HWND"]
fn java_v39_world_lifecycle_tls_shared_preflight_and_v38_peer() {
    run_canonical_move(true, true);
}

fn run_canonical_move(operations: bool, lifecycle: bool) {
    let CanonicalPair {
        deadline,
        shared,
        mut peer,
        mut a,
        mut b,
        sun,
        moon,
        ticket,
        request,
        guest_request,
        mut owner_follow,
        mut guest_follow,
        ..
    } = if lifecycle {
        canonical_pair_versions(39, 38)
    } else if operations {
        canonical_pair_versions(38, 37)
    } else {
        canonical_pair()
    };
    assert_eq!(a.supports_battle_operations(), operations);
    assert_eq!(b.supports_battle_operations(), lifecycle);
    assert_eq!(a.supports_battle_world(), lifecycle);
    assert!(!b.supports_battle_world());
    if operations {
        use crate::modern_lito::{battle_action, battle_operation, battle_window::Stamp};
        let proof = battle_action::Guard {
            stamp: Stamp::of(owner_follow.observation().unwrap()),
            target: None,
        };
        let r = a
            .battle_operation(proof, |id, _| {
                world::encode_request(
                    id,
                    &world::Request {
                        action: world::Action::Move,
                        heading: 2,
                        sequence: 1,
                        from_x: 32724,
                        from_y: 32784,
                        ..world_request(sun, &ticket, world::Action::Move)
                    },
                )
            })
            .unwrap();
        assert_eq!(r.rejection, battle_action::Rejection::NotActive);
        assert_eq!(r.disposition, battle_operation::Disposition::Rejected);
        assert_eq!(r.lifecycle.phase, world::Phase::Loading);
        assert!(r.data.actors.0.iter().all(|a| a.last_move_sequence == 0));
    }
    // The v39 owner qualifies its own canonical resources; only an old guest
    // uses WorldRuntime. Other version regressions retain their old loader.
    let guest_loading = b
        .world(world_request(moon, &ticket, world::Action::Read))
        .unwrap()
        .frame
        .unwrap();
    let loading = if lifecycle {
        // Retained solely for the OLD guest's command identities below.
        // Never submitted to the v39 owner's resource worker.
        guest_loading.clone()
    } else {
        a.world(world_request(sun, &ticket, world::Action::Read))
            .unwrap()
            .frame
            .unwrap()
    };
    let mut qualified = WorldRuntime::new(shared.clone()).unwrap();
    let mut canonical_qualified = BattleRuntime::new(shared.clone()).unwrap();
    let mut guest_qualified = WorldRuntime::new(shared.clone()).unwrap();
    if lifecycle {
        canonical_qualified
            .attach(owner_follow.observation().unwrap().clone())
            .unwrap();
    } else {
        qualified
            .update(Arc::new(loading.clone()), actor(&loading, SUN).entity_id)
            .unwrap();
    }
    guest_qualified
        .update(
            Arc::new(guest_loading.clone()),
            actor(&guest_loading, MOON).entity_id,
        )
        .unwrap();
    let mut keepalive = Instant::now();
    let mut canonical_loaded = None;
    loop {
        let first = if lifecycle {
            match canonical_qualified.poll() {
                BattleStatus::Ready(surface) => {
                    canonical_loaded = canonical_qualified
                        .loaded_request(&surface, owner_follow.observation().unwrap());
                    canonical_loaded.is_some()
                }
                BattleStatus::Failed(reason) => panic!("canonical preflight failed: {reason}"),
                _ => false,
            }
        } else {
            rendered(&mut qualified, &loading, SUN).is_some()
        };
        let second = rendered(&mut guest_qualified, &guest_loading, MOON);
        if first && second.is_some() {
            break;
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            a.ping().unwrap();
            b.ping().unwrap();
            keepalive = Instant::now();
        }
        deadline.pause("qualified existing SceneOnly barrier");
    }
    for (connection, context) in [(&mut a, sun), (&mut b, moon)] {
        if connection.supports_battle_world() {
            assert!(
                qualified.is_quiescent(),
                "v39 must not start a legacy loader"
            );
            let reply = connection
                .battle_world(canonical_loaded.take().unwrap())
                .unwrap();
            assert_eq!(reply.status, world::Status::Ok);
            assert_eq!(reply.state, battle_world::State::Data);
            let d = reply.data.unwrap();
            assert!(
                d.actors
                    .0
                    .iter()
                    .find(|x| x.entity == d.personal.entity)
                    .unwrap()
                    .loaded
            );
            assert!(connection.world_snapshot().is_none());
            continue;
        }
        let reply = connection
            .world(world::Request {
                manifest_digest: loading.manifest.digest,
                ..world_request(context, &loading, world::Action::Loaded)
            })
            .unwrap();
        assert_eq!(reply.status, world::Status::Ok);
    }
    qualified.shutdown();
    canonical_qualified.shutdown();
    guest_qualified.shutdown();
    while !qualified.is_quiescent()
        || !guest_qualified.is_quiescent()
        || !canonical_qualified.is_quiescent()
    {
        qualified.poll();
        canonical_qualified.poll();
        guest_qualified.poll();
        deadline.pause("qualified scene retirement");
    }
    drop(qualified);
    drop(guest_qualified);
    loop {
        let view = read(&mut a, &request, &mut owner_follow).data.unwrap();
        read(&mut b, &guest_request, &mut guest_follow);
        if view.world.phase == data::Phase::Active {
            break;
        }
        assert_eq!(view.world.phase, data::Phase::Countdown);
        deadline.pause("real countdown before canonical movement");
    }
    let active = owner_follow.observation().unwrap().clone();
    let mut scene = crate::modern_lito::battle_scene::SharedScene::load(
        &shared,
        active.clone(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let retained = scene
        .render(Instant::now(), &AtomicBool::new(false))
        .unwrap();
    let entity = active.data().personal.entity;
    let before = active
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == entity)
        .unwrap();
    let from = WorldCell {
        x: before.position.x,
        y: before.position.y,
    };
    let (heading, destination) = DIRECTIONS
        .iter()
        .enumerate()
        .map(|(heading, &(dx, dy))| {
            (
                heading as u8,
                WorldCell {
                    x: from.x + dx,
                    y: from.y + dy,
                },
            )
        })
        .find(|(_, to)| {
            scene.map().can_step(from, *to)
                && active
                    .data()
                    .actors
                    .0
                    .iter()
                    .all(|a| a.position.x != to.x || a.position.y != to.y)
        })
        .expect("actual shared spawn legal unoccupied edge");
    let next_sequence = before.last_move_sequence as u64 + if operations { 2 } else { 1 };
    let command = world::Request {
        action: world::Action::Move,
        heading,
        sequence: next_sequence,
        from_x: from.x,
        from_y: from.y,
        match_id: ticket.match_id,
        generation: ticket.generation,
        ..world::Request::read(sun, CHANNEL)
    };
    if operations {
        use crate::modern_lito::{battle_action, battle_operation, battle_window::Stamp};
        let proof = battle_action::Guard {
            stamp: Stamp::of(&active),
            target: None,
        };
        let mut stale = proof;
        stale.stamp.life += 1;
        let first = world::Request {
            sequence: next_sequence - 1,
            ..command
        };
        let r = a
            .battle_operation(stale, |id, _| world::encode_request(id, &first))
            .unwrap();
        assert_eq!(r.rejection, battle_action::Rejection::StaleLife);
        assert_eq!(r.disposition, battle_operation::Disposition::Rejected);
        assert_eq!(
            r.data
                .actors
                .0
                .iter()
                .find(|a| a.entity == entity)
                .unwrap()
                .last_move_sequence,
            before.last_move_sequence
        );
        let bad_position = world::Request {
            from_x: from.x + 10,
            ..first
        };
        let denied = a
            .battle_operation(proof, |id, _| world::encode_request(id, &bad_position))
            .unwrap();
        assert_eq!(denied.rejection, battle_action::Rejection::None);
        assert_eq!(denied.disposition, battle_operation::Disposition::Denied);
        assert_eq!(denied.reason, battle_operation::Reason::StalePosition);
        let denied_actor = denied
            .data
            .actors
            .0
            .iter()
            .find(|a| a.entity == entity)
            .unwrap();
        assert_eq!(denied_actor.position, before.position);
        assert_eq!(denied_actor.last_move_sequence, next_sequence as i64 - 1);
        // A denial consumes this move request sequence. Feed its real canonical
        // observation before the next accepted step, as the player controller
        // does. Skipping it deliberately suppresses inferred single-step art.
        owner_follow
            .accept(battle_read::UnvalidatedReply {
                echo: request.clone(),
                state: battle_read::State::Data,
                data: Some(denied.data.clone()),
            })
            .unwrap();
        scene
            .observe_at(
                owner_follow.observation().unwrap().clone(),
                Instant::now(),
                &AtomicBool::new(false),
            )
            .unwrap();
        let legacy_before = a.world_snapshot().cloned();
        let moved = a
            .battle_operation(proof, |id, _| world::encode_request(id, &command))
            .unwrap();
        assert_eq!(moved.disposition, battle_operation::Disposition::Applied);
        assert_eq!(moved.rejection, battle_action::Rejection::None);
        assert_eq!(moved.reason, battle_operation::Reason::None);
        assert_eq!(moved.lifecycle.phase, world::Phase::Active);
        assert_eq!(moved.lifecycle.settlement, None);
        assert_eq!(moved.data.personal.player, request.player());
        assert_eq!(moved.data.world.identity, request.world);
        assert!(moved.data.presentation.is_some());
        let actor = moved
            .data
            .actors
            .0
            .iter()
            .find(|a| a.entity == entity)
            .unwrap();
        assert_eq!(
            (actor.position.x, actor.position.y),
            (destination.x, destination.y)
        );
        assert_eq!(actor.last_move_sequence, next_sequence as i64);
        assert_eq!(
            a.world_snapshot(),
            legacy_before.as_ref(),
            "new receipt must not overwrite legacy cache"
        );
        assert_eq!(
            denied
                .data
                .actors
                .0
                .iter()
                .find(|a| a.entity == entity)
                .unwrap()
                .position
                .x,
            from.x
        );
    } else {
        let move_reply = a.world(command).unwrap();
        assert_eq!(move_reply.status, world::Status::Ok);
    }
    // No conversion of move_reply.frame: actual canonical service publication
    // is the ONLY source of the new presentation's actor/private state.
    read(&mut a, &request, &mut owner_follow);
    let moved = owner_follow.observation().unwrap().clone();
    let body = moved
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == entity)
        .unwrap();
    assert_eq!(
        (body.position.x, body.position.y),
        (destination.x, destination.y)
    );
    assert_eq!(body.last_move_sequence, next_sequence as i64);
    let at = Instant::now();
    scene
        .observe_at(moved.clone(), at, &AtomicBool::new(false))
        .unwrap();
    eprintln!(
        "canonical walk qualification: {:?}, step={}ns",
        scene.walk_info(entity as u64).unwrap(),
        body.step_interval_nanos
    );
    let mut frames = BTreeSet::new();
    for millis in [0, 50, 100, 150] {
        let image = scene
            .render(at + Duration::from_millis(millis), &AtomicBool::new(false))
            .unwrap();
        assert_eq!(image.observation.data(), moved.data());
        let drawn = image
            .geometry
            .actors
            .iter()
            .find(|a| a.entity_id == entity as u64)
            .unwrap();
        eprintln!(
            "canonical walk sample {millis}ms: source={} sprite={} subframe={}",
            drawn.source_frame, drawn.sprite_name, drawn.subframe
        );
        assert_eq!(drawn.cell, destination);
        frames.insert((
            drawn.sprite_name.clone(),
            drawn.subframe,
            drawn.source_frame,
        ));
    }
    assert!(frames.len() > 1, "real shared walking sprite must advance");
    let rest_at = at + Duration::from_millis(250);
    let rest = scene.render(rest_at, &AtomicBool::new(false)).unwrap();
    scene
        .observe_at(moved.clone(), rest_at, &AtomicBool::new(false))
        .unwrap();
    let duplicate = scene.render(rest_at, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        rest.bgra, duplicate.bgra,
        "duplicate observation cannot restart walk"
    );
    assert_eq!(retained.observation.data(), active.data());
    assert_eq!(
        retained
            .geometry
            .actors
            .iter()
            .find(|a| a.entity_id == entity as u64)
            .unwrap()
            .cell,
        from
    );
    if operations {
        use crate::modern_lito::{battle_action, battle_operation, battle_window::Stamp};
        b.world(world_request(moon, &ticket, world::Action::Leave))
            .unwrap();
        let proof = battle_action::Guard {
            stamp: Stamp::of(&moved),
            target: None,
        };
        let r = a
            .battle_operation(proof, |id, _| {
                world::encode_request(
                    id,
                    &world::Request {
                        sequence: next_sequence + 1,
                        from_x: destination.x,
                        from_y: destination.y,
                        ..command
                    },
                )
            })
            .unwrap();
        assert_eq!(r.rejection, battle_action::Rejection::NotActive);
        assert_eq!(r.disposition, battle_operation::Disposition::Rejected);
        assert_eq!(r.lifecycle.phase, world::Phase::Aborted);
        assert_eq!(r.data.world.phase, data::Phase::Aborted);
        assert_eq!(r.lifecycle.settlement, None);
        if lifecycle {
            let aborted = a
                .battle_world(battle_world::Request::read(
                    sun,
                    CHANNEL,
                    Some(&request.world),
                ))
                .unwrap();
            assert_eq!(
                aborted.attachment.unwrap().lifecycle.phase,
                world::Phase::Aborted
            );
            let left = a
                .battle_world(battle_world::Request {
                    action: battle_world::Action::Leave,
                    ..battle_world::Request::read(sun, CHANNEL, Some(&request.world))
                })
                .unwrap();
            assert_eq!(left.status, world::Status::Ok);
            assert_eq!(left.state, battle_world::State::Detached);
            assert!(left.attachment.is_none() && left.data.is_none());
            let detached = a
                .battle_world(battle_world::Request::read(sun, CHANNEL, None))
                .unwrap();
            assert_eq!(detached.state, battle_world::State::Detached);
            assert_world_snapshot(&a, &detached);
            assert!(
                a.battle_world(battle_world::Request::read(
                    sun,
                    CHANNEL,
                    Some(&request.world)
                ))
                .is_err(),
                "retired ticket is rejected before server identity failure"
            );
            assert!(
                a.read_battle(request.clone()).is_err(),
                "v39 cannot bypass lifecycle chronology"
            );
            assert!(a
                .battle_operation(proof, |_, _| panic!("retired operation must not encode"))
                .is_err());
            assert!(a.world_snapshot().is_none());
            a.ping().unwrap();
        } else {
            a.world(world_request(sun, &ticket, world::Action::Leave))
                .unwrap();
        }
    } else {
        a.world(world_request(sun, &ticket, world::Action::Leave))
            .unwrap();
        let aborted = read(&mut b, &guest_request, &mut guest_follow)
            .data
            .unwrap();
        assert_eq!(aborted.world.phase, data::Phase::Aborted);
        b.world(world_request(moon, &ticket, world::Action::Leave))
            .unwrap();
    }
    drop(scene);
    a.close().unwrap();
    b.close().unwrap();
    assert!(a.battle_snapshot().is_none() && b.battle_snapshot().is_none());
    peer.expect_successful_exit(&deadline);
    eprintln!("canonical actual TLS movement: {} distinct shared sprite samples, immutable old pixels, duplicate does not restart; LOAD ACK only from existing full preflight", frames.len());
}
