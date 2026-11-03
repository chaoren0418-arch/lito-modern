//! Modern-only presentation preferences. Never reads/writes TW configuration,
//! resources, server profiles, credentials, or device-wide mixer settings.
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, OnceLock,
    },
    thread::JoinHandle,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub sound_enabled: bool,
    pub volume_percent: u8,
    pub background_sound: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            volume_percent: 100,
            background_sound: true,
        }
    }
}
impl Preferences {
    pub fn validate(self) -> Result<Self> {
        ensure!(self.volume_percent <= 100, "volume outside 0..100");
        Ok(self)
    }
    pub fn audible_gain(self, foreground: bool) -> u8 {
        if self.sound_enabled && (foreground || self.background_sound) {
            self.volume_percent
        } else {
            0
        }
    }
    fn packed(self) -> u32 {
        u32::from(self.volume_percent)
            | (u32::from(self.sound_enabled) << 8)
            | (u32::from(self.background_sound) << 9)
    }
}

/// One atomic snapshot: a worker cannot observe a new volume with old flags.
pub struct SharedPreferences(AtomicU64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Snapshot {
    pub values: Preferences,
    pub revision: u64,
}
impl Default for SharedPreferences {
    fn default() -> Self {
        Self(AtomicU64::new(u64::from(Preferences::default().packed())))
    }
}
impl SharedPreferences {
    pub fn get(&self) -> Preferences {
        self.snapshot().values
    }
    pub(crate) fn snapshot(&self) -> Snapshot {
        let p = self.0.load(Ordering::Acquire);
        Snapshot {
            revision: p >> 10,
            values: Preferences {
                volume_percent: (p & 255) as u8,
                sound_enabled: p & 256 != 0,
                background_sound: p & 512 != 0,
            },
        }
    }
    pub fn apply(&self, p: Preferences) -> Result<()> {
        let bits = u64::from(p.validate()?.packed());
        let mut previous = self.0.load(Ordering::Acquire);
        loop {
            if previous & 1023 == bits {
                return Ok(());
            }
            let epoch = (previous & !1023)
                .checked_add(1024)
                .context("preferences revision exhausted")?;
            match self.0.compare_exchange_weak(
                previous,
                epoch | bits,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(()),
                Err(actual) => previous = actual,
            }
        }
    }
}
pub fn shared() -> Arc<SharedPreferences> {
    static SHARED: OnceLock<Arc<SharedPreferences>> = OnceLock::new();
    SHARED
        .get_or_init(|| Arc::new(SharedPreferences::default()))
        .clone()
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u16,
    preferences: Preferences,
}
const MAX_BYTES: u64 = 1024;
fn decode(bytes: &[u8]) -> Result<Preferences> {
    ensure!(
        bytes.len() as u64 <= MAX_BYTES,
        "preferences file exceeds bound"
    );
    let d: Document = serde_json::from_slice(bytes)?;
    ensure!(d.schema == 1, "unknown preferences schema");
    d.preferences.validate()
}
fn encode(p: Preferences) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(&Document {
        schema: 1,
        preferences: p.validate()?,
    })?)
}
fn local_path(path: &Path) -> bool {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return false;
    }
    #[cfg(windows)]
    {
        use std::path::Prefix;
        matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_)))
    }
    #[cfg(not(windows))]
    {
        true
    }
}
fn production_parent() -> Result<PathBuf> {
    // Dedicated per-user app directory, never a fallback to cwd/shared/TW.
    let parent =
        PathBuf::from(std::env::var_os("LOCALAPPDATA").context("LocalAppData unavailable")?);
    ensure!(
        local_path(&parent),
        "LocalAppData is not a local absolute path"
    );
    Ok(parent)
}
fn pin(path: &Path) -> Result<Vec<File>> {
    use super::resources::{reject_reparse_chain, verify_open_path};
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetDriveTypeW(root: *const u16) -> u32;
        }
        let root = path
            .ancestors()
            .last()
            .context("preferences drive absent")?;
        let wide: Vec<_> = root.as_os_str().encode_wide().chain(Some(0)).collect();
        ensure!(
            unsafe { GetDriveTypeW(wide.as_ptr()) } != 4,
            "network preferences drive"
        );
    }
    reject_reparse_chain(path)?;
    let mut guards = Vec::new();
    for p in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
        if p.as_os_str().is_empty() {
            continue;
        }
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // Allow changes to children, but no rename/delete of our pinned
            // directory chain. Resource-reader sharing is deliberately stricter.
            options
                .custom_flags(0x0020_0000 | 0x0200_0000)
                .share_mode(3);
        }
        let file = options.open(p)?;
        verify_open_path(&file, p, true)?;
        guards.push(file);
    }
    Ok(guards)
}
fn entry_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}
fn read(parent: &Path) -> Result<Preferences> {
    ensure!(local_path(parent), "unsafe preferences parent");
    let _parents = pin(parent).context("pin load parent")?;
    let directory = parent.join("LiToModern");
    if !entry_exists(&directory)? {
        return Ok(Preferences::default());
    }
    let _directory = pin(&directory)?;
    let path = directory.join("preferences.json");
    if !entry_exists(&path)? {
        return Ok(Preferences::default());
    }
    let file = super::resources::open_readonly(&path, false)?;
    super::resources::verify_open_path(&file, &path, false)?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    decode(&bytes)
}
fn write(parent: &Path, p: Preferences) -> Result<()> {
    ensure!(local_path(parent), "unsafe preferences parent");
    let bytes = encode(p)?;
    let _parents = pin(parent).context("pin save parent")?;
    let directory = parent.join("LiToModern");
    match fs::create_dir(&directory) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(anyhow::Error::new(e).context("create settings directory")),
    }
    let _directory = pin(&directory).context("pin settings directory")?;
    let destination = directory.join("preferences.json");
    if entry_exists(&destination)? {
        let current = super::resources::open_readonly(&destination, false)?;
        super::resources::verify_open_path(&current, &destination, false)?;
    }
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let mut owned = None;
    for _ in 0..16 {
        let path = directory.join(format!(
            ".preferences-{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                owned = Some((path, file));
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(anyhow::Error::new(e).context("create settings staging file")),
        }
    }
    let (path, mut file) = owned.context("preferences staging collision limit")?;
    let result = (|| -> Result<()> {
        file.write_all(&bytes)
            .context("write settings staging file")?;
        file.sync_all().context("flush settings staging file")?;
        drop(file);
        // Atomic replacement changes only our app entry, not a hardlink target.
        // Pinned directories cannot be redirected while this worker writes.
        fs::rename(&path, &destination).context("replace settings destination")?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&path);
    } // exact create_new-owned temporary only
    result
}

