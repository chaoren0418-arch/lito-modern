//! Operator-only R383 host. It starts an owned Main/SQL fixture and publishes
//! a profile for *separate official* LiToModern.exe processes. It never creates
//! a substitute battle HWND or sends gameplay commands on the operator's behalf.
use super::*;
use sha2::{Digest, Sha256};
use std::{ffi::OsStr, fs::OpenOptions, time::SystemTime};

struct EphemeralProfile(PathBuf);
impl Drop for EphemeralProfile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn forward_operator_lines(mut input: impl std::io::BufRead, sender: std::sync::mpsc::Sender<String>) {
    loop {
        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) | Err(_) => { let _ = sender.send(String::new()); break; }
            Ok(_) => {
                let stop = line.trim().eq_ignore_ascii_case("stop");
                if sender.send(line).is_err() || stop { break; }
            }
        }
    }
}

#[test]
fn r407_operator_input_does_not_retire_on_unrecognized_or_blank_line() {
    let (sender, receiver) = std::sync::mpsc::channel();
    forward_operator_lines(std::io::Cursor::new(b"\nstatus\nstop\nignored\n"), sender);
    assert_eq!(receiver.recv().unwrap(), "\n");
    assert_eq!(receiver.recv().unwrap(), "status\n");
    assert_eq!(receiver.recv().unwrap(), "stop\n");
    assert!(receiver.recv().is_err());
}

fn fixed_official_app() -> PathBuf {
    let app = explicit_path("LITO_MODERN_R383_OFFICIAL_APP");
    assert_eq!(app.file_name(), Some(OsStr::new("LiToModern.exe")));
    assert!(app.is_file(), "fixed official EXE is missing");
    let expected = std::env::var("LITO_MODERN_R383_OFFICIAL_SHA256")
        .expect("set exact 64-digit official EXE SHA256");
    assert!(expected.len() == 64 && expected.bytes().all(|c| c.is_ascii_hexdigit()));
    let actual = format!("{:X}", Sha256::digest(std::fs::read(&app).unwrap()));
    assert_eq!(
        actual,
        expected.to_ascii_uppercase(),
        "official EXE changed"
    );
    app
}

fn publish_profile(endpoint: SocketAddr, shared: &PathBuf, fixtures: &PathBuf, dev: bool) -> EphemeralProfile {
    let xml = explicit_path("LITO_EFFECT_CATALOG_PATH");
    assert!(xml.is_file(), "shared effect catalog missing");
    let mut doc = profile_document(endpoint, shared, &private_root(fixtures));
    doc["schema"] = serde_json::json!(2);
    doc["gameplay_protocol"] = serde_json::json!("modern_v47");
    doc["gameplay_extension"] = serde_json::json!("player_healing_v1");
    doc["effect_catalog_path"] = serde_json::json!(xml);
    doc["profile_id"] = serde_json::json!(if dev {"r407-owned-dev-map514"} else {"r383-owned-slow-healing"});
    doc["display_name"] = serde_json::json!(if dev {"R407 owned Map514 playable development"} else {"R383 owned Map514 Slow and Healing"});
    let directory = std::env::current_dir()
        .unwrap()
        .join("target/r383-owned-profiles");
    assert!(
        directory.is_absolute()
            && directory
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("Cargo.toml")
                .is_file()
    );
    std::fs::create_dir_all(&directory).unwrap();
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = directory.join(format!("{}-{}-{nonce}.json", if dev {"r407"} else {"r383"}, std::process::id()));
    let bytes = serde_json::to_vec_pretty(&doc).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
    drop(file);
    let loaded = ModernProfile::load(&path).expect("official profile parses from disk");
    assert_eq!(loaded.profile_id(), if dev {"r407-owned-dev-map514"} else {"r383-owned-slow-healing"});
    assert_eq!(loaded.effect_catalog_path(), Some(xml.as_path()));
    assert_eq!(
        loaded.gameplay_extension(),
        Some(crate::modern_lito::profile::GameplayExtension::PlayerHealingV1)
    );
    EphemeralProfile(path)
}

#[test]
#[ignore = "OPERATOR ONLY: owns fresh Main/SQL for two separate official EXE windows; needs native screenshot-backed UI and two completed client sessions"]
fn r383_owned_main_for_official_exe_two_window_acceptance() {
    let app = fixed_official_app();
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let deadline = Deadline(Instant::now() + Duration::from_secs(630));
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernSlowSqlTestPeer",
        "slow-healing-window",
        Some("600000"),
    );
    let profile = publish_profile(peer.endpoint, &shared, &fixtures, false);
    eprintln!("R383_OFFICIAL_APP={}", app.display());
    eprintln!("R383_OFFICIAL_PROFILE={}", profile.0.display());
    eprintln!("R383_OWNED_ENDPOINT={}", peer.endpoint);
    eprintln!("R383_READY: open two official EXE windows with --profile; close both normally before 600 s");
    loop {
        deadline.check("two official client sessions must end before owned peer deadline");
        if peer
            .child
            .try_wait()
            .expect("poll owned Main/SQL peer")
            .is_some()
        {
            break;
        }
        deadline.pause("waiting for official EXE exit and server cleanup");
    }
    peer.expect_successful_exit(&deadline);
    eprintln!("R383_OWNED_PEER_CLEANUP verified=true");
}

