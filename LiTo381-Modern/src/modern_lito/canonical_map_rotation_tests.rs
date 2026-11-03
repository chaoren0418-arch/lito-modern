//! The existing eleven shared arenas, now through actual v42 controllers.
//! Three saved rounds per pair; same owners and runtimes, no legacy frames.
//! Qualified synthetic sources/RNG, real shared art/geometry and isolated SQL.
use super::*;

macro_rules! rotation {
    ($name:ident, $mode:literal, $maps:expr) => {
        #[test]
        #[ignore = "two v42 controllers, three real shared-map rounds, isolated SQL; no native UI"]
        fn $name() {
            run($mode, $maps);
        }
    };
}
rotation!(
    java_v42_shared_map_rotation_509_514_509,
    "rotation",
    [509, 514, 509]
);
rotation!(
    java_v42_shared_map_rotation_552_551_552,
    "rotation-shared",
    [552, 551, 552]
);
rotation!(
    java_v42_shared_map_rotation_531_508_531,
    "rotation-shared-508-531",
    [531, 508, 531]
);
rotation!(
    java_v42_shared_map_rotation_534_531_534,
    "rotation-shared-531-534",
    [534, 531, 534]
);
rotation!(
    java_v42_shared_map_rotation_516_531_516,
    "rotation-shared-516-531",
    [516, 531, 516]
);
rotation!(
    java_v42_shared_map_rotation_504_531_504,
    "rotation-shared-504-531",
    [504, 531, 504]
);
rotation!(
    java_v42_shared_map_rotation_505_531_505,
    "rotation-shared-505-531",
    [505, 531, 505]
);
rotation!(
    java_v42_shared_map_rotation_506_531_506,
    "rotation-shared-506-531",
    [506, 531, 506]
);

macro_rules! ordinary_rotation {
    ($name:ident, $mode:literal, $maps:expr) => {
        #[test]
        #[ignore = "two ordinary v47 controllers, three real shared-map rounds, isolated SQL; no native UI"]
        fn $name() {
            run_contract($mode, $maps, true);
        }
    };
}
ordinary_rotation!(java_v47_shared_map_rotation_509_514_509, "rotation", [509, 514, 509]);
ordinary_rotation!(java_v47_shared_map_rotation_552_551_552, "rotation-shared", [552, 551, 552]);
ordinary_rotation!(java_v47_shared_map_rotation_531_508_531, "rotation-shared-508-531", [531, 508, 531]);
ordinary_rotation!(java_v47_shared_map_rotation_534_531_534, "rotation-shared-531-534", [534, 531, 534]);
ordinary_rotation!(java_v47_shared_map_rotation_516_531_516, "rotation-shared-516-531", [516, 531, 516]);
ordinary_rotation!(java_v47_shared_map_rotation_504_531_504, "rotation-shared-504-531", [504, 531, 504]);
ordinary_rotation!(java_v47_shared_map_rotation_505_531_505, "rotation-shared-505-531", [505, 531, 505]);
ordinary_rotation!(java_v47_shared_map_rotation_506_531_506, "rotation-shared-506-531", [506, 531, 506]);

