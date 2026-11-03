use super::*;
use crate::modern_lito::{
    native_pet_race_component::{Cell, Contract, NAMESPACE},
    native_pet_race_quit_session::{Delivery, View, RETURN_SOURCE_SHA},
    native_pet_race_session::{OuterContract, ReceiptScope},
    native_pet_race_terrain::NativeTerrain,
    native_resources::{BytePin, NativeSectorKey, SectorPin},
    resources::SharedMapSectorKey,
    world_map::SectorInput,
};
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};
fn root(var: &str) -> PathBuf {
    let p = PathBuf::from(std::env::var_os(var).unwrap());
    assert!(p.is_absolute());
    p
}
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn replay() -> Value {
    serde_json::from_slice(
        &fs::read(root("LITO_R537_INPUT_ROOT").join("actual-return-replay.json")).unwrap(),
    )
    .unwrap()
}
fn session(seq: &Value) -> (CandidateSession, Cell) {
    let tr = root("LITO_R537_TERRAIN_ROOT");
    let rows: Value = serde_json::from_slice(
        &fs::read(tr.join("s32-layer3-to-actual-reader-byte-comparison.json")).unwrap(),
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
            bytes: fs::read(tr.join(format!("native/566/{k}.s32"))).unwrap(),
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
    let c = Contract::parse(&fs::read(tr.join("adopted-r533-component-contract.json")).unwrap())
        .unwrap();
    let o = OuterContract::parse(
        &fs::read(root("LITO_R537_OUTER_ROOT").join("outer-lifecycle-contract-v1.json")).unwrap(),
    )
    .unwrap();
    let rc = ReturnContract::parse(
        &fs::read(root("LITO_R537_INPUT_ROOT").join("version_lifecycle_contract.json")).unwrap(),
    )
    .unwrap();
    let spawn = c.course(NAMESPACE, 566).unwrap().spawns[0];
    let alias = hex(seq["self_alias_hex"].as_str().unwrap());
    let base = Session::begin_offline(
        &o,
        &c,
        terrain,
        &[(7, spawn)],
        7,
        seq["origin_room_id"].as_u64().unwrap() as u32,
        &alias,
        0,
        i64::MAX,
    )
    .unwrap();
    (rc.compose_offline(base).unwrap(), spawn)
}
fn seed(s: &mut CandidateSession) -> ReceiptScope {
    let scope = s.receipt_scope();
    // Explicit synthetic offline seed; the actual return fixture is not WORLD admission.
    for b in [
        vec![0xFD, 1, 1, 0, 0, 0, 1, b'A', 0],
        vec![0xFD, 2, 4, 0, 1, 0],
        vec![0xFD, 0],
    ] {
        s.observe_offline(&scope, &b, Delivery::HypotheticalFixedReplay)
            .unwrap();
    }
    assert_eq!(s.snapshot().candidate_source, RETURN_SOURCE_SHA);
    assert_eq!(s.snapshot().view, View::Prediction);
    scope
}
fn body(f: &Value) -> Vec<u8> {
    let b = hex(f["body_hex"].as_str().unwrap());
    assert_eq!(f["client_delivery_proven"], false);
    assert_eq!(
        b,
        fs::read(f["body_file"]["path"].as_str().unwrap()).unwrap()
    );
    b
}
fn result_seed(s: &mut CandidateSession, seq: &Value) -> ReceiptScope {
    let scope = seed(s);
    for f in seq["frames"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["stage"] == "prepare_actual_active_D9")
    {
        s.observe_offline(&scope, &body(f), Delivery::HypotheticalFixedReplay)
            .unwrap();
    }
    assert_eq!(s.snapshot().view, View::Result);
    scope
}
fn save(name: &str, v: Value) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root("LITO_R537_OUTPUT_ROOT").join(name))
        .unwrap();
    writeln!(f, "{}", serde_json::to_string_pretty(&v).unwrap()).unwrap();
}
#[test]
fn r537_native_return_actual_per_receiver_result_self_peer_owner_delete_sequences() {
    let r = replay();
    assert_eq!(r["per_receiver_ordinal_only"], true);
    assert_eq!(r["cross_socket_total_order"], false);
    let mut checks = Vec::new();
    let (mut setup, mut returns, mut acknowledgements, mut confirms, mut deletes, mut owners) =
        (0, 0, 0, 0, 0, 0);
    for seq in r["sequences"].as_array().unwrap() {
        let (mut s, spawn) = session(seq);
        let scope = seed(&mut s);
        let old = s.command().unwrap();
        if seq["role"] == "quitter" {
            let q = s.prepare_active_quit(&scope).unwrap();
            assert_eq!(s.note_offline_active_quit_written(&q).unwrap(), [0xD9]);
        }
        for f in seq["frames"].as_array().unwrap() {
            let b = body(f);
            let stage = f["stage"].as_str().unwrap();
            if stage.ends_with("RESULT_D9") {
                assert_eq!(s.result_ack(&scope).unwrap(), [0xD9]);
                acknowledgements += 1;
            }
            if !f["own_exit_name_hex"].is_null() {
                assert_eq!(s.snapshot().view, View::Result);
                let c = s.prepare_result_confirm(&scope).unwrap();
                assert_eq!(s.note_offline_confirm_written(&c).unwrap(), [0xD0]);
                confirms += 1;
            }
            let before = s.snapshot();
            s.observe_offline(&scope, &b, Delivery::HypotheticalFixedReplay)
                .unwrap();
            if f["RESULT_return_output"] == true {
                returns += 1;
            } else {
                setup += 1;
            }
            match b[0] {
                0xF0 if seq["role"] != "quitter" => assert_eq!(
                    s.snapshot().view,
                    View::Result,
                    "RESULT acknowledgement retains membership view"
                ),
                0xD0 if f["own_exit_name_hex"].is_null() => {
                    assert_eq!(s.snapshot().view, before.view);
                    assert!(!s.snapshot().self_exit_notified);
                }
                0xD5 => {
                    owners += 1;
                    assert_eq!(s.snapshot().view, before.view);
                    assert!(!s.snapshot().current_reply_assumed);
                }
                0xCF => {
                    deletes += 1;
                    assert_eq!(
                        s.snapshot().view,
                        before.view,
                        "deletion alone cannot finish a pending return"
                    );
                }
                _ => {}
            }
        }
        assert_eq!(s.snapshot().view, View::Channel);
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
        assert!(
            !s.snapshot().actual_room_users_known
                && !s.snapshot().complete_catalog_known
                && !s.snapshot().wire_generation_known
        );
        if seq["scenario"] == "default_room_last_member_reset" {
            assert!(s
                .snapshot()
                .room_rows
                .iter()
                .any(|row| row.room_id == 2 && row.displayed_users_wire == 1 && row.capacity == 8));
        } else {
            assert!(s.snapshot().deleted_room_ids.contains(&3));
            assert!(!s.snapshot().room_rows.iter().any(|row| row.room_id == 3));
            if seq["role"] != "quitter" {
                assert!(s.snapshot().room_rows.iter().any(|row| row.room_id == 2));
            }
        }
        checks.push(json!({"scenario":seq["scenario"],"role":seq["role"],"snapshot_under_fixed_delivery_assumption":s.snapshot(),"cross_socket_total_order_claim":false}));
    }
    assert_eq!(
        (
            checks.len(),
            setup,
            returns,
            acknowledgements,
            confirms,
            deletes,
            owners
        ),
        (6, 17, 22, 4, 4, 3, 2)
    );
    save(
        "r537-result-return-replay-check.json",
        json!({"state":"R537_NEW_VERSION_ACTUAL_BODY_OFFLINE_RETURN_PASS","sequences":6,
        "actual_setup_bodies":setup,"actual_return_bodies":returns,"RESULT_D9_ack_intents":acknowledgements,"RESULT_D0_confirm_intents":confirms,
        "CF_deletions":deletes,"D5_owner_updates":owners,"checks":checks,"client_receipt_proven":false,"actual_users_inferred_from_floor1":false,
        "old_PASS_or_Java_calls_replayed":false,"SQL_TLS_GUI_provider":0,"new_full_bindings":0}),
    );
}
#[test]
fn r537_native_return_partial_missing_untagged_epoch_and_atomic_counterexamples() {
    let r = replay();
    let seq = &r["sequences"][5];
    assert_eq!(seq["role"], "healthy-peer");
    let frames = seq["frames"].as_array().unwrap();
    let own = body(
        frames
            .iter()
            .find(|f| !f["own_exit_name_hex"].is_null())
            .unwrap(),
    );
    let peer = body(
        frames
            .iter()
            .find(|f| {
                f["body_hex"].as_str().unwrap().starts_with("D0")
                    && f["own_exit_name_hex"].is_null()
            })
            .unwrap(),
    );
    let owner = body(
        frames
            .iter()
            .find(|f| f["body_hex"].as_str().unwrap().starts_with("D5"))
            .unwrap(),
    );
    let deleted = body(
        frames
            .iter()
            .find(|f| f["body_hex"].as_str().unwrap().starts_with("CF"))
            .unwrap(),
    );
    let cc = body(
        frames
            .iter()
            .find(|f| f["body_hex"].as_str().unwrap().starts_with("CC"))
            .unwrap(),
    );
    let (mut missing, _) = session(seq);
    let scope = result_seed(&mut missing, seq);
    let confirm = missing.prepare_result_confirm(&scope).unwrap();
    missing.note_offline_confirm_written(&confirm).unwrap();
    for b in [&peer, &owner, &deleted, &cc] {
        missing
            .observe_offline(&scope, b, Delivery::HypotheticalFixedReplay)
            .unwrap();
    }
    assert_eq!(
        missing.snapshot().view,
        View::ReturnUnknown,
        "peer exit/owner/delete/catalog cannot substitute own D0"
    );
    assert!(!missing.snapshot().self_exit_notified);
    missing.finish_offline_receipt_window(&scope).unwrap();
    let (mut no_list, _) = session(seq);
    let no_list_scope = result_seed(&mut no_list, seq);
    let c = no_list.prepare_result_confirm(&no_list_scope).unwrap();
    no_list.note_offline_confirm_written(&c).unwrap();
    no_list
        .observe_offline(&no_list_scope, &own, Delivery::HypotheticalFixedReplay)
        .unwrap();
    no_list
        .observe_offline(&no_list_scope, &deleted, Delivery::HypotheticalFixedReplay)
        .unwrap();
    assert_eq!(no_list.snapshot().view, View::Returning);
    no_list
        .finish_offline_receipt_window(&no_list_scope)
        .unwrap();
    assert_eq!(
        no_list.snapshot().view,
        View::ReturnUnknown,
        "no zero-total-catalog marker fabricated"
    );
    let (mut late, _) = session(seq);
    let late_scope = result_seed(&mut late, seq);
    let c = late.prepare_result_confirm(&late_scope).unwrap();
    late.note_offline_confirm_written(&c).unwrap();
    late.observe(&late_scope, &own).unwrap();
    late.observe_offline(&late_scope, &cc, Delivery::HypotheticalFixedReplay)
        .unwrap();
    assert_eq!(late.snapshot().view, View::ReturnUnknown);
    assert!(late.snapshot().ambiguous_receipts);
    let mut malformed = 0;
    for b in [&owner, &deleted] {
        for n in 0..b.len() {
            let before = late.snapshot();
            assert!(late.observe(&late_scope, &b[..n]).is_err());
            assert_eq!(late.snapshot(), before);
            malformed += 1;
        }
    }
    for b in [vec![0xCF, 0, 0, 0, 0], vec![0xD5, 6, 3, 0, 0, 0, b'A', 0]] {
        let before = late.snapshot();
        assert!(late.observe(&late_scope, &b).is_err());
        assert_eq!(late.snapshot(), before);
        malformed += 1;
    }
    let mut partial_checks = Vec::new();
    for part in r["partial_failures"].as_array().unwrap() {
        let context = if part["failed_role"] == 0 {
            &r["sequences"][0]
        } else {
            &r["sequences"][1]
        };
        let (mut s, spawn) = session(context);
        let sc = seed(&mut s);
        let old = s.command().unwrap();
        if part["failed_role"] == 0 {
            let q = s.prepare_active_quit(&sc).unwrap();
            s.note_offline_active_quit_written(&q).unwrap();
        }
        let prefix = fs::read(part["raw_prefix"]["path"].as_str().unwrap()).unwrap();
        let declared = part["declared_wire_length"].as_u64().unwrap() as u16;
        let before = s.snapshot();
        assert!(s.observe(&sc, &prefix).is_err());
        assert_eq!(s.snapshot(), before);
        assert!(s
            .note_offline_partial_failure(&sc, &prefix, prefix.len() as u16)
            .is_err());
        assert_eq!(s.snapshot(), before);
        s.note_offline_partial_failure(&sc, &prefix, declared)
            .unwrap();
        assert_eq!(s.snapshot().view, View::Disconnected);
        assert!(s.command().is_none());
        let stored = s.snapshot().partial_transport.unwrap();
        assert_eq!(stored.raw_encrypted_prefix, prefix);
        assert!(!stored.complete_application_body && !stored.client_receipt_proven);
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
        assert!(s.observe(&sc, &cc).is_err());
        partial_checks.push(json!({"source_case":part,"local_fail_close_snapshot_under_failure_assumption":s.snapshot()}));
    }
    late.disconnect(&late_scope).unwrap();
    let (mut fresh, _) = session(seq);
    let _fresh_scope = seed(&mut fresh);
    assert!(fresh.observe(&late_scope, &own).is_err());
    assert!(fresh.note_offline_confirm_written(&c).is_err());
    let mut contract =
        fs::read(root("LITO_R537_INPUT_ROOT").join("version_lifecycle_contract.json")).unwrap();
    contract.push(b' ');
    assert!(ReturnContract::parse(&contract).is_err());
    assert!(ReturnContract::parse(
        &fs::read(root("LITO_R537_PREVIOUS_ROOT").join("lifecycle_contract.json")).unwrap()
    )
    .is_err());
    save(
        "r537-return-partial-counterexamples-check.json",
        json!({"state":"R537_NEW_RETURN_PARTIAL_AND_UNKNOWN_COUNTEREXAMPLES_PASS",
        "partial_failures":partial_checks,"malformed_atomic_rejections":malformed,"partial_prefix_decoded_or_padded":false,
        "missing_own_D0_or_CC_remains_unknown":true,"untagged_return_unknown":true,"deleted_target_zero_rows_means_zero_catalog":false,
        "client_receipt_proven":false,"old_PASS_replayed":false,"SQL_TLS_GUI_provider":0,"new_full_bindings":0}),
    );
}
