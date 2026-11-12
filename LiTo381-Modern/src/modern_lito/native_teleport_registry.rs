//! Source-native replacement for the historical 13-entry overlay registry and
//! 14-entry fixture binding. Used by the test executable and owned runner;
//! never reads 工作佇列.json, candidate directories or old build qualifications.
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

struct Source {
    path: &'static str,
    bytes: &'static [u8],
}
macro_rules! source {
    ($path:literal) => {
        Source { path: $path, bytes: include_bytes!(concat!("../../", $path)) }
    };
}
struct Entry {
    legacy_key: &'static str,
    group: &'static str,
    disposition: &'static str,
    implementation: &'static str,
    verification: &'static str,
    sources: &'static [Source],
}

// These are logical migration entries, not a claim that all historical code
// has been admitted. Baseline keyboard/skill files retain their current code;
// the absent blind QA is explicitly excluded, not silently counted as a test.
static REGISTRY: &[Entry] = &[
    Entry { legacy_key: "src/modern_lito/world_raster.rs", group: "keyboard", disposition: "current-baseline",
        implementation: "src/modern_lito/world_raster.rs", verification: "cargo check --all-targets; keyboard behavior outside T023-1",
        sources: &[source!("src/modern_lito/world_raster.rs")] },
    Entry { legacy_key: "src/modern_lito/native_world_scene.rs", group: "keyboard", disposition: "current-baseline",
        implementation: "src/modern_lito/native_world_scene.rs", verification: "cargo check --all-targets; keyboard behavior outside T023-1",
        sources: &[source!("src/modern_lito/native_world_scene.rs")] },
    Entry { legacy_key: "src/modern_lito/native_login_window.rs", group: "keyboard", disposition: "current-baseline",
        implementation: "src/modern_lito/native_login_window.rs", verification: "cargo check --all-targets; GUI outside T023-1",
        sources: &[source!("src/modern_lito/native_login_window.rs")] },
    Entry { legacy_key: "tests/native_skill_selection.rs", group: "keyboard", disposition: "current-baseline",
        implementation: "tests/native_skill_selection.rs", verification: "cargo test --test native_skill_selection",
        sources: &[source!("tests/native_skill_selection.rs")] },
    Entry { legacy_key: "src/modern_lito/native_skills.rs", group: "blind20", disposition: "current-baseline",
        implementation: "src/modern_lito/native_skills.rs", verification: "cargo check --all-targets; historical blind20 overlay not admitted",
        sources: &[source!("src/modern_lito/native_skills.rs")] },
    Entry { legacy_key: "src/modern_lito/native_skill_selection.rs", group: "blind20", disposition: "current-baseline",
        implementation: "src/modern_lito/native_skill_selection.rs", verification: "cargo test --test native_skill_selection; no blind20 qualification",
        sources: &[source!("src/modern_lito/native_skill_selection.rs")] },
    Entry { legacy_key: "tests/native_blind.rs", group: "blind20", disposition: "excluded-candidate-only",
        implementation: "No historical blind QA is compiled or represented as passing",
        verification: "recovery_registry::candidate_only_qa_is_not_admitted",
        sources: &[] },
    Entry { legacy_key: "src/modern_lito/native_login.rs", group: "cancel-recovery", disposition: "rebuilt-in-src",
        implementation: "LoginController::{cancel,take_teleport_recovery,refresh_recovery_source}",
        verification: "recovery_wire::* and native_login::teleport_recovery_tests::*",
        sources: &[source!("src/modern_lito/native_login.rs")] },
    Entry { legacy_key: "src/modern_lito/native_rental_return_tests.rs", group: "cancel-recovery", disposition: "split-in-src",
        implementation: "Existing rental tests retained; cancellation/source gates in native_teleport_recovery_tests.rs",
        verification: "cargo test --lib -- modern_lito::native_login::",
        sources: &[source!("src/modern_lito/native_rental_return_tests.rs"), source!("src/modern_lito/native_teleport_recovery_tests.rs")] },
    Entry { legacy_key: "src/modern_lito/native_teleport_controller_tests.rs", group: "cancel-recovery", disposition: "rebuilt-in-src",
        implementation: "Encrypted FaultPeer and public recovery cases",
        verification: "recovery_wire::*",
        sources: &[source!("src/modern_lito/native_teleport_controller_tests.rs")] },
    Entry { legacy_key: "tests/native_teleport_failure.rs", group: "cancel-recovery", disposition: "rebuilt-in-src",
        implementation: "observe_source_expiry, source_handoff_expiry_confirms_c8_before_manual_selection",
        verification: "recovery_wire::natural_source_wait_then_complete_c8_and_manual_selection; opt-in owned entry",
        sources: &[source!("tests/native_teleport_failure.rs")] },
    Entry { legacy_key: "tests/oracle/NativeTeleportFailureHarness.java", group: "cancel-recovery", disposition: "rebuilt-current-fixture",
        implementation: "Nondefault source, 33/31s scenario, memory-only owned fixture",
        verification: "javac against E149; owned runtime remains opt-in",
        sources: &[source!("tests/oracle/NativeTeleportFailureHarness.java")] },
    Entry { legacy_key: "tests/oracle/NativeRecoveryReadonlySnapshot.java", group: "cancel-recovery", disposition: "replaced-current-fixture",
        implementation: "NativeTeleportFailureHarness.stagedSnapshot and NativePeerHarness.observe; no historical item observer admission",
        verification: "javac against E149; full item observer belongs to T023-4",
        sources: &[source!("tests/oracle/NativeTeleportFailureHarness.java"), source!("tests/oracle/NativePeerHarness.java")] },
];
static BOOTSTRAP: Entry = Entry {
    legacy_key: "tests/oracle/NativeRecoveryProductionBootstrap.java", group: "fixture-bootstrap", disposition: "replaced-current-fixture",
    implementation: "NativeTeleportFailureHarness.main creates memory repositories before accepting sessions; production/SQL bootstrap excluded",
    verification: "javac against E149; database.mode=memory, database.read.only=true, state.persistence.enabled=false",
    sources: &[source!("tests/oracle/NativeTeleportFailureHarness.java"), source!("tests/oracle/NativePeerHarness.java")],
};

