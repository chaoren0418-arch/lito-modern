use super::*;
use crate::modern_lito::{application_protocol as app, battle_read as read, protocol};
fn request() -> read::Request {
    read::Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: 1u64 << 63,
        },
        channel_id: 0,
        world: data::WorldId {
            room: 101,
            match_id: 102,
            generation: 103,
        },
    }
}
fn response(state: u8, payload: &[u8]) -> Vec<u8> {
    let mut b = read::encode_request(7, &request()).unwrap();
    b[4..8].copy_from_slice(&((read::MIN_RESULT_BYTES + payload.len()) as u32).to_be_bytes());
    b[12..14].copy_from_slice(&read::RESULT_KIND.to_be_bytes());
    b.extend([state, 0, 0, 0]);
    b.extend((payload.len() as u32).to_be_bytes());
    b.extend(payload);
    b
}
fn parse(b: &[u8]) -> Result<read::UnvalidatedReply, app::ApplicationError> {
    match app::decode_response_version(b, 35)?.response {
        app::Response::BattleData(r) => Ok(r),
        _ => panic!("wrong response kind"),
    }
}
#[test]
fn schema_two_requires_v36_and_cannot_downgrade_or_upgrade_silently() {
    let mut d = data_tests::fixture();
    let one = response(1, &encode(&d).unwrap());
    d.presentation = Some(data::List(
        d.actors
            .0
            .iter()
            .map(|a| data::PublicPresentation {
                entity: a.entity,
                shot: None,
                item: None,
            })
            .collect(),
    ));
    let encoded = encode(&d).unwrap();
    assert_eq!(inspect(&encoded).unwrap().schema(), PRESENTATION_SCHEMA);
    let two = response(1, &encoded);
    assert!(app::decode_response_version(&one, 35).is_ok());
    assert!(app::decode_response_version(&two, 36).is_ok());
    assert!(app::decode_response_version(&one, 36).is_err());
    assert!(app::decode_response_version(&two, 35).is_err());
    for n in 0..encoded.len() {
        assert!(inspect(&encoded[..n]).is_err());
    }
    let mut wrong = encoded.clone();
    wrong[4..6].copy_from_slice(&SCHEMA.to_be_bytes());
    assert!(inspect(&wrong).is_err());
    let mut wrong = encoded.clone();
    wrong[12..14].copy_from_slice(&12u16.to_be_bytes());
    assert!(inspect(&wrong).is_err());
    let mut wrong = encoded;
    wrong.push(0);
    let n = wrong.len() as u32;
    wrong[8..12].copy_from_slice(&n.to_be_bytes());
    assert!(inspect(&wrong).is_err());
}
#[test]
fn request_bytes_pin_kind_id_epoch_channel_and_unshifted_world() {
    let b = read::encode_request(u32::MAX, &request()).unwrap();
    assert_eq!(b.len(), 64);
    assert_eq!(
        &b[..16],
        &[76, 84, 77, 49, 0, 0, 0, 64, 255, 255, 255, 255, 0, 60, 0, 0]
    );
    assert_eq!(
        &b[16..40],
        &[0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 13, 5, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        &b[40..],
        &[0, 0, 0, 0, 0, 0, 0, 101, 0, 0, 0, 0, 0, 0, 0, 102, 0, 0, 0, 0, 0, 0, 0, 103]
    );
    assert_eq!(format!("{:?}", request()), "BattleReadRequest[redacted]");
    for field in 0..7 {
        let mut r = request();
        match field {
            0 => r.context.key.source_id = u32::MAX,
            1 => r.context.key.object_id = u64::MAX,
            2 => r.context.epoch = 0,
            3 => r.channel_id = 4,
            4 => r.world.room = 0,
            5 => r.world.match_id = -1,
            _ => r.world.generation = i64::MIN,
        };
        assert!(read::encode_request(7, &r).is_err());
    }
    assert!(read::encode_request(0, &request()).is_err());
}
#[test]
fn service_states_are_separate_from_engine_phase_and_need_no_fake_payload() {
    for (tag, state) in [(0, read::State::Preparing), (2, read::State::Unavailable)] {
        let b = response(tag, &[]);
        let r = parse(&b).unwrap();
        assert_eq!(r.state, state);
        assert_eq!(r.echo, request());
        assert!(r.data.is_none());
        assert!(r.personal_view().unwrap().is_none());
        assert_eq!(format!("{r:?}"), "UnvalidatedBattleRead[redacted]");
    }
    for tag in [1, 3, 255] {
        assert!(parse(&response(tag, &[])).is_err());
    }
    let payload = encode(&data_tests::fixture()).unwrap();
    for tag in [0, 2] {
        assert!(parse(&response(tag, &payload)).is_err());
    }
}
#[test]
fn data_binds_echo_and_preserves_the_real_engine_phase_and_revision() {
    let d = data_tests::fixture();
    let b = response(1, &encode(&d).unwrap());
    let r = parse(&b).unwrap();
    let private = r.personal_view().unwrap().unwrap();
    assert_eq!(private.player(), &request().player());
    assert_eq!(private.world(), &request().world);
    assert!(private.source().is_none());
    assert_eq!(r.data, Some(d));
    assert_eq!(r.state, read::State::Data);
    for at in [16, 20, 28, 39, 47, 55, 63, 65, 66, 67, 71] {
        let mut wrong = b.clone();
        wrong[at] ^= 1;
        assert!(parse(&wrong).is_err(), "echo/reserved/length offset {at}");
    }
    for v in 1..35 {
        assert!(app::decode_response_version(&b, v).is_err());
    }
    for n in 0..b.len() {
        assert!(parse(&b[..n]).is_err());
    }
    let mut extra = b.clone();
    extra.push(0);
    assert!(parse(&extra).is_err());
}
#[test]
fn checked_personal_input_revalidates_mutable_reply_and_preserves_owned_view() {
    let mut r = parse(&response(1, &encode(&data_tests::fixture()).unwrap())).unwrap();
    let private = r.personal_view().unwrap().unwrap();
    r.echo.context.epoch ^= 1;
    assert!(r.personal_view().is_err());
    assert_eq!(private.player(), &request().player());
    r.echo = request();
    r.state = read::State::Preparing;
    assert!(r.personal_view().is_err());
    r.state = read::State::Data;
    r.data = None;
    assert!(r.personal_view().is_err());
    r.state = read::State::Unavailable;
    assert!(r.personal_view().unwrap().is_none());
    // A scene payload has no spells feature. A well-shaped invented skill must
    // be rejected by the real response decoder, not merely by an optional UI.
    let mut d = data_tests::fixture();
    d.personal.skills.0.push(data::Skill {
        id: 5,
        name: data::Text("Teleport".into()),
        mp_cost: 10,
        reuse_millis: 10,
        support: data::Support::Teleport,
    });
    d.check_combat_values(&d.personal.player, &d.world.identity)
        .unwrap();
    assert!(parse(&response(1, &encode(&d).unwrap())).is_err());
}
#[test]
fn capability_version_and_new_kind_size_are_checked_before_body_allocation() {
    struct HeaderOnly {
        b: Vec<u8>,
        at: usize,
    }
    impl std::io::Read for HeaderOnly {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            assert!(self.at < 16, "unnegotiated body was read");
            let n = out.len().min(16 - self.at);
            out[..n].copy_from_slice(&self.b[self.at..self.at + n]);
            self.at += n;
            Ok(n)
        }
    }
    let mut b = response(1, &[]);
    b[4..8].copy_from_slice(&(read::MAX_FRAME_BYTES as u32).to_be_bytes());
    for (version, allowed) in [(34, true), (35, false)] {
        assert!(app::read_response_with_battle(
            &mut HeaderOnly {
                b: b[..16].to_vec(),
                at: 0
            },
            version,
            allowed
        )
        .is_err());
    }
    for (kind, length) in [
        (read::RESULT_KIND, read::MAX_FRAME_BYTES + 1),
        (4, app::MAX_FRAME_BYTES + 1),
    ] {
        let mut header = b[..16].to_vec();
        header[12..14].copy_from_slice(&kind.to_be_bytes());
        header[4..8].copy_from_slice(&(length as u32).to_be_bytes());
        assert!(
            app::read_response_with_battle(&mut HeaderOnly { b: header, at: 0 }, 35, true).is_err()
        );
    }
    let b = response(0, &[]);
    let mut coalesced = [b.clone(), b].concat().as_slice().to_vec();
    let mut cursor = std::io::Cursor::new(&mut coalesced);
    app::read_response_with_battle(&mut cursor, 35, true).unwrap();
    assert_eq!(cursor.position(), 72);
}
#[test]
fn battle_capability_cannot_imply_unnegotiated_map_or_combat_features() {
    let mut ack = protocol::Ack {
        request_id: 1,
        version: 35,
        numeric_schema: 1,
        capabilities: protocol::RUNTIME_CAPABILITIES | protocol::CAP_BATTLE_DATA,
    };
    let mut reply = parse(&response(1, &encode(&data_tests::fixture()).unwrap())).unwrap();
    assert!(read::reply_negotiated(&reply, &ack));
    ack.capabilities &= !protocol::CAP_WORLD;
    assert!(!read::reply_negotiated(&reply, &ack));
    ack.capabilities |= protocol::CAP_WORLD;
    reply.data.as_mut().unwrap().manifest.sectors.0 =
        vec![reply.data.as_ref().unwrap().manifest.sectors.0[0].clone(); 17];
    assert!(!read::reply_negotiated(&reply, &ack));
    ack.capabilities |= protocol::CAP_LARGE_MAPS;
    assert!(read::reply_negotiated(&reply, &ack));
    ack.version = 34;
    assert!(!read::reply_negotiated(&reply, &ack));
    assert_eq!(
        protocol::RUNTIME_CAPABILITIES & protocol::CAP_BATTLE_DATA,
        0
    );
    assert_eq!(
        protocol::RUNTIME_WORLD_VERSION,
        37,
        "normal controller admits the read-only schema-2 follower; diagnostic offer remains opt-in"
    );
}
