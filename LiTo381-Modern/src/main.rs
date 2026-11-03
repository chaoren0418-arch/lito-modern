#![cfg_attr(not(test), windows_subsystem = "windows")]
use anyhow::{ensure, Context, Result};
use lito_modern::modern_lito::{self, profile::ModernProfile};
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

#[derive(Debug, PartialEq, Eq)]
enum Route {
    Pick,
    NativeConfig(PathBuf),
    CheckNativeConfig(PathBuf),
    ConfigureNative,
    OfflineScene(PathBuf),
    VerifyResources { root: PathBuf, output: Option<PathBuf> },
    ProbeNative(std::net::SocketAddr),
    ProbeNativeLogin(std::net::SocketAddr),
    ProbeNativeGame(std::net::SocketAddr),
    ProbeNativeLobby(std::net::SocketAddr),
    NativeLogin(std::net::SocketAddr),
    Profile(PathBuf),
    ProbeProfile(PathBuf),
    #[cfg(debug_assertions)]
    CanonicalProfile(PathBuf),
    #[cfg(debug_assertions)]
    TargetHasteProfile(PathBuf),
    #[cfg(debug_assertions)]
    CooperativeHasteProfile(PathBuf),
    #[cfg(debug_assertions)]
    SlowProfile(PathBuf),
    Help,
    Version,
    #[cfg(debug_assertions)]
    Preview,
    #[cfg(debug_assertions)]
    Scene,
}
fn route(args: &[OsString]) -> Result<Route> {
    match args.get(1).map(OsString::as_os_str) {
        None => Ok(Route::Pick),
        Some(x) if (x == OsStr::new("--native-config") || x == OsStr::new("--check-native-config")) && args.len() == 3 => {
            let path = PathBuf::from(&args[2]);
            ensure!(path.is_absolute(), "原生登入設定檔必須使用絕對路徑");
            Ok(if x == OsStr::new("--check-native-config") { Route::CheckNativeConfig(path) } else { Route::NativeConfig(path) })
        }
        Some(x) if x == OsStr::new("--configure-native") && args.len() == 2 => Ok(Route::ConfigureNative),
        Some(x) if x == OsStr::new("--help") && args.len() == 2 => Ok(Route::Help),
        Some(x) if x == OsStr::new("--version") && args.len() == 2 => Ok(Route::Version),
        Some(x) if x==OsStr::new("--native-login") && (3..=4).contains(&args.len())=>{
            if let Some(root)=args.get(3){ensure!(PathBuf::from(root).is_absolute(),"resource root must be absolute");}
            let peer:std::net::SocketAddr=args[2].to_str().ok_or_else(||anyhow::anyhow!("native endpoint Unicode"))?.parse()?;
            ensure!(peer.port()!=0 && !peer.ip().is_unspecified() && !peer.ip().is_multicast(),"請指定有效的登入端點");Ok(Route::NativeLogin(peer))
        },
        Some(x) if (x == OsStr::new("--probe-native-loopback") || x == OsStr::new("--probe-native-login") || x==OsStr::new("--probe-native-game") || x==OsStr::new("--probe-native-lobby")) && args.len() == 3 => {
            let peer: std::net::SocketAddr = args[2].to_str().ok_or_else(||anyhow::anyhow!("native endpoint Unicode"))?.parse()?;
            ensure!(peer.ip().is_loopback() && peer.port()!=0,"只允許明確的隔離 loopback 測試端點");
            Ok(if x==OsStr::new("--probe-native-lobby"){Route::ProbeNativeLobby(peer)}else if x==OsStr::new("--probe-native-game"){Route::ProbeNativeGame(peer)}else if x==OsStr::new("--probe-native-login"){Route::ProbeNativeLogin(peer)}else{Route::ProbeNative(peer)})
        }
        Some(x) if x == OsStr::new("--offline-scene") && args.len() == 3 => {
            let path = PathBuf::from(&args[2]);
            ensure!(path.is_absolute(), "資源目錄必須使用絕對路徑");
            Ok(Route::OfflineScene(path))
        }
        Some(x) if x == OsStr::new("--verify-resources") && matches!(args.len(), 3 | 5) => {
            let root = PathBuf::from(&args[2]);
            ensure!(root.is_absolute(), "資源目錄必須使用絕對路徑");
            let output = if args.len() == 5 {
                ensure!(args[3] == OsStr::new("--output"), "請使用 --output 指定診斷輸出目錄");
                let path = PathBuf::from(&args[4]);
                ensure!(path.is_absolute(), "診斷輸出目錄必須使用絕對路徑");
                Some(path)
            } else { None };
            Ok(Route::VerifyResources { root, output })
        }
        Some(x) if x == OsStr::new("--probe-profile") && args.len() == 3 => {
            let path = PathBuf::from(&args[2]);
            ensure!(path.is_absolute(), "LiTo 設定必須使用絕對路徑");
            Ok(Route::ProbeProfile(path))
        }
        #[cfg(debug_assertions)]
        Some(x) if x == OsStr::new("--lito-preview") => Ok(Route::Preview),
        #[cfg(debug_assertions)]
        Some(x) if x == OsStr::new("--lito-scene") => Ok(Route::Scene),
        #[cfg(debug_assertions)]
        Some(x) if x == OsStr::new("--canonical-profile") && args.len() == 3 => {
            let path = PathBuf::from(&args[2]);
            ensure!(path.is_absolute(), "LiTo 設定必須使用絕對路徑");
            Ok(Route::CanonicalProfile(path))
        }
        #[cfg(debug_assertions)]
        Some(x) if x == OsStr::new("--target-haste-profile") && args.len() == 3 => {
            let path = PathBuf::from(&args[2]);
            ensure!(path.is_absolute(), "LiTo 設定必須使用絕對路徑");
            Ok(Route::TargetHasteProfile(path))
        }
        #[cfg(debug_assertions)]
        Some(x) if x == OsStr::new("--cooperative-haste-profile") && args.len() == 3 => {
            let path = PathBuf::from(&args[2]);
            ensure!(path.is_absolute(), "LiTo 設定必須使用絕對路徑");
            Ok(Route::CooperativeHasteProfile(path))
        }
        #[cfg(debug_assertions)]
        Some(x) if x == OsStr::new("--slow-profile") && args.len() == 3 => {
            let path = PathBuf::from(&args[2]);
            ensure!(path.is_absolute(), "LiTo 設定必須使用絕對路徑");
            Ok(Route::SlowProfile(path))
        }
        Some(x)
            if (x == OsStr::new("--lito") || x == OsStr::new("--profile")) && args.len() == 3 =>
        {
            let path = PathBuf::from(&args[2]);
            ensure!(path.is_absolute(), "LiTo 設定必須使用絕對路徑");
            Ok(Route::Profile(path))
        }
        _ => anyhow::bail!(
            "使用 --help 查看離線場景、資源驗證與隔離原生連線診斷參數"
        ),
    }
}
fn run() -> Result<i32> {
    let args = std::env::args_os().collect::<Vec<_>>();
    let selected = match route(&args)? {
        Route::NativeLogin(peer) => {
            let root = if let Some(root) = args.get(3) { PathBuf::from(root) } else {
                let path = modern_lito::native_startup::default_path(&std::env::current_exe()?)?;
                modern_lito::native_startup::Settings::load(&path)
                    .context("請指定資源根目錄，或先以 --configure-native 設定")?.resource_root().to_owned()
            };
            let settings = modern_lito::native_startup::Settings::new(&peer.to_string(), root.to_str().ok_or_else(||anyhow::anyhow!("資源路徑必須是 Unicode"))?)?;
            return run_native_settings(settings);
        }
        Route::CheckNativeConfig(path) => {
            let settings = modern_lito::native_startup::Settings::load(&path)?;
            settings.validate_available_root()?;
            println!("{}",serde_json::json!({"schema":1,"status":"valid_native_settings","login_endpoint":settings.endpoint().to_string(),"resource_root":settings.resource_root(),"network_started":false,"settings_written":false,"resource_archives_verified":false}));
            return Ok(0);
        }
        Route::NativeConfig(path) => return run_native_settings(modern_lito::native_startup::Settings::load(&path)?),
        Route::ConfigureNative => return run_native_default(true),
        Route::ProbeNativeLogin(peer)|Route::ProbeNativeGame(peer)|Route::ProbeNativeLobby(peer) => {
            use std::io::Read;
            #[derive(serde::Deserialize)]
            struct Input {account:String,password:String,retry_password:Option<String>,source_index:usize,lobby_mode:Option<String>}
            impl Drop for Input {fn drop(&mut self){use zeroize::Zeroize;self.account.zeroize();self.password.zeroize();if let Some(value)=&mut self.retry_password{value.zeroize();}}}
            let mut bytes=zeroize::Zeroizing::new(Vec::new());std::io::stdin().take(4097).read_to_end(&mut bytes)?;
            ensure!(bytes.len()<=4096,"native login diagnostic input limit");
            let input:Input=serde_json::from_slice(&bytes)?;
            let report=if args[1]==OsStr::new("--probe-native-lobby"){modern_lito::native_login::probe_lobby(peer,&input.account,&input.password,input.source_index,input.lobby_mode.as_deref().unwrap_or("enter"))?}else if args[1]==OsStr::new("--probe-native-game"){modern_lito::native_login::probe_game(peer,&input.account,&input.password,input.source_index,true)?}else{modern_lito::native_login::probe(peer,&input.account,&input.password,input.retry_password.as_deref(),input.source_index)?};
            println!("{}",report);
            return Ok(0);
        }
        Route::ProbeNative(peer) => {
            println!("{}",modern_lito::native_connection::probe_loopback(peer)?);
            return Ok(0);
        }
        Route::OfflineScene(path) => return run_offline_scene(&path),
        Route::VerifyResources { root, output } => {
            let output = output.unwrap_or(std::env::current_dir()?.join("verification"));
            println!("{}", modern_lito::rebuild_verification::run(&root, &output)?);
            return Ok(0);
        }
        Route::Version => {
            // A read-only, bounded diagnostic: no profile selection, resources,
            // credentials, network, settings writes or native windows.
            println!("{}", version_document());
            return Ok(0);
        }
        Route::ProbeProfile(path) => {
            let report = modern_lito::session::profile_probe::run(&path);
            let code = if report["success"] == true { 0 } else { 2 };
            println!("{report}");
            return Ok(code);
        }
        #[cfg(debug_assertions)]
        Route::Preview => {
            return modern_lito::preview::run_from_args(&unicode_args(&args)?).map(|_| 0)
        }
        #[cfg(debug_assertions)]
        Route::Scene => {
            return modern_lito::scene_preview::run_from_args(&unicode_args(&args)?).map(|_| 0)
        }
        #[cfg(debug_assertions)]
        Route::CanonicalProfile(path) => {
            let profile = ModernProfile::load(&path)?;
            let controller =
                modern_lito::session::SessionController::new_canonical_development(profile.clone());
            return modern_lito::login_window::run(&profile, Box::new(controller)).map(|_| 0);
        }
        #[cfg(debug_assertions)]
        Route::TargetHasteProfile(path) => {
            let profile = ModernProfile::load(&path)?;
            let controller = modern_lito::session::SessionController::new_target_haste_development(
                profile.clone(),
            );
            return modern_lito::login_window::run(&profile, Box::new(controller)).map(|_| 0);
        }
        #[cfg(debug_assertions)]
        Route::CooperativeHasteProfile(path) => {
            let profile = ModernProfile::load(&path)?;
            let controller =
                modern_lito::session::SessionController::new_cooperative_haste_development(
                    profile.clone(),
                );
            return modern_lito::login_window::run(&profile, Box::new(controller)).map(|_| 0);
        }
        #[cfg(debug_assertions)]
        Route::SlowProfile(path) => {
            let profile = ModernProfile::load(&path)?;
            let controller =
                modern_lito::session::SessionController::new_slow_development(profile.clone());
            return modern_lito::login_window::run(&profile, Box::new(controller)).map(|_| 0);
        }
        Route::Pick => return run_native_default(false),
        Route::Profile(path) => Some(path),
        Route::Help => {
            message("直接開啟進入原生登入；首次啟動可設定登入 IP:連接埠與資源根目錄。\n--configure-native 重新設定原生登入。\n--native-config <設定檔絕對路徑> 使用指定的原生登入設定。\n--check-native-config <設定檔絕對路徑> 不連線檢查設定。\n--offline-scene <資源目錄絕對路徑> 選擇只讀資源。\n--verify-resources <資源目錄絕對路徑> [--output <診斷目錄絕對路徑>] 執行無視窗資源與輸入驗證；省略輸出目錄時寫入目前工作目錄的 verification。\n--native-login <IP:port> [資源根目錄絕對路徑] 開啟原生帳密／來源選服視窗；省略資源根目錄時讀取程式旁原生登入設定。\n--probe-native-login <127.0.0.1:port> 由 stdin JSON 接受隔離测试帳密。\n--probe-native-loopback <127.0.0.1:port> 只驗證隔離原生 TCP 版本交換。\n--profile <設定檔絕對路徑> 保留歷史 Modern 協定候選；與現行正式伺服器相容性尚待驗證。\n--version 提供不連線的版本資訊。\n此程式不啟動原生 LiTo.bin 或 TW。", false);
            None
        }
    };
    if let Some(path) = selected {
        let profile = ModernProfile::load(&path)?;
        // The validated profile selects the contract in Debug and Release.
        // Old schema1 stays on v37; schema2 is an explicit, non-fallback v47.
        let controller = modern_lito::session::SessionController::new(profile.clone());
        modern_lito::login_window::run(&profile, Box::new(controller))?;
    }
    Ok(0)
}
fn run_native_settings(settings: modern_lito::native_startup::Settings) -> Result<i32> {
    settings.validate_available_root()?;
    modern_lito::native_login_window::run_with_resources(settings.endpoint(), settings.resource_root().to_owned())?;
    Ok(0)
}
fn run_native_default(configure: bool) -> Result<i32> {
    let path = modern_lito::native_startup::default_path(&std::env::current_exe()?)?;
    let (initial, error) = match std::fs::symlink_metadata(&path) {
        Ok(_) => match modern_lito::native_startup::Settings::load(&path) {
            Ok(value) => (Some(value), None),
            Err(error) => (None, Some(format!("無法讀取先前設定：{error:#}"))),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (None, None),
        Err(error) => (None, Some(format!("無法讀取先前設定：{error}"))),
    };
    if !configure {
        if let Some(settings) = initial.as_ref() {
            if settings.validate_available_root().is_ok() { return run_native_settings(settings.clone()); }
        }
    }
    if let Some(settings) = modern_lito::native_startup_window::run(initial, path, error)? {
        return run_native_settings(settings);
    }
    Ok(0)
}
fn run_offline_scene(root: &std::path::Path) -> Result<i32> {
    modern_lito::scene_preview::run_from_args(&[
        "LiToModern".into(), "--lito-scene".into(),
        root.to_str().ok_or_else(|| anyhow::anyhow!("資源路徑必須是 Unicode"))?.into(), "0".into()
    ])?;
    Ok(0)
}
fn version_document() -> serde_json::Value {
    serde_json::json!({
        "schema": 1,
        "application": "LiToModern",
        "version": env!("CARGO_PKG_VERSION"),
        "build": if cfg!(debug_assertions) { "debug" } else { "release" },
        "status": "development_candidate",
        "pointer_bits": usize::BITS,
        "default_entry": "native_login_setup",
        "native_settings_file": modern_lito::native_startup::FILE_NAME,
        "formal_server_compatible": false,
        "profile_protocols": ["legacy_v37", "modern_v47"],
        "extensions": modern_lito::profile::GameplayExtension::ALL.map(|e| e.as_str()),
        "launches_native_bin": false,
        "launches_tw": false
    })
}
#[cfg(debug_assertions)]
fn unicode_args(args: &[OsString]) -> Result<Vec<String>> {
    args.iter()
        .map(|arg| {
            arg.clone()
                .into_string()
                .map_err(|_| anyhow::anyhow!("開發驗證參數必須是有效 Unicode"))
        })
        .collect()
}
fn message(text: &str, error: bool) {
    use windows::{
        core::{w, PCWSTR},
        Win32::UI::WindowsAndMessaging::*,
    };
    let text: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            w!("LiTo Modern"),
            MB_OK
                | if error {
                    MB_ICONERROR
                } else {
                    MB_ICONINFORMATION
                },
        );
    }
}
fn main() {
    match run() {
        Ok(0) => {}
        Ok(code) => std::process::exit(code),
        Err(error) => {
            if matches!(std::env::args_os().nth(1).as_deref(),Some(x) if x==OsStr::new("--check-native-config") || x==OsStr::new("--verify-resources") || x==OsStr::new("--probe-native-loopback") || x==OsStr::new("--probe-native-login") || x==OsStr::new("--probe-native-game") || x==OsStr::new("--probe-native-lobby")) {
                println!("{}", serde_json::json!({"schema":1,"status":"failed","error":format!("{error:#}")}));
                std::process::exit(2);
            }
            if std::env::args_os().nth(1).as_deref() == Some(OsStr::new("--probe-profile")) {
                // Even malformed probe arguments stay machine-readable and
                // cannot open a blocking native error dialog or echo secrets.
                println!(
                    "{}",
                    modern_lito::session::profile_probe::invalid_arguments()
                );
                std::process::exit(2);
            }
            message(&format!("啟動失敗：{error:#}"), true);
            std::process::exit(1);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_is_an_exact_noninteractive_route_in_debug_and_release() {
        assert_eq!(
            route(&args(&["LiToModern.exe", "--version"])).unwrap(),
            Route::Version
        );
        assert!(route(&args(&["LiToModern.exe", "--version", "extra"])).is_err());
        let doc = version_document();
        assert_eq!(doc["application"], "LiToModern");
        assert_eq!(
            doc["build"],
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
        );
        assert_eq!(
            doc["profile_protocols"],
            serde_json::json!(["legacy_v37", "modern_v47"])
        );
        assert_eq!(
            doc["extensions"],
            serde_json::json!([
                "player_healing_v1",
                "armor_protection_v1",
                "player_healing_armor_v1",
                "armor_presentation_v1",
                "player_healing_armor_presentation_v1",
                "full_vip_v1",
                "full_vip_v2"
            ])
        );
        assert_eq!(doc["launches_native_bin"], false);
        assert_eq!(doc["launches_tw"], false);
        assert_eq!(doc["status"], "development_candidate");
    }
    #[test]
    fn native_startup_is_default_and_config_routes_are_exact() {
        assert_eq!(route(&args(&["LiToModern.exe"])).unwrap(),Route::Pick);
        assert_eq!(route(&args(&["LiToModern.exe","--configure-native"])).unwrap(),Route::ConfigureNative);
        let path=r"D:\獨立套件\LiToModern.native.json";
        assert_eq!(route(&args(&["LiToModern.exe","--native-config",path])).unwrap(),Route::NativeConfig(path.into()));
        for values in [vec!["LiToModern.exe","--native-config"], vec!["LiToModern.exe","--native-config","relative.json"], vec!["LiToModern.exe","--native-config",path,"extra"], vec!["LiToModern.exe","--configure-native","extra"]] { assert!(route(&args(&values)).is_err()); }
        assert_eq!(route(&args(&["LiToModern.exe","--check-native-config",path])).unwrap(),Route::CheckNativeConfig(path.into()));
        assert_eq!(version_document()["default_entry"],"native_login_setup");
        assert_eq!(version_document()["native_settings_file"],"LiToModern.native.json");
    }
    #[test]
    fn resource_diagnostic_output_is_explicit_absolute_and_exact() {
        let root = r"D:\original assets";
        let output = r"D:\writable diagnostics";
        assert_eq!(route(&args(&["LiToModern.exe", "--verify-resources", root])).unwrap(),
                   Route::VerifyResources { root: root.into(), output: None });
        assert_eq!(route(&args(&["LiToModern.exe", "--verify-resources", root, "--output", output])).unwrap(),
                   Route::VerifyResources { root: root.into(), output: Some(output.into()) });
        for values in [
            vec!["LiToModern.exe", "--verify-resources", root, "--output"],
            vec!["LiToModern.exe", "--verify-resources", root, "--output", "relative"],
            vec!["LiToModern.exe", "--verify-resources", root, "--unknown", output],
            vec!["LiToModern.exe", "--verify-resources", root, "--output", output, "extra"],
            vec!["LiToModern.exe", "--verify-resources", "relative", "--output", output],
            vec!["LiToModern.exe", "--offline-scene", root, "--output", output],
        ] { assert!(route(&args(&values)).is_err()); }
    }
    #[test]
    fn probe_is_explicit_exact_absolute_and_distinct_from_interactive_login() {
        let path = r"D:\profiles\modern.json";
        assert_eq!(
            route(&args(&["LiToModern.exe", "--probe-profile", path])).unwrap(),
            Route::ProbeProfile(PathBuf::from(path))
        );
        for values in [
            vec!["LiToModern.exe", "--probe-profile"],
            vec!["LiToModern.exe", "--probe-profile", "relative.json"],
            vec!["LiToModern.exe", "--probe-profile", path, "private-secret"],
        ] {
            assert!(route(&args(&values)).is_err());
        }
        assert_eq!(
            route(&args(&["LiToModern.exe", "--profile", path])).unwrap(),
            Route::Profile(PathBuf::from(path))
        );
    }
    #[test]
    fn packaged_profile_explicitly_selects_modern_without_changing_legacy_example() {
        use modern_lito::profile::{GameplayExtension, GameplayProtocol};
        let bytes = include_bytes!("../packaging/profile.example.json");
        let profile = ModernProfile::parse(bytes).unwrap();
        assert_eq!(profile.gameplay_protocol(), GameplayProtocol::ModernV47);
        assert_eq!(profile.gameplay_extension(), None);
        let mut doc: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        doc["gameplay_extension"] = serde_json::json!("player_healing_v1");
        assert_eq!(
            ModernProfile::parse(&serde_json::to_vec(&doc).unwrap())
                .unwrap()
                .gameplay_extension(),
            Some(GameplayExtension::PlayerHealingV1)
        );
        assert_eq!(
            ModernProfile::parse(include_bytes!("../config/profile.example.json"))
                .unwrap()
                .gameplay_protocol(),
            GameplayProtocol::LegacyV37
        );
    }
    #[test]
    fn ordinary_profile_route_selects_v47_including_release_without_debug_flags() {
        use modern_lito::login_window::{LoginController, LoginPhase};
        let mut doc: serde_json::Value =
            serde_json::from_str(include_str!("../config/profile.example.json")).unwrap();
        let legacy = ModernProfile::parse(&serde_json::to_vec(&doc).unwrap()).unwrap();
        assert_eq!(
            legacy.gameplay_protocol(),
            modern_lito::profile::GameplayProtocol::LegacyV37
        );
        doc["schema"] = serde_json::json!(2);
        doc["gameplay_protocol"] = serde_json::json!("modern_v47");
        let modern = ModernProfile::parse(&serde_json::to_vec(&doc).unwrap()).unwrap();
        assert_eq!(
            modern.gameplay_protocol(),
            modern_lito::profile::GameplayProtocol::ModernV47
        );
        let controller = modern_lito::session::SessionController::new(modern);
        assert_eq!(controller.snapshot().phase, LoginPhase::Disconnected);
        assert!(controller.snapshot().status.contains("v47"));
        assert!(controller.is_quiescent());
        let path = "D:\\測試\\Modern v47.json";
        assert_eq!(
            route(&args(&["LiToModern.exe", "--profile", path])).unwrap(),
            Route::Profile(PathBuf::from(path))
        );
        assert_eq!(route(&args(&["LiToModern.exe"])).unwrap(), Route::Pick);
    }
    #[cfg(debug_assertions)]
    #[test]
    fn slow_uses_one_explicit_debug_profile_and_never_a_tw_fallback() {
        let path = "D:\\test.json";
        assert_eq!(
            route(&args(&["LiToModern.exe", "--slow-profile", path])).unwrap(),
            Route::SlowProfile(PathBuf::from(path))
        );
        for tail in [vec![], vec!["relative.json"], vec![path, "extra"]] {
            let mut values = vec!["LiToModern.exe", "--slow-profile"];
            values.extend(tail);
            assert!(route(&args(&values)).is_err());
        }
    }
    #[test]
    fn native_diagnostic_route_requires_exact_loopback_endpoint() {
        for endpoint in ["127.0.0.1:32123", "[::1]:32123"] {
            assert_eq!(route(&args(&["LiToModern.exe", "--probe-native-loopback", endpoint])).unwrap(), Route::ProbeNative(endpoint.parse().unwrap()));
        }
        for tail in [vec![], vec!["localhost:32123"], vec!["127.0.0.1:0"], vec!["192.0.2.1:32123"], vec!["127.0.0.1:32123", "extra"]] {
            let mut values=vec!["LiToModern.exe", "--probe-native-loopback"];values.extend(tail);
            assert!(route(&args(&values)).is_err());
        }
    }
    #[test]
    fn native_login_ui_and_probe_have_distinct_endpoint_scope() {
        assert_eq!(route(&args(&["LiToModern.exe", "--native-login", "192.0.2.1:32000"])).unwrap(), Route::NativeLogin("192.0.2.1:32000".parse().unwrap()));
        assert!(route(&args(&["LiToModern.exe", "--probe-native-login", "192.0.2.1:32000"])).is_err());
        for endpoint in ["0.0.0.0:32000", "127.0.0.1:0", "localhost:32000", "224.0.0.1:32000"] {
            assert!(route(&args(&["LiToModern.exe", "--native-login", endpoint])).is_err());
        }
    }
    #[test]
    fn native_game_probe_has_exact_loopback_route() {
        let endpoint = "127.0.0.1:32000";
        assert_eq!(route(&args(&["LiToModern.exe", "--probe-native-game", endpoint])).unwrap(), Route::ProbeNativeGame(endpoint.parse().unwrap()));
        for tail in [vec![], vec!["localhost:32000"], vec!["127.0.0.1:0"], vec!["192.0.2.1:32000"], vec![endpoint, "extra"]] {
            let mut values = vec!["LiToModern.exe", "--probe-native-game"]; values.extend(tail);
            assert!(route(&args(&values)).is_err());
        }
    }
    #[test]
    fn native_lobby_probe_requires_exact_loopback_endpoint() {
        assert_eq!(route(&args(&["LiToModern.exe","--probe-native-lobby","127.0.0.1:32000"])).unwrap(),Route::ProbeNativeLobby("127.0.0.1:32000".parse().unwrap()));
        for endpoint in ["192.0.2.1:32000","127.0.0.1:0","localhost:32000"] {assert!(route(&args(&["LiToModern.exe","--probe-native-lobby",endpoint])).is_err());}
    }
    fn args(xs: &[&str]) -> Vec<OsString> {
        xs.iter().map(OsString::from).collect()
    }
    #[test]
    fn standalone_profile_selection_has_no_tw_fallback() {
        assert_eq!(route(&args(&["LiToModern.exe"])).unwrap(), Route::Pick);
        for flag in ["--tw", "--stage2", "--tw-bootstrap", "127.0.0.1", "--gui"] {
            assert!(route(&args(&["LiToModern.exe", flag])).is_err());
        }
    }
    #[test]
    fn explicit_profile_is_absolute_and_exact() {
        let path = "D:\\測試\\現代 LiTo.json";
        for flag in ["--lito", "--profile"] {
            assert_eq!(
                route(&args(&["LiToModern.exe", flag, path])).unwrap(),
                Route::Profile(PathBuf::from(path))
            );
            assert!(route(&args(&["LiToModern.exe", flag, "relative.json"])).is_err());
            assert!(route(&args(&["LiToModern.exe", flag, path, "extra"])).is_err());
        }
    }
    #[cfg(debug_assertions)]
    #[test]
    fn original_development_preview_routes_remain_debug_only() {
        assert_eq!(
            route(&args(&["LiToModern.exe", "--lito-preview"])).unwrap(),
            Route::Preview
        );
        assert_eq!(
            route(&args(&["LiToModern.exe", "--lito-scene"])).unwrap(),
            Route::Scene
        );
    }
    #[cfg(debug_assertions)]
    #[test]
    fn canonical_development_uses_the_same_exe_and_one_explicit_profile() {
        let path = "D:\\測試\\現代 LiTo.json";
        assert_eq!(
            route(&args(&["LiToModern.exe", "--canonical-profile", path])).unwrap(),
            Route::CanonicalProfile(PathBuf::from(path))
        );
        for tail in [vec![], vec!["relative.json"], vec![path, "extra"]] {
            let mut values = vec!["LiToModern.exe", "--canonical-profile"];
            values.extend(tail);
            assert!(route(&args(&values)).is_err());
        }
        assert_eq!(
            route(&args(&["LiToModern.exe", "--profile", path])).unwrap(),
            Route::Profile(PathBuf::from(path))
        );
    }
    #[cfg(not(debug_assertions))]
    #[test]
    fn release_rejects_the_unaccepted_canonical_development_entry() {
        for flag in [
            "--canonical-profile",
            "--target-haste-profile",
            "--cooperative-haste-profile",
            "--slow-profile",
        ] {
            assert!(route(&args(&["LiToModern.exe", flag, "D:\\test.json"])).is_err());
        }
    }
    #[cfg(debug_assertions)]
    #[test]
    fn cooperative_haste_is_one_explicit_debug_route_not_a_new_exe() {
        let path = "D:\\測試\\現代 LiTo.json";
        assert_eq!(
            route(&args(&[
                "LiToModern.exe",
                "--cooperative-haste-profile",
                path
            ]))
            .unwrap(),
            Route::CooperativeHasteProfile(PathBuf::from(path))
        );
        for tail in [vec![], vec!["relative.json"], vec![path, "extra"]] {
            let mut values = vec!["LiToModern.exe", "--cooperative-haste-profile"];
            values.extend(tail);
            assert!(route(&args(&values)).is_err());
        }
    }
    #[cfg(debug_assertions)]
    #[test]
    fn target_haste_development_is_explicit_and_does_not_replace_existing_routes() {
        let path = "D:\\測試\\現代 LiTo.json";
        assert_eq!(
            route(&args(&["LiToModern.exe", "--target-haste-profile", path])).unwrap(),
            Route::TargetHasteProfile(PathBuf::from(path))
        );
        for tail in [vec![], vec!["relative.json"], vec![path, "extra"]] {
            let mut values = vec!["LiToModern.exe", "--target-haste-profile"];
            values.extend(tail);
            assert!(route(&args(&values)).is_err());
        }
        assert_eq!(
            route(&args(&["LiToModern.exe", "--canonical-profile", path])).unwrap(),
            Route::CanonicalProfile(PathBuf::from(path))
        );
        assert_eq!(
            route(&args(&["LiToModern.exe", "--profile", path])).unwrap(),
            Route::Profile(PathBuf::from(path))
        );
    }
}
