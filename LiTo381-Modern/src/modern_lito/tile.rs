//! Classic, uncompressed Lineage TIL to bounded 24 x 24 RGBA tiles.
//!
//! This implements the shared Tile archive's observed types 0/1 (right/left
//! half-diamonds), 2/3 (RGB555 scanline RLE), 6/7 (50% opacity RLE),
//! 8/9 (half-diamonds retaining flag 0x08), and
//! 16/17 (cloud-alpha half-diamonds), 18/19, 22/23 and 34/35 (RLE with the
//! reference decoder's cloud/smoke/blood effects).
//! The default preserves actual pixel coverage, as PakViewer's direct map
//! renderer does: covered RGB555 zero is opaque black, while skipped pixels
//! stay transparent. An explicit PreviewColorKey policy instead follows
//! `L1Til.RenderBlockToBgra(..., transparentBackground: true)` for types 0..=3/8/9.
//! Types 8/9 use the existing C# reference's diamond geometry and opaque color
//! policy; bit 3 is retained, with no additional native-bin semantics inferred.
//! Nonzero colors expand 5-bit channels to 8-bit. Types
//! 6/7 preserve every covered pixel, including RGB555 zero, at alpha 128:
//! `L1Til.RenderBlockDirect(..., applyTypeAlpha: true)` blends these black
//! shadow pixels, unlike `RenderBlockToBgra`, which discards all zero colors.
//! Uncovered pixels remain transparent. RGBA is straight 8-bit color; source
//! 50% semantics are retained, not the direct renderer's 5-bit rounding.
//! Types 16/17, 18/19 and 34/35 follow `L1Til.CalculateInvertedAlpha` and
//! `GetInvertedAlphaRenderColor`: their four explicit color keys are transparent,
//! alpha uses the maximum RGB555 component (16..=19) or its inverse (34/35),
//! near-gray pixels use 0x739c or 0x6318 respectively, and colored pixels retain
//! their hue at 2/3 channel intensity. This is the
//! existing PakViewer reference's effect policy, not an original-bin capture.
//! As with half-opacity tiles, composition uses straight RGBA8 rather than the
//! direct reference renderer's rounded RGB555 destination arithmetic.
//! Native `sub_483A80` +7BB..7F7 and `sub_4853E0` +96..C7 prove that
//! types 22/23 select RLE geometry and cloud flag 8 before half-opacity 256.
//! Their retained bit 0x04 therefore does not halve cloud alpha. Only these
//! two observed combinations are added; other unknown types still refuse.
//! Remaster, hybrid coordinates, compressed containers and other effect types are
//! reported as unsupported instead of being clipped or reinterpreted.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const CLASSIC_TILE_SIDE: u32 = 24;
const PIXELS: usize = 24 * 24;
const RGBA_BYTES: usize = PIXELS * 4;
const HARD_MAX_TILES: usize = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ClassicTileType {
    DiamondRight = 0,
    DiamondLeft = 1,
    RleEven = 2,
    RleOdd = 3,
    RleEvenHalfOpacity = 6,
    RleOddHalfOpacity = 7,
    DiamondRightFlag8 = 8,
    DiamondLeftFlag8 = 9,
    DiamondRightCloudAlpha = 16,
    DiamondLeftCloudAlpha = 17,
    RleEvenCloudAlpha = 18,
    RleOddCloudAlpha = 19,
    RleEvenCloudFlag4 = 22,
    RleOddCloudFlag4 = 23,
    RleEvenInvertedAlpha = 34,
    RleOddInvertedAlpha = 35,
}

/// Zero RGB555 is not sufficient to identify an uncovered pixel. A map must
/// preserve RLE/diamond coverage; an isolated asset preview may deliberately
/// color-key black to match the older high-level C# preview API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TileZeroPolicy {
    PreserveCoverage,
    /// Applies only to types 0..=3/8/9. Half-opacity types 6/7 always retain covered
    /// black; 16/17/18/19/22/23/34/35 use explicit effect color keys in both policies.
    PreviewColorKey,
    /// Original RGB555 plus binary encoded coverage, before any type effect.
    /// Internal world roof blending only; never a substitute for preview alpha.
    RawRgb555Coverage,
}

impl ClassicTileType {
    fn from_byte(value: u8) -> Result<Self, TileError> {
        match value {
            0 => Ok(Self::DiamondRight),
            1 => Ok(Self::DiamondLeft),
            2 => Ok(Self::RleEven),
            3 => Ok(Self::RleOdd),
            6 => Ok(Self::RleEvenHalfOpacity),
            7 => Ok(Self::RleOddHalfOpacity),
            8 => Ok(Self::DiamondRightFlag8),
            9 => Ok(Self::DiamondLeftFlag8),
            16 => Ok(Self::DiamondRightCloudAlpha),
            17 => Ok(Self::DiamondLeftCloudAlpha),
            18 => Ok(Self::RleEvenCloudAlpha),
            19 => Ok(Self::RleOddCloudAlpha),
            22 => Ok(Self::RleEvenCloudFlag4),
            23 => Ok(Self::RleOddCloudFlag4),
            34 => Ok(Self::RleEvenInvertedAlpha),
            35 => Ok(Self::RleOddInvertedAlpha),
            _ => Err(TileError::UnsupportedType(value)),
        }
    }

    fn is_half_opacity(self) -> bool {
        matches!(self, Self::RleEvenHalfOpacity | Self::RleOddHalfOpacity)
    }

    fn is_inverted_alpha(self) -> bool {
        matches!(
            self,
            Self::DiamondRightCloudAlpha
                | Self::DiamondLeftCloudAlpha
                | Self::RleEvenCloudAlpha
                | Self::RleOddCloudAlpha
                | Self::RleEvenCloudFlag4
                | Self::RleOddCloudFlag4
                | Self::RleEvenInvertedAlpha
                | Self::RleOddInvertedAlpha
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TileLimits {
    pub max_input_bytes: usize,
    pub max_tiles: usize,
    pub max_total_pixels: usize,
}

impl Default for TileLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024 * 1024,
            max_tiles: 16_384,
            max_total_pixels: 16 * 1024 * 1024,
        }
    }
}

impl TileLimits {
    fn validate(self) -> Result<Self, TileError> {
        if self.max_input_bytes == 0
            || self.max_input_bytes > 256 * 1024 * 1024
            || self.max_tiles == 0
            || self.max_tiles > HARD_MAX_TILES
            || self.max_total_pixels == 0
            || self.max_total_pixels > 64 * 1024 * 1024
        {
            return Err(TileError::InvalidLimits);
        }
        Ok(self)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TileError {
    InvalidLimits,
    LimitExceeded(&'static str),
    Truncated { offset: usize, needed: usize },
    UnsupportedFormat(&'static str),
    UnsupportedType(u8),
    InvalidData(&'static str),
    AllocationFailed,
}

impl fmt::Display for TileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits => write!(f, "invalid bounded TIL limits"),
            Self::LimitExceeded(what) => write!(f, "TIL exceeds {what} limit"),
            Self::Truncated { offset, needed } => {
                write!(f, "truncated TIL at byte {offset}, need {needed} bytes")
            }
            Self::UnsupportedFormat(what) => write!(f, "unsupported TIL format: {what}"),
            Self::UnsupportedType(kind) => write!(f, "unsupported classic TIL block type {kind}"),
            Self::InvalidData(what) => write!(f, "invalid classic TIL: {what}"),
            Self::AllocationFailed => write!(f, "unable to allocate bounded TIL output"),
        }
    }
}
impl std::error::Error for TileError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedTile {
    pub tile_type: ClassicTileType,
    /// Relative to the TIL block-data start, not to the PAK or index header.
    pub source_offset: u32,
    /// RLE's declared content rectangle. Diamond tiles report the whole cell.
    /// These offsets are already applied inside `rgba`; do not add them again
    /// when placing the complete tile on a map.
    pub x_offset: u8,
    pub y_offset: u8,
    pub content_width: u8,
    pub content_height: u8,
    pub width: u32,
    pub height: u32,
    /// Row-major straight RGBA8, always 24 * 24 * 4 bytes.
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedTileSet {
    /// Preserves the original tile index, including aliased block offsets.
    pub tiles: Vec<DecodedTile>,
}

struct Reader<'a> {
    data: &'a [u8],
    position: usize,
    base: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8], base: usize) -> Self {
        Self {
            data,
            position: 0,
            base,
        }
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], TileError> {
        let end = self
            .position
            .checked_add(count)
            .ok_or(TileError::InvalidData("offset overflow"))?;
        let result = self
            .data
            .get(self.position..end)
            .ok_or(TileError::Truncated {
                offset: self.base + self.position,
                needed: count,
            })?;
        self.position = end;
        Ok(result)
    }
    fn byte(&mut self) -> Result<u8, TileError> {
        Ok(self.take(1)?[0])
    }
    fn dword(&mut self) -> Result<u32, TileError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
}

fn bounded_vec<T>(length: usize) -> Result<Vec<T>, TileError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(length)
        .map_err(|_| TileError::AllocationFailed)?;
    Ok(output)
}

pub fn decode_classic_til(data: &[u8]) -> Result<DecodedTileSet, TileError> {
    decode_classic_til_with_limits(data, TileLimits::default())
}

pub fn decode_classic_til_with_limits(
    data: &[u8],
    limits: TileLimits,
) -> Result<DecodedTileSet, TileError> {
    decode_classic_til_with_policy(data, limits, TileZeroPolicy::PreserveCoverage)
}

pub fn decode_classic_til_with_policy(
    data: &[u8],
    limits: TileLimits,
    zero_policy: TileZeroPolicy,
) -> Result<DecodedTileSet, TileError> {
    Ok(DecodedTileSet {
        tiles: decode_entries(data, limits, zero_policy, None)?
            .into_iter()
            .map(|(_, tile)| tile)
            .collect(),
    })
}

