//! Actual Main/JDBC/GM/TLS + shared554: targeted43 NPC/BOT isolation,
//! death/terminal cleanup, typed SQL settlement and source reload. No HWND.
use super::*;
use crate::modern_lito::{battle_payload::haste as effect, battle_target_haste as haste};

fn cast(cs: &mut [Client; 2], d: &Deadline, target: Target, count: i64) -> effect::Detail {
    wait(cs, d, "target43 current Quest pixels", |cs| {
        cs[0].rendered()
    });
    let old =
        haste::Intent::from_observation(cs[0].view().canonical.observation().unwrap(), target)
            .unwrap();
    let before = own_actor(&cs[0]).vital.unwrap().values.mp;
    let v = input(cs, 0, d, Kind::Haste(target));
    assert_eq!(
        v.canonical.outcome(),
        Some(Outcome::TargetHaste(haste::Reason::None))
    );
    let a = own_actor(&cs[0]);
    let h = a
        .spells
        .histories
        .0
        .iter()
        .find(|h| h.lane == data::Lane::Haste)
        .unwrap();
    assert_eq!(
        (h.ledger.casts, h.ledger.cost, h.ledger.spent),
        (count, 15, count * 15)
    );
    assert_eq!(a.vital.as_ref().unwrap().values.mp, before - 15);
    let r = h.receipt.as_ref().unwrap();
    let data::Detail::Haste(detail) = &r.detail else {
        panic!("43 receipt")
    };
    wait(cs, d, "target43 exact Quest receipt painted", |cs| {
        cs[0].rendered()
            && cs[0].surface.as_ref().is_some_and(|s| {
                s.observation.data().actors.0.iter().any(|a| {
                    a.entity == s.observation.data().personal.entity
                        && a.spells
                            .histories
                            .0
                            .iter()
                            .any(|h| h.lane == data::Lane::Haste && h.receipt.as_ref() == Some(r))
                })
            })
    });
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::target_haste(old)))
        .is_err());
    wait(cs, d, "target43 shared casting recovery", |cs| {
        cs[0].rendered()
            && cs[0]
                .view()
                .canonical
                .observation()
                .unwrap()
                .data()
                .world
                .loading_elapsed_nanos
                >= r.loading_elapsed_nanos + 1_200_000_000
    });
    detail.clone()
}

fn moved_ready(cs: &mut [Client; 2], d: &Deadline) {
    // Finished pixels are not a cooldown. Wait the server-announced MOVE
    // interval from its confirmed receipt before the next cast or attack.
    let next = Instant::now()
        + Duration::from_nanos(own_actor(&cs[0]).step_interval_nanos.try_into().unwrap());
    wait(cs, d, "target43 MOVE recovery and pixels", |cs| {
        cs[0].rendered() && Instant::now() >= next
    });
}
fn effects(c: &Client) -> effect::State {
    c.view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .haste
        .as_ref()
        .unwrap()
        .state
        .clone()
}

