//! Opt-in program-level integration, not a GUI/focus test. Two authenticated TLS
//! owners load the real shared S32/Tile/Text/SPR assets independently, acknowledge
//! their first actual render, and present only server-confirmed world movement.
//! The v5 records case owns a fresh isolated MySQL under a kill-on-close process
//! job. No existing database, native process, listener configuration or asset is edited.

use super::{
    application_protocol::{CharacterKey, Credentials, SelectionContext},
    combat_protocol as combat,
    login_window::{LoginCommand, LoginController, LoginPhase, LoginSnapshot},
    profile::ModernProfile,
    protocol::Hello,
    record_protocol as record,
    resources::SharedMapSectorKey,
    rooms,
    session::SessionController,
    spell_protocol as spell,
    transport::{connect_and_negotiate, Cancellation, TransportLimits},
    world_map::{SectorBinding, WorldCell, WorldMap, DIRECTIONS},
    world_navigation::{Navigation, Status as NavigationStatus},
    world_path, world_protocol as world,
    world_runtime::{RenderedWorld, RuntimeStatus, WorldRuntime, HEIGHT, WIDTH},
};
use base64::Engine;
#[path = "battle_read_interop_tests.rs"]
mod battle_read_interop_tests;
#[path = "binding_clock_interop_tests.rs"]
mod binding_clock_interop_tests;
#[path = "disintegrate_interop_tests.rs"]
mod disintegrate_interop_tests;
#[path = "battle_equipment_controller_tests.rs"]
mod equipment_controller;
#[path = "fire_storm_interop_tests.rs"]
mod fire_storm_interop_tests;
#[path = "honor_interop_tests.rs"]
mod honor_interop_tests;
#[path = "item_interop_tests.rs"]
mod item_interop_tests;
#[path = "map_rotation_interop_tests.rs"]
mod map_rotation_interop_tests;
#[path = "match_descriptor_interop_tests.rs"]
mod match_descriptor_interop_tests;
#[path = "natural_regen_interop_tests.rs"]
mod natural_regen_interop_tests;
#[path = "pet_race_interop_tests.rs"]
mod pet_race_interop_tests;
#[path = "quest_sql_interop_tests.rs"]
mod quest_sql_interop_tests;
#[path = "r383_official_window_host_tests.rs"]
mod r383_official_window_host_tests;
#[path = "ranged_interop_tests.rs"]
mod ranged_interop_tests;
#[path = "spawn_egress_interop_tests.rs"]
mod spawn_egress_interop_tests;
#[path = "timed_interop_tests.rs"]
mod timed_interop_tests;
#[path = "weight_interop_tests.rs"]
mod weight_interop_tests;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io::{BufRead, BufReader, Read, Write},
    net::SocketAddr,
    os::windows::io::AsRawHandle,
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{atomic::AtomicBool, mpsc, Arc},
    thread,
    time::{Duration, Instant},
};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicAccountingInformation,
            JobObjectExtendedLimitInformation, QueryInformationJobObject, SetInformationJobObject,
            JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        },
    },
};

const CHANNEL: u8 = 0;
const SUN: CharacterKey = CharacterKey {
    source_id: 7,
    object_id: 3333,
};
const MOON: CharacterKey = CharacterKey {
    source_id: 42,
    object_id: 3333,
};
const POLL: Duration = Duration::from_millis(10);

/// Every protocol/poll/start/normal-retirement stage shares this one deadline.
/// Resource I/O itself remains bounded synchronous filesystem work; this test
/// does not invent a hard OS filesystem cancellation guarantee.
struct Deadline(Instant);
impl Deadline {
    fn new() -> Self {
        Self(Instant::now() + Duration::from_secs(50))
    }
    fn check(&self, stage: &'static str) {
        assert!(
            Instant::now() < self.0,
            "world integration deadline: {stage}"
        );
    }
    fn remaining(&self, maximum: Duration, stage: &'static str) -> Duration {
        self.check(stage);
        self.0
            .saturating_duration_since(Instant::now())
            .min(maximum)
    }
    fn pause(&self, stage: &'static str) {
        thread::sleep(self.remaining(POLL, stage));
    }
}

/// Only the test's gated Java child and its descendants belong to this job.
/// Assignment must succeed before GO permits Java to start the isolated mysqld.
pub(super) struct OwnedProcessJob(HANDLE);
impl OwnedProcessJob {
    pub(super) fn new() -> Self {
        let job = Self(unsafe { CreateJobObjectW(None, PCWSTR::null()) }.expect("owned test job"));
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const std::ffi::c_void,
                std::mem::size_of_val(&limits) as u32,
            )
        }
        .expect("kill-on-close owned test job");
        job
    }
    pub(super) fn assign(&self, child: &Child) {
        unsafe { AssignProcessToJobObject(self.0, HANDLE(child.as_raw_handle())) }
            .expect("assign gated Java before any SQL process exists");
    }
    pub(super) fn active_processes(&self) -> u32 {
        let mut accounting = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        unsafe {
            QueryInformationJobObject(
                Some(self.0),
                JobObjectBasicAccountingInformation,
                &mut accounting as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of_val(&accounting) as u32,
                None,
            )
        }
        .expect("query only the owned test process tree");
        accounting.ActiveProcesses
    }
}
impl Drop for OwnedProcessJob {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}
struct JavaPeer {
    child: Child,
    endpoint: SocketAddr,
    process_job: Option<OwnedProcessJob>,
    control: Option<ChildStdin>,
}
fn peer_ready_budget(mode: &str) -> Duration {
    Duration::from_secs(
        if matches!(
            mode,
            "mixed-modes-sql"
                | "mixed-results-sql"
                | "mixed-quest-results-sql"
                | "mixed-objectives-sql"
                | "mixed-all-results-sql"
                | "slow-healing-dev"
        ) {
            // Owned SQL, pinned map policies and catalogs are installed before
            // READY. Relocated concurrent development reproduced 60s startup
            // expiry after source qualification. This bounded startup allowance
            // does not change the total scenario or any gameplay deadline.
            // Keep the total scenario deadline and all gameplay deadlines unchanged.
            120
        } else if matches!(
            mode,
            "quest-sql"
                | "greater-haste-sql"
                | "target-haste-sql"
                | "healing-sql"
                | "healing-reject-sql"
                | "healing-mixed-sql"
                | "healing-quest-sql"
                | "armor-sql"
                | "armor-render-sql"
                | "armor-reject-sql"
                | "slow-sql"
                | "slow-window"
                | "slow-healing-window"
                | "slow-quest-sql"
                | "target-haste-quest-sql"
                | "target-haste-time-sql"
                | "target-haste-timeout-sql"
                | "quest-supplies-sql"
                | "quest-teleport-sql"
                | "quest-supplied-teleport-sql"
                | "quest-magic-sql"
                | "quest-bots-sql"
                | "quest-supplied-bots-sql"
                | "time-attack-sql"
                | "npc77-time-attack-sql"
                | "npc4-time-attack-sql"
                | "quest-npc77-sql"
                | "quest-npc70-sql"
                | "quest-matchmaking-sql"
                | "npc70-time-attack-sql"
                | "siege-sql"
                | "siege-shared-sql"
                | "records"
                | "magic"
                | "bots"
                | "items"
                | "weight"
                | "timed"
                | "ammo"
                | "canonical-ammo"
                | "inventory-ammo"
                | "inventory-ammo-elf"
                | "life-ammo"
                | "life-fire"
                | "life-honor"
                | "canonical-honor"
                | "life-disintegrate"
                | "canonical-disintegrate"
                | "canonical-equipment"
                | "regen"
                | "rotation"
                | "rotation-shared"
                | "rotation-shared-508-531"
                | "rotation-shared-531-534"
                | "rotation-shared-516-531"
                | "honor"
                | "rotation-shared-504-531"
                | "rotation-shared-505-531"
                | "rotation-shared-506-531"
                | "fire"
                | "fire-honor"
                | "spawn3"
                | "spawn8"
                | "spawn20"
                | "spawn-honor20"
                | "descriptor-honor"
                | "descriptor-legacy"
                | "clock-scenes"
                | "sql-world-policy"
                | "race-finish"
                | "race-withdraw"
                | "race-disconnect"
        ) {
            20
        } else {
            5
        },
    )
}

#[test]
fn sql_peer_ready_budget_includes_healing_without_changing_gameplay_deadlines() {
    for mode in [
        "quest-sql",
        "target-haste-sql",
        "healing-sql",
        "healing-reject-sql",
        "healing-mixed-sql",
        "healing-quest-sql",
        "armor-sql",
        "armor-reject-sql",
        "armor-render-sql",
    ] {
        assert_eq!(peer_ready_budget(mode), Duration::from_secs(20));
    }
    assert_eq!(
        peer_ready_budget("mixed-objectives-sql"),
        Duration::from_secs(120)
    );
    assert_eq!(
        peer_ready_budget("mixed-all-results-sql"),
        Duration::from_secs(120)
    );
    assert_eq!(peer_ready_budget("unknown-mode"), Duration::from_secs(5));
    assert_eq!(peer_ready_budget("slow-healing-dev"), Duration::from_secs(120));
    assert_eq!(peer_ready_budget("slow-healing-window"), Duration::from_secs(20));
}

