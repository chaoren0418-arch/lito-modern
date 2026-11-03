//! Actual Rust requests through a real room-authorized Java engine, back into
//! Rust decoding. These are payload tests, not TLS/SQL/UI acceptance.
use super::*;
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
struct Peer {
    child: Child,
    input: Option<ChildStdin>,
    lines: mpsc::Receiver<String>,
    reader: Option<std::thread::JoinHandle<()>>,
}
impl Peer {
    fn start(scenario: &str) -> Self {
        let mut c =
            Command::new(std::env::var_os("LITO_MODERN_TEST_JAVA").expect("explicit JDK 17"));
        c.arg("-cp")
            .arg(
                std::env::var_os("LITO_MODERN_TEST_JAVA_CLASSPATH")
                    .expect("compiled test classpath"),
            )
            .arg("local.lito381.server.ModernNpcFireStormWirePeer")
            .arg(scenario)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(0x08000000);
        }
        let mut child = c.spawn().expect("Java skill70 payload peer");
        let input = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for l in BufReader::new(stdout.take(131072)).lines() {
                let Ok(l) = l else { break };
                if tx.send(l).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            input,
            lines,
            reader: Some(reader),
        }
    }
    fn line(&self) -> String {
        self.lines
            .recv_timeout(Duration::from_secs(15))
            .expect("bounded peer response")
    }
    fn send(&mut self, r: &Request) -> String {
        let b = encode_request(r, VERSION).unwrap();
        let h = b.iter().map(|n| format!("{n:02x}")).collect::<String>();
        let i = self.input.as_mut().unwrap();
        writeln!(i, "{h}").unwrap();
        i.flush().unwrap();
        self.line()
    }
    fn finish(&mut self) {
        assert_eq!(self.line(), "CLOSED");
        self.input.take();
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(10));
        }
        self.reader.take().unwrap().join().unwrap();
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.input.take();
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        if let Some(r) = self.reader.take() {
            let _ = r.join();
        }
    }
}
fn decoded(hex: &str, scope: Scope) -> Vec<State> {
    let b = super::tests::bytes(hex);
    let (s, n) = decode_states(&b, scope, 2, VERSION).unwrap();
    assert_eq!(n, b.len());
    assert_eq!(b, encode_states(&s, scope, 2, VERSION).unwrap());
    s
}
fn scenario(name: &str, casts: usize) {
    let mut p = Peer::start(name);
    let hello = p.line();
    let fields = hello.split(':').collect::<Vec<_>>();
    assert_eq!(fields.len(), 4);
    assert_eq!(fields[0], "READY");
    let mut request = decode_request(&super::tests::bytes(fields[1]), VERSION).unwrap();
    let scope = request.scope;
    let mut revision = fields[2].parse::<i64>().unwrap();
    let mut old = decoded(fields[3], scope);
    assert_eq!(request.context.key.source_id, 7);
    assert_eq!(
        request.context.key.object_id,
        if name == "time" { 3333 } else { 1 }
    );
    assert_eq!(old[0].count, 0);
    assert_eq!(old[1].count, 0);
    let mut actual = 0;
    let mut historical = 0;
    for i in 0..casts + 2 {
        request.sequence = if i == 1 {
            1
        } else if i == 0 {
            1
        } else {
            i as u64
        };
        let line = p.send(&request);
        let v = line.split(':').collect::<Vec<_>>();
        assert_eq!(v.len(), 6, "{line}");
        let next_revision = v[1].parse::<i64>().unwrap();
        let states = decoded(v[5], scope);
        assert!(
            follows(scope, &old, revision, scope, &states, next_revision),
            "{name}/{i}: non-monotonic"
        );
        if i == 1 || i == 2 {
            assert_eq!(v[0], if i == 1 { "SECURITY" } else { "TOO_FAST" });
            assert_eq!(old, states);
        } else {
            assert_eq!(v[0], "NONE");
            actual += 1;
        }
        assert_eq!(states[0].count, actual);
        assert_eq!(states[0].mp_spent, 48 * actual);
        assert_eq!(v[2].parse::<i64>().unwrap(), 1000 - 48 * actual);
        assert_eq!(v[3], "1000");
        assert_eq!(states[1].count, 0);
        if let Some(s) = &states[0].structure {
            if s.count < states[0].count {
                historical += 1;
                assert!(states[0].current_structure().is_none());
            } else {
                assert!(states[0].current_structure().is_some());
            }
        }
        assert_eq!(v[4], if i == casts + 1 { "FINISHED" } else { "ACTIVE" });
        old = states;
        revision = next_revision;
    }
    let proof = old[0].structure.as_ref().unwrap();
    assert!(proof.outcome.death);
    assert_eq!(actual, casts as i64);
    match name {
        "time" => {
            assert!(matches!(proof.id, StructureId::Tower(_)));
            assert_eq!(proof.count, 1);
            assert_eq!(historical, 4);
            assert_eq!(old[0].targets.len(), 2);
        }
        "quest-symbols" => {
            assert!(matches!(
                proof.id,
                StructureId::Quest {
                    kind: QuestKind::FinalSymbol,
                    ..
                }
            ));
            assert_eq!(proof.count, 8);
            assert_eq!(historical, 3);
        }
        _ => {
            assert!(matches!(
                proof.id,
                StructureId::Quest {
                    kind: QuestKind::FinalGuardian,
                    ..
                }
            ));
            assert_eq!(proof.count, 5);
        }
    }
    p.finish();
}
#[test]
#[ignore = "requires compiled Java test peer and explicit JDK17 classpath"]
fn rust_java_fire_storm_time_tower_four_waves_and_retained_proof() {
    scenario("time", 5);
}
#[test]
#[ignore = "requires compiled Java test peer and explicit JDK17 classpath"]
fn rust_java_fire_storm_quest_symbols_stages_and_terminal_proof() {
    scenario("quest-symbols", 8);
}
#[test]
#[ignore = "requires compiled Java test peer and explicit JDK17 classpath"]
fn rust_java_fire_storm_quest_guardian_final_kill_and_terminal_proof() {
    scenario("quest-guardian", 5);
}
