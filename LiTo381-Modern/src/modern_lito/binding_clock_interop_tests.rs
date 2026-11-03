//! Parsed per-binding clocks, not client delays or synthetic world frames.
//! Two authenticated owners and retained render workers load real shared maps;
//! both SceneOnly rounds end only by the server's safety timeout, without SQL
//! results. This listener has no shop: wallet coverage remains in the separate
//! complete match-descriptor rotation tests, not invented here.
use super::super::{
    equipment, protocol, shop_protocol as shop,
    transport::{ModernTlsConnection, TransportError},
};
use super::*;

const CAPS: u32 = 65_903;
const IO_TIMEOUT: Duration = Duration::from_secs(2);
// Existing SessionController world polling is 200 ms; the test pumps at POLL.
// This is an observation bound, never a server deadline extension.
const OBSERVATION: Duration = Duration::from_millis(2210);

#[derive(Clone, Copy)]
struct Binding {
    map: u32,
    load_ms: u32,
    countdown_ns: u64,
    active_ms: u32,
    bytes: u32,
    raw: &'static str,
    canonical: &'static str,
    anchor: WorldCell,
}
const BINDINGS: [Binding; 2] = [
    Binding {
        map: 514,
        load_ms: 30_000,
        countdown_ns: 1_000_000_000,
        active_ms: 4000,
        bytes: 137784,
        raw: "ccfe52b614dfbdf97fd713f80d61e4cdd35e7c8c3000a06b4bba8868c96d1271",
        canonical: "2feecc83de156a03c62e44effb7106067b24098a9ef7250cc471f7bf826da7a8",
        anchor: WorldCell { x: 32724, y: 32784 },
    },
    Binding {
        map: 509,
        load_ms: 25_000,
        countdown_ns: 2_000_000_000,
        active_ms: 6000,
        bytes: 106355,
        raw: "8580337a4ad3593d697a416062c4da063f8fd6c2ba311f0b694fcfc59710cb9d",
        canonical: "1178261c2d6a968251f2eadfb2029bd0febade95b7151c15872f355784df87f8",
        anchor: WorldCell { x: 32740, y: 32796 },
    },
];
fn digest(s: &str) -> [u8; 32] {
    assert_eq!(s.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap())
}
fn policy(frame: &world::Frame, binding: Binding) {
    assert_eq!(
        frame.descriptor,
        Some(world::MatchDescriptor {
            ruleset: world::MatchRuleset::SceneOnly,
            features: 0,
            preparation_timeout_millis: 3000,
            load_timeout_millis: binding.load_ms,
            countdown_nanos: binding.countdown_ns,
            active_timeout_millis: binding.active_ms,
        })
    );
    assert!(!frame.combat_enabled(true));
    assert!(frame.completion.is_none());
    assert!(frame.casts.is_empty());
    assert_eq!(
        (
            frame.room.summary.settings.mode,
            frame.room.summary.settings.map_id
        ),
        (1, binding.map)
    );
    assert_eq!(frame.manifest.map_id, binding.map);
    assert_eq!(frame.manifest.digest, digest(binding.canonical));
    assert_eq!(frame.manifest.sectors.len(), 1);
    let sector = &frame.manifest.sectors[0];
    assert_eq!(
        (sector.x, sector.y, sector.byte_length),
        (0x7fff, 0x8000, binding.bytes)
    );
    assert_eq!(sector.sha256, digest(binding.raw));
}
fn pump(owner: &mut SessionController) -> LoginSnapshot {
    owner.poll();
    let v = owner.snapshot();
    assert_ne!(v.phase, LoginPhase::Failed, "{}", v.status);
    assert!(!v.combat_supported);
    assert!(v.combat.is_none());
    v
}
fn connect_owner(
    peer: &JavaPeer,
    shared: &PathBuf,
    ca: &[u8],
    d: &Deadline,
) -> (SessionController, SelectionContext) {
    let mut owner = SessionController::new(profile(peer.endpoint, shared, ca));
    submit(&mut owner, d, "clock verified TLS", || {
        LoginCommand::Connect
    });
    wait_owner(&mut owner, d, "clock TLS ready", |v| {
        v.phase == LoginPhase::LoginReady
    });
    submit(&mut owner, d, "clock authentication", || {
        LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        }
    });
    wait_owner(&mut owner, d, "clock source selection", |v| {
        v.phase == LoginPhase::ChoosingSource
    });
    submit(&mut owner, d, "clock source7", || {
        LoginCommand::SelectSource(SUN.source_id)
    });
    let roster = wait_owner(&mut owner, d, "clock source7 roster", |v| {
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
    submit(&mut owner, d, "clock source7 selected", || {
        LoginCommand::SelectCharacter(context)
    });
    wait_owner(
        &mut owner,
        d,
        "clock source7 channel and real absent record",
        |v| {
            v.phase == LoginPhase::ChoosingChannel
                && !v.record_pending
                && v.record.totals().is_some()
        },
    );
    assert_eq!(owner.snapshot().record.totals(), Some(absent()));
    submit(&mut owner, d, "clock enter channel", || {
        LoginCommand::EnterChannel {
            context,
            channel_id: CHANNEL,
        }
    });
    wait_owner(&mut owner, d, "clock lobby ACK", |v| {
        v.phase == LoginPhase::InLobby
    });
    (owner, context)
}
fn connect_guest(peer: &JavaPeer, ca: &[u8]) -> (ModernTlsConnection, SelectionContext) {
    let mut guest = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(ca),
        Hello::current(100),
        TransportLimits {
            connect_timeout: Duration::from_millis(500),
            total_timeout: IO_TIMEOUT,
            io_poll_interval: Duration::from_millis(25),
        },
        Cancellation::new(),
    )
    .unwrap();
    assert_eq!(
        (
            guest.ack().unwrap().version,
            guest.ack().unwrap().capabilities
        ),
        (15, CAPS)
    );
    assert_eq!(
        CAPS & (protocol::CAP_EQUIPMENT | protocol::CAP_COMBAT | protocol::CAP_SHOP),
        0
    );
    guest
        .authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    assert_eq!(
        guest.characters(MOON.source_id, 1).unwrap().characters[0].key,
        MOON
    );
    let context = SelectionContext {
        key: MOON,
        epoch: 1,
    };
    guest.select_character(context).unwrap();
    assert_eq!(guest.record().unwrap().totals, Some(absent()));
    guest.enter_channel(CHANNEL).unwrap();
    (guest, context)
}
fn absent() -> record::Totals {
    record::Totals {
        stored: false,
        wins: 0,
        losses: 0,
        revision: 0,
    }
}
fn records(
    owner: &mut SessionController,
    guest: &mut ModernTlsConnection,
    sc: SelectionContext,
    d: &Deadline,
) {
    submit(owner, d, "clock explicit source-bound record read", || {
        LoginCommand::ReadRecord
    });
    let v = wait_owner(owner, d, "clock record ACK", |v| {
        !v.record_pending && v.record.context() == Some(sc) && v.record.totals().is_some()
    });
    assert_eq!(v.record.totals(), Some(absent()));
    assert_eq!(guest.record().unwrap().totals, Some(absent()));
}
fn unsupported(
    owner: &mut SessionController,
    guest: &mut ModernTlsConnection,
    sc: SelectionContext,
    mc: SelectionContext,
    frame: Option<&world::Frame>,
) {
    let before = owner.snapshot();
    for command in [
        LoginCommand::ReadSkills,
        LoginCommand::Equipment(equipment::Request::read(sc)),
        LoginCommand::Shop(shop::Request::read(sc, CHANNEL, before.rooms.joined())),
        LoginCommand::UseItem { item_id: 40010 },
        LoginCommand::CombatAttack {
            target_entity_id: 2,
        },
        LoginCommand::CastTeleport,
    ] {
        assert!(
            owner.command(command).is_err(),
            "unnegotiated operation must not be queued"
        );
    }
    let after = owner.snapshot();
    assert!(
        !after.spell_pending
            && !after.equipment_pending
            && !after.shop_pending
            && !after.item_pending
            && !after.combat_pending
            && !after.ammo_pending
    );
    assert_eq!(after.equipment, before.equipment);
    assert_eq!(after.skills, before.skills);
    assert_eq!(after.shop, before.shop);
    assert_eq!(after.world, before.world);
    assert_eq!(after.combat, before.combat);
    assert!(matches!(
        guest.skills(),
        Err(TransportError::InvalidState(_))
    ));
    assert!(matches!(
        guest.equipment(equipment::Request::read(mc)),
        Err(TransportError::InvalidState(_))
    ));
    // Even unsupported SHOP carries the actual room scope, never the old
    // unscoped None workaround which would confound capability with RoomChanged.
    let room = guest
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap();
    assert_eq!(
        (room.status, room.reason),
        (rooms::Status::Ok, rooms::Reason::None)
    );
    assert!(matches!(
        guest.shop(shop::Request::read(mc, CHANNEL, room.joined.as_ref())),
        Err(TransportError::InvalidState(_))
    ));
    if let Some(frame) = frame {
        assert!(matches!(
            guest.combat(combat::Request::read(mc, CHANNEL, frame)),
            Err(TransportError::InvalidState(_))
        ));
    }
    guest.ping().unwrap();
}
fn room(owner: &mut SessionController, sc: SelectionContext, d: &Deadline) -> rooms::RoomSnapshot {
    submit(owner, d, "clock room READ", || {
        LoginCommand::Rooms(rooms::Request::read(sc, CHANNEL).unwrap())
    });
    wait_owner(owner, d, "clock room READ ACK", |v| {
        !v.room_pending && v.rooms.joined().is_some()
    })
    .rooms
    .joined()
    .unwrap()
    .clone()
}
fn draft(sc: SelectionContext, b: Binding, old: Option<&rooms::RoomSnapshot>) -> rooms::Request {
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
                mode: 1,
                attributes: 16,
                map_id: b.map,
                name: "Parsed binding clock rotation".into(),
            },
            String::new(),
        )
        .unwrap(),
    )
    .unwrap()
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
            assert!(image.ready_to_ack);
            assert!(image.combat.is_none() && image.audio.is_none() && image.objectives.is_empty());
            assert_eq!(
                image.navigation_map().unwrap().canonical_digest(),
                frame.manifest.digest
            );
            Some(image)
        }
        RuntimeStatus::Failed(reason) => panic!("clock real shared renderer failed: {reason:?}"),
        _ => None,
    }
}

