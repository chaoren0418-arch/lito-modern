use super::*;
use crate::modern_lito::{
    native_resources::{BytePin, NativeSectorKey, SectorPin},
    resources::SharedMapSectorKey,
    world_map::SectorInput,
};
use serde_json::json;
use std::{fs, io::Write, path::PathBuf, sync::atomic::AtomicBool};
fn input_root(var: &str) -> PathBuf {
    let p = PathBuf::from(std::env::var_os(var).expect("explicit owned fixture root"));
    assert!(p.is_absolute());
    p
}
fn sha(s: &str) -> [u8; 32] {
    let mut a = [0; 32];
    for (i, v) in a.iter_mut().enumerate() {
        *v = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
    }
    a
}
fn session() -> (Session, Cell) {
    let terrain_root = input_root("LITO_R535_TERRAIN_ROOT");
    let rows: serde_json::Value = serde_json::from_slice(
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
                sha256: sha(r["raw"]["sha256"].as_str().unwrap()),
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
        &fs::read(input_root("LITO_R535_INPUT_ROOT").join("outer-lifecycle-contract-v1.json"))
            .unwrap(),
    )
    .unwrap();
    let spawn = c.course(NAMESPACE, 566).unwrap().spawns[0];
    (
        Session::begin_offline(&o, &c, terrain, &[(7, spawn)], 7, 43, b"A", 0, i64::MAX).unwrap(),
        spawn,
    )
}
fn roster(local: u16) -> Vec<u8> {
    let mut b = vec![0xFD, 1, 2, 0];
    b.extend(local.to_le_bytes());
    b.extend([1, b'A', 0, 2, b'A', 0]);
    b
}
fn lap(max: u16, current: u16) -> Vec<u8> {
    let mut b = vec![0xFD, 2];
    b.extend(max.to_le_bytes());
    b.extend(current.to_le_bytes());
    b
}
fn exit(room: u32, name: &[u8]) -> Vec<u8> {
    let mut b = vec![0xD0];
    b.extend(room.to_le_bytes());
    b.extend(name);
    b.push(0);
    b
}
fn time(name: &[u8], ms: u32) -> Vec<u8> {
    let mut b = vec![0xFD, 3];
    b.extend(name);
    b.push(0);
    b.extend(ms.to_le_bytes());
    b
}
fn result(force: i32) -> Vec<u8> {
    let mut b = vec![0xD3];
    b.extend(force.to_le_bytes());
    b
}
fn catalog() -> Vec<u8> {
    let mut b = vec![0xCC, 0];
    b.extend(43u32.to_le_bytes());
    b.extend([b'R', 0, 0]);
    b.extend(1u16.to_le_bytes());
    b.extend(10u16.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    b.push(0);
    b
}
fn stats() -> Vec<u8> {
    let mut b = vec![0xDB, b'A', 0];
    for v in [0u32, 3, 2, 0] {
        b.extend(v.to_le_bytes());
    }
    b.extend([1, 1]);
    b
}
fn seed(s: &mut Session) -> ReceiptScope {
    let scope = s.receipt_scope();
    s.observe(&scope, &roster(0)).unwrap();
    assert!(s.command().is_none());
    s.observe(&scope, &lap(4, 1)).unwrap();
    assert!(s.command().is_none());
    s.observe(&scope, &[0xFD, 0]).unwrap();
    assert!(s.command().is_some());
    scope
}
fn evidence(name: &str, v: serde_json::Value) {
    let output = input_root("LITO_R535_OUTPUT_ROOT").join(name);
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    writeln!(f, "{}", serde_json::to_string_pretty(&v).unwrap()).unwrap();
}

#[test]
#[ignore = "requires: local evidence roots (snapshot JSON holds absolute D:\\ paths + SHA pins; only runnable on the original machine)"]
fn r535_native_session_prediction_wire_result_return_and_epoch_boundaries() {
    let (mut s, spawn) = session();
    let scope = seed(&mut s);
    let command = s.command().unwrap();
    assert!(
        s.prepare_result_confirm(&scope).is_err(),
        "active C_D0 producer is unqualified"
    );
    let to = Cell {
        x: spawn.x + 1,
        y: spawn.y,
    };
    let m = s.predict(&command, to, 1).unwrap();
    let x = (spawn.x as u16).to_le_bytes();
    let y = (spawn.y as u16).to_le_bytes();
    assert_eq!(m.application_body, [0x0A, x[0], x[1], y[0], y[1], 2]);
    assert_eq!(m.step.race.completed_laps, 0);
    s.observe(&scope, &lap(4, 2)).unwrap();
    assert_eq!(s.snapshot().lap_values, Some((4, 2)));
    assert_eq!(s.predicted_position().unwrap(), to);
    s.observe(&scope, &time(b"A", u32::MAX)).unwrap();
    assert_eq!(s.snapshot().named_times.len(), 1);
    assert!(
        s.command().is_some(),
        "named finish has no placement/terminal authority"
    );
    s.observe(&scope, &exit(43, b"peer")).unwrap();
    assert!(s.command().is_some());
    s.observe(&scope, &result(-1)).unwrap();
    assert_eq!(s.snapshot().winning_force, Some(-1));
    assert!(s.command().is_none());
    assert!(s.predict(&command, spawn, 2).is_err());
    assert!(s.predicted_position().is_err());
    s.observe(&scope, &stats()).unwrap();
    assert_eq!(s.snapshot().result_rows[0].kills, 3);
    assert_eq!(s.snapshot().result_rows[0].force_bytes, [1, 1]);
    assert_eq!(s.result_ack(&scope).unwrap(), [0xD9]);
    s.observe(&scope, &[0xF0, 0]).unwrap();
    assert_eq!(s.snapshot().view, View::Result);
    assert!(!s.snapshot().self_exit_notified);
    assert_eq!(s.snapshot().room_id, 43);
    s.observe(&scope, &catalog()).unwrap();
    assert_eq!(
        s.snapshot().view,
        View::Result,
        "CC alone is not local departure"
    );
    s.observe(&scope, &exit(43, b"A")).unwrap();
    assert!(
        !s.snapshot().self_exit_notified,
        "uncorrelated notification has no own exit ACK"
    );
    let old_confirm = s.prepare_result_confirm(&scope).unwrap();
    let confirm = s.prepare_result_confirm(&scope).unwrap();
    assert!(s.note_offline_confirm_written(&old_confirm).is_err());
    assert_eq!(s.note_offline_confirm_written(&confirm).unwrap(), [0xD0]);
    assert!(s.note_offline_confirm_written(&confirm).is_err());
    s.observe(&scope, &exit(42, b"A")).unwrap();
    assert_eq!(s.snapshot().view, View::Result, "room ID is not object ID");
    s.observe(&scope, &exit(43, b"A")).unwrap();
    assert_eq!(s.snapshot().view, View::Returning);
    assert!(!s.snapshot().room_list_row_observed);
    s.observe(&scope, &[0xF0, 0]).unwrap();
    assert_eq!(s.snapshot().view, View::Returning);
    s.observe(&scope, &catalog()).unwrap();
    assert_eq!(s.snapshot().view, View::Channel);
    s.observe(&scope, &exit(43, b"A")).unwrap();
    assert_eq!(s.snapshot().view, View::Channel);
    assert!(s.observe(&scope, &[0xFD, 0]).is_err());
    s.disconnect(&scope).unwrap();
    assert!(s.observe(&scope, &stats()).is_err());
    let (mut next, _) = session();
    assert!(next.observe(&scope, &roster(0)).is_err());
    assert!(next.predict(&command, to, 1).is_err());
    let next_scope = seed(&mut next);
    assert!(!Arc::ptr_eq(&scope.owner, &next_scope.owner));
    let next_command = next.command().unwrap();
    next.observe(&next_scope, &exit(43, b"A")).unwrap();
    assert_eq!(next.snapshot().view, View::Interrupted);
    assert!(next.command().is_none());
    assert!(!next.snapshot().self_exit_notified);
    assert!(next.predict(&next_command, to, 2).is_err());
    evidence(
        "r535-session-lifecycle-check.json",
        json!({"state":"R535_NEW_OFFLINE_EPOCH_WIRE_AND_RETURN_SEQUENCE_PASS","wire_snapshot":s.snapshot(),"new_epoch_snapshot":next.snapshot(),"prepared_native_move_body":m.application_body,"prediction_is_server_gate_ACK":false,"FD3_has_placement_or_owner_id":false,"FD0_has_server_UTC_or_deadline":false,"F0_proves_departure":false,"raw_unmarked_late_wire_generation_resolved":false,"source_roomlist_supplement":"r535-roomlist-source-supplement.json","active_D0_command_offered":false,"actual_socket_write":false,"SQL_TLS_GUI_provider":0,"new_full_bindings":0}),
    );
}

#[test]
#[ignore = "requires: local evidence roots (snapshot JSON holds absolute D:\\ paths + SHA pins; only runnable on the original machine)"]
fn r535_native_session_malformed_atomicity_seed_and_revocation_counterexamples() {
    let (mut s, spawn) = session();
    let scope = seed(&mut s);
    let valid = [
        roster(0),
        lap(4, 2),
        vec![0xFD, 0],
        time(b"A", 5),
        result(1),
        stats(),
        vec![0xF0, 0],
        exit(43, b"A"),
        catalog(),
    ];
    let mut rejections = 0;
    for body in &valid {
        assert!(decode(body).is_ok());
        for n in 0..body.len() {
            let before = s.snapshot();
            assert!(s.observe(&scope, &body[..n]).is_err(), "prefix{n} {body:?}");
            assert_eq!(s.snapshot(), before);
            rejections += 1;
        }
    }
    for body in [
        vec![0xFD, 99],
        vec![0xFD, 0, 1],
        vec![0xF0, 1],
        vec![0xFD, 1, 11, 0, 0, 0],
        roster(2),
        vec![0x0A],
    ] {
        let before = s.snapshot();
        assert!(s.observe(&scope, &body).is_err());
        assert_eq!(s.snapshot(), before);
        rejections += 1;
    }
    let command = s.command().unwrap();
    assert!(
        s.predict(&command, spawn, 2).is_err(),
        "same-cell query is not a C_MOVE heading"
    );
    assert_eq!(s.predicted_position().unwrap(), spawn);
    s.observe(&scope, &lap(9, 7)).unwrap();
    assert_eq!(s.snapshot().lap_values, Some((9, 7)));
    assert!(s.command().is_none());
    assert!(s
        .predict(
            &command,
            Cell {
                x: spawn.x + 1,
                y: spawn.y
            },
            3
        )
        .is_err());
    let (mut observer, _) = session();
    let observer_scope = observer.receipt_scope();
    observer.observe(&observer_scope, &roster(0xFFFF)).unwrap();
    observer.observe(&observer_scope, &lap(4, 1)).unwrap();
    observer.observe(&observer_scope, &[0xFD, 0]).unwrap();
    assert!(observer.command().is_none());
    let (mut revised, _) = session();
    let revised_scope = seed(&mut revised);
    let c = revised.command().unwrap();
    revised.observe(&revised_scope, &roster(0)).unwrap();
    revised.observe(&revised_scope, &lap(4, 1)).unwrap();
    revised.observe(&revised_scope, &[0xFD, 0]).unwrap();
    assert!(
        revised.command().is_none(),
        "new roster cannot revive old predicted owner"
    );
    assert!(revised
        .predict(
            &c,
            Cell {
                x: spawn.x + 1,
                y: spawn.y
            },
            4
        )
        .is_err());
    s.disconnect(&scope).unwrap();
    assert!(s.observe(&scope, &result(1)).is_err());
    let root = input_root("LITO_R535_INPUT_ROOT");
    let mut bad = fs::read(root.join("outer-lifecycle-contract-v1.json")).unwrap();
    bad.push(b' ');
    assert!(OuterContract::parse(&bad).is_err());
    evidence(
        "r535-session-atomic-errors-check.json",
        json!({"state":"R535_NEW_OFFLINE_COUNTEREXAMPLES_PASS","malformed_prefix_or_domain_rejections":rejections,"snapshot_atomicity":true,"observer_has_no_local_command":true,"changed_roster_does_not_revive_old_owner":true,"local_clock_or_prepared_body_is_server_admission":false,"old_R533_96_or_R534_route_PASS_replayed":false,"SQL_TLS_GUI_provider":0,"new_full_bindings":0}),
    );
}
