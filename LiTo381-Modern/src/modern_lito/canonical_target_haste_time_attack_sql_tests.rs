//! Production Main/owned SQL/shared554, same v45 development controller and
//! painted skill-page intent as the client. No native540 or HWND claim.
use super::*;
use crate::modern_lito::{battle_payload::haste as effect, battle_target_haste as haste};

fn state(c: &Client) -> data::TimeAttack {
    c.view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .time_attack
        .clone()
        .unwrap()
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
fn caster(c: &Client) -> data::Spell {
    own_actor(c)
        .spells
        .histories
        .0
        .into_iter()
        .find(|s| s.lane == data::Lane::Haste)
        .unwrap()
}
fn player(c: &Client) -> Target {
    let a = own_actor(c);
    Target::Player {
        entity: a.entity,
        life: a.resources.unwrap().life + 1,
    }
}
fn pause(cs: &mut [Client; 2], d: &Deadline, ms: u64) {
    let until = Instant::now() + Duration::from_millis(ms);
    wait(cs, d, "cooperative43 action cadence", |_| {
        Instant::now() >= until
    });
}
fn haste(cs: &mut [Client; 2], d: &Deadline, target: Target, count: i64) {
    wait(cs, d, "cooperative43 current painted input", |cs| {
        cs[0].rendered()
    });
    let old =
        haste::Intent::from_observation(cs[0].view().canonical.observation().unwrap(), target)
            .unwrap();
    // A queued click can be retired by the worker's final world-revision
    // check. Only a confirmed NotSent permits a NEW painted click here;
    // never retry an operation sent to the server or an unknown outcome.
    let mut not_sent = 0;
    loop {
        let before = own_actor(&cs[0]).spells;
        let v = input(cs, 0, d, Kind::Haste(target));
        if v.canonical.outcome() == Some(Outcome::NotSent) {
            assert_eq!(
                own_actor(&cs[0]).spells,
                before,
                "NotSent cannot change the sequence, MP spending or cast receipt"
            );
            not_sent += 1;
            assert!(not_sent <= 3, "painted haste input repeatedly retired");
            eprintln!(
                "COOPERATIVE43_NOT_SENT fresh_click={not_sent} casts={}",
                before
                    .histories
                    .0
                    .iter()
                    .find(|s| s.lane == data::Lane::Haste)
                    .unwrap()
                    .ledger
                    .casts
            );
            wait(
                cs,
                d,
                "cooperative43 repaint after confirmed non-send",
                |cs| cs[0].rendered(),
            );
            continue;
        }
        assert_eq!(
            v.canonical.outcome(),
            Some(Outcome::TargetHaste(haste::Reason::None))
        );
        break;
    }
    let h = caster(&cs[0]);
    assert_eq!(
        (h.ledger.casts, h.ledger.cost, h.ledger.spent),
        (count, 15, count * 15)
    );
    let receipt = h.receipt.as_ref().unwrap();
    assert!(matches!(receipt.detail, data::Detail::Haste(_)));
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::target_haste(old)))
        .is_err());
    wait(
        cs,
        d,
        "cooperative43 recovery and authoritative pixels",
        |cs| {
            cs[0].rendered()
                && cs[0]
                    .view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .loading_elapsed_nanos
                    >= receipt.loading_elapsed_nanos + 1_200_000_000
        },
    );
}
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
fn target(cs: &mut [Client; 2], i: usize, d: &Deadline, wave: i32) -> Target {
    wait(cs, d, "cooperative43 current live schema5 target", |cs| {
        cs[i].rendered()
            && state(&cs[i]).wave == wave
            && cs[i]
                .surface
                .as_ref()
                .unwrap()
                .geometry
                .npcs
                .iter()
                .any(|n| n.alive)
    });
    let motion = effects(&cs[i]).npcs.unwrap();
    let view = cs[i].view();
    let raw = view.canonical.observation().unwrap().data();
    assert!(raw.npc_motion.is_none());
    assert_eq!(
        motion.actors.0.len(),
        raw.npcs.as_ref().unwrap().actors.0.len()
    );
    let n = cs[i]
        .surface
        .as_ref()
        .unwrap()
        .geometry
        .npcs
        .iter()
        .find(|n| n.alive)
        .unwrap();
    Target::Npc {
        serial: n.id.serial as i64,
        life: n.life as i64,
    }
}
fn light_arrow(cs: &mut [Client; 2], i: usize, d: &Deadline, wave: i32) {
    let target = target(cs, i, d, wave);
    let v = input(cs, i, d, Kind::LightArrow(target));
    applied(&v);
    let a = own_actor(&cs[i]);
    let h = a
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
#[test]
#[ignore = "v45 Main/owned SQL/shared554: targeted43 PC/NPC, leader departure, source-owned results and return; no HWND"]
fn java_v45_main_sql_time_attack_haste_departure_clear_and_return() {
    flow(false);
}
#[test]
#[ignore = "v45 Main/owned SQL/shared554: targeted43 PC/NPC, real deadline clears effects, two losses saved and return; no HWND"]
fn java_v45_main_sql_time_attack_haste_timeout_saved_and_return() {
    flow(true);
}

fn flow(timeout: bool) {
    let d = Deadline(Instant::now() + Duration::from_secs(170));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernHasteTimeAttackSqlTestPeer",
        if timeout {
            "target-haste-timeout-sql"
        } else {
            "target-haste-time-sql"
        },
        Some("170000"),
    );
    let p = profile(peer.endpoint, &shared, &private_root(&fixtures));
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new_cooperative_haste_development(p.clone()),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "cooperative43 TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "cooperative43 source choice", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = target_haste_sql::select(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        enter(&mut cs, i, &d, contexts[i]);
        assert!(cs[i].view().canonical.target_haste_supported());
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
                        capacity: 4,
                        mode: 4,
                        map_id: 540,
                        attributes: 0,
                        name: "Cooperative43 SQL".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "cooperative43 creator membership", |cs| {
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
    wait(&mut cs, &d, "cooperative43 joined membership", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    assert!(cs[0]
        .view()
        .rooms
        .joined()
        .unwrap()
        .members
        .iter()
        .all(|m| m.team == 0));
    for i in 0..2 {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Ready, 1);
    }
    cs[0].load = true;
    wait(&mut cs, &d, "cooperative43 first shared LOAD", |cs| {
        !cs[0].load && cs.iter().all(|c| c.surface.is_some())
    });
    for c in &cs {
        let v = c.view();
        let raw = v.canonical.observation().unwrap().data();
        assert_eq!(raw.world.phase, data::Phase::Loading);
        assert_eq!(
            (raw.room.mode, raw.room.map, raw.manifest.map),
            (4, 540, 554)
        );
        assert!(raw.npc_motion.is_none());
        assert!(raw
            .haste
            .as_ref()
            .unwrap()
            .state
            .npcs
            .as_ref()
            .unwrap()
            .actors
            .0
            .is_empty());
    }
    cs[1].load = true;
    wait(&mut cs, &d, "cooperative43 active shared scene", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    let world_id = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .identity
        .clone();
    assert_eq!(state(&cs[0]).phase, data::TimePhase::WaitingForTower);
    for i in 0..2 {
        let t = player(&cs[i]);
        haste(&mut cs, &d, t, i as i64 + 1);
    }
    assert!(own_actor(&cs[1]).buffs.unwrap().haste_until_nanos > 0);
    // Intent framing is not learned-skill permission; the player input gate is.
    assert!(!battle_input::skill_available(43, &cs[1].view()));
    assert!(!battle_input::ready(
        Kind::Haste(player(&cs[1])),
        &cs[1].view()
    ));
    for _ in 0..10 {
        if state(&cs[0]).tower_hp == 0 {
            break;
        }
        let v = input(&mut cs, 0, &d, Kind::Attack(Target::TimeTower));
        applied(&v);
        if state(&cs[0]).tower_hp > 0 {
            pause(&mut cs, &d, 750);
        }
    }
    assert_eq!(state(&cs[0]).tower_hp, 0);
    pause(&mut cs, &d, 750);
    let n = target(&mut cs, 0, &d, 0);
    haste(&mut cs, &d, n, 3);
    assert!(effects(&cs[0])
        .npcs
        .unwrap()
        .actors
        .0
        .iter()
        .any(|n| n.speed.haste_until_millis > 0));
    if !timeout {
        light_arrow(&mut cs, 0, &d, 0);
        pause(&mut cs, &d, 1250);
        let boss = target(&mut cs, 0, &d, 1);
        haste(&mut cs, &d, boss, 4);
        time_attack::leave(&mut cs, 0, &d, contexts[0], &world_id);
        wait(
            &mut cs,
            &d,
            "cooperative43 departure preserves other target grants",
            |cs| {
                cs[1].rendered() && state(&cs[1]).leader == 2 && state(&cs[1]).departed.0 == vec![1]
            },
        );
        let v = cs[1].view();
        let raw = v.canonical.observation().unwrap().data();
        assert_eq!(raw.actors.0.len(), 2);
        assert_eq!(raw.actors.0[0].buffs.as_ref().unwrap().haste_until_nanos, 0);
        assert!(raw.actors.0[1].buffs.as_ref().unwrap().haste_until_nanos > 0);
        assert!(effects(&cs[1])
            .npcs
            .unwrap()
            .actors
            .0
            .iter()
            .any(|n| n.speed.haste_until_millis > 0));
        light_arrow(&mut cs, 1, &d, 1);
    }
    wait(
        &mut cs,
        &d,
        "cooperative43 terminal effects and durable personal results",
        |cs| {
            (if timeout { 0..2 } else { 1..2 }).all(|i| {
                cs[i].rendered()
                    && cs[i].view().canonical.lifecycle().is_some_and(|l| {
                        l.phase == world::Phase::Finished
                            && l.settlement == Some(world::SettlementState::Saved)
                    })
            })
        },
    );
    for i in if timeout { 0..2 } else { 1..2 } {
        let v = cs[i].view();
        let raw = v.canonical.observation().unwrap().data();
        assert_eq!(
            state(&cs[i]).end,
            if timeout {
                data::TimeEnd::Timeout
            } else {
                data::TimeEnd::Cleared
            }
        );
        let h = effects(&cs[i]);
        assert!(h.targets.0.iter().all(|e| e.active.is_none()));
        assert!(h
            .npcs
            .unwrap()
            .actors
            .0
            .iter()
            .all(|n| n.speed.haste_until_millis == 0));
        assert!(raw
            .actors
            .0
            .iter()
            .all(|a| a.buffs.as_ref().unwrap().haste_until_nanos == 0));
        let result = raw.result.as_ref().unwrap();
        assert_eq!(result.participants.0.len(), 2);
        assert!(result
            .participants
            .0
            .iter()
            .all(|p| p.identity.id == 3333 && p.team == 0));
        for p in &result.participants.0 {
            assert_eq!(
                p.verdict,
                if timeout || p.identity.source == 7 {
                    data::Verdict::Loss
                } else {
                    data::Verdict::Win
                }
            );
        }
        let h = raw.actors.0[0]
            .spells
            .histories
            .0
            .iter()
            .find(|h| h.lane == data::Lane::Haste)
            .unwrap();
        assert_eq!(h.ledger.spent, if timeout { 45 } else { 60 });
    }
    for i in if timeout { 0..2 } else { 1..2 } {
        time_attack::leave(&mut cs, i, &d, contexts[i], &world_id);
    }
    for i in 0..2 {
        let context = target_haste_sql::select(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        wait(
            &mut cs,
            &d,
            "cooperative43 reselected source record",
            |cs| {
                cs[i].view().record.context() == Some(context)
                    && cs[i].view().record.totals().is_some()
            },
        );
        let won = !timeout && i == 1;
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: i64::from(won),
                losses: i64::from(!won),
                revision: 1
            })
        );
        target_haste_sql::saved(&cs[i], context);
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "cooperative43 normal shutdown", |cs| {
        cs.iter()
            .all(|c| c.runtime.is_quiescent() && c.controller.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
