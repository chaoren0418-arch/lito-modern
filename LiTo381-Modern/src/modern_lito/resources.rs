//! Read-only, archive-qualified access to shared Lineage resources.
//!
//! This module does not load LiTo/LT* archives, interpret graphics, or launch a
//! game. Sprite/Tile accept the observed unprotected old28 indexes. Shared
//! Text.idx/Text.pak use the separately tested L1 cipher from PakViewer; its
//! protected index body and each payload are decoded at their own boundaries.

use super::resource_cipher::{decode_payload_in_place, L1IndexReader};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

const RECORD_BYTES: u64 = 28;
const HARD_MAX_ENTRIES: u32 = 1_000_000;
const HARD_MAX_ENTRY_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_SHARED_MAP_SECTOR_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_SHARED_SOUND_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_SHARED_TW_ACTION_PACK_BYTES: u64 = 16 * 1024 * 1024;

/// Explicit shared TW data source, never an arbitrary filename or LT fallback.
/// This reads data only; it neither invokes TW injection nor changes its files.
pub(crate) fn read_shared_tw_action_pack(root: &Path) -> Result<Vec<u8>, ResourceError> {
    read_shared_loose_file(
        root,
        &[],
        "TW13081901.pak",
        MAX_SHARED_TW_ACTION_PACK_BYTES,
        |size| ResourceError::EntryTooLarge {
            name: "TW13081901.pak".into(),
            size,
            limit: MAX_SHARED_TW_ACTION_PACK_BYTES,
        },
    )
}

/// Explicit launcher TW catalog namespace used by the current shared profile.
/// This is a selected data source, never a fallback from a missing root pack.
pub(crate) fn read_shared_launcher_tw_action_pack(root: &Path) -> Result<Vec<u8>, ResourceError> {
    read_shared_loose_file(
        root,
        &["launcher".to_owned()],
        "TW13081901.pak",
        MAX_SHARED_TW_ACTION_PACK_BYTES,
        |size| ResourceError::EntryTooLarge {
            name: "launcher/TW13081901.pak".into(),
            size,
            limit: MAX_SHARED_TW_ACTION_PACK_BYTES,
        },
    )
}

/// The shared client stores numeric effect IDs as `Sound/<id>.wav`.
/// This deliberately cannot select MIDI/music or an arbitrary loose path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedSoundKey {
    pub sound_id: u32,
}

impl SharedSoundKey {
    pub fn file_name(self) -> String {
        format!("{}.wav", self.sound_id)
    }
}

/// Read one shared sound effect, not an LT sound or an extension fallback.
/// Format recognition belongs to `audio::parse_pcm_wav`, not the extension.
pub fn read_shared_sound(
    root: impl AsRef<Path>,
    key: SharedSoundKey,
) -> Result<Vec<u8>, ResourceError> {
    read_shared_sound_with_limit(root, key, MAX_SHARED_SOUND_BYTES)
}

/// Apply a caller's remaining sound-cache budget before allocating or reading
/// the payload. The limit must be positive and cannot exceed the hard ceiling;
/// this never truncates a sound to fit the available budget.
pub fn read_shared_sound_with_limit(
    root: impl AsRef<Path>,
    key: SharedSoundKey,
    max_bytes: u64,
) -> Result<Vec<u8>, ResourceError> {
    if max_bytes == 0 || max_bytes > MAX_SHARED_SOUND_BYTES {
        return Err(ResourceError::InvalidLimits);
    }
    read_shared_loose_file(
        root.as_ref(),
        &["Sound".to_owned()],
        &key.file_name(),
        max_bytes,
        |size| ResourceError::SoundTooLarge {
            size,
            limit: max_bytes,
        },
    )
}

/// A typed request into the shared `map` namespace. There is no arbitrary
/// filename/extension or LT-map fallback in this interface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedMapSectorKey {
    pub map_id: u32,
    pub sector_x: u16,
    pub sector_y: u16,
}

impl SharedMapSectorKey {
    pub fn file_name(self) -> String {
        format!("{:04x}{:04x}.s32", self.sector_x, self.sector_y)
    }

    /// Coordinate mapping from MapSegInfo, independent of the numeric map ID.
    pub fn world_origin(self) -> (i32, i32) {
        (
            32704 + (i32::from(self.sector_x) - 0x7fff) * 64,
            32704 + (i32::from(self.sector_y) - 0x7fff) * 64,
        )
    }
}

/// Read exactly one shared S32 sector, with a hard 8 MiB bound. Directory and
/// data handles use the same confinement/reparse/sharing checks as archives.
/// This does not enumerate a map, read SEG substitutes, or consult LT assets.
pub fn read_shared_map_sector(
    root: impl AsRef<Path>,
    key: SharedMapSectorKey,
) -> Result<Vec<u8>, ResourceError> {
    read_shared_map_sector_with_limit(root, key, MAX_SHARED_MAP_SECTOR_BYTES)
}
/// Enforce the caller's manifest/input budget before payload allocation. This
/// never truncates a file and keeps the same confined, owned read-only handle.
pub fn read_shared_map_sector_with_limit(
    root: impl AsRef<Path>,
    key: SharedMapSectorKey,
    max_bytes: u64,
) -> Result<Vec<u8>, ResourceError> {
    if max_bytes == 0 || max_bytes > MAX_SHARED_MAP_SECTOR_BYTES {
        return Err(ResourceError::InvalidLimits);
    }
    read_shared_loose_file(
        root.as_ref(),
        &["map".to_owned(), key.map_id.to_string()],
        &key.file_name(),
        max_bytes,
        |size| ResourceError::SectorTooLarge {
            size,
            limit: max_bytes,
        },
    )
}

// Private: callers above construct every component from fixed namespaces and
// numeric keys. Never expose this as an arbitrary loose-file API.
fn read_shared_loose_file(
    supplied_root: &Path,
    subdirectories: &[String],
    filename: &str,
    max_bytes: u64,
    oversized: impl FnOnce(u64) -> ResourceError,
) -> Result<Vec<u8>, ResourceError> {
    reject_root_components(supplied_root)?;
    let absolute = if supplied_root.is_absolute() {
        supplied_root.to_owned()
    } else {
        std::env::current_dir()?.join(supplied_root)
    };
    reject_reparse_chain(&absolute)?;
    let root = fs::canonicalize(absolute)?;
    reject_root_components(&root)?;
    let mut path = root;
    let mut directories = Vec::with_capacity(1 + subdirectories.len());
    for component in std::iter::once(None).chain(subdirectories.iter().map(Some)) {
        if let Some(component) = component {
            path.push(component);
        }
        reject_reparse_chain(&path)?;
        let handle = open_readonly(&path, true)?;
        verify_open_path(&handle, &path, true)?;
        directories.push((path.clone(), handle));
    }
    path.push(filename);
    reject_reparse_chain(&path)?;
    let mut file = open_readonly(&path, false)?;
    verify_open_path(&file, &path, false)?;
    for (path, handle) in &directories {
        verify_open_path(handle, path, true)?;
    }
    let length = file.metadata()?.len();
    if length > max_bytes {
        return Err(oversized(length));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length as usize)
        .map_err(|_| ResourceError::AllocationFailed)?;
    bytes.resize(length as usize, 0);
    file.read_exact(&mut bytes)?;
    if file.read(&mut [0u8; 1])? != 0 {
        return Err(ResourceError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "shared loose resource changed size during bounded read",
        )));
    }
    Ok(bytes)
}

/// Selection is explicit: an entry in one archive never silently overrides
/// an entry with the same name in another archive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SharedArchiveKind {
    Sprite,
    SpriteShard(u8),
    Tile,
    Text,
}

