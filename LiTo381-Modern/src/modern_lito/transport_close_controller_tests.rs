// Included in transport::tests. Strict peer decrypts TLS records and observes
// close_notify, not just an EOF or a zero server failure counter. No HWND/auth.
fn close_oracle(mode: &str) -> JavaPeer {
    JavaPeer::start_class("local.lito381.server.ModernTlsCloseTestPeer", mode, "15000")
}

#[test]
#[ignore = "strict Java SSLEngine close_notify oracle, generated loopback TLS identity; no auth"]
fn java_tls_close_oracle_accepts_explicit_notify_and_distinguishes_abrupt_drop() {
    for mode in ["idle", "idle-tls12", "abrupt"] {
        let mut peer = close_oracle(mode);
        let mut connection = connect_and_negotiate(peer.endpoint, "localhost", Some(&private_root()),
            Hello::current(301), integration_limits(), Cancellation::new()).unwrap();
        if mode != "abrupt" {
            connection.close().unwrap();
            connection.close().unwrap();
            assert_eq!(connection.state(), ClientState::Closed);
        }
        drop(connection);
        peer.expect_successful_exit();
    }
}

#[test]
#[ignore = "strict Java TLS close oracle: ordinary cancellation stays hard; frame-boundary close has its own deadline only"]
fn java_tls_close_cancelled_api_and_idle_worker_have_distinct_terminal_paths() {
    for idle in [false, true] {
        let mut peer = close_oracle(if idle { "idle" } else { "abrupt" });
        let cancelled = Cancellation::new();
        let mut connection = connect_and_negotiate(peer.endpoint, "localhost", Some(&private_root()),
            Hello::current(302), integration_limits(), cancelled.clone()).unwrap();
        cancelled.cancel();
        if idle { connection.close_idle_worker().unwrap(); } else { connection.close().unwrap(); }
        assert!(cancelled.is_cancelled(), "close must never reset the application stop token");
        assert_eq!(connection.state(), ClientState::Closed);
        assert!(connection.ack().is_none());
        assert!(connection.ping().is_err(), "a closing connection cannot resume application IO");
        connection.close_idle_worker().unwrap(); // Already closed: no late alert or new IO.
        peer.expect_successful_exit();
    }
}

fn close_controller_flow(mode: &str, disconnect: bool) {
    use super::super::{login_window::{LoginCommand, LoginController, LoginPhase},
        profile::ModernProfile, session::SessionController};
    use base64::Engine;
    let mut peer = close_oracle(mode);
    let profile = ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
        "schema":1, "profile_id":"strict-close-interop", "display_name":"Owned close oracle",
        "shared_resource_root":"D:\\synthetic-test", "endpoint":peer.endpoint.to_string(),
        "tls_server_name":"localhost", "trust":{"kind":"private_ca",
        "certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())},
        "connect_timeout_ms":500, "exchange_timeout_ms":10000
    })).unwrap()).unwrap();
    let mut controller = SessionController::new(profile);
    controller.command(LoginCommand::Connect).unwrap();
    let ready = Instant::now() + Duration::from_secs(5);
    loop {
        controller.poll();
        let view = controller.snapshot();
        assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
        if view.phase == LoginPhase::LoginReady { break; }
        assert!(Instant::now() < ready);thread::sleep(Duration::from_millis(5));
    }
    if mode == "held-ping" {
        // Proven in-flight application exchange, not an arbitrary sleep/race.
        assert_eq!(peer.read_bounded_line("PING_HELD").trim_end(), "PING_HELD");
    }
    let started = Instant::now();
    controller.command(if disconnect { LoginCommand::Disconnect } else { LoginCommand::Shutdown }).unwrap();
    assert!(started.elapsed() < Duration::from_millis(100), "UI command must not join or wait on TLS");
    while !controller.is_quiescent() {
        controller.poll();
        assert!(started.elapsed() < Duration::from_millis(1000), "bounded owned-worker retirement");
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(controller.snapshot().phase, LoginPhase::Disconnected);
    assert!(controller.snapshot().sources.is_empty() && controller.snapshot().characters.is_empty());
    controller.command(LoginCommand::Shutdown).unwrap();
    assert!(controller.command(LoginCommand::Connect).is_err());
    peer.expect_successful_exit();
}

#[test]
#[ignore = "strict Java TLS1.2/1.3 close proof for actual SessionController idle disconnect/shutdown; no auth"]
fn java_tls_close_controller_idle_disconnect_shutdown_and_silent_peer() {
    for (mode, disconnect) in [("idle", true), ("idle-tls12", false), ("silent-after-ack", false)] {
        close_controller_flow(mode, disconnect);
    }
}

#[test]
#[ignore = "Java peer proves a held real PING; uncertain in-flight shutdown remains bounded hard cancellation"]
fn java_tls_close_controller_pending_ping_cancels_without_clean_close_claim() {
    close_controller_flow("held-ping", false);
}

