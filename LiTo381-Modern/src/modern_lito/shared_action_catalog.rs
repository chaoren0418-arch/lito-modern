//! A single, worker-owned shared Text/list.spr acquisition for one world load.
//!
//! This is not a cache or a certificate that every action can be rendered.
//! Every consumer must still qualify its own actions, aliases, cues and assets.
//! The original Text archive retains its readonly root/IDX/PAK handles until
//! this context is dropped after all consumers finish. No cursor is exposed.
//! This pins those files, not an atomic generation of every SPR/WAV in the root.

#[cfg(test)]
use super::resources::ResourceProtection;
use super::{
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    sprite_actions::{ActionError, ActionLimits, SpriteActionCatalog},
};
use sha2::{Digest, Sha256};
use std::{
    fmt,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

const MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug)]
pub enum SharedCatalogError {
    Cancelled,
    Bound(&'static str),
    Resource(ResourceError),
    Action(ActionError),
}
impl fmt::Display for SharedCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "shared action catalog loading cancelled"),
            Self::Bound(reason) => write!(f, "shared action catalog exceeds {reason} bound"),
            Self::Resource(error) => write!(f, "shared action catalog resource: {error}"),
            Self::Action(error) => write!(f, "shared action catalog parse: {error}"),
        }
    }
}
impl std::error::Error for SharedCatalogError {}
impl From<ResourceError> for SharedCatalogError {
    fn from(error: ResourceError) -> Self {
        Self::Resource(error)
    }
}
impl From<ActionError> for SharedCatalogError {
    fn from(error: ActionError) -> Self {
        Self::Action(error)
    }
}

/// Opaque, non-cloneable, per-load context. Only the checked shared reader can
/// construct it; callers cannot pair a forged digest/catalog with another root.
pub struct SharedActionCatalog {
    archive: SharedArchive,
    catalog: SpriteActionCatalog,
    byte_length: usize,
    sha256: [u8; 32],
}
impl SharedActionCatalog {
    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self, SharedCatalogError> {
        Self::load_with_byte_limit(root,cancel,MAX_BYTES)
    }
    /// Renderer acceptance retains the original ActionLimits byte bound.
    pub(crate) fn load_for_model(root:&Path,cancel:&AtomicBool)->Result<Self,SharedCatalogError> {
        Self::load_with_byte_limit(root,cancel,ActionLimits::default().max_bytes)
    }
    fn load_with_byte_limit(root:&Path,cancel:&AtomicBool,max_bytes:usize)->Result<Self,SharedCatalogError> {
        #[cfg(test)]
        test_counts::increment(|counts| &mut counts.load_attempts);
        active(cancel)?;
        let mut archive = SharedArchive::open(root, SharedArchiveKind::Text)?;
        let expected = archive.entry("list.spr")?.size as usize;
        if expected == 0 || expected > max_bytes {
            return Err(SharedCatalogError::Bound("action catalog bytes"));
        }
        active(cancel)?;
        #[cfg(test)]
        test_counts::increment(|counts| &mut counts.reads);
        let bytes = archive.read_entry("list.spr")?;
        active(cancel)?;
        // Both the digest and parser consume this one owned, decoded byte read.
        if bytes.len() != expected {
            return Err(SharedCatalogError::Resource(ResourceError::MalformedIndex(
                "catalog length changed during read",
            )));
        }
        let sha256 = Sha256::digest(&bytes).into();
        #[cfg(test)]
        test_counts::increment(|counts| &mut counts.parses);
        let catalog = SpriteActionCatalog::parse_for_inspection(
            &bytes,
            ActionLimits {
                max_bytes,
                ..Default::default()
            },
        )?;
        drop(bytes);
        active(cancel)?;
        Ok(Self {
            archive,
            catalog,
            byte_length: expected,
            sha256,
        })
    }

