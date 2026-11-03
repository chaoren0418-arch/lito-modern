//! Mixed skill1 domains on one production Main and the same two ordinary TLS
//! owners. Real shared532/514, SELECT-only sources, GM capacity and return.
use super::*;

fn room(cs: &mut [Client; 2], d: &Deadline, contexts: [SelectionContext; 2], honor: bool) {
    send(cs, 0, d, || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(contexts[0], CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: if honor { 4 } else { 2 },
                        mode: if honor { 5 } else { 1 },
                        map_id: if honor { 532 } else { 514 },
                        attributes: 0,
                        name: "Mixed healing Main SQL".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(cs, d, "mixed healing creator joined", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
    let r = cs[0].view().rooms.joined().unwrap().summary.clone();
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
    wait(cs, d, "mixed healing both members", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    room_action(cs, 1, d, contexts[1], rooms::Action::Team, 1);
    if honor {
        let v = bots::gm(cs, 0, d, ".de 20");
        let r = v.rooms.joined().unwrap();
        assert_eq!(
            r.members.len(),
            4,
            "GM request is clamped to actual remaining capacity"
        );
        for team in [0, 1] {
            assert_eq!(r.members.iter().filter(|m| m.team == team).count(), 2);
        }
        let bots: Vec<_> = r
            .members
            .iter()
            .filter(|m| m.kind == rooms::MemberKind::ServerBot)
            .collect();
        assert_eq!(bots.len(), 2);
        for b in bots {
            assert!(b.ready);
            assert_eq!(b.identity().source_id, 7);
            assert_eq!(b.source_name, "太陽神");
            assert!(
                r.member(b.key).is_none(),
                "typed BOT is never a SQL character lookup"
            );
        }
    }
    for i in 0..2 {
        room_action(cs, i, d, contexts[i], rooms::Action::Ready, 1);
        cs[i].load = true;
    }
    wait(
        cs,
        d,
        "mixed healing automatic start and shared LOAD",
        |cs| {
            cs.iter().all(|c| {
                c.rendered()
                    && c.view().canonical.observation().unwrap().data().world.phase
                        == data::Phase::Active
            })
        },
    );
    for c in cs.iter() {
        let v = c.view();
        let data = v.canonical.observation().unwrap().data();
        assert_eq!(data.actors.0.len(), if honor { 4 } else { 2 });
        assert_eq!(
            data.healing.as_ref().unwrap().actors.0.len(),
            data.actors.0.len()
        );
        assert_eq!(data.honor.objectives.0.len(), if honor { 2 } else { 0 });
        assert!(data
            .actors
            .0
            .iter()
            .all(|a| a.loaded && a.spells.sequence == 0));
        assert!(data
            .healing
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .all(|a| a.incoming.received == 0
                && a.outgoing.as_ref().is_none_or(|o| o.ledger.casts == 0)));
    }
}

pub(super) fn recovery(cs: &mut [Client; 2], d: &Deadline, i: usize) {
    let a = own_actor(&cs[i]);
    let next = cs[i]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .loading_elapsed_nanos
        + crate::modern_lito::battle_motion::paced_nanos(&a, a.cast_interval_nanos as u64, false)
            as i64;
    wait(cs, d, "mixed healing confirmed cast recovery", |cs| {
        cs[i].rendered()
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

fn ledgers(c: &Client, player: i64, tower: i64, sequence: i64, spent: i64) {
    let v = c.view();
    let o = v.canonical.observation().unwrap();
    let a = own_actor(c);
    let p = o
        .data()
        .healing
        .as_ref()
        .unwrap()
        .actors
        .0
        .iter()
        .find(|h| h.entity == a.entity)
        .unwrap();
    let h = a
        .spells
        .histories
        .0
        .iter()
        .find(|h| h.lane == data::Lane::Healing);
    assert_eq!(p.outgoing.as_ref().unwrap().ledger.casts, player);
    assert_eq!(h.map_or(0, |h| h.ledger.casts), tower);
    assert_eq!(a.spells.sequence, sequence);
    assert_eq!(
        p.outgoing.as_ref().unwrap().ledger.spent + h.map_or(0, |h| h.ledger.spent),
        spent
    );
}

fn tower_hp(c: &Client, entity: i64) -> i64 {
    c.view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .honor
        .objectives
        .0
        .iter()
        .find(|t| t.entity == entity)
        .unwrap()
        .hp
}

pub(super) fn leave(cs: &mut [Client; 2], d: &Deadline, contexts: [SelectionContext; 2]) {
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
        cs[i].load = false;
        let q = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&world))
        });
        send(cs, i, d, || LoginCommand::Canonical(q.clone()));
        wait(cs, d, "mixed healing world retired", |cs| {
            !cs[i].view().world_pending
                && !cs[i].view().canonical.attached()
                && cs[i].runtime.is_quiescent()
        });
    }
    for i in (0..2).rev() {
        room_action(cs, i, d, contexts[i], rooms::Action::Leave, 0);
    }
}

#[test]
#[ignore = "one production Main/owned SQL, actual shared532/514 and two ordinary extended workers: tower/player/BOT healing, GM clamp, mode/source return; not HWND"]
fn java_healing_main_sql_ordinary_mixed_honor_bots_and_dm_return() {
    let d = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernPlayerHealingSqlTestPeer",
        "healing-mixed-sql",
        Some("180000"),
    );
    let profile = healing_profile(peer.endpoint, &shared, &fixtures);
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
        wait(&mut cs, &d, "mixed healing TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "mixed healing authenticated", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = select_healer(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        enter(&mut cs, i, &d, contexts[i]);
    }
    room(&mut cs, &d, contexts, true);
    let v = cs[0].view();
    let o = v.canonical.observation().unwrap();
    let tower = o
        .data()
        .honor
        .objectives
        .0
        .iter()
        .find(|t| t.team == 0)
        .unwrap()
        .entity;
    let bot = o
        .data()
        .actors
        .0
        .iter()
        .find(|a| {
            let m = o.data().scene_member(a).unwrap();
            m.identity.kind == data::MemberKind::ServerBot && m.team == 0
        })
        .unwrap()
        .entity;
    let reply = input(
        &mut cs,
        1,
        &d,
        Kind::LightArrow(Target::Honor { entity: tower }),
    );
    assert_eq!(
        reply.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    wait(
        &mut cs,
        &d,
        "mixed tower damage reaches both owners",
        |cs| cs.iter().all(|c| tower_hp(c, tower) < 3000),
    );
    let hurt = tower_hp(&cs[0], tower);
    let reply = input(&mut cs, 0, &d, Kind::Heal(Target::Honor { entity: tower }));
    assert_eq!(
        reply.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    assert!(tower_hp(&cs[0], tower) > hurt);
    ledgers(&cs[0], 0, 1, 1, 0);
    recovery(&mut cs, &d, 1);
    let paid = heal(&mut cs, 1, 1, &d);
    ledgers(&cs[1], 1, 0, 2, 4); // skill4 sequence + participant heal, independent Honor lane.
    recovery(&mut cs, &d, 0);
    let old = heal_entity(&mut cs, 0, bot, &d);
    assert_eq!(
        old.target.identity.kind,
        crate::modern_lito::participant::MemberKind::ServerBot
    );
    assert_eq!(old.target.identity.source_id, 7);
    ledgers(&cs[0], 1, 1, 2, 0);
    recovery(&mut cs, &d, 0);
    let reply = input(&mut cs, 0, &d, Kind::Heal(Target::Honor { entity: tower }));
    assert_eq!(
        reply.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    ledgers(&cs[0], 1, 2, 3, 0);
    recovery(&mut cs, &d, 0);
    let cross = heal(&mut cs, 0, 1, &d);
    assert_eq!(cross.target.identity.source_id, 42);
    ledgers(&cs[0], 2, 2, 4, 0);
    leave(&mut cs, &d, contexts);
    room(&mut cs, &d, contexts, false);
    for (i, stale) in [(0, old), (1, paid)] {
        assert!(cs[i]
            .controller
            .command(LoginCommand::Canonical(Command::player_healing(stale)))
            .is_err());
    }
    heal(&mut cs, 1, 1, &d);
    ledgers(&cs[1], 1, 0, 1, 4);
    leave(&mut cs, &d, contexts);
    for i in 0..2 {
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "mixed healing return characters", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    // Both source-scoped characters must be released before either owner
    // selects the other's character. Room/world departure alone is not logout.
    for i in 0..2 {
        let context = select_healer(&mut cs, i, &d, if i == 0 { 42 } else { 7 });
        assert_ne!(context, contexts[i]);
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "mixed healing all owners retired", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
    eprintln!("HEALING_MIXED_MAIN_SQL: one Main/two ordinary extended TLS owners, real shared532 Honor+514 DM, capacity4 .de20 adds2 balanced source-named passive BOTs, automatic LOAD/start, injured tower repair, player/BOT/cross-source heals, independent ledgers/shared sequence, 744/action19/109 pixels/cues, stale old-match rejection, fresh mode and source reload; imported SQL unchanged, zero false results, no native HWND claim");
}