#[test]
#[ignore = "v15 parsed clock-scenes Java peer; actual shared514/509 and isolated SQL, same SessionController/TLS/runtimes; no GUI"]
fn java_binding_clocks_real_scene_load_countdown_move_and_safety_abort_no_results() {
    run_binding_scenes(
        "local.lito381.server.ModernWorldClockTestPeer",
        "clock-scenes",
        BINDINGS,
    );
}

#[test]
#[ignore = "published SELECT-only isolated MySQL policy through ModernMain into two real shared worlds, same TLS/controller/renderers; no GUI"]
fn java_sql_world_policy_changes_real_births_and_clocks_through_two_shared_worlds() {
    // Deliberately differ from both retained local configuration anchors. A
    // successful handshake, or an ignored SQL snapshot, cannot satisfy the
    // exact authoritative LOADING actor assertions in the common runner.
    let bindings = [
        Binding {
            anchor: WorldCell { x: 32725, y: 32784 },
            ..BINDINGS[0]
        },
        Binding {
            anchor: WorldCell { x: 32740, y: 32795 },
            ..BINDINGS[1]
        },
    ];
    for (sql, local) in bindings.iter().zip(BINDINGS) {
        assert_ne!(sql.anchor, local.anchor);
    }
    run_binding_scenes(
        "local.lito381.server.ModernWorldPolicyTestPeer",
        "sql-world-policy",
        bindings,
    );
}