impl SharedArchiveKind {
    pub fn file_names(self) -> Result<(String, String), ResourceError> {
        let stem = match self {
            Self::Sprite => "Sprite".to_owned(),
            Self::SpriteShard(shard @ 0..=15) => format!("Sprite{shard:02}"),
            Self::SpriteShard(shard) => return Err(ResourceError::InvalidShard(shard)),
            Self::Tile => "Tile".to_owned(),
            Self::Text => "Text".to_owned(),
        };
        Ok((format!("{stem}.idx"), format!("{stem}.pak")))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceProtection {
    PlainOld28,
    /// Shared Text only; sourced from PakViewer's OldL1Handler/PakTools.
    L1ProtectedOld28,
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceLimits {
    pub max_entries: u32,
    pub max_entry_bytes: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_entries: 250_000,
            max_entry_bytes: 64 * 1024 * 1024,
        }
    }
}

impl ResourceLimits {
    fn validate(self) -> Result<Self, ResourceError> {
        if self.max_entries == 0
            || self.max_entries > HARD_MAX_ENTRIES
            || self.max_entry_bytes == 0
            || self.max_entry_bytes > HARD_MAX_ENTRY_BYTES
        {
            return Err(ResourceError::InvalidLimits);
        }
        Ok(self)
    }
}

#[derive(Debug)]
pub enum ResourceError {
    Io(io::Error),
    InvalidShard(u8),
    InvalidLimits,
    UnsafePath(PathBuf),
    InvalidEntryName,
    MalformedIndex(&'static str),
    TooManyEntries(u32),
    DuplicateEntry(String),
    AmbiguousEntry {
        name: String,
        count: usize,
    },
    EntryTooLarge {
        name: String,
        size: u64,
        limit: u64,
    },
    EntryOutOfBounds {
        name: String,
        offset: u64,
        size: u64,
        pak_size: u64,
    },
    MissingEntry(String),
    AllocationFailed,
    SectorTooLarge {
        size: u64,
        limit: u64,
    },
    SoundTooLarge {
        size: u64,
        limit: u64,
    },
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "shared resource I/O: {error}"),
            Self::InvalidShard(shard) => write!(f, "Sprite shard {shard} is outside 00..15"),
            Self::InvalidLimits => write!(f, "resource limits exceed the bounded reader contract"),
            Self::UnsafePath(path) => write!(
                f,
                "resource path is not a confined regular file/directory: {}",
                path.display()
            ),
            Self::InvalidEntryName => write!(
                f,
                "entry name must be a safe ASCII basename of at most 20 bytes"
            ),
            Self::MalformedIndex(reason) => write!(f, "invalid old28 index: {reason}"),
            Self::TooManyEntries(count) => {
                write!(f, "index count {count} exceeds the configured limit")
            }
            Self::DuplicateEntry(name) => write!(f, "duplicate case-insensitive entry: {name}"),
            Self::AmbiguousEntry { name, count } => write!(
                f,
                "entry {name} has {count} records in the selected archive; no implicit winner is permitted"
            ),
            Self::EntryTooLarge { name, size, limit } => {
                write!(f, "entry {name} size {size} exceeds limit {limit}")
            }
            Self::EntryOutOfBounds {
                name,
                offset,
                size,
                pak_size,
            } => write!(
                f,
                "entry {name} range {offset}+{size} exceeds PAK size {pak_size}"
            ),
            Self::MissingEntry(name) => {
                write!(f, "entry is absent from the selected archive: {name}")
            }
            Self::AllocationFailed => write!(f, "unable to allocate the bounded entry buffer"),
            Self::SectorTooLarge { size, limit } => {
                write!(f, "shared map sector size {size} exceeds limit {limit}")
            }
            Self::SoundTooLarge { size, limit } => {
                write!(f, "shared sound size {size} exceeds limit {limit}")
            }
        }
    }
}

impl std::error::Error for ResourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for ResourceError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceEntry {
    /// ASCII lowercase key; lookup applies the same normalization.
    pub name: String,
    pub offset: u64,
    pub size: u32,
}

/// A verified index plus the same read-only PAK handle used for all reads.
/// Windows handles deny write/delete sharing for the lifetime of the archive.
/// `&mut self` serializes seeks; callers can open separate archive objects for
/// independent threads instead of sharing an unsynchronized cursor.
pub struct SharedArchive {
    kind: SharedArchiveKind,
    root: PathBuf,
    entries: BTreeMap<String, Vec<ResourceEntry>>,
    record_count: usize,
    pak: File,
    _index: File,
    _root_directory: File,
    selected_names: Option<BTreeSet<String>>,
}

impl SharedArchive {
    pub fn open(root: impl AsRef<Path>, kind: SharedArchiveKind) -> Result<Self, ResourceError> {
        Self::open_with_limits(root, kind, ResourceLimits::default())
    }

    pub fn open_with_limits(
        root: impl AsRef<Path>,
        kind: SharedArchiveKind,
        limits: ResourceLimits,
    ) -> Result<Self, ResourceError> {
        Self::open_index(root.as_ref(), kind, limits, None)
    }

    /// Retain only these names, but validate all plain Sprite index records,
    /// including duplicates and malformed unselected entries. Queries outside
    /// this explicit set fail rather than being reported as archive misses.
    pub fn open_for_names(
        root: impl AsRef<Path>,
        kind: SharedArchiveKind,
        names: &[&str],
        cancel: &AtomicBool,
    ) -> Result<Self, ResourceError> {
        if !matches!(
            kind,
            SharedArchiveKind::Sprite | SharedArchiveKind::SpriteShard(_)
        ) || names.is_empty()
            || names.len() > 64
        {
            return Err(ResourceError::InvalidLimits);
        }
        let names = names
            .iter()
            .map(|n| normalize_name_with_policy(n.as_bytes(), false))
            .collect::<Result<BTreeSet<_>, _>>()?;
        Self::open_index(
            root.as_ref(),
            kind,
            ResourceLimits::default(),
            Some((names, cancel)),
        )
    }

    fn open_index(
        supplied_root: &Path,
        kind: SharedArchiveKind,
        limits: ResourceLimits,
        selection: Option<(BTreeSet<String>, &AtomicBool)>,
    ) -> Result<Self, ResourceError> {
        if let Some((_, cancel)) = &selection {
            check_index_cancel(cancel)?;
        }
        let (idx_name, pak_name) = kind.file_names()?;
        let limits = limits.validate()?;
        reject_root_components(supplied_root)?;
        let absolute_root = if supplied_root.is_absolute() {
            supplied_root.to_owned()
        } else {
            std::env::current_dir()?.join(supplied_root)
        };
        reject_reparse_chain(&absolute_root)?;
        let root = fs::canonicalize(&absolute_root)?;
        reject_root_components(&root)?;
        let root_directory = open_readonly(&root, true)?;
        verify_open_path(&root_directory, &root, true)?;

        let idx_path = root.join(idx_name);
        let pak_path = root.join(pak_name);
        reject_reparse_chain(&idx_path)?;
        reject_reparse_chain(&pak_path)?;
        let index = open_readonly(&idx_path, false)?;
        let pak = open_readonly(&pak_path, false)?;
        verify_open_path(&index, &idx_path, false)?;
        verify_open_path(&pak, &pak_path, false)?;
        // Recheck the root identity after both child handles are opened.
        verify_open_path(&root_directory, &root, true)?;

        let index_size = index.metadata()?.len();
        let pak_size = pak.metadata()?.len();
        let entries = if let Some((names, cancel)) = &selection {
            parse_selected_old28(
                BufReader::new(&index),
                index_size,
                pak_size,
                limits,
                names,
                cancel,
            )?
        } else if kind == SharedArchiveKind::Text {
            parse_old28(
                L1IndexReader::new(BufReader::new(&index)),
                index_size,
                pak_size,
                limits,
                true,
            )?
        } else {
            parse_old28(BufReader::new(&index), index_size, pak_size, limits, false)?
        };
        let record_count = entries.values().map(Vec::len).sum();
        Ok(Self {
            kind,
            root,
            entries,
            record_count,
            pak,
            _index: index,
            _root_directory: root_directory,
            selected_names: selection.map(|(names, _)| names),
        })
    }

