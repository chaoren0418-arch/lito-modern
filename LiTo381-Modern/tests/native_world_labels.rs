use lito_modern::modern_lito::{
    native_inventory_names::Catalog,
    native_world::World,
    native_world_labels::{self, Hover},
    native_world_scene::Model,
};
use std::{path::Path, sync::atomic::AtomicBool};
fn world() -> World {
    let f: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for k in ["world", "start", "status", "actor", "bot", "hp", "mp"] {
        let h = f["packets"][k].as_str().unwrap();
        let b: Vec<_> = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect();
        w.accept(&b).unwrap();
    }
    w
}
#[test]
fn lawful_palette_boundaries_and_hover_identity() {
    for (v, c) in [
        (i16::MIN, 0x7c00),
        (-1, 0x7fde),
        (0, 0x7fff),
        (499, 0x7fff),
        (500, 0x7bdf),
        (i16::MAX, 0x1f),
    ] {
        assert_eq!(native_world_labels::lawful_color(v), c);
    }
    let mut w = world();
    let h = Hover::capture(&w, 2).unwrap();
    w.actors.get_mut(&2).unwrap().dead = true;
    assert_eq!(h.resolve(&w), Some(2)); // corpses still have names
    let mut p = vec![0x34];
    p.extend(2u32.to_le_bytes());
    p.push(1);
    w.accept(&p).unwrap();
    assert_eq!(h.resolve(&w), None);
    p[5] = 0;
    w.accept(&p).unwrap();
    assert_eq!(h.resolve(&w), None);
    assert!(Hover::capture(&w, 2).is_some());
    w.generation += 1;
    assert_eq!(h.resolve(&w), None);
}
#[test]
fn cp950_layout_uses_body_bounds_and_native_prefix_rules() {
    let mut w = world();
    let own = w.own_actor().unwrap().id;
    let c = Catalog::from_tables(b"1\r\nzero", b"1\r\nzero").unwrap();
    {
        let a = w.actors.get_mut(&own).unwrap();
        a.name = "測試角色".into();
        a.emblem = 0;
    }
    {
        let a = w.actors.get_mut(&2).unwrap();
        a.name = "+7 Sword".into();
        a.title = "稱號".into();
        a.tail_byte = 1;
    }
    let h = Hover::capture(&w, 2);
    let hits = [(2, [700, 20, 720, 50]), (own, [390, 200, 430, 260])];
    let rows = native_world_labels::layout(&w, &c, &hits, 424, h);
    assert_eq!(rows.len(), 3);
    assert_eq!(
        (&*rows[0].text, rows[0].x, rows[0].y, rows[0].color),
        ("測試角色", 376, 183, 0x7c00)
    );
    assert_eq!((rows[1].kind, rows[1].y), ("title", 48));
    assert_eq!(
        (&*rows[2].text, rows[2].y, rows[2].color),
        (" Sword", 61, 31)
    );
    assert_eq!(
        native_world_labels::layout(&w, &c, &hits, 424, None).len(),
        1
    );
    w.map_id = Some(500);
    assert!(native_world_labels::layout(&w, &c, &hits, 424, None).is_empty());
}
#[test]
fn real_world_labels_are_click_through_and_do_not_mutate_authority() {
    let w = world();
    let before = serde_json::to_value(&w).unwrap();
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel).unwrap();
    let plain = model.draw_at(&w, &cancel, 0).unwrap();
    let hover = model
        .draw_at_labels(&w, &cancel, 0, Hover::capture(&w, 2))
        .unwrap();
    assert_eq!(before, serde_json::to_value(&w).unwrap());
    assert_eq!(plain.hits, hover.hits);
    for y in 0..600 {
        for x in 0..800 {
            assert_eq!(plain.pick(x, y, 0), hover.pick(x, y, 0));
        }
    }
    assert_eq!(
        plain.receipt["world_labels"]["lines"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        hover.receipt["world_labels"]["lines"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_ne!(plain.canvas.rgba(), hover.canvas.rgba());
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    for (name, frame) in [("own", plain), ("hover", hover)] {
        let mut encoder = png::Encoder::new(
            std::fs::File::create(out.join(format!("native-world-labels-{name}.png"))).unwrap(),
            800,
            600,
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(frame.canvas.rgba())
            .unwrap();
        std::fs::write(
            out.join(format!("native-world-labels-{name}.json")),
            serde_json::to_vec_pretty(&frame.receipt).unwrap(),
        )
        .unwrap();
    }
}
