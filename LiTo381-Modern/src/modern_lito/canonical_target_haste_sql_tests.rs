//! Production Main + owned real SQL + two v44 controllers + shared514/SPR.
//! Exact painted inputs, not a native HWND/click or release-delivery claim.
use super::*;
use crate::modern_lito::{battle_target_haste as haste, spell_protocol::SkillsState};

pub(super) fn saved(c: &Client, context: SelectionContext) {
    let v = c.view();
    let CharacterDataState::Ready(s) = &v.character_data else {
        panic!("SQL character missing")
    };
    let sun = context.key.source_id == 7;
    assert_eq!(s.context, context);
    assert_eq!(s.stats.intelligence, if sun { 5_000_000_000 } else { 12 });
    assert_eq!(
        s.stats.experience,
        if sun {
            9_007_199_254_740_993
        } else {
            9_007_199_254_740_995
        }
    );
    assert_eq!((s.stats.max_hp, s.stats.max_mp), (1000, 300));
    assert_eq!(s.stats.food225, if sun { 225 } else { 180 });
    assert_eq!(s.items.len(), if sun { 3 } else { 2 });
    assert_eq!(s.items[1].quantity, if sun { 2 } else { 5 });
    assert_eq!(gear_snapshot(c).source.context, context);
    assert_eq!(gear_snapshot(c).equipped(1001), Some(true));
    let SkillsState::Ready(skills) = &v.skills else {
        panic!("SQL learned catalog missing")
    };
    assert_eq!(skills.context, context);
    assert_eq!(skills.haste().is_some(), sun);
}

pub(super) fn select(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    source_id: u32,
) -> SelectionContext {
    send(cs, i, d, || LoginCommand::SelectSource(source_id));
    wait(cs, d, "Haste43 SQL roster", |cs| {
        cs[i].view().phase == LoginPhase::CharactersReady
    });
    let v = cs[i].view();
    assert_eq!(v.selected_source, Some(source_id));
    assert_eq!(v.characters.len(), 1);
    assert_eq!(v.characters[0].object_id, 3333);
    let context = SelectionContext {
        key: CharacterKey {
            source_id,
            object_id: 3333,
        },
        epoch: v.roster_epoch,
    };
    send(cs, i, d, || LoginCommand::SelectCharacter(context));
    wait(cs, d, "Haste43 SQL source qualification", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::ChoosingChannel
            && !v.record_pending
            && !v.spell_pending
            && matches!(v.equipment, equipment::State::Ready(_))
    });
    saved(&cs[i], context);
    context
}

fn lane(c: &Client) -> data::Spell {
    own_actor(c)
        .spells
        .histories
        .0
        .into_iter()
        .find(|s| s.lane == data::Lane::Haste)
        .expect("v44 Haste43 ledger")
}

