// Included in transport::tests. Real TLS/engine receipts, explicit in-memory
// repositories/RNG, full shared map554 assets; no SQL, native bin or HWND claim.
fn npc_actions_login(peer: &JavaPeer, source: u32) -> (ModernTlsConnection, SelectionContext) {
    use crate::modern_lito::protocol as p;
    let mut hello = Hello::current(100);
    hello.max_version = 29;
    hello.supported_capabilities |= p::CAP_NPC | p::CAP_NPC_DISINTEGRATE;
    let mut c = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(&private_root()),
        hello,
        integration_limits(),
        Cancellation::new(),
    )
    .unwrap();
    assert_eq!(c.ack().unwrap().version, 29);
    c.authenticate(fixture_credentials("fixture-password"))
        .unwrap();
    let roster = c.characters(source, 1).unwrap();
    let context = SelectionContext {
        key: roster.characters[0].key,
        epoch: 1,
    };
    c.select_character(context).unwrap();
    c.enter_channel(0).unwrap();
    let skills = c.skills().unwrap();
    for id in [4, 5, 77] {
        assert!(skills.skills.iter().any(|s| s.skill_id == id));
    }
    (c, context)
}

#[test]
#[ignore = "real Java TLS skill77/4/5 then NPC melee, shared map554 full compositor; fixture repositories/RNG and replay receipt clocks, no SQL/HWND"]
fn java_npc_actions_live_receipts_shared_renderer_melee_arbitration() {
    use crate::modern_lito::{
        npc_disintegrate_protocol as dis, world_runtime::live_skill_tests as render,
    };
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernNpcServiceTestPeer",
        "npc-actions-service-shared-v29",
        "60000",
    );
    let (mut owner, a) = npc_actions_login(&peer, 7);
    let (mut guest, b) = npc_actions_login(&peer, 42);
    let mut common = rooms::Common::read(a, 0);
    common.action = rooms::Action::Create;
    let room = owner
        .rooms(
            rooms::Request::draft(
                common,
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 4,
                        mode: 2,
                        attributes: 16,
                        map_id: 554,
                        name: "Live spell melee".into(),
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
    let join = rooms::Common {
        context: b,
        channel_id: 0,
        action: rooms::Action::Join,
        value: 0,
        room_id: room.summary.room_id,
        expected_revision: room.summary.revision,
    };
    assert_eq!(
        guest
            .rooms(rooms::Request::join(join, String::new()).unwrap())
            .unwrap()
            .status,
        rooms::Status::Ok
    );
    npc70_room_action(&mut owner, a, rooms::Action::Ready);
    npc70_room_action(&mut guest, b, rooms::Action::Ready);
    let room = owner
        .rooms(rooms::Request::read(a, 0).unwrap())
        .unwrap()
        .joined
        .unwrap();
    let mut prepare = world::Request::read(a, 0);
    prepare.action = world::Action::Prepare;
    prepare.room_id = room.summary.room_id;
    prepare.expected_room_revision = room.summary.revision;
    assert_eq!(owner.world(prepare).unwrap().status, world::Status::Ok);
    let deadline = Instant::now() + Duration::from_secs(5);
    let loading = loop {
        let f = owner
            .world(world::Request::read(a, 0))
            .unwrap()
            .frame
            .unwrap();
        if f.phase != world::Phase::Preparing {
            break f;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(loading.phase, world::Phase::Loading);
    guest.world(world::Request::read(b, 0)).unwrap();
    let read = npc70_read(a, &loading);
    let first = owner.npc(read).unwrap();
    // The owner really qualifies all manifest/assets and draws before LOADED.
    // The second session is an explicitly protocol-only peer, not a second UI.
    drop(render::Probe::new(&first));
    for (c, context) in [(&mut owner, a), (&mut guest, b)] {
        let mut q = world::Request::read(context, 0);
        q.action = world::Action::Loaded;
        q.match_id = loading.match_id;
        q.generation = loading.generation;
        q.manifest_digest = loading.manifest.digest;
        assert_eq!(c.world(q).unwrap().status, world::Status::Ok);
    }
    let mut captured = Vec::new();
    for skill in [77, 4, 5] {
        // Wait for real server cooldowns. Presentation replay later deliberately
        // models queued/coalesced delivery; it does not alter these receipts.
        thread::sleep(Duration::from_millis(1300));
        let before = owner.npc(read).unwrap();
        let f = before.frame.as_ref().unwrap();
        assert_eq!(f.combat.world.phase, world::Phase::Active);
        let target = f.actors.iter().find(|n| n.alive()).unwrap();
        let target_id = npc::Id {
            scope: f.scope,
            serial: target.serial,
        };
        let sequence = f.combat.world.cast(1).unwrap().request_sequence + 1;
        let cast = match skill {
            77 => {
                let r = owner
                    .disintegrate_npc(dis::Request {
                        context: a,
                        channel_id: 0,
                        target: target_id,
                        sequence,
                        caster_life: 1,
                        target_life: target.life,
                    })
                    .unwrap();
                assert_eq!(r.status, npc::Status::Ok, "{:?}", r.reason);
                r.npc_reply()
            }
            4 => {
                let r = owner
                    .cast_npc(npc_spell::Request {
                        context: a,
                        channel_id: 0,
                        target: target_id,
                        sequence,
                        caster_life: 1,
                        target_life: target.life,
                    })
                    .unwrap();
                assert_eq!(r.status, npc::Status::Ok, "{:?}", r.reason);
                r.npc_reply()
            }
            5 => {
                let r = owner
                    .cast(spell::CastRequest {
                        context: a,
                        channel_id: 0,
                        match_id: loading.match_id,
                        generation: loading.generation,
                        sequence,
                        skill_id: 5,
                        target_entity_id: 0,
                    })
                    .unwrap();
                assert_eq!(r.status, spell::Status::Ok, "{:?}", r.reason);
                let next = owner.npc(read).unwrap();
                let event = next.frame.as_ref().unwrap().combat.world.cast(1).unwrap();
                assert_ne!(event.from, event.to);
                assert_eq!(event, r.frame.as_ref().unwrap().world.cast(1).unwrap());
                next
            }
            _ => unreachable!(),
        };
        let c = cast.frame.as_ref().unwrap();
        let cost = match skill {
            77 => 70,
            4 => 3,
            _ => 10,
        };
        assert_eq!(
            f.combat.vital(1).unwrap().mp - c.combat.vital(1).unwrap().mp,
            cost
        );
        thread::sleep(Duration::from_millis(1300));
        let melee = owner
            .npc(npc::Request {
                action: npc::Action::Attack,
                sequence: c.combat.vital(1).unwrap().attack_sequence + 1,
                serial: target.serial,
                life: target.life,
                ..read
            })
            .unwrap();
        assert_eq!(
            melee.status,
            npc::Status::Ok,
            "skill {skill}: {:?}",
            melee.reason
        );
        captured.push((skill, before, cast, melee));
    }
    // Retire the live owners before expensive receipt replay; a failed renderer
    // must not leave TLS workers, room presence or a child Java process behind.
    let mut leave = world::Request::read(a, 0);
    leave.action = world::Action::Leave;
    leave.match_id = loading.match_id;
    leave.generation = loading.generation;
    assert!(owner.world(leave).unwrap().frame.is_none());
    assert!(owner.npc_snapshot().is_none());
    let guest_read = npc70_read(b, &loading);
    assert!(guest.npc(guest_read).unwrap().frame.unwrap().closed);
    leave.context = b;
    assert!(guest.world(leave).unwrap().frame.is_none());
    for (c, context) in [(&mut owner, a), (&mut guest, b)] {
        npc70_room_action(c, context, rooms::Action::Leave);
        assert_eq!(
            c.return_characters().unwrap().snapshot.phase,
            NavigationPhase::Characters
        );
    }
    let roster = owner.characters(42, 2).unwrap();
    let next = SelectionContext {
        key: roster.characters[0].key,
        epoch: 2,
    };
    assert_eq!(
        owner.select_character(next).unwrap().status,
        NavigationStatus::Ok
    );
    owner.enter_channel(0).unwrap();
    assert!(owner.npc(read).is_err());
    guest.close().unwrap();
    owner.close().unwrap();
    peer.expect_successful_exit();
    for (skill, before, cast, melee) in captured {
        render::prove(skill, &before, &cast, &melee);
        eprintln!("live shared skill {skill} -> typed melee receipt replay passed");
    }
}
