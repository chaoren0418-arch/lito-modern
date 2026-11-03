// Included in transport::tests. Opt-in real Rust/Java TLS, not SQL or HWND.
#[test]
fn skill54_catalog_requires_the_complete_explicit_v42_capability() {
    use crate::modern_lito::protocol as p;
    let mut reply=spell::SkillsReply {
        context:SelectionContext{ key:application::CharacterKey{source_id:7,object_id:3333},epoch:1 },
        status:spell::Status::Ok,
        skills:vec![spell::Skill{skill_id:54,name:"has".into(),mp_cost:60,reuse_millis:10,support:spell::Support::GreaterHaste}],
    };
    let ack=Ack{request_id:1,version:42,numeric_schema:1,capabilities:p::KNOWN_CAPABILITIES};
    assert!(skill_catalog_negotiated(&reply,&ack));
    assert!(!skill_catalog_negotiated(&reply,&Ack{version:41,..ack}));
    for cap in [p::CAP_GREATER_HASTE,p::CAP_TIMED,p::CAP_BATTLE_DATA,p::CAP_ACTION_LIFE,p::CAP_MATCH_DESCRIPTOR] {
        let off=Ack{capabilities:ack.capabilities & !cap,..ack};
        assert!(!skill_catalog_negotiated(&reply,&off));
        reply.skills[0].support=spell::Support::UnsupportedTemplate;
        assert!(!skill_catalog_negotiated(&reply,&off));
        reply.skills[0].support=spell::Support::Unimplemented;
        assert!(skill_catalog_negotiated(&reply,&off));
        reply.skills[0].support=spell::Support::GreaterHaste;
    }
}
#[test]
#[ignore = "v31 real NPC70 two controllers, full shared map554/SPR1819/Sound710 preflight, source skills/kill/retire/return; fixture repositories, no SQL/HWND"]
fn java_npc70_v31_controllers_real_shared_cast_kill_retire_and_source_switch() {
    npc_shared_controller_flow(false, false, false, false, true);
}
#[test]
fn npc70_and_skill77_catalog_capabilities_are_independent() {
    use crate::modern_lito::protocol as p;
    let context = SelectionContext {
        key: application::CharacterKey {
            source_id: 7,
            object_id: 3333,
        },
        epoch: 1,
    };
    let mut r = spell::SkillsReply {
        context,
        status: spell::Status::Ok,
        skills: vec![
            spell::Skill {
                skill_id: 70,
                name: "70".into(),
                mp_cost: 48,
                reuse_millis: 2000,
                support: spell::Support::FireStorm,
            },
            spell::Skill {
                skill_id: 77,
                name: "77".into(),
                mp_cost: 70,
                reuse_millis: 5000,
                support: spell::Support::Disintegrate,
            },
        ],
    };
    let caps = (p::RUNTIME_CAPABILITIES & !p::CAP_FIRE_STORM)
        | p::CAP_NPC
        | p::CAP_NPC_FIRE_STORM
        | p::CAP_NPC_DISINTEGRATE;
    let ack = Ack {
        request_id: 1,
        version: 31,
        numeric_schema: 1,
        capabilities: caps,
    };
    assert!(skill_catalog_negotiated(&r, &ack));
    for bit in [p::CAP_NPC_FIRE_STORM, p::CAP_NPC_DISINTEGRATE] {
        assert!(!skill_catalog_negotiated(
            &r,
            &Ack {
                capabilities: caps & !bit,
                ..ack
            }
        ));
    }
    r.skills[0].support = spell::Support::Unimplemented;
    assert!(skill_catalog_negotiated(
        &r,
        &Ack {
            capabilities: caps & !p::CAP_NPC_FIRE_STORM,
            ..ack
        }
    ));
    r.skills[0].support = spell::Support::FireStorm;
    assert!(skill_catalog_negotiated(
        &r,
        &Ack {
            capabilities: caps & !p::CAP_NPC_FIRE_STORM | p::CAP_FIRE_STORM,
            ..ack
        }
    ));
    assert!(!skill_catalog_negotiated(&r, &Ack { version: 30, ..ack }));
}
#[test]
fn npc31_tls_budget_accepts_the_versioned_envelope_without_relaxing_old_sessions() {
    for version in 1..31 {
        assert_eq!(npc_wire_budget(version), 512 * 1024);
    }
    assert_eq!(npc_wire_budget(31), 2 * 1024 * 1024);
}

