const HASTE44_CAPABILITIES: u32 =
    crate::modern_lito::protocol::KNOWN_CAPABILITIES & !crate::modern_lito::protocol::CAP_SLOW_DATA;

#[test]
fn haste44_offer_requires_explicit_capability_dependencies_and_preserves_player_default() {
    use crate::modern_lito::protocol as p;
    let mut hello = p::Hello::current(1);
    assert_eq!(hello.supported_capabilities & p::CAP_HASTE, 0);
    assert_eq!(p::RUNTIME_WORLD_VERSION, 37);
    hello.max_version = 44;
    hello.supported_capabilities = HASTE44_CAPABILITIES;
    assert!(p::encode(p::Message::Hello(hello)).is_ok());
    for version in 1..44 {
        let mut old = hello;
        old.max_version = version;
        assert!(p::encode(p::Message::Hello(old)).is_err());
        assert!(p::encode(p::Message::Ack(p::Ack {
            request_id: 1,
            version,
            numeric_schema: 1,
            capabilities: HASTE44_CAPABILITIES
        }))
        .is_err());
    }
    for bit in [
        p::CAP_TIMED,
        p::CAP_BATTLE_DATA,
        p::CAP_ACTION_LIFE,
        p::CAP_MATCH_DESCRIPTOR,
    ] {
        let mut bad = hello;
        bad.supported_capabilities &= !bit;
        assert!(p::encode(p::Message::Hello(bad)).is_err());
        bad = hello;
        bad.required_capabilities = HASTE44_CAPABILITIES & !bit;
        assert!(p::encode(p::Message::Hello(bad)).is_err());
    }
}

#[test]
#[ignore = "explicit JDK17/TLS fixtures; production TLS/decoder/Follower with engine publications, not room/UI/cast-command qualification"]
fn java_haste44_authenticated_tls_two_owners_catchup_replay_and_owner_rejection_are_atomic() {
    use crate::modern_lito::{
        battle_follow, battle_payload::data, battle_read as read, protocol as p,
    };
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernHasteReadTestPeer",
        "normal",
        "15000",
    );
    let ca = private_root();
    for _ in 0..2 {
        let scope = peer.read_bounded_line("SCOPE");
        let mut fields = scope.split_whitespace();
        assert_eq!(fields.next(), Some("SCOPE"));
        let values = fields
            .map(|s| s.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(values.len(), 7);
        let request = read::Request {
            context: SelectionContext {
                key: application::CharacterKey {
                    source_id: values[0] as u32,
                    object_id: values[1] as u64,
                },
                epoch: values[2] as u64,
            },
            channel_id: values[3] as u8,
            world: data::WorldId {
                room: values[4],
                match_id: values[5],
                generation: values[6],
            },
        };
        request.validate().unwrap();
        let mut hello = p::Hello::current(1);
        hello.min_version = 44;
        hello.max_version = 44;
        hello.supported_capabilities = HASTE44_CAPABILITIES;
        hello.required_capabilities = HASTE44_CAPABILITIES;
        let mut c = connect_and_negotiate(
            peer.endpoint,
            "localhost",
            Some(&ca),
            hello,
            TransportLimits::default(),
            Cancellation::new(),
        )
        .unwrap();
        c.authenticate(fixture_credentials("fixture-password"))
            .unwrap();
        let mut follower = battle_follow::Follower::new(request.clone(), c.ack().unwrap()).unwrap();
        for step in 0..10 {
            let id = c.next_request_id().unwrap();
            let frame = read::encode_request(id, &request).unwrap();
            // Test-only transport exchange: the public v44 player lifecycle uses
            // BattleWorld, not Read. No selected lobby or LOAD authority is faked.
            let Response::BattleData(reply) = c.exchange(&frame, id, Stage::BattleData).unwrap()
            else {
                panic!("expected battle read");
            };
            let keep = follower.observation().cloned();
            if step == 5 || step == 6 {
                assert!(follower.accept(reply).is_err());
                assert_eq!(
                    follower.observation().unwrap().data(),
                    keep.as_ref().unwrap().data()
                );
                assert_eq!(
                    follower.observation().unwrap().personal(),
                    keep.as_ref().unwrap().personal()
                );
                assert_eq!(follower.state(), read::State::Data);
                continue;
            }
            assert_eq!(reply.echo, request);
            follower.accept(reply).unwrap();
            let next = follower.observation().unwrap();
            assert!(next.data().haste.is_some());
            assert!(next.data().npc_motion.is_none());
            assert_eq!(next.personal().player(), &request.player());
            if let Some(old) = keep {
                old.check_successor(next).unwrap();
            }
            if step == 2 || step == 3 {
                assert_eq!(next.data().haste.as_ref().unwrap().state.targets.0.len(), 1);
            }
            if step >= 7 {
                assert!(next
                    .data()
                    .haste
                    .as_ref()
                    .unwrap()
                    .state
                    .targets
                    .0
                    .is_empty());
            }
            if step >= 8 {
                assert_eq!(next.data().world.phase, data::Phase::Aborted);
            }
        }
        c.close().unwrap();
    }
    peer.expect_successful_exit();
}