#[test]
#[ignore = "v44 Main/owned SQL/GM/shared554: targeted NPC and BOT, two-force isolation, death/terminal cleanup, four waves, saved win/loss, source reload; no HWND"]
fn java_v44_main_sql_target_haste_npc_bot_quest_saved_results_and_return() {
    let d = Deadline(Instant::now() + Duration::from_secs(170));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernHasteQuestSqlTestPeer",
        "target-haste-quest-sql",
        Some("170000"),
    );
    let profile = profile(peer.endpoint, &shared, &private_root(&fixtures));
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new_target_haste_development(profile.clone()),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "target43 Quest TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "target43 Quest sources", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = target_haste_sql::select(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        enter(&mut cs, i, &d, contexts[i]);
        assert!(cs[i].view().canonical.target_haste_supported());
        assert!(cs[i].view().gm_supported);
    }
    create(&mut cs, &d, contexts[0]);
    let room = cs[0].view().rooms.joined().unwrap().summary.clone();
    send(&mut cs, 1, &d, || {
        LoginCommand::Rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: room.room_id,
                    expected_revision: room.revision,
                    ..rooms::Common::read(contexts[1], CHANNEL)
                },
                String::new(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "target43 Quest joined", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    let denied = gm(&mut cs, 1, &d, ".de 20");
    room_status(
        &denied,
        rooms::Status::Denied,
        rooms::Reason::GmNotAuthorized,
    );
    let granted = gm(&mut cs, 0, &d, ".de 20");
    room_status(&granted, rooms::Status::Ok, rooms::Reason::None);
    let bot_ids = balanced(&granted, 2, 2);
    for i in 0..2 {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Ready, 1);
    }
    cs[0].load = true;
    wait(&mut cs, &d, "target43 Quest first LOAD", |cs| {
        cs.iter().all(|c| c.surface.is_some()) && !cs[0].load
    });
    for i in 0..2 {
        assert_eq!(
            cs[i]
                .view()
                .canonical
                .observation()
                .unwrap()
                .data()
                .world
                .phase,
            data::Phase::Loading
        );
        force(&cs[i], i as i32);
    }
    cs[1].load = true;
    wait(&mut cs, &d, "target43 Quest active", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    let world = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .identity
        .clone();
    wait(&mut cs, &d, "target43 first Quest wave born", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .npcs
                    .as_ref()
                    .unwrap()
                    .actors
                    .0
                    .iter()
                    .any(|n| n.hp > 0)
        })
    });
    let npcs = cs.each_ref().map(|c| {
        c.view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .npcs
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .find(|n| n.hp > 0)
            .unwrap()
            .clone()
    });
    assert_ne!(npcs[0].serial, npcs[1].serial);
    assert_eq!(
        npcs[0].cell, npcs[1].cell,
        "same coordinates must not merge isolated force NPCs"
    );
    let targets = cs.each_ref().map(|c| {
        let v = c.view();
        let data = v.canonical.observation().unwrap().data();
        let a = data
            .actors
            .0
            .iter()
            .find(|a| {
                data.room.members.0[(a.entity - 1) as usize].identity.kind
                    == data::MemberKind::ServerBot
            })
            .unwrap();
        assert_eq!(
            data.room.members.0[(a.entity - 1) as usize].source_name.0,
            "太陽神"
        );
        Target::Player {
            entity: a.entity,
            life: a.resources.as_ref().unwrap().life + 1,
        }
    });
    for foreign in [
        Target::Npc {
            serial: npcs[1].serial,
            life: npcs[1].life,
        },
        targets[1],
    ] {
        assert!(haste::Intent::from_observation(
            cs[0].view().canonical.observation().unwrap(),
            foreign
        )
        .is_none());
        // ready() asks whether the skill can enter target-selection mode;
        // the actual painted/selected-target adapters must reject this target.
        let v = cs[0].view();
        let surface = cs[0].surface.as_ref().unwrap();
        let invalid = Intent {
            stamp: Stamp::of(&surface.observation),
            kind: Kind::Haste(foreign),
        };
        assert!(battle_input::command(surface, invalid, &v).is_err());
        assert!(battle_input::target_command(invalid, &v).is_err());
    }
    let own_npc = Target::Npc {
        serial: npcs[0].serial,
        life: npcs[0].life,
    };
    let first = cast(&mut cs, &d, own_npc, 1);
    let effect::StatusTarget::Npc(n) = &first.target else {
        panic!("NPC receipt")
    };
    assert_eq!(
        (n.id.world.clone(), n.id.serial, n.life, n.lane),
        (world.clone(), npcs[0].serial, npcs[0].life, 0)
    );
    assert_eq!(
        n.after.haste_until_millis - n.after.observed_millis,
        1_200_000
    );
    let state = effects(&cs[0]);
    let motion = state
        .npcs
        .as_ref()
        .unwrap()
        .actors
        .0
        .iter()
        .find(|a| a.id.serial == n.id.serial)
        .unwrap();
    assert_eq!((motion.walk_millis, motion.attack_millis), (500, 1000));
    assert_eq!(motion.speed.haste_until_millis, n.after.haste_until_millis);
    assert_eq!(motion.speed.revision, n.after.revision);
    assert!(effects(&cs[1]).targets.0.iter().all(|e| e.active.is_none()));
    assert_eq!(
        own_actor(&cs[0]).buffs.as_ref().unwrap().haste_until_nanos,
        0
    );

    let second = cast(&mut cs, &d, targets[0], 2);
    let effect::StatusTarget::Player(bot) = &second.target else {
        panic!("BOT receipt")
    };
    assert_eq!(bot.participant.identity.kind, data::MemberKind::ServerBot);
    assert_eq!(bot.participant.identity.source, 7);
    assert!(bot_ids.contains(&(bot.participant.identity.id as u64)));
    assert_eq!(
        bot.after.haste_until_nanos - second.active_elapsed_nanos,
        1_200_000_000_000
    );
    let state = effects(&cs[0]);
    assert!(
        state
            .targets
            .0
            .iter()
            .any(|e| e.target == effect::Target::Npc(n.id.clone()) && e.active.is_some()),
        "casting on a BOT must not erase the earlier NPC grant"
    );
    assert!(state.targets.0.iter().any(|e| e.target
        == effect::Target::Player(bot.participant.clone())
        && e.active.is_some()));
    force(&cs[0], 0);
    force(&cs[1], 1);
    assert!(effects(&cs[1]).targets.0.iter().all(|e| e.active.is_none()));

    // Move through the painted-input adapter after casting, then return to the
    // original objective approach cell after both forces' waves are cleared.
    let origin = own_actor(&cs[0]).position;
    let from = WorldCell {
        x: origin.x,
        y: origin.y,
    };
    let surface = cs[0].surface.as_ref().unwrap();
    let (heading, to) = DIRECTIONS
        .iter()
        .enumerate()
        .find_map(|(h, (dx, dy))| {
            let to = WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            };
            (surface.navigation.as_ref().unwrap().can_step(from, to)
                // A free cell now can be a scheduled wave/objective birth.
                // Keep this round-trip off those declared Quest cells.
                && (to.x != npcs[0].cell.x || to.y != npcs[0].cell.y)
                && surface.observation.data().quest.as_ref().unwrap().targets.0.iter()
                    .all(|target| target.cell.x != to.x || target.cell.y != to.y)
                && surface
                    .observation
                    .data()
                    .actors
                    .0
                    .iter()
                    .all(|a| a.position.x != to.x || a.position.y != to.y)
                && surface
                    .observation
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
        .unwrap();
    let moved = input(&mut cs, 0, &d, Kind::Local(Local::Step(heading)));
    assert_eq!(
        moved.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    assert_eq!(own_actor(&cs[0]).position, data::Cell { x: to.x, y: to.y });
    moved_ready(&mut cs, &d);
    let dead_target =
        haste::Intent::from_observation(cs[0].view().canonical.observation().unwrap(), own_npc)
            .unwrap();
    for i in 0..2 {
        waves(&mut cs, i, &d);
    }
    let state = effects(&cs[0]);
    assert!(
        state
            .targets
            .0
            .iter()
            .filter(|e| e.target == effect::Target::Npc(n.id.clone()))
            .all(|e| e.active.is_none()),
        "NPC death retires the active grant"
    );
    assert!(haste::Intent::from_observation(
        cs[0].view().canonical.observation().unwrap(),
        own_npc
    )
    .is_none());
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::target_haste(dead_target)))
        .is_err());
    assert!(
        state.targets.0.iter().any(
            |e| e.target == effect::Target::Player(bot.participant.clone()) && e.active.is_some()
        ),
        "living BOT keeps its own grant after NPC death"
    );
    let back = DIRECTIONS
        .iter()
        .position(|(dx, dy)| to.x + dx == origin.x && to.y + dy == origin.y)
        .unwrap() as u8;
    let moved = input(&mut cs, 0, &d, Kind::Local(Local::Step(back)));
    assert_eq!(
        moved.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    assert_eq!(own_actor(&cs[0]).position, origin);
    moved_ready(&mut cs, &d);
    let mut next = Instant::now();
    loop {
        for c in &mut cs {
            c.pump();
        }
        let v = cs[0].view();
        let o = v.canonical.observation().unwrap();
        if o.data().world.phase == data::Phase::Finished {
            break;
        }
        if cs[0].rendered() && !v.world_pending && Instant::now() >= next {
            let t = &o.data().quest.as_ref().unwrap().objectives.0[0];
            let result = input(
                &mut cs,
                0,
                &d,
                Kind::Attack(Target::Quest {
                    slot: t.id.slot,
                    born: t.born_revision,
                }),
            );
            assert_eq!(
                result.canonical.outcome(),
                Some(Outcome::Operation(
                    Rejection::None,
                    Disposition::Applied,
                    Reason::None
                ))
            );
            next = Instant::now() + Duration::from_millis(750);
        }
        d.pause("target43 Quest objective finish");
    }
    wait(
        &mut cs,
        &d,
        "target43 Quest saved result and terminal pixels",
        |cs| {
            cs.iter().all(|c| {
                c.rendered()
                    && c.view().canonical.lifecycle().is_some_and(|l| {
                        l.phase == world::Phase::Finished
                            && l.settlement == Some(world::SettlementState::Saved)
                    })
            })
        },
    );
    for c in &cs {
        let state = effects(c);
        assert!(
            state.targets.0.iter().all(|e| e.active.is_none()),
            "terminal removes active43"
        );
        assert!(state
            .npcs
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .all(|m| m.speed.haste_until_millis == 0));
        assert!(c
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .actors
            .0
            .iter()
            .all(|a| a.buffs.as_ref().unwrap().haste_until_nanos == 0));
        let v = c.view();
        let result = v
            .canonical
            .observation()
            .unwrap()
            .data()
            .result
            .as_ref()
            .unwrap();
        assert_eq!(result.participants.0.len(), 4);
        assert_eq!(
            result
                .participants
                .0
                .iter()
                .filter(|p| p.identity.kind == data::MemberKind::ServerBot)
                .map(|p| p.identity.id as u64)
                .collect::<BTreeSet<_>>(),
            bot_ids
        );
    }
    for i in 0..2 {
        leave_world(&mut cs, i, &d, contexts[i], &world);
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: i64::from(i == 0),
                losses: i64::from(i == 1),
                revision: 1
            })
        );
        target_haste_sql::saved(&cs[i], contexts[i]);
    }
    let cleared = gm(&mut cs, 0, &d, ".de 0");
    room_status(&cleared, rooms::Status::Ok, rooms::Reason::None);
    for i in (0..2).rev() {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "target43 Quest roster return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    let moon = target_haste_sql::select(&mut cs, 0, &d, 42);
    enter(&mut cs, 0, &d, moon);
    create(&mut cs, &d, moon);
    let denied = gm(&mut cs, 0, &d, ".de 20");
    room_status(
        &denied,
        rooms::Status::Denied,
        rooms::Reason::GmNotAuthorized,
    );
    assert_eq!(denied.record.totals().unwrap().losses, 1);
    room_action(&mut cs, 0, &d, moon, rooms::Action::Leave, 0);
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "target43 Quest owned shutdown", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