impl JavaPeer {
    fn start(fixtures: &PathBuf, deadline: &Deadline) -> Self {
        Self::start_class(
            fixtures,
            deadline,
            "local.lito381.server.ModernWorldTestPeer",
        )
    }
    fn start_class(fixtures: &PathBuf, deadline: &Deadline, class: &'static str) -> Self {
        Self::start_mode(fixtures, deadline, class, "normal")
    }
    fn start_mode(
        fixtures: &PathBuf,
        deadline: &Deadline,
        class: &'static str,
        mode: &'static str,
    ) -> Self {
        Self::start_mode_lifetime(fixtures, deadline, class, mode, None)
    }
    fn start_mode_lifetime(
        fixtures: &PathBuf,
        deadline: &Deadline,
        class: &'static str,
        mode: &'static str,
        lifetime: Option<&'static str>,
    ) -> Self {
        let executable = std::env::var_os("LITO_MODERN_TEST_JAVA")
            .expect("set explicit LITO_MODERN_TEST_JAVA (JDK 17)");
        let classpath = std::env::var_os("LITO_MODERN_TEST_JAVA_CLASSPATH")
            .expect("set compiled Java test classpath");
        let working_directory = if matches!(
            mode,
            "quest-sql"
                | "mixed-modes-sql"
                | "mixed-results-sql"
                | "mixed-quest-results-sql"
                | "mixed-objectives-sql"
                | "mixed-all-results-sql"
                | "greater-haste-sql"
                | "target-haste-sql"
                | "healing-sql"
                | "healing-reject-sql"
                | "healing-mixed-sql"
                | "healing-quest-sql"
                | "armor-sql"
                | "armor-render-sql"
                | "armor-reject-sql"
                | "slow-sql"
                | "slow-window"
                | "slow-healing-window"
                | "slow-healing-dev"
                | "slow-quest-sql"
                | "target-haste-quest-sql"
                | "target-haste-time-sql"
                | "target-haste-timeout-sql"
                | "quest-supplies-sql"
                | "quest-teleport-sql"
                | "quest-supplied-teleport-sql"
                | "quest-magic-sql"
                | "quest-bots-sql"
                | "quest-supplied-bots-sql"
                | "time-attack-sql"
                | "npc77-time-attack-sql"
                | "npc4-time-attack-sql"
                | "quest-npc77-sql"
                | "quest-npc70-sql"
                | "quest-matchmaking-sql"
                | "npc70-time-attack-sql"
                | "siege-sql"
                | "siege-shared-sql"
                | "records"
                | "magic"
                | "bots"
                | "items"
                | "weight"
                | "timed"
                | "ammo"
                | "canonical-ammo"
                | "inventory-ammo"
                | "inventory-ammo-elf"
                | "life-ammo"
                | "life-fire"
                | "life-honor"
                | "canonical-honor"
                | "life-disintegrate"
                | "canonical-disintegrate"
                | "canonical-equipment"
                | "regen"
                | "rotation"
                | "rotation-shared"
                | "rotation-shared-508-531"
                | "rotation-shared-531-534"
                | "rotation-shared-516-531"
                | "honor"
                | "rotation-shared-504-531"
                | "rotation-shared-505-531"
                | "rotation-shared-506-531"
                | "fire"
                | "fire-honor"
                | "disintegrate"
                | "spawn3"
                | "spawn8"
                | "spawn20"
                | "spawn-honor20"
                | "descriptor-honor"
                | "descriptor-legacy"
                | "clock-scenes"
                | "sql-world-policy"
                | "race-finish"
                | "race-withdraw"
                | "race-disconnect"
        ) {
            let server = explicit_path("LITO_MODERN_TEST_SERVER_ROOT");
            assert!(
                server.join("db/modern_match_settlement.sql").is_file(),
                "explicit test server root must contain the actual settlement DDL"
            );
            server
        } else {
            std::env::current_dir().expect("unchanged test working directory")
        };
        let process_job = matches!(
            mode,
            "quest-sql"
                | "mixed-modes-sql"
                | "mixed-results-sql"
                | "mixed-quest-results-sql"
                | "mixed-objectives-sql"
                | "mixed-all-results-sql"
                | "greater-haste-sql"
                | "target-haste-sql"
                | "healing-sql"
                | "healing-reject-sql"
                | "healing-mixed-sql"
                | "healing-quest-sql"
                | "armor-sql"
                | "armor-render-sql"
                | "armor-reject-sql"
                | "slow-sql"
                | "slow-window"
                | "slow-healing-window"
                | "slow-healing-dev"
                | "slow-quest-sql"
                | "target-haste-quest-sql"
                | "target-haste-time-sql"
                | "target-haste-timeout-sql"
                | "quest-supplies-sql"
                | "quest-teleport-sql"
                | "quest-supplied-teleport-sql"
                | "quest-magic-sql"
                | "quest-bots-sql"
                | "quest-supplied-bots-sql"
                | "time-attack-sql"
                | "npc77-time-attack-sql"
                | "npc4-time-attack-sql"
                | "quest-npc77-sql"
                | "quest-npc70-sql"
                | "quest-matchmaking-sql"
                | "npc70-time-attack-sql"
                | "siege-sql"
                | "siege-shared-sql"
                | "records"
                | "magic"
                | "bots"
                | "items"
                | "weight"
                | "timed"
                | "ammo"
                | "canonical-ammo"
                | "inventory-ammo"
                | "inventory-ammo-elf"
                | "life-ammo"
                | "life-fire"
                | "life-honor"
                | "canonical-honor"
                | "life-disintegrate"
                | "canonical-disintegrate"
                | "canonical-equipment"
                | "regen"
                | "rotation"
                | "rotation-shared"
                | "rotation-shared-508-531"
                | "rotation-shared-531-534"
                | "honor"
                | "rotation-shared-516-531"
                | "rotation-shared-504-531"
                | "rotation-shared-505-531"
                | "rotation-shared-506-531"
                | "fire"
                | "fire-honor"
                | "disintegrate"
                | "spawn3"
                | "spawn8"
                | "spawn20"
                | "spawn-honor20"
                | "descriptor-honor"
                | "descriptor-legacy"
                | "clock-scenes"
                | "sql-world-policy"
                | "race-finish"
                | "race-withdraw"
                | "race-disconnect"
        )
        .then(OwnedProcessJob::new);
        let slow_boot_trace = class == "local.lito381.server.ModernSlowSqlTestPeer";
        let spawn_started = Instant::now();
        let child = Command::new(executable)
            .current_dir(working_directory)
            .arg("-cp")
            .arg(classpath)
            .arg(class)
            .arg("serve")
            .arg(fixtures)
            .arg(mode)
            .arg(lifetime.unwrap_or(
                if matches!(
                    mode,
                    "quest-sql"
                        | "mixed-modes-sql"
                        | "mixed-results-sql"
                        | "mixed-quest-results-sql"
                        | "mixed-objectives-sql"
                        | "mixed-all-results-sql"
                        | "greater-haste-sql"
                        | "target-haste-sql"
                        | "healing-sql"
                        | "healing-reject-sql"
                        | "healing-mixed-sql"
                        | "healing-quest-sql"
                        | "armor-sql"
                        | "armor-render-sql"
                        | "armor-reject-sql"
                        | "slow-sql"
                        | "slow-window"
                        | "slow-healing-window"
                        | "slow-quest-sql"
                        | "target-haste-quest-sql"
                        | "target-haste-time-sql"
                        | "target-haste-timeout-sql"
                        | "quest-supplies-sql"
                        | "quest-teleport-sql"
                        | "quest-supplied-teleport-sql"
                        | "quest-magic-sql"
                        | "quest-bots-sql"
                        | "quest-supplied-bots-sql"
                        | "time-attack-sql"
                        | "npc77-time-attack-sql"
                        | "npc4-time-attack-sql"
                        | "quest-npc77-sql"
                        | "quest-npc70-sql"
                        | "quest-matchmaking-sql"
                        | "npc70-time-attack-sql"
                        | "siege-sql"
                        | "siege-shared-sql"
                        | "honor"
                        | "fire"
                        | "fire-honor"
                        | "spawn3"
                        | "spawn8"
                        | "spawn20"
                        | "spawn-honor20"
                        | "descriptor-honor"
                        | "descriptor-legacy"
                        | "canonical-ammo"
                        | "inventory-ammo"
                        | "inventory-ammo-elf"
                        | "life-ammo"
                        | "life-fire"
                        | "life-honor"
                        | "canonical-honor"
                        | "clock-scenes"
                        | "sql-world-policy"
                        | "race-finish"
                        | "race-withdraw"
                        | "race-disconnect"
                ) {
                    "180000"
                } else {
                    "60000"
                },
            ))
            .env_remove("LITO_MODERN_PROBE_ACCOUNT")
            .env_remove("LITO_MODERN_PROBE_PASSWORD")
            .stdin(if process_job.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("launch owned world test peer");
        if slow_boot_trace {
            eprintln!(
                "R398_OWNED_BOOT stage=child-spawned mode={mode} elapsed_ms={} pid={}",
                spawn_started.elapsed().as_millis(),
                child.id()
            );
        }
        let mut peer = Self {
            child,
            endpoint: "127.0.0.1:0".parse().unwrap(),
            process_job,
            control: None,
        };
        if let Some(job) = &peer.process_job {
            job.assign(&peer.child);
            let mut gate = peer.child
                .stdin
                .take()
                .expect("owned process gate");
            gate.write_all(b"GO\n")
                .expect("release only the assigned process tree");
            if mode == "slow-healing-dev" { peer.control = Some(gate); }
            if slow_boot_trace {
                eprintln!(
                    "R398_OWNED_BOOT stage=owned-gate-sent mode={mode} elapsed_ms={} pid={}",
                    spawn_started.elapsed().as_millis(),
                    peer.child.id()
                );
            }
        }
        // Deadline validation can panic; do it before owning a reader thread.
        let ready_timeout = deadline.remaining(peer_ready_budget(mode), "peer READY");
        let stdout = peer.child.stdout.take().expect("owned peer stdout");
        let (sender, receiver) = mpsc::sync_channel(1);
        let reader = thread::spawn(move || {
            let mut bounded = BufReader::new(stdout).take(128);
            let mut line = String::new();
            let _ = sender.send(bounded.read_line(&mut line).map(|_| line));
        });
        let result = receiver.recv_timeout(ready_timeout);
        if result.is_err() {
            if slow_boot_trace {
                eprintln!(
                    "R398_OWNED_BOOT stage=ready-wait-failed mode={mode} elapsed_ms={} pid={} wait={:?} child={:?}",
                    spawn_started.elapsed().as_millis(),
                    peer.child.id(),
                    result.as_ref().err(),
                    peer.child.try_wait()
                );
            }
            // Kill only this owned test process, so its owned pipe reader can
            // retire before we join. Never enumerate or terminate game servers.
            peer.terminate_owned();
        }
        reader.join().expect("owned READY reader retired");
        let line = result
            .expect("bounded Java READY deadline")
            .expect("read Java READY");
        assert!(line.ends_with('\n'), "complete bounded READY line required");
        let port = line
            .trim_end()
            .strip_prefix("READY ")
            .expect("expected READY port only")
            .parse::<u16>()
            .expect("numeric READY port");
        assert_ne!(port, 0);
        peer.endpoint.set_port(port);
        eprintln!("OWNED_PEER_READY mode={mode} elapsed_ms={} pid={} endpoint={}",
            spawn_started.elapsed().as_millis(), peer.child.id(), peer.endpoint);
        peer
    }
    fn expect_successful_exit(&mut self, deadline: &Deadline) {
        let stage_end =
            Instant::now() + deadline.remaining(Duration::from_secs(6), "peer retirement");
        loop {
            if let Some(status) = self.child.try_wait().expect("poll owned peer") {
                // Exit 0 includes Java's source-qualified fresh-read counters,
                // zero failed sessions, and actual Rooms/Presence/World cleanup.
                assert!(
                    status.success(),
                    "world peer rejected the complete flow/cleanup"
                );
                if let Some(job) = &self.process_job {
                    while job.active_processes() != 0 {
                        assert!(
                            Instant::now() < stage_end,
                            "SQL descendant did not retire normally"
                        );
                        deadline.pause("SQL process-tree retirement");
                    }
                }
                return;
            }
            assert!(
                Instant::now() < stage_end,
                "owned Java peer did not retire normally"
            );
            deadline.pause("peer retirement");
        }
    }
    fn stop_dev(&mut self, deadline: &Deadline) {
        let mut control = self.control.take().expect("owned dev operator pipe");
        control.write_all(b"STOP\n").expect("send dev stop");
        control.flush().expect("flush dev stop");
        drop(control);
        let stage_end = Instant::now() + deadline.remaining(Duration::from_secs(30), "dev peer retirement");
        loop {
            if let Some(status) = self.child.try_wait().expect("poll dev peer") {
                assert!(status.success(), "dev peer could not close owned Main/SQL cleanly");
                if let Some(job) = &self.process_job {
                    while job.active_processes() != 0 {
                        assert!(Instant::now() < stage_end, "dev SQL descendant retained");
                        deadline.pause("dev SQL retirement");
                    }
                }
                return;
            }
            assert!(Instant::now() < stage_end, "dev peer stop timed out");
            deadline.pause("dev peer retirement");
        }
    }
    fn terminate_owned(&mut self) {
        // Closing the job retires Java and every SQL descendant even after a
        // panic/READY timeout. No PID search, parent heuristic, or service kill.
        drop(self.process_job.take());
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
impl Drop for JavaPeer {
    fn drop(&mut self) {
        self.terminate_owned();
    }
}

fn explicit_path(variable: &'static str) -> PathBuf {
    let value = PathBuf::from(
        std::env::var_os(variable).expect("explicit world test environment required"),
    );
    assert!(value.is_absolute(), "test paths must be absolute");
    value
}
fn private_root(fixtures: &PathBuf) -> Vec<u8> {
    let file = std::fs::File::open(fixtures.join("root.der")).expect("prepared private CA fixture");
    let mut bytes = Vec::new();
    file.take(65_537)
        .read_to_end(&mut bytes)
        .expect("bounded private CA read");
    assert!(!bytes.is_empty() && bytes.len() <= 65_536);
    bytes
}
fn profile(endpoint: SocketAddr, shared: &PathBuf, root: &[u8]) -> ModernProfile {
    ModernProfile::parse(&serde_json::to_vec(&profile_document(endpoint, shared, root)).unwrap())
        .expect("strict private-CA world profile")
}
/// Both branches run the identical scenario; retain historical regressions
/// while explicitly exercising the same configured entry used by main.rs.
fn mode_controller(
    endpoint: SocketAddr,
    shared: &PathBuf,
    root: &[u8],
    ordinary: bool,
) -> SessionController {
    if ordinary {
        SessionController::new(configured_modern_profile(endpoint, shared, root))
    } else {
        SessionController::new_canonical_development(profile(endpoint, shared, root))
    }
}

fn configured_modern_profile(endpoint: SocketAddr, shared: &PathBuf, root: &[u8]) -> ModernProfile {
    let mut document = profile_document(endpoint, shared, root);
    document["schema"] = serde_json::json!(2);
    document["gameplay_protocol"] = serde_json::json!("modern_v47");
    ModernProfile::parse(&serde_json::to_vec(&document).unwrap())
        .expect("strict schema2 private-CA Modern profile")
}
fn profile_document(endpoint: SocketAddr, shared: &PathBuf, root: &[u8]) -> serde_json::Value {
    serde_json::json!({
        "schema": 1, "profile_id": "world-interop", "display_name": "Owned world TLS test",
        "shared_resource_root": shared, "endpoint": endpoint.to_string(), "tls_server_name": "localhost",
        "trust": { "kind": "private_ca", "certificate_der_base64": base64::engine::general_purpose::STANDARD.encode(root) },
        "connect_timeout_ms": 500, "exchange_timeout_ms": 2000
    })
}
fn wait_owner(
    owner: &mut SessionController,
    deadline: &Deadline,
    stage: &'static str,
    done: impl Fn(&LoginSnapshot) -> bool,
) -> LoginSnapshot {
    loop {
        deadline.check(stage);
        owner.poll();
        let view = owner.snapshot();
        assert_ne!(
            view.phase,
            LoginPhase::Failed,
            "controller failed at {stage}: {}",
            view.status
        );
        if stage == "combat owner LOAD ACK" {
            if let Some(frame) = view.world.frame() {
                assert_ne!(
                    frame.phase,
                    world::Phase::Aborted,
                    "combat LOAD cannot succeed after server abort: {:?}; status {}",
                    frame.abort_reason,
                    view.status
                );
            }
        }
        if done(&view) {
            return view;
        }
        deadline.pause(stage);
    }
}
fn submit(
    owner: &mut SessionController,
    deadline: &Deadline,
    stage: &'static str,
    make: impl Fn() -> LoginCommand,
) {
    // Retry only local admission failure: command() returning Err has not queued
    // a request. Never repeat an accepted command or an uncertain wire exchange.
    let end = Instant::now() + deadline.remaining(Duration::from_millis(500), stage);
    loop {
        deadline.check(stage);
        if owner.command(make()).is_ok() {
            return;
        }
        assert!(Instant::now() < end, "controller did not admit {stage}");
        deadline.pause(stage);
    }
}
fn room_command(
    context: SelectionContext,
    snapshot: &rooms::RoomSnapshot,
    action: rooms::Action,
    value: u8,
) -> rooms::Request {
    rooms::Request::simple(rooms::Common {
        context,
        channel_id: CHANNEL,
        action,
        value,
        room_id: snapshot.summary.room_id,
        expected_revision: snapshot.summary.revision,
    })
    .unwrap()
}
fn actor(frame: &world::Frame, key: CharacterKey) -> &world::Actor {
    frame
        .actors
        .iter()
        .find(|actor| actor.matches_character(key))
        .expect("server-owned actor key")
}
fn rendered(
    runtime: &mut WorldRuntime,
    frame: &world::Frame,
    key: CharacterKey,
) -> Option<Arc<RenderedWorld>> {
    match runtime.poll() {
        RuntimeStatus::Ready(image) if image.revision == frame.revision => {
            assert_eq!(
                (image.match_id, image.generation),
                (frame.match_id, frame.generation)
            );
            assert_eq!(image.selected_entity_id, actor(frame, key).entity_id);
            assert_eq!(image.manifest_digest, frame.manifest.digest);
            assert_eq!((image.width, image.height), (WIDTH, HEIGHT));
            assert_eq!(image.bgra.len(), WIDTH as usize * HEIGHT as usize * 4);
            assert!(
                image.ready_to_ack,
                "only an actual complete render can authorize LOAD ACK"
            );
            Some(image)
        }
        RuntimeStatus::Failed(_) => panic!("actual shared world resource/render worker failed"),
        _ => None,
    }
}
fn world_request(
    context: SelectionContext,
    frame: &world::Frame,
    action: world::Action,
) -> world::Request {
    world::Request {
        action,
        match_id: frame.match_id,
        generation: frame.generation,
        ..world::Request::read(context, CHANNEL)
    }
}

/// Pick a genuinely reachable five-step target from the exact rendered map,
/// independently of Navigation's worker. This fixture search explores at most
/// the 11x11 neighborhood and never assumes an LT map/spawn or actor position.
fn five_step_goal(image: &RenderedWorld, frame: &world::Frame) -> WorldCell {
    let map = image
        .navigation_map()
        .expect("rendered qualified navigation map");
    assert_eq!(map.canonical_digest(), frame.manifest.digest);
    let selected = actor(frame, SUN);
    assert_eq!(image.projection.anchor_cell, selected.position);
    let occupied: BTreeSet<_> = frame
        .actors
        .iter()
        .filter(|a| a.entity_id != selected.entity_id)
        .map(|a| a.position)
        .collect();
    let mut visited = BTreeMap::from([(selected.position, 0usize)]);
    let mut queue = VecDeque::from([selected.position]);
    while let Some(from) = queue.pop_front() {
        let depth = visited[&from];
        if depth == 5 {
            let pixel = image.projection.cell_to_pixel(from).unwrap();
            assert!(pixel.x >= 0 && pixel.x < WIDTH as i32);
            assert!(pixel.y >= 0 && pixel.y < HEIGHT as i32);
            assert_eq!(image.projection.pixel_to_cell(pixel), Some(from));
            return from;
        }
        for (heading, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
            if !world_path::can_step(&map, from, heading as u8, &occupied) {
                continue;
            }
            let to = WorldCell {
                x: from.x.checked_add(dx).unwrap(),
                y: from.y.checked_add(dy).unwrap(),
            };
            if let std::collections::btree_map::Entry::Vacant(entry) = visited.entry(to) {
                entry.insert(depth + 1);
                queue.push_back(to);
            }
        }
        assert!(
            visited.len() <= 121,
            "five-step fixture search stays bounded"
        );
    }
    panic!("real shared spawn neighborhood must provide a five-step route");
}

fn combat_images(
    sun: &mut WorldRuntime,
    moon: &mut WorldRuntime,
    frame: &Arc<combat::Frame>,
    deadline: &Deadline,
) -> Arc<RenderedWorld> {
    combat_images_with_progress(sun, moon, frame, deadline, || {})
}
fn combat_images_with_progress(
    sun: &mut WorldRuntime,
    moon: &mut WorldRuntime,
    frame: &Arc<combat::Frame>,
    deadline: &Deadline,
    mut progress: impl FnMut(),
) -> Arc<RenderedWorld> {
    sun.update_combat(
        Arc::new(frame.world.clone()),
        actor(&frame.world, SUN).entity_id,
        Some(frame.clone()),
    )
    .unwrap();
    moon.update_combat(
        Arc::new(frame.world.clone()),
        actor(&frame.world, MOON).entity_id,
        Some(frame.clone()),
    )
    .unwrap();
    let render_deadline = Instant::now() + Duration::from_secs(12);
    loop {
        assert!(
            Instant::now() < render_deadline,
            "bounded dual render: expected revision={} phase={:?} damage_counts={:?}; sun={:?}; moon={:?}",
            frame.world.revision,
            frame.world.phase,
            frame
                .damages
                .iter()
                .map(|d| d.cast_count)
                .collect::<Vec<_>>(),
            sun.poll(),
            moon.poll()
        );
        deadline.check("two actual atomic combat renders");
        progress();
        if let (Some(sun), Some(moon)) = (
            rendered(sun, &frame.world, SUN),
            rendered(moon, &frame.world, MOON),
        ) {
            assert_eq!(
                sun.projection.anchor_cell,
                actor(&frame.world, SUN).position
            );
            assert_eq!(
                moon.projection.anchor_cell,
                actor(&frame.world, MOON).position
            );
            return sun;
        }
        deadline.pause("two actual atomic combat renders");
    }
}

#[test]
#[ignore = "requires owned Java combat TLS peer and real shared Lineage assets; no DB/GUI/native process"]
fn java_combat_controller_authoritative_attack_hurt_death_atomic_render_and_normal_retirement() {
    combat_controller_flow(false, false, false, false, false);
}

#[test]
#[ignore = "requires owned Java combat TLS peer and real shared armed SPR/WAV; no DB/GUI/audio device"]
fn java_armed_combat_controller_qualified_weapon_sound_and_normal_retirement() {
    combat_controller_flow(true, false, false, false, false);
}

#[test]
#[ignore = "requires owned v3 Java settlement peer and real shared assets; writer is in-memory fixture, no SQL/GUI"]
fn java_combat_deathmatch_result_saved_and_both_owners_return_to_same_room() {
    combat_controller_flow(false, true, false, false, false);
}

#[test]
#[ignore = "requires owned v4 Java TLS peer and actual shared teleport SPR/WAV; synthetic learned rows, no SQL/GUI/device"]
fn java_learned_teleport_controller_atomic_mp_position_shared_effect_then_move_melee_result_return()
{
    combat_controller_flow(false, true, true, false, false);
}

#[test]
#[ignore = "requires owned v5 Java peer, explicit MySQL installation and real shared assets; fresh isolated SQL, no existing DB/GUI"]
fn java_real_mysql_battle_both_clients_read_distinct_saved_totals_and_replay_is_idempotent() {
    combat_controller_flow(false, true, true, true, false);
}

#[test]
#[ignore = "requires owned v6 Java peer, fresh isolated MySQL and shared light-arrow SPR/WAV; no production DB/GUI/device"]
fn java_light_arrow_after_teleport_shared_native_effect_magic_lethal_and_real_mysql_records() {
    combat_controller_flow(false, true, true, true, true);
}

fn combat_controller_flow(armed: bool, settlement: bool, spells: bool, records: bool, magic: bool) {
    let deadline = Deadline(Instant::now() + Duration::from_secs(60));
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernCombatTestPeer",
        if magic {
            "magic"
        } else if records {
            "records"
        } else if spells {
            "teleport"
        } else if settlement {
            "settlement"
        } else if armed {
            "armed"
        } else {
            "normal"
        },
    );
    let mut owner = SessionController::new(profile(peer.endpoint, &shared, &ca));
    submit(&mut owner, &deadline, "combat connect", || {
        LoginCommand::Connect
    });
    wait_owner(&mut owner, &deadline, "combat TLS ready", |v| {
        v.phase == LoginPhase::LoginReady
    });
    submit(&mut owner, &deadline, "combat AUTH", || {
        LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        }
    });
    wait_owner(&mut owner, &deadline, "combat sources", |v| {
        v.phase == LoginPhase::ChoosingSource
    });
    submit(&mut owner, &deadline, "combat source", || {
        LoginCommand::SelectSource(7)
    });
    let roster = wait_owner(&mut owner, &deadline, "combat roster", |v| {
        v.phase == LoginPhase::CharactersReady
    });
    assert_eq!(roster.characters.len(), 1);
    let context = SelectionContext {
        key: SUN,
        epoch: roster.roster_epoch,
    };
    submit(&mut owner, &deadline, "combat selection", || {
        LoginCommand::SelectCharacter(context)
    });
    wait_owner(&mut owner, &deadline, "combat channels", |v| {
        v.phase == LoginPhase::ChoosingChannel
    });
    if records {
        let initial = wait_owner(
            &mut owner,
            &deadline,
            "actual SQL absent owner totals",
            |v| {
                v.record.context() == Some(context)
                    && v.record.totals().is_some()
                    && !v.record_pending
            },
        );
        assert_eq!(
            initial.record.totals(),
            Some(record::Totals {
                stored: false,
                wins: 0,
                losses: 0,
                revision: 0,
            })
        );
    }
    submit(&mut owner, &deadline, "combat lobby", || {
        LoginCommand::EnterChannel {
            context,
            channel_id: CHANNEL,
        }
    });
    let lobby = wait_owner(&mut owner, &deadline, "combat lobby ACK", |v| {
        v.phase == LoginPhase::InLobby
    });
    assert!(lobby.combat_supported);
    assert!(lobby.combat.is_none());
    let mut guest = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(&ca),
        Hello::current(100),
        TransportLimits {
            connect_timeout: Duration::from_millis(500),
            total_timeout: Duration::from_secs(2),
            io_poll_interval: Duration::from_millis(25),
        },
        Cancellation::new(),
    )
    .unwrap();
    assert!(guest.supports_combat());
    assert_eq!(guest.ack().unwrap().capabilities, 255);
    guest
        .authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    let rows = guest.characters(42, 1).unwrap();
    assert_eq!(rows.characters[0].key, MOON);
    let moon_context = SelectionContext {
        key: MOON,
        epoch: 1,
    };
    guest.select_character(moon_context).unwrap();
    if records {
        assert!(guest.supports_record());
        let initial = guest.record().unwrap();
        assert_eq!(initial.context, moon_context);
        assert_eq!(initial.status, record::Status::Ok);
        assert_eq!(
            initial.totals,
            Some(record::Totals {
                stored: false,
                wins: 0,
                losses: 0,
                revision: 0,
            })
        );
    }
    if spells {
        assert_eq!(
            guest.ack().unwrap().version,
            if magic {
                6
            } else if records {
                5
            } else {
                4
            }
        );
        let empty = guest.skills().unwrap();
        assert_eq!(empty.status, spell::Status::Ok);
        assert!(
            empty.skills.is_empty(),
            "same object ID in the other source has not learned teleport"
        );
        let learned = wait_owner(&mut owner, &deadline, "actual source learned skills", |v| {
            v.skills.teleport().is_some() && !v.spell_pending
        });
        assert_eq!(learned.skills.teleport().unwrap().mp_cost, 10);
        if magic {
            assert_eq!(learned.skills.light_arrow().unwrap().mp_cost, 3);
        }
    }
    guest.enter_channel(CHANNEL).unwrap();
    submit(&mut owner, &deadline, "combat CREATE", || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(context, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "Combat TLS interop".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    let created = wait_owner(&mut owner, &deadline, "combat creator JOIN", |v| {
        !v.room_pending && v.rooms.joined().is_some()
    });
    let room = created.rooms.joined().unwrap();
    assert_eq!(room.owner, SUN);
    let directory = guest
        .rooms(rooms::Request::read(moon_context, CHANNEL).unwrap())
        .unwrap();
    let summary = directory
        .rooms
        .iter()
        .find(|r| r.room_id == room.summary.room_id)
        .unwrap();
    let joined = guest
        .rooms(
            rooms::Request::join(
                rooms::Common {
                    context: moon_context,
                    channel_id: CHANNEL,
                    action: rooms::Action::Join,
                    value: 0,
                    room_id: summary.room_id,
                    expected_revision: summary.revision,
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(joined.status, rooms::Status::Ok);
    assert_eq!(joined.joined.unwrap().members.len(), 2);
    let seen = wait_owner(&mut owner, &deadline, "combat guest refresh", |v| {
        v.rooms.joined().is_some_and(|r| r.members.len() == 2)
    });
    let room = seen.rooms.joined().unwrap().clone();
    submit(&mut owner, &deadline, "combat owner READY", || {
        LoginCommand::Rooms(room_command(context, &room, rooms::Action::Ready, 1))
    });
    wait_owner(&mut owner, &deadline, "combat owner READY ACK", |v| {
        !v.room_pending
            && v.rooms.joined().is_some_and(|r| {
                r.members
                    .iter()
                    .any(|m| m.matches_character(SUN) && m.ready)
            })
    });
    let room = guest
        .rooms(rooms::Request::read(moon_context, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    assert!(
        guest
            .rooms(room_command(moon_context, &room, rooms::Action::Ready, 1))
            .unwrap()
            .joined
            .unwrap()
            .summary
            .satisfied
    );
    let loading = wait_owner(
        &mut owner,
        &deadline,
        "combat PREPARE qualification and atomic Loading",
        |v| {
            v.combat
                .as_ref()
                .is_some_and(|f| f.world.phase == world::Phase::Loading)
        },
    )
    .combat
    .unwrap();
    let expected_hp = if magic {
        40
    } else if armed {
        60
    } else {
        20
    };
    let expected_weapon = if armed { 1 } else { 0 };
    assert!(loading.vitals.iter().all(|v| v.hp == expected_hp
        && v.max_hp == expected_hp
        && v.weapon_type == expected_weapon
        && v.attack_sequence == 0
        && v.attack_revision == 0
        && v.death_revision == 0));
    if magic {
        assert!(loading
            .vitals
            .iter()
            .all(|v| v.mp == 100 && v.max_mp == 100));
        assert_eq!(loading.damages.len(), 2);
        assert!(loading
            .damages
            .iter()
            .all(|d| d.cast_count == 0 && d.mp_spent == 0 && d.mp_cost == 3));
    }
    guest
        .world(world_request(
            moon_context,
            &loading.world,
            world::Action::Read,
        ))
        .unwrap();
    let moon_loading = guest
        .combat(combat::Request::read(moon_context, CHANNEL, &loading.world))
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(moon_loading, *loading);
    let mut sun_runtime = WorldRuntime::new(shared.clone()).unwrap();
    let mut moon_runtime = WorldRuntime::new(shared).unwrap();
    let first_render_started = Instant::now();
    let mut keepalive = Instant::now();
    combat_images_with_progress(
        &mut sun_runtime,
        &mut moon_runtime,
        &loading,
        &deadline,
        || {
            owner.poll();
            let latest = owner.snapshot();
            assert_ne!(
                latest.phase,
                LoginPhase::Failed,
                "owner failed while loading: {}",
                latest.status
            );
            assert!(
                latest
                    .world
                    .frame()
                    .is_some_and(|frame| frame.phase == world::Phase::Loading),
                "combat first render lost Loading: {:?}",
                latest.world.frame().map(|f| (f.phase, f.abort_reason))
            );
            if keepalive.elapsed() >= Duration::from_secs(1) {
                guest
                    .ping()
                    .expect("guest remains connected during actual resource loading");
                keepalive = Instant::now();
            }
        },
    );
    eprintln!(
        "combat initial dual render {:?}",
        first_render_started.elapsed()
    );
    let sun_loaded = world::Request {
        manifest_digest: loading.world.manifest.digest,
        ..world_request(context, &loading.world, world::Action::Loaded)
    };
    submit(&mut owner, &deadline, "combat rendered LOAD", || {
        LoginCommand::World(sun_loaded)
    });
    wait_owner(&mut owner, &deadline, "combat owner LOAD ACK", |v| {
        !v.world_pending && v.world.frame().is_some_and(|f| actor(f, SUN).loaded)
    });
    let countdown_started = Instant::now();
    let countdown = guest
        .world(world::Request {
            manifest_digest: loading.world.manifest.digest,
            ..world_request(moon_context, &loading.world, world::Action::Loaded)
        })
        .unwrap();
    assert_eq!(countdown.frame.unwrap().phase, world::Phase::Countdown);
    let mut current = wait_owner(&mut owner, &deadline, "combat actual countdown", |v| {
        v.combat
            .as_ref()
            .is_some_and(|f| f.world.phase == world::Phase::Active)
    })
    .combat
    .unwrap();
    assert!(countdown_started.elapsed() >= Duration::from_secs(3));
    let mut initial_image = combat_images(&mut sun_runtime, &mut moon_runtime, &current, &deadline);
    assert!(initial_image
        .audio
        .as_ref()
        .expect("qualified shared sound bank")
        .events
        .is_empty());
    let sun_entity = actor(&current.world, SUN).entity_id;
    let moon_entity = actor(&current.world, MOON).entity_id;
    if spells {
        assert_eq!(current.world.casts.len(), 2);
        let before = current.clone();
        let from = actor(&before.world, SUN).position;
        submit(
            &mut owner,
            &deadline,
            "targetless teleport admission",
            || LoginCommand::CastTeleport,
        );
        assert!(owner.snapshot().spell_pending);
        assert_eq!(owner.snapshot().combat.as_ref(), Some(&before));
        current = wait_owner(&mut owner, &deadline, "atomic teleport ACK", |v| {
            !v.spell_pending
                && v.combat
                    .as_ref()
                    .is_some_and(|f| f.world.cast(sun_entity).is_some_and(|c| c.cast_count == 1))
        })
        .combat
        .unwrap();
        let cast = *current.world.cast(sun_entity).unwrap();
        assert_eq!(cast.request_sequence, 1);
        assert_eq!(cast.cast_revision, current.world.revision);
        assert_eq!(cast.from, from);
        assert_eq!(cast.to, actor(&current.world, SUN).position);
        assert_ne!(cast.to, from);
        assert_eq!(
            actor(&current.world, SUN).sequence,
            actor(&before.world, SUN).sequence
        );
        assert_eq!(
            current.vital(sun_entity).unwrap().mp,
            before.vital(sun_entity).unwrap().mp - 10
        );
        assert_eq!(
            current.vital(sun_entity).unwrap().hp,
            before.vital(sun_entity).unwrap().hp
        );
        assert_eq!(current.vital(sun_entity).unwrap().attack_sequence, 0);
        let other = guest
            .combat(combat::Request::read(moon_context, CHANNEL, &current.world))
            .unwrap()
            .frame
            .unwrap();
        assert_eq!(
            other, *current,
            "other source observes the same MP/teleport atomically"
        );
        let teleported_image =
            combat_images(&mut sun_runtime, &mut moon_runtime, &current, &deadline);
        assert_eq!(teleported_image.projection.anchor_cell, cast.to);
        assert_ne!(teleported_image.bgra.as_ref(), initial_image.bgra.as_ref());
        let visual_until =
            Instant::now() + Duration::from_millis(u64::from(cast.cast_interval_millis).max(800));
        let mut sound119 = false;
        while Instant::now() < visual_until {
            if let Some(image) = rendered(&mut sun_runtime, &current.world, SUN) {
                sound119 |= image.audio.as_ref().is_some_and(|audio| {
                    audio
                        .events
                        .iter()
                        .any(|event| event.group.candidates().contains(&119))
                });
            }
            owner.poll();
            deadline.pause("shared teleport effect and sound cue without device");
        }
        assert!(
            sound119,
            "actual shared effect169 must produce sound119 exactly through the common bounded audio sequencer"
        );
        let denied = guest
            .cast(spell::CastRequest {
                context: moon_context,
                channel_id: CHANNEL,
                match_id: current.world.match_id,
                generation: current.world.generation,
                sequence: 1,
                skill_id: 5,
                target_entity_id: 0,
            })
            .unwrap();
        assert_eq!(denied.status, spell::Status::Denied);
        assert_eq!(denied.reason, spell::Reason::Unlearned);
        let denied_frame = denied.frame.unwrap();
        assert_eq!(
            denied_frame
                .world
                .cast(moon_entity)
                .unwrap()
                .request_sequence,
            1
        );
        assert_eq!(denied_frame.world.cast(moon_entity).unwrap().cast_count, 0);
        assert_eq!(denied_frame.vitals, current.vitals);
        current = wait_owner(&mut owner, &deadline, "denied cast READ propagation", |v| {
            v.combat
                .as_ref()
                .is_some_and(|f| f.world.revision >= denied_frame.world.revision)
        })
        .combat
        .unwrap();
        let heading = DIRECTIONS
            .iter()
            .position(|&(dx, dy)| cast.to.x + dx == from.x && cast.to.y + dy == from.y)
            .expect("server RNG fixture selects one legitimate adjacent destination")
            as u8;
        let movement = world::Request {
            heading,
            sequence: actor(&current.world, SUN).sequence + 1,
            from_x: cast.to.x,
            from_y: cast.to.y,
            ..world_request(context, &current.world, world::Action::Move)
        };
        submit(
            &mut owner,
            &deadline,
            "ordinary MOVE after teleport",
            || LoginCommand::World(movement),
        );
        current = wait_owner(
            &mut owner,
            &deadline,
            "MOVE remains usable after teleport",
            |v| {
                !v.world_pending
                    && v.combat
                        .as_ref()
                        .is_some_and(|f| actor(&f.world, SUN).sequence == movement.sequence)
            },
        )
        .combat
        .unwrap();
        assert_eq!(actor(&current.world, SUN).position, from);
        assert_eq!(current.world.cast(sun_entity).unwrap(), &cast);
        initial_image = combat_images(&mut sun_runtime, &mut moon_runtime, &current, &deadline);
        // A successful MOVE still imposes the original step interval on melee.
        let after_move =
            Instant::now() + Duration::from_nanos(actor(&current.world, SUN).step_interval_nanos);
        while Instant::now() < after_move {
            deadline.pause("original cross-action MOVE pace");
        }
    }
    let start_positions = current
        .world
        .actors
        .iter()
        .map(|a| a.position)
        .collect::<Vec<_>>();
    let mut attacks = 0;
    let mut next_allowed = Instant::now();
    let mut last_image = initial_image.clone();
    let mut sound_sequence = 0;
    let mut light_arrow_sound = false;
    // Real server RNG and one-second authority pacing. Bound attempts as well
    // as the complete test deadline; no client HP arithmetic or forced outcome.
    while current.vital(moon_entity).unwrap().hp > 0 {
        assert!(
            attacks < 24,
            "24 correctly paced server attacks did not reach death"
        );
        while Instant::now() < next_allowed {
            deadline.pause("server attack interval");
        }
        let before = current.clone();
        let sequence = if magic {
            before.world.cast(sun_entity).unwrap().request_sequence + 1
        } else {
            before.vital(sun_entity).unwrap().attack_sequence + 1
        };
        submit(&mut owner, &deadline, "combat ATTACK admission", || {
            if magic {
                LoginCommand::CastLightArrow {
                    target_entity_id: moon_entity,
                }
            } else {
                LoginCommand::CombatAttack {
                    target_entity_id: moon_entity,
                }
            }
        });
        let pending = owner.snapshot();
        assert!(if magic {
            pending.spell_pending
        } else {
            pending.combat_pending
        });
        assert_eq!(pending.combat.as_ref(), Some(&before));
        assert_eq!(
            pending
                .combat
                .as_ref()
                .unwrap()
                .vital(moon_entity)
                .unwrap()
                .hp,
            before.vital(moon_entity).unwrap().hp
        );
        let confirmed = wait_owner(
            &mut owner,
            &deadline,
            "combat authoritative ATTACK ACK",
            |v| {
                !v.combat_pending
                    && !v.spell_pending
                    && v.combat.as_ref().is_some_and(|f| {
                        if magic {
                            f.world.cast(sun_entity).unwrap().request_sequence == sequence
                        } else {
                            f.vital(sun_entity).unwrap().attack_sequence == sequence
                        }
                    })
            },
        );
        current = confirmed.combat.unwrap();
        let attacker = current.vital(sun_entity).unwrap();
        let target = current.vital(moon_entity).unwrap();
        if magic {
            let cast = &current.damages[0];
            assert!(
                target.hp < before.vital(moon_entity).unwrap().hp,
                "balanced level80 magic fixture must cause actual server damage; immunity is tested separately"
            );
            let prior = &before.damages[0];
            assert_eq!(cast.cast_count, prior.cast_count + 1);
            // The controller may already have received the asynchronous SAVED
            // receipt after validating the exact lethal CAST ACK. That receipt
            // advances world revision only, never the original damage revision.
            let damage_revision = current
                .world
                .completion
                .as_ref()
                .map_or(current.world.revision, |c| c.event.engine_revision + 1);
            assert_eq!(cast.cast_revision, damage_revision as u64);
            assert_eq!(cast.skill_id, 4);
            assert_eq!(cast.target_entity_id, moon_entity);
            assert_eq!(cast.from, actor(&before.world, SUN).position);
            assert_eq!(cast.to, actor(&before.world, MOON).position);
            assert_eq!(cast.mp_cost, 3);
            assert_eq!(attacker.mp, before.vital(sun_entity).unwrap().mp - 3);
            assert_eq!(attacker.attack_sequence, 0);
            assert_eq!(attacker.attack_revision, 0);
            assert_eq!(attacker.last_target_entity_id, 0);
            let previous_teleport = before.world.cast(sun_entity).unwrap();
            let teleport = current.world.cast(sun_entity).unwrap();
            assert_eq!(teleport.cast_count, 1);
            assert_eq!(teleport.cast_revision, previous_teleport.cast_revision);
            assert_eq!(teleport.mp_spent, 10);
            assert_eq!(teleport.from, previous_teleport.from);
            assert_eq!(teleport.to, previous_teleport.to);
            assert_eq!(attacker.max_mp - attacker.mp, 10 + cast.mp_spent as i64);
        } else {
            assert_eq!(attacker.attack_sequence, sequence);
            assert!(attacker.attack_revision > before.vital(sun_entity).unwrap().attack_revision);
            assert_eq!(attacker.last_target_entity_id, moon_entity);
        }
        assert!(target.hp <= before.vital(moon_entity).unwrap().hp);
        if target.hp < before.vital(moon_entity).unwrap().hp {
            assert!(target.hurt_revision > before.vital(moon_entity).unwrap().hurt_revision);
        }
        assert_eq!(
            current
                .world
                .actors
                .iter()
                .map(|a| a.position)
                .collect::<Vec<_>>(),
            start_positions
        );
        assert!(current.world.revision > before.world.revision);
        assert_eq!((target.hp == 0), (target.death_revision > 0));
        let other = guest
            .combat(combat::Request::read(moon_context, CHANNEL, &current.world))
            .unwrap()
            .frame
            .unwrap();
        if settlement && target.hp == 0 {
            assert_eq!(other.world.phase, world::Phase::Finished);
            assert_eq!(
                other.world.completion.as_ref().unwrap().event,
                current.world.completion.as_ref().unwrap().event
            );
            assert_eq!(other.vitals, current.vitals);
            assert_eq!(other.damages, current.damages);
            assert_eq!(other.world.actors, current.world.actors);
            assert!(other.world.revision >= current.world.revision);
        } else {
            assert_eq!(
                other, *current,
                "other TLS owner sees identical world and vitals commit"
            );
        }
        next_allowed = Instant::now()
            + if magic {
                Duration::from_millis(u64::from(
                    current.world.cast(sun_entity).unwrap().cast_interval_millis,
                ))
            } else {
                Duration::from_nanos(attacker.attack_interval_nanos)
            };
        last_image = combat_images(&mut sun_runtime, &mut moon_runtime, &current, &deadline);
        let sound_deadline =
            Instant::now() + Duration::from_millis(if magic { 1_200 } else { 450 });
        while Instant::now() < sound_deadline {
            if let Some(image) = rendered(&mut sun_runtime, &current.world, SUN) {
                let audio = image
                    .audio
                    .as_ref()
                    .expect("same world has qualified shared sounds");
                for event in audio.events.iter() {
                    sound_sequence = sound_sequence.max(event.sequence);
                    light_arrow_sound |= event.group.candidates().contains(&97);
                    assert!(event.group.candidates().iter().all(|id| *id < 10_000));
                }
                last_image = image;
            }
            deadline.pause("qualified combat animation sound events (no device)");
        }
        attacks += 1;
    }
    assert!(
        attacks >= 2,
        "fixture must exercise more than one authoritative attack"
    );
    assert!(last_image.bgra.as_ref() != initial_image.bgra.as_ref());
    assert!(
        sound_sequence > 0,
        "actual shared animation must deliver sound cues"
    );
    if magic {
        assert!(
            light_arrow_sound,
            "shared 167 launch must emit sound97 through the bounded sequencer"
        );
    }
    let dead = current.vital(moon_entity).unwrap().clone();
    assert_eq!(dead.hp, 0);
    assert!(dead.hurt_revision > 0 && dead.death_revision > 0);
    let dead_actor = actor(&current.world, MOON).clone();
    let denied_move = guest
        .world(world::Request {
            heading: 2,
            sequence: dead_actor.sequence + 1,
            from_x: dead_actor.position.x,
            from_y: dead_actor.position.y,
            ..world_request(moon_context, &current.world, world::Action::Move)
        })
        .unwrap();
    assert_eq!(denied_move.status, world::Status::Denied);
    assert_eq!(
        actor(denied_move.frame.as_ref().unwrap(), MOON).position,
        dead_actor.position
    );
    let dead_refresh = guest
        .combat(combat::Request::read(
            moon_context,
            CHANNEL,
            denied_move.frame.as_ref().unwrap(),
        ))
        .unwrap()
        .frame
        .unwrap();
    let denied_attack = guest
        .combat(combat::Request {
            action: combat::Action::Attack,
            sequence: dead_refresh.vital(moon_entity).unwrap().attack_sequence + 1,
            target_entity_id: sun_entity,
            ..combat::Request::read(moon_context, CHANNEL, &dead_refresh.world)
        })
        .unwrap();
    assert_eq!(denied_attack.status, combat::Status::Denied);
    assert_eq!(
        denied_attack.reason,
        if settlement {
            combat::Reason::NotActive
        } else {
            combat::Reason::Dead
        }
    );
    let terminal = denied_attack.frame.unwrap();
    assert_eq!(terminal.vital(moon_entity).unwrap().hp, 0);
    assert_eq!(
        terminal.vital(moon_entity).unwrap().attack_revision,
        dead.attack_revision
    );
    let seen = wait_owner(
        &mut owner,
        &deadline,
        "owner sees dead player's denied mutation",
        |v| {
            v.combat
                .as_ref()
                .is_some_and(|f| f.world.revision >= terminal.world.revision)
        },
    );
    assert_eq!(
        seen.combat.as_ref().unwrap().vital(moon_entity).unwrap().hp,
        0
    );
    if settlement {
        let saved = wait_owner(
            &mut owner,
            &deadline,
            "confirmed result writer completion",
            |v| {
                v.world.frame().is_some_and(|f| {
                    f.phase == world::Phase::Finished
                        && f.completion
                            .as_ref()
                            .is_some_and(|c| c.state == world::SettlementState::Saved)
                })
            },
        );
        let completion = saved.world.frame().unwrap().completion.as_ref().unwrap();
        assert_eq!(completion.event.participants.len(), 2);
        assert_eq!(completion.event.participants[0].key, SUN);
        assert_eq!(completion.event.participants[1].key, MOON);
        assert_eq!(
            completion.event.participants[0].verdict,
            world::Verdict::Win
        );
        assert_eq!(
            completion.event.participants[1].verdict,
            world::Verdict::Loss
        );
        assert_eq!(
            completion.event,
            terminal.world.completion.as_ref().unwrap().event
        );
    }
    let leave = world_request(context, &terminal.world, world::Action::Leave);
    submit(&mut owner, &deadline, "combat LEAVE", || {
        LoginCommand::World(leave)
    });
    let left = wait_owner(&mut owner, &deadline, "combat LEAVE ACK", |v| {
        !v.world_pending && v.world.frame().is_none()
    });
    assert!(left.combat.is_none());
    assert!(!left.combat_pending);
    let ended = guest
        .world(world_request(
            moon_context,
            &terminal.world,
            world::Action::Read,
        ))
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(
        ended.phase,
        if settlement {
            world::Phase::Finished
        } else {
            world::Phase::Aborted
        }
    );
    if settlement {
        assert_eq!(
            ended.completion.as_ref().unwrap().state,
            world::SettlementState::Saved
        );
        assert_eq!(
            ended.completion.as_ref().unwrap().event,
            terminal.world.completion.as_ref().unwrap().event
        );
    }
    guest
        .world(world_request(moon_context, &ended, world::Action::Leave))
        .unwrap();
    if settlement {
        let returned = wait_owner(
            &mut owner,
            &deadline,
            "returned room ready flags reset",
            |v| v.world.frame().is_none() && v.rooms.joined().is_some_and(|r| r.summary.ready == 0),
        );
        let room = returned.rooms.joined().unwrap();
        assert_eq!(room.summary.room_id, ended.room.summary.room_id);
        assert_eq!(room.members.len(), 2);
        assert!(!room.summary.satisfied);
    }
    if records {
        let expected = record::Totals {
            stored: true,
            wins: 1,
            losses: 0,
            revision: 1,
        };
        let persisted = wait_owner(
            &mut owner,
            &deadline,
            "client reads committed SQL victory after return",
            |v| {
                v.record.context() == Some(context)
                    && v.record.totals() == Some(expected)
                    && !v.record_pending
            },
        );
        assert!(matches!(
            persisted.record,
            record::State::Ready {
                refresh_failure: None,
                ..
            }
        ));
        let loser = guest.record().unwrap();
        assert_eq!(loser.context, moon_context);
        assert_eq!(loser.status, record::Status::Ok);
        assert_eq!(
            loser.totals,
            Some(record::Totals {
                stored: true,
                wins: 0,
                losses: 1,
                revision: 1
            })
        );
        submit(
            &mut owner,
            &deadline,
            "manual persisted totals refresh",
            || LoginCommand::ReadRecord,
        );
        let refreshed = wait_owner(&mut owner, &deadline, "manual SQL refresh ACK", |v| {
            !v.record_pending
        });
        assert_eq!(
            refreshed.record.totals(),
            Some(expected),
            "replayed UUID must not count twice"
        );
    }
    sun_runtime.shutdown();
    moon_runtime.shutdown();
    while !sun_runtime.is_quiescent() || !moon_runtime.is_quiescent() {
        sun_runtime.poll();
        moon_runtime.poll();
        deadline.pause("combat render retirement");
    }
    guest.close().unwrap();
    submit(&mut owner, &deadline, "combat normal shutdown", || {
        LoginCommand::Shutdown
    });
    while !owner.is_quiescent() {
        owner.poll();
        deadline.pause("combat TLS owner retirement");
    }
    assert_eq!(owner.snapshot().phase, LoginPhase::Disconnected);
    peer.expect_successful_exit(&deadline);
    deadline.check("complete combat flow");
}

/// One human owns this real renderer; bots are authority participants, never a
/// fake TLS/SQL character owner. The matching frame includes every bot pixel.
fn bot_combat_image(
    runtime: &mut WorldRuntime,
    frame: &Arc<combat::Frame>,
    owner: &mut SessionController,
    deadline: &Deadline,
) -> Arc<RenderedWorld> {
    runtime
        .update_combat(
            Arc::new(frame.world.clone()),
            actor(&frame.world, SUN).entity_id,
            Some(frame.clone()),
        )
        .unwrap();
    let end = Instant::now()
        + deadline.remaining(Duration::from_secs(12), "single human bot-world render");
    loop {
        owner.poll();
        assert_ne!(
            owner.snapshot().phase,
            LoginPhase::Failed,
            "bot-world controller must stay connected"
        );
        if let Some(image) = rendered(runtime, &frame.world, SUN) {
            assert_eq!(
                image.projection.anchor_cell,
                actor(&frame.world, SUN).position
            );
            assert_eq!(image.combat.as_deref(), Some(frame.as_ref()));
            assert_eq!(image.actors.len(), frame.world.actors.len());
            return image;
        }
        assert!(
            Instant::now() < end,
            "first real bot-world render did not finish within its bounded stage"
        );
        deadline.pause("single human bot-world render");
    }
}

#[test]
#[ignore = "requires owned v7 GM Java peer, fresh isolated MySQL and real shared assets; no existing DB/UI/audio device"]
fn java_tls_gm_bot_cap_clear_readd_real_world_magic_result_and_human_only_totals() {
    use super::rooms::MemberKind;
    let deadline = Deadline(Instant::now() + Duration::from_secs(60));
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start_mode(
        &fixtures,
        &deadline,
        "local.lito381.server.ModernCombatTestPeer",
        "bots",
    );
    let mut owner = SessionController::new(profile(peer.endpoint, &shared, &ca));
    submit(&mut owner, &deadline, "bot fixture connect", || {
        LoginCommand::Connect
    });
    wait_owner(&mut owner, &deadline, "bot fixture TLS", |v| {
        v.phase == LoginPhase::LoginReady
    });
    submit(&mut owner, &deadline, "bot fixture AUTH", || {
        LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        }
    });
    wait_owner(&mut owner, &deadline, "bot fixture sources", |v| {
        v.phase == LoginPhase::ChoosingSource
    });
    submit(&mut owner, &deadline, "bot fixture select source", || {
        LoginCommand::SelectSource(7)
    });
    let roster = wait_owner(&mut owner, &deadline, "bot fixture roster", |v| {
        v.phase == LoginPhase::CharactersReady
    });
    assert_eq!(roster.characters.len(), 1);
    assert_eq!(
        (
            roster.characters[0].source_id,
            roster.characters[0].object_id
        ),
        (SUN.source_id, SUN.object_id)
    );
    let context = SelectionContext {
        key: SUN,
        epoch: roster.roster_epoch,
    };
    submit(&mut owner, &deadline, "bot fixture select human", || {
        LoginCommand::SelectCharacter(context)
    });
    wait_owner(&mut owner, &deadline, "bot fixture channel list", |v| {
        v.phase == LoginPhase::ChoosingChannel
    });
    let absent = wait_owner(
        &mut owner,
        &deadline,
        "bot fixture initial SQL record",
        |v| v.record.context() == Some(context) && v.record.totals().is_some() && !v.record_pending,
    );
    assert_eq!(
        absent.record.totals(),
        Some(record::Totals {
            stored: false,
            wins: 0,
            losses: 0,
            revision: 0
        })
    );
    submit(&mut owner, &deadline, "bot fixture lobby", || {
        LoginCommand::EnterChannel {
            context,
            channel_id: CHANNEL,
        }
    });
    let lobby = wait_owner(&mut owner, &deadline, "bot fixture lobby ACK", |v| {
        v.phase == LoginPhase::InLobby
    });
    assert!(lobby.gm_supported && lobby.combat_supported);
    assert!(lobby.skills.teleport().is_some() && lobby.skills.light_arrow().is_some());
    submit(&mut owner, &deadline, "bot fixture CREATE", || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(context, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 2,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "GM bot typed TLS interop".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    let created = wait_owner(&mut owner, &deadline, "bot fixture own room", |v| {
        !v.room_pending && v.rooms.joined().is_some()
    });
    let room_id = created.rooms.joined().unwrap().summary.room_id;
    assert_eq!(created.rooms.joined().unwrap().members.len(), 1);
    let mut bot_id = 0;
    for (count, expected_count, expected_id) in [(20, 2, Some(1)), (0, 1, None), (1, 2, Some(2))] {
        let before = owner.snapshot();
        let before_room = before.rooms.joined().unwrap();
        let revision = before_room.summary.revision;
        submit(
            &mut owner,
            &deadline,
            "server-authorized GM command",
            || LoginCommand::GmBots { count },
        );
        assert!(owner.snapshot().room_pending);
        assert_eq!(
            owner.snapshot().rooms,
            before.rooms,
            "no optimistic fake member insertion"
        );
        let next = wait_owner(&mut owner, &deadline, "GM tagged result ACK", |v| {
            !v.room_pending
        });
        let room = next
            .rooms
            .joined()
            .expect("GM operation retains human room");
        assert_eq!(room.summary.room_id, room_id);
        assert_eq!(
            room.summary.revision,
            revision + 1,
            "one batch, one authoritative revision"
        );
        assert_eq!(room.members.len(), expected_count, "{}", next.status);
        assert_eq!(room.owner, SUN);
        assert_eq!(room.member(SUN).unwrap().kind, MemberKind::Human);
        assert!(!room.member(SUN).unwrap().ready);
        if let Some(id) = expected_id {
            let bot = &room.members[1];
            assert_eq!(bot.kind, MemberKind::ServerBot);
            assert_eq!((bot.key.source_id, bot.key.object_id), (7, id));
            assert!(bot.name.starts_with("BOT"));
            assert!(!bot.source_name.is_empty());
            assert_eq!(bot.team, 1);
            assert!(bot.ready);
            assert!(
                room.member(bot.key).is_none(),
                "numeric bot DTO is not a human context"
            );
            bot_id = id;
        } else {
            assert!(room.members.iter().all(|m| m.kind == MemberKind::Human));
        }
    }
    let room = owner.snapshot().rooms.joined().unwrap().clone();
    submit(
        &mut owner,
        &deadline,
        "human READY with server-owned bot",
        || LoginCommand::Rooms(room_command(context, &room, rooms::Action::Ready, 1)),
    );
    let loading = wait_owner(
        &mut owner,
        &deadline,
        "bot-world automatic PREPARE/qualified Loading",
        |v| {
            if let Some(f) = v.world.frame() {
                assert_ne!(f.phase, world::Phase::Aborted, "{}", v.status);
            }
            v.combat
                .as_ref()
                .is_some_and(|f| f.world.phase == world::Phase::Loading)
        },
    )
    .combat
    .unwrap();
    assert_eq!(loading.world.actors.len(), 2);
    let human = actor(&loading.world, SUN).entity_id;
    let bot = &loading.world.actors[1];
    let bot_entity = bot.entity_id;
    assert_eq!(bot.kind, MemberKind::ServerBot);
    assert_eq!(bot.key.object_id, bot_id);
    assert!(!bot.matches_character(bot.key));
    assert_eq!(loading.vital(human).unwrap().max_hp, 40);
    assert_eq!(loading.vital(human).unwrap().max_mp, 100);
    assert_eq!(loading.vital(bot_entity).unwrap().hp, 40);
    assert_eq!(loading.vital(bot_entity).unwrap().max_mp, 0);
    assert!(
        bot.loaded,
        "server qualifies bot without fabricating a second TLS owner"
    );
    assert!(!actor(&loading.world, SUN).loaded);
    assert_eq!(loading.world.casts.len(), 2);
    assert_eq!(loading.damages.len(), 2);
    let mut runtime = WorldRuntime::new(shared).unwrap();
    let initial = bot_combat_image(&mut runtime, &loading, &mut owner, &deadline);
    assert!(initial.ready_to_ack);
    let loaded = world::Request {
        manifest_digest: loading.world.manifest.digest,
        ..world_request(context, &loading.world, world::Action::Loaded)
    };
    let countdown_start = Instant::now();
    submit(
        &mut owner,
        &deadline,
        "human LOAD after real bot render",
        || LoginCommand::World(loaded),
    );
    let mut current = wait_owner(&mut owner, &deadline, "server bot-world countdown", |v| {
        v.combat
            .as_ref()
            .is_some_and(|f| f.world.phase == world::Phase::Active)
    })
    .combat
    .unwrap();
    assert!(countdown_start.elapsed() >= Duration::from_secs(3));
    let initial = bot_combat_image(&mut runtime, &current, &mut owner, &deadline);
    let before = current.clone();
    let from = actor(&before.world, SUN).position;
    submit(&mut owner, &deadline, "bot-world teleport", || {
        LoginCommand::CastTeleport
    });
    current = wait_owner(&mut owner, &deadline, "bot-world teleport exact ACK", |v| {
        !v.spell_pending
            && v.combat
                .as_ref()
                .is_some_and(|f| f.world.cast(human).is_some_and(|c| c.cast_count == 1))
    })
    .combat
    .unwrap();
    let teleport = *current.world.cast(human).unwrap();
    assert_eq!(teleport.from, from);
    assert_ne!(teleport.to, from);
    assert_eq!(actor(&current.world, SUN).position, teleport.to);
    assert_eq!(
        actor(&current.world, SUN).sequence,
        actor(&before.world, SUN).sequence
    );
    assert_eq!(current.vital(human).unwrap().mp, 90);
    assert_eq!(current.vital(bot_entity), before.vital(bot_entity));
    let image = bot_combat_image(&mut runtime, &current, &mut owner, &deadline);
    assert_eq!(image.projection.anchor_cell, teleport.to);
    assert_ne!(image.bgra, initial.bgra);
    let until =
        Instant::now() + Duration::from_millis(u64::from(teleport.cast_interval_millis).max(800));
    let mut sound119 = false;
    while Instant::now() < until {
        if let Some(image) = rendered(&mut runtime, &current.world, SUN) {
            sound119 |= image
                .audio
                .as_ref()
                .is_some_and(|a| a.events.iter().any(|e| e.group.candidates().contains(&119)));
        }
        owner.poll();
        deadline.pause("bot-world teleport visual/audio cue");
    }
    assert!(sound119);
    let heading = DIRECTIONS
        .iter()
        .position(|&(dx, dy)| teleport.to.x + dx == from.x && teleport.to.y + dy == from.y)
        .expect("explicit peer RNG gives adjacent legitimate destination") as u8;
    let movement = world::Request {
        heading,
        sequence: actor(&current.world, SUN).sequence + 1,
        from_x: teleport.to.x,
        from_y: teleport.to.y,
        ..world_request(context, &current.world, world::Action::Move)
    };
    submit(
        &mut owner,
        &deadline,
        "MOVE after GM-world teleport",
        || LoginCommand::World(movement),
    );
    current = wait_owner(&mut owner, &deadline, "MOVE ACK after teleport", |v| {
        !v.world_pending
            && v.combat
                .as_ref()
                .is_some_and(|f| actor(&f.world, SUN).sequence == movement.sequence)
    })
    .combat
    .unwrap();
    assert_eq!(actor(&current.world, SUN).position, from);
    assert_eq!(current.world.cast(human).unwrap(), &teleport);
    bot_combat_image(&mut runtime, &current, &mut owner, &deadline);
    let mut allowed =
        Instant::now() + Duration::from_nanos(actor(&current.world, SUN).step_interval_nanos);
    let mut attempts = 0;
    let mut sound97 = false;
    while current.vital(bot_entity).unwrap().hp > 0 {
        assert!(
            attempts < 24,
            "bounded balanced fixture must finish without fake damage"
        );
        while Instant::now() < allowed {
            owner.poll();
            deadline.pause("confirmed server action pace");
        }
        let before = current.clone();
        let sequence = before.world.cast(human).unwrap().request_sequence + 1;
        submit(
            &mut owner,
            &deadline,
            "targeted magic against tagged bot",
            || LoginCommand::CastLightArrow {
                target_entity_id: bot_entity,
            },
        );
        assert!(owner.snapshot().spell_pending);
        assert_eq!(owner.snapshot().combat.as_ref(), Some(&before));
        current = wait_owner(&mut owner, &deadline, "atomic bot damage ACK", |v| {
            !v.spell_pending
                && v.combat
                    .as_ref()
                    .is_some_and(|f| f.world.cast(human).unwrap().request_sequence == sequence)
        })
        .combat
        .unwrap();
        let damage = &current.damages[0];
        let prior = &before.damages[0];
        assert_eq!(damage.cast_count, prior.cast_count + 1);
        assert_eq!(damage.skill_id, 4);
        assert_eq!(damage.target_entity_id, bot_entity);
        assert_eq!(damage.mp_cost, 3);
        assert_eq!(damage.from, from);
        assert_eq!(damage.to, before.world.actors[1].position);
        assert_eq!(
            current.vital(human).unwrap().mp,
            before.vital(human).unwrap().mp - 3
        );
        assert!(
            current.vital(bot_entity).unwrap().hp < before.vital(bot_entity).unwrap().hp,
            "known level80/MR40 bot fixture must take server damage"
        );
        assert_eq!(current.vital(human).unwrap().attack_revision, 0);
        let tp = current.world.cast(human).unwrap();
        assert_eq!(
            (tp.cast_count, tp.cast_revision, tp.mp_spent, tp.from, tp.to),
            (1, teleport.cast_revision, 10, teleport.from, teleport.to)
        );
        allowed = Instant::now() + Duration::from_millis(u64::from(tp.cast_interval_millis));
        bot_combat_image(&mut runtime, &current, &mut owner, &deadline);
        let visual_until = Instant::now() + Duration::from_millis(1200);
        while Instant::now() < visual_until {
            if let Some(image) = rendered(&mut runtime, &current.world, SUN) {
                sound97 |= image
                    .audio
                    .as_ref()
                    .is_some_and(|a| a.events.iter().any(|e| e.group.candidates().contains(&97)));
            }
            owner.poll();
            deadline.pause("actual shared bot light-arrow projectile/sound cue");
        }
        attempts += 1;
    }
    assert!(attempts >= 2 && sound97);
    assert_eq!(current.world.phase, world::Phase::Finished);
    assert!(current.vital(bot_entity).unwrap().death_revision > 0);
    let terminal = current.clone();
    let saved = wait_owner(
        &mut owner,
        &deadline,
        "bot full-roster SQL settlement saved",
        |v| {
            v.world
                .frame()
                .and_then(|f| f.completion.as_ref())
                .is_some_and(|c| c.state == world::SettlementState::Saved)
        },
    );
    let result = saved.world.frame().unwrap().completion.as_ref().unwrap();
    assert_eq!(
        result.event,
        terminal.world.completion.as_ref().unwrap().event
    );
    assert_eq!(result.event.participants.len(), 2);
    assert_eq!(result.event.participants[0].kind, MemberKind::Human);
    assert!(result.event.participants[0].matches_character(SUN));
    assert_eq!(result.event.participants[0].verdict, world::Verdict::Win);
    assert_eq!(result.event.participants[1].kind, MemberKind::ServerBot);
    assert_eq!(result.event.participants[1].key.object_id, bot_id);
    assert_eq!(result.event.participants[1].verdict, world::Verdict::Loss);
    let leave = world_request(context, saved.world.frame().unwrap(), world::Action::Leave);
    submit(&mut owner, &deadline, "bot-world LEAVE", || {
        LoginCommand::World(leave)
    });
    let left = wait_owner(&mut owner, &deadline, "bot-world room return ACK", |v| {
        !v.world_pending && v.world.frame().is_none()
    });
    assert!(left.combat.is_none());
    assert_eq!(left.navigation.as_ref().unwrap().context, context);
    let expected = record::Totals {
        stored: true,
        wins: 1,
        losses: 0,
        revision: 1,
    };
    wait_owner(
        &mut owner,
        &deadline,
        "only human SQL totals after bot match",
        |v| {
            !v.record_pending
                && v.record.context() == Some(context)
                && v.record.totals() == Some(expected)
        },
    );
    submit(&mut owner, &deadline, "manual human record refresh", || {
        LoginCommand::ReadRecord
    });
    let checked = wait_owner(
        &mut owner,
        &deadline,
        "same committed UUID record refresh",
        |v| !v.record_pending,
    );
    assert_eq!(checked.record.totals(), Some(expected));
    // Never construct SelectionContext from the bot's numeric ID. The Java peer
    // independently checks one bot row and only the human total in isolated SQL.
    runtime.shutdown();
    while !runtime.is_quiescent() {
        runtime.poll();
        deadline.pause("bot-world resource owner retirement");
    }
    submit(
        &mut owner,
        &deadline,
        "single human normal TLS shutdown",
        || LoginCommand::Shutdown,
    );
    while !owner.is_quiescent() {
        owner.poll();
        deadline.pause("single human TLS cleanup");
    }
    assert_eq!(owner.snapshot().phase, LoginPhase::Disconnected);
    peer.expect_successful_exit(&deadline);
}

#[test]
#[ignore = "requires prepared Java world TLS peer plus explicit real shared Lineage resource root; no DB/GUI"]
fn java_world_controller_two_real_runtimes_auto_prepare_countdown_confirmed_move_and_retire() {
    let deadline = Deadline::new();
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let ca = private_root(&fixtures);
    let mut peer = JavaPeer::start(&fixtures, &deadline);
    let mut owner = SessionController::new(profile(peer.endpoint, &shared, &ca));
    submit(&mut owner, &deadline, "connect", || LoginCommand::Connect);
    wait_owner(&mut owner, &deadline, "verified TLS", |v| {
        v.phase == LoginPhase::LoginReady
    });
    submit(&mut owner, &deadline, "AUTH", || LoginCommand::Login {
        account: "fixture-user".into(),
        password: "fixture-password".into(),
    });
    wait_owner(&mut owner, &deadline, "source list", |v| {
        v.phase == LoginPhase::ChoosingSource
    });
    submit(&mut owner, &deadline, "source 7", || {
        LoginCommand::SelectSource(7)
    });
    let roster = wait_owner(&mut owner, &deadline, "source 7 roster", |v| {
        v.phase == LoginPhase::CharactersReady
    });
    assert_eq!(roster.characters.len(), 1);
    assert_eq!(
        (
            roster.characters[0].source_id,
            roster.characters[0].object_id
        ),
        (7, 3333)
    );
    let sun_context = SelectionContext {
        key: SUN,
        epoch: roster.roster_epoch,
    };
    submit(&mut owner, &deadline, "select source 7 character", || {
        LoginCommand::SelectCharacter(sun_context)
    });
    wait_owner(&mut owner, &deadline, "channels", |v| {
        v.phase == LoginPhase::ChoosingChannel
    });
    submit(&mut owner, &deadline, "channel 0", || {
        LoginCommand::EnterChannel {
            context: sun_context,
            channel_id: CHANNEL,
        }
    });
    let lobby = wait_owner(&mut owner, &deadline, "confirmed lobby", |v| {
        v.phase == LoginPhase::InLobby
    });
    assert!(lobby.world.frame().is_none());
    let rooms::State::Ready(directory) = &lobby.rooms else {
        panic!("confirmed room directory required")
    };
    assert!(directory
        .options
        .iter()
        .all(|option| option.world_available == (option.mode == 1 && option.map_id == 514)));
    assert_eq!(
        directory
            .options
            .iter()
            .filter(|option| option.world_available)
            .count(),
        1
    );

    let mut guest = connect_and_negotiate(
        peer.endpoint,
        "localhost",
        Some(&ca),
        Hello::current(100),
        TransportLimits {
            connect_timeout: Duration::from_millis(500),
            total_timeout: Duration::from_secs(2),
            io_poll_interval: Duration::from_millis(25),
        },
        Cancellation::new(),
    )
    .unwrap();
    guest
        .authenticate(Credentials::new("fixture-user".into(), "fixture-password".into()).unwrap())
        .unwrap();
    let guest_roster = guest.characters(42, 1).unwrap();
    assert_eq!(guest_roster.characters.len(), 1);
    assert_eq!(guest_roster.characters[0].key, MOON);
    let moon_context = SelectionContext {
        key: MOON,
        epoch: 1,
    };
    guest.select_character(moon_context).unwrap();
    guest.enter_channel(CHANNEL).unwrap();

    submit(&mut owner, &deadline, "CREATE + creator JOIN", || {
        LoginCommand::Rooms(
            rooms::Request::draft(
                rooms::Common {
                    action: rooms::Action::Create,
                    ..rooms::Common::read(sun_context, CHANNEL)
                },
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 4,
                        mode: 1,
                        attributes: 16,
                        map_id: 514,
                        name: "World interop".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    });
    let created = wait_owner(&mut owner, &deadline, "creator membership", |v| {
        !v.room_pending && v.rooms.joined().is_some()
    });
    let room = created.rooms.joined().unwrap();
    assert_eq!(room.owner, SUN);
    assert_eq!(room.members.len(), 1);
    let guest_directory = guest
        .rooms(rooms::Request::read(moon_context, CHANNEL).unwrap())
        .unwrap();
    let summary = guest_directory
        .rooms
        .iter()
        .find(|row| row.room_id == room.summary.room_id)
        .unwrap();
    let joined = guest
        .rooms(
            rooms::Request::join(
                rooms::Common {
                    context: moon_context,
                    channel_id: CHANNEL,
                    action: rooms::Action::Join,
                    value: 0,
                    room_id: summary.room_id,
                    expected_revision: summary.revision,
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(joined.status, rooms::Status::Ok);
    let joined = joined.joined.unwrap();
    assert_eq!(
        joined
            .members
            .iter()
            .map(|m| (m.key, m.team))
            .collect::<Vec<_>>(),
        vec![(SUN, 0), (MOON, 1)]
    );
    let seen = wait_owner(&mut owner, &deadline, "guest membership refresh", |v| {
        v.rooms.joined().is_some_and(|r| r.members.len() == 2)
    });
    let ready_room = seen.rooms.joined().unwrap().clone();
    submit(&mut owner, &deadline, "owner READY", || {
        LoginCommand::Rooms(room_command(
            sun_context,
            &ready_room,
            rooms::Action::Ready,
            1,
        ))
    });
    let pending = owner.snapshot();
    assert!(pending.room_pending);
    assert!(
        !pending
            .rooms
            .joined()
            .unwrap()
            .members
            .iter()
            .find(|m| m.matches_character(SUN))
            .unwrap()
            .ready,
        "accepted local command must not optimistically ready the owner"
    );
    wait_owner(&mut owner, &deadline, "owner READY ACK", |v| {
        !v.room_pending
            && v.rooms.joined().is_some_and(|r| {
                r.members
                    .iter()
                    .any(|m| m.matches_character(SUN) && m.ready)
            })
    });
    let latest = guest
        .rooms(rooms::Request::read(moon_context, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    let all_ready = guest
        .rooms(room_command(moon_context, &latest, rooms::Action::Ready, 1))
        .unwrap();
    assert!(all_ready.joined.as_ref().unwrap().summary.satisfied);

    // No manual owner PREPARE: the production session worker sees the actual
    // all-ready room revision and its explicit available mode/map option.
    let loading = wait_owner(&mut owner, &deadline, "automatic PREPARE -> LOADING", |v| {
        if let Some(frame) = v.world.frame() {
            assert_ne!(frame.phase, world::Phase::Aborted);
        }
        v.world
            .frame()
            .is_some_and(|f| f.phase == world::Phase::Loading)
    })
    .world
    .frame()
    .unwrap()
    .clone();
    assert_eq!(loading.room.summary.settings.mode, 1);
    assert_eq!(loading.room.summary.settings.map_id, 514);
    assert_eq!(loading.manifest.map_id, 514);
    assert_eq!(loading.actors.len(), 2);
    assert!(loading.actors.iter().all(|a| !a.loaded && a.sequence == 0));
    assert_eq!(
        actor(&loading, SUN).position,
        WorldCell { x: 32724, y: 32784 }
    );
    assert_ne!(
        actor(&loading, SUN).entity_id,
        actor(&loading, MOON).entity_id
    );
    let guest_loading = guest
        .world(world_request(moon_context, &loading, world::Action::Read))
        .unwrap()
        .frame
        .unwrap();
    assert!(
        guest_loading == loading,
        "both owners receive the same authoritative world snapshot"
    );

    let mut sun_runtime = WorldRuntime::new(shared.clone()).unwrap();
    let mut moon_runtime = WorldRuntime::new(shared.clone()).unwrap();
    sun_runtime
        .update(Arc::new(loading.clone()), actor(&loading, SUN).entity_id)
        .unwrap();
    moon_runtime
        .update(Arc::new(guest_loading), actor(&loading, MOON).entity_id)
        .unwrap();
    let mut keepalive = Instant::now();
    loop {
        deadline.check("two independent first renders");
        owner.poll();
        let observed = owner.snapshot();
        assert!(
            observed
                .world
                .frame()
                .is_some_and(|f| f.phase == world::Phase::Loading),
            "world first render lost Loading: {:?}; status {}",
            observed.world.frame().map(|f| (f.phase, f.abort_reason)),
            observed.status
        );
        let sun_image = rendered(&mut sun_runtime, &loading, SUN);
        let moon_image = rendered(&mut moon_runtime, &loading, MOON);
        if sun_image.is_some() && moon_image.is_some() {
            break;
        }
        if keepalive.elapsed() >= Duration::from_secs(1) {
            guest.ping().unwrap();
            keepalive = Instant::now();
        }
        deadline.pause("two independent first renders");
    }
    let sun_loaded = world::Request {
        manifest_digest: loading.manifest.digest,
        ..world_request(sun_context, &loading, world::Action::Loaded)
    };
    submit(&mut owner, &deadline, "owner rendered LOAD ACK", || {
        LoginCommand::World(sun_loaded)
    });
    assert!(
        !actor(owner.snapshot().world.frame().unwrap(), SUN).loaded,
        "queueing LOAD ACK does not locally cross the load barrier"
    );
    wait_owner(&mut owner, &deadline, "owner LOAD confirmation", |v| {
        !v.world_pending
            && v.world.frame().is_some_and(|f| {
                f.phase == world::Phase::Loading && actor(f, SUN).loaded && !actor(f, MOON).loaded
            })
    });
    let countdown_sent = Instant::now();
    let countdown = guest
        .world(world::Request {
            manifest_digest: loading.manifest.digest,
            ..world_request(moon_context, &loading, world::Action::Loaded)
        })
        .unwrap();
    assert_eq!(countdown.status, world::Status::Ok);
    let countdown = countdown.frame.unwrap();
    assert_eq!(countdown.phase, world::Phase::Countdown);
    assert!(countdown.actors.iter().all(|a| a.loaded));
    let active = wait_owner(&mut owner, &deadline, "real server countdown", |v| {
        if let Some(frame) = v.world.frame() {
            assert_ne!(frame.phase, world::Phase::Aborted);
        }
        v.world
            .frame()
            .is_some_and(|f| f.phase == world::Phase::Active)
    })
    .world
    .frame()
    .unwrap()
    .clone();
    assert!(
        countdown_sent.elapsed() >= Duration::from_secs(3),
        "server countdown must not be locally skipped"
    );
    sun_runtime
        .update(Arc::new(active.clone()), actor(&active, SUN).entity_id)
        .unwrap();
    moon_runtime
        .update(Arc::new(active.clone()), actor(&active, MOON).entity_id)
        .unwrap();
    let before_image = loop {
        deadline.check("active render");
        if let Some(image) = rendered(&mut sun_runtime, &active, SUN) {
            break image;
        }
        deadline.pause("active render");
    };

    // Use the exact authenticated manifest to choose a legal unoccupied edge,
    // never a guessed client/server map alias. The server still decides success.
    let bindings: Vec<_> = active
        .manifest
        .sectors
        .iter()
        .map(|s| SectorBinding {
            key: SharedMapSectorKey {
                map_id: active.manifest.map_id,
                sector_x: s.x,
                sector_y: s.y,
            },
            byte_length: s.byte_length,
            sha256: s.sha256,
        })
        .collect();
    let map = WorldMap::load_shared(&shared, &bindings, &AtomicBool::new(false)).unwrap();
    let before = actor(&active, SUN).clone();
    let (heading, destination) = DIRECTIONS
        .iter()
        .enumerate()
        .map(|(heading, &(dx, dy))| {
            (
                heading as u8,
                WorldCell {
                    x: before.position.x + dx,
                    y: before.position.y + dy,
                },
            )
        })
        .find(|(_, to)| {
            map.can_step(before.position, *to) && active.actors.iter().all(|a| a.position != *to)
        })
        .expect("fixed shared spawn has a legal unoccupied edge");
    let move_request = world::Request {
        heading,
        sequence: before.sequence + 1,
        from_x: before.position.x,
        from_y: before.position.y,
        ..world_request(sun_context, &active, world::Action::Move)
    };
    submit(&mut owner, &deadline, "MOVE", || {
        LoginCommand::World(move_request)
    });
    let queued = owner.snapshot();
    assert!(queued.world_pending);
    assert_eq!(
        actor(queued.world.frame().unwrap(), SUN).position,
        before.position
    );
    assert_eq!(
        actor(queued.world.frame().unwrap(), SUN).sequence,
        before.sequence
    );
    assert_eq!(
        before_image.revision, active.revision,
        "renderer still has the prior confirmed revision"
    );
    let moved = wait_owner(&mut owner, &deadline, "server MOVE ACK", |v| {
        !v.world_pending
            && v.world
                .frame()
                .is_some_and(|f| actor(f, SUN).sequence == move_request.sequence)
    });
    let manual_ack_received = Instant::now();
    let world::State::Ready(reply) = &moved.world else {
        unreachable!()
    };
    assert_eq!(reply.status, world::Status::Ok);
    let moved = reply.frame.as_ref().unwrap().clone();
    assert_eq!(actor(&moved, SUN).position, destination);
    assert_eq!(actor(&moved, SUN).heading, heading);
    assert_eq!(actor(&moved, MOON).position, actor(&active, MOON).position);
    let other_view = guest
        .world(world_request(moon_context, &moved, world::Action::Read))
        .unwrap()
        .frame
        .unwrap();
    assert!(
        other_view == moved,
        "other TLS source confirms the same committed movement"
    );
    sun_runtime
        .update(Arc::new(moved.clone()), before.entity_id)
        .unwrap();
    moon_runtime
        .update(Arc::new(other_view), actor(&moved, MOON).entity_id)
        .unwrap();
    let moved_image = loop {
        deadline.check("confirmed movement image");
        if let (Some(after), Some(_)) = (
            rendered(&mut sun_runtime, &moved, SUN),
            rendered(&mut moon_runtime, &moved, MOON),
        ) {
            assert!(
                after.bgra.as_ref() != before_image.bgra.as_ref(),
                "confirmed scene must actually be redrawn"
            );
            assert_eq!(after.projection.anchor_cell, actor(&moved, SUN).position);
            break after;
        }
        deadline.pause("confirmed movement image");
    };

    // The UI route starts from the actual displayed camera/geometry and an
    // unchanged server-confirmed frame, not a speculative local actor position.
    let goal = five_step_goal(&moved_image, &moved);
    let route_map = moved_image.navigation_map().unwrap();
    let goal_pixel = moved_image.projection.cell_to_pixel(goal).unwrap();
    assert_eq!(moved_image.projection.pixel_to_cell(goal_pixel), Some(goal));
    let mut navigation = Navigation::default();
    // The prior step was deliberately sent outside Navigation. Respect its
    // confirmed server interval before this independent route's first command;
    // this does not claim Navigation retains cooldown across separate routes.
    let first_route_allowed = manual_ack_received
        .checked_add(Duration::from_nanos(actor(&moved, SUN).step_interval_nanos))
        .unwrap();
    while Instant::now() < first_route_allowed {
        deadline.pause("manual MOVE to route cooldown");
    }
    navigation
        .request(route_map, Arc::new(moved.clone()), before.entity_id, goal)
        .unwrap();
    let mut routed = moved.clone();
    let mut completed_steps = 0usize;
    let mut last_image = moved_image;
    loop {
        deadline.check("five-step confirmed route");
        navigation.poll();
        let intent = navigation.next_step(&routed, before.entity_id, Instant::now());
        match navigation.status() {
            NavigationStatus::Arrived => break,
            NavigationStatus::Stopped(reason) => panic!("real TLS route stopped: {reason}"),
            _ => {}
        }
        let Some(intent) = intent else {
            deadline.pause("route worker or server-paced next step");
            continue;
        };
        assert!(
            completed_steps < 5,
            "a shortest five-step route cannot add steps"
        );
        let prior = actor(&routed, SUN).clone();
        assert_eq!(intent.from, prior.position);
        assert_eq!(intent.sequence, prior.sequence + 1);
        assert_eq!(
            (intent.match_id, intent.generation),
            (routed.match_id, routed.generation)
        );
        assert!(map.can_step(intent.from, intent.to));
        assert_ne!(intent.to, actor(&routed, MOON).position);
        assert_eq!(last_image.projection.anchor_cell, prior.position);
        assert_eq!(last_image.revision, routed.revision);
        let request = world::Request {
            heading: intent.heading,
            sequence: intent.sequence,
            from_x: intent.from.x,
            from_y: intent.from.y,
            ..world_request(sun_context, &routed, world::Action::Move)
        };
        submit(&mut owner, &deadline, "route MOVE admission", || {
            LoginCommand::World(request)
        });
        navigation.submitted(intent).unwrap();
        assert_eq!(navigation.status(), NavigationStatus::WaitingForAck);
        assert!(navigation
            .next_step(&routed, before.entity_id, Instant::now())
            .is_none());
        let queued = owner.snapshot();
        assert!(queued.world_pending);
        assert_eq!(
            actor(queued.world.frame().unwrap(), SUN).position,
            prior.position
        );
        assert_eq!(
            actor(queued.world.frame().unwrap(), SUN).sequence,
            prior.sequence
        );
        assert_eq!(last_image.projection.anchor_cell, prior.position);
        let confirmed = wait_owner(&mut owner, &deadline, "route MOVE server ACK", |v| {
            !v.world_pending
                && v.world
                    .frame()
                    .is_some_and(|f| actor(f, SUN).sequence == intent.sequence)
        });
        let world::State::Ready(reply) = &confirmed.world else {
            unreachable!()
        };
        assert_eq!(reply.status, world::Status::Ok);
        let next = reply.frame.as_ref().unwrap().clone();
        assert_eq!(actor(&next, SUN).position, intent.to);
        assert_eq!(actor(&next, SUN).heading, intent.heading);
        assert_eq!(actor(&next, SUN).sequence, intent.sequence);
        assert!(next.revision > routed.revision);
        assert_eq!(actor(&next, MOON), actor(&routed, MOON));
        // Only a genuine TLS ACK can advance Navigation. The next step remains
        // delayed by the server interval even if rendering happens instantly.
        assert!(navigation
            .next_step(&next, before.entity_id, Instant::now())
            .is_none());
        assert!(matches!(
            navigation.status(),
            NavigationStatus::Walking | NavigationStatus::Arrived
        ));
        let other = guest
            .world(world_request(moon_context, &next, world::Action::Read))
            .unwrap()
            .frame
            .unwrap();
        assert_eq!(other, next, "both TLS clients see every route commit");
        sun_runtime
            .update(Arc::new(next.clone()), before.entity_id)
            .unwrap();
        moon_runtime
            .update(Arc::new(other), actor(&next, MOON).entity_id)
            .unwrap();
        loop {
            deadline.check("route confirmed image");
            if let (Some(sun_image), Some(moon_image)) = (
                rendered(&mut sun_runtime, &next, SUN),
                rendered(&mut moon_runtime, &next, MOON),
            ) {
                assert_eq!(sun_image.projection.anchor_cell, intent.to);
                assert_eq!(
                    moon_image.projection.anchor_cell,
                    actor(&next, MOON).position
                );
                assert_eq!(
                    sun_image
                        .projection
                        .pixel_to_cell(sun_image.projection.anchor_pixel),
                    Some(intent.to)
                );
                assert!(
                    sun_image.bgra.as_ref() != last_image.bgra.as_ref(),
                    "each confirmed step renders new pixels"
                );
                last_image = sun_image;
                break;
            }
            deadline.pause("route confirmed image");
        }
        routed = next;
        completed_steps += 1;
    }
    assert_eq!(completed_steps, 5);
    assert_eq!(actor(&routed, SUN).position, goal);
    assert_eq!(
        actor(&routed, SUN).sequence,
        actor(&moved, SUN).sequence + 5
    );
    assert_eq!(last_image.projection.anchor_cell, goal);
    assert_eq!(navigation.status(), NavigationStatus::Arrived);
    assert!(navigation.is_quiescent(), "actual route worker has retired");
    navigation.cancel();
    navigation.poll();
    assert_eq!(navigation.status(), NavigationStatus::Idle);
    assert!(navigation.is_quiescent());

    let leave = world_request(sun_context, &routed, world::Action::Leave);
    submit(&mut owner, &deadline, "world LEAVE", || {
        LoginCommand::World(leave)
    });
    assert!(
        owner.snapshot().world.frame().is_some(),
        "pending LEAVE retains the confirmed world"
    );
    let returned = wait_owner(&mut owner, &deadline, "world LEAVE + room refresh", |v| {
        !v.world_pending
            && v.world.frame().is_none()
            && v.rooms.joined().is_some_and(|r| r.summary.ready == 0)
    });
    assert!(returned
        .rooms
        .joined()
        .unwrap()
        .members
        .iter()
        .all(|m| !m.ready));
    assert_eq!(returned.phase, LoginPhase::InLobby);
    let terminal = guest
        .world(world_request(moon_context, &routed, world::Action::Read))
        .unwrap()
        .frame
        .unwrap();
    assert_eq!(terminal.phase, world::Phase::Aborted);
    assert!(guest
        .world(world_request(moon_context, &terminal, world::Action::Leave))
        .unwrap()
        .frame
        .is_none());
    let guest_room = guest
        .rooms(rooms::Request::read(moon_context, CHANNEL).unwrap())
        .unwrap()
        .joined
        .unwrap();
    assert_eq!(guest_room.summary.ready, 0);
    assert!(guest
        .rooms(room_command(
            moon_context,
            &guest_room,
            rooms::Action::Leave,
            0
        ))
        .unwrap()
        .joined
        .is_none());
    let sole = wait_owner(&mut owner, &deadline, "guest room departure", |v| {
        v.rooms.joined().is_some_and(|r| r.members.len() == 1)
    });
    let sole_room = sole.rooms.joined().unwrap().clone();
    submit(&mut owner, &deadline, "owner room LEAVE", || {
        LoginCommand::Rooms(room_command(
            sun_context,
            &sole_room,
            rooms::Action::Leave,
            0,
        ))
    });
    wait_owner(&mut owner, &deadline, "empty room deletion", |v| {
        !v.room_pending && v.rooms.joined().is_none()
    });

    sun_runtime.shutdown();
    moon_runtime.shutdown();
    loop {
        sun_runtime.poll();
        moon_runtime.poll();
        if sun_runtime.is_quiescent() && moon_runtime.is_quiescent() {
            break;
        }
        deadline.pause("actual resource/render worker retirement");
    }
    guest.close().unwrap();
    submit(&mut owner, &deadline, "session shutdown", || {
        LoginCommand::Shutdown
    });
    loop {
        owner.poll();
        if owner.is_quiescent() {
            break;
        }
        deadline.pause("actual TLS session worker retirement");
    }
    assert_eq!(owner.snapshot().phase, LoginPhase::Disconnected);
    peer.expect_successful_exit(&deadline);
    deadline.check("complete program-level flow");
}
