//! Real complete Java publication bytes -> all twelve typed Rust sections ->
//! fresh Rust encoding -> exact Java digest. Not full semantic/TLS/UI admission.
use super::*;
#[path = "battle_armor_wire_tests.rs"]
mod armor;
#[path = "battle_armor_admission_tests.rs"]
mod armor_admission;
#[path = "battle_follow_interop_tests.rs"]
mod follow;
#[path = "battle_greater_haste_wire_tests.rs"]
mod greater_haste;
#[path = "battle_haste_wire_tests.rs"]
mod haste;
#[path = "battle_healing_wire_tests.rs"]
mod healing;
#[path = "battle_healing_admission_tests.rs"]
mod healing_admission;
#[path = "battle_player_healing_wire_tests.rs"]
mod healing_command;
#[path = "battle_healing_reply_tests.rs"]
mod healing_replies;
#[path = "battle_healing_scene_tests.rs"]
mod healing_scene;
#[path = "battle_npc_motion_wire_tests.rs"]
mod npc_motion;
#[path = "battle_operation_interop_tests.rs"]
mod operations;
#[path = "battle_presentation_interop_tests.rs"]
mod presentation;
#[path = "battle_speed_wire_tests.rs"]
mod speed;
#[path = "battle_vip_wire_tests.rs"]
mod vip;
#[path = "battle_vip_selected_tests.rs"]
mod vip_selected;
#[path = "battle_vip_equipment_interop_tests.rs"]
mod vip_equipment;
#[path = "battle_world_interop_tests.rs"]
mod worlds;
use sha2::{Digest, Sha256};
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
struct Peer {
    child: Child,
    lines: mpsc::Receiver<String>,
    reader: Option<std::thread::JoinHandle<()>>,
}
impl Peer {
    fn start() -> Self {
        Self::start_mode(false)
    }
    fn start_mode(follow: bool) -> Self {
        Self::start_named(if follow { Some("follow") } else { None })
    }
    fn start_named(mode: Option<&str>) -> Self {
        use std::os::windows::process::CommandExt;
        let mut c =
            Command::new(std::env::var_os("LITO_MODERN_TEST_JAVA").expect("explicit JDK17"));
        c.arg("-cp")
            .arg(
                std::env::var_os("LITO_MODERN_TEST_JAVA_CLASSPATH")
                    .expect("compiled test classpath"),
            )
            .arg("local.lito381.server.ModernBattleWirePeer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .creation_flags(0x08000000);
        if let Some(mode) = mode {
            c.arg(mode);
        }
        let mut child = c.spawn().expect("owned battle payload peer");
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout.take(16 * 1024 * 1024)).lines() {
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
    fn line(&self) -> String {
        self.lines
            .recv_timeout(Duration::from_secs(20))
            .expect("bounded payload peer response")
    }
    fn ack(&mut self, b: &[u8]) {
        let digest = Sha256::digest(b);
        let text = digest
            .iter()
            .map(|x| format!("{x:02x}"))
            .collect::<String>();
        let input = self.child.stdin.as_mut().unwrap();
        writeln!(input, "{text}").unwrap();
        input.flush().unwrap();
    }
    fn finish(&mut self) {
        self.child.stdin.take();
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
        self.child.stdin.take();
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        if let Some(r) = self.reader.take() {
            let _ = r.join();
        }
    }
}
fn i64_at(b: &[u8], at: usize) -> i64 {
    i64::from_be_bytes(b[at..at + 8].try_into().unwrap())
}
fn i32_at(b: &[u8], at: usize) -> i32 {
    i32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}
#[test]
#[ignore = "explicit compiled Java real publication peer; typed round trip, not full semantic/TLS/UI admission"]
fn java_complete_battle_payloads_preserve_sections_owner_and_engine_revision() {
    let mut peer = Peer::start();
    let mut labels = Vec::new();
    let mut honor_public = None;
    let mut quest_result = None;
    let mut lanes = std::collections::BTreeSet::new();
    let mut target_kinds = std::collections::BTreeSet::new();
    let mut detail_kinds = std::collections::BTreeSet::new();
    let mut combat_mutations = 0;
    let mut personal_mutations = 0;
    let mut scene_mutations = 0;
    let mut mode_mutations = 0;
    loop {
        let line = peer.line();
        if line.starts_with("CLOSED:") {
            assert_eq!(line, "CLOSED:40");
            break;
        }
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "FRAME");
        let label = fields[1];
        let metadata = fields[2]
            .split(',')
            .map(|s| s.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(metadata.len(), 30);
        let hex = fields[3];
        assert!(hex.len() % 2 == 0 && hex.len() <= MAX_BYTES * 2);
        let bytes = hex
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let payload = inspect(&bytes).unwrap();
        let typed = decode(&bytes).expect("all twelve typed sections");
        assert_eq!(format!("{typed:?}"), "UnvalidatedBattleData[redacted]");
        let rebuilt = encode(&typed).expect("new encoding from typed values, not retained input");
        assert_eq!(rebuilt, bytes, "{label}: every typed field must round trip");
        assert_eq!(decode(&rebuilt).unwrap(), typed);
        assert_eq!(typed.world.identity.room, metadata[0]);
        assert_eq!(typed.world.identity.match_id, metadata[1]);
        assert_eq!(typed.world.identity.generation, metadata[2]);
        assert_eq!(typed.world.revision, metadata[3]);
        assert_eq!(i64::from(typed.personal.player.source), metadata[4]);
        assert_eq!(typed.personal.player.object, metadata[5]);
        assert_eq!(typed.personal.player.epoch, metadata[6]);
        assert_eq!(typed.personal.entity, metadata[7]);
        let a = &typed.actors.0[0];
        let v = a.vital.as_ref();
        let facts = [
            typed.world.phase as i64,
            i64::from(typed.world.lane),
            typed.world.descriptor.ruleset as i64,
            i64::from(typed.room.mode),
            i64::from(typed.room.map),
            i64::from(typed.manifest.map),
            typed.actors.0.len() as i64,
            a.entity,
            i64::from(a.position.x),
            i64::from(a.position.y),
            v.map_or(0, |v| v.values.hp),
            v.map_or(0, |v| v.values.maximum_hp),
            v.map_or(0, |v| v.values.mp),
            v.map_or(0, |v| v.values.maximum_mp),
            v.map_or(0, |v| v.ac),
            typed
                .personal
                .equipment
                .as_ref()
                .map_or(-1, |e| e.saved.food225),
            typed.personal.skills.0.len() as i64,
            a.spells
                .histories
                .0
                .iter()
                .filter(|s| s.receipt.is_some())
                .count() as i64,
            typed.npcs.as_ref().map_or(-1, |n| n.actors.0.len() as i64),
            typed.result.as_ref().map_or(-1, |r| r.engine_revision),
            typed
                .personal
                .history
                .as_ref()
                .map_or(-1, |h| h.0.resources.flows.spent),
            typed.personal.ammo.as_ref().map_or(-1, |a| a.shot_count),
        ];
        assert_eq!(
            facts,
            metadata[8..],
            "{label}: independently read Java field meanings"
        );
        for actor in &typed.actors.0 {
            let mut impacts = Vec::new();
            for spell in &actor.spells.histories.0 {
                if let Some(r) = &spell.receipt {
                    lanes.insert(r.impact.lane as u8);
                    impacts.push(&r.impact);
                    detail_kinds.insert(match r.detail {
                        data::Detail::Ordinary => 0,
                        data::Detail::Arrow { .. } => 1,
                        data::Detail::FireStorm { .. } => 2,
                        data::Detail::NpcFireStorm { .. } => 3,
                        data::Detail::Teleport { .. } => 4,
                        data::Detail::Disintegrate { .. } => 5,
                        data::Detail::GreaterHaste { .. } => 6,
                        data::Detail::Haste(_) => 7,
                        data::Detail::Slow(_) => 8,
                        data::Detail::GreaterHasteNeutralized(_) => 9,
                    });
                }
            }
            if let Some(s) = &actor.spells.structure {
                impacts.push(&s.impact);
            }
            for impact in impacts {
                for effect in &impact.targets.0 {
                    target_kinds.insert(match effect.target {
                        data::Target::Participant { .. } => 0,
                        data::Target::Npc { .. } => 1,
                        data::Target::Honor { .. } => 2,
                        data::Target::Quest { .. } => 3,
                        data::Target::TimeTower(_) => 4,
                    });
                }
            }
        }
        let expected_world = data::WorldId {
            room: metadata[0],
            match_id: metadata[1],
            generation: metadata[2],
        };
        let expected_player = data::Player {
            source: i32::try_from(metadata[4]).unwrap(),
            object: metadata[5],
            epoch: metadata[6],
            channel: 0,
        };
        typed
            .check_context(&expected_player, &expected_world)
            .unwrap_or_else(|_| panic!("{label}: exact source/world/visibility binding"));
        typed
            .check_combat_values(&expected_player, &expected_world)
            .unwrap_or_else(|_| {
                panic!("{label}: combat resources, spell events and private history")
            });
        combat_mutations += combat_tests::real_payload_mutations(label, &typed);
        let private = typed
            .personal_view(&expected_player, &expected_world)
            .unwrap_or_else(|_| panic!("{label}: source inventory/lifetimes/weight/ammo/skills"));
        assert_eq!(private.player(), &expected_player);
        assert_eq!(private.world(), &expected_world);
        assert_eq!(private.revision(), typed.world.revision);
        typed
            .check_scene_values(&expected_player, &expected_world)
            .unwrap_or_else(|_| panic!("{label}: canonical scene consistency"));
        if typed.world.phase != data::Phase::Aborted {
            let observation = crate::modern_lito::battle_scene::tests::observation(typed.clone());
            crate::modern_lito::battle_scene::baseline_counts(observation).unwrap();
        }
        personal_tests::real_payload_positive_controls(label, &typed);
        scene_mutations += scene_tests::real_payload_mutations(label, &typed);
        mode_mutations += mode_tests::real_payload_mutations(label, &typed);
        personal_mutations += personal_tests::real_payload_mutations(label, &typed);
        if typed.personal.equipment.is_some() {
            let mut foreign = typed.clone();
            foreign.personal.equipment.as_mut().unwrap().source += 1;
            assert!(foreign
                .check_context(&expected_player, &expected_world)
                .is_err());
            let mut foreign = typed.clone();
            foreign
                .personal
                .history
                .as_mut()
                .unwrap()
                .0
                .identity
                .source
                .epoch ^= 1;
            assert!(foreign
                .check_context(&expected_player, &expected_world)
                .is_err());
        }
        if let Some(n) = &typed.npcs {
            if !n.actors.0.is_empty() {
                let mut foreign = typed.clone();
                foreign.npcs.as_mut().unwrap().actors.0[0].source += 1;
                assert!(foreign
                    .check_context(&expected_player, &expected_world)
                    .is_err());
                if typed.world.lane >= 0 {
                    let mut foreign = typed.clone();
                    foreign.npcs.as_mut().unwrap().actors.0[0].lane ^= 1;
                    assert!(foreign
                        .check_context(&expected_player, &expected_world)
                        .is_err());
                }
            }
        }
        if typed
            .quest
            .as_ref()
            .is_some_and(|q| !q.targets.0.is_empty())
        {
            let mut foreign = typed.clone();
            foreign.quest.as_mut().unwrap().targets.0[0].lane ^= 1;
            assert!(foreign
                .check_context(&expected_player, &expected_world)
                .is_err());
        }
        if typed.race.is_some() {
            let mut foreign = typed.clone();
            foreign.race.as_mut().unwrap().runners.0[0].entity += 1;
            assert!(foreign
                .check_context(&expected_player, &expected_world)
                .is_err());
        }
        if typed.result.is_some() {
            let mut foreign = typed.clone();
            foreign.result.as_mut().unwrap().participants.0[0]
                .identity
                .source += 1;
            assert!(foreign
                .check_context(&expected_player, &expected_world)
                .is_err());
        }
        for field in 0..4 {
            let mut wrong = expected_player.clone();
            match field {
                0 => wrong.source += 1,
                1 => wrong.object += 1,
                2 => wrong.epoch ^= 1,
                _ => wrong.channel = 1,
            };
            assert!(
                typed.check_context(&wrong, &expected_world).is_err(),
                "{label} mismatched selection {field}"
            );
        }
        for section in [
            Section::World,
            Section::Room,
            Section::Manifest,
            Section::Actors,
            Section::Honor,
            Section::Quest,
            Section::TimeAttack,
            Section::Siege,
            Section::Race,
            Section::Npcs,
            Section::Personal,
            Section::Result,
        ] {
            let original = payload.section(section);
            let mut added = original.to_vec();
            added.push(0);
            let added = super::data_tests::replacement(&bytes, section as usize, &added);
            assert!(inspect(&added).is_ok());
            assert!(
                decode(&added).is_err(),
                "{label} inner trailing {section:?}"
            );
            let short = super::data_tests::replacement(
                &bytes,
                section as usize,
                &original[..original.len() - 1],
            );
            assert!(inspect(&short).is_ok());
            assert!(
                decode(&short).is_err(),
                "{label} inner truncation {section:?}"
            );
        }
        let world = payload.section(Section::World);
        let personal = payload.section(Section::Personal);
        assert_eq!(
            (0..4).map(|n| i64_at(world, n * 8)).collect::<Vec<_>>(),
            metadata[..4]
        );
        assert_eq!(
            [
                i32_at(personal, 0) as i64,
                i64_at(personal, 4),
                i64_at(personal, 12),
                i64_at(personal, 24)
            ],
            metadata[4..8]
        );
        assert_eq!(i32_at(personal, 20), 0);
        let actors = payload.section(Section::Actors);
        assert!(i32_at(actors, 0) > 0);
        match label {
            "honor-a" => {
                assert_eq!(world[38], 3);
                honor_public = Some((world.to_vec(), actors.to_vec()));
            }
            "honor-b" => {
                let old = honor_public.as_ref().unwrap();
                assert_eq!(world, old.0);
                assert_eq!(actors, old.1);
                assert_ne!(metadata[4], 7);
            }
            "quest-win" => {
                assert_eq!(i32_at(world, 34), 0);
                assert_eq!(i32_at(actors, 0), 1);
                assert_eq!(i64_at(actors, 4), 1);
                quest_result = Some(payload.section(Section::Result).to_vec());
            }
            "quest-loss" => {
                assert_eq!(i32_at(world, 34), 1);
                assert_eq!(i32_at(actors, 0), 1);
                assert_eq!(i64_at(actors, 4), 2);
                assert_eq!(
                    payload.section(Section::Result),
                    quest_result.as_ref().unwrap()
                );
            }
            "time" => {
                assert_eq!(world[38], 6);
                assert_eq!(payload.section(Section::TimeAttack)[0], 1);
                assert_eq!(payload.section(Section::Result)[0], 1);
            }
            "siege" => {
                assert_eq!(world[38], 7);
                assert_eq!(i32_at(actors, 0), 30);
                assert_eq!(payload.section(Section::Siege)[0], 1);
                assert_eq!(payload.section(Section::Result)[0], 1);
            }
            "race" => {
                assert_eq!(world[38], 4);
                assert_eq!(personal[32], 0);
                assert_eq!(payload.section(Section::Result), [0]);
                assert_eq!(payload.section(Section::Race)[0], 1);
            }
            "honor-skills" => assert_eq!(
                a.spells
                    .histories
                    .0
                    .iter()
                    .filter(|s| s.receipt.is_some())
                    .count(),
                3
            ),
            "disintegrate" => assert!(a
                .spells
                .histories
                .0
                .iter()
                .any(|s| s.lane == data::Lane::Disintegrate && s.receipt.is_some())),
            "npc-skills" => assert_eq!(
                a.spells
                    .histories
                    .0
                    .iter()
                    .filter(|s| s.receipt.is_some())
                    .count(),
                3
            ),
            "quest-structure" => assert!(a
                .spells
                .structure
                .as_ref()
                .is_some_and(|s| s.quest_kind.is_some())),
            "time-structure" => assert!(a.spells.structure.as_ref().is_some_and(|s| s
                .impact
                .targets
                .0
                .iter()
                .any(|e| matches!(e.target, data::Target::TimeTower(_))))),
            "ammo" => assert_eq!(typed.personal.ammo.as_ref().unwrap().shot_count, 1),
            "regen-items" => assert!(typed.personal.supplies.as_ref().unwrap().history.uses > 0),
            "hundred" => {
                assert_eq!(typed.actors.0.len(), 100);
                assert!(bytes.len() > 64 * 1024);
                assert!(typed.actors.0.iter().all(|a| a
                    .spells
                    .histories
                    .0
                    .iter()
                    .any(|s| s.lane == data::Lane::Teleport && s.receipt.is_some())));
            }
            "honor-equipment-eras" => {
                assert_eq!(typed.honor.lives.0[0].respawn_count, 2);
                assert_eq!(typed.honor.lives.0[0].respawn_hp_gain, 290);
                assert_eq!(
                    typed.actors.0[0].vital.as_ref().unwrap().values.maximum_hp,
                    100
                );
            }
            "quest-dead" => assert_eq!(typed.quest.as_ref().unwrap().lives.0[0].deaths, 1),
            "quest-revived" => assert_eq!(typed.quest.as_ref().unwrap().lives.0[0].respawns, 1),
            "quest-aborted" | "time-aborted" => {
                assert_eq!(typed.world.phase, data::Phase::Aborted);
                assert!(typed.npcs.as_ref().unwrap().closed);
                assert!(typed.result.is_none());
            }
            "time-cleared" | "time-timeout" | "time-departed-win" => {
                assert_eq!(typed.world.phase, data::Phase::Finished);
                assert!(typed.result.as_ref().unwrap().time_attack.is_some());
                assert_eq!(
                    typed.time_attack.as_ref().unwrap().end,
                    if label == "time-timeout" {
                        data::TimeEnd::Timeout
                    } else {
                        data::TimeEnd::Cleared
                    }
                );
                if label == "time-departed-win" {
                    assert_eq!(typed.time_attack.as_ref().unwrap().departed.0, vec![1]);
                    assert_eq!(
                        typed.result.as_ref().unwrap().participants.0[0].verdict,
                        data::Verdict::Loss
                    );
                    assert_eq!(
                        typed.result.as_ref().unwrap().participants.0[1].verdict,
                        data::Verdict::Win
                    );
                }
            }
            "npc-retired" => {
                assert!(typed.npcs.as_ref().unwrap().actors.0.is_empty());
                assert!(typed.actors.0[0]
                    .spells
                    .histories
                    .0
                    .iter()
                    .any(|s| s.lane == data::Lane::NpcLightArrow && s.receipt.is_some()));
            }
            "honor-terminal-arrow"
            | "honor-terminal-melee"
            | "elimination-terminal"
            | "siege-inner-win" => {
                assert_eq!(typed.world.phase, data::Phase::Finished);
                assert!(typed.result.is_some());
            }
            "honor-aborted" | "siege-aborted" | "race-aborted" => {
                assert_eq!(typed.world.phase, data::Phase::Aborted);
                assert!(typed.result.is_none());
            }
            "siege-waiting" | "race-waiting" => assert_eq!(typed.world.phase, data::Phase::Loading),
            "siege-outer" | "siege-inner" | "race-running" | "race-first" | "race-two"
            | "race-left" | "race-disconnected" => {
                assert_eq!(typed.world.phase, data::Phase::Active)
            }
            _ => panic!("unexpected peer case"),
        }
        assert!(inspect(&bytes[..bytes.len() - 1]).is_err());
        peer.ack(&rebuilt);
        labels.push(label.to_string());
    }
    assert_eq!(
        labels,
        [
            "honor-a",
            "honor-b",
            "quest-win",
            "quest-loss",
            "time",
            "siege",
            "race",
            "honor-skills",
            "disintegrate",
            "npc-skills",
            "quest-structure",
            "time-structure",
            "ammo",
            "regen-items",
            "hundred",
            "honor-equipment-eras",
            "quest-dead",
            "quest-revived",
            "quest-aborted",
            "time-cleared",
            "time-timeout",
            "time-aborted",
            "time-departed-win",
            "npc-retired",
            "honor-terminal-arrow",
            "honor-terminal-melee",
            "honor-aborted",
            "siege-waiting",
            "siege-outer",
            "siege-inner",
            "siege-inner-win",
            "siege-aborted",
            "race-waiting",
            "race-running",
            "race-first",
            "race-two",
            "race-left",
            "race-disconnected",
            "race-aborted",
            "elimination-terminal"
        ]
    );
    assert_eq!(lanes, (0u8..8).collect());
    assert_eq!(target_kinds, (0..5).collect());
    assert_eq!(detail_kinds, (0..6).collect());
    assert!(combat_mutations >= 300);
    eprintln!("LTB1 combat semantic mutations rejected: {combat_mutations}");
    assert!(personal_mutations >= 200);
    eprintln!("LTB1 private semantic mutations rejected: {personal_mutations}");
    assert!(scene_mutations >= 200);
    eprintln!("LTB1 scene semantic mutations rejected: {scene_mutations}");
    assert!(mode_mutations >= 300);
    eprintln!("LTB1 mode semantic mutations rejected: {mode_mutations}");
    peer.finish();
}
