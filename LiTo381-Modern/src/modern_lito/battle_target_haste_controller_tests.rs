//! Actual v44 controllers and shared renderer. Synthetic SELECT-only sources, not SQL or HWND.
use super::*;
use crate::modern_lito::{
    battle_input, battle_target_haste as haste,
    battle_window::{Intent, Kind, Local},
    spell_protocol::SkillsState,
};

fn self_target(
    o: &crate::modern_lito::battle_follow::Observation,
) -> crate::modern_lito::battle_window::Target {
    let a = o
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .unwrap();
    crate::modern_lito::battle_window::Target::Player {
        entity: a.entity,
        life: a.resources.as_ref().unwrap().life + 1,
    }
}
fn cast_command(c: &Client) -> Command {
    let v = c.view();
    let s = c.surface.as_ref().unwrap();
    battle_input::command(
        s,
        Intent {
            stamp: Stamp::of(&s.observation),
            kind: Kind::Haste(self_target(&s.observation)),
        },
        &v,
    )
    .unwrap()
}
fn self_mp(c: &Client) -> i64 {
    let v = c.view();
    let o = v.canonical.observation().unwrap();
    o.data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .unwrap()
        .vital
        .as_ref()
        .unwrap()
        .values
        .mp
}
#[test]
#[ignore = "v44 two actual controllers, synthetic source qualification, shared755/action19, cast/move/return; no SQL or HWND"]
fn java_v44_target_haste_controllers_scoped_input_shared_cast_move_and_return() {
    let d = Deadline::new();
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernHasteTestPeer",
        "normal",
        Some("60000"),
    );
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new_target_haste_development(profile(
            peer.endpoint,
            &shared,
            &ca,
        )),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "sources", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        send(&mut cs, i, &d, || {
            LoginCommand::SelectSource(if i == 0 { 7 } else { 42 })
        });
        wait(&mut cs, &d, "roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
        let roster = cs[i].view();
        let context = SelectionContext {
            key: CharacterKey {
                source_id: roster.selected_source.unwrap(),
                object_id: 3333,
            },
            epoch: roster.roster_epoch,
        };
        send(&mut cs, i, &d, || LoginCommand::SelectCharacter(context));
        wait(&mut cs, &d, "catalogs", |cs| {
            let v = cs[i].view();
            v.phase == LoginPhase::ChoosingChannel
                && !v.record_pending
                && !v.spell_pending
                && matches!(v.equipment, equipment::State::Ready(_))
        });
        send(&mut cs, i, &d, || LoginCommand::EnterChannel {
            context,
            channel_id: CHANNEL,
        });
        wait(&mut cs, &d, "lobby", |cs| {
            let v = cs[i].view();
            v.phase == LoginPhase::InLobby && v.canonical.equipment_supported() && !v.room_pending
        });
    }
    let contexts = cs.each_ref().map(|c| c.view().navigation.unwrap().context);
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
                        name: "Greater Haste controllers".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "create", |cs| {
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
    wait(&mut cs, &d, "join", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    for i in 0..2 {
        send(&mut cs, i, &d, || {
            LoginCommand::Rooms(rooms::Request::read(contexts[i], CHANNEL).unwrap())
        });
        wait(&mut cs, &d, "fresh room before ready", |cs| {
            !cs[i].view().room_pending
        });
        let room = cs[i].view().rooms.joined().unwrap().clone();
        send(&mut cs, i, &d, || {
            LoginCommand::Rooms(room_command(contexts[i], &room, rooms::Action::Ready, 1))
        });
        wait(&mut cs, &d, "ready", |cs| !cs[i].view().room_pending);
        assert!(
            cs[i]
                .view()
                .rooms
                .joined()
                .unwrap()
                .members
                .iter()
                .find(|m| m.key == contexts[i].key)
                .unwrap()
                .ready
        );
    }
    for c in &mut cs {
        c.load = true;
    }
    wait(&mut cs, &d, "active and rendered", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view()
                    .canonical
                    .lifecycle()
                    .is_some_and(|l| l.phase == world::Phase::Active)
        })
    });

    assert!(cs
        .iter()
        .all(|c| c.view().canonical.target_haste_supported()));
    for (i, c) in cs.iter().enumerate() {
        let v = c.view();
        let SkillsState::Ready(s) = v.skills else {
            panic!("source skills");
        };
        assert_eq!(s.context, contexts[i]);
        assert_eq!(s.haste().is_some(), i == 0);
    }
    assert!(battle_input::skill_available(43, &cs[0].view()));
    assert!(!battle_input::skill_available(43, &cs[1].view()));
    assert!(!battle_input::ready(
        Kind::Haste(self_target(cs[1].view().canonical.observation().unwrap())),
        &cs[1].view()
    ));
    let v = cs[0].view();
    let old = haste::Intent::from_observation(
        v.canonical.observation().unwrap(),
        self_target(v.canonical.observation().unwrap()),
    )
    .unwrap();
    let mut bad = v.clone();
    bad.skills = cs[1].view().skills;
    let s = cs[0].surface.as_ref().unwrap();
    assert!(battle_input::command(
        s,
        Intent {
            stamp: Stamp::of(&s.observation),
            kind: Kind::Haste(self_target(&s.observation))
        },
        &bad
    )
    .is_err());
    assert_eq!((self_mp(&cs[0]), self_mp(&cs[1])), (300, 300));
    let page = crate::modern_lito::battle_skillbook::model(
        cs[0].surface.as_ref().unwrap(),
        &cs[0].view(),
        |s| s.to_owned(),
    )
    .unwrap();
    let choice = page.choice(43).expect("qualified targeted skill row");
    assert!(matches!(
        crate::modern_lito::battle_skillbook::select(
            choice,
            cs[0].surface.as_ref().unwrap(),
            &cs[0].view()
        )
        .unwrap(),
        crate::modern_lito::battle_skillbook::Selection::Target(
            crate::modern_lito::battle_window::Skill::Haste
        )
    ));
    let surface = cs[0].surface.as_ref().unwrap();
    let wanted = self_target(&surface.observation);
    let viewport = crate::modern_lito::compositor::Viewport {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    };
    let own = surface
        .geometry
        .actors
        .iter()
        .find(|p| p.entity_id as i64 == surface.observation.data().personal.entity)
        .unwrap();
    let mut hit = None;
    for y in own.top_left.y.max(0)..(own.top_left.y + own.height as i32).min(600) {
        for x in own.top_left.x.max(0)..(own.top_left.x + own.width as i32).min(800) {
            let point = crate::modern_lito::world_map::PixelPoint { x, y };
            if crate::modern_lito::battle_window::hit(
                surface,
                viewport,
                point,
                crate::modern_lito::battle_window::Skill::Haste,
            )
            .is_some_and(|h| h.intent.kind == Kind::Haste(wanted))
            {
                hit = Some(point);
                break;
            }
        }
        if hit.is_some() {
            break;
        }
    }
    let point = hit.expect("actual shared self body must be selectable for support spell");
    assert_ne!(
        crate::modern_lito::battle_window::hit(
            surface,
            viewport,
            point,
            crate::modern_lito::battle_window::Skill::Melee
        )
        .unwrap()
        .intent
        .kind,
        Kind::Attack(wanted)
    );
    let q = cast_command(&cs[0]);
    let prior = cs[0].view().canonical.receipt_epoch();
    send(&mut cs, 0, &d, || LoginCommand::Canonical(q.clone()));
    wait(&mut cs, &d, "v44 exact cast receipt", |cs| {
        cs[0].view().canonical.received_after(&q, prior).is_some()
    });
    assert_eq!(
        cs[0].view().canonical.outcome(),
        Some(Outcome::TargetHaste(haste::Reason::None))
    );
    assert_eq!((self_mp(&cs[0]), self_mp(&cs[1])), (260, 300));
    assert!(!old.current(cs[0].view().canonical.observation().unwrap()));
    assert!(cs[0]
        .view()
        .canonical
        .command_notice()
        .contains("已由伺服器確認"));
    let accepted = cs[0].view().canonical.observation().unwrap().clone();
    let cast_revision = accepted.data().world.revision;
    let accepted_buff = accepted
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == old.stamp.entity)
        .unwrap()
        .buffs
        .as_ref()
        .unwrap();
    wait(&mut cs, &d, "v44 confirmed haste pixels", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.surface.as_ref().is_some_and(|s| {
                    s.observation.data().world.revision >= cast_revision
                        && s.observation.data().actors.0.iter().any(|a| {
                            a.entity == old.stamp.entity
                                && a.vital.as_ref().is_some_and(|v| v.values.mp == 260)
                                && a.buffs.as_ref().is_some_and(|b| {
                                    b.revision == accepted_buff.revision
                                        && b.haste_origin == accepted_buff.haste_origin
                                        && b.haste_until_nanos == accepted_buff.haste_until_nanos
                                })
                                && a.spells.histories.0.iter().any(|h| {
                                    h.lane == crate::modern_lito::battle_payload::data::Lane::Haste
                                        && h.receipt.as_ref().is_some_and(|r| {
                                            r.impact.usage.revision == cast_revision
                                        })
                                })
                        })
                })
        })
    });
    // No automatic retry: the old proof must be refused, not rebased to current resources.
    assert!(cs[0]
        .controller
        .command(LoginCommand::Canonical(Command::target_haste(old)))
        .is_err());
    let w = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .identity
        .clone();
    let revision = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .revision;
    wait(&mut cs, &d, "v44 cast recovery", |cs| {
        cs.iter().all(|c| c.rendered())
            && cs[0].view().canonical.observation().is_some_and(|o| {
                o.data().world.loading_elapsed_nanos
                    >= o.data()
                        .actors
                        .0
                        .iter()
                        .find(|a| a.entity == o.data().personal.entity)
                        .unwrap()
                        .spells
                        .histories
                        .0
                        .iter()
                        .find(|h| h.lane == crate::modern_lito::battle_payload::data::Lane::Haste)
                        .unwrap()
                        .receipt
                        .as_ref()
                        .unwrap()
                        .loading_elapsed_nanos
                        + 2_000_000_000
            })
    });
    let s = cs[0].surface.as_ref().unwrap();
    let v = cs[0].view();
    let o = v.canonical.observation().unwrap();
    let a = o
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .unwrap();
    let from = WorldCell {
        x: a.position.x,
        y: a.position.y,
    };
    let (h, to) = crate::modern_lito::world_map::DIRECTIONS
        .iter()
        .enumerate()
        .find_map(|(h, (dx, dy))| {
            let to = WorldCell {
                x: from.x + dx,
                y: from.y + dy,
            };
            s.navigation
                .as_ref()
                .unwrap()
                .can_step(from, to)
                .then_some((h as u8, to))
        })
        .unwrap();
    let q = battle_input::command(
        s,
        Intent {
            stamp: Stamp::of(&s.observation),
            kind: Kind::Local(Local::Step(h)),
        },
        &v,
    )
    .unwrap();
    let prior = v.canonical.receipt_epoch();
    send(&mut cs, 0, &d, || LoginCommand::Canonical(q.clone()));
    wait(&mut cs, &d, "v44 move after cast", |cs| {
        cs[0].view().canonical.received_after(&q, prior).is_some()
    });
    assert!(matches!(
        cs[0].view().canonical.outcome(),
        Some(Outcome::Operation(
            _,
            crate::modern_lito::battle_operation::Disposition::Applied,
            _
        ))
    ));
    let v = cs[0].view();
    let o = v.canonical.observation().unwrap();
    let a = o
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .unwrap();
    assert_eq!((a.position.x, a.position.y), (to.x, to.y));
    assert!(o.data().world.revision > revision);
    for i in 0..2 {
        let q = Command::world(battle_world::Request {
            action: battle_world::Action::Leave,
            ..battle_world::Request::read(contexts[i], CHANNEL, Some(&w))
        });
        send(&mut cs, i, &d, || LoginCommand::Canonical(q.clone()));
        wait(&mut cs, &d, "v44 leave", |cs| {
            !cs[i].view().world_pending && !cs[i].view().canonical.attached()
        });
        assert!(cs[i].view().canonical.observation().is_none());
        assert!(
            cs[i].surface.is_none(),
            "returned client must retire battle pixels"
        );
        assert!(!battle_input::skill_available(43, &cs[i].view()));
        assert!(cs[i]
            .controller
            .command(LoginCommand::Canonical(Command::target_haste(old)))
            .is_err());
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    loop {
        for c in &mut cs {
            c.controller.poll();
            c.runtime.poll();
        }
        if cs
            .iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
        {
            break;
        }
        d.pause("owned controller/resource shutdown");
    }
    peer.expect_successful_exit(&d);
}
