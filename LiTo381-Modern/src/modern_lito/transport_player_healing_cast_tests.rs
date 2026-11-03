// Two real authenticated Rust owners. No server reflection supplies HP, MP,
// command proofs or LOAD acknowledgements. Synthetic source rows, real assets.
fn healing_login(peer: &JavaPeer, source: u32) -> (ModernTlsConnection, SelectionContext) {
    let mut c = connect_and_negotiate_extended(
        peer.endpoint,
        "localhost",
        Some(&private_root()),
        Hello {
            request_id: 1,
            min_version: 47,
            max_version: 47,
            supported_capabilities: KNOWN_CAPABILITIES,
            required_capabilities: e::DEPENDENCIES,
            numeric_schema: 1,
        },
        e::HEALING,
        integration_limits(),
        Cancellation::new(),
    )
    .unwrap();
    c.authenticate(fixture_credentials("fixture-password"))
        .unwrap();
    c.characters(source, 1).unwrap();
    let context = SelectionContext {
        key: application::CharacterKey {
            source_id: source,
            object_id: 3333,
        },
        epoch: 1,
    };
    c.select_character(context).unwrap();
    c.enter_channel(0).unwrap();
    (c, context)
}
fn healing_observation(c: &ModernTlsConnection) -> crate::modern_lito::battle_follow::Observation {
    c.battle_snapshot().unwrap().observation().unwrap().clone()
}
fn healing_vitals(o: &crate::modern_lito::battle_follow::Observation) -> &payload::data::Vitals {
    &o.data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .unwrap()
        .resources
        .as_ref()
        .unwrap()
        .vitals
}
#[test]
#[ignore = "two real ModernServer TLS owners, shared preflight, melee damage and healing/control receipt; synthetic sources, not SQL/HWND"]
fn java_healing_runtime_two_owner_attack_heal_pending_and_leave() {
    use crate::modern_lito::{
        battle_action::Guard,
        battle_control::{Command as GameCommand, Outcome, State as Control},
        battle_operation::Disposition,
        battle_player_healing as heal,
        battle_window::{Stamp, Target},
    };
    let shared = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernPlayerHealingRuntimeTestPeer",
        "two",
        "60000",
    );
    let deadline = Instant::now() + Duration::from_secs(50);
    let (mut a, sun) = healing_login(&peer, 7);
    let (mut b, moon) = healing_login(&peer, 42);
    let created = a
        .rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(sun, 0)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "healing two owners".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let first = created.joined.unwrap();
    let mut room = b
        .rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: first.summary.room_id,
                    expected_revision: first.summary.revision,
                    ..rooms::Common::read(moon, 0)
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap()
        .joined
        .unwrap();
    assert_ne!(
        room.member(sun.key).unwrap().team,
        room.member(moon.key).unwrap().team
    );
    for (c, context) in [(&mut a, sun), (&mut b, moon)] {
        room = c
            .rooms(
                rooms::Request::simple(rooms::Common {
                    context,
                    channel_id: 0,
                    action: rooms::Action::Ready,
                    value: 1,
                    room_id: room.summary.room_id,
                    expected_revision: room.summary.revision,
                })
                .unwrap(),
            )
            .unwrap()
            .joined
            .unwrap();
    }
    assert!(room.summary.satisfied);
    let prepared = a
        .battle_world(battle_world::Request {
            action: battle_world::Action::Prepare,
            room: room.summary.room_id as i64,
            room_revision: room.summary.revision,
            ..battle_world::Request::read(sun, 0, None)
        })
        .unwrap();
    let key = prepared.attachment.unwrap().world;
    let read_a = battle_world::Request::read(sun, 0, Some(&key));
    let read_b = battle_world::Request::read(moon, 0, Some(&key));
    loop {
        if a.battle_world(read_a.clone()).unwrap().data.is_some() {
            break;
        }
        pause(deadline, "two-owner preparation");
    }
    b.battle_world(read_b.clone()).unwrap();
    let first_a = healing_observation(&a);
    let first_b = healing_observation(&b);
    let mut render_a = BattleRuntime::new(shared.clone()).unwrap();
    let mut render_b = BattleRuntime::new(shared).unwrap();
    render_a.attach(first_a.clone()).unwrap();
    render_b.attach(first_b.clone()).unwrap();
    let mut surfaces = [None, None];
    let mut keepalive = Instant::now();
    while surfaces.iter().any(Option::is_none) {
        for (i, runtime) in [&mut render_a, &mut render_b].into_iter().enumerate() {
            match runtime.poll() {
                Status::Ready(s) => {
                    assert!(runtime.prepared(&s));
                    surfaces[i] = Some(s);
                }
                Status::Failed(reason) => panic!("two-owner shared preload: {reason}"),
                _ => {}
            }
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            a.ping().unwrap();
            b.ping().unwrap();
            keepalive = Instant::now();
        }
        pause(deadline, "both shared resource proofs");
    }
    let load_a = render_a
        .loaded_request(surfaces[0].as_ref().unwrap(), &first_a)
        .unwrap();
    let load_b = render_b
        .loaded_request(surfaces[1].as_ref().unwrap(), &first_b)
        .unwrap();
    a.battle_world(load_a).unwrap();
    b.battle_world(load_b).unwrap();
    a.battle_world(read_a.clone()).unwrap();
    b.battle_world(read_b.clone()).unwrap();
    let active_a = healing_observation(&a);
    let active_b = healing_observation(&b);
    assert_eq!(active_a.data().world.phase, payload::data::Phase::Active);
    assert_eq!(active_b.data().world.phase, payload::data::Phase::Active);
    let target_entity = active_a.data().personal.entity;
    let victim = active_b
        .data()
        .actors
        .0
        .iter()
        .find(|p| p.entity == target_entity)
        .unwrap();
    let attack = combat::Request {
        context: moon,
        channel_id: 0,
        action: combat::Action::Attack,
        match_id: key.match_id as u64,
        generation: key.generation as u64,
        sequence: 1,
        target_entity_id: target_entity as u64,
    };
    let damage = b
        .battle_operation(
            Guard {
                stamp: Stamp::of(&active_b),
                target: Some(Target::Player {
                    entity: target_entity,
                    life: victim.resources.as_ref().unwrap().life + 1,
                }),
            },
            |id, _| combat::encode_request(id, &attack),
        )
        .unwrap();
    assert_eq!(
        damage.disposition,
        Disposition::Applied,
        "real melee must injure before heal"
    );
    a.battle_world(read_a.clone()).unwrap();
    let hurt = healing_observation(&a);
    assert!(healing_vitals(&hurt).hp < healing_vitals(&active_a).hp);
    assert!(healing_vitals(&hurt).hp > 0);
    let q = heal::Intent::from_observation(&hurt, target_entity).unwrap();
    let retained = a.battle_snapshot().unwrap().clone();
    for mutate in [
        (|q: &mut heal::Intent| q.stamp.context.key.source_id = 42) as fn(&mut heal::Intent),
        |q| q.stamp.context.epoch += 1,
        |q| q.stamp.life += 1,
        |q| q.equipment_revision += 1,
        |q| q.x += 1,
        |q| q.sequence += 1,
        |q| q.target.life += 1,
    ] {
        let mut stale = q;
        mutate(&mut stale);
        assert!(a.player_healing(stale).is_err());
        assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
    }
    a.ping().unwrap();
    let mut control = Control::default();
    control.align(&a);
    assert!(control.player_healing_supported());
    let command = GameCommand::player_healing(q);
    control.execute(&mut a, command.clone(), 1).unwrap();
    assert!(control.confirmed(&command, 1));
    assert_eq!(
        control.outcome(),
        Some(Outcome::PlayerHealing(heal::Reason::None))
    );
    let healed = healing_observation(&a);
    assert!(
        healing_vitals(&healed).hp > healing_vitals(&hurt).hp,
        "server receipt restored actual missing HP"
    );
    assert_eq!(healing_vitals(&healed).mp, healing_vitals(&hurt).mp - 4);
    assert_eq!(retained.observation().unwrap().data(), hurt.data());
    assert_eq!(control.observation().unwrap().data(), healed.data());
    let after = a.battle_snapshot().unwrap().clone();
    control.execute(&mut a, command.clone(), 2).unwrap();
    assert_eq!(
        control.outcome(),
        Some(Outcome::NotSent),
        "stale queued sequence must not resend"
    );
    assert!(Arc::ptr_eq(&after, a.battle_snapshot().unwrap()));
    // A current command may be denied by the server; it still consumes its
    // sequence exactly once and must not fabricate healing or extra MP cost.
    let again = heal::Intent::from_observation(&healed, target_entity).unwrap();
    control
        .execute(&mut a, GameCommand::player_healing(again), 3)
        .unwrap();
    assert_eq!(
        control.outcome(),
        Some(Outcome::PlayerHealing(heal::Reason::TooFast))
    );
    let denied = healing_observation(&a);
    assert_eq!(healing_vitals(&denied), healing_vitals(&healed));
    assert_eq!(denied.data().healing, healed.data().healing);
    render_a.update(denied.clone()).unwrap();
    loop {
        match render_a.poll() {
            Status::Ready(s) if s.observation.data() == denied.data() => {
                assert_eq!(healing_vitals(&s.observation), healing_vitals(&healed));
                break;
            }
            Status::Failed(reason) => panic!("healed complete scene: {reason}"),
            _ => {}
        }
        pause(deadline, "healed data reaches shared renderer");
    }
    b.battle_world(read_b.clone()).unwrap();
    let seen = healing_observation(&b);
    let seen_victim = seen
        .data()
        .actors
        .0
        .iter()
        .find(|p| p.entity == target_entity)
        .unwrap();
    assert_eq!(
        seen_victim.resources.as_ref().unwrap().vitals.hp,
        healing_vitals(&healed).hp
    );
    assert!(
        seen.data()
            .healing
            .as_ref()
            .unwrap()
            .actors
            .0
            .iter()
            .find(|p| p.entity == target_entity)
            .unwrap()
            .incoming
            .received
            > 0
    );
    assert!(a.world_snapshot().is_none() && a.combat_snapshot().is_none());
    assert!(b.world_snapshot().is_none() && b.combat_snapshot().is_none());
    // An observer catches the authoritative healed data, not private owner history.
    for (c, read) in [(&mut b, read_b), (&mut a, read_a)] {
        assert_eq!(
            c.battle_world(battle_world::Request {
                action: battle_world::Action::Leave,
                ..read
            })
            .unwrap()
            .state,
            battle_world::State::Detached
        );
        c.ping().unwrap();
        c.close().unwrap();
    }
    for runtime in [&mut render_a, &mut render_b] {
        runtime.shutdown();
        while !runtime.is_quiescent() {
            runtime.poll();
            pause(deadline, "two renderers retire");
        }
    }
    assert_eq!(peer.read_bounded_line("CLOSED").trim(), "CLOSED two");
    peer.expect_successful_exit();
    eprintln!("HEALING_LIVE_CAST: two scoped Rust TLS owners, actual shared preflight, real melee damage then HP restore/MP cost through controller command and kind78/79; stale local intents/replayed sequence not sent, server cooldown no extra cost, observer full data and normal cleanup; synthetic sources, no SQL or healing-FX/UI claim");
}
