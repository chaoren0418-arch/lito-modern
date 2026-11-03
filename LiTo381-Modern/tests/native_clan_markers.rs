use lito_modern::modern_lito::{
    compositor::Canvas,
    native_clan_markers::{Bank, Clock, Runtime},
    native_world::World,
    native_world_scene::Model,
    world_map::PixelPoint,
};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path, sync::atomic::AtomicBool};
fn bytes(v: &serde_json::Value) -> Vec<u8> {
    let h = v.as_str().unwrap();
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
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
fn root() -> &'static Path {
    Path::new(r"D:\lineage開發\0-客戶端")
}
fn output() -> std::path::PathBuf {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    std::fs::create_dir_all(&p).unwrap();
    p
}
fn png(path: &Path, c: &Canvas) {
    let mut e = png::Encoder::new(std::fs::File::create(path).unwrap(), c.width(), c.height());
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header()
        .unwrap()
        .write_image_data(c.rgba())
        .unwrap();
}
#[test]
fn shared_clock_draws_then_advances_with_native_threshold_and_modern_epoch() {
    let mut w = world();
    let own = w.own_actor().unwrap().id;
    w.actors.get_mut(&2).unwrap().clan_marker_active = true;
    w.actors.get_mut(&own).unwrap().clan_marker_active = true;
    let centers = BTreeMap::from([(2, PixelPoint { x: 400, y: 300 })]);
    let mut c = Clock::default();
    for (ms, phase) in [
        (0, 0),
        (349, 0),
        (350, 0),
        (350, 1),
        (700, 1),
        (700, 2),
        (50_000, 2),
        (50_000, 3),
    ] {
        let rows = c.sample(&w, &centers, ms * 1_000_000);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].phase, phase);
        assert_eq!(rows[0].anchor, Some([413, 238]));
    }
    // Hidden/offscreen absence still consumes a single shared delta.
    assert_eq!(c.sample(&w, &BTreeMap::new(), 50_350_000_000)[0].phase, 3);
    assert_eq!(c.sample(&w, &centers, 50_350_000_000)[0].phase, 4);
    w.generation += 1;
    assert_eq!(c.sample(&w, &centers, 50_350_000_000)[0].phase, 0);
    let mut second = w.actors[&2].clone();
    second.id = 3;
    second.incarnation += 1;
    w.actors.insert(3, second);
    let r = c.sample(&w, &centers, 50_700_000_000);
    assert_eq!(
        r.iter().map(|r| (r.id, r.phase)).collect::<Vec<_>>(),
        vec![(2, 0), (3, 1)]
    );
    assert_eq!(c.sample(&w, &centers, 50_700_000_000)[0].phase, 2);
    // No carry after rewind, ownership change, or local actor replacement.
    assert_eq!(c.sample(&w, &centers, 0)[0].phase, 0);
    w.actors.get_mut(&own).unwrap().incarnation += 1;
    assert_eq!(c.sample(&w, &centers, 1_000_000_000)[0].phase, 0);
    w.actors.get_mut(&3).unwrap().name = w.own_actor().unwrap().name.to_ascii_uppercase();
    assert_eq!(c.sample(&w, &centers, 1_000_000_000).len(), 1);
    w.actors.get_mut(&2).unwrap().clan_marker_active = false;
    assert!(c.sample(&w, &centers, 2_000_000_000).is_empty());
}
#[test]
fn seven_original_images_match_scalar_pixels_with_clipping_and_cancellation() {
    let cancel = AtomicBool::new(false);
    let b = Bank::load(root(), &cancel).unwrap();
    let fixture: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-clan-markers-current.json"))
            .unwrap();
    let mut cases = Vec::new();
    for (phase, img) in b.images.iter().enumerate() {
        assert_eq!(
            format!("{:X}", Sha256::digest(&img.rgba)),
            fixture["cases"][phase]["rgba_sha256"]
        );
        assert_eq!(
            b.sources["members"][phase]["sha256"],
            fixture["cases"][phase]["sha256"]
        );
        for (placement, (x, y)) in [(20, 12), (-10, -7), (58, 42)].into_iter().enumerate() {
            let mut c = Canvas::new(64, 48).unwrap();
            c.clear([24, 48, 72]);
            let mut expected = c.rgba().to_vec();
            for sy in 0..18 {
                for sx in 0..24 {
                    let (dx, dy) = (sx + x, sy + y);
                    let from = (sy * 24 + sx) as usize * 4;
                    if (0..64).contains(&dx) && (0..48).contains(&dy) && img.rgba[from + 3] != 0 {
                        let to = (dy * 64 + dx) as usize * 4;
                        expected[to..to + 4].copy_from_slice(&img.rgba[from..from + 4]);
                    }
                }
            }
            b.blit(&mut c, phase as u8, PixelPoint { x, y }).unwrap();
            assert_eq!(c.rgba(), expected);
            let path = output().join(format!("native-clan-marker-{phase}-{placement}.png"));
            png(&path, &c);
            cases.push(serde_json::json!({"phase":phase,"placement":placement,"anchor":[x,y],"png":path,"rgba_sha256":format!("{:X}",Sha256::digest(c.rgba()))}));
        }
    }
    assert!(Bank::load(root(), &AtomicBool::new(true)).is_err());
    let mut c = Canvas::new(8, 8).unwrap();
    assert!(b.blit(&mut c, 7, PixelPoint { x: 0, y: 0 }).is_err());
    std::fs::write(
        output().join("native-clan-markers.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"sources":b.sources,"cases":cases})).unwrap(),
    )
    .unwrap();
}
#[test]
fn metadata_marker_overlay_keeps_authority_and_all_pixel_targets() {
    let mut w = world();
    let cancel = AtomicBool::new(false);
    let mut model = Model::load(root(), &w, &cancel).unwrap();
    let before = model.draw_at(&w, &cancel, 0).unwrap();
    let f: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-label-updates-current.json"))
            .unwrap();
    let mut packet = bytes(&f["packets"]["clan"]);
    w.accept(&packet).unwrap();
    let authority = serde_json::to_value(&w).unwrap();
    let after = model.draw_at(&w, &cancel, 0).unwrap();
    assert_eq!(authority, serde_json::to_value(&w).unwrap());
    assert_ne!(before.canvas.rgba(), after.canvas.rgba());
    assert_eq!(before.hits, after.hits);
    for y in 0..600 {
        for x in 0..800 {
            assert_eq!(before.pick(x, y, 0), after.pick(x, y, 0));
        }
    }
    let rows = after.receipt["clan_markers"]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["drawn"], true);
    assert_eq!(rows[0]["marker"]["phase"], 0);
    png(
        &output().join("native-clan-marker-world.png"),
        &after.canvas,
    );
    std::fs::write(
        output().join("native-clan-marker-world.json"),
        serde_json::to_vec_pretty(&after.receipt).unwrap(),
    )
    .unwrap();
    let mut hidden = vec![0x34];
    hidden.extend(2u32.to_le_bytes());
    hidden.push(1);
    w.accept(&hidden).unwrap();
    let frame = model.draw_at(&w, &cancel, 0).unwrap();
    assert_eq!(frame.receipt["clan_markers"]["rows"][0]["drawn"], false);
    let n = packet.len();
    packet[n - 4..].fill(0);
    w.accept(&packet).unwrap();
    assert!(
        model.draw_at(&w, &cancel, 0).unwrap().receipt["clan_markers"]["rows"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    // Cancellation does not commit clock/assets or modify the supplied canvas.
    let mut rt = Runtime::default();
    let mut canvas = Canvas::new(64, 48).unwrap();
    let pixels = canvas.rgba().to_vec();
    assert!(rt
        .draw(
            root(),
            &w,
            &BTreeMap::new(),
            &mut canvas,
            0,
            &AtomicBool::new(true)
        )
        .is_err());
    assert_eq!(canvas.rgba(), pixels);
}
