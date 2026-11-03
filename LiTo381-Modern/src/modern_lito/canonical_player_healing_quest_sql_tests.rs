//! Actual Quest audiences through ordinary workers/Main/SQL/shared554. A
//! visible room roster is not permission to render or heal another instance.
use super::*;

fn isolated(c: &Client, lane: i32) {
    let v = c.view();
    let o = v.canonical.observation().unwrap();
    let data = o.data();
    assert_eq!(data.world.lane, lane);
    assert_eq!(data.room.members.0.len(), 4);
    let visible: BTreeSet<_> = data.actors.0.iter().map(|a| a.entity).collect();
    assert_eq!(visible.len(), 2);
    assert_eq!(
        data.healing
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .map(|h| h.entity)
            .collect::<BTreeSet<_>>(),
        visible
    );
    let surface = c.surface.as_ref().unwrap();
    battle_window::validate(surface).unwrap();
    assert_eq!(
        surface
            .geometry
            .actors
            .iter()
            .map(|p| p.entity_id as i64)
            .collect::<BTreeSet<_>>(),
        visible
    );
    for a in &data.actors.0 {
        let m = data.scene_member(a).unwrap();
        assert_eq!(m.team, lane);
        assert!(a.loaded);
        if m.identity.kind == data::MemberKind::ServerBot {
            assert_eq!(m.source_name.0, "太陽神");
            assert_eq!(
                a.spells.sequence, 0,
                "passive BOT has no SQL or cast authority"
            );
            assert!(data
                .healing
                .as_ref()
                .unwrap()
                .actors
                .0
                .iter()
                .find(|h| h.entity == a.entity)
                .unwrap()
                .outgoing
                .is_none());
        }
    }
    assert!(data
        .npcs
        .as_ref()
        .unwrap()
        .actors
        .0
        .iter()
        .all(|n| n.lane == lane));
}

fn bot(c: &Client) -> i64 {
    let v = c.view();
    let o = v.canonical.observation().unwrap();
    o.data()
        .actors
        .0
        .iter()
        .find(|a| o.data().scene_member(a).unwrap().identity.kind == data::MemberKind::ServerBot)
        .unwrap()
        .entity
}

fn foreign_rejected(cs: &mut [Client; 2]) {
    for i in 0..2 {
        let own = cs[i].view();
        let other = cs[1 - i].view();
        let o = own.canonical.observation().unwrap();
        let foreign = other.canonical.observation().unwrap();
        for entity in [own_actor(&cs[1 - i]).entity, bot(&cs[1 - i])] {
            assert!(healing::Intent::from_observation(o, entity).is_none());
            let captured = healing::Intent::from_observation(foreign, entity).unwrap();
            let forged = healing::Intent {
                target: captured.target,
                ..healing::Intent::from_observation(o, own_actor(&cs[i]).entity).unwrap()
            };
            assert!(
                forged.validate().is_ok(),
                "well-shaped foreign target must fail current authority, not wire shape"
            );
            assert!(!forged.current(o));
            let intent = Intent {
                stamp: Stamp::of(o),
                kind: Kind::Heal(Target::Player {
                    entity,
                    life: captured.target.life,
                }),
            };
            assert!(battle_input::target_command(intent, &own).is_err());
            assert!(battle_input::command(cs[i].surface.as_ref().unwrap(), intent, &own).is_err());
            assert!(cs[i]
                .controller
                .command(LoginCommand::Canonical(Command::player_healing(forged)))
                .is_err());
            assert!(!cs[i].view().world_pending);
            // command() polls the real worker first. NPC movement, injury and
            // natural recovery may legitimately advance; compare only the
            // healing authority that this denied command must not change.
            let latest = cs[i].view();
            let current = latest.canonical.observation().unwrap();
            assert_eq!(
                current.data().healing.as_ref().unwrap().actors,
                o.data().healing.as_ref().unwrap().actors
            );
            assert_eq!(own_actor(&cs[i]).spells.sequence, forged.sequence - 1);
        }
    }
}

