use lito_modern::modern_lito::{native_npc_document::Catalog, native_npc_images::Bank};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::atomic::AtomicBool};

#[test]
fn original_page_images_match_independent_rgb555_reference_and_page_bindings() {
    let catalog = Catalog::load(Path::new(r"D:\lineage開發\0-客戶端"), &AtomicBool::new(false)).unwrap();
    assert!(catalog.image_error.is_none());
    let bank = catalog.images.as_ref().unwrap();
    let page = catalog.get("TeleportMe").unwrap();
    for (action, expected) in [
        ("LT_TELEPORT_B", "CDFFE5F32A3AAC9A70AC0BCC212092D99175FCBAD8EDBCFB870D19C36132829C"),
        ("LT_TELEPORT_C", "6ECEF518A516984FB65915492F139DA8DEC4B7C4BF130029A8F7548E12A7EB00"),
    ] {
        let image = bank.glyph("TeleportMe", &page.images, action).unwrap();
        assert_eq!((image.width, image.height), (13, 14));
        assert_eq!(format!("{:X}", Sha256::digest(&image.rgba)), expected);
        assert!(image.rgba.chunks_exact(4).any(|p| p[3] == 0));
        assert!(bank.glyph("Maya", &page.images, action).is_none());
        assert!(bank.glyph("TeleportMe", &[], action).is_none());
    }
    assert!(bank.glyph("TeleportMe", &page.images, "LT_TELEPORT_A").is_none());
    assert!(bank.glyph("TeleportMe", &page.images, "buy").is_none());
    assert!(bank.glyph("TeleportMe", &[7311], "LT_TELEPORT_C").is_none());
    assert_eq!(bank.sources["members"].as_array().unwrap().len(), 2);
}

#[test]
fn cancelled_image_worker_never_returns_a_publishable_bank() {
    assert!(Bank::load(Path::new(r"D:\lineage開發\0-客戶端"), &AtomicBool::new(true)).is_err());
}

#[test]
fn absent_image_archive_preserves_original_text_actions_and_reports_the_asset_error() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "target/current/npc-images-missing-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let lt = root.join("LiTo");
    std::fs::create_dir_all(&lt).unwrap();
    for member in ["LTtext.idx", "LTtext.pak"] {
        let source = Path::new(r"D:\lineage開發\0-客戶端\LiTo").join(member);
        assert!(source.metadata().unwrap().len() <= 1024 * 1024);
        std::fs::copy(source, lt.join(member)).unwrap();
    }
    let catalog = Catalog::load(&root, &AtomicBool::new(false)).unwrap();
    assert!(catalog.images.is_none());
    assert!(catalog.image_error.as_ref().is_some_and(|s| !s.is_empty()));
    assert_eq!(catalog.get("TeleportMe").unwrap().actions.len(), 2);
    assert_eq!(catalog.get("inn").unwrap().actions.len(), 3);
}

#[test]
fn cancelling_the_real_document_worker_does_not_publish_cached_images() {
    use lito_modern::modern_lito::native_npc_document::Service;
    let mut service = Service::start(Path::new(r"D:\lineage開發\0-客戶端").into()).unwrap();
    service.cancel();
    let limit = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while !service.quiescent() && std::time::Instant::now() < limit {
        service.poll();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(service.quiescent());
    assert!(service.catalog().is_none() && service.error.is_none());
}
