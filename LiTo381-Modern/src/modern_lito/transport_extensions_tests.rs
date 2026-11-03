mod extensions_interop {
    use super::*;
    use crate::modern_lito::protocol::{extensions as e, KNOWN_CAPABILITIES};
    fn offer() -> Hello {
        Hello {
            request_id: 0xfedcba98,
            min_version: 47,
            max_version: 47,
            supported_capabilities: KNOWN_CAPABILITIES,
            required_capabilities: e::DEPENDENCIES,
            numeric_schema: 1,
        }
    }
    #[test]
    #[ignore = "explicit compiled Java peer/private CA; actual TLS extension handshake"]
    fn java_extended_tls_negotiates_healing_without_auth_or_legacy_downgrade() {
        let root = private_root();
        for mode in [
            "commands",
            "tls12",
            "data",
            "fragmented",
            "denied",
            "legacy-ack",
            "unoffered",
        ] {
            let mut peer = JavaPeer::start_class(
                "local.lito381.server.ModernExtendedSessionTestPeer",
                mode,
                "5000",
            );
            let ext = if mode == "data" || mode == "unoffered" {
                e::Offer {
                    supported: 1,
                    required: 1,
                }
            } else {
                e::HEALING
            };
            let result = connect_and_negotiate_extended(
                peer.endpoint,
                "localhost",
                Some(&root),
                offer(),
                ext,
                integration_limits(),
                Cancellation::new(),
            );
            if ["denied", "legacy-ack", "unoffered"].contains(&mode) {
                assert!(result.is_err(), "reject {mode}");
            } else {
                let mut connection = result.unwrap_or_else(|x| panic!("{mode}: {x:?}"));
                assert_eq!(
                    connection.negotiated_extensions(),
                    Some(if mode == "data" { 1 } else { 3 })
                );
                assert_eq!(connection.application_state(), ApplicationState::Negotiated);
                let ack = connection.ack().unwrap();
                assert!(e::allows_data(
                    ack,
                    connection.negotiated_extensions().unwrap()
                ));
                assert_eq!(
                    e::allows_commands(ack, connection.negotiated_extensions().unwrap()),
                    mode != "data"
                );
                assert_eq!(
                    connection.tls_version(),
                    if mode == "tls12" {
                        TlsVersion::Tls12
                    } else {
                        TlsVersion::Tls13
                    }
                );
                connection.close().unwrap();
                assert_eq!(connection.negotiated_extensions(), None);
                assert_eq!(connection.state(), ClientState::Closed);
                assert_eq!(connection.application_state(), ApplicationState::Closed);
                connection.close().unwrap();
            }
            assert_eq!(
                peer.read_bounded_line("CLOSED").trim(),
                format!("CLOSED {mode}")
            );
            peer.expect_successful_exit();
        }
        eprintln!("EXTENDED_TLS: seven real Java/Rust connections; independent healing DATA/COMMANDS, TLS1.2/1.3, fragmented ACK, missing required extensions, legacy downgrade and unoffered grants; no authentication/gameplay claim");
    }
}
