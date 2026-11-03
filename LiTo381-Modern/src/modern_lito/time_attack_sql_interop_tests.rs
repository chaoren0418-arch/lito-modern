//! Real production Main/TLS/isolated SQL and shared render workers. No HWND.
use super::quest_npc70_sql_interop_tests as npc70_sql;
use super::*;

pub(super) fn leave_world(cs: &mut [Client; 2], i: usize, d: &Deadline) {
    cs[i].auto_ack = false;
    let v = cs[i].view();
    let f = v.world.frame().unwrap();
    let mut request = world::Request::read(v.navigation.as_ref().unwrap().context, CHANNEL);
    request.action = world::Action::Leave;
    request.match_id = f.match_id;
    request.generation = f.generation;
    cs[i].command(LoginCommand::World(request));
    wait(cs, d, "cooperative world return", |cs| {
        cs[i].view().world.frame().is_none()
            && !cs[i].view().world_pending
            && cs[i].runtime.is_quiescent()
    });
}
pub(super) fn back_to_roster(cs: &mut [Client; 2], i: usize, d: &Deadline) {
    let context = cs[i].view().navigation.as_ref().unwrap().context;
    room(cs, i, d, rooms::Request::read(context, CHANNEL).unwrap());
    let s = cs[i].view().rooms.joined().unwrap().summary.clone();
    room(
        cs,
        i,
        d,
        rooms::Request::simple(rooms::Common {
            context,
            channel_id: CHANNEL,
            action: rooms::Action::Leave,
            value: 0,
            room_id: s.room_id,
            expected_revision: s.revision,
        })
        .unwrap(),
    );
    cs[i].command(LoginCommand::LeaveLobby(context));
    wait(cs, d, "cooperative back to channels", |cs| {
        cs[i].view().phase == LoginPhase::ChoosingChannel
    });
    cs[i].command(LoginCommand::ReturnCharacters(context));
    wait(cs, d, "cooperative back to roster", |cs| {
        cs[i].view().phase == LoginPhase::CharactersReady
    });
}

