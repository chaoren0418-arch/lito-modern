//! Pure v41 controllers, actual shared532, owned SQL settlement. Existing
//! synthetic source/camps/+100 ranged fixture, NOT production balance or Main.
use super::*;
use crate::modern_lito::battle_scene::ObjectiveTarget;

fn state(c: &Client) -> data::Honor {
    c.view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .honor
        .clone()
}
fn tower(c: &Client, id: i64) -> data::HonorObjective {
    state(c)
        .objectives
        .0
        .into_iter()
        .find(|t| t.entity == id)
        .unwrap()
}
fn hud(c: &Client) -> String {
    battle_window::hud(c.view().canonical.observation().unwrap()).join("\n")
}
fn pause(cs: &mut [Client; 2], d: &Deadline, ms: u64) {
    let until = Instant::now() + Duration::from_millis(ms);
    wait(cs, d, "honor real player cadence", |_| {
        Instant::now() >= until
    });
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
fn select(cs: &mut [Client; 2], i: usize, d: &Deadline, source: u32) -> SelectionContext {
    send(cs, i, d, || LoginCommand::SelectSource(source));
    wait(cs, d, "honor roster", |cs| {
        cs[i].view().phase == LoginPhase::CharactersReady
    });
    let v = cs[i].view();
    assert_eq!(v.selected_source, Some(source));
    assert_eq!(v.characters.len(), 1);
    let context = SelectionContext {
        key: CharacterKey {
            source_id: source,
            object_id: 3333,
        },
        epoch: v.roster_epoch,
    };
    send(cs, i, d, || LoginCommand::SelectCharacter(context));
    wait(cs, d, "honor source catalogs", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::ChoosingChannel
            && !v.record_pending
            && !v.spell_pending
            && v.record.context() == Some(context)
            && v.record.totals().is_some()
            && matches!(v.equipment, equipment::State::Ready(_))
    });
    let v = cs[i].view();
    let CharacterDataState::Ready(s) = &v.character_data else {
        panic!("honor source data")
    };
    assert_eq!(s.context, context);
    assert_eq!(s.stats.max_hp, if source == 7 { 100 } else { 40 });
    context
}
fn purchase(cs: &mut [Client; 2], i: usize, d: &Deadline, context: SelectionContext) {
    send(cs, i, d, || {
        LoginCommand::Shop(shop::Request::read(context, CHANNEL, None))
    });
    wait(cs, d, "honor shop read", |cs| !cs[i].view().shop_pending);
    assert_eq!(cs[i].view().shop.snapshot().unwrap().offers.len(), 9);
    for id in [
        40010, 40011, 40012, 40013, 40014, 40016, 40068, 40744, 40746,
    ] {
        let wallet = cs[i].view().shop.snapshot().unwrap().clone();
        let q = shop::Request::read(context, CHANNEL, None).mutation(
            &wallet,
            shop::Action::Buy,
            id,
            if i == 0 && id == 40744 { 80 } else { 1 },
        );
        send(cs, i, d, || LoginCommand::Shop(q));
        assert_eq!(cs[i].view().shop.snapshot(), Some(&wallet));
        wait(cs, d, "honor shop receipt", |cs| !cs[i].view().shop_pending);
        assert_eq!(cs[i].view().shop.reply().unwrap().status, shop::Status::Ok);
    }
}
fn cast(cs: &mut [Client; 2], i: usize, d: &Deadline, heal: bool, entity: i64) {
    let lane = if heal {
        data::Lane::Healing
    } else {
        data::Lane::LightArrow
    };
    let before = own_actor(&cs[i])
        .spells
        .histories
        .0
        .into_iter()
        .find(|s| s.lane == lane)
        .unwrap();
    pause(cs, d, 750);
    let target = Target::Honor { entity };
    let v = input(
        cs,
        i,
        d,
        if heal {
            Kind::Heal(target)
        } else {
            Kind::LightArrow(target)
        },
    );
    applied(&v);
    let after = own_actor(&cs[i])
        .spells
        .histories
        .0
        .into_iter()
        .find(|s| s.lane == lane)
        .unwrap();
    assert_eq!(after.ledger.casts, before.ledger.casts + 1);
    assert_eq!(
        after.ledger.spent,
        before.ledger.spent + i64::from(after.ledger.cost)
    );
}

