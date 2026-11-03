//! Bounded, read-only shared inventory icons; not SPRs, ground graphics or LT assets.
//!
//! Proven naming/routing (not an ID similarity heuristic): LiTo's `sub_43F6C0`
//! reads `list.ico` and formats each icon as `%d.tbt`; `sub_43FFC0` sums the
//! filename's ASCII bytes modulo 16. TW13081901's `sub_45B340` / `sub_4A7110`
//! implement the same decimal name and hash. Both try that SpriteNN member,
//! then Sprite only when the member is absent. LiTo's later LTlist.ico path is
//! deliberately excluded. The selected list declares a range, not availability.
//!
//! TBT parsing follows PakViewer `ImageConverter.LoadL1Image` (NOT its Raw
//! overload, whose skip units differ): four u8 header fields, rows of spans,
//! byte skips / 2, little-endian RGB555. Unlike the older launcher decoder,
//! truncated rows are errors, never silently replaced with transparent rows.
//! Raw covered RGB555 zero is retained: `reference_rgba` applies PakViewer's
//! zero-key preview convention, not a claim about all native blit/alpha paths.
//! IMG, PNG, alternate encodings, odd byte skips and high-bit pixels have no
//! implicit fallback. Offsets are retained, never guessed into an inventory UI.

use super::resources::{ResourceError, SharedArchive, SharedArchiveKind};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub const LIST_ENTRY_NAME: &str = "list.ico";
pub const MAX_DECLARED_ICONS: u32 = 1_000_000;
pub const MAX_TBT_BYTES: usize = 256 * 1024;
pub const MAX_TBT_WIDTH: usize = 512;
pub const MAX_TBT_HEIGHT: usize = 255;
pub const MAX_TBT_PIXELS: usize = MAX_TBT_WIDTH * MAX_TBT_HEIGHT;
const MAX_LIST_BYTES: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IconSource {
    pub archive: SharedArchiveKind,
    pub name: String,
}

#[derive(Debug)]
pub enum IconError {
    Cancelled,
    Resource(ResourceError),
    InvalidList,
    OutsideDeclaredRange { graphic: i64, declared: u32 },
    PayloadTooLarge { size: u64, limit: usize },
    Truncated,
    TrailingBytes,
    EmptyImage,
    DimensionLimit,
    OddByteSkip,
    UnsupportedPixelHighBit,
    AllocationFailed,
}

impl fmt::Display for IconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "shared inventory icon loading cancelled"),
            Self::Resource(error) => write!(f, "{error}"),
            Self::InvalidList => write!(f, "unsupported shared list.ico count format"),
            Self::OutsideDeclaredRange { graphic, declared } => write!(
                f,
                "inventory graphic {graphic} is outside shared list.ico range 0..{declared}"
            ),
            Self::PayloadTooLarge { size, limit } => {
                write!(f, "icon payload {size} bytes exceeds {limit} byte limit")
            }
            Self::Truncated => write!(f, "truncated TBT header, row or span"),
            Self::TrailingBytes => write!(f, "unsupported bytes after the final TBT row"),
            Self::EmptyImage => write!(f, "zero-dimensional TBT icon is unsupported"),
            Self::DimensionLimit => write!(f, "TBT span or dimensions exceed icon limits"),
            Self::OddByteSkip => write!(f, "unaligned TBT byte skip is unsupported"),
            Self::UnsupportedPixelHighBit => write!(f, "TBT pixel is not supported RGB555"),
            Self::AllocationFailed => write!(f, "unable to allocate bounded icon pixels"),
        }
    }
}