#[test]
#[ignore = "strict TLS1.2/1.3 oracle: cancellation before any application write must send only close_notify, no PING or AUTH"]
fn java_tls_close_cancel_before_first_write_is_clean_without_sending_the_request() {
    for mode in ["idle", "idle-tls12"] {
        for authentication in [false, true] {
            let mut peer = close_oracle(mode);
            let cancel = Cancellation::new();
            let mut connection = connect_and_negotiate(peer.endpoint, "localhost", Some(&private_root()),
                Hello::current(303), integration_limits(), cancel.clone()).unwrap();
            cancel.cancel();
            let started = Instant::now();
            let result = if authentication {
                connection.authenticate(Credentials::new("unsent-fixture-account".into(), "unsent-fixture-password".into()).unwrap()).map(|_| ())
            } else { connection.ping() };
            assert!(matches!(result, Err(TransportError::Cancelled)), "cancellation is not a successful request");
            assert!(started.elapsed() < Duration::from_millis(1000), "bounded close-only IO");
            assert!(cancel.is_cancelled()); assert_eq!(connection.state(), ClientState::Closed);
            assert!(connection.ack().is_none());
            // The independent peer requires an authenticated TLS close and rejects
            // even a single application byte. A bare TCP EOF is an explicit failure.
            peer.expect_successful_exit();
        }
    }
}

fn session_failure_flow(held: bool) {
    use super::super::{login_window::{LoginCommand, LoginController, LoginPhase},
        profile::ModernProfile, session::SessionController};
    use base64::Engine;
    let mut peer = JavaPeer::start_class("local.lito381.server.ModernSessionFailureTestPeer",
        if held { "held-characters" } else { "idle" }, "15000");
    let profile = ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
        "schema":1, "profile_id":"session-failure-diagnosis", "display_name":"Owned session failure diagnosis",
        "shared_resource_root":"D:\\synthetic-test", "endpoint":peer.endpoint.to_string(),
        "tls_server_name":"localhost", "trust":{"kind":"private_ca",
        "certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())},
        "connect_timeout_ms":500, "exchange_timeout_ms":10000
    })).unwrap()).unwrap();
    let mut controller = SessionController::new(profile);
    let until = Instant::now() + Duration::from_secs(5);
    fn phase(c: &mut SessionController, wanted: LoginPhase, until: Instant) {
        loop {
            c.poll(); let view = c.snapshot(); assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
            if view.phase == wanted { return; }
            assert!(Instant::now() < until, "controlled phase deadline"); thread::sleep(Duration::from_millis(5));
        }
    }
    controller.command(LoginCommand::Connect).unwrap(); phase(&mut controller, LoginPhase::LoginReady, until);
    controller.command(LoginCommand::Login { account: "fixture-user".into(), password: "fixture-password".into() }).unwrap();
    phase(&mut controller, LoginPhase::ChoosingSource, until);
    controller.command(LoginCommand::SelectSource(7)).unwrap();
    if held {
        assert_eq!(peer.read_bounded_line("READ_HELD").trim_end(), "READ_HELD");
        assert!(controller.snapshot().characters.is_empty(), "held reply cannot invent a roster");
    } else { phase(&mut controller, LoginPhase::CharactersReady, until); }
    let start = Instant::now(); controller.command(LoginCommand::Shutdown).unwrap();
    assert!(start.elapsed() < Duration::from_millis(100), "shutdown command cannot join IO");
    while !controller.is_quiescent() {
        controller.poll(); assert!(start.elapsed() < Duration::from_millis(1000)); thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(controller.snapshot().phase, LoginPhase::Disconnected);
    assert!(controller.snapshot().characters.is_empty() && controller.snapshot().sources.is_empty());
    if held {
        peer.child.stdin.as_mut().unwrap().write_all(b"RELEASE\n").unwrap();
    }
    let diagnosis = peer.read_bounded_line("DIAG");
    if held {
        assert!(diagnosis.starts_with("DIAG APPLICATION_") && diagnosis.trim_end().ends_with(" 1"));
        eprintln!("controlled cancellation: {}", diagnosis.trim_end());
    } else { assert_eq!(diagnosis.trim_end(), "DIAG CLEAN 0"); }
    controller.poll(); assert!(controller.snapshot().characters.is_empty(), "late read result cannot resurrect a closed controller");
    peer.expect_successful_exit();
}

#[test]
#[ignore = "production ModernServer clean TLS/roster shutdown has zero failure diagnostics; synthetic repositories"]
fn java_session_failure_diagnostics_idle_authenticated_close_is_clean() { session_failure_flow(false); }

#[test]
#[ignore = "production listener: server proves held roster read, actual controller cancels before RELEASE, exact transport failure and no replay"]
fn java_session_failure_diagnostics_inflight_cancel_is_not_a_gameplay_or_protocol_failure() { session_failure_flow(true); }
