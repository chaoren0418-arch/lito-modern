//! Settings for the original native protocol, owned only by this client.
use super::resources::{open_readonly, reject_reparse_chain, verify_open_path};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::SocketAddr,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const FILE_NAME: &str = "LiToModern.native.json";
pub const MAX_BYTES: usize = 16 * 1024;
pub const MAX_ROOT_UTF16: usize = 2048;
static SAVE_ID: AtomicU64 = AtomicU64::new(0);

struct PendingSettingsFile {
    file: File,
    committed: bool,
    #[cfg(not(windows))]
    path: PathBuf,
}
impl Drop for PendingSettingsFile {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        #[cfg(windows)]
        unsafe {
            use std::{ffi::c_void, os::windows::io::AsRawHandle};
            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn SetFileInformationByHandle(
                    file: *mut c_void,
                    class: i32,
                    info: *const c_void,
                    bytes: u32,
                ) -> i32;
            }
            // Delete only this still-open, CREATE_NEW-owned staging object.
            // No path lookup and no directory/other user's file is removed.
            let delete: u32 = 1;
            let _ = SetFileInformationByHandle(
                self.file.as_raw_handle(),
                4,
                (&delete as *const u32).cast(),
                4,
            );
        }
        #[cfg(not(windows))]
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(windows)]
fn rename_same_directory(file: &File, target: &Path) -> Result<()> {
    use std::{
        ffi::c_void,
        mem::{offset_of, size_of},
        os::windows::{ffi::OsStrExt, io::AsRawHandle},
    };
    #[repr(C)]
    struct RenameInfo {
        replace: u32,
        root: *mut c_void,
        length: u32,
        name: [u16; 1],
    }
    #[repr(C)]
    struct IoStatus {
        status: usize,
        information: usize,
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtSetInformationFile(
            file: *mut c_void,
            status: *mut IoStatus,
            info: *mut c_void,
            bytes: u32,
            class: i32,
        ) -> i32;
        fn RtlNtStatusToDosError(status: i32) -> u32;
    }
    let name: Vec<u16> = target
        .file_name()
        .context("設定檔缺少名稱")?
        .encode_wide()
        .collect();
    ensure!(name.len() <= MAX_ROOT_UTF16, "設定檔名稱過長");
    let bytes = size_of::<RenameInfo>() + name.len() * 2;
    let mut storage = vec![0u64; bytes.div_ceil(size_of::<u64>())];
    // Rename an owned open file by a simple leaf name, keeping the verified
    // parent/ancestor guards. The Win32 wrapper resolves a relative FileName
    // against CWD. Native FileRenameInformation with a NULL RootDirectory and
    // a simple leaf keeps the source file's directory and never reopens CWD.
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information
    unsafe {
        let info = storage.as_mut_ptr().cast::<RenameInfo>();
        (*info).replace = 1;
        (*info).root = std::ptr::null_mut();
        (*info).length = (name.len() * 2) as u32;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            storage
                .as_mut_ptr()
                .cast::<u8>()
                .add(offset_of!(RenameInfo, name))
                .cast::<u16>(),
            name.len(),
        );
        let mut io = IoStatus {
            status: 0,
            information: 0,
        };
        let status =
            NtSetInformationFile(file.as_raw_handle(), &mut io, info.cast(), bytes as u32, 10);
        if status < 0 {
            return Err(std::io::Error::from_raw_os_error(
                RtlNtStatusToDosError(status) as i32,
            ))
            .context("無法儲存設定；原設定保留");
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    schema: u32,
    login_endpoint: SocketAddr,
    resource_root: PathBuf,
}

fn local_absolute(path: &Path) -> bool {
    if !path.is_absolute()
        || path.components().any(|c| match c {
            Component::ParentDir => true,
            Component::Normal(s) => s.to_str().is_none_or(|s| {
                s.ends_with(['.', ' ']) || s.contains(':') || s.chars().any(char::is_control)
            }),
            _ => false,
        })
    {
        return false;
    }
    #[cfg(windows)]
    {
        use std::path::Prefix;
        matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
    }
    #[cfg(not(windows))]
    true
}

fn directory_guards(path: &Path) -> Result<Vec<(PathBuf, File)>> {
    ensure!(local_absolute(path), "請使用本機磁碟的絕對路徑");
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetDriveTypeW(root: *const u16) -> u32;
        }
        let root = path.ancestors().last().context("缺少磁碟根目錄")?;
        let wide: Vec<u16> = root.as_os_str().encode_wide().chain(Some(0)).collect();
        ensure!(
            unsafe { GetDriveTypeW(wide.as_ptr()) } != 4,
            "請使用本機磁碟的資源及設定"
        );
    }
    reject_reparse_chain(path)?;
    let mut parents: Vec<_> = path.ancestors().collect();
    parents.reverse();
    let mut guards = Vec::with_capacity(parents.len());
    for parent in parents {
        let file = open_readonly(parent, true)?;
        verify_open_path(&file, parent, true)?;
        guards.push((parent.to_owned(), file));
    }
    Ok(guards)
}