    /// A lazy slot owned by one Model, dropped with that Model. No global or
    /// cross-root reuse. The original context keeps Text/IDX/PAK handles pinned.
    pub(crate) fn reuse_for_model<'a>(
        root: &Path, cancel: &AtomicBool, slot: &'a mut Option<Self>,
    ) -> Result<&'a Self, SharedCatalogError> {
        active(cancel)?;
        if let Some(context) = slot.as_ref() {
            let metadata=std::fs::symlink_metadata(root).map_err(ResourceError::from)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(ResourceError::UnsafePath(root.to_path_buf()).into());
            }
            #[cfg(windows)] {
                use std::os::windows::fs::MetadataExt;
                if metadata.file_attributes() & 0x400 != 0 {
                    return Err(ResourceError::UnsafePath(root.to_path_buf()).into());
                }
            }
            let actual=root.canonicalize().map_err(ResourceError::from)?;
            if actual != context.root() {
                return Err(ResourceError::UnsafePath(root.to_path_buf()).into());
            }
        } else {
            *slot=Some(Self::load(root,cancel)?);
        }
        active(cancel)?;
        let context=slot.as_ref().expect("successful per-Model catalog acquisition");
        // Effects retain their narrower existing bound, even when the renderer
        // accepted a larger catalog. Never widen an effect's admissible input.
        if context.byte_length()==0 || context.byte_length()>MAX_BYTES {
            return Err(SharedCatalogError::Bound("action catalog bytes"));
        }
        Ok(context)
    }

    pub fn root(&self) -> &Path {
        self.archive.root()
    }
    pub fn catalog(&self) -> &SpriteActionCatalog {
        &self.catalog
    }
    pub fn sha256(&self) -> [u8; 32] {
        self.sha256
    }
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }
    pub fn index_records(&self) -> usize {
        self.archive.len()
    }
    #[cfg(test)]
    pub fn protection(&self) -> ResourceProtection {
        self.archive.protection()
    }
}

fn active(cancel: &AtomicBool) -> Result<(), SharedCatalogError> {
    if cancel.load(Ordering::Acquire) {
        Err(SharedCatalogError::Cancelled)
    } else {
        Ok(())
    }
}

/// Counts attempts on the calling loader thread only; not a process-wide cache,
/// timing measurement, or production diagnostic. Reset only when that thread
/// has no active acquisition. Reads/parses count attempted operations.
#[cfg(test)]
pub(crate) mod test_counts {
    use std::cell::Cell;
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub(crate) struct Counts {
        pub load_attempts: usize,
        pub reads: usize,
        pub parses: usize,
    }
    thread_local! { static COUNTS: Cell<Counts> = Cell::new(Counts::default()); }
    pub(crate) fn snapshot() -> Counts {
        COUNTS.with(Cell::get)
    }
    pub(crate) fn reset() {
        COUNTS.with(|counts| counts.set(Counts::default()));
    }
    pub(super) fn increment(field: impl FnOnce(&mut Counts) -> &mut usize) {
        COUNTS.with(|cell| {
            let mut counts = cell.get();
            let value = field(&mut counts);
            *value = value
                .checked_add(1)
                .expect("test acquisition count overflow");
            cell.set(counts);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::test_counts::{self, Counts};
    use super::*;
    use std::{fs, io, path::PathBuf, sync::atomic::AtomicU64};

    // Independently encoded with the existing C# PakTools.Encode by the root
    // verifier. These are synthetic archives, never copied from a player root.
    const PLAIN: &[u8] = b"#1 1 test\n0.idle(0 1,0.0:4)\n";
    const PAK: &str = "09650BCC2968E3DA04AA72DD594AD0EF7D51C9B8E3D09AB83A34290A";
    const INDEX: &str = "010000008AAEA2508B4613F002E191124BC40D32B2057E3B1D78581F1C000000";
    const DUPLICATE_INDEX: &str = "020000008AAEA2508B4613F002E191124BC40D32B2057E3B1D78581F1B2A3486149FA7B83E01029976E881A3B2057E3B1D78581F5451AB5F4547CDBF";
    const SHA: &str = "7EDEC02EB15A25327233812A270CB0F7B1EF0D068AB4CE5A9BD6E3CA872D343A";

    fn hex(value: &str) -> Vec<u8> {
        assert_eq!(value.len() % 2, 0);
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let digit = |b: u8| (b as char).to_digit(16).unwrap() as u8;
                digit(pair[0]) * 16 + digit(pair[1])
            })
            .collect()
    }

    struct Fixture {
        root: PathBuf,
        parent: PathBuf,
    }
    impl Fixture {
        fn new(index: &[u8], pak: &[u8]) -> Self {
            static SERIAL: AtomicU64 = AtomicU64::new(0);
            let parent = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target/current/modern-lito-shared-catalog-tests");
            fs::create_dir_all(&parent).unwrap();
            let parent = fs::canonicalize(parent).unwrap();
            loop {
                let root = parent.join(format!(
                    "fixture-{}-{}",
                    std::process::id(),
                    SERIAL.fetch_add(1, Ordering::Relaxed)
                ));
                match fs::create_dir(&root) {
                    Ok(()) => {
                        let fixture = Self { root, parent };
                        fs::write(fixture.root.join("Text.idx"), index).unwrap();
                        fs::write(fixture.root.join("Text.pak"), pak).unwrap();
                        return fixture;
                    }
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("create isolated catalog fixture: {error}"),
                }
            }
        }
        fn valid() -> Self {
            Self::new(&hex(INDEX), &hex(PAK))
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            // A supplied resource root is never wrapped by this fixture. Check
            // this exclusively created leaf again before recursive cleanup.
            if self.root.parent() == Some(self.parent.as_path())
                && self
                    .root
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("fixture-"))
                && fs::canonicalize(&self.root).is_ok_and(|path| path == self.root)
                && fs::symlink_metadata(&self.root).is_ok_and(|metadata| {
                    #[cfg(windows)]
                    {
                        use std::os::windows::fs::MetadataExt;
                        metadata.is_dir() && metadata.file_attributes() & 0x400 == 0
                    }
                    #[cfg(not(windows))]
                    {
                        metadata.is_dir() && !metadata.file_type().is_symlink()
                    }
                })
            {
                let _ = fs::remove_dir_all(&self.root);
            }
        }
    }

