//! Candidate integration test; not installed or compiled with the active source.
//! Uses the owned formal NativePeerHarness rental-return memory fixture only.
use anyhow::{ensure, Context, Result};
use lito_modern::modern_lito::{
    native_connection::Limits,
    native_login::{LoginController, LoginPhase as Phase, Snapshot},
};
use serde_json::{json, Value};
use std::{net::SocketAddr, path::Path, thread, time::{Duration, Instant}};

fn wait(controller: &mut LoginController, ready: impl Fn(&Snapshot) -> bool) -> Result<()> {
    let end = Instant::now() + Duration::from_secs(15);
    loop {
        controller.poll();
        ensure!(!matches!(controller.snapshot().phase, Phase::Failed | Phase::Closed),
            "client failed: {:?}", controller.snapshot().error);
        if ready(controller.snapshot()) { return Ok(()); }
        if controller.snapshot().notice.as_ref().is_some_and(|notice| notice.acknowledgement().is_some()) {
            controller.acknowledge_notice()?;
        }
        ensure!(Instant::now() < end, "timeout at {:?}", controller.snapshot().phase);
        thread::sleep(Duration::from_millis(10));
    }
}

fn rejects_channel_without_state_change(controller: &mut LoginController) -> Result<Value> {
    ensure!(controller.snapshot().phase == Phase::CharacterSelected, "selected character required");
    ensure!(controller.snapshot().services.as_ref().is_some_and(|directory| !directory.rooms.is_empty()),
        "formal advertised rental service required");
    let before = serde_json::to_value(controller.snapshot())?;
    let epoch = controller.transport().snapshot().epoch;
    let error = controller.enter_channel(0).expect_err("rental service must reject ordinary channel entry");
    ensure!(error.to_string().contains("此入口提供租裝服務"), "unexpected rejection: {error:#}");
    ensure!(serde_json::to_value(controller.snapshot())? == before, "rejected request changed game state");
    ensure!(controller.transport().snapshot().epoch == epoch, "rejected request changed connection");
    Ok(json!({"snapshot": before, "epoch": epoch, "error": error.to_string()}))
}

fn player(root: &Path, peer: SocketAddr, role: &str, account: &str, inventory_count: usize) -> Result<()> {
    let mut controller = LoginController::default();
    let result = (|| -> Result<()> {
        controller.start(peer, Limits { read_chunk: 2, write_chunk: 3, ..Limits::default() })?;
        wait(&mut controller, |snapshot| snapshot.phase == Phase::ReadyForLogin)?;
        controller.login(account, "TestOnlyPassword")?;
        wait(&mut controller, |snapshot| snapshot.phase == Phase::SourcesReady)?;
        let source = controller.snapshot().sources.iter().position(|source| source.name == "測試乙")
            .context("advertised source 測試乙 missing")?;
        controller.select_source(source)?;
        wait(&mut controller, |snapshot| snapshot.phase == Phase::RedirectReady)?;
        controller.begin_game()?;
        wait(&mut controller, |snapshot| snapshot.phase == Phase::CharactersReady)?;
        ensure!(controller.snapshot().characters.len() == 1, "unexpected fixture characters");
        controller.select_character(0)?;
        wait(&mut controller, |snapshot| snapshot.phase == Phase::CharacterSelected
            && snapshot.services.as_ref().is_some_and(|directory| !directory.rooms.is_empty()))?;
        let rejected_before_entry = rejects_channel_without_state_change(&mut controller)?;
        controller.select_rental_set(1)?;
        wait(&mut controller, |snapshot| snapshot.phase == Phase::RentalReady)?;
        controller.join_rental_room(0)?;
        wait(&mut controller, |snapshot| snapshot.phase == Phase::BattlePending && snapshot.world.ready()
            && snapshot.inventory.items.len() == inventory_count)?;
        ensure!(controller.snapshot().world.map_id == Some(500), "rental town missing");
        let entered = controller.snapshot().clone();
        controller.return_from_rental()?;
        wait(&mut controller, |snapshot| snapshot.phase == Phase::CharactersReady)?;
        let returned = controller.snapshot().clone();
        ensure!(returned.selected_character.is_none() && returned.rental_set.is_none()
            && returned.joined_room.is_none() && returned.world.actors.is_empty() && !returned.world.ready(),
            "rental scene retained after formal return");
        ensure!(returned.selected_source.as_deref() == Some("測試乙"), "return source changed");
        controller.select_character(0)?;
        wait(&mut controller, |snapshot| snapshot.phase == Phase::CharacterSelected
            && snapshot.services.as_ref().is_some_and(|directory| !directory.rooms.is_empty()))?;
        let rejected_after_return = rejects_channel_without_state_change(&mut controller)?;
        std::fs::write(root.join(format!("{role}-rental-entry-guard.json")), serde_json::to_vec_pretty(&json!({
            "role": role, "account": account, "rejected_before_entry": rejected_before_entry,
            "entered": entered, "returned": returned, "rejected_after_return": rejected_after_return,
            "scope": "controller rejection, valid rental entry and C10 return; no GUI or hotel claim"
        }))?)?;
        Ok(())
    })();
    controller.cancel();
    let end = Instant::now() + Duration::from_secs(3);
    while !controller.is_quiescent() && Instant::now() < end {
        controller.poll();
        thread::sleep(Duration::from_millis(10));
    }
    ensure!(controller.is_quiescent(), "owned client worker leaked");
    result
}

#[test]
#[ignore = "requires owned formal75 rental-return loopback harness and independent packet readback"]
fn rental_entry_rejects_ordinary_channel_before_wire_and_remains_usable_after_return() {
    let root = std::path::PathBuf::from(std::env::var("LITO_RENTAL_ENTRY_GUARD_ROOT").expect("owned fixture root"));
    assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/current").canonicalize().unwrap()));
    let ready: Value = serde_json::from_slice(&std::fs::read(root.join("ready.json")).unwrap()).unwrap();
    assert_eq!(ready["jar_sha256"], "75da69b0d2f7f9d394ee6bf2fdb396b153c75d9b938069b0999ed8dac4f50371");
    let peer: SocketAddr = format!("127.0.0.1:{}", ready["port"].as_u64().unwrap()).parse().unwrap();
    thread::scope(|scope| {
        let a = scope.spawn(|| player(&root, peer, "a", "PeerFixture", 45));
        let b = scope.spawn(|| player(&root, peer, "b", "PeerFixture2", 43));
        let a = a.join().unwrap(); let b = b.join().unwrap();
        assert!(a.is_ok() && b.is_ok(), "a={a:?}; b={b:?}");
    });
}