impl Settings {
    pub fn new(endpoint: &str, root: &str) -> Result<Self> {
        ensure!(
            endpoint.len() <= 128 && !endpoint.chars().any(char::is_control),
            "登入位址格式不正確"
        );
        let login_endpoint: SocketAddr = endpoint
            .parse()
            .context("請輸入登入 IP:連接埠，例如 192.168.1.10:32000")?;
        let value = Self {
            schema: 1,
            login_endpoint,
            resource_root: root.into(),
        };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        ensure!(self.schema == 1, "不支援此原生登入設定版本");
        let ip = self.login_endpoint.ip();
        ensure!(
            self.login_endpoint.port() != 0
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !matches!(ip, std::net::IpAddr::V4(v) if v.is_broadcast()),
            "請指定有效的登入 IP 與連接埠"
        );
        let root = self
            .resource_root
            .to_str()
            .context("資源路徑必須是 Unicode")?;
        ensure!(
            root.encode_utf16().count() <= MAX_ROOT_UTF16 && local_absolute(&self.resource_root),
            "請指定資源根目錄的本機絕對路徑"
        );
        Ok(())
    }
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= MAX_BYTES, "原生登入設定過大");
        let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
        // serde validates duplicate keys and rejects credentials/legacy fields.
        let settings: Self = serde_json::from_slice(bytes).context("原生登入設定格式不正確")?;
        settings.validate()?;
        Ok(settings)
    }
    pub fn endpoint(&self) -> SocketAddr {
        self.login_endpoint
    }
    pub fn resource_root(&self) -> &Path {
        &self.resource_root
    }
    pub fn validate_available_root(&self) -> Result<()> {
        self.validate()?;
        directory_guards(&self.resource_root).context("無法讀取指定的資源根目錄")?;
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self> {
        ensure!(local_absolute(path), "設定檔必須使用本機絕對路徑");
        let guards = directory_guards(path.parent().context("設定檔缺少目錄")?)?;
        reject_reparse_chain(path)?;
        let mut file = open_readonly(path, false)?;
        verify_open_path(&file, path, false)?;
        ensure!(
            file.metadata()?.len() <= MAX_BYTES as u64,
            "原生登入設定過大"
        );
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        verify_open_path(&file, path, false)?;
        for (path, file) in &guards {
            verify_open_path(file, path, true)?;
        }
        Self::parse(&bytes)
    }
    /// Explicit UI opt-in only. Contains no account, password or server state.
    /// Rename replaces a whole valid document; a failed save keeps the old one.
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        ensure!(local_absolute(path), "設定檔必須使用本機絕對路徑");
        let parent = path.parent().context("設定檔缺少目錄")?;
        let guards = directory_guards(parent)?;
        match fs::symlink_metadata(path) {
            Ok(_) => {
                reject_reparse_chain(path)?;
                let file = open_readonly(path, false)?;
                verify_open_path(&file, path, false)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        ensure!(bytes.len() <= MAX_BYTES, "原生登入設定過大");
        let temporary = parent.join(format!(
            ".LiToModern.native.{}.{}.tmp",
            std::process::id(),
            SAVE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // Read/write/DELETE on this newly created file, no write/delete
            // sharing, and no traversal through a replaced final component.
            options
                .access_mode(0xc0010000)
                .share_mode(1)
                .custom_flags(0x00200000);
        }
        let mut file = PendingSettingsFile {
            file: options.open(&temporary)?,
            committed: false,
            #[cfg(not(windows))]
            path: temporary.clone(),
        };
        verify_open_path(&file.file, &temporary, false)?;
        file.file.write_all(&bytes)?;
        file.file.sync_all()?;
        for (path, file) in &guards {
            verify_open_path(file, path, true)?;
        }
        // Commit the whole document, or keep the original and retire staging.
        #[cfg(windows)]
        rename_same_directory(&file.file, path)?;
        #[cfg(not(windows))]
        fs::rename(&temporary, path).context("無法儲存設定；原設定保留")?;
        file.committed = true;
        drop(file);
        ensure!(Self::load(path)? == *self, "儲存後設定讀回不符");
        Ok(())
    }
}

pub fn default_path(executable: &Path) -> Result<PathBuf> {
    ensure!(local_absolute(executable), "程式路徑必須是本機絕對路徑");
    Ok(executable.parent().context("程式缺少目錄")?.join(FILE_NAME))
}
