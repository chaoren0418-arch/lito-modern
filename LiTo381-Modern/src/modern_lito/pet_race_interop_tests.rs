//! Real v16 race wire/renderer/SQL lifecycle. No invented positions, progress,
//! timers, or LOADED acknowledgements; root alone runs the isolated peers.
use super::super::{race_protocol as race, transport::ModernTlsConnection};
use super::*;

const CAPS: u32 = 196_975;
const PEER: &str = "local.lito381.server.ModernPetRaceTestPeer";
const IO: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Flow {
    Finish,
    Withdraw,
    Disconnect,
}
impl Flow {
    fn mode(self) -> &'static str {
        match self {
            Self::Finish => "race-finish",
            Self::Withdraw => "race-withdraw",
            Self::Disconnect => "race-disconnect",
        }
    }
    fn winner(self) -> CharacterKey {
        if self == Self::Withdraw {
            MOON
        } else {
            SUN
        }
    }
    fn loser_reason(self) -> race::DnfReason {
        match self {
            Self::Finish => race::DnfReason::WindowClosed,
            Self::Withdraw => race::DnfReason::Left,
            Self::Disconnect => race::DnfReason::Disconnected,
        }
    }
}
fn digest(s: &str) -> [u8; 32] {
    std::array::from_fn(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap())
}
fn absent() -> record::Totals {
    record::Totals {
        stored: false,
        wins: 0,
        losses: 0,
        revision: 0,
    }
}
fn data(f: &world::Frame) -> &race::RaceData {
    f.race.as_ref().expect("authoritative race appendix")
}
fn runner(f: &world::Frame, key: CharacterKey) -> &race::Runner {
    data(f).runner(actor(f, key).entity_id).unwrap()
}
fn policy(f: &world::Frame) {
    f.validate(SUN).unwrap();
    assert_eq!(
        f.descriptor,
        Some(world::MatchDescriptor {
            ruleset: world::MatchRuleset::PetRace,
            features: world::MatchDescriptor::PET_RACE,
            preparation_timeout_millis: 30_000,
            load_timeout_millis: 30_000,
            countdown_nanos: 1_000_000_000,
            active_timeout_millis: 120_000,
        })
    );
    assert!(!f.combat_enabled(true) && f.completion.is_none() && f.casts.is_empty());
    assert_eq!(
        (
            f.room.summary.settings.mode,
            f.room.summary.settings.map_id,
            f.manifest.map_id
        ),
        (0, 565, 5143)
    );
    assert_eq!(
        f.manifest.digest,
        digest("f6c171c57eeae9e9223af51b12e89a1f0640086150abb41252e5e742c6feffde")
    );
    let pins = [
        (
            0x7fff,
            0x8000,
            127474,
            "72f52a1f3dbc6db65650c4c470f254a6a1d94b4283dc928b78edb4921aacdb15",
        ),
        (
            0x7fff,
            0x8001,
            141309,
            "2712d11716d060cf220d3f33ab2c26a190bb4c333ab78e185eb24baf996effea",
        ),
        (
            0x8000,
            0x8000,
            111184,
            "c8f0ef53c400f2558e7239c21d092be10a42613fd2b74ef0fe9270d05a3e5cb8",
        ),
        (
            0x8000,
            0x8001,
            99133,
            "054ca18e1cb46f4084135fc5409bb692bb8e99b998c50579148f6b2059bcae0d",
        ),
    ];
    assert_eq!(f.manifest.sectors.len(), pins.len());
    for (s, (x, y, len, sha)) in f.manifest.sectors.iter().zip(pins) {
        assert_eq!(
            (s.x, s.y, s.byte_length, s.sha256),
            (x, y, len, digest(sha))
        );
    }
    assert_eq!(data(f).gates.len(), 29);
    assert_eq!(data(f).runners.len(), 2);
    assert!(data(f).runners.iter().all(|r| r.base_graphics_id == 2145));
    assert!(f.actors.iter().all(|a| a.step_interval_nanos == 1_000_000));
    assert_eq!(
        f.room
            .members
            .iter()
            .map(|m| (m.key, m.class, m.sex, m.level))
            .collect::<Vec<_>>(),
        [(SUN, 1, 0, 80), (MOON, 3, 1, 80)]
    );
    assert_ne!(
        f.phase,
        world::Phase::Aborted,
        "race aborted: {:?}",
        f.abort_reason
    );
}
fn pump(owner: &mut SessionController) -> LoginSnapshot {
    owner.poll();
    let v = owner.snapshot();
    assert_ne!(v.phase, LoginPhase::Failed, "{}", v.status);
    assert!(!v.combat_supported && v.combat.is_none());
    v
}
fn connect_owner(
    peer: &JavaPeer,
    shared: &PathBuf,
    ca: &[u8],
    d: &Deadline,
) -> (SessionController, SelectionContext) {
    let mut owner = SessionController::new(profile(peer.endpoint, shared, ca));
    submit(&mut owner, d, "race owner CONNECT", || {
        LoginCommand::Connect
    });
    wait_owner(&mut owner, d, "race verified TLS", |v| {
        v.phase == LoginPhase::LoginReady
    });
    submit(&mut owner, d, "race owner AUTH", || LoginCommand::Login {
        account: "fixture-user".into(),
        password: "fixture-password".into(),
    });
    wait_owner(&mut owner, d, "race source chooser", |v| {
        v.phase == LoginPhase::ChoosingSource
    });
    submit(&mut owner, d, "race source7", || {
        LoginCommand::SelectSource(7)
    });
    let roster = wait_owner(&mut owner, d, "race source7 roster", |v| {
        v.phase == LoginPhase::CharactersReady
    });
    assert_eq!(roster.characters.len(), 1);
    assert_eq!(
        (
            roster.characters[0].source_id,
            roster.characters[0].object_id
        ),
        (7, 3333)
    );
    let sc = SelectionContext {
        key: SUN,
        epoch: roster.roster_epoch,
    };
    submit(&mut owner, d, "race Sun selected", || {
        LoginCommand::SelectCharacter(sc)
    });
    wait_owner(&mut owner, d, "race selected record", |v| {
        v.phase == LoginPhase::ChoosingChannel && !v.record_pending && v.record.totals().is_some()
    });
    assert_eq!(owner.snapshot().record.totals(), Some(absent()));
    submit(&mut owner, d, "race Sun channel", || {
        LoginCommand::EnterChannel {
            context: sc,
            channel_id: CHANNEL,
        }
    });
    wait_owner(&mut owner, d, "race Sun lobby", |v| {
        v.phase == LoginPhase::InLobby
    });
    (owner, sc)
}
fn connect_guest(peer: &JavaPeer, ca: &[u8]) -> (ModernTlsConnection, SelectionContext) {
    let mut guest = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(ca),
        Hello::current(100),
        TransportLimits {
            connect_timeout: Duration::from_millis(500),
            total_timeout: IO,
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
        (16, CAPS)
    );
    guest
        .authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    assert_eq!(guest.characters(42, 1).unwrap().characters[0].key, MOON);
    let mc = SelectionContext {
        key: MOON,
        epoch: 1,
    };
    guest.select_character(mc).unwrap();
    assert_eq!(guest.record().unwrap().totals, Some(absent()));
    guest.enter_channel(CHANNEL).unwrap();
    (guest, mc)
}
fn owner_room(
    owner: &mut SessionController,
    sc: SelectionContext,
    d: &Deadline,
) -> rooms::RoomSnapshot {
    submit(owner, d, "race scoped Rooms READ", || {
        LoginCommand::Rooms(rooms::Request::read(sc, CHANNEL).unwrap())
    });
    wait_owner(owner, d, "race Rooms READ ACK", |v| {
        !v.room_pending && v.rooms.joined().is_some()
    })
    .rooms
    .joined()
    .unwrap()
    .clone()
}
fn read_guest(
    guest: &mut ModernTlsConnection,
    mc: SelectionContext,
    f: &world::Frame,
) -> world::Frame {
    let request = world_request(mc, f, world::Action::Read);
    let reply = guest.world(request).unwrap();
    assert_eq!(
        (reply.status, reply.reason),
        (world::Status::Ok, world::Reason::None)
    );
    assert!(reply.confirms(&request));
    let frame = reply.frame.unwrap();
    policy(&frame);
    frame
}
fn image(
    runtime: &mut WorldRuntime,
    f: &world::Frame,
    key: CharacterKey,
) -> Option<Arc<RenderedWorld>> {
    let image = rendered(runtime, f, key)?;
    assert_eq!(
        *image.world, *f,
        "pixels retain the exact authoritative roster"
    );
    super::super::world_window::validate_player_surface(
        super::super::world_window::HitSurface {
            match_id: image.match_id,
            generation: image.generation,
            revision: image.revision,
            selected_entity_id: image.selected_entity_id,
            projection: image.projection,
        },
        &super::super::world_window::PlayerSurface {
            frame: image.world.clone(),
            actors: image.actors.clone(),
        },
    )
    .unwrap();
    assert!(image.combat.is_none() && image.objectives.is_empty());
    assert!(
        image.audio.as_ref().unwrap().events.is_empty(),
        "four qualified pets have no sound cues"
    );
    assert_eq!(
        image
            .actors
            .iter()
            .map(|a| a.entity_id)
            .collect::<BTreeSet<_>>(),
        f.actors
            .iter()
            .filter(|a| f.actor_occupies(a.entity_id))
            .map(|a| a.entity_id)
            .collect()
    );
    assert!(image.actors.iter().all(|a| a.graphics_id == 2145));
    Some(image)
}
fn submit_frames(sr: &mut WorldRuntime, mr: &mut WorldRuntime, f: &world::Frame, flow: Flow) {
    if flow != Flow::Withdraw {
        sr.update(Arc::new(f.clone()), actor(f, SUN).entity_id)
            .unwrap();
    }
    if flow != Flow::Disconnect {
        mr.update(Arc::new(f.clone()), actor(f, MOON).entity_id)
            .unwrap();
    }
    for runtime in [sr, mr] {
        if let RuntimeStatus::Failed(reason) = runtime.poll() {
            panic!("race renderer: {reason}");
        }
    }
}
fn retired(runtime: &mut WorldRuntime, owner: &mut SessionController, d: &Deadline) {
    runtime.clear();
    while !runtime.is_quiescent() {
        runtime.poll();
        pump(owner);
        d.pause("race owned renderer retirement");
    }
}