fn room(
    cs: &mut [Client; 2],
    d: &Deadline,
    contexts: [SelectionContext; 2],
    swapped: bool,
) -> BTreeSet<u64> {
    send(cs, 0, d, || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(contexts[0], CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 4,
                        mode: 2,
                        map_id: 554,
                        attributes: 16,
                        name: "Healing private Quest lanes".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(cs, d, "healing Quest creator", |cs| {
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
    wait(cs, d, "healing Quest both owners", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    for i in 0..2 {
        room_action(
            cs,
            i,
            d,
            contexts[i],
            rooms::Action::Team,
            (if swapped { 1 - i } else { i }) as u8,
        );
    }
    // The other owner's team mutation has its own room revision. Obtain it
    // before submitting the one immutable GM request; never retry a mutation
    // whose result is unknown or bypass the server's revision fence.
    let team_revision = cs[1].view().rooms.joined().unwrap().summary.revision;
    let old_revision = cs[0].view().rooms.joined().unwrap().summary.revision;
    send(cs, 0, d, || {
        LoginCommand::Rooms(rooms::Request::read(contexts[0], CHANNEL).unwrap())
    });
    wait(
        cs,
        d,
        "healing Quest creator observes both team choices",
        |cs| {
            !cs[0].view().room_pending
                && cs[0]
                    .view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.summary.revision >= team_revision)
        },
    );
    eprintln!("HEALING_QUEST_TEAM_READ swapped={swapped} before={old_revision} peer={team_revision} after={}", cs[0].view().rooms.joined().unwrap().summary.revision);
    let created = bots::gm(cs, 0, d, ".de 20");
    let rooms::State::Ready(reply) = &created.rooms else {
        panic!("Quest GM reply")
    };
    assert_eq!(
        (reply.status, reply.reason),
        (rooms::Status::Ok, rooms::Reason::None)
    );
    let r = created.rooms.joined().unwrap();
    assert_eq!(r.members.len(), 4);
    for team in [0, 1] {
        assert_eq!(r.members.iter().filter(|m| m.team == team).count(), 2);
    }
    let ids = r
        .members
        .iter()
        .filter(|m| m.kind == rooms::MemberKind::ServerBot)
        .map(|m| {
            assert!(m.ready);
            assert_eq!(m.source_name, "太陽神");
            m.identity().id
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 2);
    for i in 0..2 {
        room_action(cs, i, d, contexts[i], rooms::Action::Ready, 1);
        cs[i].load = true;
    }
    wait(
        cs,
        d,
        "healing Quest real shared LOAD and auto start",
        |cs| {
            cs.iter().all(|c| {
                c.rendered()
                    && c.view().canonical.observation().unwrap().data().world.phase
                        == data::Phase::Active
            })
        },
    );
    for i in 0..2 {
        isolated(&cs[i], (if swapped { 1 - i } else { i }) as i32);
        let v = cs[i].view();
        let o = v.canonical.observation().unwrap();
        assert!(o
            .data()
            .healing
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .all(|h| h.incoming.received == 0
                && h.outgoing.as_ref().is_none_or(|h| h.ledger.casts == 0)));
        assert_eq!(own_actor(&cs[i]).spells.sequence, 0);
        if let Some(n) = o
            .data()
            .npcs
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .find(|n| n.hp > 0)
        {
            assert!(battle_input::target_command(
                Intent {
                    stamp: Stamp::of(o),
                    kind: Kind::Heal(Target::Npc {
                        serial: n.serial,
                        life: n.life
                    })
                },
                &v
            )
            .is_err());
        }
    }
    foreign_rejected(cs);
    ids
}

fn record(c: &Client, winner: bool) {
    assert_eq!(
        c.view().record.totals(),
        Some(crate::modern_lito::record_protocol::Totals {
            stored: true,
            wins: i64::from(winner),
            losses: i64::from(!winner),
            revision: 1,
        })
    );
}

#[test]
#[ignore = "ordinary extended workers/Main/owned SQL/shared554: two Quest lanes, private BOT/player heals and audio, four waves/saved results, swapped-team restart; not HWND"]
fn java_healing_main_sql_ordinary_quest_isolation_saved_and_swapped_return() {
    let d = Deadline(Instant::now() + Duration::from_secs(240));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernPlayerHealingSqlTestPeer",
        "healing-quest-sql",
        Some("240000"),
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
        wait(&mut cs, &d, "healing Quest extended TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "healing Quest source selection", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = select_healer_hp(&mut cs, i, &d, if i == 0 { 7 } else { 42 }, 1000);
        enter(&mut cs, i, &d, contexts[i]);
    }
    let first_bots = room(&mut cs, &d, contexts, false);
    wait(
        &mut cs,
        &d,
        "both players injured by actual Quest NPCs",
        |cs| cs.iter().all(|c| hp(c) > 0 && hp(c) < 1000),
    );
    for i in 0..2 {
        waves(&mut cs, i, &d, false);
    }
    wait(&mut cs, &d, "both objective stages rendered", |cs| {
        cs.iter().all(Client::rendered)
    });
    for i in 0..2 {
        isolated(&cs[i], i as i32);
    }
    foreign_rejected(&mut cs);
    mixed::recovery(&mut cs, &d, 1);
    let damaged = hp(&cs[1]);
    let paid = heal(&mut cs, 1, 1, &d);
    assert!(
        hp(&cs[1]) > damaged,
        "ordinary heal restores real NPC injury"
    );
    mixed::recovery(&mut cs, &d, 0);
    let own_bot = bot(&cs[0]);
    let free = heal_entity(&mut cs, 0, own_bot, &d);
    mixed::recovery(&mut cs, &d, 1);
    let other_bot = bot(&cs[1]);
    let bot_paid = heal_entity(&mut cs, 1, other_bot, &d);
    assert_ne!(free.target.identity.id, bot_paid.target.identity.id);
    for i in 0..2 {
        isolated(&cs[i], i as i32);
        let v = cs[i].view();
        let o = v.canonical.observation().unwrap();
        let ledger = o
            .data()
            .healing
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .find(|h| h.entity == o.data().personal.entity)
            .unwrap()
            .outgoing
            .as_ref()
            .unwrap();
        assert_eq!(
            (ledger.ledger.casts, ledger.ledger.spent),
            if i == 0 { (1, 0) } else { (2, 8) }
        );
        assert_eq!(
            own_actor(&cs[i]).spells.sequence,
            if i == 0 { 1 } else { 2 }
        );
    }
    foreign_rejected(&mut cs);
    let target = cs[0]
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
    mixed::recovery(&mut cs, &d, 0);
    let result = input(
        &mut cs,
        0,
        &d,
        Kind::Attack(Target::Quest {
            slot: target.id.slot,
            born: target.born_revision,
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
    wait(
        &mut cs,
        &d,
        "healing Quest both durable results rendered",
        |cs| {
            cs.iter().all(|c| {
                c.view().canonical.lifecycle().is_some_and(|l| {
                    l.phase == world::Phase::Finished
                        && l.settlement == Some(world::SettlementState::Saved)
                }) && c
                    .surface
                    .as_ref()
                    .is_some_and(|s| s.observation.data().result.is_some())
            })
        },
    );
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
        (2, 554, 4)
    );
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
    let mut result_bots = BTreeSet::new();
    for p in &result.participants.0 {
        assert_eq!(
            p.verdict,
            if p.team == 0 {
                data::Verdict::Win
            } else {
                data::Verdict::Loss
            }
        );
        if p.identity.kind == data::MemberKind::ServerBot {
            result_bots.insert(p.identity.id as u64);
        } else {
            assert_eq!(p.identity.id, 3333);
            assert_eq!(p.identity.source, if p.team == 0 { 7 } else { 42 });
        }
    }
    assert_eq!(result_bots, first_bots);
    mixed::leave(&mut cs, &d, contexts);
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::ReadRecord);
        wait(&mut cs, &d, "healing Quest source result readback", |cs| {
            !cs[i].view().record_pending
        });
        record(&cs[i], i == 0);
    }
    let next_bots = room(&mut cs, &d, contexts, true);
    assert!(next_bots.is_disjoint(&first_bots));
    for (i, old) in [(0, free), (1, paid), (1, bot_paid)] {
        assert!(!old.current(cs[i].view().canonical.observation().unwrap()));
        assert!(cs[i]
            .controller
            .command(LoginCommand::Canonical(Command::player_healing(old)))
            .is_err());
    }
    foreign_rejected(&mut cs);
    mixed::leave(&mut cs, &d, contexts);
    for i in 0..2 {
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "healing Quest return roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    for i in 0..2 {
        let context = select_healer_hp(&mut cs, i, &d, if i == 0 { 42 } else { 7 }, 1000);
        // Character data is published before the scheduled record lookup.
        // NotRequested is not a successful empty SQL result.
        wait(
            &mut cs,
            &d,
            "healing Quest new-source record refresh",
            |cs| {
                let v = cs[i].view();
                !v.record_pending
                    && v.record.context() == Some(context)
                    && v.record.totals().is_some()
            },
        );
        record(&cs[i], i == 1);
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "healing Quest all owners retired", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
    eprintln!("HEALING_QUEST_MAIN_SQL: two ordinary extended workers/Main/SQL/shared554; each lane renders only own human+BOT, foreign human/BOT proofs and live NPC heal rejected, NPC injury/four waves, self and both source-named BOT heals with zero/4MP costs, no other-lane healing accounts/audio, saved four-participant result with typed BOT journal, swapped-team new match and retired BOT/stale cast rejection, exact source result reload; imported sources unchanged, no HWND claim");
}
