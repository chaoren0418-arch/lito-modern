//! Actual SessionController v39, shared resources and owned SQL. The v27 guest
//! exercises coexistence; it cannot supply a legacy Frame to the v39 renderer.
use super::*;
use crate::modern_lito::{
    battle_action::{Guard, Rejection},
    battle_control::{Command as PlayerCommand, Operation, Outcome},
    battle_operation::{Disposition, Reason},
    battle_player,
    battle_runtime::{BattleRuntime, Status},
    battle_window::{Stamp, Target},
    battle_world,
};

fn no_legacy(view: &LoginSnapshot) {
    assert!(view.world.frame().is_none());
    assert!(view.combat.is_none() && view.npc.is_none());
    assert!(view.battle.observation().is_none());
}
fn command(
    owner: &mut SessionController,
    d: &Deadline,
    label: &'static str,
    q: PlayerCommand,
) -> LoginSnapshot {
    submit(owner, d, label, || LoginCommand::Canonical(q.clone()));
    assert!(owner.snapshot().world_pending);
    let view = wait_owner(owner, d, label, |v| !v.world_pending);
    no_legacy(&view);
    view
}

fn rendered(
    runtime: &mut BattleRuntime,
    view: &LoginSnapshot,
    guest: &mut ModernTlsConnection,
    deadline: &Deadline,
) -> Arc<crate::modern_lito::battle_scene::Surface> {
    let observation = view.canonical.observation().unwrap();
    let mut keepalive = Instant::now();
    loop {
        match battle_player::update(runtime, view, false).unwrap() {
            Status::Ready(surface) if surface.observation.data() == observation.data() => {
                assert!(battle_player::admitted(&surface, view));
                assert!(battle_player::controls_ready(&surface, view));
                return surface;
            }
            Status::Failed(reason) => panic!("canonical confirmed scene failed: {reason}"),
            _ => {}
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            guest.ping().unwrap();
            keepalive = Instant::now();
        }
        deadline.pause("canonical confirmed render");
    }
}

