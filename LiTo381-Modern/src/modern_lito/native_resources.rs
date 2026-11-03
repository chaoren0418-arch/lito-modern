//! Explicit pinned Native inspection assets. Default-disabled, no wire codec,
//! session admission, map-ID source inference, or Shared resource fallback.
//! Completed assets own bytes/pixels only. Cancellable I/O is chunked; OS reads
//! and bounded single-entry decoders are not hard-deadline promises.
use super::{
    compositor::Canvas,
    map::SpriteEffect,
    resources::{open_readonly, reject_reparse_chain, verify_open_path, SharedMapSectorKey},
    scene::{static_draw_list, CameraRect, DrawTile},
    tile::{ClassicTileType, DecodedTile, TileLimits, ValidatedTileInput},
    world_map::{SectorInput, WorldCell, WorldMap},
    world_roofs::{self, Group, RoofMap, Shade},
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Component, Path, PathBuf},
    sync::{Arc,Mutex,MutexGuard,OnceLock,TryLockError,Weak,atomic::{AtomicBool, Ordering}},
    time::Instant,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NativeAccess {
    #[default]
    Disabled,
    PinnedInspection,
}
/// Local source contract, deliberately unrelated to unimplemented wire enums.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceNamespace {
    NativeLitoV1,
    Shared,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ArchiveRole {
    TileIndex,
    TilePak,
    SpriteIndex,
    SpritePak,
}
impl ArchiveRole {
    fn name(self) -> &'static str {
        match self {
            Self::TileIndex => "LTtile.idx",
            Self::TilePak => "LTtile.pak",
            Self::SpriteIndex => "LTsprite.idx",
            Self::SpritePak => "LTsprite.pak",
        }
    }
    fn limit(self, limits: Limits) -> usize {
        match self {
            Self::TileIndex | Self::SpriteIndex => limits.index_bytes,
            _ => limits.pak_bytes,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BytePin {
    pub bytes: u64,
    pub sha256: [u8; 32],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchivePin {
    pub role: ArchiveRole,
    pub pin: BytePin,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NativeSectorKey {
    pub map_id: u32,
    pub x: u16,
    pub y: u16,
}
impl NativeSectorKey {
    fn geometry_key(self) -> SharedMapSectorKey {
        SharedMapSectorKey {
            map_id: self.map_id,
            sector_x: self.x,
            sector_y: self.y,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SectorPin {
    pub key: NativeSectorKey,
    pub pin: BytePin,
}
/// Caller-supplied verified plan data; construction itself proves nothing.
/// Archive roles are exact and ordered. Paths come only from local source roots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeAssetPlan {
    pub namespace: ResourceNamespace,
    pub map_id: u32,
    pub canonical_sha256: [u8; 32],
    pub sectors: Vec<SectorPin>,
    pub archives: Vec<ArchivePin>,
}
#[derive(Clone, Debug)]
pub struct NativeSources {
    pub map_root: PathBuf,
    pub archive_root: PathBuf,
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub index_bytes: usize,
    pub pak_bytes: usize,
    pub entry_bytes: usize,
    pub sector_bytes: usize,
    pub map_bytes: usize,
    pub decoded_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            index_bytes: 8 << 20,
            pak_bytes: 256 << 20,
            entry_bytes: 8 << 20,
            sector_bytes: 8 << 20,
            map_bytes: 32 << 20,
            decoded_bytes: 64 << 20,
        }
    }
}
impl Limits {
    fn validate(self) -> Result<Self> {
        let hard = Self::default();
        ensure!(
            self.index_bytes > 0
                && self.index_bytes <= hard.index_bytes
                && self.pak_bytes > 0
                && self.pak_bytes <= hard.pak_bytes
                && self.entry_bytes > 0
                && self.entry_bytes <= hard.entry_bytes
                && self.sector_bytes > 0
                && self.sector_bytes <= hard.sector_bytes
                && self.map_bytes > 0
                && self.map_bytes <= hard.map_bytes
                && self.decoded_bytes > 0
                && self.decoded_bytes <= hard.decoded_bytes,
            "invalid Native limits"
        );
        Ok(self)
    }
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "Native load/render cancelled"
    );
    Ok(())
}
fn pin_valid(pin: BytePin, limit: usize) -> Result<()> {
    ensure!(
        pin.bytes > 0 && pin.bytes <= limit as u64 && pin.sha256 != [0; 32],
        "Native pin/budget invalid"
    );
    Ok(())
}
fn hash_reader(reader: &mut impl Read, size: u64, cancel: &AtomicBool) -> Result<[u8; 32]> {
    active(cancel)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut remaining = size;
    while remaining > 0 {
        active(cancel)?;
        let take = remaining.min(buffer.len() as u64) as usize;
        reader.read_exact(&mut buffer[..take])?;
        hash.update(&buffer[..take]);
        remaining -= take as u64;
    }
    active(cancel)?;
    ensure!(
        reader.read(&mut [0; 1])? == 0,
        "Native file grew during read"
    );
    active(cancel)?;
    Ok(hash.finalize().into())
}
struct PinnedFile {
    path: PathBuf,
    file: File,
    directories: Vec<(PathBuf, File)>,
    size: u64,
}
impl PinnedFile {
    fn open(
        root: &Path,
        parts: &[String],
        name: &str,
        pin: BytePin,
        limit: usize,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        pin_valid(pin, limit)?;
        Ok(Self::open_checked(root, parts, name, Some(pin), limit, cancel)?.0)
    }
    /// Observe and hash one retained, write/delete-denying handle. Callers that
    /// do not have an external expected pin must not read/hash then reopen it.
    fn observe(root: &Path, name: &str, limit: usize, cancel: &AtomicBool) -> Result<(Self, BytePin)> {
        Self::open_checked(root, &[], name, None, limit, cancel)
    }
    fn open_checked(root: &Path, parts: &[String], name: &str, expected: Option<BytePin>,
        limit: usize, cancel: &AtomicBool) -> Result<(Self, BytePin)> {
        active(cancel)?;
        ensure!(
            root.is_absolute()
                && !root
                    .components()
                    .any(|c| matches!(c, Component::ParentDir | Component::CurDir)),
            "Native root must be explicit confined absolute path"
        );
        reject_reparse_chain(root)?;
        let mut path = fs::canonicalize(root)?;
        let mut directories = Vec::new();
        for component in std::iter::once(None).chain(parts.iter().map(Some)) {
            active(cancel)?;
            if let Some(component) = component {
                path.push(component);
            }
            reject_reparse_chain(&path)?;
            let handle = open_readonly(&path, true)?;
            verify_open_path(&handle, &path, true)?;
            directories.push((path.clone(), handle));
        }
        path.push(name);
        reject_reparse_chain(&path)?;
        let file = open_readonly(&path, false)?;
        verify_open_path(&file, &path, false)?;
        let size = file.metadata()?.len();
        ensure!(size > 0 && size <= limit as u64, "Native observed byte budget");
        if let Some(pin) = expected { ensure!(size == pin.bytes, "Native size differs from pin"); }
        let mut opened = Self {
            path,
            file,
            directories,
            size,
        };
        opened.verify()?;
        let pin = BytePin { bytes: size, sha256: hash_reader(&mut opened.file, size, cancel)? };
        if let Some(expected) = expected {
            ensure!(pin == expected, "Native opened bytes differ from pin");
        }
        active(cancel)?;
        opened.verify()?;
        Ok((opened, pin))
    }
    fn verify(&self) -> Result<()> {
        verify_open_path(&self.file, &self.path, false)?;
        for (path, file) in &self.directories {
            verify_open_path(file, path, true)?;
        }
        ensure!(
            self.file.metadata()?.len() == self.size,
            "Native size drift"
        );
        Ok(())
    }
    fn read_range(&mut self, offset: u64, size: usize, cancel: &AtomicBool) -> Result<Vec<u8>> {
        active(cancel)?;
        self.verify()?;
        ensure!(
            offset
                .checked_add(size as u64)
                .is_some_and(|end| end <= self.size),
            "Native entry outside archive"
        );
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size)?;
        bytes.resize(size, 0);
        self.file.seek(SeekFrom::Start(offset))?;
        for part in bytes.chunks_mut(64 * 1024) {
            active(cancel)?;
            self.file.read_exact(part)?;
        }
        active(cancel)?;
        self.verify()?;
        Ok(bytes)
    }
}
fn parse_index(
    index: &[u8],
    pak_size: u64,
    entry_limit: usize,
    cancel: &AtomicBool,
) -> Result<BTreeMap<String, (u64, usize)>> {
    active(cancel)?;
    ensure!(index.len() >= 4, "Native index truncated");
    let count = u32::from_le_bytes(index[..4].try_into()?) as usize;
    ensure!(
        count > 0 && count <= 250_000 && index.len() == 4 + count * 28,
        "Native old28 count/length invalid"
    );
    let mut entries = BTreeMap::new();
    for record in index[4..].chunks_exact(28) {
        active(cancel)?;
        let raw = &record[4..24];
        let end = raw.iter().position(|b| *b == 0).unwrap_or(20);
        ensure!(
            end > 0 && raw[end..].iter().all(|b| *b == 0),
            "Native name padding invalid"
        );
        ensure!(
            raw[..end]
                .iter()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-')),
            "Native member name invalid"
        );
        let name = std::str::from_utf8(&raw[..end])?.to_ascii_lowercase();
        let offset = u64::from(u32::from_le_bytes(record[..4].try_into()?));
        let size = u32::from_le_bytes(record[24..].try_into()?) as usize;
        ensure!(
            size <= entry_limit
                && offset
                    .checked_add(size as u64)
                    .is_some_and(|end| end <= pak_size),
            "Native member bounds invalid"
        );
        ensure!(
            entries.insert(name, (offset, size)).is_none(),
            "duplicate Native member"
        );
    }
    Ok(entries)
}
struct NativeArchive {
    index: PinnedFile,
    pak: PinnedFile,
    entries: BTreeMap<String, (u64, usize)>,
}
/// The native BASE Sprite fallback consults LiTo/LTicon before BASE Sprite.
pub(crate) struct NativeIconArchive {
    index: PinnedFile,
    pak: PinnedFile,
    entries: BTreeMap<String, (u64, usize)>,
}
impl NativeIconArchive {
    pub fn open(root:&Path,index_pin:BytePin,pak_pin:BytePin,cancel:&AtomicBool)->Result<Self> {
        let mut index=PinnedFile::open(root,&[],"LTicon.idx",index_pin,1024*1024,cancel)?;
        let raw=index.read_range(0,index_pin.bytes as usize,cancel)?;
        let pak=PinnedFile::open(root,&[],"LTicon.pak",pak_pin,32*1024*1024,cancel)?;
        let entries=parse_index(&raw,pak.size,256*1024,cancel)?;
        Ok(Self{index,pak,entries})
    }
    pub fn read_optional(&mut self,name:&str,limit:usize,cancel:&AtomicBool)->Result<Option<Vec<u8>>> {
        active(cancel)?;
        let bytes=if let Some(&(offset,size))=self.entries.get(&name.to_ascii_lowercase()) {
            ensure!(size>0 && size<=limit,"Native icon member budget");
            Some(self.pak.read_range(offset,size,cancel)?)
        } else {None};
        self.index.verify()?;self.pak.verify()?;active(cancel)?;Ok(bytes)
    }
}
/// Qualified world IMG sources. Archive handles remain read-only and pinned.
pub(crate) struct NativeSurfaceArchive {
    index: PinnedFile,
    pak: PinnedFile,
    entries: BTreeMap<String, (u64, usize)>,
}
impl NativeSurfaceArchive {
    pub fn observe(root: &Path, cancel: &AtomicBool) -> Result<(Self, BytePin, BytePin)> {
        let (mut index, index_pin) = PinnedFile::observe(root, "LTsurf.idx", 1024 * 1024, cancel)?;
        let raw = index.read_range(0, index_pin.bytes as usize, cancel)?;
        let (pak, pak_pin) = PinnedFile::observe(root, "LTsurf.pak", 128 * 1024 * 1024, cancel)?;
        let entries = parse_index(&raw, pak.size, 8 * 1024 * 1024, cancel)?;
        Ok((Self { index, pak, entries }, index_pin, pak_pin))
    }
    pub fn open(root: &Path, index_pin: BytePin, pak_pin: BytePin, cancel: &AtomicBool) -> Result<Self> {
        let mut index = PinnedFile::open(root, &[], "LTsurf.idx", index_pin, 1024 * 1024, cancel)?;
        let raw = index.read_range(0, index_pin.bytes as usize, cancel)?;
        let pak = PinnedFile::open(root, &[], "LTsurf.pak", pak_pin, 128 * 1024 * 1024, cancel)?;
        let entries = parse_index(&raw, pak.size, 8 * 1024 * 1024, cancel)?;
        Ok(Self { index, pak, entries })
    }
    pub fn contains(&self, name: &str) -> Result<bool> {
        self.index.verify()?;
        self.pak.verify()?;
        Ok(self.entries.contains_key(&name.to_ascii_lowercase()))
    }
    pub fn read(&mut self, name: &str, limit: usize, cancel: &AtomicBool) -> Result<Vec<u8>> {
        active(cancel)?;
        let &(offset, size) = self.entries.get(&name.to_ascii_lowercase()).context("Native surface member absent")?;
        ensure!(size > 0 && size <= limit, "Native surface member budget");
        let bytes = self.pak.read_range(offset, size, cancel)?;
        self.index.verify()?;
        self.pak.verify()?;
        active(cancel)?;
        Ok(bytes)
    }
}
/// Native Text routing needs an authenticated index miss before BASE fallback.
/// A missing/corrupt archive is an error, never a miss. Fixed callers choose names.
pub(crate) fn read_native_text_optional(root: &Path, index_pin: BytePin, pak_pin: BytePin,
    member: &str, cancel: &AtomicBool) -> Result<Option<Vec<u8>>> {
    let mut idx = PinnedFile::open(root, &[], "LTtext.idx", index_pin, 1024 * 1024, cancel)?;
    let mut raw = idx.read_range(0, index_pin.bytes as usize, cancel)?;
    ensure!(raw.len() >= 4, "Native text index truncated");
    super::resource_cipher::decode_payload_in_place(&mut raw[4..]);
    let mut pak = PinnedFile::open(root, &[], "LTtext.pak", pak_pin, 1024 * 1024, cancel)?;
    let entries = parse_index(&raw, pak.size, 1024 * 1024, cancel)?;
    let bytes = if let Some(&(offset, size)) = entries.get(member) {
        ensure!(size > 0, "Native text selected member empty");
        let mut bytes = pak.read_range(offset, size, cancel)?;
        super::resource_cipher::decode_payload_in_place(&mut bytes);
        Some(bytes)
    } else { None };
    active(cancel)?; idx.verify()?; pak.verify()?;
    Ok(bytes)
}
/// Explicit Native catalog/SPR inspection. Names and pins come from a local
/// provenance contract, never from a map ID or Shared resource search.
pub(crate) fn read_native_inspection_member(
    root: &Path,
    text: bool,
    index_pin: BytePin,
    pak_pin: BytePin,
    member: &str,
    cancel: &AtomicBool,
) -> Result<Vec<u8>> {
    let (idx_name, pak_name, bound) = if text {
        ("LTtext.idx", "LTtext.pak", 1024 * 1024)
    } else {
        ("LTsprite.idx", "LTsprite.pak", 256 * 1024 * 1024)
    };
    let mut idx = PinnedFile::open(root, &[], idx_name, index_pin, 1024 * 1024, cancel)?;
    let mut raw = idx.read_range(0, index_pin.bytes as usize, cancel)?;
    if text {
        ensure!(raw.len() >= 4, "Native protected index truncated");
        super::resource_cipher::decode_payload_in_place(&mut raw[4..]);
    }
    let mut pak = PinnedFile::open(root, &[], pak_name, pak_pin, bound, cancel)?;
    let entries = parse_index(&raw, pak.size, if text {1024 * 1024} else {8 * 1024 * 1024}, cancel)?;
    let &(offset, size) = entries.get(member).context("Native inspection member absent; no fallback")?;
    ensure!(size > 0 && size <= 1024 * 1024, "Native selected inspection member empty/overbound");
    let mut bytes = pak.read_range(offset, size, cancel)?;
    if text {
        super::resource_cipher::decode_payload_in_place(&mut bytes);
    }
    active(cancel)?;
    idx.verify()?;
    pak.verify()?;
    Ok(bytes)
}
/// Catalog90/91 verification reads only the fixed protected LTlist member.
/// Exact index entry, raw and decoded hashes are all required independently.
pub(crate) fn verify_native_catalog_member(root:&Path,index_pin:BytePin,pak_pin:BytePin,
    offset:u64,member_pin:BytePin,decoded_sha:[u8;32],cancel:&AtomicBool)->Result<()> {
    pin_valid(member_pin,1024*1024)?;
    let mut idx=PinnedFile::open(root,&[],"LTtext.idx",index_pin,8*1024*1024,cancel)?;
    let mut raw=idx.read_range(0,index_pin.bytes as usize,cancel)?;
    ensure!(raw.len()>=4,"Native catalog index truncated");
    super::resource_cipher::decode_payload_in_place(&mut raw[4..]);
    let mut pak=PinnedFile::open(root,&[],"LTtext.pak",pak_pin,256*1024*1024,cancel)?;
    let entries=parse_index(&raw,pak.size,1024*1024,cancel)?;
    let &(actual_offset,size)=entries.get("ltlist.spr").context("Native catalog fixed member absent")?;
    ensure!(actual_offset==offset && size as u64==member_pin.bytes,"Native catalog index entry differs from authority");
    let mut bytes=pak.read_range(offset,size,cancel)?;
    ensure!(<[u8;32]>::from(Sha256::digest(&bytes))==member_pin.sha256,"Native catalog raw member drift");
    super::resource_cipher::decode_payload_in_place(&mut bytes);
    ensure!(<[u8;32]>::from(Sha256::digest(&bytes))==decoded_sha,"Native catalog decoded member drift");
    active(cancel)?;idx.verify()?;pak.verify()?;Ok(())
}
/// Explicit original BASE archive route for a Native body study. This is not
/// the Shared loader or namespace fallback, and is not SourceAttachment proof.
#[derive(Clone, Copy)]
pub(crate) enum NativeBaseArchive { Text, Sprite }
pub(crate) struct NativeBaseInspectionArchive {
    index: PinnedFile,
    pak: PinnedFile,
    raw_index: Vec<u8>,
    text: bool,
}
impl NativeBaseInspectionArchive {
    pub fn open(root:&Path,kind:NativeBaseArchive,index_pin:BytePin,pak_pin:BytePin,cancel:&AtomicBool)->Result<Self> {
        let text=matches!(kind,NativeBaseArchive::Text);
        let stem=if text {"Text"} else {"Sprite"};
        let mut index=PinnedFile::open(root,&[],&format!("{stem}.idx"),index_pin,4*1024*1024,cancel)?;
        let mut raw=index.read_range(0,index_pin.bytes as usize,cancel)?;
        if text {ensure!(raw.len()>=4,"Base text index truncated");super::resource_cipher::decode_payload_in_place(&mut raw[4..]);}
        let pak=PinnedFile::open(root,&[],&format!("{stem}.pak"),pak_pin,2usize*1024*1024*1024,cancel)?;
        validate_base_index(&raw,pak.size,cancel)?;
        active(cancel)?;
        Ok(Self{index,pak,raw_index:raw,text})
    }
    pub fn read_member(&mut self,name:&str,cancel:&AtomicBool)->Result<Vec<u8>> {
        let (offset,size)=find_base_member(&self.raw_index,name,cancel)?.context("Explicit Native base body member absent")?;
        ensure!(size>0 && size<=8*1024*1024,"Base body selected member empty/overbound");
        let mut bytes=self.pak.read_range(offset,size,cancel)?;
        if self.text {super::resource_cipher::decode_payload_in_place(&mut bytes);}
        self.index.verify()?;self.pak.verify()?;active(cancel)?;Ok(bytes)
    }
    pub fn read_verified_member(&mut self,name:&str,offset:u64,raw_pin:BytePin,decoded_pin:Option<BytePin>,cancel:&AtomicBool)->Result<Vec<u8>> {
        pin_valid(raw_pin,if self.text{8*1024*1024}else{1024*1024})?;
        let actual=find_base_member(&self.raw_index,name,cancel)?.context("Authoritative base member absent")?;
        ensure!(actual==(offset,raw_pin.bytes as usize),"Authoritative base member range differs");
        let mut bytes=self.pak.read_range(offset,actual.1,cancel)?;
        ensure!(<[u8;32]>::from(Sha256::digest(&bytes))==raw_pin.sha256,"Authoritative packed member differs");
        if self.text{super::resource_cipher::decode_payload_in_place(&mut bytes);}
        if let Some(pin)=decoded_pin{pin_valid(pin,8*1024*1024)?;ensure!(bytes.len() as u64==pin.bytes && <[u8;32]>::from(Sha256::digest(&bytes))==pin.sha256,"Authoritative decoded member differs");}
        self.index.verify()?;self.pak.verify()?;active(cancel)?;Ok(bytes)
    }
}
pub(crate) fn read_actor_provenance(root:&Path,sha256:[u8;32],cancel:&AtomicBool)->Result<Vec<u8>> {
    // Fixed name and bounded file; producer JSON paths are never followed.
    let path=root.join("source-verification.json");let bytes=fs::metadata(&path)?.len();
    let mut file=PinnedFile::open(root,&[],"source-verification.json",BytePin{bytes,sha256},64*1024,cancel)?;
    file.read_range(0,bytes as usize,cancel)
}
// BASE Text contains Big5 member names. Preserve every raw name; never decode
// or open unrelated names as paths. Only the explicit safe ASCII member is
// selected. All records still undergo count, padding and range/budget checks.
fn validate_base_index(index:&[u8],pak_size:u64,cancel:&AtomicBool)->Result<()> {
    ensure!(index.len()>=4,"Base old28 header truncated");
    let count=u32::from_le_bytes(index[..4].try_into()?) as usize;
    ensure!(count>0 && count<=250_000 && index.len()==4+count*28,"Base old28 count/length invalid");
    for row in index[4..].chunks_exact(28){active(cancel)?;let name=&row[4..24];let end=name.iter().position(|b|*b==0).unwrap_or(20);
        ensure!(end>0 && name[end..].iter().all(|b|*b==0),"Base raw member padding invalid");
        let offset=u64::from(u32::from_le_bytes(row[..4].try_into()?));let size=u64::from(u32::from_le_bytes(row[24..].try_into()?));
        ensure!(size<=64*1024*1024 && offset.checked_add(size).is_some_and(|e|e<=pak_size),"Base raw member bounds invalid");
    }Ok(())
}
fn find_base_member(index:&[u8],name:&str,cancel:&AtomicBool)->Result<Option<(u64,usize)>> {
    ensure!(!name.is_empty() && name.len()<=20 && name.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'.'|b'-')),"Base selected name invalid");
    let mut found=None;
    for row in index[4..].chunks_exact(28){active(cancel)?;let raw=&row[4..24];let end=raw.iter().position(|b|*b==0).unwrap_or(20);
        if raw[..end].eq_ignore_ascii_case(name.as_bytes()) {ensure!(found.is_none(),"Duplicate base selected member");found=Some((u64::from(u32::from_le_bytes(row[..4].try_into()?)),u32::from_le_bytes(row[24..].try_into()?) as usize));}
    }Ok(found)
}
/// Original 440010 looks in hash-selected SpriteNN before BASE Sprite. A
/// pinned index MISS, not a corrupt/missing archive or decode error, is required
/// before accepting a base member. Unread shard PAK contents are not qualified.
pub(crate) fn verify_native_body_shard_miss(root:&Path,shard:u8,index_pin:BytePin,pak_bytes:u64,name:&str,cancel:&AtomicBool)->Result<()> {
    ensure!(shard<16 && name.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'.'|b'-'))
        && name.bytes().map(u32::from).sum::<u32>()%16==u32::from(shard),"Native body hash route mismatch");
    let mut index=PinnedFile::open(root,&[],&format!("Sprite{shard:02}.idx"),index_pin,4*1024*1024,cancel)?;
    let raw=index.read_range(0,index_pin.bytes as usize,cancel)?;
    ensure!(pak_bytes>0 && pak_bytes<=2u64*1024*1024*1024,"Shard PAK length bound");
    validate_base_index(&raw,pak_bytes,cancel)?;
    ensure!(find_base_member(&raw,name,cancel)?.is_none(),"Native body shard override present; base route refused");
    index.verify()?;active(cancel)?;Ok(())
}
impl NativeArchive {
    fn open(
        root: &Path,
        index: ArchivePin,
        pak: ArchivePin,
        limits: Limits,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let mut idx = PinnedFile::open(
            root,
            &[],
            index.role.name(),
            index.pin,
            limits.index_bytes,
            cancel,
        )?;
        let pak = PinnedFile::open(
            root,
            &[],
            pak.role.name(),
            pak.pin,
            limits.pak_bytes,
            cancel,
        )?;
        Self::from_open(idx, pak, limits, cancel)
    }
    fn from_open(
        mut idx: PinnedFile,
        pak: PinnedFile,
        limits: Limits,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let raw = idx.read_range(0, idx.size as usize, cancel)?;
        let entries = parse_index(&raw, pak.size, limits.entry_bytes, cancel)?;
        Ok(Self { index: idx, pak, entries })
    }
    fn member(&mut self, name: &str, cancel: &AtomicBool) -> Result<Vec<u8>> {
        let &(offset, size) = self
            .entries
            .get(name)
            .context("required Native member absent; no fallback")?;
        ensure!(size > 0, "required Native member empty");
        self.pak.read_range(offset, size, cancel)
    }
}
/// Four source pins observed on the same read-only handles used for static
/// loading. The existing caller-supplied-pin API remains separate.
pub struct NativeObservedArchives {
    pins: [ArchivePin; 4],
    tile: Option<NativeArchive>,
    sprite: Option<NativeArchive>,
}
/// One parsed S32 generation, backed by write/delete-denying source handles.
/// Only immutable geometry is shared; residence pixels keep their own budgets.
pub struct NativeRetainedGeometry {
    root:PathBuf,
    map_id:u32,
    sectors:Vec<SectorPin>,
    files:Vec<(String,PinnedFile)>,
    map:Arc<WorldMap>,
}
fn native_sector_names(root:&Path,map_id:u32,cancel:&AtomicBool)->Result<Vec<String>> {
    active(cancel)?;
    ensure!(root.is_absolute()&&!root.components().any(|c|matches!(c,Component::ParentDir|Component::CurDir)),
        "Native root must be explicit confined absolute path");
    reject_reparse_chain(root)?;
    let directory=root.join(map_id.to_string());reject_reparse_chain(&directory)?;
    let mut names=Vec::new();
    for entry in fs::read_dir(&directory)? {
        active(cancel)?;let entry=entry?;
        let name=entry.file_name().into_string().map_err(|_|anyhow::anyhow!("Native map filename encoding"))?;
        if !name.ends_with(".s32"){continue;}
        ensure!(name.len()==12&&name[..8].bytes().all(|b|b.is_ascii_hexdigit()),"Native map sector name");
        ensure!(names.len()<64,"Native sector count budget");names.push(name);
    }
    names.sort();ensure!(!names.is_empty(),"no Native S32 sectors");Ok(names)
}
impl NativeRetainedGeometry {
    pub fn open(root:&Path,map_id:u32,limits:Limits,cancel:&AtomicBool)->Result<Self> {
        active(cancel)?;let limits=limits.validate()?;
        ensure!(root.is_absolute()&&map_id<=i32::MAX as u32,"Native geometry root/map identity");
        reject_reparse_chain(root)?;let canonical=fs::canonicalize(root)?;
        let names=native_sector_names(root,map_id,cancel)?;
        let mut sectors=Vec::new();let mut files=Vec::new();let mut inputs=Vec::new();let mut total=0u64;
        for name in names {
            let key=NativeSectorKey{map_id,x:u16::from_str_radix(&name[..4],16)?,y:u16::from_str_radix(&name[4..8],16)?};
            let (mut file,pin)=PinnedFile::open_checked(root,&[map_id.to_string()],&name,None,limits.sector_bytes,cancel)?;
            total=total.checked_add(pin.bytes).context("Native geometry byte overflow")?;
            ensure!(total<=limits.map_bytes as u64,"Native map input budget exceeded");
            inputs.push(SectorInput{key:key.geometry_key(),bytes:file.read_range(0,pin.bytes as usize,cancel)?});
            sectors.push(SectorPin{key,pin});files.push((name,file));
        }
        let map=Arc::new(WorldMap::from_native_sector_bytes(inputs,cancel)?);
        let retained=Self{root:canonical,map_id,sectors,files,map};
        retained.validate_sources(root,map_id,cancel)?;Ok(retained)
    }
    pub fn geometry(&self)->&WorldMap {&self.map}
    pub fn sector_pins(&self)->&[SectorPin] {&self.sectors}
    pub fn validate_sources(&self,root:&Path,map_id:u32,cancel:&AtomicBool)->Result<()> {
        active(cancel)?;reject_reparse_chain(root)?;
        ensure!(map_id==self.map_id&&fs::canonicalize(root)?==self.root,"Retained Native geometry root/map differs");
        let names=native_sector_names(root,map_id,cancel)?;
        ensure!(names.len()==self.files.len()&&names.iter().zip(&self.files).all(|(name,(old,_))|name==old),
            "Retained Native geometry sector set changed");
        for (_,file) in &self.files {active(cancel)?;file.verify()?;}
        active(cancel)
    }
}
impl NativeObservedArchives {
    pub fn open(root: &Path, limits: Limits, cancel: &AtomicBool) -> Result<Self> {
        let limits = limits.validate()?;
        let (tile_index, tile_index_pin) =
            PinnedFile::observe(root, ArchiveRole::TileIndex.name(), limits.index_bytes, cancel)?;
        let (tile_pak, tile_pak_pin) =
            PinnedFile::observe(root, ArchiveRole::TilePak.name(), limits.pak_bytes, cancel)?;
        let (sprite_index, sprite_index_pin) =
            PinnedFile::observe(root, ArchiveRole::SpriteIndex.name(), limits.index_bytes, cancel)?;
        let (sprite_pak, sprite_pak_pin) =
            PinnedFile::observe(root, ArchiveRole::SpritePak.name(), limits.pak_bytes, cancel)?;
        let pins = [
            ArchivePin { role: ArchiveRole::TileIndex, pin: tile_index_pin },
            ArchivePin { role: ArchiveRole::TilePak, pin: tile_pak_pin },
            ArchivePin { role: ArchiveRole::SpriteIndex, pin: sprite_index_pin },
            ArchivePin { role: ArchiveRole::SpritePak, pin: sprite_pak_pin },
        ];
        let tile = NativeArchive::from_open(tile_index, tile_pak, limits, cancel)?;
        let sprite = NativeArchive::from_open(sprite_index, sprite_pak, limits, cancel)?;
        Ok(Self { pins, tile: Some(tile), sprite: Some(sprite) })
    }
    pub fn pins(&self) -> [ArchivePin; 4] { self.pins }
    /// Initially hashed bytes stay write/delete-denied for this owner's entire
    /// lifetime. Revalidate their exact paths and handles without hashing again.
    pub fn validate_sources(&self,root:&Path,cancel:&AtomicBool)->Result<()> {
        active(cancel)?;
        reject_reparse_chain(root)?;
        let root=fs::canonicalize(root)?;
        let tile=self.tile.as_ref().context("Observed Tile archive absent")?;
        let sprite=self.sprite.as_ref().context("Observed Sprite archive absent")?;
        for (file,role) in [(&tile.index,ArchiveRole::TileIndex),(&tile.pak,ArchiveRole::TilePak),
            (&sprite.index,ArchiveRole::SpriteIndex),(&sprite.pak,ArchiveRole::SpritePak)] {
            active(cancel)?;
            ensure!(file.path==root.join(role.name()),"Observed Native archive root differs");
            file.verify()?;
        }
        active(cancel)
    }
}
/// A scene's share of the same initially hashed, write/delete-denying handles.
/// Only source handles/indexes are shared; decoded assets remain scene-owned.
#[derive(Clone)]
pub struct SharedNativeArchives {
    owner: Arc<Mutex<NativeObservedArchives>>,
    reused: bool,
}
type SharedArchiveKey = (PathBuf, [usize; 6]);
type SharedArchiveRegistry = BTreeMap<SharedArchiveKey, Weak<Mutex<NativeObservedArchives>>>;
static SHARED_ARCHIVES: OnceLock<Mutex<SharedArchiveRegistry>> = OnceLock::new();