#[test]
#[ignore = "owned real SQL/TLS contract smoke only; does not count as official EXE or HWND acceptance"]
fn r383_combined_profile_two_source_contract_smoke() {
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let deadline = Deadline(Instant::now() + Duration::from_secs(160));
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernSlowSqlTestPeer",
        "slow-healing-window",
        Some("600000"),
    );
    let file = publish_profile(peer.endpoint, &shared, &fixtures, false);
    let profile = ModernProfile::load(&file.0).unwrap();
    let mut clients = [
        SessionController::new(profile.clone()),
        SessionController::new(profile),
    ];
    for (i, client) in clients.iter_mut().enumerate() {
        submit(client, &deadline, "combined connect", || {
            LoginCommand::Connect
        });
        wait_owner(client, &deadline, "combined TLS", |s| {
            s.phase == LoginPhase::LoginReady
        });
        submit(client, &deadline, "combined login", || {
            LoginCommand::Login {
                account: "fixture-user".into(),
                password: "fixture-password".into(),
            }
        });
        wait_owner(client, &deadline, "combined authentication", |s| {
            s.phase == LoginPhase::ChoosingSource
        });
        let source = if i == 0 { 7 } else { 42 };
        submit(client, &deadline, "combined source", || {
            LoginCommand::SelectSource(source)
        });
        let roster = wait_owner(client, &deadline, "combined roster", |s| {
            s.phase == LoginPhase::CharactersReady
        });
        assert_eq!(roster.characters.len(), 1);
        let selection = SelectionContext {
            key: CharacterKey {
                source_id: source,
                object_id: 3333,
            },
            epoch: roster.roster_epoch,
        };
        submit(client, &deadline, "combined character", || {
            LoginCommand::SelectCharacter(selection)
        });
        let selected = wait_owner(client, &deadline, "combined character data", |s| {
            s.phase == LoginPhase::ChoosingChannel
        });
        assert!(selected.canonical.slow_supported());
        assert!(selected.canonical.player_healing_supported());
    }
    for client in &mut clients {
        submit(client, &deadline, "combined normal shutdown", || {
            LoginCommand::Shutdown
        });
        while !client.is_quiescent() {
            client.poll();
            deadline.pause("combined owner retirement");
        }
    }
    peer.expect_successful_exit(&deadline);
    eprintln!("R383_COMBINED_CONTRACT source7/source42+Slow+Healing+clean TLS/SQL only; no HWND");
}

#[test]
#[ignore = "owned real Main/SQL/TLS/Map514 dev host boots and stops without clients; no HWND/gameplay"]
fn r407_owned_dev_showcase_boot_stop_smoke() {
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let deadline = Deadline(Instant::now() + Duration::from_secs(120));
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures, &deadline, "local.lito381.server.ModernSlowSqlTestPeer",
        "slow-healing-dev", Some("60000"));
    let profile = publish_profile(peer.endpoint, &shared, &fixtures, true);
    assert_eq!(ModernProfile::load(&profile.0).unwrap().profile_id(), "r407-owned-dev-map514");
    peer.stop_dev(&deadline);
    eprintln!("R407_DEV_SMOKE owned Main/SQL/TLS/Map514 boot and operator stop; no client/UI claim");
}

#[test]
#[ignore = "OPERATOR ONLY: owned live Main/SQL, real Map514 and official EXE profile; type stop in host terminal; not an automatic game verdict"]
fn r407_owned_dev_showcase_operator() {
    let app = fixed_official_app();
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let deadline = Deadline(Instant::now() + Duration::from_secs(43_230));
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures, &deadline, "local.lito381.server.ModernSlowSqlTestPeer",
        "slow-healing-dev", Some("43200000"));
    let profile = publish_profile(peer.endpoint, &shared, &fixtures, true);
    eprintln!("R407_OFFICIAL_APP={}", app.display());
    eprintln!("R407_OFFICIAL_PROFILE={}", profile.0.display());
    eprintln!("R407_OWNED_ENDPOINT={}", peer.endpoint);
    eprintln!("R407_DEV_READY: open the fixed EXE with --profile; source 7 has GM .de, Map514/room capacity 2; host stays until terminal input stop (hard safety 12 h)");
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let input = std::io::stdin();
        forward_operator_lines(input.lock(), sender);
    });
    loop {
        deadline.check("owned dev host lifetime");
        assert!(peer.child.try_wait().unwrap().is_none(), "owned dev Main exited unexpectedly");
        match receiver.recv_timeout(Duration::from_millis(250)) {
            Ok(line) if line.trim().eq_ignore_ascii_case("stop") || line.is_empty() => break,
            Ok(_) => eprintln!("R407_DEV_COMMAND: type stop to close the owned showcase"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {},
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    peer.stop_dev(&deadline);
    eprintln!("R407_DEV_CLEANUP verified=true; ephemeral profile and SQL retire with this host");
}
