use lito_modern::modern_lito::native_game::{self, Directory, Notice, Packet};
fn packet(key: &str) -> Vec<u8> {
    let fixture: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-game-current.json")).unwrap();
    let s = fixture["packets"][key].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn deployed_character_row_keeps_signed_fields_rank_order_and_original_name() {
    let Packet::Row(row) = native_game::parse(&packet("row")).unwrap() else {
        panic!()
    };
    let f = row.fields();
    assert_eq!(
        (
            f.class,
            f.sex,
            f.lawful,
            f.max_hp,
            f.max_mp,
            f.armor_class,
            f.level
        ),
        (6, 1, -32768, 32767, 30000, -128, 127)
    );
    assert_eq!(f.native_time, u32::MAX);
    assert_eq!(
        (
            f.ranks.overall,
            f.ranks.level,
            f.ranks.classic_source,
            f.ranks.channel,
            f.ranks.race
        ),
        (1, 4, 2, 3, 5)
    );
    assert_eq!(
        (
            f.movements.overall,
            f.movements.level,
            f.movements.classic_source,
            f.movements.channel,
            f.movements.race
        ),
        (-1, 4, 2, -3, -5)
    );
    assert_eq!(
        (f.disconnects, f.win_rate_denominator, f.wins, f.losses),
        (2, 10, 6, 4)
    );
    let mut d = Directory::default();
    d.start(1).unwrap();
    d.push(row).unwrap();
    let view = d.view().unwrap();
    assert_eq!(view[0].name, "重建角色");
    assert_eq!(view[0].clan, "測試盟");
    assert_eq!(d.select(0).unwrap().as_slice(), packet("select"));
    assert!(d.select(1).is_err());
}
#[test]
fn deployed_game_envelopes_reject_truncation_trailing_bytes_and_invalid_projection() {
    for key in [
        "count",
        "empty",
        "row",
        "ready",
        "notice438",
        "notice45",
        "news",
        "ACCEPTED",
        "SILENT_REJECTED",
        "PARTICIPATION_RESTRICTED",
        "INVALID_CHARACTER_NAME",
    ] {
        let b = packet(key);
        assert!(native_game::parse(&b).is_ok(), "{key}");
        for n in 0..b.len() {
            assert!(native_game::parse(&b[..n]).is_err(), "{key} at {n}");
        }
        let mut tail = b;
        tail.push(0);
        assert!(native_game::parse(&tail).is_err(), "{key}");
    }
    assert!(native_game::parse(&[3, 9]).is_err());
    assert!(native_game::parse(&[0xc8, 2]).is_err());
    assert!(native_game::parse(&[0x4b, 0, 128]).is_err());
    let row = packet("row");
    let suffix = row.len() - 79;
    for (offset, value) in [
        (0, 7),
        (1, 2),
        (5, 128),
        (9, 0),
        (10, 128),
        (16, 1),
        (57, 5),
        (78, 0),
    ] {
        let mut b = row.clone();
        b[suffix + offset] = value;
        assert!(native_game::parse(&b).is_err(), "offset {offset}");
    }
    let Packet::News(text) = native_game::parse(&packet("news")).unwrap() else {
        panic!()
    };
    assert_eq!(text, "請選擇角色。\n重建測試。");
    assert!(native_game::parse(&[0x38, b'a', 0, b'b', 0]).is_err());
    assert!(native_game::parse(&[0x38, 0x1b, 0]).is_err());
}
#[test]
fn complete_directory_is_atomic_bounded_and_rejects_duplicate_names() {
    let Packet::Row(row) = native_game::parse(&packet("row")).unwrap() else {
        panic!()
    };
    let mut d = Directory::default();
    assert!(d.push(row.clone()).is_err());
    assert!(d.view().is_err());
    assert!(d.start(9).is_err());
    d.start(2).unwrap();
    assert!(d.start(2).is_err());
    d.push(row.clone()).unwrap();
    assert!(!d.complete());
    assert!(d.view().is_err());
    assert!(d.select(0).is_err());
    assert!(d.push(row).is_err());
    let mut empty = Directory::default();
    empty.start(0).unwrap();
    assert!(empty.complete());
    assert!(empty.view().unwrap().is_empty());
    assert!(empty.select(0).is_err());
    let mut full = Directory::default();
    full.start(8).unwrap();
    for i in 0..8 {
        let b = packet("row");
        let end = b[1..].iter().position(|v| *v == 0).unwrap() + 1;
        let mut renamed = vec![0xcb, b'A' + i, 0];
        renamed.extend_from_slice(&b[end + 1..]);
        let Packet::Row(row) = native_game::parse(&renamed).unwrap() else {
            panic!()
        };
        full.push(row).unwrap();
    }
    assert_eq!(full.view().unwrap().len(), 8);
    assert_eq!(full.select(7).unwrap().as_slice(), [0xc9, b'H', 0]);
    let Packet::Row(row) = native_game::parse(&packet("row")).unwrap() else {
        panic!()
    };
    assert!(full.push(row).is_err());
}
#[test]
fn notice_acknowledgements_match_deployed_parsers() {
    for (id, expected) in [
        (Some(438), Some(0x61)),
        (Some(45), Some(0x46)),
        (None, None),
    ] {
        assert_eq!(
            Notice {
                message_id: id,
                text: None
            }
            .acknowledgement(),
            expected
        );
    }
    assert_eq!(native_game::parse(&packet("ready")).unwrap(), Packet::Ready);
}