    #[test]
    fn same_decoded_bytes_supply_digest_catalog_and_readonly_provenance_once() {
        let fixture = Fixture::valid();
        test_counts::reset();
        let context = SharedActionCatalog::load(&fixture.root, &AtomicBool::new(false)).unwrap();
        assert_eq!(context.root(), fs::canonicalize(&fixture.root).unwrap());
        assert_eq!(context.byte_length(), PLAIN.len());
        assert_eq!(context.index_records(), 1);
        assert_eq!(context.protection(), ResourceProtection::L1ProtectedOld28);
        assert_eq!(context.sha256().as_slice(), hex(SHA));
        let independently_hashed: [u8; 32] = Sha256::digest(PLAIN).into();
        assert_eq!(context.sha256(), independently_hashed);
        assert_eq!(context.catalog().sprites.len(), 1);
        let animation = context
            .catalog()
            .resolve_action(1, super::super::sprite_actions::ActionKey::new(0))
            .unwrap()
            .animation;
        assert_eq!(animation.frames.len(), 1);
        assert_eq!(animation.frames[0].duration_ticks, 4);
        for _ in 0..8 {
            assert!(std::ptr::eq(context.catalog(), context.catalog()));
            assert_eq!(context.sha256().as_slice(), hex(SHA));
        }
        assert_eq!(
            test_counts::snapshot(),
            Counts {
                load_attempts: 1,
                reads: 1,
                parses: 1
            }
        );
    }

    #[test]
    fn ambiguous_real_encrypted_index_fails_before_any_payload_read_or_parse() {
        let fixture = Fixture::new(&hex(DUPLICATE_INDEX), &hex(PAK));
        test_counts::reset();
        assert!(matches!(
            SharedActionCatalog::load(&fixture.root, &AtomicBool::new(false)),
            Err(SharedCatalogError::Resource(ResourceError::AmbiguousEntry { name, count: 2 })) if name == "list.spr"
        ));
        assert_eq!(
            test_counts::snapshot(),
            Counts {
                load_attempts: 1,
                reads: 0,
                parses: 0
            }
        );
    }

    #[test]
    fn cancellation_precedes_root_access_and_keeps_its_typed_error() {
        test_counts::reset();
        assert!(matches!(
            SharedActionCatalog::load(
                Path::new("not-a-shared-catalog-root"),
                &AtomicBool::new(true)
            ),
            Err(SharedCatalogError::Cancelled)
        ));
        assert_eq!(
            test_counts::snapshot(),
            Counts {
                load_attempts: 1,
                reads: 0,
                parses: 0
            }
        );
    }