fn npc70_login(peer: &JavaPeer, source: u32) -> (ModernTlsConnection, SelectionContext) {
    use crate::modern_lito::protocol as p;
    let mut hello = Hello::current(100);
    hello.max_version = 31;
    hello.supported_capabilities =
        (hello.supported_capabilities & !p::CAP_FIRE_STORM) | p::CAP_NPC | p::CAP_NPC_FIRE_STORM;
    let mut c = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(&private_root()),
        hello,
        integration_limits(),
        Cancellation::new(),
    )
    .unwrap();
    assert_eq!(c.ack().unwrap().version, 31);
    assert!(crate::modern_lito::npc_fire_storm_protocol::negotiated(
        &c.ack().unwrap()
    ));
    assert!(!c.supports_fire_storm(), "NPC70 cannot piggyback on PvP70");
    c.authenticate(fixture_credentials("fixture-password"))
        .unwrap();
    let roster = c.characters(source, 1).unwrap();
    let context = SelectionContext {
        key: roster.characters[0].key,
        epoch: 1,
    };
    c.select_character(context).unwrap();
    c.enter_channel(0).unwrap();
    (c, context)
}
fn npc70_room_action(
    c: &mut ModernTlsConnection,
    context: SelectionContext,
    action: rooms::Action,
) {
    let room = c
        .rooms(rooms::Request::read(context, 0).unwrap())
        .unwrap()
        .joined
        .unwrap();
    let q = rooms::Common {
        context,
        channel_id: 0,
        action,
        value: u8::from(action == rooms::Action::Ready),
        room_id: room.summary.room_id,
        expected_revision: room.summary.revision,
    };
    assert_eq!(
        c.rooms(rooms::Request::simple(q).unwrap()).unwrap().status,
        rooms::Status::Ok
    );
}
fn npc70_read(context: SelectionContext, f: &world::Frame) -> npc::Request {
    npc::Request {
        context,
        channel_id: 0,
        action: npc::Action::Read,
        world: npc::Scope {
            room_id: f.room.summary.room_id,
            match_id: f.match_id,
            generation: f.generation,
        },
        sequence: 0,
        serial: 0,
        life: 0,
    }
}
#[test]
#[ignore = "compiled Java NPC70 service peer; real Rust TLS and source-scoped in-memory repositories/flat geometry, no SQL/renderer/HWND"]
fn java_npc70_v31_public_connection_cast_denial_source_reload_return_and_retire() {
    use crate::modern_lito::npc_fire_storm_protocol as fire;
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernNpcServiceTestPeer",
        "npc70-service-v31",
        "30000",
    );
    let (mut owner, a) = npc70_login(&peer, 7);
    let (mut guest, b) = npc70_login(&peer, 42);
    assert!(owner
        .skills()
        .unwrap()
        .skills
        .iter()
        .any(|s| s.skill_id == 70 && s.support == spell::Support::FireStorm));
    assert!(!guest
        .skills()
        .unwrap()
        .skills
        .iter()
        .any(|s| s.skill_id == 70));
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
                        name: "NPC70 TLS".into(),
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
    assert!(loading
        .descriptor
        .unwrap()
        .has(world::MatchDescriptor::NPC_FIRE_STORM));
    assert!(!loading
        .descriptor
        .unwrap()
        .has(world::MatchDescriptor::FIRE_STORM));
    guest.world(world::Request::read(b, 0)).unwrap();
    let read = npc70_read(a, &loading);
    let first = owner.npc(read).unwrap();
    let plan = crate::modern_lito::npc_presentation::Plan::from_reply(&first).unwrap();
    assert!(first.frame.as_ref().unwrap().actors.is_empty());
    // Only protocol LOADED here; shared renderer is exercised separately.
    for (c, context) in [(&mut owner, a), (&mut guest, b)] {
        let mut q = world::Request::read(context, 0);
        q.action = world::Action::Loaded;
        q.match_id = loading.match_id;
        q.generation = loading.generation;
        q.manifest_digest = loading.manifest.digest;
        assert_eq!(c.world(q).unwrap().status, world::Status::Ok);
    }
    let active = owner.npc(read).unwrap();
    plan.validate(&active).unwrap();
    let f = active.frame.as_ref().unwrap();
    let q = fire::Request {
        context: a,
        channel_id: 0,
        scope: f.scope,
        sequence: 1,
        caster_life: 1,
    };
    assert!(q.ready_in(f));
    let hit = owner.fire_storm_npc(q).unwrap();
    assert_eq!(hit.status, npc::Status::Ok);
    let f = hit.frame.as_ref().unwrap();
    let book = f.combat.npc_fire(1).unwrap();
    assert_eq!((book.count, book.mp_spent), (1, 48));
    assert_eq!(book.targets.len(), 1);
    assert_eq!(f.combat.vital(1).unwrap().mp, 952);
    assert_eq!(
        owner.world_snapshot().unwrap().frame.as_ref(),
        Some(&f.combat.world)
    );
    assert_eq!(
        owner.combat_snapshot().unwrap().frame.as_ref(),
        Some(&f.combat)
    );
    let denied = owner
        .fire_storm_npc(fire::Request { sequence: 2, ..q })
        .unwrap();
    assert_eq!(denied.reason, fire::Reason::TooFast);
    assert_eq!(
        denied.frame.as_ref().unwrap().combat.npc_fires,
        f.combat.npc_fires
    );
    assert!(owner.fire_storm_npc(q).is_err());
    for invalid in [
        fire::Request {
            context: b,
            sequence: 3,
            ..q
        },
        fire::Request {
            caster_life: 2,
            sequence: 3,
            ..q
        },
        fire::Request {
            channel_id: 1,
            sequence: 3,
            ..q
        },
    ] {
        assert!(owner.fire_storm_npc(invalid).is_err());
        assert!(owner.npc_snapshot().is_some());
    }
    let guest_read = npc70_read(b, &loading);
    guest.npc(guest_read).unwrap();
    let unlearned = guest
        .fire_storm_npc(fire::Request { context: b, ..q })
        .unwrap();
    assert_eq!(unlearned.reason, fire::Reason::Unlearned);
    assert_eq!(
        unlearned
            .frame
            .as_ref()
            .unwrap()
            .combat
            .npc_fire(2)
            .unwrap()
            .count,
        0
    );
    thread::sleep(Duration::from_millis(2100));
    let second = owner
        .fire_storm_npc(fire::Request { sequence: 3, ..q })
        .unwrap();
    assert_eq!(second.status, npc::Status::Ok);
    assert_eq!(
        second
            .frame
            .as_ref()
            .unwrap()
            .combat
            .npc_fire(1)
            .unwrap()
            .mp_spent,
        96
    );
    let mut leave = world::Request::read(a, 0);
    leave.action = world::Action::Leave;
    leave.match_id = loading.match_id;
    leave.generation = loading.generation;
    assert!(owner.world(leave).unwrap().frame.is_none());
    assert!(owner.npc_snapshot().is_none());
    assert!(owner
        .fire_storm_npc(fire::Request { sequence: 4, ..q })
        .is_err());
    assert!(guest.npc(guest_read).unwrap().frame.unwrap().closed);
    leave.context = b;
    assert!(guest.world(leave).unwrap().frame.is_none());
    npc70_room_action(&mut guest, b, rooms::Action::Leave);
    assert_eq!(
        guest.return_characters().unwrap().snapshot.phase,
        NavigationPhase::Characters
    );
    npc70_room_action(&mut owner, a, rooms::Action::Leave);
    assert_eq!(
        owner.return_characters().unwrap().snapshot.phase,
        NavigationPhase::Characters
    );
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
    assert!(!owner
        .skills()
        .unwrap()
        .skills
        .iter()
        .any(|s| s.skill_id == 70));
    assert!(owner.fire_storm_npc(q).is_err());
    guest.close().unwrap();
    owner.close().unwrap();
    peer.expect_successful_exit();
}
