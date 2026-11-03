use lito_modern::modern_lito::{
    native_lobby::{self, Packet},
    native_login::LoginController,
};
fn fixture() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("fixtures/native-lobby-current.json")).unwrap()
}
fn bytes(v: &serde_json::Value) -> Vec<u8> {
    let s = v.as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn deployed_channel_results_are_exact_and_keep_full_unsigned_tails() {
    let f = fixture();
    assert_eq!(
        serde_json::json!(native_lobby::CHANNEL_NAMES),
        f["channel_names"]
    );
    for (key, value) in f["packets"].as_object().unwrap() {
        let b = bytes(value);
        if key.starts_with("request_") {
            let id = b[1];
            if id <= 3 {
                assert_eq!(native_lobby::enter(id).unwrap(), b);
            } else {
                assert!(native_lobby::enter(id).is_err());
            }
            continue;
        }
        assert!(native_lobby::parse(&b).is_ok(), "{key}");
        for n in 0..b.len() {
            assert!(native_lobby::parse(&b[..n]).is_err(), "{key} truncated {n}");
        }
        let mut tail = b;
        tail.push(0);
        assert!(native_lobby::parse(&tail).is_err(), "{key} trailing");
    }
    let mut b = bytes(&f["packets"]["success_3"]);
    b[3..13].fill(255);
    assert_eq!(
        native_lobby::parse(&b).unwrap(),
        Packet::Enter {
            result: 0,
            channel: 3,
            words: Some((u32::MAX, u16::MAX, u32::MAX))
        }
    );
    assert!(native_lobby::parse(&[0xc9, 10, 99]).is_err());
    assert!(native_lobby::parse(&[0xc9, 2, 0]).is_err());
    assert!(native_lobby::parse(&[0xca, 100, 0, 0, 0, 0]).is_err());
    let Packet::Channel(c) = native_lobby::parse(&bytes(&f["packets"]["population"])).unwrap()
    else {
        panic!()
    };
    assert_eq!((c.id, c.users, c.capacity), (3, 65535, 65535));
}
#[test]
fn deployed_chinese_room_row_is_bounded_and_updates_by_server_id() {
    let f = fixture();
    let Packet::Room(r) = native_lobby::parse(&bytes(&f["packets"]["room"])).unwrap() else {
        panic!()
    };
    assert_eq!(
        (r.id, r.users, r.capacity, r.attributes, r.option),
        (u32::MAX, 32768, 65535, 0x81, 255)
    );
    assert_eq!(r.name, "中文房間");
    let long = bytes(&f["packets"]["long_room"]);
    let Packet::Room(long_row) = native_lobby::parse(&long).unwrap() else {
        panic!()
    };
    assert_eq!(long_row.name.len(), 89);
    let mut oversized = long;
    oversized.insert(6, b'N');
    assert!(native_lobby::parse(&oversized).is_err());
    let mut rows = vec![];
    native_lobby::upsert(&mut rows, r.clone()).unwrap();
    let mut changed = r.clone();
    changed.users = 1;
    native_lobby::upsert(&mut rows, changed).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].users, 1);
    for id in 0..511 {
        let mut row = r.clone();
        row.id = id;
        native_lobby::upsert(&mut rows, row).unwrap();
    }
    let mut excess = r;
    excess.id = 1000;
    assert!(native_lobby::upsert(&mut rows, excess).is_err());
}
#[test]
fn lobby_actions_do_not_run_without_live_character_and_channel() {
    let mut c = LoginController::default();
    assert!(c.enter_channel(0).is_err());
    assert!(c.refresh_rooms().is_err());
    assert!(c.return_characters().is_err());
    assert!(c.is_quiescent());
}