#[test]
#[ignore = "requires explicit shared resources, compiled Java peer, TLS fixtures and isolated MySQL installation"]
fn java_time_attack_sql_two_controllers_shared_tower_leader_departure_waves_and_source_results() {
    time_attack_sql_flow(false);
}
#[test]
#[ignore = "v29 NPC77 through real ModernMain/SELECT-only sources/result SQL, two shared-render controllers, tower/waves/leader departure/settlement; no HWND"]
fn java_npc77_time_attack_sql_shared_casters_source_costs_kill_settlement_and_return() {
    time_attack_sql_flow(true);
}
pub(super) fn npc77_request(
    c: &Client,
    target: npc::Id,
    target_life: u64,
) -> crate::modern_lito::npc_disintegrate_protocol::Request {
    let v = c.view();
    let f = v.npc.as_ref().unwrap().frame.as_ref().unwrap();
    let w = &f.combat.world;
    let context = v.navigation.as_ref().unwrap().context;
    let entity = w
        .actors
        .iter()
        .find(|a| a.matches_character(context.key))
        .unwrap()
        .entity_id;
    crate::modern_lito::npc_disintegrate_protocol::Request {
        context,
        channel_id: CHANNEL,
        target,
        target_life,
        caster_life: quest_life::life(w, entity).map_or(1, |l| l.incarnation()),
        sequence: w.cast(entity).unwrap().request_sequence + 1,
    }
}
pub(super) fn npc77_audio(c: &Client) -> u64 {
    c.rendered
        .as_ref()
        .and_then(|r| r.audio.as_ref())
        .and_then(|a| {
            a.events
                .iter()
                .filter(|e| e.group.candidates() == [717])
                .map(|e| e.sequence)
                .max()
        })
        .unwrap_or(0)
}
#[test]
#[ignore = "v30 NPC light arrow, real Main/isolated SQL and two shared renderers; no HWND"]
fn java_npc4_time_attack_sql_shared_kills_saved_result_departure_and_return() {
    time_attack_spell_sql_flow(false, true);
}
#[test]
#[ignore = "v31 NPC70 real Main/isolated SQL and two shared renderers, fire tower/waves/departure/saved records/source reload; no HWND"]
fn java_npc70_time_attack_sql_shared_tower_casters_departure_saved_records_and_source_reload() {
    time_attack_all_spells_sql_flow(false, false, true);
}
fn light_arrow_audio(c: &Client) -> u64 {
    c.rendered
        .as_ref()
        .and_then(|r| r.audio.as_ref())
        .and_then(|a| {
            a.events
                .iter()
                .filter(|e| e.group.candidates() == [97])
                .map(|e| e.sequence)
                .max()
        })
        .unwrap_or(0)
}
fn time_attack_sql_flow(npc77: bool) {
    time_attack_spell_sql_flow(npc77, false);
}
fn time_attack_spell_sql_flow(npc77: bool, npc4: bool) {
    time_attack_all_spells_sql_flow(npc77, npc4, false);
}
fn time_attack_all_spells_sql_flow(npc77: bool, npc4: bool, npc70: bool) {
    assert!(usize::from(npc77) + usize::from(npc4) + usize::from(npc70) <= 1);
    let d = Deadline(Instant::now() + Duration::from_secs(160));
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernTimeAttackSqlTestPeer",
        if npc70 {
            "npc70-time-attack-sql"
        } else if npc77 {
            "npc77-time-attack-sql"
        } else if npc4 {
            "npc4-time-attack-sql"
        } else {
            "time-attack-sql"
        },
    );
    let p = profile(peer.endpoint, &shared, &private_root(&fixtures));
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new(p.clone()),
        runtime: WorldRuntime::new(shared.clone()).unwrap(),
        rendered: None,
        auto_ack: false,
        acks: BTreeSet::new(),
        light_arrow_sound: false,
        canonical: Some(
            crate::modern_lito::battle_runtime::BattleRuntime::new(shared.clone()).unwrap(),
        ),
        canonical_surface: None,
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    for i in 0..2 {
        cs[i].command(LoginCommand::Connect);
        wait(&mut cs, &d, "cooperative TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        cs[i].command(LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "cooperative login", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = selected_with_mp(
            &mut cs,
            i,
            &d,
            if i == 0 { 7 } else { 42 },
            if npc77 || npc70 {
                300
            } else if i == 0 {
                30
            } else {
                40
            },
        );
        if npc77 {
            assert!(cs[i].view().skills.disintegrate().is_some());
        }
        if npc70 {
            npc70_sql::assert_skill(&cs[i]);
        }
        assert!(!cs[i].view().record.totals().unwrap().stored);
        cs[i].command(LoginCommand::EnterChannel {
            context: contexts[i],
            channel_id: CHANNEL,
        });
        wait(&mut cs, &d, "cooperative lobby", |cs| {
            cs[i].view().phase == LoginPhase::InLobby
        });
    }
    let mut create = rooms::Common::read(contexts[0], CHANNEL);
    create.action = rooms::Action::Create;
    room(
        &mut cs,
        0,
        &d,
        rooms::Request::draft(
            create,
            rooms::Draft::new(
                rooms::PublicDraft {
                    capacity: 4,
                    mode: 4,
                    map_id: 540,
                    attributes: 0,
                    name: "Shared cooperative SQL".into(),
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    let s = cs[0].view().rooms.joined().unwrap().summary.clone();
    room(
        &mut cs,
        1,
        &d,
        rooms::Request::join(
            rooms::Common {
                context: contexts[1],
                channel_id: CHANNEL,
                action: rooms::Action::Join,
                value: 0,
                room_id: s.room_id,
                expected_revision: s.revision,
            },
            String::new(),
        )
        .unwrap(),
    );
    for i in 0..2 {
        room(
            &mut cs,
            i,
            &d,
            rooms::Request::read(contexts[i], CHANNEL).unwrap(),
        );
        let joined = cs[i].view().rooms.joined().unwrap().clone();
        assert!(joined.members.iter().all(|m| m.team == 0));
        room(
            &mut cs,
            i,
            &d,
            rooms::Request::simple(rooms::Common {
                context: contexts[i],
                channel_id: CHANNEL,
                action: rooms::Action::Ready,
                value: 1,
                room_id: joined.summary.room_id,
                expected_revision: joined.summary.revision,
            })
            .unwrap(),
        );
    }
    cs[0].auto_ack = true;
    wait(&mut cs, &d, "cooperative complete shared preload", |cs| {
        if npc70 {
            for c in cs {
                if let Some(f) = c.view().world.frame() {
                    assert_ne!(
                        f.phase,
                        world::Phase::Aborted,
                        "NPC70 SQL preparation aborted: {:?}",
                        f.abort_reason
                    );
                }
            }
        }
        cs.iter().all(|c| {
            c.rendered.as_ref().is_some_and(|r| {
                r.ready_to_ack
                    && r.npc.is_some()
                    && r.time_attack_tower.as_ref().is_some_and(|p| p.alive)
            })
        }) && cs[0]
            .view()
            .world
            .frame()
            .is_some_and(|w| w.actors[0].loaded)
    });
    for c in &cs {
        assert_eq!(c.view().world.frame().unwrap().phase, world::Phase::Loading);
        assert_eq!(c.view().world.frame().unwrap().manifest.map_id, 554);
        assert!(c.rendered.as_ref().unwrap().npcs.is_empty());
    }
    cs[1].auto_ack = true;
    wait(&mut cs, &d, "cooperative both loaded", |cs| {
        cs.iter().all(|c| {
            c.view()
                .world
                .frame()
                .is_some_and(|f| f.phase == world::Phase::Active)
        })
    });
    followed(
        &mut cs,
        &d,
        crate::modern_lito::battle_payload::data::Phase::Active,
    );
    let v = cs[0].view();
    let target = cs[0]
        .rendered
        .as_ref()
        .unwrap()
        .time_attack_tower
        .as_ref()
        .unwrap()
        .id;
    let mut foreign = target;
    foreign.scope.generation += 1;
    assert!(cs[0]
        .controller
        .command(LoginCommand::TimeAttackTowerAttack { target: foreign })
        .is_err());
    if npc70 {
        let result = npc70_sql::cast(&mut cs, 0, &d);
        assert_eq!(result.count, 1);
        assert!(result.targets.is_empty());
        let structure = result.current_structure().unwrap();
        assert_eq!(
            structure.id,
            crate::modern_lito::npc_fire_storm_wire::StructureId::Tower(target)
        );
        assert!(structure.outcome.death);
        assert_eq!(structure.outcome.applied_damage, 1);
    } else {
        cs[0].command(LoginCommand::TimeAttackTowerAttack { target });
        assert_eq!(cs[0].view().combat, v.combat);
    }
    wait(&mut cs, &d, "cooperative tower attack receipt", |cs| {
        !cs[0].view().combat_pending
            && cs.iter().all(|c| {
                c.rendered
                    .as_ref()
                    .is_some_and(|r| r.time_attack_tower.as_ref().is_some_and(|p| !p.alive))
            })
    });
    let mut first_guest_sound = 0;
    if npc70 {
        wait(&mut cs, &d, "NPC70 shared tower effect", |cs| {
            cs.iter().all(|c| npc70_sql::audio(c) > 0)
        });
        let sounds = [npc70_sql::audio(&cs[0]), npc70_sql::audio(&cs[1])];
        let ready = Instant::now() + Duration::from_millis(2100);
        wait(
            &mut cs,
            &d,
            "NPC70 tower cooldown and first SQL wave",
            |cs| {
                Instant::now() >= ready
                    && cs[0]
                        .rendered
                        .as_ref()
                        .is_some_and(|r| r.npcs.iter().any(|n| n.alive))
            },
        );
        let before = cs[0]
            .view()
            .combat
            .as_ref()
            .unwrap()
            .npc_fire(1)
            .unwrap()
            .clone();
        let result = npc70_sql::cast(&mut cs, 0, &d);
        assert_eq!((result.count, result.mp_cost, result.mp_spent), (2, 46, 92));
        assert_eq!(result.targets.len(), 1);
        assert!(result.targets[0].outcome.death);
        assert_eq!(result.targets[0].outcome.applied_damage, 1);
        assert_eq!(
            result.structure, before.structure,
            "retain tower history without re-hitting it"
        );
        assert!(result.current_structure().is_none());
        assert_eq!(
            cs[0]
                .view()
                .combat
                .as_ref()
                .unwrap()
                .npc_fire(2)
                .unwrap()
                .count,
            0
        );
        wait(
            &mut cs,
            &d,
            "NPC70 wave effect on both shared renderers",
            |cs| (0..2).all(|i| npc70_sql::audio(&cs[i]) > sounds[i]),
        );
        first_guest_sound = npc70_sql::audio(&cs[1]);
        wait(&mut cs, &d, "NPC70 next cooperative wave", |cs| {
            cs[1]
                .view()
                .world
                .frame()
                .is_some_and(|f| f.time_attack.as_ref().unwrap().wave == 1)
        });
    }
    if npc77 {
        let cast_ready = Instant::now() + Duration::from_millis(1250);
        wait(&mut cs, &d, "tower action cooldown before NPC77", |_| {
            Instant::now() >= cast_ready
        });
        wait(&mut cs, &d, "NPC77 first SQL wave painted", |cs| {
            cs[0]
                .rendered
                .as_ref()
                .is_some_and(|r| r.npcs.iter().any(|n| n.alive))
        });
        let view = cs[0].view();
        let f = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
        assert_eq!(f.combat.world.time_attack.as_ref().unwrap().wave, 0);
        let n = f.actors.iter().find(|a| a.alive()).unwrap();
        let target = npc::Id {
            scope: f.scope,
            serial: n.serial,
        };
        let q = npc77_request(&cs[0], target, n.life);
        let mut foreign = q;
        foreign.context = contexts[1];
        assert!(cs[0]
            .controller
            .command(LoginCommand::CastNpcDisintegrate(foreign))
            .is_err());
        let mut stale = q;
        stale.target_life += 1;
        assert!(cs[0]
            .controller
            .command(LoginCommand::CastNpcDisintegrate(stale))
            .is_err());
        let before = f.combat.npc_disintegrate(1).unwrap().clone();
        cs[0].command(LoginCommand::CastNpcDisintegrate(q));
        assert_eq!(
            cs[0].view().combat.as_ref().unwrap().npc_disintegrate(1),
            Some(&before)
        );
        assert!(cs[0]
            .controller
            .command(LoginCommand::CastNpcDisintegrate(q))
            .is_err());
        wait(&mut cs, &d, "NPC77 source7 SQL kill", |cs| {
            !cs[0].view().spell_pending
                && cs[0]
                    .view()
                    .combat
                    .as_ref()
                    .is_some_and(|f| f.world.cast(1).unwrap().request_sequence == q.sequence)
        });
        let v = cs[0].view();
        let f = v.combat.as_ref().unwrap();
        let s = f.npc_disintegrate(1).unwrap();
        assert_eq!(s.count, 1, "{}", v.status);
        assert_eq!((s.mp_cost, s.mp_spent, s.lawful), (68, 68, 501));
        assert!(s.hit.as_ref().unwrap().death);
        assert_eq!(s.hit.as_ref().unwrap().target, target);
        assert_eq!(f.npc_disintegrate(2).unwrap().count, 0);
        wait(
            &mut cs,
            &d,
            "NPC77 first effect on both real renderers",
            |cs| cs.iter().all(|c| npc77_audio(c) > 0),
        );
        first_guest_sound = npc77_audio(&cs[1]);
        wait(&mut cs, &d, "NPC77 next cooperative wave", |cs| {
            cs[1]
                .view()
                .world
                .frame()
                .is_some_and(|f| f.time_attack.as_ref().unwrap().wave == 1)
        });
    }
    if npc4 {
        let ready = Instant::now() + Duration::from_millis(1250);
        wait(
            &mut cs,
            &d,
            "tower cooldown and NPC light arrow target",
            |cs| {
                Instant::now() >= ready
                    && cs[0]
                        .rendered
                        .as_ref()
                        .is_some_and(|r| r.npcs.iter().any(|n| n.alive))
            },
        );
        let v = cs[0].view();
        let f = v.npc.as_ref().unwrap().frame.as_ref().unwrap();
        let n = f.actors.iter().find(|n| n.alive()).unwrap();
        let target = npc::Id {
            scope: f.scope,
            serial: n.serial,
        };
        let life = n.life;
        let before = f.combat.npc_spell(1).unwrap().clone();
        assert!(cs[0]
            .controller
            .command(LoginCommand::CastNpcLightArrow {
                target,
                life: life + 1
            })
            .is_err());
        cs[0].command(LoginCommand::CastNpcLightArrow { target, life });
        assert_eq!(
            cs[0].view().combat.as_ref().unwrap().npc_spell(1),
            Some(&before)
        );
        assert!(cs[0]
            .controller
            .command(LoginCommand::CastNpcLightArrow { target, life })
            .is_err());
        wait(&mut cs, &d, "source7 real light arrow kill", |cs| {
            !cs[0].view().spell_pending
                && cs[0]
                    .view()
                    .combat
                    .as_ref()
                    .is_some_and(|f| f.npc_spell(1).unwrap().cast_count == 1)
        });
        let v = cs[0].view();
        let f = v.combat.as_ref().unwrap();
        let s = f.npc_spell(1).unwrap();
        assert_eq!((s.mp_cost, s.mp_spent), (1, 1));
        assert_eq!(s.receipt.as_ref().unwrap().target, target);
        assert!(s.receipt.as_ref().unwrap().death);
        assert_eq!(f.npc_spell(2).unwrap().cast_count, 0);
        assert_eq!(f.world.cast(1).unwrap().cast_count, 0);
        wait(
            &mut cs,
            &d,
            "light arrow flight and sound97 on both shared renderers",
            |cs| cs.iter().all(|c| light_arrow_audio(c) > 0),
        );
        first_guest_sound = light_arrow_audio(&cs[1]);
        wait(&mut cs, &d, "light arrow advances cooperative wave", |cs| {
            cs[1]
                .view()
                .world
                .frame()
                .is_some_and(|f| f.time_attack.as_ref().unwrap().wave == 1)
        });
    }
    leave_world(&mut cs, 0, &d);
    back_to_roster(&mut cs, 0, &d);
    wait(
        &mut cs,
        &d,
        "cooperative leader transferred and retired actor hidden",
        |cs| {
            cs[1].rendered.as_ref().is_some_and(|r| {
                r.actors.len() == 1
                    && r.combat.as_ref().is_some_and(|c| {
                        c.world.time_attack.as_ref().is_some_and(|t| {
                            t.leader == 2 && t.departed(1) && c.world.actors.len() == 2
                        })
                    })
            })
        },
    );
    let mut next = Instant::now();
    let mut waves = BTreeSet::new();
    if npc77 || npc4 || npc70 {
        waves.insert(0);
    }
    loop {
        wait(&mut cs, &d, "cooperative waves", |_| true);
        let view = cs[1].view();
        let f = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
        if f.combat.world.phase == world::Phase::Finished {
            break;
        }
        if !view.combat_pending && !view.spell_pending && Instant::now() >= next {
            if let Some(actor) = f.actors.iter().find(|a| a.alive()) {
                let target = npc::Id {
                    scope: f.scope,
                    serial: actor.serial,
                };
                if cs[1].rendered.as_ref().is_some_and(|r| {
                    r.npcs
                        .iter()
                        .any(|p| p.id == target && p.life == actor.life && p.alive)
                }) {
                    waves.insert(f.combat.world.time_attack.as_ref().unwrap().wave);
                    if npc70 {
                        npc70_sql::cast(&mut cs, 1, &d);
                    } else if npc77 {
                        let q = npc77_request(&cs[1], target, actor.life);
                        cs[1].command(LoginCommand::CastNpcDisintegrate(q));
                    } else if npc4 {
                        cs[1].command(LoginCommand::CastNpcLightArrow {
                            target,
                            life: actor.life,
                        });
                    } else {
                        cs[1].command(LoginCommand::NpcAttack {
                            target,
                            life: actor.life,
                        });
                    }
                    next = Instant::now()
                        + Duration::from_millis(if npc70 {
                            2100
                        } else if npc77 {
                            5100
                        } else if npc4 {
                            1250
                        } else {
                            750
                        });
                }
            }
        }
        d.pause("cooperative real RNG waves");
    }
    assert_eq!(waves.len(), 2);
    wait(&mut cs, &d, "cooperative result saved and rendered", |cs| {
        cs[1]
            .rendered
            .as_ref()
            .and_then(|r| r.combat.as_ref())
            .and_then(|c| c.world.time_attack.as_ref())
            .is_some_and(|t| {
                t.receipt
                    .as_ref()
                    .is_some_and(|r| r.state == world::SettlementState::Saved)
            })
    });
    if npc70 {
        wait(&mut cs, &d, "NPC70 terminal effect after SQL Saved", |cs| {
            npc70_sql::audio(&cs[1]) > first_guest_sound
        });
        let v = cs[1].view();
        let f = v.combat.as_ref().unwrap();
        for (id, count, cost) in [(1, 2, 46), (2, 1, 48)] {
            let s = f.npc_fire(id).unwrap();
            assert_eq!(
                (s.count, s.mp_cost, s.mp_spent),
                (count, cost, count * i64::from(cost))
            );
            assert_eq!(s.targets.len(), 1);
            assert!(s.targets[0].outcome.death);
            assert_eq!(s.targets[0].outcome.applied_damage, 1);
        }
        assert_eq!(
            f.world.actors[0].key.object_id,
            f.world.actors[1].key.object_id
        );
        assert_ne!(
            f.world.actors[0].key.source_id,
            f.world.actors[1].key.source_id
        );
    }
    if npc77 {
        wait(
            &mut cs,
            &d,
            "NPC77 terminal effect after saved result",
            |cs| npc77_audio(&cs[1]) > first_guest_sound,
        );
        let v = cs[1].view();
        let f = v.combat.as_ref().unwrap();
        for (id, cost) in [(1, 68), (2, 70)] {
            let s = f.npc_disintegrate(id).unwrap();
            assert_eq!(
                (s.count, s.mp_cost, s.mp_spent, s.lawful),
                (1, cost, cost as i64, 501)
            );
            assert!(s.hit.as_ref().unwrap().death);
            assert_eq!(s.hit.as_ref().unwrap().applied_damage, 1);
        }
        assert_eq!(
            f.world.actors[0].key.object_id,
            f.world.actors[1].key.object_id
        );
        assert_ne!(
            f.world.actors[0].key.source_id,
            f.world.actors[1].key.source_id
        );
    }
    if npc4 {
        wait(
            &mut cs,
            &d,
            "final light arrow projectile survives Saved result",
            |cs| light_arrow_audio(&cs[1]) > first_guest_sound,
        );
        let v = cs[1].view();
        let f = v.combat.as_ref().unwrap();
        for (id, cost) in [(1, 1), (2, 3)] {
            let s = f.npc_spell(id).unwrap();
            assert_eq!((s.cast_count, s.mp_cost, s.mp_spent), (1, cost, cost));
            assert!(s.receipt.as_ref().unwrap().death);
            assert_eq!(s.receipt.as_ref().unwrap().applied_damage, 1);
            assert_eq!(f.world.cast(id).unwrap().cast_count, 0);
        }
        assert!(f.npc_disintegrations.is_empty() && f.damages.is_empty());
        let mut forged = f.as_ref().clone();
        let anchor = f
            .world
            .time_attack
            .as_ref()
            .unwrap()
            .receipt
            .as_ref()
            .unwrap()
            .revision
            + 1;
        assert!(
            anchor < f.world.revision,
            "Saved publishes only a newer snapshot, not a cast"
        );
        forged.npc_spells[1].revision = f.world.revision;
        assert!(
            forged.validate().is_err(),
            "no new cast after the terminal anchor"
        );
    }
    let done = cs[1].view().world.frame().unwrap().clone();
    let state = done.time_attack.as_ref().unwrap();
    assert!(state.won(2) && !state.won(1));
    assert_eq!(
        state.end,
        crate::modern_lito::time_attack_protocol::End::Cleared
    );
    assert!(
        crate::modern_lito::time_attack_presentation::result_notice(&done, contexts[1].key)
            .unwrap()
            .contains("本場勝利")
    );
    assert!(
        crate::modern_lito::time_attack_presentation::result_notice(&done, contexts[0].key)
            .unwrap()
            .contains("本場敗北")
    );
    assert_eq!((cs[0].acks.len(), cs[1].acks.len()), (1, 1));
    leave_world(&mut cs, 1, &d);
    back_to_roster(&mut cs, 1, &d);
    for i in 0..2 {
        selected_with_mp(
            &mut cs,
            i,
            &d,
            if i == 0 { 7 } else { 42 },
            if npc77 || npc70 {
                300
            } else if i == 0 {
                30
            } else {
                40
            },
        );
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: if i == 1 { 1 } else { 0 },
                losses: if i == 0 { 1 } else { 0 },
                revision: 1
            })
        );
    }
    cs[1].command(LoginCommand::Shutdown);
    wait(&mut cs, &d, "cooperative guest retirement", |cs| {
        cs[1].controller.is_quiescent()
    });
    cs[0].command(LoginCommand::ReturnCharacters(
        cs[0].view().navigation.as_ref().unwrap().context,
    ));
    wait(&mut cs, &d, "cooperative source switch", |cs| {
        cs[0].view().phase == LoginPhase::CharactersReady
    });
    selected_with_mp(&mut cs, 0, &d, 42, if npc77 || npc70 { 300 } else { 40 });
    if npc70 {
        npc70_sql::assert_skill(&cs[0]);
    }
    assert_eq!(cs[0].view().record.totals().unwrap().wins, 1);
    cs[0].command(LoginCommand::Shutdown);
    wait(&mut cs, &d, "cooperative owners retired", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
