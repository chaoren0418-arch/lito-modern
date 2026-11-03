//! Real Java engine/history payload interop. No TLS, GUI, or advanced SET claim.
use super::*;
use crate::modern_lito::equipment_history;
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
        Self::start_class("local.lito381.server.ModernEquipmentHistoryWirePeer")
    }
    fn start_class(class: &str) -> Self {
        let mut c =
            Command::new(std::env::var_os("LITO_MODERN_TEST_JAVA").expect("explicit JDK17"));
        c.arg("-cp")
            .arg(
                std::env::var_os("LITO_MODERN_TEST_JAVA_CLASSPATH")
                    .expect("compiled Java test classpath"),
            )
            .arg(class)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(0x08000000);
        }
        let mut child = c.spawn().expect("Java history peer");
        let input = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout.take(2 * 1024 * 1024)).lines() {
                let Ok(line) = line else {
                    break;
                };
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
            .recv_timeout(Duration::from_secs(20))
            .expect("bounded Java history response")
    }
    fn send(&mut self, bytes: &[u8]) {
        assert!(bytes.len() <= MAX_BYTES);
        let hex = bytes.iter().map(|n| format!("{n:02x}")).collect::<String>();
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{hex}").unwrap();
        input.flush().unwrap();
    }
    fn finish(&mut self) {
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
fn bytes(hex: &str) -> Vec<u8> {
    assert!(hex.len() <= MAX_BYTES * 2 && hex.len() % 2 == 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
#[test]
#[ignore = "requires compiled Java player-healing owner-section peer; not TLS/GUI acceptance"]
fn java_player_healing_owner_schema5_real_engine_cross_source_gear_life_and_effects_round_trip() {
    let mut peer = Peer::start_class("local.lito381.server.ModernPlayerHealingHistoryWirePeer");
    let mut count = 0;
    let mut engine_states = 0;
    let mut engine_before: BTreeMap<u32, State> = BTreeMap::new();
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:55");
            break;
        }
        let fields: Vec<_> = line.splitn(3, ':').collect();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], "STATE");
        let label = fields[1];
        let raw = bytes(fields[2]);
        let state = decode_with_schema(&raw, PLAYER_HEALING_SCHEMA).unwrap();
        assert!(state.valid());
        assert!(state.healing.is_some());
        assert_eq!(
            raw,
            encode_with_schema(&state, PLAYER_HEALING_SCHEMA).unwrap()
        );
        assert_eq!(
            state,
            decode_bound_with_schema(
                &raw,
                state.identity,
                state.world_revision,
                PLAYER_HEALING_SCHEMA
            )
            .unwrap()
        );
        for old in 1..=4 {
            assert!(decode_with_schema(&raw, old).is_err());
            assert!(encode_with_schema(&state, old).is_err());
        }
        let h = state.healing.unwrap();
        if label == "initial-caster" {
            let mut wrong = raw.clone();
            wrong[55] ^= 2; // Epoch 1 -> 3: valid format, wrong owner binding.
            assert!(decode_with_schema(&wrong, PLAYER_HEALING_SCHEMA).is_ok());
            assert!(decode_bound_with_schema(
                &wrong,
                state.identity,
                state.world_revision,
                PLAYER_HEALING_SCHEMA
            )
            .is_err());
            peer.send(&wrong);
            assert_eq!(peer.line(), "REJECTED");
        }
        if label == "cross-recipient" {
            assert!(h.outgoing.is_none());
            assert_eq!(state.identity.source.key.source_id, 42);
            assert_eq!(h.incoming.last.unwrap().caster.identity.source_id, 7);
        }
        if label == "full-free" {
            assert_eq!(h.incoming.received, 2);
            assert_eq!(h.incoming.restored, 8);
            let sent = h.outgoing.unwrap();
            assert_eq!(sent.spending.current.spent, 4);
            assert_eq!(sent.last.unwrap().use_receipt.cost, 0);
            assert_eq!(sent.last.unwrap().target.applied, 0);
        }
        if label == "revived-free" {
            assert_eq!(h.outgoing.unwrap().last.unwrap().target.before.life, 2);
        }
        if label == "wide" {
            assert_eq!(state.identity.source.epoch, u64::MAX);
            assert_eq!(state.resources.vitals.hp, i64::MAX);
        }
        if label.starts_with("effects-") {
            let flags: u8 = label.rsplit('-').next().unwrap().parse().unwrap();
            let effects = if label.starts_with("effects-caster-") {
                h.outgoing.unwrap().last.unwrap().effects
            } else {
                h.incoming.last.unwrap().effects
            };
            assert_eq!(effects.water_life, flags & 1 != 0);
            assert_eq!(effects.pollute_water, flags & 2 != 0);
            assert_eq!(effects.dirty_current, flags & 4 != 0);
            assert_eq!(effects.healing_erosion, flags & 8 != 0);
        }
        if label.starts_with("engine-") {
            engine_states += 1;
            let key = state.identity.source.key.source_id;
            if label.starts_with("engine-denied-") {
                let before = &engine_before[&key];
                assert_eq!(state.resources, before.resources);
                assert_eq!(state.healing, before.healing);
            }
            engine_before.insert(key, state.clone());
        }
        if label == "engine-cross-42" {
            assert_eq!(h.incoming.received, 1);
            assert_eq!(h.incoming.last.unwrap().caster.identity.source_id, 7);
        }
        if label == "engine-bot-7" {
            assert!(
                matches!(h.outgoing.unwrap().last.unwrap().target.before.target,
                Target::Participant(p) if p.identity.kind == MemberKind::ServerBot)
            );
        }
        if label == "engine-self-7" {
            assert!(h.incoming.restored > 0);
            assert!(state.resources.flows.damaged > 0);
        }
        peer.send(&raw);
        count += 1;
        assert_eq!(peer.line(), format!("ACK:{count}"));
    }
    assert_eq!(count, 55);
    assert_eq!(engine_states, 12);
    peer.finish();
}
#[test]
#[ignore = "requires explicitly compiled Java payload peer; not TLS/GUI acceptance"]
fn java_real_casters_and_mixed_price_history_round_trip_in_both_directions() {
    let mut peer = Peer::start();
    let mut count = 0;
    let mut previous: Option<State> = None;
    let mut quest_structure = None;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:20");
            break;
        }
        let fields = line.splitn(3, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], "STATE");
        let label = fields[1];
        let raw = bytes(fields[2]);
        let state = decode(&raw).unwrap();
        assert_eq!(raw, encode(&state).unwrap());
        assert_eq!(
            state,
            decode_bound(&raw, state.identity, state.world_revision).unwrap()
        );
        match label {
            "all-lanes" => {
                assert_eq!(state, equipment_history::tests::populated());
                let mut wrong = raw.clone();
                wrong[55] ^= 1;
                assert!(decode(&wrong).is_ok());
                peer.send(&wrong);
                assert_eq!(peer.line(), "REJECTED");
            }
            "repriced" => assert!(equipment_history::follows_reprofile(
                previous.as_ref().unwrap(),
                &state
            )),
            label if label.starts_with("mixed-") => {
                let i = label[6..].parse::<usize>().unwrap();
                assert!(equipment_history::follows_cast(
                    previous.as_ref().unwrap(),
                    &state,
                    LANES[i]
                ));
                assert_eq!(state.spells[&LANES[i]].current.spent, 10 + i as i64);
            }
            "maximum" => assert_eq!(state, super::tests::maximum()),
            "wide" => {
                assert_eq!(state.identity.source.epoch, u64::MAX);
                assert_eq!(state.resources.vitals.maximum_hp, i64::MAX);
            }
            "real-honor-cast" => assert!(equipment_history::follows_cast(
                previous.as_ref().unwrap(),
                &state,
                Lane::LightArrow
            )),
            "real-npc-arrow" => assert!(equipment_history::follows_cast(
                previous.as_ref().unwrap(),
                &state,
                Lane::NpcLightArrow
            )),
            "real-npc-fire" => {
                assert_eq!(state.spells[&Lane::NpcFireStorm].current.casts, 1);
                assert_eq!(state.spells[&Lane::NpcLightArrow].current.casts, 1);
            }
            "real-npc-dis" => {
                assert_eq!(state.spells[&Lane::NpcDisintegrate].current.casts, 1);
                assert_eq!(state.targets.len(), 3);
            }
            "real-quest-structure" => {
                quest_structure = state.targets[&Lane::NpcFireStorm].structure.clone();
                assert!(quest_structure.is_some());
            }
            "real-quest-retained" => assert_eq!(
                state.targets[&Lane::NpcFireStorm].structure,
                quest_structure
            ),
            "real-honor-before" | "real-npc-before" => assert!(state.targets.is_empty()),
            _ => panic!("unexpected scenario {label}"),
        }
        peer.send(&encode(&state).unwrap());
        count += 1;
        assert_eq!(peer.line(), format!("ACK:{count}"));
        previous = Some(state);
    }
    assert_eq!(count, 20);
    peer.finish();
}
