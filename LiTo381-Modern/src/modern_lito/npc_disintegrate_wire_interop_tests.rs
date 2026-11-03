//! Real Rust -> Java engine -> Rust payload interop. Deliberately not Main/TLS,
//! an application codec, shared-resource rendering or player UI acceptance.
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
    fn start() -> Self {
        let cp = std::env::var_os("LITO_MODERN_TEST_JAVA_CLASSPATH")
            .expect("compiled Java test classpath");
        let executable =
            std::env::var_os("LITO_MODERN_TEST_JAVA").expect("explicit JDK 17 test executable");
        let mut command = Command::new(executable);
        command
            .arg("-cp")
            .arg(cp)
            .arg("local.lito381.server.ModernNpcDisintegrateWirePeer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().expect("Java NPC77 test peer");
        let input = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            // The complete fixture transcript is <16 KiB, even for errors.
            for line in BufReader::new(stdout.take(65536)).lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
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
            .expect("bounded Java peer response")
    }
    fn send(&mut self, request: &Request) -> (String, i64, i64, i64, Vec<State>) {
        let b = encode_request(request, VERSION).unwrap();
        let hex = b.iter().map(|n| format!("{n:02x}")).collect::<String>();
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{hex}").unwrap();
        input.flush().unwrap();
        let line = self.line();
        let fields = line.split(':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 5, "{line}");
        let data = super::tests::bytes(fields[4]);
        let (states, used) = decode_states(&data, request.target.scope, 2, VERSION).unwrap();
        assert_eq!(used, data.len());
        (
            fields[0].into(),
            fields[1].parse().unwrap(),
            fields[2].parse().unwrap(),
            fields[3].parse().unwrap(),
            states,
        )
    }
    fn finish(&mut self) {
        assert_eq!(self.line(), "CLOSED");
        self.input.take();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(Instant::now() < deadline, "Java peer failed to retire");
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
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[test]
#[ignore = "requires compiled Java test peer and LITO_MODERN_TEST_JAVA_CLASSPATH"]
fn rust_requests_execute_real_java_npc_skill77_and_replay_does_not_double_spend() {
    let mut peer = Peer::start();
    let hello = peer.line();
    let data = super::tests::bytes(hello.strip_prefix("READY:").expect("peer greeting"));
    let mut r = decode_request(&data, VERSION).unwrap();
    let first = peer.send(&r);
    assert_eq!(first.0, "NONE");
    assert_eq!((first.2, first.3), (930, 9568));
    assert_eq!((first.4[0].count, first.4[0].lawful), (1, 501));
    assert_eq!(first.4[0].hit.as_ref().unwrap().target, r.target);
    assert_eq!(first.4[0].revision, first.1);
    r.sequence = 2;
    let denied = peer.send(&r);
    assert_eq!(denied.0, "TOO_FAST");
    assert_eq!((denied.2, denied.3), (930, 9568));
    assert_eq!(first.4, denied.4);
    assert!(follows(
        r.target.scope,
        &first.4,
        first.1,
        r.target.scope,
        &denied.4,
        denied.1
    ));
    r.sequence = 3;
    let second = peer.send(&r);
    assert_eq!(second.0, "NONE");
    assert_eq!((second.2, second.3), (860, 9136));
    assert_eq!((second.4[0].count, second.4[0].lawful), (2, 502));
    assert!(follows(
        r.target.scope,
        &first.4,
        first.1,
        r.target.scope,
        &second.4,
        second.1
    ));
    let replay = peer.send(&r);
    assert_eq!(replay.0, "SECURITY");
    assert_eq!((replay.2, replay.3), (second.2, second.3));
    assert_eq!(replay.4, second.4);
    peer.finish();
}
