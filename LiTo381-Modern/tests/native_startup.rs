use lito_modern::modern_lito::native_startup::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static SAVE_ID: AtomicU64 = AtomicU64::new(0);
fn fixture() -> Settings {
    Settings::new("127.0.0.1:32000", r"D:\原始資源").unwrap()
}
#[test]
fn endpoints_roots_and_schema_are_explicit_without_legacy_fields() {
    for ip in ["127.0.0.1:32000", "192.0.2.1:32000", "[::1]:32000"] {
        let s = Settings::new(ip, r"D:\原始資源").unwrap();
        assert_eq!(s.endpoint().to_string(), ip);
    }
    for ip in [
        "",
        "localhost:32000",
        "0.0.0.0:32000",
        "127.0.0.1:0",
        "224.0.0.1:32000",
        "255.255.255.255:32000",
        "[::]:32000",
    ] {
        assert!(Settings::new(ip, r"D:\資源").is_err(), "{ip}");
    }
    for root in [
        "relative",
        r"D:relative",
        r"D:\a\..\b",
        r"\\server\share",
        r"D:\a:stream",
        "D:\\bad\nname",
        r"D:\trailing.\x",
    ] {
        assert!(Settings::new("127.0.0.1:32000", root).is_err(), "{root:?}");
    }
    let value = serde_json::to_value(fixture()).unwrap();
    for field in ["account", "password", "gameplay_protocol", "trust"] {
        let mut value = value.clone();
        value[field] = serde_json::json!("unused");
        assert!(Settings::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    let mut value = value;
    value["schema"] = serde_json::json!(2);
    assert!(Settings::parse(&serde_json::to_vec(&value).unwrap()).is_err());
}
#[test]
fn unicode_bom_duplicate_keys_and_input_budgets_are_checked() {
    let bytes = serde_json::to_vec(&fixture()).unwrap();
    assert_eq!(Settings::parse(&bytes).unwrap(), fixture());
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(&bytes);
    assert_eq!(Settings::parse(&bom).unwrap(), fixture());
    assert!(Settings::parse(br#"{"schema":1,"schema":1,"login_endpoint":"127.0.0.1:32000","resource_root":"D:\\assets"}"#).is_err());
    assert!(Settings::parse(&vec![b' '; MAX_BYTES + 1]).is_err());
    assert!(Settings::new(
        "127.0.0.1:32000",
        &format!("D:\\{}", "字".repeat(MAX_ROOT_UTF16))
    )
    .is_err());
}
#[test]
fn settings_are_adjacent_to_executable_independent_of_working_directory() {
    assert_eq!(
        default_path(Path::new(r"D:\移動套件\LiToModern.exe")).unwrap(),
        PathBuf::from(r"D:\移動套件\LiToModern.native.json")
    );
    assert!(default_path(Path::new("LiToModern.exe")).is_err());
}
#[test]
fn bounded_file_save_replaces_atomically_and_read_is_unchanged() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/current/native-startup-tests")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SAVE_ID.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&folder).unwrap();
    let cwd_settings = std::env::current_dir().unwrap().join(FILE_NAME);
    let cwd_before = fs::read(&cwd_settings).ok();
    let path = folder.join(FILE_NAME);
    let original = Settings::new("127.0.0.1:32000", folder.to_str().unwrap()).unwrap();
    original.save(&path).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(Settings::load(&path).unwrap(), original);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let replacement = Settings::new("[::1]:32001", folder.to_str().unwrap()).unwrap();
    replacement.save(&path).unwrap();
    assert_eq!(Settings::load(&path).unwrap(), replacement);
    replacement.validate_available_root().unwrap();
    assert_eq!(fs::read_dir(&folder).unwrap().count(), 1);
    assert_eq!(fs::read(&cwd_settings).ok(), cwd_before);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let saved = fs::read(&path).unwrap();
        let lock = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&path)
            .unwrap();
        assert!(original.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), saved);
        assert_eq!(
            fs::read_dir(&folder).unwrap().count(),
            1,
            "failed save leaked staging"
        );
        drop(lock);
    }
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 3);
    fs::write(folder.join("oversized.json"), vec![b' '; MAX_BYTES + 1]).unwrap();
    assert!(Settings::load(&folder.join("oversized.json")).is_err());
}