fn target(c: &Client) -> crate::modern_lito::battle_window::Target {
    let a = own_actor(c);
    crate::modern_lito::battle_window::Target::Player {
        entity: a.entity,
        life: a.resources.as_ref().unwrap().life + 1,
    }
}
fn proof(c: &Client) -> haste::Intent {
    haste::Intent::from_observation(c.view().canonical.observation().unwrap(), target(c)).unwrap()
}
fn cast(
    cs: &mut [Client; 2],
    d: &Deadline,
    target_index: usize,
    count: i64,
    cost: i32,
    spent: i64,
) {
    wait(cs, d, "Haste43 SQL current pixels", |cs| {
        cs.iter().all(|c| c.rendered())
    });
    let before = own_actor(&cs[0]);
    let t = target(&cs[target_index]);
    let old =
        haste::Intent::from_observation(cs[0].view().canonical.observation().unwrap(), t).unwrap();
    assert!(crate::modern_lito::battle_window::skill_status(
        cs[0].view().canonical.observation().unwrap(),
        crate::modern_lito::battle_window::Skill::Haste
    )
    .unwrap()
    .contains(&format!("MP {cost}")));
    let v = input(cs, 0, d, Kind::Haste(t));
    assert_eq!(
        v.canonical.outcome(),
        Some(Outcome::TargetHaste(haste::Reason::None))
    );
    let accepted = own_actor(&cs[0]);
    let h = lane(&cs[0]);
    assert_eq!(
        (h.ledger.casts, h.ledger.cost, h.ledger.spent),
        (count, cost, spent)
    );
    assert_eq!(accepted.spells.sequence, count);
    assert_eq!(accepted.vital.as_ref().unwrap().values.mp, 300 - spent);
    let r = h.receipt.as_ref().unwrap();
    assert_eq!(
        r.impact.usage.before.vitals.mp,
        before.vital.as_ref().unwrap().values.mp
    );
    assert_eq!(r.impact.usage.mp_after, 300 - spent);
    assert!(r.impact.targets.0.is_empty());
    let data::Detail::Haste(detail) = &r.detail else {
        panic!("targeted receipt")
    };
    let crate::modern_lito::battle_payload::haste::StatusTarget::Player(p) = &detail.target else {
        panic!("player target")
    };
    assert_eq!(p.participant.entity, own_actor(&cs[target_index]).entity);
    assert_eq!(
        p.participant.identity.source,
        if target_index == 0 { 7 } else { 42 }
    );
    assert_eq!(p.participant.identity.id, 3333);
    assert_eq!(p.after.haste_origin, data::HasteOrigin::Haste);
    assert_eq!(
        p.after.haste_until_nanos - detail.active_elapsed_nanos,
        1_200_000_000_000
    );
    wait(
        cs,
        d,
        "Haste43 SQL exact cast and target on both surfaces",
        |cs| {
            cs.iter().all(|c| {
                c.rendered()
                    && c.surface.as_ref().is_some_and(|s| {
                        let actors = &s.observation.data().actors.0;
                        s.observation.data().world.revision >= r.impact.usage.revision
                            && actors.iter().any(|a| {
                                a.entity == accepted.entity
                                    && a.vital.as_ref().unwrap().values.mp == 300 - spent
                                    && a.spells.histories.0.iter().any(|x| {
                                        x.lane == data::Lane::Haste && x.receipt == h.receipt
                                    })
                            })
                            && actors.iter().any(|a| {
                                a.entity == p.participant.entity
                                    && a.buffs.as_ref().is_some_and(|b| {
                                        b.revision == p.after.revision
                                            && b.haste_origin == p.after.haste_origin
                                            && b.haste_until_nanos == p.after.haste_until_nanos
                                    })
                            })
                    })
            })
        },
    );
    if count == 1 {
        assert_eq!(
            own_actor(&cs[0]).buffs.as_ref().unwrap().haste_until_nanos,
            0,
            "targeting Moon must not accelerate Sun"
        );
    }
    assert_eq!(own_actor(&cs[1]).vital.unwrap().values.mp, 300);
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::target_haste(old)))
        .is_err());
    wait(cs, d, "Haste43 SQL cast recovery", |cs| {
        cs[0].rendered()
            && cs[0]
                .view()
                .canonical
                .observation()
                .unwrap()
                .data()
                .world
                .loading_elapsed_nanos
                >= r.loading_elapsed_nanos + 700_000_000
    });
}

fn helm(cs: &mut [Client; 2], d: &Deadline, desired: bool, cost: i32) {
    wait(cs, d, "Haste43 SQL inventory pixels", |cs| cs[0].rendered());
    let v = cs[0].view();
    let s = cs[0].surface.clone().unwrap();
    let original = gear_snapshot(&cs[0]);
    let old_cast = proof(&cs[0]);
    let prior_cast = lane(&cs[0]);
    let prior_mp = own_actor(&cs[0]).vital.unwrap().values.mp;
    let model = battle_inventory::model(&s, &v, str::to_owned).unwrap();
    let click = model.request(1003, desired).unwrap();
    let q = battle_inventory::command(click, &s.observation, &v).unwrap();
    let receipt_epoch = v.canonical.receipt_epoch();
    send(cs, 0, d, || LoginCommand::Canonical(q.clone()));
    assert!(
        Arc::ptr_eq(&original, &gear_snapshot(&cs[0])),
        "no optimistic equipment"
    );
    wait(cs, d, "Haste43 SQL SET receipt", |cs| {
        cs[0].rendered() && gear_snapshot(&cs[0]).revision == original.revision + 1
    });
    let v = cs[0].view();
    assert_eq!(
        v.canonical.received_after(&q, receipt_epoch),
        Some(Outcome::Equipment(
            gear::Reason::None,
            gear::Disposition::Applied,
            Some(equipment::Status::Ok)
        ))
    );
    assert_eq!(gear_snapshot(&cs[0]).equipped(1003), Some(desired));
    assert_eq!(original.equipped(1003), Some(!desired));
    assert_eq!(own_actor(&cs[0]).vital.unwrap().values.mp, prior_mp);
    assert_eq!(lane(&cs[0]).ledger.cost, cost);
    assert_eq!(lane(&cs[0]).ledger.spent, prior_cast.ledger.spent);
    assert_eq!(
        lane(&cs[0]).receipt,
        prior_cast.receipt,
        "SET must not reprice historical casts"
    );
    assert!(battle_inventory::command(click, &s.observation, &v).is_none());
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::target_haste(old_cast)))
        .is_err());
}

