use lito_modern::modern_lito::{
    native_world::{self, World},
    native_world_view::{Fence, Key},
};
fn fixture() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("fixtures/native-motion-current.json")).unwrap()
}
fn bytes(v: &serde_json::Value) -> Vec<u8> {
    let s = v.as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn active() -> World {
    let f: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for key in ["world", "start", "status", "actor"] {
        w.accept(&bytes(&f["packets"][key])).unwrap();
    }
    w
}
#[test]
fn all_eight_moves_send_source_and_predict_formal_destination() {
    for row in fixture()["moves"].as_array().unwrap() {
        let mut w = active();
        let h = row["heading"].as_u64().unwrap() as u8;
        assert_eq!(w.move_request(h).unwrap(), bytes(&row["body"]));
        w.predict_step(h).unwrap();
        let a = w.own_actor().unwrap();
        assert_eq!(
            [a.x, a.y],
            [
                row["destination"][0].as_u64().unwrap() as u16,
                row["destination"][1].as_u64().unwrap() as u16
            ]
        );
        assert!(a.position_predicted);
        assert_eq!(w.predicted_moves, 1);
    }
    assert!(native_world::destination(0, 0, 0).is_err());
    assert!(native_world::destination(65535, 65535, 3).is_err());
    let mut w = active();
    assert!(w.move_request(8).is_err());
    w.hp = Some([0, 200]);
    assert!(w.predict_step(0).is_err());
    assert_eq!(w.predicted_moves, 0);
}
#[test]
fn authoritative_correction_replaces_prediction_and_remove_retires_own_actor() {
    let f = fixture();
    let mut w = active();
    w.predict_step(0).unwrap();
    w.accept(&bytes(&f["packets"]["correction"])).unwrap();
    let a = w.own_actor().unwrap();
    assert_eq!((a.x, a.y, a.heading), (32720, 32780, 7));
    assert!(!a.position_predicted);
    assert_eq!(w.local_corrections, 1);
    w.accept(&bytes(&f["packets"]["heading"])).unwrap();
    assert_eq!(w.own_actor().unwrap().heading, 3);
    w.accept(&bytes(&f["packets"]["lock"])).unwrap();
    assert_eq!(w.movement_locks, 1);
    assert_eq!(bytes(&f["unlock"]), [0x5a]);
    w.accept(&bytes(&f["packets"]["remove"])).unwrap();
    assert!(!w.ready());
    assert!(w.move_request(0).is_err());
    w.accept(&bytes(&f["packets"]["correction"])).unwrap();
    assert_eq!(w.orphan_updates, 1);
    assert!(w.actors.is_empty());
}
#[test]
fn deployed_remote_movement_uses_source_cell_in_all_directions() {
    let f = fixture();
    let wf: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    for h in 0..8 {
        let mut w = active();
        w.accept(&bytes(&wf["packets"]["bot"])).unwrap();
        w.accept(&bytes(&f["packets"][format!("remote_{h}")]))
            .unwrap();
        let a = &w.actors[&2];
        assert_eq!((a.x, a.y, a.heading), (32720, 32780, h));
        let sources=[[32720,32781],[32719,32781],[32719,32780],[32719,32779],
            [32720,32779],[32721,32779],[32721,32780],[32721,32781]];
        assert_eq!(a.motion_from, Some(sources[h as usize]));
        assert_eq!(w.local_corrections, 0);
    }
    let mut w = active();
    let mut bad = bytes(&f["packets"]["correction"]);
    bad[5..9].fill(0); // heading NW would underflow the destination.
    let before = serde_json::to_value(&w).unwrap();
    assert!(w.accept(&bad).is_err());
    assert_eq!(serde_json::to_value(w).unwrap(), before);
}
#[test]
fn remote_correction_uses_received_source_instead_of_interpolating_from_old_position() {
    let wf:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w=active();w.accept(&bytes(&wf["packets"]["bot"])).unwrap();
    assert_eq!([w.actors[&2].x,w.actors[&2].y],[32721,32781]);
    let mut packet=vec![0x12];packet.extend_from_slice(&2u32.to_le_bytes());
    packet.extend_from_slice(&32740u16.to_le_bytes());packet.extend_from_slice(&32750u16.to_le_bytes());packet.push(2);
    assert!(w.accept(&packet).unwrap());let actor=&w.actors[&2];
    assert_eq!(actor.motion_from,Some([32740,32750]));assert_eq!([actor.x,actor.y],[32741,32750]);
}
#[test]
fn malformed_updates_never_partially_mutate_world() {
    for (key, value) in fixture()["packets"].as_object().unwrap() {
        let b = bytes(value);
        let mut invalid: Vec<Vec<u8>> = (0..b.len()).map(|n| b[..n].to_vec()).collect();
        let mut tail = b.clone();
        tail.push(0);
        invalid.push(tail);
        if key == "heading" || key == "correction" {
            let mut bad = b;
            *bad.last_mut().unwrap() = 8;
            invalid.push(bad);
        }
        for bad in invalid {
            let mut w = active();
            let before = serde_json::to_value(&w).unwrap();
            assert!(w.accept(&bad).is_err(), "{key} {bad:?}");
            assert_eq!(serde_json::to_value(w).unwrap(), before);
        }
    }
}
#[test]
fn old_render_cannot_publish_after_map_generation_or_connection_changes() {
    let a = Key {
        connection: 2,
        generation: 1,
        map: 564,
    };
    let mut f = Fence::default();
    assert!(f.select(Some(a)));
    assert!(f.accepts(a));
    let b = Key { generation: 2, ..a };
    assert!(f.select(Some(b)));
    assert!(!f.accepts(a));
    assert!(f.accepts(b));
    let c = Key { connection: 4, ..a };
    assert!(f.select(Some(c)));
    assert!(!f.accepts(b));
    assert!(!f.accepts(a));
    assert!(f.accepts(c));
    f.select(None);
    assert!(!f.accepts(c));
    let mut w = active();
    w.predict_step(0).unwrap();
    w.accept(&[0x28, 0x34, 2, 0]).unwrap();
    assert!(!w.ready());
    assert_eq!(w.predicted_moves, 0);
    assert!(w.move_request(0).is_err());
}