fn source_lock<'a,T>(mutex:&'a Mutex<T>,cancel:&AtomicBool)->Result<MutexGuard<'a,T>> {
    loop {
        active(cancel)?;
        match mutex.try_lock() {
            Ok(guard)=>{active(cancel)?;return Ok(guard);},
            Err(TryLockError::WouldBlock)=>std::thread::sleep(std::time::Duration::from_millis(1)),
            Err(TryLockError::Poisoned(_))=>anyhow::bail!("shared Native source owner poisoned"),
        }
    }
}
impl SharedNativeArchives {
    pub fn open(root:&Path,limits:Limits,cancel:&AtomicBool)->Result<Self> {
        active(cancel)?;let limits=limits.validate()?;
        ensure!(root.is_absolute()&&!root.components().any(|c|matches!(c,Component::ParentDir|Component::CurDir)),
            "Native root must be explicit confined absolute path");
        reject_reparse_chain(root)?;
        let key=(fs::canonicalize(root)?,[limits.index_bytes,limits.pak_bytes,limits.entry_bytes,
            limits.sector_bytes,limits.map_bytes,limits.decoded_bytes]);
        let registry=SHARED_ARCHIVES.get_or_init(||Mutex::new(BTreeMap::new()));
        let mut registry=source_lock(registry,cancel)?;
        registry.retain(|_,owner|owner.strong_count()>0);
        if let Some(owner)=registry.get(&key).and_then(Weak::upgrade) {
            drop(registry);
            source_lock(&owner,cancel)?.validate_sources(root,cancel)?;
            return Ok(Self{owner,reused:true});
        }
        ensure!(registry.len()<64,"shared Native source root/limits budget");
        // Serialize the first observation so concurrent scenes never hash the
        // same files independently. Other waiters can still cancel their wait.
        let owner=Arc::new(Mutex::new(NativeObservedArchives::open(root,limits,cancel)?));
        registry.insert(key,Arc::downgrade(&owner));
        Ok(Self{owner,reused:false})
    }
    pub fn reused_source_owner(&self)->bool {self.reused}
    /// A single cancellable lease also serializes the archives' seek cursors.
    pub fn lease(&self,cancel:&AtomicBool)->Result<MutexGuard<'_,NativeObservedArchives>> {
        source_lock(&self.owner,cancel)
    }
}
#[derive(Default)]
pub struct NativeResourceAdapter {
    access: NativeAccess,
    draw_reference_limit: usize,
}
impl NativeResourceAdapter {
    pub fn pinned_inspection() -> Self {
        Self {
            access: NativeAccess::PinnedInspection,
            draw_reference_limit: 262_144,
        }
    }
    /// Whole runtime cities retain every reference. This bounded metadata budget
    /// is independent of the unchanged 32 MiB encoded / 64 MiB decoded limits.
    /// The original map500 has 18 sectors and 351,509 references.
    pub fn pinned_world() -> Self {
        Self {access:NativeAccess::PinnedInspection,draw_reference_limit:1_048_576}
    }
    pub fn load_static(
        &self,
        sources: &NativeSources,
        plan: &NativeAssetPlan,
        limits: Limits,
        cancel: &AtomicBool,
    ) -> Result<NativeStaticAssets> {
        self.load_static_with_observed(sources, plan, limits, None, None, None, cancel)
    }
    pub fn load_static_observed(
        &self,
        sources: &NativeSources,
        plan: &NativeAssetPlan,
        limits: Limits,
        mut observed: NativeObservedArchives,
        cancel: &AtomicBool,
    ) -> Result<NativeStaticAssets> {
        self.load_static_with_observed(sources, plan, limits, Some(&mut observed), None, None, cancel)
    }
    /// Retain verified whole-map geometry while decoding the 800x600 view union
    /// for its current 8x8 cell block, including interpolation padding.
    pub fn load_static_observed_window(
        &self, sources:&NativeSources, plan:&NativeAssetPlan, limits:Limits,
        mut observed:NativeObservedArchives, view:[u16;2], cancel:&AtomicBool,
    )->Result<NativeStaticAssets>{
        self.load_static_with_observed(sources,plan,limits,Some(&mut observed),Some(view),None,cancel)
    }
    /// Borrow one locked archive generation across successive residence windows.
    /// Failure or cancellation leaves the owner's source handles intact.
    pub fn load_static_retained_window(
        &self,sources:&NativeSources,plan:&NativeAssetPlan,limits:Limits,
        observed:&mut NativeObservedArchives,view:[u16;2],cancel:&AtomicBool,
    )->Result<NativeStaticAssets> {
        self.load_static_with_observed(sources,plan,limits,Some(observed),Some(view),None,cancel)
    }
    /// Reuse exactly the retained S32 generation while decoding a new window.
    pub fn load_static_retained_geometry_window(
        &self,sources:&NativeSources,plan:&NativeAssetPlan,limits:Limits,
        observed:&mut NativeObservedArchives,geometry:&NativeRetainedGeometry,view:[u16;2],cancel:&AtomicBool,
    )->Result<NativeStaticAssets> {
        self.load_static_with_observed(sources,plan,limits,Some(observed),Some(view),Some(geometry),cancel)
    }
    fn load_static_with_observed(
        &self,
        sources: &NativeSources,
        plan: &NativeAssetPlan,
        limits: Limits,
        mut observed: Option<&mut NativeObservedArchives>,
        resident: Option<[u16;2]>,
        retained: Option<&NativeRetainedGeometry>,
        cancel: &AtomicBool,
    ) -> Result<NativeStaticAssets> {
        active(cancel)?;
        ensure!(
            self.access == NativeAccess::PinnedInspection,
            "Native adapter is disabled"
        );
        let limits = limits.validate()?;
        if let Some(ref observed) = observed {
            ensure!(observed.pins.as_slice() == plan.archives.as_slice(), "Observed Native archive pins differ from plan");
            observed.validate_sources(&sources.archive_root,cancel)?;
        }
        ensure!(
            plan.namespace == ResourceNamespace::NativeLitoV1
                && plan.map_id <= i32::MAX as u32
                && plan.canonical_sha256 != [0; 32],
            "Native source/identity invalid"
        );
        let roles = [
            ArchiveRole::TileIndex,
            ArchiveRole::TilePak,
            ArchiveRole::SpriteIndex,
            ArchiveRole::SpritePak,
        ];
        ensure!(
            plan.archives.len() == 4 && plan.archives.iter().zip(roles).all(|(p, r)| p.role == r),
            "Native archive roles must be exact and ordered"
        );
        for pin in &plan.archives {
            pin_valid(pin.pin, pin.role.limit(limits))?;
        }
        ensure!(
            !plan.sectors.is_empty() && plan.sectors.len() <= 64,
            "Native sector count invalid"
        );
        let mut seen = BTreeSet::new();
        let mut total = 0u64;
        for pin in &plan.sectors {
            pin_valid(pin.pin, limits.sector_bytes)?;
            ensure!(
                pin.key.map_id == plan.map_id && seen.insert(pin.key),
                "Native sector identity/duplicate invalid"
            );
            total = total
                .checked_add(pin.pin.bytes)
                .context("Native map byte overflow")?;
            ensure!(
                total <= limits.map_bytes as u64,
                "Native map input budget exceeded"
            );
        }
        let map=if let Some(retained)=retained {
            retained.validate_sources(&sources.map_root,plan.map_id,cancel)?;
            ensure!(retained.sectors==plan.sectors,"Retained Native geometry pins differ from plan");
            Arc::clone(&retained.map)
        }else{
        let mut inputs = Vec::new();
        for pin in &plan.sectors {
            let key = pin.key;
            let mut file = PinnedFile::open(
                &sources.map_root,
                &[key.map_id.to_string()],
                &format!("{:04x}{:04x}.s32", key.x, key.y),
                pin.pin,
                limits.sector_bytes,
                cancel,
            )?;
            inputs.push(SectorInput {
                key: key.geometry_key(),
                bytes: file.read_range(0, pin.pin.bytes as usize, cancel)?,
            });
        }
        Arc::new(WorldMap::from_native_sector_bytes(inputs, cancel)?)
        };
        ensure!(
            map.canonical_digest() == plan.canonical_sha256,
            "Native canonical geometry differs from pin"
        );
        let resident_bounds=resident.map(|view|{
            let bindings=map.bindings();
            let x=i32::from(view[0]);let y=i32::from(view[1]);
            let min_x=bindings.iter().map(|b|b.key.world_origin().0).min().unwrap();
            let min_y=bindings.iter().map(|b|b.key.world_origin().1).min().unwrap();
            let max_x=bindings.iter().map(|b|b.key.world_origin().0+64).max().unwrap();
            let max_y=bindings.iter().map(|b|b.key.world_origin().1+64).max().unwrap();
            ensure!(x>=min_x&&x<max_x&&y>=min_y&&y<max_y,"Native view outside source geometry bounds");
            let (ox,oy)=bindings[0].key.world_origin();
            let (dx,dy)=((x&!7)-ox,(y&!7)-oy);
            let (px,py)=(24+24*(dx+dy),768+12*(-dx+dy));
            // Union of every camera in the block, plus two cells of motion.
            Ok::<[i32;4],anyhow::Error>([px-496,py-432,px+832,py+432])
        }).transpose()?;
        let mut draw = Vec::new();
        let mut effects = Vec::new();
        let mut opacity_records = 0usize;
        for binding in map.bindings() {
            active(cancel)?;
            let sector = map.sector(binding.key).context("Native sector absent")?;
            let (sx, sy) = (
                i32::from(binding.key.sector_x) - i32::from(plan.sectors[0].key.x),
                i32::from(binding.key.sector_y) - i32::from(plan.sectors[0].key.y),
            );
            let mut tiles = static_draw_list(sector)?;
            ensure!(
                draw.len().checked_add(tiles.len()).is_some_and(|n|n<=self.draw_reference_limit),
                "Native draw budget exceeded"
            );
            for tile in &mut tiles {
                tile.x = tile
                    .x
                    .checked_add(1536 * (sx + sy))
                    .context("Native projected X overflow")?;
                tile.y = tile
                    .y
                    .checked_add(768 * (-sx + sy))
                    .context("Native projected Y overflow")?;
            }
            // Preserve source group ordinal per object, including nonordinal serialized IDs.
            ensure!(
                sector.objects.len() <= 65_536,
                "Native object group ordinal budget exceeded"
            );
            let floor = sector.floor.len() + sector.overlays.len();
            let mut groups = vec![None; floor];
            for (i, group) in sector.objects.iter().enumerate() {
                groups.extend(std::iter::repeat_n(
                    Some(Group(binding.key.sector_x, binding.key.sector_y, i as u16)),
                    group.tiles.len(),
                ));
            }
            draw.extend(
                tiles
                    .into_iter()
                    .zip(groups)
                    .filter(|(t,_)|resident_bounds.is_none_or(|[l,top,r,b]|t.x<r&&t.y<b&&t.x+24>l&&t.y+24>top))
                    .map(|(tile, group)| NativeDraw { tile, group, sector:NativeSectorKey{map_id:plan.map_id,x:binding.key.sector_x,y:binding.key.sector_y} }),
            );
            opacity_records += sector.opacity.len();
            effects.extend(
                sector
                    .effects
                    .iter()
                    .copied()
                    .map(|effect| NativeEffectRecord {
                        sector: NativeSectorKey {
                            map_id: plan.map_id,
                            x: binding.key.sector_x,
                            y: binding.key.sector_y,
                        },
                        effect,
                    }),
            );
        }
        let mut required: BTreeMap<u16, BTreeSet<usize>> = BTreeMap::new();
        for d in &draw {
            required
                .entry(d.tile.archive_entry_id)
                .or_default()
                .insert(usize::from(d.tile.sub_index));
        }
        let mut opened_archive;
        let archive = if let Some(ref mut observed) = observed {
            observed.tile.as_mut().context("Observed Tile archive absent")?
        } else {
            opened_archive=NativeArchive::open(&sources.archive_root, plan.archives[0], plan.archives[1], limits, cancel)?;
            &mut opened_archive
        };
        let mut tiles = BTreeMap::new();
        let mut encoded = BTreeMap::new();
        let mut decoded_bytes = 0usize;
        let mut encoded_bytes = 0usize;
        for (id, ids) in &required {
            active(cancel)?;
            let raw = archive.member(&format!("{id}.til"), cancel)?;
            encoded_bytes = encoded_bytes
                .checked_add(raw.len())
                .context("Native retained source overflow")?;
            ensure!(
                encoded_bytes <= limits.map_bytes,
                "Native retained source budget exceeded"
            );
            let remaining = limits
                .decoded_bytes
                .checked_sub(decoded_bytes)
                .context("Native decoded budget exhausted")?;
            ensure!(
                ids.len()
                    .checked_mul(24 * 24 * 4)
                    .is_some_and(|bytes| bytes <= remaining),
                "Native selected pixel budget exceeded"
            );
            let tile_limits = TileLimits {
                max_input_bytes: limits.entry_bytes,
                max_total_pixels: remaining / 4,
                ..Default::default()
            };
            let input = ValidatedTileInput::new_with_empty_tail(&raw, tile_limits)?;
            // Check cancellation between selected slots; the table is validated once.
            for &slot in ids {
                active(cancel)?;
                let decoded = input
                    .decode_selected(&BTreeSet::from([slot]), false)?
                    .remove(&slot)
                    .context("Native selected slot absent")?;
                decoded_bytes = decoded_bytes
                    .checked_add(decoded.rgba.len())
                    .context("Native pixel size overflow")?;
                ensure!(
                    decoded_bytes <= limits.decoded_bytes,
                    "Native pixel budget exceeded"
                );
                tiles.insert((*id, slot as u8), decoded);
            }
            encoded.insert(*id, raw);
        }
        // Verify both Sprite pins/index even when no effect playback is selected.
        let opened_sprites;
        let _sprites = if let Some(ref observed) = observed {
            observed.sprite.as_ref().context("Observed Sprite archive absent")?
        } else {
            opened_sprites=NativeArchive::open(&sources.archive_root, plan.archives[2], plan.archives[3], limits, cancel)?;
            &opened_sprites
        };
        active(cancel)?;
        Ok(NativeStaticAssets {
            plan: plan.clone(),
            draw_reference_limit: self.draw_reference_limit,
            map,
            draw,
            tiles,
            encoded,
            decoded_bytes,
            cache_limit: limits.decoded_bytes,
            opacity_records,
            effects,
        })
    }
}
struct NativeDraw {
    sector:NativeSectorKey,
    tile: DrawTile,
    group: Option<Group>,
}
#[derive(Clone, Copy, Debug)]
pub struct NativeEffectRecord {
    pub sector: NativeSectorKey,
    pub effect: SpriteEffect,
}
pub struct NativeStaticAssets {
    plan: NativeAssetPlan,
    draw_reference_limit: usize,
    map: Arc<WorldMap>,
    draw: Vec<NativeDraw>,
    tiles: BTreeMap<(u16, u8), DecodedTile>,
    encoded: BTreeMap<u16, Vec<u8>>,
    decoded_bytes: usize,
    cache_limit: usize,
    opacity_records: usize,
    effects: Vec<NativeEffectRecord>,
}

