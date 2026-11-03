//! Pure v41 controllers + Main/owned SQL + unchanged shared Kent resources.
//! The player's Route/painted command adapters, never legacy world frames,
//! drive portal traversal and all three towers through saved settlement.
use super::*;
use crate::modern_lito::{
    battle_route::Route, battle_scene::ObjectiveTarget, world_navigation::Status,
    world_path::PathLimits,
};
use std::sync::atomic::AtomicBool;

fn cell(c: &data::Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}
fn policy(c: &Client) -> data::Siege {
    c.view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .siege
        .clone()
        .unwrap()
}
fn pause(cs: &mut [Client; 2], d: &Deadline, duration: Duration) {
    let until = Instant::now() + duration;
    wait(cs, d, "siege player cadence", |_| Instant::now() >= until);
}
fn rendered(cs: &mut [Client; 2], i: usize, d: &Deadline) {
    wait(cs, d, "siege current shared pixels", |cs| cs[i].rendered());
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

fn enter(cs: &mut [Client; 2], i: usize, d: &Deadline, context: SelectionContext) {
    send(cs, i, d, || LoginCommand::EnterChannel {
        context,
        channel_id: CHANNEL,
    });
    wait(cs, d, "siege canonical lobby", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::InLobby && !v.room_pending && !v.record_pending && !v.spell_pending
    });
    let v = cs[i].view();
    assert!(v.canonical.equipment_supported());
    // This unchanged siege preset has no items rule; it must not acquire the
    // quest fixture's SHOP capability merely because both use the same SQL.
    assert_eq!(v.shop, shop::State::Unsupported);
    assert!(!login_window::can_open_shop(&v));
}

fn walk(cs: &mut [Client; 2], i: usize, d: &Deadline, destination: WorldCell) {
    rendered(cs, i, d);
    let before = own_actor(&cs[i]);
    if cell(&before.position) == destination {
        return;
    }
    let mut route = Route::default();
    let surface = cs[i].surface.as_ref().unwrap();
    route
        .request(
            surface,
            Intent {
                stamp: Stamp::of(&surface.observation),
                kind: Kind::Navigate(destination),
            },
            &cs[i].view(),
        )
        .unwrap();
    let mut sent = 0;
    loop {
        for c in cs.iter_mut() {
            c.pump();
        }
        let view = cs[i].view();
        if let Some(q) = route.next(cs[i].surface.as_deref(), &view, Instant::now()) {
            let actor = own_actor(&cs[i]);
            if cs[i]
                .controller
                .command(LoginCommand::Canonical(q.clone()))
                .is_ok()
            {
                route.submitted(&q).unwrap();
                let pending = own_actor(&cs[i]);
                assert_eq!(
                    (pending.position, pending.last_move_sequence),
                    (actor.position, actor.last_move_sequence),
                    "route may not predict MOVE"
                );
                sent += 1;
                assert!(sent <= 100, "bounded Kent route");
            }
        }
        assert!(
            !matches!(route.status(), Status::Stopped(_)),
            "canonical siege route stopped: {:?}, receipt {:?}",
            route.status(),
            view.canonical.outcome()
        );
        if route.status() == Status::Arrived {
            let a = own_actor(&cs[i]);
            assert_eq!(cell(&a.position), destination);
            assert_eq!(a.last_move_sequence, before.last_move_sequence + sent);
            assert!(sent > 0);
            break;
        }
        d.pause("pure v41 siege ACK-driven route");
    }
    route.cancel();
    while !route.is_quiescent() {
        route.poll();
        d.pause("siege route retirement");
    }
    rendered(cs, i, d);
    // Arrived means MOVE was acknowledged, not that its cadence has elapsed.
    // Fast renderers must not make the subsequent tower attack too early.
    pause(
        cs,
        d,
        Duration::from_nanos(own_actor(&cs[i]).step_interval_nanos.try_into().unwrap()),
    );
}

fn approach(cs: &mut [Client; 2], i: usize, d: &Deadline, slot: u8) {
    rendered(cs, i, d);
    let p = policy(&cs[i]);
    let target = cell(&p.towers.0[slot as usize].cell);
    let navigation = cs[i].surface.as_ref().unwrap().navigation.as_ref().unwrap();
    let goal = [0usize, 2, 4, 6]
        .into_iter()
        .filter_map(|h| {
            let (dx, dy) = crate::modern_lito::world_map::DIRECTIONS[h];
            let goal = WorldCell {
                x: target.x + dx,
                y: target.y + dy,
            };
            if !navigation.map().can_step(goal, target) {
                return None;
            }
            navigation
                .plan(goal, PathLimits::default(), &AtomicBool::new(false))
                .ok()
                .map(|p| (p.steps(), goal))
        })
        .min_by_key(|(steps, _)| *steps)
        .expect("reachable shared Kent tower")
        .1;
    walk(cs, i, d, goal);
}

