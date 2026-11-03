//! Explicit bounded pre-authentication probe. Reads exactly one profile,
//! attempts its ordinary verified TLS/HELLO once, then closes. No credentials,
//! source queries, assets, windows, preferences writes or protocol fallback.
use super::{connect_profile, Cancellation, ModernProfile, SessionController};
use crate::modern_lito::{
    profile::ProfileError,
    transport::{Stage, TlsVersion, TransportError},
};
use serde_json::{json, Value};
use std::{path::Path, time::Instant};

fn document() -> Value {
    json!({
        "schema": 1,
        "operation": "profile_probe",
        "success": false,
        "stage": "profile",
        "configured_protocol": null,
        "configured_extension": null,
        "negotiated": null,
        "profile_validated": false,
        "verified_tls_and_protocol": false,
        "connection_closed": false,
        "not_checked": ["authentication", "source_databases", "shared_resources", "gameplay"],
        "error": null
    })
}

fn fail(report: &mut Value, stage: &str, code: &str, guidance: &str) {
    report["stage"] = json!(stage);
    report["error"] = json!({"code": code, "guidance": guidance});
}

pub fn invalid_arguments() -> Value {
    let mut report = document();
    fail(
        &mut report,
        "arguments",
        "invalid_arguments",
        "使用 LiToModern.exe --probe-profile <設定檔絕對路徑>；不接受帳密或其他參數。",
    );
    report
}

pub fn run(path: &Path) -> Value {
    match ModernProfile::load(path) {
        Ok(profile) => probe(&profile, Cancellation::new()),
        Err(error) => {
            let mut report = document();
            let code = match error {
                ProfileError::TooLarge => "profile_too_large",
                ProfileError::InvalidDocument => "invalid_document",
                ProfileError::UnsupportedSchema => "unsupported_schema",
                ProfileError::InvalidIdentity => "invalid_identity",
                ProfileError::InvalidEndpoint => "invalid_endpoint",
                ProfileError::InvalidServerName => "invalid_server_name",
                ProfileError::InvalidTimeout => "invalid_timeout",
                ProfileError::InvalidTrust => "invalid_trust",
                ProfileError::UnsafeResourceRoot => "unsafe_resource_root",
                ProfileError::UnsafeEffectCatalog => "unsafe_effect_catalog",
                ProfileError::UnsafeProfilePath => "unsafe_profile_path",
                ProfileError::ReadFailed => "profile_read_failed",
            };
            fail(
                &mut report,
                "profile",
                code,
                "檢查設定格式、絕對路徑與讀取權限；設定不可含帳密、私人金鑰或資料庫密碼。",
            );
            report
        }
    }
}

