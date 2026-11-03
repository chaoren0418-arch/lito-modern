use super::*;
use crate::modern_lito::{profile::ModernProfile, session::profile_probe};
use base64::Engine;

fn profile(peer: &JavaPeer, extension: Option<&str>, legacy: bool) -> ModernProfile {
    let mut doc = serde_json::json!({
        "schema": if legacy { 1 } else { 2 },
        "profile_id": "probe-test",
        "display_name": "private-secret-must-not-be-printed",
        "shared_resource_root": "D:/never-opened-private-probe-resources",
        "endpoint": peer.endpoint.to_string(),
        "tls_server_name": "localhost",
        "trust": {"kind": "private_ca", "certificate_der_base64": base64::engine::general_purpose::STANDARD.encode(private_root())},
        "connect_timeout_ms": 1000,
        "exchange_timeout_ms": 2000
    });
    if !legacy {
        doc["gameplay_protocol"] = serde_json::json!("modern_v47");
    }
    if let Some(e) = extension {
        doc["gameplay_extension"] = serde_json::json!(e);
    }
    ModernProfile::parse(&serde_json::to_vec(&doc).unwrap()).unwrap()
}

fn assert_scope(report: &serde_json::Value) {
    assert_eq!(report["operation"], "profile_probe");
    assert_eq!(
        report["not_checked"],
        serde_json::json!([
            "authentication",
            "source_databases",
            "shared_resources",
            "gameplay"
        ])
    );
    assert!(!report.to_string().contains("private-secret"));
    assert!(!report.to_string().contains("never-opened"));
    assert!(!report.to_string().contains("certificate_der_base64"));
}

#[test]
#[ignore = "actual Java TLS: ordinary v37/v47 and all five extensions, close after HELLO without AUTH or resource/SQL access"]
fn java_profile_probe_ordinary_contracts_all_extensions_and_tls12_close_without_auth() {
    let cases = [
        (None, true, "full"),
        (None, false, "full"),
        (None, false, "tls12"),
        (Some("player_healing_v1"), false, "full"),
        (Some("armor_protection_v1"), false, "full"),
        (Some("player_healing_armor_v1"), false, "full"),
        (Some("armor_presentation_v1"), false, "full"),
        (Some("player_healing_armor_presentation_v1"), false, "full"),
    ];
    for (extension, legacy, mode) in cases {
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernProfileProbeTestPeer",
            mode,
            "5000",
        );
        let report = profile_probe::probe(&profile(&peer, extension, legacy), Cancellation::new());
        assert_scope(&report);
        assert_eq!(report["success"], true, "{report}");
        assert_eq!(report["profile_validated"], true);
        assert_eq!(report["verified_tls_and_protocol"], true);
        assert_eq!(report["connection_closed"], true);
        assert_eq!(
            report["negotiated"]["version"],
            if legacy { 37 } else { 47 }
        );
        assert_eq!(report["configured_extension"], serde_json::json!(extension));
        let bits = match extension {
            None => 0,
            Some("player_healing_v1") => 3,
            Some("armor_protection_v1") => 4,
            Some("player_healing_armor_v1") => 7,
            Some("armor_presentation_v1") => 12,
            Some(_) => 15,
        };
        assert_eq!(report["negotiated"]["extensions"], bits);
        if mode == "tls12" {
            assert_eq!(report["negotiated"]["tls"], "TLS1.2");
        }
        peer.expect_successful_exit();
        eprintln!("PROFILE_PROBE_POSITIVE legacy={legacy} extension={extension:?} mode={mode} no_auth=true");
    }
}

#[test]
#[ignore = "actual Java TLS: deny old server/missing extension, wrong CA/DNS/expired/plaintext and bounded stalls without fallback or AUTH"]
fn java_profile_probe_rejects_incompatible_trust_protocol_and_deadlines_without_fallback() {
    for (mode, extension) in [
        ("legacy", None),
        ("no-extensions", Some("armor_presentation_v1")),
    ] {
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernProfileProbeTestPeer",
            mode,
            "5000",
        );
        let report = profile_probe::probe(&profile(&peer, extension, false), Cancellation::new());
        assert_scope(&report);
        assert_eq!(report["success"], false);
        assert!(report["negotiated"].is_null());
        assert_eq!(report["stage"], "negotiation");
        peer.expect_successful_exit();
    }
    for (mode, wrong_name, no_ca) in [
        ("tls", true, false),
        ("tls", false, true),
        ("expired", false, false),
        ("plain", false, false),
    ] {
        let peer = JavaPeer::start(mode);
        let mut doc: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../config/profile.example.json")).unwrap();
        doc["schema"] = serde_json::json!(2);
        doc["gameplay_protocol"] = serde_json::json!("modern_v47");
        doc["endpoint"] = serde_json::json!(peer.endpoint.to_string());
        doc["tls_server_name"] = serde_json::json!(if wrong_name {
            "wrong.invalid"
        } else {
            "localhost"
        });
        doc["trust"] = if no_ca {
            serde_json::json!({"kind":"web_pki"})
        } else {
            serde_json::json!({"kind":"private_ca","certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())})
        };
        let report = profile_probe::probe(
            &ModernProfile::parse(&serde_json::to_vec(&doc).unwrap()).unwrap(),
            Cancellation::new(),
        );
        assert_scope(&report);
        assert_eq!(report["success"], false);
        assert_eq!(
            report["error"]["code"],
            if mode == "plain" {
                "tls_rejected"
            } else {
                "certificate_rejected"
            }
        );
    }
    let mut peer = JavaPeer::start_class(
        "local.lito381.server.ModernProfileProbeTestPeer",
        "stall-ack",
        "1500",
    );
    let mut doc: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../config/profile.example.json")).unwrap();
    doc["schema"] = serde_json::json!(2);
    doc["gameplay_protocol"] = serde_json::json!("modern_v47");
    doc["endpoint"] = serde_json::json!(peer.endpoint.to_string());
    doc["trust"] = serde_json::json!({"kind":"private_ca","certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())});
    doc["connect_timeout_ms"] = serde_json::json!(100);
    doc["exchange_timeout_ms"] = serde_json::json!(500);
    let profile = ModernProfile::parse(&serde_json::to_vec(&doc).unwrap()).unwrap();
    let started = Instant::now();
    let report = profile_probe::probe(&profile, Cancellation::new());
    assert_eq!(report["success"], false);
    assert_eq!(report["error"]["code"], "deadline_exceeded");
    assert_eq!(report["stage"], "negotiation");
    assert!(started.elapsed() < Duration::from_secs(2));
    peer.expect_successful_exit();
    let cancel = Cancellation::new();
    cancel.cancel();
    let report = profile_probe::probe(&profile, cancel);
    assert_eq!(report["error"]["code"], "cancelled");
    assert_scope(&report);
}
