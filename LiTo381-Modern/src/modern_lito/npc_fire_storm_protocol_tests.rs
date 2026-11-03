use super::super::protocol;
use super::*;
use std::io::{Read, Write};
#[path = "npc_fire_storm_frame_golden.rs"]
mod golden;

#[test]
fn npc70_capability_is_v31_dependency_checked_and_not_player_advertised() {
    let bit = protocol::CAP_NPC_FIRE_STORM;
    assert_eq!(bit, 2097152);
    assert_eq!(
        protocol::KNOWN_CAPABILITIES
            & !(protocol::CAP_SLOW_DATA
                | protocol::CAP_HASTE
                | protocol::CAP_GREATER_HASTE
                | protocol::CAP_ACTION_LIFE
                | protocol::CAP_CHAT
                | protocol::CAP_DIRECT_CHAT
                | protocol::CAP_FILTERED_CHAT
                | protocol::CAP_MATCHMAKING
                | protocol::CAP_LARGE_MAPS
                | protocol::CAP_BATTLE_DATA),
        4194303
    );
    assert_eq!(protocol::RUNTIME_CAPABILITIES & bit, 0);
    let caps = protocol::RUNTIME_CAPABILITIES | protocol::CAP_NPC | bit;
    let mut h = protocol::Hello::current(1);
    h.supported_capabilities = caps;
    h.required_capabilities = caps;
    assert!(protocol::encode(protocol::Message::Hello(h)).is_ok());
    for version in 1..31 {
        h.max_version = version;
        assert!(protocol::encode(protocol::Message::Hello(h)).is_err());
        assert!(protocol::encode(protocol::Message::Ack(protocol::Ack {
            request_id: 1,
            version,
            numeric_schema: 1,
            capabilities: caps
        }))
        .is_err());
    }
    h.max_version = 31;
    for dep in [
        protocol::CAP_NPC,
        protocol::CAP_REGEN,
        protocol::CAP_MATCH_DESCRIPTOR,
        protocol::CAP_COMBAT,
    ] {
        h.supported_capabilities = caps & !dep;
        h.required_capabilities = 3;
        assert!(protocol::encode(protocol::Message::Hello(h)).is_err());
        h.supported_capabilities = caps;
        h.required_capabilities = caps & !dep;
        assert!(protocol::encode(protocol::Message::Hello(h)).is_err());
    }
}
fn bytes(h: &str) -> Vec<u8> {
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
pub(crate) fn sample(scenario: &str, name: &str) -> (Request, Vec<u8>, npc::Frame) {
    let fields: Vec<_> = golden::VECTORS
        .iter()
        .find(|(s, l)| *s == scenario && l.split(':').nth(1) == Some(name))
        .unwrap()
        .1
        .split(':')
        .collect();
    let context = app::SelectionContext {
        key: app::CharacterKey {
            source_id: fields[2].parse().unwrap(),
            object_id: fields[3].parse().unwrap(),
        },
        epoch: fields[4].parse().unwrap(),
    };
    let raw = bytes(fields[5]);
    let (f, n) = npc::decode_frame(&raw, context.key, 31).unwrap();
    assert_eq!(n, raw.len());
    let a = f
        .combat
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(context.key))
        .unwrap();
    let r = Request {
        context,
        channel_id: 0,
        scope: f.scope,
        sequence: f
            .combat
            .world
            .cast(a.entity_id)
            .unwrap()
            .request_sequence
            .max(1),
        caster_life: super::super::quest_life::life(&f.combat.world, a.entity_id)
            .map_or(1, |l| l.incarnation()),
    };
    (r, raw, f)
}
pub(crate) fn snapshot(scenario: &str, name: &str) -> npc::Reply {
    let (r, _, f) = sample(scenario, name);
    let reply = npc::Reply {
        echo: npc::Request {
            context: r.context,
            channel_id: r.channel_id,
            action: npc::Action::Read,
            world: r.scope,
            sequence: 0,
            serial: 0,
            life: 0,
        },
        status: npc::Status::Ok,
        reason: npc::Reason::None,
        frame: Some(f),
    };
    reply.validate().unwrap();
    reply
}
fn packet(r: &Request, raw: Option<&[u8]>, status: npc::Status, reason: Reason) -> Vec<u8> {
    let mut b = app::header(91, RESULT_KIND, 84 + raw.map_or(0, |v| v.len())).unwrap();
    b.extend(wire::encode_request(r, 31).unwrap());
    b.extend([status as u8, reason as u8, u8::from(raw.is_some()), 0]);
    if let Some(v) = raw {
        b.extend(v);
    }
    b
}
fn decoded(b: &[u8]) -> Reply {
    let f = app::decode_response_version(b, 31).unwrap();
    assert_eq!(f.request_id, 91);
    let app::Response::NpcFireStorm(r) = f.response else {
        panic!("wrong result kind")
    };
    r
}
#[test]
fn exact_request_and_absent_response_preserve_wide_context_without_client_damage() {
    let r = Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: 7,
                object_id: 9007199254740993,
            },
            epoch: u64::MAX,
        },
        channel_id: 3,
        scope: npc::Scope {
            room_id: 11,
            match_id: 12,
            generation: 13,
        },
        sequence: 17,
        caster_life: 19,
    };
    let q = encode_request(u32::MAX, &r, 31).unwrap();
    assert_eq!(q.len(), 80);
    assert_eq!(q[12..16], [0, 48, 0, 0]);
    assert_eq!(&q[16..],bytes("000000070020000000000001ffffffffffffffff03000000000000000000000b000000000000000c000000000000000d00000000000000110000000000000013"));
    assert_eq!(wire::decode_request(&q[16..], 31).unwrap(), r);
    let b = packet(&r, None, npc::Status::Unavailable, Reason::NotActive);
    assert_eq!(b.len(), 84);
    assert!(decoded(&b).confirms(&r));
    for v in 1..=30 {
        assert!(encode_request(1, &r, v).is_err());
        assert!(app::decode_response_version(&b, v).is_err());
    }
    assert_eq!(
        encode_request(1, &r, 32).unwrap(),
        encode_request(1, &r, 31).unwrap()
    );
    assert!(encode_request(1, &r, protocol::MAX_VERSION + 1).is_err());
    assert!(encode_request(0, &r, 31).is_err());
}
#[test]
fn real_complete_success_denial_and_terminal_results_validate_the_owner_receipt() {
    let (r, raw, _) = sample("normal", "hit");
    let hit = decoded(&packet(&r, Some(&raw), npc::Status::Ok, Reason::None));
    assert!(hit.confirms(&r));
    let (_, _, before) = sample("normal", "before");
    assert!(r.ready_in(&before));
    assert!(confirms_transition(&before, &hit));
    let (r, raw, _) = sample("normal", "denied");
    let deny = decoded(&packet(
        &r,
        Some(&raw),
        npc::Status::Denied,
        Reason::TooFast,
    ));
    assert!(confirms_transition(hit.frame.as_ref().unwrap(), &deny));
    assert!(!confirms_transition(&before, &deny));
    for name in [
        "554", "553", "513", "527", "518", "519", "520", "521", "522",
    ] {
        let final_name = if matches!(name, "513" | "527") {
            "cast-8-own"
        } else {
            "cast-5-own"
        };
        let (r, raw, _) = sample(name, final_name);
        let done = decoded(&packet(&r, Some(&raw), npc::Status::Ok, Reason::None));
        assert_eq!(
            done.frame.unwrap().combat.world.phase,
            world::Phase::Finished
        );
    }
    let (r, raw, _) = sample("time", "wave-4");
    assert_eq!(
        decoded(&packet(&r, Some(&raw), npc::Status::Ok, Reason::None)).status,
        npc::Status::Ok
    );
}
#[test]
fn forged_success_reason_scope_life_and_owner_changes_fail_atomically() {
    let (r, raw, _) = sample("normal", "hit");
    for reason in [Reason::NotActive, Reason::Dead, Reason::NoMp] {
        assert!(app::decode_response_version(
            &packet(&r, Some(&raw), npc::Status::Denied, reason),
            31
        )
        .is_err());
    }
    for edit in [0, 1, 2, 3] {
        let mut bad = r;
        match edit {
            0 => bad.sequence += 1,
            1 => bad.caster_life += 1,
            2 => bad.scope.generation += 1,
            _ => bad.context.key.source_id = 2,
        }
        assert!(app::decode_response_version(
            &packet(&bad, Some(&raw), npc::Status::Ok, Reason::None),
            31
        )
        .is_err());
    }
    let (_, _, loading) = sample("normal", "loading");
    assert!(!r.ready_in(&loading));
    let mut p = decoded(&packet(&r, Some(&raw), npc::Status::Ok, Reason::None));
    p.echo.context.epoch += 1;
    assert!(!p.confirms(&r));
    let mut old = sample("normal", "before").2;
    old.combat.world.actors[0].sequence += 1;
    assert!(!confirms_transition(
        &old,
        &decoded(&packet(&r, Some(&raw), npc::Status::Ok, Reason::None))
    ));
}