/// Explicit classic WORLD map4 source. Does not alter NativeLitoV1 admission or
/// search another archive when a member is absent. The caller supplies at most
/// a 3x3 window from original map/4; Tile is the original BASE old28 archive.
pub(crate) fn classic_world4_assets(root:&Path,map:WorldMap,sectors:Vec<SectorPin>,cancel:&AtomicBool)->Result<NativeStaticAssets>{
    use super::resources::{SharedArchive,SharedArchiveKind};
    active(cancel)?;
    ensure!(!sectors.is_empty()&&sectors.len()<=9&&sectors.iter().all(|s|s.key.map_id==4),"classic map4 window identity");
    let limits=Limits::default();let draw_reference_limit=1_048_576;
    let mut draw=Vec::new();let mut effects=Vec::new();let mut opacity_records=0;
    for binding in map.bindings(){
        active(cancel)?;let sector=map.sector(binding.key).context("classic sector missing")?;
        let mut groups=vec![None;sector.floor.len()+sector.overlays.len()];
        for (i,g) in sector.objects.iter().enumerate(){groups.extend(std::iter::repeat_n(Some(Group(binding.key.sector_x,binding.key.sector_y,u16::try_from(i)?)),g.tiles.len()));}
        let mut tiles=static_draw_list(sector)?;ensure!(tiles.len()==groups.len()&&draw.len()+tiles.len()<=draw_reference_limit,"classic draw count budget");
        let sx=i32::from(binding.key.sector_x)-i32::from(sectors[0].key.x);let sy=i32::from(binding.key.sector_y)-i32::from(sectors[0].key.y);
        for t in &mut tiles{t.x=t.x.checked_add(1536*(sx+sy)).context("classic draw X overflow")?;t.y=t.y.checked_add(768*(-sx+sy)).context("classic draw Y overflow")?;}
        draw.extend(tiles.into_iter().zip(groups).map(|(tile,group)|NativeDraw{tile,group,sector:NativeSectorKey{map_id:4,x:binding.key.sector_x,y:binding.key.sector_y}}));
        opacity_records+=sector.opacity.len();
        effects.extend(sector.effects.iter().copied().map(|effect|NativeEffectRecord{sector:NativeSectorKey{map_id:4,x:binding.key.sector_x,y:binding.key.sector_y},effect}));
    }
    let mut required:BTreeMap<u16,BTreeSet<usize>>=BTreeMap::new();
    for d in &draw{required.entry(d.tile.archive_entry_id).or_default().insert(d.tile.sub_index.into());}
    let mut archive=SharedArchive::open(root,SharedArchiveKind::Tile)?;
    let mut tiles=BTreeMap::new();let mut encoded=BTreeMap::new();let mut decoded_bytes=0usize;let mut encoded_bytes=0usize;
    for (id,slots) in required{
        active(cancel)?;let raw=archive.read_entry(&format!("{id}.til"))?;encoded_bytes=encoded_bytes.checked_add(raw.len()).context("classic tile byte overflow")?;
        ensure!(encoded_bytes<=limits.map_bytes,"classic encoded tile budget");
        let remaining=limits.decoded_bytes.checked_sub(decoded_bytes).context("classic decoded tile budget")?;
        let input=ValidatedTileInput::new_with_empty_tail(&raw,TileLimits{max_input_bytes:limits.entry_bytes,max_total_pixels:remaining/4,..Default::default()})?;
        for slot in slots{
            active(cancel)?;let tile=input.decode_selected(&BTreeSet::from([slot]),false)?.remove(&slot).context("classic tile slot absent")?;
            decoded_bytes=decoded_bytes.checked_add(tile.rgba.len()).context("classic decoded byte overflow")?;ensure!(decoded_bytes<=limits.decoded_bytes,"classic decoded tile budget");
            tiles.insert((id,u8::try_from(slot)?),tile);
        }
        encoded.insert(id,raw);
    }
    active(cancel)?;
    let plan=NativeAssetPlan{namespace:ResourceNamespace::Shared,map_id:4,canonical_sha256:map.canonical_digest(),sectors,archives:Vec::new()};
    Ok(NativeStaticAssets{plan,draw_reference_limit,map:Arc::new(map),draw,tiles,encoded,decoded_bytes,cache_limit:limits.decoded_bytes,opacity_records,effects})
}
impl NativeStaticAssets {
    /// Source-order half-cell references for default-off scene composition.
    /// Preserves geometry and decoded pixels; does not select roof runtime state.
    pub fn scene_terrain(&self)->Result<Vec<(super::world_depth::NativeDrawCell,Option<u8>,(u16,u8))>> {
        use super::world_depth::NativeDrawCell;
        let mut draws=Vec::new();
        for d in &self.draw {
            let (x,y)=d.sector.geometry_key().world_origin();let origin=NativeDrawCell::from_world(WorldCell{x,y}).context("Native terrain origin overflow")?;let t=&d.tile;
            let cell=NativeDrawCell{half_x:origin.half_x.checked_add(i32::from(t.half_x)).context("Native terrain X overflow")?,y:origin.y.checked_add(i32::from(t.half_y)).context("Native terrain Y overflow")?};
            draws.push((cell,if t.layer == -2{None}else{Some(u8::try_from(t.layer.max(0))?)},(t.archive_entry_id,t.sub_index)));
        }
        ensure!(draws.len()==self.draw.len(),"Native terrain source count changed");Ok(draws)
    }
    pub(crate) fn scene_groups(&self)->Vec<Option<Group>> { self.draw.iter().map(|d|d.group).collect() }
    pub fn scene_tiles(&self)->&BTreeMap<(u16,u8),DecodedTile>{&self.tiles}
    /// Original bytes after the TIL kind. For raw diamonds these are pixel
    /// bytes, not the RLE rectangle synthesized by a preview decoder.
    pub(crate) fn scene_source_header(&self,key:(u16,u8))->Result<[i8;4]> {
        let tile=self.tiles.get(&key).context("scene tile header source absent")?;
        let bytes=self.encoded.get(&key.0).context("scene encoded tile header absent")?;
        let count=u32::from_le_bytes(bytes.get(..4).context("short scene tile table")?.try_into()?) as usize;
        let start=count.checked_add(1).and_then(|n|n.checked_mul(4)).and_then(|n|n.checked_add(4))
            .and_then(|n|n.checked_add(tile.source_offset as usize)).context("scene tile header overflow")?;
        let prefix=bytes.get(start..start.checked_add(5).context("scene tile prefix overflow")?).context("short scene tile prefix")?;
        ensure!(prefix[0]==tile.tile_type as u8,"scene tile header kind mismatch");
        Ok([prefix[1] as i8,prefix[2] as i8,prefix[3] as i8,prefix[4] as i8])
    }
    /// Actual roof source pixels, without importing the modern roof clock.
    pub fn scene_roof_raw(&self,cancel:&AtomicBool)->Result<BTreeMap<(u16,u8),DecodedTile>>{Ok(self.prepare_roofs(cancel)?.raw)}
    /// Floors copy original RGB555 even when their preview has cloud/key alpha.
    /// Decode only the additional effect slots; share slots already used by roofs.
    pub fn scene_raw_sources(&self,cancel:&AtomicBool)->Result<BTreeMap<(u16,u8),DecodedTile>> {
        let mut raw=self.scene_roof_raw(cancel)?;
        let mut remaining=self.cache_limit.checked_sub(self.decoded_bytes)
            .and_then(|v|v.checked_sub(raw.values().map(|t|t.rgba.len()).sum()))
            .context("native floor/roof raw budget")?;
        let mut ids:BTreeMap<u16,BTreeSet<usize>>=BTreeMap::new();
        for d in &self.draw {
            let key=(d.tile.archive_entry_id,d.tile.sub_index);
            if d.tile.layer == -2 && self.tiles[&key].tile_type as u8 &0x30!=0 && !raw.contains_key(&key) {
                ids.entry(key.0).or_default().insert(key.1.into());
            }
        }
        for (id,slots) in ids {
            active(cancel)?;
            let input=ValidatedTileInput::new_with_empty_tail(self.encoded.get(&id).context("floor encoded source absent")?,
                TileLimits{max_input_bytes:8<<20,max_total_pixels:remaining/4,..Default::default()})?;
            for slot in slots {
                active(cancel)?;let tile=input.decode_selected(&BTreeSet::from([slot]),true)?.remove(&slot).context("floor raw slot absent")?;
                remaining=remaining.checked_sub(tile.rgba.len()).context("native floor retained budget")?;raw.insert((id,slot as u8),tile);
            }
        }
        Ok(raw)
    }
    /// Local verified resource identity only; no match/generation/LOADED grant.
    pub fn verified_plan(&self) -> &NativeAssetPlan {
        &self.plan
    }
    pub fn geometry(&self) -> &WorldMap {
        &self.map
    }
    pub fn resident_sector_count(&self)->usize {self.draw.iter().map(|d|(d.sector.x,d.sector.y)).collect::<BTreeSet<_>>().len()}
    pub fn decoded_slots(&self) -> usize {
        self.tiles.len()
    }
    pub fn tile_entries(&self) -> usize {
        self.encoded.len()
    }
    pub fn decoded_bytes(&self) -> usize {
        self.decoded_bytes
    }
    pub fn draw_references(&self) -> usize {
        self.draw.len()
    }
    pub fn draw_reference_limit(&self)->usize {self.draw_reference_limit}
    pub fn opacity_records(&self) -> usize {
        self.opacity_records
    }
    /// Coordinates/extended data are retained; no action/frame/duration guessed.
    pub fn effect_records(&self) -> &[NativeEffectRecord] {
        &self.effects
    }
    pub fn render_static(&self, camera: CameraRect, cancel: &AtomicBool) -> Result<Canvas> {
        self.render(camera, None, cancel)
    }
    fn render(
        &self,
        camera: CameraRect,
        roofs: Option<(&world_roofs::Frame, &BTreeMap<(u16, u8), DecodedTile>)>,
        cancel: &AtomicBool,
    ) -> Result<Canvas> {
        active(cancel)?;
        ensure!(
            camera.width > 0 && camera.height > 0 && camera.width <= 4096 && camera.height <= 4096,
            "Native camera budget invalid"
        );
        let mut canvas = Canvas::new(camera.width, camera.height)?;
        canvas.clear([18, 25, 35]);
        let mut ordered: Vec<_> = self.draw.iter().collect();
        ordered.sort_by_key(|d| d.tile.layer);
        for d in ordered {
            active(cancel)?;
            let t = &d.tile;
            let decoded = &self.tiles[&(t.archive_entry_id, t.sub_index)];
            let x = i64::from(t.x) - i64::from(camera.x);
            let y = i64::from(t.y) - i64::from(camera.y);
            if x + 24 <= 0
                || y + 24 <= 0
                || x >= i64::from(camera.width)
                || y >= i64::from(camera.height)
            {
                continue;
            }
            let (x, y) = (i32::try_from(x)?, i32::try_from(y)?);
            if let Some((frame, raw)) = roofs {
                let half = matches!(
                    decoded.tile_type,
                    ClassicTileType::RleEvenHalfOpacity | ClassicTileType::RleOddHalfOpacity
                );
                let shade = frame.shade(d.group, half);
                if shade != Shade::Native {
                    let raw = raw
                        .get(&(t.archive_entry_id, t.sub_index))
                        .context("Native roof raw coverage absent")?;
                    canvas.blit_roof_rgb555(&raw.rgba, x, y, shade)?;
                    continue;
                }
            }
            canvas.blit_rgba(&decoded.rgba, decoded.width, decoded.height, x, y)?;
        }
        active(cancel)?;
        Ok(canvas)
    }
    /// Reuse qualified Layer5 group/clock/compositor rules on the pinned map.
    /// This explicit CPU inspection does not prove original timer/alias behavior.
    pub fn prepare_roofs(&self, cancel: &AtomicBool) -> Result<NativeRoofRenderer> {
        active(cancel)?;
        let roofs = RoofMap::new(&self.map)?;
        let mut ids: BTreeMap<u16, BTreeSet<usize>> = BTreeMap::new();
        for d in &self.draw {
            if d.group.is_some_and(|g| roofs.controls(g)) {
                ids.entry(d.tile.archive_entry_id)
                    .or_default()
                    .insert(usize::from(d.tile.sub_index));
            }
        }
        let mut raw = BTreeMap::new();
        let mut remaining = self
            .cache_limit
            .checked_sub(self.decoded_bytes)
            .context("Native roof budget exhausted")?;
        for (id, slots) in ids {
            let input = ValidatedTileInput::new_with_empty_tail(
                &self.encoded[&id],
                TileLimits {
                    max_input_bytes: 8 << 20,
                    max_total_pixels: remaining / 4,
                    ..Default::default()
                },
            )?;
            for slot in slots {
                active(cancel)?;
                let tile = input
                    .decode_selected(&BTreeSet::from([slot]), true)?
                    .remove(&slot)
                    .context("Native roof slot absent")?;
                remaining = remaining
                    .checked_sub(tile.rgba.len())
                    .context("Native roof retained budget exceeded")?;
                raw.insert((id, slot as u8), tile);
            }
        }
        active(cancel)?;
        Ok(NativeRoofRenderer {
            plan: self.plan.clone(),
            roofs,
            raw,
            animation: world_roofs::Animation::default(),
        })
    }
}
pub struct NativeRoofRenderer {
    plan: NativeAssetPlan,
    roofs: RoofMap,
    raw: BTreeMap<(u16, u8), DecodedTile>,
    animation: world_roofs::Animation,
}
impl NativeRoofRenderer {
    pub fn render(
        &mut self,
        assets: &NativeStaticAssets,
        camera: CameraRect,
        world: u64,
        selected: u64,
        cell: WorldCell,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Canvas> {
        active(cancel)?;
        ensure!(
            self.plan == assets.plan,
            "Native roof source/geometry identity mismatch"
        );
        let frame = self
            .animation
            .sample(&self.roofs, world, selected, cell, now)?;
        assets.render(camera, Some((&frame, &self.raw)), cancel)
    }
}

#[cfg(test)]
#[path = "native_resources_tests.rs"]
mod tests;

/// Separate full-city profile. Existing load_static and its total draw cap are unchanged.
#[path = "native_city_resources.rs"]
pub mod city;
