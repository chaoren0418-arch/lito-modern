// Actual production typed TLS methods and atomic service tracker. This is not
// the SessionController/UI or shared-resource renderer. LOAD uses wire digest.
mod vip_world_tls_interop {
    use super::*;
    use crate::modern_lito::{
        battle_follow::Follower,
        battle_payload as payload, battle_read as read, battle_vip_equipment as gear,
        battle_vip_world as vip, battle_world as world,
        protocol::{extensions as e, KNOWN_CAPABILITIES},
    };
    fn login(peer: &JavaPeer, source: u32) -> (ModernTlsConnection, SelectionContext) {
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
                supported: 112,
                required: 112,
            },
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
    fn world(c: &mut ModernTlsConnection, q: &world::Request) -> vip::Reply {
        let r = c.vip_world(q.clone()).unwrap();
        assert_eq!(r.echo(), q);
        let s = c.battle_snapshot().unwrap();
        assert!(s.full_vip());
        assert_eq!(s.observation().map(|o| o.data()), r.data());
        assert!(
            s.presentation_observation().is_none(),
            "VIP wire is not shared presentation proof"
        );
        r
    }
    fn follow(f: &mut Follower, q: &read::Request, r: &vip::Reply) {
        assert!(r.data().is_some());
        f.accept(read::UnvalidatedReply {
            echo: q.clone(),
            state: read::State::Data,
            data: r.data().cloned(),
        })
        .unwrap();
    }
    fn equipment(
        c: &mut ModernTlsConnection,
        q: &gear::Request,
        f: &mut Follower,
    ) -> (gear::Reply, Option<gear::Quote>) {
        let (r, quote) = c.vip_equipment(*q).unwrap();
        r.accept(q, f).unwrap();
        assert_eq!(
            c.battle_snapshot().unwrap().observation().unwrap().data(),
            f.observation().unwrap().data()
        );
        for case in 0..7 {
            let mut tracker = c.battle_state.clone();
            let before = tracker.snapshot().unwrap().clone();
            let mut bad = r.clone();
            match case {
                0 => bad.echo.stamp.context.key.source_id ^= 1,
                1 => bad.echo.stamp.context.epoch += 1,
                2 => bad.lifecycle.revision = 0,
                3 => bad.data.world.revision = 0,
                4 => bad.data.vip.as_mut().unwrap().owner.versions.saved += 1,
                5 => bad.data.personal.entity += 1,
                _ => bad.durable_revision = -2,
            }
            assert!(tracker
                .accept_vip_equipment(&bad, q, c.negotiated_selection().unwrap())
                .is_err());
            assert!(
                Arc::ptr_eq(&before, tracker.snapshot().unwrap()),
                "receipt mutation {case}"
            );
        }
        (r, quote)
    }
    fn ready(c: &mut ModernTlsConnection, context: SelectionContext) -> rooms::RoomSnapshot {
        let room = c
            .rooms(rooms::Request::simple(rooms::Common::read(context, 0)).unwrap())
            .unwrap()
            .joined
            .unwrap();
        c.rooms(
            rooms::Request::simple(rooms::Common {
                action: rooms::Action::Ready,
                value: 1,
                room_id: room.summary.room_id,
                expected_revision: room.summary.revision,
                ..rooms::Common::read(context, 0)
            })
            .unwrap(),
        )
        .unwrap()
        .joined
        .unwrap()
    }
    include!("transport_vip_fault_tests.rs");
    include!("transport_vip_controller_tests.rs");
    #[test]
    #[ignore = "actual production Rust TLS methods/atomic state and Java auth/two-source VIP lifecycle/file CAS; no SQL/Main/UI/render proof"]
    fn java_vip_world_two_sources_full_data_gear_leave_and_reload() {
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernVipWorldRuntimeTestPeer",
            "two",
            "60000",
        );
        let (mut a, sun) = login(&peer, 7);
        let (mut b, moon) = login(&peer, 42);
        let initial = a
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
                            name: "VIP TLS".into(),
                        },
                        String::new(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
            .unwrap()
            .joined
            .unwrap();
        b.rooms(
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: initial.summary.room_id,
                    expected_revision: initial.summary.revision,
                    ..rooms::Common::read(moon, 0)
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut previous = 0;
        for saved in [0, 1] {
            ready(&mut a, sun);
            let room = ready(&mut b, moon);
            assert!(room.summary.satisfied);
            let prepare = world::Request {
                action: world::Action::Prepare,
                room: room.summary.room_id as i64,
                room_revision: room.summary.revision,
                ..world::Request::read(sun, 0, None)
            };
            let r = world(&mut a, &prepare);
            assert_eq!(r.state(), world::State::BeforeEngine);
            let key = r.attachment().unwrap().world.clone();
            assert!(key.match_id > previous);
            previous = key.match_id;
            let read_a = world::Request::read(sun, 0, Some(&key));
            let read_b = world::Request::read(moon, 0, Some(&key));
            let retained = a.battle_snapshot().unwrap().clone();
            let id = a.last_request_id;
            assert!(
                a.battle_world(read_a.clone()).is_err(),
                "ordinary endpoint must refuse VIP even during preparation"
            );
            assert!(
                a.vip_world(prepare.clone()).is_err(),
                "no implicit replacement"
            );
            let mut wrong = read_a.clone();
            wrong.context = moon;
            assert!(a.vip_world(wrong).is_err());
            assert_eq!(a.last_request_id, id);
            assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
            a.ping().unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            let loaded = loop {
                let r = world(&mut a, &read_a);
                if r.data().is_some() {
                    break r;
                }
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(5));
            };
            let rq = read::Request {
                context: sun,
                channel_id: 0,
                world: key.clone(),
            };
            let mut follower =
                Follower::new_selected(rq.clone(), a.negotiated_selection().unwrap()).unwrap();
            follow(&mut follower, &rq, &loaded);
            assert_eq!(
                loaded
                    .data()
                    .unwrap()
                    .vip
                    .as_ref()
                    .unwrap()
                    .owner
                    .versions
                    .saved,
                saved
            );
            assert_eq!(
                loaded.data().unwrap().world.phase,
                payload::data::Phase::Loading
            );
            let digest = loaded.data().unwrap().manifest.digest;
            world(&mut b, &read_b);
            let la = world::Request {
                action: world::Action::Loaded,
                digest,
                ..read_a.clone()
            };
            let lb = world::Request {
                action: world::Action::Loaded,
                digest,
                ..read_b.clone()
            };
            follow(&mut follower, &rq, &world(&mut a, &la));
            assert_eq!(
                world(&mut b, &lb).data().unwrap().world.phase,
                payload::data::Phase::Active
            );
            follow(&mut follower, &rq, &world(&mut a, &read_a));
            let o = follower.observation().unwrap();
            assert!(!o.legacy_commands_supported());
            if saved == 0 {
                let own = o
                    .data()
                    .actors
                    .0
                    .iter()
                    .find(|x| x.entity == o.data().personal.entity)
                    .unwrap();
                assert!(own.vital.as_ref().unwrap().values.hp > i32::MAX as i64);
                let q = gear::Request::inspect(o).unwrap();
                let stale_inspect = q;
                let (r, quote) = equipment(&mut a, &q, &mut follower);
                assert_eq!(r.outcome, gear::Outcome::Inspected);
                let q = quote
                    .unwrap()
                    .set(follower.observation().unwrap(), 1, false)
                    .unwrap();
                let (r, _) = equipment(&mut a, &q, &mut follower);
                assert_eq!(r.outcome, gear::Outcome::Applied);
                assert_eq!(r.data.vip.as_ref().unwrap().owner.versions.saved, 1);
                let (rejected, _) = equipment(&mut a, &stale_inspect, &mut follower);
                assert_eq!(rejected.rejection, gear::Rejection::StaleSavedEquipment);
                assert_eq!(rejected.outcome, gear::Outcome::Rejected);
            }
            follow(&mut follower, &rq, &world(&mut a, &read_a));
            assert_eq!(
                follower
                    .observation()
                    .unwrap()
                    .data()
                    .vip
                    .as_ref()
                    .unwrap()
                    .owner
                    .versions
                    .saved,
                1
            );
            assert_eq!(
                world(&mut b, &read_b)
                    .data()
                    .unwrap()
                    .vip
                    .as_ref()
                    .unwrap()
                    .owner
                    .versions
                    .saved,
                0
            );
            assert_eq!(
                world(
                    &mut a,
                    &world::Request {
                        action: world::Action::Leave,
                        ..read_a.clone()
                    }
                )
                .state(),
                world::State::Detached
            );
            assert_eq!(
                world(&mut b, &read_b).data().unwrap().world.phase,
                payload::data::Phase::Aborted
            );
            assert_eq!(
                world(
                    &mut b,
                    &world::Request {
                        action: world::Action::Leave,
                        ..read_b
                    }
                )
                .state(),
                world::State::Detached
            );
            assert_eq!(
                world(&mut a, &world::Request::read(sun, 0, None)).state(),
                world::State::Detached
            );
            assert!(
                a.battle_snapshot()
                    .is_some_and(|s| s.attachment().is_none() && s.observation().is_none())
                    && a.world_snapshot().is_none()
                    && a.combat_snapshot().is_none(),
                "leave retains only confirmed detachment, never legacy frames"
            );
            let retained = a.battle_snapshot().unwrap().clone();
            let id = a.last_request_id;
            assert!(a.vip_world(read_a).is_err());
            assert_eq!(a.last_request_id, id);
            assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
        }
        for (c, context) in [(&mut b, moon), (&mut a, sun)] {
            let room = c
                .rooms(rooms::Request::simple(rooms::Common::read(context, 0)).unwrap())
                .unwrap()
                .joined
                .unwrap();
            let left = c
                .rooms(
                    rooms::Request::simple(rooms::Common {
                        action: rooms::Action::Leave,
                        room_id: room.summary.room_id,
                        expected_revision: room.summary.revision,
                        ..rooms::Common::read(context, 0)
                    })
                    .unwrap(),
                )
                .unwrap();
            assert_eq!(left.status, rooms::Status::Ok);
            assert!(left.joined.is_none());
        }
        for c in [&mut b, &mut a] {
            assert_eq!(c.leave_lobby().unwrap().status, NavigationStatus::Ok);
            assert_eq!(c.return_characters().unwrap().status, NavigationStatus::Ok);
        }
        b.close().unwrap();
        assert!(a.battle_snapshot().is_none());
        a.characters(42, 2).unwrap();
        let switched = SelectionContext {
            key: moon.key,
            epoch: 2,
        };
        assert_eq!(
            a.select_character(switched).unwrap().status,
            NavigationStatus::Ok
        );
        assert_eq!(a.enter_channel(0).unwrap().status, NavigationStatus::Ok);
        world(&mut a, &world::Request::read(switched, 0, None));
        let retained = a.battle_snapshot().unwrap().clone();
        let id = a.last_request_id;
        assert!(a.vip_world(world::Request::read(sun, 0, None)).is_err());
        assert_eq!(a.last_request_id, id);
        assert!(Arc::ptr_eq(&retained, a.battle_snapshot().unwrap()));
        a.cancellation.cancel();
        assert!(matches!(
            a.vip_world(world::Request::read(switched, 0, None)),
            Err(TransportError::Cancelled)
        ));
        assert!(a.battle_snapshot().is_none() && a.navigation().is_none());
        assert_eq!(retained.scope(), (switched, 0));
        a.close().unwrap();
        assert_eq!(peer.read_bounded_line("CLOSED").trim_end(), "CLOSED two");
        peer.expect_successful_exit();
    }
}