/// Decode every block referenced by the complete map, retaining original IDs.
/// The entire offset table/container is validated, including unused offsets;
/// unused block payloads are not decoded. No missing required block is replaced
/// with a blank tile. Encoded bytes/table count and retained pixels are bounded
/// independently, so sparse selection cannot bypass the input/table limits.
pub fn decode_classic_til_selected(
    data: &[u8],
    limits: TileLimits,
    selected: &BTreeSet<usize>,
) -> Result<BTreeMap<usize, DecodedTile>, TileError> {
    decode_selected_with_policy(data, limits, selected, TileZeroPolicy::PreserveCoverage)
}

/// Explicit opt-in for a classic asset whose unused terminal table slots all
/// equal its exact encoded EOF. IDs and real block boundaries are preserved.
/// A selected empty slot is an error, never a transparent replacement tile.
/// Interior empty slots, out-of-body offsets, trailing bytes and all existing
/// input/table/pixel limits remain rejected. Ordinary Shared/full decoders keep
/// their strict contract; this does not establish a Native/Shared asset binding.
pub fn decode_classic_til_selected_with_empty_tail(
    data: &[u8],
    limits: TileLimits,
    selected: &BTreeSet<usize>,
) -> Result<BTreeMap<usize, DecodedTile>, TileError> {
    ValidatedTileInput::new_with_empty_tail(data, limits)?.decode_selected(selected, false)
}

pub(crate) fn decode_roof_til_selected(
    data: &[u8],
    limits: TileLimits,
    selected: &BTreeSet<usize>,
) -> Result<BTreeMap<usize, DecodedTile>, TileError> {
    decode_selected_with_policy(data, limits, selected, TileZeroPolicy::RawRgb555Coverage)
}

fn decode_selected_with_policy(
    data: &[u8],
    limits: TileLimits,
    selected: &BTreeSet<usize>,
    policy: TileZeroPolicy,
) -> Result<BTreeMap<usize, DecodedTile>, TileError> {
    if selected.is_empty() {
        return Err(TileError::InvalidData("empty tile selection"));
    }
    Ok(decode_entries(data, limits, policy, Some(selected))?
        .into_iter()
        .collect())
}

fn decode_entries(
    data: &[u8],
    limits: TileLimits,
    zero_policy: TileZeroPolicy,
    selected: Option<&BTreeSet<usize>>,
) -> Result<Vec<(usize, DecodedTile)>, TileError> {
    ValidatedTileInput::new(data, limits)?.decode_entries(zero_policy, selected)
}

/// Immutable borrowed payload and a fully checked offset table. A bounded
/// batch decoder can reuse this index without sorting/scanning the entire
/// original table again for every 64 selected tiles. No pixels are cached here.
pub(crate) struct ValidatedTileInput<'a> {
    body: &'a [u8],
    data_start: usize,
    offsets: Vec<u32>,
    boundaries: Vec<u32>,
    limits: TileLimits,
}
impl<'a> ValidatedTileInput<'a> {
    pub(crate) fn new(data: &'a [u8], limits: TileLimits) -> Result<Self, TileError> {
        Self::validate(data, limits, false)
    }

    pub(crate) fn new_with_empty_tail(data: &'a [u8], limits: TileLimits) -> Result<Self, TileError> {
        Self::validate(data, limits, true)
    }

    fn validate(
        data: &'a [u8],
        limits: TileLimits,
        allow_empty_tail: bool,
    ) -> Result<Self, TileError> {
        let limits = limits.validate()?;
        if data.len() > limits.max_input_bytes {
            return Err(TileError::LimitExceeded("input bytes"));
        }
        let mut reader = Reader::new(data, 0);
        let count = reader.dword()? as usize;
        // Do not guess compression from just the low byte of a valid raw count:
        // 120/376/91 are legal counts whose first byte resembles a stream marker.
        if count > HARD_MAX_TILES
            && (data.first() == Some(&0x78) || matches!(data.first(), Some(0x5b | 0x1b)))
        {
            return Err(TileError::UnsupportedFormat("compressed container"));
        }
        if count == 0 {
            return Err(TileError::InvalidData("zero tile count"));
        }
        if count > limits.max_tiles {
            return Err(TileError::LimitExceeded("tile count"));
        }
        let table_bytes = count
            .checked_add(1)
            .and_then(|n| n.checked_mul(4))
            .ok_or(TileError::InvalidData("table size overflow"))?;
        // Prove the complete offset table exists before allocating by its count.
        let table = reader.take(table_bytes)?;
        let body = &data[reader.position..];
        let data_start = reader.position;
        let mut table_reader = Reader::new(table, 4);
        let mut offsets = bounded_vec(count)?;
        for _ in 0..count {
            offsets.push(table_reader.dword()?);
        }
        let encoded_end = table_reader.dword()? as usize;
        if encoded_end > body.len() {
            return Err(TileError::InvalidData("end offset exceeds body"));
        }
        if encoded_end != body.len() {
            return Err(TileError::UnsupportedFormat(
                "trailing data after end offset",
            ));
        }
        if offsets.iter().any(|off| *off as usize > encoded_end) {
            return Err(TileError::InvalidData("block offset outside body"));
        }
        if let Some(first) = offsets.iter().position(|off| *off as usize == encoded_end) {
            if !allow_empty_tail {
                return Err(TileError::InvalidData("block offset outside body"));
            }
            if first == 0 || offsets[first..].iter().any(|off| *off as usize != encoded_end) {
                return Err(TileError::InvalidData("empty slots must be a terminal suffix"));
            }
        }
        let mut boundaries = bounded_vec(count + 1)?;
        boundaries.extend(offsets.iter().copied().filter(|off| (*off as usize) < encoded_end));
        boundaries.sort_unstable();
        boundaries.dedup();
        if boundaries.first() != Some(&0) {
            return Err(TileError::UnsupportedFormat("prefix before first block"));
        }
        boundaries.push(encoded_end as u32);
        Ok(Self {
            body,
            data_start,
            offsets,
            boundaries,
            limits,
        })
    }

    pub(crate) fn decode_selected(
        &self,
        selected: &BTreeSet<usize>,
        raw_roof: bool,
    ) -> Result<BTreeMap<usize, DecodedTile>, TileError> {
        if selected.is_empty() {
            return Err(TileError::InvalidData("empty tile selection"));
        }
        let policy = if raw_roof {
            TileZeroPolicy::RawRgb555Coverage
        } else {
            TileZeroPolicy::PreserveCoverage
        };
        Ok(self
            .decode_entries(policy, Some(selected))?
            .into_iter()
            .collect())
    }

    fn decode_entries(
        &self,
        zero_policy: TileZeroPolicy,
        selected: Option<&BTreeSet<usize>>,
    ) -> Result<Vec<(usize, DecodedTile)>, TileError> {
        let count = self.offsets.len();
        if selected.is_some_and(|ids| ids.last().is_some_and(|id| *id >= count)) {
            return Err(TileError::InvalidData("selected tile index outside table"));
        }
        let retained = selected.map_or(count, BTreeSet::len);
        let total_pixels = retained
            .checked_mul(PIXELS)
            .ok_or(TileError::LimitExceeded("total pixels"))?;
        if total_pixels > self.limits.max_total_pixels {
            return Err(TileError::LimitExceeded("total pixels"));
        }
        let mut tiles = bounded_vec(retained)?;
        let mut decode = |index: usize| -> Result<(), TileError> {
            let offset = self.offsets[index];
            if offset as usize == self.body.len() {
                return Err(TileError::InvalidData("selected tile slot has no block"));
            }
            let position = self
                .boundaries
                .binary_search(&offset)
                .map_err(|_| TileError::InvalidData("missing block boundary"))?;
            let start = offset as usize;
            let end = self.boundaries[position + 1] as usize;
            tiles.push((
                index,
                decode_block(
                    &self.body[start..end],
                    self.data_start + start,
                    offset,
                    zero_policy,
                )?,
            ));
            Ok(())
        };
        if let Some(indices) = selected {
            for &index in indices {
                decode(index)?;
            }
        } else {
            for index in 0..count {
                decode(index)?;
            }
        }
        Ok(tiles)
    }
}

