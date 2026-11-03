use lito_modern::modern_lito::{
    native_login::LoginController,
    native_room::{self, Packet},
};
fn fixture() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("fixtures/native-room-current.json")).unwrap()
}
fn bytes(v: &serde_json::Value) -> Vec<u8> {
    let s = v.as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn room_envelopes_match_deployed_jar_and_reject_partial_or_trailing_fields() {
    for (name, value) in fixture()["packets"].as_object().unwrap() {
        if name == "join_request" {
            continue;
        }
        let b = bytes(value);
        assert!(native_room::parse(&b).is_ok(), "{name}");
        for n in 0..b.len() {
            assert!(native_room::parse(&b[..n]).is_err(), "{name} truncated {n}");
        }
        let mut tail = b;
        tail.push(0);
        assert!(native_room::parse(&tail).is_err(), "{name} trailing");
    }
    assert!(native_room::parse(&[0xce, 2]).is_err());
    assert!(native_room::parse(&[0xd1, b'a', 0, 3]).is_err());
}
#[test]
fn joined_room_preserves_settings_but_never_serializes_password() {
    let f = fixture();
    let Packet::Joined(room) = native_room::parse(&bytes(&f["packets"]["joined"])).unwrap() else {
        panic!()
    };
    assert_eq!(
        (room.id, room.map_id, room.users_summary, room.capacity),
        (0xfedcba98, 32768, 2, 8)
    );
    assert_eq!(room.name, "測試房間");
    assert_eq!(room.owner, "測試房主");
    assert_eq!(room.summary, [1, 2, 3, 4, 5]);
    assert_eq!(room.options, [1, 2, 3, 4]);
    assert_eq!(room.selections, [5, 6]);
    assert!(room.password_required);
    assert!(!serde_json::to_string(&room).unwrap().contains("密碼"));
    assert!(room.members.is_empty());
    assert_eq!(
        native_room::join(room.id, "密碼").unwrap().as_slice(),
        bytes(&f["packets"]["join_request"])
    );
    assert!(native_room::join(room.id, "密碼長").is_err());
    assert!(native_room::join(0, "").is_err());
}
#[test]
fn participant_ready_two_and_signed_selector_are_authoritative_incremental_fields() {
    let f = fixture();
    let Packet::Joined(mut room) = native_room::parse(&bytes(&f["packets"]["joined"])).unwrap()
    else {
        panic!()
    };
    for state in 0..=2 {
        let Packet::Member { room: id, member } =
            native_room::parse(&bytes(&f["packets"][format!("member_{state}")])).unwrap()
        else {
            panic!()
        };
        assert_eq!(id, room.id);
        assert_eq!(
            (
                member.state,
                member.preview_state,
                member.channel_selector,
                member.rank,
                member.grade
            ),
            (state, state, -1, i32::MAX as u32, 14)
        );
        assert_eq!(
            (
                member.matches,
                member.wins,
                member.losses,
                member.disconnects
            ),
            (10, 6, 4, 1)
        );
        assert_eq!(member.affiliation, "測試盟");
        native_room::upsert(&mut room, member).unwrap();
        assert_eq!(room.members.len(), 1);
        assert_eq!(room.members[0].state, state);
    }
    assert_eq!(
        native_room::parse(&bytes(&f["packets"]["countdown"])).unwrap(),
        Packet::Countdown(255)
    );
}
#[test]
fn room_actions_require_current_server_lobby_and_membership() {
    let mut c = LoginController::default();
    assert!(c.join_room(0, "").is_err());
    assert!(c.set_ready(true).is_err());
    assert!(c.leave_room().is_err());
    assert!(c.is_quiescent());
}
