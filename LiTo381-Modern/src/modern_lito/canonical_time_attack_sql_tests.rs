//! v43 cooperative game: complete NPC motion roster, shared tower/waves, leader
//! withdrawal, visible HUD and per-person SQL result. Never a legacy Frame.
use super::*;

fn state(c: &Client) -> data::TimeAttack {
    let view = c.view();
    let o = view.canonical.observation().unwrap();
    let motion = o
        .data()
        .npc_motion
        .as_ref()
        .expect("v43 mandatory motion section")
        .state
        .as_ref()
        .expect("Time Attack NPC roster, including empty/closed");
    assert_eq!(
        motion.actors.0.len(),
        o.data().npcs.as_ref().unwrap().actors.0.len()
    );
    c.view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .time_attack
        .clone()
        .unwrap()
}
fn sound(c: &Client, id: u32) -> u64 {
    c.view()
        .canonical
        .observation()
        .as_ref()
        .map_or(0, |o| c.audio.sequence(Stamp::of(o), id))
}
fn history(c: &Client, fire: bool) -> data::Spell {
    own_actor(c)
        .spells
        .histories
        .0
        .into_iter()
        .find(|s| {
            s.lane
                == if fire {
                    data::Lane::NpcFireStorm
                } else {
                    data::Lane::NpcDisintegrate
                }
        })
        .unwrap()
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
fn pause(cs: &mut [Client; 2], d: &Deadline, ms: u64) {
    let ready = Instant::now() + Duration::from_millis(ms);
    wait(cs, d, "cooperative cast cadence", |_| {
        Instant::now() >= ready
    });
}

/// Neither controller sends a command during this window. UI polling and a
/// renderer must not be the network worker's only source of timely wakeups.
fn idle_read_cadence(cs: &mut [Client; 2], d: &Deadline) {
    let start = Instant::now();
    let initial = std::array::from_fn::<_, 2, _>(|i| {
        cs[i]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .world
            .loading_elapsed_nanos
    });
    let latest = std::cell::Cell::new(initial);
    let samples = std::cell::RefCell::new([Vec::<Duration>::new(), Vec::new()]);
    wait(cs, d, "idle canonical world read cadence", |cs| {
        let mut last = latest.get();
        for i in 0..2 {
            let v = cs[i].view();
            let data = v.canonical.observation().unwrap().data();
            assert_eq!(data.world.phase, data::Phase::Active);
            assert_eq!(
                data.time_attack.as_ref().unwrap().phase,
                data::TimePhase::WaitingForTower
            );
            assert!(!v.world_pending);
            let clock = data.world.loading_elapsed_nanos;
            assert!(clock >= last[i]);
            if clock != last[i] {
                samples.borrow_mut()[i].push(start.elapsed());
                last[i] = clock;
            }
        }
        latest.set(last);
        start.elapsed() >= Duration::from_secs(4)
    });
    let samples = samples.into_inner();
    for (i, reads) in samples.iter().enumerate() {
        let gaps = reads.windows(2).map(|s| s[1] - s[0]).collect::<Vec<_>>();
        eprintln!(
            "COOP_IDLE_READS owner={i} count={} over={:?} gaps={gaps:?}",
            reads.len(),
            start.elapsed()
        );
    }
    assert!(
        samples.iter().all(|s| s.len() >= 12),
        "200ms canonical cadence must not depend on 750ms chat/1s room wakes; counts={:?}",
        samples.each_ref().map(|s| s.len())
    );
}
fn target(cs: &mut [Client; 2], i: usize, d: &Deadline, wave: i32) -> Target {
    wait(cs, d, "cooperative current live target", |cs| {
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
    let s = cs[i].surface.as_ref().unwrap();
    let n = s.geometry.npcs.iter().find(|n| n.alive).unwrap();
    Target::Npc {
        serial: n.id.serial as i64,
        life: n.life as i64,
    }
}
fn cast(cs: &mut [Client; 2], i: usize, d: &Deadline, fire: bool, wave: i32) {
    let target = target(cs, i, d, wave);
    let before = history(&cs[i], fire).ledger.casts;
    let result = input(
        cs,
        i,
        d,
        if fire {
            Kind::Local(Local::FireStorm)
        } else {
            Kind::Disintegrate(target)
        },
    );
    applied(&result);
    let s = history(&cs[i], fire);
    let cost = if fire {
        if i == 0 {
            46
        } else {
            48
        }
    } else if i == 0 {
        68
    } else {
        70
    };
    assert_eq!(s.ledger.casts, before + 1);
    assert_eq!(s.ledger.cost, cost);
    assert_eq!(s.ledger.spent, s.ledger.casts * i64::from(cost));
    assert_eq!(
        s.last.as_ref().unwrap().before.vitals.mp - s.last.as_ref().unwrap().mp_after,
        i64::from(cost)
    );
    if !fire {
        assert_eq!(own_actor(&cs[i]).spells.lawful, 501);
    }
    if let Target::Npc { serial, life } = target {
        assert!(
            !battle_window::target_current(
                result.canonical.observation().unwrap(),
                Target::Npc { serial, life }
            ),
            "lethal skill must retire the exact target life"
        );
    }
}
pub(super) fn leave(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    context: SelectionContext,
    world: &data::WorldId,
) {
    let leave = Command::world(battle_world::Request {
        action: battle_world::Action::Leave,
        ..battle_world::Request::read(context, CHANNEL, Some(world))
    });
    send(cs, i, d, || LoginCommand::Canonical(leave.clone()));
    wait(cs, d, "cooperative canonical world return", |cs| {
        !cs[i].view().world_pending
            && !cs[i].view().canonical.attached()
            && cs[i].runtime.is_quiescent()
    });
    assert!(cs[i].view().rooms.joined().is_some());
    room_action(cs, i, d, context, rooms::Action::Leave, 0);
    send(cs, i, d, || LoginCommand::LeaveLobby(context));
    wait(cs, d, "cooperative channels return", |cs| {
        cs[i].view().phase == LoginPhase::ChoosingChannel
    });
    send(cs, i, d, || LoginCommand::ReturnCharacters(context));
    wait(cs, d, "cooperative roster return", |cs| {
        cs[i].view().phase == LoginPhase::CharactersReady
    });
}

#[test]
#[ignore = "v43 Main/owned SQL/shared554: NPC motion, fire tower/waves, leader departure, HUD, source costs, saved personal result and return; no HWND"]
fn java_v43_sql_time_attack_fire_leader_departure_hud_saved_and_return() {
    flow(true, false);
}
#[test]
#[ignore = "v43 Main/owned SQL/shared554: NPC motion, disintegrate deaths, leader departure, HUD, costs/lawful, saved personal result and return; no HWND"]
fn java_v43_sql_time_attack_disintegrate_leader_departure_hud_saved_and_return() {
    flow(false, false);
}

#[test]
#[ignore = "ordinary schema2 v47 + Main/owned SQL/shared554: Fire Storm, leader departure, saved result and return; no HWND"]
fn java_ordinary_v47_time_attack_fire_saved_and_return() {
    flow(true, true);
}
#[test]
#[ignore = "ordinary schema2 v47 + Main/owned SQL/shared554: Disintegrate, leader departure, saved result and return; no HWND"]
fn java_ordinary_v47_time_attack_disintegrate_saved_and_return() {
    flow(false, true);
}
fn flow(fire: bool, ordinary: bool) {
    let d = Deadline(Instant::now() + Duration::from_secs(170));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernTimeAttackSqlTestPeer",
        if fire {
            "npc70-time-attack-sql"
        } else {
            "npc77-time-attack-sql"
        },
    );
    let mut cs = std::array::from_fn(|_| Client {
        controller: mode_controller(peer.endpoint, &shared, &private_root(&fixtures), ordinary),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "cooperative canonical TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "cooperative canonical login", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = selected(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
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
                        capacity: 4,
                        mode: 4,
                        map_id: 540,
                        attributes: 0,
                        name: "Canonical cooperative SQL".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "cooperative creator", |cs| {
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
    wait(&mut cs, &d, "cooperative joined", |cs| {
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
    wait(&mut cs, &d, "cooperative complete shared preload", |cs| {
        !cs[0].load
            && cs.iter().all(|c| {
                c.surface
                    .as_ref()
                    .is_some_and(|s| s.objectives.len() == 1 && s.geometry.npcs.is_empty())
            })
    });
    for c in &cs {
        let v = c.view();
        let raw = v.canonical.observation().unwrap().data();
        assert_eq!(
            (raw.room.mode, raw.room.map, raw.manifest.map),
            (4, 540, 554)
        );
        assert_eq!(raw.world.phase, data::Phase::Loading);
        assert_eq!(
            raw.world.descriptor.ruleset,
            data::Ruleset::CooperativeTimeAttack
        );
    }
    cs[1].load = true;
    wait(&mut cs, &d, "cooperative active", |cs| {
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
    idle_read_cadence(&mut cs, &d);
    let before = state(&cs[0]);
    assert_eq!(before.phase, data::TimePhase::WaitingForTower);
    for c in &cs {
        let hud = battle_window::hud(&c.surface.as_ref().unwrap().observation).join("\n");
        assert!(
            hud.contains("擊破啟動塔後開始") && hud.contains("波次 1/2") && hud.contains("時限")
        );
        assert!(hud.contains("留場存活 2 · 離隊 0"));
    }
    let sound_id = if fire { 710 } else { 717 };
    let tower = input(
        &mut cs,
        0,
        &d,
        if fire {
            Kind::Local(Local::FireStorm)
        } else {
            Kind::Attack(Target::TimeTower)
        },
    );
    applied(&tower);
    wait(&mut cs, &d, "cooperative tower retired", |cs| {
        cs.iter().all(|c| {
            c.surface
                .as_ref()
                .is_some_and(|s| s.observation.data().time_attack.as_ref().unwrap().tower_hp == 0)
        })
    });
    assert_eq!(state(&cs[0]).tower_hp, 0);
    if fire {
        assert_eq!(history(&cs[0], fire).ledger.casts, 1);
        let structure = own_actor(&cs[0]).spells.structure.unwrap();
        assert_eq!(structure.impact.targets.0.len(), 1);
        assert!(matches!(&structure.impact.targets.0[0].target,
            data::Target::TimeTower(id) if *id == world_id));
        wait(&mut cs, &d, "cooperative shared tower fire sound", |cs| {
            cs.iter().all(|c| sound(c, sound_id) > 0)
        });
    }
    pause(&mut cs, &d, if fire { 2100 } else { 1250 });
    let prior = [sound(&cs[0], sound_id), sound(&cs[1], sound_id)];
    cast(&mut cs, 0, &d, fire, 0);
    let heard = std::cell::Cell::new(prior);
    let sound_wait = Instant::now();
    let diagnosed = std::cell::Cell::new(0);
    wait(
        &mut cs,
        &d,
        "cooperative shared first-wave skill effect",
        |cs| {
            let last = heard.get();
            let next = std::array::from_fn(|i| last[i].max(sound(&cs[i], sound_id)));
            heard.set(next);
            let elapsed = sound_wait.elapsed().as_millis();
            if elapsed / 250 >= diagnosed.get() && elapsed < 6000 {
                diagnosed.set(elapsed / 250 + 1);
                for (i, c) in cs.iter().enumerate() {
                    let Some(s) = &c.surface else {
                        eprintln!("COOP_SOUND ms={elapsed} owner={i} surface=none");
                        continue;
                    };
                    let raw = s.observation.data();
                    let a = raw.actors.0.iter().find(|a| a.entity == 1).unwrap();
                    let cast = a
                        .spells
                        .histories
                        .0
                        .iter()
                        .find(|s| {
                            s.lane
                                == if fire {
                                    data::Lane::NpcFireStorm
                                } else {
                                    data::Lane::NpcDisintegrate
                                }
                        })
                        .and_then(|s| s.receipt.as_ref());
                    eprintln!("COOP_SOUND ms={elapsed} owner={i} revision={} wave={} hp={:?} move={} receipt={:?} age={:?} sounds={:?} heard={next:?} skipped={}",
                        raw.world.revision, raw.time_attack.as_ref().unwrap().wave,
                        a.vital.as_ref().map(|v| v.values.hp), a.last_move_sequence,
                        cast.map(|r| r.impact.usage.revision),
                        cast.map(|r| raw.world.loading_elapsed_nanos-r.loading_elapsed_nanos),
                        s.audio.as_ref().map(|a| a.events.iter().map(|e| e.group.candidates()).collect::<Vec<_>>()), s.effects_skipped);
                }
            }
            (0..2).all(|i| next[i] > prior[i])
        },
    );
    wait(&mut cs, &d, "cooperative second wave", |cs| {
        state(&cs[1]).wave == 1
    });
    let first_guest_sound = heard.get()[1];
    let removed = own_actor(&cs[0]).entity;
    leave(&mut cs, 0, &d, contexts[0], &world_id);
    wait(&mut cs, &d, "cooperative leader transfer painted", |cs| {
        cs[1].rendered()
            && cs[1].surface.as_ref().is_some_and(|s| {
                s.geometry.actors.len() == 1
                    && s.observation
                        .data()
                        .time_attack
                        .as_ref()
                        .is_some_and(|t| t.leader == 2 && t.departed.0 == vec![removed])
            })
    });
    let v = cs[1].view();
    let raw = v.canonical.observation().unwrap().data();
    assert_eq!(
        raw.actors.0.len(),
        2,
        "wire history retains the departed identity"
    );
    assert!(!crate::modern_lito::battle_space::occupies(raw, removed));
    let hud = battle_window::hud(&cs[1].surface.as_ref().unwrap().observation).join("\n");
    let leader = &raw.room.members.0[1];
    assert!(hud.contains(&format!(
        "隊長：{}({})",
        leader.name.0, leader.source_name.0
    )));
    assert!(
        hud.contains("留場存活 1 · 離隊 1")
            && hud.contains("波次 2/2")
            && hud.contains("伺服器剩餘")
    );
    assert!(!cs[0].view().canonical.attached() && cs[0].surface.is_none());
    cast(&mut cs, 1, &d, fire, 1);
    wait(&mut cs, &d, "cooperative canonical SQL saved", |cs| {
        cs[1].view().canonical.lifecycle().is_some_and(|l| {
            l.phase == world::Phase::Finished && l.settlement == Some(world::SettlementState::Saved)
        }) && cs[1]
            .surface
            .as_ref()
            .is_some_and(|s| s.observation.data().world.phase == data::Phase::Finished)
            && sound(&cs[1], sound_id) > first_guest_sound
    });
    let done = cs[1].view();
    let raw = done.canonical.observation().unwrap().data();
    let result = raw.result.as_ref().unwrap();
    assert_eq!(state(&cs[1]).end, data::TimeEnd::Cleared);
    assert_eq!(result.participants.0.len(), 2);
    assert!(result.participants.0.iter().all(|p| p.team == 0));
    for p in &result.participants.0 {
        assert_eq!(p.identity.id, 3333);
        assert_eq!(
            p.verdict,
            if p.identity.source == 7 {
                data::Verdict::Loss
            } else {
                data::Verdict::Win
            }
        );
    }
    let hud = battle_window::hud(&cs[1].surface.as_ref().unwrap().observation).join("\n");
    assert!(hud.contains("全部波次完成") && hud.contains("本場勝利") && !hud.contains("已保存"));
    assert!(
        cs[0].view().canonical.observation().is_none(),
        "no terminal private snapshot delivered to departed roster"
    );
    leave(&mut cs, 1, &d, contexts[1], &world_id);
    for i in 0..2 {
        let context = selected(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        wait(
            &mut cs,
            &d,
            "cooperative reselected automatic record",
            |cs| {
                cs[i].view().record.context() == Some(context)
                    && cs[i].view().record.totals().is_some()
            },
        );
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: i64::from(i == 1),
                losses: i64::from(i == 0),
                revision: 1
            })
        );
        source(&cs[i], context);
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "cooperative canonical shutdown", |cs| {
        cs.iter()
            .all(|c| c.runtime.is_quiescent() && c.controller.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
