//! Actual Main/JDBC/GM/TLS + shared554: Slow29 NPC/BOT isolation,
//! death/terminal cleanup, typed SQL settlement and source reload. No HWND.
use super::*;
use crate::modern_lito::{battle_payload::speed as effect, battle_slow as slow};

fn visible_target(c: &Client, target: Target) {
    use crate::modern_lito::{compositor::Viewport, world_map::PixelPoint};
    let s = c.surface.as_ref().unwrap();
    let (top, width, height) = match target {
        Target::Player { entity, .. } => {
            let p = s
                .geometry
                .actors
                .iter()
                .find(|p| p.entity_id == entity as u64)
                .unwrap();
            (p.top_left, p.width, p.height)
        }
        Target::Npc { serial, life } => {
            let p = s
                .geometry
                .npcs
                .iter()
                .find(|p| p.id.serial == serial as u64 && p.life == life as u64)
                .unwrap();
            (p.top_left, p.width, p.height)
        }
        _ => panic!("only real NPC/BOT target"),
    };
    let viewport = Viewport {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    };
    let point = (top.y.max(0)..(top.y + height as i32).min(600))
        .find_map(|y| {
            (top.x.max(0)..(top.x + width as i32).min(800)).find_map(|x| {
                let p = PixelPoint { x, y };
                battle_window::hit(s, viewport, p, battle_window::Skill::Slow)
                    .filter(|hit| hit.intent.kind == Kind::Slow(target))
                    .map(|_| p)
            })
        })
        .expect("frontmost shared SPR pixels must select this NPC/BOT");
    for (scale, x, y) in [(1, 0, 0), (2, 17, 29)] {
        let view = Viewport {
            x,
            y,
            width: 800 * scale,
            height: 600 * scale,
        };
        let point = PixelPoint {
            x: x + point.x * scale,
            y: y + point.y * scale,
        };
        let hit = battle_window::hit(s, view, point, battle_window::Skill::Slow).unwrap();
        assert_eq!(hit.intent.kind, Kind::Slow(target));
        let name = hit.name.unwrap().0;
        if matches!(target, Target::Player { .. }) {
            assert!(name.starts_with("BOT") && name.contains("太陽神"));
        }
        assert!(!name.is_empty());
        assert!(battle_window::hit(
            s,
            view,
            PixelPoint { x: x - 1, y: y - 1 },
            battle_window::Skill::Slow
        )
        .is_none());
    }
}

// Production probability is capped at 90%; a confirmed paid miss is legal.
// A new explicit test click may try again (at most eight times), never resend
// an uncertain mutation or replace production RNG with a test probability.
fn cast(cs: &mut [Client; 2], d: &Deadline, target: Target, count: &mut i64) -> effect::SlowDetail {
    for attempt in 0..8 {
        wait(cs, d, "Slow29 current Quest pixels", |cs| cs[0].rendered());
        slow_sql::material_inventory(&cs[0], *count);
        visible_target(&cs[0], target);
        let old =
            slow::Intent::from_observation(cs[0].view().canonical.observation().unwrap(), target)
                .unwrap();
        let before = own_actor(&cs[0]).spells;
        let v = loop {
            let v = input(cs, 0, d, Kind::Slow(target));
            if v.canonical.outcome() != Some(Outcome::NotSent) {
                break v;
            }
            assert_eq!(
                own_actor(&cs[0]).spells,
                before,
                "confirmed non-send cannot consume"
            );
            d.pause("Slow29 fresh input after confirmed non-send");
        };
        assert_eq!(
            v.canonical.outcome(),
            Some(Outcome::Slow(slow::Reason::None))
        );
        *count += 1;
        let a = own_actor(&cs[0]);
        let h = a
            .spells
            .histories
            .0
            .iter()
            .find(|h| h.lane == data::Lane::Slow)
            .unwrap();
        assert_eq!(
            (h.ledger.casts, h.ledger.cost, h.ledger.spent),
            (*count, 15, *count * 15)
        );
        let r = h.receipt.as_ref().unwrap();
        let data::Detail::Slow(detail) = &r.detail else {
            panic!("Slow29 receipt")
        };
        assert_eq!(detail.probability, 90);
        assert_eq!(detail.equipment_haste_items, 0);
        assert!(detail.missing_effect.is_none());
        wait(cs, d, "Slow29 exact Quest receipt painted", |cs| {
            cs[0].rendered()
                && cs[0].surface.as_ref().is_some_and(|s| {
                    s.observation.data().actors.0.iter().any(|a| {
                        a.entity == s.observation.data().personal.entity
                            && a.spells.histories.0.iter().any(|h| {
                                h.lane == data::Lane::Slow && h.receipt.as_ref() == Some(r)
                            })
                    })
                })
        });
        slow_sql::material_inventory(&cs[0], *count);
        assert!(cs[0]
            .controller
            .command(LoginCommand::Canonical(Command::slow(old)))
            .is_err());
        wait(cs, d, "Slow29 Quest cast recovery", |cs| {
            cs[0].rendered()
                && cs[0]
                    .view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .loading_elapsed_nanos
                    >= r.loading_elapsed_nanos + 1_200_000_000
        });
        if detail.roll <= detail.probability {
            return detail.clone();
        }
        eprintln!("SLOW29_PAID_MISS attempt={} uses={}", attempt + 1, count);
    }
    panic!("production RNG did not apply Slow within eight explicitly confirmed casts");
}

