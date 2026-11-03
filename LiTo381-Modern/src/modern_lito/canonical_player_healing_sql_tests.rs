//! Ordinary controller/profile -> production Main -> owned read-only SQL ->
//! shared renderer -> skill page -> painted hit -> healing. No injected provider,
//! reflection, native HWND/mouse or audio-device claim.
use super::*;
use crate::modern_lito::{battle_player_healing as healing, spell_protocol::SkillsState};
#[path = "canonical_player_healing_mixed_sql_tests.rs"]
mod mixed;
#[path = "canonical_player_healing_quest_sql_tests.rs"]
mod quest;

fn healing_profile(endpoint: SocketAddr, shared: &PathBuf, fixtures: &PathBuf) -> ModernProfile {
    let mut doc = profile_document(endpoint, shared, &private_root(fixtures));
    doc["schema"] = serde_json::json!(2);
    doc["gameplay_protocol"] = serde_json::json!("modern_v47");
    doc["gameplay_extension"] = serde_json::json!("player_healing_v1");
    ModernProfile::parse(&serde_json::to_vec(&doc).unwrap()).unwrap()
}

#[test]
#[ignore = "ordinary extended controller against non-healing production Main/owned SQL; one rejected connection, never legacy retry or authentication"]
fn java_healing_main_sql_ordinary_controller_rejects_incompatible_listener() {
    let d = Deadline(Instant::now() + Duration::from_secs(45));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernPlayerHealingSqlTestPeer",
        "healing-reject-sql",
        Some("45000"),
    );
    let mut controller = SessionController::new(healing_profile(peer.endpoint, &shared, &fixtures));
    controller.command(LoginCommand::Connect).unwrap();
    loop {
        controller.poll();
        let v = controller.snapshot();
        assert_ne!(
            v.phase,
            LoginPhase::LoginReady,
            "incompatible listener cannot request credentials"
        );
        if v.phase == LoginPhase::Failed {
            break;
        }
        d.pause("explicit healing handshake rejection");
    }
    controller.command(LoginCommand::Shutdown).unwrap();
    while !controller.is_quiescent() {
        controller.poll();
        d.pause("rejected worker retirement");
    }
    peer.expect_successful_exit(&d);
    eprintln!("HEALING_MAIN_REJECT: ordinary configured worker, exactly one failed pre-auth connection to non-healing Main, no legacy fallback, source SQL unchanged and owned cleanup");
}

