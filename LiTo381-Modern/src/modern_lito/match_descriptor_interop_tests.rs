//! Same authenticated TLS owners/listener and retained render workers across
//! SceneOnly -> complete Honor -> SceneOnly. Synthetic source/RNG fixtures;
//! actual shared resources, server authority and isolated settlement SQL.
use super::super::{
    honor_protocol as honor, protocol, shop_protocol as shop,
    transport::{ModernTlsConnection, TransportError},
};
use super::*;

const RED: i64 = 40010;
const ARROW: i64 = 40744;
const CAPS: u32 = 32767 | protocol::CAP_MATCH_DESCRIPTOR;

fn descriptor(frame: &world::Frame, honor: bool) -> world::MatchDescriptor {
    let expected = world::MatchDescriptor {
        ruleset: if honor {
            world::MatchRuleset::HonorTowers
        } else {
            world::MatchRuleset::SceneOnly
        },
        features: if honor { 511 } else { 0 },
        preparation_timeout_millis: 3000,
        load_timeout_millis: 30_000,
        countdown_nanos: 3_000_000_000,
        active_timeout_millis: 160_000,
    };
    assert_eq!(frame.descriptor, Some(expected));
    assert_eq!(frame.combat_enabled(true), honor);
    assert_eq!(expected.results(), honor);
    assert_eq!(frame.room.summary.settings.mode, if honor { 5 } else { 1 });
    assert_eq!(
        frame.room.summary.settings.map_id,
        if honor { 532 } else { 514 }
    );
    assert_eq!(frame.manifest.map_id, if honor { 532 } else { 514 });
    expected
}

fn owner_connect(
    peer: &JavaPeer,
    shared: &PathBuf,
    ca: &[u8],
    d: &Deadline,
) -> (SessionController, SelectionContext) {
    let mut owner = SessionController::new(profile(peer.endpoint, shared, ca));
    submit(&mut owner, d, "descriptor verified TLS", || {
        LoginCommand::Connect
    });
    wait_owner(&mut owner, d, "descriptor TLS ready", |v| {
        v.phase == LoginPhase::LoginReady
    });
    submit(&mut owner, d, "descriptor auth", || LoginCommand::Login {
        account: "fixture-user".into(),
        password: "fixture-password".into(),
    });
    wait_owner(&mut owner, d, "descriptor sources", |v| {
        v.phase == LoginPhase::ChoosingSource
    });
    submit(&mut owner, d, "descriptor Sun source", || {
        LoginCommand::SelectSource(SUN.source_id)
    });
    let roster = wait_owner(&mut owner, d, "descriptor Sun roster", |v| {
        v.phase == LoginPhase::CharactersReady
    });
    assert_eq!(roster.characters.len(), 1);
    assert_eq!(
        (
            roster.characters[0].source_id,
            roster.characters[0].object_id
        ),
        (SUN.source_id, SUN.object_id)
    );
    let context = SelectionContext {
        key: SUN,
        epoch: roster.roster_epoch,
    };
    submit(&mut owner, d, "descriptor Sun selection", || {
        LoginCommand::SelectCharacter(context)
    });
    wait_owner(&mut owner, d, "descriptor channels", |v| {
        v.phase == LoginPhase::ChoosingChannel
    });
    let selected = wait_owner(
        &mut owner,
        d,
        "descriptor source skills and SQL record",
        |v| {
            !v.spell_pending
                && v.skills.light_arrow().is_some()
                && !v.record_pending
                && v.record.totals().is_some()
        },
    );
    assert!(
        selected.combat_supported,
        "listener combat capability stays enabled even in SceneOnly"
    );
    assert_eq!(
        selected.record.totals(),
        Some(record::Totals {
            stored: false,
            wins: 0,
            losses: 0,
            revision: 0
        })
    );
    submit(&mut owner, d, "descriptor lobby", || {
        LoginCommand::EnterChannel {
            context,
            channel_id: CHANNEL,
        }
    });
    wait_owner(&mut owner, d, "descriptor lobby ACK", |v| {
        v.phase == LoginPhase::InLobby
    });
    (owner, context)
}

fn guest_connect(peer: &JavaPeer, ca: &[u8]) -> (ModernTlsConnection, SelectionContext) {
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
        (15, CAPS)
    );
    c.authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    assert_eq!(
        c.characters(MOON.source_id, 1).unwrap().characters[0].key,
        MOON
    );
    let context = SelectionContext {
        key: MOON,
        epoch: 1,
    };
    c.select_character(context).unwrap();
    assert_eq!(
        c.record().unwrap().totals,
        Some(record::Totals {
            stored: false,
            wins: 0,
            losses: 0,
            revision: 0
        })
    );
    assert_eq!(c.skills().unwrap().status, spell::Status::Ok);
    c.enter_channel(CHANNEL).unwrap();
    (c, context)
}

fn owner_room(
    owner: &mut SessionController,
    sc: SelectionContext,
    d: &Deadline,
) -> rooms::RoomSnapshot {
    submit(owner, d, "descriptor room read", || {
        LoginCommand::Rooms(rooms::Request::read(sc, CHANNEL).unwrap())
    });
    wait_owner(owner, d, "descriptor room reply", |v| {
        !v.room_pending && v.rooms.joined().is_some()
    })
    .rooms
    .joined()
    .unwrap()
    .clone()
}