enum Operation {
    Load,
    Save(Preferences),
}
/// HWND owner polls this single owned worker. Applying affects only this app;
/// persistence failures are visible and are never called a successful save.
pub struct PreferenceSession {
    pub values: Arc<SharedPreferences>,
    parent: Option<PathBuf>,
    job: Option<JoinHandle<Result<Option<Preferences>>>>,
    loading: bool,
    ready: bool,
    closing: bool,
    status: &'static str,
}
impl PreferenceSession {
    pub fn new() -> Result<Self> {
        Self::start(shared(), None)
    }
    /// Native acceptance must never load or save the operator's real profile.
    /// Use the production codec/worker in a newly owned project-target parent.
    #[cfg(test)]
    pub(crate) fn isolated_test(values: Arc<SharedPreferences>) -> Result<Self> {
        static NEXT_TEST: AtomicU64 = AtomicU64::new(1);
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/ui-preferences");
        ensure!(
            local_path(&root),
            "test preferences require a local project root"
        );
        let _target = pin(root.parent().context("test target parent")?)?;
        if !entry_exists(&root)? {
            fs::create_dir(&root)?;
        }
        let _test_root = pin(&root)?;
        let parent = root.join(format!(
            "{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
            NEXT_TEST.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&parent)?;
        let session = Self::start(values, Some(parent.clone()))?;
        eprintln!("UI_TEST_PREFERENCES parent={}", parent.display());
        Ok(session)
    }
    fn start(values: Arc<SharedPreferences>, parent: Option<PathBuf>) -> Result<Self> {
        let mut s = Self {
            values,
            parent,
            job: None,
            loading: false,
            ready: false,
            closing: false,
            status: "讀取現代 LiTo 個人設定中…",
        };
        s.spawn(Operation::Load)?;
        Ok(s)
    }
    fn spawn(&mut self, operation: Operation) -> Result<()> {
        ensure!(
            self.job.is_none() && !self.closing,
            "preferences worker is busy"
        );
        self.loading = matches!(operation, Operation::Load);
        let parent = self.parent.clone();
        self.job = Some(
            std::thread::Builder::new()
                .name("lito-preferences".into())
                .spawn(move || {
                    let parent = parent.map(Ok).unwrap_or_else(production_parent)?;
                    match operation {
                        Operation::Load => Ok(Some(read(&parent)?)),
                        Operation::Save(p) => {
                            write(&parent, p)?;
                            Ok(None)
                        }
                    }
                })?,
        );
        Ok(())
    }
    pub fn poll(&mut self) {
        if self.job.as_ref().is_none_or(|j| !j.is_finished()) {
            return;
        }
        let result = self
            .job
            .take()
            .unwrap()
            .join()
            .unwrap_or_else(|_| Err(anyhow::anyhow!("preferences worker panicked")));
        self.ready = true;
        self.status = match result {
            Ok(Some(p)) => {
                let _ = self.values.apply(p);
                "已載入現代 LiTo 個人設定。"
            }
            Ok(None) => "已套用並保存；不修改 TW／Windows 裝置音量。",
            Err(_) if self.loading => "設定讀取失敗；目前值仍有效。可套用以保存新的個人設定。",
            Err(_) => "已在本程式套用，但保存失敗；重新啟動後可能不保留。",
        };
    }
    pub fn apply(&mut self, p: Preferences) -> Result<()> {
        ensure!(self.can_apply(), "preferences are not ready");
        self.values.apply(p)?;
        if let Err(e) = self.spawn(Operation::Save(p)) {
            self.status = "已在本程式套用，但無法啟動保存工作。";
            return Err(e);
        }
        self.status = "已套用，正在保存現代 LiTo 個人設定…";
        Ok(())
    }
    pub fn can_apply(&self) -> bool {
        self.ready && self.job.is_none() && !self.closing
    }
    pub fn status(&self) -> &'static str {
        self.status
    }
    pub fn close(&mut self) {
        self.closing = true;
    }
    pub fn is_quiescent(&self) -> bool {
        self.job.is_none()
    }
}
impl Drop for PreferenceSession {
    fn drop(&mut self) {
        // Exceptional unwind only; normal windows keep pumping until poll retires it.
        if let Some(job) = self.job.take() {
            let _ = job.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_version_detects_settings_aba_without_changing_persisted_schema() {
        let shared = SharedPreferences::default();
        let initial = shared.snapshot();
        shared.apply(initial.values).unwrap();
        assert_eq!(shared.snapshot(), initial);
        shared
            .apply(Preferences {
                sound_enabled: false,
                ..initial.values
            })
            .unwrap();
        let muted = shared.snapshot();
        shared.apply(initial.values).unwrap();
        let restored = shared.snapshot();
        assert_eq!(restored.values, initial.values);
        assert_eq!(muted.revision, initial.revision + 1);
        assert_eq!(restored.revision, initial.revision + 2);
        assert_eq!(
            encode(initial.values).unwrap(),
            encode(restored.values).unwrap()
        );
        shared.0.store(
            (u64::MAX & !1023) | u64::from(initial.values.packed()),
            Ordering::Release,
        );
        let last = shared.snapshot();
        assert!(shared
            .apply(Preferences {
                sound_enabled: false,
                ..last.values
            })
            .is_err());
        assert_eq!(
            shared.snapshot(),
            last,
            "exhaustion cannot wrap to an old stamp"
        );
    }
    #[test]
    fn strict_preferences_roundtrip_and_atomic_flags_preserve_volume_while_muted() {
        let values = SharedPreferences::default();
        for percent in [0, 1, 50, 99, 100] {
            for sound in [false, true] {
                for background in [false, true] {
                    let p = Preferences {
                        sound_enabled: sound,
                        volume_percent: percent,
                        background_sound: background,
                    };
                    assert_eq!(decode(&encode(p).unwrap()).unwrap(), p);
                    values.apply(p).unwrap();
                    assert_eq!(values.get(), p);
                    assert_eq!(p.audible_gain(true), if sound { percent } else { 0 });
                    assert_eq!(
                        p.audible_gain(false),
                        if sound && background { percent } else { 0 }
                    );
                }
            }
        }
        let previous = values.get();
        assert!(values
            .apply(Preferences {
                volume_percent: 101,
                ..previous
            })
            .is_err());
        assert_eq!(values.get(), previous);
        for doc in [
            r#"{"schema":2,"preferences":{"sound_enabled":true,"volume_percent":50,"background_sound":false}}"#,
            r#"{"schema":1,"preferences":{"sound_enabled":true,"volume_percent":101,"background_sound":false}}"#,
            r#"{"schema":1,"preferences":{"sound_enabled":true,"volume_percent":50,"background_sound":false,"password":"x"}}"#,
            r#"{"schema":1,"preferences":{"sound_enabled":true,"volume_percent":50}}"#,
        ] {
            assert!(decode(doc.as_bytes()).is_err());
        }
        assert!(decode(&vec![b' '; 1025]).is_err());
    }
    fn wait(s: &mut PreferenceSession) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !s.is_quiescent() {
            assert!(std::time::Instant::now() < deadline);
            s.poll();
            std::thread::yield_now();
        }
    }
    #[test]
    fn native_acceptance_preferences_use_an_owned_parent_and_real_persistence() {
        let values = Arc::new(SharedPreferences::default());
        let mut session = PreferenceSession::isolated_test(values.clone()).unwrap();
        let parent = session
            .parent
            .clone()
            .expect("must not use production_parent");
        assert!(
            parent.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/ui-preferences"))
        );
        wait(&mut session);
        assert!(session.can_apply());
        assert!(
            !parent.join("LiToModern").exists(),
            "read does not create a profile"
        );
        let desired = Preferences {
            volume_percent: 37,
            sound_enabled: false,
            background_sound: false,
        };
        session.apply(desired).unwrap();
        session.close();
        wait(&mut session);
        assert_eq!(values.get(), desired);
        assert!(session.status().contains("已套用並保存"));
        assert_eq!(read(&parent).unwrap(), desired);
        let reloaded = Arc::new(SharedPreferences::default());
        let mut again = PreferenceSession::start(reloaded.clone(), Some(parent.clone())).unwrap();
        wait(&mut again);
        assert_eq!(reloaded.get(), desired);
        again.close();
        drop(again);
        drop(session);
        fs::remove_file(parent.join("LiToModern/preferences.json")).unwrap();
        fs::remove_dir(parent.join("LiToModern")).unwrap();
        fs::remove_dir(parent).unwrap();
    }
    #[test]
    fn owned_worker_load_apply_reload_and_corrupt_file_do_not_touch_neighbor_tw_settings() {
        let parent = std::env::temp_dir().join(format!(
            "lito-preferences-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&parent).unwrap();
        let neighbor = parent.join("lineage.cfg");
        fs::write(&neighbor, b"TW sentinel").unwrap();
        let original = Arc::new(SharedPreferences::default());
        let mut session = PreferenceSession::start(original.clone(), Some(parent.clone())).unwrap();
        assert!(!session.can_apply());
        wait(&mut session);
        assert!(
            !parent.join("LiToModern").exists(),
            "load never creates settings"
        );
        let p = Preferences {
            sound_enabled: false,
            volume_percent: 37,
            background_sound: false,
        };
        session.apply(p).unwrap();
        assert_eq!(original.get(), p);
        assert!(!session.can_apply());
        session.close();
        wait(&mut session);
        assert!(!session.can_apply());
        assert!(
            session.status().contains("已套用並保存"),
            "status={}",
            session.status()
        );
        assert_eq!(read(&parent).unwrap(), p);
        write(&parent, Preferences::default()).unwrap();
        assert_eq!(read(&parent).unwrap(), Preferences::default());
        let path = parent.join("LiToModern/preferences.json");
        fs::write(&path, b"bad JSON").unwrap();
        assert!(read(&parent).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"bad JSON");
        assert_eq!(fs::read(&neighbor).unwrap(), b"TW sentinel");
        assert_eq!(fs::read_dir(parent.join("LiToModern")).unwrap().count(), 1);
        drop(session);
        fs::remove_file(path).unwrap();
        fs::remove_dir(parent.join("LiToModern")).unwrap();
        fs::remove_file(neighbor).unwrap();
        fs::remove_dir(parent).unwrap();
    }
    #[test]
    fn unsafe_paths_and_missing_parent_do_not_create_a_fallback_directory() {
        assert!(read(Path::new("relative")).is_err());
        assert!(write(Path::new("relative"), Preferences::default()).is_err());
        assert!(!local_path(Path::new("C:/some/../where")));
        let path = std::env::temp_dir().join("lito-preferences-parent-that-does-not-exist");
        assert!(!path.exists());
        assert!(read(&path).is_err());
        assert!(write(&path, Preferences::default()).is_err());
        assert!(!path.exists());
    }
    #[test]
    fn busy_worker_close_is_nonblocking_and_cannot_accept_or_replay_another_apply() {
        let values = Arc::new(SharedPreferences::default());
        let (release, receiver) = std::sync::mpsc::channel();
        let mut session = PreferenceSession {
            values: values.clone(),
            parent: None,
            job: Some(std::thread::spawn(move || {
                receiver.recv().unwrap();
                Ok(None)
            })),
            loading: false,
            ready: true,
            closing: false,
            status: "test worker",
        };
        let attempted = Preferences {
            volume_percent: 37,
            ..Preferences::default()
        };
        assert!(session.apply(attempted).is_err());
        session.poll();
        assert!(!session.is_quiescent());
        session.close();
        assert!(!session.is_quiescent());
        assert!(!session.can_apply());
        assert_eq!(values.get(), Preferences::default());
        release.send(()).unwrap();
        wait(&mut session);
        assert!(session.is_quiescent());
        assert!(session.apply(attempted).is_err());
        assert_eq!(values.get(), Preferences::default());
    }
}