// A live NPC tick advances the world even when this caster and friendly BOT
// retain every fact in the painted request. Keep the original proof bytes.
fn unchanged_painted_slow(cs: &mut [Client; 2], d: &Deadline, target: Target) {
    use crate::modern_lito::battle_player;
    wait(cs, d, "capture stable Slow29 BOT pixels", |cs| {
        let v = cs[0].view();
        cs[0].rendered()
            && cs[0].surface.as_ref().is_some_and(|s| {
                let painted = slow::Intent::from_observation(&s.observation, target);
                v.canonical
                    .observation()
                    .and_then(|o| slow::Intent::from_observation(o, target))
                    .is_some_and(|mut current| {
                        let Some(painted) = painted else { return false };
                        current.stamp.revision = painted.stamp.revision;
                        current == painted
                    })
            })
    });
    let surface = cs[0].surface.clone().unwrap();
    let painted = slow::Intent::from_observation(&surface.observation, target).unwrap();
    let bytes = slow::encode_intent(&painted, slow::VERSION).unwrap();
    wait(
        cs,
        d,
        "NPC update with unchanged caster and BOT proof",
        |cs| {
            let v = cs[0].view();
            let Some(o) = v.canonical.observation() else {
                return false;
            };
            let Some(mut newer) = slow::Intent::from_observation(o, target) else {
                return false;
            };
            let advanced = newer.stamp.revision > painted.stamp.revision;
            newer.stamp.revision = painted.stamp.revision;
            advanced
                && newer == painted
                && battle_player::controls_ready(&surface, &v)
                && !v.world_pending
        },
    );
    let v = cs[0].view();
    let observed_revision = v.canonical.observation().unwrap().data().world.revision;
    let command = battle_input::command(
        &surface,
        Intent {
            stamp: painted.stamp,
            kind: Kind::Slow(target),
        },
        &v,
    )
    .expect("unchanged painted Slow must survive an unrelated NPC world revision");
    assert_eq!(
        command,
        Command::slow(painted),
        "must not rebase a painted request"
    );
    assert_eq!(slow::encode_intent(&painted, slow::VERSION).unwrap(), bytes);
    assert!(painted.current(v.canonical.observation().unwrap()));
    let before = own_actor(&cs[0]);
    let materials = v
        .canonical
        .observation()
        .unwrap()
        .data()
        .personal
        .materials
        .clone();
    // Exactly one captured command: no rebuild or retry on admission/non-send.
    cs[0]
        .controller
        .command(LoginCommand::Canonical(command))
        .unwrap();
    assert!(cs[0].view().world_pending);
    assert_eq!(own_actor(&cs[0]).spells, before.spells);
    wait(cs, d, "unchanged old-proof Slow receipt", |cs| {
        !cs[0].view().world_pending
    });
    assert_eq!(
        cs[0].view().canonical.outcome(),
        Some(Outcome::Slow(slow::Reason::SameTeam))
    );
    let after = own_actor(&cs[0]);
    assert_eq!(after.spells.sequence, before.spells.sequence + 1);
    let mut normalized = after.spells;
    normalized.sequence = before.spells.sequence;
    normalized.request_revision = before.spells.request_revision;
    assert_eq!(normalized, before.spells);
    assert_eq!(
        after.vital.as_ref().unwrap().values.mp,
        before.vital.as_ref().unwrap().values.mp
    );
    assert_eq!(
        cs[0]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .personal
            .materials,
        materials
    );
    assert_eq!(slow::encode_intent(&painted, slow::VERSION).unwrap(), bytes);
    eprintln!("OLD_PAINTED_SLOW painted_revision={} admission_revision={} sequence={} outcome=SameTeam immutable=true retries=0",
        painted.stamp.revision, observed_revision, painted.sequence);
}

