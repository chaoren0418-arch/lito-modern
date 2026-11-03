//! Actual Main/SQL/TLS and ordinary controllers: schema8/missing11101 cannot
//! pass the resource barrier. A separately named gfx0 fixture must qualify
//! shared228 and enter through the real runtime. Both return to an ordinary
//! map on the same connections; no operator source changes or forged LOAD.
use super::*;

fn armor_profile(
    endpoint: SocketAddr,
    shared: &PathBuf,
    fixtures: &PathBuf,
    extension: &str,
) -> ModernProfile {
    let mut doc = profile_document(endpoint, shared, &private_root(fixtures));
    doc["schema"] = serde_json::json!(2);
    doc["gameplay_protocol"] = serde_json::json!("modern_v47");
    doc["gameplay_extension"] = serde_json::json!(extension);
    ModernProfile::parse(&serde_json::to_vec(&doc).unwrap()).unwrap()
}

#[test]
#[ignore = "ordinary armor profiles against a healing-only Main/owned SQL; no fallback authentication"]
fn java_armor_main_sql_profiles_reject_healing_only_listener_before_authentication() {
    reject_profiles(&["armor_protection_v1", "player_healing_armor_v1"]);
}
#[test]
#[ignore = "new required presentation profiles against actual healing-only Main/owned SQL/TLS; reject before auth"]
fn java_armor_main_sql_presentation_profiles_reject_before_authentication() {
    reject_profiles(&[
        "armor_presentation_v1",
        "player_healing_armor_presentation_v1",
    ]);
}
fn reject_profiles(extensions: &[&str]) {
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    for extension in extensions {
        let d = Deadline(Instant::now() + Duration::from_secs(45));
        let mut peer = JavaPeer::start_mode_lifetime(
            &fixtures,
            &d,
            "local.lito381.server.ModernArmorProtectionSqlTestPeer",
            "armor-reject-sql",
            Some("45000"),
        );
        let mut c =
            SessionController::new(armor_profile(peer.endpoint, &shared, &fixtures, extension));
        c.command(LoginCommand::Connect).unwrap();
        loop {
            c.poll();
            let v = c.snapshot();
            assert_ne!(v.phase, LoginPhase::LoginReady);
            if v.phase == LoginPhase::Failed {
                break;
            }
            d.pause("required armor pre-auth rejection");
        }
        c.command(LoginCommand::Shutdown).unwrap();
        while !c.is_quiescent() {
            c.poll();
            d.pause("rejected armor worker retirement");
        }
        peer.expect_successful_exit(&d);
    }
    eprintln!("ARMOR_PROFILE_REJECT: both exact required profiles rejected before auth; no reconnect or legacy fallback");
}

