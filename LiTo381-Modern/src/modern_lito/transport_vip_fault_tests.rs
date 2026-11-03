// Faults are injected only AFTER the actual server has completed its work.
// Rust sends every operation through production methods, never raw sockets.
fn fault_active(
    a: &mut ModernTlsConnection,
    b: &mut ModernTlsConnection,
    sun: SelectionContext,
    moon: SelectionContext,
) -> (world::Request, world::Request) {
    let room = a
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
                        name: "VIP fault".into(),
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
                room_id: room.summary.room_id,
                expected_revision: room.summary.revision,
                ..rooms::Common::read(moon, 0)
            },
            String::new(),
        )
        .unwrap(),
    )
    .unwrap();
    ready(a, sun);
    let room = ready(b, moon);
    let first = world(
        a,
        &world::Request {
            action: world::Action::Prepare,
            room: room.summary.room_id as i64,
            room_revision: room.summary.revision,
            ..world::Request::read(sun, 0, None)
        },
    );
    let key = &first.attachment().unwrap().world;
    let ra = world::Request::read(sun, 0, Some(key));
    let rb = world::Request::read(moon, 0, Some(key));
    let deadline = Instant::now() + Duration::from_secs(5);
    let loading = loop {
        let r = world(a, &ra);
        if r.data().is_some() {
            break r;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    };
    world(b, &rb);
    let digest = loading.data().unwrap().manifest.digest;
    world(
        a,
        &world::Request {
            action: world::Action::Loaded,
            digest,
            ..ra.clone()
        },
    );
    world(
        b,
        &world::Request {
            action: world::Action::Loaded,
            digest,
            ..rb.clone()
        },
    );
    (ra, rb)
}
#[test]
#[ignore = "11 controlled TLS reply faults after actual VIP world/SET execution; real client atomic state, cancellation/deadline/no retry; no UI"]
fn java_vip_transport_faults_abort_uncertain_world_and_saved_equipment_without_retry() {
    for mode in [
        "world-id",
        "world-source",
        "world-length",
        "world-old-kind",
        "world-truncated",
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
        let (mut a, sun) = login(&peer, 7);
        let (mut b, moon) = login(&peer, 42);
        let (ra, rb) = fault_active(&mut a, &mut b, sun, moon);
        let gear = if mode.starts_with("gear") {
            world(&mut a, &ra);
            let inspect =
                gear::Request::inspect(a.battle_snapshot().unwrap().observation().unwrap())
                    .unwrap();
            let (_, quote) = a.vip_equipment(inspect).unwrap();
            Some(
                quote
                    .unwrap()
                    .set(
                        a.battle_snapshot().unwrap().observation().unwrap(),
                        1,
                        false,
                    )
                    .unwrap(),
            )
        } else {
            None
        };
        let retained = a.battle_snapshot().unwrap().clone();
        let old = retained.observation().unwrap().data().clone();
        let token = a.cancellation.clone();
        if mode.ends_with("timeout") {
            a.set_exchange_timeout(Duration::from_millis(500)).unwrap();
        }
        let sent = ra.clone();
        let task = thread::spawn(move || {
            let result = match gear {
                Some(q) => a.vip_equipment(q).map(|_| ()),
                None => a.vip_world(sent).map(|_| ()),
            };
            (a, result)
        });
        if mode.ends_with("cancel") || mode.ends_with("timeout") {
            assert_eq!(
                peer.read_bounded_line("HELD").trim_end(),
                format!("HELD {mode}")
            );
            if mode.ends_with("cancel") {
                token.cancel();
            }
        }
        let (mut a, result) = task.join().unwrap();
        let error = result.expect_err(mode);
        if mode.ends_with("cancel") {
            assert!(
                matches!(error, TransportError::Cancelled),
                "{mode}: {error}"
            );
        } else if mode.ends_with("timeout") {
            assert!(
                matches!(
                    error,
                    TransportError::DeadlineExceeded(Stage::VipWorld | Stage::VipEquipment)
                ),
                "{mode}: {error}"
            );
        }
        assert!(a.battle_snapshot().is_none() && a.navigation().is_none() && a.principal.is_none());
        assert_eq!(retained.observation().unwrap().data(), &old);
        assert!(retained.full_vip());
        let id = a.last_request_id;
        assert!(a.vip_world(ra).is_err());
        if let Some(q) = gear {
            assert!(a.vip_equipment(q).is_err());
        }
        assert_eq!(
            a.last_request_id, id,
            "no automatic or manual retry on closed owner"
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let r = world(&mut b, &rb);
            if r.attachment().unwrap().lifecycle.phase
                == crate::modern_lito::world_protocol::Phase::Aborted
            {
                break;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
        world(
            &mut b,
            &world::Request {
                action: world::Action::Leave,
                ..rb
            },
        );
        b.close().unwrap();
        assert_eq!(
            peer.read_bounded_line("CLOSED").trim_end(),
            format!("CLOSED {mode}")
        );
        peer.expect_successful_exit();
    }
}
