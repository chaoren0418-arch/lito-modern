//! E149 loopback fixture. All player state arrives through LoginController/TCP.
//! Reuses NativePeerHarness's memory repositories and real ClientSession workers.
use anyhow::{ensure, Context, Result};
use lito_modern::modern_lito::{
    native_connection::Limits,
    native_login::{LoginController, LoginPhase, Snapshot},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

pub const REFERENCE_SHA: &str = "e1498871c7be6b287821c57ef5d95933db1740d9b4d4a80f6c3465f21d99d498";
pub const ACCOUNTS: [&str; 2] = ["PeerFixture", "PeerFixture2"];
pub const ACTOR_IDS: [u32; 2] = [1, 3];
static NEXT_RUN: AtomicU64 = AtomicU64::new(0);

pub fn java_path(path: &Path) -> Result<PathBuf> {
    let text = path.to_str().context("JDK requires a Unicode path")?;
    // std::fs::canonicalize uses Windows verbatim paths, which JDK classpaths reject.
    if let Some(drive_path) = text.strip_prefix(r"\\?\") {
        ensure!(
            drive_path.as_bytes().get(1) == Some(&b':'),
            "fixture requires a local drive"
        );
        return Ok(PathBuf::from(drive_path));
    }
    Ok(path.to_path_buf())
}

pub fn verify_reference(jar: &Path) -> Result<()> {
    let actual = format!("{:x}", Sha256::digest(fs::read(jar)?));
    ensure!(
        actual == REFERENCE_SHA,
        "T043 requires pinned E149, got {actual}"
    );
    Ok(())
}

pub fn ready_endpoints(ready: &Value) -> Result<[SocketAddr; 2]> {
    ensure!(ready["jar_sha256"] == REFERENCE_SHA, "ready JAR mismatch");
    let endpoint = |name: &str, host: Ipv4Addr| -> Result<SocketAddr> {
        let port = u16::try_from(ready[name].as_u64().context("missing port")?)?;
        ensure!(!matches!(port, 0 | 2000 | 3306), "forbidden {name}");
        Ok(SocketAddr::from((host, port)))
    };
    Ok([
        endpoint("port", Ipv4Addr::LOCALHOST)?,
        endpoint("game_port", Ipv4Addr::new(127, 0, 0, 2))?,
    ])
}

fn java_command(program: &Path, root: &Path) -> Command {
    let mut command = Command::new(program);
    command.current_dir(root).stdin(Stdio::null());
    command.env("TEMP", root).env("TMP", root);
    // The fixture owns the complete JVM command and classpath.
    for name in [
        "JAVA_TOOL_OPTIONS",
        "JDK_JAVA_OPTIONS",
        "_JAVA_OPTIONS",
        "CLASSPATH",
    ] {
        command.env_remove(name);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    command
}

/// Owns only the child it spawned. Drop also handles assertions/unwinding.
struct OwnedJava(Child);

impl OwnedJava {
    fn wait(&mut self, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self.0.try_wait()? {
                ensure!(status.success(), "Java fixture exited {status}");
                return Ok(());
            }
            ensure!(Instant::now() < deadline, "Java fixture cleanup timeout");
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for OwnedJava {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

/// Kept live after bootstrap so T043-2 can operate both production controllers.
pub struct TwoPlayerBattle {
    pub players: [LoginController; 2],
    pub root: PathBuf,
    server: OwnedJava,
    game_endpoint: SocketAddr,
}

impl TwoPlayerBattle {
    pub fn start() -> Result<Self> {
        let jar = PathBuf::from(
            std::env::var_os("LITO_T043_REFERENCE_JAR")
                .context("set LITO_T043_REFERENCE_JAR to the read-only E149 JAR")?,
        )
        .canonicalize()?;
        let jar = java_path(&jar)?;
        verify_reference(&jar)?;
        let jdk = PathBuf::from(std::env::var_os("JAVA_HOME").context("set JAVA_HOME to JDK 17")?)
            .join("bin");
        let javac = jdk.join(format!("javac{}", std::env::consts::EXE_SUFFIX));
        let java = jdk.join(format!("java{}", std::env::consts::EXE_SUFFIX));
        ensure!(
            javac.is_file() && java.is_file(),
            "JAVA_HOME must contain both java and javac"
        );
        let project = Path::new(env!("CARGO_MANIFEST_DIR"));
        let target = project.join("target/current/t043-two-player");
        fs::create_dir_all(&target)?;
        let root = target.join(format!(
            "run-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
            NEXT_RUN.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?; // Fresh directory, no stale ready/result reuse.
        let root = root.canonicalize()?;
        ensure!(
            root.starts_with(project.canonicalize()?),
            "fixture outside worktree"
        );
        let root = java_path(&root)?;
        eprintln!("T043 evidence {}", root.display());
        let classes = root.join("classes");
        fs::create_dir(&classes)?;
        let mut compiler = OwnedJava(
            java_command(&javac, &root)
                .arg(format!("-J-Djava.io.tmpdir={}", root.display()))
                .arg("-J-Dfile.encoding=UTF-8")
                .args(["-encoding", "UTF-8", "-cp"])
                .arg(&jar)
                .arg("-d")
                .arg(&classes)
                .arg(project.join("tests/oracle/NativePeerHarness.java"))
                .stdout(File::create(root.join("javac.stdout.log"))?)
                .stderr(File::create(root.join("javac.stderr.log"))?)
                .spawn()?,
        );
        compiler.wait(Duration::from_secs(60))?;
        let classpath = std::env::join_paths([classes.as_path(), jar.as_path()])?;
        let server = OwnedJava(
            java_command(&java, &root)
                .arg(format!("-Djava.io.tmpdir={}", root.display()))
                .args(["-Dfile.encoding=UTF-8", "-Xverify:all"])
                // Current native_status_receipt accepts schema0's 37-byte S_STATUS.
                .arg("-Dlito.numeric.breakthrough=0")
                .arg("-cp")
                .arg(classpath)
                .arg("local.lito381.server.NativePeerHarness")
                .arg(&root)
                .arg("melee")
                .stdout(File::create(root.join("server.stdout.log"))?)
                .stderr(File::create(root.join("server.stderr.log"))?)
                .spawn()?,
        );
        let mut fixture = Self {
            players: [LoginController::default(), LoginController::default()],
            root,
            server,
            game_endpoint: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        };
        let deadline = Instant::now() + Duration::from_secs(30);
        let endpoints = loop {
            fixture.ensure_server_running()?;
            // The existing harness writes ready.json directly. Accept only a full document.
            if let Ok(bytes) = fs::read(fixture.root.join("ready.json")) {
                if let Ok(ready) = serde_json::from_slice(&bytes) {
                    break ready_endpoints(&ready)?;
                }
            }
            ensure!(Instant::now() < deadline, "Java ready timeout");
            thread::sleep(Duration::from_millis(10));
        };
        verify_reference(&jar)?;
        fixture.game_endpoint = endpoints[1];
        for player in &mut fixture.players {
            player.start(
                endpoints[0],
                Limits {
                    read_chunk: 2,
                    write_chunk: 3,
                    ..Limits::default()
                },
            )?;
        }
        Ok(fixture)
    }

    fn ensure_server_running(&mut self) -> Result<()> {
        ensure!(
            self.server.0.try_wait()?.is_none(),
            "Java fixture stopped, see {}",
            self.root.display()
        );
        Ok(())
    }

    pub fn poll(&mut self) -> Result<()> {
        self.ensure_server_running()?;
        for (index, player) in self.players.iter_mut().enumerate() {
            player.poll();
            ensure!(
                !matches!(
                    player.snapshot().phase,
                    LoginPhase::Failed | LoginPhase::Closed
                ),
                "player {index} failed at {:?}: {:?}",
                player.snapshot().phase,
                player.snapshot().error
            );
        }
        Ok(())
    }

    /// Both peers must join and acknowledge opposing forces before either readies.
    pub fn enter_battle(&mut self) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(45);
        let mut submitted = [false; 2];
        let mut last_refresh = [Instant::now() - Duration::from_secs(1); 2];
        loop {
            self.poll()?;
            let both_in_room = self.players.iter().all(|p| {
                let s = p.snapshot();
                s.phase == LoginPhase::RoomReady
                    && s.joined_room.as_ref().is_some_and(|r| {
                        r.members.len() == 2
                            && r.members.iter().any(|m| m.force == 0)
                            && r.members.iter().any(|m| m.force == 1)
                    })
            });
            for index in 0..2 {
                let player = &mut self.players[index];
                if player
                    .snapshot()
                    .notice
                    .as_ref()
                    .is_some_and(|n| n.acknowledgement().is_some())
                {
                    player.acknowledge_notice()?;
                }
                match player.snapshot().phase {
                    LoginPhase::ReadyForLogin => {
                        ensure!(!submitted[index], "LOGIN rejected for peer {index}");
                        player.login(ACCOUNTS[index], "TestOnlyPassword")?;
                        submitted[index] = true;
                    }
                    LoginPhase::SourcesReady => player.select_source(1)?,
                    LoginPhase::RedirectReady => {
                        ensure!(
                            player.snapshot().redirect == Some(self.game_endpoint),
                            "unowned GAME redirect"
                        );
                        player.begin_game()?;
                    }
                    LoginPhase::CharactersReady => player.select_character(0)?,
                    LoginPhase::CharacterSelected => player.enter_channel(0)?,
                    LoginPhase::LobbyReady => {
                        if !player.snapshot().rooms.is_empty() {
                            player.join_room(0, "")?;
                        } else if last_refresh[index].elapsed() > Duration::from_millis(250) {
                            player.refresh_rooms()?;
                            last_refresh[index] = Instant::now();
                        }
                    }
                    LoginPhase::RoomReady => {
                        let s = player.snapshot();
                        let room = s.joined_room.as_ref().context("missing room")?;
                        let own = room
                            .members
                            .iter()
                            .find(|m| Some(m.name.as_str()) == s.selected_character.as_deref())
                            .context("missing own membership")?;
                        if own.force != index as u8 {
                            player.set_force(index as u8)?;
                        } else if both_in_room && own.state == 0 {
                            player.set_ready(true)?;
                        }
                    }
                    _ => {}
                }
            }
            if self.players.iter().all(|p| {
                let s = p.snapshot();
                s.phase == LoginPhase::BattlePending
                    && s.world.ready()
                    && s.world.integrity_challenge.is_some()
                    && ACTOR_IDS.iter().all(|id| s.world.actors.contains_key(id))
            }) {
                return self.verify_battle();
            }
            ensure!(
                Instant::now() < deadline,
                "two-player bootstrap timeout: {:?} / {:?}",
                self.players[0].snapshot().phase,
                self.players[1].snapshot().phase
            );
            thread::sleep(Duration::from_millis(5));
        }
    }

    pub fn verify_battle(&self) -> Result<()> {
        let room_id = self.players[0]
            .snapshot()
            .joined_room
            .as_ref()
            .context("peer A room")?
            .id;
        for (index, player) in self.players.iter().enumerate() {
            let s = player.snapshot();
            ensure!(
                s.phase == LoginPhase::BattlePending && s.world.ready(),
                "battle not ready"
            );
            ensure!(
                s.joined_room.as_ref().context("peer room")?.id == room_id,
                "different rooms"
            );
            ensure!(!s.observer, "observer is not a combatant");
            ensure!(
                s.world.own_actor().context("own actor")?.id == ACTOR_IDS[index],
                "wrong own identity"
            );
            ensure!(
                ACTOR_IDS.iter().all(|id| s.world.actors.contains_key(id)),
                "missing real peer"
            );
            ensure!(!s.world.actors.contains_key(&2), "training bot present");
        }
        ensure!(
            self.players[0].snapshot().selected_character
                != self.players[1].snapshot().selected_character,
            "duplicate character identity"
        );
        Ok(())
    }

    pub fn save_snapshots(&self, name: &str) -> Result<()> {
        // Callers supply a file stem, never a relative/absolute output path.
        ensure!(
            !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'),
            "invalid evidence name"
        );
        let snapshots: Vec<&Snapshot> = self.players.iter().map(|p| p.snapshot()).collect();
        fs::write(
            self.root.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&json!({
                "jar_sha256": REFERENCE_SHA, "players": snapshots
            }))?,
        )?;
        Ok(())
    }

    /// Cancel both peers before waiting for server cleanup. A disconnect is not a battle result test.
    pub fn close(mut self) -> Result<Value> {
        self.cancel_players()?;
        self.server.wait(Duration::from_secs(15))?;
        let result: Value = serde_json::from_slice(&fs::read(self.root.join("result.json"))?)?;
        ensure!(
            result["status"] == "sessions_closed"
                && result["actual_client_sessions"] == 4
                && result["game_sessions"] == 2
                && result["memberships_empty"] == true
                && result["authority_mutations"] == 0
                && result["sql_access"] == false
                && result["state_persistent"] == false
                && result["jar_sha256"] == REFERENCE_SHA,
            "invalid server cleanup receipt: {result}"
        );
        Ok(result)
    }

    fn cancel_players(&mut self) -> Result<()> {
        for player in &mut self.players {
            player.cancel();
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            for player in &mut self.players {
                player.poll();
            }
            if self.players.iter().all(|p| p.is_quiescent()) {
                return Ok(());
            }
            ensure!(Instant::now() < deadline, "client workers not reclaimed");
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for TwoPlayerBattle {
    fn drop(&mut self) {
        // Also runs on a failed assertion. OwnedJava then reaps only this fixture's child.
        let _ = self.cancel_players();
    }
}
