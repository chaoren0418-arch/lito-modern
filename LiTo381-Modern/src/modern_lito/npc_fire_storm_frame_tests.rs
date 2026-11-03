use super::super::{application_protocol as app, protocol};
use super::*;

#[path = "npc_fire_storm_frame_golden.rs"]
mod golden;

fn bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn decode(line: &str) -> (String, Vec<u8>, npc::Reply) {
    let fields: Vec<_> = line.split(':').collect();
    assert_eq!(fields.len(), 6);
    assert_eq!(fields[0], "FRAME");
    let context = app::SelectionContext {
        key: app::CharacterKey {
            source_id: fields[2].parse().unwrap(),
            object_id: fields[3].parse().unwrap(),
        },
        epoch: fields[4].parse().unwrap(),
    };
    let data = bytes(fields[5]);
    let (frame, used) = npc::decode_frame(&data, context.key, 31)
        .unwrap_or_else(|e| panic!("{}: {e:?}", fields[1]));
    assert_eq!(used, data.len());
    let reply = npc::Reply {
        echo: npc::Request {
            context,
            channel_id: 0,
            action: npc::Action::Read,
            world: frame.scope,
            sequence: 0,
            serial: 0,
            life: 0,
        },
        status: npc::Status::Ok,
        reason: npc::Reason::None,
        frame: Some(frame),
    };
    reply.validate().unwrap();
    (fields[1].to_string(), data, reply)
}
fn row(scenario: &str, name: &str) -> (String, Vec<u8>, npc::Reply) {
    decode(
        golden::VECTORS
            .iter()
            .find(|(s, l)| *s == scenario && l.split(':').nth(1) == Some(name))
            .unwrap()
            .1,
    )
}
fn frame(scenario: &str, name: &str) -> npc::Frame {
    row(scenario, name).2.frame.unwrap()
}
fn journal<'a>(lines: impl IntoIterator<Item = &'a str>) {
    let mut previous = BTreeMap::<u32, npc::Reply>::new();
    for line in lines {
        let (name, _, reply) = decode(line);
        let key = reply.echo.context.key.source_id;
        assert!(npc::follows(previous.get(&key), &reply), "journal: {name}");
        if let Some(old) = previous.get(&key) {
            if old.frame.as_ref().unwrap().combat.world.revision
                < reply.frame.as_ref().unwrap().combat.world.revision
            {
                assert!(!npc::follows(Some(&reply), old), "rewind: {name}");
            }
        }
        previous.insert(key, reply);
    }
}