impl std::error::Error for IconError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Resource(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ResourceError> for IconError {
    fn from(error: ResourceError) -> Self {
        Self::Resource(error)
    }
}

/// No global cache or worker: the caller owns this reader and its lifetime.
/// It holds at most Sprite plus 16 shard IDX/PAK handle pairs; only a requested
/// small payload is read. A missing/corrupt archive is an error (not a member
/// miss); only an explicit MissingEntry permits the proven base fallback.
pub struct SharedInventoryIcons {
    root: PathBuf,
    base: SharedArchive,
    shards: [Option<SharedArchive>; 16],
    declared: u32,
    list_source: IconSource,
}

impl SharedInventoryIcons {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, IconError> {
        Self::open_cancellable(root, &AtomicBool::new(false))
    }

    /// Intended for a caller-owned resource worker, never the UI/heartbeat
    /// thread. Cancellation is checked around file operations and each decode
    /// row. The underlying synchronous IDX/payload read cannot be interrupted.
    pub fn open_cancellable(
        root: impl AsRef<Path>,
        cancelled: &AtomicBool,
    ) -> Result<Self, IconError> {
        check_cancelled(cancelled)?;
        let base = SharedArchive::open(root, SharedArchiveKind::Sprite)?;
        check_cancelled(cancelled)?;
        let mut result = Self {
            root: base.root().to_owned(),
            base,
            shards: std::array::from_fn(|_| None),
            declared: 0,
            list_source: IconSource {
                archive: SharedArchiveKind::Sprite,
                name: LIST_ENTRY_NAME.to_owned(),
            },
        };
        let (source, bytes) = result.read_member(LIST_ENTRY_NAME, MAX_LIST_BYTES, cancelled)?;
        result.declared = parse_list(&bytes)?;
        result.list_source = source;
        check_cancelled(cancelled)?;
        Ok(result)
    }

    pub fn declared_count(&self) -> u32 {
        self.declared
    }

    pub fn list_source(&self) -> &IconSource {
        &self.list_source
    }

    /// The original i64 graphic is validated before formatting; no u16 cast,
    /// clamp, modulo ID, LT offset subtraction or ground-graphic substitution.
    pub fn load(
        &mut self,
        inventory_graphic: i64,
        cancelled: &AtomicBool,
    ) -> Result<InventoryIcon, IconError> {
        check_cancelled(cancelled)?;
        let name = icon_name(inventory_graphic, self.declared)?;
        let (source, bytes) = self.read_member(&name, MAX_TBT_BYTES, cancelled)?;
        let image = decode_tbt_cancellable(&bytes, cancelled)?;
        check_cancelled(cancelled)?;
        Ok(InventoryIcon {
            inventory_graphic,
            source,
            image,
        })
    }

    fn read_member(
        &mut self,
        name: &str,
        limit: usize,
        cancelled: &AtomicBool,
    ) -> Result<(IconSource, Vec<u8>), IconError> {
        check_cancelled(cancelled)?;
        let shard = filename_shard(name);
        if self.shards[shard as usize].is_none() {
            let archive = SharedArchive::open(&self.root, SharedArchiveKind::SpriteShard(shard))?;
            check_cancelled(cancelled)?;
            self.shards[shard as usize] = Some(archive);
        }
        let part = self.shards[shard as usize].as_mut().unwrap();
        let base = &mut self.base;
        let result = missing_member_fallback(
            || read_bounded(part, name, limit, cancelled),
            || read_bounded(base, name, limit, cancelled),
        )?;
        check_cancelled(cancelled)?;
        Ok(result)
    }
}

fn missing_member_fallback<T>(
    preferred: impl FnOnce() -> Result<T, IconError>,
    base: impl FnOnce() -> Result<T, IconError>,
) -> Result<T, IconError> {
    match preferred() {
        Err(IconError::Resource(ResourceError::MissingEntry(_))) => base(),
        result => result,
    }
}

fn read_bounded(
    archive: &mut SharedArchive,
    name: &str,
    limit: usize,
    cancelled: &AtomicBool,
) -> Result<(IconSource, Vec<u8>), IconError> {
    check_cancelled(cancelled)?;
    let size = u64::from(archive.entry(name)?.size);
    if size > limit as u64 {
        return Err(IconError::PayloadTooLarge { size, limit });
    }
    let bytes = archive.read_entry(name)?;
    check_cancelled(cancelled)?;
    Ok((
        IconSource {
            archive: archive.kind(),
            name: name.to_owned(),
        },
        bytes,
    ))
}

fn filename_shard(name: &str) -> u8 {
    // All call sites use one fixed ASCII basename or bounded decimal digits.
    (name.bytes().map(u32::from).sum::<u32>() & 15) as u8
}

fn icon_name(graphic: i64, declared: u32) -> Result<String, IconError> {
    if graphic < 0 || graphic >= i64::from(declared) {
        return Err(IconError::OutsideDeclaredRange { graphic, declared });
    }
    Ok(format!("{graphic}.tbt"))
}

fn parse_list(bytes: &[u8]) -> Result<u32, IconError> {
    if bytes.len() > MAX_LIST_BYTES {
        return Err(IconError::InvalidList);
    }
    // Actual selected Sprite05 = 99999 CRLF, base Sprite = 2364 CRLF.
    // This is a count-only file, not a Windows ICO or a sequential name table.
    let digits = bytes.strip_suffix(b"\r\n").ok_or(IconError::InvalidList)?;
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return Err(IconError::InvalidList);
    }
    let mut count = 0u32;
    for digit in digits {
        count = count
            .checked_mul(10)
            .and_then(|value| value.checked_add(u32::from(digit - b'0')))
            .ok_or(IconError::InvalidList)?;
    }
    if count == 0 || count > MAX_DECLARED_ICONS {
        return Err(IconError::InvalidList);
    }
    Ok(count)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryIcon {
    inventory_graphic: i64,
    source: IconSource,
    image: TbtImage,
}

impl InventoryIcon {
    pub fn inventory_graphic(&self) -> i64 {
        self.inventory_graphic
    }
    pub fn source(&self) -> &IconSource {
        &self.source
    }
    pub fn image(&self) -> &TbtImage {
        &self.image
    }
}

/// Immutable natural-size pixels. `coverage` distinguishes encoded RGB555 zero
/// from skipped/uncovered space, so a later renderer can choose a proven blend
/// policy without losing original information. Raw offsets are not applied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TbtImage {
    offset: [u8; 2],
    header_width: u8,
    width: usize,
    height: usize,
    rgb555: Vec<u16>,
    coverage: Vec<bool>,
}