fn transport_failure(report: &mut Value, error: &TransportError) {
    use crate::modern_lito::protocol::ErrorKind;
    let (stage, code, guidance) = match error {
        TransportError::Cancelled => ("connection", "cancelled", "連線檢查已取消；未登入。"),
        TransportError::DeadlineExceeded(stage) => (
            match stage {
                Stage::Connect => "connect",
                Stage::TlsHandshake => "tls",
                Stage::Negotiation => "negotiation",
                Stage::Close => "close",
                _ => "connection",
            },
            "deadline_exceeded",
            "已達設定的連線期限；檢查端點與伺服器狀態，未自動重試。",
        ),
        TransportError::Tls(rustls::Error::InvalidCertificate(_)) => (
            "tls",
            "certificate_rejected",
            "確認憑證有效期、伺服器名稱與公開 CA 配對；不可停用憑證驗證。",
        ),
        TransportError::Tls(_) => (
            "tls",
            "tls_rejected",
            "確認端點使用相容 TLS，而非原生 LiTo 連接埠。",
        ),
        TransportError::InvalidConfiguration(_) => (
            "configuration",
            "invalid_tls_configuration",
            "檢查公開 DER CA、TLS 名稱與逾時設定；不接受驗證繞過。",
        ),
        TransportError::Protocol(error) => (
            "negotiation",
            match error.kind {
                ErrorKind::UnsupportedVersion => "unsupported_version",
                ErrorKind::UnsupportedCapability => "unsupported_capability",
                ErrorKind::UnsupportedSchema => "unsupported_numeric_schema",
                _ => "protocol_rejected",
            },
            "確認伺服器版本及明確啟用的擴充與設定相符；未降級或重試。",
        ),
        TransportError::Io(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => (
            "connect",
            "connection_refused",
            "端點拒絕連線；確認 Modern 服務已啟動且端口正確。",
        ),
        TransportError::Io(_) => (
            "connection",
            "io_error",
            "連線未完成；檢查網路與伺服器紀錄。",
        ),
        _ => (
            "connection",
            "connection_rejected",
            "連線未通過完整檢查；未登入或重試。",
        ),
    };
    // Never serialize external errors, their causes, paths, peer text or values.
    fail(report, stage, code, guidance);
}

pub(crate) fn probe(profile: &ModernProfile, cancel: Cancellation) -> Value {
    let started = Instant::now();
    let mut report = document();
    report["profile_validated"] = json!(true);
    report["configured_protocol"] = json!(profile.gameplay_protocol().as_str());
    report["configured_extension"] = json!(profile.gameplay_extension().map(|e| e.as_str()));
    // Construction has no I/O. Reuse the ordinary player's exact contract,
    // including required capabilities; not a relaxed diagnostic-only HELLO.
    let owner = SessionController::new(profile.clone());
    match connect_profile(profile, owner.handshake, owner.extension_offer, cancel) {
        Err(error) => transport_failure(&mut report, &error),
        Ok(mut connection) => {
            if let Some(ack) = connection.ack() {
                report["verified_tls_and_protocol"] = json!(true);
                report["negotiated"] = json!({
                    "version": ack.version,
                    "numeric_schema": ack.numeric_schema,
                    "capabilities": ack.capabilities,
                    "extensions": connection.negotiated_extensions(),
                    "tls": match connection.tls_version() { TlsVersion::Tls12 => "TLS1.2", TlsVersion::Tls13 => "TLS1.3" }
                });
                // A bounded TLS close, never AUTH/PING/roster or a gameplay ACK.
                match connection.close() {
                    Ok(()) => {
                        report["success"] = json!(true);
                        report["stage"] = json!("complete");
                    }
                    Err(error) => {
                        transport_failure(&mut report, &error);
                        report["stage"] = json!("close");
                    }
                }
            } else {
                fail(
                    &mut report,
                    "negotiation",
                    "missing_ack",
                    "缺少已驗證握手；未登入。",
                );
            }
            // Drop always retires the owned socket, including close failure.
            drop(connection);
            report["connection_closed"] = json!(true);
        }
    }
    report["elapsed_ms"] = json!(started.elapsed().as_millis() as u64);
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argument_and_file_failures_are_noninteractive_redacted_and_not_success() {
        for result in [
            invalid_arguments(),
            run(Path::new("relative-private-secret.json")),
        ] {
            assert_eq!(result["success"], false);
            assert_eq!(result["profile_validated"], false);
            assert_eq!(result["verified_tls_and_protocol"], false);
            assert!(result["negotiated"].is_null());
            assert!(!result.to_string().contains("private-secret"));
            assert_eq!(result["not_checked"].as_array().unwrap().len(), 4);
        }
    }

    #[test]
    fn transport_reports_do_not_echo_secret_error_details_or_claim_authentication() {
        for error in [
            TransportError::Io(std::io::Error::other("private-secret-password")),
            TransportError::InvalidConfiguration("private-secret-key"),
            TransportError::Tls(rustls::Error::General("private-secret-peer".into())),
            TransportError::DeadlineExceeded(Stage::Negotiation),
            TransportError::Cancelled,
        ] {
            let mut report = document();
            transport_failure(&mut report, &error);
            assert_eq!(report["success"], false);
            assert!(!report.to_string().contains("private-secret"));
            assert!(!report["error"]["code"].as_str().unwrap().is_empty());
        }
    }

    #[test]
    fn published_protocol_and_extension_names_roundtrip_the_actual_profile_enum() {
        use crate::modern_lito::profile::{GameplayExtension, GameplayProtocol};
        for p in [GameplayProtocol::LegacyV37, GameplayProtocol::ModernV47] {
            assert_eq!(
                serde_json::from_value::<GameplayProtocol>(json!(p.as_str())).unwrap(),
                p
            );
        }
        let mut names = std::collections::BTreeSet::new();
        for e in GameplayExtension::ALL {
            assert!(names.insert(e.as_str()));
            assert_eq!(
                serde_json::from_value::<GameplayExtension>(json!(e.as_str())).unwrap(),
                e
            );
        }
        assert_eq!(names.len(), 7);
    }
}