/// Bounded BFS over the exact decoded map, including occupied diagonal elbows.
/// When already inside the expected gate, first find an outside cell; entry,
/// not standing inside a rectangle, is what may advance authority progress.
fn next_step(map: &WorldMap, f: &world::Frame, key: CharacterKey) -> (u8, WorldCell) {
    let a = actor(f, key);
    let r = runner(f, key);
    assert!(r.competing());
    let gate = &data(f).gates[usize::try_from(r.next_checkpoint).unwrap()];
    let leave = gate.contains(a.position);
    let occupied = f
        .actors
        .iter()
        .filter(|x| x.entity_id != a.entity_id && f.actor_occupies(x.entity_id))
        .map(|x| x.position)
        .collect::<BTreeSet<_>>();
    let mut queue = VecDeque::from([a.position]);
    let mut previous = BTreeMap::from([(a.position, (a.position, 0u8))]);
    let goal = loop {
        let cell = queue
            .pop_front()
            .expect("qualified race gate reachable with active occupancy");
        if cell != a.position && gate.contains(cell) != leave {
            break cell;
        }
        for (heading, (dx, dy)) in DIRECTIONS.iter().copied().enumerate() {
            let to = WorldCell {
                x: cell.x + dx,
                y: cell.y + dy,
            };
            if !previous.contains_key(&to) && world_path::step_open(map, cell, to, &occupied) {
                assert!(previous.len() < 16_384, "bounded four-sector race BFS");
                previous.insert(to, (cell, heading as u8));
                queue.push_back(to);
            }
        }
    };
    let mut cursor = goal;
    loop {
        let (from, heading) = previous[&cursor];
        if from == a.position {
            return (heading, cursor);
        }
        cursor = from;
    }
}
fn assert_move(old: &world::Frame, new: &world::Frame, key: CharacterKey, to: WorldCell) {
    policy(new);
    assert!(race::follows(old, new));
    let a = actor(old, key);
    let b = actor(new, key);
    assert_eq!((b.sequence, b.position), (a.sequence + 1, to));
    let before = runner(old, key);
    let after = runner(new, key);
    let gate = &data(old).gates[before.next_checkpoint as usize];
    let crossing = !gate.contains(a.position) && gate.contains(to);
    let next = before.next_checkpoint + if crossing { 1 } else { 0 };
    let wrap = next == data(old).gates.len() as i64;
    assert_eq!(
        (after.completed_laps, after.next_checkpoint),
        (
            before.completed_laps + if wrap { 1 } else { 0 },
            if wrap { 0 } else { next }
        )
    );
    assert_eq!(
        after.progress_move_sequence,
        if crossing {
            b.sequence
        } else {
            before.progress_move_sequence
        }
    );
    assert_eq!(
        after.revision,
        if crossing {
            new.revision
        } else {
            before.revision
        }
    );
}