    #[test]
    fn catalog_byte_bounds_are_checked_before_read_and_parse() {
        // One old28 record is 28 bytes: after three encrypted eight-byte
        // blocks its final u32 length remains in the original short tail.
        for size in [0, MAX_BYTES as u32 + 1] {
            let mut index = hex(INDEX);
            index[28..32].copy_from_slice(&size.to_le_bytes());
            let fixture = Fixture::new(&index, &[]);
            fs::OpenOptions::new()
                .write(true)
                .open(fixture.root.join("Text.pak"))
                .unwrap()
                .set_len(u64::from(size))
                .unwrap();
            test_counts::reset();
            assert!(matches!(
                SharedActionCatalog::load(&fixture.root, &AtomicBool::new(false)),
                Err(SharedCatalogError::Bound(_))
            ));
            assert_eq!(
                test_counts::snapshot(),
                Counts {
                    load_attempts: 1,
                    reads: 0,
                    parses: 0
                }
            );
        }
    }

    #[test]
    fn malformed_catalog_is_action_error_and_failure_releases_the_archive() {
        let mut index = hex(INDEX);
        index[28..32].copy_from_slice(&1u32.to_le_bytes());
        let fixture = Fixture::new(&index, b"x"); // Short payload is unciphered.
        test_counts::reset();
        assert!(matches!(
            SharedActionCatalog::load(&fixture.root, &AtomicBool::new(false)),
            Err(SharedCatalogError::Action(_))
        ));
        assert_eq!(
            test_counts::snapshot(),
            Counts {
                load_attempts: 1,
                reads: 1,
                parses: 1
            }
        );
        for name in ["Text.idx", "Text.pak"] {
            fs::OpenOptions::new()
                .write(true)
                .open(fixture.root.join(name))
                .unwrap();
        }
    }

    #[test]
    fn distinct_roots_and_loads_never_share_a_global_catalog_object() {
        let first = Fixture::valid();
        let mut changed = hex(PAK);
        // The four-byte plaintext tail changes only the valid action tick4->5.
        let tick = changed.len() - 3;
        assert_eq!(changed[tick], b'4');
        changed[tick] = b'5';
        let second = Fixture::new(&hex(INDEX), &changed);
        test_counts::reset();
        let a = SharedActionCatalog::load(&first.root, &AtomicBool::new(false)).unwrap();
        let b = SharedActionCatalog::load(&second.root, &AtomicBool::new(false)).unwrap();
        assert_ne!(a.root(), b.root());
        assert_ne!(a.sha256(), b.sha256());
        assert!(!std::ptr::eq(a.catalog(), b.catalog()));
        assert_eq!(
            b.catalog()
                .resolve_action(1, super::super::sprite_actions::ActionKey::new(0))
                .unwrap()
                .animation
                .frames[0]
                .duration_ticks,
            5
        );
        assert_eq!(
            test_counts::snapshot(),
            Counts {
                load_attempts: 2,
                reads: 2,
                parses: 2
            }
        );
    }

