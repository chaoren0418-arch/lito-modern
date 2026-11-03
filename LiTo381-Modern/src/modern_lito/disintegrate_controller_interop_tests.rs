//! Production SessionController -> typed TLS command -> Java authority -> SQL.
//! Real shared-resource first render; Win32 mouse/device acceptance is separate.
use super::*;

#[track_caller]
fn assert_no_predicted_disintegrate(
    before: &combat::Frame,
    view: &LoginSnapshot,
    caster: u64,
    sequence: u64,
) {
    let current = view.combat.as_ref().unwrap();
    assert_eq!(current.world.match_id, before.world.match_id);
    assert_eq!(current.world.generation, before.world.generation);
    assert_eq!(current.world.phase, before.world.phase);
    assert!(current.world.revision >= before.world.revision);
    assert_eq!(view.world.frame(), Some(&current.world));
    // command() polls already-published authority before validating/reserving.
    // Background READ revisions are legal; a locally predicted cast is not.
    assert_eq!(current.world.cast(caster), before.world.cast(caster));
    assert!(current.world.cast(caster).unwrap().request_sequence < sequence);
    assert_eq!(current.disintegrations, before.disintegrations);
    for prior in &before.vitals {
        let next = current.vital(prior.entity_id).unwrap();
        let a = before.regen(prior.entity_id).unwrap();
        let b = current.regen(prior.entity_id).unwrap();
        assert_eq!(
            next.mp - prior.mp,
            (b.mp_gain - a.mp_gain) - (b.mp_loss - a.mp_loss),
            "no unconfirmed MP deduction"
        );
        assert_eq!(
            next.hp - prior.hp,
            (b.hp_gain - a.hp_gain) - (b.hp_loss - a.hp_loss) + (b.item_gain - a.item_gain)
                - (b.damage_loss - a.damage_loss),
            "HP changes require the confirmed recovery/damage ledger"
        );
    }
    if current.world.revision == before.world.revision {
        assert_eq!(current.as_ref(), before);
    }
}

#[test]
#[ignore = "v27 production controller, shared514/SPR/717 first render and fresh isolated MySQL; no HWND"]
fn java_disintegrate_player_controller_catalog_input_kill_record_return_and_shutdown() {
    run_controller(false);
}

#[test]
#[ignore = "v37 controller life-bound skill77 + v27 peer coexistence, real shared514 and isolated SQL; no HWND"]
fn java_v37_disintegrate_controller_guarded_kill_saved_return_with_v27_peer() {
    run_controller(true);
}

fn canonical_image(
    runtime: &mut crate::modern_lito::battle_runtime::BattleRuntime,
    owner: &mut SessionController,
    guest: &mut ModernTlsConnection,
    phase: crate::modern_lito::battle_payload::data::Phase,
    deadline: &Deadline,
) -> Arc<crate::modern_lito::battle_scene::Surface> {
    let mut keepalive = Instant::now();
    loop {
        owner.poll();
        let view = owner.snapshot();
        match crate::modern_lito::battle_player::update(runtime, &view, false).unwrap() {
            crate::modern_lito::battle_runtime::Status::Ready(surface)
                if surface.observation.data().world.phase == phase
                    && crate::modern_lito::battle_player::controls_ready(&surface, &view) =>
            {
                assert!(runtime.prepared(&surface));
                assert_eq!(surface.bgra.len(), 800 * 600 * 4);
                assert!(surface.bgra.chunks_exact(4).all(|p| p[3] == 255));
                if let Some((_, audio)) = runtime.audio_view().publication {
                    if audio.frame.is_some() {
                        assert_eq!(audio.gain, Some(50));
                    }
                }
                return surface;
            }
            crate::modern_lito::battle_runtime::Status::Failed(reason) => {
                panic!("canonical controller renderer failed: {reason}")
            }
            _ => {}
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            guest.ping().unwrap();
            keepalive = Instant::now();
        }
        deadline.pause("normal controller canonical resources/pixels");
    }
}