fn run(flow: Flow) {
    let d = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(&fixtures, &d, PEER, flow.mode());
    assert!(peer.process_job.is_some());
    let (mut owner, sc) = connect_owner(&peer, &shared, &ca, &d);
    let (guest, mc) = connect_guest(&peer, &ca);
    let mut guest = Some(guest);
    let create_request = || {
        rooms::Request::draft(
            rooms::Common {
                action: rooms::Action::Create,
                ..rooms::Common::read(sc, CHANNEL)
            },
            rooms::Draft::new(
                rooms::PublicDraft {
                    capacity: 2,
                    mode: 0,
                    attributes: 0,
                    map_id: 565,
                    name: "Shared pet race lifecycle".into(),
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap()
    };
    submit(&mut owner, &d, "race CREATE", || {
        LoginCommand::Rooms(create_request())
    });
    let created = wait_owner(&mut owner, &d, "race CREATE ACK", |v| {
        !v.room_pending && v.rooms.joined().is_some()
    });
    let room = created.rooms.joined().unwrap();
    let room_id = room.summary.room_id;
    let joined = guest
        .as_mut()
        .unwrap()
        .rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id,
                    expected_revision: room.summary.revision,
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
    let room = owner_room(&mut owner, sc, &d);
    submit(&mut owner, &d, "race Sun READY", || {
        LoginCommand::Rooms(room_command(sc, &room, rooms::Action::Ready, 1))
    });
    wait_owner(&mut owner, &d, "race Sun READY ACK", |v| {
        !v.room_pending
            && v.rooms
                .joined()
                .is_some_and(|r| r.member(SUN).is_some_and(|m| m.ready))
    });
    let read = guest
        .as_mut()
        .unwrap()
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap();
    assert_eq!(read.status, rooms::Status::Ok);
    let ready = guest
        .as_mut()
        .unwrap()
        .rooms(room_command(
            mc,
            read.joined.as_ref().unwrap(),
            rooms::Action::Ready,
            1,
        ))
        .unwrap();
    assert_eq!(ready.status, rooms::Status::Ok);
    assert!(ready.joined.unwrap().summary.satisfied);
    let mut last_guest = Instant::now();
    let loading = loop {
        d.check("race automatic PREPARE");
        let v = pump(&mut owner);
        if let Some(f) = v.world.frame() {
            policy(f);
            if f.phase == world::Phase::Loading {
                break f.clone();
            }
        }
        if last_guest.elapsed() >= Duration::from_secs(1) {
            guest.as_mut().unwrap().ping().unwrap();
            last_guest = Instant::now();
        }
        d.pause("race prepare transition");
    };
    assert_eq!(
        actor(&loading, SUN).position,
        WorldCell { x: 32768, y: 32849 }
    );
    assert!(loading.actors.iter().all(|a| !a.loaded && a.sequence == 0));
    assert_eq!(read_guest(guest.as_mut().unwrap(), mc, &loading), loading);
    let mut sr = WorldRuntime::new(shared.clone()).unwrap();
    let mut mr = WorldRuntime::new(shared).unwrap();
    sr.update(Arc::new(loading.clone()), actor(&loading, SUN).entity_id)
        .unwrap();
    mr.update(Arc::new(loading.clone()), actor(&loading, MOON).entity_id)
        .unwrap();
    let load_start = Instant::now();
    let map = loop {
        d.check("race two real first renderers");
        assert!(load_start.elapsed() < Duration::from_secs(30));
        let v = pump(&mut owner);
        assert_eq!(v.world.frame().unwrap().phase, world::Phase::Loading);
        if let (Some(s), Some(_)) = (
            image(&mut sr, &loading, SUN),
            image(&mut mr, &loading, MOON),
        ) {
            break s.navigation_map().unwrap();
        }
        if last_guest.elapsed() >= Duration::from_secs(1) {
            guest.as_mut().unwrap().ping().unwrap();
            last_guest = Instant::now();
        }
        d.pause("race actual assets before LOADED");
    };
    assert_eq!(map.map_id(), 5143);
    let loaded = world::Request {
        manifest_digest: loading.manifest.digest,
        ..world_request(sc, &loading, world::Action::Loaded)
    };
    submit(&mut owner, &d, "race first image Sun LOADED", || {
        LoginCommand::World(loaded)
    });
    let one = wait_owner(&mut owner, &d, "race single-owner barrier", |v| {
        !v.world_pending && v.world.frame().is_some_and(|f| actor(f, SUN).loaded)
    });
    assert_eq!(one.world.frame().unwrap().phase, world::Phase::Loading);
    assert!(!actor(one.world.frame().unwrap(), MOON).loaded);
    let countdown_before = Instant::now();
    let reply = guest
        .as_mut()
        .unwrap()
        .world(world::Request {
            manifest_digest: loading.manifest.digest,
            ..world_request(mc, &loading, world::Action::Loaded)
        })
        .unwrap();
    assert_eq!(reply.status, world::Status::Ok);
    let countdown = reply.frame.unwrap();
    assert_eq!(countdown.phase, world::Phase::Countdown);
    policy(&countdown);
    let mut frame = loop {
        d.check("race real countdown");
        let v = pump(&mut owner);
        let f = v.world.frame().unwrap();
        policy(f);
        if f.phase == world::Phase::Active {
            assert!(countdown_before.elapsed() >= Duration::from_secs(1));
            break f.clone();
        }
        d.pause("race countdown poll");
    };
    assert_eq!(data(&frame).state, race::RaceState::Running);
    if flow == Flow::Withdraw {
        let leave = world_request(sc, &frame, world::Action::Leave);
        submit(&mut owner, &d, "race Sun real WorldLeave", || {
            LoginCommand::World(leave)
        });
        wait_owner(&mut owner, &d, "race Sun WorldLeave ACK", |v| {
            !v.world_pending && v.world.frame().is_none()
        });
        let room = owner_room(&mut owner, sc, &d);
        submit(&mut owner, &d, "race Sun fresh RoomsLeave", || {
            LoginCommand::Rooms(room_command(sc, &room, rooms::Action::Leave, 0))
        });
        wait_owner(&mut owner, &d, "race Sun RoomsLeave ACK", |v| {
            !v.room_pending && v.rooms.joined().is_none()
        });
        retired(&mut sr, &mut owner, &d);
        frame = read_guest(guest.as_mut().unwrap(), mc, &frame);
        assert_eq!(frame.phase, world::Phase::Active);
        assert_eq!(runner(&frame, SUN).dnf, race::DnfReason::Left);
        assert!(!frame.actor_occupies(actor(&frame, SUN).entity_id));
    } else if flow == Flow::Disconnect {
        guest.take().unwrap().close().unwrap();
        retired(&mut mr, &mut owner, &d);
        frame = wait_owner(
            &mut owner,
            &d,
            "race true TLS disconnect changes only Moon",
            |v| {
                v.world
                    .frame()
                    .is_some_and(|f| runner(f, MOON).dnf == race::DnfReason::Disconnected)
            },
        )
        .world
        .frame()
        .unwrap()
        .clone();
        assert_eq!(frame.phase, world::Phase::Active);
        assert!(!frame.actor_occupies(actor(&frame, MOON).entity_id));
    }
    let winner = flow.winner();
    let winner_context = if winner == SUN { sc } else { mc };
    let mut moves = 0usize;
    let mut first_before = None;
    let mut first_after = None;
    while runner(&frame, winner).placement == 0 {
        d.check("race ordered gate MOVEs");
        assert_eq!(frame.phase, world::Phase::Active);
        let (heading, to) = next_step(&map, &frame, winner);
        let a = actor(&frame, winner);
        let request = world::Request {
            heading,
            sequence: a.sequence + 1,
            from_x: a.position.x,
            from_y: a.position.y,
            ..world_request(winner_context, &frame, world::Action::Move)
        };
        let before = Instant::now();
        let next = if winner == SUN {
            submit(&mut owner, &d, "race authoritative Sun MOVE", || {
                LoginCommand::World(request)
            });
            wait_owner(&mut owner, &d, "race exact Sun MOVE ACK", |v| {
                !v.world_pending
                    && v.world
                        .frame()
                        .is_some_and(|f| actor(f, SUN).sequence == request.sequence)
            })
            .world
            .frame()
            .unwrap()
            .clone()
        } else {
            // Explicit QA cadence exceeds the configured 1ms server admission.
            thread::sleep(d.remaining(Duration::from_millis(2), "race raw MOVE cadence"));
            let reply = guest.as_mut().unwrap().world(request).unwrap();
            assert_eq!(
                (reply.status, reply.reason),
                (world::Status::Ok, world::Reason::None)
            );
            assert!(reply.confirms(&request));
            reply.frame.unwrap()
        };
        let after = Instant::now();
        assert_move(&frame, &next, winner, to);
        frame = next;
        moves += 1;
        assert!(moves <= 4_096, "bounded real route, no endless gate chase");
        submit_frames(&mut sr, &mut mr, &frame, flow);
        pump(&mut owner);
        if runner(&frame, winner).placement > 0 {
            first_before = Some(before);
            first_after = Some(after);
            break;
        }
        if last_guest.elapsed() >= Duration::from_millis(750) {
            if let Some(g) = &mut guest {
                let fresh = read_guest(g, mc, &frame);
                assert!(race::follows(&frame, &fresh));
                frame = fresh;
                g.ping().unwrap();
            }
            last_guest = Instant::now();
        }
    }
    assert!(moves >= 4 * data(&frame).gates.len());
    let first_before = first_before.unwrap();
    let first_after = first_after.unwrap();
    assert_eq!(runner(&frame, winner).placement, 1);
    assert_eq!(runner(&frame, winner).completed_laps, 4);
    assert_eq!(data(&frame).state, race::RaceState::FinishWindow);
    assert_eq!(frame.phase, world::Phase::Active);
    assert_eq!(
        data(&frame).finish_window_deadline_elapsed_nanos
            - runner(&frame, winner).finish_elapsed_nanos,
        10_000_000_000
    );
    assert!(!frame.actor_occupies(actor(&frame, winner).entity_id));
    let first = frame.clone();
    let mut pending_seen = false;
    let saved = loop {
        d.check("race actual ten-second finish window and SAVED");
        assert!(
            Instant::now() < first_after + Duration::from_secs(14),
            "10s window plus bounded store/transport observation"
        );
        let v = pump(&mut owner);
        let next = if flow == Flow::Disconnect {
            v.world.frame().unwrap().clone()
        } else {
            read_guest(guest.as_mut().unwrap(), mc, &frame)
        };
        policy(&next);
        assert!(race::follows(&frame, &next));
        frame = next;
        submit_frames(&mut sr, &mut mr, &frame, flow);
        match data(&frame).state {
            race::RaceState::FinishWindow => assert_eq!(frame.phase, world::Phase::Active),
            race::RaceState::Complete => {
                assert!(
                    first_before.elapsed() + Duration::from_millis(100) >= Duration::from_secs(10),
                    "not an immediate all-finished shortcut"
                );
                assert_eq!(frame.phase, world::Phase::Finished);
                match data(&frame).settlement {
                    race::Settlement::Saved => break frame.clone(),
                    race::Settlement::Pending => pending_seen = true,
                    other => panic!("race must durably save, not {other:?}"),
                }
            }
            other => panic!("race lost finish window: {other:?}"),
        }
        d.pause("race finish-window authoritative polling");
    };
    eprintln!(
        "{} moves={moves}, actual finish window={:.3}s, pending observed={pending_seen}",
        flow.mode(),
        first_before.elapsed().as_secs_f64()
    );
    let loser = if winner == SUN { MOON } else { SUN };
    assert_eq!(runner(&saved, loser).dnf, flow.loser_reason());
    assert_eq!(runner(&saved, loser).placement, 0);
    assert_eq!(
        data(&saved).first_finisher_entity_id,
        actor(&saved, winner).entity_id
    );
    assert_eq!(
        data(&saved).finish_window_deadline_elapsed_nanos,
        data(&first).finish_window_deadline_elapsed_nanos
    );
    assert_ne!(data(&saved).result_id, [0; 16]);
    assert_ne!(data(&saved).result_content_sha256, [0; 32]);
    assert!(saved.completion.is_none() && data(&saved).runners.iter().all(|r| !r.competing()));
    loop {
        d.check("race terminal pixels contain no inactive pets");
        pump(&mut owner);
        let s = flow == Flow::Withdraw
            || image(&mut sr, &saved, SUN).is_some_and(|p| p.actors.is_empty());
        let m = flow == Flow::Disconnect
            || image(&mut mr, &saved, MOON).is_some_and(|p| p.actors.is_empty());
        if s && m {
            break;
        }
        d.pause("race terminal renderer");
    }
    if flow != Flow::Withdraw {
        // SessionController may observe the same SAVED by its own 200ms poll.
        wait_owner(&mut owner, &d, "race owner sees real SAVED", |v| {
            v.world
                .frame()
                .is_some_and(|f| data(f).settlement == race::Settlement::Saved)
        });
        let request = world_request(sc, &saved, world::Action::Leave);
        submit(&mut owner, &d, "race normal Sun terminal LEAVE", || {
            LoginCommand::World(request)
        });
        wait_owner(&mut owner, &d, "race Sun terminal detached", |v| {
            !v.world_pending && v.world.frame().is_none()
        });
    }
    if let Some(g) = &mut guest {
        let held = read_guest(g, mc, &saved);
        assert_eq!(held, saved);
        let reply = g
            .world(world_request(mc, &held, world::Action::Leave))
            .unwrap();
        assert_eq!(reply.status, world::Status::Ok);
        assert!(reply.frame.is_none());
        assert_eq!(g.record().unwrap().totals, Some(absent()));
        g.ping().unwrap();
        let r = g.rooms(rooms::Request::read(mc, CHANNEL).unwrap()).unwrap();
        assert_eq!(r.status, rooms::Status::Ok);
        assert_eq!(r.joined.as_ref().unwrap().summary.room_id, room_id);
        assert_eq!(r.joined.as_ref().unwrap().summary.ready, 0);
    }
    submit(&mut owner, &d, "race DM record remains absent", || {
        LoginCommand::ReadRecord
    });
    wait_owner(&mut owner, &d, "race DM record read ACK", |v| {
        !v.record_pending && v.record.totals() == Some(absent())
    });
    if flow == Flow::Withdraw {
        assert!(owner.snapshot().rooms.joined().is_none());
    } else {
        let room = owner_room(&mut owner, sc, &d);
        assert_eq!(room.summary.room_id, room_id);
        assert!(!room.summary.satisfied && room.summary.ready == 0);
    }
    sr.shutdown();
    mr.shutdown();
    while !sr.is_quiescent() || !mr.is_quiescent() {
        sr.poll();
        mr.poll();
        pump(&mut owner);
        d.pause("race both owned workers retire");
    }
    if let Some(mut g) = guest {
        g.close().unwrap();
    }
    submit(
        &mut owner,
        &d,
        "race normal controller TLS shutdown",
        || LoginCommand::Shutdown,
    );
    while !owner.is_quiescent() {
        owner.poll();
        d.pause("race owned SessionController retirement");
    }
    peer.expect_successful_exit(&d);
}

#[test]
#[ignore = "real v16 two-owner TLS, complete shared5143/pet renderers, four laps and isolated race SQL; no GUI"]
fn java_pet_race_four_ordered_laps_real_window_saved_and_retire() {
    run(Flow::Finish)
}
#[test]
#[ignore = "real v16 WorldLeave + fresh RoomsLeave preserves Moon race, LEFT identity and isolated race SQL"]
fn java_pet_race_world_withdraw_does_not_abort_other_runner() {
    run(Flow::Withdraw)
}
#[test]
#[ignore = "real v16 Moon TLS disconnect preserves Sun race, DISCONNECTED identity and isolated race SQL"]
fn java_pet_race_true_tls_disconnect_does_not_abort_other_runner() {
    run(Flow::Disconnect)
}
