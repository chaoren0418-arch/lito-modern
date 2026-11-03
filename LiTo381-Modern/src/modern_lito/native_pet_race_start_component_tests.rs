use super::*;
use crate::modern_lito::{
    native_pet_race_quit_session::View,
    native_pet_race_wire_boundary::{OfflineWireSession, WireBindings},
};
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
fn documents() -> (WireBindings, Value, Value) {
    let path = root("LITO_R540_INPUT_ROOT");
    let b = fs::read(path.join("wire-session-bindings.json")).unwrap();
    (
        WireBindings::parse(&b).unwrap(),
        serde_json::from_slice(&b).unwrap(),
        serde_json::from_slice(&fs::read(path.join("per-frame-replay.json")).unwrap()).unwrap(),
    )
}
fn hex(s: &str) -> Vec<u8> {
    crate::modern_lito::native_pet_race_wire_boundary::from_hex(s).unwrap()
}
fn model(bindings: &WireBindings, row: &Value, untagged: bool) -> OfflineWireSession {
    let id = row["id"].as_str().unwrap();
    let wire = fs::read(row["wire"]["path"].as_str().unwrap()).unwrap();
    let session = bindings.room_start_component_fixture(id).unwrap();
    assert_eq!(session.snapshot().view, View::Room);
    assert!(session.snapshot().roster.is_none() && session.command().is_none());
    let lane = bindings.lane(id, &wire).unwrap();
    if untagged {
        OfflineWireSession::untagged_boundary(lane, session).unwrap()
    } else {
        OfflineWireSession::fixed_trace(lane, session).unwrap()
    }
}
fn component_contract() -> Contract {
    Contract::parse(
        &fs::read(root("LITO_R540_COMPONENT_ROOT").join("adopted-r533-component-contract.json"))
            .unwrap(),
    )
    .unwrap()
}
fn save(name: &str, v: Value) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root("LITO_R540_OUTPUT_ROOT").join(name))
        .unwrap();
    writeln!(f, "{}", serde_json::to_string_pretty(&v).unwrap()).unwrap();
}
fn denied(s: &mut OfflineWireSession) {
    assert_eq!(s.snapshot().session.view, View::Room);
    assert!(s.command().is_none());
    assert!(
        s.prepare_active_quit().is_err()
            && s.result_ack().is_err()
            && s.prepare_result_confirm().is_err()
            && s.prepare_result_wire().is_err()
    );
    let c = s.snapshot().start_component.unwrap();
    assert_eq!(c.client_phase, "ROOM");
    assert!(
        !c.live_world_admitted
            && !c.movement_enabled
            && !c.renderer_loaded
            && !c.wire_generation_known
            && !c.local_clock_synchronized
            && !c.battle_rules_whole_class_equivalent
    );
}
fn ready(s: &mut OfflineWireSession, row: &Value, replay: &Value) {
    let counter = s.snapshot().receiver.as_ref().unwrap().counter.unwrap();
    let write = s.prepare_source_ready_wire().unwrap();
    let view = s.ready_write_view(&write).unwrap();
    assert_eq!(
        view.bytes().unwrap(),
        hex(row["start"]["ready_frame_hex"].as_str().unwrap())
    );
    assert_eq!(
        view.bytes().unwrap(),
        fs::read(replay["sent"]["path"].as_str().unwrap()).unwrap()
    );
    assert_eq!(
        s.snapshot().sender.as_ref().unwrap().counter,
        Some(counter + 2)
    );
    assert_eq!(
        s.snapshot().receiver.as_ref().unwrap().counter,
        Some(counter)
    );
    assert!(!s.note_offline_ready_wire_written(&write, 0, 1).unwrap());
    assert!(
        !s.snapshot()
            .start_component
            .as_ref()
            .unwrap()
            .own_ready_request_assumed_written
    );
    assert!(view.bytes().is_err());
    assert!(s.note_offline_ready_wire_written(&write, 1, 3).unwrap());
    assert!(
        s.snapshot()
            .start_component
            .unwrap()
            .own_ready_request_assumed_written
    );
    assert!(s.ready_write_view(&write).is_err() && s.prepare_source_ready_wire().is_err());
}
fn feed_exact(
    s: &mut OfflineWireSession,
    raw: &[u8],
    start: usize,
    end: usize,
    pattern: usize,
    frames: &[Value],
    bodies: &mut Vec<Vec<u8>>,
) {
    if pattern == 3 {
        let mut at = start;
        while at < end {
            let b = s.feed(&raw[at..end]).unwrap();
            assert!(b.consumed > 0 && !b.terminal);
            at += b.consumed;
            bodies.extend(b.application_bodies);
        }
        return;
    }
    for f in frames.iter().filter(|f| {
        let at = f["offset"].as_u64().unwrap() as usize;
        at >= start && at < end
    }) {
        let at = f["offset"].as_u64().unwrap() as usize;
        let len = f["length"].as_u64().unwrap() as usize;
        let chunks = match pattern {
            0 => vec![len],
            1 => vec![1, 1, len - 3, 1],
            _ => vec![1; len],
        };
        let mut n = 0;
        let dispatched = s.snapshot().application_bodies_dispatched;
        let tx = s.snapshot().sender;
        for cap in chunks {
            let b = s.feed(&raw[at + n..at + n + cap]).unwrap();
            assert_eq!(b.consumed, cap);
            assert!(!b.terminal);
            n += cap;
            assert_eq!(s.snapshot().sender, tx);
            assert_eq!(
                s.snapshot().receiver.as_ref().unwrap().counter,
                Some(
                    f[if n == len {
                        "counter_after"
                    } else {
                        "counter_before"
                    }]
                    .as_u64()
                    .unwrap()
                )
            );
            if n < len {
                assert!(b.application_bodies.is_empty());
                assert_eq!(s.snapshot().application_bodies_dispatched, dispatched);
            } else {
                assert_eq!(
                    b.application_bodies,
                    vec![hex(f["body_hex"].as_str().unwrap())]
                );
                bodies.extend(b.application_bodies);
            }
        }
        denied(s);
    }
}
fn start_through_prepare(
    s: &mut OfflineWireSession,
    wire: &[u8],
    row: &Value,
    replay: &Value,
) -> Vec<Vec<u8>> {
    s.feed(&wire[..7]).unwrap();
    ready(s, row, replay);
    let mut bodies = Vec::new();
    for (start, end) in [(7, 107), (107, 111), (111, 115)] {
        bodies.extend(s.feed(&wire[start..end]).unwrap().application_bodies);
        s.replay_next_start_milestone().unwrap();
    }
    assert_eq!(
        s.replay_next_start_milestone().unwrap(),
        MilestoneKind::Begin
    );
    assert_eq!(
        s.replay_next_start_milestone().unwrap(),
        MilestoneKind::Prepare
    );
    bodies
}
#[test]
fn r540_start_component_keyed_ready_countdown_roster_hud_actor_and_teardown_scope() {
    let (bindings, doc, replay) = documents();
    let contract = component_contract();
    let mut checks = Vec::new();
    for (index, row) in doc["lanes"].as_array().unwrap().iter().enumerate() {
        let trace = &replay["lanes"][index];
        let frames = trace["server_frames"].as_array().unwrap();
        let wire = fs::read(row["wire"]["path"].as_str().unwrap()).unwrap();
        for pattern in 0..4 {
            let mut s = model(&bindings, row, false);
            denied(&mut s);
            assert!(
                s.prepare_source_ready_wire().is_err()
                    && s.begin_restricted_start_clock(&contract).is_err()
                    && s.replay_next_start_milestone().is_err()
            );
            for (start, end) in [(0, 1), (1, 2), (2, 6), (6, 7)] {
                s.feed(&wire[start..end]).unwrap();
                if end < 7 {
                    assert!(s.snapshot().receiver.as_ref().unwrap().counter.is_none());
                    assert!(s.prepare_source_ready_wire().is_err());
                }
            }
            ready(&mut s, row, trace);
            let mut bodies = Vec::new();
            for (start, end, kind) in [
                (7, 107, MilestoneKind::CountdownStarted),
                (107, 111, MilestoneKind::Tick),
                (111, 115, MilestoneKind::Tick),
            ] {
                feed_exact(&mut s, &wire, start, end, pattern, frames, &mut bodies);
                assert_eq!(s.replay_next_start_milestone().unwrap(), kind);
                assert_eq!(
                    s.snapshot()
                        .start_component
                        .as_ref()
                        .unwrap()
                        .source_local_generation,
                    11
                );
            }
            assert_eq!(
                s.replay_next_start_milestone().unwrap(),
                MilestoneKind::Begin
            );
            denied(&mut s);
            let begin = s.snapshot();
            let c = begin.start_component.as_ref().unwrap();
            assert_eq!(c.source_match_phase, SourceMatchPhase::Running);
            assert_eq!(c.source_local_generation, 12);
            assert_eq!(c.source_roster_ids, [101, 102, 103]);
            assert_eq!(
                c.source_deadline_nanos.unwrap() - c.source_start_nanos.unwrap(),
                300_000_000_000
            );
            assert!(c.offline_clock.is_none());
            if pattern != 0 {
                s.begin_restricted_start_clock(&contract).unwrap();
                assert!(s.begin_restricted_start_clock(&contract).is_err());
            }
            assert_eq!(
                s.replay_next_start_milestone().unwrap(),
                MilestoneKind::Prepare
            );
            feed_exact(&mut s, &wire, 115, 423, pattern, frames, &mut bodies);
            denied(&mut s);
            let hud = s.snapshot();
            let roster = hud.session.roster.as_ref().unwrap();
            assert_eq!(roster.rows.len(), 3);
            assert_eq!(roster.local_index, Some(index as u16));
            assert_eq!(hud.session.lap_values, Some((4, 1)));
            assert!(hud.session.hud_start_observed);
            let c = hud.start_component.as_ref().unwrap();
            assert_eq!(c.countdown_receipts, [3, 2, 1]);
            assert_eq!(c.ready_observations.len(), 3);
            assert!(c.hud_timer_start_observed);
            assert_eq!(c.actors.len(), 3);
            for actor in &c.actors {
                assert_eq!(actor.gfx, 5065);
                assert!(
                    actor.ancillary_fields_synthetic
                        && !actor.scene_prerequisites_qualified
                        && !actor.drawable_actor_installed
                );
                assert_eq!(
                    actor.hp_ratio,
                    if actor.object_id == 101 + index as u32 {
                        255
                    } else {
                        100
                    }
                );
            }
            if pattern == 0 {
                assert!(c.offline_clock.is_none());
                assert!(s.advance_restricted_start_elapsed(1).is_err());
            }
            assert_eq!(
                s.replay_next_start_milestone().unwrap(),
                MilestoneKind::DirectGate
            );
            let gate = s.snapshot();
            denied(&mut s);
            if pattern != 0 {
                let t = gate
                    .start_component
                    .as_ref()
                    .unwrap()
                    .offline_direct_gate
                    .unwrap();
                assert!(t.gate_advanced && !t.lap_completed && !t.finished);
                assert_eq!(t.completed_laps, 0);
                assert_eq!(
                    gate.start_component
                        .as_ref()
                        .unwrap()
                        .offline_panel
                        .unwrap()
                        .kind,
                    super::super::native_pet_race_component::PanelType::None
                );
            } else {
                assert!(gate
                    .start_component
                    .as_ref()
                    .unwrap()
                    .offline_direct_gate
                    .is_none());
            }
            assert_eq!(gate.start_component.as_ref().unwrap().actors, c.actors); // Synthetic component edge never commits actor position.
            assert_eq!(
                s.replay_next_start_milestone().unwrap(),
                MilestoneKind::Teardown
            );
            let after_teardown = s.snapshot();
            let c = after_teardown.start_component.as_ref().unwrap();
            assert_eq!(c.source_local_generation, 13);
            assert_eq!(c.source_match_phase, SourceMatchPhase::Waiting);
            assert!(!c.active && c.teardown_observed && c.offline_clock.is_none());
            assert!(
                s.begin_restricted_start_clock(&contract).is_err()
                    && s.replay_next_start_milestone().is_err()
            );
            feed_exact(&mut s, &wire, 423, wire.len(), pattern, frames, &mut bodies);
            denied(&mut s);
            assert!(!s.snapshot().session.self_exit_notified);
            assert_eq!(s.snapshot().session.owner_updates.len(), index);
            let expected: Vec<Vec<u8>> = frames
                .iter()
                .map(|f| hex(f["body_hex"].as_str().unwrap()))
                .collect();
            assert_eq!(bodies, expected);
            let before_eof = s.snapshot();
            s.finish_eof().unwrap();
            let after_eof = s.snapshot();
            assert_eq!(after_eof.session.view, View::Disconnected);
            assert!(
                after_eof.receiver.is_none()
                    && after_eof.sender.is_none()
                    && after_eof.start_component.as_ref().unwrap().closed
            );
            assert!(
                s.replay_next_start_milestone().is_err()
                    && s.advance_restricted_start_elapsed(0).is_err()
            );
            checks.push(json!({"lane":row["id"],"pattern":pattern,"begin":begin,"HUD_actor":hud,"direct_component_gate":gate,"teardown":after_teardown,"before_EOF":before_eof,"after_EOF":after_eof}));
        }
    }
    save(
        "r540-start-keyed-component-check.json",
        json!({"streams":3,"own_keys":3,"source_start_bodies":36,"source_teardown_bodies":6,"source_READY_commands":3,"segmented_merged_patterns":12,"local_generation_path":[11,12,13],"client_phase_stays_ROOM":true,"full_WORLD":false,"C_MOVE":false,"checks":checks}),
    );
}
#[test]
fn r540_start_component_unknown_clock_restricted_elapsed_and_untagged_read() {
    let (bindings, doc, replay) = documents();
    let contract = component_contract();
    let row = &doc["lanes"][0];
    let trace = &replay["lanes"][0];
    let wire = fs::read(row["wire"]["path"].as_str().unwrap()).unwrap();
    let mut s = model(&bindings, row, false);
    start_through_prepare(&mut s, &wire, row, trace);
    s.feed(&wire[115..423]).unwrap();
    let unknown = s.snapshot();
    assert!(unknown
        .start_component
        .as_ref()
        .unwrap()
        .offline_clock
        .is_none());
    assert!(s.advance_restricted_start_elapsed(1).is_err());
    s.begin_restricted_start_clock(&contract).unwrap();
    s.advance_restricted_start_elapsed(100).unwrap();
    let before = s.snapshot();
    assert!(
        s.advance_restricted_start_elapsed(99).is_err()
            && s.advance_restricted_start_elapsed(-1).is_err()
    );
    assert_eq!(s.snapshot(), before);
    assert!(s
        .advance_restricted_start_elapsed(row["start"]["source_start_nanos"].as_i64().unwrap())
        .is_err());
    assert!(s.advance_restricted_start_elapsed(300_000_000_001).is_err());
    s.advance_restricted_start_elapsed(300_000_000_000).unwrap();
    let deadline = s.snapshot();
    assert_eq!(
        deadline
            .start_component
            .as_ref()
            .unwrap()
            .offline_clock
            .as_ref()
            .unwrap()
            .elapsed_nanos,
        300_000_000_000
    );
    denied(&mut s);
    s.finish_eof().unwrap();
    assert!(s
        .snapshot()
        .start_component
        .unwrap()
        .offline_clock
        .is_none());
    let mut untagged = Vec::new();
    for (index, row) in doc["lanes"].as_array().unwrap().iter().enumerate() {
        let raw = fs::read(row["wire"]["path"].as_str().unwrap()).unwrap();
        let mut s = model(&bindings, row, true);
        let mut seed = Cursor::new(raw[..7].to_vec());
        assert_eq!(
            s.read_untagged_from(&mut seed, 7)
                .unwrap()
                .batch
                .complete_frames,
            1
        );
        ready(&mut s, row, &replay["lanes"][index]);
        let mut reader = Cursor::new(raw[7..].to_vec());
        let first = s.read_untagged_from(&mut reader, 4096).unwrap();
        assert_eq!(first.batch.complete_frames, 8);
        assert!(s.snapshot().queued_wire_bytes > 0);
        let mut bodies = first.batch.application_bodies;
        while s.snapshot().queued_wire_bytes > 0 {
            let step = s.read_untagged_from(&mut reader, 4096).unwrap();
            assert_eq!(step.socket_or_reader_bytes, 0);
            bodies.extend(step.batch.application_bodies);
        }
        denied(&mut s);
        let before = s.snapshot();
        let c = before.start_component.as_ref().unwrap();
        assert!(
            c.offline_clock.is_none()
                && c.source_milestones.is_empty()
                && c.source_start_nanos.is_none()
        );
        assert_eq!(c.source_local_generation, 11);
        assert_eq!(c.source_match_phase, SourceMatchPhase::Waiting);
        assert!(
            s.replay_next_start_milestone().is_err()
                && s.begin_restricted_start_clock(&contract).is_err()
        );
        assert_eq!(
            bodies.len(),
            replay["lanes"][index]["server_frames"]
                .as_array()
                .unwrap()
                .len()
        );
        assert!(s.read_untagged_from(&mut reader, 4096).unwrap().eof);
        assert!(s.snapshot().start_component.as_ref().unwrap().closed);
        let mut fixed = model(&bindings, row, false);
        let mut unread = Cursor::new(raw);
        assert!(fixed.read_untagged_from(&mut unread, 7).is_err());
        assert_eq!(unread.position(), 0);
        untagged.push(json!({"lane":row["id"],"before_EOF":before,"after_EOF":s.snapshot()}));
    }
    save(
        "r540-clock-domain-and-untagged-check.json",
        json!({"default_local_clock":"Unknown","server_absolute_nanoTime_is_not_local_or_UTC":true,"restricted_elapsed_deadline":300_000_000_000_i64,"backward_negative_absolute_and_over_deadline_rejected":true,"unknown_after_HUD":unknown,"restricted_deadline":deadline,"untagged":untagged,"untagged_source_milestone_import_refused":3,"live_authority":false}),
    );
}
#[test]
fn r540_start_component_atomic_decoders_foreign_binding_partial_eof_and_ready_retirement() {
    let (bindings, doc, replay) = documents();
    let row = &doc["lanes"][0];
    let raw = fs::read(row["wire"]["path"].as_str().unwrap()).unwrap();
    let frames = replay["lanes"][0]["server_frames"].as_array().unwrap();
    let mut prefix_rejections = 0;
    let binding: StartBinding = serde_json::from_value(row["start"].clone()).unwrap();
    for f in frames
        .iter()
        .filter(|f| matches!(hex(f["body_hex"].as_str().unwrap())[0], 0xD1 | 0xE1 | 0x0B))
    {
        let body = hex(f["body_hex"].as_str().unwrap());
        let mut component = StartComponent::begin(binding.clone()).unwrap();
        let initial = component.snapshot();
        for n in 0..body.len() {
            assert!(component.observe(&body[..n]).is_err());
            assert_eq!(component.snapshot(), initial);
            prefix_rejections += 1;
        }
        let mut trailing = body.clone();
        trailing.push(0);
        assert!(component.observe(&trailing).is_err());
        assert_eq!(component.snapshot(), initial);
        component.observe(&body).unwrap();
    }
    let mut same = StartComponent::begin(binding).unwrap();
    let body = hex(frames[0]["body_hex"].as_str().unwrap());
    for _ in 0..3 {
        same.observe(&body).unwrap();
    }
    same.observe(&[0xE1, 3]).unwrap();
    assert!(same.next_milestone(107).is_err()); // repeated alias cannot supply three roster members
    let id = row["id"].as_str().unwrap();
    let mut changed = raw.clone();
    changed[3] ^= 1;
    assert!(bindings.lane(id, &changed).is_err());
    assert!(OfflineWireSession::fixed_trace(
        bindings.lane(id, &raw).unwrap(),
        bindings.room_start_component_fixture("role1").unwrap()
    )
    .is_err());
    let mut s = model(&bindings, row, false);
    s.feed(&raw[..7]).unwrap();
    let ticket = s.prepare_source_ready_wire().unwrap();
    let view = s.ready_write_view(&ticket).unwrap();
    assert!(!s.note_offline_ready_wire_written(&ticket, 0, 2).unwrap());
    s.finish_eof().unwrap();
    assert!(view.bytes().is_err() && s.note_offline_ready_wire_written(&ticket, 2, 2).is_err());
    assert!(
        !s.snapshot()
            .start_component
            .as_ref()
            .unwrap()
            .own_ready_request_assumed_written
    );
    let mut partial = model(&bindings, row, false);
    let mut at = 0;
    while at < 222 {
        at += partial.feed(&raw[at..222]).unwrap().consumed;
    }
    assert_eq!(
        partial
            .snapshot()
            .start_component
            .as_ref()
            .unwrap()
            .actors
            .len(),
        0
    );
    partial.feed(&raw[222..225]).unwrap();
    let before = partial.snapshot();
    partial.finish_eof().unwrap();
    let after = partial.snapshot();
    assert_eq!(after.terminal.as_ref().unwrap().reason, "BodyEof");
    assert_eq!(
        after.terminal.as_ref().unwrap().before.counter,
        before.receiver.unwrap().counter
    );
    assert!(
        after.start_component.as_ref().unwrap().actors.is_empty()
            && after.start_component.as_ref().unwrap().closed
    );
    assert_eq!(
        after
            .session
            .partial_transport
            .as_ref()
            .unwrap()
            .raw_encrypted_prefix,
        &raw[222..225]
    );
    let mut swapped = model(&bindings, row, false);
    let other = fs::read(doc["lanes"][1]["wire"]["path"].as_str().unwrap()).unwrap();
    assert!(swapped.feed(&other[..7]).unwrap().terminal);
    assert!(swapped.snapshot().start_component.as_ref().unwrap().closed);
    let mut candidate = bindings.room_start_component_fixture(id).unwrap();
    let scope = candidate.receipt_scope();
    let roster = hex(frames[6]["body_hex"].as_str().unwrap());
    let base = candidate.snapshot();
    for n in 0..roster.len() {
        assert!(candidate.observe(&scope, &roster[..n]).is_err());
        assert_eq!(candidate.snapshot(), base);
    }
    candidate.disconnect(&scope).unwrap();
    assert!(candidate.observe(&scope, &roster).is_err());
    save(
        "r540-start-boundary-retirement-check.json",
        json!({"complete_atomic_prefix_rejections":prefix_rejections,"FD1_prefix_rejections":roster.len(),"duplicate_READY_alias_does_not_complete_roster":true,"foreign_key_and_fixture_scope_refused":true,"pending_READY_EOF":s.snapshot(),"actual_actor_ciphertext_shortened_CPU_EOF":after,"source_observed_partial_trace":false,"stale_receipt_scope_refused":true,"live_admission":false}),
    );
}