fn run_controller(life_actions: bool) {
    let d = Deadline::new();
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &d,
        "local.lito381.server.ModernDisintegrateTestPeer",
        if life_actions {
            "life-disintegrate"
        } else {
            "disintegrate"
        },
    );
    let mut owner = SessionController::new(profile(peer.endpoint, &shared, &ca));
    submit(&mut owner, &d, "dis controller connect", || {
        LoginCommand::Connect
    });
    wait_owner(&mut owner, &d, "dis controller TLS", |v| {
        v.phase == LoginPhase::LoginReady
    });
    submit(&mut owner, &d, "dis controller login", || {
        LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        }
    });
    wait_owner(&mut owner, &d, "dis controller sources", |v| {
        v.phase == LoginPhase::ChoosingSource
    });
    submit(&mut owner, &d, "dis controller source", || {
        LoginCommand::SelectSource(SUN.source_id)
    });
    let roster = wait_owner(&mut owner, &d, "dis controller roster", |v| {
        v.phase == LoginPhase::CharactersReady
    });
    let sc = SelectionContext {
        key: SUN,
        epoch: roster.roster_epoch,
    };
    submit(&mut owner, &d, "dis controller selection", || {
        LoginCommand::SelectCharacter(sc)
    });
    let selected = wait_owner(&mut owner, &d, "dis controller qualified catalog", |v| {
        v.phase == LoginPhase::ChoosingChannel
            && !v.spell_pending
            && v.skills.disintegrate().is_some()
            && !v.record_pending
            && v.record.totals().is_some()
    });
    assert_eq!(selected.action_supported, life_actions);
    assert_eq!(selected.battle_supported, life_actions);
    assert_eq!(selected.record.totals().unwrap().revision, 0);
    assert_eq!(selected.skills.disintegrate().unwrap().mp_cost, 70);
    submit(&mut owner, &d, "dis controller channel", || {
        LoginCommand::EnterChannel {
            context: sc,
            channel_id: CHANNEL,
        }
    });
    wait_owner(&mut owner, &d, "dis controller lobby", |v| {
        v.phase == LoginPhase::InLobby
    });
    for _ in 0..5 {
        submit(&mut owner, &d, "dis controller shop", || {
            LoginCommand::Shop(shop::Request::read(sc, CHANNEL, None))
        });
        wait_owner(&mut owner, &d, "dis controller shop result", |v| {
            !v.shop_pending
        });
    }
    let (mut moon, mc) = connect(&peer, &ca, MOON);
    submit(&mut owner, &d, "dis controller create", || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(sc, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "Controller skill77".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    let created = wait_owner(&mut owner, &d, "dis controller own room", |v| {
        !v.room_pending && v.rooms.joined().is_some()
    });
    let room = created.rooms.joined().unwrap();
    let directory = moon
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap();
    let summary = directory
        .rooms
        .iter()
        .find(|r| r.room_id == room.summary.room_id)
        .unwrap();
    moon.rooms(
        rooms::Request::join(
            rooms::Common {
                action: rooms::Action::Join,
                room_id: summary.room_id,
                expected_revision: summary.revision,
                ..rooms::Common::read(mc, CHANNEL)
            },
            String::new(),
        )
        .unwrap(),
    )
    .unwrap();
    let joined = wait_owner(&mut owner, &d, "dis controller guest joined", |v| {
        v.rooms.joined().is_some_and(|r| r.members.len() == 2)
    });
    submit(&mut owner, &d, "dis controller ready", || {
        LoginCommand::Rooms(room_command(
            sc,
            joined.rooms.joined().unwrap(),
            rooms::Action::Ready,
            1,
        ))
    });
    wait_owner(&mut owner, &d, "dis controller ready ack", |v| {
        !v.room_pending
    });
    let room = moon
        .rooms(rooms::Request::read(mc, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    moon.rooms(room_command(mc, &room, rooms::Action::Ready, 1))
        .unwrap();
    let preparation_started = Instant::now();
    let loading = wait_owner(&mut owner, &d, "dis controller auto prepare", |v| {
        v.combat
            .as_ref()
            .is_some_and(|f| f.world.phase == world::Phase::Loading)
    })
    .combat
    .unwrap();
    assert!(loading
        .world
        .descriptor
        .unwrap()
        .has(world::MatchDescriptor::DISINTEGRATE));
    let preparation_elapsed = preparation_started.elapsed();
    let read_started = Instant::now();
    let read_result = moon.world(world_request(mc, &loading.world, world::Action::Read));
    if let Err(failure) = &read_result {
        // Do not kill the peer at unwrap before its bounded session diagnostics
        // can report the actual failure. No retry or successful-flow substitution.
        eprintln!(
            "DISINTEGRATE_PRELOAD_READ wait_ms={} read_ms={} error={failure:?}",
            preparation_elapsed.as_millis(),
            read_started.elapsed().as_millis()
        );
        drop(owner);
        drop(moon);
        peer.expect_successful_exit(&d);
        panic!("Disintegrate preload read failed: {failure:?}");
    }
    read_result.unwrap();
    let preferences = Arc::new(crate::modern_lito::preferences::SharedPreferences::default());
    preferences
        .apply(crate::modern_lito::preferences::Preferences {
            volume_percent: 50,
            ..Default::default()
        })
        .unwrap();
    let mut runtime = WorldRuntime::with_preferences(shared.clone(), preferences.clone()).unwrap();
    let mut canonical =
        crate::modern_lito::battle_runtime::BattleRuntime::with_preferences(shared, preferences)
            .unwrap();
    let loaded = if life_actions {
        // Same canonical readiness and LOAD builder as the real window. No
        // legacy renderer is started before the actual server LOAD ACK.
        let mut keepalive = Instant::now();
        loop {
            owner.poll();
            let view = owner.snapshot();
            let status =
                crate::modern_lito::battle_player::update(&mut canonical, &view, false).unwrap();
            if let crate::modern_lito::battle_runtime::Status::Ready(surface) = status {
                if let Some(request) =
                    crate::modern_lito::battle_player::loaded_request(&canonical, &surface, &view)
                {
                    assert!(runtime.is_quiescent());
                    assert!(canonical.prepared(&surface));
                    assert_eq!(surface.bgra.len(), 800 * 600 * 4);
                    if let Some((_, audio)) = canonical.audio_view().publication {
                        if audio.frame.is_some() {
                            assert_eq!(audio.gain, Some(50));
                        }
                    }
                    break request;
                }
            } else if let crate::modern_lito::battle_runtime::Status::Failed(reason) = status {
                panic!("canonical controller preflight failed: {reason}");
            }
            if keepalive.elapsed() >= Duration::from_secs(1) {
                moon.ping().unwrap();
                keepalive = Instant::now();
            }
            d.pause("controller canonical full-resource LOAD barrier");
        }
    } else {
        let image = bot_combat_image(&mut runtime, &loading, &mut owner, &d);
        assert!(image.ready_to_ack);
        assert_eq!(image.audio_gain, Some(50));
        capture(&image, "controller-loading");
        world::Request {
            manifest_digest: image.manifest_digest,
            ..world_request(sc, &loading.world, world::Action::Loaded)
        }
    };
    submit(&mut owner, &d, "dis controller first-render LOAD", || {
        LoginCommand::World(loaded)
    });
    wait_owner(&mut owner, &d, "dis controller LOAD ack", |v| {
        !v.world_pending
    });
    moon.world(world::Request {
        manifest_digest: loading.world.manifest.digest,
        ..world_request(mc, &loading.world, world::Action::Loaded)
    })
    .unwrap();
    let active = wait_owner(&mut owner, &d, "dis controller active", |v| {
        v.combat
            .as_ref()
            .is_some_and(|f| f.world.phase == world::Phase::Active)
    })
    .combat
    .unwrap();
    if life_actions {
        canonical_image(
            &mut canonical,
            &mut owner,
            &mut moon,
            crate::modern_lito::battle_payload::data::Phase::Active,
            &d,
        );
        assert!(
            runtime.is_quiescent(),
            "normal canonical UI does not start an old renderer after LOAD"
        );
    } else {
        bot_combat_image(&mut runtime, &active, &mut owner, &d);
    }
    let sid = actor(&active.world, SUN).entity_id;
    let mid = actor(&active.world, MOON).entity_id;
    let current = read(&mut moon, mc, &active.world);
    let denied = moon.disintegrate(request(&current, mc, sid)).unwrap();
    assert_eq!(denied.reason, dis::Reason::Unlearned);
    let denied_sequence = denied
        .frame
        .as_ref()
        .unwrap()
        .world
        .cast(mid)
        .unwrap()
        .request_sequence;
    let view = wait_owner(
        &mut owner,
        &d,
        "dis controller observes guest denial",
        |v| {
            v.combat
                .as_ref()
                .is_some_and(|f| f.world.cast(mid).unwrap().request_sequence == denied_sequence)
                && (!life_actions
                    || crate::modern_lito::battle_action::for_command(
                        v,
                        &LoginCommand::CastDisintegrate(request(
                            v.combat.as_ref().unwrap(),
                            sc,
                            mid,
                        )),
                    )
                    .is_ok_and(|g| g.is_some()))
        },
    );
    let before = view.combat.as_ref().unwrap();
    let good = request(before, sc, mid);
    let mut stale = good;
    stale.target_life += 1;
    assert!(owner
        .command(LoginCommand::CastDisintegrate(stale))
        .is_err());
    assert!(!owner.snapshot().spell_pending);
    assert_no_predicted_disintegrate(before, &owner.snapshot(), sid, good.sequence);
    submit(&mut owner, &d, "dis controller typed cast", || {
        LoginCommand::CastDisintegrate(good)
    });
    assert!(owner.snapshot().spell_pending);
    assert_no_predicted_disintegrate(before, &owner.snapshot(), sid, good.sequence);
    assert!(owner.command(LoginCommand::CastDisintegrate(good)).is_err());
    assert!(owner.command(LoginCommand::CastTeleport).is_err());
    let result = wait_owner(&mut owner, &d, "dis controller exact cast ack", |v| {
        !v.spell_pending
            && v.combat
                .as_ref()
                .is_some_and(|f| f.disintegrate(sid).unwrap().count == 1)
    });
    let finished = result.combat.unwrap();
    assert_eq!(finished.world.phase, world::Phase::Finished);
    assert_eq!(finished.vital(mid).unwrap().hp, 0);
    let receipt = finished.disintegrate(sid).unwrap();
    assert_eq!(receipt.lawful, 501);
    assert_eq!(
        finished.vital(sid).unwrap().mp,
        before.vital(sid).unwrap().mp - i64::from(receipt.mp_cost)
    );
    assert_eq!(
        finished.world.cast(sid).unwrap().request_sequence,
        good.sequence
    );
    if life_actions {
        let terminal = canonical_image(
            &mut canonical,
            &mut owner,
            &mut moon,
            crate::modern_lito::battle_payload::data::Phase::Finished,
            &d,
        );
        assert_eq!(
            terminal
                .observation
                .data()
                .actors
                .0
                .iter()
                .find(|a| a.entity == mid as i64)
                .unwrap()
                .vital
                .as_ref()
                .unwrap()
                .values
                .hp,
            0
        );
        assert!(terminal.observation.data().result.is_some());
        assert!(runtime.is_quiescent());
    } else {
        let terminal = bot_combat_image(&mut runtime, &finished, &mut owner, &d);
        assert_eq!(terminal.audio_gain, Some(50));
        capture(&terminal, "controller-finished");
    }
    assert!(owner.command(LoginCommand::CastDisintegrate(good)).is_err());
    let saved = wait_owner(&mut owner, &d, "dis controller saved SQL", |v| {
        v.world
            .frame()
            .and_then(|w| w.completion.as_ref())
            .is_some_and(|c| c.state == world::SettlementState::Saved)
    });
    let observed = read(&mut moon, mc, saved.world.frame().unwrap());
    assert_eq!(
        observed.disintegrations,
        saved.combat.as_ref().unwrap().disintegrations
    );
    let lost = moon.record().unwrap().totals.unwrap();
    assert_eq!((lost.wins, lost.losses, lost.revision), (0, 1, 1));
    assert_eq!(moon.record().unwrap().totals, Some(lost));
    let leave = world_request(sc, saved.world.frame().unwrap(), world::Action::Leave);
    submit(&mut owner, &d, "dis controller leave", || {
        LoginCommand::World(leave)
    });
    let left = wait_owner(&mut owner, &d, "dis controller room return", |v| {
        !v.world_pending && v.world.frame().is_none()
    });
    assert!(left.combat.is_none());
    assert_eq!(left.navigation.as_ref().unwrap().context, sc);
    moon.world(world_request(mc, &finished.world, world::Action::Leave))
        .unwrap();
    let record = wait_owner(&mut owner, &d, "dis controller persisted record", |v| {
        !v.record_pending
            && v.record
                .totals()
                .is_some_and(|t| t.wins == 1 && t.losses == 0 && t.revision == 1)
    });
    assert_eq!(record.record.context(), Some(sc));
    let returned = wait_owner(&mut owner, &d, "dis controller ready reset", |v| {
        v.rooms
            .joined()
            .is_some_and(|r| r.summary.ready == 0 && !r.summary.satisfied)
    });
    assert_eq!(
        returned.rooms.joined().unwrap().summary.room_id,
        room.summary.room_id
    );
    runtime.shutdown();
    canonical.shutdown();
    while !runtime.is_quiescent() || !canonical.is_quiescent() {
        runtime.poll();
        canonical.poll();
        owner.poll();
        d.pause("dis controller renderer retire");
    }
    moon.close().unwrap();
    submit(&mut owner, &d, "dis controller normal shutdown", || {
        LoginCommand::Shutdown
    });
    while !owner.is_quiescent() {
        owner.poll();
        d.pause("dis controller TLS retire");
    }
    assert_eq!(owner.snapshot().phase, LoginPhase::Disconnected);
    peer.expect_successful_exit(&d);
}
