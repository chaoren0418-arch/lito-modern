//! Pure v41 teleport through the real rendered-intent path. A changed camera,
//! renewed navigation proof and successful subsequent MOVE prove recovery;
//! accepting CAST alone does not prove that the player is no longer stuck.
use super::*;

fn saved(c: &Client, context: SelectionContext) {
    let v = c.view();
    let CharacterDataState::Ready(s) = &v.character_data else {
        panic!("missing SQL character")
    };
    assert_eq!(s.context, context);
    assert_eq!(s.stats.max_hp, 1000);
    assert_eq!(
        s.stats.max_mp,
        if context.key.source_id == 7 { 30 } else { 40 }
    );
    assert_eq!(
        s.stats.food225,
        if context.key.source_id == 7 { 225 } else { 180 }
    );
    assert_eq!(s.items.len(), 2);
    assert_eq!(
        s.items[1].quantity,
        if context.key.source_id == 7 { 2 } else { 5 }
    );
    assert_eq!(gear_snapshot(c).equipped(1001), Some(true));
}

fn lane(c: &Client) -> data::Spell {
    own_actor(c)
        .spells
        .histories
        .0
        .into_iter()
        .find(|s| s.lane == data::Lane::Teleport)
        .expect("negotiated teleport lane")
}

fn free_step(c: &Client) -> (u8, WorldCell) {
    let s = c.surface.as_ref().unwrap();
    let a = own_actor(c);
    let from = WorldCell {
        x: a.position.x,
        y: a.position.y,
    };
    DIRECTIONS
        .iter()
        .enumerate()
        .find_map(|(h, (dx, dy))| {
            let to = WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            };
            (s.navigation.as_ref().unwrap().can_step(from, to)
                && s.observation
                    .data()
                    .actors
                    .0
                    .iter()
                    .all(|a| a.position.x != to.x || a.position.y != to.y)
                && s.observation
                    .data()
                    .npcs
                    .as_ref()
                    .unwrap()
                    .actors
                    .0
                    .iter()
                    .all(|n| n.hp == 0 || n.cell.x != to.x || n.cell.y != to.y))
            .then_some((h as u8, to))
        })
        .expect("qualified teleport destination must retain a free step")
}

fn cast_and_move(cs: &mut [Client; 2], i: usize, d: &Deadline, count: i64) {
    wait(cs, d, "v41 teleport current pixels", |cs| cs[i].rendered());
    let old = cs[i].surface.clone().unwrap();
    let origin = own_actor(&cs[i]);
    let (old_heading, _) = free_step(&cs[i]);
    let stale_intent = Intent {
        stamp: Stamp::of(&old.observation),
        kind: Kind::Local(Local::Step(old_heading)),
    };
    assert!(battle_input::command(&old, stale_intent, &cs[i].view()).is_ok());
    assert!(battle_input::skill_available(5, &cs[i].view()));
    let v = input(cs, i, d, Kind::Local(Local::Teleport));
    assert_eq!(
        v.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        )),
        "teleport entity={} cast={count}",
        origin.entity
    );
    let confirmed = own_actor(&cs[i]);
    let cast = lane(&cs[i]);
    assert_eq!(cast.ledger.casts, count);
    assert_eq!(cast.ledger.cost, 10);
    assert_eq!(cast.ledger.spent, count * 10);
    assert_eq!(confirmed.spells.sequence, count);
    let r = cast.receipt.as_ref().unwrap();
    assert_eq!(r.impact.usage.before.cell, origin.position);
    assert_eq!(r.impact.usage.before.life, Stamp::of(&old.observation).life);
    assert_eq!(
        r.impact.usage.mp_after,
        r.impact.usage.before.vitals.mp - 10
    );
    let data::Detail::Teleport { destination } = &r.detail else {
        panic!("teleport receipt")
    };
    assert_eq!(&confirmed.position, destination);
    assert_ne!(confirmed.position, origin.position);
    assert!(!battle_player::controls_ready(&old, &v));
    assert!(
        battle_input::command(&old, stale_intent, &v).is_err(),
        "pre-teleport terrain cannot authorize a new move"
    );
    let destination = confirmed.position.clone();
    let sound = std::cell::Cell::new(false);
    wait(
        cs,
        d,
        "v41 teleport shared render and navigation renewal",
        |cs| {
            if let Some(s) = &cs[i].surface {
                sound.set(
                    sound.get()
                        || s.audio.as_ref().is_some_and(|audio| {
                            audio.events.iter().any(|e| {
                                // Mixed cues retain their own qualified bank;
                                // the frame default may be the body-sound bank.
                                e.group.candidates().contains(&119)
                                    && e.bank.as_ref().unwrap_or(&audio.bank).get(119).is_some()
                            })
                        }),
                );
            }
            cs[i].rendered()
                && cs[i].surface.as_ref().is_some_and(|s| {
                    s.observation.data().actors.0.iter().any(|a| {
                        a.entity == confirmed.entity
                            && a.position == destination
                            && a.spells.sequence == count
                    })
                })
        },
    );
    let s = cs[i].surface.as_ref().unwrap();
    battle_window::validate(s).unwrap();
    let at = WorldCell {
        x: destination.x,
        y: destination.y,
    };
    assert!(s.navigation.as_ref().unwrap().map().standing(at));
    let p = s
        .geometry
        .actors
        .iter()
        .find(|p| p.entity_id == confirmed.entity as u64)
        .unwrap();
    assert_eq!(s.geometry.projection.pixel_to_cell(p.anchor), Some(at));
    assert_eq!(p.anchor.x, s.width as i32 / 2);
    assert_eq!(p.anchor.y, s.height as i32 / 2);
    assert!(
        sound.get(),
        "confirmed teleport must reach the shared sound119 event bank"
    );
    // Keep both workers active while the authoritative 1200 ms cast cadence
    // elapses. This is not a synchronous sleep or a command resend.
    let resume = Instant::now() + Duration::from_millis(1250);
    wait(cs, d, "v41 teleport cast cadence", |_| {
        Instant::now() >= resume
    });
    let before_move = own_actor(&cs[i]);
    let (heading, to) = free_step(&cs[i]);
    let moved = input(cs, i, d, Kind::Local(Local::Step(heading)));
    // A completed render is not a cooldown. Optimized rendering can finish
    // before this server-announced movement interval; don't let slow debug
    // pixels accidentally pace the next test CAST. Never resend a denial.
    let next_action = Instant::now()
        + Duration::from_nanos(own_actor(&cs[i]).step_interval_nanos.try_into().unwrap());
    assert_eq!(
        moved.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    assert_eq!(own_actor(&cs[i]).position, data::Cell { x: to.x, y: to.y });
    assert_eq!(
        own_actor(&cs[i]).last_move_sequence,
        before_move.last_move_sequence + 1
    );
    assert_eq!(lane(&cs[i]).ledger.casts, count);
    wait(cs, d, "v41 movement after teleport rendered", |cs| {
        cs[i].rendered() && Instant::now() >= next_action
    });
    battle_window::validate(cs[i].surface.as_ref().unwrap()).unwrap();
}

