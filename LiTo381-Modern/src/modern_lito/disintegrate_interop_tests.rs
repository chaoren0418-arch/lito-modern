//! Two real TLS owners + shared Lineage map + isolated SQL settlement. Source
//! skill qualification is synthetic. The opt-in render case also uses two real
//! shared-resource workers before LOAD ACK; neither case is device/UI acceptance.
use super::super::{
    disintegrate_cast_protocol as dis, protocol, shop_protocol as shop,
    transport::ModernTlsConnection,
};
use super::*;
#[path = "battle_controller_interop_tests.rs"]
mod canonical_controller;
#[path = "disintegrate_controller_interop_tests.rs"]
mod controller;
fn connect(
    peer: &JavaPeer,
    ca: &[u8],
    key: CharacterKey,
) -> (ModernTlsConnection, SelectionContext) {
    let mut hello = Hello::current(100);
    hello.max_version = 27;
    hello.supported_capabilities |= protocol::CAP_DISINTEGRATE;
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
    assert!(dis::negotiated(&c.ack().unwrap()));
    c.authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    assert_eq!(
        c.characters(key.source_id, 1).unwrap().characters[0].key,
        key
    );
    let context = SelectionContext { key, epoch: 1 };
    c.select_character(context).unwrap();
    let skills = c.skills().unwrap();
    assert_eq!(skills.disintegrate().is_some(), key == SUN);
    assert_eq!(c.record().unwrap().totals.unwrap().revision, 0);
    c.enter_channel(CHANNEL).unwrap();
    for _ in 0..5 {
        assert_eq!(
            c.shop(shop::Request::read(context, CHANNEL, None))
                .unwrap()
                .status,
            shop::Status::Ok
        );
    }
    (c, context)
}
fn read(c: &mut ModernTlsConnection, ctx: SelectionContext, w: &world::Frame) -> combat::Frame {
    c.combat(combat::Request::read(ctx, CHANNEL, w))
        .unwrap()
        .frame
        .unwrap()
}
fn request(f: &combat::Frame, ctx: SelectionContext, target: u64) -> dis::Request {
    dis::Request {
        context: ctx,
        channel_id: CHANNEL,
        match_id: f.world.match_id,
        generation: f.world.generation,
        sequence: f
            .world
            .cast(actor(&f.world, ctx.key).entity_id)
            .unwrap()
            .request_sequence
            + 1,
        caster_life: 1,
        target_entity_id: target,
        target_life: 1,
    }
}
#[test]
#[ignore = "v27 Rust/Java TLS, synthetic qualified skills, shared514 map and isolated MySQL; no UI"]
fn java_disintegrate_typed_life_denial_kill_saved_sql_and_return() {
    run(false);
}
#[test]
#[ignore = "v27 Rust/Java TLS, dual shared514/SPR/717 first render and owned isolated SQL settlement; no HWND/device"]
fn java_disintegrate_dual_shared_render_launch_kill_saved_and_retire() {
    run(true);
}
fn images(
    runtimes: &mut (WorldRuntime, WorldRuntime),
    f: &combat::Frame,
    d: &Deadline,
    sun: &mut ModernTlsConnection,
    moon: &mut ModernTlsConnection,
) -> (Arc<RenderedWorld>, Arc<RenderedWorld>) {
    let shared = Arc::new(f.clone());
    for (runtime, key) in [(&mut runtimes.0, SUN), (&mut runtimes.1, MOON)] {
        runtime
            .update_combat(
                Arc::new(f.world.clone()),
                actor(&f.world, key).entity_id,
                Some(shared.clone()),
            )
            .unwrap();
    }
    let mut ping = Instant::now();
    loop {
        d.check("Disintegrate complete dual shared render");
        if ping.elapsed() >= Duration::from_secs(1) {
            sun.ping().unwrap();
            moon.ping().unwrap();
            ping = Instant::now();
        }
        if let (Some(s), Some(m)) = (
            rendered(&mut runtimes.0, &f.world, SUN),
            rendered(&mut runtimes.1, &f.world, MOON),
        ) {
            assert_eq!(s.combat.as_deref(), Some(f));
            assert_eq!(m.combat.as_deref(), Some(f));
            assert!(s.ready_to_ack && m.ready_to_ack);
            return (s, m);
        }
        d.pause("Disintegrate dual workers");
    }
}
fn capture(image: &RenderedWorld, label: &str) {
    if let Some(dir) = std::env::var_os("LITO_MODERN_DISINTEGRATE_CAPTURE_DIR") {
        let dir = PathBuf::from(dir).canonicalize().unwrap();
        assert!(dir.starts_with(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .canonicalize()
                .unwrap()
        ));
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(format!("tls-{label}.png")))
            .unwrap();
        let mut rgba = image.bgra.to_vec();
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        let mut encoder = png::Encoder::new(file, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&rgba)
            .unwrap();
    }
}
fn run(render: bool) {
    let deadline = Deadline::new();
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernDisintegrateTestPeer",
        "disintegrate",
    );
    assert!(peer.process_job.is_some());
    let (mut sun, sc) = connect(&peer, &ca, SUN);
    let (mut moon, mc) = connect(&peer, &ca, MOON);
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
                        name: "Shared typed Disintegrate".into(),
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
    let mut w = sun
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
    while w.phase == world::Phase::Preparing {
        deadline.pause("skill77 authenticated preparation");
        w = sun
            .world(world_request(sc, &w, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    assert_eq!(w.phase, world::Phase::Loading);
    assert!(w
        .descriptor
        .unwrap()
        .has(world::MatchDescriptor::DISINTEGRATE));
    moon.world(world_request(mc, &w, world::Action::Read))
        .unwrap();
    // This headless protocol fixture verifies the pinned shared map, not a first render.
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let bindings: Vec<_> = w
        .manifest
        .sectors
        .iter()
        .map(|s| SectorBinding {
            key: SharedMapSectorKey {
                map_id: w.manifest.map_id,
                sector_x: s.x,
                sector_y: s.y,
            },
            byte_length: s.byte_length,
            sha256: s.sha256,
        })
        .collect();
    let map = WorldMap::load_shared(&shared, &bindings, &AtomicBool::new(false)).unwrap();
    assert_eq!(map.canonical_digest(), w.manifest.digest);
    let mut runtimes = render.then(|| {
        (
            WorldRuntime::new(shared.clone()).unwrap(),
            WorldRuntime::new(shared).unwrap(),
        )
    });
    if let Some(runtimes) = &mut runtimes {
        let loading = read(&mut sun, sc, &w);
        let pair = images(runtimes, &loading, &deadline, &mut sun, &mut moon);
        assert_eq!(pair.0.manifest_digest, map.canonical_digest());
        assert_eq!(pair.1.manifest_digest, map.canonical_digest());
        capture(&pair.0, "loading-sun");
        capture(&pair.1, "loading-moon");
    }
    sun.world(world::Request {
        manifest_digest: w.manifest.digest,
        ..world_request(sc, &w, world::Action::Loaded)
    })
    .unwrap();
    w = moon
        .world(world::Request {
            manifest_digest: w.manifest.digest,
            ..world_request(mc, &w, world::Action::Loaded)
        })
        .unwrap()
        .frame
        .unwrap();
    while w.phase != world::Phase::Active {
        deadline.pause("skill77 countdown");
        w = sun
            .world(world_request(sc, &w, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    let active = read(&mut sun, sc, &w);
    if let Some(runtimes) = &mut runtimes {
        images(runtimes, &active, &deadline, &mut sun, &mut moon);
    }
    let sid = actor(&w, SUN).entity_id;
    let mid = actor(&w, MOON).entity_id;
    let current = read(&mut moon, mc, &w);
    let denied = moon.disintegrate(request(&current, mc, sid)).unwrap();
    assert_eq!(
        (denied.status, denied.reason),
        (spell::Status::Denied, dis::Reason::Unlearned)
    );
    assert_eq!(
        denied.frame.as_ref().unwrap().disintegrations,
        current.disintegrations
    );
    let current = read(&mut sun, sc, &w);
    let good = request(&current, sc, mid);
    let mut stale = good;
    stale.target_life = 2;
    assert!(sun.disintegrate(stale).is_err());
    sun.ping().unwrap();
    let result = sun.disintegrate(good).unwrap();
    assert_eq!(
        (result.status, result.reason),
        (spell::Status::Ok, dis::Reason::None)
    );
    let mut finished = result.frame.unwrap();
    assert_eq!(finished.world.phase, world::Phase::Finished);
    assert_eq!(finished.vital(mid).unwrap().hp, 0);
    let receipt = finished.disintegrate(sid).unwrap().clone();
    assert_eq!(receipt.count, 1);
    assert_eq!(receipt.lawful, 501);
    assert_eq!(
        finished.vital(sid).unwrap().mp,
        active.vital(sid).unwrap().mp - i64::from(receipt.mp_cost)
    );
    if let Some(runtimes) = &mut runtimes {
        let pair = images(runtimes, &finished, &deadline, &mut sun, &mut moon);
        let mut seen = [BTreeSet::new(), BTreeSet::new()];
        let mut captured = [false; 2];
        let started = Instant::now();
        loop {
            deadline.check("Disintegrate kill effect cue and expiry");
            for (i, image) in [
                rendered(&mut runtimes.0, &finished.world, SUN),
                rendered(&mut runtimes.1, &finished.world, MOON),
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(image) = image {
                    assert_eq!(image.combat.as_deref(), Some(&finished));
                    let sound = image
                        .audio
                        .as_ref()
                        .unwrap()
                        .events
                        .iter()
                        .filter(|e| e.group.candidates() == [717])
                        .collect::<Vec<_>>();
                    for e in &sound {
                        seen[i].insert(e.sequence);
                        assert!(e.bank.as_ref().unwrap().get(717).is_some());
                    }
                    if !sound.is_empty() && !captured[i] {
                        capture(&image, if i == 0 { "kill-sun" } else { "kill-moon" });
                        captured[i] = true;
                    }
                    if started.elapsed() >= Duration::from_secs(4) {
                        assert!(sound.is_empty(), "expired717 must not replay");
                    }
                }
            }
            if started.elapsed() >= Duration::from_secs(4) {
                break;
            }
            deadline.pause("Disintegrate presentation-only ticks");
        }
        assert_eq!(
            seen.map(|s| s.len()),
            [1, 1],
            "one caster Hit/717 per independently owned renderer"
        );
        assert_eq!(captured, [true, true]);
        assert_eq!(pair.0.combat.as_ref().unwrap().vital(mid).unwrap().hp, 0);
        assert_eq!(pair.1.combat.as_ref().unwrap().vital(mid).unwrap().hp, 0);
    }
    assert!(sun.disintegrate(good).is_err());
    sun.ping().unwrap();
    while finished.world.completion.as_ref().unwrap().state != world::SettlementState::Saved {
        deadline.pause("skill77 durable SQL");
        finished = read(&mut sun, sc, &finished.world);
    }
    assert_eq!(finished.disintegrate(sid).unwrap(), &receipt);
    let observed = read(&mut moon, mc, &finished.world);
    assert_eq!(observed.disintegrations, finished.disintegrations);
    let won = sun.record().unwrap().totals.unwrap();
    let lost = moon.record().unwrap().totals.unwrap();
    assert_eq!((won.wins, won.losses, won.revision), (1, 0, 1));
    assert_eq!((lost.wins, lost.losses, lost.revision), (0, 1, 1));
    assert_eq!(sun.record().unwrap().totals, Some(won));
    assert_eq!(moon.record().unwrap().totals, Some(lost));
    sun.world(world_request(sc, &finished.world, world::Action::Leave))
        .unwrap();
    moon.world(world_request(mc, &finished.world, world::Action::Leave))
        .unwrap();
    let returned = sun
        .rooms(rooms::Request::read(sc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    assert_eq!(returned.summary.ready, 0);
    assert!(!returned.summary.satisfied);
    if let Some((s, m)) = &mut runtimes {
        s.clear();
        m.clear();
        while !s.is_quiescent() || !m.is_quiescent() {
            s.poll();
            m.poll();
            deadline.pause("Disintegrate two workers retire");
        }
    }
    sun.close().unwrap();
    moon.close().unwrap();
    peer.expect_successful_exit(&deadline);
}