impl TbtImage {
    pub fn raw_offset(&self) -> [u8; 2] {
        self.offset
    }
    pub fn header_width(&self) -> u8 {
        self.header_width
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn rgb555(&self) -> &[u16] {
        &self.rgb555
    }
    pub fn coverage(&self) -> &[bool] {
        &self.coverage
    }

    /// PakViewer LoadL1Image's natural-size RGBA8 reference output, including
    /// its RGB555-zero transparency. NOT generic native alpha or UI placement.
    /// Channels use bit replication; there is no grayscale-as-alpha heuristic.
    pub fn reference_rgba(&self) -> Result<Vec<u8>, IconError> {
        let mut rgba = allocated(self.rgb555.len() * 4, 0u8)?;
        for (index, &pixel) in self.rgb555.iter().enumerate() {
            if !self.coverage[index] || pixel == 0 {
                continue;
            }
            let r = ((pixel >> 10) & 31) as u8;
            let g = ((pixel >> 5) & 31) as u8;
            let b = (pixel & 31) as u8;
            rgba[index * 4..index * 4 + 4].copy_from_slice(&[
                (r << 3) | (r >> 2),
                (g << 3) | (g >> 2),
                (b << 3) | (b >> 2),
                255,
            ]);
        }
        Ok(rgba)
    }
}

pub fn decode_tbt(bytes: &[u8]) -> Result<TbtImage, IconError> {
    decode_tbt_cancellable(bytes, &AtomicBool::new(false))
}

pub fn decode_tbt_cancellable(bytes: &[u8], cancelled: &AtomicBool) -> Result<TbtImage, IconError> {
    check_cancelled(cancelled)?;
    if bytes.len() > MAX_TBT_BYTES {
        return Err(IconError::PayloadTooLarge {
            size: bytes.len() as u64,
            limit: MAX_TBT_BYTES,
        });
    }
    let header = bytes.get(..4).ok_or(IconError::Truncated)?;
    let height = usize::from(header[3]);
    let header_width = header[2];
    if header_width == 0 || height == 0 {
        return Err(IconError::EmptyImage);
    }
    let width = walk_rows(
        bytes,
        height,
        usize::from(header_width),
        cancelled,
        |_, _, _| {},
    )?;
    let pixels = width.checked_mul(height).ok_or(IconError::DimensionLimit)?;
    if pixels > MAX_TBT_PIXELS || height > MAX_TBT_HEIGHT {
        return Err(IconError::DimensionLimit);
    }
    check_cancelled(cancelled)?;
    let mut rgb555 = allocated(pixels, 0u16)?;
    let mut coverage = allocated(pixels, false)?;
    walk_rows(bytes, height, width, cancelled, |y, x, value| {
        rgb555[y * width + x] = value;
        coverage[y * width + x] = true;
    })?;
    check_cancelled(cancelled)?;
    Ok(TbtImage {
        offset: [header[0], header[1]],
        header_width,
        width,
        height,
        rgb555,
        coverage,
    })
}

fn walk_rows(
    bytes: &[u8],
    height: usize,
    mut width: usize,
    cancelled: &AtomicBool,
    mut pixel: impl FnMut(usize, usize, u16),
) -> Result<usize, IconError> {
    let mut cursor = 4;
    for y in 0..height {
        check_cancelled(cancelled)?;
        let spans = take_byte(bytes, &mut cursor)?;
        let mut x = 0usize;
        for _ in 0..spans {
            let skip = take_byte(bytes, &mut cursor)?;
            if skip & 1 != 0 {
                return Err(IconError::OddByteSkip);
            }
            let count = usize::from(take_byte(bytes, &mut cursor)?);
            x = x
                .checked_add(usize::from(skip / 2))
                .ok_or(IconError::DimensionLimit)?;
            let end = x.checked_add(count).ok_or(IconError::DimensionLimit)?;
            if end > MAX_TBT_WIDTH {
                return Err(IconError::DimensionLimit);
            }
            width = width.max(end);
            for column in x..end {
                let lo = take_byte(bytes, &mut cursor)?;
                let hi = take_byte(bytes, &mut cursor)?;
                let value = u16::from_le_bytes([lo, hi]);
                if value & 0x8000 != 0 {
                    return Err(IconError::UnsupportedPixelHighBit);
                }
                pixel(y, column, value);
            }
            x = end;
        }
    }
    if cursor != bytes.len() {
        return Err(IconError::TrailingBytes);
    }
    Ok(width)
}

fn take_byte(bytes: &[u8], cursor: &mut usize) -> Result<u8, IconError> {
    let result = *bytes.get(*cursor).ok_or(IconError::Truncated)?;
    *cursor += 1;
    Ok(result)
}

fn allocated<T: Clone>(len: usize, value: T) -> Result<Vec<T>, IconError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(len)
        .map_err(|_| IconError::AllocationFailed)?;
    result.resize(len, value);
    Ok(result)
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), IconError> {
    if cancelled.load(Ordering::Relaxed) {
        Err(IconError::Cancelled)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn native_decimal_name_and_ascii_hash_have_no_id_reindexing() {
        assert_eq!(filename_shard(LIST_ENTRY_NAME), 5);
        assert_eq!(filename_shard("1068.tbt"), 7);
        assert_eq!(filename_shard("2401.tbt"), 15);
        assert_eq!(icon_name(0, 99999).unwrap(), "0.tbt");
        assert_eq!(icon_name(65536, 99999).unwrap(), "65536.tbt");
        for id in [-1, 99999, i64::MAX] {
            assert!(matches!(
                icon_name(id, 99999),
                Err(IconError::OutsideDeclaredRange { .. })
            ));
        }
    }

    #[test]
    fn count_only_list_is_bounded_and_not_an_availability_claim() {
        assert_eq!(parse_list(b"99999\r\n").unwrap(), 99999);
        assert_eq!(parse_list(b"2364\r\n").unwrap(), 2364);
        for bad in [
            b"".as_slice(),
            b"0\r\n",
            b"99999",
            b"99\n",
            b"-1\r\n",
            b"1\r\n2\r\n",
            b" 1\r\n",
            b"1 \r\n",
            b"1000001\r\n",
            b"4294967296\r\n",
            b"999999999999999999999\r\n",
            b"\xef\xbb\xbf1\r\n",
        ] {
            assert!(matches!(parse_list(bad), Err(IconError::InvalidList)));
        }
    }

    #[test]
    fn only_missing_preferred_member_allows_base_lookup() {
        let base_called = Cell::new(false);
        let value = missing_member_fallback(
            || Ok(7),
            || {
                base_called.set(true);
                Ok(8)
            },
        )
        .unwrap();
        assert_eq!(value, 7);
        assert!(!base_called.get());
        assert_eq!(
            missing_member_fallback(
                || Err(IconError::Resource(ResourceError::MissingEntry(
                    "5.tbt".into()
                ))),
                || Ok(8)
            )
            .unwrap(),
            8
        );
        for error in [
            IconError::Resource(ResourceError::AmbiguousEntry {
                name: "5.tbt".into(),
                count: 2,
            }),
            IconError::Resource(ResourceError::DuplicateEntry("5.tbt".into())),
            IconError::Resource(ResourceError::MalformedIndex("fixture")),
            IconError::Resource(ResourceError::Io(std::io::Error::from(
                std::io::ErrorKind::NotFound,
            ))),
            IconError::PayloadTooLarge {
                size: 100,
                limit: 4,
            },
            IconError::Cancelled,
        ] {
            assert!(missing_member_fallback(
                || Err(error),
                || {
                    base_called.set(true);
                    Ok(8)
                }
            )
            .is_err());
            assert!(!base_called.get());
        }
    }

    #[test]
    fn rle_byte_skip_and_rgb555_are_not_rgb565_or_grayscale_alpha() {
        let bytes = [5, 3, 4, 1, 1, 4, 2, 0xff, 0x7f, 0x00, 0x7c];
        let image = decode_tbt(&bytes).unwrap();
        assert_eq!(image.raw_offset(), [5, 3]);
        assert_eq!(
            (image.width(), image.height(), image.header_width()),
            (4, 1, 4)
        );
        assert_eq!(image.rgb555(), &[0, 0, 0x7fff, 0x7c00]);
        assert_eq!(image.coverage(), &[false, false, true, true]);
        assert_eq!(
            image.reference_rgba().unwrap(),
            [0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 255]
        );
    }

    #[test]
    fn covered_zero_is_preserved_before_reference_zero_key() {
        let image = decode_tbt(&[0, 0, 2, 1, 1, 2, 1, 0, 0]).unwrap();
        assert_eq!(image.rgb555(), &[0, 0]);
        assert_eq!(image.coverage(), &[false, true]);
        assert_eq!(image.reference_rgba().unwrap(), [0; 8]);
    }

    #[test]
    fn span_width_can_extend_header_but_never_the_bounded_surface() {
        let bytes = [0, 0, 1, 1, 1, 2, 2, 0x1f, 0, 0xe0, 3];
        let image = decode_tbt(&bytes).unwrap();
        assert_eq!((image.header_width(), image.width()), (1, 3));
        assert_eq!(image.rgb555(), &[0, 31, 992]);
        let mut wide = vec![0, 0, 1, 1, 5];
        for _ in 0..5 {
            wide.extend_from_slice(&[254, 0]);
        }
        assert!(matches!(decode_tbt(&wide), Err(IconError::DimensionLimit)));
    }

    #[test]
    fn every_truncation_is_rejected_without_partial_image_or_row_padding() {
        let complete = [0, 0, 1, 2, 1, 0, 1, 0, 0x7c, 0];
        assert!(decode_tbt(&complete).is_ok());
        for end in 0..complete.len() {
            assert!(
                matches!(decode_tbt(&complete[..end]), Err(IconError::Truncated)),
                "prefix {end}"
            );
        }
        let mut trailing = complete.to_vec();
        trailing.push(0);
        assert!(matches!(
            decode_tbt(&trailing),
            Err(IconError::TrailingBytes)
        ));
    }

    #[test]
    fn empty_unproven_alignment_pixel_format_and_large_payload_fail_explicitly() {
        assert!(matches!(
            decode_tbt(&[0, 0, 0, 1]),
            Err(IconError::EmptyImage)
        ));
        assert!(matches!(
            decode_tbt(&[0, 0, 1, 0]),
            Err(IconError::EmptyImage)
        ));
        assert!(matches!(
            decode_tbt(&[0, 0, 1, 1, 1, 1, 0]),
            Err(IconError::OddByteSkip)
        ));
        assert!(matches!(
            decode_tbt(&[0, 0, 1, 1, 1, 0, 1, 0, 0x80]),
            Err(IconError::UnsupportedPixelHighBit)
        ));
        assert!(matches!(
            decode_tbt(&vec![0; MAX_TBT_BYTES + 1]),
            Err(IconError::PayloadTooLarge { .. })
        ));
    }

    #[test]
    fn pre_cancelled_loader_does_not_open_paths_or_publish_pixels() {
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            SharedInventoryIcons::open_cancellable("does-not-exist", &cancelled),
            Err(IconError::Cancelled)
        ));
        assert!(matches!(
            decode_tbt_cancellable(&[0, 0, 1, 1, 0], &cancelled),
            Err(IconError::Cancelled)
        ));
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; reads eight small TBTs and selected indexes only"]
    fn real_shared_inventory_icons_match_independent_pakviewer_load_l1_image() {
        use sha2::{Digest, Sha256};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let cancelled = AtomicBool::new(false);
        let mut reader = SharedInventoryIcons::open(Path::new(&root)).unwrap();
        assert_eq!(reader.declared_count(), 99999);
        assert_eq!(
            reader.list_source().archive,
            SharedArchiveKind::SpriteShard(5)
        );
        let cases = [
            (
                0,
                SharedArchiveKind::Sprite,
                711,
                31,
                31,
                [0, 0],
                280,
                20,
                "3E7FCDD96AF16B13D491245B91AE49C9E9153597515B760A8CD0569CE89D1EC5",
                "53C6C313255C5AB1FA2FA9BC195202BEBF46144C3D9542C1E81ED043E40088AC",
            ),
            (
                5,
                SharedArchiveKind::Sprite,
                803,
                29,
                23,
                [2, 7],
                360,
                0,
                "440043937792F507F54C8D6CA1878DF1AA660E9446AEA3ED2C2E35833576E3A7",
                "F73FB98C70BE418D6370AD3D66C31A6B99F5CA494A10F69A0FA1EF345CF8DD50",
            ),
            (
                77,
                SharedArchiveKind::Sprite,
                594,
                16,
                24,
                [8, 4],
                259,
                0,
                "51047F9B5F219D5CCABD6ED0D6C091DFCCEC341E38DEA0A7C96A747DFD06051D",
                "5C3335E7C0730F26D41734FFEB97DDBD66A87333305CEC117E8DAD79FDA3F483",
            ),
            (
                508,
                SharedArchiveKind::Sprite,
                894,
                32,
                32,
                [0, 0],
                399,
                0,
                "11749EE6967B0CB283D785DF450CF652BAE1AD99FE056D66CE9F9A56D49C75CA",
                "8F5B655D147C81FBDF3597D5F342D8DF077E2022DEB742AB208A3CFE6955ABF7",
            ),
            (
                628,
                SharedArchiveKind::Sprite,
                879,
                28,
                23,
                [2, 5],
                394,
                0,
                "570DC0307084B90D8FAB4F56700BBC79AC72A3761EE7C722A5E662A8E89354E0",
                "E6E6EBA1E8589D9232AFB90D33694CB64DA8666D37AE3B9713CBE4109F3450C5",
            ),
            (
                912,
                SharedArchiveKind::Sprite,
                756,
                19,
                30,
                [6, 1],
                330,
                0,
                "144AA2D8EE0F8D18204231A6CFEAB8005A4BA68CF045369DC6B123DC7850DC62",
                "273AC19CB839687FF0E0F76C58A587A8A246CCA5B3D2E6741CD352DF9A26179D",
            ),
            (
                1068,
                SharedArchiveKind::SpriteShard(7),
                446,
                22,
                26,
                [4, 4],
                162,
                0,
                "AC905FD43D99E18C6BBB6C0B96CE04B7D238638D42D0976E2C7446D733FD2550",
                "AB3A783BA47F369F1D70B0FC6134702BCC69A01E3176E98C280F960B37AB57B4",
            ),
            (
                2401,
                SharedArchiveKind::SpriteShard(15),
                1632,
                30,
                32,
                [1, 0],
                761,
                0,
                "D3C31A2274A30A6ED1E44ED4D698127EF4E6D8EC34CC26F224A10D6E675F857D",
                "684787B922C775B344D4CBDB9EE858898528165EB73070F2D09B9043F158114F",
            ),
        ];
        for (id, archive, size, width, height, offset, covered, zero, raw_sha, rgba_sha) in cases {
            let name = icon_name(id, reader.declared_count()).unwrap();
            let (source, raw) = reader
                .read_member(&name, MAX_TBT_BYTES, &cancelled)
                .unwrap();
            assert_eq!(source, IconSource { archive, name });
            assert_eq!(raw.len(), size);
            assert_eq!(format!("{:X}", Sha256::digest(&raw)), raw_sha);
            let icon = reader.load(id, &cancelled).unwrap();
            assert_eq!(icon.source(), &source);
            assert_eq!(icon.inventory_graphic(), id);
            let image = icon.image();
            assert_eq!(
                (image.width(), image.height(), image.raw_offset()),
                (width, height, offset)
            );
            assert_eq!(
                image.coverage().iter().filter(|&&value| value).count(),
                covered
            );
            assert_eq!(
                image
                    .rgb555()
                    .iter()
                    .zip(image.coverage())
                    .filter(|(value, coverage)| **value == 0 && **coverage)
                    .count(),
                zero
            );
            assert_eq!(
                format!("{:X}", Sha256::digest(image.reference_rgba().unwrap())),
                rgba_sha
            );
        }
        // Declared but absent is not icon 0 or a similarly numbered asset.
        assert!(matches!(
            reader.load(99998, &cancelled),
            Err(IconError::Resource(ResourceError::MissingEntry(_)))
        ));
        assert!(matches!(
            reader.load(99999, &cancelled),
            Err(IconError::OutsideDeclaredRange { .. })
        ));
        cancelled.store(true, Ordering::Relaxed);
        assert!(matches!(
            reader.load(5, &cancelled),
            Err(IconError::Cancelled)
        ));
    }

    #[test]
    #[ignore = "requires the real shared Lineage Sprite archives"]
    fn real_shared_icons_cover_all_graphics_in_independent_two_source_inventory_fixture() {
        // Numeric-only independent DB probe, SHA256 776F6B1844F083C21CECFFFEA74AA02161FF2FEC4716064C13008DF4F5883488:
        // 2 sources x 4 characters, 784 item rows, these 146 unique invgfx values.
        // This is resource availability/decoding coverage, NOT a second visual
        // golden and not a claim that the complete item catalog has been tested.
        let ids: [i64; 146] = [
            4, 5, 30, 51, 58, 73, 75, 76, 79, 80, 82, 87, 89, 90, 96, 99, 104, 130, 235, 236, 237,
            238, 239, 240, 242, 243, 244, 245, 249, 250, 251, 252, 254, 255, 256, 257, 264, 301,
            318, 320, 324, 332, 435, 442, 443, 444, 446, 449, 451, 452, 454, 455, 460, 467, 468,
            470, 471, 472, 473, 474, 475, 478, 487, 543, 544, 545, 546, 621, 625, 626, 630, 631,
            636, 637, 673, 674, 675, 676, 680, 682, 683, 685, 686, 687, 699, 705, 706, 710, 828,
            858, 859, 860, 862, 864, 903, 905, 957, 965, 966, 967, 968, 1053, 1093, 1137, 1139,
            1166, 1171, 1187, 1188, 1194, 1201, 1205, 1341, 1416, 1480, 1524, 1529, 1635, 1760,
            1779, 2244, 2442, 2516, 2553, 2560, 2608, 2727, 2796, 2833, 2926, 3035, 3212, 3723,
            4110, 5645, 6007, 6027, 6101, 6102, 6103, 8101, 8103, 8105, 8106, 8110, 8112,
        ];
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared resource root");
        let cancelled = AtomicBool::new(false);
        let mut reader = SharedInventoryIcons::open_cancellable(PathBuf::from(root), &cancelled)
            .expect("read shared inventory index");
        let mut failures = Vec::new();
        let mut from_base = 0;
        for id in ids {
            match reader.load(id, &cancelled) {
                Ok(icon) => {
                    assert_eq!(icon.inventory_graphic(), id);
                    assert!(icon.image().coverage().iter().any(|value| *value));
                    if icon.source().archive == SharedArchiveKind::Sprite {
                        from_base += 1;
                    }
                }
                Err(error) => failures.push(format!("{id}: {error}")),
            }
        }
        assert!(
            failures.is_empty(),
            "unresolved shared inventory graphics: {failures:?}"
        );
        eprintln!(
            "shared source-inventory graphics: {} decoded, {from_base} base, {} shard",
            ids.len(),
            ids.len() - from_base
        );
    }
}
