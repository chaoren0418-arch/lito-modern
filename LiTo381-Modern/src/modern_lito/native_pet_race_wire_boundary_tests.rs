use super::*;
use crate::modern_lito::{
    native_pet_race_component::{Contract, NAMESPACE},
    native_pet_race_return_profile::ReturnContract,
    native_pet_race_session::{OuterContract, Session},
    native_pet_race_terrain::NativeTerrain,
    native_resources::{BytePin, NativeSectorKey, SectorPin},
    resources::SharedMapSectorKey,
    world_map::SectorInput,
};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Cursor, Write},
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc, OnceLock},
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
fn binding() -> (WireBindings, Value) {
    let b = fs::read(root("LITO_R538_INPUT_ROOT").join("wire-session-bindings.json")).unwrap();
    (
        WireBindings::parse(&b).unwrap(),
        serde_json::from_slice(&b).unwrap(),
    )
}
fn model() -> (CandidateSession, Cell) {
    static TERRAIN: OnceLock<Arc<NativeTerrain>> = OnceLock::new();
    let tr = root("LITO_R538_TERRAIN_ROOT");
    let terrain = TERRAIN
        .get_or_init(|| {
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
            Arc::new(
                NativeTerrain::from_sector_bytes(NAMESPACE, inputs, &pins, &AtomicBool::new(false))
                    .unwrap(),
            )
        })
        .clone();
    let c = Contract::parse(&fs::read(tr.join("adopted-r533-component-contract.json")).unwrap())
        .unwrap();
    let o = OuterContract::parse(
        &fs::read(root("LITO_R538_OUTER_ROOT").join("outer-lifecycle-contract-v1.json")).unwrap(),
    )
    .unwrap();
    let rc = ReturnContract::parse(
        &fs::read(root("LITO_R538_RETURN_ROOT").join("version_lifecycle_contract.json")).unwrap(),
    )
    .unwrap();
    let spawn = c.course(NAMESPACE, 566).unwrap().spawns[0];
    let base =
        Session::begin_offline(&o, &c, terrain, &[(7, spawn)], 7, 2, b"A", 0, i64::MAX).unwrap();
    let mut session = rc.compose_offline(base).unwrap();
    let scope = session.receipt_scope();
    // Synthetic source-shaped game state before this recorded wire lane; not WORLD admission.
    for b in [
        vec![0xFD, 1, 1, 0, 0, 0, 1, b'A', 0],
        vec![0xFD, 2, 4, 0, 1, 0],
        vec![0xFD, 0],
    ] {
        session
            .observe_offline(&scope, &b, Delivery::HypotheticalFixedReplay)
            .unwrap();
    }
    (session, spawn)
}
fn wire(row: &Value) -> Vec<u8> {
    fs::read(row["wire"]["path"].as_str().unwrap()).unwrap()
}
fn active(s: &mut OfflineWireSession) {
    let q = s.prepare_active_quit().unwrap();
    assert_eq!(s.note_offline_active_quit_written(&q).unwrap(), [0xD9]);
}
fn save(name: &str, v: Value) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root("LITO_R538_OUTPUT_ROOT").join(name))
        .unwrap();
    writeln!(f, "{}", serde_json::to_string_pretty(&v).unwrap()).unwrap();
}
#[test]
fn r538_native_wire_existing_receiver_complete_split_merge_and_untagged_read_boundary() {
    let (bindings, doc) = binding();
    let row = doc["lanes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["partial_eof"] == false)
        .unwrap();
    let raw = wire(row);
    assert_eq!(raw.len(), 39);
    let id = row["id"].as_str().unwrap();
    let expected: Vec<Vec<u8>> = row["server_complete_application_bodies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| hex(v.as_str().unwrap()))
        .collect();
    assert_eq!(expected.len(), 2);
    let mut patterns: Vec<Vec<usize>> = [1, 2, 6, 7, 8, 9, 10, 11, 12, 38]
        .into_iter()
        .map(|cut| vec![cut, raw.len() - cut])
        .collect();
    patterns.push(vec![raw.len()]);
    patterns.push(vec![1; raw.len()]);
    let mut checks = Vec::new();
    for chunks in patterns {
        let (m, _) = model();
        let mut s = OfflineWireSession::fixed_trace(bindings.lane(id, &raw).unwrap(), m).unwrap();
        active(&mut s);
        let mut cursor = 0;
        let mut bodies = Vec::new();
        for n in &chunks {
            let batch = s.feed(&raw[cursor..cursor + n]).unwrap();
            assert_eq!(batch.consumed, *n);
            assert!(!batch.terminal);
            cursor += n;
            bodies.extend(batch.application_bodies);
            if cursor < 11 {
                assert_eq!(s.snapshot().application_bodies_dispatched, 0);
                assert!(!s.snapshot().session.quit_reply_observed);
            } else if cursor < 39 {
                assert_eq!(s.snapshot().application_bodies_dispatched, 1);
                assert_eq!(s.snapshot().session.view, View::Returning);
            }
        }
        assert_eq!(bodies, expected);
        assert_eq!(s.snapshot().complete_frames, 3);
        assert_eq!(s.snapshot().session.view, View::Channel);
        let seeded = u64::from_le_bytes([raw[3], raw[4], raw[5], raw[6], 0xA1, 0x6C, 0x54, 0x87]);
        assert_eq!(s.snapshot().receiver.unwrap().counter, Some(seeded + 28));
        checks.push(json!({"chunks":chunks,"snapshot_before_EOF":s.snapshot()}));
        s.finish_eof().unwrap();
        assert_eq!(s.snapshot().terminal.unwrap().reason, "CleanEof");
        assert_eq!(s.snapshot().session.view, View::Disconnected);
        assert!(s.feed(&raw).is_err());
    }
    let (m, _) = model();
    let mut untagged =
        OfflineWireSession::untagged_boundary(bindings.lane(id, &raw).unwrap(), m).unwrap();
    active(&mut untagged);
    let mut reader = Cursor::new(raw.clone());
    let mut received = 0;
    while received < raw.len() {
        let step = untagged.read_untagged_from(&mut reader, 5).unwrap();
        assert!(!step.eof);
        received += step.socket_or_reader_bytes;
    }
    assert_eq!(untagged.snapshot().application_bodies_dispatched, 2);
    assert_eq!(untagged.snapshot().session.view, View::ReturnUnknown);
    assert!(untagged.snapshot().session.ambiguous_receipts);
    let untagged_before_eof = untagged.snapshot();
    assert!(untagged.read_untagged_from(&mut reader, 5).unwrap().eof);
    assert_eq!(untagged.snapshot().session.view, View::Disconnected);
    let (m, _) = model();
    let mut fixed = OfflineWireSession::fixed_trace(bindings.lane(id, &raw).unwrap(), m).unwrap();
    let mut no_reads = Cursor::new(raw);
    assert!(fixed.read_untagged_from(&mut no_reads, 5).is_err());
    assert_eq!(
        no_reads.position(),
        0,
        "hypothetical policy cannot enter generic Read path"
    );
    save(
        "r538-incremental-wire-complete-check.json",
        json!({"state":"R538_NEW_KEYED_WIRE_RECEIVER_COMPOSITION_PASS","split_merge_patterns":checks.len(),"checks":checks,
        "actual_keyed_application_bodies":2,"untagged_before_EOF":untagged_before_eof,"existing_cipher_decoder_reused":true,"hypothetical_policy_allowed_on_Read":false,
        "actual_socket_created":false,"server_memory_trace_is_TCP_loopback":false,"old_PASS_replayed":false,"SQL_TLS_GUI_provider":0,"new_full_bindings":0}),
    );
}
#[test]
fn r538_native_wire_actual_partial_eof_missing_key_foreign_stream_and_retirement() {
    let (bindings, doc) = binding();
    let mut checks = Vec::new();
    let mut header_checks = Vec::new();
    for row in doc["lanes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["partial_eof"] == true)
    {
        let raw = wire(row);
        let id = row["id"].as_str().unwrap();
        let (m, spawn) = model();
        let mut s = OfflineWireSession::fixed_trace(bindings.lane(id, &raw).unwrap(), m).unwrap();
        let old = s.command().unwrap();
        if id.contains("receiver0") {
            active(&mut s);
        }
        s.feed(&raw[..7]).unwrap();
        let initial = s.snapshot().receiver.unwrap().counter;
        for b in &raw[7..] {
            let batch = s.feed(std::slice::from_ref(b)).unwrap();
            assert!(batch.application_bodies.is_empty());
        }
        assert_eq!(s.snapshot().application_bodies_dispatched, 0);
        assert_eq!(s.snapshot().receiver.unwrap().counter, initial);
        assert!(!s.snapshot().session.quit_reply_observed);
        s.finish_eof().unwrap();
        let snapshot = s.snapshot();
        assert_eq!(snapshot.terminal.as_ref().unwrap().reason, "BodyEof");
        assert_eq!(snapshot.session.view, View::Disconnected);
        assert_eq!(snapshot.terminal.as_ref().unwrap().before.counter, initial);
        assert_eq!(
            snapshot
                .terminal
                .as_ref()
                .unwrap()
                .raw_pending_or_rejected_frame,
            raw[7..]
        );
        assert_eq!(
            snapshot
                .session
                .partial_transport
                .as_ref()
                .unwrap()
                .raw_encrypted_prefix,
            raw[7..]
        );
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
        assert!(s.feed(&raw).is_err());
        checks.push(json!({"id":id,"snapshot":snapshot}));
        let (m, _) = model();
        let mut header =
            OfflineWireSession::fixed_trace(bindings.lane(id, &raw).unwrap(), m).unwrap();
        header.feed(&raw[..8]).unwrap();
        header.finish_eof().unwrap();
        let snapshot = header.snapshot();
        assert_eq!(snapshot.terminal.as_ref().unwrap().reason, "HeaderEof");
        assert_eq!(
            snapshot
                .terminal
                .as_ref()
                .unwrap()
                .raw_pending_or_rejected_frame,
            raw[7..8]
        );
        assert_eq!(snapshot.application_bodies_dispatched, 0);
        header_checks.push(
            json!({"id":id,"hypothetical_shortened_delivery_prefix":true,"snapshot":snapshot}),
        );
    }
    assert_eq!(checks.len(), 2);
    let good = doc["lanes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["partial_eof"] == false)
        .unwrap();
    let raw = wire(good);
    let id = good["id"].as_str().unwrap();
    assert!(bindings.lane(id, &raw[7..]).is_err());
    assert!(bindings
        .lane("RESULT-return-receiver-with-no-S_KEY", &raw)
        .is_err());
    let bad = wire(&doc["lanes"][0]);
    let (m, _) = model();
    let mut foreign = OfflineWireSession::fixed_trace(bindings.lane(id, &raw).unwrap(), m).unwrap();
    assert!(foreign.feed(&bad[..7]).unwrap().terminal);
    assert_eq!(
        foreign.snapshot().terminal.unwrap().reason,
        "BindingMismatch"
    );
    assert_eq!(foreign.snapshot().application_bodies_dispatched, 0);
    let (m, _) = model();
    let mut tail = OfflineWireSession::fixed_trace(bindings.lane(id, &raw).unwrap(), m).unwrap();
    tail.feed(&raw[..7]).unwrap();
    assert!(tail.feed(&bad[7..]).unwrap().terminal);
    assert_eq!(tail.snapshot().application_bodies_dispatched, 0);
    let (m, _) = model();
    let mut reordered =
        OfflineWireSession::fixed_trace(bindings.lane(id, &raw).unwrap(), m).unwrap();
    assert!(reordered.feed(&raw[11..]).unwrap().terminal);
    let mut tampered =
        fs::read(root("LITO_R538_INPUT_ROOT").join("wire-session-bindings.json")).unwrap();
    tampered.push(b' ');
    assert!(WireBindings::parse(&tampered).is_err());
    save(
        "r538-partial-eof-and-binding-errors-check.json",
        json!({"state":"R538_NEW_PARTIAL_EOF_AND_KEY_BINDING_COUNTEREXAMPLES_PASS","actual_memory_partial_streams":checks,
        "header_EOF_delivery_prefix_variants":header_checks,"missing_key_and_foreign_key_or_tail_refused":true,"partial_counter_advanced":false,"partial_dispatched_as_body":false,
        "six_RESULT_wire_keys_invented":false,"live_wire_generation_proven":false,"old_PASS_replayed":false,"SQL_TLS_GUI_provider":0,"new_full_bindings":0}),
    );
}

#[test]
fn r538_native_wire_actual_loopback_received_ordered_streams_and_partial_eof() {
    let bytes =
        fs::read(root("LITO_R538_LOOPBACK_ROOT").join("wire-session-bindings.json")).unwrap();
    let bindings = WireBindings::parse(&bytes).unwrap();
    let doc: Value = serde_json::from_slice(&bytes).unwrap();
    let mut checks = Vec::new();
    let (mut bodies, mut partials, mut healthy, mut quitters, mut observed_eof) = (0, 0, 0, 0, 0);
    for row in doc["lanes"].as_array().unwrap() {
        let raw = wire(row);
        let id = row["id"].as_str().unwrap();
        let (m, spawn) = model();
        let mut session =
            OfflineWireSession::fixed_trace(bindings.lane(id, &raw).unwrap(), m).unwrap();
        let old = session.command().unwrap();
        if row["role"] == "quitter" {
            active(&mut session);
        }
        let mut decoded = Vec::new();
        for range in [0..1, 1..2, 2..7, 7..raw.len()] {
            let batch = session.feed(&raw[range.clone()]).unwrap();
            assert_eq!(batch.consumed, range.len());
            assert!(!batch.terminal);
            decoded.extend(batch.application_bodies);
        }
        let expected: Vec<Vec<u8>> = row["server_complete_application_bodies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| hex(v.as_str().unwrap()))
            .collect();
        assert_eq!(decoded, expected);
        bodies += decoded.len();
        assert!(
            !session.snapshot().session.self_exit_notified,
            "interleaved D0 is peer departure, not own RESULT confirm"
        );
        let before_eof = session.snapshot();
        if row["role"] == "healthy-peer" {
            healthy += 1;
            assert_eq!(before_eof.session.view, View::Result);
            assert!(!before_eof.session.commands_enabled);
            let ops: Vec<u8> = decoded.iter().map(|b| b[0]).collect();
            let expected = if row["mode"] == "close_while_peer_blocked_read" {
                vec![0x72, 0x0F, 0xD3, 0xD0]
            } else {
                vec![0x72, 0xD0, 0x0F, 0xD3]
            };
            assert_eq!(ops, expected);
            assert_eq!(row["owner_actual_EOF_before_healthy_teardown"], false);
            assert!(session
                .predict(
                    &old,
                    Cell {
                        x: spawn.x + 1,
                        y: spawn.y
                    },
                    1
                )
                .is_err());
        } else {
            assert_eq!(row["owner_actual_EOF_before_healthy_teardown"], true);
            observed_eof += 1;
            if row["role"] == "quitter" {
                quitters += 1;
                assert_eq!(before_eof.session.view, View::Channel);
            } else {
                partials += 1;
                assert_eq!(before_eof.application_bodies_dispatched, 0);
            }
            session.finish_eof().unwrap();
            assert_eq!(session.snapshot().session.view, View::Disconnected);
            if row["partial_eof"] == true {
                let end = session.snapshot().terminal.unwrap();
                assert_eq!(end.reason, "BodyEof");
                assert_eq!(end.raw_pending_or_rejected_frame, raw[7..]);
                assert_eq!(end.raw_pending_or_rejected_frame.len(), 5);
                assert_eq!(end.before.body_bytes, 3);
                assert_eq!(end.before.body_expected, Some(21));
            } else {
                assert_eq!(session.snapshot().terminal.unwrap().reason, "CleanEof");
            }
            assert!(session
                .predict(
                    &old,
                    Cell {
                        x: spawn.x + 1,
                        y: spawn.y
                    },
                    1
                )
                .is_err());
        }
        checks.push(json!({"id":id,"owner_actually_received_stream":true,"Modern_fixed_trace_delivery_assumed":true,
            "before_source_observed_EOF":before_eof,"after_EOF_if_observed":session.snapshot(),"healthy_EOF_not_fabricated":true}));
    }
    assert_eq!(
        (
            checks.len(),
            bodies,
            partials,
            healthy,
            quitters,
            observed_eof
        ),
        (6, 13, 2, 2, 2, 4)
    );
    let rows = doc["lanes"].as_array().unwrap();
    let a = &rows[0];
    let b = &rows[3];
    let (m, _) = model();
    let raw = wire(a);
    let foreign = wire(b);
    let mut cross =
        OfflineWireSession::fixed_trace(bindings.lane(a["id"].as_str().unwrap(), &raw).unwrap(), m)
            .unwrap();
    assert!(cross.feed(&foreign[..7]).unwrap().terminal);
    assert_eq!(cross.snapshot().application_bodies_dispatched, 0);
    let (m, _) = model();
    let raw = wire(b);
    let mut read = Cursor::new(raw.clone());
    let mut untagged = OfflineWireSession::untagged_boundary(
        bindings.lane(b["id"].as_str().unwrap(), &raw).unwrap(),
        m,
    )
    .unwrap();
    active(&mut untagged);
    untagged
        .read_untagged_from(&mut read, MAX_READ_BYTES)
        .unwrap();
    assert_eq!(untagged.snapshot().session.view, View::ReturnUnknown);
    save(
        "r538-actual-loopback-receive-order-check.json",
        json!({"state":"R538_NEW_ACTUAL_RECEIVED_STREAM_DECODER_REPLAY_PASS",
        "streams":6,"actual_received_complete_bodies":13,"actual_partial_tails":2,"source_observed_EOF_applied":4,
        "healthy_RESULT_views":2,"quitter_Channel_views_under_fixed_trace_assumption":2,"checks":checks,
        "untagged_current_quitter_return":untagged.snapshot(),"peer_D0_as_own_confirm":false,
        "cross_session_S_KEY_borrowed":false,"server_or_TCP_restarted":false,"old_66ED_RESULT_keys_supplied":false,
        "full_player_ACK_or_live_WORLD_proven":false,"old_PASS_replayed":false,"SQL_TLS_GUI_provider":0,"new_full_bindings":0}),
    );
}
