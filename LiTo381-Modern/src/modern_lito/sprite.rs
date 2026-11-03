//! Bounded classic Lineage SPR decoding, independent of either native client.
//!
//! The contract follows PakViewer's `SprReader.LoadRaw`: RGB555 or a RGB555
//! palette, 24 x 24 RLE blocks, signed isometric placement, and expanded frame
//! bounds. Transparency is represented directly by RGBA alpha; no palette
//! color is sacrificed as an intermediate bitmap mask. SP2/SPX are not this
//! format. The word after the offset table is retained as metadata: observed
//! palette SPRs contain a value larger than the encoded body, not an EOF offset.

use std::{fmt, sync::Arc};

const BLOCK_SIDE: usize = 24;
const BLOCK_PIXELS: usize = BLOCK_SIDE * BLOCK_SIDE;
const TRANSPARENT: u16 = 0x8000;

#[derive(Clone, Copy, Debug)]
pub struct SpriteLimits {
    pub max_input_bytes: usize,
    pub max_frames: usize,
    pub max_blocks: usize,
    pub max_block_references: usize,
    pub max_frame_dimension: u32,
    pub max_total_pixels: usize,
}

impl Default for SpriteLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024 * 1024,
            max_frames: 255,
            max_blocks: 16_384,
            max_block_references: 65_536,
            max_frame_dimension: 4096,
            max_total_pixels: 16 * 1024 * 1024,
        }
    }
}

impl SpriteLimits {
    fn validate(self) -> Result<Self, SpriteError> {
        if self.max_input_bytes == 0
            || self.max_input_bytes > 256 * 1024 * 1024
            || self.max_frames == 0
            || self.max_frames > 255
            || self.max_blocks == 0
            || self.max_blocks > 65_536
            || self.max_block_references == 0
            || self.max_block_references > 262_144
            || self.max_frame_dimension == 0
            || self.max_frame_dimension > 8192
            || self.max_total_pixels == 0
            || self.max_total_pixels > 64 * 1024 * 1024
        {
            return Err(SpriteError::InvalidLimits);
        }
        Ok(self)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SpriteError {
    InvalidLimits,
    LimitExceeded(&'static str),
    Truncated { offset: usize, needed: usize },
    UnsupportedFormat(&'static str),
    InvalidData(&'static str),
    AllocationFailed,
}

impl fmt::Display for SpriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits => write!(f, "invalid bounded SPR decode limits"),
            Self::LimitExceeded(what) => write!(f, "SPR exceeds {what} limit"),
            Self::Truncated { offset, needed } => {
                write!(f, "truncated SPR at byte {offset}, need {needed} bytes")
            }
            Self::UnsupportedFormat(what) => write!(f, "unsupported SPR format: {what}"),
            Self::InvalidData(what) => write!(f, "invalid classic SPR: {what}"),
            Self::AllocationFailed => write!(f, "unable to allocate bounded SPR output"),
        }
    }
}
impl std::error::Error for SpriteError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteFrame {
    /// Qualified source fragments for world interleaving; inspection-only
    /// callers retain None and keep their established flattened pixels.
    pub fragments: Option<SpriteFrameFragments>,
    pub x_offset: i32,
    pub y_offset: i32,
    pub width: u32,
    pub height: u32,
    /// The first block's metadata, as in the reference reader. It does not
    /// rotate or flip pixels; block placement is specified by signed A/B.
    pub frame_type: u8,
    pub unknown1: u16,
    pub unknown2: u16,
    /// Row-major, straight RGBA8. Transparent pixels are [0, 0, 0, 0].
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedSprite {
    pub frames: Vec<SpriteFrame>,
    pub has_palette: bool,
    pub block_table_metadata: u32,
}

/// A source SPR fragment, not a rectangle cut back out of a flattened frame.
/// Native 44014B sign-extends A/B, retains the depth byte and block ID;
/// 4CD9D0 adds A/B to the actor's half-cell position and passes depth to
/// 483770. Depth is neither a blend flag nor a whole-actor Z coordinate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpriteBlockReference {
    pub half_x: i8,
    pub y: i8,
    pub depth: u8,
    pub block_id: u16,
}

/// Optional lossless input for native half-cell interleaving. References keep
/// file order (equal-depth insertion is reversed by the native cell list).
/// Each atlas slot is one 24x24 straight-RGBA block, including covered black.
/// Empty frames keep an empty reference list; they do not acquire a fake tile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteFragments {
    pub frames: Vec<Vec<SpriteBlockReference>>,
    pub blocks: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteFrameFragments {
    pub references: Vec<SpriteBlockReference>,
    pub atlas: Arc<Vec<CompactSpriteBlock>>,
}

/// Lossless RGB555 runs, or a dense block when runs would be larger. The
/// transparent sentinel is distinct from covered black. Private storage means
/// the raster never has to trust unvalidated run offsets while painting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactSpriteBlock {
    dense: bool,
    words: Vec<u16>,
}

impl CompactSpriteBlock {
    fn from_pixels(pixels: &[u16; BLOCK_PIXELS]) -> Result<Self, SpriteError> {
        let mut runs = bounded_vec(BLOCK_PIXELS)?;
        let mut index = 0;
        while index < BLOCK_PIXELS {
            if pixels[index] == TRANSPARENT {
                index += 1;
                continue;
            }
            let start = index;
            while index < BLOCK_PIXELS && pixels[index] != TRANSPARENT {
                index += 1;
            }
            // Avoid retaining a run representation larger than the dense one.
            if runs.len() + 2 + index - start >= BLOCK_PIXELS {
                let mut words = bounded_vec(BLOCK_PIXELS)?;
                words.extend_from_slice(pixels);
                return Ok(Self { dense: true, words });
            }
            runs.push(start as u16);
            runs.push((index - start) as u16);
            runs.extend_from_slice(&pixels[start..index]);
        }
        // Retained capacity, not merely length, must reflect compact storage.
        let mut words = bounded_vec(runs.len())?;
        words.extend_from_slice(&runs);
        Ok(Self {
            dense: false,
            words,
        })
    }

    pub fn retained_bytes(&self) -> usize {
        self.words.capacity() * 2
    }

