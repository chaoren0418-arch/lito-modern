#![cfg_attr(not(test), windows_subsystem = "windows")]
//! Explicit bounded 485F player candidate; the default LiToModern binary stays separate.
use anyhow::{ensure, Result};
use lito_modern::modern_lito::{
    native_backend_485f_entry::Accepted, native_city_window,
    native_offline_preview_resources::Inputs, native_resources::BytePin,
};
use std::{ffi::{OsStr, OsString}, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
struct CandidateRoute {
    ready: PathBuf,
    trusted_sha256: String,
}

fn route(args: &[OsString]) -> Result<CandidateRoute> {
    ensure!(
        args.len() == 4 && args[1] == OsStr::new("--candidate-backend-485f"),
        "使用 --candidate-backend-485f <fresh READY 絕對路徑> <監製核准 SHA256>"
    );
    let ready = PathBuf::from(&args[2]);
    let sha = args[3].to_str().ok_or_else(|| anyhow::anyhow!("ASCII SHA required"))?;
    ensure!(
        ready.is_absolute() && sha.len() == 64 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
        "absolute fresh READY plus trusted SHA256 required"
    );
    Ok(CandidateRoute { ready, trusted_sha256: sha.to_ascii_uppercase() })
}

// Exact R532 resource inputs, now reproducible without custom R532_* env vars.
// Existing resource adapters verify these manifests and their nested payload pins.
fn resource_inputs() -> Result<(Inputs, Inputs)> {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input = |manifest: &str, bytes: u64, hex: &str| -> Result<Inputs> {
        let mut sha256 = [0; 32];
        for (i, byte) in sha256.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)?;
        }
        Ok(Inputs { manifest: project.join(manifest), project: project.clone(),
                    manifest_pin: BytePin { bytes, sha256 } })
    };
    Ok((
        input("logs/continuation-20261003/r486-map-binding.json", 208039,
              "14805BC5D26A88EEAAE3B182A4A477D30AE7B0F26212B760F1CFA29B452DCA6E")?,
        input("logs/continuation-20261003/r506-native-menu-skin-binding.json", 37911,
              "27475914E201922FF0661EDA513B85E73C3E2CBF33FECB3CE71C7345CEF9D379")?,
    ))
}

// This boundary passes configuration only. CPU checks replace the one effectful
// launch callback; production uses the original Accepted/load/window path below.
fn dispatch(
    route: CandidateRoute,
    launch: impl FnOnce(CandidateRoute, Inputs, Inputs) -> Result<()>,
) -> Result<()> {
    let (inputs, skin_inputs) = resource_inputs()?;
    launch(route, inputs, skin_inputs)
}

fn run() -> Result<()> {
    let args = std::env::args_os().collect::<Vec<_>>();
    dispatch(route(&args)?, |route, inputs, skin_inputs| {
        let accepted = Accepted::load(&route.ready, &route.trusted_sha256)?;
        native_city_window::run_backend_485f(inputs, skin_inputs, accepted)?;
        Ok(())
    })
}

fn main() {
    if let Err(error) = run() {
        use windows::{core::PCWSTR, Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK}};
        let text = format!("485F 候選啟動失敗：{error:#}").encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let title = "LiTo Modern · 485F Candidate".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        unsafe { MessageBoxW(None, PCWSTR(text.as_ptr()), PCWSTR(title.as_ptr()), MB_OK | MB_ICONERROR); }
        std::process::exit(1);
    }
}

#[cfg(test)]
mod r542_candidate_checks {
    use super::*;
    fn args(values: &[&str]) -> Vec<OsString> { values.iter().map(OsString::from).collect() }

    #[test]
    fn exact_explicit_route_rejects_malformed_input_and_default_fallback() {
        let ready = r"D:\不存在的測試路徑\fresh ready-contract.json";
        let sha = "aB01".repeat(16);
        let parsed = route(&args(&["candidate", "--candidate-backend-485f", ready, &sha])).unwrap();
        assert_eq!(parsed.ready, PathBuf::from(ready));
        assert_eq!(parsed.trusted_sha256, sha.to_ascii_uppercase());
        for values in [
            vec!["candidate"], vec!["candidate", "--profile", ready, &sha],
            vec!["candidate", "--candidate-backend-485f"],
            vec!["candidate", "--candidate-backend-485f", ready],
            vec!["candidate", "--candidate-backend-485f", ready, &sha, "extra"],
            vec!["candidate", "--candidate-backend-485f", "relative.json", &sha],
            vec!["candidate", "--candidate-backend-485f", r"D:relative.json", &sha],
            vec!["candidate", "--candidate-backend-485f", ready, "bad"],
        ] { assert!(route(&args(&values)).is_err(), "{values:?}"); }
        for bad in ["G".repeat(64), "A".repeat(63), "A".repeat(65), "é".repeat(32)] {
            assert!(route(&args(&["candidate", "--candidate-backend-485f", ready, &bad])).is_err());
        }
        assert!(route(&[]).is_err());
    }

    #[test]
    fn parsed_route_dispatches_pinned_configuration_without_reading_ready_or_claiming() {
        let ready = r"D:\不存在的測試路徑\no READY and no claim.json";
        let sha = "b".repeat(64);
        let mut calls = 0;
        dispatch(route(&args(&["candidate", "--candidate-backend-485f", ready, &sha])).unwrap(),
            |selected, maps, skin| {
                calls += 1;
                assert_eq!(selected.ready, PathBuf::from(ready));
                assert_eq!(selected.trusted_sha256, "B".repeat(64));
                let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                assert_eq!(maps.project, root);
                assert_eq!(skin.project, root);
                assert_eq!(maps.manifest, root.join("logs/continuation-20261003/r486-map-binding.json"));
                assert_eq!(skin.manifest, root.join("logs/continuation-20261003/r506-native-menu-skin-binding.json"));
                assert_eq!((maps.manifest_pin.bytes, skin.manifest_pin.bytes), (208039, 37911));
                assert_eq!(maps.manifest_pin.sha256[..4], [0x14, 0x80, 0x5b, 0xc5]);
                assert_eq!(skin.manifest_pin.sha256[..4], [0x27, 0x47, 0x59, 0x14]);
                Ok(())
            }).unwrap();
        assert_eq!(calls, 1);
        let failure = dispatch(route(&args(&["candidate", "--candidate-backend-485f", ready, &sha])).unwrap(),
            |_, _, _| anyhow::bail!("CPU launch refused"));
        assert_eq!(failure.unwrap_err().to_string(), "CPU launch refused");
    }
}