fn select_healer(cs: &mut [Client; 2], i: usize, d: &Deadline, source: u32) -> SelectionContext {
    select_healer_hp(cs, i, d, source, 100)
}
pub(super) fn select_healer_hp(
    cs: &mut [Client; 2],
    i: usize,
    d: &Deadline,
    source: u32,
    max_hp: i64,
) -> SelectionContext {
    send(cs, i, d, || LoginCommand::SelectSource(source));
    wait(cs, d, "healing SQL roster", |cs| {
        cs[i].view().phase == LoginPhase::CharactersReady
    });
    let v = cs[i].view();
    assert_eq!(v.characters.len(), 1);
    assert_eq!(v.characters[0].object_id, 3333);
    let context = SelectionContext {
        key: CharacterKey {
            source_id: source,
            object_id: 3333,
        },
        epoch: v.roster_epoch,
    };
    send(cs, i, d, || LoginCommand::SelectCharacter(context));
    wait(cs, d, "healing SQL character", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::ChoosingChannel
            && !v.record_pending
            && !v.spell_pending
            && matches!(v.equipment, equipment::State::Ready(_))
    });
    let v = cs[i].view();
    let CharacterDataState::Ready(s) = &v.character_data else {
        panic!("source stats")
    };
    assert_eq!(s.context, context);
    assert_eq!((s.stats.max_hp, s.stats.max_mp), (max_hp, 300));
    assert_eq!(
        s.stats.intelligence,
        if source == 7 { 5_000_000_000 } else { 12 }
    );
    assert_eq!(s.stats.food225, if source == 7 { 225 } else { 180 });
    let SkillsState::Ready(skills) = &v.skills else {
        panic!("source skills")
    };
    assert_eq!(skills.context, context);
    assert!(skills.skills.iter().any(|s| s.skill_id == 1));
    context
}
pub(super) fn heal(cs: &mut [Client; 2], i: usize, target_i: usize, d: &Deadline) -> healing::Intent {
    heal_entity(cs, i, own_actor(&cs[target_i]).entity, d)
}
fn heal_entity(cs: &mut [Client; 2], i: usize, entity: i64, d: &Deadline) -> healing::Intent {
    use crate::modern_lito::{
        battle_skillbook as book, battle_window::Skill, compositor::Viewport, world_map::PixelPoint,
    };
    wait(cs, d, "healing painted input", |cs| cs[i].rendered());
    let view = cs[i].view();
    let surface = cs[i].surface.as_ref().unwrap();
    let target_actor = view
        .canonical
        .observation()
        .unwrap()
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == entity)
        .unwrap()
        .clone();
    let target = Target::Player {
        entity: target_actor.entity,
        life: target_actor.resources.as_ref().unwrap().life + 1,
    };
    let intended = Kind::Heal(target);
    let page = skill_page_intent(surface, &view, intended)
        .expect("player heal enabled on actual skill page");
    assert!(battle_input::skill_available(1, &view));
    assert!(battle_input::healing_ready(&view));
    let model = book::model(surface, &view, str::to_owned).unwrap();
    assert_eq!(
        book::select(model.choice(1).unwrap(), surface, &view),
        Some(book::Selection::Target(Skill::Heal))
    );
    assert!(
        battle_window::skill_status(&surface.observation, Skill::Heal)
            .unwrap()
            .contains("（角色）")
    );
    if entity == own_actor(&cs[i]).entity {
        assert!(
            !battle_window::target_current(&surface.observation, target),
            "self-heal must not enable self-attack"
        );
    }
    let bounds = surface
        .geometry
        .actors
        .iter()
        .find(|a| a.entity_id == target_actor.entity as u64)
        .unwrap();
    let viewport = Viewport {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    };
    let point = (bounds.top_left.y.max(0)..(bounds.top_left.y + bounds.height as i32).min(600))
        .flat_map(|y| {
            (bounds.top_left.x.max(0)..(bounds.top_left.x + bounds.width as i32).min(800))
                .map(move |x| PixelPoint { x, y })
        })
        .find(|p| {
            battle_window::hit(surface, viewport, *p, Skill::Heal).is_some_and(|h| h.intent == page)
        })
        .expect("actual visible sprite pixel produces exactly the selected healing target");
    let clicked = battle_window::hit(surface, viewport, point, Skill::Heal)
        .unwrap()
        .intent;
    let scaled = Viewport {
        x: 37,
        y: 53,
        width: 1600,
        height: 1200,
    };
    assert_eq!(
        battle_window::hit(
            surface,
            scaled,
            PixelPoint {
                x: 37 + point.x * 2 + 1,
                y: 53 + point.y * 2 + 1
            },
            Skill::Heal
        )
        .unwrap()
        .intent,
        clicked
    );
    for kind in [
        Kind::Heal(Target::Player {
            entity: target_actor.entity,
            life: 0,
        }),
        Kind::Heal(Target::Npc { serial: 1, life: 1 }),
        Kind::Heal(Target::Quest { slot: 0, born: 1 }),
    ] {
        assert!(battle_input::command(surface, Intent { kind, ..clicked }, &view).is_err());
    }
    let mut pending = view.clone();
    pending.world_pending = true;
    assert!(!battle_input::healing_ready(&pending));
    assert!(battle_input::command(surface, clicked, &pending).is_err());
    let command = battle_input::command(surface, clicked, &view).unwrap();
    let audio_before = cs.each_ref().map(|c| {
        c.audio
            .sequence(Stamp::of(c.view().canonical.observation().unwrap()), 109)
    });
    let q = healing::Intent::from_observation(&surface.observation, target_actor.entity).unwrap();
    let audience = cs.each_ref().map(|c| {
        let v = c.view();
        let o = v.canonical.observation().unwrap();
        (
            o.data().actors.0.iter().any(|a| a.entity == q.stamp.entity),
            o.data().healing.as_ref().unwrap().actors.clone(),
        )
    });
    let before = loop {
        for c in cs.iter_mut() {
            c.pump();
        }
        let before = cs[i].view().canonical.observation().unwrap().clone();
        if cs[i]
            .controller
            .command(LoginCommand::Canonical(command.clone()))
            .is_ok()
        {
            break before;
        }
        d.pause("healing local inbox admission");
    };
    assert_eq!(
        cs[i].view().canonical.observation().unwrap().data(),
        before.data(),
        "no optimistic HP/MP"
    );
    wait(cs, d, "healing SQL command receipt", |cs| {
        !cs[i].view().world_pending
    });
    eprintln!(
        "HEALING_SKILL_PAGE_HIT source={} sequence={} outcome={:?}",
        q.stamp.context.key.source_id,
        q.sequence,
        cs[i].view().canonical.outcome()
    );
    assert_eq!(
        cs[i].view().canonical.outcome(),
        Some(Outcome::PlayerHealing(healing::Reason::None))
    );
    let after = cs[i].view().canonical.observation().unwrap().clone();
    wait(
        cs,
        d,
        "eligible renderers emit confirmed healing cue",
        |cs| {
            cs.iter().enumerate().all(|(n, c)| {
                !audience[n].0
                    || c.audio
                        .sequence(Stamp::of(c.view().canonical.observation().unwrap()), 109)
                        > audio_before[n]
            })
        },
    );
    wait(cs, d, "all owners catch up after healing", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .revision
                    >= after.data().world.revision
        })
    });
    for (n, c) in cs.iter().enumerate().filter(|(n, _)| !audience[*n].0) {
        let v = c.view();
        let o = v.canonical.observation().unwrap();
        assert_eq!(
            c.audio.sequence(Stamp::of(o), 109),
            audio_before[n],
            "foreign lane has no healing cue"
        );
        assert_eq!(
            o.data().healing.as_ref().unwrap().actors,
            audience[n].1,
            "foreign lane healing accounts unchanged"
        );
    }
    crate::modern_lito::battle_scene::tests::verify_player_healing_shared(
        &explicit_path("LITO_SHARED_RESOURCE_ROOT"),
        before,
        after,
        q.stamp.entity,
    );
    q
}
fn hp(c: &Client) -> i64 {
    own_actor(c).resources.as_ref().unwrap().vitals.hp
}
#[test]
#[ignore = "ordinary extended profile and two real workers, production Main/owned SQL/shared assets, injury/healing/zero-cost/source return; not HWND"]
fn java_healing_main_sql_ordinary_controllers_cast_and_return_sources() {
    let d = Deadline(Instant::now() + Duration::from_secs(150));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernPlayerHealingSqlTestPeer",
        "healing-sql",
        Some("150000"),
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
        wait(&mut cs, &d, "healing ordinary extended TLS", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "healing authentication", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        contexts[i] = select_healer(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        enter(&mut cs, i, &d, contexts[i]);
        assert!(cs[i].view().canonical.player_healing_supported());
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
                        attributes: 0,
                        map_id: 514,
                        name: "healing Main SQL".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    wait(&mut cs, &d, "healing created room", |cs| {
        !cs[0].view().room_pending && cs[0].view().rooms.joined().is_some()
    });
    let summary = cs[0].view().rooms.joined().unwrap().summary.clone();
    send(&mut cs, 1, &d, || {
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
    wait(&mut cs, &d, "healing joined room", |cs| {
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
    wait(&mut cs, &d, "healing both real LOAD", |cs| {
        cs.iter().all(|c| {
            c.rendered()
                && c.view().canonical.observation().unwrap().data().world.phase
                    == data::Phase::Active
        })
    });
    let room = cs[0].view().rooms.joined().unwrap().clone();
    assert_ne!(
        room.member(contexts[0].key).unwrap().team,
        room.member(contexts[1].key).unwrap().team
    );
    assert_eq!((hp(&cs[0]), hp(&cs[1])), (100, 100));
    let victim = own_actor(&cs[1]);
    let target = Target::Player {
        entity: victim.entity,
        life: victim.resources.as_ref().unwrap().life + 1,
    };
    // Production RNG remains authoritative. Each fully acknowledged miss can
    // be followed by a new sequenced attack; never retry an uncertain command.
    for _ in 0..12 {
        if hp(&cs[1]) < 100 {
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
        wait(&mut cs, &d, "other SQL owner sees attack", |cs| {
            cs[1]
                .view()
                .canonical
                .observation()
                .is_some_and(|o| o.data().world.revision >= revision)
        });
    }
    let damaged = hp(&cs[1]);
    assert!(damaged > 0 && damaged < 100, "real melee injury required");
    let old = cs[1].view().canonical.observation().unwrap().clone();
    let paid = heal(&mut cs, 1, 1, &d);
    assert!(hp(&cs[1]) > damaged);
    let healed = cs[1].view().canonical.observation().unwrap().clone();
    let outgoing = &healed
        .data()
        .healing
        .as_ref()
        .unwrap()
        .actors
        .0
        .iter()
        .find(|a| a.entity == paid.stamp.entity)
        .unwrap()
        .outgoing
        .as_ref()
        .unwrap();
    assert_eq!(outgoing.ledger.spent, 4);
    assert_eq!(outgoing.ledger.casts, 1);
    let incoming = &healed
        .data()
        .healing
        .as_ref()
        .unwrap()
        .actors
        .0
        .iter()
        .find(|a| a.entity == paid.stamp.entity)
        .unwrap()
        .incoming;
    assert_eq!(incoming.received, 1);
    assert!(incoming.restored > 0);
    assert_eq!(incoming.restored, hp(&cs[1]) - damaged);
    assert_eq!(
        old.data()
            .actors
            .0
            .iter()
            .find(|a| a.entity == paid.stamp.entity)
            .unwrap()
            .resources
            .as_ref()
            .unwrap()
            .vitals
            .hp,
        damaged
    );
    assert!(cs[1]
        .controller
        .command(LoginCommand::Canonical(Command::player_healing(paid)))
        .is_err());
    // The zero-cost caster just attacked. Wait on confirmed server elapsed
    // time for its actual attack pace before beginning a different action.
    let attacker = own_actor(&cs[0]);
    let recovery = crate::modern_lito::battle_motion::paced_nanos(
        &attacker,
        attacker.vital.as_ref().unwrap().attack_interval_nanos as u64,
        false,
    ) as i64;
    let ready_at = cs[0]
        .view()
        .canonical
        .observation()
        .unwrap()
        .data()
        .world
        .loading_elapsed_nanos
        + recovery;
    wait(
        &mut cs,
        &d,
        "healing after confirmed attack recovery",
        |cs| {
            cs[0].rendered()
                && cs[0]
                    .view()
                    .canonical
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .loading_elapsed_nanos
                    >= ready_at
        },
    );
    // Ordinary PVP teams share one instance. Explicitly selecting the other
    // source is legal healing, not an implicit self-target or tower fallback.
    let zero = heal(&mut cs, 0, 1, &d);
    assert_eq!(zero.target.identity.source_id, 42);
    assert_ne!(zero.target.entity, zero.stamp.entity);
    let v = cs[0].view();
    let outgoing = v
        .canonical
        .observation()
        .unwrap()
        .data()
        .healing
        .as_ref()
        .unwrap()
        .actors
        .0
        .iter()
        .find(|a| a.entity == zero.stamp.entity)
        .unwrap()
        .outgoing
        .as_ref()
        .unwrap();
    assert_eq!(
        (
            outgoing.ledger.cost,
            outgoing.ledger.spent,
            outgoing.ledger.casts
        ),
        (0, 0, 1)
    );
    let healed_hp = hp(&cs[1]);
    wait(&mut cs, &d, "observer and renderer see healing", |cs| {
        cs.iter().all(Client::rendered)
            && cs[0]
                .view()
                .canonical
                .observation()
                .unwrap()
                .data()
                .actors
                .0
                .iter()
                .find(|a| a.entity == paid.stamp.entity)
                .unwrap()
                .resources
                .as_ref()
                .unwrap()
                .vitals
                .hp
                >= healed_hp
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
        wait(&mut cs, &d, "healing leave", |cs| {
            !cs[i].view().world_pending
                && !cs[i].view().canonical.attached()
                && cs[i].runtime.is_quiescent()
        });
    }
    for i in (0..2).rev() {
        room_action(&mut cs, i, &d, contexts[i], rooms::Action::Leave, 0);
        send(&mut cs, i, &d, || {
            LoginCommand::ReturnCharacters(contexts[i])
        });
        wait(&mut cs, &d, "healing character return", |cs| {
            cs[i].view().phase == LoginPhase::CharactersReady
        });
    }
    for i in 0..2 {
        let context = select_healer(&mut cs, i, &d, if i == 0 { 42 } else { 7 });
        assert_ne!(context, contexts[i]);
        assert!(cs[i]
            .controller
            .command(LoginCommand::Canonical(Command::player_healing(
                if i == 0 { zero } else { paid }
            )))
            .is_err());
    }
    for c in &mut cs {
        c.runtime.shutdown();
        c.controller.command(LoginCommand::Shutdown).unwrap();
    }
    wait(&mut cs, &d, "healing workers retired", |cs| {
        cs.iter()
            .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
    });
    peer.expect_successful_exit(&d);
    eprintln!("HEALING_MAIN_SQL: ordinary explicit profile, two actual SessionController workers, production Main/JDBC/RNG, real shared LOAD, skill-page selection and actual visible-pixel hit at 1x/2x offset viewport, self paid heal and cross-source INT5e9 zero-cost heal, observer/render catchup, shared744/action19/audio109 on both renderers, stale/pending/unsupported-target rejection and source swap; imported sources unchanged, no native HWND/mouse or audio-device claim");
}