fn owner_shop(
    owner: &mut SessionController,
    sc: SelectionContext,
    d: &Deadline,
    action: shop::Action,
    id: i64,
    quantity: i64,
) -> Arc<shop::Snapshot> {
    let before = owner.snapshot();
    let previous = before.shop.snapshot().cloned();
    let read = shop::Request::read(sc, CHANNEL, before.rooms.joined());
    let request = if action == shop::Action::Read {
        read
    } else {
        read.mutation(previous.as_ref().unwrap(), action, id, quantity)
    };
    submit(owner, d, "descriptor shop request", || {
        LoginCommand::Shop(request)
    });
    assert!(owner.snapshot().shop_pending);
    assert_eq!(
        owner.snapshot().shop.snapshot(),
        previous.as_ref(),
        "no optimistic charge/reset"
    );
    let after = wait_owner(owner, d, "descriptor shop ACK", |v| {
        !v.shop_pending && v.shop.reply().is_some_and(|r| r.echo == request)
    });
    let reply = after.shop.reply().unwrap();
    assert_eq!(
        (reply.status, reply.reason),
        (shop::Status::Ok, shop::Reason::None)
    );
    assert_eq!(reply.echo.context, sc);
    reply.snapshot.as_ref().unwrap().clone()
}

fn guest_shop_request(c: &mut ModernTlsConnection, mc: SelectionContext) -> shop::Request {
    let room = c.rooms(rooms::Request::read(mc, CHANNEL).unwrap()).unwrap();
    assert_eq!(
        (room.status, room.reason),
        (rooms::Status::Ok, rooms::Reason::None)
    );
    assert_eq!(room.common.context, mc);
    shop::Request::read(mc, CHANNEL, room.joined.as_ref())
}

fn guest_shop(c: &mut ModernTlsConnection, mc: SelectionContext) -> Arc<shop::Snapshot> {
    let request = guest_shop_request(c, mc);
    let reply = c.shop(request).unwrap();
    assert_eq!(reply.echo, request);
    assert_eq!(
        (reply.status, reply.reason),
        (shop::Status::Ok, shop::Reason::None)
    );
    let room = reply.room.as_ref().unwrap();
    assert_eq!(
        (room.status, room.reason),
        (rooms::Status::Ok, rooms::Reason::None)
    );
    reply.snapshot.unwrap()
}

fn record_owner(
    owner: &mut SessionController,
    sc: SelectionContext,
    d: &Deadline,
) -> record::Totals {
    submit(owner, d, "descriptor source-bound SQL read", || {
        LoginCommand::ReadRecord
    });
    let v = wait_owner(owner, d, "descriptor SQL reply", |v| {
        !v.record_pending && v.record.context() == Some(sc) && v.record.totals().is_some()
    });
    v.record.totals().unwrap()
}

fn pause(
    owner: &mut SessionController,
    guest: &mut ModernTlsConnection,
    d: &Deadline,
    delay: Duration,
) {
    let end = Instant::now() + delay;
    let mut ping = Instant::now();
    while Instant::now() < end {
        owner.poll();
        assert_ne!(owner.snapshot().phase, LoginPhase::Failed);
        if ping.elapsed() >= Duration::from_secs(1) {
            guest.ping().unwrap();
            ping = Instant::now();
        }
        d.pause("descriptor actual monotonic cadence");
    }
}

