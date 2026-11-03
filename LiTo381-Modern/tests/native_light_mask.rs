use lito_modern::modern_lito::native_light_mask::{
    Mask, Source, ORIGINAL_HEIGHT, ORIGINAL_WIDTH, SOURCE_SHA256,
};
use sha2::{Digest, Sha256};
use std::sync::atomic::AtomicBool;

const LMP: &[u8] = include_bytes!("fixtures/native-light-source.lmp");
const VECTORS: &[u8] = include_bytes!("fixtures/native-light-mask-original.json");

#[test]
fn every_original_compositor_case_matches_the_executed_native_mask_digest() {
    let source = Source::parse(LMP).unwrap();
    let vectors: serde_json::Value = serde_json::from_slice(VECTORS).unwrap();
    let cases = vectors["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 330);
    assert_eq!(format!("{:x}", Sha256::digest(LMP)), SOURCE_SHA256);
    for (case, v) in cases.iter().enumerate() {
        let mut mask = Mask::new(ORIGINAL_WIDTH, ORIGINAL_HEIGHT).unwrap();
        if !v["clear"].as_bool().unwrap() {
            let seed = v["seed"].as_u64().unwrap() as usize;
            let initial: Vec<_> = (0..ORIGINAL_WIDTH * ORIGINAL_HEIGHT)
                .map(|i| ((i * 17 + seed) & 255) as u8)
                .collect();
            mask.compose_plane(&initial, ORIGINAL_WIDTH, ORIGINAL_HEIGHT, 0, 0)
                .unwrap();
        }
        for op in v["ops"].as_array().unwrap() {
            let x = op["x"].as_i64().unwrap() as i32;
            let y = op["y"].as_i64().unwrap() as i32;
            if let Some(range) = op["slot"].as_u64() {
                mask.compose(source.stencil(range as u8).unwrap(), x, y);
            } else {
                let width = op["w"].as_i64().unwrap().max(0) as usize;
                let height = op["h"].as_i64().unwrap().max(0) as usize;
                let pixels: Vec<_> = if op["kind"] == "pairs" {
                    (0..width * height).map(|i| (i / 256) as u8).collect()
                } else {
                    vec![op["value"].as_u64().unwrap() as u8; width * height]
                };
                mask.compose_plane(&pixels, width, height, x, y).unwrap();
            }
        }
        assert_eq!(
            format!("{:X}", Sha256::digest(mask.pixels())),
            v["original_native_sha256"].as_str().unwrap(),
            "native case {case}"
        );
        mask.clear();
        assert!(mask.pixels().iter().all(|b| *b == 0));
    }
}

#[test]
fn source_header_preserves_shared_ranges_empty_slot_and_exact_original_pixels() {
    let s = Source::parse(LMP).unwrap();
    assert_eq!(s.sha256(), <[u8; 32]>::from(Sha256::digest(LMP)));
    let dims = [
        (16, 8),
        (16, 8),
        (32, 16),
        (32, 16),
        (72, 36),
        (72, 36),
        (144, 72),
        (144, 72),
        (200, 100),
        (200, 100),
        (240, 120),
        (240, 120),
        (288, 144),
        (288, 144),
        (312, 156),
        (0, 0),
    ];
    for (i, (w, h)) in dims.into_iter().enumerate() {
        let t = s.stencil(i as u8).unwrap();
        assert_eq!((t.width(), t.height(), t.pixels().len()), (w, h, w * h));
        if i % 2 == 0 && i < 14 {
            assert_eq!(
                t.pixels().as_ptr(),
                s.stencil(i as u8 + 1).unwrap().pixels().as_ptr()
            );
        }
    }
    assert!(s.stencil(16).is_err());
    assert!(s.stencil(255).is_err());
    for len in [0, 63, 64, 191, LMP.len() - 1] {
        assert!(Source::parse(&LMP[..len]).is_err());
    }
    for (at, value) in [
        (64, -1i32),
        (68, -1),
        (64, 321),
        (68, 205),
        (68, 0),
        (0, 1),
        (0, i32::MAX),
    ] {
        let mut bad = LMP.to_vec();
        bad[at..at + 4].copy_from_slice(&value.to_le_bytes());
        assert!(
            Source::parse(&bad).is_err(),
            "malformed header at {at} = {value}"
        );
    }
    assert!(Source::parse(&vec![0; (1 << 20) + 1]).is_err());
}

#[test]
fn mask_uses_max_instead_of_sum_and_rejects_invalid_input_before_mutation() {
    let mut m = Mask::new(2, 2).unwrap();
    m.compose_plane(&[128, 200, 255, 0], 2, 2, 0, 0).unwrap();
    m.compose_plane(&[129, 100, 10, 8], 2, 2, 0, 0).unwrap();
    assert_eq!(m.pixels(), [129, 200, 255, 8]);
    for (p, w, h) in [
        (&[7u8][..], 2, 2),
        (&[][..], usize::MAX, 1),
        (&[][..], 1, usize::MAX),
    ] {
        assert!(m.compose_plane(p, w, h, 0, 0).is_err());
        assert_eq!(m.pixels(), [129, 200, 255, 8]);
    }
    for (x, y) in [(i32::MIN, i32::MIN), (i32::MAX, i32::MAX)] {
        m.compose_plane(&[255; 4], 2, 2, x, y).unwrap();
        assert_eq!(m.pixels(), [129, 200, 255, 8]);
    }
    for (w, h) in [(0, 1), (1, 0), (2049, 1), (usize::MAX, 1)] {
        assert!(Mask::new(w, h).is_err());
    }
    assert!(Mask::for_pixels(usize::MAX, 1).is_err());
    let dynamic = Mask::for_pixels(801, 601).unwrap();
    assert_eq!(
        (dynamic.width(), dynamic.height(), dynamic.pixels().len()),
        (401, 301, 120701)
    );
}

#[test]
fn original_center_alignment_truncates_negative_halves_then_clears_x_low_bits() {
    let s = Source::parse(LMP).unwrap();
    for (cx, cy) in [
        (-1, -1),
        (15, 7),
        (16, 8),
        (17, 9),
        (320, 204),
        (i32::MIN, i32::MAX),
    ] {
        let mut centered = Mask::new(320, 204).unwrap();
        let mut placed = Mask::new(320, 204).unwrap();
        centered.compose_centered(s.stencil(0).unwrap(), cx, cy);
        let x = (((i64::from(cx) - 16) / 2) & !3) as i32;
        let y = ((i64::from(cy) - 8) / 2) as i32;
        placed.compose(s.stencil(0).unwrap(), x, y);
        assert_eq!(centered.pixels(), placed.pixels());
    }
    let mut m = Mask::new(320, 204).unwrap();
    m.compose_centered(s.stencil(15).unwrap(), 100, 100);
    assert!(m.pixels().iter().all(|b| *b == 0));
}

#[test]
fn loader_uses_only_the_named_base_resource_and_checks_hash_and_cancellation() {
    use std::path::PathBuf;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/current/scratch")
        .join(format!("native-light-mask-loader-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut idx = 1u32.to_le_bytes().to_vec();
    idx.extend(0u32.to_le_bytes());
    let mut name = [0u8; 20];
    name[..10].copy_from_slice(b"source.lmp");
    idx.extend(name);
    idx.extend((LMP.len() as u32).to_le_bytes());
    std::fs::write(root.join("Sprite.idx"), idx).unwrap();
    std::fs::write(root.join("Sprite.pak"), LMP).unwrap();
    let cancel = AtomicBool::new(false);
    let loaded = Source::load(&root, &cancel).unwrap();
    assert_eq!(loaded.stencil(14).unwrap().width(), 312);
    cancel.store(true, std::sync::atomic::Ordering::Release);
    assert!(Source::load(&root, &cancel).is_err());
    cancel.store(false, std::sync::atomic::Ordering::Release);
    let mut bad = LMP.to_vec();
    *bad.last_mut().unwrap() ^= 1;
    std::fs::write(root.join("Sprite.pak"), bad).unwrap();
    assert!(Source::load(&root, &cancel).is_err());
    // Only the verified task-local fixture paths are touched.
    std::fs::remove_file(root.join("Sprite.idx")).unwrap();
    std::fs::remove_file(root.join("Sprite.pak")).unwrap();
    std::fs::remove_dir(root).unwrap();
}
