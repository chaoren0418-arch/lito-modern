use super::*;
use serde_json::{json, Value};
use std::{
    fs,
    io::{Cursor, Write},
    path::PathBuf,
};
fn root(var: &str) -> PathBuf {
    let p = PathBuf::from(std::env::var_os(var).unwrap());
    assert!(p.is_absolute());
    p
}
fn bindings() -> (WireBindings, Value) {
    let b = fs::read(root("LITO_R539_INPUT_ROOT").join("wire-session-bindings.json")).unwrap();
    (
        WireBindings::parse(&b).unwrap(),
        serde_json::from_slice(&b).unwrap(),
    )
}
fn model(bindings: &WireBindings, row: &Value, untagged: bool) -> OfflineWireSession {
    let id = row["id"].as_str().unwrap();
    let wire = fs::read(row["wire"]["path"].as_str().unwrap()).unwrap();
    let session = bindings.result_start_fixture(id).unwrap();
    let initial = session.snapshot();
    assert!(
        initial.roster.is_none() && initial.lap_values.is_none() && initial.winning_force.is_none()
    );
    assert!(
        initial.result_rows.is_empty()
            && initial.named_times.is_empty()
            && session.command().is_none()
    );
    let lane = bindings.lane(id, &wire).unwrap();
    if untagged {
        OfflineWireSession::untagged_boundary(lane, session).unwrap()
    } else {
        OfflineWireSession::fixed_trace(lane, session).unwrap()
    }
}
fn save(name: &str, v: Value) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root("LITO_R539_OUTPUT_ROOT").join(name))
        .unwrap();
    writeln!(f, "{}", serde_json::to_string_pretty(&v).unwrap()).unwrap();
}
fn command_if_due(s: &mut OfflineWireSession, row: &Value, out: &mut Vec<u8>) {
    let index = s.snapshot().source_client_frames_completed;
    if let Some(f) = row["fixture"]["client_frames"]
        .as_array()
        .unwrap()
        .get(index)
    {
        if f["replay_after_rx_offset"].as_u64().unwrap() as usize
            == s.snapshot().wire_bytes_consumed
        {
            let rx = s.snapshot().receiver.unwrap();
            let write = s.prepare_result_wire().unwrap();
            assert_eq!(
                s.snapshot().sender.as_ref().unwrap().counter,
                Some(f["counter_after"].as_u64().unwrap())
            );
            assert_eq!(s.snapshot().receiver.unwrap(), rx);
            assert!(s.prepare_result_wire().is_err()); // Same command cannot overlap a pending write.
            let view = s.result_write_view(&write).unwrap();
            let raw = view.bytes().unwrap().to_vec();
            assert_eq!(raw, from_hex(f["frame_hex"].as_str().unwrap()).unwrap());
            assert!(!s.note_offline_result_wire_written(&write, 0, 1).unwrap());
            assert!(view.bytes().is_err());
            assert!(s.note_offline_result_wire_written(&write, 0, 1).is_err()); // stale offset
            let tail = s.result_write_view(&write).unwrap();
            assert_eq!(tail.bytes().unwrap(), &raw[1..]);
            assert!(s.note_offline_result_wire_written(&write, 1, 2).unwrap());
            assert!(tail.bytes().is_err() && s.result_write_view(&write).is_err());
            assert!(
                !s.snapshot().session.self_exit_notified
                    && !s.snapshot().session.current_reply_assumed
            );
            out.extend(raw);
        }
    }
}
fn expected_phase(row: &Value, body: &[u8], s: &OfflineWireSession) {
    let state = s.snapshot().session;
    if row["role"] == "quitter" {
        assert_eq!(state.view, View::Channel);
        assert!(!state.self_exit_notified && !state.current_reply_assumed);
    } else {
        let alias = from_hex(row["fixture"]["alias_hex"].as_str().unwrap()).unwrap();
        match body[0] {
            0xF0 => {
                assert_eq!(state.view, View::Result);
                assert!(!state.self_exit_notified && !state.current_reply_assumed);
            }
            0xD0 if body[5..body.len() - 1] == alias => {
                assert_eq!(state.view, View::Returning);
                assert!(state.self_exit_notified && state.current_reply_assumed);
            }
            0xD0 | 0xD5 => {
                assert_eq!(state.view, View::Result);
                assert!(!state.self_exit_notified && !state.current_reply_assumed);
            }
            0xCC => assert_eq!(state.view, View::Channel),
            0xCF => assert!(matches!(state.view, View::Returning | View::Channel)),
            _ => panic!("unexpected RESULT source opcode"),
        }
    }
    assert!(
        !state.commands_enabled
            && !state.wire_generation_known
            && !state.actual_room_users_known
            && !state.complete_catalog_known
    );
}
#[test]
fn r539_result_wire_direct_fixture_start_bidirectional_counters_and_segmented_frames() {
    let (bindings, doc) = bindings();
    let mut checks = Vec::new();
    let mut unique_bodies = 0;
    let mut unique_commands = 0;
    for row in doc["lanes"].as_array().unwrap() {
        unique_bodies += row["server_frames"].as_array().unwrap().len();
        unique_commands += row["fixture"]["client_frames"].as_array().unwrap().len();
        let wire = fs::read(row["wire"]["path"].as_str().unwrap()).unwrap();
        for pattern in [0, 1, 2] {
            let mut s = model(&bindings, row, false);
            assert!(s.prepare_result_wire().is_err());
            let mut sent = Vec::new();
            let mut observed = Vec::new();
            for n in [1, 1, 4, 1] {
                let offset = s.snapshot().wire_bytes_consumed;
                let b = s.feed(&wire[offset..offset + n]).unwrap();
                assert_eq!(b.consumed, n);
                assert!(b.application_bodies.is_empty());
                if offset + n < 7 {
                    assert!(s.snapshot().receiver.unwrap().counter.is_none());
                    assert!(s.snapshot().sender.unwrap().counter.is_none());
                }
            }
            assert_eq!(
                s.snapshot().receiver.as_ref().unwrap().counter,
                s.snapshot().sender.as_ref().unwrap().counter
            );
            for f in row["server_frames"].as_array().unwrap() {
                command_if_due(&mut s, row, &mut sent);
                let at = f["offset"].as_u64().unwrap() as usize;
                let len = f["length"].as_u64().unwrap() as usize;
                let chunks = match pattern {
                    0 => vec![len],
                    1 => vec![1, 1, len - 3, 1],
                    _ => vec![1; len],
                };
                let mut read = 0;
                let tx = s.snapshot().sender;
                for n in chunks {
                    let b = s.feed(&wire[at + read..at + read + n]).unwrap();
                    assert_eq!(b.consumed, n);
                    assert!(!b.terminal);
                    read += n;
                    assert_eq!(s.snapshot().sender, tx);
                    if read < len {
                        assert!(b.application_bodies.is_empty());
                        assert_eq!(
                            s.snapshot().receiver.unwrap().counter,
                            Some(f["counter_before"].as_u64().unwrap())
                        );
                    } else {
                        assert_eq!(
                            b.application_bodies,
                            vec![from_hex(f["body_hex"].as_str().unwrap()).unwrap()]
                        );
                        observed.extend(b.application_bodies);
                        assert_eq!(
                            s.snapshot().receiver.unwrap().counter,
                            Some(f["counter_after"].as_u64().unwrap())
                        );
                    }
                }
                expected_phase(row, observed.last().unwrap(), &s);
            }
            assert_eq!(
                sent,
                fs::read(row["sent"]["path"].as_str().unwrap()).unwrap()
            );
            let before = s.snapshot();
            assert_eq!(before.session.view, View::Channel);
            assert_eq!(
                before.source_client_frames_completed,
                row["fixture"]["client_frames"].as_array().unwrap().len()
            );
            assert!(before.session.roster.is_none() && before.session.winning_force.is_none());
            if row["scenario"] == "default_room_last_member_reset" {
                let room = before
                    .session
                    .room_rows
                    .iter()
                    .find(|r| r.room_id == 2)
                    .unwrap();
                assert_eq!(room.name_raw, b"Local Lobby");
                assert_eq!((room.displayed_users_wire, room.capacity), (1, 8));
            } else {
                assert!(before.session.deleted_room_ids.contains(&3));
                assert!(!before.session.room_rows.iter().any(|r| r.room_id == 3));
            }
            if row["role"] == "healthy-peer" {
                assert_eq!(before.session.owner_updates.len(), 1);
            }
            s.finish_eof().unwrap();
            let after = s.snapshot();
            assert_eq!(after.terminal.as_ref().unwrap().reason, "CleanEof");
            assert_eq!(after.session.view, View::Disconnected);
            assert!(after.receiver.is_none() && after.sender.is_none());
            assert!(
                s.feed(&[]).is_err() && s.prepare_result_wire().is_err() && s.result_ack().is_err()
            );
            checks.push(
                json!({"lane":row["id"],"pattern":pattern,"before_EOF":before,"after_EOF":after}),
            );
        }
        let mut merged = model(&bindings, row, false);
        let mut sent = Vec::new();
        let mut observed = Vec::new();
        let mut cuts: Vec<usize> = row["fixture"]["client_frames"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["replay_after_rx_offset"].as_u64().unwrap() as usize)
            .collect();
        cuts.push(wire.len());
        for end in cuts {
            command_if_due(&mut merged, row, &mut sent);
            let start = merged.snapshot().wire_bytes_consumed;
            observed.extend(merged.feed(&wire[start..end]).unwrap().application_bodies);
        }
        let expected: Vec<Vec<u8>> = row["server_frames"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| from_hex(f["body_hex"].as_str().unwrap()).unwrap())
            .collect();
        assert_eq!(observed, expected);
        assert_eq!(
            sent,
            fs::read(row["sent"]["path"].as_str().unwrap()).unwrap()
        );
        assert_eq!(merged.snapshot().session.view, View::Channel);
        let before = merged.snapshot();
        merged.finish_eof().unwrap();
        checks.push(json!({"lane":row["id"],"pattern":"merged_to_command_windows","before_EOF":before,"after_EOF":merged.snapshot()}));
    }
    assert_eq!((unique_bodies, unique_commands), (22, 8));
    save(
        "r539-result-wire-bidirectional-check.json",
        json!({"streams":6,"own_keys":6,"actual_unique_source_bodies":unique_bodies,"actual_unique_source_commands":unique_commands,"segmented_and_merged_replay_patterns":24,"synthetic_D3_or_combat":0,"existing_Receiver_and_Sender":true,"source_EOF_scopes":6,"Modern_actual_socket":false,"checks":checks}),
    );
}
#[test]
fn r539_result_wire_untagged_read_never_promotes_source_fixture_to_live_authority() {
    let (bindings, doc) = bindings();
    let mut checks = Vec::new();
    for row in doc["lanes"].as_array().unwrap() {
        let wire = fs::read(row["wire"]["path"].as_str().unwrap()).unwrap();
        let mut s = model(&bindings, row, true);
        let mut sent = Vec::new();
        let mut spans = vec![(0, 7)];
        spans.extend(row["server_frames"].as_array().unwrap().iter().map(|f| {
            (
                f["offset"].as_u64().unwrap() as usize,
                f["length"].as_u64().unwrap() as usize,
            )
        }));
        for (at, len) in spans {
            command_if_due(&mut s, row, &mut sent);
            let mut r = Cursor::new(wire[at..at + len].to_vec());
            while r.position() < (len as u64) {
                let b = s.read_untagged_from(&mut r, 2).unwrap();
                assert!(!b.eof && !b.batch.terminal);
            }
        }
        let before = s.snapshot();
        assert!(!before.client_receipt_proven && !before.wire_generation_known);
        assert_eq!(
            before.session.view,
            if row["role"] == "quitter" {
                View::Channel
            } else {
                View::ReturnUnknown
            }
        );
        assert!(!before.session.current_reply_assumed && !before.session.current_room_list_assumed);
        assert!(!before.session.actual_room_users_known && !before.session.complete_catalog_known);
        let mut eof = Cursor::new(Vec::<u8>::new());
        assert!(s.read_untagged_from(&mut eof, 2).unwrap().eof);
        assert_eq!(s.snapshot().session.view, View::Disconnected);
        let mut fixed = model(&bindings, row, false);
        let mut unread = Cursor::new(wire);
        assert!(fixed.read_untagged_from(&mut unread, 2).is_err());
        assert_eq!(unread.position(), 0);
        checks
            .push(json!({"lane":row["id"],"untagged_before_EOF":before,"after_EOF":s.snapshot()}));
    }
    save(
        "r539-untagged-result-boundary-check.json",
        json!({"streams":6,"generic_Read_fixed_policy_refused":6,"RESULT_peers_return_unknown":4,"CHANNEL_watchers_initial_condition":2,"Modern_receipt_ACK":false,"checks":checks}),
    );
}
#[test]
fn r539_result_wire_binding_pending_write_and_eof_retirement_fail_closed() {
    let (bindings, doc) = bindings();
    let row = &doc["lanes"][1];
    let other = &doc["lanes"][2];
    let wire = fs::read(row["wire"]["path"].as_str().unwrap()).unwrap();
    let mut changed = wire.clone();
    changed[3] ^= 1;
    assert!(bindings
        .lane(row["id"].as_str().unwrap(), &changed)
        .is_err());
    assert!(bindings.lane("unknown", &wire).is_err());
    assert!(WireBindings::parse(b"{}").is_err());
    assert!(OfflineWireSession::fixed_trace(
        bindings.lane(row["id"].as_str().unwrap(), &wire).unwrap(),
        bindings
            .result_start_fixture(other["id"].as_str().unwrap())
            .unwrap()
    )
    .is_err());
    let mut s = model(&bindings, row, false);
    s.feed(&wire[..7]).unwrap();
    let write = s.prepare_result_wire().unwrap();
    let view = s.result_write_view(&write).unwrap();
    assert!(!s.note_offline_result_wire_written(&write, 0, 1).unwrap());
    let tail = s.result_write_view(&write).unwrap();
    let before = s.snapshot();
    s.finish_eof().unwrap();
    assert!(view.bytes().is_err() && tail.bytes().is_err());
    assert!(
        s.result_write_view(&write).is_err()
            && s.note_offline_result_wire_written(&write, 1, 2).is_err()
    );
    assert_eq!(
        s.snapshot().terminal.as_ref().unwrap().sender_before,
        before.sender
    );
    assert_eq!(s.snapshot().session.view, View::Disconnected);
    let mut zero = model(&bindings, row, false);
    zero.feed(&wire[..7]).unwrap();
    let z = zero.prepare_result_wire().unwrap();
    let zv = zero.result_write_view(&z).unwrap();
    assert!(!zero.note_offline_result_wire_written(&z, 0, 0).unwrap());
    assert!(zv.bytes().is_err());
    assert!(zero
        .snapshot()
        .terminal
        .as_ref()
        .unwrap()
        .reason
        .contains("ZeroWrite"));
    let mut own = model(&bindings, row, false);
    own.feed(&wire[..7]).unwrap();
    let own_write = own.prepare_result_wire().unwrap();
    let own_view = own.result_write_view(&own_write).unwrap();
    let mut foreign = model(&bindings, row, false);
    foreign.feed(&wire[..7]).unwrap();
    assert!(foreign.result_write_view(&own_write).is_err());
    assert!(foreign
        .note_offline_result_wire_written(&own_write, 0, 3)
        .is_err());
    assert!(own_view.bytes().is_ok());
    own.note_offline_result_wire_written(&own_write, 0, 3)
        .unwrap();
    own.feed(&wire[7..11]).unwrap();
    let confirm = own.prepare_result_wire().unwrap();
    let cview = own.result_write_view(&confirm).unwrap();
    own.note_offline_result_wire_written(&confirm, 0, 2)
        .unwrap();
    assert!(
        !own.snapshot().session.self_exit_notified && !own.snapshot().session.current_reply_assumed
    );
    // Actual source ciphertext prefix is replayed as a new shortened EOF case;
    // it is not claimed to be an owner-observed partial connection.
    own.feed(&wire[11..12]).unwrap();
    let rx = own.snapshot().receiver.unwrap();
    own.finish_eof().unwrap();
    assert_eq!(
        own.snapshot().terminal.as_ref().unwrap().reason,
        "HeaderEof"
    );
    assert_eq!(own.snapshot().terminal.as_ref().unwrap().before, rx);
    assert!(
        cview.bytes().is_err()
            && own
                .note_offline_result_wire_written(&confirm, 2, 1)
                .is_err()
    );
    let mut partial = model(&bindings, row, false);
    partial.feed(&wire[..7]).unwrap();
    let mut sent = Vec::new();
    command_if_due(&mut partial, row, &mut sent);
    partial.feed(&wire[7..10]).unwrap();
    let p = partial.snapshot();
    assert_eq!(p.application_bodies_dispatched, 0);
    assert!(p.receiver.as_ref().unwrap().counter.is_some());
    partial.finish_eof().unwrap();
    assert_eq!(
        partial.snapshot().terminal.as_ref().unwrap().reason,
        "BodyEof"
    );
    assert_eq!(
        partial.snapshot().terminal.as_ref().unwrap().before.counter,
        p.receiver.unwrap().counter
    );
    assert_eq!(
        partial
            .snapshot()
            .session
            .partial_transport
            .as_ref()
            .unwrap()
            .raw_encrypted_prefix,
        &wire[7..10]
    );
    assert!(
        !partial
            .snapshot()
            .session
            .partial_transport
            .unwrap()
            .complete_application_body
    );
    let mut missing = model(&bindings, row, false);
    missing.feed(&wire[..11]).unwrap();
    assert_eq!(missing.snapshot().session.view, View::Result);
    missing.feed(&wire[11..46]).unwrap();
    assert!(
        !missing.snapshot().session.self_exit_notified
            && !missing.snapshot().session.current_reply_assumed
    );
    missing.feed(&wire[46..]).unwrap();
    assert_ne!(missing.snapshot().session.view, View::Channel);
    save(
        "r539-result-wire-retirement-check.json",
        json!({"changed_seed_rejected":true,"foreign_fixture_lane_rejected":true,"missing_confirm_cannot_return":true,"pending_write_EOF":s.snapshot(),"zero_write":zero.snapshot(),"Header_EOF":own.snapshot(),"Body_EOF":partial.snapshot(),"old_scope_and_write_views_retired":true,"partial_variant_is_CPU_counterexample_not_actual_owner_trace":true,"Modern_network":0}),
    );
}