    #[cfg(windows)]
    #[test]
    fn live_context_pins_root_index_and_pak_until_context_drop() {
        let fixture = Fixture::valid();
        let context = SharedActionCatalog::load(&fixture.root, &AtomicBool::new(false)).unwrap();
        for name in ["Text.idx", "Text.pak"] {
            let file = fixture.root.join(name);
            assert!(fs::OpenOptions::new().write(true).open(&file).is_err());
            assert!(fs::remove_file(&file).is_err());
            assert!(fs::rename(&file, fixture.root.join(format!("{name}.renamed"))).is_err());
        }
        let renamed = fixture.parent.join(format!(
            "{}-renamed",
            fixture.root.file_name().unwrap().to_string_lossy()
        ));
        assert!(fs::rename(&fixture.root, &renamed).is_err());
        assert_eq!(context.sha256().as_slice(), hex(SHA));
        drop(context);
        for name in ["Text.idx", "Text.pak"] {
            fs::OpenOptions::new()
                .write(true)
                .open(fixture.root.join(name))
                .unwrap();
        }
    }
    #[test]
    fn model_slot_reads_and_parses_once_then_releases_pinned_files() {
        let fixture=Fixture::valid();let cancel=AtomicBool::new(false);let mut slot=None;test_counts::reset();
        let a=SharedActionCatalog::reuse_for_model(&fixture.root,&cancel,&mut slot).unwrap().catalog() as *const _;
        for _ in 0..8 {let b=SharedActionCatalog::reuse_for_model(&fixture.root,&cancel,&mut slot).unwrap();assert_eq!(a,b.catalog() as *const _);assert_eq!(b.sha256().as_slice(),hex(SHA));}
        assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:1,parses:1});
        #[cfg(windows)] for name in ["Text.idx","Text.pak"] {assert!(fs::OpenOptions::new().write(true).open(fixture.root.join(name)).is_err());}
        drop(slot);
        for name in ["Text.idx","Text.pak"] {fs::OpenOptions::new().write(true).open(fixture.root.join(name)).unwrap();}
    }
    #[test]
    fn model_slot_cached_cancellation_precedes_missing_root_access() {
        let fixture=Fixture::valid();let mut slot=None;let cancel=AtomicBool::new(false);test_counts::reset();
        SharedActionCatalog::reuse_for_model(&fixture.root,&cancel,&mut slot).unwrap();
        assert!(matches!(SharedActionCatalog::reuse_for_model(Path::new("missing-cancelled-root"),&AtomicBool::new(true),&mut slot),Err(SharedCatalogError::Cancelled)));
        assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:1,parses:1});
        assert_eq!(slot.as_ref().unwrap().sha256().as_slice(),hex(SHA));
    }
    #[test]
    fn model_slot_refuses_cross_root_reuse_and_retains_original_owner() {
        let a=Fixture::valid();let b=Fixture::valid();let cancel=AtomicBool::new(false);let mut slot=None;test_counts::reset();
        SharedActionCatalog::reuse_for_model(&a.root,&cancel,&mut slot).unwrap();
        assert!(matches!(SharedActionCatalog::reuse_for_model(&b.root,&cancel,&mut slot),Err(SharedCatalogError::Resource(ResourceError::UnsafePath(_)))));
        assert_eq!(slot.as_ref().unwrap().root(),a.root);assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:1,parses:1});
        drop(slot);let mut next=None;SharedActionCatalog::reuse_for_model(&b.root,&cancel,&mut next).unwrap();
        assert_eq!(next.as_ref().unwrap().root(),b.root);assert_eq!(test_counts::snapshot(),Counts{load_attempts:2,reads:2,parses:2});
    }
    #[test]
    fn model_slot_ambiguous_failure_never_publishes_a_cached_context() {
        let fixture=Fixture::new(&hex(DUPLICATE_INDEX),&hex(PAK));let mut slot=None;test_counts::reset();
        assert!(matches!(SharedActionCatalog::reuse_for_model(&fixture.root,&AtomicBool::new(false),&mut slot),Err(SharedCatalogError::Resource(ResourceError::AmbiguousEntry{count:2,..}))));
        assert!(slot.is_none());assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:0,parses:0});
    }

    #[test]
    fn renderer_context_is_reused_once_without_widening_effect_acceptance() {
        let fixture=Fixture::valid();let cancel=AtomicBool::new(false);test_counts::reset();
        let mut slot=Some(SharedActionCatalog::load_for_model(&fixture.root,&cancel).unwrap());
        SharedActionCatalog::reuse_for_model(&fixture.root,&cancel,&mut slot).unwrap();
        assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:1,parses:1});
        // Exercise the narrower consumer boundary on an otherwise valid owner.
        slot.as_mut().unwrap().byte_length=MAX_BYTES+1;
        assert!(matches!(SharedActionCatalog::reuse_for_model(&fixture.root,&cancel,&mut slot),Err(SharedCatalogError::Bound(_))));
        assert!(slot.is_some());assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:1,parses:1});
    }
    #[test]
    fn renderer_original_byte_limit_rejects_before_payload_read_or_parse() {
        let size=ActionLimits::default().max_bytes as u32+1;
        let mut index=hex(INDEX);index[28..32].copy_from_slice(&size.to_le_bytes());
        let fixture=Fixture::new(&index,&[]);
        fs::OpenOptions::new().write(true).open(fixture.root.join("Text.pak")).unwrap().set_len(u64::from(size)).unwrap();
        test_counts::reset();
        assert!(matches!(SharedActionCatalog::load_for_model(&fixture.root,&AtomicBool::new(false)),Err(SharedCatalogError::Bound(_))));
        assert_eq!(test_counts::snapshot(),Counts{load_attempts:1,reads:0,parses:0});
    }

}