    pub fn kind(&self) -> SharedArchiveKind {
        self.kind
    }
    pub fn protection(&self) -> ResourceProtection {
        if self.kind == SharedArchiveKind::Text {
            ResourceProtection::L1ProtectedOld28
        } else {
            ResourceProtection::PlainOld28
        }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Retained index records, including explicitly preserved Text duplicates.
    /// `open_for_names` retains only the explicitly selected set.
    pub fn len(&self) -> usize {
        self.record_count
    }
    /// Number of distinct case-insensitive names. Text can have more records
    /// than names; any ambiguous lookup is explicitly rejected.
    pub fn unique_name_count(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// Every record, grouped in deterministic case-insensitive name order.
    /// Duplicate records retain their order within the original Text index.
    pub fn entries(&self) -> impl Iterator<Item = &ResourceEntry> {
        self.entries.values().flat_map(|records| records.iter())
    }

    pub fn entry(&self, name: &str) -> Result<&ResourceEntry, ResourceError> {
        let key =
            normalize_name_with_policy(name.as_bytes(), self.kind == SharedArchiveKind::Text)?;
        if self
            .selected_names
            .as_ref()
            .is_some_and(|names| !names.contains(&key))
        {
            return Err(ResourceError::InvalidLimits);
        }
        match self.entries.get(&key) {
            None => Err(ResourceError::MissingEntry(key)),
            Some(records) if records.len() == 1 => Ok(&records[0]),
            Some(records) => Err(ResourceError::AmbiguousEntry {
                name: key,
                count: records.len(),
            }),
        }
    }

    /// Reads exactly the indexed byte range. Truncation is an error; there is
    /// no 64 KiB clipping, adjacent-entry read or loose-file fallback. Shared
    /// Text entries are L1-decoded in the same bounded buffer, with no expansion.
    pub fn read_entry(&mut self, name: &str) -> Result<Vec<u8>, ResourceError> {
        let entry = self.entry(name)?;
        let (offset, length) = (entry.offset, entry.size as usize);
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| ResourceError::AllocationFailed)?;
        bytes.resize(length, 0);
        self.pak.seek(SeekFrom::Start(offset))?;
        self.pak.read_exact(&mut bytes)?;
        if self.kind == SharedArchiveKind::Text {
            decode_payload_in_place(&mut bytes);
        }
        Ok(bytes)
    }

    /// Tile-only bounded sub-entry I/O on the same pinned plain PAK handle.
    /// Never exposes protected Text ranges or permits reading a neighboring entry.
    pub(crate) fn read_tile_range_into(
        &mut self,
        name: &str,
        relative: usize,
        output: &mut [u8],
    ) -> Result<(), ResourceError> {
        if self.kind != SharedArchiveKind::Tile || output.is_empty() {
            return Err(ResourceError::InvalidLimits);
        }
        let entry = self.entry(name)?;
        let end = relative
            .checked_add(output.len())
            .ok_or(ResourceError::InvalidLimits)?;
        if end > entry.size as usize {
            return Err(ResourceError::InvalidLimits);
        }
        let start = entry
            .offset
            .checked_add(relative as u64)
            .ok_or(ResourceError::InvalidLimits)?;
        self.pak.seek(SeekFrom::Start(start))?;
        self.pak.read_exact(output)?;
        Ok(())
    }
}

#[cfg(test)]
fn normalize_name(bytes: &[u8]) -> Result<String, ResourceError> {
    normalize_name_with_policy(bytes, false)
}

fn normalize_name_with_policy(
    bytes: &[u8],
    text_internal_spaces: bool,
) -> Result<String, ResourceError> {
    validate_name_with_policy(bytes, text_internal_spaces)?;
    Ok(bytes
        .iter()
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect())
}
fn validate_name_with_policy(
    bytes: &[u8],
    text_internal_spaces: bool,
) -> Result<(), ResourceError> {
    if bytes.is_empty()
        || bytes.len() > 20
        || bytes[0] == b'.'
        || bytes.first() == Some(&b' ')
        || bytes.last() == Some(&b' ')
        || bytes.iter().any(|byte| {
            !byte.is_ascii_alphanumeric()
                && !matches!(byte, b'_' | b'-' | b'.')
                && !(text_internal_spaces && *byte == b' ')
        })
    {
        return Err(ResourceError::InvalidEntryName);
    }
    Ok(())
}

fn check_index_cancel(cancel: &AtomicBool) -> Result<(), ResourceError> {
    if cancel.load(Ordering::Acquire) {
        return Err(ResourceError::Io(io::Error::new(
            io::ErrorKind::Interrupted,
            "selected index cancelled",
        )));
    }
    Ok(())
}

fn parse_selected_old28(
    mut reader: impl Read,
    index_size: u64,
    pak_size: u64,
    limits: ResourceLimits,
    selected: &BTreeSet<String>,
    cancel: &AtomicBool,
) -> Result<BTreeMap<String, Vec<ResourceEntry>>, ResourceError> {
    check_index_cancel(cancel)?;
    let limits = limits.validate()?;
    if index_size < 4 {
        return Err(ResourceError::MalformedIndex("truncated count"));
    }
    let mut count_bytes = [0; 4];
    reader.read_exact(&mut count_bytes)?;
    if matches!(&count_bytes, b"_IDX" | b"_EXT") || &count_bytes[..3] == b"RMS" {
        return Err(ResourceError::MalformedIndex(
            "protected or extended index is unsupported",
        ));
    }
    let count = u32::from_le_bytes(count_bytes);
    if count > limits.max_entries {
        return Err(ResourceError::TooManyEntries(count));
    }
    if index_size != 4 + u64::from(count) * RECORD_BYTES {
        return Err(ResourceError::MalformedIndex(
            "count does not match the exact file length",
        ));
    }
    let keys: HashSet<[u8; 20]> = selected
        .iter()
        .map(|name| {
            let mut key = [0; 20];
            key[..name.len()].copy_from_slice(name.as_bytes());
            key
        })
        .collect();
    let mut seen = HashSet::<[u8; 20]>::new();
    seen.try_reserve(count as usize)
        .map_err(|_| ResourceError::AllocationFailed)?;
    let mut entries = BTreeMap::new();
    for ordinal in 0..count {
        if ordinal % 256 == 0 {
            check_index_cancel(cancel)?;
        }
        let mut record = [0; 28];
        reader.read_exact(&mut record)?;
        let raw = &record[4..24];
        let end = raw.iter().position(|b| *b == 0).unwrap_or(20);
        if raw[end..].iter().any(|b| *b != 0) {
            return Err(ResourceError::MalformedIndex(
                "nonzero bytes after filename terminator",
            ));
        }
        validate_name_with_policy(&raw[..end], false)?;
        let mut key = [0; 20];
        for (dst, src) in key[..end].iter_mut().zip(&raw[..end]) {
            *dst = src.to_ascii_lowercase();
        }
        let name = || std::str::from_utf8(&key[..end]).unwrap().to_owned();
        let offset = u64::from(u32::from_le_bytes(record[..4].try_into().unwrap()));
        let size = u32::from_le_bytes(record[24..].try_into().unwrap());
        if u64::from(size) > limits.max_entry_bytes {
            return Err(ResourceError::EntryTooLarge {
                name: name(),
                size: u64::from(size),
                limit: limits.max_entry_bytes,
            });
        }
        if offset + u64::from(size) > pak_size {
            return Err(ResourceError::EntryOutOfBounds {
                name: name(),
                offset,
                size: u64::from(size),
                pak_size,
            });
        }
        if !seen.insert(key) {
            return Err(ResourceError::DuplicateEntry(name()));
        }
        if keys.contains(&key) {
            let name = name();
            entries.insert(name.clone(), vec![ResourceEntry { name, offset, size }]);
        }
    }
    if reader.read(&mut [0; 1])? != 0 {
        return Err(ResourceError::MalformedIndex("trailing bytes"));
    }
    check_index_cancel(cancel)?;
    Ok(entries)
}

fn parse_old28(
    mut reader: impl Read,
    index_size: u64,
    pak_size: u64,
    limits: ResourceLimits,
    text_policy: bool,
) -> Result<BTreeMap<String, Vec<ResourceEntry>>, ResourceError> {
    let limits = limits.validate()?;
    if index_size < 4 {
        return Err(ResourceError::MalformedIndex("truncated count"));
    }
    let mut count_bytes = [0; 4];
    reader.read_exact(&mut count_bytes)?;
    // _IDX/_EXT/RMS are not the old28 format, even if bytes look like a count.
    if matches!(&count_bytes, b"_IDX" | b"_EXT") || &count_bytes[..3] == b"RMS" {
        return Err(ResourceError::MalformedIndex(
            "protected or extended index is unsupported",
        ));
    }
    let count = u32::from_le_bytes(count_bytes);
    if count > limits.max_entries {
        return Err(ResourceError::TooManyEntries(count));
    }
    let expected = 4 + u64::from(count) * RECORD_BYTES;
    if index_size != expected {
        return Err(ResourceError::MalformedIndex(
            "count does not match the exact file length",
        ));
    }
    let mut entries: BTreeMap<String, Vec<ResourceEntry>> = BTreeMap::new();
    for _ in 0..count {
        let mut record = [0; RECORD_BYTES as usize];
        reader.read_exact(&mut record)?;
        let name_bytes = &record[4..24];
        let end = name_bytes.iter().position(|byte| *byte == 0).unwrap_or(20);
        if name_bytes[end..].iter().any(|byte| *byte != 0) {
            return Err(ResourceError::MalformedIndex(
                "nonzero bytes after filename terminator",
            ));
        }
        let name = normalize_name_with_policy(&name_bytes[..end], text_policy)?;
        let offset = u64::from(u32::from_le_bytes(record[0..4].try_into().unwrap()));
        let size = u32::from_le_bytes(record[24..28].try_into().unwrap());
        if u64::from(size) > limits.max_entry_bytes {
            return Err(ResourceError::EntryTooLarge {
                name,
                size: u64::from(size),
                limit: limits.max_entry_bytes,
            });
        }
        // Promoting the unsigned old28 fields before addition prevents u32 wrap.
        if offset + u64::from(size) > pak_size {
            return Err(ResourceError::EntryOutOfBounds {
                name,
                offset,
                size: u64::from(size),
                pak_size,
            });
        }
        let entry = ResourceEntry {
            name: name.clone(),
            offset,
            size,
        };
        let records = entries.entry(name.clone()).or_default();
        if !text_policy && !records.is_empty() {
            return Err(ResourceError::DuplicateEntry(name));
        }
        records.push(entry);
    }
    // Also detects an index changed while being read on platforms that permit it.
    let mut trailing = [0];
    if reader.read(&mut trailing)? != 0 {
        return Err(ResourceError::MalformedIndex("trailing bytes"));
    }
    Ok(entries)
}

fn reject_root_components(path: &Path) -> Result<(), ResourceError> {
    for component in path.components() {
        match component {
            Component::ParentDir => return Err(ResourceError::UnsafePath(path.to_owned())),
            Component::Normal(name) => {
                let name = name.to_string_lossy();
                if [
                    "lito", "ltmap", "lttext", "ltsound", "ltsurf", "lttile", "ltsprite", "lticon",
                ]
                .iter()
                .any(|blocked| name.eq_ignore_ascii_case(blocked))
                {
                    return Err(ResourceError::UnsafePath(path.to_owned()));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn is_link(metadata: &Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // Includes junctions and other reparse types, not only symlinks.
        return metadata.file_attributes() & 0x400 != 0;
    }
    #[cfg(not(windows))]
    false
}

pub(crate) fn reject_reparse_chain(path: &Path) -> Result<(), ResourceError> {
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        if is_link(&fs::symlink_metadata(ancestor)?) {
            return Err(ResourceError::UnsafePath(path.to_owned()));
        }
    }
    Ok(())
}

pub(crate) fn open_readonly(path: &Path, directory: bool) -> Result<File, ResourceError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // OPEN_REPARSE_POINT avoids following a replaced final component;
        // BACKUP_SEMANTICS opens directories. No write/delete sharing.
        options.custom_flags(0x0020_0000 | if directory { 0x0200_0000 } else { 0 });
        options.share_mode(0x0000_0001);
    }
    #[cfg(not(windows))]
    let _ = directory;
    Ok(options.open(path)?)
}

pub(crate) fn verify_open_path(
    file: &File,
    expected: &Path,
    directory: bool,
) -> Result<(), ResourceError> {
    let metadata = file.metadata()?;
    if is_link(&metadata)
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        return Err(ResourceError::UnsafePath(expected.to_owned()));
    }
    reject_reparse_chain(expected)?;
    #[cfg(windows)]
    {
        let actual = final_handle_path(file)?;
        let canonical = fs::canonicalize(expected)?;
        if actual != canonical {
            return Err(ResourceError::UnsafePath(expected.to_owned()));
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let at_path = fs::metadata(expected)?;
        if metadata.dev() != at_path.dev() || metadata.ino() != at_path.ino() {
            return Err(ResourceError::UnsafePath(expected.to_owned()));
        }
    }
    Ok(())
}

#[cfg(windows)]
pub(crate) fn final_handle_path(file: &File) -> Result<PathBuf, ResourceError> {
    use std::ffi::{c_void, OsString};
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::io::AsRawHandle;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFinalPathNameByHandleW(
            handle: *mut c_void,
            output: *mut u16,
            size: u32,
            flags: u32,
        ) -> u32;
    }
    let mut output = vec![0u16; 32_768];
    // SAFETY: a live owned File supplies the handle and the writable buffer has
    // exactly the stated length. Windows does not retain the output pointer.
    let written = unsafe {
        GetFinalPathNameByHandleW(
            file.as_raw_handle(),
            output.as_mut_ptr(),
            output.len() as u32,
            0,
        )
    };
    if written == 0 {
        return Err(io::Error::last_os_error().into());
    }
    if written as usize >= output.len() {
        return Err(ResourceError::MalformedIndex("resolved path is too long"));
    }
    output.truncate(written as usize);
    Ok(PathBuf::from(OsString::from_wide(&output)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn index(records: &[(&str, u32, u32)]) -> Vec<u8> {
        let mut bytes = (records.len() as u32).to_le_bytes().to_vec();
        for (name, offset, size) in records {
            let mut row = [0u8; 28];
            row[..4].copy_from_slice(&offset.to_le_bytes());
            row[4..4 + name.len()].copy_from_slice(name.as_bytes());
            row[24..].copy_from_slice(&size.to_le_bytes());
            bytes.extend(row);
        }
        bytes
    }

    fn parse(
        bytes: &[u8],
        pak_size: u64,
    ) -> Result<BTreeMap<String, ResourceEntry>, ResourceError> {
        parse_old28(
            Cursor::new(bytes),
            bytes.len() as u64,
            pak_size,
            ResourceLimits::default(),
            false,
        )
        .map(|entries| {
            entries
                .into_iter()
                .map(|(name, mut records)| (name, records.remove(0)))
                .collect()
        })
    }

    struct Fixture(PathBuf, PathBuf);
    impl Fixture {
        fn new() -> Self {
            static SERIAL: AtomicU64 = AtomicU64::new(0);
            let base = option_env!("CARGO_MANIFEST_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap());
            let parent = base.join("target").join("modern-lito-resource-tests");
            fs::create_dir_all(&parent).unwrap();
            let parent = fs::canonicalize(parent).unwrap();
            loop {
                let path = parent.join(format!(
                    "fixture-{}-{}",
                    std::process::id(),
                    SERIAL.fetch_add(1, Ordering::Relaxed)
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path, parent),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("create isolated resource fixture: {error}"),
                }
            }
        }
        fn archive(&self, stem: &str, bytes: &[u8], data: &[u8]) {
            fs::write(self.0.join(format!("{stem}.idx")), bytes).unwrap();
            fs::write(self.0.join(format!("{stem}.pak")), data).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            // Only an exclusively created leaf directly under the validated
            // test parent can be removed. A supplied shared root is never a
            // Fixture and never reaches this cleanup path.
            if self.0.parent() == Some(self.1.as_path())
                && self
                    .0
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("fixture-"))
                && fs::symlink_metadata(&self.0)
                    .is_ok_and(|metadata| metadata.is_dir() && !is_link(&metadata))
            {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
    }

    #[test]
    fn old28_unsigned_offsets_and_case_normalization() {
        let bytes = index(&[("Knight.SPR", 0x8000_0000, 7)]);
        let entries = parse(&bytes, 0x8000_0007).unwrap();
        assert_eq!(entries["knight.spr"].offset, 0x8000_0000);
        assert_eq!(entries["knight.spr"].size, 7);
    }

    #[test]
    fn tile_subranges_cannot_escape_the_indexed_entry_or_read_other_archives() {
        let fixture = Fixture::new();
        fixture.archive(
            "Tile",
            &index(&[("0.til", 2, 3), ("1.til", 5, 2)]),
            &[90, 91, 1, 2, 3, 99, 98],
        );
        let mut archive = SharedArchive::open(&fixture.0, SharedArchiveKind::Tile).unwrap();
        let mut pair = [0; 2];
        archive.read_tile_range_into("0.til", 1, &mut pair).unwrap();
        assert_eq!(pair, [2, 3]);
        assert!(archive.read_tile_range_into("0.til", 2, &mut pair).is_err());
        assert!(archive
            .read_tile_range_into("0.til", usize::MAX, &mut pair)
            .is_err());
        assert!(archive.read_tile_range_into("0.til", 0, &mut []).is_err());
        fixture.archive("Sprite", &index(&[("0.til", 0, 2)]), &[1, 2]);
        let mut other = SharedArchive::open(&fixture.0, SharedArchiveKind::Sprite).unwrap();
        assert!(other.read_tile_range_into("0.til", 0, &mut pair).is_err());
    }

    #[test]
    fn count_truncation_and_trailing_bytes_are_rejected() {
        assert!(matches!(
            parse(&[0, 0, 0], 0),
            Err(ResourceError::MalformedIndex(_))
        ));
        let mut bytes = index(&[("0.til", 0, 1)]);
        bytes.pop();
        assert!(matches!(
            parse(&bytes, 1),
            Err(ResourceError::MalformedIndex(_))
        ));
        let mut bytes = index(&[]);
        bytes.push(0);
        assert!(matches!(
            parse(&bytes, 0),
            Err(ResourceError::MalformedIndex(_))
        ));
        assert!(parse(&index(&[]), 0).unwrap().is_empty());
    }

    #[test]
    fn physically_short_reader_is_not_silently_accepted() {
        let bytes = index(&[("0.til", 0, 1)]);
        let result = parse_old28(
            Cursor::new(&bytes[..10]),
            bytes.len() as u64,
            1,
            ResourceLimits::default(),
            false,
        );
        assert!(
            matches!(result, Err(ResourceError::Io(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof)
        );
    }

    #[test]
    fn entry_count_and_byte_limits_are_enforced_before_allocation() {
        assert!(matches!(
            parse(&u32::MAX.to_le_bytes(), 0),
            Err(ResourceError::TooManyEntries(_))
        ));
        let bytes = index(&[("large.spr", 0, 65 * 1024 * 1024)]);
        assert!(matches!(
            parse(&bytes, u64::MAX),
            Err(ResourceError::EntryTooLarge { .. })
        ));
        let invalid = ResourceLimits {
            max_entries: u32::MAX,
            max_entry_bytes: u64::MAX,
        };
        assert!(matches!(
            parse_old28(Cursor::new(index(&[])), 4, 0, invalid, false),
            Err(ResourceError::InvalidLimits)
        ));
    }

    #[test]
    fn entry_range_must_fit_pak_without_u32_wrap() {
        let bytes = index(&[("0.til", u32::MAX, 2)]);
        assert!(matches!(
            parse(&bytes, u32::MAX as u64),
            Err(ResourceError::EntryOutOfBounds { .. })
        ));
        assert!(parse(&bytes, u32::MAX as u64 + 2).is_ok());
        assert!(matches!(
            parse(&index(&[("x.spr", 8, 4)]), 10),
            Err(ResourceError::EntryOutOfBounds { .. })
        ));
    }

    #[test]
    fn duplicate_names_are_rejected_instead_of_choosing_a_winner() {
        let bytes = index(&[("A.SPR", 0, 1), ("a.spr", 1, 1)]);
        assert!(
            matches!(parse(&bytes, 2), Err(ResourceError::DuplicateEntry(ref name)) if name == "a.spr")
        );
    }

    #[test]
    fn unsafe_names_and_nonzero_padding_are_rejected() {
        for name in [
            "../x.spr",
            "..",
            ".hidden",
            "x/y.spr",
            "x\\y.spr",
            "C:x.spr",
            "x\0y",
            "技能.spr",
            "",
        ] {
            assert!(
                normalize_name(name.as_bytes()).is_err(),
                "accepted {name:?}"
            );
        }
        let mut bytes = index(&[("a.spr", 0, 1)]);
        bytes[8 + 7] = b'X';
        assert!(matches!(
            parse(&bytes, 1),
            Err(ResourceError::MalformedIndex(_))
        ));
        let bytes = index(&[("1234567890123456.spr", 0, 1)]);
        assert!(parse(&bytes, 1).is_ok());
    }

    #[test]
    fn archive_allowlist_rejects_invalid_shards_and_unrelated_formats() {
        assert_eq!(
            SharedArchiveKind::SpriteShard(0).file_names().unwrap().0,
            "Sprite00.idx"
        );
        assert_eq!(
            SharedArchiveKind::SpriteShard(15).file_names().unwrap().1,
            "Sprite15.pak"
        );
        assert!(matches!(
            SharedArchiveKind::SpriteShard(16).file_names(),
            Err(ResourceError::InvalidShard(16))
        ));
        assert_eq!(
            SharedArchiveKind::Text.file_names().unwrap(),
            ("Text.idx".to_owned(), "Text.pak".to_owned())
        );
        for header in [b"_IDX", b"_EXT", b"RMS\0"] {
            assert!(matches!(
                parse(header, 0),
                Err(ResourceError::MalformedIndex(_))
            ));
        }
    }

    #[test]
    fn root_components_reject_lt_packages_and_parent_escape() {
        for path in [
            "client/LiTo",
            "client/LTmap",
            "client/../outside",
            "client/ltsurf/child",
        ] {
            assert!(matches!(
                reject_root_components(Path::new(path)),
                Err(ResourceError::UnsafePath(_))
            ));
        }
        assert!(reject_root_components(Path::new("client/lineage-shared")).is_ok());
    }

    #[test]
    fn bounded_reads_keep_large_entries_and_do_not_cross_into_neighbors() {
        let fixture = Fixture::new();
        let mut payload = vec![0x55; 70_001];
        payload.extend_from_slice(&[0x77, 0x88]);
        fixture.archive(
            "Sprite",
            &index(&[("large.spr", 0, 70_001), ("next.spr", 70_001, 2)]),
            &payload,
        );
        let mut archive = SharedArchive::open(&fixture.0, SharedArchiveKind::Sprite).unwrap();
        assert_eq!(archive.len(), 2);
        assert_eq!(archive.read_entry("LARGE.SPR").unwrap(), vec![0x55; 70_001]);
        assert_eq!(archive.read_entry("next.spr").unwrap(), vec![0x77, 0x88]);
        assert!(matches!(
            archive.read_entry("../next.spr"),
            Err(ResourceError::InvalidEntryName)
        ));
        assert!(matches!(
            archive.read_entry("absent.spr"),
            Err(ResourceError::MissingEntry(_))
        ));
    }

    #[test]
    fn same_name_in_different_archives_requires_explicit_selection() {
        let fixture = Fixture::new();
        fixture.archive("Sprite", &index(&[("same.img", 0, 1)]), &[11]);
        fixture.archive("Sprite00", &index(&[("same.img", 0, 1)]), &[22]);
        let mut base = SharedArchive::open(&fixture.0, SharedArchiveKind::Sprite).unwrap();
        let mut shard = SharedArchive::open(&fixture.0, SharedArchiveKind::SpriteShard(0)).unwrap();
        assert_eq!(base.read_entry("same.img").unwrap(), vec![11]);
        assert_eq!(shard.read_entry("same.img").unwrap(), vec![22]);
    }

    #[test]
    fn on_disk_invalid_range_and_truncated_index_fail_open() {
        let fixture = Fixture::new();
        fixture.archive("Tile", &index(&[("0.til", 1, 3)]), &[0, 1]);
        assert!(matches!(
            SharedArchive::open(&fixture.0, SharedArchiveKind::Tile),
            Err(ResourceError::EntryOutOfBounds { .. })
        ));
        fixture.archive("Tile", &[1, 0, 0, 0], &[]);
        assert!(matches!(
            SharedArchive::open(&fixture.0, SharedArchiveKind::Tile),
            Err(ResourceError::MalformedIndex(_))
        ));
    }

    #[test]
    fn protected_text_matches_csharp_fixture_across_record_and_payload_boundaries() {
        // C# PakTools.Encode independently generated the following fixture.
        // Two 28-byte records share one continuous eight-byte cipher stream.
        let encrypted_body = [
            0xFB, 0xA7, 0x6C, 0xD1, 0x03, 0x85, 0x48, 0x6B, 0xA4, 0x83, 0xC0, 0x9D, 0x8B, 0x7A,
            0xD0, 0x0A, 0xB2, 0x05, 0x7E, 0x3B, 0x1D, 0x78, 0x58, 0x1F, 0x02, 0x0B, 0x22, 0x2E,
            0x47, 0x4F, 0x57, 0x3E, 0x69, 0xFF, 0x81, 0x9B, 0xA0, 0x21, 0x8C, 0x2F, 0x8E, 0x8B,
            0xAE, 0x66, 0x72, 0xA0, 0x2D, 0x2F, 0xBA, 0xBD, 0x0D, 0x08, 0x8A, 0x90, 0x38, 0x87,
        ];
        let mut protected_index = 2u32.to_le_bytes().to_vec();
        protected_index.extend(encrypted_body);
        let mut payload = vec![
            0xC2, 0xE7, 0xEC, 0x5E, 0x0E, 0xA5, 0xD5, 0x54, 0x72, 0x6C, 0x64,
        ];
        // A seven-byte entry is left unchanged even though its file offset is 11.
        payload.extend_from_slice(b"second!");
        let fixture = Fixture::new();
        fixture.archive("Text", &protected_index, &payload);
        let mut archive = SharedArchive::open(&fixture.0, SharedArchiveKind::Text).unwrap();
        assert_eq!(archive.protection(), ResourceProtection::L1ProtectedOld28);
        assert_eq!(archive.len(), 2);
        assert_eq!(archive.read_entry("first.txt").unwrap(), b"hello world");
        assert_eq!(archive.read_entry("SECOND.TXT").unwrap(), b"second!");
        assert_eq!(archive.read_entry("first.txt").unwrap(), b"hello world");
        // Protected files are not silently accepted for Sprite/Tile.
        fixture.archive("Sprite", &protected_index, &payload);
        assert!(SharedArchive::open(&fixture.0, SharedArchiveKind::Sprite).is_err());
    }

    #[test]
    fn plain_or_truncated_text_is_not_mislabeled_as_valid_l1() {
        let fixture = Fixture::new();
        fixture.archive("Text", &index(&[("first.txt", 0, 11)]), b"hello world");
        assert!(SharedArchive::open(&fixture.0, SharedArchiveKind::Text).is_err());
        fixture.archive("Text", &[1, 0, 0, 0], b"hello world");
        assert!(matches!(
            SharedArchive::open(&fixture.0, SharedArchiveKind::Text),
            Err(ResourceError::MalformedIndex(_))
        ));
        fixture.archive("Text", &u32::MAX.to_le_bytes(), &[]);
        assert!(matches!(
            SharedArchive::open(&fixture.0, SharedArchiveKind::Text),
            Err(ResourceError::TooManyEntries(_))
        ));
    }

    #[test]
    fn shared_sector_key_is_numeric_and_reads_only_the_selected_sector() {
        let key = SharedMapSectorKey {
            map_id: 4,
            sector_x: 0x7ffc,
            sector_y: 0x7ff7,
        };
        assert_eq!(key.file_name(), "7ffc7ff7.s32");
        assert_eq!(key.world_origin(), (32512, 32192));
        let max = SharedMapSectorKey {
            map_id: u32::MAX,
            sector_x: u16::MAX,
            sector_y: 0,
        };
        assert_eq!(max.file_name(), "ffff0000.s32");
        let fixture = Fixture::new();
        let directory = fixture.0.join("map").join("4");
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join(key.file_name()), [1, 2, 3, 4]).unwrap();
        fs::write(directory.join("7ffc7ff8.s32"), [9, 8, 7]).unwrap();
        assert_eq!(
            read_shared_map_sector(&fixture.0, key).unwrap(),
            vec![1, 2, 3, 4]
        );
        assert!(read_shared_map_sector(&fixture.0, max).is_err());
    }

    #[test]
    fn oversized_shared_sector_is_rejected_before_payload_allocation() {
        let fixture = Fixture::new();
        let key = SharedMapSectorKey {
            map_id: 4,
            sector_x: 0x7ffc,
            sector_y: 0x7ff7,
        };
        let directory = fixture.0.join("map").join("4");
        fs::create_dir_all(&directory).unwrap();
        let file = File::create(directory.join(key.file_name())).unwrap();
        file.set_len(MAX_SHARED_MAP_SECTOR_BYTES + 1).unwrap();
        drop(file);
        assert!(matches!(
            read_shared_map_sector(&fixture.0, key),
            Err(ResourceError::SectorTooLarge { .. })
        ));
    }
    #[test]
    fn shared_sector_remaining_budget_is_checked_before_read_and_never_truncates() {
        let fixture = Fixture::new();
        let key = SharedMapSectorKey {
            map_id: 514,
            sector_x: 0x7fff,
            sector_y: 0x8000,
        };
        let directory = fixture.0.join("map/514");
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join(key.file_name()), [1, 2, 3, 4]).unwrap();
        for limit in [0, MAX_SHARED_MAP_SECTOR_BYTES + 1] {
            assert!(matches!(
                read_shared_map_sector_with_limit(&fixture.0, key, limit),
                Err(ResourceError::InvalidLimits)
            ));
        }
        assert!(matches!(
            read_shared_map_sector_with_limit(&fixture.0, key, 3),
            Err(ResourceError::SectorTooLarge { size: 4, limit: 3 })
        ));
        assert_eq!(
            read_shared_map_sector_with_limit(&fixture.0, key, 4).unwrap(),
            [1, 2, 3, 4]
        );
    }

    #[test]
    fn missing_shared_map_has_no_lt_namespace_fallback() {
        let fixture = Fixture::new();
        let key = SharedMapSectorKey {
            map_id: 564,
            sector_x: 0x7fff,
            sector_y: 0x8000,
        };
        let lt_root = fixture.0.join("LiTo");
        let directory = lt_root.join("LTmap").join("564");
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join(key.file_name()), [1, 2, 3]).unwrap();
        assert!(read_shared_map_sector(&fixture.0, key).is_err());
        assert!(matches!(
            read_shared_map_sector(lt_root, key),
            Err(ResourceError::UnsafePath(_))
        ));
    }

    #[test]
    fn shared_sound_key_reads_exactly_one_numeric_wav() {
        let key = SharedSoundKey { sound_id: 1539 };
        assert_eq!(key.file_name(), "1539.wav");
        assert_eq!(
            SharedSoundKey { sound_id: u32::MAX }.file_name(),
            "4294967295.wav"
        );
        let fixture = Fixture::new();
        let sound = fixture.0.join("Sound");
        fs::create_dir(&sound).unwrap();
        fs::write(sound.join("1539.wav"), [1, 2, 3, 4]).unwrap();
        fs::write(sound.join("1540.wav"), [8, 9]).unwrap();
        assert_eq!(read_shared_sound(&fixture.0, key).unwrap(), [1, 2, 3, 4]);
        assert!(read_shared_sound(&fixture.0, SharedSoundKey { sound_id: 2 }).is_err());
    }

    #[test]
    fn shared_sound_size_limit_applies_before_payload_allocation() {
        let fixture = Fixture::new();
        let sound = fixture.0.join("Sound");
        fs::create_dir(&sound).unwrap();
        let file = File::create(sound.join("1539.wav")).unwrap();
        file.set_len(MAX_SHARED_SOUND_BYTES + 1).unwrap();
        drop(file);
        assert!(matches!(
            read_shared_sound(&fixture.0, SharedSoundKey { sound_id: 1539 }),
            Err(ResourceError::SoundTooLarge { .. })
        ));
    }

    #[test]
    fn shared_sound_remaining_budget_is_exact_and_never_truncates() {
        let fixture = Fixture::new();
        let sound = fixture.0.join("Sound");
        fs::create_dir(&sound).unwrap();
        fs::write(sound.join("1539.wav"), [1, 2, 3, 4]).unwrap();
        let key = SharedSoundKey { sound_id: 1539 };
        assert_eq!(
            read_shared_sound_with_limit(&fixture.0, key, 4).unwrap(),
            [1, 2, 3, 4]
        );
        assert!(matches!(
            read_shared_sound_with_limit(&fixture.0, key, 3),
            Err(ResourceError::SoundTooLarge { size: 4, limit: 3 })
        ));
        assert_eq!(
            read_shared_sound_with_limit(&fixture.0, key, MAX_SHARED_SOUND_BYTES).unwrap(),
            read_shared_sound(&fixture.0, key).unwrap()
        );
    }

    #[test]
    fn invalid_sound_budget_is_rejected_before_any_path_io() {
        let fixture = Fixture::new();
        let absent = fixture.0.join("missing-root");
        let key = SharedSoundKey { sound_id: 1539 };
        for max_bytes in [0, MAX_SHARED_SOUND_BYTES + 1, u64::MAX] {
            assert!(matches!(
                read_shared_sound_with_limit(&absent, key, max_bytes),
                Err(ResourceError::InvalidLimits)
            ));
        }
    }

    #[test]
    fn shared_sound_has_no_midi_lt_or_parent_directory_fallback() {
        let fixture = Fixture::new();
        fs::create_dir(fixture.0.join("Sound")).unwrap();
        fs::write(fixture.0.join("Sound/0.mid"), [1, 2]).unwrap();
        assert!(read_shared_sound(&fixture.0, SharedSoundKey { sound_id: 0 }).is_err());
        let lt = fixture.0.join("LTSound");
        fs::create_dir(&lt).unwrap();
        fs::write(lt.join("1539.wav"), [3, 4]).unwrap();
        let key = SharedSoundKey { sound_id: 1539 };
        assert!(read_shared_sound(&fixture.0, key).is_err());
        assert!(matches!(
            read_shared_sound(&lt, key),
            Err(ResourceError::UnsafePath(_))
        ));
        // Windows canonical paths have a verbatim prefix. PathBuf::join("..")
        // normalizes those before the reader sees them, so preserve the raw
        // parent component instead of accidentally testing another root.
        let mut parent_root = fixture.0.as_os_str().to_os_string();
        parent_root.push(std::path::MAIN_SEPARATOR.to_string());
        parent_root.push("..");
        let parent_root = PathBuf::from(parent_root);
        assert!(parent_root
            .components()
            .any(|component| component == Component::ParentDir));
        let result = read_shared_sound(&parent_root, key);
        assert!(
            matches!(result, Err(ResourceError::UnsafePath(_))),
            "raw parent path {parent_root:?} produced {result:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn verbatim_join_normalizes_parent_before_the_reader_receives_it() {
        // Reproduce the original faulty test input under our own isolated
        // fixture, without consulting its shared target parent directory.
        let fixture = Fixture::new();
        let child = fixture.0.join("child");
        fs::create_dir(&child).unwrap();
        let canonical_child = fs::canonicalize(&child).unwrap();
        let joined = canonical_child.join("..");
        assert_eq!(joined, fixture.0);
        assert!(!joined
            .components()
            .any(|component| component == Component::ParentDir));
        let result = read_shared_sound(&joined, SharedSoundKey { sound_id: 1539 });
        assert!(
            matches!(&result, Err(ResourceError::Io(error)) if error.kind() == io::ErrorKind::NotFound),
            "normalized path {joined:?} produced {result:?}"
        );
        eprintln!("Verbatim join removed parent component before reader: {result:?}");
    }

    #[test]
    fn text_preserves_duplicates_without_a_winner_and_accepts_only_internal_spaces() {
        // Generated by C# PakTools.Encode for three records: same.txt,
        // SAME.TXT and two words.txt, each one byte at offsets zero/one/two.
        let encrypted_body = [
            0xD2, 0x77, 0x35, 0xF6, 0x2F, 0x7C, 0x71, 0x00, 0x0E, 0x8E, 0x27, 0xAB, 0xEC, 0x79,
            0x16, 0x07, 0xB2, 0x05, 0x7E, 0x3B, 0x1D, 0x78, 0x58, 0x1F, 0x83, 0x67, 0xA5, 0x43,
            0x49, 0xAE, 0x31, 0xDE, 0x3D, 0x5C, 0x08, 0x54, 0x73, 0x16, 0x50, 0x2F, 0xB2, 0x05,
            0x7E, 0x3B, 0x1D, 0x78, 0x58, 0x1F, 0xC6, 0xAA, 0x60, 0x96, 0x78, 0x41, 0xB9, 0xBB,
            0x10, 0xC8, 0xF3, 0xD2, 0xE9, 0x21, 0x33, 0x6A, 0xC1, 0xCB, 0x66, 0x11, 0xC9, 0xB0,
            0x01, 0xCF, 0x78, 0xBA, 0x16, 0x97, 0xBA, 0xA7, 0x4B, 0x03, 0x01, 0x00, 0x00, 0x00,
        ];
        let mut protected_index = 3u32.to_le_bytes().to_vec();
        protected_index.extend(encrypted_body);
        let fixture = Fixture::new();
        fixture.archive("Text", &protected_index, b"abc");
        let mut archive = SharedArchive::open(&fixture.0, SharedArchiveKind::Text).unwrap();
        assert_eq!(archive.len(), 3);
        assert_eq!(archive.entries().count(), 3);
        assert_eq!(archive.unique_name_count(), 2);
        assert_eq!(archive.read_entry("TWO WORDS.TXT").unwrap(), b"c");
        for name in ["same.txt", "SAME.TXT"] {
            assert!(matches!(
                archive.entry(name),
                Err(ResourceError::AmbiguousEntry { count: 2, .. })
            ));
            assert!(matches!(
                archive.read_entry(name),
                Err(ResourceError::AmbiguousEntry { count: 2, .. })
            ));
        }
        for name in [
            " two words.txt",
            "two words.txt ",
            "two\twords.txt",
            "two/words.txt",
            "two:words.txt",
        ] {
            assert!(matches!(
                archive.read_entry(name),
                Err(ResourceError::InvalidEntryName)
            ));
        }
        // Space allowance is deliberately restricted to the Text namespace.
        assert!(normalize_name(b"two words.txt").is_err());
        for name in [" leading.txt", "trailing.txt "] {
            let plain = index(&[(name, 0, 1)]);
            assert!(matches!(
                parse_old28(
                    Cursor::new(&plain),
                    plain.len() as u64,
                    1,
                    ResourceLimits::default(),
                    true
                ),
                Err(ResourceError::InvalidEntryName)
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_archive_and_root_are_rejected() {
        let fixture = Fixture::new();
        let outside = Fixture::new();
        outside.archive("Sprite", &index(&[("x.spr", 0, 1)]), &[9]);
        std::os::unix::fs::symlink(outside.0.join("Sprite.idx"), fixture.0.join("Sprite.idx"))
            .unwrap();
        fs::write(fixture.0.join("Sprite.pak"), [9]).unwrap();
        assert!(matches!(
            SharedArchive::open(&fixture.0, SharedArchiveKind::Sprite),
            Err(ResourceError::UnsafePath(_))
        ));
        std::os::unix::fs::symlink(&outside.0, fixture.0.join("linked-root")).unwrap();
        assert!(matches!(
            SharedArchive::open(fixture.0.join("linked-root"), SharedArchiveKind::Sprite),
            Err(ResourceError::UnsafePath(_))
        ));
    }

    #[cfg(windows)]
    #[test]
    fn retained_windows_handle_denies_writes_and_delete_sharing() {
        let fixture = Fixture::new();
        fixture.archive("Sprite", &index(&[("x.spr", 0, 1)]), &[9]);
        let mut archive = SharedArchive::open(&fixture.0, SharedArchiveKind::Sprite).unwrap();
        assert!(OpenOptions::new()
            .write(true)
            .open(fixture.0.join("Sprite.pak"))
            .is_err());
        assert!(fs::rename(fixture.0.join("Sprite.pak"), fixture.0.join("moved.pak")).is_err());
        assert_eq!(archive.read_entry("x.spr").unwrap(), vec![9]);
    }

    /// Requires Windows Developer Mode or permission to create symlinks.
    #[cfg(windows)]
    #[test]
    #[ignore = "requires Windows symlink creation privilege; run explicitly"]
    fn windows_reparse_root_and_archive_are_rejected() {
        let fixture = Fixture::new();
        let outside = Fixture::new();
        outside.archive("Sprite", &index(&[("x.spr", 0, 1)]), &[9]);
        std::os::windows::fs::symlink_dir(&outside.0, fixture.0.join("linked-root")).unwrap();
        assert!(matches!(
            SharedArchive::open(fixture.0.join("linked-root"), SharedArchiveKind::Sprite),
            Err(ResourceError::UnsafePath(_))
        ));
        std::os::windows::fs::symlink_file(
            outside.0.join("Sprite.idx"),
            fixture.0.join("Sprite.idx"),
        )
        .unwrap();
        fs::write(fixture.0.join("Sprite.pak"), [9]).unwrap();
        assert!(matches!(
            SharedArchive::open(&fixture.0, SharedArchiveKind::Sprite),
            Err(ResourceError::UnsafePath(_))
        ));
        fs::create_dir_all(fixture.0.join("map")).unwrap();
        std::os::windows::fs::symlink_dir(&outside.0, fixture.0.join("map").join("4")).unwrap();
        let key = SharedMapSectorKey {
            map_id: 4,
            sector_x: 0x7fff,
            sector_y: 0x8000,
        };
        assert!(matches!(
            read_shared_map_sector(&fixture.0, key),
            Err(ResourceError::UnsafePath(_))
        ));
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "requires Windows symlink creation privilege; run explicitly"]
    fn windows_reparse_sound_directory_and_file_are_rejected() {
        let fixture = Fixture::new();
        let outside = Fixture::new();
        fs::write(outside.0.join("1539.wav"), [1, 2]).unwrap();
        std::os::windows::fs::symlink_dir(&outside.0, fixture.0.join("Sound")).unwrap();
        let key = SharedSoundKey { sound_id: 1539 };
        assert!(matches!(
            read_shared_sound(&fixture.0, key),
            Err(ResourceError::UnsafePath(_))
        ));
        let fixture_file = Fixture::new();
        fs::create_dir(fixture_file.0.join("Sound")).unwrap();
        std::os::windows::fs::symlink_file(
            outside.0.join("1539.wav"),
            fixture_file.0.join("Sound/1539.wav"),
        )
        .unwrap();
        assert!(matches!(
            read_shared_sound(&fixture_file.0, key),
            Err(ResourceError::UnsafePath(_))
        ));
    }

    /// Read-only sampling, not a graphics or gameplay acceptance test.
    /// Set LITO_SHARED_RESOURCE_ROOT to the Lineage installation directory.
    #[test]
    #[ignore = "requires an explicit LITO_SHARED_RESOURCE_ROOT installation"]
    fn real_shared_indexes_and_bounded_payloads() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT")
            .expect("set LITO_SHARED_RESOURCE_ROOT to the shared Lineage root");
        let kinds = std::iter::once(SharedArchiveKind::Sprite)
            .chain((0..=15).map(SharedArchiveKind::SpriteShard))
            .chain(std::iter::once(SharedArchiveKind::Tile));
        let mut total_entries = 0usize;
        let mut sampled_bytes = 0usize;
        for kind in kinds {
            let mut archive = SharedArchive::open(Path::new(&root), kind).unwrap();
            assert!(!archive.is_empty());
            let sample = archive
                .entries()
                .find(|entry| entry.size > 0 && entry.size <= 512 * 1024)
                .expect("a bounded sample entry")
                .clone();
            let data = archive.read_entry(&sample.name).unwrap();
            assert_eq!(data.len(), sample.size as usize);
            total_entries += archive.len();
            sampled_bytes += data.len();
            eprintln!(
                "{kind:?}: {} entries; sampled {} ({} bytes)",
                archive.len(),
                sample.name,
                data.len()
            );
        }
        eprintln!("18 archives: {total_entries} entries; {sampled_bytes} sampled payload bytes");
    }

    /// Exact selected installation reference, independently decoded and parsed
    /// with the existing C# PakFile/SprListParser on 2026-09-14.
    #[test]
    #[ignore = "requires the selected LITO_SHARED_RESOURCE_ROOT Text reference"]
    fn real_shared_text_matches_csharp_reference() {
        use sha2::{Digest, Sha256};
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT")
            .expect("set LITO_SHARED_RESOURCE_ROOT to the shared Lineage root");
        let mut archive = SharedArchive::open(Path::new(&root), SharedArchiveKind::Text).unwrap();
        assert_eq!(archive.protection(), ResourceProtection::L1ProtectedOld28);
        // Separate installed reference, independently read with CSharp
        // Lin.Helper.Core 1.5.4 on 2026-10-20. Pin both complete source files;
        // retain the historical reference and all payload/ambiguity assertions.
        let installed_index = std::fs::read(Path::new(&root).join("Text.idx")).unwrap();
        let installed_pak = std::fs::read(Path::new(&root).join("Text.pak")).unwrap();
        // The 2026-10-20 index only reorders duplicate-name records. The
        // independent C# reader verified the complete 37497-record multiset.
        // Accept only either exact observed index, with the same pinned pak.
        let current_profile = matches!(format!("{:X}", Sha256::digest(&installed_index)).as_str(),
            "CF4D420DDAB51BADB9CFF20F60C9ABB2E04156404A443897FBB8A44FED909D7E"
            | "B85FAF2685CAB65357800368228ED3A5A96108E512CF883FA63AD2F74A44A91B")
            && format!("{:X}", Sha256::digest(&installed_pak))
                == "19BFE9D44C55F697D55CA23292509AEA56DBBC1F534427AFC740522081B7448C";
        let (records, names) = if current_profile { (37_497, 28_361) } else { (37_496, 28_360) };
        assert_eq!(archive.len(), records);
        assert_eq!(archive.unique_name_count(), names);
        let ambiguous_names = archive
            .entries
            .keys()
            .filter(|name| {
                matches!(
                    archive.entry(name),
                    Err(ResourceError::AmbiguousEntry { .. })
                )
            })
            .count();
        assert_eq!(ambiguous_names, 110);
        assert!(matches!(
            archive.read_entry("corruptsoulq1-e.html"),
            Err(ResourceError::AmbiguousEntry { count: 35, .. })
        ));
        let bytes = archive.read_entry("list.spr").unwrap();
        assert_eq!(bytes.len(), 4_067_417);
        let digest = format!("{:X}", Sha256::digest(&bytes));
        assert_eq!(
            digest,
            "523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127"
        );
        assert!(bytes.starts_with(b"11297 0 41210\r\n#0\t312=3225\tprince\r\n\t0.walk(1 4,0.0:4<479 0.1:4 0.2:4<478 0.3:4)\r\n"));
        let records: Vec<_> = bytes
            .split(|byte| *byte == b'\n')
            .filter(|line| line.starts_with(b"#"))
            .collect();
        assert_eq!(records.len(), 11_297);
        assert_eq!(*records.last().unwrap(), b"#11296 48=3858 hanbok woman\r");
        eprintln!(
            "Text: {} entries; list.spr {} bytes; SHA256 {digest}; {} sprite headers",
            archive.len(),
            bytes.len(),
            records.len()
        );
    }
}