#[test]
#[ignore = "Main + owned SELECT-only SQL + v44 controllers/shared514: INT5e9, helm15/33 cost, other-source/self casts, move/abort/source reload; no HWND"]
fn java_v44_main_sql_target_haste_wide_stats_live_gear_cast_move_and_source_return() {
    let d = Deadline(Instant::now() + Duration::from_secs(150));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernHasteSqlTestPeer",
        "target-haste-sql",
        Some("150000"),
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
        wait(&mut cs, &d, "Haste43 SQL TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "Haste43 SQL authentication", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = select(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
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
                        capacity: 2,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "Main SQL target Haste".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "Haste43 SQL room created", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
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
    wait(&mut cs, &d, "Haste43 SQL room joined", |cs| {
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
    wait(&mut cs, &d, "Haste43 SQL shared world loaded", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    assert_eq!(gear_snapshot(&cs[0]).equipped(1003), Some(true));
    assert_eq!(lane(&cs[0]).ledger.cost, 15);
    assert!(battle_input::skill_available(43, &cs[0].view()));
    assert!(!battle_input::skill_available(43, &cs[1].view()));
    assert!(!battle_input::ready(
        Kind::Haste(target(&cs[1])),
        &cs[1].view()
    ));
    let stale_source = proof(&cs[0]);
    cast(&mut cs, &d, 1, 1, 15, 15);
    helm(&mut cs, &d, false, 33);
    cast(&mut cs, &d, 0, 2, 33, 48);
    helm(&mut cs, &d, true, 15);
    cast(&mut cs, &d, 1, 3, 15, 63);
    let a = own_actor(&cs[0]);
    let from = WorldCell {
        x: a.position.x,
        y: a.position.y,
    };
    let s = cs[0].surface.as_ref().unwrap();
    let (heading, to) = DIRECTIONS
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
                    .all(|a| a.position.x != to.x || a.position.y != to.y))
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
    assert_eq!(
        own_actor(&cs[0]).last_move_sequence,
        a.last_move_sequence + 1
    );
    wait(&mut cs, &d, "Haste43 SQL moved pixels", |cs| {
        cs[0].rendered()
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
    for i in 0..2 {
        let q = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&world))
        });
        send(&mut cs, i, &d, || LoginCommand::Canonical(q.clone()));
        wait(&mut cs, &d, "Haste43 SQL battle return", |cs| {
            !cs[i].view().world_pending
                && !cs[i].view().canonical.attached()
                && cs[i].runtime.is_quiescent()
        });
        assert!(cs[i].surface.is_none());
        assert!(!battle_input::skill_available(43, &cs[i].view()));
        send(&mut cs, i, &d, || LoginCommand::ReadRecord);
        wait(&mut cs, &d, "Haste43 SQL no false win", |cs| {
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
        wait(&mut cs, &d, "Haste43 SQL character return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    let moon = select(&mut cs, 0, &d, 42);
    assert_ne!(moon, contexts[0]);
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::target_haste(stale_source)))
        .is_err());
    enter(&mut cs, 0, &d, moon);
    assert!(matches!(cs[0].view().skills, SkillsState::Ready(ref s) if s.haste().is_none()));
    assert!(gear_snapshot(&cs[0]).equipped(1003).is_none());
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(
        &mut cs,
        &d,
        "Haste43 SQL controller and resource shutdown",
        |cs| {
            cs.iter()
                .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
        },
    );
    peer.expect_successful_exit(&d);
}