#[test]
#[ignore = "v41 controllers/owned SQL/shared532: heal, real five-second life, tower phases, saved result/HUD/source return; no HWND"]
fn java_v41_honor_controllers_shared_heal_respawn_towers_saved_hud_and_return() {
    flow(false);
}
#[test]
#[ignore = "ordinary schema2 v47 + owned TLS/SQL/shared532: healing/respawn/towers/saved return; synthetic source, no HWND"]
fn java_ordinary_v47_honor_saved_and_return() {
    flow(true);
}
fn flow(ordinary: bool) {
    let d = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        if ordinary {
            "local.lito381.server.ModernOrdinaryHonorTestPeer"
        } else {
            "local.lito381.server.ModernHonorTestPeer"
        },
        "canonical-honor",
    );
    assert!(peer.process_job.is_some());
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
        wait(&mut cs, &d, "honor TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "honor login", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = select(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        assert_eq!(cs[i].view().record.totals().unwrap().revision, 0);
        enter(&mut cs, i, &d, contexts[i]);
        purchase(&mut cs, i, &d, contexts[i]);
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
                        mode: 5,
                        attributes: 16,
                        map_id: 532,
                        name: "Canonical shared532 Honor".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "honor creator joins", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
    let room = cs[0].view().rooms.joined().unwrap().clone();
    send(&mut cs, 1, &d, || {
        LoginCommand::Rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: room.summary.room_id,
                    expected_revision: room.summary.revision,
                    ..rooms::Common::read(contexts[1], CHANNEL)
                },
                String::new(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "honor two members", |cs| {
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
    }
    cs[0].load = true;
    wait(&mut cs, &d, "honor first qualified LOAD", |cs| {
        !cs[0].load
            && cs
                .iter()
                .all(|c| c.surface.as_ref().is_some_and(|s| s.objectives.len() == 2))
    });
    for c in &cs {
        let v = c.view();
        let o = v.canonical.observation().unwrap();
        let data = o.data();
        assert_eq!(data.world.phase, data::Phase::Loading);
        assert_eq!(data.world.descriptor.ruleset, data::Ruleset::HonorTowers);
        assert_eq!(
            (data.room.mode, data.room.map, data.manifest.map),
            (5, 532, 532)
        );
        assert_eq!(data.manifest.sectors.0.len(), 6);
        assert!(data
            .room
            .members
            .0
            .iter()
            .all(|m| !m.source_name.0.is_empty()));
        assert_eq!(tower(c, 3).hp, 3000);
        assert_eq!(tower(c, 4).hp, 3000);
        battle_window::validate(c.surface.as_ref().unwrap()).unwrap();
    }
    cs[1].load = true;
    wait(
        &mut cs,
        &d,
        "honor countdown and actual shared pixels",
        |cs| {
            cs.iter().all(|c| {
                c.rendered()
                    && c.view().canonical.observation().unwrap().data().world.phase
                        == data::Phase::Active
            })
        },
    );
    assert!(hud(&cs[0]).contains("己方隊0"));
    assert!(hud(&cs[1]).contains("己方隊1"));
    cast(&mut cs, 1, &d, true, 4);
    assert_eq!(tower(&cs[1], 4).hp, 3000);
    assert_eq!(tower(&cs[1], 4).heal_gain, 0);
    cast(&mut cs, 1, &d, false, 3);
    let damaged = tower(&cs[1], 3);
    assert!(damaged.hp < 3000 && damaged.damage_loss > 0);
    let mut heals = 0;
    while tower(&cs[0], 3).hp != 3000 || tower(&cs[0], 3).damage_loss != damaged.damage_loss {
        wait(&mut cs, &d, "honor tower damage follows both", |cs| {
            tower(&cs[0], 3).damage_loss == damaged.damage_loss
        });
        if tower(&cs[0], 3).hp == 3000 {
            break;
        }
        cast(&mut cs, 0, &d, true, 3);
        heals += 1;
        assert!(heals <= 20);
    }
    assert_eq!(tower(&cs[0], 3).heal_gain, damaged.damage_loss);
    pause(&mut cs, &d, 1050);
    wait(&mut cs, &d, "honor pre-kill current pixels", |cs| {
        cs.iter().all(Client::rendered)
    });
    let old_picture = cs[0].surface.as_ref().unwrap().clone();
    let old_skill_picture = cs[1].surface.as_ref().unwrap().clone();
    let old_skill_choice = crate::modern_lito::battle_skillbook::model(
        &old_skill_picture,
        &cs[1].view(),
        str::to_owned,
    )
    .unwrap()
    .choice(1)
    .expect("living owner's heal choice");
    let old_target = Target::Player { entity: 2, life: 1 };
    let stale = Intent {
        stamp: Stamp::of(&old_picture.observation),
        kind: Kind::Attack(old_target),
    };
    let old_ammo = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .personal()
        .ammunition()
        .cloned();
    let killed = input(&mut cs, 0, &d, Kind::Attack(old_target));
    applied(&killed);
    let killed_at = Instant::now();
    wait(&mut cs, &d, "honor death and waiting HUD", |cs| {
        state(&cs[1]).lives.0[1].death_count == 1
    });
    assert_eq!(own_actor(&cs[1]).vital.unwrap().values.hp, 0);
    assert!(crate::modern_lito::battle_skillbook::select(
        old_skill_choice,
        &old_skill_picture,
        &cs[1].view(),
    )
    .is_none());
    assert!(hud(&cs[1]).contains("等待伺服器自動復活"));
    assert!(!battle_input::ready(
        Kind::Local(Local::Step(0)),
        &cs[1].view()
    ));
    assert_eq!(
        cs[1]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .world
            .phase,
        data::Phase::Active
    );
    assert!(cs[1]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .result
        .is_none());
    let dead_life = state(&cs[1]).lives.0[1].clone();
    assert!(dead_life.pending_due_millis >= 5000);
    wait(
        &mut cs,
        &d,
        "honor actual five-second server respawn",
        |cs| {
            cs.iter().all(|c| state(c).lives.0[1].respawn_count == 1)
                && cs.iter().all(Client::rendered)
        },
    );
    assert!(killed_at.elapsed() >= Duration::from_millis(4500));
    assert_eq!(own_actor(&cs[1]).vital.unwrap().values.hp, 40);
    assert_eq!(state(&cs[1]).lives.0[1].pending_due_millis, 0);
    assert!(hud(&cs[1]).contains("死亡1／已復活1"));
    assert!(!hud(&cs[1]).contains("等待伺服器自動復活"));
    assert_eq!(
        Stamp::of(cs[1].view().canonical.observation().unwrap()).life,
        2
    );
    assert!(!battle_window::target_current(
        cs[0].view().canonical.observation().unwrap(),
        old_target
    ));
    assert!(battle_input::command(&old_picture, stale, &cs[0].view()).is_err());
    assert!(crate::modern_lito::battle_skillbook::select(
        old_skill_choice,
        &old_skill_picture,
        &cs[1].view(),
    )
    .is_none());
    let revived_picture = cs[1].surface.as_ref().unwrap();
    assert!(crate::modern_lito::battle_skillbook::select(
        old_skill_choice,
        revived_picture,
        &cs[1].view(),
    )
    .is_none());
    assert!(crate::modern_lito::battle_skillbook::model(
        revived_picture,
        &cs[1].view(),
        str::to_owned,
    )
    .unwrap()
    .choice(1)
    .is_some());
    assert_ne!(
        cs[0]
            .view()
            .canonical
            .observation()
            .unwrap()
            .personal()
            .ammunition(),
        old_ammo.as_ref()
    );
    // The newly revived owner can actually cast, not merely display nonzero HP.
    cast(&mut cs, 1, &d, true, 4);
    cast(&mut cs, 0, &d, false, 4);
    let mut phases = std::collections::BTreeSet::from([0]);
    let mut hits = 0;
    loop {
        if tower(&cs[0], 4).hp == 0 {
            break;
        }
        pause(&mut cs, &d, 1050);
        let v = input(&mut cs, 0, &d, Kind::Attack(Target::Honor { entity: 4 }));
        applied(&v);
        hits += 1;
        assert!(hits <= 60, "unchanged +100 ranged source fixture bound");
        let t = tower(&cs[0], 4);
        phases.insert(t.action);
        wait(&mut cs, &d, "honor objective shared damage art", |cs| {
            cs[0].surface.as_ref().is_some_and(|s| {
                s.observation.data().honor.objectives.0[1].revision == t.revision
                    && s.objectives
                        .iter()
                        .any(|(id, _)| *id == ObjectiveTarget::Honor { entity: 4, team: 1 })
            })
        });
    }
    assert_eq!(
        phases,
        std::collections::BTreeSet::from([0, 32, 33, 34, 35])
    );
    wait(
        &mut cs,
        &d,
        "honor saved terminal and shared collapse",
        |cs| {
            cs.iter().all(|c| {
                c.view().canonical.lifecycle().is_some_and(|l| {
                    l.phase == world::Phase::Finished
                        && l.settlement == Some(world::SettlementState::Saved)
                }) && c.surface.as_ref().is_some_and(|s| {
                    s.objectives.iter().any(|(id, art)| {
                        *id == ObjectiveTarget::Honor { entity: 4, team: 1 } && art.subframe == 20
                    })
                })
            })
        },
    );
    assert!(tower(&cs[0], 3).hp > 0);
    assert!(own_actor(&cs[1]).vital.unwrap().values.hp > 0);
    assert!(hud(&cs[0]).contains("本場勝利"));
    assert!(hud(&cs[1]).contains("本場敗北"));
    for c in &cs {
        assert!(crate::modern_lito::battle_skillbook::model(
            c.surface.as_ref().unwrap(),
            &c.view(),
            str::to_owned,
        )
        .unwrap()
        .rows
        .iter()
        .all(|r| r.choice.is_none()));
    }
    let view = cs[0].view();
    let raw = view.canonical.observation().unwrap().data();
    let world_id = raw.world.identity.clone();
    let result = raw.result.as_ref().unwrap();
    assert_eq!((result.mode, result.map), (5, 532));
    assert_eq!(
        cs[1]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .result
            .as_ref(),
        Some(result)
    );
    for i in 0..2 {
        let leave = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&world_id))
        });
        send(&mut cs, i, &d, || LoginCommand::Canonical(leave.clone()));
        wait(&mut cs, &d, "honor confirmed world retirement", |cs| {
            !cs[i].view().world_pending
                && !cs[i].view().canonical.attached()
                && cs[i].runtime.is_quiescent()
        });
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        send(&mut cs, i, &d, || LoginCommand::LeaveLobby(contexts[i]));
        wait(&mut cs, &d, "honor channel return", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingChannel
        });
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "honor roster return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
        assert!(cs[i].surface.is_none());
    }
    // Both release their identities first; equal object IDs in different
    // sources then intentionally swap owners without mixing totals.
    for i in 0..2 {
        let source = if i == 0 { 42 } else { 7 };
        let selected = select(&mut cs, i, &d, source);
        assert!(selected.epoch > contexts[i].epoch);
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: i64::from(source == 7),
                losses: i64::from(source == 42),
                revision: 1
            })
        );
        send(&mut cs, i, &d, || LoginCommand::ReadRecord);
        wait(&mut cs, &d, "honor idempotent SQL reread", |cs| {
            !cs[i].view().record_pending
        });
        assert_eq!(cs[i].view().record.totals().unwrap().revision, 1);
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "honor normal controller shutdown", |cs| {
        cs.iter().all(|c| {
            c.runtime.is_quiescent()
                && c.controller.is_quiescent()
                && c.view().phase == LoginPhase::Disconnected
        })
    });
    peer.expect_successful_exit(&d);
}