fn anchor(map: u32) -> WorldCell {
    let (x, y) = match map {
        504 | 505 | 506 => (32933, 32862),
        508 => (32832, 33151),
        509 => (32740, 32796),
        514 => (32724, 32784),
        516 => (33123, 32772),
        531 => (32799, 32797),
        534 => (32817, 32832),
        551 => (32729, 32790),
        552 => (32736, 32858),
        _ => panic!("unqualified rotation arena"),
    };
    WorldCell { x, y }
}
fn draft(cs: &mut [Client; 2], d: &Deadline, context: SelectionContext, map: u32, create: bool) {
    let v = cs[0].view();
    let public = rooms::PublicDraft {
        capacity: 2,
        mode: 1,
        attributes: 16,
        map_id: map,
        name: "Canonical shared map rotation".into(),
    };
    let rooms::State::Ready(directory) = &v.rooms else {
        panic!("rotation room directory");
    };
    let option = directory.mode_option(1, map).unwrap();
    assert!(option.world_available && option.minimum_players <= 2 && option.maximum_players >= 2);
    let common = if create {
        rooms::Common {
            action: rooms::Action::Create,
            ..rooms::Common::read(context, CHANNEL)
        }
    } else {
        let r = v.rooms.joined().unwrap();
        rooms::Common {
            action: rooms::Action::Configure,
            room_id: r.summary.room_id,
            expected_revision: r.summary.revision,
            ..rooms::Common::read(context, CHANNEL)
        }
    };
    send(cs, 0, d, || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                common,
                rooms::Draft::new(public.clone(), String::new()).unwrap(),
            )
            .unwrap(),
        )
    });
    wait(cs, d, "rotation room draft", |cs| {
        !cs[0].view().room_pending
    });
    assert_eq!(
        cs[0].view().rooms.joined().unwrap().summary.settings.map_id,
        map
    );
    assert_eq!(cs[0].view().rooms.joined().unwrap().summary.ready, 0);
}
fn refresh_room(cs: &mut [Client; 2], i: usize, d: &Deadline, context: SelectionContext) {
    send(cs, i, d, || {
        LoginCommand::Rooms(rooms::Request::read(context, CHANNEL).unwrap())
    });
    wait(cs, d, "rotation room read", |cs| !cs[i].view().room_pending);
}
#[track_caller]
fn applied(v: &LoginSnapshot) {
    assert_eq!(
        v.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
}
fn round(
    cs: &mut [Client; 2],
    d: &Deadline,
    contexts: [SelectionContext; 2],
    map: u32,
    teleport: bool,
    number: i64,
    previous: &mut Option<(Arc<Surface>, Intent, Command)>,
    hashes: &mut BTreeMap<u32, [u8; 32]>,
    results: &mut BTreeSet<[u8; 16]>,
) {
    for i in 0..2 {
        room_action(cs, i, d, contexts[i], rooms::Action::Ready, 1);
        cs[i].load = true;
    }
    wait(cs, d, "rotation both admitted active pictures", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view()
                    .canonical
                    .lifecycle()
                    .is_some_and(|l| l.phase == world::Phase::Active)
        })
    });
    let owner = cs[0].surface.clone().unwrap();
    let o = &owner.observation;
    assert_eq!(o.data().manifest.map, map as i32);
    let id = o.data().world.identity.clone();
    let nav = owner.navigation.as_ref().unwrap();
    assert_eq!(nav.map().map_id(), map);
    if let Some(old) = hashes.insert(map, nav.map().canonical_digest()) {
        assert_eq!(old, nav.map().canonical_digest());
    }
    assert_eq!(own_actor(&cs[0]).position.x, anchor(map).x);
    assert_eq!(own_actor(&cs[0]).position.y, anchor(map).y);
    for c in cs.iter() {
        let a = own_actor(c);
        let v = a.vital.unwrap().values;
        assert_eq!(v.hp, v.maximum_hp);
        assert_eq!(v.mp, v.maximum_mp);
        assert_eq!(a.spells.sequence, 0);
        assert_eq!(a.last_move_sequence, 0);
        battle_window::validate(c.surface.as_ref().unwrap()).unwrap();
    }
    if let Some((old, intent, command)) = previous.take() {
        assert!(id.match_id > old.observation.data().world.identity.match_id);
        assert!(id.generation > old.observation.data().world.identity.generation);
        assert_eq!(id.room, old.observation.data().world.identity.room);
        assert_ne!(
            old.observation.data().manifest.digest,
            o.data().manifest.digest
        );
        assert!(!battle_player::admitted(&old, &cs[0].view()));
        assert!(battle_input::command(&old, intent, &cs[0].view()).is_err());
        assert!(cs[0]
            .controller
            .command(LoginCommand::Canonical(command))
            .is_err());
    }
    let target = Target::Player {
        entity: 2,
        life: battle_window::life(&own_actor(&cs[1])),
    };
    let intent = Intent {
        stamp: Stamp::of(o),
        kind: Kind::LightArrow(target),
    };
    let command = battle_input::command(&owner, intent, &cs[0].view()).unwrap();
    *previous = Some((owner, intent, command));
    if teleport {
        let start = anchor(map);
        let old = cs[0].surface.clone().unwrap();
        let before_revision = cs[0].view().canonical.observation().unwrap().data().world.revision;
        let v = input(cs, 0, d, Kind::Local(Local::Teleport));
        applied(&v);
        let landed = own_actor(&cs[0]);
        let teleport_spell = landed
            .spells
            .histories
            .0
            .iter()
            .find(|s| s.lane == data::Lane::Teleport)
            .unwrap();
        assert_eq!(
            (teleport_spell.ledger.casts, teleport_spell.ledger.spent),
            (1, 10)
        );
        let recovered = teleport_spell
            .receipt
            .as_ref()
            .unwrap()
            .loading_elapsed_nanos
            + landed.cast_interval_nanos;
        let receipt = teleport_spell.receipt.as_ref().unwrap();
        let receipt_diagnostic = format!("ROTATION_TELEPORT_RECEIPT map={} before_world_revision={} reply_world_revision={} usage_revision={} receipt_loading_nanos={} reply_loading_nanos={}",
            map, before_revision, v.canonical.observation().unwrap().data().world.revision,
            receipt.impact.usage.revision, receipt.loading_elapsed_nanos,
            v.canonical.observation().unwrap().data().world.loading_elapsed_nanos);
        assert_ne!((landed.position.x, landed.position.y), (start.x, start.y));
        wait(cs, d, "rotation teleported pixels", |cs| {
            cs[0].rendered()
                && cs[0]
                    .surface
                    .as_ref()
                    .is_some_and(|s| s.observation.data().actors.0[0].position == landed.position)
        });
        assert!(!battle_player::controls_ready(&old, &cs[0].view()));
        wait(cs, d, "rotation confirmed teleport cadence", |cs| {
            cs[0].rendered()
                && cs[0]
                    .view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .loading_elapsed_nanos
                    >= recovered
        });
        eprintln!("{receipt_diagnostic}");
        crate::modern_lito::battle_teleport::dump_timing_diagnostics();
        cs[0].audio.dump_diagnostics();
        assert!(
            cs[0].audio.sequence(
                Stamp::of(cs[0].view().canonical.observation().unwrap()),
                119
            ) > 0
        );
        let from = WorldCell {
            x: landed.position.x,
            y: landed.position.y,
        };
        assert!(cs[0]
            .surface
            .as_ref()
            .unwrap()
            .navigation
            .as_ref()
            .unwrap()
            .can_step(from, start));
        let h = DIRECTIONS
            .iter()
            .position(|(dx, dy)| from.x + dx == start.x && from.y + dy == start.y)
            .unwrap() as u8;
        let v = input(cs, 0, d, Kind::Local(Local::Step(h)));
        applied(&v);
        wait(cs, d, "rotation post-teleport move", |cs| {
            cs[0].rendered()
                && own_actor(&cs[0]).position.x == start.x
                && own_actor(&cs[0]).position.y == start.y
        });
        assert_eq!(
            own_actor(&cs[0]).last_move_sequence,
            landed.last_move_sequence + 1
        );
        let moved = cs[0].view();
        let recovered = moved
            .canonical
            .observation()
            .unwrap()
            .data()
            .world
            .loading_elapsed_nanos
            + own_actor(&cs[0]).step_interval_nanos;
        wait(cs, d, "rotation original move-to-spell cadence", |cs| {
            cs[0].rendered()
                && cs[0]
                    .view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .loading_elapsed_nanos
                    >= recovered
        });
    }
    let mut casts = 0;
    loop {
        let before = own_actor(&cs[0]);
        if cs[0].view().canonical.lifecycle().unwrap().phase == world::Phase::Finished {
            break;
        }
        let v = input(cs, 0, d, Kind::LightArrow(target));
        applied(&v);
        casts += 1;
        assert!(casts <= 8);
        let a = own_actor(&cs[0]);
        assert_eq!(a.spells.sequence, before.spells.sequence + 1);
        let s = a
            .spells
            .histories
            .0
            .iter()
            .find(|s| s.lane == data::Lane::LightArrow)
            .unwrap();
        let receipt = s.receipt.as_ref().unwrap();
        assert_eq!(
            a.vital.as_ref().unwrap().values.mp,
            before.vital.unwrap().values.mp - i64::from(s.ledger.cost)
        );
        let next = receipt.loading_elapsed_nanos + a.cast_interval_nanos;
        wait(cs, d, "rotation acknowledged spell cadence", |cs| {
            let v = cs[0].view();
            let o = v.canonical.observation().unwrap();
            o.data().world.phase == data::Phase::Finished
                || (cs[0].rendered() && o.data().world.loading_elapsed_nanos >= next)
        });
    }
    wait(cs, d, "rotation both saved rendered results", |cs| {
        cs.iter().all(|c| {
            c.view().canonical.lifecycle().is_some_and(|l| {
                l.phase == world::Phase::Finished
                    && l.settlement == Some(world::SettlementState::Saved)
            }) && c.surface.as_ref().is_some_and(|s| {
                s.observation.data().result.is_some()
                    && s.observation.data().world.phase == data::Phase::Finished
            })
        })
    });
    let result = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .result
        .clone()
        .unwrap();
    assert_eq!((result.mode, result.map), (1, map as i32));
    assert!(results.insert(result.result_id));
    assert_eq!(
        cs[1]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .result
            .as_ref(),
        Some(&result)
    );
    assert_eq!(own_actor(&cs[1]).vital.unwrap().values.hp, 0);
    for i in 0..2 {
        // The player must return first; the record button cannot overtake world ownership.
        assert!(cs[i].controller.command(LoginCommand::ReadRecord).is_err());
        let leave = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&id))
        });
        send(cs, i, d, || LoginCommand::Canonical(leave.clone()));
        wait(cs, d, "rotation return and retire old renderer", |cs| {
            !cs[i].view().world_pending
                && !cs[i].view().canonical.attached()
                && cs[i].runtime.is_quiescent()
        });
        assert!(cs[i].surface.is_none());
        refresh_room(cs, i, d, contexts[i]);
        assert_eq!(cs[i].view().rooms.joined().unwrap().summary.ready, 0);
        send(cs, i, d, || LoginCommand::ReadRecord);
        wait(cs, d, "rotation durable totals", |cs| {
            !cs[i].view().record_pending
        });
        assert_eq!(cs[i].view().record.context(), Some(contexts[i]));
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: if i == 0 { number } else { 0 },
                losses: if i == 1 { number } else { 0 },
                revision: number
            })
        );
        if i == 0 {
            assert_eq!(
                cs[1]
                    .view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .result
                    .as_ref(),
                Some(&result)
            );
        }
    }
}
fn run(mode: &'static str, maps: [u32; 3]) {
    run_contract(mode, maps, false);
}
fn run_contract(mode: &'static str, maps: [u32; 3], ordinary_v47: bool) {
    let d = Deadline(Instant::now() + Duration::from_secs(150));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        if ordinary_v47 {
            "local.lito381.server.ModernOrdinaryMapRotationTestPeer"
        } else {
            "local.lito381.server.ModernCanonicalMapRotationTestPeer"
        },
        mode,
        Some("160000"),
    );
    assert!(peer.process_job.is_some());
    let mut cs = std::array::from_fn(|_| Client {
        controller: if ordinary_v47 {
            SessionController::new(configured_modern_profile(peer.endpoint, &shared, &ca))
        } else {
            SessionController::new_canonical_development(profile(peer.endpoint, &shared, &ca))
        },
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "rotation TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "rotation sources", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        send(&mut cs, i, &d, || {
            LoginCommand::SelectSource(if i == 0 { 7 } else { 42 })
        });
        wait(&mut cs, &d, "rotation roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
        let v = cs[i].view();
        let context = SelectionContext {
            key: CharacterKey {
                source_id: v.selected_source.unwrap(),
                object_id: 3333,
            },
            epoch: v.roster_epoch,
        };
        send(&mut cs, i, &d, || LoginCommand::SelectCharacter(context));
        wait(&mut cs, &d, "rotation source catalogs", |cs| {
            let v = cs[i].view();
            v.phase == LoginPhase::ChoosingChannel
                && !v.record_pending
                && !v.spell_pending
                && matches!(v.equipment, equipment::State::Ready(_))
        });
        send(&mut cs, i, &d, || LoginCommand::EnterChannel {
            context,
            channel_id: CHANNEL,
        });
        wait(&mut cs, &d, "rotation lobby", |cs| {
            cs[i].view().phase == LoginPhase::InLobby && !cs[i].view().room_pending
        });
        assert!(cs[i].view().canonical.equipment_supported());
    }
    let contexts = cs.each_ref().map(|c| c.view().navigation.unwrap().context);
    draft(&mut cs, &d, contexts[0], maps[0], true);
    let r = cs[0].view().rooms.joined().unwrap().clone();
    let room_id = r.summary.room_id;
    let join = rooms::Common {
        action: rooms::Action::Join,
        room_id,
        expected_revision: r.summary.revision,
        ..rooms::Common::read(contexts[1], CHANNEL)
    };
    send(&mut cs, 1, &d, || {
        LoginCommand::Rooms(rooms::Request::join(join, String::new()).unwrap())
    });
    wait(&mut cs, &d, "rotation shared room", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    let mut old = None;
    let mut hashes = BTreeMap::new();
    let mut results = BTreeSet::new();
    for (n, map) in maps.into_iter().enumerate() {
        if n > 0 {
            refresh_room(&mut cs, 0, &d, contexts[0]);
            draft(&mut cs, &d, contexts[0], map, false);
        }
        assert_eq!(
            cs[0].view().rooms.joined().unwrap().summary.room_id,
            room_id
        );
        round(
            &mut cs,
            &d,
            contexts,
            map,
            mode != "rotation",
            n as i64 + 1,
            &mut old,
            &mut hashes,
            &mut results,
        );
        let contract = if ordinary_v47 { "ordinary v47" } else { "canonical" };
        eprintln!("{contract} rotation shared map {map}: round {} saved, same room {room_id}, both owners returned",n+1);
    }
    assert_eq!(results.len(), 3);
    assert_eq!(hashes.len(), 2);
    for i in (0..2).rev() {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        send(&mut cs, i, &d, || LoginCommand::LeaveLobby(contexts[i]));
        wait(&mut cs, &d, "rotation channels", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingChannel
        });
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "rotation refreshed roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
        assert!(cs[i].surface.is_none());
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    loop {
        for c in &mut cs {
            c.controller.poll();
            c.runtime.poll();
        }
        if cs
            .iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
        {
            break;
        }
        d.pause("rotation shutdown");
    }
    peer.expect_successful_exit(&d);
}
