use lito_modern::modern_lito::native_login::{self, LoginController, Packet};
use std::net::Ipv4Addr;
fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn oracle() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("fixtures/native-login-current.json")).unwrap()
}
#[test]
fn chinese_credentials_and_source_match_deployed_java_ms950_parser() {
    let oracle = oracle();
    assert_eq!(
        native_login::login_request("重建帳號", "測試密碼", Ipv4Addr::LOCALHOST)
            .unwrap()
            .as_slice(),
        hex(oracle["login"].as_str().unwrap())
    );
    assert_eq!(
        native_login::source_request("測試乙").unwrap().as_slice(),
        hex(oracle["source_request"].as_str().unwrap())
    );
    for text in ["plain", "測試乙", "密碼１２３"] {
        let encoded = native_login::encode_text(text, 63).unwrap();
        assert_eq!(native_login::decode_text(&encoded).unwrap(), text);
    }
    for text in ["", "bad\0name", "bad\nname", "test😀"] {
        assert!(native_login::encode_text(text, 63).is_err());
    }
    assert!(native_login::login_request(&"a".repeat(65), "password", Ipv4Addr::LOCALHOST).is_err());
    assert!(
        native_login::login_request("account", &"密".repeat(129), Ipv4Addr::LOCALHOST).is_err()
    );
    assert!(native_login::source_request(&"密".repeat(32)).is_err());
}
#[test]
fn deployed_login_replies_retain_full_failure_body_and_unsigned_endpoints() {
    let oracle = oracle();
    for (name, value) in oracle["replies"].as_object().unwrap() {
        let bytes = hex(value.as_str().unwrap());
        let parsed = native_login::parse(&bytes).unwrap();
        match name.as_str() {
            "sources" => match parsed {
                Packet::Sources(rows) => {
                    assert_eq!(rows.len(), 2);
                    assert_eq!(rows[1].name, "測試乙");
                    assert_eq!(rows[1].endpoint, "127.0.0.2:32100".parse().unwrap());
                }
                _ => panic!("sources"),
            },
            "redirect" => assert_eq!(
                parsed,
                Packet::Redirect {
                    index: 0,
                    endpoint: "127.0.0.2:32100".parse().unwrap()
                }
            ),
            "login_ok" => assert_eq!(parsed, Packet::LoginOk),
            _ => match parsed {
                Packet::LoginCheck(check) => {
                    assert_eq!(check.server_name, "LiTo 重建測試");
                    assert_eq!(check.value, 3600);
                    assert_eq!(
                        check.result,
                        match name.as_str() {
                            "ACCEPTED" => 0,
                            "SYSTEM_ERROR" => 5,
                            "ACCESS_FAILED" => 8,
                            "ACCOUNT_BANNED" => 0x1a,
                            "ACCOUNT_IN_USE" => 0x16,
                            _ => panic!(),
                        }
                    );
                }
                _ => panic!("login check"),
            },
        }
        for n in 0..bytes.len() {
            assert!(
                native_login::parse(&bytes[..n]).is_err(),
                "accepted truncated {name} at {n}"
            );
        }
        let mut tail = bytes.clone();
        tail.push(0);
        assert!(native_login::parse(&tail).is_err());
    }
    assert!(native_login::parse(&[0, 8]).is_err());
    assert!(native_login::parse(&[0x3b, 128]).is_err());
    let mut bad = hex(oracle["replies"]["redirect"].as_str().unwrap());
    bad[1] = 128;
    assert!(native_login::parse(&bad).is_err());
}
#[test]
fn credentials_and_selection_require_current_live_state() {
    let mut c = LoginController::default();
    assert!(c.login("重建帳號", "測試密碼").is_err());
    assert!(c.select_source(0).is_err());
    assert!(c.begin_game().is_err());
    assert!(c.acknowledge_notice().is_err());
    assert!(c.select_character(0).is_err());
    let state = serde_json::to_string(c.snapshot()).unwrap();
    assert!(!state.contains("測試密碼"));
    assert!(c.is_quiescent());
}
