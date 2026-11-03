use lito_modern::modern_lito::native_world::{self, World};
fn fixture() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap()
}
fn bytes(v: &serde_json::Value) -> Vec<u8> {
    let s = v.as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn active() -> World {
    let f = fixture();
    let mut w = World::default();
    w.accept(&bytes(&f["packets"]["world"])).unwrap();
    w.accept(&[0xd2]).unwrap();
    w
}
#[test]
fn clearing_authority_does_not_reuse_a_prior_render_generation() {
    let f=fixture();let mut w=active();
    for key in ["status","actor","hp","mp"] {w.accept(&bytes(&f["packets"][key])).unwrap();}
    let retired=w.generation;assert!(w.ready());
    w.clear_preserving_generation();
    assert!(!w.ready());assert!(w.actors.is_empty());assert!(w.hp.is_none());assert!(w.mp.is_none());
    w.accept(&bytes(&f["packets"]["world"])).unwrap();
    w.accept(&[0xd2]).unwrap();
    for key in ["status","actor","hp","mp"] {w.accept(&bytes(&f["packets"][key])).unwrap();}
    assert!(w.ready());assert!(w.generation>retired);
}
#[test]
fn deployed_world_fields_and_local_object_identity_are_preserved() {
    let f = fixture();
    let mut w = active();
    for key in [
        "status",
        "actor",
        "bot",
        "door",
        "limit",
        "time",
        "hp",
        "mp",
        "challenge",
    ] {
        assert!(w.accept(&bytes(&f["packets"][key])).unwrap(), "{key}");
    }
    assert!(w.ready());
    assert_eq!(w.map_id, Some(564));
    assert_eq!(w.own_actor().unwrap().id, 0xfedcba98);
    assert_eq!(w.actors.len(), 3);
    let a = w.own_actor().unwrap();
    assert_eq!(
        (a.x, a.y, a.heading, a.speed, a.lawful),
        (32720, 32780, 7, 2, -32768)
    );
    assert_eq!(a.name, "測試角色");
    assert_eq!(a.clan, "測試盟");
    assert_eq!(a.status_tail, Some([255, 0, 0, 0, 0]));
    assert_eq!(w.actors[&2].status_tail, None);
    assert_eq!(w.status.unwrap().experience, u32::MAX);
    assert_eq!(w.hp, Some([100, 200]));
    assert_eq!(w.mp, Some([50, 90]));
    assert_eq!(w.time_limit, Some(300));
    assert!(w.actors[&0x62000000].name.is_empty());
    assert_eq!(w.actors[&0x62000000].walk_action, 29);
}
#[test]
fn known_world_packets_reject_every_truncation_and_extra_tail() {
    let f = fixture();
    for (key, value) in f["packets"].as_object().unwrap() {
        let b = bytes(value);
        for n in 0..b.len() {
            let mut w = active();
            let result = w.accept(&b[..n]);
            assert!(
                result.is_err() || matches!(result, Ok(false)),
                "{key} accepted partial {n}"
            );
        }
        let mut tail = b;
        tail.push(0);
        let mut w = active();
        assert!(w.accept(&tail).is_err(), "{key} accepted tail");
    }
}
#[test]
fn new_world_clears_old_actors_status_and_challenges() {
    let f = fixture();
    let mut w = active();
    for key in ["status", "actor", "challenge"] {
        w.accept(&bytes(&f["packets"][key])).unwrap();
    }
    assert!(w.ready());
    w.accept(&[0x28, 5, 0, 0]).unwrap();
    assert_eq!(w.generation, 2);
    assert!(!w.ready());
    assert!(w.actors.is_empty() && w.status.is_none() && w.integrity_challenge.is_none());
    assert!(w.accept(&[0xd2]).unwrap());
    assert!(w.accept(&[0xd2]).is_err());
    assert!(!w.accept(&[0xfe, 0]).unwrap());
    assert!(w.unhandled_opcodes.contains(&0xfe));
}
#[test]
fn world_actor_updates_use_id_and_do_not_require_display_names() {
    let f = fixture();
    let mut w = active();
    let mut b = bytes(&f["packets"]["actor"]);
    w.accept(&b).unwrap();
    b[1..3].copy_from_slice(&65535u16.to_le_bytes());
    w.accept(&b).unwrap();
    assert_eq!(w.actors.len(), 1);
    assert_eq!(w.actors[&0xfedcba98].x, 65535);
    b[12] = 8;
    assert!(native_world::actor(&b).is_err());
}