fn pump(c: &mut Client) {
    c.controller.poll();
    let v = c.view();
    assert_ne!(v.phase, LoginPhase::Failed, "{}", v.status);
    if v.canonical.observation().is_some_and(|o| {
        o.data().armor.as_ref().is_some_and(|a| {
            a.presentation
                .as_ref()
                .is_none_or(|p| p.required.graphics.0.contains(&11101))
        })
    }) {
        // Observe the real full decoder/controller state without pretending
        // that a data-only fixture has satisfied the player's resource barrier.
        login_window::validate_snapshot(&v).unwrap();
        assert!(
            v.world.frame().is_none() && v.combat.is_none() && v.battle.observation().is_none()
        );
        assert!(!c.load && c.surface.is_none());
    } else {
        c.pump();
    }
}
fn wait(
    cs: &mut [Client; 2],
    d: &Deadline,
    label: &'static str,
    done: impl Fn(&[Client; 2]) -> bool,
) {
    loop {
        for c in cs.iter_mut() {
            pump(c);
        }
        if done(cs) {
            return;
        }
        d.pause(label);
    }
}
fn send(cs: &mut [Client; 2], i: usize, d: &Deadline, build: impl Fn() -> LoginCommand) {
    loop {
        for c in cs.iter_mut() {
            pump(c);
        }
        if cs[i].controller.command(build()).is_ok() {
            return;
        }
        d.pause("armor local inbox admission");
    }
}
fn room_action(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    context: SelectionContext,
    action: rooms::Action,
    value: u8,
) {
    send(cs, i, d, || {
        LoginCommand::Rooms(rooms::Request::read(context, CHANNEL).unwrap())
    });
    wait(cs, d, "armor room read", |cs| !cs[i].view().room_pending);
    let room = cs[i].view().rooms.joined().unwrap().clone();
    send(cs, i, d, || {
        LoginCommand::Rooms(room_command(context, &room, action, value))
    });
    wait(cs, d, "armor room mutation", |cs| {
        !cs[i].view().room_pending
    });
    let rooms::State::Ready(reply) = cs[i].view().rooms else {
        panic!("room reply")
    };
    assert_eq!(reply.status, rooms::Status::Ok);
}
fn armor_equipment(cs: &mut [Client; 2], i: usize, d: &Deadline, desired: bool) {
    let context = gear_snapshot(&cs[i]).source.context;
    send(cs, i, d, || {
        LoginCommand::Equipment(equipment::Request::read(context))
    });
    wait(cs, d, "armor equipment token", |cs| {
        !cs[i].view().equipment_pending
    });
    let before = gear_snapshot(&cs[i]);
    assert_eq!(before.equipped(3000089), Some(!desired));
    let request = equipment::Request::set(&before, 3000089, desired);
    send(cs, i, d, || LoginCommand::Equipment(request));
    wait(cs, d, "durable armor equipment receipt", |cs| {
        !cs[i].view().equipment_pending
    });
    let after = gear_snapshot(&cs[i]);
    assert_eq!(after.revision, before.revision + 1);
    assert_eq!(after.source.context, context);
    assert_eq!(after.equipped(3000089), Some(desired));
    eprintln!(
        "ARMOR_PRESENTATION_EQUIPMENT source={} revision={} equipped={desired}",
        context.key.source_id, after.revision
    );
}
fn room(cs: &mut [Client; 2], d: &Deadline, contexts: [SelectionContext; 2], map: u32) {
    send(cs, 0, d, || {
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
                        attributes: 0,
                        map_id: map,
                        name: "armor routing".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(cs, d, "armor room created", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
    let summary = cs[0].view().rooms.joined().unwrap().summary.clone();
    send(cs, 1, d, || {
        LoginCommand::Rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: summary.room_id,
                    expected_revision: summary.revision,
                    ..rooms::Common::read(contexts[1], CHANNEL)
                },
                String::new(),
            )
            .unwrap(),
        )
    });
    wait(cs, d, "armor room joined", |cs| {
        cs.iter().all(|c| {
            !c.view().room_pending
                && c.view()
                    .rooms
                    .joined()
                    .is_some_and(|r| r.members.len() == 2)
        })
    });
    for i in 0..2 {
        room_action(cs, i, d, contexts[i], rooms::Action::Ready, 1);
    }
}
fn leave(cs: &mut [Client; 2], d: &Deadline, contexts: [SelectionContext; 2]) {
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
        wait(cs, d, "armor world departure", |cs| {
            !cs[i].view().world_pending && cs[i].view().canonical.observation().is_none()
        });
    }
    for i in (0..2).rev() {
        room_action(cs, i, d, contexts[i], rooms::Action::Leave, 0);
    }
    wait(cs, d, "old renderer retired", |cs| {
        cs.iter()
            .all(|c| c.runtime.is_quiescent() && c.surface.is_none())
    });
}

