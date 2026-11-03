//! One production Main, two ordinary schema2/v47 controllers, seven rooms.
//! Real SQL and shared pixels; distinct withdrawal and completed-mode scenarios.
//! Shared mappings are explicit alternatives, not native geometry or HWND acceptance.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scenario {
    Withdraw,
    Pvp,
    PvpQuest,
    PvpObjectives,
    AllModes,
}

fn room(cs: &mut [Client; 2], d: &Deadline, contexts: [SelectionContext; 2], mode: u8, map: u32) {
    send(cs, 0, d, || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(contexts[0], CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: if mode == 3 { 30 } else { 2 },
                        mode,
                        map_id: map,
                        attributes: 0,
                        name: "Same Main mode transition".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(cs, d, "mixed room creator", |cs| !cs[0].view().room_pending);
    let r = cs[0]
        .view()
        .rooms
        .joined()
        .expect("creator joined mixed room")
        .summary
        .clone();
    assert_eq!((r.settings.mode, r.settings.map_id), (mode, map));
    send(cs, 1, d, || {
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
    wait(cs, d, "mixed both room owners", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    if mode == 3 {
        let v = bots::gm(cs, 0, d, ".de 28");
        let members = &v.rooms.joined().unwrap().members;
        assert_eq!(members.len(), 30);
        for team in [0, 1] {
            assert_eq!(members.iter().filter(|m| m.team == team).count(), 15);
        }
    } else if mode != 0 && mode != 4 {
        room_action(cs, 1, d, contexts[1], rooms::Action::Team, 1);
    }
    for i in 0..2 {
        room_action(cs, i, d, contexts[i], rooms::Action::Ready, 1);
        cs[i].load = true;
    }
    wait(cs, d, "mixed both qualified active pictures", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view()
                    .canonical
                    .lifecycle()
                    .is_some_and(|l| l.phase == world::Phase::Active)
        })
    });
}

#[test]
#[ignore = "one Main/owned SQL, same ordinary v47 TLS owners: Slow DM/race/Honor/Siege/Time/Quest/DM; no native UI"]
fn java_v47_main_sql_same_connections_six_modes_retire_and_reload() {
    flow(Scenario::Withdraw);
}

#[test]
#[ignore = "one Main/owned SQL, same v47 owners: alternating DM winners across all six modes, exact result totals and source reload; no native UI"]
fn java_v47_main_sql_same_connections_alternating_winners_and_mode_results() {
    flow(Scenario::Pvp);
}

#[test]
#[ignore = "one Main/owned SQL, same v47 owners: DM victory, six modes, both Quest lanes/four waves/objective/saved result, opposite DM victory and source reload; no native UI"]
fn java_v47_main_sql_same_connections_pvp_quest_victories_and_source_reload() {
    flow(Scenario::PvpQuest);
}

#[test]
#[ignore = "one Main/owned SQL, same v47 owners: DM, Honor tower, cooperative tower/two waves, Quest/four waves, opposite DM victories and exact source SQL; no native UI"]
fn java_v47_main_sql_same_connections_pvp_honor_time_quest_results() {
    flow(Scenario::PvpObjectives);
}

#[test]
#[ignore = "one Main/owned SQL, same ordinary v47 owners: all six mapped modes reach saved completion, ranked race separate from combat, 30-person Siege, seven rooms and source reload; no HWND"]
fn java_v47_main_sql_same_connections_all_six_modes_saved_and_source_reload() {
    flow(Scenario::AllModes);
}

fn duel(cs: &mut [Client; 2], d: &Deadline, winner: usize) -> [u8; 16] {
    let loser = 1 - winner;
    let target_actor = own_actor(&cs[loser]);
    let target = Target::Player {
        entity: target_actor.entity,
        life: battle_window::life(&target_actor),
    };
    assert_eq!(target_actor.vital.as_ref().unwrap().values.hp, 1000);
    let mut operations = 0;
    loop {
        let v = cs[winner].view();
        let data = v.canonical.observation().unwrap().data();
        if data.world.phase == data::Phase::Finished {
            break;
        }
        assert_eq!(data.world.phase, data::Phase::Active);
        assert!(operations < 128, "bounded actual combat, not forced result");
        let actor = own_actor(&cs[winner]);
        let recovery = crate::modern_lito::battle_motion::paced_nanos(
            &actor,
            if winner == 0 {
                actor.cast_interval_nanos
            } else {
                actor.vital.as_ref().unwrap().attack_interval_nanos
            } as u64,
            winner != 0,
        );
        let next = data.world.loading_elapsed_nanos + recovery as i64;
        wait(cs, d, "mixed real attack/cast recovery", |cs| {
            cs[winner].rendered()
                && cs[winner]
                    .view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .loading_elapsed_nanos
                    >= next
        });
        let before = own_actor(&cs[winner]);
        let kind = if winner == 0 {
            assert!(battle_input::skill_available(4, &cs[winner].view()));
            Kind::LightArrow(target)
        } else {
            assert!((before.position.x - target_actor.position.x).abs() <= 1);
            assert!((before.position.y - target_actor.position.y).abs() <= 1);
            Kind::Attack(target)
        };
        let reply = input(cs, winner, d, kind);
        assert_eq!(
            reply.canonical.outcome(),
            Some(Outcome::Operation(
                Rejection::None,
                Disposition::Applied,
                Reason::None
            ))
        );
        operations += 1;
        if winner == 0 {
            assert_eq!(
                own_actor(&cs[winner]).spells.sequence,
                before.spells.sequence + 1
            );
        }
    }
    let result = saved_result(cs, d, 1, 514, [winner == 0, winner == 1]);
    assert_eq!(own_actor(&cs[loser]).vital.unwrap().values.hp, 0);
    assert!(own_actor(&cs[winner]).vital.unwrap().values.hp > 0);
    eprintln!(
        "MIXED_VICTORY winnerSource={} operations={operations} bothSaved=true loserHp=0",
        if winner == 0 { 7 } else { 42 }
    );
    result
}

fn quest(cs: &mut [Client; 2], d: &Deadline) -> [u8; 16] {
    for i in 0..2 {
        waves(cs, i, d, false);
    }
    let moon = cs[1]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .quest
        .as_ref()
        .unwrap()
        .objectives
        .0[0]
        .clone();
    assert_eq!(moon.hp, 1);
    let foreign = Intent {
        stamp: Stamp::of(cs[0].view().canonical.observation().unwrap()),
        kind: Kind::Attack(Target::Quest {
            slot: moon.id.slot,
            born: moon.born_revision,
        }),
    };
    assert!(battle_input::target_command(foreign, &cs[0].view()).is_err());
    let actor = own_actor(&cs[0]);
    let recovery = crate::modern_lito::battle_motion::paced_nanos(
        &actor,
        actor.vital.as_ref().unwrap().attack_interval_nanos as u64,
        true,
    );
    let next = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .loading_elapsed_nanos
        + recovery as i64;
    wait(cs, d, "mixed Quest actual attack recovery", |cs| {
        cs[0].rendered()
            && !cs[0].view().world_pending
            && cs[0]
                .view()
                .canonical
                .observation()
                .unwrap()
                .data()
                .world
                .loading_elapsed_nanos
                >= next
    });
    let sun = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .quest
        .as_ref()
        .unwrap()
        .objectives
        .0[0]
        .clone();
    assert_eq!(sun.hp, 1);
    assert_ne!(sun.id.slot, moon.id.slot);
    let reply = input(
        cs,
        0,
        d,
        Kind::Attack(Target::Quest {
            slot: sun.id.slot,
            born: sun.born_revision,
        }),
    );
    assert_eq!(
        reply.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    let result = saved_result(cs, d, 2, 554, [true, false]);
    assert_eq!(
        cs[0]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .quest
            .as_ref()
            .unwrap()
            .objectives
            .0[0]
            .hp,
        0
    );
    assert_eq!(
        cs[1]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .quest
            .as_ref()
            .unwrap()
            .objectives
            .0[0],
        moon
    );
    eprintln!("MIXED_QUEST_VICTORY lanes=2 stagesPerLane=4 winnerSource=7 bothSaved=true foreignLaneUnchanged=true");
    result
}

fn cast_recovery(cs: &mut [Client; 2], d: &Deadline, i: usize) {
    let actor = own_actor(&cs[i]);
    let recovery = crate::modern_lito::battle_motion::paced_nanos(
        &actor,
        actor.cast_interval_nanos as u64,
        false,
    );
    let next = cs[i]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .loading_elapsed_nanos
        + recovery as i64;
    wait(cs, d, "mixed objective cast recovery", |cs| {
        cs[i].rendered()
            && !cs[i].view().world_pending
            && cs[i]
                .view()
                .canonical
                .observation()
                .unwrap()
                .data()
                .world
                .loading_elapsed_nanos
                >= next
    });
}

fn honor(cs: &mut [Client; 2], d: &Deadline) -> [u8; 16] {
    let initial = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .honor
        .clone();
    let own = initial
        .objectives
        .0
        .iter()
        .find(|t| t.team == 0)
        .unwrap()
        .clone();
    let target = initial
        .objectives
        .0
        .iter()
        .find(|t| t.team == 1)
        .unwrap()
        .clone();
    assert_eq!(target.hp, 3000);
    let mut casts = 0;
    loop {
        if cs[0]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .world
            .phase
            == data::Phase::Finished
        {
            break;
        }
        assert!(casts < 64, "bounded actual Honor combat");
        cast_recovery(cs, d, 0);
        let before = own_actor(&cs[0]);
        let v = input(
            cs,
            0,
            d,
            Kind::LightArrow(Target::Honor {
                entity: target.entity,
            }),
        );
        assert_eq!(
            v.canonical.outcome(),
            Some(Outcome::Operation(
                Rejection::None,
                Disposition::Applied,
                Reason::None
            ))
        );
        assert_eq!(
            own_actor(&cs[0]).spells.sequence,
            before.spells.sequence + 1
        );
        casts += 1;
    }
    let result = saved_result(cs, d, 5, 532, [true, false]);
    for c in cs.iter() {
        let v = c.view();
        let towers = &v.canonical.observation().unwrap().data().honor.objectives.0;
        assert_eq!(
            towers
                .iter()
                .find(|t| t.entity == target.entity)
                .unwrap()
                .hp,
            0
        );
        assert_eq!(towers.iter().find(|t| t.entity == own.entity), Some(&own));
    }
    eprintln!("MIXED_HONOR_VICTORY winnerSource=7 casts={casts} targetInitialHp=3000 ownTowerUnchanged=true bothSaved=true");
    result
}

fn time_state(c: &Client) -> data::TimeAttack {
    c.view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .time_attack
        .clone()
        .unwrap()
}

fn cooperative(cs: &mut [Client; 2], d: &Deadline) -> [u8; 16] {
    for c in cs.iter() {
        let t = time_state(c);
        assert_eq!(t.phase, data::TimePhase::WaitingForTower);
        assert_eq!(t.duration_millis, 20000);
        assert_eq!(
            t.waves.0,
            vec![data::TimeKind::Ordinary, data::TimeKind::FinalBosses]
        );
        assert_eq!(t.tower_hp, 1);
        assert!(t.departed.0.is_empty());
        assert!(battle_window::hud(&c.surface.as_ref().unwrap().observation)
            .join("\n")
            .contains("擊破啟動塔後開始"));
    }
    let v = input(cs, 0, d, Kind::Attack(Target::TimeTower));
    assert_eq!(
        v.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    wait(
        cs,
        d,
        "mixed cooperative tower destroyed in both pictures",
        |cs| {
            cs.iter().all(|c| {
                c.surface.as_ref().is_some_and(|s| {
                    s.observation.data().time_attack.as_ref().unwrap().tower_hp == 0
                })
            })
        },
    );
    for i in 0..2 {
        cast_recovery(cs, d, i);
        wait(cs, d, "mixed cooperative next wave painted", |cs| {
            assert_eq!(
                cs[i]
                    .view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .phase,
                data::Phase::Active
            );
            cs[i].rendered()
                && time_state(&cs[i]).wave == i as i32
                && cs[i]
                    .surface
                    .as_ref()
                    .unwrap()
                    .geometry
                    .npcs
                    .iter()
                    .any(|n| n.alive)
        });
        let n = cs[i]
            .surface
            .as_ref()
            .unwrap()
            .geometry
            .npcs
            .iter()
            .find(|n| n.alive)
            .unwrap();
        let target = Target::Npc {
            serial: n.id.serial as i64,
            life: n.life as i64,
        };
        let before = own_actor(&cs[i]);
        let v = input(cs, i, d, Kind::LightArrow(target));
        assert_eq!(
            v.canonical.outcome(),
            Some(Outcome::Operation(
                Rejection::None,
                Disposition::Applied,
                Reason::None
            ))
        );
        let after = own_actor(&cs[i]);
        assert_eq!(after.spells.sequence, before.spells.sequence + 1);
        let h = after
            .spells
            .histories
            .0
            .iter()
            .find(|h| h.lane == data::Lane::NpcLightArrow)
            .unwrap();
        assert_eq!(h.ledger.casts, 1);
        assert_eq!(h.ledger.spent, i64::from(h.ledger.cost));
        assert!(!battle_window::target_current(
            v.canonical.observation().unwrap(),
            target
        ));
    }
    let result = saved_result(cs, d, 4, 540, [true, true]);
    for c in cs.iter() {
        let t = time_state(c);
        assert_eq!(
            (t.phase, t.end),
            (data::TimePhase::Finished, data::TimeEnd::Cleared)
        );
        assert!(t.departed.0.is_empty());
        // TimeAttack.living contains NPC IDs, not player entities. A clear
        // must leave no live enemies; surviving players have separate vitals.
        assert!(t.living.0.is_empty());
        let v = c.view();
        let actors = &v.canonical.observation().unwrap().data().actors.0;
        assert_eq!(actors.len(), 2);
        assert!(actors
            .iter()
            .all(|a| a.vital.as_ref().unwrap().values.hp > 0));
        assert_eq!(t.tower_hp, 0);
        let hud = battle_window::hud(&c.surface.as_ref().unwrap().observation).join("\n");
        assert!(hud.contains("全部波次完成") && hud.contains("留場存活 2 · 離隊 0"));
    }
    eprintln!("MIXED_TIME_VICTORY waves=2 activePlayers=2 winners=7,42 bothSaved=true");
    result
}

fn saved_result(
    cs: &mut [Client; 2],
    d: &Deadline,
    mode: i32,
    map: i32,
    winners: [bool; 2],
) -> [u8; 16] {
    wait(cs, d, "mixed both saved rendered victory results", |cs| {
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
    assert_eq!(
        (result.mode, result.map, result.participants.0.len()),
        (mode, map, 2)
    );
    for i in 0..2 {
        let view = cs[i].view();
        assert_eq!(
            view.canonical.observation().unwrap().data().result.as_ref(),
            Some(&result)
        );
        assert_eq!(
            cs[i]
                .surface
                .as_ref()
                .unwrap()
                .observation
                .data()
                .result
                .as_ref(),
            Some(&result)
        );
        battle_window::validate(cs[i].surface.as_ref().unwrap()).unwrap();
        let p = result
            .participants
            .0
            .iter()
            .find(|p| p.identity.source == if i == 0 { 7 } else { 42 })
            .unwrap();
        assert_eq!(p.identity.id, 3333);
        assert_eq!(p.team, if mode == 4 { 0 } else { i as i32 });
        assert_eq!(
            p.verdict,
            if winners[i] {
                data::Verdict::Win
            } else {
                data::Verdict::Loss
            }
        );
        assert!(cs[i].controller.command(LoginCommand::ReadRecord).is_err());
    }
    result.result_id
}

fn flow(scenario: Scenario) {
    let expired_vip = match std::env::var_os("LITO_MODERN_TEST_EXPIRED_VIP") {
        None => false,
        Some(value) => {
            assert!(value == "true" && scenario == Scenario::AllModes, "invalid explicit VIP fixture");
            true
        }
    };
    let results = scenario != Scenario::Withdraw;
    let all_modes = scenario == Scenario::AllModes;
    let objectives = matches!(scenario, Scenario::PvpObjectives | Scenario::AllModes);
    let d = Deadline(
        Instant::now()
            + Duration::from_secs(if all_modes {
                600
            } else if results {
                400
            } else {
                280
            }),
    );
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernMixedModesSqlTestPeer",
        match scenario {
            Scenario::Withdraw => "mixed-modes-sql",
            Scenario::Pvp => "mixed-results-sql",
            Scenario::PvpQuest => "mixed-quest-results-sql",
            Scenario::PvpObjectives => "mixed-objectives-sql",
            Scenario::AllModes => "mixed-all-results-sql",
        },
        Some(if all_modes {
            "610000"
        } else if results {
            "410000"
        } else {
            "290000"
        }),
    );
    assert!(peer.process_job.is_some());
    let profile = configured_modern_profile(peer.endpoint, &shared, &private_root(&fixtures));
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new(profile.clone()),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "mixed TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "mixed login", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = slow_sql::select_with_expired_vip(&mut cs, i, &d, if i == 0 { 7 } else { 42 }, expired_vip);
        enter(&mut cs, i, &d, contexts[i]);
        let v = cs[i].view();
        let rooms::State::Ready(r) = &v.rooms else {
            panic!("mixed directory")
        };
        for (mode, map) in [(0, 565), (1, 514), (2, 554), (3, 547), (4, 540), (5, 532)] {
            assert!(r.mode_option(mode, map).unwrap().world_available);
        }
        assert!(v.canonical.slow_supported()); // Listener advertises the union, not each world's policy.
    }
    let mut previous: Option<(Arc<Surface>, Intent, Command)> = None;
    let mut room_ids = BTreeSet::new();
    let mut cooperative_defeats = 0;
    let mut wins = [0; 2];
    let mut losses = [0; 2];
    let mut result_ids = BTreeSet::new();
    let mut duels = 0;
    for (mode, logical, physical) in [
        (1, 514, 514),
        (0, 565, 5143),
        (5, 532, 532),
        (3, 547, 4),
        (4, 540, 554),
        (2, 554, 554),
        (1, 514, 514),
    ] {
        // A spent material must not survive round retirement. Every purchase is real SQL-qualified shop traffic.
        if mode == 1 {
            for i in 0..2 {
                slow_sql::gems(&mut cs, i, &d, contexts[i]);
            }
        }
        room(&mut cs, &d, contexts, mode, logical);
        let current = cs[0].surface.clone().unwrap();
        let id = current.observation.data().world.identity.clone();
        assert!(room_ids.insert(id.room));
        for c in &cs {
            let v = c.view();
            let s = c.surface.as_ref().unwrap();
            let b = s.observation.data();
            assert_eq!(
                (b.room.mode, b.room.map, b.manifest.map),
                (i32::from(mode), logical as i32, physical)
            );
            assert_eq!(
                s.navigation.as_ref().unwrap().map().map_id(),
                physical as u32
            );
            assert_eq!(b.race.is_some(), mode == 0);
            assert_eq!(b.siege.is_some(), mode == 3);
            assert_eq!(b.time_attack.is_some(), mode == 4);
            assert_eq!(b.quest.is_some(), mode == 2);
            assert_eq!(!b.honor.objectives.0.is_empty(), mode == 5);
            assert_eq!(b.speed.is_some(), mode == 1 || mode == 2);
            assert_eq!(b.personal.materials.is_some(), mode == 1 || mode == 2);
            assert_eq!(b.personal.supplies.is_some(), mode != 0 && mode != 3);
            assert!(b.result.is_none());
            assert_eq!(own_actor(c).spells.sequence, 0);
            assert_eq!(own_actor(c).last_move_sequence, 0);
            if mode == 1 {
                let fresh = own_actor(c).vital.unwrap().values;
                assert_eq!((fresh.hp, fresh.mp), (1000, 300));
            }
            battle_window::validate(s).unwrap();
            assert!(
                !login_window::can_open_shop(&v),
                "active world cannot open lobby shop"
            );
            if mode != 1 && mode != 2 {
                assert!(!battle_input::skill_available(29, &v));
            }
            if mode == 2 {
                assert!(b.personal.materials.as_ref().unwrap().stocks.0.is_empty());
            }
        }
        if let Some((old, intent, command)) = previous.take() {
            assert_ne!(id, old.observation.data().world.identity);
            assert!(!battle_player::admitted(&old, &cs[0].view()));
            assert!(battle_input::command(&old, intent, &cs[0].view()).is_err());
            assert!(cs[0]
                .controller
                .command(LoginCommand::Canonical(command))
                .is_err());
        }
        // Keep an exact original rendered operation, then prove it cannot cross the next world boundary.
        let (intent, command) = (0..8)
            .find_map(|heading| {
                let intent = Intent {
                    stamp: Stamp::of(&current.observation),
                    kind: Kind::Local(Local::Step(heading)),
                };
                battle_input::command(&current, intent, &cs[0].view())
                    .ok()
                    .map(|q| (intent, q))
            })
            .expect("at least one qualified adjacent movement intent");
        previous = Some((current, intent, command));
        if mode == 1 {
            let a = own_actor(&cs[0]);
            let target = Target::Player {
                entity: a.entity,
                life: battle_window::life(&a),
            };
            let v = input(&mut cs, 0, &d, Kind::Slow(target));
            assert_eq!(
                v.canonical.outcome(),
                Some(Outcome::Slow(crate::modern_lito::battle_slow::Reason::None))
            );
            let after = own_actor(&cs[0]);
            assert_eq!(after.vital.unwrap().values.mp, 285);
            assert_eq!(after.spells.sequence, 1);
            assert_eq!(
                v.canonical
                    .observation()
                    .unwrap()
                    .data()
                    .personal
                    .materials
                    .as_ref()
                    .unwrap()
                    .stocks
                    .0[0]
                    .remaining,
                3
            );
            if results {
                let winner = duels;
                assert!(winner < 2);
                assert!(result_ids.insert(duel(&mut cs, &d, winner)));
                duels += 1;
                wins[winner] += 1;
                losses[1 - winner] += 1;
            }
        }
        if mode == 0 && all_modes {
            assert!(result_ids.insert(super::race::finish_current(&mut cs, &d, contexts)));
            // Ranked racing must never change battle WIN/LOSS totals.
        }
        if mode == 3 && all_modes {
            let (result, attacker) = super::siege::finish_current(&mut cs, &d);
            assert!(result_ids.insert(result));
            wins[attacker] += 1;
            losses[1 - attacker] += 1;
            let v = cs[attacker].view();
            let saved = v
                .canonical
                .observation()
                .unwrap()
                .data()
                .result
                .as_ref()
                .unwrap();
            for p in &saved.participants.0 {
                let won = p.team == saved.siege.as_ref().unwrap().winner;
                assert_eq!(
                    p.verdict,
                    if won {
                        data::Verdict::Win
                    } else {
                        data::Verdict::Loss
                    }
                );
                if p.identity.kind == data::MemberKind::Human {
                    assert_eq!(p.identity.id, 3333);
                    assert_eq!(p.identity.source, if p.team == 0 { 7 } else { 42 });
                }
            }
        }
        if mode == 5 && objectives {
            assert!(result_ids.insert(honor(&mut cs, &d)));
            wins[0] += 1;
            losses[1] += 1;
        }
        if mode == 4 && objectives {
            assert!(result_ids.insert(cooperative(&mut cs, &d)));
            for total in &mut wins {
                *total += 1;
            }
        }
        if mode == 2
            && matches!(
                scenario,
                Scenario::PvpQuest | Scenario::PvpObjectives | Scenario::AllModes
            )
        {
            assert!(result_ids.insert(quest(&mut cs, &d)));
            wins[0] += 1;
            losses[1] += 1;
        }
        for i in 0..2 {
            let q = Command::world(battle_world::Request {
                action: battle_world::Action::Leave,
                ..battle_world::Request::read(contexts[i], CHANNEL, Some(&id))
            });
            send(&mut cs, i, &d, || LoginCommand::Canonical(q.clone()));
            wait(&mut cs, &d, "mixed old renderer retired", |cs| {
                !cs[i].view().world_pending
                    && !cs[i].view().canonical.attached()
                    && cs[i].runtime.is_quiescent()
            });
            assert!(cs[i].surface.is_none());
            assert!(!battle_input::skill_available(29, &cs[i].view()));
        }
        for i in (0..2).rev() {
            room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        }
        if mode == 4 && !objectives {
            cooperative_defeats += 1;
        }
        for i in 0..2 {
            let expected = record::Totals {
                stored: cooperative_defeats + wins[i] + losses[i] > 0,
                wins: wins[i],
                losses: cooperative_defeats + losses[i],
                revision: cooperative_defeats + wins[i] + losses[i],
            };
            // Read-only polling waits for the asynchronous durable journal; never repeat a gameplay mutation.
            loop {
                send(&mut cs, i, &d, || LoginCommand::ReadRecord);
                wait(&mut cs, &d, "mixed durable totals", |cs| {
                    !cs[i].view().record_pending
                });
                if cs[i].view().record.totals() == Some(expected) {
                    break;
                }
                d.pause("mixed cooperative departure saved");
            }
            slow_sql::saved_with_expired_vip(&cs[i], contexts[i], expired_vip);
        }
        eprintln!("MIXED_MODES mode={mode} logical={logical} physical={physical} room={} loaded=true retired=true sameConnections=true", id.room);
    }
    assert_eq!(room_ids.len(), 7);
    assert_eq!(duels, if results { 2 } else { 0 });
    assert_eq!(
        result_ids.len(),
        match scenario {
            Scenario::Withdraw => 0,
            Scenario::Pvp => 2,
            Scenario::PvpQuest => 3,
            Scenario::PvpObjectives => 5,
            Scenario::AllModes => 7,
        }
    );
    for i in (0..2).rev() {
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "mixed return roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    let moon = slow_sql::select_with_expired_vip(&mut cs, 0, &d, 42, expired_vip);
    assert_ne!(moon, contexts[0]);
    // Selection publishes source data before its scheduled read-only record
    // refresh completes. Wait for that exact context, without issuing a retry
    // or treating !record_pending/NotRequested as a completed SQL lookup.
    wait(
        &mut cs,
        &d,
        "mixed new-source automatic record refresh",
        |cs| {
            let v = cs[0].view();
            !v.record_pending && v.record.context() == Some(moon) && v.record.totals().is_some()
        },
    );
    assert_eq!(
        cs[0].view().record.totals(),
        Some(record::Totals {
            stored: true,
            wins: wins[1],
            losses: losses[1] + cooperative_defeats,
            revision: wins[1] + losses[1] + cooperative_defeats,
        })
    );
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(previous.unwrap().2))
        .is_err());
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "mixed normal shutdown", |cs| {
        cs.iter()
            .all(|c| c.runtime.is_quiescent() && c.controller.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
