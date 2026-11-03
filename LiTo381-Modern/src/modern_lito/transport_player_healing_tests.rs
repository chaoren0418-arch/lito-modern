// The real Rust TLS owner and state tracker, not stdout-supplied battle data.
// Source rows are synthetic; shared Lineage map/SPR/Tile data are read-only.
mod player_healing_runtime_interop {
    use super::*;
    use crate::modern_lito::{
        battle_payload as payload, battle_read,
        battle_runtime::{BattleRuntime, Status},
        battle_world,
        protocol::{extensions as e, KNOWN_CAPABILITIES},
    };

    fn pause(deadline: Instant, stage: &str) {
        assert!(Instant::now() < deadline, "healing TLS deadline: {stage}");
        thread::sleep(Duration::from_millis(10));
    }
    include!("transport_player_healing_cast_tests.rs");

    #[test]
    #[ignore = "actual ModernServer TLS/auth/room/BOT/shared renderer/LOAD/leave; explicit synthetic source, no SQL/UI claim"]
    fn java_healing_runtime_selected_tls_preserves_full_state_through_shared_load_and_leave() {
        let shared = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
        assert!(shared.is_absolute());
        for (mode, extensions) in [("data", 1), ("commands", 3)] {
            let mut peer = JavaPeer::start_class(
                "local.lito381.server.ModernPlayerHealingRuntimeTestPeer",
                mode,
                "60000",
            );
            let deadline = Instant::now() + Duration::from_secs(50);
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
                e::Offer {
                    supported: extensions,
                    required: extensions,
                },
                integration_limits(),
                Cancellation::new(),
            )
            .unwrap();
            let selected = c.negotiated_selection().unwrap();
            assert!(selected.allows_data());
            assert_eq!(selected.allows_commands(), extensions == 3);
            c.authenticate(fixture_credentials("fixture-password"))
                .unwrap();
            c.characters(7, 1).unwrap();
            let context = SelectionContext {
                key: application::CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: 1,
            };
            c.select_character(context).unwrap();
            assert_eq!(
                c.enter_channel(0).unwrap().snapshot.phase,
                NavigationPhase::Lobby
            );
            let created = c
                .rooms(
                    rooms::Request::draft(
                        rooms::Common {
                            action: rooms::Action::Create,
                            ..rooms::Common::read(context, 0)
                        },
                        rooms::Draft::new(
                            rooms::PublicDraft {
                                capacity: 2,
                                mode: 1,
                                attributes: 16,
                                map_id: 514,
                                name: "shared healing TLS".into(),
                            },
                            String::new(),
                        )
                        .unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap();
            assert_eq!(created.status, rooms::Status::Ok);
            let room = created.joined.unwrap();
            assert!(room.member(context.key).is_some(), "creator joins own room");
            let bots = c
                .gm_bots(gm::Request {
                    context,
                    channel_id: 0,
                    room_id: room.summary.room_id,
                    expected_revision: room.summary.revision,
                    count: 20,
                })
                .unwrap();
            assert_eq!(bots.added, 1, "capacity clamps requested bots");
            let room = bots.room.unwrap().joined.unwrap();
            let bot = room
                .members
                .iter()
                .find(|m| m.kind == rooms::MemberKind::ServerBot)
                .unwrap();
            assert!(bot.ready);
            assert_eq!(bot.source_name, "太陽神");
            assert_ne!(bot.team, room.member(context.key).unwrap().team);
            let ready = c
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
            assert!(ready.summary.satisfied);
            let prepare = battle_world::Request {
                action: battle_world::Action::Prepare,
                room: ready.summary.room_id as i64,
                room_revision: ready.summary.revision,
                ..battle_world::Request::read(context, 0, None)
            };
            let prepared = c.battle_world(prepare.clone()).unwrap();
            let key = prepared.attachment.unwrap().world;
            let read = battle_world::Request::read(context, 0, Some(&key));
            loop {
                if c.battle_world(read.clone()).unwrap().data.is_some() {
                    break;
                }
                pause(deadline, "world preparation");
            }
            let first = c.battle_snapshot().unwrap().observation().unwrap().clone();
            assert!(first.data().healing.is_some());
            assert_eq!(first.data().world.phase, payload::data::Phase::Loading);
            assert_eq!(
                payload::encode(first.data()).unwrap()[4..6],
                7u16.to_be_bytes()
            );
            assert!(c.world_snapshot().is_none() && c.combat_snapshot().is_none());
            // Neither an altered source nor repeated PREPARE may replace the
            // complete retained state or send an inadmissible lifecycle request.
            let retained = c.battle_snapshot().unwrap().clone();
            let mut wrong = read.clone();
            wrong.context.key.source_id = 42;
            for request in [wrong, prepare] {
                assert!(matches!(
                    c.battle_world(request),
                    Err(TransportError::InvalidState(_))
                ));
                assert!(Arc::ptr_eq(&retained, c.battle_snapshot().unwrap()));
            }
            c.ping().unwrap();
            let mut runtime = BattleRuntime::new(shared.clone()).unwrap();
            runtime.attach(first.clone()).unwrap();
            let mut keepalive = Instant::now();
            let surface = loop {
                match runtime.poll() {
                    Status::Ready(surface) => {
                        assert!(runtime.prepared(&surface));
                        break surface;
                    }
                    Status::Failed(reason) => panic!("shared healing preflight: {reason}"),
                    _ => {}
                }
                if keepalive.elapsed() >= Duration::from_secs(1) {
                    c.ping().unwrap();
                    keepalive = Instant::now();
                }
                pause(deadline, "shared map/sprite/tile rendering");
            };
            c.battle_world(read.clone()).unwrap();
            let loading = c.battle_snapshot().unwrap().observation().unwrap().clone();
            assert_eq!(
                loading.data().world.phase,
                payload::data::Phase::Loading,
                "resource preflight must complete within the server load deadline"
            );
            let loaded = runtime
                .loaded_request(&surface, &loading)
                .expect("actual resource LOAD proof");
            assert_eq!(
                c.battle_world(loaded).unwrap().state,
                battle_world::State::Data
            );
            loop {
                c.battle_world(read.clone()).unwrap();
                if c.battle_snapshot()
                    .unwrap()
                    .observation()
                    .unwrap()
                    .data()
                    .world
                    .phase
                    == payload::data::Phase::Active
                {
                    break;
                }
                pause(deadline, "all players loaded/countdown");
            }
            let active = c.battle_snapshot().unwrap().observation().unwrap().clone();
            assert!(active.data().actors.0.iter().all(|a| a.loaded));
            assert!(active.data().healing.is_some());
            if extensions == 1 {
                use crate::modern_lito::{
                    battle_control::{Command, Outcome, State},
                    battle_player_healing::Intent,
                };
                let q = Intent::from_observation(&active, active.data().personal.entity).unwrap();
                let retained = c.battle_snapshot().unwrap().clone();
                assert!(!c.supports_player_healing());
                assert!(c.player_healing(q).is_err());
                let mut control = State::default();
                control
                    .execute(&mut c, Command::player_healing(q), 1)
                    .unwrap();
                assert_eq!(control.outcome(), Some(Outcome::NotSent));
                assert!(Arc::ptr_eq(&retained, c.battle_snapshot().unwrap()));
                c.ping().unwrap();
            }
            runtime.update(active.clone()).unwrap();
            assert!(runtime.loaded_request(&surface, &active).is_none());
            let surface = loop {
                match runtime.poll() {
                    Status::Ready(s) if s.observation.data() == active.data() => break s,
                    Status::Failed(reason) => panic!("active shared scene: {reason}"),
                    _ => {}
                }
                pause(deadline, "active shared pixels");
            };
            use crate::modern_lito::world_map::{WorldCell, DIRECTIONS};
            let actor = active
                .data()
                .actors
                .0
                .iter()
                .find(|a| a.entity == active.data().personal.entity)
                .unwrap();
            let from = WorldCell {
                x: actor.position.x,
                y: actor.position.y,
            };
            let (heading, target) = DIRECTIONS
                .iter()
                .enumerate()
                .map(|(i, &(dx, dy))| {
                    (
                        i as u8,
                        WorldCell {
                            x: from.x + dx,
                            y: from.y + dy,
                        },
                    )
                })
                .find(|(_, to)| {
                    surface
                        .navigation
                        .as_ref()
                        .unwrap()
                        .map()
                        .can_step(from, *to)
                        && active
                            .data()
                            .actors
                            .0
                            .iter()
                            .all(|a| a.position.x != to.x || a.position.y != to.y)
                })
                .expect("walkable unoccupied shared-map neighbor");
            let command = world::Request {
                action: world::Action::Move,
                heading,
                sequence: 1,
                from_x: from.x,
                from_y: from.y,
                match_id: key.match_id as u64,
                generation: key.generation as u64,
                ..world::Request::read(context, 0)
            };
            let moved = c
                .battle_operation(
                    crate::modern_lito::battle_action::Guard {
                        stamp: crate::modern_lito::battle_window::Stamp::of(&active),
                        target: None,
                    },
                    |id, _| world::encode_request(id, &command),
                )
                .unwrap();
            assert_eq!(
                moved.disposition,
                crate::modern_lito::battle_operation::Disposition::Applied
            );
            assert!(moved.data.healing.is_some());
            let moved_observation = c.battle_snapshot().unwrap().observation().unwrap().clone();
            assert_eq!(moved_observation.data(), &moved.data);
            runtime.update(moved_observation.clone()).unwrap();
            loop {
                match runtime.poll() {
                    Status::Ready(s) if s.observation.data() == moved_observation.data() => {
                        assert_eq!(
                            s.geometry
                                .actors
                                .iter()
                                .find(|a| a.entity_id == actor.entity as u64)
                                .unwrap()
                                .cell,
                            target
                        );
                        break;
                    }
                    Status::Failed(reason) => panic!("moved shared scene: {reason}"),
                    _ => {}
                }
                pause(deadline, "confirmed move pixels");
            }
            // Explicit lower-level READ probe additionally tests the real TLS
            // Application reader. v39+ public read_battle remains closed and the
            // lifecycle owner continues to commit through battle_world only.
            let q = battle_read::Request {
                context,
                channel_id: 0,
                world: key.clone(),
            };
            assert!(c.read_battle(q.clone()).is_err());
            let id = c.next_request_id().unwrap();
            let frame = battle_read::encode_request(id, &q).unwrap();
            let Response::BattleData(reply) = c.exchange(&frame, id, Stage::BattleData).unwrap()
            else {
                panic!("canonical READ response");
            };
            assert_eq!(reply.echo, q);
            let mut follower =
                crate::modern_lito::battle_follow::Follower::new_selected(q, selected).unwrap();
            follower.accept(reply).unwrap();
            moved_observation
                .check_successor(follower.observation().unwrap())
                .unwrap();
            assert!(follower.observation().unwrap().data().healing.is_some());
            let left = c
                .battle_world(battle_world::Request {
                    action: battle_world::Action::Leave,
                    ..read
                })
                .unwrap();
            assert_eq!(left.state, battle_world::State::Detached);
            assert!(c.battle_snapshot().unwrap().observation().is_none());
            assert!(c.battle_snapshot().unwrap().attachment().is_none());
            assert_eq!(retained.observation().unwrap().data(), first.data());
            c.ping().unwrap();
            runtime.shutdown();
            while !runtime.is_quiescent() {
                runtime.poll();
                pause(deadline, "renderer retirement");
            }
            c.close().unwrap();
            assert!(c.negotiated_selection().is_none());
            assert_eq!(
                peer.read_bounded_line("CLOSED").trim(),
                format!("CLOSED {mode}")
            );
            peer.expect_successful_exit();
        }
        eprintln!("HEALING_RUNTIME_TLS: DATA and COMMANDS-selected owners; real authentication, room creator, capacity-clamped named ready BOT, full LTB7 state, shared renderer and verified LOAD, active world, canonical move and confirmed pixels, TLS READ, atomic source rejection, leave and cleanup; no healing cast, SQL or native UI claim");
    }
}