fn moved_ready(cs: &mut [Client; 2], d: &Deadline) {
    // Finished pixels are not a cooldown. Wait the server-announced MOVE
    // interval from its confirmed receipt before the next cast or attack.
    let next = Instant::now()
        + Duration::from_nanos(own_actor(&cs[0]).step_interval_nanos.try_into().unwrap());
    wait(cs, d, "Slow29 MOVE recovery and pixels", |cs| {
        cs[0].rendered() && Instant::now() >= next
    });
}
fn effects(c: &Client) -> effect::State {
    c.view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .speed
        .as_ref()
        .unwrap()
        .state
        .clone()
}

#[test]
#[ignore = "v47 Main/owned SQL/GM/shared554: NPC Slow and friendly-BOT rejection, two-force isolation, death/terminal cleanup, four waves, saved win/loss, source reload; no HWND"]
fn java_v47_main_sql_slow_npc_bot_quest_materials_saved_results_and_return() {
    let d = Deadline(Instant::now() + Duration::from_secs(170));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernSlowQuestSqlTestPeer",
        "slow-quest-sql",
        Some("170000"),
    );
    let profile = configured_modern_profile(peer.endpoint, &shared, &private_root(&fixtures));
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
        wait(&mut cs, &d, "Slow29 Quest TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "Slow29 Quest sources", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = slow_sql::select(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        enter(&mut cs, i, &d, contexts[i]);
        assert!(cs[i].view().canonical.slow_supported());
        assert!(cs[i].view().gm_supported);
        slow_sql::gems_count(&mut cs, i, &d, contexts[i], 20);
    }
    create(&mut cs, &d, contexts[0]);
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
    wait(&mut cs, &d, "Slow29 Quest joined", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    let denied = gm(&mut cs, 1, &d, ".de 20");
    room_status(
        &denied,
        rooms::Status::Denied,
        rooms::Reason::GmNotAuthorized,
    );
    let effected = gm(&mut cs, 0, &d, ".de 20");
    room_status(&effected, rooms::Status::Ok, rooms::Reason::None);
    let bot_ids = balanced(&effected, 2, 2);
    for i in 0..2 {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Ready, 1);
    }
    cs[0].load = true;
    wait(&mut cs, &d, "Slow29 Quest first LOAD", |cs| {
        cs.iter().all(|c| c.surface.is_some()) && !cs[0].load
    });
    for i in 0..2 {
        assert_eq!(
            cs[i]
                .view()
                .canonical
                .observation()
                .unwrap()
                .data()
                .world
                .phase,
            data::Phase::Loading
        );
        force(&cs[i], i as i32);
    }
    cs[1].load = true;
    wait(&mut cs, &d, "Slow29 Quest active", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    for c in &cs {
        assert_eq!(
            c.view()
                .canonical
                .observation()
                .unwrap()
                .data()
                .personal
                .materials
                .as_ref()
                .unwrap()
                .stocks
                .0[0]
                .initial,
            20
        );
    }
    let world = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .identity
        .clone();
    wait(&mut cs, &d, "Slow29 first Quest wave born", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .npcs
                    .as_ref()
                    .unwrap()
                    .actors
                    .0
                    .iter()
                    .any(|n| n.hp > 0)
        })
    });
    let npcs = cs.each_ref().map(|c| {
        c.view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .npcs
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .find(|n| n.hp > 0)
            .unwrap()
            .clone()
    });
    assert_ne!(npcs[0].serial, npcs[1].serial);
    assert_eq!(
        npcs[0].cell, npcs[1].cell,
        "same coordinates must not merge isolated force NPCs"
    );
    let targets = cs.each_ref().map(|c| {
        let v = c.view();
        let data = v.canonical.observation().unwrap().data();
        let a = data
            .actors
            .0
            .iter()
            .find(|a| {
                data.room.members.0[(a.entity - 1) as usize].identity.kind
                    == data::MemberKind::ServerBot
            })
            .unwrap();
        assert_eq!(
            data.room.members.0[(a.entity - 1) as usize].source_name.0,
            "太陽神"
        );
        Target::Player {
            entity: a.entity,
            life: a.resources.as_ref().unwrap().life + 1,
        }
    });
    for foreign in [
        Target::Npc {
            serial: npcs[1].serial,
            life: npcs[1].life,
        },
        targets[1],
    ] {
        assert!(slow::Intent::from_observation(
            cs[0].view().canonical.observation().unwrap(),
            foreign
        )
        .is_none());
        // ready() asks whether the skill can enter target-selection mode;
        // the actual painted/selected-target adapters must reject this target.
        let v = cs[0].view();
        let surface = cs[0].surface.as_ref().unwrap();
        let invalid = Intent {
            stamp: Stamp::of(&surface.observation),
            kind: Kind::Slow(foreign),
        };
        assert!(battle_input::command(surface, invalid, &v).is_err());
        assert!(battle_input::target_command(invalid, &v).is_err());
    }
    let own_npc = Target::Npc {
        serial: npcs[0].serial,
        life: npcs[0].life,
    };
    let mut uses = 0;
    let first = cast(&mut cs, &d, own_npc, &mut uses);
    let effect::StatusTarget::Npc(n) = &first.target else {
        panic!("NPC receipt")
    };
    assert_eq!(
        (n.id.world.clone(), n.id.serial, n.life, n.lane),
        (world.clone(), npcs[0].serial, npcs[0].life, 0)
    );
    assert_eq!(n.after.slow_until_millis - n.after.observed_millis, 64_000);
    let state = effects(&cs[0]);
    let motion = state
        .npcs
        .as_ref()
        .unwrap()
        .actors
        .0
        .iter()
        .find(|a| a.id.serial == n.id.serial)
        .unwrap();
    assert_eq!((motion.walk_millis, motion.attack_millis), (500, 1000));
    assert_eq!(motion.speed.slow_until_millis, n.after.slow_until_millis);
    assert_eq!(motion.speed.revision, n.after.revision);
    assert!(effects(&cs[1]).targets.0.iter().all(|e| e.active.is_none()));
    assert_eq!(
        own_actor(&cs[0]).buffs.as_ref().unwrap().slow_until_nanos,
        0
    );

    // Negative skill29 is not positive skill43: a same-force BOT must be
    // rejected without payment. Do not weaken team policy to match this test.
    wait(&mut cs, &d, "Slow29 same-team BOT pixels", |cs| {
        cs[0].rendered()
    });
    visible_target(&cs[0], targets[0]);
    let before_bot_actor = own_actor(&cs[0]);
    let before_bot = before_bot_actor.spells.clone();
    let before_materials = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .personal
        .materials
        .clone();
    let v = loop {
        let v = input(&mut cs, 0, &d, Kind::Slow(targets[0]));
        if v.canonical.outcome() != Some(Outcome::NotSent) {
            break v;
        }
        assert_eq!(own_actor(&cs[0]).spells, before_bot);
    };
    assert_eq!(
        v.canonical.outcome(),
        Some(Outcome::Slow(slow::Reason::SameTeam))
    );
    let after_bot = own_actor(&cs[0]);
    assert_eq!(after_bot.spells.sequence, before_bot.sequence + 1);
    assert!(after_bot.spells.request_revision > before_bot.request_revision);
    // Denied requests still retire their sequence against replay. Only these
    // two request-ledger fields change; paid histories and MP do not.
    let mut normalized = after_bot.spells.clone();
    normalized.sequence = before_bot.sequence;
    normalized.request_revision = before_bot.request_revision;
    assert_eq!(normalized, before_bot);
    assert_eq!(
        after_bot.vital.as_ref().unwrap().values.mp,
        before_bot_actor.vital.as_ref().unwrap().values.mp
    );
    assert_eq!(
        cs[0]
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .personal
            .materials,
        before_materials
    );
    wait(&mut cs, &d, "Slow29 rejected BOT confirmed pixels", |cs| {
        cs[0].rendered()
    });
    slow_sql::material_inventory(&cs[0], uses);
    let state = effects(&cs[0]);
    assert!(
        state
            .targets
            .0
            .iter()
            .any(|e| e.target == effect::Target::Npc(n.id.clone()) && e.active.is_some()),
        "rejected friendly BOT must not erase the NPC effect"
    );
    assert!(state
        .targets
        .0
        .iter()
        .filter(|e| matches!(e.target, effect::Target::Player(_)))
        .all(|e| e.active.is_none()));
    for a in &cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .actors
        .0
    {
        assert_eq!(a.buffs.as_ref().unwrap().slow_until_nanos, 0);
    }
    unchanged_painted_slow(&mut cs, &d, targets[0]);
    slow_sql::material_inventory(&cs[1], 0);
    force(&cs[0], 0);
    force(&cs[1], 1);
    assert!(effects(&cs[1]).targets.0.iter().all(|e| e.active.is_none()));

    // Move through the painted-input adapter after casting, then return to the
    // original objective approach cell after both forces' waves are cleared.
    let origin = own_actor(&cs[0]).position;
    let from = WorldCell {
        x: origin.x,
        y: origin.y,
    };
    let surface = cs[0].surface.as_ref().unwrap();
    let (heading, to) = DIRECTIONS
        .iter()
        .enumerate()
        .find_map(|(h, (dx, dy))| {
            let to = WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            };
            (surface.navigation.as_ref().unwrap().can_step(from, to)
                // A free cell now can be a scheduled wave/objective birth.
                // Keep this round-trip off those declared Quest cells.
                && (to.x != npcs[0].cell.x || to.y != npcs[0].cell.y)
                && surface.observation.data().quest.as_ref().unwrap().targets.0.iter()
                    .all(|target| target.cell.x != to.x || target.cell.y != to.y)
                && surface
                    .observation
                    .data()
                    .actors
                    .0
                    .iter()
                    .all(|a| a.position.x != to.x || a.position.y != to.y)
                && surface
                    .observation
                    .data()
                    .npcs
                    .as_ref()
                    .unwrap()
                    .actors
                    .0
                    .iter()
                    .all(|n| n.hp == 0 || n.cell.x != to.x || n.cell.y != to.y))
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
    moved_ready(&mut cs, &d);
    let dead_target =
        slow::Intent::from_observation(cs[0].view().canonical.observation().unwrap(), own_npc)
            .unwrap();
    for i in 0..2 {
        waves(&mut cs, i, &d);
    }
    let state = effects(&cs[0]);
    assert!(
        state
            .targets
            .0
            .iter()
            .filter(|e| e.target == effect::Target::Npc(n.id.clone()))
            .all(|e| e.active.is_none()),
        "NPC death retires the active effect"
    );
    assert!(
        slow::Intent::from_observation(cs[0].view().canonical.observation().unwrap(), own_npc)
            .is_none()
    );
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::slow(dead_target)))
        .is_err());
    assert!(
        state
            .targets
            .0
            .iter()
            .filter(|e| matches!(e.target, effect::Target::Player(_)))
            .all(|e| e.active.is_none()),
        "rejected friendly BOT never gains a Slow lease"
    );
    let back = DIRECTIONS
        .iter()
        .position(|(dx, dy)| to.x + dx == origin.x && to.y + dy == origin.y)
        .unwrap() as u8;
    let moved = input(&mut cs, 0, &d, Kind::Local(Local::Step(back)));
    assert_eq!(
        moved.canonical.outcome(),
        Some(Outcome::Operation(
            Rejection::None,
            Disposition::Applied,
            Reason::None
        ))
    );
    assert_eq!(own_actor(&cs[0]).position, origin);
    moved_ready(&mut cs, &d);
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
            let t = &o.data().quest.as_ref().unwrap().objectives.0[0];
            let result = input(
                &mut cs,
                0,
                &d,
                Kind::Attack(Target::Quest {
                    slot: t.id.slot,
                    born: t.born_revision,
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
        d.pause("Slow29 Quest objective finish");
    }
    wait(
        &mut cs,
        &d,
        "Slow29 Quest saved result and terminal pixels",
        |cs| {
            cs.iter().all(|c| {
                c.rendered()
                    && c.view().canonical.lifecycle().is_some_and(|l| {
                        l.phase == world::Phase::Finished
                            && l.settlement == Some(world::SettlementState::Saved)
                    })
            })
        },
    );
    for c in &cs {
        let state = effects(c);
        assert!(
            state.targets.0.iter().all(|e| e.active.is_none()),
            "terminal removes active29"
        );
        assert!(state
            .npcs
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .all(|m| m.speed.slow_until_millis == 0));
        assert!(c
            .view()
            .canonical
            .observation()
            .unwrap()
            .data()
            .actors
            .0
            .iter()
            .all(|a| a.buffs.as_ref().unwrap().slow_until_nanos == 0));
        let v = c.view();
        let result = v
            .canonical
            .observation()
            .unwrap()
            .data()
            .result
            .as_ref()
            .unwrap();
        assert_eq!(result.participants.0.len(), 4);
        assert_eq!(
            result
                .participants
                .0
                .iter()
                .filter(|p| p.identity.kind == data::MemberKind::ServerBot)
                .map(|p| p.identity.id as u64)
                .collect::<BTreeSet<_>>(),
            bot_ids
        );
    }
    for i in 0..2 {
        leave_world(&mut cs, i, &d, contexts[i], &world);
        assert_eq!(
            cs[i].view().record.totals(),
            Some(record::Totals {
                stored: true,
                wins: i64::from(i == 0),
                losses: i64::from(i == 1),
                revision: 1
            })
        );
        slow_sql::saved(&cs[i], contexts[i]);
    }
    let cleared = gm(&mut cs, 0, &d, ".de 0");
    room_status(&cleared, rooms::Status::Ok, rooms::Reason::None);
    for i in (0..2).rev() {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "Slow29 Quest roster return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    let moon = slow_sql::select(&mut cs, 0, &d, 42);
    enter(&mut cs, 0, &d, moon);
    create(&mut cs, &d, moon);
    let denied = gm(&mut cs, 0, &d, ".de 20");
    room_status(
        &denied,
        rooms::Status::Denied,
        rooms::Reason::GmNotAuthorized,
    );
    assert_eq!(denied.record.totals().unwrap().losses, 1);
    room_action(&mut cs, 0, &d, moon, rooms::Action::Leave, 0);
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "Slow29 Quest owned shutdown", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
}