#[test]
fn v32_v33_chat_sessions_decode_all_160_unchanged_v31_nested_npc_world_frames() {
    assert_eq!(
        super::super::world_protocol::CODEC_MAX_VERSION,
        protocol::IMPLEMENTED_WORLD_VERSION
    );
    assert!(protocol::RUNTIME_WORLD_VERSION <= super::super::world_protocol::CODEC_MAX_VERSION);
    assert_eq!(golden::VECTORS.len(), 160);
    for (scenario, line) in golden::VECTORS {
        let (name, data, old) = decode(line);
        for version in [32, 33, 34] {
            let (new, used) = npc::decode_frame(&data, old.echo.context.key, version)
                .unwrap_or_else(|e| panic!("v{version} {scenario}/{name}: {e:?}"));
            assert_eq!(used, data.len());
            assert_eq!(
                Some(new),
                old.frame,
                "v{version} must preserve every field: {scenario}/{name}"
            );
        }
        assert!(npc::decode_frame(&data, old.echo.context.key, protocol::MAX_VERSION + 1).is_err());
    }
}
#[test]
fn all_160_java_complete_frames_validate_and_preserve_source_identity() {
    assert_eq!(golden::VECTORS.len(), 160);
    for (scenario, line) in golden::VECTORS {
        let (name, data, r) = decode(line);
        let f = r.frame.as_ref().unwrap();
        assert!(f.combat.world.descriptor.unwrap().has(D::NPC_FIRE_STORM));
        assert!(!f.names.as_ref().unwrap().is_empty());
        assert!(npc::follows(None, &r), "{scenario}/{name}");
        assert!(data.len() < npc::frame_limit(31));
        assert_eq!(
            r.echo.context.key.object_id,
            if *scenario == "time" { 3333 } else { 1 }
        );
    }
}
#[test]
fn real_engine_journals_keep_atomic_hp_mana_and_shared_sequence_history() {
    for scenario in [
        "normal", "lives", "time", "554", "553", "513", "527", "518", "519", "520", "521", "522",
    ] {
        journal(
            golden::VECTORS
                .iter()
                .filter(|(s, _)| *s == scenario)
                .map(|(_, l)| *l),
        );
    }
    let f = frame("normal", "hit");
    let s = f.combat.npc_fire(1).unwrap();
    assert_eq!(
        (s.count, s.mp_spent, f.combat.vital(1).unwrap().mp),
        (1, 48, 952)
    );
    assert_eq!(s.targets.len(), 2);
    assert_eq!(s.targets[0].hp_before, 9007199254740993);
    assert_eq!(*f.combat.npc_fire(3).unwrap(), State::initial(3, 0));
    let m = frame("normal", "second");
    assert_eq!(m.combat.npc_fire(1).unwrap().mp_spent, 96);
    assert_eq!(m.combat.npc_spell(1).unwrap().mp_spent, 3);
    assert_eq!(m.combat.npc_disintegrate(1).unwrap().mp_spent, 70);
    assert_eq!(m.combat.world.cast(1).unwrap().request_sequence, 5);
}
#[test]
fn each_quest_map_has_private_sparse_own_data_and_a_real_magic_terminal_proof() {
    for scenario in [
        "554", "553", "513", "527", "518", "519", "520", "521", "522",
    ] {
        let max = if matches!(scenario, "513" | "527") {
            8
        } else {
            5
        };
        let own = frame(scenario, &format!("cast-{max}-own"));
        let other = frame(scenario, &format!("cast-{max}-other"));
        assert_eq!(own.combat.world.phase, world::Phase::Finished);
        assert_eq!(
            other.combat.world.quest.as_ref().unwrap().winning_lane,
            Some(0)
        );
        assert_eq!(other.combat.npc_fires.len(), 1);
        assert_eq!(other.combat.npc_fires[0].entity_id, 2);
        assert_eq!(other.combat.npc_fires[0].count, 0);
        assert!(other.combat.npc_fires[0].structure.is_none());
        assert!(other.combat.world.quest.as_ref().unwrap().actors.is_empty());
        assert!(other
            .combat
            .world
            .quest
            .as_ref()
            .unwrap()
            .targets
            .iter()
            .all(|t| t.lane() == 1));
        assert!(own
            .combat
            .world
            .quest
            .as_ref()
            .unwrap()
            .exchanges
            .is_empty());
        let mut missing = own.clone();
        missing.combat.npc_fires[0].structure = None;
        assert!(missing.validate().is_err());
    }
}
#[test]
fn time_tower_history_survives_departure_all_waves_and_later_saved_publication() {
    let tower = frame("time", "tower");
    let saved = frame("time", "saved");
    assert_eq!(
        tower.combat.npc_fires[0].structure,
        saved.combat.npc_fires[0].structure
    );
    assert_eq!(saved.combat.world.phase, world::Phase::Finished);
    let t = saved.combat.world.time_attack.as_ref().unwrap();
    assert!(t.departed(1));
    assert!(t.won(2));
    assert!(t.exchanges.is_empty());
    assert_eq!(
        t.receipt.as_ref().unwrap().state,
        world::SettlementState::Saved
    );
    assert!(
        t.validate().is_err(),
        "legacy appendix alone must not accept a magic-only proof"
    );
    let mut no_proof = saved.clone();
    no_proof.combat.npc_fires[0].structure = None;
    assert!(no_proof.validate().is_err());
    let mut later = saved.clone();
    later.combat.npc_fires[1].revision = saved.combat.world.revision;
    assert!(
        later.validate().is_err(),
        "saving a result is not a fresh cast"
    );
}
#[test]
fn previous_life_structure_is_not_a_fresh_hit_after_revival() {
    let before = frame("lives", "prior");
    let after = frame("lives", "next-life");
    assert_eq!(
        before.combat.npc_fires[0].structure,
        after.combat.npc_fires[0].structure
    );
    assert_eq!(after.combat.npc_fires[0].caster_life, 2);
    assert_eq!(
        after.combat.npc_fires[0]
            .structure
            .as_ref()
            .unwrap()
            .caster_life,
        1
    );
    assert!(after.combat.npc_fires[0].current_structure().is_none());
    let mut invalid = after.clone();
    invalid.combat.npc_fires[0]
        .structure
        .as_mut()
        .unwrap()
        .caster_life = 2;
    assert!(invalid.validate().is_err());
}
#[test]
fn altered_ledger_mana_life_scope_origin_and_npc_authority_are_rejected() {
    let good = frame("normal", "hit");
    let changes: Vec<Box<dyn Fn(&mut npc::Frame)>> = vec![
        Box::new(|f| f.combat.npc_fires.clear()),
        Box::new(|f| f.combat.vitals[0].mp += 1),
        Box::new(|f| f.combat.npc_fires[0].caster_life += 1),
        Box::new(|f| f.combat.npc_fires[0].scope.as_mut().unwrap().generation += 1),
        Box::new(|f| f.combat.npc_fires[0].origin.as_mut().unwrap().x += 1),
        Box::new(|f| f.combat.npc_fires[0].targets[0].life += 1),
        Box::new(|f| f.combat.npc_fires[0].targets[0].cell.x += 1),
        Box::new(|f| {
            f.actors.remove(0);
        }),
        Box::new(|f| {
            let h = &mut f.combat.npc_fires[0].targets[0];
            h.hp_before += 1;
            h.outcome.remaining_hp += 1;
        }),
        Box::new(|f| f.combat.world.descriptor.as_mut().unwrap().features &= !D::NPC_FIRE_STORM),
    ];
    for (i, change) in changes.into_iter().enumerate() {
        let mut altered = good.clone();
        change(&mut altered);
        assert!(altered.validate().is_err(), "mutation {i}");
    }
    let mut collision = frame("normal", "second");
    collision.combat.npc_fires[0].revision = collision.combat.npc_spells[0].revision;
    assert!(collision.validate().is_err());
}
#[test]
fn missing_history_and_cross_spell_sequence_reuse_do_not_follow_prior_authority() {
    let prior = row("normal", "hit").2;
    let mut next = row("normal", "second").2;
    next.frame.as_mut().unwrap().combat.world.casts[0].request_sequence = 2;
    assert!(!npc::follows(Some(&prior), &next));
    let initial = row("time", "tower").2;
    let mut after = row("time", "saved").2;
    after.frame.as_mut().unwrap().combat.npc_fires[0].structure = None;
    assert!(!npc::follows(Some(&initial), &after));
}
#[test]
fn all_truncations_and_old_or_future_nested_versions_are_rejected_without_panic() {
    let (_, data, reply) = row("time", "tower");
    let key = reply.echo.context.key;
    for n in 0..data.len() {
        assert!(
            npc::decode_frame(&data[..n], key, 31).is_err(),
            "prefix {n}"
        );
    }
    for version in (1..=30).chain([protocol::MAX_VERSION + 1, u16::MAX]) {
        assert!(
            npc::decode_frame(&data, key, version).is_err(),
            "version {version}"
        );
    }
    for i in 0..data.len() {
        let mut changed = data.clone();
        changed[i] ^= 0x80;
        let decoded = std::panic::catch_unwind(|| npc::decode_frame(&changed, key, 31));
        assert!(decoded.is_ok(), "panic at byte {i}");
        if let Ok(Ok((f, n))) = decoded {
            assert_eq!(n, data.len());
            f.validate().unwrap();
        }
    }
    assert!(npc::decode_frame(&vec![0; npc::frame_limit(31) + 1], key, 31).is_err());
}
#[test]
fn npc70_frames_require_their_version_and_explicit_capability_not_legacy_modes() {
    let r = row("normal", "hit").2;
    for version in [30, 31, 32] {
        let ack = protocol::Ack {
            request_id: 1,
            version,
            numeric_schema: 1,
            capabilities: protocol::KNOWN_CAPABILITIES,
        };
        assert_eq!(
            world::descriptor_negotiated(&app::Response::Npc(r.clone()), &ack),
            version >= 31
        );
        assert!(!world::descriptor_negotiated(
            &app::Response::Npc(r.clone()),
            &protocol::Ack {
                capabilities: ack.capabilities & !protocol::CAP_NPC_FIRE_STORM,
                ..ack
            }
        ));
    }
    assert_eq!(protocol::MAX_VERSION, 47);
    assert_eq!(protocol::RUNTIME_WORLD_VERSION, 37);
    assert_eq!(npc::frame_limit(30), 256 * 1024);
    assert_eq!(npc::frame_limit(31), 1024 * 1024);
    assert!(app::decode_response_version(&[], 31).is_err());
    let base = frame("normal", "loading").combat.world.descriptor.unwrap();
    for flag in [D::NPC, D::SPELLS, D::REGEN, D::ITEMS, D::WEIGHT] {
        let mut d = base;
        d.features &= !flag;
        assert!(d.validate(2).is_err());
    }
}