fn decode_block(
    data: &[u8],
    base: usize,
    source_offset: u32,
    zero_policy: TileZeroPolicy,
) -> Result<DecodedTile, TileError> {
    let mut reader = Reader::new(data, base);
    let tile_type = ClassicTileType::from_byte(reader.byte()?)?;
    let mut rgba = bounded_vec(RGBA_BYTES)?;
    rgba.resize(RGBA_BYTES, 0);
    let (x_offset, y_offset, content_width, content_height) = match tile_type {
        ClassicTileType::DiamondRight
        | ClassicTileType::DiamondLeft
        | ClassicTileType::DiamondRightFlag8
        | ClassicTileType::DiamondLeftFlag8
        | ClassicTileType::DiamondRightCloudAlpha
        | ClassicTileType::DiamondLeftCloudAlpha => {
            // Exactly 288 RGB555 pixels; 48px diamonds contain 1152 pixels.
            if data.len() > 577 {
                return Err(TileError::UnsupportedFormat("non-24px diamond block"));
            }
            for y in 0..24usize {
                let count = if y <= 11 { (y + 1) * 2 } else { (23 - y) * 2 };
                let start_x = if tile_type as u8 & 1 == 0 {
                    24 - count
                } else {
                    0
                };
                let pixels = reader.take(count * 2)?;
                write_rgb555_run(
                    &mut rgba[(y * 24 + start_x) * 4..(y * 24 + start_x + count) * 4],
                    pixels,
                    tile_type,
                    zero_policy,
                );
            }
            (0, 0, 24, 24)
        }
        ClassicTileType::RleEven
        | ClassicTileType::RleOdd
        | ClassicTileType::RleEvenHalfOpacity
        | ClassicTileType::RleOddHalfOpacity
        | ClassicTileType::RleEvenCloudAlpha
        | ClassicTileType::RleOddCloudAlpha
        | ClassicTileType::RleEvenCloudFlag4
        | ClassicTileType::RleOddCloudFlag4
        | ClassicTileType::RleEvenInvertedAlpha
        | ClassicTileType::RleOddInvertedAlpha => {
            let x_offset = reader.byte()?;
            let y_offset = reader.byte()?;
            let width = reader.byte()?;
            let height = reader.byte()?;
            let right = usize::from(x_offset) + usize::from(width);
            let bottom = usize::from(y_offset) + usize::from(height);
            if right > 24 || bottom > 24 {
                return Err(TileError::UnsupportedFormat(
                    "non-24px RLE content rectangle",
                ));
            }
            for y in usize::from(y_offset)..bottom {
                let mut x = usize::from(x_offset);
                let segments = reader.byte()?;
                for _ in 0..segments {
                    let skip = reader.byte()?;
                    if skip % 2 != 0 {
                        return Err(TileError::UnsupportedFormat("odd RGB555 byte skip"));
                    }
                    let count = usize::from(reader.byte()?);
                    x += usize::from(skip / 2);
                    if x > right || x + count > right {
                        return Err(TileError::InvalidData(
                            "RLE segment exceeds content rectangle",
                        ));
                    }
                    let pixels = reader.take(count * 2)?;
                    write_rgb555_run(
                        &mut rgba[(y * 24 + x) * 4..(y * 24 + x + count) * 4],
                        pixels,
                        tile_type,
                        zero_policy,
                    );
                    x += count;
                }
            }
            (x_offset, y_offset, width, height)
        }
    };
    if reader.position != data.len() {
        return Err(TileError::UnsupportedFormat("trailing block data"));
    }
    Ok(DecodedTile {
        tile_type,
        source_offset,
        x_offset,
        y_offset,
        content_width,
        content_height,
        width: CLASSIC_TILE_SIDE,
        height: CLASSIC_TILE_SIDE,
        rgba,
    })
}

/// The caller validates the whole source run and destination rectangle before
/// entering this loop. Select color policy once per run, not once per pixel.
fn write_rgb555_run(
    rgba: &mut [u8],
    encoded: &[u8],
    tile_type: ClassicTileType,
    zero_policy: TileZeroPolicy,
) {
    debug_assert_eq!(encoded.len() % 2, 0);
    debug_assert_eq!(rgba.len(), encoded.len() * 2);
    if tile_type.is_inverted_alpha() && zero_policy != TileZeroPolicy::RawRgb555Coverage {
        let inverse = tile_type as u8 & 0x20 != 0;
        for (out, pair) in rgba.chunks_exact_mut(4).zip(encoded.chunks_exact(2)) {
            out.copy_from_slice(&inverted_alpha_rgba(
                u16::from_le_bytes([pair[0], pair[1]]),
                inverse,
            ));
        }
    } else {
        let alpha =
            if tile_type.is_half_opacity() && zero_policy != TileZeroPolicy::RawRgb555Coverage {
                128
            } else {
                255
            };
        let color_key =
            !tile_type.is_half_opacity() && zero_policy == TileZeroPolicy::PreviewColorKey;
        for (out, pair) in rgba.chunks_exact_mut(4).zip(encoded.chunks_exact(2)) {
            let rgb555 = u16::from_le_bytes([pair[0], pair[1]]);
            if color_key && rgb555 == 0 {
                out.fill(0);
                continue;
            }
            let (r, g, b) = ((rgb555 >> 10) & 31, (rgb555 >> 5) & 31, rgb555 & 31);
            out.copy_from_slice(&[
                ((r << 3) | (r >> 2)) as u8,
                ((g << 3) | (g >> 2)) as u8,
                ((b << 3) | (b >> 2)) as u8,
                alpha,
            ]);
        }
    }
}