fn destroy(cs: &mut [Client; 2], i: usize, d: &Deadline, slot: u8) {
    for _ in 0..40 {
        rendered(cs, i, d);
        if policy(&cs[i]).objectives.0[slot as usize].hp == 0 {
            return;
        }
        let s = cs[i].surface.as_ref().unwrap();
        assert!(s.objectives.iter().any(|(target, _)| matches!(target,
            ObjectiveTarget::Siege { slot: actual, .. } if *actual as u8 == slot)));
        // Keep real server damage/RNG and the unchanged 10,000-HP towers. The
        // existing isolated fixture's JDBC weapon is the only test balance.
        let view = input(cs, i, d, Kind::Attack(Target::Siege { slot }));
        applied(&view);
        let interval = own_actor(&cs[i]).vital.unwrap().attack_interval_nanos;
        pause(cs, d, Duration::from_nanos(interval.try_into().unwrap()));
    }
    panic!("bounded shared Kent tower destruction");
}

#[test]
#[ignore = "pure v41 Main/isolated SQL/shared Kent: 30 actors, canonical route, portal, towers/gate, saved result and return; no HWND"]
fn java_v41_sql_siege_shared_kent_route_portal_towers_saved_results_and_return() {
    flow(false);
}
#[test]
#[ignore = "ordinary schema2 v47 + Main/owned SQL/shared Kent: no shop/Slow, route/portal/towers/saved return; no HWND"]
fn java_ordinary_v47_siege_shared_kent_saved_and_return() {
    flow(true);
}
pub(super) fn finish_current(cs: &mut [Client; 2], d: &Deadline) -> ([u8; 16], usize) {
    let initial = policy(&cs[0]);
    assert_eq!(initial.gate_revision, 0);
    assert!(initial.objectives.0.iter().all(|t| t.hp == 10_000));
    let attacker = {
        let view = cs[0].view();
        let raw = view.canonical.observation().unwrap().data();
        if raw.room.members.0[(raw.personal.entity - 1) as usize].team == initial.attacker {
            0
        } else {
            1
        }
    };
    let defender = 1 - attacker;
    for c in cs.iter() {
        let s = c.surface.as_ref().unwrap();
        let n = s.navigation.as_ref().unwrap();
        assert!(initial
            .gate
            .0
            .iter()
            .all(|g| n.blocked().contains(&cell(g))));
        assert!(!battle_window::target_current(
            &s.observation,
            Target::Siege { slot: 2 }
        ));
    }
    walk(cs, defender, d, cell(&initial.portal_inside));
    for destination in [&initial.portal_outside, &initial.portal_inside] {
        let step = own_actor(&cs[defender]).step_interval_nanos;
        pause(cs, d, Duration::from_nanos(step as u64));
        assert!(battle_window::portal(
            cs[defender].view().canonical.observation().unwrap()
        ));
        let before = own_actor(&cs[defender]);
        let v = input(cs, defender, d, Kind::Portal);
        applied(&v);
        let after = own_actor(&cs[defender]);
        assert_eq!(&after.position, destination);
        assert_eq!(after.last_move_sequence, before.last_move_sequence + 1);
        assert_eq!(policy(&cs[defender]).gate_revision, 0);
        rendered(cs, defender, d);
    }
    for slot in [0, 1] {
        approach(cs, attacker, d, slot);
        let closed_picture = cs[attacker].surface.as_ref().unwrap().clone();
        destroy(cs, attacker, d, slot);
        if slot == 0 {
            assert_eq!(policy(&cs[attacker]).gate_revision, 0);
        } else {
            let view = cs[attacker].view();
            assert!(policy(&cs[attacker]).gate_revision > 0);
            assert!(
                !battle_player::controls_ready(&closed_picture, &view),
                "same-pose closed-gate picture cannot authorize a new open-gate route"
            );
            let mut stale_route = Route::default();
            assert!(stale_route
                .request(
                    &closed_picture,
                    Intent {
                        stamp: Stamp::of(&closed_picture.observation),
                        kind: Kind::Navigate(cell(&initial.portal_outside)),
                    },
                    &view
                )
                .is_err());
            assert!(stale_route.is_quiescent());
        }
    }
    rendered(cs, attacker, d);
    let opened = policy(&cs[attacker]);
    assert!(opened.gate_revision > 0);
    assert_eq!(opened.phase, data::SiegePhase::Inner);
    assert!(initial.gate.0.iter().all(|g| !cs[attacker]
        .surface
        .as_ref()
        .unwrap()
        .navigation
        .as_ref()
        .unwrap()
        .blocked()
        .contains(&cell(g))));
    walk(cs, attacker, d, cell(&initial.portal_outside));
    walk(
        cs,
        attacker,
        d,
        WorldCell {
            x: initial.portal_outside.x,
            y: initial.portal_outside.y - 1,
        },
    );
    approach(cs, attacker, d, 2);
    destroy(cs, attacker, d, 2);
    wait(cs, d, "siege canonical saved ruins", |cs| {
        cs.iter().all(|c| {
            c.view().canonical.lifecycle().is_some_and(|l| {
                l.phase == world::Phase::Finished
                    && l.settlement == Some(world::SettlementState::Saved)
            }) && c.surface.as_ref().is_some_and(|s| {
                s.observation.data().world.phase == data::Phase::Finished
                    && s.objectives.len() == 3
                    && s.objectives
                        .iter()
                        .all(|(_, t)| t.source_frame == 0 && t.subframe == 20)
            })
        })
    });
    let v = cs[attacker].view();
    let raw = v.canonical.observation().unwrap().data();
    let result = raw.result.as_ref().unwrap();
    assert_eq!(result.participants.0.len(), 30);
    assert_eq!(
        result
            .participants
            .0
            .iter()
            .filter(|p| p.identity.kind == data::MemberKind::ServerBot)
            .count(),
        28
    );
    assert_eq!(
        result.siege.as_ref().unwrap().end,
        data::SiegeEnd::InnerDestroyed
    );
    assert_eq!(result.siege.as_ref().unwrap().winner, initial.attacker);
    for i in 0..2 {
        let view = cs[i].view();
        let status = crate::modern_lito::login_window::canonical_world::result_status(
            view.canonical.display_status(false, None), view.canonical.observation());
        let verdict = if i == attacker { "本場勝利" } else { "本場敗北" };
        assert!(status.starts_with(verdict), "canonical siege world status: {status}");
        assert!(status.contains("戰績已由伺服器確認保存"));
        eprintln!("R394_SIEGE_RESULT source={} verdict={} saved=true",
            view.selected_source.unwrap(), verdict);
    }
    assert_eq!(
        cs[defender]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .result
            .as_ref(),
        Some(result)
    );
    (result.result_id, attacker)
}
fn flow(ordinary: bool) {
    let d = Deadline(Instant::now() + Duration::from_secs(170));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernSiegeSqlTestPeer",
        "siege-shared-sql",
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
        wait(&mut cs, &d, "siege canonical TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "siege canonical login", |cs| {
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
                        capacity: 100,
                        mode: 3,
                        map_id: 547,
                        attributes: 0,
                        name: "Canonical Kent SQL".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "siege canonical creator", |cs| {
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
    wait(&mut cs, &d, "siege canonical joined", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    let v = super::bots::gm(&mut cs, 0, &d, ".de 28");
    assert_eq!(v.rooms.joined().unwrap().members.len(), 30);
    for team in [0, 1] {
        assert_eq!(
            v.rooms
                .joined()
                .unwrap()
                .members
                .iter()
                .filter(|m| m.team == team)
                .count(),
            15
        );
    }
    for i in 0..2 {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Ready, 1);
    }
    cs[0].load = true;
    wait(&mut cs, &d, "siege canonical shared preload", |cs| {
        !cs[0].load
            && cs
                .iter()
                .all(|c| c.surface.as_ref().is_some_and(|s| s.objectives.len() == 3))
    });
    for c in &cs {
        let view = c.view();
        let raw = view.canonical.observation().unwrap().data();
        assert_eq!(raw.world.phase, data::Phase::Loading);
        assert_eq!((raw.room.mode, raw.room.map, raw.manifest.map), (3, 547, 4));
        assert_eq!(raw.manifest.sectors.0.len(), 9);
        assert_eq!(raw.actors.0.len(), 30);
        assert!(raw
            .actors
            .0
            .iter()
            .filter(
                |a| raw.room.members.0[(a.entity - 1) as usize].identity.kind
                    == data::MemberKind::ServerBot
            )
            .all(|a| a.loaded));
        battle_window::validate(c.surface.as_ref().unwrap()).unwrap();
    }
    cs[1].load = true;
    wait(&mut cs, &d, "siege canonical active", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    let (_, attacker) = finish_current(&mut cs, &d);
    let world_id = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .identity
        .clone();
    for i in 0..2 {
        let leave = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&world_id))
        });
        send(&mut cs, i, &d, || LoginCommand::Canonical(leave.clone()));
        wait(&mut cs, &d, "siege canonical world return", |cs| {
            !cs[i].view().world_pending
                && !cs[i].view().canonical.attached()
                && cs[i].runtime.is_quiescent()
        });
        assert!(cs[i].view().rooms.joined().is_some());
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "siege canonical roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
        let selected = selected(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        // Roster/gear completion precedes the bounded automatic record read.
        // Wait for this new epoch's actual reply, not just an idle inbox.
        wait(&mut cs, &d, "siege reselected automatic SQL record", |cs| {
            let v = cs[i].view();
            !v.record_pending && v.record.context() == Some(selected) && v.record.totals().is_some()
        });
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: i64::from(i == attacker),
                losses: i64::from(i != attacker),
                revision: 1,
            })
        );
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "siege canonical clean shutdown", |cs| {
        cs.iter()
            .all(|c| c.runtime.is_quiescent() && c.controller.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
