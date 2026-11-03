//! Explicit, read-only profile selection. No CWD mutation, connection, profile
//! persistence, or recent-document entry; the caller still validates the file.

use anyhow::{ensure, Result};
use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::PathBuf};
use windows::{
    core::{w, HRESULT, PWSTR},
    Win32::{
        Foundation::{ERROR_CANCELLED, HWND},
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
        },
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FileOpenDialog, IFileOpenDialog, FILEOPENDIALOGOPTIONS,
            FOS_DONTADDTORECENT, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_NOCHANGEDIR,
            FOS_PATHMUSTEXIST, SIGDN_FILESYSPATH,
        },
    },
};

pub(super) const MAX_PATH_UTF16: usize = 32_760;
const OPTIONS: FILEOPENDIALOGOPTIONS = FILEOPENDIALOGOPTIONS(
    FOS_FILEMUSTEXIST.0
        | FOS_PATHMUSTEXIST.0
        | FOS_FORCEFILESYSTEM.0
        | FOS_NOCHANGEDIR.0
        | FOS_DONTADDTORECENT.0,
);

fn selected_path(text: &[u16]) -> Result<PathBuf> {
    ensure!(
        !text.is_empty() && text.len() <= MAX_PATH_UTF16 && !text.contains(&0),
        "invalid selected profile path length"
    );
    let path = PathBuf::from(OsString::from_wide(text));
    ensure!(path.is_absolute(), "selected profile path is not absolute");
    Ok(path)
}

struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

struct ShellPath(PWSTR);
impl Drop for ShellPath {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0 .0.cast())) };
    }
}

/// Must be called on the owning UI thread without a live RefCell borrow:
/// Show pumps a nested modal message loop. All COM objects drop before the
/// balanced apartment lease (including S_FALSE/already-initialized success).
pub(super) unsafe fn choose(owner: HWND) -> Result<Option<PathBuf>> {
    CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()?;
    let _apartment = Apartment;
    let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
    dialog.SetOptions(OPTIONS)?;
    dialog.SetFileTypes(&[COMDLG_FILTERSPEC {
        pszName: w!("LiTo 現代設定 (*.json)"),
        pszSpec: w!("*.json"),
    }])?;
    dialog.SetTitle(w!("選擇 LiTo 現代設定檔 — 不會自動連線"))?;
    dialog.SetOkButtonLabel(w!("選擇設定檔"))?;
    match dialog.Show(Some(owner)) {
        Ok(()) => {}
        Err(error) if error.code() == HRESULT::from_win32(ERROR_CANCELLED.0) => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    let item = dialog.GetResult()?;
    let path = ShellPath(item.GetDisplayName(SIGDN_FILESYSPATH)?);
    ensure!(!path.0.is_null(), "selected profile path missing");
    // Shell owns a NUL-terminated allocation until ShellPath drops; preserve
    // native UTF-16 (including non-BMP names) without lossy string conversion.
    selected_path(path.0.as_wide()).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::ffi::OsStrExt;

    #[test]
    fn selection_preserves_explicit_unicode_path_without_trimming_or_cwd_guessing() {
        let text: Vec<_> = "D:\\測試 資料\\🌞modern-profile.json"
            .encode_utf16()
            .collect();
        let path = selected_path(&text).unwrap();
        assert_eq!(path.as_os_str().encode_wide().collect::<Vec<_>>(), text);
        for invalid in [
            "",
            "relative.json",
            "D:relative.json",
            "\\rooted.json",
            "D:\\bad\0.json",
        ] {
            assert!(selected_path(&invalid.encode_utf16().collect::<Vec<_>>()).is_err());
        }
    }

    #[test]
    fn selection_matches_edit_limit_without_silently_truncating() {
        let mut text: Vec<_> = "D:\\".encode_utf16().collect();
        text.resize(MAX_PATH_UTF16, b'x' as u16);
        assert!(selected_path(&text).is_ok());
        text.push(b'x' as u16);
        assert!(selected_path(&text).is_err());
    }

    #[test]
    fn picker_only_selects_existing_files_without_recent_entries_or_cwd_changes() {
        use windows::Win32::UI::Shell::{FOS_ALLOWMULTISELECT, FOS_CREATEPROMPT, FOS_PICKFOLDERS};
        for required in [
            FOS_FILEMUSTEXIST,
            FOS_PATHMUSTEXIST,
            FOS_FORCEFILESYSTEM,
            FOS_NOCHANGEDIR,
            FOS_DONTADDTORECENT,
        ] {
            assert_ne!(OPTIONS.0 & required.0, 0);
        }
        for forbidden in [FOS_ALLOWMULTISELECT, FOS_CREATEPROMPT, FOS_PICKFOLDERS] {
            assert_eq!(OPTIONS.0 & forbidden.0, 0);
        }
    }
}