#[test]
fn later_world_events_do_not_erase_an_exact_success_receipt() {
    let (r, raw, frame) = sample("normal", "regen");
    assert!(frame.combat.npc_fire(1).unwrap().revision < frame.combat.world.revision);
    let reply = decoded(&packet(&r, Some(&raw), npc::Status::Ok, Reason::None));
    let (_, _, before) = sample("normal", "disintegrate");
    assert!(confirms_transition(&before, &reply));
    let mut wrong = r;
    wrong.sequence += 1;
    assert!(app::decode_response_version(
        &packet(&wrong, Some(&raw), npc::Status::Ok, Reason::None),
        31
    )
    .is_err());
}

#[test]
fn other_spell_confirmations_cannot_rewrite_the_same_owners_fire_history() {
    use super::super::{npc_disintegrate_protocol as disintegrate, npc_spell_protocol as arrow};
    let (r, _, after) = sample("normal", "arrow");
    let s = after.combat.npc_spell(1).unwrap();
    let hit = s.receipt.as_ref().unwrap();
    let reply = arrow::Reply {
        echo: arrow::Request {
            context: r.context,
            channel_id: 0,
            target: hit.target,
            sequence: s.request_sequence,
            caster_life: s.caster_life,
            target_life: hit.target_life,
        },
        status: npc::Status::Ok,
        reason: arrow::Reason::None,
        frame: Some(after.clone()),
    };
    reply.validate().unwrap();
    let mut before = sample("normal", "denied").2;
    assert!(arrow::confirms_transition(&before, &reply));
    before.combat.npc_fires[0].origin.as_mut().unwrap().x -= 1;
    before.validate().unwrap();
    assert!(!arrow::confirms_transition(&before, &reply));

    let (r, _, after) = sample("normal", "disintegrate");
    let s = after.combat.npc_disintegrate(1).unwrap();
    let hit = s.hit.as_ref().unwrap();
    let reply = disintegrate::Reply {
        echo: disintegrate::Request {
            context: r.context,
            channel_id: 0,
            target: hit.target,
            sequence: s.last_sequence,
            caster_life: hit.caster_life,
            target_life: hit.target_life,
        },
        status: npc::Status::Ok,
        reason: disintegrate::Reason::None,
        frame: Some(after.clone()),
    };
    reply.validate().unwrap();
    let mut before = sample("normal", "arrow").2;
    assert!(disintegrate::confirms_transition(&before, &reply));
    before.combat.npc_fires[0].origin.as_mut().unwrap().x -= 1;
    before.validate().unwrap();
    assert!(!disintegrate::confirms_transition(&before, &reply));
}
#[test]
fn complete_packet_truncations_reserved_bytes_enums_and_trailing_data_fail() {
    let (r, raw, _) = sample("normal", "hit");
    let b = packet(&r, Some(&raw), npc::Status::Ok, Reason::None);
    for n in 0..b.len() {
        assert!(app::decode_response_version(&b[..n], 31).is_err(), "{n}");
    }
    for offset in [15, 37, 38, 39, 80, 81, 82, 83] {
        let mut bad = b.clone();
        bad[offset] = 255;
        assert!(app::decode_response_version(&bad, 31).is_err(), "{offset}");
    }
    let mut extra = b.clone();
    extra.push(0);
    let n = extra.len() as u32;
    extra[4..8].copy_from_slice(&n.to_be_bytes());
    assert!(app::decode_response_version(&extra, 31).is_err());
    let mut combined = b.clone();
    combined.extend(&b);
    let mut stream = std::io::Cursor::new(combined);
    assert_eq!(
        app::read_response_version(&mut stream, 31)
            .unwrap()
            .response,
        app::Response::NpcFireStorm(decoded(&b))
    );
    assert_eq!(stream.position(), b.len() as u64);
    assert!(app::read_response_version(&mut stream, 31).is_ok());
}
#[test]
fn invalid_negotiated_kind_or_size_is_rejected_before_body_read_or_allocation() {
    struct HeaderOnly(std::io::Cursor<Vec<u8>>);
    impl Read for HeaderOnly {
        fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
            assert!(self.0.position() < 16, "body read for invalid header");
            self.0.read(b)
        }
    }
    for (kind, size, version) in [
        (39, 256 * 1024 + 1, 30),
        (41, 256 * 1024 + 1, 30),
        (47, 256 * 1024 + 1, 30),
        (49, 84, 30),
        (49, 83, 31),
        (39, 1024 * 1024 + 1, 31),
        (41, 1024 * 1024 + 1, 31),
        (47, 1024 * 1024 + 1, 31),
        (49, 1024 * 1024 + 1, 31),
    ] {
        let mut h = b"LTM1".to_vec();
        h.extend((size as u32).to_be_bytes());
        h.extend(91_u32.to_be_bytes());
        h.extend((kind as u16).to_be_bytes());
        h.extend([0, 0]);
        assert!(
            app::read_response_version(&mut HeaderOnly(std::io::Cursor::new(h)), version).is_err()
        );
    }
}
#[test]
fn npc70_explicit_opt_in_is_required_even_for_absent_results() {
    let (r, raw, _) = sample("normal", "hit");
    let response =
        app::decode_response_version(&packet(&r, Some(&raw), npc::Status::Ok, Reason::None), 31)
            .unwrap()
            .response;
    assert_eq!(protocol::MAX_VERSION, 47);
    assert_eq!(protocol::RUNTIME_WORLD_VERSION, 37);
    let ack = protocol::Ack {
        request_id: 1,
        version: 31,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    };
    let absent = app::Response::NpcFireStorm(Reply {
        echo: r,
        frame: None,
        status: npc::Status::Unavailable,
        reason: Reason::NotActive,
    });
    for response in [response, absent] {
        assert!(world::descriptor_negotiated(&response, &ack));
        for bit in [
            protocol::CAP_NPC_FIRE_STORM,
            protocol::CAP_NPC,
            protocol::CAP_REGEN,
            protocol::CAP_MATCH_DESCRIPTOR,
        ] {
            assert!(!world::descriptor_negotiated(
                &response,
                &protocol::Ack {
                    capabilities: ack.capabilities & !bit,
                    ..ack
                }
            ));
        }
        assert!(!world::descriptor_negotiated(
            &response,
            &protocol::Ack { version: 30, ..ack }
        ));
    }
}

