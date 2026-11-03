// Real SessionController workers, explicit profile and exact queued receipts.
// Protocol-only LOAD below is deliberate: no native window or VIP renderer claim.
mod vip_controller {
    use super::*;
    use crate::modern_lito::{
        battle_control as control,
        login_window::{LoginCommand as C, LoginController, LoginPhase as P, LoginSnapshot},
        profile::ModernProfile,
        session::SessionController,
    };
    use base64::Engine;
    include!("transport_vip_worker_fault_tests.rs");
    fn wait(
        c: &mut SessionController,
        predicate: impl Fn(&LoginSnapshot) -> bool,
    ) -> LoginSnapshot {
        let end = Instant::now() + Duration::from_secs(8);
        loop {
            c.poll();
            let s = c.snapshot();
            assert_ne!(s.phase, P::Failed, "{}", s.status);
            if predicate(&s) {
                return s;
            }
            assert!(
                Instant::now() < end,
                "worker deadline: {:?} {}",
                s.phase,
                s.status
            );
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn phase(c: &mut SessionController, p: P) -> LoginSnapshot {
        wait(c, |s| s.phase == p)
    }
    fn start(peer: &JavaPeer, source: u32) -> (SessionController, SelectionContext) {
        start_checked(peer, source, None)
    }
    fn start_checked(
        peer: &JavaPeer,
        source: u32,
        expected_label: Option<&str>,
    ) -> (SessionController, SelectionContext) {
        start_checked_extension(peer, source, expected_label, "full_vip_v1")
    }
    fn start_checked_extension(
        peer: &JavaPeer,
        source: u32,
        expected_label: Option<&str>,
        extension: &str,
    ) -> (SessionController, SelectionContext) {
        let profile = ModernProfile::parse(
            &serde_json::to_vec(&serde_json::json!({
                "schema":2, "gameplay_protocol":"modern_v47", "gameplay_extension":extension,
                "profile_id":"vip-worker", "display_name":"Owned full VIP worker test",
                "shared_resource_root":std::env::var("LITO_SHARED_RESOURCE_ROOT").unwrap(),
                "endpoint":peer.endpoint.to_string(), "tls_server_name":"localhost",
                "trust":{"kind":"private_ca","certificate_der_base64":
                    base64::engine::general_purpose::STANDARD.encode(private_root())},
                "connect_timeout_ms":1000, "exchange_timeout_ms":2000
            }))
            .unwrap(),
        )
        .unwrap();
        let mut c = SessionController::new(profile);
        c.command(C::Connect).unwrap();
        phase(&mut c, P::LoginReady);
        c.command(C::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        })
        .unwrap();
        phase(&mut c, P::ChoosingSource);
        let context = select_checked(&mut c, source, expected_label);
        (c, context)
    }
    fn select_checked(
        c: &mut SessionController,
        source: u32,
        expected_label: Option<&str>,
    ) -> SelectionContext {
        c.command(C::SelectSource(source)).unwrap();
        let roster = phase(c, P::CharactersReady);
        assert_eq!(roster.characters.len(), 1);
        assert_eq!(roster.characters[0].source_id, source);
        assert_eq!(roster.characters[0].object_id, 3333);
        if let Some(label) = expected_label {
            assert!(roster.characters[0].label.contains(label));
        }
        let context = SelectionContext {
            key: application::CharacterKey {
                source_id: source,
                object_id: 3333,
            },
            epoch: roster.roster_epoch,
        };
        c.command(C::SelectCharacter(context)).unwrap();
        phase(c, P::ChoosingChannel);
        c.command(C::EnterChannel {
            context,
            channel_id: 0,
        })
        .unwrap();
        phase(c, P::InLobby);
        let s = wait(c, |s| s.canonical.snapshot().is_some());
        assert!(s.canonical.vip_world_supported());
        assert!(s.canonical.observation().is_none() && s.world.frame().is_none());
        context
    }
    fn room(c: &mut SessionController, q: rooms::Request) -> LoginSnapshot {
        c.command(C::Rooms(q)).unwrap();
        wait(c, |s| !s.room_pending)
    }
    fn command(c: &mut SessionController, q: control::Command) -> LoginSnapshot {
        let prior = c.snapshot().canonical.receipt_epoch();
        c.command(C::Canonical(q.clone())).unwrap();
        // A queue write is not the command receipt.
        assert!(!c.snapshot().canonical.received_after(&q, prior).is_some());
        let s = wait(c, |s| {
            s.canonical.received_after(&q, prior).is_some() && !s.world_pending
        });
        assert_ne!(s.canonical.outcome(), Some(control::Outcome::NotSent));
        s
    }
    fn ready_worker(c: &mut SessionController, context: SelectionContext) {
        let s = room(
            c,
            rooms::Request::simple(rooms::Common::read(context, 0)).unwrap(),
        );
        let joined = s.rooms.joined().unwrap();
        room(
            c,
            rooms::Request::simple(rooms::Common {
                action: rooms::Action::Ready,
                room_id: joined.summary.room_id,
                expected_revision: joined.summary.revision,
                value: 1,
                ..rooms::Common::read(context, 0)
            })
            .unwrap(),
        );
    }
    fn world_phase(
        c: &mut SessionController,
        expected: crate::modern_lito::world_protocol::Phase,
    ) -> LoginSnapshot {
        wait(c, |s| {
            s.canonical.lifecycle().is_some_and(|l| l.phase == expected)
        })
    }
    #[test]
    #[ignore = "actual full VIP profile/two SessionController workers/TLS/atomic saved receipts/source reload; protocol LOAD, not renderer/Main SQL"]
    fn java_vip_profile_workers_prepare_poll_equipment_leave_and_source_switch() {
        let peer = JavaPeer::start_class(
            "local.lito381.server.ModernVipWorldRuntimeTestPeer",
            "two",
            "60000",
        );
        two_sources(peer, 1, false, false, "two");
    }
    #[test]
    #[ignore = "actual configured Main/owned MySQL/TLS/full_vip_v1 SessionControllers; protocol LOAD, not renderer"]
    fn java_vip_main_sql_workers_preserve_i64_equipment_reentry_and_source_switch() {
        let peer = JavaPeer::start_class(
            "local.lito381.server.ModernVipMainSqlTestPeer",
            "vip-sql",
            "120000",
        );
        assert!(
            peer.process_job.is_some(),
            "SQL child tree must be GO-gated and Job-owned"
        );
        two_sources(peer, 3_000_087, true, true, "vip-sql");
    }
    #[test]
    #[ignore = "actual Main/owned MySQL/TLS full_vip_v2 worker world/READ/gear; no shared renderer"]
    fn java_vip_appearance_main_sql_workers_keep_public_state_through_equipment_and_reentry() {
        let peer = JavaPeer::start_class(
            "local.lito381.server.ModernVipMainSqlTestPeer",
            "vip-sql",
            "120000",
        );
        assert!(peer.process_job.is_some());
        two_sources_extension(peer, 3_000_087, true, true, "vip-sql", "full_vip_v2");
    }
    fn sql_state(s: &LoginSnapshot, source: i32, saved: i64, equipped: bool) {
        let o = s.canonical.vip_observation().unwrap().data();
        let v = &o.vip.as_ref().unwrap().owner;
        let bonus = if equipped { 4_294_967_299 } else { 0 };
        assert_eq!(v.player.source, source);
        assert_eq!(v.versions.saved, saved);
        assert_eq!(v.abilities.0[7], bonus);
        let gear = o.personal.equipment.as_ref().unwrap();
        assert_eq!(gear.effective.maximum_hp, 100 + bonus);
        assert_eq!(
            gear.inventory
                .0
                .iter()
                .find(|i| i.object == 3_000_087)
                .unwrap()
                .equipped,
            source == 42,
            "runtime overrides do not rewrite the imported SQL snapshot"
        );
        assert_eq!(
            gear.equipped
                .0
                .iter()
                .find(|i| i.object == 3_000_087)
                .unwrap()
                .equipped,
            equipped
        );
        assert!(
            v.shape.is_none(),
            "expired VIP must not invent an active shape"
        );
    }
    fn two_sources(peer: JavaPeer, item: i64, desired: bool, sql: bool, mode: &str) {
        two_sources_extension(peer, item, desired, sql, mode, "full_vip_v1");
    }
    fn two_sources_extension(
        mut peer: JavaPeer,
        item: i64,
        desired: bool,
        sql: bool,
        mode: &str,
        extension: &str,
    ) {
        use crate::modern_lito::world_protocol::Phase;
        let (mut a, sun) =
            start_checked_extension(&peer, 7, sql.then_some("SQL Sun Knight"), extension);
        let (mut b, moon) =
            start_checked_extension(&peer, 42, sql.then_some("SQL Moon Mage"), extension);
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
                        name: "VIP worker".into(),
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
        let mut last_match = 0;
        for round in 0..2 {
            ready_worker(&mut a, sun);
            ready_worker(&mut b, moon);
            let sa = world_phase(&mut a, Phase::Loading);
            let sb = world_phase(&mut b, Phase::Loading);
            if sql {
                sql_state(&sa, 7, round, round != 0);
                sql_state(&sb, 42, 0, true);
            }
            for s in [&sa, &sb] {
                let d = s.canonical.vip_observation().unwrap().data();
                assert_eq!(
                    d.vip.as_ref().unwrap().appearances.is_some(),
                    extension == "full_vip_v2"
                );
            }
            let key = sa
                .canonical
                .snapshot()
                .unwrap()
                .attachment()
                .unwrap()
                .world
                .clone();
            assert!(key.match_id > last_match);
            last_match = key.match_id;
            let ra = world::Request::read(sun, 0, Some(&key));
            let rb = world::Request::read(moon, 0, Some(&key));
            let digest = sa
                .canonical
                .vip_observation()
                .unwrap()
                .data()
                .manifest
                .digest;
            assert_eq!(
                sb.canonical
                    .vip_observation()
                    .unwrap()
                    .data()
                    .manifest
                    .digest,
                digest
            );
            assert!(a
                .command(C::Canonical(control::Command::world(ra.clone())))
                .is_err());
            // Passive preparation/read never create a queued-command ACK.
            if round == 0 {
                assert!(sa.canonical.receipt_epoch().is_none());
            }
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
                    ..rb.clone()
                }),
            );
            let sa = world_phase(&mut a, Phase::Active);
            world_phase(&mut b, Phase::Active);
            assert!(sa.canonical.observation().is_none());
            assert_eq!(
                sa.canonical
                    .vip_observation()
                    .unwrap()
                    .data()
                    .vip
                    .as_ref()
                    .unwrap()
                    .owner
                    .versions
                    .saved,
                round
            );
            if round == 0 {
                let inspected = command(
                    &mut a,
                    control::Command::vip_equipment(
                        gear::Request::inspect(sa.canonical.vip_observation().unwrap()).unwrap(),
                    ),
                );
                assert_eq!(
                    inspected.canonical.vip_equipment_reply().unwrap().outcome,
                    gear::Outcome::Inspected
                );
                let quote = inspected.canonical.vip_quote().unwrap();
                let set = quote
                    .set(
                        inspected.canonical.vip_observation().unwrap(),
                        item,
                        desired,
                    )
                    .unwrap();
                let saved = command(&mut a, control::Command::vip_equipment(set));
                assert_eq!(
                    saved
                        .canonical
                        .vip_equipment_reply()
                        .unwrap()
                        .durable_revision,
                    1
                );
                let epoch = saved.canonical.receipt_epoch();
                if sql {
                    sql_state(&saved, 7, 1, true);
                }
                if extension == "full_vip_v2" {
                    let o = saved.canonical.vip_observation().unwrap().data();
                    let v = o.vip.as_ref().unwrap();
                    let appearances = &v.appearances.as_ref().unwrap().0;
                    assert_eq!(appearances.len(), 2);
                    assert_eq!((appearances[0].source, appearances[1].source), (7, 42));
                    assert_eq!(appearances[0].runtime, v.owner.versions.runtime);
                    assert_eq!(appearances[0].lifecycle, v.owner.versions.lifecycle);
                    assert!(appearances
                        .iter()
                        .all(|a| a.graphics == 0 && a.object == 3333));
                }
                assert!(quote
                    .set(saved.canonical.vip_observation().unwrap(), item, !desired)
                    .is_err());
                assert!(a
                    .command(C::Canonical(control::Command::vip_equipment(set)))
                    .is_err());
                let newer = wait(&mut a, |s| {
                    s.canonical
                        .snapshot()
                        .is_some_and(|next| !Arc::ptr_eq(next, saved.canonical.snapshot().unwrap()))
                });
                assert_eq!(newer.canonical.receipt_epoch(), epoch);
                assert_eq!(
                    newer
                        .canonical
                        .vip_equipment_reply()
                        .unwrap()
                        .durable_revision,
                    1
                );
                assert!(newer.canonical.vip_quote().is_some());
                assert_eq!(
                    b.snapshot()
                        .canonical
                        .vip_observation()
                        .unwrap()
                        .data()
                        .vip
                        .as_ref()
                        .unwrap()
                        .owner
                        .versions
                        .saved,
                    0
                );
            }
            command(
                &mut a,
                control::Command::vip_world(world::Request {
                    action: world::Action::Leave,
                    ..ra.clone()
                }),
            );
            world_phase(&mut b, Phase::Aborted);
            command(
                &mut b,
                control::Command::vip_world(world::Request {
                    action: world::Action::Leave,
                    ..rb
                }),
            );
            assert!(!a.snapshot().canonical.attached());
            assert!(a.snapshot().canonical.vip_quote().is_none());
            assert!(a
                .command(C::Canonical(control::Command::vip_world(ra)))
                .is_err());
        }
        for (c, context) in [(&mut b, moon), (&mut a, sun)] {
            let s = room(
                c,
                rooms::Request::simple(rooms::Common::read(context, 0)).unwrap(),
            );
            let r = s.rooms.joined().unwrap();
            room(
                c,
                rooms::Request::simple(rooms::Common {
                    action: rooms::Action::Leave,
                    room_id: r.summary.room_id,
                    expected_revision: r.summary.revision,
                    ..rooms::Common::read(context, 0)
                })
                .unwrap(),
            );
            c.command(C::LeaveLobby(context)).unwrap();
            phase(c, P::ChoosingChannel);
            c.command(C::ReturnCharacters(context)).unwrap();
            phase(c, P::CharactersReady);
            assert!(c.snapshot().canonical.snapshot().is_none());
            assert!(c.snapshot().canonical.vip_quote().is_none());
        }
        b.command(C::Disconnect).unwrap();
        phase(&mut b, P::Disconnected);
        let switched = select_checked(&mut a, 42, sql.then_some("SQL Moon Mage"));
        assert_ne!(switched.epoch, sun.epoch);
        assert!(a
            .command(C::Canonical(control::Command::vip_world(
                world::Request::read(sun, 0, None)
            )))
            .is_err());
        a.command(C::Disconnect).unwrap();
        phase(&mut a, P::Disconnected);
        assert!(a.is_quiescent() && b.is_quiescent());
        assert_eq!(
            peer.read_bounded_line("CLOSED").trim_end(),
            format!("CLOSED {mode}")
        );
        peer.expect_successful_exit();
    }
    #[test]
    #[ignore = "actual Main SQL source whitelist through real full_vip_v1 worker; no implicit fallback or GUI"]
    fn java_vip_main_sql_unselected_source_stays_connected_and_switches_cleanly() {
        use crate::modern_lito::world_protocol::{Reason, Status};
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernVipMainSqlTestPeer",
            "vip-subset-sql",
            "60000",
        );
        assert!(peer.process_job.is_some());
        let (mut c, moon) = start_checked(&peer, 42, Some("SQL Moon Mage"));
        assert!(!c.snapshot().canonical.attached());
        let q = control::Command::vip_world(world::Request::read(moon, 0, None));
        let unavailable = command(&mut c, q);
        assert_eq!(
            unavailable.canonical.outcome(),
            Some(control::Outcome::VipWorld(
                Status::Unavailable,
                Reason::NoBinding
            ))
        );
        assert!(
            unavailable.canonical.vip_observation().is_none()
                && unavailable.canonical.vip_quote().is_none()
        );
        c.command(C::LeaveLobby(moon)).unwrap();
        phase(&mut c, P::ChoosingChannel);
        c.command(C::ReturnCharacters(moon)).unwrap();
        phase(&mut c, P::CharactersReady);
        let sun = select_checked(&mut c, 7, Some("SQL Sun Knight"));
        assert_ne!(sun.epoch, moon.epoch);
        let available = command(
            &mut c,
            control::Command::vip_world(world::Request::read(sun, 0, None)),
        );
        assert_eq!(
            available.canonical.outcome(),
            Some(control::Outcome::VipWorld(Status::Ok, Reason::None))
        );
        assert!(c
            .command(C::Canonical(control::Command::vip_world(
                world::Request::read(moon, 0, None)
            )))
            .is_err());
        c.command(C::Disconnect).unwrap();
        phase(&mut c, P::Disconnected);
        assert!(c.is_quiescent());
        assert_eq!(
            peer.read_bounded_line("CLOSED").trim_end(),
            "CLOSED vip-subset-sql"
        );
        peer.expect_successful_exit();
    }
}
