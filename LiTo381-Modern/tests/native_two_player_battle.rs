#[path = "support/two_player_battle.rs"]
mod two_player_battle;
#[path = "support/two_player_lifecycle.rs"]
mod two_player_lifecycle;

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use two_player_battle::{ready_endpoints, TwoPlayerBattle, ACCOUNTS, ACTOR_IDS, REFERENCE_SHA};

#[test]
#[ignore = "requires pinned E149, JAVA_HOME JDK17 and read-only client assets; owns loopback memory server"]
fn two_real_players_attack_vitals_death_settlement_and_reentry() -> Result<()> {
    let mut fixture = TwoPlayerBattle::start()?;
    let result = two_player_lifecycle::exercise(&mut fixture);
    fixture.save_snapshots(if result.is_ok() {
        "lifecycle-complete"
    } else {
        "lifecycle-failure"
    })?;
    result?;
    fixture.close()?;
    Ok(())
}

#[test]
fn reference_rejects_missing_file_and_legacy_fixture() {
    let project = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(
        two_player_battle::java_path(std::path::Path::new(r"\\?\D:\fixture\reference.jar"))
            .unwrap(),
        std::path::PathBuf::from(r"D:\fixture\reference.jar")
    );
    assert!(two_player_battle::java_path(std::path::Path::new(r"\\?\UNC\server\share")).is_err());
    assert!(two_player_battle::verify_reference(
        &project.join("tests/fixtures/native-combat-current.json")
    )
    .is_err());
    assert!(
        two_player_battle::verify_reference(&project.join("tests/fixtures/t043-missing.jar"))
            .is_err()
    );
}

#[test]
fn ready_contract_rejects_wrong_jar_missing_and_forbidden_ports() {
    let valid = json!({"jar_sha256": REFERENCE_SHA, "port": 45001, "game_port": 45002});
    let endpoints = ready_endpoints(&valid).unwrap();
    assert_eq!(endpoints[0].to_string(), "127.0.0.1:45001");
    assert_eq!(endpoints[1].to_string(), "127.0.0.2:45002");
    let mut wrong_jar = valid.clone();
    wrong_jar["jar_sha256"] = json!("schema37");
    assert!(ready_endpoints(&wrong_jar).is_err());
    for field in ["port", "game_port"] {
        for bad in [
            Value::Null,
            json!(-1),
            json!(0),
            json!(2000),
            json!(3306),
            json!(65536),
            json!("45001"),
        ] {
            let mut ready = valid.clone();
            ready[field] = bad;
            assert!(ready_endpoints(&ready).is_err(), "{ready}");
        }
    }
}

#[test]
#[ignore = "requires read-only E149 via LITO_T043_REFERENCE_JAR and JAVA_HOME for JDK 17; owns loopback memory server"]
fn two_real_players_enter_same_battle_without_training_bot() -> Result<()> {
    let mut fixture = TwoPlayerBattle::start()?;
    let root = fixture.root.clone();
    let entered = fixture.enter_battle();
    fixture.save_snapshots("bootstrap")?;
    entered?;
    // Keep both connections alive long enough for the existing server observer to sample them.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        fixture.poll()?;
        let trace = std::fs::read_to_string(root.join("server-trace.jsonl"))?;
        // Ignore only an unfinished final write. Malformed complete rows fail the test.
        let complete = trace.rsplit_once('\n').map_or("", |(rows, _)| rows);
        let rows: Vec<Value> = complete
            .lines()
            .map(serde_json::from_str)
            .collect::<serde_json::Result<_>>()?;
        let states: Vec<Value> = rows.into_iter().map(|row| row["state"].clone()).collect();
        let peers: Vec<&Value> = ACCOUNTS
            .iter()
            .zip(ACTOR_IDS)
            .filter_map(|(account, id)| {
                states.iter().rev().find(|s| {
                    s["test_account"] == *account
                        && s["object_id"] == id
                        && s["phase"] == "BATTLE"
                        && s["observer"] == false
                })
            })
            .collect();
        if peers.len() == 2 {
            ensure!(
                peers[0]["session_identity"] != peers[1]["session_identity"],
                "shared GAME session"
            );
            let room = fixture.players[0]
                .snapshot()
                .joined_room
                .as_ref()
                .context("room")?
                .id;
            ensure!(
                peers.iter().all(|s| s["room"] == room),
                "server/client room mismatch"
            );
            fixture.verify_battle()?;
            std::fs::write(
                root.join("server-peers.json"),
                serde_json::to_vec_pretty(&peers)?,
            )?;
            break;
        }
        ensure!(
            std::time::Instant::now() < deadline,
            "two authoritative GAME sessions not observed"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    fixture.close()?;
    Ok(())
}
