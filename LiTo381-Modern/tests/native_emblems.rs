use lito_modern::modern_lito::{
    compositor::Canvas, native_emblems::*, native_world::World, native_world_labels::Line,
};
use std::{
    collections::BTreeSet,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn fixture() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("fixtures/native-emblems-current.json")).unwrap()
}
fn packet(id: u32, word: u16) -> Vec<u8> {
    let mut v = vec![0x35];
    v.extend(id.to_le_bytes());
    for _ in 0..192 {
        v.extend(word.to_le_bytes());
    }
    v
}
#[test]
fn deployed_wire_unsigned_ids_and_exact_envelope() {
    let f = fixture();
    assert_eq!(f["invalid_requests_rejected"], 4);
    for c in f["cases"].as_array().unwrap() {
        let id = c["id"].as_u64().unwrap() as u32;
        assert_eq!(request(id).unwrap(), bytes(c["request"].as_str().unwrap()));
        let body = bytes(c["response"].as_str().unwrap());
        let im = decode(&body).unwrap();
        assert_eq!(im.id, id);
        assert_eq!(
            &im.rgba[..32],
            &[
                0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 255,
                0, 255, 0, 255, 0, 0, 255, 255, 132, 132, 132, 255
            ]
        );
        for n in 0..body.len() {
            assert!(decode(&body[..n]).is_err());
        }
        let mut extra = body.clone();
        extra.push(0);
        assert!(decode(&extra).is_err());
    }
    assert!(request(0).is_err());
    assert!(decode(&packet(0, 1)).is_err());
}
#[test]
fn all_native_words_preserve_zero_key_and_high_bit_coverage() {
    for word in 0..=u16::MAX {
        let im = decode(&packet(1, word)).unwrap();
        let p = &im.rgba[..4];
        for (i, shift) in [10, 5, 0].into_iter().enumerate() {
            let n = ((word >> shift) & 31) as u32;
            assert_eq!(p[i], (n * 8 + n / 4) as u8);
        }
        assert_eq!(p[3], if word == 0 { 0 } else { 255 });
    }
}
#[test]
fn bounded_requests_silent_miss_late_response_and_atomic_rejection() {
    let now = Instant::now();
    let wanted = (1..=600).collect::<BTreeSet<_>>();
    let mut c = Cache::default();
    for _ in 0..PENDING_LIMIT {
        let id = c.next_request(&wanted, now).unwrap();
        c.mark_sent(id, now).unwrap();
    }
    assert!(c.next_request(&wanted, now).is_none());
    assert_eq!(c.pending(), 32);
    assert!(c.mark_sent(1, now).is_err());
    for tick in 1..=16 {
        let t = now + Duration::from_secs(tick * 6);
        for _ in 0..32 {
            if let Some(id) = c.next_request(&wanted, t) {
                c.mark_sent(id, t).unwrap();
            }
        }
    }
    assert_eq!(c.len(), 512);
    assert_eq!(c.requests, 512);
    assert!(c
        .next_request(&wanted, now + Duration::from_secs(200))
        .is_none());
    let old = serde_json::to_value(&c).unwrap();
    assert!(c.accept(&packet(1, 0x7c00)[..20]).is_err());
    assert_eq!(old, serde_json::to_value(&c).unwrap());
    assert!(c.accept(&packet(1, 0x7c00)).unwrap());
    assert!(c.image(1).is_some());
    assert!(!c.accept(&packet(1, 0x001f)).unwrap());
    assert!(!c.accept(&packet(999, 1)).unwrap());
    let next = BTreeSet::from([600]);
    assert_eq!(
        c.next_request(&next, now + Duration::from_secs(201)),
        Some(600)
    );
    c.mark_sent(600, now).unwrap();
    assert_eq!(c.len(), 512);
    assert_eq!(c.evictions, 1);
}
fn world() -> World {
    let f: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for k in ["world", "start", "status", "actor", "bot", "hp", "mp"] {
        w.accept(&bytes(f["packets"][k].as_str().unwrap())).unwrap();
    }
    w
}
#[test]
fn world_reset_and_current_actor_id_fence_rendering() {
    let mut w = world();
    let id = w.own_actor().unwrap().id;
    w.actors.get_mut(&id).unwrap().emblem = 7;
    w.emblems.mark_sent(7, Instant::now()).unwrap();
    let before = w.visual_revision;
    w.accept(&packet(7, 0x8000)).unwrap();
    assert_eq!(w.visual_revision, before + 1);
    let lines = vec![Line {
        id,
        kind: "own",
        text: "name".into(),
        x: 20,
        y: 1,
        color: 0x7fff,
    }];
    let token = AtomicBool::new(false);
    let mut canvas = Canvas::new(24, 16).unwrap();
    let r = draw(&w.emblems, &w, &lines, &mut canvas, &token).unwrap();
    assert_eq!(r["rows"].as_array().unwrap().len(), 1);
    assert_eq!(
        &canvas.rgba()[(24 + 2) * 4..(24 + 2) * 4 + 4],
        &[0, 0, 0, 255]
    );
    w.actors.get_mut(&id).unwrap().emblem = 8;
    assert!(
        draw(&w.emblems, &w, &lines, &mut canvas, &token).unwrap()["rows"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    w.accept(&[0x28, 0x34, 2, 0]).unwrap();
    assert_eq!(w.emblems.len(), 0);
    w.accept(&packet(7, 1)).unwrap();
    assert_eq!(w.emblems.len(), 0);
}
