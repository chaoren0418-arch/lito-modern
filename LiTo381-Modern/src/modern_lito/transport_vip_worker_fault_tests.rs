// Includes real worker queue cancellation and failure publication.
#[test]
#[ignore = "seven injected real TLS faults in production SessionController workers; uncertain saves never retried; not native UI"]
fn java_vip_workers_faults_clear_receipts_and_do_not_retry_uncertain_saves() {
    use crate::modern_lito::world_protocol::Phase;
    for mode in [
        "world-id",
        "world-replay",
        "world-timeout",
        "world-cancel",
        "gear-id",
        "gear-timeout",
        "gear-cancel",
    ] {
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernVipTransportFaultTestPeer",
            mode,
            "30000",
        );
        let (mut a, sun) = start(&peer, 7);
        let (mut b, moon) = start(&peer, 42);
        room(
            &mut a,
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
                        name: "Worker fault".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        let r = a.snapshot().rooms.joined().unwrap().summary.clone();
        room(
            &mut b,
            rooms::Request::join(
                rooms::Common {
                    action: rooms::Action::Join,
                    room_id: r.room_id,
                    expected_revision: r.revision,
                    ..rooms::Common::read(moon, 0)
                },
                String::new(),
            )
            .unwrap(),
        );
        ready_worker(&mut a, sun);
        ready_worker(&mut b, moon);
        let loading = world_phase(&mut a, Phase::Loading);
        world_phase(&mut b, Phase::Loading);
        let key = loading
            .canonical
            .snapshot()
            .unwrap()
            .attachment()
            .unwrap()
            .world
            .clone();
        let ra = world::Request::read(sun, 0, Some(&key));
        let rb = world::Request::read(moon, 0, Some(&key));
        let digest = loading
            .canonical
            .vip_observation()
            .unwrap()
            .data()
            .manifest
            .digest;
        command(
            &mut a,
            control::Command::vip_world(world::Request {
                action: world::Action::Loaded,
                digest,
                ..ra.clone()
            }),
        );
        command(
            &mut b,
            control::Command::vip_world(world::Request {
                action: world::Action::Loaded,
                digest,
                ..rb
            }),
        );
        let (retained, mutation) = if mode.starts_with("gear") {
            let active = world_phase(&mut a, Phase::Active);
            let inspected = command(
                &mut a,
                control::Command::vip_equipment(
                    gear::Request::inspect(active.canonical.vip_observation().unwrap()).unwrap(),
                ),
            );
            let q = inspected
                .canonical
                .vip_quote()
                .unwrap()
                .set(inspected.canonical.vip_observation().unwrap(), 1, false)
                .unwrap();
            a.command(C::Canonical(control::Command::vip_equipment(q)))
                .unwrap();
            (inspected.canonical, Some(q))
        } else {
            (loading.canonical, None)
        };
        let retained_data = retained.vip_observation().unwrap().data().clone();
        if mode.ends_with("timeout") || mode.ends_with("cancel") {
            assert_eq!(
                peer.read_bounded_line("HELD").trim_end(),
                format!("HELD {mode}")
            );
        }
        if mode.ends_with("cancel") {
            a.command(C::Disconnect).unwrap();
            phase(&mut a, P::Disconnected);
        } else {
            let deadline = Instant::now() + Duration::from_secs(8);
            loop {
                a.poll();
                if a.snapshot().phase == P::Failed {
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "fault did not retire worker: {mode}"
                );
                thread::sleep(Duration::from_millis(5));
            }
        }
        let closed = a.snapshot();
        assert!(
            closed.canonical.snapshot().is_none() && closed.canonical.receipt_epoch().is_none()
        );
        assert!(
            closed.canonical.vip_quote().is_none()
                && closed.canonical.vip_equipment_reply().is_none()
        );
        assert!(closed.navigation.is_none());
        assert_eq!(retained.vip_observation().unwrap().data(), &retained_data);
        assert!(a
            .command(C::Canonical(control::Command::vip_world(ra)))
            .is_err());
        if let Some(q) = mutation {
            assert!(a
                .command(C::Canonical(control::Command::vip_equipment(q)))
                .is_err());
        }
        world_phase(&mut b, Phase::Aborted);
        let leave = b.snapshot().canonical.leave_command().unwrap();
        assert_eq!(leave.audit_kind(), "vip-world-leave");
        command(&mut b, leave);
        b.command(C::Disconnect).unwrap();
        phase(&mut b, P::Disconnected);
        a.command(C::Disconnect).unwrap();
        phase(&mut a, P::Disconnected);
        assert!(a.is_quiescent() && b.is_quiescent());
        assert_eq!(
            peer.read_bounded_line("CLOSED").trim_end(),
            format!("CLOSED {mode}")
        );
        peer.expect_successful_exit();
        eprintln!("VIP_WORKER_FAULT_PASS {mode}");
    }
}
