use lito_modern::modern_lito::{
    native_ground_pickup::{self as pickup, Choice, Feedback, Pending},
    native_login::{LoginController, LoginPhase, Snapshot},
    native_world_scene::{Frame, Model},
};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
const ID: u32 = 0xf1234567;
fn packet(data: &[u8], key: &str) -> Vec<u8> {
    let j: serde_json::Value = serde_json::from_slice(data).unwrap();
    let s = j["packets"][key].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn p(key: &str) -> Vec<u8> {
    packet(
        include_bytes!("fixtures/native-ground-pickup-current.json"),
        key,
    )
}
fn state() -> Snapshot {
    let mut s = LoginController::default().snapshot().clone();
    s.phase = LoginPhase::BattlePending;
    s.game_connected = true;
    s.inventory.begin(2, "測試角色").unwrap();
    for key in ["world", "start", "status", "actor", "bot"] {
        s.world
            .accept(&packet(
                include_bytes!("fixtures/native-world-current.json"),
                key,
            ))
            .unwrap();
    }
    let mut own = s.world.actors.remove(&0xfedcba98).unwrap();
    own.id = 1;
    own.x = 32729;
    own.y = 32797;
    s.world.actors.insert(1, own);
    s.world.status.as_mut().unwrap().object_id = 1;
    s.world.hp = Some([50, 50]);
    s.world.accept(&p("ground")).unwrap();
    s
}
fn scene(s: &Snapshot) -> (Model, Frame) {
    let cancel = AtomicBool::new(false);
    let mut m = Model::load(Path::new(r"D:\lineage開發\0-客戶端"), &s.world, &cancel).unwrap();
    let f = m.draw_at(&s.world, &cancel, 0).unwrap();
    (m, f)
}
#[test]
fn formal_all_unsigned_wire_x_and_quantity_vectors() {
    let mut sha = Sha256::new();
    for x in 0..=u16::MAX {
        sha.update(pickup::wire_x(x).to_le_bytes());
    }
    let fixture: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-ground-pickup-current.json"))
            .unwrap();
    assert_eq!(
        format!("{:x}", sha.finalize()),
        fixture["wire_x_all_u16_sha256"]
    );
    let mut s = state();
    let (_, f) = scene(&s);
    let mut c = Choice::capture(&s, &f, ID).unwrap();
    assert_eq!(pickup::request(&s, &c, 2).unwrap(), p("request"));
    for n in [0, 5, u32::MAX] {
        assert!(pickup::request(&s, &c, n).is_err());
    }
    s.world.actors.remove(&ID);
    c.item.id = u32::MAX;
    c.item.x = u16::MAX;
    c.item.y = 0;
    c.item.experience = u32::MAX;
    s.world.actors.insert(c.item.id, c.item.clone());
    let own = s.world.actors.get_mut(&1).unwrap();
    own.x = u16::MAX;
    own.y = 0;
    assert_eq!(pickup::request(&s, &c, u32::MAX).unwrap(), p("boundary"));
}
#[test]
fn capture_requires_actual_type9_resource_and_fresh_render() {
    let mut s = state();
    let (_, f) = scene(&s);
    assert_eq!(f.ground_items.len(), 1);
    let c = Choice::capture(&s, &f, ID).unwrap();
    assert_eq!(c.item.graphics, 773);
    assert!(Choice::capture(&s, &f, 1).is_err());
    assert!(Choice::capture(&s, &f, 2).is_err());
    let before = serde_json::to_value(&s).unwrap();
    pickup::request(&s, &c, 1).unwrap();
    assert_eq!(serde_json::to_value(&s).unwrap(), before);
    s.world.accept(&p("partial")).unwrap();
    assert!(c.validate(&s).is_err());
    assert!(Choice::capture(&s, &f, ID).is_err());
    let (_, next) = scene(&s);
    let fresh = Choice::capture(&s, &next, ID).unwrap();
    assert_eq!(fresh.item.experience, 2);
    assert_ne!(fresh.item.incarnation, c.item.incarnation);
}
#[test]
fn stale_pickup_choice_never_rebinds_same_id_or_new_owner() {
    let s = state();
    let (_, f) = scene(&s);
    let c = Choice::capture(&s, &f, ID).unwrap();
    for n in 0..12 {
        let mut next = s.clone();
        match n {
            0 => next.inventory.owner.as_mut().unwrap().connection += 1,
            1 => next.inventory.owner.as_mut().unwrap().selection += 1,
            2 => next.world.generation += 1,
            3 => next.world.local_corrections += 1,
            4 => next.world.actors.get_mut(&1).unwrap().incarnation += 1,
            5 => next.world.actors.get_mut(&ID).unwrap().incarnation += 1,
            6 => next.world.actors.get_mut(&ID).unwrap().experience -= 1,
            7 => {
                next.world.actors.remove(&ID);
            }
            8 => next.world.hp = Some([0, 50]),
            9 => {
                next.world.movement_blocks.insert(2);
            }
            10 => next.phase = LoginPhase::LobbyReady,
            _ => next.world.actors.get_mut(&1).unwrap().x += 4,
        }
        assert!(pickup::request(&next, &c, 1).is_err(), "variant{n}");
    }
    let mut edge = s.clone();
    edge.world.actors.get_mut(&1).unwrap().x += 3;
    assert!(pickup::request(&edge, &c, 1).is_ok());
}
#[test]
fn split_ground_and_inventory_updates_are_observations_not_success_ack() {
    let s = state();
    let (_, f) = scene(&s);
    let c = Choice::capture(&s, &f, ID).unwrap();
    let now = Instant::now();
    let pending = Pending::new(&s, c.clone(), now);
    assert_eq!(pending.observe(&s, now), Feedback::Pending);
    assert_eq!(
        pending.observe(&s, now + Duration::from_secs(2)),
        Feedback::NoResponse
    );
    let mut ground = s.clone();
    ground.world.accept(&p("partial")).unwrap();
    assert_eq!(pending.observe(&ground, now), Feedback::Pending);
    assert_eq!(
        pending.observe(&ground, now + Duration::from_secs(2)),
        Feedback::NoResponse
    );
    ground.inventory.accept(2, &p("inventory")).unwrap();
    assert_eq!(pending.observe(&ground, now), Feedback::ObservedUpdates);
    let mut inventory = s.clone();
    inventory.inventory.accept(2, &p("inventory")).unwrap();
    assert_eq!(pending.observe(&inventory, now), Feedback::Pending);
    inventory.world.accept(&p("removed")).unwrap();
    assert_eq!(pending.observe(&inventory, now), Feedback::ObservedUpdates);
    inventory.world.generation += 1;
    assert_eq!(pending.observe(&inventory, now), Feedback::Interrupted);
    let mut dead = s.clone();
    dead.world.actors.get_mut(&1).unwrap().dead = true;
    assert_eq!(pending.observe(&dead, now), Feedback::Interrupted);
}
#[test]
fn formal_ground_replacement_and_removal_are_atomic() {
    let mut s = state();
    let before = serde_json::to_value(&s.world).unwrap();
    for key in ["partial", "removed"] {
        let b = p(key);
        for n in 0..b.len() {
            assert!(s.world.accept(&b[..n]).is_err());
            assert_eq!(serde_json::to_value(&s.world).unwrap(), before);
        }
    }
    s.world.accept(&p("partial")).unwrap();
    assert_eq!(s.world.actors[&ID].experience, 2);
    s.world.accept(&p("removed")).unwrap();
    assert!(!s.world.actors.contains_key(&ID));
    assert!(s.inventory.items.is_empty());
}
#[test]
fn formal_pickup_action15_uses_original_frames_then_returns_to_idle() {
    let mut s = state();
    let (mut model, _) = scene(&s);
    s.world.accept(&p("direction")).unwrap();
    s.world.accept(&p("action")).unwrap();
    let unchanged = serde_json::to_value(&s).unwrap();
    let mut rows = vec![];
    let mut seen = false;
    let mut idle = false;
    for ms in (0..=2500u64).step_by(100) {
        let f = model
            .draw_at(&s.world, &AtomicBool::new(false), ms * 1_000_000)
            .unwrap();
        let own = f.receipt["actors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == 1)
            .unwrap();
        assert_eq!(own["rendered"], true);
        if own["action"] == 15 {
            seen = true;
        }
        if seen && own["phase"] == "idle" {
            idle = true;
        }
        assert_eq!(f.ground_items.len(), 1);
        let mut r = f.receipt;
        r["ms"] = serde_json::json!(ms);
        rows.push(r);
        assert_eq!(serde_json::to_value(&s).unwrap(), unchanged);
    }
    assert!(seen && idle);
    assert_eq!(s.world.own_attack_packets, 0);
    assert_eq!(s.world.own_cast_packets, 0);
    std::fs::write(
        "target/current/verification/native-ground-pickup-action.json",
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
