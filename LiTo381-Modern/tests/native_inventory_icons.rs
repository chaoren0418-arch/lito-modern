use lito_modern::modern_lito::native_inventory_icons::{Icon, Reader, Service, MAX_VISIBLE};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
fn source() -> &'static Path {
    Path::new(r"D:\lineage開發\0-客戶端")
}
fn settle(s: &mut Service, ids: &[u16]) {
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        s.tick(ids);
        if s.quiescent() {
            break;
        }
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn covered_black_is_opaque_and_uncovered_pixels_use_background() {
    let i = Icon::decode(
        1,
        "test",
        "1.tbt",
        &[0, 0, 3, 1, 1, 2, 2, 0, 0, 0, 124],
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!((i.covered_pixels, i.covered_zero), (2, 1));
    assert_eq!(&i.cell[..12], &[0, 0, 0, 0, 0, 0, 0, 255, 255, 0, 0, 255]);
    assert_eq!(
        &i.bgra([1, 2, 3])[..12],
        &[3, 2, 1, 255, 0, 0, 0, 255, 0, 0, 255, 255]
    );
}
#[test]
fn signed_offsets_clip_without_wrapping_or_losing_coverage_count() {
    let i = Icon::decode(
        1,
        "test",
        "1.tbt",
        &[
            255, 255, 2, 2, 1, 0, 2, 0, 124, 0, 124, 1, 0, 2, 0, 124, 31, 0,
        ],
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(i.offset, [-1, -1]);
    assert_eq!(i.covered_pixels, 4);
    assert_eq!(&i.cell[..4], &[0, 0, 255, 255]);
    assert!(i.cell[4..].iter().all(|x| *x == 0));
}
#[test]
fn malformed_native_images_and_cancelled_loads_fail_closed() {
    let valid = [0, 0, 1, 1, 1, 0, 1, 0, 0];
    for end in 0..valid.len() {
        assert!(Icon::decode(0, "t", "t", &valid[..end], &AtomicBool::new(false)).is_err());
    }
    for bytes in [
        vec![0, 0, 1, 1, 1, 1, 1, 0, 0],
        vec![0, 0, 1, 1, 1, 0, 1, 0, 128],
        vec![0, 0, 1, 1, 0, 0],
    ] {
        assert!(Icon::decode(0, "t", "t", &bytes, &AtomicBool::new(false)).is_err());
    }
    let mut tall = vec![0, 0, 1, 128];
    tall.extend([0; 128]);
    assert!(Icon::decode(0, "t", "t", &tall, &AtomicBool::new(false)).is_err());
    assert!(Reader::open(Path::new("never-opened"), &AtomicBool::new(true)).is_err());
    assert!(Icon::decode(0, "t", "t", &valid, &AtomicBool::new(true)).is_err());
}
#[test]
fn original_samples_and_every_present_lito_icon_match_independent_reference() {
    let f: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-inventory-icons-current.json")).unwrap();
    let cancel = AtomicBool::new(false);
    let mut r = Reader::open(source(), &cancel).unwrap();
    let mut samples = vec![];
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
    std::fs::create_dir_all(&output).unwrap();
    for sample in f["samples"].as_array().unwrap() {
        let id = sample["id"].as_u64().unwrap() as u16;
        let icon = r.load(id, &cancel).unwrap();
        let actual = serde_json::to_value(&icon).unwrap();
        assert_eq!(&actual, sample);
        let file =
            std::fs::File::create(output.join(format!("native-inventory-icon-{id}.png"))).unwrap();
        let mut encoder = png::Encoder::new(file, 32, 32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&icon.cell)
            .unwrap();
        samples.push(actual);
    }
    let mut rows = String::new();
    let ids = f["lito_present_ids"].as_array().unwrap();
    for id in ids {
        let id = id.as_u64().unwrap() as u16;
        let icon = r.load(id, &cancel).unwrap();
        rows.push_str(&format!("{id}:{}\n", icon.cell_sha256));
    }
    let hash = format!("{:X}", Sha256::digest(rows.as_bytes()));
    assert_eq!(hash, f["lito_cells_sha256"]);
    assert_eq!(ids.len(), 226);
    assert!(r.load(20233, &cancel).is_err());
    assert!(r.load(32768, &cancel).is_err());
    std::fs::write(
        output.join("native-inventory-icons.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"samples":samples,"lito_count":ids.len(),"lito_cells_sha256":hash}),
        )
        .unwrap(),
    )
    .unwrap();
}
#[test]
fn viewport_changes_publish_only_current_intrinsic_ids_and_close_reclaims_worker() {
    let mut s = Service::new(source().into());
    s.tick(&[0, 80, 235]);
    s.tick(&[20001]);
    settle(&mut s, &[20001]);
    assert_eq!(s.cached(), 1);
    assert!(s.icon(20001).is_some());
    assert!(s.icon(80).is_none());
    s.tick(&[80, 235]);
    s.cancel();
    settle(&mut s, &[]);
    assert_eq!(s.cached(), 0);
    s.tick(&[0]);
    assert!(s.quiescent() && s.cached() == 0);
}
#[test]
fn visible_budget_hidden_rows_and_failures_are_bounded_without_retry_loops() {
    let mut s = Service::new(source().into());
    let ids: Vec<u16> = (32768..32868).collect();
    settle(&mut s, &ids);
    assert_eq!(s.cached(), MAX_VISIBLE);
    assert!(s.error(32768).is_some());
    assert!(!s.tick(&ids));
    assert!(s.quiescent());
    settle(&mut s, &[]);
    assert_eq!(s.cached(), 0);
    settle(&mut s, &[80]);
    assert!(s.icon(80).is_some());
    let mut missing = Service::new(source().join("missing-icon-root"));
    settle(&mut missing, &[80]);
    assert!(missing.error(80).is_some());
    assert!(!missing.tick(&[80]));
}
#[test]
fn corrupt_primary_archive_cannot_be_treated_as_base_archive_miss() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "target/current/scratch/icon-corrupt-{}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let lt = root.join("LiTo");
    std::fs::create_dir(&lt).unwrap();
    let mut idx = std::fs::read(source().join("LiTo/LTicon.idx")).unwrap();
    idx[..4].copy_from_slice(&0u32.to_le_bytes());
    std::fs::write(lt.join("LTicon.idx"), idx).unwrap();
    std::fs::copy(source().join("LiTo/LTicon.pak"), lt.join("LTicon.pak")).unwrap();
    let result = Reader::open(&root, &AtomicBool::new(false));
    for name in ["LTicon.idx", "LTicon.pak"] {
        std::fs::remove_file(lt.join(name)).unwrap();
    }
    std::fs::remove_dir(lt).unwrap();
    std::fs::remove_dir(root).unwrap();
    match result {
        Err(e) => assert!(e.to_string().contains("Native old28 count/length invalid")),
        Ok(_) => panic!("corruption accepted"),
    }
}