#[test]
#[ignore = "v39 real controller + v27 peer, shared514/SPR, isolated SQL kill/saved/return; no HWND"]
fn java_v39_controller_shared_load_move_kill_saved_navigation_and_shutdown() {
    controller_flow(false);
}
#[test]
#[ignore = "INTERACTIVE real canonical HWND + TLS/shared514/isolated SQL; operator must move, cast77, return and close"]
fn java_canonical_player_window_move_cast_saved_return_and_close() {
    controller_flow(true);
}
fn controller_flow(window: bool) {
    let d = if window {
        Deadline(Instant::now() + Duration::from_secs(580))
    } else {
        Deadline::new()
    };
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernDisintegrateTestPeer",
        "canonical-disintegrate",
        window.then_some("600000"),
    );
    let mut owner = SessionController::new_canonical(profile(peer.endpoint, &shared, &ca));
    submit(&mut owner, &d, "v39 connect", || LoginCommand::Connect);
    wait_owner(&mut owner, &d, "v39 TLS", |v| {
        v.phase == LoginPhase::LoginReady
    });
    submit(&mut owner, &d, "v39 auth", || LoginCommand::Login {
        account: "fixture-user".into(),
        password: "fixture-password".into(),
    });
    wait_owner(&mut owner, &d, "v39 sources", |v| {
        v.phase == LoginPhase::ChoosingSource
    });
    submit(&mut owner, &d, "v39 source", || {
        LoginCommand::SelectSource(SUN.source_id)
    });
    let roster = wait_owner(&mut owner, &d, "v39 roster", |v| {
        v.phase == LoginPhase::CharactersReady
    });
    let sc = SelectionContext {
        key: SUN,
        epoch: roster.roster_epoch,
    };
    submit(&mut owner, &d, "v39 selection", || {
        LoginCommand::SelectCharacter(sc)
    });
    let selected = wait_owner(&mut owner, &d, "v39 catalogs", |v| {
        v.phase == LoginPhase::ChoosingChannel
            && !v.spell_pending
            && v.skills.disintegrate().is_some()
            && !v.record_pending
            && v.record.totals().is_some()
    });
    assert_eq!(selected.record.totals().unwrap().revision, 0);
    submit(&mut owner, &d, "v39 channel", || {
        LoginCommand::EnterChannel {
            context: sc,
            channel_id: CHANNEL,
        }
    });
    let lobby = wait_owner(&mut owner, &d, "v39 lobby", |v| {
        v.phase == LoginPhase::InLobby && v.canonical.supported()
    });
    no_legacy(&lobby);
    assert!(!lobby.canonical.attached());
    for _ in 0..5 {
        submit(&mut owner, &d, "v39 shop", || {
            LoginCommand::Shop(shop::Request::read(sc, CHANNEL, None))
        });
        wait_owner(&mut owner, &d, "v39 shop ACK", |v| !v.shop_pending);
    }
    let (mut guest, gc) = connect(&peer, &ca, MOON);
    // Use the same admission decision as the player button, with the actual
    // peer's catalog (not a locally enabled fixture option).
    let create_common = crate::modern_lito::login_window::room_common(
        &owner.snapshot(),
        rooms::Action::Create,
        None,
        0,
    )
    .expect("bound server map must keep the player create button enabled");
    submit(&mut owner, &d, "v39 create", || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                create_common,
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "Canonical controller".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    let created = wait_owner(&mut owner, &d, "v39 own room", |v| {
        !v.room_pending && v.rooms.joined().is_some()
    });
    let room_id = created.rooms.joined().unwrap().summary.room_id;
    let directory = guest
        .rooms(rooms::Request::read(gc, CHANNEL).unwrap())
        .unwrap();
    let room = directory
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
                    expected_revision: room.revision,
                    ..rooms::Common::read(gc, CHANNEL)
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap();
    let joined = wait_owner(&mut owner, &d, "v39 guest joined", |v| {
        v.rooms.joined().is_some_and(|r| r.members.len() == 2)
    });
    let ready_common =
        crate::modern_lito::login_window::room_common(&joined, rooms::Action::Ready, None, 1)
            .expect("bound map must keep the player ready button enabled");
    assert_eq!(
        ready_common,
        room_command(sc, joined.rooms.joined().unwrap(), rooms::Action::Ready, 1).common
    );
    submit(&mut owner, &d, "v39 ready", || {
        LoginCommand::Rooms(rooms::Request::simple(ready_common).unwrap())
    });
    wait_owner(&mut owner, &d, "v39 ready ACK", |v| !v.room_pending);
    let room = guest
        .rooms(rooms::Request::read(gc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    guest
        .rooms(room_command(gc, &room, rooms::Action::Ready, 1))
        .unwrap();
    let loading = wait_owner(&mut owner, &d, "v39 automatic prepare", |v| {
        v.canonical
            .lifecycle()
            .is_some_and(|l| l.phase == world::Phase::Loading)
    });
    no_legacy(&loading);
    let initial = loading.canonical.observation().unwrap().clone();
    let key = initial.data().world.identity.clone();
    if window {
        let ticket = guest
            .world(world::Request {
                match_id: key.match_id as u64,
                generation: key.generation as u64,
                ..world::Request::read(gc, CHANNEL)
            })
            .unwrap()
            .frame
            .unwrap();
        guest
            .world(world::Request {
                manifest_digest: ticket.manifest.digest,
                ..world_request(gc, &ticket, world::Action::Loaded)
            })
            .unwrap();
        let (stop, receive) = mpsc::sync_channel::<()>(1);
        // Always join this owned keepalive, including an Err from the HWND owner.
        let result = thread::scope(|scope| {
            let worker = scope.spawn(move || {
                while receive.recv_timeout(Duration::from_millis(500))
                    == Err(mpsc::RecvTimeoutError::Timeout)
                {
                    guest.ping().unwrap();
                }
                let mut records = Vec::new();
                for _ in 0..3 {
                    let totals = guest.record().unwrap().totals.unwrap();
                    records.push((totals.wins, totals.losses, totals.revision));
                }
                guest
                    .world(world_request(gc, &ticket, world::Action::Leave))
                    .unwrap();
                guest.close().unwrap();
                records
            });
            let audit = Arc::new(std::sync::Mutex::new(WindowAudit::default()));
            let result = crate::modern_lito::login_window::run_connected_test(
                &profile(peer.endpoint, &shared, &ca),
                Box::new(WindowController {
                    owner,
                    audit: audit.clone(),
                    started: Instant::now(),
                }),
                d.0,
            );
            let _ = stop.send(());
            let records = worker.join();
            let audit = audit.lock().unwrap();
            eprintln!("canonical HWND audit: {:?}", *audit);
            result.unwrap();
            let records = records.unwrap();
            eprintln!("canonical HWND guest record reads: {:?}", records);
            assert!(
                audit.active
                    && audit.moved
                    && audit.cast
                    && audit.saved
                    && audit.returned
                    && audit.closed,
                "interactive window incomplete: {:?}",
                *audit
            );
            assert_eq!(records, vec![(0, 1, 1); 3]);
        });
        let _ = result;
        peer.expect_successful_exit(&d);
        return;
    }
    assert!(initial.data().actors.0.iter().all(|a| !a.loaded));
    assert!(owner
        .command(LoginCommand::World(world::Request::read(sc, CHANNEL)))
        .is_err());
    assert!(owner.command(LoginCommand::LeaveLobby(sc)).is_err());
    let mut runtime = BattleRuntime::new(shared).unwrap();
    let mut keepalive = Instant::now();
    let surface = loop {
        owner.poll();
        let view = owner.snapshot();
        no_legacy(&view);
        match battle_player::update(&mut runtime, &view, false).unwrap() {
            Status::Ready(surface) if battle_player::load_ready(&runtime, &surface, &view) => {
                break surface
            }
            Status::Failed(reason) => panic!("v39 controller resources: {reason}"),
            _ => {}
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            guest.ping().unwrap();
            keepalive = Instant::now();
        }
        d.pause("v39 controller complete shared render");
    };
    assert_eq!(surface.bgra.len(), 800 * 600 * 4);
    owner.poll();
    let before_load = owner.snapshot();
    assert!(
        before_load
            .canonical
            .observation()
            .unwrap()
            .data()
            .actors
            .0
            .iter()
            .all(|a| !a.loaded),
        "polling/rendering cannot submit LOAD"
    );
    assert!(!battle_player::controls_ready(&surface, &before_load));
    assert!(battle_player::loaded_request(&runtime, &surface, &before_load).is_none());
    // A valid resource proof is not transferable to another selected owner,
    // channel, pending departure, or an in-flight LOAD.
    for case in 0..8 {
        let mut wrong = before_load.clone();
        match case {
            0 => wrong.selected_source = Some(sc.key.source_id + 1),
            1 => wrong.navigation.as_mut().unwrap().context.key.object_id += 1,
            2 => wrong.navigation.as_mut().unwrap().context.epoch += 1,
            3 => wrong.navigation.as_mut().unwrap().snapshot.channel_id = Some(CHANNEL ^ 1),
            4 => wrong.phase = LoginPhase::LeavingLobby,
            5 => wrong.battle_supported = false,
            6 => wrong.canonical.hide(),
            _ => wrong.world_pending = true,
        }
        assert!(battle_player::canonical_loaded_command(&runtime, &surface, &wrong).is_none());
        if case != 7 {
            assert!(!battle_player::admitted(&surface, &wrong));
        }
    }
    let load = battle_player::canonical_loaded_command(&runtime, &surface, &before_load).unwrap();
    let loaded = command(&mut owner, &d, "v39 qualified LOAD", load);
    assert!(matches!(
        loaded.canonical.outcome(),
        Some(Outcome::World(world::Status::Ok, world::Reason::None))
    ));
    assert!(battle_player::canonical_loaded_command(&runtime, &surface, &loaded).is_none());
    let ticket = guest
        .world(world::Request {
            match_id: key.match_id as u64,
            generation: key.generation as u64,
            ..world::Request::read(gc, CHANNEL)
        })
        .unwrap()
        .frame
        .unwrap();
    guest
        .world(world::Request {
            manifest_digest: ticket.manifest.digest,
            ..world_request(gc, &ticket, world::Action::Loaded)
        })
        .unwrap();
    let active = wait_owner(&mut owner, &d, "v39 countdown", |v| {
        v.canonical
            .lifecycle()
            .is_some_and(|l| l.phase == world::Phase::Active)
    });
    no_legacy(&active);
    let o = active.canonical.observation().unwrap();
    let surface = rendered(&mut runtime, &active, &mut guest, &d);
    let me = o
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .unwrap();
    let target = o
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity != me.entity)
        .unwrap();
    let target_id = target.entity;
    let from = WorldCell {
        x: me.position.x,
        y: me.position.y,
    };
    let (heading, destination) = DIRECTIONS
        .iter()
        .enumerate()
        .map(|(h, &(x, y))| {
            (
                h as u8,
                WorldCell {
                    x: from.x + x,
                    y: from.y + y,
                },
            )
        })
        .find(|(_, to)| {
            surface
                .navigation
                .as_ref()
                .unwrap()
                .map()
                .can_step(from, *to)
                && o.data()
                    .actors
                    .0
                    .iter()
                    .all(|a| a.position.x != to.x || a.position.y != to.y)
        })
        .unwrap();
    let movement = world::Request {
        action: world::Action::Move,
        match_id: key.match_id as u64,
        generation: key.generation as u64,
        sequence: me.last_move_sequence as u64 + 1,
        from_x: from.x,
        from_y: from.y,
        heading,
        ..world::Request::read(sc, CHANNEL)
    };
    let mut route = crate::modern_lito::battle_route::Route::default();
    route
        .request(
            &surface,
            crate::modern_lito::battle_window::Intent {
                stamp: Stamp::of(&surface.observation),
                kind: crate::modern_lito::battle_window::Kind::Navigate(destination),
            },
            &active,
        )
        .unwrap();
    let step = loop {
        if let Some(q) = route.next(Some(&surface), &active, Instant::now()) {
            break q;
        }
        d.pause("owned canonical route planning");
    };
    submit(&mut owner, &d, "v39 routed move", || {
        LoginCommand::Canonical(step.clone())
    });
    route.submitted(&step).unwrap();
    assert_eq!(route.status(), NavigationStatus::WaitingForAck);
    let moved = wait_owner(&mut owner, &d, "v39 route exact receipt", |v| {
        !v.world_pending
    });
    no_legacy(&moved);
    assert!(route.next(None, &moved, Instant::now()).is_none());
    assert_eq!(route.status(), NavigationStatus::Arrived);
    assert!(route.is_quiescent());
    assert!(matches!(
        moved.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    ));
    let current = moved.canonical.observation().unwrap();
    let moved_actor = current
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == current.data().personal.entity)
        .unwrap();
    assert_eq!(
        (moved_actor.position.x, moved_actor.position.y),
        (destination.x, destination.y)
    );
    assert!(!battle_player::controls_ready(&surface, &moved));
    assert!(
        crate::modern_lito::battle_input::command(
            &surface,
            crate::modern_lito::battle_window::Intent {
                stamp: Stamp::of(&surface.observation),
                kind: crate::modern_lito::battle_window::Kind::Local(
                    crate::modern_lito::battle_window::Local::Step(heading)
                )
            },
            &moved
        )
        .is_err(),
        "old painted pose cannot issue the next step"
    );
    let moved_image = rendered(&mut runtime, &moved, &mut guest, &d);
    assert_eq!(
        moved_image
            .geometry
            .actors
            .iter()
            .find(|a| a.entity_id == moved_actor.entity as u64)
            .unwrap()
            .cell,
        destination
    );
    // An acknowledged move does not waive the server's cross-action cadence.
    // Bound the next *new* cast by confirmed server time, not an arbitrary
    // sleep, a command replay, or a weakened TooFast expectation.
    let cast_after = current
        .data()
        .world
        .active_elapsed_nanos
        .checked_add(moved_actor.step_interval_nanos)
        .unwrap();
    // Explicit stale position is an authoritative denial, not a disconnect or
    // unconfirmed ACK, and must not be replayed with repaired coordinates.
    let denied_move = world::Request {
        sequence: movement.sequence + 1,
        ..movement
    };
    let denied = command(
        &mut owner,
        &d,
        "v39 denied move",
        PlayerCommand::operation(
            Guard {
                stamp: Stamp::of(current),
                target: None,
            },
            Operation::Move(denied_move),
        ),
    );
    assert!(matches!(
        denied.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Denied,
            Reason::StalePosition
        ))
    ));
    let paced = wait_owner(&mut owner, &d, "v39 movement cadence before cast", |v| {
        v.canonical
            .observation()
            .is_some_and(|o| o.data().world.active_elapsed_nanos >= cast_after)
    });
    let current = paced.canonical.observation().unwrap();
    let me = current
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == current.data().personal.entity)
        .unwrap();
    let target = current
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == target_id)
        .unwrap();
    let cast = dis::Request {
        context: sc,
        channel_id: CHANNEL,
        match_id: key.match_id as u64,
        generation: key.generation as u64,
        sequence: me.spells.sequence as u64 + 1,
        caster_life: crate::modern_lito::battle_window::life(me) as u64,
        target_entity_id: target_id as u64,
        target_life: crate::modern_lito::battle_window::life(target) as u64,
    };
    let cast_command = PlayerCommand::operation(
        Guard {
            stamp: Stamp::of(current),
            target: Some(Target::Player {
                entity: target_id,
                life: cast.target_life as i64,
            }),
        },
        Operation::Disintegrate(cast),
    );
    let target_intent = crate::modern_lito::battle_window::Intent {
        stamp: Stamp::of(current),
        kind: crate::modern_lito::battle_window::Kind::Disintegrate(Target::Player {
            entity: target_id,
            life: cast.target_life as i64,
        }),
    };
    assert_eq!(
        crate::modern_lito::battle_input::target_command(target_intent, &paced).unwrap(),
        cast_command
    );
    for case in 0..6 {
        let mut wrong = paced.clone();
        match case {
            0 => wrong.skills = Default::default(),
            1 => {
                let crate::modern_lito::spell_protocol::SkillsState::Ready(reply) =
                    &mut wrong.skills
                else {
                    panic!("qualified catalog")
                };
                Arc::make_mut(reply).context.epoch ^= 1;
            }
            2 => wrong.world_pending = true,
            3 => wrong.selected_source = Some(sc.key.source_id + 1),
            4 => wrong.canonical.hide(),
            _ => wrong.battle_supported = false,
        }
        assert!(crate::modern_lito::battle_input::target_command(target_intent, &wrong).is_err());
    }
    submit(&mut owner, &d, "v39 painted disintegrate kill", || {
        LoginCommand::PaintedTarget(target_intent)
    });
    assert!(owner.snapshot().world_pending);
    let killed = wait_owner(&mut owner, &d, "v39 painted disintegrate receipt", |v| {
        !v.world_pending
    });
    no_legacy(&killed);
    assert!(
        matches!(
            killed.canonical.outcome(),
            Some(Outcome::Operation(
                Rejection::None,
                Disposition::Applied,
                Reason::None
            ))
        ),
        "canonical disintegrate outcome: {:?}",
        killed.canonical.outcome()
    );
    assert_eq!(
        killed.canonical.lifecycle().unwrap().phase,
        world::Phase::Finished
    );
    let killed_data = killed.canonical.observation().unwrap().data();
    assert_eq!(
        killed_data
            .actors
            .0
            .iter()
            .find(|a| a.entity == target_id)
            .unwrap()
            .vital
            .as_ref()
            .unwrap()
            .values
            .hp,
        0
    );
    assert!(killed_data.result.is_some());
    let terminal = rendered(&mut runtime, &killed, &mut guest, &d);
    assert_eq!(terminal.bgra.len(), 800 * 600 * 4);
    assert!(terminal.observation.data().result.is_some());
    assert!(runtime
        .loaded_request(&surface, killed.canonical.observation().unwrap())
        .is_none());
    assert!(owner
        .command(LoginCommand::Canonical(cast_command))
        .is_err());
    let saved = wait_owner(&mut owner, &d, "v39 SQL saved", |v| {
        v.canonical
            .lifecycle()
            .is_some_and(|l| l.settlement == Some(world::SettlementState::Saved))
    });
    no_legacy(&saved);
    let totals = guest.record().unwrap().totals.unwrap();
    assert_eq!((totals.wins, totals.losses, totals.revision), (0, 1, 1));
    assert_eq!(guest.record().unwrap().totals, Some(totals));
    let leave = PlayerCommand::world(battle_world::Request {
        action: battle_world::Action::Leave,
        ..battle_world::Request::read(sc, CHANNEL, Some(&key))
    });
    submit(&mut owner, &d, "v39 leave", || {
        LoginCommand::Canonical(leave.clone())
    });
    assert!(
        owner.snapshot().canonical.observation().is_none(),
        "pending departure hides pixels but is not an ACK"
    );
    assert!(owner.snapshot().world_pending);
    let leaving = owner.snapshot();
    assert!(!battle_player::admitted(&terminal, &leaving));
    battle_player::update(&mut runtime, &leaving, false).unwrap();
    assert!(runtime.observation().is_none());
    let left = wait_owner(&mut owner, &d, "v39 leave ACK", |v| {
        !v.world_pending && !v.canonical.attached()
    });
    no_legacy(&left);
    assert!(!battle_player::admitted(&terminal, &left));
    guest
        .world(world_request(gc, &ticket, world::Action::Leave))
        .unwrap();
    let recorded = wait_owner(&mut owner, &d, "v39 winner record", |v| {
        !v.record_pending
            && v.record
                .totals()
                .is_some_and(|t| (t.wins, t.losses, t.revision) == (1, 0, 1))
    });
    assert_eq!(recorded.record.context(), Some(sc));
    let returned = wait_owner(&mut owner, &d, "v39 ready reset", |v| {
        v.rooms
            .joined()
            .is_some_and(|r| r.summary.ready == 0 && !r.summary.satisfied)
    });
    submit(&mut owner, &d, "v39 leave room", || {
        LoginCommand::Rooms(room_command(
            sc,
            returned.rooms.joined().unwrap(),
            rooms::Action::Leave,
            0,
        ))
    });
    wait_owner(&mut owner, &d, "v39 room departure", |v| {
        !v.room_pending && v.rooms.joined().is_none()
    });
    submit(&mut owner, &d, "v39 lobby return", || {
        LoginCommand::LeaveLobby(sc)
    });
    wait_owner(&mut owner, &d, "v39 channels again", |v| {
        v.phase == LoginPhase::ChoosingChannel
    });
    submit(&mut owner, &d, "v39 reenter", || {
        LoginCommand::EnterChannel {
            context: sc,
            channel_id: CHANNEL,
        }
    });
    wait_owner(&mut owner, &d, "v39 lobby again", |v| {
        v.phase == LoginPhase::InLobby && v.canonical.supported() && !v.canonical.attached()
    });
    submit(&mut owner, &d, "v39 roster return", || {
        LoginCommand::ReturnCharacters(sc)
    });
    let roster = wait_owner(&mut owner, &d, "v39 roster again", |v| {
        v.phase == LoginPhase::CharactersReady
    });
    no_legacy(&roster);
    assert!(!roster.canonical.attached());
    runtime.shutdown();
    while !runtime.is_quiescent() {
        runtime.poll();
        owner.poll();
        d.pause("v39 renderer retire");
    }
    guest.close().unwrap();
    submit(&mut owner, &d, "v39 shutdown", || LoginCommand::Shutdown);
    while !owner.is_quiescent() {
        owner.poll();
        d.pause("v39 TLS retire");
    }
    assert_eq!(owner.snapshot().phase, LoginPhase::Disconnected);
    peer.expect_successful_exit(&d);
}