struct Peer {
    child: std::process::Child,
    input: Option<std::process::ChildStdin>,
    lines: std::sync::mpsc::Receiver<String>,
    reader: Option<std::thread::JoinHandle<()>>,
}
impl Peer {
    fn start(scenario: &str) -> Self {
        use std::{
            io::{BufRead, BufReader},
            process::{Command, Stdio},
        };
        let mut c =
            Command::new(std::env::var_os("LITO_MODERN_TEST_JAVA").expect("explicit JDK17"));
        c.arg("-cp")
            .arg(
                std::env::var_os("LITO_MODERN_TEST_JAVA_CLASSPATH")
                    .expect("compiled test classpath"),
            )
            .arg("local.lito381.server.ModernNpcFireStormProtocolPeer")
            .arg(scenario)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(0x08000000);
        }
        let mut child = c.spawn().unwrap();
        let input = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            for l in BufReader::new(stdout.take(8 * 1024 * 1024)).lines() {
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
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("bounded application peer response")
    }
    fn send(&mut self, r: &Request) -> String {
        let b = encode_request(91, r, 31).unwrap();
        let h = b.iter().map(|n| format!("{n:02x}")).collect::<String>();
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{h}").unwrap();
        input.flush().unwrap();
        self.line()
    }
    fn finish(&mut self) {
        assert_eq!(self.line(), "CLOSED");
        self.input.take();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(10));
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
fn live(scenario: &str, casts: usize) {
    let mut peer = Peer::start(scenario);
    let hello = peer.line();
    let fields: Vec<_> = hello.split(':').collect();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0], "READY");
    let q = bytes(fields[1]);
    assert_eq!(q.len(), 80);
    let mut r = wire::decode_request(&q[16..], 31).unwrap();
    let app::Response::Npc(mut old) = app::decode_response_version(&bytes(fields[2]), 31)
        .unwrap()
        .response
    else {
        panic!("initial NPC read")
    };
    for i in 0..casts + 3 {
        r.sequence = if i <= 1 { 1 } else { i as u64 };
        let response = peer.send(&r);
        let (tag, hex) = response.split_once(':').unwrap();
        if i == 1 {
            assert_eq!(tag, "REJECTED");
            let app::Response::Npc(replay) = app::decode_response_version(&bytes(hex), 31)
                .unwrap()
                .response
            else {
                panic!("replay read")
            };
            assert_eq!(old.frame, replay.frame);
            continue;
        }
        assert_eq!(tag, "RESULT");
        let reply = decoded(&bytes(hex));
        assert!(reply.confirms(&r));
        assert!(
            confirms_transition(old.frame.as_ref().unwrap(), &reply),
            "{scenario}/{i}"
        );
        assert_eq!(
            reply.reason,
            if i == 2 {
                Reason::TooFast
            } else if i == casts + 2 {
                Reason::NotActive
            } else {
                Reason::None
            }
        );
        old = reply.npc_reply();
    }
    let f = old.frame.unwrap();
    assert_eq!(f.combat.npc_fire(1).unwrap().count, casts as i64);
    assert_eq!(f.combat.npc_fire(1).unwrap().mp_spent, 48 * casts as i64);
    assert_eq!(
        f.combat.world.phase,
        if scenario == "normal" {
            world::Phase::Aborted
        } else {
            world::Phase::Finished
        }
    );
    peer.finish();
}
#[test]
#[ignore = "requires compiled Java application engine peer; not TLS/SQL/UI"]
fn java_npc70_application_live_normal_replay_denial_and_abort() {
    live("normal", 2);
}
#[test]
#[ignore = "requires compiled Java application engine peer; test Quest geometry"]
fn java_npc70_application_live_quest_symbols_terminal() {
    live("symbols", 8);
}
#[test]
#[ignore = "requires compiled Java application engine peer; test Quest geometry"]
fn java_npc70_application_live_quest_guardian_terminal() {
    live("guardian", 5);
}
#[test]
#[ignore = "requires compiled Java application engine peer; test Time Attack geometry"]
fn java_npc70_application_live_time_tower_waves_terminal() {
    live("time", 5);
}
#[test]
#[ignore = "requires compiled Java application peer; synthetic maximum-count frames"]
fn java_npc70_application_live_all_four_large_reply_kinds() {
    let mut peer = Peer::start("large");
    for expected_kind in [39_u16, 41, 47, 49] {
        let line = peer.line();
        let (tag, hex) = line.split_once(':').unwrap();
        assert_eq!(tag, "LARGE");
        let b = bytes(hex);
        assert!(b.len() > 320 * 1024);
        assert!(b.len() < 1024 * 1024);
        assert_eq!(
            u16::from_be_bytes(b[12..14].try_into().unwrap()),
            expected_kind
        );
        let response = app::read_response_version(&mut std::io::Cursor::new(&b), 31)
            .unwrap()
            .response;
        let f = match response {
            app::Response::Npc(r) => r.frame,
            app::Response::NpcSpell(r) => r.frame,
            app::Response::NpcDisintegrate(r) => r.frame,
            app::Response::NpcFireStorm(r) => r.frame,
            _ => panic!("wrong kind"),
        }
        .unwrap();
        assert_eq!(f.combat.npc_fires.len(), 100);
        assert_eq!(
            f.combat
                .npc_fires
                .iter()
                .map(|s| s.targets.len())
                .sum::<usize>(),
            4800
        );
        assert_eq!(f.actors.len(), 96);
        assert!(app::decode_response_version(&b, 30).is_err());
        eprintln!("NPC70 application kind {expected_kind}: {} bytes", b.len());
    }
    peer.finish();
}
