use super::*;
use crate::modern_lito::{
    native_pet_race_component::{Contract, NAMESPACE},
    native_pet_race_session::OuterContract,
    native_pet_race_terrain::NativeTerrain,
    native_resources::{BytePin, NativeSectorKey, SectorPin},
    resources::SharedMapSectorKey,
    world_map::SectorInput,
};
use serde_json::{json, Value};
use std::{fs, io::Write, path::PathBuf, sync::atomic::AtomicBool};
fn root(var: &str) -> PathBuf {
    let p = PathBuf::from(std::env::var_os(var).unwrap());
    assert!(p.is_absolute());
    p
}
fn hex(s: &str) -> Vec<u8> {
    assert_eq!(s.len() % 2, 0);
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn session() -> (CandidateSession, Cell) {
    let terrain_root = root("LITO_R536_TERRAIN_ROOT");
    let rows: Value = serde_json::from_slice(
        &fs::read(terrain_root.join("s32-layer3-to-actual-reader-byte-comparison.json")).unwrap(),
    )
    .unwrap();
    let mut inputs = Vec::new();
    let mut pins = Vec::new();
    for r in rows["sectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["map"] == 566)
    {
        let k = r["sector"].as_str().unwrap();
        let x = u16::from_str_radix(&k[..4], 16).unwrap();
        let y = u16::from_str_radix(&k[4..], 16).unwrap();
        inputs.push(SectorInput {
            key: SharedMapSectorKey {
                map_id: 566,
                sector_x: x,
                sector_y: y,
            },
            bytes: fs::read(terrain_root.join(format!("native/566/{k}.s32"))).unwrap(),
        });
        pins.push(SectorPin {
            key: NativeSectorKey { map_id: 566, x, y },
            pin: BytePin {
                bytes: r["raw"]["bytes"].as_u64().unwrap(),
                sha256: hex(r["raw"]["sha256"].as_str().unwrap())
                    .try_into()
                    .unwrap(),
            },
        });
    }
    let terrain = Arc::new(
        NativeTerrain::from_sector_bytes(NAMESPACE, inputs, &pins, &AtomicBool::new(false))
            .unwrap(),
    );
    let c = Contract::parse(
        &fs::read(terrain_root.join("adopted-r533-component-contract.json")).unwrap(),
    )
    .unwrap();
    let o = OuterContract::parse(
        &fs::read(root("LITO_R536_OUTER_ROOT").join("outer-lifecycle-contract-v1.json")).unwrap(),
    )
    .unwrap();
    let inputs = root("LITO_R536_INPUT_ROOT");
    let q = QuitContract::parse(
        &fs::read(inputs.join("lifecycle_contract.json")).unwrap(),
        &fs::read(inputs.join("version_binding.json")).unwrap(),
    )
    .unwrap();
    let spawn = c.course(NAMESPACE, 566).unwrap().spawns[0];
    let base =
        Session::begin_offline(&o, &c, terrain, &[(7, spawn)], 7, 2, b"A", 0, i64::MAX).unwrap();
    (
        CandidateSession::from_offline_predecessor(&q, base).unwrap(),
        spawn,
    )
}
fn seed(s: &mut CandidateSession) -> ReceiptScope {
    let scope = s.receipt_scope();
    // Synthetic source-shaped seed, not a server-produced/admitted roster.
    for b in [
        vec![0xFD, 1, 1, 0, 0, 0, 1, b'A', 0],
        vec![0xFD, 2, 4, 0, 1, 0],
        vec![0xFD, 0],
    ] {
        s.observe_offline(&scope, &b, Delivery::HypotheticalFixedReplay)
            .unwrap();
    }
    assert_eq!(s.snapshot().view, View::Prediction);
    scope
}
fn replay() -> Value {
    serde_json::from_slice(
        &fs::read(root("LITO_R536_INPUT_ROOT").join("actual-application-replay.json")).unwrap(),
    )
    .unwrap()
}
fn bodies(v: &Value) -> Vec<Vec<u8>> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|r| {
            assert_eq!(r["server_complete_flush"], true);
            assert_eq!(r["client_delivery_proven"], false);
            hex(r["body_hex"].as_str().unwrap())
        })
        .collect()
}
fn save(name: &str, v: Value) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root("LITO_R536_OUTPUT_ROOT").join(name))
        .unwrap();
    writeln!(f, "{}", serde_json::to_string_pretty(&v).unwrap()).unwrap();
}
#[test]
fn r536_native_quit_actual_generated_normal_io_peer_and_duplicate_sequences() {
    let r = replay();
    assert_eq!(r["client_delivery_ACK"], false);
    let mut checks = Vec::new();
    let mut delivered_bodies = 0;
    let mut channel = 0;
    let mut result = 0;
    let mut unknown = 0;
    for case in r["cases"].as_array().unwrap() {
        for stream in case["streams"].as_array().unwrap() {
            let (mut s, spawn) = session();
            let scope = seed(&mut s);
            let old = s.command().unwrap();
            let role = stream["role"].as_str().unwrap();
            if role == "quitter" {
                let token = s.prepare_active_quit(&scope).unwrap();
                assert_eq!(s.note_offline_active_quit_written(&token).unwrap(), [0xD9]);
            }
            let first = bodies(&stream["first_bodies"]);
            for b in &first {
                s.observe_offline(&scope, b, Delivery::HypotheticalFixedReplay)
                    .unwrap();
                delivered_bodies += 1;
            }
            if role == "quitter" {
                let complete =
                    first.iter().any(|b| b[0] == 0xF0) && first.iter().any(|b| b[0] == 0xCC);
                if complete {
                    assert_eq!(s.snapshot().view, View::Channel);
                } else {
                    assert_ne!(s.snapshot().view, View::Channel);
                }
                assert!(
                    !s.snapshot().self_exit_notified,
                    "active D9 output has no D0"
                );
            } else if first.iter().any(|b| b[0] == 0xD3) {
                assert_eq!(s.snapshot().view, View::Result);
                assert_eq!(s.result_ack(&scope).unwrap(), [0xD9]);
            } else {
                assert!(
                    s.result_ack(&scope).is_err(),
                    "server RESULT metadata cannot supply missing client D3"
                );
            }
            for b in bodies(&stream["duplicate_bodies"]) {
                s.observe_offline(&scope, &b, Delivery::HypotheticalFixedReplay)
                    .unwrap();
                delivered_bodies += 1;
                if role != "quitter" {
                    assert_ne!(
                        s.snapshot().view,
                        View::Channel,
                        "RESULT D9 F0 retains room"
                    );
                }
            }
            s.finish_offline_receipt_window(&scope).unwrap();
            assert!(s.command().is_none());
            assert!(s
                .predict(
                    &old,
                    Cell {
                        x: spawn.x + 1,
                        y: spawn.y
                    },
                    1
                )
                .is_err());
            match s.snapshot().view {
                View::Channel => channel += 1,
                View::Result => result += 1,
                View::ReturnUnknown => unknown += 1,
                v => panic!("unexpected {v:?}"),
            }
            assert!(!s.snapshot().wire_generation_known);
            checks.push(json!({"scenario":case["scenario"],"role":role,"client_snapshot_under_hypothetical_delivery":s.snapshot(),
                "server_poststate_separate":stream["server_poststate"],"server_flush_is_client_ACK":false}));
        }
    }
    assert_eq!(
        (checks.len(), delivered_bodies, channel, result, unknown),
        (18, 39, 3, 9, 6)
    );
    save(
        "r536-actual-application-replay-check.json",
        json!({"state":"R536_NEW_CANDIDATE_OFFLINE_REPLAY_PASS",
        "streams":18,"actual_server_successful_bodies_assumed_delivered":39,"local_channel_views":channel,"local_result_views":result,
        "unknown_views":unknown,"checks":checks,"old_server_methods_or_R533_535_PASS_replayed":false,"SQL_TLS_GUI_provider":0,"new_full_bindings":0}),
    );
}
#[test]
fn r536_native_quit_intent_version_missing_reply_and_late_receipt_counterexamples() {
    let r = replay();
    let normal = &r["cases"][0];
    assert_eq!(normal["scenario"], "normal");
    let quitter = bodies(&normal["streams"][0]["first_bodies"]);
    let f0 = &quitter[0];
    let cc = &quitter[1];
    assert_eq!((f0[0], cc[0]), (0xF0, 0xCC));
    let peer = bodies(&normal["streams"][1]["first_bodies"]);
    let d3 = peer.iter().find(|b| b[0] == 0xD3).unwrap();
    let (mut idle, _) = session();
    let scope = seed(&mut idle);
    for b in &quitter {
        idle.observe_offline(&scope, b, Delivery::HypotheticalFixedReplay)
            .unwrap();
    }
    assert_eq!(
        idle.snapshot().view,
        View::Prediction,
        "F0/CC without an active intent cannot leave"
    );
    let command = idle.command().unwrap();
    let token = idle.prepare_active_quit(&scope).unwrap();
    for b in &quitter {
        idle.observe_offline(&scope, b, Delivery::HypotheticalFixedReplay)
            .unwrap();
    }
    assert_eq!(
        idle.snapshot().view,
        View::ReturnUnknown,
        "uncommitted intent has no current reply"
    );
    assert_eq!(
        idle.note_offline_active_quit_written(&token).unwrap(),
        [0xD9]
    );
    assert!(idle.note_offline_active_quit_written(&token).is_err());
    idle.observe_offline(&scope, cc, Delivery::HypotheticalFixedReplay)
        .unwrap();
    assert_eq!(
        idle.snapshot().view,
        View::ReturnUnknown,
        "CC before current F0 cannot complete"
    );
    assert!(!idle.snapshot().current_reply_assumed);
    idle.observe(&scope, f0).unwrap();
    assert!(idle.snapshot().ambiguous_receipts);
    for b in &quitter {
        idle.observe_offline(&scope, b, Delivery::HypotheticalFixedReplay)
            .unwrap();
    }
    assert_eq!(
        idle.snapshot().view,
        View::ReturnUnknown,
        "unmarked late receipt ambiguity cannot be repaired by more bytes"
    );
    let (mut missing, _) = session();
    let missing_scope = seed(&mut missing);
    let pending = missing.prepare_active_quit(&missing_scope).unwrap();
    missing.note_offline_active_quit_written(&pending).unwrap();
    missing
        .observe_offline(&missing_scope, f0, Delivery::HypotheticalFixedReplay)
        .unwrap();
    assert_eq!(missing.snapshot().view, View::Returning);
    missing
        .finish_offline_receipt_window(&missing_scope)
        .unwrap();
    assert_eq!(
        missing.snapshot().view,
        View::ReturnUnknown,
        "missing/empty room list has no completion marker"
    );
    let (mut race, _) = session();
    let race_scope = seed(&mut race);
    let superseded = race.prepare_active_quit(&race_scope).unwrap();
    race.observe_offline(&race_scope, d3, Delivery::HypotheticalFixedReplay)
        .unwrap();
    assert!(race.note_offline_active_quit_written(&superseded).is_err());
    assert_eq!(race.result_ack(&race_scope).unwrap(), [0xD9]);
    for b in &quitter {
        race.observe_offline(&race_scope, b, Delivery::HypotheticalFixedReplay)
            .unwrap();
    }
    assert_eq!(
        race.snapshot().view,
        View::Result,
        "same actual F0/CC bytes in RESULT do not leave"
    );
    assert!(race.prepare_active_quit(&race_scope).is_err());
    let confirm = race.prepare_result_confirm(&race_scope).unwrap();
    assert_eq!(race.note_offline_confirm_written(&confirm).unwrap(), [0xD0]);
    assert!(race.note_offline_confirm_written(&confirm).is_err());
    race.observe_offline(&race_scope, f0, Delivery::HypotheticalFixedReplay)
        .unwrap();
    race.observe_offline(&race_scope, cc, Delivery::HypotheticalFixedReplay)
        .unwrap();
    assert_eq!(
        race.snapshot().view,
        View::ReturnUnknown,
        "RESULT confirm requires own D0, missing candidate output is not fabricated"
    );
    let before = race.snapshot();
    let mut invalid = peer[0].clone();
    invalid.push(1);
    assert!(race.observe(&race_scope, &invalid).is_err());
    assert_eq!(race.snapshot(), before);
    let mut rejections = 1;
    for b in &peer {
        for n in 0..b.len() {
            let before = race.snapshot();
            assert!(race.observe(&race_scope, &b[..n]).is_err());
            assert_eq!(race.snapshot(), before);
            rejections += 1;
        }
    }
    idle.disconnect(&scope).unwrap();
    assert!(idle.observe(&scope, f0).is_err());
    assert_eq!(idle.snapshot().view, View::Disconnected);
    let (mut next, spawn) = session();
    let next_scope = seed(&mut next);
    assert!(next.observe(&scope, f0).is_err());
    assert!(next.note_offline_active_quit_written(&token).is_err());
    assert!(next
        .predict(
            &command,
            Cell {
                x: spawn.x + 1,
                y: spawn.y
            },
            1
        )
        .is_err());
    next.observe(&next_scope, f0).unwrap();
    next.observe(&next_scope, cc).unwrap();
    assert_eq!(
        next.snapshot().view,
        View::Prediction,
        "old F0 under no current intent has no exit authority"
    );
    let input = root("LITO_R536_INPUT_ROOT");
    let order = fs::read(input.join("lifecycle_contract.json")).unwrap();
    let mut versions = fs::read(input.join("version_binding.json")).unwrap();
    versions.push(b' ');
    assert!(QuitContract::parse(&order, &versions).is_err());
    save(
        "r536-quit-intent-counterexamples-check.json",
        json!({"state":"R536_NEW_QUIT_PROFILE_INTENT_AND_UNKNOWN_COUNTEREXAMPLES_PASS",
        "malformed_atomic_rejections":rejections,"no_active_D0_offered":true,"arbitrary_F0_CC_cannot_exit":true,
        "untagged_late_receipts_remain_unknown":true,"missing_RESULT_D0_not_fabricated":true,
        "RESULT_confirm_positive_actual_candidate_replay_available":false,"old_R535_PASS_replayed":false,
        "SQL_TLS_GUI_provider":0,"new_full_bindings":0}),
    );
}