// Extra inputs are dependencies, deliberately not disguised as legacy slots.
static DEPENDENCIES: &[Source] = &[
    source!("Cargo.toml"), source!("Cargo.lock"), source!("build.rs"),
    source!("src/lib.rs"), source!("src/modern_lito/mod.rs"),
    source!("src/modern_lito/native_teleport_registry.rs"),
    source!("src/modern_lito/native_connection.rs"), source!("src/modern_lito/native_game.rs"),
    source!("src/modern_lito/native_teleport.rs"), source!("src/modern_lito/native_services.rs"),
    source!("tests/fixtures/native-game-current.json"),
    source!("tests/fixtures/native-teleport-controller-current.json"),
    Source { path: "../維護工具/run-teleport-failure.ps1", bytes: include_bytes!("../../../維護工具/run-teleport-failure.ps1") },
];

fn digest(bytes: &[u8]) -> String { format!("{:X}", Sha256::digest(bytes)) }

/// Compare current worktree bytes with the bytes compiled into THIS test image.
/// A stale image or changed Java/runner is rejected before the owned server runs.
pub fn checkout_binding() -> Result<Value> {
    binding_with(|name| std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(name))
        .with_context(|| format!("missing current source: {name}")))
}

fn binding_with(mut read: impl FnMut(&str) -> Result<Vec<u8>>) -> Result<Value> {
    let mut source_pins = BTreeMap::new();
    for source in REGISTRY.iter().chain([&BOOTSTRAP]).flat_map(|e| e.sources).chain(DEPENDENCIES) {
        if source_pins.contains_key(source.path) { continue; }
        let actual = read(source.path)?;
        ensure!(actual == source.bytes, "test image/source drift: {}", source.path);
        source_pins.insert(source.path, digest(source.bytes));
    }
    let entry = |e: &Entry| json!({"legacy_key":e.legacy_key,"group":e.group,
        "disposition":e.disposition,"implementation":e.implementation,"verification":e.verification,
        "current_sources":e.sources.iter().map(|s|s.path).collect::<Vec<_>>()});
    let registry: Vec<_> = REGISTRY.iter().map(entry).collect();
    let fixture: Vec<_> = REGISTRY.iter().chain([&BOOTSTRAP]).map(entry).collect();
    Ok(json!({"schema":"current-src-teleport-binding-v1", "registry":registry,"fixture":fixture,
        "source_pins":source_pins,"scope":"T023-1-current-controller-memory-fixture",
        "historical_overlay_applied":false,"historical_qualification_reused":false,
        "full_historical_14_file_behavior_verified":false}))
}

#[test]
fn source_binding_matches_compiled_checkout() {
    let binding = checkout_binding().unwrap();
    let registry = binding["registry"].as_array().unwrap();
    let fixture = binding["fixture"].as_array().unwrap();
    assert_eq!(registry.len(), 13);
    assert_eq!(fixture.len(), 14);
    let keys: std::collections::BTreeSet<_> = fixture.iter().map(|e| e["legacy_key"].as_str().unwrap()).collect();
    assert_eq!(keys.len(), 14);
    assert_eq!(&fixture[..13], registry.as_slice());
    assert_eq!(registry.iter().filter(|e| e["group"] == "keyboard").count(), 4);
    assert_eq!(registry.iter().filter(|e| e["group"] == "blind20").count(), 3);
    assert_eq!(registry.iter().filter(|e| e["group"] == "cancel-recovery").count(), 6);
    println!("LITO_TELEPORT_BINDING={binding}");
}

#[test]
fn stale_image_or_missing_source_rejects_the_binding() {
    for path in ["src/modern_lito/native_login.rs", "tests/oracle/NativeTeleportFailureHarness.java", "../維護工具/run-teleport-failure.ps1"] {
        for missing in [false, true] {
            let result = binding_with(|name| {
                if name == path {
                    ensure!(!missing, "missing fixture input: {name}");
                    return Ok(b"stale source".to_vec());
                }
                Ok(std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(name))?)
            });
            let error = result.unwrap_err().to_string();
            assert!(error.contains(path), "wrong rejection: {path}, missing={missing}: {error}");
        }
    }
}

#[test]
fn candidate_only_qa_is_not_admitted() {
    let binding = checkout_binding().unwrap();
    let excluded = &binding["registry"][6];
    assert_eq!(excluded["legacy_key"], "tests/native_blind.rs");
    assert_eq!(excluded["disposition"], "excluded-candidate-only");
    assert!(excluded["current_sources"].as_array().unwrap().is_empty());
    assert!(binding["source_pins"].get("tests/native_blind.rs").is_none());
    assert_eq!(binding["full_historical_14_file_behavior_verified"], false);
}