fn run_binding_scenes(class: &'static str, mode: &'static str, bindings: [Binding; 2]) {
    let d = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(&fixtures, &d, class, mode);
    assert!(
        peer.process_job.is_some(),
        "all isolated SQL descendants are GO-gated and Job-owned"
    );
    let (mut owner, sc) = connect_owner(&peer, &shared, &ca, &d);
    let (mut guest, mc) = connect_guest(&peer, &ca);
    unsupported(&mut owner, &mut guest, sc, mc, None);
    records(&mut owner, &mut guest, sc, &d);
    let mut sr = WorldRuntime::new(shared.clone()).unwrap();
    let mut mr = WorldRuntime::new(shared).unwrap();
    let mut room_id = None;
    let mut scopes = BTreeSet::new();
    let mut previous_match = 0;
    for (round, b) in bindings.into_iter().enumerate() {
        let old = room_id.map(|_| room(&mut owner, sc, &d));
        submit(&mut owner, &d, "clock room configure", || {
            LoginCommand::Rooms(draft(sc, b, old.as_ref()))
        });
        let configured = wait_owner(&mut owner, &d, "clock configured ACK", |v| {
            !v.room_pending
                && v.rooms
                    .joined()
                    .is_some_and(|r| r.summary.settings.map_id == b.map)
        });
        let configured = configured.rooms.joined().unwrap();
        assert_eq!(configured.summary.ready, 0);
        if let Some(id) = room_id {
            assert_eq!(configured.summary.room_id, id);
        } else {
            room_id = Some(configured.summary.room_id);
            let joined = guest
                .rooms(
                    rooms::Request::join(
                        rooms::Common {
                            action: rooms::Action::Join,
                            room_id: configured.summary.room_id,
                            expected_revision: configured.summary.revision,
                            ..rooms::Common::read(mc, CHANNEL)
                        },
                        String::new(),
                    )
                    .unwrap(),
                )
                .unwrap();
            assert_eq!(
                (joined.status, joined.reason),
                (rooms::Status::Ok, rooms::Reason::None)
            );
        }
        let r = room(&mut owner, sc, &d);
        submit(&mut owner, &d, "clock Sun READY", || {
            LoginCommand::Rooms(room_command(sc, &r, rooms::Action::Ready, 1))
        });
        wait_owner(&mut owner, &d, "clock Sun READY ACK", |v| {
            !v.room_pending
                && v.rooms
                    .joined()
                    .is_some_and(|r| r.member(SUN).is_some_and(|m| m.ready))
        });
        let r = guest
            .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
            .unwrap()
            .joined
            .unwrap();
        let ready = guest
            .rooms(room_command(mc, &r, rooms::Action::Ready, 1))
            .unwrap();
        assert_eq!(
            (ready.status, ready.reason),
            (rooms::Status::Ok, rooms::Reason::None)
        );
        assert!(ready.joined.unwrap().summary.satisfied);
        let mut ping = Instant::now();
        let mut preparing_seen = false;
        let loading = loop {
            d.check("clock real controller auto-PREPARE");
            let v = pump(&mut owner);
            if let Some(f) = v.world.frame().filter(|f| f.match_id != previous_match) {
                policy(f, b);
                assert_ne!(f.phase, world::Phase::Aborted, "{:?}", f.abort_reason);
                preparing_seen |= f.phase == world::Phase::Preparing;
                if f.phase == world::Phase::Loading {
                    break Arc::new(f.clone());
                }
            }
            if ping.elapsed() >= Duration::from_secs(1) {
                guest.ping().unwrap();
                ping = Instant::now();
            }
            d.pause("clock preparing poll");
        };
        eprintln!(
            "clock map {} actual PREPARING observed: {}",
            b.map, preparing_seen
        );
        assert!(scopes.insert((loading.match_id, loading.generation)));
        previous_match = loading.match_id;
        assert_eq!(loading.room.summary.room_id, room_id.unwrap());
        assert_eq!(
            loading
                .room
                .members
                .iter()
                .map(|m| (m.key, m.class, m.sex, m.level))
                .collect::<Vec<_>>(),
            [(SUN, 1, 0, 80), (MOON, 3, 1, 80)]
        );
        assert_eq!(actor(&loading, SUN).position, b.anchor);
        assert!(loading.actors.iter().all(|a| !a.loaded && a.sequence == 0));
        assert_eq!(
            guest
                .world(world_request(mc, &loading, world::Action::Read))
                .unwrap()
                .frame
                .as_ref(),
            Some(&*loading)
        );
        sr.update(loading.clone(), actor(&loading, SUN).entity_id)
            .unwrap();
        mr.update(loading.clone(), actor(&loading, MOON).entity_id)
            .unwrap();
        let image_start = Instant::now();
        let image = loop {
            d.check("clock complete two actual first images");
            assert!(
                image_start.elapsed() < Duration::from_millis(u64::from(b.load_ms)),
                "actual binding LOAD deadline; no extended stage"
            );
            let v = pump(&mut owner);
            let f = v.world.frame().unwrap();
            policy(f, b);
            assert_eq!(
                (f.match_id, f.generation, f.phase),
                (loading.match_id, loading.generation, world::Phase::Loading)
            );
            if let (Some(s), Some(_)) = (
                scoped_image(&mut sr, &loading, SUN),
                scoped_image(&mut mr, &loading, MOON),
            ) {
                break s;
            }
            if ping.elapsed() >= Duration::from_secs(1) {
                guest.ping().unwrap();
                ping = Instant::now();
            }
            d.pause("clock retained resource workers");
        };
        eprintln!(
            "clock map {} two actual first images: {:.3}s",
            b.map,
            image_start.elapsed().as_secs_f64()
        );
        assert_eq!(image.navigation_map().unwrap().map_id(), b.map);
        let loaded = world::Request {
            manifest_digest: loading.manifest.digest,
            ..world_request(sc, &loading, world::Action::Loaded)
        };
        submit(&mut owner, &d, "clock Sun actual-image LOADED", || {
            LoginCommand::World(loaded)
        });
        let one = wait_owner(&mut owner, &d, "clock one real ACK barrier", |v| {
            !v.world_pending && v.world.frame().is_some_and(|f| actor(f, SUN).loaded)
        });
        let one = one.world.frame().unwrap();
        policy(one, b);
        assert_eq!(one.phase, world::Phase::Loading);
        assert!(!actor(one, MOON).loaded);
        // The actual server countdown start lies inside this request interval.
        let start_before = Instant::now();
        let countdown = guest
            .world(world::Request {
                manifest_digest: loading.manifest.digest,
                ..world_request(mc, &loading, world::Action::Loaded)
            })
            .unwrap()
            .frame
            .unwrap();
        let start_after = Instant::now();
        policy(&countdown, b);
        assert_eq!(countdown.phase, world::Phase::Countdown);
        let countdown_delay = Duration::from_nanos(b.countdown_ns);
        let active_delay = Duration::from_millis(u64::from(b.active_ms));
        let active = loop {
            d.check("clock observed actual countdown");
            assert!(
                Instant::now() < start_after + countdown_delay + OBSERVATION,
                "bounded countdown observation"
            );
            let v = pump(&mut owner);
            let f = v.world.frame().unwrap();
            policy(f, b);
            assert_ne!(f.phase, world::Phase::Aborted, "{:?}", f.abort_reason);
            if f.phase == world::Phase::Active {
                assert!(
                    Instant::now() >= start_before + countdown_delay,
                    "server countdown was not shortened"
                );
                break Arc::new(f.clone());
            }
            if ping.elapsed() >= Duration::from_secs(1) {
                guest.ping().unwrap();
                ping = Instant::now();
            }
            d.pause("clock countdown state poll");
        };
        let from = actor(&active, SUN);
        let map = image.navigation_map().unwrap();
        let (heading, to) = DIRECTIONS
            .iter()
            .enumerate()
            .map(|(i, &(dx, dy))| {
                (
                    i as u8,
                    WorldCell {
                        x: from.position.x + dx,
                        y: from.position.y + dy,
                    },
                )
            })
            .find(|(_, to)| {
                map.can_step(from.position, *to) && active.actors.iter().all(|a| a.position != *to)
            })
            .unwrap();
        let movement = world::Request {
            heading,
            sequence: from.sequence + 1,
            from_x: from.position.x,
            from_y: from.position.y,
            ..world_request(sc, &active, world::Action::Move)
        };
        submit(&mut owner, &d, "clock real active MOVE", || {
            LoginCommand::World(movement)
        });
        let moved = wait_owner(&mut owner, &d, "clock active MOVE ACK", |v| {
            !v.world_pending
                && v.world
                    .frame()
                    .is_some_and(|f| actor(f, SUN).sequence == from.sequence + 1)
        });
        let moved = moved.world.frame().unwrap();
        policy(moved, b);
        assert_eq!(moved.phase, world::Phase::Active);
        assert_eq!(actor(moved, SUN).position, to);
        assert_eq!(
            guest
                .world(world_request(mc, moved, world::Action::Read))
                .unwrap()
                .frame
                .as_ref(),
            Some(moved)
        );
        unsupported(&mut owner, &mut guest, sc, mc, Some(moved));
        let mut after_previous_expiry = false;
        let terminal = loop {
            d.check("clock authoritative ACTIVE_TIMEOUT");
            assert!(
                Instant::now() < start_after + countdown_delay + active_delay + OBSERVATION,
                "binding timeout not replaced by listener45s/oldclock"
            );
            let v = pump(&mut owner);
            let f = v.world.frame().unwrap();
            policy(f, b);
            if f.phase == world::Phase::Aborted {
                assert!(
                    Instant::now() >= start_before + countdown_delay + active_delay,
                    "server active timeout was not shortened"
                );
                assert_eq!(f.abort_reason, world::AbortReason::ActiveTimeout);
                assert_eq!(actor(f, SUN).position, to);
                break Arc::new(f.clone());
            }
            assert_eq!(f.phase, world::Phase::Active);
            if round == 1
                && !after_previous_expiry
                && Instant::now()
                    >= start_after
                        + countdown_delay
                        + Duration::from_millis(u64::from(bindings[0].active_ms))
                        + Duration::from_millis(200)
            {
                // A new on-wire READ begun beyond the old expiry proves this
                // is current server state, not a retained pre-expiry UI frame.
                let fresh = guest
                    .world(world_request(mc, f, world::Action::Read))
                    .unwrap()
                    .frame
                    .unwrap();
                policy(&fresh, b);
                assert_eq!(fresh.phase, world::Phase::Active);
                after_previous_expiry = true;
            }
            if ping.elapsed() >= Duration::from_secs(1) {
                guest.ping().unwrap();
                ping = Instant::now();
            }
            d.pause("clock active state poll, not synthetic sleep completion");
        };
        eprintln!("clock map {} actual ACTIVE_TIMEOUT at {:.3}s since final LOAD send (countdown={}ns, active={}ms)",
            b.map,start_before.elapsed().as_secs_f64(),b.countdown_ns,b.active_ms);
        if round == 1 {
            assert!(
                after_previous_expiry,
                "new 6s ACTIVE demonstrably outlives previous 4s clock"
            );
        }
        let remote = guest
            .world(world_request(mc, &terminal, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
        assert_eq!(remote, *terminal);
        assert!(remote.completion.is_none());
        assert!(sr
            .update(terminal.clone(), actor(&terminal, SUN).entity_id)
            .is_err());
        assert!(mr
            .update(terminal.clone(), actor(&terminal, MOON).entity_id)
            .is_err());
        // The real UI clears ABORTED presentation instead of rendering it.
        // Retire the owned workers, but retain these exact two reusable runtime
        // objects for the next binding; do not substitute shutdown/recreation.
        sr.clear();
        mr.clear();
        while !sr.is_quiescent() || !mr.is_quiescent() {
            sr.poll();
            mr.poll();
            pump(&mut owner);
            if ping.elapsed() >= Duration::from_secs(1) {
                guest.ping().unwrap();
                ping = Instant::now();
            }
            d.pause("clock ABORTED presentation owned-worker retirement");
        }
        let leave = world_request(sc, &terminal, world::Action::Leave);
        submit(&mut owner, &d, "clock Sun LEAVE", || {
            LoginCommand::World(leave)
        });
        wait_owner(&mut owner, &d, "clock Sun retired world", |v| {
            !v.world_pending && v.world.frame().is_none()
        });
        let held = guest
            .world(world_request(mc, &terminal, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
        assert_eq!(held, *terminal);
        guest
            .world(world_request(mc, &held, world::Action::Leave))
            .unwrap();
        let r = room(&mut owner, sc, &d);
        assert_eq!(r.summary.room_id, room_id.unwrap());
        assert_eq!(r.owner, SUN);
        assert_eq!(
            r.members.iter().map(|m| m.key).collect::<Vec<_>>(),
            [SUN, MOON]
        );
        assert_eq!(r.summary.ready, 0);
        assert!(!r.summary.satisfied);
        records(&mut owner, &mut guest, sc, &d);
        unsupported(&mut owner, &mut guest, sc, mc, None);
        assert_eq!(
            (
                guest.ack().unwrap().version,
                guest.ack().unwrap().capabilities
            ),
            (15, CAPS)
        );
    }
    assert_eq!(scopes.len(), 2);
    sr.shutdown();
    mr.shutdown();
    while !sr.is_quiescent() || !mr.is_quiescent() {
        sr.poll();
        mr.poll();
        d.pause("clock owned render retirement");
    }
    guest.close().unwrap();
    submit(&mut owner, &d, "clock normal TLS shutdown", || {
        LoginCommand::Shutdown
    });
    while !owner.is_quiescent() {
        owner.poll();
        d.pause("clock SessionController retirement");
    }
    assert_eq!(owner.snapshot().phase, LoginPhase::Disconnected);
    // Java independently verifies actual source reads and all three result SQL
    // tables remain empty, then owns normal connection/fixture process teardown.
    peer.expect_successful_exit(&d);
}
