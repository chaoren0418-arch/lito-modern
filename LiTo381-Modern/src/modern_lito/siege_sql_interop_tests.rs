//! Real Main/TLS/isolated SQL/controllers/shared sprite workers. Both a synthetic
//! pinned S32 arena and unchanged shared Kent terrain through the normal loader.
use super::time_attack_sql_interop_tests::{back_to_roster, leave_world};
use super::*;
use crate::modern_lito::{
    siege_protocol as siege,
    world_map::{WorldCell, DIRECTIONS},
};
fn atomic(cs: &mut [Client; 2], i: usize, d: &Deadline) {
    wait(cs, d, "siege atomic authority", |cs| {
        let v = cs[i].view();
        !v.world_pending
            && !v.combat_pending
            && v.combat
                .as_ref()
                .is_some_and(|f| v.world.frame() == Some(&f.world))
    });
}
fn walk(cs: &mut [Client; 2], i: usize, d: &Deadline, target: WorldCell) {
    use crate::modern_lito::{siege_space, world_path};
    if cs[i].canonical.is_some() {
        walk_canonical(cs, i, d, target);
        return;
    }
    for _ in 0..100 {
        atomic(cs, i, d);
        let v = cs[i].view();
        let f = v.world.frame().unwrap();
        let context = v.navigation.as_ref().unwrap().context;
        let a = f
            .actors
            .iter()
            .find(|a| a.matches_character(context.key))
            .unwrap();
        if a.position == target {
            return;
        }
        let map = cs[i].rendered.as_ref().unwrap().navigation_map().unwrap();
        let mut blocked = siege_space::blockers(f.siege.as_ref().unwrap());
        blocked.extend(
            f.actors
                .iter()
                .filter(|other| other.entity_id != a.entity_id)
                .map(|other| other.position),
        );
        let path = world_path::plan(
            &map,
            a.position,
            target,
            &blocked,
            world_path::PathLimits::default(),
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
        let heading = path.headings()[0];
        let interval = Duration::from_nanos(a.step_interval_nanos);
        let mut q = world::Request::read(context, CHANNEL);
        q.action = world::Action::Move;
        q.match_id = f.match_id;
        q.generation = f.generation;
        q.sequence = a.sequence + 1;
        q.heading = heading;
        q.from_x = a.position.x;
        q.from_y = a.position.y;
        cs[i].command(LoginCommand::World(q));
        wait(cs, d, "siege movement ack", |cs| {
            !cs[i].view().world_pending
        });
        thread::sleep(d.remaining(interval, "siege move cadence"));
    }
    panic!("bounded siege fixture walk exhausted");
}
fn walk_canonical(cs: &mut [Client; 2], i: usize, d: &Deadline, target: WorldCell) {
    use crate::modern_lito::{
        battle_player,
        battle_window::{Intent, Kind, Stamp},
        world_navigation::{Navigation, Status},
    };
    atomic(cs, i, d);
    let mut route = Navigation::default();
    let mut started = false;
    let mut sent = 0;
    loop {
        for c in cs.iter_mut() {
            c.pump("canonical ground route");
        }
        route.poll();
        let v = cs[i].view();
        if v.world_pending || v.combat_pending || v.spell_pending {
            d.pause("canonical command busy");
            continue;
        }
        let Some(frame) = v
            .combat
            .as_ref()
            .filter(|f| v.world.frame() == Some(&f.world))
        else {
            d.pause("canonical atomic movement authority");
            continue;
        };
        let context = v.navigation.as_ref().unwrap().context;
        let a = frame
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(context.key))
            .unwrap();
        if a.position == target {
            route.cancel();
            while !route.is_quiescent() {
                route.poll();
                d.pause("route worker retirement");
            }
            return;
        }
        if !started {
            if let Some(o) = v.battle.observation() {
                let runtime = cs[i].canonical.as_ref().unwrap();
                let intent = Intent {
                    stamp: Stamp::of(o),
                    kind: Kind::Navigate(target),
                };
                if runtime.routing_map(o).is_some()
                    && battle_player::validate_intent(intent, &v).is_ok()
                {
                    battle_player::navigate(runtime, &v, intent, &mut route).unwrap();
                    started = true;
                }
            }
        }
        let picture_ready = cs[i]
            .canonical_surface
            .as_ref()
            .is_some_and(|s| battle_player::controls_ready(s, &v));
        if let Some(next) = route
            .next_step_combat(frame, a.entity_id, Instant::now())
            .filter(|_| picture_ready)
        {
            let mut q = world::Request::read(context, CHANNEL);
            q.action = world::Action::Move;
            q.match_id = next.match_id;
            q.generation = next.generation;
            q.sequence = next.sequence;
            q.heading = next.heading;
            q.from_x = next.from.x;
            q.from_y = next.from.y;
            let before = (a.position, a.sequence);
            cs[i].command(LoginCommand::World(q));
            route.submitted(next).unwrap();
            let after = cs[i].view();
            let actor = after
                .world
                .frame()
                .unwrap()
                .actors
                .iter()
                .find(|a| a.matches_character(context.key))
                .unwrap();
            // command() may publish an already-received read envelope; that
            // must not be confused with predicting this new MOVE's position.
            assert_eq!(
                (actor.position, actor.sequence),
                before,
                "routing never predicts a MOVE ACK"
            );
            sent += 1;
            assert!(sent <= 100, "bounded canonical route");
        }
        assert!(
            !matches!(route.status(), Status::Stopped(_)),
            "canonical route stopped: {:?}",
            route.status()
        );
        d.pause("canonical ACK-driven walking");
    }
}
fn approach(cs: &mut [Client; 2], i: usize, d: &Deadline, slot: siege::Slot) {
    use crate::modern_lito::{siege_space, world_path};
    atomic(cs, i, d);
    let v = cs[i].view();
    let w = v.world.frame().unwrap();
    let key = v.navigation.as_ref().unwrap().context.key;
    let a = w.actors.iter().find(|a| a.matches_character(key)).unwrap();
    let target = w.siege.as_ref().unwrap().objectives[slot as usize].cell;
    let map = cs[i].rendered.as_ref().unwrap().navigation_map().unwrap();
    let mut blocked = siege_space::blockers(w.siege.as_ref().unwrap());
    blocked.extend(
        w.actors
            .iter()
            .filter(|other| other.entity_id != a.entity_id)
            .map(|other| other.position),
    );
    // Cardinal attack squares avoid any invented diagonal attack shortcut.
    let nearest = [0, 2, 4, 6]
        .into_iter()
        .filter_map(|h| {
            let (dx, dy) = DIRECTIONS[h];
            let goal = WorldCell {
                x: target.x + dx,
                y: target.y + dy,
            };
            if !map.can_step(goal, target) {
                return None;
            }
            world_path::plan(
                &map,
                a.position,
                goal,
                &blocked,
                world_path::PathLimits::default(),
                &std::sync::atomic::AtomicBool::new(false),
            )
            .ok()
            .map(|p| (p.steps(), goal))
        })
        .min_by_key(|(steps, _)| *steps)
        .expect("reachable tower attack square")
        .1;
    walk(cs, i, d, nearest);
}
fn destroy(cs: &mut [Client; 2], i: usize, d: &Deadline, slot: siege::Slot) {
    for _ in 0..40 {
        atomic(cs, i, d);
        let v = cs[i].view();
        let w = v.world.frame().unwrap();
        if w.siege.as_ref().unwrap().objectives[slot as usize].hp == 0 {
            return;
        }
        let target = siege::Target {
            scope: siege::Scope::of(w),
            slot,
        };
        let command = if cs[i].canonical.is_some() {
            use crate::modern_lito::{
                battle_player,
                battle_window::{Intent, Kind, Stamp, Target},
            };
            let make = |v: &LoginSnapshot| {
                v.battle.observation().and_then(|o| {
                    let intent = Intent {
                        stamp: Stamp::of(o),
                        kind: Kind::Attack(Target::Siege { slot: slot as u8 }),
                    };
                    battle_player::command(intent, v)
                        .ok()
                        .map(|_| LoginCommand::PaintedTarget(intent))
                })
            };
            wait(cs, d, "canonical painted siege command admission", |cs| {
                make(&cs[i].view()).is_some()
            });
            make(&cs[i].view()).unwrap()
        } else {
            LoginCommand::SiegeAttack { target }
        };
        let before = cs[i].view().combat;
        cs[i].command(command);
        assert_eq!(cs[i].view().combat, before);
        wait(cs, d, "siege typed tower ack", |cs| {
            !cs[i].view().combat_pending
        });
    }
    panic!("bounded siege fixture tower attack exhausted");
}
#[test]
#[ignore = "requires explicit shared sprites, compiled Java peer, TLS fixtures and isolated MySQL; synthetic siege arena"]
fn java_siege_sql_two_controllers_shared_towers_portal_gm_bots_result_and_return() {
    run(false);
}
#[test]
#[ignore = "requires actual shared Kent map4 sectors, SPR/Tile/WAV, compiled Java peer and isolated MySQL"]
fn java_siege_shared_kent_two_controllers_real_map_preload_path_portal_result_and_return() {
    run(true);
}
fn run(real_shared_map: bool) {
    let d = Deadline(Instant::now() + Duration::from_secs(160));
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernSiegeSqlTestPeer",
        if real_shared_map {
            "siege-shared-sql"
        } else {
            "siege-sql"
        },
    );
    let p = profile(peer.endpoint, &shared, &private_root(&fixtures));
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new(p.clone()),
        runtime: if real_shared_map {
            WorldRuntime::new(shared.clone()).unwrap()
        } else {
            WorldRuntime::siege_fixture(shared.clone())
        },
        rendered: None,
        auto_ack: false,
        acks: BTreeSet::new(),
        light_arrow_sound: false,
        canonical: real_shared_map.then(|| {
            crate::modern_lito::battle_runtime::BattleRuntime::new(shared.clone()).unwrap()
        }),
        canonical_surface: None,
    });
    let mut contexts = [SelectionContext { key: SUN, epoch: 1 }; 2];
    for i in 0..2 {
        cs[i].command(LoginCommand::Connect);
        wait(&mut cs, &d, "siege TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        cs[i].command(LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "siege login", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = selected(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        assert!(!cs[i].view().record.totals().unwrap().stored);
        cs[i].command(LoginCommand::EnterChannel {
            context: contexts[i],
            channel_id: CHANNEL,
        });
        wait(&mut cs, &d, "siege lobby", |cs| {
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
                    capacity: 100,
                    mode: 3,
                    map_id: 547,
                    attributes: 0,
                    name: "Shared siege SQL".into(),
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
    room(
        &mut cs,
        0,
        &d,
        rooms::Request::read(contexts[0], CHANNEL).unwrap(),
    );
    cs[0].command(LoginCommand::GmBots { count: 28 });
    wait(&mut cs, &d, "siege GM bots", |cs| {
        !cs[0].view().room_pending
    });
    assert_eq!(cs[0].view().rooms.joined().unwrap().members.len(), 30);
    for i in 0..2 {
        room(
            &mut cs,
            i,
            &d,
            rooms::Request::read(contexts[i], CHANNEL).unwrap(),
        );
        let s = cs[i].view().rooms.joined().unwrap().summary.clone();
        room(
            &mut cs,
            i,
            &d,
            rooms::Request::simple(rooms::Common {
                context: contexts[i],
                channel_id: CHANNEL,
                action: rooms::Action::Ready,
                value: 1,
                room_id: s.room_id,
                expected_revision: s.revision,
            })
            .unwrap(),
        );
    }
    cs[0].auto_ack = true;
    wait(&mut cs, &d, "siege shared preflight", |cs| {
        cs.iter().all(|c| {
            c.rendered
                .as_ref()
                .is_some_and(|r| r.ready_to_ack && r.siege_towers.len() == 3 && r.npc.is_none())
        }) && cs[0]
            .view()
            .world
            .frame()
            .is_some_and(|w| w.actors[0].loaded)
    });
    assert_eq!(
        cs[0].view().world.frame().unwrap().phase,
        world::Phase::Loading
    );
    cs[1].auto_ack = true;
    wait(&mut cs, &d, "siege active", |cs| {
        cs.iter().all(|c| {
            c.view()
                .world
                .frame()
                .is_some_and(|w| w.phase == world::Phase::Active)
        })
    });
    atomic(&mut cs, 0, &d);
    followed(
        &mut cs,
        &d,
        crate::modern_lito::battle_payload::data::Phase::Active,
    );
    if real_shared_map {
        wait(
            &mut cs,
            &d,
            "canonical controls and retired legacy workers",
            |cs| {
                cs.iter().all(|c| {
                    c.runtime.preflight_finished()
                        && c.runtime.is_quiescent()
                        && c.canonical_surface.as_ref().is_some_and(|s| {
                            crate::modern_lito::battle_player::controls_ready(s, &c.view())
                        })
                })
            },
        );
        for c in &cs {
            let surface = c.canonical_surface.as_ref().unwrap();
            let view = c.view();
            let mut missing = view.clone();
            missing.battle_supported = false;
            assert!(!crate::modern_lito::battle_player::controls_ready(
                surface, &missing
            ));
            missing = view.clone();
            missing.combat = None;
            assert!(!crate::modern_lito::battle_player::controls_ready(
                surface, &missing
            ));
            missing = view;
            missing.navigation = None;
            assert!(!crate::modern_lito::battle_player::controls_ready(
                surface, &missing
            ));
        }
    }
    let w = cs[0].view().world.frame().unwrap().clone();
    let policy = w.siege.as_ref().unwrap().clone();
    let attacker = if w.member(1).unwrap().team == policy.attacker {
        0
    } else {
        1
    };
    let defender = 1 - attacker;
    if real_shared_map {
        wait(&mut cs, &d, "real Kent active pixels", |cs| {
            cs.iter().all(|c| {
                c.canonical_surface.as_ref().is_some_and(|s| {
                    s.observation.data().world.phase
                        == crate::modern_lito::battle_payload::data::Phase::Active
                })
            })
        });
        capture(&cs[attacker], "active-attacker");
        capture(&cs[defender], "active-defender");
    }
    // Defender traverses the configured portal while gate stays shut, then returns.
    walk(&mut cs, defender, &d, policy.portal_inside);
    for destination in [policy.portal_outside, policy.portal_inside] {
        atomic(&mut cs, defender, &d);
        let v = cs[defender].view();
        let cadence = v
            .world
            .frame()
            .unwrap()
            .actors
            .iter()
            .find(|a| a.matches_character(contexts[defender].key))
            .unwrap()
            .step_interval_nanos;
        thread::sleep(d.remaining(Duration::from_nanos(cadence), "siege portal cadence"));
        cs[defender].command(LoginCommand::SiegePortal {
            scope: siege::Scope::of(v.world.frame().unwrap()),
        });
        assert_eq!(cs[defender].view().world, v.world);
        wait(&mut cs, &d, "siege portal receipt", |cs| {
            !cs[defender].view().combat_pending
        });
        let f = cs[defender].view().world.frame().unwrap().clone();
        assert_eq!(
            f.actors
                .iter()
                .find(|a| a.matches_character(contexts[defender].key))
                .unwrap()
                .position,
            destination
        );
        assert!(!f.siege.as_ref().unwrap().open());
    }
    approach(&mut cs, attacker, &d, siege::Slot::OuterLeft);
    destroy(&mut cs, attacker, &d, siege::Slot::OuterLeft);
    approach(&mut cs, attacker, &d, siege::Slot::OuterRight);
    destroy(&mut cs, attacker, &d, siege::Slot::OuterRight);
    walk(&mut cs, attacker, &d, policy.portal_outside);
    walk(
        &mut cs,
        attacker,
        &d,
        WorldCell {
            x: policy.portal_outside.x,
            y: policy.portal_outside.y - 1,
        },
    );
    approach(&mut cs, attacker, &d, siege::Slot::Inner);
    destroy(&mut cs, attacker, &d, siege::Slot::Inner);
    wait(
        &mut cs,
        &d,
        "siege durable result and collapsed towers",
        |cs| {
            cs.iter().all(|c| {
                if c.canonical.is_some() {
                    return c.canonical_surface.as_ref().is_some_and(|r| {
                        r.observation
                            .data()
                            .siege
                            .as_ref()
                            .is_some_and(|s| s.objectives.0.iter().all(|t| t.hp == 0))
                    }) && c.view().world.frame().is_some_and(|f| {
                        f.siege
                            .as_ref()
                            .unwrap()
                            .receipt
                            .as_ref()
                            .is_some_and(|r| r.state == world::SettlementState::Saved)
                    });
                }
                c.rendered.as_ref().is_some_and(|r| {
                    r.siege_towers.iter().all(|t| !t.alive)
                        && r.combat.as_ref().is_some_and(|f| {
                            f.world
                                .siege
                                .as_ref()
                                .unwrap()
                                .receipt
                                .as_ref()
                                .is_some_and(|r| r.state == world::SettlementState::Saved)
                        })
                })
            })
        },
    );
    assert_eq!((cs[0].acks.len(), cs[1].acks.len()), (1, 1));
    followed(
        &mut cs,
        &d,
        crate::modern_lito::battle_payload::data::Phase::Finished,
    );
    if real_shared_map {
        // The pinned shared tower bank has 17 timed collapse frames followed
        // by source frame 0 / subframe 20 (independent TowerAssets oracle).
        // Keep pumping both real workers until ruins, not just HP=0, are drawn.
        wait(&mut cs, &d, "real Kent held tower ruins", |cs| {
            cs.iter().all(|c| {
                c.canonical_surface.as_ref().is_some_and(|r| {
                    r.objectives.len() == 3
                        && r.objectives
                            .iter()
                            .all(|(_, t)| t.source_frame == 0 && t.subframe == 20)
                        && r.observation
                            .data()
                            .siege
                            .as_ref()
                            .unwrap()
                            .objectives
                            .0
                            .iter()
                            .all(|t| t.hp == 0)
                        && c.runtime.is_quiescent()
                        && c.runtime.preflight_finished()
                })
            })
        });
        capture(&cs[attacker], "finished-attacker");
        if std::env::var_os("LITO_CANONICAL_WINDOW_INSPECT").is_some() {
            native_canonical_inspection(&mut cs, attacker, &d);
        }
    }
    for i in 0..2 {
        leave_world(&mut cs, i, &d);
        assert!(
            cs[i].view().battle.request().is_none(),
            "departed canonical attachment retained"
        );
        back_to_roster(&mut cs, i, &d);
        assert!(
            cs[i].view().battle.request().is_none(),
            "canonical private data survived return to roster"
        );
    }
    for i in 0..2 {
        selected(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        let r = cs[i].view().record.totals().unwrap();
        assert_eq!(
            (r.wins, r.losses, r.revision),
            (i64::from(i == attacker), i64::from(i != attacker), 1)
        );
        cs[i].command(LoginCommand::Shutdown);
    }
    wait(&mut cs, &d, "siege all owners retired", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
// Optional exact render-worker pixels, not a fabricated HWND/UI screenshot.
fn capture(client: &Client, stage: &str) {
    let Some(directory) = std::env::var_os("LITO_MODERN_SIEGE_CAPTURE_DIR") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    assert!(directory.is_absolute() && directory.is_dir());
    let (pixels, width, height) = if client.canonical.is_some() {
        let r = client
            .canonical_surface
            .as_ref()
            .expect("current canonical capture");
        (&r.bgra, r.width, r.height)
    } else {
        let r = client.rendered.as_ref().unwrap();
        (&r.bgra, r.width, r.height)
    };
    let mut rgba = pixels.to_vec();
    for p in rgba.chunks_exact_mut(4) {
        p.swap(0, 2);
    }
    let path = directory.join(format!("kent-{stage}.png"));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&rgba)
        .unwrap();
    eprintln!("shared Kent worker pixels: {}", path.display());
}