#[test]
#[ignore = "two ordinary armor/healing controllers, Main/owned SQL/TLS; full armor LOADING and safe return, real ordinary-map shared LOAD; no armor render/active/UI claim"]
fn java_armor_main_sql_controller_preserves_loading_and_returns_to_ordinary_world() {
    controller_roundtrip("player_healing_armor_v1", false, false);
}
#[test]
#[ignore = "schema9 ordinary profile/Main/owned SQL/TLS; owner presentation LOADING then real ordinary-map LOAD; no armor rendering claim"]
fn java_armor_main_sql_presentation_controller_preserves_loading_and_returns_to_ordinary_world() {
    controller_roundtrip("player_healing_armor_presentation_v1", true, false);
}
#[test]
#[ignore = "actual Main/isolated SQL/TLS, explicitly seeded gfx0, ordinary schema9 controller/shared installed map LOAD/ACTIVE/proc and safe return; not native UI or production11101"]
fn java_armor_main_sql_explicit_gfx0_shared_runtime_loads_and_renders_proc() {
    controller_roundtrip("player_healing_armor_presentation_v1", true, true);
}
fn controller_roundtrip(extension: &str, presentation: bool, render: bool) {
    let d = Deadline(Instant::now() + Duration::from_secs(180));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernArmorProtectionSqlTestPeer",
        if render {
            "armor-render-sql"
        } else {
            "armor-sql"
        },
        Some("180000"),
    );
    let profile = armor_profile(peer.endpoint, &shared, &fixtures, extension);
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
        wait(&mut cs, &d, "armor TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "armor authentication", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] =
            healing_sql::select_healer_hp(&mut cs, i, &d, if i == 0 { 7 } else { 42 }, 10000);
        enter(&mut cs, i, &d, contexts[i]);
        assert!(cs[i].view().canonical.player_healing_supported());
    }
    if presentation {
        // The existing SQL peer begins with proc armor unequipped. Exercise
        // real durable owner SETs, never replace the read-only imported rows.
        for i in 0..2 {
            armor_equipment(&mut cs, i, &d, true);
        }
    }
    room(&mut cs, &d, contexts, 514);
    wait(&mut cs, &d, "both full armor loading observations", |cs| {
        cs.iter().all(|c| {
            c.view()
                .canonical
                .observation()
                .is_some_and(|o| o.data().armor.is_some())
        })
    });
    let saved = cs
        .each_ref()
        .map(|c| c.view().canonical.observation().unwrap().clone());
    for (i, o) in saved.iter().enumerate() {
        assert_eq!(o.data().world.phase, data::Phase::Loading);
        assert_eq!(
            o.data().personal.player.source,
            contexts[i].key.source_id as i32
        );
        let armor = o.data().armor.as_ref().unwrap();
        assert_eq!(armor.presentation.is_some(), presentation);
        if let Some(p) = &armor.presentation {
            assert_eq!(p.entity, o.data().personal.entity);
            assert_eq!(p.world, o.data().world.identity);
            assert_eq!(p.world_revision, o.data().world.revision);
            assert!(p.batch.is_none());
            assert_eq!(
                p.required.graphics.0,
                if render { vec![228] } else { vec![228, 11101] }
            );
            assert_eq!(p.required.icons.0, vec![1562]);
            assert_eq!(
                &crate::modern_lito::battle_payload::encode(o.data()).unwrap()[4..6],
                &9u16.to_be_bytes()
            );
        }
        assert!(armor.healing.is_some());
        assert!(o.data().healing.is_none());
        assert_ne!(
            o.data().world.descriptor.features & crate::modern_lito::battle_payload::armor::FEATURE,
            0
        );
        assert_eq!(armor.actors.0.len(), 2);
        assert!(armor.actors.0.iter().all(|a| a
            .state
            .as_ref()
            .is_some_and(|s| s.sequence == 0 && s.until_nanos == 0)));
        assert_eq!(
            crate::modern_lito::battle_scene::baseline_counts(o.clone()).is_ok(),
            presentation
        );
        // Actual shared load must reject missing11101, not merely a synthetic
        // scene-input check. Schema8 still fails before resource qualification.
        if !render {
            let error = match crate::modern_lito::battle_scene::SharedScene::load(
                &shared,
                o.clone(),
                &std::sync::atomic::AtomicBool::new(false),
            ) {
                Ok(_) => panic!("unqualified armor resources cannot issue LOAD"),
                Err(e) => format!("{e:#}"),
            };
            assert!(
                error.contains(if presentation {
                    "armor graphic 11101"
                } else {
                    "armor presentation metadata required"
                }),
                "{error}"
            );
        }
    }
    assert_ne!(
        saved[0].data().personal.player,
        saved[1].data().personal.player
    );
    if render {
        for c in &mut cs {
            c.load = true;
        }
        wait(
            &mut cs,
            &d,
            "schema9 actual shared resource LOAD/ACTIVE",
            |cs| {
                cs.iter().all(|c| {
                    c.rendered()
                        && c.view().canonical.observation().unwrap().data().world.phase
                            == data::Phase::Active
                })
            },
        );
        let victim = own_actor(&cs[1]);
        let target = Target::Player {
            entity: victim.entity,
            life: victim.resources.as_ref().unwrap().life + 1,
        };
        for _ in 0..12 {
            if own_actor(&cs[1]).resources.as_ref().unwrap().vitals.hp < 10000 {
                break;
            }
            let result = input(&mut cs, 0, &d, Kind::Attack(target));
            assert_eq!(
                result.canonical.outcome(),
                Some(Outcome::Operation(
                    Rejection::None,
                    Disposition::Applied,
                    Reason::None
                ))
            );
            let revision = result
                .canonical
                .observation()
                .unwrap()
                .data()
                .world
                .revision;
            wait(
                &mut cs,
                &d,
                "defender receives acknowledged SQL attack",
                |cs| {
                    cs[1]
                        .view()
                        .canonical
                        .observation()
                        .is_some_and(|o| o.data().world.revision >= revision)
                },
            );
            if own_actor(&cs[1]).resources.as_ref().unwrap().vitals.hp >= 10000 {
                // A fully acknowledged miss may be followed by a NEW attack,
                // but never retry before the authoritative attack interval.
                let nanos = own_actor(&cs[0])
                    .vital
                    .as_ref()
                    .unwrap()
                    .attack_interval_nanos;
                let until = Instant::now()
                    + Duration::from_nanos(nanos.try_into().unwrap())
                    + Duration::from_millis(1);
                while Instant::now() < until {
                    for c in &mut cs {
                        pump(c);
                    }
                    d.pause("SQL proc attack pacing after an acknowledged miss");
                }
            }
        }
        wait(
            &mut cs,
            &d,
            "SQL armor protection proc reaches renderer",
            |cs| {
                let o = cs[1].view().canonical.observation().unwrap().clone();
                o.data().armor.as_ref().unwrap().actors.0.iter().any(|a| {
                    a.entity == victim.entity
                        && a.state
                            .as_ref()
                            .is_some_and(|s| s.sequence > 0 && s.until_nanos > 0)
                }) && cs[1].rendered()
                    && o.data()
                        .armor
                        .as_ref()
                        .unwrap()
                        .presentation
                        .as_ref()
                        .unwrap()
                        .batch
                        .as_ref()
                        .is_some_and(|batch| {
                            cs[1].surface.as_ref().is_some_and(|surface| {
                                surface.observation.data().world.revision >= batch.revision
                            })
                        })
            },
        );
        let o = cs[1].view().canonical.observation().unwrap().clone();
        assert_eq!(
            o.data()
                .armor
                .as_ref()
                .unwrap()
                .presentation
                .as_ref()
                .unwrap()
                .batch
                .as_ref()
                .unwrap()
                .visuals
                .0
                .len(),
            1
        );
        assert!(
            cs[1].audio.sequence(Stamp::of(&o), 132) > 0,
            "actual runtime emits shared132"
        );
        assert_eq!(
            cs[0].audio.sequence(
                Stamp::of(cs[0].view().canonical.observation().unwrap()),
                132
            ),
            0,
            "self proc is not broadcast"
        );
        eprintln!("ARMOR_SQL_GFX0_ACTIVE: real installed shared514 LOAD, two sources, actual SQL/attack/protection, owner-only shared228/132; no11101 substitution");
    } else {
        for c in &mut cs {
            loop {
                let view = c.view();
                match battle_player::update(&mut c.runtime, &view, false).unwrap() {
                    BattleStatus::Failed(_) => break,
                    BattleStatus::Ready(_) => panic!("missing armor assets minted a surface"),
                    _ => d.pause("actual negative armor runtime barrier"),
                }
            }
            assert!(c.surface.is_none() && !c.load);
            assert_eq!(
                c.view().canonical.observation().unwrap().data().world.phase,
                data::Phase::Loading
            );
        }
    }
    leave(&mut cs, &d, contexts);
    if presentation {
        for i in 0..2 {
            armor_equipment(&mut cs, i, &d, false);
        }
    }
    room(&mut cs, &d, contexts, 509);
    for c in &mut cs {
        c.load = true;
    }
    wait(
        &mut cs,
        &d,
        "same TLS connections load ordinary mapping",
        |cs| {
            cs.iter().all(|c| {
                c.rendered()
                    && c.view()
                        .canonical
                        .observation()
                        .is_some_and(|o| o.data().world.phase == data::Phase::Active)
            })
        },
    );
    for (i, c) in cs.iter().enumerate() {
        let v = c.view();
        let o = v.canonical.observation().unwrap();
        assert!(o.data().armor.is_none() && o.data().healing.is_some());
        assert_eq!(o.data().manifest.map, 514);
        assert_eq!(o.data().room.map, 509);
        assert_eq!(
            o.data().personal.player.source,
            contexts[i].key.source_id as i32
        );
        assert_ne!(o.data().world.identity, saved[i].data().world.identity);
    }
    let before = own_actor(&cs[1]).resources.unwrap().vitals.mp;
    healing_sql::heal(&mut cs, 1, 1, &d);
    assert!(own_actor(&cs[1]).resources.unwrap().vitals.mp < before);
    assert!(cs.iter().all(|c| c
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .armor
        .is_none()));
    leave(&mut cs, &d, contexts);
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::LeaveLobby(contexts[i]));
        wait(&mut cs, &d, "armor session leave lobby", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingChannel
        });
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "armor session return roster", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    // Reuse the same authenticated session after both source leases retire.
    let next = healing_sql::select_healer_hp(&mut cs, 0, &d, 42, 10000);
    assert!(next.epoch > contexts[0].epoch);
    assert_ne!(next.key, contexts[0].key);
    assert!(!cs[0].view().canonical.attached());
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Shutdown);
        cs[i].runtime.shutdown();
    }
    wait(&mut cs, &d, "armor clients retire", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
    eprintln!("ARMOR_MAIN_RUST_TLS: explicitGfx0Render={render}; full armor owner/real resource barrier, leave and same-connection ordinary schema7 LOAD/ACTIVE/heal/source switch; no native UI or production11101 success claim");
}