    pub(crate) fn write_rgba(&self, output: &mut [u8; BLOCK_PIXELS * 4]) {
        output.fill(0);
        if self.dense {
            for (index, &color) in self.words.iter().enumerate() {
                if color != TRANSPARENT {
                    output[index * 4..index * 4 + 4].copy_from_slice(&rgb555_rgba(color));
                }
            }
        } else {
            let mut cursor = 0;
            while cursor < self.words.len() {
                let start = self.words[cursor] as usize;
                let count = self.words[cursor + 1] as usize;
                for (index, &color) in self.words[cursor + 2..cursor + 2 + count]
                    .iter()
                    .enumerate()
                {
                    let target = (start + index) * 4;
                    output[target..target + 4].copy_from_slice(&rgb555_rgba(color));
                }
                cursor += count + 2;
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn test_rgba(rgba: &[u8]) -> Self {
        assert_eq!(rgba.len(), BLOCK_PIXELS * 4);
        let mut pixels = [TRANSPARENT; BLOCK_PIXELS];
        for (index, p) in rgba.chunks_exact(4).enumerate() {
            assert!(matches!(p[3], 0 | 255));
            if p[3] == 255 {
                pixels[index] =
                    ((p[0] as u16 >> 3) << 10) | ((p[1] as u16 >> 3) << 5) | (p[2] as u16 >> 3);
            }
        }
        Self::from_pixels(&pixels).unwrap()
    }
}

impl DecodedSprite {
    /// Retained pixel storage: flat RGBA plus compact RGB555 atlas capacity.
    /// Charges each shared atlas once within this decoded animation.
    pub fn retained_rgba_bytes(&self) -> Option<usize> {
        let mut atlases = std::collections::BTreeSet::new();
        let mut bytes = 0usize;
        for frame in &self.frames {
            bytes = bytes.checked_add(frame.rgba.len())?;
            if let Some(fragments) = &frame.fragments {
                if atlases.insert(Arc::as_ptr(&fragments.atlas) as usize) {
                    for block in fragments.atlas.iter() {
                        bytes = bytes.checked_add(block.retained_bytes())?;
                    }
                }
            }
        }
        Some(bytes)
    }
}

/// Same bounded parser, with lossless fragments attached to each world frame.
/// The shared atlas is moved, not copied into each animation frame.
pub fn decode_world_spr_with_limits(
    data: &[u8],
    limits: SpriteLimits,
) -> Result<DecodedSprite, SpriteError> {
    Ok(decode_impl(data, limits, FragmentRetention::Compact)?.0)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayeredSprite {
    /// Existing inspection pixels remain source-order composed, unchanged.
    pub flattened: DecodedSprite,
    pub fragments: SpriteFragments,
}

#[derive(Clone, Copy)]
struct BlockRef {
    a: i8,
    b: i8,
    frame_type: u8,
    id: usize,
}

struct FrameDef {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    unknown1: u16,
    unknown2: u16,
    blocks: Vec<BlockRef>,
}
impl FrameDef {
    /// Observed shared attachment1582 death subframes: zero block references,
    /// (192,120,-128,-120), metadata(FFFF,0) or (0F0F,F1F1) (242-19).
    /// Native 4CD9D0 skips its block loop;
    /// C# LoadRaw leaves Pixels null with negative declared dimensions. Preserve
    /// the source slot/offsets but normalize this exact no-draw marker to 0x0.
    /// Shared11747/11749 additionally have (432,132,-368,-468,FFFF,0),
    /// subframes13..17: independently checked raw headers and C# null Pixels.
    /// A reversed rectangle with blocks or other metadata remains malformed.
    fn empty_sentinel(&self) -> bool {
        self.blocks.is_empty()
            && matches!(
                (
                    self.left,
                    self.top,
                    self.right,
                    self.bottom,
                    self.unknown1,
                    self.unknown2
                ),
                (192, 120, -128, -120, u16::MAX, 0)
                    | (192, 120, -128, -120, 0x0f0f, 0xf1f1)
                    | (432, 132, -368, -468, u16::MAX, 0)
                    | (352, 120, -288, -360, u16::MAX, 0)
                    | (544, 384, -480, -384, u16::MAX, 0)
            )
    }
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
    fn take(&mut self, n: usize) -> Result<&'a [u8], SpriteError> {
        let end = self
            .position
            .checked_add(n)
            .ok_or(SpriteError::InvalidData("offset overflow"))?;
        let result = self
            .data
            .get(self.position..end)
            .ok_or(SpriteError::Truncated {
                offset: self.base + self.position,
                needed: n,
            })?;
        self.position = end;
        Ok(result)
    }
    fn byte(&mut self) -> Result<u8, SpriteError> {
        Ok(self.take(1)?[0])
    }
    fn word(&mut self) -> Result<u16, SpriteError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn signed_word(&mut self) -> Result<i32, SpriteError> {
        Ok(self.word()? as i16 as i32)
    }
    fn dword(&mut self) -> Result<u32, SpriteError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
}

fn bounded_vec<T>(capacity: usize) -> Result<Vec<T>, SpriteError> {
    let mut out = Vec::new();
    out.try_reserve_exact(capacity)
        .map_err(|_| SpriteError::AllocationFailed)?;
    Ok(out)
}

pub fn decode_classic_spr(data: &[u8]) -> Result<DecodedSprite, SpriteError> {
    decode_classic_spr_with_limits(data, SpriteLimits::default())
}

pub fn decode_classic_spr_with_limits(
    data: &[u8],
    limits: SpriteLimits,
) -> Result<DecodedSprite, SpriteError> {
    Ok(decode_impl(data, limits, FragmentRetention::None)?.0)
}

/// Retains the original fragments as well as the legacy inspection frames.
/// `max_total_pixels` charges BOTH retained representations. Ordinary callers
/// do not retain an atlas or change their existing pixel-budget semantics.
/// This alone does not enable actor/map occlusion in WorldScene.
pub fn decode_classic_spr_layered(
    data: &[u8],
    limits: SpriteLimits,
) -> Result<LayeredSprite, SpriteError> {
    let (flattened, fragments) = decode_impl(data, limits, FragmentRetention::Rgba)?;
    Ok(LayeredSprite {
        flattened,
        fragments: fragments.ok_or(SpriteError::InvalidData("missing fragment retention"))?,
    })
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum FragmentRetention {
    None,
    Rgba,
    Compact,
}

fn decode_impl(
    data: &[u8],
    limits: SpriteLimits,
    retention: FragmentRetention,
) -> Result<(DecodedSprite, Option<SpriteFragments>), SpriteError> {
    let limits = limits.validate()?;
    if data.len() > limits.max_input_bytes {
        return Err(SpriteError::LimitExceeded("input bytes"));
    }
    if data.starts_with(b"SPX") || data.starts_with(b"SP2") || data.first() == Some(&0xfe) {
        return Err(SpriteError::UnsupportedFormat(
            "SP2/SPX or ambiguous 0xFE marker",
        ));
    }
    let mut reader = Reader::new(data, 0);
    let first = reader.byte()?;
    let has_palette = first == 255;
    let mut palette = [0u16; 256];
    let mut palette_size = 0;
    let frame_count = if has_palette {
        let encoded_size = reader.byte()?;
        palette_size = if encoded_size == 0 {
            256
        } else {
            encoded_size as usize
        };
        for color in &mut palette[..palette_size] {
            *color = reader.word()?;
        }
        reader.byte()? as usize
    } else {
        first as usize
    };
    if frame_count == 0 {
        return Err(SpriteError::InvalidData("zero frame count"));
    }
    if frame_count > limits.max_frames {
        return Err(SpriteError::LimitExceeded("frame count"));
    }
    let mut definitions = bounded_vec(frame_count)?;
    let mut reference_count = 0usize;
    for _ in 0..frame_count {
        let mut frame = FrameDef {
            left: reader.signed_word()?,
            top: reader.signed_word()?,
            right: reader.signed_word()?,
            bottom: reader.signed_word()?,
            unknown1: reader.word()?,
            unknown2: reader.word()?,
            blocks: Vec::new(),
        };
        let count = reader.word()? as usize;
        if (frame.right < frame.left || frame.bottom < frame.top)
            && !(count == 0 && frame.empty_sentinel())
        {
            return Err(SpriteError::InvalidData("reversed declared frame bounds"));
        }
        reference_count = reference_count
            .checked_add(count)
            .ok_or(SpriteError::LimitExceeded("block references"))?;
        if reference_count > limits.max_block_references {
            return Err(SpriteError::LimitExceeded("block references"));
        }
        frame.blocks = bounded_vec(count)?;
        for _ in 0..count {
            frame.blocks.push(BlockRef {
                a: reader.byte()? as i8,
                b: reader.byte()? as i8,
                frame_type: reader.byte()?,
                id: reader.word()? as usize,
            });
        }
        definitions.push(frame);
    }
    let block_count =
        usize::try_from(reader.dword()?).map_err(|_| SpriteError::LimitExceeded("block count"))?;
    if block_count > limits.max_blocks {
        return Err(SpriteError::LimitExceeded("block count"));
    }
    let retained_block_pixels = if retention == FragmentRetention::Rgba {
        block_count
            .checked_mul(BLOCK_PIXELS)
            .filter(|count| *count <= limits.max_total_pixels)
            .ok_or(SpriteError::LimitExceeded("total pixels"))?
    } else {
        0
    };
    let mut offsets = bounded_vec(block_count)?;
    for _ in 0..block_count {
        offsets.push(
            usize::try_from(reader.dword()?)
                .map_err(|_| SpriteError::InvalidData("block offset overflow"))?,
        );
    }
    let block_table_metadata = reader.dword()?;
    let data_start = reader.position;
    let body = &data[data_start..];
    if block_count == 0 && !body.is_empty() {
        return Err(SpriteError::UnsupportedFormat("data without blocks"));
    }
    for frame in &definitions {
        if frame.blocks.iter().any(|b| b.id >= block_count) {
            return Err(SpriteError::InvalidData("block reference outside table"));
        }
    }
    // Table entries may alias or be unordered. A block reader is confined by
    // the next distinct physical offset, so malformed RLE cannot eat a neighbor.
    let mut boundaries = bounded_vec(block_count + 1)?;
    boundaries.extend_from_slice(&offsets);
    boundaries.sort_unstable();
    boundaries.dedup();
    if boundaries.first().is_some_and(|p| *p != 0) {
        return Err(SpriteError::UnsupportedFormat("prefix before first block"));
    }
    if boundaries.last().is_some_and(|p| *p >= body.len()) {
        return Err(SpriteError::InvalidData("block offset outside body"));
    }
    boundaries.push(body.len());
    let mut blocks = bounded_vec(block_count)?;
    for &start in &offsets {
        let boundary = boundaries
            .binary_search(&start)
            .map_err(|_| SpriteError::InvalidData("missing block boundary"))?;
        let end = boundaries[boundary + 1];
        blocks.push(decode_block(
            &body[start..end],
            data_start + start,
            &palette[..palette_size],
            has_palette,
        )?);
    }
    let compact = if retention == FragmentRetention::Compact {
        let mut atlas = bounded_vec(blocks.len())?;
        for block in &blocks {
            atlas.push(CompactSpriteBlock::from_pixels(block)?);
        }
        Some(Arc::new(atlas))
    } else {
        None
    };
    let compact_bytes = compact
        .as_ref()
        .map_or(Some(0usize), |atlas| {
            atlas
                .iter()
                .try_fold(0usize, |sum, block| sum.checked_add(block.retained_bytes()))
        })
        .ok_or(SpriteError::LimitExceeded("fragment bytes"))?;
    let mut frames = bounded_vec(frame_count)?;
    let mut fragment_frames = if retention != FragmentRetention::None {
        Some(bounded_vec(frame_count)?)
    } else {
        None
    };
    let mut total_pixels = retained_block_pixels + compact_bytes.div_ceil(4);
    for definition in definitions {
        let (left, top, right, bottom) = frame_bounds(&definition);
        let width =
            u32::try_from(right - left + 1).map_err(|_| SpriteError::InvalidData("frame width"))?;
        let height = u32::try_from(bottom - top + 1)
            .map_err(|_| SpriteError::InvalidData("frame height"))?;
        if width > limits.max_frame_dimension || height > limits.max_frame_dimension {
            return Err(SpriteError::LimitExceeded("frame dimensions"));
        }
        let pixels = (width as usize)
            .checked_mul(height as usize)
            .ok_or(SpriteError::LimitExceeded("pixels"))?;
        total_pixels = total_pixels
            .checked_add(pixels)
            .ok_or(SpriteError::LimitExceeded("total pixels"))?;
        if total_pixels > limits.max_total_pixels {
            return Err(SpriteError::LimitExceeded("total pixels"));
        }
        let rgba_len = pixels
            .checked_mul(4)
            .ok_or(SpriteError::LimitExceeded("RGBA bytes"))?;
        let mut rgba = bounded_vec(rgba_len)?;
        rgba.resize(rgba_len, 0);
        for reference in &definition.blocks {
            let (x, y) = block_position(reference.a, reference.b);
            let x = (x - left) as usize;
            let y = (y - top) as usize;
            for (index, &color) in blocks[reference.id].iter().enumerate() {
                if color == TRANSPARENT {
                    continue;
                }
                // Bounds were computed from these same 24x24 placements;
                // checked indexing remains a final guard at the output edge.
                let pixel = (y + index / BLOCK_SIDE)
                    .checked_mul(width as usize)
                    .and_then(|row| row.checked_add(x + index % BLOCK_SIDE))
                    .and_then(|pixel| pixel.checked_mul(4))
                    .ok_or(SpriteError::InvalidData("composed pixel overflow"))?;
                let destination = rgba
                    .get_mut(pixel..pixel + 4)
                    .ok_or(SpriteError::InvalidData("composed pixel outside frame"))?;
                destination.copy_from_slice(&rgb555_rgba(color));
            }
        }
        frames.push(SpriteFrame {
            fragments: None,
            x_offset: left,
            y_offset: top,
            width,
            height,
            frame_type: definition.blocks.first().map_or(0, |b| b.frame_type),
            unknown1: definition.unknown1,
            unknown2: definition.unknown2,
            rgba,
        });
        if let Some(fragment_frames) = &mut fragment_frames {
            let mut references = bounded_vec(definition.blocks.len())?;
            references.extend(definition.blocks.into_iter().map(|reference| {
                SpriteBlockReference {
                    half_x: reference.a,
                    y: reference.b,
                    depth: reference.frame_type,
                    block_id: reference.id as u16,
                }
            }));
            fragment_frames.push(references);
        }
    }
    if let Some(atlas) = compact {
        for (frame, references) in frames.iter_mut().zip(fragment_frames.take().unwrap()) {
            frame.fragments = Some(SpriteFrameFragments {
                references,
                atlas: Arc::clone(&atlas),
            });
        }
    }
    let fragments = if let Some(frames) = fragment_frames {
        let mut atlas = bounded_vec(blocks.len())?;
        for block in blocks {
            let mut rgba = bounded_vec(BLOCK_PIXELS * 4)?;
            for color in block {
                rgba.extend_from_slice(&if color == TRANSPARENT {
                    [0; 4]
                } else {
                    rgb555_rgba(color)
                });
            }
            atlas.push(rgba);
        }
        Some(SpriteFragments {
            frames,
            blocks: atlas,
        })
    } else {
        None
    };
    Ok((
        DecodedSprite {
            frames,
            has_palette,
            block_table_metadata,
        },
        fragments,
    ))
}

fn block_position(a: i8, b: i8) -> (i32, i32) {
    let a = i32::from(a);
    let b = i32::from(b);
    // C# signed division truncates toward zero. Adjusting a negative A first
    // yields the reference's floor(A/2), including negative even values.
    let half = (if a < 0 { a - 1 } else { a }) / 2;
    (24 * (b + a - half), 12 * (b - half))
}

fn frame_bounds(frame: &FrameDef) -> (i32, i32, i32, i32) {
    if frame.empty_sentinel() {
        return (frame.left, frame.top, frame.left - 1, frame.top - 1);
    }
    if frame.blocks.is_empty() {
        return (frame.left, frame.top, frame.right, frame.bottom);
    }
    let mut bounds = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for reference in &frame.blocks {
        let (x, y) = block_position(reference.a, reference.b);
        bounds.0 = bounds.0.min(x);
        bounds.1 = bounds.1.min(y);
        bounds.2 = bounds.2.max(x + 23);
        bounds.3 = bounds.3.max(y + 23);
    }
    bounds
}

fn decode_block(
    data: &[u8],
    base: usize,
    palette: &[u16],
    has_palette: bool,
) -> Result<[u16; BLOCK_PIXELS], SpriteError> {
    let mut reader = Reader::new(data, base);
    let start_x = reader.byte()? as usize;
    let start_y = reader.byte()? as usize;
    let _encoded_width = reader.byte()?;
    let lines = reader.byte()? as usize;
    if start_x > BLOCK_SIDE || start_y > BLOCK_SIDE || start_y + lines > BLOCK_SIDE {
        return Err(SpriteError::InvalidData("RLE block exceeds 24x24"));
    }
    let mut pixels = [TRANSPARENT; BLOCK_PIXELS];
    for line in 0..lines {
        let mut x = start_x;
        let segments = reader.byte()?;
        for _ in 0..segments {
            let skip = reader.byte()?;
            if skip % 2 != 0 {
                return Err(SpriteError::UnsupportedFormat("odd RGB555 byte skip"));
            }
            x += (skip / 2) as usize;
            let count = reader.byte()? as usize;
            if x > BLOCK_SIDE || x + count > BLOCK_SIDE {
                return Err(SpriteError::InvalidData("RLE row crosses block edge"));
            }
            for _ in 0..count {
                let color = if has_palette {
                    *palette
                        .get(reader.byte()? as usize)
                        .ok_or(SpriteError::InvalidData("palette index outside palette"))?
                } else {
                    reader.word()?
                };
                pixels[(start_y + line) * BLOCK_SIDE + x] = color;
                x += 1;
            }
        }
    }
    if reader.position != data.len() {
        return Err(SpriteError::UnsupportedFormat(
            "trailing bytes in RLE block",
        ));
    }
    Ok(pixels)
}

fn rgb555_rgba(color: u16) -> [u8; 4] {
    let expand = |v: u16| ((v << 3) | (v >> 2)) as u8;
    [
        expand((color >> 10) & 31),
        expand((color >> 5) & 31),
        expand(color & 31),
        255,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_world_blocks_are_lossless_and_budgeted_with_flattened_pixels() {
        let data = fixture(
            None,
            &[(0, 0, 0), (0, 0, 1)],
            &[
                raw_block(&[0, 0x8000, 0xffff, 0x7c00]),
                raw_block(&[0x03e0]),
            ],
        );
        let layered = decode_classic_spr_layered(&data, SpriteLimits::default()).unwrap();
        let world = decode_world_spr_with_limits(&data, SpriteLimits::default()).unwrap();
        let frame = &world.frames[0];
        assert_eq!(frame.rgba, layered.flattened.frames[0].rgba);
        let source = frame.fragments.as_ref().unwrap();
        assert_eq!(source.references, layered.fragments.frames[0]);
        for (compact, expected) in source.atlas.iter().zip(&layered.fragments.blocks) {
            let mut actual = [0u8; 2304];
            compact.write_rgba(&mut actual);
            assert_eq!(actual.as_slice(), expected);
        }
        let bytes = world.retained_rgba_bytes().unwrap();
        assert_eq!(bytes, 2304 + 20);
        assert!(decode_world_spr_with_limits(
            &data,
            SpriteLimits {
                max_total_pixels: bytes / 4,
                ..SpriteLimits::default()
            }
        )
        .is_ok());
        assert!(decode_world_spr_with_limits(
            &data,
            SpriteLimits {
                max_total_pixels: bytes / 4 - 1,
                ..SpriteLimits::default()
            }
        )
        .is_err());
        // Dense/fallback and fully transparent blocks preserve every pixel,
        // including high-bit RGB555 values other than the exact 8000 sentinel.
        for pixels in [
            [0xffff; 576],
            [0x8000; 576],
            std::array::from_fn(|i| if i % 2 == 0 { 0x8000 } else { 0 }),
        ] {
            let compact = CompactSpriteBlock::from_pixels(&pixels).unwrap();
            assert!(compact.retained_bytes() <= 1152);
            let mut actual = [0; 2304];
            compact.write_rgba(&mut actual);
            for (p, color) in actual.chunks_exact(4).zip(pixels) {
                assert_eq!(
                    p,
                    if color == 0x8000 {
                        [0; 4]
                    } else {
                        rgb555_rgba(color)
                    }
                );
            }
        }
    }

    fn fixture(palette: Option<&[u16]>, refs: &[(i8, i8, u16)], blocks: &[Vec<u8>]) -> Vec<u8> {
        let mut data = Vec::new();
        if let Some(palette) = palette {
            data.extend([255, palette.len() as u8]);
            for color in palette {
                data.extend(color.to_le_bytes());
            }
        }
        data.push(1);
        for value in [0i16, 0, 23, 23] {
            data.extend(value.to_le_bytes());
        }
        data.extend([0u8; 4]);
        data.extend((refs.len() as u16).to_le_bytes());
        for &(a, b, id) in refs {
            data.extend([a as u8, b as u8, 18]);
            data.extend(id.to_le_bytes());
        }
        data.extend((blocks.len() as u32).to_le_bytes());
        let mut offset = 0u32;
        for block in blocks {
            data.extend(offset.to_le_bytes());
            offset += block.len() as u32;
        }
        data.extend(offset.to_le_bytes());
        for block in blocks {
            data.extend(block);
        }
        data
    }

    fn raw_block(colors: &[u16]) -> Vec<u8> {
        let mut data = vec![0, 0, colors.len() as u8, 1, 1, 0, colors.len() as u8];
        for color in colors {
            data.extend(color.to_le_bytes());
        }
        data
    }

    fn assert_source_order_recomposition(layered: &LayeredSprite) {
        use super::super::world_depth::NativeDrawCell;
        let origin = NativeDrawCell { half_x: 0, y: 0 };
        assert_eq!(
            layered.flattened.frames.len(),
            layered.fragments.frames.len()
        );
        for (frame, references) in layered
            .flattened
            .frames
            .iter()
            .zip(&layered.fragments.frames)
        {
            let mut actual = vec![0; frame.rgba.len()];
            for &reference in references {
                let (x, y) = origin.fragment_pixel_offset(reference).unwrap();
                let x = usize::try_from(x - i64::from(frame.x_offset)).unwrap();
                let y = usize::try_from(y - i64::from(frame.y_offset)).unwrap();
                let pixels = &layered.fragments.blocks[reference.block_id as usize];
                assert_eq!(pixels.len(), 24 * 24 * 4);
                for (index, rgba) in pixels.chunks_exact(4).enumerate() {
                    if rgba[3] != 0 {
                        let target = ((y + index / 24) * frame.width as usize + x + index % 24) * 4;
                        actual[target..target + 4].copy_from_slice(rgba);
                    }
                }
            }
            assert_eq!(actual, frame.rgba);
        }
    }

    #[test]
    fn layered_retention_keeps_occluded_colors_source_depths_ids_and_opaque_black() {
        let mut data = fixture(
            None,
            &[(0, 0, 0), (0, 0, 1), (-1, 1, 0)],
            &[
                raw_block(&[0x7c00, 0, 0x8000]),
                raw_block(&[0x03e0, 0x8000, 0x001f]),
            ],
        );
        for (index, depth) in [255, 0, 18].into_iter().enumerate() {
            data[17 + 5 * index] = depth;
        }
        let decoded = decode_classic_spr_layered(&data, SpriteLimits::default()).unwrap();
        assert_eq!(decoded.flattened, decode_classic_spr(&data).unwrap());
        assert_eq!(
            decoded.fragments.frames[0],
            [
                SpriteBlockReference {
                    half_x: 0,
                    y: 0,
                    depth: 255,
                    block_id: 0
                },
                SpriteBlockReference {
                    half_x: 0,
                    y: 0,
                    depth: 0,
                    block_id: 1
                },
                SpriteBlockReference {
                    half_x: -1,
                    y: 1,
                    depth: 18,
                    block_id: 0
                },
            ]
        );
        assert_eq!(
            &decoded.fragments.blocks[0][..12],
            &[255, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 0]
        );
        assert_eq!(
            &decoded.fragments.blocks[1][..12],
            &[0, 255, 0, 255, 0, 0, 0, 0, 0, 0, 255, 255]
        );
        // Source-order inspection loses the red at the first overlapping
        // position, but interleaving must still be able to recover it.
        assert_eq!(&decoded.flattened.frames[0].rgba[..4], &[0, 255, 0, 255]);
        assert_source_order_recomposition(&decoded);
    }

    #[test]
    fn layered_native_depth_can_put_only_one_actor_fragment_behind_a_wall() {
        use super::super::world_depth::NativeDrawCell;
        let mut data = fixture(
            None,
            &[(0, 0, 0), (1, 0, 1)],
            &[raw_block(&[0x7c00]), raw_block(&[0x03e0])],
        );
        data[17] = 10;
        data[22] = 30;
        let decoded = decode_classic_spr_layered(&data, SpriteLimits::default()).unwrap();
        let origin = NativeDrawCell { half_x: 0, y: 0 };
        let references = &decoded.fragments.frames[0];
        let mut draws = Vec::new();
        let wall = [0, 0, 255, 255];
        for (index, &reference) in references.iter().enumerate() {
            let cell = origin.fragment(reference).unwrap();
            draws.push((cell.overlay_key(20, index as u64), index, wall.as_slice()));
            draws.push((
                cell.overlay_key(reference.depth, index as u64 + 2),
                index,
                &decoded.fragments.blocks[reference.block_id as usize][..4],
            ));
        }
        draws.sort_by_key(|draw| draw.0);
        let mut actual = [[0u8; 4]; 2];
        for (_, index, rgba) in draws {
            actual[index].copy_from_slice(rgba);
        }
        assert_eq!(actual, [[0, 0, 255, 255], [0, 255, 0, 255]]);
        assert_source_order_recomposition(&decoded);
    }

    #[test]
    fn layered_palette_aliases_preserve_logical_indices_and_empty_frames() {
        let mut data = fixture(
            Some(&[0, 0x03e0]),
            &[(0, 0, 0), (1, 0, 1)],
            &[vec![0, 0, 2, 1, 1, 0, 2, 0, 1], vec![]],
        );
        // Two logical block IDs deliberately alias the same physical payload.
        let table = 6 + 15 + 2 * 5;
        data[table + 8..table + 12].copy_from_slice(&0u32.to_le_bytes());
        let decoded = decode_classic_spr_layered(&data, SpriteLimits::default()).unwrap();
        assert_eq!(decoded.fragments.blocks.len(), 2);
        assert_eq!(decoded.fragments.blocks[0], decoded.fragments.blocks[1]);
        assert_eq!(decoded.fragments.frames[0][1].block_id, 1);
        assert_eq!(
            &decoded.fragments.blocks[0][..8],
            &[0, 0, 0, 255, 0, 255, 0, 255]
        );
        assert_source_order_recomposition(&decoded);
        let mut empty = vec![1];
        empty.extend(empty_marker());
        empty.extend([0; 8]);
        let decoded = decode_classic_spr_layered(&empty, SpriteLimits::default()).unwrap();
        assert!(decoded.fragments.blocks.is_empty());
        assert_eq!(decoded.fragments.frames, vec![Vec::new()]);
        assert_eq!(
            (
                decoded.flattened.frames[0].width,
                decoded.flattened.frames[0].height
            ),
            (0, 0)
        );
        assert_source_order_recomposition(&decoded);
    }

    #[test]
    fn layered_budget_charges_atlas_once_per_slot_plus_every_retained_frame() {
        let one = fixture(None, &[(0, 0, 0)], &[raw_block(&[0])]);
        let legacy_limit = SpriteLimits {
            max_total_pixels: 576,
            ..Default::default()
        };
        assert!(decode_classic_spr_with_limits(&one, legacy_limit).is_ok());
        for pixels in [575, 576, 1151] {
            assert_eq!(
                decode_classic_spr_layered(
                    &one,
                    SpriteLimits {
                        max_total_pixels: pixels,
                        ..Default::default()
                    }
                ),
                Err(SpriteError::LimitExceeded("total pixels"))
            );
        }
        assert!(decode_classic_spr_layered(
            &one,
            SpriteLimits {
                max_total_pixels: 1152,
                ..Default::default()
            }
        )
        .is_ok());
        let mut two_frames = vec![2];
        two_frames.extend_from_slice(&one[1..20]);
        two_frames.extend_from_slice(&one[1..]);
        assert_eq!(
            decode_classic_spr_layered(
                &two_frames,
                SpriteLimits {
                    max_total_pixels: 1727,
                    ..Default::default()
                }
            ),
            Err(SpriteError::LimitExceeded("total pixels"))
        );
        let decoded = decode_classic_spr_layered(
            &two_frames,
            SpriteLimits {
                max_total_pixels: 1728,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(decoded.fragments.blocks.len(), 1);
        assert_eq!(decoded.fragments.frames.len(), 2);
        assert_source_order_recomposition(&decoded);
    }

    #[test]
    fn layered_rejects_invalid_refs_palette_rle_markers_and_bounds() {
        let samples = [
            fixture(None, &[(0, 0, 1)], &[raw_block(&[1])]),
            fixture(Some(&[0]), &[(0, 0, 0)], &[vec![0, 0, 1, 1, 1, 0, 1, 1]]),
            fixture(None, &[(0, 0, 0)], &[vec![0, 0, 1, 1, 1, 1, 1, 0, 0]]),
            fixture(None, &[(0, 0, 0)], &[vec![0, 24, 0, 1]]),
            fixture(
                None,
                &[(0, 0, 0)],
                &[vec![0, 0, 1, 1, 1, 0, 1], raw_block(&[1])],
            ),
            b"SPX1".to_vec(),
        ];
        for data in samples {
            assert!(decode_classic_spr(&data).is_err());
            assert!(decode_classic_spr_layered(&data, SpriteLimits::default()).is_err());
        }
    }

    #[test]
    fn direct_rgb555_pixels_transparency_and_expanded_negative_anchor() {
        let block = raw_block(&[0x7c00, 0x03e0, 0x001f, 0x8000, 0]);
        let sprite = decode_classic_spr(&fixture(None, &[(-1, 0, 0)], &[block])).unwrap();
        let frame = &sprite.frames[0];
        assert_eq!(
            (frame.x_offset, frame.y_offset, frame.width, frame.height),
            (0, 12, 24, 24)
        );
        assert_eq!(
            &frame.rgba[..20],
            &[255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 0, 0, 0, 0, 0, 0, 0, 255]
        );
        assert!(frame.rgba[20..].iter().all(|v| *v == 0));
        assert_eq!(block_position(-2, -1), (-48, 0));
        assert!(!sprite.has_palette);
    }

    fn empty_marker() -> Vec<u8> {
        [192i16, 120, -128, -120, -1, 0, 0]
            .into_iter()
            .flat_map(i16::to_le_bytes)
            .collect()
    }
    #[test]
    fn qualified_zero_block_sentinel_keeps_frame_slot_without_pixels_or_fake_rectangle() {
        let one = fixture(None, &[(0, 0, 0)], &[raw_block(&[0x7c00])]);
        let mut mixed = vec![3];
        mixed.extend_from_slice(&one[1..20]);
        mixed.extend(empty_marker());
        mixed.extend_from_slice(&one[1..]);
        let decoded = decode_classic_spr(&mixed).unwrap();
        assert_eq!(decoded.frames.len(), 3);
        assert_eq!(decoded.frames[0], decoded.frames[2]);
        let empty = &decoded.frames[1];
        assert_eq!(
            (
                empty.x_offset,
                empty.y_offset,
                empty.width,
                empty.height,
                empty.unknown1,
                empty.unknown2
            ),
            (192, 120, 0, 0, u16::MAX, 0)
        );
        assert_eq!(empty.frame_type, 0);
        assert!(empty.rgba.is_empty());
        let mut canvas = super::super::compositor::Canvas::new(8, 8).unwrap();
        canvas.clear([30, 60, 90]);
        let before = canvas.rgba().to_vec();
        canvas
            .blit_screen_rgb555(
                &empty.rgba,
                empty.width,
                empty.height,
                empty.x_offset,
                empty.y_offset,
            )
            .unwrap();
        assert_eq!(canvas.rgba(), before);
        let mut all_empty = vec![1];
        all_empty.extend(empty_marker());
        all_empty.extend([0; 8]);
        assert!(decode_classic_spr(&all_empty).unwrap().frames[0]
            .rgba
            .is_empty());
        for end in 0..all_empty.len() {
            assert!(decode_classic_spr(&all_empty[..end]).is_err());
        }
        all_empty[9..11].copy_from_slice(&0x0f0fu16.to_le_bytes());
        all_empty[11..13].copy_from_slice(&0xf1f1u16.to_le_bytes());
        let second = decode_classic_spr(&all_empty).unwrap();
        assert_eq!(
            (second.frames[0].unknown1, second.frames[0].unknown2),
            (0x0f0f, 0xf1f1)
        );
        assert!(second.frames[0].rgba.is_empty());
    }
    #[test]
    fn empty_sentinel_does_not_relax_corrupt_bounds_counts_metadata_or_block_body() {
        for offset in [0, 2, 4, 6, 8, 10, 12] {
            let mut data = vec![1];
            let mut marker = empty_marker();
            marker[offset] ^= 1;
            data.extend(marker);
            data.extend([0; 8]);
            assert!(decode_classic_spr(&data).is_err(), "marker field {offset}");
        }
        let mut data = vec![1];
        data.extend(empty_marker());
        data.extend([0; 8]);
        data.push(9);
        assert!(matches!(
            decode_classic_spr(&data),
            Err(SpriteError::UnsupportedFormat("data without blocks"))
        ));
        // Even an empty frame does not suppress corrupt unreferenced RLE data.
        let mut data = vec![1];
        data.extend(empty_marker());
        data.extend(1u32.to_le_bytes());
        data.extend([0; 8]);
        data.extend([0, 24, 0, 1]);
        assert!(matches!(
            decode_classic_spr(&data),
            Err(SpriteError::InvalidData("RLE block exceeds 24x24"))
        ));
    }

    #[test]
    fn shared_disintegrate_empty_marker_is_exact_and_preserves_layer_slots() {
        let marker: Vec<_> = [432i16, 132, -368, -468, -1, 0, 0]
            .into_iter()
            .flat_map(i16::to_le_bytes)
            .collect();
        let mut data = vec![1];
        data.extend(&marker);
        data.extend([0; 8]);
        let flat = decode_classic_spr(&data).unwrap();
        let layered = decode_classic_spr_layered(&data, SpriteLimits::default()).unwrap();
        let world = decode_world_spr_with_limits(&data, SpriteLimits::default()).unwrap();
        assert_eq!(flat, layered.flattened);
        for decoded in [&flat, &world] {
            let f = &decoded.frames[0];
            assert_eq!(
                (f.x_offset, f.y_offset, f.width, f.height),
                (432, 132, 0, 0)
            );
            assert_eq!((f.unknown1, f.unknown2), (u16::MAX, 0));
            assert!(f.rgba.is_empty());
        }
        assert!(layered.fragments.frames[0].is_empty());
        assert!(world.frames[0]
            .fragments
            .as_ref()
            .unwrap()
            .references
            .is_empty());
        for offset in 0..marker.len() {
            let mut bad = data.clone();
            bad[1 + offset] ^= 1;
            assert!(decode_classic_spr(&bad).is_err(), "field byte {offset}");
        }
        for end in 0..data.len() {
            assert!(decode_classic_spr(&data[..end]).is_err());
        }
        // Empty-frame support must still validate an unreferenced corrupt RLE block.
        let mut bad = vec![1];
        bad.extend(marker);
        bad.extend(1u32.to_le_bytes());
        bad.extend([0; 8]);
        bad.extend([0, 24, 0, 1]);
        assert!(decode_classic_spr(&bad).is_err());
    }

    #[test]
    fn palette_rgba_matches_explicit_pixels_and_does_not_erase_reserved_colors() {
        let palette = [31744, 768, 31, 32736, 1023, 31775, 32767, 0x8000];
        let block = vec![0, 0, 8, 1, 1, 0, 8, 0, 1, 2, 3, 4, 5, 6, 7];
        let sprite = decode_classic_spr(&fixture(Some(&palette), &[(0, 0, 0)], &[block])).unwrap();
        let expected = [
            255, 0, 0, 255, 0, 198, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255, 0, 255, 255, 255,
            255, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0,
        ];
        assert_eq!(&sprite.frames[0].rgba[..32], &expected);
        assert!(sprite.has_palette);
    }

    #[test]
    fn every_short_prefix_fails_without_panicking() {
        for palette in [false, true] {
            let data = if palette {
                fixture(
                    Some(&[0x7c00]),
                    &[(0, 0, 0)],
                    &[vec![0, 0, 1, 1, 1, 0, 1, 0]],
                )
            } else {
                fixture(None, &[(0, 0, 0)], &[raw_block(&[0x7c00])])
            };
            for end in 0..data.len() {
                assert!(
                    decode_classic_spr(&data[..end]).is_err(),
                    "accepted prefix {end}"
                );
                assert!(
                    decode_classic_spr_layered(&data[..end], SpriteLimits::default()).is_err(),
                    "layered decoder accepted prefix {end}"
                );
            }
            assert!(decode_classic_spr(&data).is_ok());
        }
    }

    #[test]
    fn bad_palette_id_block_id_row_and_neighbor_reads_fail() {
        assert!(matches!(
            decode_classic_spr(&fixture(
                Some(&[0]),
                &[(0, 0, 0)],
                &[vec![0, 0, 1, 1, 1, 0, 1, 1]]
            )),
            Err(SpriteError::InvalidData("palette index outside palette"))
        ));
        assert!(matches!(
            decode_classic_spr(&fixture(None, &[(0, 0, 1)], &[raw_block(&[1])])),
            Err(SpriteError::InvalidData("block reference outside table"))
        ));
        for block in [
            vec![0, 24, 0, 1],
            vec![24, 0, 1, 1, 1, 0, 1, 0, 0],
            vec![0, 0, 25, 1, 1, 0, 25],
        ] {
            assert!(decode_classic_spr(&fixture(None, &[(0, 0, 0)], &[block])).is_err());
        }
        let short = vec![0, 0, 1, 1, 1, 0, 1];
        assert!(matches!(
            decode_classic_spr(&fixture(None, &[(0, 0, 0)], &[short, raw_block(&[1])])),
            Err(SpriteError::Truncated { .. })
        ));
    }

    #[test]
    fn offsets_and_dimensions_are_bounded_before_allocations() {
        let data = fixture(None, &[(0, 0, 0)], &[raw_block(&[1])]);
        for limits in [
            SpriteLimits {
                max_input_bytes: 1,
                ..Default::default()
            },
            SpriteLimits {
                max_frame_dimension: 23,
                ..Default::default()
            },
            SpriteLimits {
                max_total_pixels: 575,
                ..Default::default()
            },
        ] {
            assert!(matches!(
                decode_classic_spr_with_limits(&data, limits),
                Err(SpriteError::LimitExceeded(_))
            ));
        }
        let mut bad = data.clone();
        bad[24..28].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode_classic_spr(&bad).is_err());
        let mut huge_table = data.clone();
        huge_table[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            decode_classic_spr(&huge_table),
            Err(SpriteError::LimitExceeded("block count"))
        ));
        let two = fixture(
            None,
            &[(i8::MIN, i8::MIN, 0), (i8::MAX, i8::MAX, 0)],
            &[raw_block(&[1])],
        );
        assert!(matches!(
            decode_classic_spr(&two),
            Err(SpriteError::LimitExceeded("frame dimensions"))
        ));
    }

    #[test]
    fn frame_reference_and_aggregate_pixel_limits_are_independent() {
        let one = fixture(None, &[(0, 0, 0)], &[raw_block(&[1])]);
        let mut two_frames = vec![2];
        two_frames.extend_from_slice(&one[1..20]);
        two_frames.extend_from_slice(&one[1..]);
        assert_eq!(decode_classic_spr(&two_frames).unwrap().frames.len(), 2);
        let cases = [
            (
                SpriteLimits {
                    max_frames: 1,
                    ..Default::default()
                },
                "frame count",
            ),
            (
                SpriteLimits {
                    max_block_references: 1,
                    ..Default::default()
                },
                "block references",
            ),
            (
                SpriteLimits {
                    max_total_pixels: 1151,
                    ..Default::default()
                },
                "total pixels",
            ),
        ];
        for (limits, expected) in cases {
            assert_eq!(
                decode_classic_spr_with_limits(&two_frames, limits),
                Err(SpriteError::LimitExceeded(expected))
            );
        }
        let invalid = SpriteLimits {
            max_total_pixels: usize::MAX,
            ..Default::default()
        };
        assert_eq!(
            decode_classic_spr_with_limits(&one, invalid),
            Err(SpriteError::InvalidLimits)
        );
    }

    #[test]
    fn unsupported_markers_odd_skips_and_trailing_blocks_are_explicit() {
        for data in [b"SPX1".as_slice(), b"SP2".as_slice(), &[0xfe, 0, 0]] {
            assert!(matches!(
                decode_classic_spr(data),
                Err(SpriteError::UnsupportedFormat(_))
            ));
        }
        for block in [vec![0, 0, 1, 1, 1, 1, 1, 0, 0], vec![0, 0, 0, 0, 99]] {
            assert!(matches!(
                decode_classic_spr(&fixture(None, &[(0, 0, 0)], &[block])),
                Err(SpriteError::UnsupportedFormat(_))
            ));
        }
        assert!(matches!(
            decode_classic_spr(&[0]),
            Err(SpriteError::InvalidData("zero frame count"))
        ));
    }

    #[test]
    fn transparent_overlap_preserves_previous_block_and_256_entry_palette() {
        let mut palette = [0u16; 256];
        palette[255] = 0x03e0;
        let data = fixture(
            Some(&palette),
            &[(0, 0, 0), (0, 0, 1)],
            &[vec![0, 0, 1, 1, 1, 0, 1, 255], vec![0, 0, 0, 0]],
        );
        let frame = &decode_classic_spr(&data).unwrap().frames[0];
        assert_eq!(&frame.rgba[..4], &[0, 255, 0, 255]);
    }

    /// The checksums were independently generated by the repository's existing
    /// Lin.Helper.Core.Sprite.SprReader.LoadRaw (net10.0, version 1.5.4).
    /// No copyrighted sample is copied into the source tree. The explicitly
    /// selected shared archive entries are read in place, without mutation.
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; compares real archives to C# RGBA goldens"]
    fn real_shared_spr_rgba_matches_csharp_reference() {
        use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
        use sha2::{Digest, Sha256};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let samples = [
            (
                SharedArchiveKind::Sprite,
                "0-1.spr",
                "980663D48CC620879631B66F4F271FAF81A1E21AF001D8804BCFE56DE2DB9CE2",
                "4BD39D0F229E7BFA78C174B080E475E39D1C330C0A0C552CCB53B0BF997EF1DE",
                vec![
                    (0, -48, 48, 72),
                    (-24, -48, 72, 96),
                    (0, -48, 48, 96),
                    (0, -48, 48, 96),
                ],
                vec![10, 14, 11, 11],
                46,
                "82AB396F8F27C7814171B16D0A10FED9359DD4CBACE42472E64B31815B0D1077",
            ),
            (
                SharedArchiveKind::SpriteShard(0),
                "10032-19.spr",
                "79EB0975CB46FB3B50039916622C7729E61DA103C776A2505EDC312A3D7FD084",
                "0B148654CFBBAE4E800F86D93401D7F8AF8F65E5D8704577899380A1F7C73817",
                vec![(0, -48, 48, 72), (0, -48, 48, 72)],
                vec![10, 10],
                20,
                "2E896ED599CA1FB2E768D0CB0E8843F9F84E027132152B1DA00E7470F3FC0AB8",
            ),
            (
                SharedArchiveKind::SpriteShard(1),
                "10034-27.spr",
                "4BCF7E1439DC2BC5C5A64805BEBAE0A47AB49D39690927DE1AAD379DBA42E45E",
                "6E67AEDCA6EC6C5A1C69E897FAAC97773134C4676F772F0933200F229D096A7F",
                vec![(0, -48, 48, 72), (0, -48, 48, 72)],
                vec![9, 10],
                19,
                "CE6CE461222A3DA819926BD475AAC218F516CB1FCFD7461E5EBEE9565912B1C2",
            ),
        ];
        for (
            kind,
            name,
            input_hash,
            rgba_hash,
            bounds,
            reference_counts,
            block_count,
            reference_hash,
        ) in samples
        {
            let mut archive = SharedArchive::open(&root, kind).unwrap();
            let data = archive.read_entry(name).unwrap();
            assert_eq!(
                format!("{:X}", Sha256::digest(&data)),
                input_hash,
                "changed sample {name}"
            );
            let sprite = decode_classic_spr(&data).unwrap();
            let actual: Vec<_> = sprite
                .frames
                .iter()
                .map(|f| (f.x_offset, f.y_offset, f.width, f.height))
                .collect();
            assert_eq!(actual, bounds, "frame bounds {name}");
            let mut checksum = Sha256::new();
            for frame in &sprite.frames {
                checksum.update(&frame.rgba);
            }
            assert_eq!(
                format!("{:X}", checksum.finalize()),
                rgba_hash,
                "RGBA pixels {name}"
            );
            let layered = decode_classic_spr_layered(&data, SpriteLimits::default()).unwrap();
            assert_eq!(
                layered.flattened, sprite,
                "legacy pixels/metadata changed: {name}"
            );
            assert_source_order_recomposition(&layered);
            let world = decode_world_spr_with_limits(&data, SpriteLimits::default()).unwrap();
            for (index, frame) in world.frames.iter().enumerate() {
                assert_eq!(frame.rgba, sprite.frames[index].rgba);
                let fragments = frame.fragments.as_ref().unwrap();
                assert_eq!(fragments.references, layered.fragments.frames[index]);
                if index > 0 {
                    assert!(Arc::ptr_eq(
                        &fragments.atlas,
                        &world.frames[0].fragments.as_ref().unwrap().atlas
                    ));
                }
                for (block, expected) in fragments.atlas.iter().zip(&layered.fragments.blocks) {
                    let mut actual = [0; 2304];
                    block.write_rgba(&mut actual);
                    assert_eq!(actual.as_slice(), expected, "compact fragment {name}");
                }
            }
            assert_eq!(layered.fragments.blocks.len(), block_count);
            assert_eq!(
                layered
                    .fragments
                    .frames
                    .iter()
                    .map(Vec::len)
                    .collect::<Vec<_>>(),
                reference_counts
            );
            // Independently read raw headers through the existing C# archive
            // extractor + PowerShell BinaryConverter (see read-only tool).
            // Pin every signed coordinate, depth, ID and source order, not
            // just the first block's old frame_type field.
            let mut checksum = Sha256::new();
            let mut depths = std::collections::BTreeSet::new();
            for references in &layered.fragments.frames {
                checksum.update((references.len() as u16).to_le_bytes());
                for reference in references {
                    checksum.update([reference.half_x as u8, reference.y as u8, reference.depth]);
                    checksum.update(reference.block_id.to_le_bytes());
                    depths.insert(reference.depth);
                }
            }
            checksum.update((layered.fragments.blocks.len() as u32).to_le_bytes());
            assert_eq!(
                format!("{:X}", checksum.finalize()),
                reference_hash,
                "fragment records {name}"
            );
            assert_eq!(depths.into_iter().collect::<Vec<_>>(), [18, 22, 26, 30, 34]);
        }
    }
    #[test]
    fn reviewed544_marker_preserves_slots_and_rejects_corruption() {
        let marker = || [544i16,384,-480,-384,-1,0,0].into_iter().flat_map(i16::to_le_bytes).collect::<Vec<_>>();
        let mut empty=vec![1];empty.extend(marker());empty.extend([0;8]);
        let decoded=decode_classic_spr(&empty).unwrap();
        let frame=&decoded.frames[0];
        assert_eq!((frame.x_offset,frame.y_offset,frame.width,frame.height,frame.unknown1,frame.unknown2),(544,384,0,0,u16::MAX,0));
        assert!(frame.rgba.is_empty());assert_eq!(frame.frame_type,0);
        let layered=decode_classic_spr_layered(&empty,SpriteLimits::default()).unwrap();
        assert_eq!(layered.flattened,decoded);assert!(layered.fragments.frames[0].is_empty());
        let one=fixture(None,&[(0,0,0)],&[raw_block(&[0x7c00])]);
        let mut mixed=vec![3];mixed.extend_from_slice(&one[1..20]);mixed.extend(marker());mixed.extend_from_slice(&one[1..]);
        let frames=decode_classic_spr(&mixed).unwrap().frames;
        assert_eq!(frames.len(),3);assert_eq!(frames[0],frames[2]);assert!(frames[1].rgba.is_empty());
        let mut canvas=super::super::compositor::Canvas::new(8,8).unwrap();canvas.clear([30,60,90]);let pixels=canvas.rgba().to_vec();
        canvas.blit_screen_rgb555(&frame.rgba,frame.width,frame.height,frame.x_offset,frame.y_offset).unwrap();assert_eq!(canvas.rgba(),pixels);
        for offset in [0,2,4,6,8,10,12] {let mut bad=vec![1];let mut m=marker();m[offset]^=1;bad.extend(m);bad.extend([0;8]);assert!(decode_classic_spr(&bad).is_err(),"marker field {offset}");}
        for end in 0..empty.len(){assert!(decode_classic_spr(&empty[..end]).is_err());}
        let mut bad=empty.clone();bad.push(9);assert!(matches!(decode_classic_spr(&bad),Err(SpriteError::UnsupportedFormat("data without blocks"))));
        let mut bad=vec![1];bad.extend(marker());bad.extend(1u32.to_le_bytes());bad.extend([0;8]);bad.extend([0,24,0,1]);
        assert!(matches!(decode_classic_spr(&bad),Err(SpriteError::InvalidData("RLE block exceeds 24x24"))));
    }

}