/// Exact type-0x10/0x20 color/alpha policy from PakViewer
/// `src/Lin.Helper.Core/Tile/L1Til.cs` CalculateInvertedAlpha and
/// GetInvertedAlphaRenderColor. The color-key comparison deliberately precedes
/// masking bit 15, as it does in that reference. No effect is inferred for any
/// other unsupported type/flag combination.
fn inverted_alpha_rgba(source: u16, inverse: bool) -> [u8; 4] {
    if matches!(source, 0 | 0x0421 | 0x7fff | 0x7fe0) {
        return [0; 4];
    }
    let r = (source >> 10) & 31;
    let g = (source >> 5) & 31;
    let b = source & 31;
    let brightness = r.max(g).max(b);
    let opacity = if inverse { 31 - brightness } else { brightness };
    let alpha = (opacity * 255 / 31) as u8;
    if alpha < 8 {
        return [0; 4];
    }
    let (r, g, b) = if r.abs_diff(g) <= 2 && g.abs_diff(b) <= 2 {
        // 0x6318 / 0x739c are fixed RGB555 (24/28 per channel), not black/white
        // or luminance-as-color. Preserve the distinction between smoke/cloud.
        let gray = if inverse { 24 } else { 28 };
        (gray, gray, gray)
    } else {
        (r * 2 / 3, g * 2 / 3, b * 2 / 3)
    };
    let expand = |v: u16| ((v << 3) | (v >> 2)) as u8;
    [expand(r), expand(g), expand(b), alpha]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(blocks: &[Vec<u8>]) -> Vec<u8> {
        let mut bytes = (blocks.len() as u32).to_le_bytes().to_vec();
        let mut offset = 0u32;
        for block in blocks {
            bytes.extend(offset.to_le_bytes());
            offset += block.len() as u32;
        }
        bytes.extend(offset.to_le_bytes());
        for block in blocks {
            bytes.extend(block);
        }
        bytes
    }
    fn diamond(kind: u8, color: u16) -> Vec<u8> {
        let mut block = vec![kind];
        for _ in 0..288 {
            block.extend(color.to_le_bytes());
        }
        block
    }
    fn pixel(tile: &DecodedTile, x: usize, y: usize) -> &[u8] {
        &tile.rgba[(y * 24 + x) * 4..(y * 24 + x + 1) * 4]
    }

    #[test]
    fn rgb555_runs_preserve_all_color_words_and_black_alpha_policies() {
        const CHANNEL: [u8; 32] = [
            0, 8, 16, 24, 33, 41, 49, 57, 66, 74, 82, 90, 99, 107, 115, 123, 132, 140, 148, 156,
            165, 173, 181, 189, 198, 206, 214, 222, 231, 239, 247, 255,
        ];
        let source: Vec<_> = (0..=u16::MAX).flat_map(u16::to_le_bytes).collect();
        for kind in [
            ClassicTileType::DiamondRight,
            ClassicTileType::DiamondLeftFlag8,
            ClassicTileType::RleEvenHalfOpacity,
            ClassicTileType::RleOddHalfOpacity,
        ] {
            for policy in [
                TileZeroPolicy::PreserveCoverage,
                TileZeroPolicy::PreviewColorKey,
                TileZeroPolicy::RawRgb555Coverage,
            ] {
                let mut out = vec![99; source.len() * 2];
                write_rgb555_run(&mut out, &source, kind, policy);
                for (word, pixel) in out.chunks_exact(4).enumerate() {
                    let half = matches!(
                        kind,
                        ClassicTileType::RleEvenHalfOpacity | ClassicTileType::RleOddHalfOpacity
                    );
                    let expected =
                        if word == 0 && !half && policy == TileZeroPolicy::PreviewColorKey {
                            [0; 4]
                        } else {
                            [
                                CHANNEL[(word >> 10) & 31],
                                CHANNEL[(word >> 5) & 31],
                                CHANNEL[word & 31],
                                if half && policy != TileZeroPolicy::RawRgb555Coverage {
                                    128
                                } else {
                                    255
                                },
                            ]
                        };
                    assert_eq!(pixel, expected);
                }
            }
        }
    }

    #[test]
    fn diamond_orientation_rgb555_and_transparent_uncovered_pixels() {
        let set = decode_classic_til(&fixture(&[diamond(0, 0x7c00), diamond(1, 0x03e0)])).unwrap();
        let right = &set.tiles[0];
        let left = &set.tiles[1];
        assert_eq!(pixel(right, 21, 0), &[0; 4]);
        assert_eq!(pixel(right, 22, 0), &[255, 0, 0, 255]);
        assert_eq!(pixel(right, 0, 11), &[255, 0, 0, 255]);
        assert_eq!(pixel(right, 0, 23), &[0; 4]);
        assert_eq!(pixel(left, 0, 0), &[0, 255, 0, 255]);
        assert_eq!(pixel(left, 2, 0), &[0; 4]);
        assert_eq!(
            right.rgba.chunks_exact(4).filter(|p| p[3] == 255).count(),
            288
        );
        assert_eq!((left.width, left.height, left.source_offset), (24, 24, 577));
    }

    #[test]
    fn cloud_diamonds_preserve_orientation_keys_and_reference_straight_alpha() {
        // Independent C# public alpha/color methods, not the Rust helper.
        let cases: [(u16, [u8; 4]); 9] = [
            (0, [0; 4]),
            (0x0421, [0; 4]),
            (0x7fff, [0; 4]),
            (0x7fe0, [0; 4]),
            (0x4210, [231, 231, 231, 131]),
            (0x5145, [107, 49, 24, 164]),
            (0x8000, [0; 4]),
            (0x8421, [231, 231, 231, 8]),
            (0xffff, [231, 231, 231, 255]),
        ];
        for kind in [16, 17] {
            for (color, expected) in cases {
                let data = fixture(&[diamond(kind, color)]);
                let map = decode_classic_til(&data).unwrap().tiles.remove(0);
                let preview = decode_classic_til_with_policy(
                    &data,
                    TileLimits::default(),
                    TileZeroPolicy::PreviewColorKey,
                )
                .unwrap()
                .tiles
                .remove(0);
                assert_eq!(map, preview);
                assert_eq!(
                    (
                        map.x_offset,
                        map.y_offset,
                        map.content_width,
                        map.content_height
                    ),
                    (0, 0, 24, 24)
                );
                for y in 0..24 {
                    let count = if y < 12 { (y + 1) * 2 } else { (23 - y) * 2 };
                    for x in 0..24 {
                        let covered = if kind == 16 {
                            x >= 24 - count
                        } else {
                            x < count
                        };
                        assert_eq!(
                            pixel(&map, x, y),
                            if covered { &expected } else { &[0; 4] },
                            "kind={kind} color={color:04x} x={x} y={y}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn cloud_diamonds_reject_every_short_block_extra_bytes_and_neighbor_overread() {
        for kind in [16, 17] {
            let block = diamond(kind, 0x4210);
            for end in 0..577 {
                assert!(
                    decode_classic_til(&fixture(&[block[..end].to_vec()])).is_err(),
                    "kind={kind} prefix={end}"
                );
            }
            assert!(decode_classic_til(&fixture(&[block.clone()])).is_ok());
            let mut extra = block.clone();
            extra.push(0);
            assert!(decode_classic_til(&fixture(&[extra])).is_err());
            assert!(decode_classic_til(&fixture(&[block[..576].to_vec(), diamond(0, 1)])).is_err());
        }
    }

    #[test]
    fn rle_offsets_skip_segments_zero_color_and_row_stride_are_exact() {
        let block = vec![3, 3, 2, 5, 2, 2, 0, 1, 0x1f, 0, 2, 2, 0, 0, 0xff, 0x7f, 0];
        let tile = &decode_classic_til_with_policy(
            &fixture(&[block]),
            TileLimits::default(),
            TileZeroPolicy::PreviewColorKey,
        )
        .unwrap()
        .tiles[0];
        assert_eq!(
            (
                tile.x_offset,
                tile.y_offset,
                tile.content_width,
                tile.content_height
            ),
            (3, 2, 5, 2)
        );
        assert_eq!(pixel(tile, 3, 2), &[0, 0, 255, 255]);
        assert_eq!(pixel(tile, 4, 2), &[0; 4]);
        assert_eq!(pixel(tile, 5, 2), &[0; 4]);
        assert_eq!(pixel(tile, 6, 2), &[255, 255, 255, 255]);
        assert!(tile.rgba[3 * 24 * 4..4 * 24 * 4].iter().all(|b| *b == 0));
    }

    #[test]
    fn map_coverage_and_preview_color_key_are_explicit_and_distinct() {
        use crate::modern_lito::compositor::Canvas;
        for (block, covered, uncovered) in [
            (diamond(0, 0), (22, 0), (21, 0)),
            (diamond(1, 0), (1, 0), (2, 0)),
            (vec![2, 2, 1, 2, 1, 1, 2, 1, 0, 0], (3, 1), (2, 1)),
            (vec![3, 2, 1, 2, 1, 1, 2, 1, 0, 0], (3, 1), (2, 1)),
        ] {
            let data = fixture(&[block]);
            let map = decode_classic_til(&data).unwrap();
            let keyed = decode_classic_til_with_policy(
                &data,
                TileLimits::default(),
                TileZeroPolicy::PreviewColorKey,
            )
            .unwrap();
            assert_eq!(pixel(&map.tiles[0], covered.0, covered.1), &[0, 0, 0, 255]);
            assert_eq!(pixel(&keyed.tiles[0], covered.0, covered.1), &[0; 4]);
            assert_eq!(pixel(&map.tiles[0], uncovered.0, uncovered.1), &[0; 4]);
            let mut canvas = Canvas::new(24, 24).unwrap();
            canvas.clear([40, 80, 120]);
            canvas.blit_rgba(&map.tiles[0].rgba, 24, 24, 0, 0).unwrap();
            let at =
                |(x, y): (usize, usize)| &canvas.rgba()[(y * 24 + x) * 4..(y * 24 + x + 1) * 4];
            assert_eq!(at(covered), &[0, 0, 0, 255]);
            assert_eq!(at(uncovered), &[40, 80, 120, 255]);
        }
    }

    #[test]
    fn half_opacity_keeps_covered_black_distinct_from_uncovered_pixels() {
        use crate::modern_lito::compositor::Canvas;
        // Three covered pixels: red, black, green. Other pixels are not part
        // of this RLE segment, so they must not dim an existing background.
        for kind in [6, 7] {
            let block = vec![kind, 2, 1, 3, 1, 1, 0, 3, 0, 0x7c, 0, 0, 0xe0, 3];
            let tile = &decode_classic_til(&fixture(&[block])).unwrap().tiles[0];
            assert_eq!(tile.tile_type as u8, kind);
            assert_eq!(pixel(tile, 1, 1), &[0, 0, 0, 0]);
            assert_eq!(pixel(tile, 2, 1), &[255, 0, 0, 128]);
            assert_eq!(pixel(tile, 3, 1), &[0, 0, 0, 128]);
            assert_eq!(pixel(tile, 4, 1), &[0, 255, 0, 128]);
            assert_eq!(pixel(tile, 5, 1), &[0, 0, 0, 0]);
            let mut canvas = Canvas::new(24, 24).unwrap();
            canvas.clear([64, 128, 192]);
            canvas.blit_rgba(&tile.rgba, 24, 24, 0, 0).unwrap();
            let at = |x: usize| &canvas.rgba()[(24 + x) * 4..(24 + x + 1) * 4];
            assert_eq!(at(1), &[64, 128, 192, 255]);
            assert_eq!(at(2), &[160, 64, 96, 255]);
            assert_eq!(at(3), &[32, 64, 96, 255]);
            assert_eq!(at(4), &[32, 192, 96, 255]);
            assert_eq!(at(5), &[64, 128, 192, 255]);
        }
    }

    #[test]
    fn every_short_prefix_and_neighbor_overread_is_an_error() {
        for block in [
            diamond(1, 1),
            vec![2, 0, 0, 1, 1, 1, 0, 1, 0xff, 0x7f],
            vec![6, 0, 0, 1, 1, 1, 0, 1, 0, 0],
            vec![7, 0, 0, 1, 1, 1, 0, 1, 0, 0],
        ] {
            let data = fixture(&[block]);
            for end in 0..data.len() {
                assert!(
                    decode_classic_til(&data[..end]).is_err(),
                    "accepted prefix {end}"
                );
            }
            assert!(decode_classic_til(&data).is_ok());
        }
        let truncated = vec![2, 0, 0, 1, 1, 1, 0, 1];
        assert!(matches!(
            decode_classic_til(&fixture(&[truncated, diamond(0, 1)])),
            Err(TileError::Truncated { .. })
        ));
    }

    #[test]
    fn bad_table_negative_offsets_end_and_limits_fail_before_output() {
        let data = fixture(&[diamond(0, 1)]);
        for index in [4, 8] {
            let mut bad = data.clone();
            bad[index..index + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(decode_classic_til(&bad).is_err());
        }
        for limits in [
            TileLimits {
                max_input_bytes: 8,
                ..Default::default()
            },
            TileLimits {
                max_total_pixels: 575,
                ..Default::default()
            },
        ] {
            assert!(matches!(
                decode_classic_til_with_limits(&data, limits),
                Err(TileError::LimitExceeded(_))
            ));
        }
        assert_eq!(
            decode_classic_til_with_limits(
                &data,
                TileLimits {
                    max_tiles: 0,
                    ..Default::default()
                }
            ),
            Err(TileError::InvalidLimits)
        );
        assert_eq!(
            decode_classic_til(&[0; 4]),
            Err(TileError::InvalidData("zero tile count"))
        );
        assert!(matches!(
            decode_classic_til(&u32::MAX.to_le_bytes()),
            Err(TileError::LimitExceeded("tile count"))
        ));
        assert!(matches!(
            decode_classic_til_with_limits(
                &fixture(&[diamond(0, 1), diamond(1, 1)]),
                TileLimits {
                    max_tiles: 1,
                    ..Default::default()
                }
            ),
            Err(TileError::LimitExceeded("tile count"))
        ));
    }

    #[test]
    fn flag8_diamonds_preserve_source_type_geometry_color_and_coverage() {
        for kind in [8, 9] {
            for color in [0, 1, 0x0421, 0x7c00, 0x7fff, 0x8000, 0xffff] {
                for policy in [
                    TileZeroPolicy::PreserveCoverage,
                    TileZeroPolicy::PreviewColorKey,
                ] {
                    let decode = |ty| {
                        decode_classic_til_with_policy(
                            &fixture(&[diamond(ty, color)]),
                            TileLimits::default(),
                            policy,
                        )
                        .unwrap()
                        .tiles
                        .remove(0)
                    };
                    let expected = decode(kind & 1);
                    let actual = decode(kind);
                    assert_eq!(actual.tile_type as u8, kind);
                    assert_eq!(actual.rgba, expected.rgba);
                    assert_eq!(
                        (
                            actual.x_offset,
                            actual.y_offset,
                            actual.content_width,
                            actual.content_height
                        ),
                        (0, 0, 24, 24)
                    );
                }
            }
            let block = diamond(kind, 0x4210);
            for end in 0..577 {
                assert!(decode_classic_til(&fixture(&[block[..end].to_vec()])).is_err());
            }
            let mut extra = block.clone();
            extra.push(0);
            assert!(decode_classic_til(&fixture(&[extra])).is_err());
            assert!(decode_classic_til(&fixture(&[block[..576].to_vec(), diamond(0, 1)])).is_err());
            let mut remaster = vec![kind];
            remaster.resize(2305, 1);
            assert!(decode_classic_til(&fixture(&[remaster])).is_err());
        }
    }

    #[test]
    fn variant_types_remaster_and_malformed_rle_are_not_clipped_or_guessed() {
        for kind in [4, 5, 10, 11, 14, 15, 20, 21, 32, 33, 255] {
            assert_eq!(
                decode_classic_til(&fixture(&[vec![kind]])),
                Err(TileError::UnsupportedType(kind))
            );
        }
        let mut large = vec![0];
        large.resize(2305, 1);
        assert!(matches!(
            decode_classic_til(&fixture(&[large])),
            Err(TileError::UnsupportedFormat(_))
        ));
        for block in [
            vec![2, 24, 0, 1, 1],
            vec![3, 0, 23, 1, 2],
            vec![2, 0, 0, 1, 1, 1, 1, 1, 1, 0],
            vec![2, 0, 0, 1, 1, 1, 0, 2],
        ] {
            assert!(decode_classic_til(&fixture(&[block])).is_err());
        }
        assert_eq!(
            decode_classic_til(&[0x78, 0x9c, 0xed, 0xc1]),
            Err(TileError::UnsupportedFormat("compressed container"))
        );
    }

    #[test]
    fn supported_rle_type_only_payloads_are_truncated_not_unknown_types() {
        for kind in [2, 3, 6, 7, 18, 19, 22, 23, 34, 35] {
            assert!(matches!(
                decode_classic_til(&fixture(&[vec![kind]])),
                Err(TileError::Truncated { .. })
            ), "supported RLE type {kind} must require its complete header");
        }
    }

    #[test]
    fn inverse_brightness_effect_preserves_reference_color_keys_hue_and_bit15_order() {
        // Values independently checked against the existing C# 1.5.4 public
        // CalculateInvertedAlpha/GetInvertedAlphaRenderColor methods.
        let cases = [
            (0, [0, 0, 0, 0]),
            (0x0421, [0, 0, 0, 0]),
            (0x7fff, [0, 0, 0, 0]),
            (0x7fe0, [0, 0, 0, 0]),
            (0x4210, [198, 198, 198, 123]),
            (0x5145, [107, 49, 24, 90]),
            // Adjacent channel differences <= 2, not total channel range <= 2.
            (0x3a12, [198, 198, 198, 106]),
            (0x7800, [165, 0, 0, 8]),
            (0x8000, [198, 198, 198, 255]),
            (0x8421, [198, 198, 198, 246]),
        ];
        for kind in [34, 35] {
            let mut block = vec![kind, 2, 3, cases.len() as u8, 1, 1, 0, cases.len() as u8];
            for (color, _) in cases {
                block.extend(u16::to_le_bytes(color));
            }
            let input = fixture(&[block]);
            let tile = decode_classic_til(&input).unwrap().tiles.remove(0);
            assert_eq!(tile.tile_type as u8, kind);
            for (index, (color, expected)) in cases.iter().enumerate() {
                assert_eq!(pixel(&tile, index + 2, 3), expected, "{kind}/{color:04x}");
            }
            assert_eq!(pixel(&tile, 1, 3), &[0; 4]);
            assert_eq!(pixel(&tile, 2, 2), &[0; 4]);
            assert_eq!(
                decode_classic_til_with_policy(
                    &input,
                    TileLimits::default(),
                    TileZeroPolicy::PreviewColorKey
                )
                .unwrap()
                .tiles[0],
                tile
            );
        }
    }

    #[test]
    fn inverse_brightness_rle_rejects_truncation_rectangle_skips_and_extra_bytes() {
        for kind in [18, 19, 22, 23, 34, 35] {
            let block = vec![kind, 23, 23, 1, 1, 1, 0, 1, 0x10, 0x42];
            assert!(decode_classic_til(&fixture(&[block.clone()])).is_ok());
            for prefix in 0..block.len() {
                assert!(decode_classic_til(&fixture(&[block[..prefix].to_vec()])).is_err());
            }
            for (index, value) in [(1, 24), (2, 24), (3, 2), (4, 2), (6, 1), (6, 2), (7, 2)] {
                let mut invalid = block.clone();
                invalid[index] = value;
                assert!(decode_classic_til(&fixture(&[invalid])).is_err());
            }
            let mut trailing = block;
            trailing.push(0);
            assert_eq!(
                decode_classic_til(&fixture(&[trailing])),
                Err(TileError::UnsupportedFormat("trailing block data"))
            );
        }
    }

    #[test]
    fn cloud_brightness_uses_reference_light_gray_not_smoke_inverse_or_raw_rgb() {
        // C# CalculateInvertedAlpha uses max(R,G,B), not average brightness;
        // GetInvertedAlphaRenderColor uses light gray 0x739c for this flag.
        let cases = [
            (0, [0, 0, 0, 0]),
            (0x0421, [0, 0, 0, 0]),
            (0x7fff, [0, 0, 0, 0]),
            (0x7fe0, [0, 0, 0, 0]),
            (0x4210, [231, 231, 231, 131]),
            (0x5145, [107, 49, 24, 164]),
            (0x3a12, [231, 231, 231, 148]),
            (0x7800, [165, 0, 0, 246]),
            (0x8000, [0, 0, 0, 0]),
            (0x8421, [231, 231, 231, 8]),
            (0xffff, [231, 231, 231, 255]),
        ];
        // Native sub_483A80/4853E0 selects cloud before flag0x04 for22/23.
        // The same independent color/alpha values must remain full cloud values.
        for kind in [18, 19, 22, 23] {
            let mut block = vec![kind, 2, 3, cases.len() as u8, 1, 1, 0, cases.len() as u8];
            for (color, _) in cases {
                block.extend(u16::to_le_bytes(color));
            }
            let input = fixture(&[block]);
            let tile = decode_classic_til(&input).unwrap().tiles.remove(0);
            assert_eq!(tile.tile_type as u8, kind);
            for (index, (color, expected)) in cases.iter().enumerate() {
                assert_eq!(pixel(&tile, index + 2, 3), expected, "{kind}/{color:04x}");
            }
            assert_eq!(pixel(&tile, 1, 3), &[0; 4]);
            assert_eq!(pixel(&tile, 2, 2), &[0; 4]);
            assert_eq!(
                decode_classic_til_with_policy(
                    &input,
                    TileLimits::default(),
                    TileZeroPolicy::PreviewColorKey
                )
                .unwrap()
                .tiles[0],
                tile
            );
        }
    }

    /// All 18 cloud diamonds in shared Tile/1207.til, independently rendered by
    /// Lin.Helper.Core RenderBlockToBgra with transparent background/type alpha.
    /// Only BGRA -> RGBA channel order is changed; these are not native captures.
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; verifies all actual Tile/1207.til cloud diamonds"]
    fn real_shared_1207_cloud_diamonds_match_independent_csharp_goldens() {
        use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
        use sha2::{Digest, Sha256};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        let bytes = archive.read_entry("1207.til").unwrap();
        assert_eq!(bytes.len(), 148_744);
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            "5d08e1e303434e98d0cdc198cc514f6044a0183939e01cd837a0d10ffcaefc50"
        );
        let set = decode_classic_til(&bytes).unwrap();
        assert_eq!(set.tiles.len(), 256);
        for (kind, count) in [(0, 119), (1, 119), (16, 9), (17, 9)] {
            assert_eq!(
                set.tiles
                    .iter()
                    .filter(|tile| tile.tile_type as u8 == kind)
                    .count(),
                count
            );
        }
        let goldens = [
            (
                78usize,
                16u8,
                45006u32,
                "3d61b6cd4bb10f535bd7c051f09b76efe8e848b8da6328a233638f06ef23a7f4",
            ),
            (
                79usize,
                17u8,
                45583u32,
                "4ba4749864460057e99a182576dc7165beaacc36c8232b9ea921faf3dee9c7dd",
            ),
            (
                80usize,
                16u8,
                46160u32,
                "fe74b062707e0d38c7f6acec360ef41a19286db1fb6bb287e3bc6543825139f4",
            ),
            (
                81usize,
                17u8,
                46737u32,
                "20e8d3c42c19a607b246d11c4e0cf1333637a1835e0527626ed26872e4faccb6",
            ),
            (
                82usize,
                16u8,
                47314u32,
                "1057fa9981dddc6ddc64fe70b2d033fdb86b3c262389df40243c8cd2d4ad8200",
            ),
            (
                83usize,
                17u8,
                47891u32,
                "b3b71611d20f07b601448c694ba9d226479f86e7af4e6c94645d6a5b2d04e349",
            ),
            (
                84usize,
                16u8,
                48468u32,
                "a495fa00b40c1c22c6948c43fb25b6ba7d5a6d0f2f387bd5d20e17c81502f79e",
            ),
            (
                85usize,
                17u8,
                49045u32,
                "8a7d33b0beaf5eba37f9b06927d8bc0b75fa75c9a2a93d6bb5b093c6327f761f",
            ),
            (
                86usize,
                16u8,
                49622u32,
                "24a954339352a88ed4b37f382f37f5fff570cc21b25d0762c8bad970884fd235",
            ),
            (
                87usize,
                17u8,
                50199u32,
                "163bc3ab8c45540fc75f14b696d421ad14c5cac03b86be78091d07cdb93f3ddd",
            ),
            (
                88usize,
                16u8,
                50776u32,
                "30b84beb1e27a75382629b5a1e4cfd2dfb8dac8b308f81b255d7dad4eb5a4e40",
            ),
            (
                89usize,
                17u8,
                51353u32,
                "e62f5aa18d9ad97002d37a95c2bbee2fe0630a6a1e3af7ffba4e398a2fda7911",
            ),
            (
                92usize,
                16u8,
                53084u32,
                "b3b54b5d7dcecce2971c4a7776957bebebceedb2e934458a7b3b73b418167d8e",
            ),
            (
                93usize,
                17u8,
                53661u32,
                "f5133dfe0ed8623f6a086a4f0fab2df103e8f37916c42361c64199d3211aded2",
            ),
            (
                94usize,
                16u8,
                54238u32,
                "fb6a757f59fdbb52b3c9ce1397443cdc654843ab7ce001faa6bf4c2e37d52902",
            ),
            (
                95usize,
                17u8,
                54815u32,
                "be5bdaafed8f39a76cae1a5d58fdb844be631ab3a91826e59e14bc6492fa927f",
            ),
            (
                102usize,
                16u8,
                58854u32,
                "45caa073a2a646f6b04d7fad28d59edabfee27ca54c76cb6335e8c211ab12924",
            ),
            (
                103usize,
                17u8,
                59431u32,
                "0a13e5deec0162e3d30c3b0a2e811dc0cc5f1151263118ca93ee2af6da353baf",
            ),
        ];
        let mut aggregate = Vec::new();
        for (id, kind, offset, sha) in goldens {
            let tile = &set.tiles[id];
            assert_eq!(tile.tile_type as u8, kind);
            assert_eq!(tile.source_offset, offset);
            assert_eq!(
                (
                    tile.x_offset,
                    tile.y_offset,
                    tile.content_width,
                    tile.content_height
                ),
                (0, 0, 24, 24)
            );
            let digest = Sha256::digest(&tile.rgba);
            assert_eq!(format!("{digest:x}"), sha, "tile {id}");
            aggregate.extend((id as i32).to_be_bytes());
            aggregate.extend(offset.to_be_bytes());
            aggregate.push(kind);
            aggregate.extend(digest);
        }
        assert_eq!(aggregate.len(), 738);
        assert_eq!(
            format!("{:x}", Sha256::digest(&aggregate)),
            "8560b78cdf27170fe13e7211ff34d3f4432da782cfbe0820c3d4867e6283377f"
        );
    }

    /// All six cloud-alpha blocks in this actual shared member, independently
    /// rendered with existing C# Lin.Helper.Core.dll 1.5.4 RenderBlockToBgra
    /// (type alpha and transparent background enabled), then BGRA -> RGBA only.
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; verifies real Tile/2413.til cloud-alpha C# goldens"]
    fn real_shared_2413_til_matches_reference_cloud_alpha_effects() {
        use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
        use sha2::{Digest, Sha256};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        let data = archive.read_entry("2413.til").unwrap();
        assert_eq!(data.len(), 88_986);
        assert_eq!(
            format!("{:X}", Sha256::digest(&data)),
            "80E1D0990006B33EF9EAD101DD25F4C1FC5576238B5A23A56D5A7CE18461DCEE"
        );
        let set = decode_classic_til(&data).unwrap();
        assert_eq!(set.tiles.len(), 256);
        for kind in [18, 19] {
            assert_eq!(
                set.tiles
                    .iter()
                    .filter(|tile| tile.tile_type as u8 == kind)
                    .count(),
                3
            );
        }
        for (id, kind, offset, content, hash) in [
            (
                82,
                18,
                21877,
                (17, 6, 7, 15),
                "24655B552DBB24C776AC7EAF4A416600EAB65579D0FEA4ABFDA85BEFF651D237",
            ),
            (
                85,
                18,
                22246,
                (3, 5, 11, 8),
                "5C61D782825CB0BBAD68F46B703E081F6C4723B5B988632B082E6C24BA8E779B",
            ),
            (
                86,
                18,
                22285,
                (19, 1, 4, 5),
                "C10C80D53DEF05D07DE000603696FA8900E9EE377A2DDB11EACC321F578C2006",
            ),
            (
                83,
                19,
                22061,
                (0, 5, 14, 16),
                "363B9E86335A2A757BD34471AAF757311CB569D7ADFC461C994B747BAFFDB2CB",
            ),
            (
                84,
                19,
                22204,
                (18, 9, 4, 5),
                "0A536BA69EF81DDBB37E89386FA88052DF3F8582E024EAA39D066593B4F1DA74",
            ),
            (
                87,
                19,
                22323,
                (2, 1, 2, 5),
                "FEF6ECACEE1317EAC2AAAB87560AFF114ED91C502C25B4BD0412941EF5DDBCA0",
            ),
        ] {
            let tile = &set.tiles[id];
            assert_eq!(tile.tile_type as u8, kind);
            assert_eq!(tile.source_offset, offset);
            assert_eq!(
                (
                    tile.x_offset,
                    tile.y_offset,
                    tile.content_width,
                    tile.content_height
                ),
                content
            );
            assert_eq!(
                format!("{:X}", Sha256::digest(&tile.rgba)),
                hash,
                "tile {id}"
            );
        }
    }

    /// Independent oracle: existing Lin.Helper.Core.dll 1.5.4 ParseToTileBlocks
    /// and RenderBlockToBgra(type alpha=true, transparent background=true), then
    /// BGRA -> RGBA channel reorder only. Unlike types 0..3/6..7, effect types
    /// 34/35 intentionally share the reference's explicit zero-color key.
    /// This verifies a bounded real shared archive member, not native BIN parity.
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; verifies real Tile/2232.til inverse-alpha C# goldens"]
    fn real_shared_2232_til_matches_reference_inverse_alpha_effects() {
        use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
        use sha2::{Digest, Sha256};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        let data = archive.read_entry("2232.til").unwrap();
        assert_eq!(data.len(), 102_413);
        assert_eq!(
            format!("{:X}", Sha256::digest(&data)),
            "57D245C445CD02ECAE7610604D8A6AE0328A7D07B94EAB04102FC1D5598B6BD6"
        );
        let set = decode_classic_til(&data).unwrap();
        assert_eq!(set.tiles.len(), 256);
        for (kind, count) in [(2, 34), (3, 34), (34, 91), (35, 97)] {
            assert_eq!(
                set.tiles
                    .iter()
                    .filter(|tile| tile.tile_type as u8 == kind)
                    .count(),
                count
            );
        }
        for (id, kind, offset, content, hash) in [
            (
                68,
                34,
                30433,
                (17, 19, 7, 4),
                "25EF33621F08426870EDDC3A5B1DE1141B839826548DB74C33EBA5CC4394EF1E",
            ),
            (
                70,
                34,
                30514,
                (2, 8, 22, 15),
                "B4E84FA585D071F09F341E6A70A2F4C6FBE76830B58B51BC8D57D22B1A1CCA38",
            ),
            (
                72,
                34,
                31407,
                (0, 9, 24, 14),
                "EEA4CE19D07131E341F3A6E5BE7F6BD31442A86B87DAEB8DE0AA3E45A5FA98E8",
            ),
            (
                69,
                35,
                30482,
                (0, 20, 4, 3),
                "902A63A629D1D592AB3FF3FD0355FBE427071BA3845A359FC1A76D9981F676E3",
            ),
            (
                71,
                35,
                30908,
                (0, 7, 24, 16),
                "8B6E8B487AA2A05F8486BD6F395CF9BB9F0EA98ACD73696777EAEB4F4E2EC051",
            ),
            (
                73,
                35,
                31758,
                (0, 21, 4, 2),
                "02369CFF8FCB7B5DD2FA09DEAF3410C127ACF6364AD5DC454066BF33405552D1",
            ),
        ] {
            let tile = &set.tiles[id];
            assert_eq!(tile.tile_type as u8, kind);
            assert_eq!(tile.source_offset, offset);
            assert_eq!(
                (
                    tile.x_offset,
                    tile.y_offset,
                    tile.content_width,
                    tile.content_height
                ),
                content
            );
            assert_eq!(
                format!("{:X}", Sha256::digest(&tile.rgba)),
                hash,
                "tile {id}"
            );
        }
    }

    /// Independent Lin.Helper.Core 1.5.4 RenderBlock on black/white backgrounds,
    /// tools/verify-kent-tile-reference.ps1. Full 256-tile RGBA stream is pinned;
    /// the two flag-8 blocks also preserve their black pixels and raw offsets.
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; verifies real Kent Tile/47.til against C#"]
    fn real_shared_kent_forty_seven_til_matches_all_reference_pixels() {
        use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
        use sha2::{Digest, Sha256};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        let data = archive.read_entry("47.til").unwrap();
        assert_eq!(data.len(), 130_499);
        assert_eq!(
            format!("{:x}", Sha256::digest(&data)),
            "7244a023246d87b2142d1d87bf789a9fbc98535ece407d624c3ad345bc7a9f4c"
        );
        let set = decode_classic_til(&data).unwrap();
        assert_eq!(set.tiles.len(), 256);
        let mut digest = Sha256::new();
        let mut counts = std::collections::BTreeMap::new();
        for tile in &set.tiles {
            digest.update(&tile.rgba);
            *counts.entry(tile.tile_type as u8).or_insert(0usize) += 1;
        }
        assert_eq!(
            counts,
            [(0, 28), (1, 31), (2, 99), (3, 96), (8, 1), (9, 1)].into()
        );
        assert_eq!(
            format!("{:x}", digest.finalize()),
            "b9da90fc75d6aeb844f88c795798675d4aa4120ec0339af8c30f160fc49f20b3"
        );
        for (index, kind, offset, black, hash) in [
            (
                199,
                8,
                99_431,
                2,
                "b562224ee462122fca5275908ffb1fa8c93bb1e0aa3888af7e477feb917bf205",
            ),
            (
                202,
                9,
                101_308,
                1,
                "a73372c388a858a32fc68d415256ff0a2d4100d49d5ff69e89b0115dca562242",
            ),
        ] {
            let tile = &set.tiles[index];
            assert_eq!(tile.tile_type as u8, kind);
            assert_eq!(tile.source_offset, offset);
            assert_eq!(
                tile.rgba
                    .chunks_exact(4)
                    .filter(|p| *p == [0, 0, 0, 255])
                    .count(),
                black
            );
            assert_eq!(format!("{:x}", Sha256::digest(&tile.rgba)), hash);
        }
    }

    #[test]
    fn roof_raw_selection_keeps_effect_color_keys_and_binary_geometry() {
        for kind in [16u8, 17, 18, 19, 34, 35] {
            for color in [0u16, 0x0421, 0x7fff, 0x7fe0, 0x1234, 0x8000] {
                let block = if kind < 18 {
                    diamond(kind, color)
                } else {
                    let mut block = vec![kind, 2, 3, 4, 1, 1, 2, 2];
                    block.extend(color.to_le_bytes());
                    block.extend(color.to_le_bytes());
                    block
                };
                let data = fixture(&[block]);
                let raw =
                    decode_roof_til_selected(&data, TileLimits::default(), &[0].into()).unwrap();
                let raw = &raw[&0];
                assert_eq!(raw.tile_type as u8, kind);
                assert_eq!(
                    raw.rgba.chunks_exact(4).filter(|p| p[3] == 255).count(),
                    if kind < 18 { 288 } else { 2 }
                );
                let p = if kind < 18 {
                    pixel(raw, if kind == 16 { 22 } else { 0 }, 0)
                } else {
                    pixel(raw, 3, 3)
                };
                let expand = |v: u16| ((v << 3) | (v >> 2)) as u8;
                assert_eq!(
                    p,
                    [
                        expand((color >> 10) & 31),
                        expand((color >> 5) & 31),
                        expand(color & 31),
                        255
                    ]
                );
                if kind >= 18 {
                    assert_eq!(pixel(raw, 2, 3), [0; 4]);
                    assert_eq!(pixel(raw, 5, 3), [0; 4]);
                }
                if matches!(color, 0 | 0x0421 | 0x7fff | 0x7fe0) {
                    assert!(decode_classic_til(&data).unwrap().tiles[0]
                        .rgba
                        .iter()
                        .all(|b| *b == 0));
                }
                assert!(decode_roof_til_selected(
                    &data,
                    TileLimits {
                        max_total_pixels: 575,
                        ..Default::default()
                    },
                    &[0].into()
                )
                .is_err());
                assert!(
                    decode_roof_til_selected(&data, TileLimits::default(), &[1].into()).is_err()
                );
                assert!(
                    decode_roof_til_selected(&data, TileLimits::default(), &BTreeSet::new())
                        .is_err()
                );
                for end in 0..data.len() {
                    assert!(decode_roof_til_selected(
                        &data[..end],
                        TileLimits::default(),
                        &[0].into()
                    )
                    .is_err());
                }
            }
        }
    }

    #[test]
    fn sparse_selection_preserves_ids_aliases_and_only_charges_required_pixels() {
        let base = fixture(&[diamond(8, 0x7c00), diamond(9, 0)]);
        let mut data = 3u32.to_le_bytes().to_vec();
        for off in [577u32, 0, 577, 1154] {
            data.extend(off.to_le_bytes());
        }
        data.extend_from_slice(&base[16..]);
        let full = decode_classic_til(&data).unwrap();
        let limits = TileLimits {
            max_total_pixels: 2 * PIXELS,
            ..Default::default()
        };
        assert!(matches!(
            decode_classic_til_with_limits(&data, limits),
            Err(TileError::LimitExceeded("total pixels"))
        ));
        let selected = decode_classic_til_selected(&data, limits, &[0, 2].into()).unwrap();
        assert_eq!(selected.len(), 2);
        assert!(!selected.contains_key(&1));
        assert_eq!(selected[&0], full.tiles[0]);
        assert_eq!(selected[&2], full.tiles[2]);
        assert_eq!(selected[&2].source_offset, 577);
        for limits in [
            TileLimits {
                max_total_pixels: 2 * PIXELS - 1,
                ..Default::default()
            },
            TileLimits {
                max_tiles: 2,
                ..Default::default()
            },
            TileLimits {
                max_input_bytes: data.len() - 1,
                ..Default::default()
            },
        ] {
            assert!(decode_classic_til_selected(&data, limits, &[0, 2].into()).is_err());
        }
        for ids in [BTreeSet::new(), [3].into(), [usize::MAX].into()] {
            assert!(decode_classic_til_selected(&data, TileLimits::default(), &ids).is_err());
        }
        // Even an unselected table entry cannot point out of bounds.
        let mut corrupt = data.clone();
        corrupt[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode_classic_til_selected(&corrupt, limits, &[0, 2].into()).is_err());
        for end in 0..data.len() {
            assert!(decode_classic_til_selected(&data[..end], limits, &[0, 2].into()).is_err());
        }
        let unused = fixture(&[diamond(8, 0), vec![255]]);
        assert!(decode_classic_til_selected(&unused, limits, &[0].into()).is_ok());
        assert_eq!(
            decode_classic_til_selected(&unused, limits, &[1].into()),
            Err(TileError::UnsupportedType(255))
        );
        let short = fixture(&[diamond(8, 1)[..576].to_vec(), diamond(0, 1)]);
        assert!(decode_classic_til_selected(&short, limits, &[0].into()).is_err());
    }

    fn aliased_fixture_with_empty_tail() -> Vec<u8> {
        let base = fixture(&[diamond(8, 0x7c00), diamond(9, 0)]);
        let mut bytes = 5u32.to_le_bytes().to_vec();
        // Three original IDs (including an unsorted alias), then two empty slots.
        for offset in [577u32, 0, 577, 1154, 1154, 1154] {
            bytes.extend(offset.to_le_bytes());
        }
        bytes.extend_from_slice(&base[16..]);
        bytes
    }

    #[test]
    fn empty_tail_opt_in_preserves_pixels_ids_aliases_and_strict_defaults() {
        let data = aliased_fixture_with_empty_tail();
        let expected = decode_classic_til(&fixture(&[diamond(8, 0x7c00), diamond(9, 0)]))
            .unwrap();
        let selected = [0, 2].into();
        let limits = TileLimits { max_total_pixels: 2 * PIXELS, ..Default::default() };
        let actual = decode_classic_til_selected_with_empty_tail(&data, limits, &selected).unwrap();
        assert_eq!(actual.len(), 2);
        assert_eq!(actual[&0], expected.tiles[1]);
        assert_eq!(actual[&2], expected.tiles[1]);
        assert_eq!(actual[&2].source_offset, 577);
        assert!(decode_classic_til(&data).is_err());
        assert!(decode_classic_til_selected(&data, limits, &selected).is_err());
        assert!(ValidatedTileInput::new(&data, limits).is_err());
        let indexed = ValidatedTileInput::new_with_empty_tail(&data, limits).unwrap();
        assert_eq!(indexed.decode_selected(&[1].into(), false).unwrap()[&1], expected.tiles[0]);
        assert_eq!(indexed.decode_selected(&[3].into(), false),
            Err(TileError::InvalidData("selected tile slot has no block")));
    }

    #[test]
    fn empty_tail_opt_in_rejects_required_empty_slots_and_malformed_unused_offsets() {
        let data = aliased_fixture_with_empty_tail();
        let limits = TileLimits::default();
        for selected in [[3].into(), [0, 4].into()] {
            assert_eq!(decode_classic_til_selected_with_empty_tail(&data, limits, &selected),
                Err(TileError::InvalidData("selected tile slot has no block")));
        }
        for selected in [BTreeSet::new(), [5].into(), [usize::MAX].into()] {
            assert!(decode_classic_til_selected_with_empty_tail(&data, limits, &selected).is_err());
        }
        for offset in [1155u32, u32::MAX] {
            let mut corrupt = data.clone();
            // The bad slot is unselected: the complete table must still be checked.
            corrupt[20..24].copy_from_slice(&offset.to_le_bytes());
            assert!(decode_classic_til_selected_with_empty_tail(&corrupt, limits, &[0].into()).is_err());
        }
        let mut interior = data.clone();
        interior[8..12].copy_from_slice(&1154u32.to_le_bytes());
        assert_eq!(decode_classic_til_selected_with_empty_tail(&interior, limits, &[0].into()),
            Err(TileError::InvalidData("empty slots must be a terminal suffix")));
        let empty = [1u32, 0, 0].into_iter().flat_map(u32::to_le_bytes).collect::<Vec<_>>();
        assert!(decode_classic_til_selected_with_empty_tail(&empty, limits, &[0].into()).is_err());
        let mut trailing = data.clone();
        trailing.push(0);
        assert!(decode_classic_til_selected_with_empty_tail(&trailing, limits, &[0].into()).is_err());
        for end in 0..data.len() {
            assert!(decode_classic_til_selected_with_empty_tail(&data[..end], limits, &[0, 2].into()).is_err());
        }
    }

    #[test]
    fn empty_tail_opt_in_keeps_complete_table_input_and_selected_pixel_budgets() {
        let data = aliased_fixture_with_empty_tail();
        for limits in [
            TileLimits { max_tiles: 4, ..Default::default() },
            TileLimits { max_input_bytes: data.len()-1, ..Default::default() },
            TileLimits { max_total_pixels: 2 * PIXELS-1, ..Default::default() },
        ] {
            assert!(decode_classic_til_selected_with_empty_tail(&data, limits, &[0, 2].into()).is_err());
        }
        let mut malformed_payload = data.clone();
        malformed_payload[28+577] = 255;
        assert_eq!(decode_classic_til_selected_with_empty_tail(&malformed_payload, TileLimits::default(), &[0].into()),
            Err(TileError::UnsupportedType(255)));
    }

    #[test]
    fn repeated_and_unordered_offsets_keep_original_tile_ids() {
        let base = fixture(&[diamond(0, 0x7c00), diamond(1, 0x001f)]);
        let mut data = 3u32.to_le_bytes().to_vec();
        for off in [577u32, 0, 577, 1154] {
            data.extend(off.to_le_bytes());
        }
        data.extend_from_slice(&base[16..]);
        let set = decode_classic_til(&data).unwrap();
        assert_eq!(set.tiles[0].rgba, set.tiles[2].rgba);
        assert_eq!(set.tiles[1].tile_type, ClassicTileType::DiamondRight);
        assert_eq!(set.tiles[0].source_offset, 577);
    }

    #[test]
    fn validated_index_reuses_bounded_batches_without_skipping_unused_offset_checks() {
        let base = fixture(&[diamond(8, 0x7c00), diamond(9, 0)]);
        let mut data = 3u32.to_le_bytes().to_vec();
        for off in [577u32, 0, 577, 1154] {
            data.extend(off.to_le_bytes());
        }
        data.extend_from_slice(&base[16..]);
        let input = ValidatedTileInput::new(
            &data,
            TileLimits {
                max_total_pixels: PIXELS,
                ..Default::default()
            },
        )
        .unwrap();
        let first = input.decode_selected(&[0].into(), false).unwrap();
        assert_eq!(first[&0].source_offset, 577);
        for raw in [false, true] {
            let aliased = input.decode_selected(&[2].into(), raw).unwrap();
            let original = input.decode_selected(&[0].into(), raw).unwrap();
            assert_eq!(aliased[&2], original[&0]);
            assert_eq!(input.decode_selected(&[0].into(), false).unwrap(), first);
        }
        for ids in [
            BTreeSet::new(),
            [0, 2].into(),
            [3].into(),
            [usize::MAX].into(),
        ] {
            assert!(input.decode_selected(&ids, false).is_err());
        }
        assert!(input.decode_selected(&[1].into(), false).is_ok());
        for end in 0..data.len() {
            assert!(ValidatedTileInput::new(&data[..end], TileLimits::default()).is_err());
        }
        let mut corrupt = data.clone();
        corrupt[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(ValidatedTileInput::new(&corrupt, TileLimits::default()).is_err());
    }

    #[test]
    fn valid_raw_counts_are_not_misclassified_as_compression_markers() {
        for count in [0x78usize, 0x5b, 0x1b, 0x178] {
            let mut data = (count as u32).to_le_bytes().to_vec();
            for _ in 0..count {
                data.extend(0u32.to_le_bytes());
            }
            data.extend(577u32.to_le_bytes());
            data.extend(diamond(0, 0));
            assert_eq!(decode_classic_til(&data).unwrap().tiles.len(), count);
        }
    }

    /// Goldens from the existing Lin.Helper.Core.dll 1.5.4:
    /// L1Til.ParseToTileBlocks + RenderBlockToBgra with type alpha enabled and
    /// transparent background, followed only by BGRA -> RGBA channel reorder.
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; compares real Tile/0.til with C# RGBA goldens"]
    fn real_shared_zero_til_matches_csharp_rgba_and_offsets() {
        use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
        use sha2::{Digest, Sha256};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        let data = archive.read_entry("0.til").unwrap();
        assert_eq!(
            format!("{:X}", Sha256::digest(&data)),
            "1CEFF7875C65D772F13AC8B997F362E6646A65C4089DB2028863795E957D5454"
        );
        let set = decode_classic_til_with_policy(
            &data,
            TileLimits::default(),
            TileZeroPolicy::PreviewColorKey,
        )
        .unwrap();
        assert_eq!(set.tiles.len(), 256);
        let goldens = [
            (
                0,
                0,
                0,
                (0, 0, 24, 24),
                "D777B061D9A2A607E5A1550387469505D372CC317C053FE20228E3CC712FD06B",
            ),
            (
                1,
                1,
                577,
                (0, 0, 24, 24),
                "A1FA40F26E7892D6E0FAE5C0242B1C19F6500A0A1AEB1C6AC9A75A7C426E7683",
            ),
            (
                9,
                2,
                4684,
                (9, 1, 15, 17),
                "75D1E8BAA05C3313F6E457E30A5D6B864CDB6492153D8CFA0517FFE831A4A835",
            ),
            (
                8,
                3,
                4616,
                (11, 11, 10, 7),
                "EEED1EC18036911734A6B1DD95B5F48B8196303B32FF38847D19B42273AEECF2",
            ),
        ];
        for (id, kind, offset, content, checksum) in goldens {
            let tile = &set.tiles[id];
            assert_eq!((tile.width, tile.height), (24, 24));
            assert_eq!(tile.tile_type as u8, kind);
            assert_eq!(tile.source_offset, offset);
            assert_eq!(
                (
                    tile.x_offset,
                    tile.y_offset,
                    tile.content_width,
                    tile.content_height
                ),
                content
            );
            assert_eq!(
                format!("{:X}", Sha256::digest(&tile.rgba)),
                checksum,
                "tile {id}"
            );
        }
        // Separate coverage-preserving oracle from C# RenderBlock on both
        // black and white RGB555 backgrounds. A pixel retaining each distinct
        // background is uncovered; equal rendered colors mean real coverage,
        // even when the source color is zero. Do not replace the preview
        // hashes above with these: they deliberately verify different APIs.
        let map = decode_classic_til(&data).unwrap();
        for (id, offset, covered_black, hash) in [
            (
                17,
                7502,
                218,
                "3157F02462D2E60107CCFB98686F5FE95B9E0F8C5C9987D3AC8DF3D7E6183532",
            ),
            (
                212,
                118314,
                277,
                "F7F77E378498D778D7DC256882197F4A53CDE6BEEA0E7C991F5132A4E0EB0B6A",
            ),
        ] {
            let tile = &map.tiles[id];
            assert_eq!(tile.tile_type, ClassicTileType::DiamondRight);
            assert_eq!(tile.source_offset, offset);
            assert_eq!(
                tile.rgba
                    .chunks_exact(4)
                    .filter(|p| *p == [0, 0, 0, 255])
                    .count(),
                covered_black
            );
            assert_eq!(format!("{:X}", Sha256::digest(&tile.rgba)), hash);
        }
    }

    /// Oracle: existing Lin.Helper.Core.dll 1.5.4 RenderBlockDirect with
    /// applyTypeAlpha=true, rendered independently on black and white RGB555
    /// canvases. Uncovered pixels retain both backgrounds; covered colors can
    /// be recovered exactly from the two rounded 5-bit results. Expanding the
    /// recovered RGB555 and assigning alpha128 produces these RGBA goldens.
    /// This verifies the C# direct-blend contract, not an original-bin capture.
    /// The high-level RenderBlockToBgra API differs: it drops covered black.
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; verifies real Tile/50.til shadows against C# direct blend"]
    fn real_shared_fifty_til_preserves_half_opacity_black_coverage() {
        use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
        use sha2::{Digest, Sha256};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        let data = archive.read_entry("50.til").unwrap();
        assert_eq!(
            format!("{:X}", Sha256::digest(&data)),
            "14FEB360E68ECC392B4BCFECE13DC8348A135A3EC7630D828BC00D663FBFAAC4"
        );
        let set = decode_classic_til(&data).unwrap();
        assert_eq!(set.tiles.len(), 256);
        assert_eq!(
            set.tiles
                .iter()
                .filter(|tile| tile.tile_type as u8 == 6)
                .count(),
            8
        );
        assert_eq!(
            set.tiles
                .iter()
                .filter(|tile| tile.tile_type as u8 == 7)
                .count(),
            13
        );
        let goldens = [
            (
                90,
                7,
                33031,
                (21, 11, 3, 1),
                3,
                "AE8D92FACDD087BE90F0A99CA63F4D4F4F8F26CA349A6694EF1564D6B5B8B3D1",
                "CEF4F597470EC406F176FF520FED1A7BEBE7B356471A37B7CB7D3D7E515F4229",
            ),
            (
                112,
                6,
                45162,
                (0, 5, 22, 11),
                136,
                "3DED618D67B6C8C8DB4C8A233E797B0D7B56F09D6BE90075BC5FD75110840614",
                "AFE1ED4443FFE8858468D98DD2BE69A54E9C00578D07E36090C6096EBED827AA",
            ),
            (
                114,
                7,
                46053,
                (0, 16, 12, 7),
                29,
                "8060E6C0F0B18CACCC106F2DF3AF7047A1A360EEDB54250EF8E3614A85BE3253",
                "59526ECD559CAE2AB43FE21FBAB20F2F32DDB6A8862BEEBC2C31B567840A3C5C",
            ),
            (
                118,
                6,
                46821,
                (6, 8, 18, 15),
                151,
                "142D3476321C50288FEB94376797B0797A0DA840748BC2C4AF52082CD6D79BBC",
                "DD5FD211B2576E23EB9767210F03F96CBAD010C56D2E51D2C06C4910AFA1E986",
            ),
        ];
        for (id, kind, offset, content, coverage, rgba_hash, direct_white_hash) in goldens {
            let tile = &set.tiles[id];
            assert_eq!(tile.tile_type as u8, kind);
            assert_eq!(tile.source_offset, offset);
            assert_eq!(
                (
                    tile.x_offset,
                    tile.y_offset,
                    tile.content_width,
                    tile.content_height
                ),
                content
            );
            assert_eq!(
                tile.rgba.chunks_exact(4).filter(|p| p[3] == 128).count(),
                coverage
            );
            assert_eq!(
                format!("{:X}", Sha256::digest(&tile.rgba)),
                rgba_hash,
                "tile {id}"
            );
            // For this comparison only, deliberately reproduce the reference
            // direct API's 5-bit channel arithmetic, not Canvas's 8-bit blend.
            let mut direct_white = Vec::with_capacity(PIXELS * 2);
            for pixel in tile.rgba.chunks_exact(4) {
                let color: u16 = if pixel[3] == 0 {
                    0x7fff
                } else {
                    let r = (u16::from(pixel[0] >> 3) + 31) / 2;
                    let g = (u16::from(pixel[1] >> 3) + 31) / 2;
                    let b = (u16::from(pixel[2] >> 3) + 31) / 2;
                    (r << 10) | (g << 5) | b
                };
                direct_white.extend(color.to_le_bytes());
            }
            assert_eq!(
                format!("{:X}", Sha256::digest(&direct_white)),
                direct_white_hash,
                "tile {id} direct-blend white background"
            );
        }
    }
}
