use super::*;
use crate::modern_lito::battle_window::Stamp;

fn fixture() -> (action::Guard, Vec<u8>, payload::UnvalidatedData) {
    let mut data = payload::data_tests::fixture();
    data.world.phase = d::Phase::Active;
    data.world.revision = 3;
    data.world.loading_elapsed_nanos = 1_000;
    data.world.active_elapsed_nanos = 0;
    for actor in &mut data.actors.0 {
        actor.loaded = true;
    }
    data.actors.0[0].last_move_sequence = 1;
    data.actors.0[0].position.x += 1;
    data.actors.0[0].heading = 2;
    data.presentation = Some(d::List(
        data.actors
            .0
            .iter()
            .map(|a| d::PublicPresentation {
                entity: a.entity,
                shot: None,
                item: None,
            })
            .collect(),
    ));
    let guard = action::Guard {
        stamp: Stamp {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: 1u64 << 63,
            },
            channel: 0,
            world: [101, 102, 103],
            entity: 1,
            life: 1,
            revision: 2,
        },
        target: None,
    };
    let inner = world::encode_request(
        9,
        &world::Request {
            action: world::Action::Move,
            heading: 2,
            match_id: 102,
            generation: 103,
            sequence: 1,
            from_x: 32768,
            from_y: 32768,
            ..world::Request::read(guard.stamp.context, 0)
        },
    )
    .unwrap();
    (guard, encode(guard, &inner, VERSION).unwrap(), data)
}
fn response(expected: &[u8], data: &payload::UnvalidatedData) -> Vec<u8> {
    let payload = payload::encode(data).unwrap();
    let mut bytes = expected.to_vec();
    bytes[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
    bytes.extend([0, 1, 0, 0]);
    bytes.extend((data.world.revision + 1).to_be_bytes());
    bytes.extend([3, 0, 0, 0]);
    bytes.extend([0; 12]);
    bytes.extend((payload.len() as u32).to_be_bytes());
    bytes.extend(payload);
    let length = bytes.len() as u32;
    bytes[4..8].copy_from_slice(&length.to_be_bytes());
    bytes
}
fn parse(
    guard: action::Guard,
    expected: &[u8],
    bytes: &[u8],
) -> Result<Reply, app::ApplicationError> {
    read_reply(&mut bytes.as_ref(), guard, expected, VERSION)
}
#[test]
fn exact_schema_two_echo_and_owner_survive_without_legacy_projection() {
    let (g, expected, data) = fixture();
    assert_eq!(expected.len(), 236);
    assert_eq!(u16_at(&expected, 12), REQUEST_KIND);
    let reply = parse(g, &expected, &response(&expected, &data)).unwrap();
    assert_eq!(reply.data, data);
    assert_eq!(reply.guard, g);
    assert_eq!(reply.disposition, Disposition::Applied);
    assert_eq!(reply.lifecycle.revision, 4);
    assert_eq!(reply.lifecycle.settlement, None);
    assert_eq!(
        format!("{reply:?}"),
        "CanonicalBattleOperationReply[redacted]"
    );
    assert_eq!(protocol::RUNTIME_WORLD_VERSION, 37);
    for version in [1, 35, 36, 37, protocol::MAX_VERSION + 1] {
        assert!(encode(g, &expected[116..], version).is_err());
        assert!(read_reply(
            &mut response(&expected, &data).as_slice(),
            g,
            &expected,
            version
        )
        .is_err());
    }
}
#[test]
fn framing_truncation_reserved_echo_and_lengths_fail_closed() {
    let (g, expected, data) = fixture();
    let bytes = response(&expected, &data);
    for n in 0..bytes.len() {
        assert!(parse(g, &expected, &bytes[..n]).is_err(), "truncation {n}");
    }
    for offset in [
        0,
        8,
        12,
        14,
        16,
        28,
        36,
        40,
        64,
        72,
        80,
        88,
        112,
        120,
        124,
        expected.len() + 3,
        expected.len() + 24,
        expected.len() + 28,
    ] {
        let mut bad = bytes.clone();
        bad[offset] ^= 1;
        assert!(parse(g, &expected, &bad).is_err(), "offset {offset}");
    }
    for offset in [0, 1, 2, 12, 13, 14, 15] {
        let mut bad = bytes.clone();
        bad[expected.len() + offset] = 255;
        assert!(parse(g, &expected, &bad).is_err(), "metadata {offset}");
    }
    let mut coalesced = bytes.clone();
    coalesced.extend(&bytes);
    let mut input = coalesced.as_slice();
    assert_eq!(
        read_reply(&mut input, g, &expected, VERSION).unwrap().data,
        data
    );
    assert_eq!(input, bytes);
    read_reply(&mut input, g, &expected, VERSION).unwrap();
    assert!(input.is_empty());
}
#[test]
fn invalid_headers_are_rejected_before_reading_or_allocating_body() {
    struct HeaderOnly {
        bytes: std::io::Cursor<Vec<u8>>,
    }
    impl Read for HeaderOnly {
        fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
            assert!(
                self.bytes.position() < 16,
                "invalid header must never read a body"
            );
            self.bytes.read(b)
        }
    }
    let (g, expected, data) = fixture();
    for len in [
        0,
        16,
        MIN_RESULT_BYTES as u32 - 1,
        MAX_RESULT_BYTES as u32 + 1,
        u32::MAX,
    ] {
        let mut bytes = response(&expected, &data)[..16].to_vec();
        bytes[4..8].copy_from_slice(&len.to_be_bytes());
        let mut reader = HeaderOnly {
            bytes: std::io::Cursor::new(bytes),
        };
        assert!(read_reply(&mut reader, g, &expected, VERSION).is_err());
    }
}
#[test]
fn canonical_scope_ack_and_lifecycle_are_one_consistent_receipt() {
    let (g, expected, data) = fixture();
    for change in 0..10 {
        let mut bad = data.clone();
        match change {
            0 => bad.personal.player.source += 1,
            1 => bad.personal.player.object += 1,
            2 => bad.personal.player.epoch += 1,
            3 => bad.personal.player.channel += 1,
            4 => bad.world.identity.match_id += 1,
            5 => bad.personal.entity = 2,
            6 => bad.actors.0[0].last_move_sequence = 0,
            7 => bad.actors.0[0].last_move_sequence = 2,
            8 => bad.world.revision = 2,
            _ => bad.presentation = None,
        }
        assert!(
            parse(g, &expected, &response(&expected, &bad)).is_err(),
            "scope/ack {change}"
        );
    }
    for (offset, value) in [
        (1, 0),
        (1, 2),
        (1, 3),
        (2, 3),
        (12, 1),
        (13, 4),
        (14, 2),
        (15, 2),
        (23, 1),
    ] {
        let mut bytes = response(&expected, &data);
        bytes[expected.len() + offset] = value;
        assert!(
            parse(g, &expected, &bytes).is_err(),
            "lifecycle/status {offset}"
        );
    }
    let mut other_life = g;
    other_life.stamp.life += 1;
    let other_request = encode(other_life, &expected[116..], VERSION).unwrap();
    assert!(parse(other_life, &other_request, &response(&other_request, &data)).is_err());
}
#[test]
fn denial_and_stale_life_are_not_fabricated_success_or_sequence_ack() {
    let (g, expected, data) = fixture();
    let mut bytes = response(&expected, &data);
    bytes[expected.len() + 1] = Disposition::Denied as u8;
    bytes[expected.len() + 2] = Reason::TooFast as u8;
    let denied = parse(g, &expected, &bytes).unwrap();
    assert_eq!(denied.rejection, action::Rejection::None);
    assert_eq!(denied.reason, Reason::TooFast);
    let mut stale = g;
    stale.stamp.life += 1;
    let expected = encode(stale, &expected[116..], VERSION).unwrap();
    let mut current = data.clone();
    current.actors.0[0].last_move_sequence = 0;
    let mut bytes = response(&expected, &current);
    bytes[expected.len()] = 2;
    bytes[expected.len() + 1] = Disposition::Rejected as u8;
    let rejected = parse(stale, &expected, &bytes).unwrap();
    assert_eq!(rejected.rejection, action::Rejection::StaleLife);
    assert_eq!(rejected.data.actors.0[0].last_move_sequence, 0);
    assert_eq!(
        denied.data, data,
        "a later recovery cannot mutate an earlier receipt"
    );
}

#[test]
fn life_rejection_must_describe_the_returned_actor_and_service_phase() {
    let (g, expected, data) = fixture();
    for rejection in [1, 2, 3, 4] {
        let mut bytes = response(&expected, &data);
        bytes[expected.len()] = rejection;
        bytes[expected.len() + 1] = Disposition::Rejected as u8;
        assert!(
            parse(g, &expected, &bytes).is_err(),
            "false life rejection {rejection}"
        );
    }
}