fn draft(sc: SelectionContext, old: Option<&rooms::RoomSnapshot>, honor: bool) -> rooms::Request {
    rooms::Request::draft(
        rooms::Common {
            action: if old.is_some() {
                rooms::Action::Configure
            } else {
                rooms::Action::Create
            },
            room_id: old.map_or(0, |r| r.summary.room_id),
            expected_revision: old.map_or(0, |r| r.summary.revision),
            ..rooms::Common::read(sc, CHANNEL)
        },
        rooms::Draft::new(
            rooms::PublicDraft {
                capacity: 2,
                mode: if honor { 5 } else { 1 },
                attributes: 16,
                map_id: if honor { 532 } else { 514 },
                name: "Same TLS match descriptor rotation".into(),
            },
            String::new(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn configure(
    owner: &mut SessionController,
    sc: SelectionContext,
    d: &Deadline,
    old: Option<&rooms::RoomSnapshot>,
    honor: bool,
) -> rooms::RoomSnapshot {
    submit(owner, d, "descriptor create/configure", || {
        LoginCommand::Rooms(draft(sc, old, honor))
    });
    let v = wait_owner(owner, d, "descriptor room configuration ACK", |v| {
        !v.room_pending
            && v.rooms.joined().is_some_and(|r| {
                r.summary.settings.mode == (if honor { 5 } else { 1 })
                    && r.summary.settings.map_id == (if honor { 532 } else { 514 })
            })
    });
    let room = v.rooms.joined().unwrap().clone();
    assert_eq!(room.summary.ready, 0);
    room
}

fn ready_and_loading(
    owner: &mut SessionController,
    guest: &mut ModernTlsConnection,
    sc: SelectionContext,
    mc: SelectionContext,
    honor: bool,
    previous_match: u64,
    d: &Deadline,
) -> LoginSnapshot {
    let room = owner_room(owner, sc, d);
    submit(owner, d, "descriptor Sun READY", || {
        LoginCommand::Rooms(room_command(sc, &room, rooms::Action::Ready, 1))
    });
    wait_owner(owner, d, "descriptor Sun READY ACK", |v| {
        !v.room_pending
            && v.rooms.joined().is_some_and(|r| {
                r.members
                    .iter()
                    .any(|m| m.matches_character(SUN) && m.ready)
            })
    });
    let room = guest
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    let ready = guest
        .rooms(room_command(mc, &room, rooms::Action::Ready, 1))
        .unwrap();
    assert_eq!(ready.status, rooms::Status::Ok);
    assert!(ready.joined.unwrap().summary.satisfied);
    // Real SessionController auto-PREPARE, not a handcrafted successful Frame.
    // A fast source worker can finish between polls. Log actual PREPARING
    // observation, but never claim this TLS case observed a phase it missed.
    let mut preparing_seen = false;
    let mut ping = Instant::now();
    loop {
        d.check("descriptor auto-PREPARE -> actual Loading");
        owner.poll();
        let view = owner.snapshot();
        assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
        assert!(view.combat_supported);
        if let Some(f) = view.world.frame().filter(|f| f.match_id != previous_match) {
            descriptor(f, honor);
            assert_ne!(f.phase, world::Phase::Aborted, "{:?}", f.abort_reason);
            preparing_seen |= f.phase == world::Phase::Preparing;
            if f.phase == world::Phase::Loading
                && (!honor || view.combat.as_ref().is_some_and(|c| c.world == *f))
            {
                assert_eq!(view.world_combat_enabled(), honor);
                if !honor {
                    assert!(
                        view.combat.is_none(),
                        "SceneOnly must not invent empty combat data"
                    );
                }
                eprintln!(
                    "descriptor match {} actual PREPARING observed: {}",
                    f.match_id, preparing_seen
                );
                return view;
            }
        }
        if ping.elapsed() >= Duration::from_secs(1) {
            guest.ping().unwrap();
            ping = Instant::now();
        }
        d.pause("descriptor preparing poll");
    }
}

fn scoped_image(
    runtime: &mut WorldRuntime,
    frame: &world::Frame,
    key: CharacterKey,
) -> Option<Arc<RenderedWorld>> {
    match runtime.poll() {
        RuntimeStatus::Ready(image)
            if (image.match_id, image.generation, image.revision)
                == (frame.match_id, frame.generation, frame.revision) =>
        {
            assert_eq!(image.selected_entity_id, actor(frame, key).entity_id);
            assert_eq!(image.manifest_digest, frame.manifest.digest);
            assert_eq!((image.width, image.height), (WIDTH, HEIGHT));
            assert_eq!(image.bgra.len(), WIDTH as usize * HEIGHT as usize * 4);
            assert!(
                image.ready_to_ack,
                "actual complete scoped image is the LOAD barrier"
            );
            Some(image)
        }
        RuntimeStatus::Failed(reason) => {
            panic!("descriptor actual shared resource/render worker failed: {reason:?}")
        }
        // Retained runtimes intentionally keep an old published image while
        // retiring/loading. An equal numeric revision is not a new-world ACK.
        _ => None,
    }
}

fn first_images(
    owner: &mut SessionController,
    guest: &mut ModernTlsConnection,
    sr: &mut WorldRuntime,
    mr: &mut WorldRuntime,
    loading: &LoginSnapshot,
    d: &Deadline,
) -> Arc<RenderedWorld> {
    let wf = Arc::new(loading.world.frame().unwrap().clone());
    let combat = loading.combat.clone();
    assert_eq!(combat.is_some(), wf.combat_enabled(true));
    sr.update_combat(wf.clone(), actor(&wf, SUN).entity_id, combat.clone())
        .unwrap();
    mr.update_combat(wf.clone(), actor(&wf, MOON).entity_id, combat.clone())
        .unwrap();
    let start = Instant::now();
    let mut ping = start;
    loop {
        d.check("descriptor complete two real shared first images");
        assert!(
            start.elapsed() < Duration::from_secs(28),
            "real first-image stage; Sun={:?} Moon={:?}",
            sr.poll(),
            mr.poll()
        );
        owner.poll();
        let view = owner.snapshot();
        assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
        assert!(
            view.world
                .frame()
                .is_some_and(|f| f.match_id == wf.match_id && f.phase == world::Phase::Loading),
            "first-render wait lost current Loading: {:?}",
            view.world.frame().map(|f| (f.phase, f.abort_reason))
        );
        if let (Some(s), Some(m)) = (scoped_image(sr, &wf, SUN), scoped_image(mr, &wf, MOON)) {
            for image in [&s, &m] {
                assert_eq!(image.combat.is_some(), combat.is_some());
                assert_eq!(image.objectives.len(), if combat.is_some() { 2 } else { 0 });
                if combat.is_none() {
                    assert!(
                        image.audio.is_none(),
                        "old Honor audio cannot cross into scene-only"
                    );
                }
                assert_eq!(
                    image.navigation_map().unwrap().canonical_digest(),
                    wf.manifest.digest
                );
            }
            return s;
        }
        if ping.elapsed() >= Duration::from_secs(1) {
            guest.ping().unwrap();
            ping = Instant::now();
        }
        d.pause("descriptor retained workers loading");
    }
}

fn load_and_active(
    owner: &mut SessionController,
    guest: &mut ModernTlsConnection,
    sc: SelectionContext,
    mc: SelectionContext,
    wf: &world::Frame,
    honor: bool,
    d: &Deadline,
) -> LoginSnapshot {
    let loaded = world::Request {
        manifest_digest: wf.manifest.digest,
        ..world_request(sc, wf, world::Action::Loaded)
    };
    submit(owner, d, "descriptor Sun actual-render LOAD", || {
        LoginCommand::World(loaded)
    });
    let one = wait_owner(owner, d, "descriptor one ACK retains barrier", |v| {
        !v.world_pending && v.world.frame().is_some_and(|f| actor(f, SUN).loaded)
    });
    let one = one.world.frame().unwrap();
    descriptor(one, honor);
    assert_eq!(one.phase, world::Phase::Loading);
    assert!(!actor(one, MOON).loaded);
    let countdown_at = Instant::now();
    let countdown = guest
        .world(world::Request {
            manifest_digest: wf.manifest.digest,
            ..world_request(mc, wf, world::Action::Loaded)
        })
        .unwrap()
        .frame
        .unwrap();
    descriptor(&countdown, honor);
    assert_eq!(countdown.phase, world::Phase::Countdown);
    let mut ping = Instant::now();
    loop {
        d.check("descriptor true global countdown");
        owner.poll();
        let v = owner.snapshot();
        assert_ne!(v.phase, LoginPhase::Failed, "{}", v.status);
        if let Some(f) = v.world.frame() {
            descriptor(f, honor);
            assert_ne!(f.phase, world::Phase::Aborted, "{:?}", f.abort_reason);
            if f.phase == world::Phase::Active
                && (!honor || v.combat.as_ref().is_some_and(|c| c.world == *f))
            {
                assert!(
                    countdown_at.elapsed() >= Duration::from_millis(2900),
                    "no shortened/faked countdown"
                );
                return v;
            }
        }
        if ping.elapsed() >= Duration::from_secs(1) {
            guest.ping().unwrap();
            ping = Instant::now();
        }
        d.pause("descriptor countdown poll");
    }
}

fn leave(
    owner: &mut SessionController,
    guest: &mut ModernTlsConnection,
    sc: SelectionContext,
    mc: SelectionContext,
    frame: &world::Frame,
    room_id: u64,
    d: &Deadline,
) {
    let request = world_request(sc, frame, world::Action::Leave);
    submit(owner, d, "descriptor Sun LEAVE", || {
        LoginCommand::World(request)
    });
    wait_owner(owner, d, "descriptor Sun released world", |v| {
        !v.world_pending && v.world.frame().is_none()
    });
    let held = guest
        .world(world_request(mc, frame, world::Action::Read))
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(held.descriptor, frame.descriptor);
    if frame.phase == world::Phase::Finished {
        assert_eq!(held.completion, frame.completion);
    }
    guest
        .world(world_request(mc, &held, world::Action::Leave))
        .unwrap();
    let r = owner_room(owner, sc, d);
    assert_eq!(r.summary.room_id, room_id);
    assert_eq!(r.summary.ready, 0);
    assert_eq!(r.members.len(), 2);
    assert!(!r.summary.satisfied);
    assert!(owner.snapshot().combat.is_none());
}

fn scene_move(
    owner: &mut SessionController,
    guest: &mut ModernTlsConnection,
    sc: SelectionContext,
    mc: SelectionContext,
    current: &world::Frame,
    image: &RenderedWorld,
    d: &Deadline,
) -> Arc<world::Frame> {
    let old = actor(current, SUN);
    let map = image.navigation_map().unwrap();
    let (heading, to) = DIRECTIONS
        .iter()
        .enumerate()
        .map(|(h, &(dx, dy))| {
            (
                h as u8,
                WorldCell {
                    x: old.position.x + dx,
                    y: old.position.y + dy,
                },
            )
        })
        .find(|(_, to)| {
            map.can_step(old.position, *to) && current.actors.iter().all(|a| a.position != *to)
        })
        .unwrap();
    assert!(owner
        .command(LoginCommand::CombatAttack {
            target_entity_id: actor(current, MOON).entity_id
        })
        .is_err());
    assert!(owner
        .command(LoginCommand::UseItem { item_id: RED })
        .is_err());
    assert!(matches!(
        guest.combat(combat::Request::read(mc, CHANNEL, current)),
        Err(TransportError::InvalidState(_))
    ));
    guest.ping().unwrap();
    let request = world::Request {
        heading,
        sequence: old.sequence + 1,
        from_x: old.position.x,
        from_y: old.position.y,
        ..world_request(sc, current, world::Action::Move)
    };
    submit(owner, d, "descriptor scene authoritative MOVE", || {
        LoginCommand::World(request)
    });
    let after = wait_owner(owner, d, "descriptor scene MOVE ACK", |v| {
        !v.world_pending
            && v.world
                .frame()
                .is_some_and(|f| actor(f, SUN).sequence == old.sequence + 1)
    });
    assert!(after.combat_supported && after.combat.is_none());
    let frame = Arc::new(after.world.frame().unwrap().clone());
    descriptor(&frame, false);
    assert_eq!(actor(&frame, SUN).position, to);
    assert!(frame.completion.is_none());
    let other = guest
        .world(world_request(mc, &frame, world::Action::Read))
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(other, *frame);
    frame
}

#[test]
#[ignore = "v15 descriptor-honor Java peer; same TLS/listener/controller, real shared514/532 workers and isolated SQL; no GUI"]
fn java_scene_honor_scene_same_tls_real_load_stock_and_single_saved_result() {
    let d = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernMatchDescriptorTestPeer",
        "descriptor-honor",
    );
    assert!(
        peer.process_job.is_some(),
        "fresh SQL must stay inside the owned GO-gated Job"
    );
    let (mut owner, sc) = owner_connect(&peer, &shared, &ca, &d);
    let (mut guest, mc) = guest_connect(&peer, &ca);
    assert_eq!(SUN.object_id, MOON.object_id);
    assert_ne!(SUN.source_id, MOON.source_id);
    let empty = owner_shop(&mut owner, sc, &d, shop::Action::Read, 0, 0);
    assert_eq!((empty.round_id, empty.balance), (1, 20_000));
    assert!(empty.stacks.is_empty());
    owner_shop(&mut owner, sc, &d, shop::Action::Buy, RED, 2);
    let purchased = owner_shop(&mut owner, sc, &d, shop::Action::Buy, ARROW, 80);
    assert_eq!(purchased.balance, 18_320);
    assert_eq!(
        purchased
            .stacks
            .iter()
            .map(|s| (s.item_id, s.quantity, s.unit_paid_price))
            .collect::<Vec<_>>(),
        vec![(RED, 2, 40), (ARROW, 80, 20)]
    );
    let moon_empty = guest_shop(&mut guest, mc);
    let moon_buy_request =
        guest_shop_request(&mut guest, mc).mutation(&moon_empty, shop::Action::Buy, RED, 1);
    let moon_buy = guest.shop(moon_buy_request).unwrap();
    assert_eq!(moon_buy.echo, moon_buy_request);
    assert_eq!(
        (moon_buy.status, moon_buy.reason),
        (shop::Status::Ok, shop::Reason::None)
    );
    let moon_purchased = moon_buy.snapshot.unwrap();
    assert_eq!(moon_purchased.balance, 19_920);

    let created = configure(&mut owner, sc, &d, None, false);
    let room_id = created.summary.room_id;
    let directory = guest
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap();
    let summary = directory
        .rooms
        .iter()
        .find(|r| r.room_id == room_id)
        .unwrap();
    guest
        .rooms(
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
    let mut ids = BTreeSet::new();
    let mut scene_digest = None;
    let mut previous_match = 0;
    let mut previous_terminal: Option<Arc<combat::Frame>> = None;
    for round in 0..3 {
        let honor_mode = round == 1;
        if round > 0 {
            let r = owner_room(&mut owner, sc, &d);
            configure(&mut owner, sc, &d, Some(&r), honor_mode);
        }
        let loading = ready_and_loading(
            &mut owner,
            &mut guest,
            sc,
            mc,
            honor_mode,
            previous_match,
            &d,
        );
        let wf = Arc::new(loading.world.frame().unwrap().clone());
        assert!(ids.insert((wf.match_id, wf.generation)));
        previous_match = wf.match_id;
        assert_eq!(wf.room.summary.room_id, room_id);
        assert!(wf.actors.iter().all(|a| !a.loaded && a.sequence == 0));
        let remote = guest
            .world(world_request(mc, &wf, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
        assert_eq!(remote, *wf);
        if honor_mode {
            let c = loading.combat.as_ref().unwrap();
            assert_eq!(
                guest
                    .combat(combat::Request::read(mc, CHANNEL, &wf))
                    .unwrap()
                    .frame
                    .unwrap(),
                **c
            );
            for n in [
                c.vitals.len(),
                c.world.casts.len(),
                c.damages.len(),
                c.items.len(),
                c.weights.len(),
                c.timed.len(),
                c.ammo.len(),
                c.regen.len(),
            ] {
                assert_eq!(n, 2, "complete descriptor-enabled roster before LOADED");
            }
            assert!(c.fires.is_empty());
            let h = c.honor.as_ref().unwrap();
            assert_eq!(h.objectives.len(), 2);
            assert!(h.objectives.iter().all(|o| o.hp == 3000 && o.action == 0));
            assert_eq!(c.item(1).unwrap().stock(RED).unwrap().remaining, 2);
            assert_eq!(c.item(2).unwrap().stock(RED).unwrap().remaining, 1);
            assert_eq!(
                c.ammo(1)
                    .unwrap()
                    .stocks
                    .iter()
                    .map(|s| s.remaining)
                    .sum::<i64>(),
                80
            );
            assert!(c
                .items
                .iter()
                .all(|s| s.use_count == 0 && s.use_revision == 0));
            assert!(c
                .ammo
                .iter()
                .all(|s| s.shot_count == 0 && s.shot_revision == 0));
            assert!(c
                .vitals
                .iter()
                .all(|v| v.hp == v.max_hp && v.mp == v.max_mp && v.attack_sequence == 0));
        } else {
            assert!(loading.combat.is_none());
            if let Some(digest) = scene_digest {
                assert_eq!(wf.manifest.digest, digest);
            } else {
                scene_digest = Some(wf.manifest.digest);
            }
            if let Some(old) = &previous_terminal {
                assert!(matches!(
                    guest.combat(combat::Request {
                        action: combat::Action::Attack,
                        sequence: old.vital(2).unwrap().attack_sequence + 1,
                        target_entity_id: 1,
                        ..combat::Request::read(mc, CHANNEL, &old.world)
                    }),
                    Err(TransportError::InvalidState(_))
                ));
                guest.ping().unwrap();
            }
        }
        // Retain the same two WorldRuntime objects, including old published
        // images, across all three scopes; no new wrapper hides stale workers.
        let image = first_images(&mut owner, &mut guest, &mut sr, &mut mr, &loading, &d);
        assert_eq!(
            image.navigation_map().unwrap().map_id(),
            if honor_mode { 532 } else { 514 }
        );
        if honor_mode {
            assert_ne!(Some(image.manifest_digest), scene_digest);
        }
        let active = load_and_active(&mut owner, &mut guest, sc, mc, &wf, honor_mode, &d);
        if !honor_mode {
            let moved = scene_move(
                &mut owner,
                &mut guest,
                sc,
                mc,
                active.world.frame().unwrap(),
                &image,
                &d,
            );
            leave(&mut owner, &mut guest, sc, mc, &moved, room_id, &d);
            let wallet = owner_shop(&mut owner, sc, &d, shop::Action::Read, 0, 0);
            let moon = guest_shop(&mut guest, mc);
            if round == 0 {
                assert_eq!(wallet, purchased, "no ITEMS capture/reset for SceneOnly");
                assert_eq!(moon, moon_purchased);
                assert_eq!(
                    record_owner(&mut owner, sc, &d),
                    record::Totals {
                        stored: false,
                        wins: 0,
                        losses: 0,
                        revision: 0
                    }
                );
                assert_eq!(
                    guest.record().unwrap().totals,
                    Some(record::Totals {
                        stored: false,
                        wins: 0,
                        losses: 0,
                        revision: 0
                    })
                );
            } else {
                assert_eq!(
                    (wallet.round_id, wallet.revision, wallet.balance),
                    (2, purchased.revision + 1, 20_000)
                );
                assert!(wallet.stacks.is_empty());
                assert_eq!(
                    (moon.round_id, moon.revision, moon.balance),
                    (2, moon_purchased.revision + 1, 20_000)
                );
                assert!(moon.stacks.is_empty());
                assert_eq!(
                    record_owner(&mut owner, sc, &d),
                    record::Totals {
                        stored: true,
                        wins: 1,
                        losses: 0,
                        revision: 1
                    }
                );
                assert_eq!(
                    guest.record().unwrap().totals,
                    Some(record::Totals {
                        stored: true,
                        wins: 0,
                        losses: 1,
                        revision: 1
                    })
                );
            }
            assert_eq!(
                owner_shop(&mut owner, sc, &d, shop::Action::Read, 0, 0),
                wallet
            );
            assert_eq!(guest_shop(&mut guest, mc), moon);
        } else {
            let before = active.combat.unwrap();
            assert_eq!(before.vital(1).unwrap().weapon_type, 4);
            submit(&mut owner, &d, "descriptor one full-HP USE", || {
                LoginCommand::UseItem { item_id: RED }
            });
            assert!(owner.snapshot().item_pending);
            assert_eq!(owner.snapshot().combat.as_ref(), Some(&before));
            let used = wait_owner(&mut owner, &d, "descriptor atomic USE", |v| {
                !v.item_pending
                    && v.combat
                        .as_ref()
                        .is_some_and(|c| c.item(1).is_some_and(|i| i.use_count == 1))
            })
            .combat
            .unwrap();
            assert_eq!(used.item(1).unwrap().stock(RED).unwrap().remaining, 1);
            assert_eq!(used.item(1).unwrap().restored, 0);
            assert_eq!(used.item(1).unwrap().effect, 189);
            assert_eq!(used.vital(1).unwrap().hp, used.vital(1).unwrap().max_hp);
            assert_eq!(used.item(2), before.item(2));
            pause(&mut owner, &mut guest, &d, Duration::from_millis(1050));
            submit(&mut owner, &d, "descriptor real basic heal", || {
                LoginCommand::CastBasicHeal {
                    target_entity_id: 3,
                }
            });
            let healed = wait_owner(&mut owner, &d, "descriptor full-HP tower heal ACK", |v| {
                !v.spell_pending
                    && v.combat
                        .as_ref()
                        .is_some_and(|c| c.honor.as_ref().is_some_and(|h| h.heals[0].count == 1))
            })
            .combat
            .unwrap();
            assert_eq!(healed.honor.as_ref().unwrap().heals[0].restored, 0);
            let mut current = healed;
            let mut shots = 0;
            while current.world.phase == world::Phase::Active {
                pause(&mut owner, &mut guest, &d, Duration::from_millis(1050));
                let old_sequence = current.vital(1).unwrap().attack_sequence;
                submit(
                    &mut owner,
                    &d,
                    "descriptor authoritative tower attack",
                    || LoginCommand::CombatAttack {
                        target_entity_id: 4,
                    },
                );
                current = wait_owner(&mut owner, &d, "descriptor attack ACK", |v| {
                    !v.combat_pending
                        && v.combat.as_ref().is_some_and(|c| {
                            c.vital(1)
                                .is_some_and(|a| a.attack_sequence == old_sequence + 1)
                        })
                })
                .combat
                .unwrap();
                descriptor(&current.world, true);
                shots += 1;
                assert!(shots <= 60, "bounded source-qualified +100 ranged fixture");
                assert_eq!(current.ammo(1).unwrap().shot_count, shots);
                assert_eq!(
                    current
                        .honor
                        .as_ref()
                        .unwrap()
                        .objective(4)
                        .unwrap()
                        .last_kind,
                    honor::Interaction::Physical
                );
                guest.ping().unwrap();
            }
            assert_eq!(current.world.phase, world::Phase::Finished);
            assert_eq!(current.honor.as_ref().unwrap().objective(4).unwrap().hp, 0);
            assert!(current.honor.as_ref().unwrap().objective(3).unwrap().hp > 0);
            let frozen_items = current.items.clone();
            let frozen_ammo = current.ammo.clone();
            let frozen_honor = current.honor.clone();
            current = wait_owner(
                &mut owner,
                &d,
                "descriptor single durable SQL settlement",
                |v| {
                    v.combat.as_ref().is_some_and(|c| {
                        c.world
                            .completion
                            .as_ref()
                            .is_some_and(|r| r.state == world::SettlementState::Saved)
                    })
                },
            )
            .combat
            .unwrap();
            descriptor(&current.world, true);
            assert_eq!(current.items, frozen_items);
            assert_eq!(current.ammo, frozen_ammo);
            assert_eq!(current.honor, frozen_honor);
            let result = current.world.completion.as_ref().unwrap();
            assert_eq!((result.event.mode, result.event.map_id), (5, 532));
            assert_eq!(result.event.participants.len(), 2);
            assert_eq!(result.event.participants[0].verdict, world::Verdict::Win);
            assert_eq!(result.event.participants[1].verdict, world::Verdict::Loss);
            guest
                .world(world_request(mc, &current.world, world::Action::Read))
                .unwrap();
            let remote = guest
                .combat(combat::Request::read(mc, CHANNEL, &current.world))
                .unwrap()
                .frame
                .unwrap();
            assert_eq!(remote, *current);
            let saved_image = combat_images(&mut sr, &mut mr, &current, &d);
            assert_eq!(saved_image.objectives.len(), 2);
            previous_terminal = Some(current.clone());
            leave(&mut owner, &mut guest, sc, mc, &current.world, room_id, &d);
            let reset = owner_shop(&mut owner, sc, &d, shop::Action::Read, 0, 0);
            assert_eq!(
                (reset.round_id, reset.revision, reset.balance),
                (2, purchased.revision + 1, 20_000)
            );
            assert!(reset.stacks.is_empty());
            assert_eq!(
                owner_shop(&mut owner, sc, &d, shop::Action::Read, 0, 0),
                reset
            );
            let moon_reset = guest_shop(&mut guest, mc);
            assert_eq!(
                (moon_reset.round_id, moon_reset.revision, moon_reset.balance),
                (2, moon_purchased.revision + 1, 20_000)
            );
            assert!(moon_reset.stacks.is_empty());
            assert_eq!(guest_shop(&mut guest, mc), moon_reset);
            assert_eq!(
                record_owner(&mut owner, sc, &d),
                record::Totals {
                    stored: true,
                    wins: 1,
                    losses: 0,
                    revision: 1
                }
            );
            assert_eq!(
                guest.record().unwrap().totals,
                Some(record::Totals {
                    stored: true,
                    wins: 0,
                    losses: 1,
                    revision: 1
                })
            );
        }
        assert_eq!(
            (
                guest.ack().unwrap().version,
                guest.ack().unwrap().capabilities
            ),
            (15, CAPS)
        );
        assert!(owner.snapshot().combat_supported);
    }
    assert_eq!(ids.len(), 3);
    assert!(previous_terminal.is_some());
    sr.shutdown();
    mr.shutdown();
    while !sr.is_quiescent() || !mr.is_quiescent() {
        sr.poll();
        mr.poll();
        d.pause("descriptor owned render retirement");
    }
    guest.close().unwrap();
    submit(&mut owner, &d, "descriptor normal TLS shutdown", || {
        LoginCommand::Shutdown
    });
    while !owner.is_quiescent() {
        owner.poll();
        d.pause("descriptor session retirement");
    }
    assert_eq!(owner.snapshot().phase, LoginPhase::Disconnected);
    peer.expect_successful_exit(&d);
}

fn legacy_connect(
    peer: &JavaPeer,
    ca: &[u8],
    key: CharacterKey,
) -> (ModernTlsConnection, SelectionContext) {
    let mut c = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(ca),
        Hello {
            max_version: 14,
            supported_capabilities: 32767,
            ..Hello::current(200)
        },
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
        (14, 32767)
    );
    c.authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    assert_eq!(
        c.characters(key.source_id, 1).unwrap().characters[0].key,
        key
    );
    let context = SelectionContext { key, epoch: 1 };
    c.select_character(context).unwrap();
    assert_eq!(
        c.record().unwrap().totals,
        Some(record::Totals {
            stored: false,
            wins: 0,
            losses: 0,
            revision: 0
        })
    );
    c.enter_channel(CHANNEL).unwrap();
    (c, context)
}

fn raw_ready(
    sun: &mut ModernTlsConnection,
    moon: &mut ModernTlsConnection,
    sc: SelectionContext,
    mc: SelectionContext,
) -> rooms::RoomSnapshot {
    let room = sun
        .rooms(rooms::Request::read(sc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    assert_eq!(
        sun.rooms(room_command(sc, &room, rooms::Action::Ready, 1))
            .unwrap()
            .status,
        rooms::Status::Ok
    );
    let room = moon
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    let reply = moon
        .rooms(room_command(mc, &room, rooms::Action::Ready, 1))
        .unwrap();
    assert_eq!(reply.status, rooms::Status::Ok);
    let ready = reply.joined.unwrap();
    assert!(ready.summary.satisfied);
    ready
}

fn prepare_request(sc: SelectionContext, room: &rooms::RoomSnapshot) -> world::Request {
    world::Request {
        action: world::Action::Prepare,
        room_id: room.summary.room_id,
        expected_room_revision: room.summary.revision,
        ..world::Request::read(sc, CHANNEL)
    }
}

#[test]
#[ignore = "mixed v15 listener with real v14 TLS owners; typed Scene refusal, compatible Honor Loading, isolated SQL remains empty; no GUI/resources"]
fn java_v14_rejects_scene_without_losing_tls_stock_or_compatible_honor() {
    let d = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernMatchDescriptorTestPeer",
        "descriptor-legacy",
    );
    assert!(peer.process_job.is_some());
    let (mut sun, sc) = legacy_connect(&peer, &ca, SUN);
    let (mut moon, mc) = legacy_connect(&peer, &ca, MOON);
    let empty = guest_shop(&mut sun, sc);
    let buy_request = guest_shop_request(&mut sun, sc).mutation(&empty, shop::Action::Buy, RED, 2);
    let bought = sun.shop(buy_request).unwrap();
    assert_eq!(bought.echo, buy_request);
    assert_eq!(
        (bought.status, bought.reason),
        (shop::Status::Ok, shop::Reason::None)
    );
    let bought = bought.snapshot.unwrap();
    assert_eq!((bought.round_id, bought.balance), (1, 19_920));
    assert_eq!(
        bought
            .stacks
            .iter()
            .map(|s| (s.item_id, s.quantity))
            .collect::<Vec<_>>(),
        [(RED, 2)]
    );
    let created = sun.rooms(draft(sc, None, false)).unwrap();
    assert_eq!(created.status, rooms::Status::Ok);
    let room = created.joined.unwrap();
    assert_eq!(
        moon.rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: room.summary.room_id,
                    expected_revision: room.summary.revision,
                    ..rooms::Common::read(mc, CHANNEL)
                },
                String::new()
            )
            .unwrap()
        )
        .unwrap()
        .status,
        rooms::Status::Ok
    );
    let ready = raw_ready(&mut sun, &mut moon, sc, mc);
    let request = prepare_request(sc, &ready);
    let refusal = sun.world(request).unwrap();
    assert_eq!(refusal.echo, request);
    assert_eq!(
        (refusal.status, refusal.reason),
        (world::Status::Unavailable, world::Reason::InvalidState)
    );
    assert!(
        refusal.frame.is_none(),
        "no fake Aborted/combat frame for an incompatible legacy mode"
    );
    sun.ping().unwrap();
    moon.ping().unwrap();
    let still_room = sun
        .rooms(rooms::Request::read(sc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    // Preparation captures one room revision; incompatible legacy admission
    // releases that ticket with one more revision and cancels every READY.
    // No membership, identity, setting or other source metadata may change.
    let mut released = ready.clone();
    released.summary.revision = ready.summary.revision.checked_add(2).unwrap();
    released.summary.ready = 0;
    released.summary.satisfied = false;
    for member in &mut released.members {
        member.ready = false;
    }
    assert_eq!(still_room, released, "exact released preparation snapshot");
    assert_eq!(
        guest_shop(&mut sun, sc),
        bought,
        "Scene refusal did not capture/reset supplies"
    );
    let absent = record::Totals {
        stored: false,
        wins: 0,
        losses: 0,
        revision: 0,
    };
    assert_eq!(sun.record().unwrap().totals, Some(absent));
    assert_eq!(moon.record().unwrap().totals, Some(absent));

    // Same two authenticated sockets, now a fully compatible legacy Honor
    // registration. No renderer is used, so neither owner sends LOADED.
    let configured = sun.rooms(draft(sc, Some(&still_room), true)).unwrap();
    assert_eq!(configured.status, rooms::Status::Ok);
    assert_eq!(configured.joined.unwrap().summary.ready, 0);
    let ready = raw_ready(&mut sun, &mut moon, sc, mc);
    let admitted = sun.world(prepare_request(sc, &ready)).unwrap();
    assert_eq!(
        (admitted.status, admitted.reason),
        (world::Status::Ok, world::Reason::None)
    );
    let mut frame = admitted.frame.unwrap();
    let prepare_start = Instant::now();
    while frame.phase == world::Phase::Preparing {
        d.pause("legacy compatible Honor source preparation");
        assert!(
            prepare_start.elapsed() < Duration::from_secs(3),
            "actual configured preparation deadline"
        );
        moon.ping().unwrap();
        frame = sun
            .world(world_request(sc, &frame, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
    }
    assert_eq!(frame.phase, world::Phase::Loading);
    assert_eq!(
        frame.descriptor, None,
        "v14 old World bytes carry no v15 appendix"
    );
    assert!(frame.combat_enabled(true));
    assert_eq!(
        (frame.room.summary.settings.mode, frame.manifest.map_id),
        (5, 532)
    );
    assert!(frame.actors.iter().all(|a| !a.loaded));
    moon.world(world_request(mc, &frame, world::Action::Read))
        .unwrap();
    let complete = sun
        .combat(combat::Request::read(sc, CHANNEL, &frame))
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(complete.world, frame);
    assert_eq!(complete.vitals.len(), 2);
    assert_eq!(complete.honor.as_ref().unwrap().objectives.len(), 2);
    assert_eq!(complete.item(1).unwrap().stock(RED).unwrap().remaining, 2);
    assert!(complete.world.completion.is_none());
    sun.world(world_request(sc, &frame, world::Action::Leave))
        .unwrap();
    let held = moon
        .world(world_request(mc, &frame, world::Action::Read))
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(held.phase, world::Phase::Aborted);
    assert!(held.completion.is_none());
    moon.world(world_request(mc, &held, world::Action::Leave))
        .unwrap();
    let reset = guest_shop(&mut sun, sc);
    assert_eq!(
        (reset.round_id, reset.revision, reset.balance),
        (2, bought.revision + 1, 20_000)
    );
    assert!(
        reset.stacks.is_empty(),
        "Honor ITEMS abort resets once; this is not Scene preservation"
    );
    assert_eq!(guest_shop(&mut sun, sc), reset);
    assert_eq!(sun.record().unwrap().totals, Some(absent));
    assert_eq!(moon.record().unwrap().totals, Some(absent));
    sun.ping().unwrap();
    moon.ping().unwrap();
    assert_eq!(
        (sun.ack().unwrap().version, moon.ack().unwrap().version),
        (14, 14)
    );
    sun.close().unwrap();
    moon.close().unwrap();
    // The owned Java fixture independently asserts zero SQL matches/members/
    // totals, no unresolved outbox record, and normal child/connection teardown.
    peer.expect_successful_exit(&d);
}