#[test]
fn retained_old_objective_cannot_claim_the_next_objectives_birth_event() {
    let original = frame("513", "cast-2-own").combat.npc_fires[0]
        .structure
        .clone()
        .unwrap();
    let mut altered = frame("513", "cast-4-own");
    let born = altered
        .combat
        .world
        .quest
        .as_ref()
        .unwrap()
        .actor(2)
        .unwrap()
        .born_revision;
    let mut forged = original;
    forged.count = 3;
    forged.sequence = 3;
    forged.revision = born;
    altered.combat.npc_fires[0].structure = Some(forged);
    altered.combat.npc_fires[0].validate().unwrap();
    assert!(altered.validate().is_err());
}
#[test]
fn individually_valid_historical_origin_cannot_rewrite_an_observed_receipt() {
    let prior = row("513", "cast-2-own").2;
    let mut after = row("513", "cast-3-own").2;
    after.frame.as_mut().unwrap().combat.npc_fires[0]
        .structure
        .as_mut()
        .unwrap()
        .origin
        .x += 1;
    after.validate().unwrap();
    assert!(
        !npc::follows(Some(&prior), &after),
        "cached structure history must stay exact"
    );
}

struct Peer {
    child: std::process::Child,
    lines: std::sync::mpsc::Receiver<String>,
    reader: Option<std::thread::JoinHandle<()>>,
}
impl Peer {
    fn start(scenario: &str) -> Self {
        use std::{
            io::{BufRead, BufReader, Read},
            process::{Command, Stdio},
        };
        let mut command =
            Command::new(std::env::var_os("LITO_MODERN_TEST_JAVA").expect("explicit test JDK"));
        command
            .arg("-cp")
            .arg(
                std::env::var_os("LITO_MODERN_TEST_JAVA_CLASSPATH")
                    .expect("compiled test classpath"),
            )
            .arg("local.lito381.server.ModernNpcFireStormFramePeer")
            .arg(scenario)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().expect("candidate atomic Java peer");
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout.take(4 * 1024 * 1024)).lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            lines,
            reader: Some(reader),
        }
    }
    fn collect(&mut self) -> Vec<String> {
        use std::time::{Duration, Instant};
        let end = Instant::now() + Duration::from_secs(30);
        let mut rows = Vec::new();
        loop {
            let line = self
                .lines
                .recv_timeout(end.saturating_duration_since(Instant::now()))
                .expect("bounded complete-frame peer");
            if line == "CLOSED" {
                break;
            }
            assert!(line.starts_with("FRAME:"));
            rows.push(line);
            assert!(rows.len() <= 32);
        }
        let exit = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(Instant::now() < exit);
            std::thread::sleep(Duration::from_millis(10));
        }
        self.reader.take().unwrap().join().unwrap();
        rows
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}
fn live(scenario: &str, expected: usize) -> Vec<String> {
    let lines = Peer::start(scenario).collect();
    assert_eq!(lines.len(), expected);
    journal(lines.iter().map(String::as_str));
    lines
}
#[test]
#[ignore = "requires compiled Java engine peer; complete atomic frame, not application/TLS/UI"]
fn java_npc70_atomic_live_mixed_magic_denial_regen_and_abort() {
    live("normal", 9);
}
#[test]
#[ignore = "requires compiled Java engine peer; complete atomic frame, not application/TLS/UI"]
fn java_npc70_atomic_live_death_revival_retains_previous_life_structure() {
    live("lives", 4);
}
#[test]
#[ignore = "requires compiled Java engine peer; complete atomic frame, not application/TLS/UI"]
fn java_npc70_atomic_live_tower_departure_four_waves_and_saved_result() {
    live("time", 9);
}
#[test]
#[ignore = "requires compiled Java engine peer; all nine logical Quest maps use test geometry"]
fn java_npc70_atomic_live_all_nine_quest_maps_with_both_private_views() {
    for map in [
        "554", "553", "513", "527", "518", "519", "520", "521", "522",
    ] {
        live(map, if matches!(map, "513" | "527") { 20 } else { 14 });
    }
}
#[test]
#[ignore = "requires compiled Java peer; synthetic maximum-count whole frame, not 100 live players"]
fn java_npc70_atomic_live_maximum_counts_exceed_legacy_envelope_without_lost_hits() {
    let lines = live("wide", 1);
    let (_, data, reply) = decode(&lines[0]);
    let f = reply.frame.unwrap();
    assert!(data.len() > npc::frame_limit(30));
    assert!(data.len() < npc::frame_limit(31) - 16);
    assert_eq!(f.combat.npc_fires.len(), 100);
    assert_eq!(f.actors.len(), 96);
    assert_eq!(
        f.combat
            .npc_fires
            .iter()
            .map(|s| s.targets.len())
            .sum::<usize>(),
        4800
    );
    assert!(f
        .combat
        .npc_fires
        .iter()
        .all(|s| s.targets.len() == 48 && s.mp_spent == 48));
    assert!(npc::decode_frame(&data, reply.echo.context.key, 30).is_err());
    let mut shared_revision = f.clone();
    shared_revision.combat.npc_fires[1].revision = shared_revision.combat.npc_fires[0].revision;
    shared_revision.combat.npc_fires[1].validate().unwrap();
    assert!(
        shared_revision.validate().is_err(),
        "two casters cannot own one authority event"
    );
    eprintln!(
        "NPC70 complete maximum-count frame: {} bytes, 100 casters, 4800 retained hits",
        data.len()
    );
}
