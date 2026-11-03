use lito_modern::modern_lito::{
    native_inventory::Item,
    native_inventory_names::{self as names, Catalog, Service},
};
use std::{
    path::Path,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
fn catalog() -> Catalog {
    Catalog::from_tables(
        b"6\r\nSword\r\n\r\n%i template\r\n\\fTcolored\r\n$0\r\n150% power",
        b"3\r\nLito sword\r\nLito shield",
    )
    .unwrap()
}
fn item(name: &str) -> Item {
    Item {
        id: 7,
        use_type: 0,
        charge: 0,
        icon: 80,
        blessing: 0,
        count: 2,
        identified: 1,
        name: name.into(),
        detail: vec![23, 0],
    }
}
fn settle(s: &mut Service) {
    let end = Instant::now() + Duration::from_secs(15);
    while !s.quiescent() {
        s.poll();
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn references_use_actual_base_slots_then_lito_offset_and_preserve_literals() {
    let c = catalog();
    assert_eq!(
        c.resolve("+9 $00000; / $20001 (2)").unwrap(),
        "+9 Sword / Lito shield (2)"
    );
    assert_eq!(c.resolve("$20000").unwrap(), "Lito sword");
    assert_eq!(c.resolve("$5").unwrap(), "150% power");
    let long = "治".repeat(127);
    assert!(long.len() > 256);
    assert_eq!(c.resolve(&long).unwrap(), long);
    assert_eq!(c.resolve("魔法寶石").unwrap(), "魔法寶石");
}
#[test]
fn invalid_or_unsupported_references_never_publish_partial_names_or_change_wire_item() {
    let c = catalog();
    for raw in [
        "$1",
        "$2",
        "$3",
        "$4",
        "$6",
        "$20002",
        "$20003",
        "$19999",
        "$4294967296",
        "$",
        "$-1",
        "$0 / $1",
        "\u{c}Y$0",
    ] {
        let i = item(raw);
        let before = i.clone();
        let row = names::label(&i, Some(&c));
        assert!(!row.resolved, "{raw}");
        assert_eq!(row.name, raw);
        assert_eq!(i, before);
        assert!(!row.text.chars().any(char::is_control));
    }
    assert!(!names::label(&item("$0"), None).resolved);
    assert!(c.resolve(&"x".repeat(names::MAX_INPUT_BYTES + 1)).is_err());
    let c =
        Catalog::from_tables(format!("1\r\n{}", "a".repeat(1000)).as_bytes(), b"1\r\nLT").unwrap();
    assert!(c.resolve("$0$0$0$0$0").is_err());
    assert!(catalog().resolve(&"$0".repeat(129)).is_err());
}
#[test]
fn labels_resolve_only_presentation_and_count_stays_authoritative() {
    let c = catalog();
    let mut i = item("$0 (2)");
    let before = i.clone();
    let a = names::label(&i, Some(&c));
    assert_eq!(a.name, "Sword (2)");
    assert_eq!(a.count, 2);
    assert_eq!(a.text, "Sword (2) × 2");
    assert_eq!(i, before);
    i.count = 1;
    let b = names::label(&i, Some(&c));
    assert_eq!(b.name, "Sword (2)");
    assert_eq!(b.count, 1);
    i.name = "$20001".into();
    assert_eq!(names::label(&i, Some(&c)).name, "Lito shield");
}
#[test]
fn original_tables_use_lito_first_base_miss_fallback_and_known_cp950_names() {
    let c = Catalog::load(
        Path::new(r"D:\lineage開發\0-客戶端"),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(c.resolve("$26 (2)").unwrap(), "治癒藥水 (2)");
    assert_eq!(
        c.resolve("$20000 / $20026").unwrap(),
        "指導員-潘塔休 / 咒術師法杖"
    );
    assert_eq!(c.sources.len(), 2);
    assert_eq!(c.sources[0].archive, "Text");
    assert_eq!(c.sources[1].archive, "LiTo/LTtext");
    assert_eq!(
        (
            c.sources[0].bytes,
            c.sources[0].declared,
            c.sources[0].available
        ),
        (249300, 17001, 16945)
    );
    assert_eq!(
        (
            c.sources[1].bytes,
            c.sources[1].declared,
            c.sources[1].available
        ),
        (2053, 173, 171)
    );
    assert_eq!(
        c.sources[0].sha256,
        "F06C94F47271FCB01F1D5AF63725DFA9A230B5F6BE3C0DD1D53CD37628ADB7DF"
    );
    assert_eq!(
        c.sources[1].sha256,
        "73243064A856FADE75DE110C5D25F6B1F19ED51C3BA7B1219BBE44ED7E84006B"
    );
    assert!(c.resolve("$20171").is_err());
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/current/verification/native-inventory-names.json");
    std::fs::write(output,serde_json::to_vec_pretty(&serde_json::json!({"sources":c.sources,"examples":[c.resolve("$26 (2)").unwrap(),c.resolve("$20000 / $20026").unwrap()]})).unwrap()).unwrap();
}
#[test]
fn worker_closure_reclaims_loading_ready_and_failed_jobs() {
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let mut s = Service::start(root.into()).unwrap();
    settle(&mut s);
    assert!(s.error.is_none());
    assert_eq!(s.catalog().unwrap().resolve("$26").unwrap(), "治癒藥水");
    s.cancel();
    assert!(s.catalog().is_none() && s.quiescent());
    let mut s = Service::start(root.into()).unwrap();
    s.cancel();
    settle(&mut s);
    assert!(s.catalog().is_none() && s.error.is_none());
    let mut s = Service::start(root.join("missing-name-test-root")).unwrap();
    settle(&mut s);
    assert!(s.catalog().is_none() && s.error.is_some());
    assert!(
        Catalog::load(Path::new("path-not-opened"), &AtomicBool::new(true))
            .unwrap_err_text()
            .contains("cancelled")
    );
}
// Keep error assertion independent of Catalog's intentionally opaque storage.
#[test]
fn corrupt_primary_archive_is_an_error_and_cannot_be_treated_as_a_base_miss() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "target/current/scratch/name-corrupt-{}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let lt = root.join("LiTo");
    std::fs::create_dir(&lt).unwrap();
    let source = Path::new(r"D:\lineage開發\0-客戶端\LiTo");
    let mut idx = std::fs::read(source.join("LTtext.idx")).unwrap();
    idx[..4].copy_from_slice(&0u32.to_le_bytes());
    std::fs::write(lt.join("LTtext.idx"), idx).unwrap();
    std::fs::copy(source.join("LTtext.pak"), lt.join("LTtext.pak")).unwrap();
    let result = Catalog::load(&root, &AtomicBool::new(false));
    for name in ["LTtext.idx", "LTtext.pak"] {
        std::fs::remove_file(lt.join(name)).unwrap();
    }
    std::fs::remove_dir(lt).unwrap();
    std::fs::remove_dir(root).unwrap();
    assert!(result
        .unwrap_err_text()
        .contains("Native old28 count/length invalid"));
}
trait ErrorText {
    fn unwrap_err_text(self) -> String;
}
impl ErrorText for anyhow::Result<Catalog> {
    fn unwrap_err_text(self) -> String {
        match self {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected error"),
        }
    }
}
