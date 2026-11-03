use lito_modern::modern_lito::{compositor::Canvas, native_world_text::Cache};
use sha2::{Digest, Sha256};
use std::sync::atomic::AtomicBool;
#[test]
fn actual_cp950_font_raster_is_cached_bounded_and_cancellable() {
    let c = AtomicBool::new(false);
    let mut cache = Cache::default();
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    let mut rows = vec![];
    for (i, text) in ["測試角色", "治癒藥水 (2)", "Magic Gem (4)", "LiTo 玩家名稱"]
        .into_iter()
        .enumerate()
    {
        let mask = cache.get(text, &c).unwrap().clone();
        assert!(mask.coverage.iter().any(|b| *b > 0));
        assert!(!mask.face.is_empty());
        assert_eq!(mask.coverage, cache.get(text, &c).unwrap().coverage);
        let mut canvas = Canvas::new(240, 40).unwrap();
        canvas.clear([58, 68, 78]);
        mask.draw(&mut canvas, 12, 12, 0x7fff, 0).unwrap();
        let path = out.join(format!("native-world-text-{i}.png"));
        let mut e = png::Encoder::new(std::fs::File::create(&path).unwrap(), 240, 40);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()
            .unwrap()
            .write_image_data(canvas.rgba())
            .unwrap();
        rows.push(serde_json::json!({"text":text,"face":mask.face,"advance":mask.advance,"mask_sha256":format!("{:x}",Sha256::digest(&mask.coverage)),"rgba_sha256":format!("{:x}",Sha256::digest(canvas.rgba())),"png":path}));
    }
    assert_eq!(cache.len(), 4);
    assert!(cache.get("測試角色", &AtomicBool::new(true)).is_err());
    for bad in [
        String::new(),
        "abc\n".into(),
        "玩家😀".into(),
        "x".repeat(769),
        "x".repeat(257),
    ] {
        assert!(cache.get(&bad, &c).is_err());
    }
    for i in 0..260 {
        cache.get(&format!("A{i}"), &c).unwrap();
    }
    assert!(cache.len() <= 256 && cache.bytes() <= 4 << 20);
    std::fs::write(out.join("native-world-text-reference.json"),serde_json::to_vec_pretty(&serde_json::json!({"cases":rows,"host_font_mapping":true,"gui_tested":false,"full_native_text_equivalence":false})).unwrap()).unwrap();
}
#[test]
fn outline_offsets_rgb555_color_and_clipping_match_scalar_composition() {
    use lito_modern::modern_lito::native_world_text::GlyphMask;
    let m = GlyphMask {
        width: 3,
        height: 3,
        advance: 3,
        face: "fixture".into(),
        coverage: vec![0, 255, 0, 255, 128, 255, 0, 255, 0],
    };
    for (x, y) in [(2, 2), (-2, -1), (5, 6)] {
        let mut c = Canvas::new(7, 7).unwrap();
        c.clear([50, 60, 70]);
        m.draw(&mut c, x, y, 0x35a7, 0x0843).unwrap();
        let mut expected = [50, 60, 70, 255].repeat(49);
        for (dx, dy, color) in [
            (-1, 0, 0x0843u16),
            (1, 0, 0x0843),
            (0, -1, 0x0843),
            (0, 1, 0x0843),
            (0, 0, 0x35a7),
        ] {
            let rgb = [
                ((color >> 10) & 31) * 8,
                ((color >> 5) & 31) * 8,
                (color & 31) * 8,
            ];
            for sy in 0..3 {
                for sx in 0..3 {
                    let xx = x + dx + sx;
                    let yy = y + dy + sy;
                    if !(0..7).contains(&xx) || !(0..7).contains(&yy) {
                        continue;
                    }
                    let a = u32::from(m.coverage[(sy * 3 + sx) as usize]);
                    let at = ((yy * 7 + xx) * 4) as usize;
                    for k in 0..3 {
                        expected[at + k] = ((u32::from(rgb[k]) * a
                            + u32::from(expected[at + k]) * (255 - a)
                            + 127)
                            / 255) as u8;
                    }
                }
            }
        }
        assert_eq!(c.rgba(), expected);
    }
}
