//! GM text -> real scoped chat adapter -> SQL authority -> typed room bots ->
//! pure v41 resource admission, combat and settlement. No bot is a SQL human.
use super::*;
#[path = "canonical_slow_quest_sql_tests.rs"]
mod slow;
#[path = "canonical_target_haste_quest_sql_tests.rs"]
mod target_haste;
use crate::modern_lito::chat_window;

fn create(cs: &mut [Client; 2], d: &Deadline, context: SelectionContext) {
    send(cs, 0, d, || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(context, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 4,
                        mode: 2,
                        attributes: 16,
                        map_id: 554,
                        name: "Canonical SQL GM dummies".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(cs, d, "v41 GM creator membership", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
}

pub(super) fn gm(cs: &mut [Client; 2], i: usize, d: &Deadline, text: &str) -> LoginSnapshot {
    // Local admission may be temporarily busy. Once admitted, never resend the
    // immutable GM mutation, even if the server returns denial or conflict.
    loop {
        for c in cs.iter_mut() {
            c.pump();
        }
        let v = cs[i].view();
        if login_window::gm_input_ready(&v) {
            if let Some(token) = v.chat.token() {
                let intent = chat_window::Intent {
                    token,
                    text: text.into(),
                    kind: chat_window::parse_kind(text).unwrap(),
                };
                if let Ok(command) = login_window::chat_command(&v, intent) {
                    if cs[i].controller.command(command).is_ok() {
                        assert!(cs[i].view().room_pending);
                        assert_eq!(cs[i].view().rooms, v.rooms, "GM input cannot predict seats");
                        break;
                    }
                }
            }
        }
        d.pause("v41 GM text admission");
    }
    wait(cs, d, "v41 SQL GM authority receipt", |cs| {
        !cs[i].view().room_pending
    });
    cs[i].view()
}

fn room_status(v: &LoginSnapshot, status: rooms::Status, reason: rooms::Reason) {
    let rooms::State::Ready(r) = &v.rooms else {
        panic!("real GM room reply missing")
    };
    assert_eq!((r.status, r.reason), (status, reason));
}

fn balanced(v: &LoginSnapshot, humans: usize, bots: usize) -> BTreeSet<u64> {
    let r = v.rooms.joined().unwrap();
    assert_eq!(r.members.len(), humans + bots);
    assert_eq!(r.members.len(), r.summary.settings.capacity as usize);
    assert_eq!(
        r.members
            .iter()
            .filter(|m| m.kind == rooms::MemberKind::Human)
            .count(),
        humans
    );
    assert_eq!(r.members.iter().filter(|m| m.team == 0).count(), 2);
    assert_eq!(r.members.iter().filter(|m| m.team == 1).count(), 2);
    let ids = r
        .members
        .iter()
        .filter(|m| m.kind == rooms::MemberKind::ServerBot)
        .map(|m| {
            assert!(m.ready);
            assert_eq!(m.source_name, "太陽神");
            assert_eq!(m.key.source_id, 7);
            assert!(m.name.starts_with("BOT"));
            assert!(
                r.member(m.key).is_none(),
                "typed bot identity is never a SQL character lookup"
            );
            m.key.object_id
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), bots);
    ids
}

fn force(c: &Client, lane: i32) {
    let v = c.view();
    let o = v.canonical.observation().unwrap();
    let d = o.data();
    assert_eq!(d.world.lane, lane);
    assert_eq!(d.room.members.0.len(), 4);
    assert_eq!(
        d.actors.0.len(),
        2,
        "opposing human/bot force stays private"
    );
    let s = c.surface.as_ref().unwrap();
    battle_window::validate(s).unwrap();
    assert_eq!(
        s.geometry
            .actors
            .iter()
            .map(|p| p.entity_id)
            .collect::<BTreeSet<_>>(),
        d.actors.0.iter().map(|a| a.entity as u64).collect()
    );
    for a in &d.actors.0 {
        let member = &d.room.members.0[(a.entity - 1) as usize];
        assert_eq!(member.team, lane);
        if member.identity.kind == data::MemberKind::ServerBot {
            assert!(
                a.loaded,
                "server dummy never waits for a nonexistent human resource ACK"
            );
            assert_eq!(
                a.spells.sequence, 0,
                "passive dummy is not an autonomous caster"
            );
            assert!(a.spells.histories.0.iter().all(|h| h.ledger.casts == 0));
            assert_eq!(member.source_name.0, "太陽神");
        }
        let p = s
            .geometry
            .actors
            .iter()
            .find(|p| p.entity_id == a.entity as u64)
            .unwrap();
        assert!(p.width > 0 && p.height > 0 && !p.sprite_name.is_empty());
    }
}

fn leave_world(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    context: SelectionContext,
    world: &data::WorldId,
) {
    let command = Command::world(battle_world::Request {
        action: battle_world::Action::Leave,
        ..battle_world::Request::read(context, CHANNEL, Some(world))
    });
    send(cs, i, d, || LoginCommand::Canonical(command.clone()));
    wait(cs, d, "v41 GM world return", |cs| {
        !cs[i].view().world_pending
            && !cs[i].view().canonical.attached()
            && cs[i].runtime.is_quiescent()
    });
    send(cs, i, d, || LoginCommand::ReadRecord);
    wait(cs, d, "v41 GM record readback", |cs| {
        !cs[i].view().record_pending
    });
}

fn waves(cs: &mut [Client; 2], i: usize, d: &Deadline) {
    let mut seen = BTreeSet::new();
    let mut next = Instant::now();
    loop {
        for c in cs.iter_mut() {
            c.pump();
        }
        let v = cs[i].view();
        let o = v.canonical.observation().unwrap();
        assert_eq!(o.data().world.phase, data::Phase::Active);
        if o.data()
            .quest
            .as_ref()
            .unwrap()
            .objectives
            .0
            .iter()
            .any(|q| q.hp > 0)
        {
            break;
        }
        if cs[i].rendered() && !v.world_pending && Instant::now() >= next {
            if let Some(n) = o.data().npcs.as_ref().unwrap().actors.0.iter().find(|n| {
                n.hp > 0
                    && cs[i]
                        .surface
                        .as_ref()
                        .unwrap()
                        .geometry
                        .npcs
                        .iter()
                        .any(|p| {
                            p.id.serial == n.serial as u64 && p.life == n.life as u64 && p.alive
                        })
            }) {
                let stage = n.stage;
                let result = input(
                    cs,
                    i,
                    d,
                    Kind::LightArrow(Target::Npc {
                        serial: n.serial,
                        life: n.life,
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
                let a = own_actor(&cs[i]);
                let spell = a
                    .spells
                    .histories
                    .0
                    .iter()
                    .find(|h| h.lane == data::Lane::NpcLightArrow)
                    .unwrap();
                let cost = if i == 0 { 1 } else { 3 };
                assert_eq!(spell.ledger.casts, stage as i64);
                assert_eq!(spell.ledger.cost, cost);
                assert_eq!(spell.ledger.spent, i64::from(stage * cost));
                seen.insert(stage);
                next = Instant::now() + Duration::from_millis(1250);
            }
        }
        d.pause("v41 GM four spell waves");
    }
    assert_eq!(seen, BTreeSet::from([1, 2, 3, 4]));
    wait(cs, d, "v41 GM post-wave shared pixels", |cs| {
        cs[i].rendered()
    });
    force(&cs[i], i as i32);
}

#[test]
#[ignore = "owned dev Main/SQL/TLS, one source-7 human + one GM bot, real Map514 LOAD and active scene; no HWND"]
fn r407_dev_single_human_bot_reaches_real_map514_world() {
    let d = Deadline(Instant::now() + Duration::from_secs(170));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(&fixtures, &d,
        "local.lito381.server.ModernSlowSqlTestPeer", "slow-healing-dev", Some("180000"));
    let mut doc = profile_document(peer.endpoint, &shared, &private_root(&fixtures));
    doc["schema"] = serde_json::json!(2);
    doc["gameplay_protocol"] = serde_json::json!("modern_v47");
    doc["gameplay_extension"] = serde_json::json!("player_healing_v1");
    let profile = ModernProfile::parse(&serde_json::to_vec(&doc).unwrap()).unwrap();
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new(profile.clone()),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None, load: false, audio: SoundAudit::default(),
    });
    send(&mut cs, 0, &d, || LoginCommand::Connect);
    wait(&mut cs, &d, "dev TLS", |cs| cs[0].view().phase == LoginPhase::LoginReady);
    send(&mut cs, 0, &d, || LoginCommand::Login {
        account: "fixture-user".into(), password: "fixture-password".into(),
    });
    wait(&mut cs, &d, "dev source list", |cs| cs[0].view().phase == LoginPhase::ChoosingSource);
    eprintln!("R407_DEV_STAGE source-list");
    send(&mut cs, 0, &d, || LoginCommand::SelectSource(7));
    wait(&mut cs, &d, "dev roster", |cs| cs[0].view().phase == LoginPhase::CharactersReady);
    let roster = cs[0].view();
    assert_eq!(roster.characters.len(), 1);
    assert_eq!(roster.characters[0].object_id, 3333);
    let context = SelectionContext { key: CharacterKey { source_id: 7, object_id: 3333 }, epoch: roster.roster_epoch };
    send(&mut cs, 0, &d, || LoginCommand::SelectCharacter(context));
    wait(&mut cs, &d, "dev selected character", |cs| {
        cs[0].view().phase == LoginPhase::ChoosingChannel && !cs[0].view().spell_pending
    });
    send(&mut cs, 0, &d, || LoginCommand::EnterChannel { context, channel_id: CHANNEL });
    wait(&mut cs, &d, "dev lobby", |cs| cs[0].view().phase == LoginPhase::InLobby && !cs[0].view().room_pending);
    eprintln!("R407_DEV_STAGE lobby");
    assert!(cs[0].view().gm_supported);
    send(&mut cs, 0, &d, || LoginCommand::Rooms(rooms::Request::draft(
        rooms::Common { action: rooms::Action::Create, ..rooms::Common::read(context, CHANNEL) },
        rooms::Draft::new(rooms::PublicDraft {
            capacity: 2, mode: 1, attributes: 0, map_id: 514,
            name: "R407 owned Map514".into(),
        }, String::new()).unwrap(),
    ).unwrap()));
    wait(&mut cs, &d, "dev creator joined", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
    eprintln!("R407_DEV_STAGE room-created");
    let granted = gm(&mut cs, 0, &d, ".de 1");
    eprintln!("R407_DEV_STAGE gm-replied");
    room_status(&granted, rooms::Status::Ok, rooms::Reason::None);
    let room = granted.rooms.joined().unwrap();
    assert_eq!((room.summary.users, room.members.len()), (2, 2));
    assert_eq!(room.members.iter().filter(|m| m.kind == rooms::MemberKind::ServerBot && m.ready).count(), 1);
    room_action(&mut cs, 0, &d, context, rooms::Action::Ready, 1);
    eprintln!("R407_DEV_STAGE owner-ready");
    let after_ready = cs[0].view();
    let joined = after_ready.rooms.joined().unwrap();
    eprintln!("R407_DEV_ROOM users={} ready={} satisfied={} teams={:?} available={:?} owner_match={} nav={:?} canonical={} receipt={}",
        joined.summary.users, joined.summary.ready, joined.summary.satisfied,
        joined.members.iter().map(|m| (m.team, m.ready, m.kind == rooms::MemberKind::ServerBot)).collect::<Vec<_>>(),
        match &after_ready.rooms { rooms::State::Ready(r) => r.options.iter().filter(|o|
            o.mode == 1 && o.map_id == 514).map(|o| o.world_available).collect::<Vec<_>>(),
            _ => Vec::new() }, joined.owner == context.key,
        after_ready.navigation.as_ref().map(|n| n.snapshot.phase),
        after_ready.canonical.supported(), after_ready.canonical.command_status());
    assert!(joined.summary.satisfied, "server did not certify two ready combatants");
    cs[0].load = true;
    let world_deadline = Instant::now() + Duration::from_secs(40);
    let mut report_at = Instant::now();
    loop {
        for client in &mut cs { client.pump(); }
        let view = cs[0].view();
        if cs[0].rendered() && view.canonical.observation().is_some_and(|o|
            o.data().world.phase == data::Phase::Active && o.data().manifest.map == 514) { break; }
        if Instant::now() >= report_at {
            eprintln!("R407_DEV_WAIT pending={} attached={} observation={:?} surface={} load={} controls={} status={} receipt={}",
                view.world_pending, view.canonical.attached(),
                view.canonical.observation().map(|o| o.data().world.phase),
                cs[0].surface.is_some(), cs[0].load,
                cs[0].surface.as_ref().is_some_and(|s| battle_player::controls_ready(s, &view)),
                view.status, view.canonical.command_status());
            report_at = Instant::now() + Duration::from_secs(5);
        }
        assert!(Instant::now() < world_deadline, "dev real Map514 active scene not reached");
        d.pause("dev real Map514 active scene");
    }
    let surface = cs[0].surface.as_ref().unwrap();
    battle_window::validate(surface).unwrap();
    assert!(cs[0].view().canonical.observation().unwrap().data().vip.is_none());
    assert!(surface.bgra.chunks_exact(4).any(|pixel| pixel[..3] != [0,0,0]));
    let world = cs[0].view().canonical.observation().unwrap().data().world.identity.clone();
    leave_world(&mut cs, 0, &d, context, &world);
    send(&mut cs, 0, &d, || LoginCommand::Shutdown);
    while !cs[0].controller.is_quiescent() { cs[0].pump(); d.pause("dev owner retirement"); }
    cs[0] = Client {
        controller: SessionController::new(profile),
        runtime: BattleRuntime::new(shared).unwrap(),
        surface: None, load: false, audio: SoundAudit::default(),
    };
    send(&mut cs, 0, &d, || LoginCommand::Connect);
    wait(&mut cs, &d, "dev reconnect TLS", |cs| cs[0].view().phase == LoginPhase::LoginReady);
    send(&mut cs, 0, &d, || LoginCommand::Login {
        account: "fixture-user".into(), password: "fixture-password".into(),
    });
    wait(&mut cs, &d, "dev reconnect sources", |cs| cs[0].view().phase == LoginPhase::ChoosingSource);
    send(&mut cs, 0, &d, || LoginCommand::SelectSource(7));
    wait(&mut cs, &d, "dev reconnect roster", |cs| cs[0].view().phase == LoginPhase::CharactersReady);
    assert_eq!(cs[0].view().characters[0].object_id, 3333);
    send(&mut cs, 0, &d, || LoginCommand::Shutdown);
    while !cs[0].controller.is_quiescent() { cs[0].pump(); d.pause("dev reconnect retirement"); }
    peer.stop_dev(&d);
    eprintln!("R407_DEV_SINGLE_HUMAN real TLS/SQL/Map514 actor HUD active with one GM bot; second official-profile login after normal exit; no HWND claim");
}

#[test]
#[ignore = "v41 GM .de parser/controller/Main/owned SQL/shared554: single-human admission, capacity, authorization, typed bots, spells, saved result and return; no HWND"]
fn java_v41_sql_gm_text_bots_single_human_capacity_authority_combat_saved_return() {
    let d = Deadline(Instant::now() + Duration::from_secs(170));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernQuestSqlTestPeer",
        "quest-supplied-bots-sql",
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
        wait(&mut cs, &d, "v41 GM TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "v41 GM sources", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = selected(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        enter(&mut cs, i, &d, contexts[i]);
        assert!(cs[i].view().gm_supported);
        assert!(
            !login_window::gm_input_ready(&cs[i].view()),
            "lobby alone is not a room"
        );
    }
    create(&mut cs, &d, contexts[0]);
    assert!(cs[0]
        .view()
        .rooms
        .joined()
        .unwrap()
        .member(contexts[0].key)
        .is_some());
    assert!(chat_window::parse_kind(".de 101").is_err());
    assert!(cs[0]
        .controller
        .command(LoginCommand::GmBots { count: 101 })
        .is_err());
    let first = gm(&mut cs, 0, &d, ".de 20");
    room_status(&first, rooms::Status::Ok, rooms::Reason::None);
    let retired_ids = balanced(&first, 1, 3);
    let full = gm(&mut cs, 0, &d, ".de 20");
    room_status(&full, rooms::Status::Ok, rooms::Reason::None);
    assert_eq!(
        full.rooms.joined(),
        first.rooms.joined(),
        "full room must not change revision or insert more dummies"
    );
    room_action(&mut cs, 0, &d, contexts[0], rooms::Action::Ready, 1);
    cs[0].load = true;
    wait(
        &mut cs,
        &d,
        "v41 single human and three bots active",
        |cs| {
            cs[0].rendered()
                && cs[0]
                    .view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .phase
                    == data::Phase::Active
        },
    );
    assert!(!cs[0].load);
    assert!(!cs[1].view().canonical.attached());
    force(&cs[0], 0);
    let first_world = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .identity
        .clone();
    assert!(!login_window::gm_input_ready(&cs[0].view()));
    assert!(cs[0]
        .controller
        .command(LoginCommand::GmBots { count: 0 })
        .is_err());
    leave_world(&mut cs, 0, &d, contexts[0], &first_world);
    assert_eq!(
        cs[0].view().record.totals(),
        Some(record::Totals {
            stored: false,
            wins: 0,
            losses: 0,
            revision: 0
        })
    );
    let cleared = gm(&mut cs, 0, &d, ".de 0");
    room_status(&cleared, rooms::Status::Ok, rooms::Reason::None);
    assert_eq!(cleared.rooms.joined().unwrap().members.len(), 1);
    let r = cleared.rooms.joined().unwrap().summary.clone();
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
    wait(&mut cs, &d, "v41 GM second human joins", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    let before = cs[1].view().rooms.joined().unwrap().clone();
    let denied = gm(&mut cs, 1, &d, ".de 20");
    room_status(
        &denied,
        rooms::Status::Denied,
        rooms::Reason::GmNotAuthorized,
    );
    assert_eq!(denied.rooms.joined(), Some(&before));
    let granted = gm(&mut cs, 0, &d, ".de 20");
    room_status(&granted, rooms::Status::Ok, rooms::Reason::None);
    let bots = balanced(&granted, 2, 2);
    assert!(
        bots.is_disjoint(&retired_ids),
        "removed bots cannot regain their old identity"
    );
    for i in 0..2 {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Ready, 1);
    }
    cs[0].load = true;
    wait(&mut cs, &d, "v41 GM human LOAD barrier", |cs| {
        cs.iter().all(|c| c.surface.is_some()) && !cs[0].load
    });
    for i in 0..2 {
        let v = cs[i].view();
        assert_eq!(
            v.canonical.observation().unwrap().data().world.phase,
            data::Phase::Loading
        );
        force(&cs[i], i as i32);
    }
    cs[1].load = true;
    wait(&mut cs, &d, "v41 GM both humans active", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    let active = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .identity
        .clone();
    assert!(active.match_id > first_world.match_id);
    for i in 0..2 {
        waves(&mut cs, i, &d);
    }
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
            let target = &o.data().quest.as_ref().unwrap().objectives.0[0];
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
            next = Instant::now() + Duration::from_millis(750);
        }
        d.pause("v41 GM objective finish");
    }
    wait(&mut cs, &d, "v41 GM SQL saved result", |cs| {
        cs.iter().all(|c| {
            c.view().canonical.lifecycle().is_some_and(|l| {
                l.phase == world::Phase::Finished
                    && l.settlement == Some(world::SettlementState::Saved)
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
    assert_eq!(result.participants.0.len(), 4);
    assert_eq!(
        result
            .participants
            .0
            .iter()
            .filter(|p| p.identity.kind == data::MemberKind::Human)
            .count(),
        2
    );
    let result_bots = result
        .participants
        .0
        .iter()
        .filter(|p| p.identity.kind == data::MemberKind::ServerBot)
        .map(|p| {
            assert_eq!(p.identity.source, 7);
            p.identity.id as u64
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(result_bots, bots);
    let winner_notice = crate::modern_lito::battle_window::result_notice(
        cs[0].view().canonical.observation().unwrap()).unwrap();
    let loser_notice = crate::modern_lito::battle_window::result_notice(
        cs[1].view().canonical.observation().unwrap()).unwrap();
    assert!(winner_notice.contains("本場勝利") && !winner_notice.contains("已保存"));
    assert!(loser_notice.contains("本場敗北") && !loser_notice.contains("已保存"));
    assert!(cs[0].view().canonical.display_status(false, None).contains("戰績已由伺服器確認保存"));
    assert!(cs[1].view().canonical.display_status(false, None).contains("戰績已由伺服器確認保存"));
    for (i, verdict) in [(0, "本場勝利"), (1, "本場敗北")] {
        let view = cs[i].view();
        let status = crate::modern_lito::login_window::canonical_world::result_status(
            view.canonical.display_status(false, None), view.canonical.observation());
        assert!(status.starts_with(verdict), "canonical world status: {status}");
        assert!(status.contains("戰績已由伺服器確認保存"),
            "result verdict is not the SQL saved receipt");
        eprintln!("R394_CANONICAL_RESULT source={} verdict={} saved=true",
            contexts[i].key.source_id, verdict);
    }
    for i in 0..2 {
        leave_world(&mut cs, i, &d, contexts[i], &active);
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: i64::from(i == 0),
                losses: i64::from(i == 1),
                revision: 1
            })
        );
        source(&cs[i], contexts[i]);
    }
    let cleared = gm(&mut cs, 0, &d, ".de 0");
    room_status(&cleared, rooms::Status::Ok, rooms::Reason::None);
    assert_eq!(cleared.rooms.joined().unwrap().members.len(), 2);
    for i in (0..2).rev() {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "v41 GM roster return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    // Same authenticated controller and same object ID, different SQL source:
    // the prior source's accepted GM grant must not be cached across selection.
    let moon = selected(&mut cs, 0, &d, 42);
    enter(&mut cs, 0, &d, moon);
    create(&mut cs, &d, moon);
    let before = cs[0].view().rooms.joined().unwrap().clone();
    let denied = gm(&mut cs, 0, &d, ".de 20");
    room_status(
        &denied,
        rooms::Status::Denied,
        rooms::Reason::GmNotAuthorized,
    );
    assert_eq!(denied.rooms.joined(), Some(&before));
    assert_eq!(denied.record.totals().unwrap().losses, 1);
    room_action(&mut cs, 0, &d, moon, rooms::Action::Leave, 0);
    send(&mut cs, 0, &d, || LoginCommand::ReturnCharacters(moon));
    wait(&mut cs, &d, "v41 GM reselected source return", |cs| {
        cs[0].view().phase == LoginPhase::CharactersReady
    });
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "v41 GM normal shutdown", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