#[derive(Default, Debug)]
struct WindowAudit {
    active: bool,
    moved: bool,
    cast: bool,
    saved: bool,
    returned: bool,
    closed: bool,
    phase: Option<LoginPhase>,
    receipt: Option<(u64, &'static str, Outcome)>,
}
struct WindowController {
    owner: SessionController,
    audit: Arc<std::sync::Mutex<WindowAudit>>,
    started: Instant,
}
impl LoginController for WindowController {
    fn command(&mut self, q: LoginCommand) -> Result<(), String> {
        let kind = match &q {
            LoginCommand::Canonical(command) => command.audit_kind(),
            LoginCommand::PaintedTarget(_) => "painted-target",
            LoginCommand::PaintedLocal { .. } => "painted-local",
            LoginCommand::Equipment(_) => "source-equipment",
            LoginCommand::Rooms(_) => "rooms",
            LoginCommand::World(_) => "legacy-world",
            LoginCommand::ReadSkills => "read-skills",
            LoginCommand::ReadRecord => "read-record",
            LoginCommand::Shop(_) => "shop",
            LoginCommand::Disconnect => "disconnect",
            LoginCommand::Shutdown => "shutdown",
            _ => "other-login-command",
        };
        let result = self.owner.command(q);
        eprintln!(
            "CANONICAL_HWND_COMMAND ms={} kind={kind} admitted={}",
            self.started.elapsed().as_millis(),
            result.is_ok()
        );
        result
    }
    fn poll(&mut self) {
        self.owner.poll();
    }
    fn is_quiescent(&self) -> bool {
        self.owner.is_quiescent()
    }
    fn snapshot(&self) -> LoginSnapshot {
        let view = self.owner.snapshot();
        if let Err(e) = crate::modern_lito::login_window::validate_snapshot(&view) {
            eprintln!("canonical HWND snapshot rejected: {e:#}");
        }
        no_legacy(&view);
        let mut audit = self.audit.lock().unwrap();
        if audit.phase != Some(view.phase) {
            eprintln!(
                "CANONICAL_HWND_PHASE ms={} phase={:?}",
                self.started.elapsed().as_millis(),
                view.phase
            );
            audit.phase = Some(view.phase);
        }
        let receipt = view.canonical.receipt_audit();
        if receipt != audit.receipt {
            eprintln!(
                "CANONICAL_HWND_RECEIPT ms={} receipt={receipt:?}",
                self.started.elapsed().as_millis()
            );
            audit.receipt = receipt;
        }
        if let Some(o) = view.canonical.observation() {
            audit.active |=
                o.data().world.phase == crate::modern_lito::battle_payload::data::Phase::Active;
            let a = o
                .data()
                .actors
                .0
                .iter()
                .find(|a| a.entity == o.data().personal.entity)
                .unwrap();
            audit.moved |= a.last_move_sequence > 0;
            audit.cast |= a.spells.histories.0.iter().any(|h| {
                h.lane == crate::modern_lito::battle_payload::data::Lane::Disintegrate
                    && h.ledger.casts > 0
            });
        }
        audit.saved |= view
            .canonical
            .lifecycle()
            .is_some_and(|l| l.settlement == Some(world::SettlementState::Saved));
        audit.returned |= audit.active
            && !view.world_pending
            && !view.canonical.attached()
            && view.phase == LoginPhase::InLobby;
        audit.closed |=
            audit.returned && view.phase == LoginPhase::Disconnected && self.owner.is_quiescent();
        view
    }
}
