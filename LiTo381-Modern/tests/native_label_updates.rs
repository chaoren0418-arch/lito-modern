use lito_modern::modern_lito::{
    native_skill_selection::Target,
    native_world::World,
    native_world_labels::{self, Hover},
    native_world_scene::Model,
};
use std::{path::Path, sync::atomic::AtomicBool};
fn bytes(v: &serde_json::Value) -> Vec<u8> {
    let h = v.as_str().unwrap();
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
fn packets() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("fixtures/native-label-updates-current.json")).unwrap()
}
fn world() -> World {
    let f: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for k in ["world", "start", "status", "actor", "bot", "hp", "mp"] {
        w.accept(&bytes(&f["packets"][k])).unwrap();
    }
    w
}
#[test]
fn formal_updates_follow_native_field_assignments_and_preserve_selection() {
    let p = packets();
    let mut w = world();
    let old = w.actors[&2].clone();
    let h = Hover::capture(&w, 2).unwrap();
    let t = Target::capture(&w, 2).unwrap();
    for (key, title, clan, emblem, active) in [
        ("title", "稱號更新", "", 0, false),
        ("pledge", "盟員稱號", "測試盟", u32::MAX, false),
        ("clan", "新血盟", "測試盟", 0x80000000, true),
        ("title_empty", "", "測試盟", 0x80000000, true),
        ("pledge_empty", "", "", 0, true),
        ("clan_empty", "", "", 0, false),
    ] {
        w.accept(&bytes(&p["packets"][key])).unwrap();
        let a = &w.actors[&2];
        assert_eq!(
            (&*a.title, &*a.clan, a.emblem, a.clan_marker_active),
            (title, clan, emblem, active)
        );
        assert_eq!(
            (
                a.incarnation,
                a.target_revision,
                a.motion_revision,
                a.action_sequence
            ),
            (
                old.incarnation,
                old.target_revision,
                old.motion_revision,
                old.action_sequence
            )
        );
        assert_eq!(h.resolve(&w), Some(2));
        assert_eq!(t.resolve(&w), Some(2));
    }
    w.accept(&bytes(&p["packets"]["title_max"])).unwrap();
    assert_eq!(w.own_actor().unwrap().title, "一二三四五六七八");
    let v = w.visual_revision;
    w.accept(&bytes(&p["packets"]["clan_empty"])).unwrap();
    assert_eq!(w.visual_revision, v);
    assert_eq!(w.hp, Some([100, 200]));
    assert_eq!(w.mp, Some([50, 90]));
}
#[test]
fn wire_bounds_truncation_and_orphans_are_atomic() {
    let p = packets();
    let mut w = world();
    for (key, title) in [
        ("clan_long", "1234567890123456"),
        ("clan_wide", "一二三四五六七八"),
    ] {
        w.accept(&bytes(&p["packets"][key])).unwrap();
        assert_eq!(w.actors[&2].title, title);
    }
    let before = serde_json::to_value(&w).unwrap();
    assert!(w.accept(&bytes(&p["packets"]["clan_split"])).is_err());
    assert_eq!(before, serde_json::to_value(&w).unwrap());
    for key in ["title", "pledge", "clan"] {
        let b = bytes(&p["packets"][key]);
        for n in 1..b.len() {
            assert!(w.accept(&b[..n]).is_err());
            assert_eq!(before, serde_json::to_value(&w).unwrap());
        }
        let mut tail = b.clone();
        tail.push(0);
        assert!(w.accept(&tail).is_err());
        assert_eq!(before, serde_json::to_value(&w).unwrap());
        let mut zero = b.clone();
        zero[1..5].fill(0);
        assert!(w.accept(&zero).is_err());
        assert_eq!(before, serde_json::to_value(&w).unwrap());
    }
    let mut nonzero = bytes(&p["packets"]["clan"]);
    let len = nonzero.len();
    nonzero[len - 4..].copy_from_slice(&u32::MAX.to_le_bytes());
    w.accept(&nonzero).unwrap();
    assert!(w.actors[&2].clan_marker_active);
    let mut orphan = bytes(&p["packets"]["title"]);
    orphan[1..5].copy_from_slice(&99u32.to_le_bytes());
    let v = w.visual_revision;
    w.accept(&orphan).unwrap();
    assert_eq!(w.orphan_updates, 1);
    assert_eq!(w.visual_revision, v);
    assert!(!w.actors.contains_key(&99));
    let f: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    w.accept(&bytes(&f["packets"]["bot"])).unwrap();
    assert!(!w.actors[&2].clan_marker_active);
    assert!(w.actors[&2].title.is_empty());
}
#[test]
fn received_title_reflows_existing_hover_without_changing_hit_surface() {
    let mut w = world();
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel).unwrap();
    let h = Hover::capture(&w, 2);
    let before = model.draw_at_labels(&w, &cancel, 0, h).unwrap();
    let p = packets();
    w.accept(&bytes(&p["packets"]["title"])).unwrap();
    assert!(!before.input_ready(&w));
    let after = model.draw_at_labels(&w, &cancel, 0, h).unwrap();
    assert_eq!(before.hits, after.hits);
    for y in 0..600 {
        for x in 0..800 {
            assert_eq!(before.pick(x, y, 0), after.pick(x, y, 0));
        }
    }
    let rows = after.receipt["world_labels"]["lines"].as_array().unwrap();
    let title = rows.iter().find(|r| r["line"]["kind"] == "title").unwrap();
    let name = rows.iter().find(|r| r["line"]["kind"] == "hover").unwrap();
    assert_eq!(title["line"]["text"], "稱號更新");
    assert_eq!(
        name["line"]["y"].as_i64().unwrap() - title["line"]["y"].as_i64().unwrap(),
        13
    );
    assert_eq!(title["line"]["color"], native_world_labels::lawful_color(0));
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    let mut encoder = png::Encoder::new(
        std::fs::File::create(out.join("native-label-update-title.png")).unwrap(),
        800,
        600,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(after.canvas.rgba())
        .unwrap();
    std::fs::write(
        out.join("native-label-update-title.json"),
        serde_json::to_vec_pretty(&after.receipt).unwrap(),
    )
    .unwrap();
}