#[test]
#[ignore = "two pure v41 controllers, Main/owned SQL/shared554: repeated teleport, stale-pixel refusal, renewed movement, abort/return; no HWND"]
fn java_v41_sql_teleport_shared_camera_input_recovery_and_abort_return() {
    let d = Deadline(Instant::now() + Duration::from_secs(150));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernQuestSqlTestPeer",
        "quest-supplied-teleport-sql",
    );
    let profile = profile(peer.endpoint, &shared, &private_root(&fixtures));
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new_canonical_development(profile.clone()),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "v41 teleport TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "v41 teleport sources", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        let source_id = if i == 0 { 7 } else { 42 };
        send(&mut cs, i, &d, || LoginCommand::SelectSource(source_id));
        wait(&mut cs, &d, "v41 teleport roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
        contexts[i] = SelectionContext {
            key: CharacterKey {
                source_id,
                object_id: 3333,
            },
            epoch: cs[i].view().roster_epoch,
        };
        send(&mut cs, i, &d, || {
            LoginCommand::SelectCharacter(contexts[i])
        });
        wait(&mut cs, &d, "v41 teleport source data", |cs| {
            let v = cs[i].view();
            v.phase == LoginPhase::ChoosingChannel
                && !v.record_pending
                && !v.spell_pending
                && matches!(v.equipment, equipment::State::Ready(_))
        });
        saved(&cs[i], contexts[i]);
        assert!(cs[i].view().skills.teleport().is_some());
        enter(&mut cs, i, &d, contexts[i]);
    }
    send(&mut cs, 0, &d, || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(contexts[0], CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 2,
                        attributes: 16,
                        map_id: 554,
                        name: "Canonical SQL teleport".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "v41 teleport create", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
    let r = cs[0].view().rooms.joined().unwrap().summary.clone();
    send(&mut cs, 1, &d, || {
        LoginCommand::Rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: r.room_id,
                    expected_revision: r.revision,
                    ..rooms::Common::read(contexts[1], CHANNEL)
                },
                String::new(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "v41 teleport join", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    for i in 0..2 {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Ready, 1);
        cs[i].load = true;
    }
    wait(&mut cs, &d, "v41 teleport active render", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    for i in 0..2 {
        for count in 1..=2 {
            cast_and_move(&mut cs, i, &d, count);
        }
    }
    let world = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .identity
        .clone();
    for i in 0..2 {
        let q = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&world))
        });
        send(&mut cs, i, &d, || LoginCommand::Canonical(q.clone()));
        wait(&mut cs, &d, "v41 teleport abort return", |cs| {
            !cs[i].view().world_pending
                && !cs[i].view().canonical.attached()
                && cs[i].runtime.is_quiescent()
        });
        send(&mut cs, i, &d, || LoginCommand::ReadRecord);
        wait(&mut cs, &d, "v41 teleport no false win", |cs| {
            !cs[i].view().record_pending
        });
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: false,
                wins: 0,
                losses: 0,
                revision: 0
            })
        );
        saved(&cs[i], contexts[i]);
    }
    for i in (0..2).rev() {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "v41 teleport roster return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "v41 teleport normal shutdown", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
