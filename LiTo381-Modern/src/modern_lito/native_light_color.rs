//! Original light remap/cache and RGB555/RGB565 channel-table postprocessing.
//! Actor selection and world policy belong to the scene lighting owner.
use super::native_light_mask::Mask;
use anyhow::{ensure, Context, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format { Rgb555, Rgb565 }

pub struct Channels { format: Format, values: Box<[[u16; 1024]; 3]> }
impl Channels {
    pub fn new(format: Format) -> Self {
        let mut values = Box::new([[0; 1024]; 3]);
        let shifts = if format == Format::Rgb555 { [0, 5, 10] } else { [0, 6, 11] };
        for (channel, shift) in values.iter_mut().zip(shifts) {
            for color in 0..32 { for level in 0..32 {
                channel[color * 32 + level] = ((color * level / 31) as u16) << shift;
            }}
        }
        Self { format, values }
    }
    /// Retained original table bytes and diagnostic table inputs share the
    /// same explicit channel masks; malformed entries never reach a surface.
    pub fn from_bytes(format: Format, bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() == 6144, "light channel table byte count");
        let mut result = Self::new(format);
        let masks = if format == Format::Rgb555 { [31, 31 << 5, 31 << 10] }
                    else { [31, 31 << 6, 31 << 11] };
        for (channel, mask) in masks.into_iter().enumerate() {
            for i in 0..1024 {
                let at = (channel * 1024 + i) * 2;
                let value = u16::from_le_bytes(bytes[at..at + 2].try_into()?);
                ensure!(value & !mask == 0, "light channel bits outside native channel");
                result.values[channel][i] = value;
            }
        }
        Ok(result)
    }
    fn pixel(&self, color: u16, level: u8) -> u16 {
        if level == 31 { return color; }
        let (green, red) = if self.format == Format::Rgb555 { (5, 10) } else { (6, 11) };
        let at = |value: u16| usize::from(value & 31) * 32 + usize::from(level);
        self.values[0][at(color)] | self.values[1][at(color >> green)]
            | self.values[2][at(color >> red)]
    }
    /// Applies to a viewport within a pitched 16-bit surface. One mask byte
    /// supplies each 2x2 pixel block. Gates are viewport-relative byte planes.
    pub fn apply_words(&self, pixels: &mut [u16], pitch: usize, origin: [usize; 2],
        size: [usize; 2], mask: &Mask, row: &[u8; 32], gate: Option<&[u8]>) -> Result<Receipt> {
        validate(size, mask, row, gate)?;
        let [width, height] = size;
        let right = origin[0].checked_add(width).context("light viewport X overflow")?;
        ensure!(pitch >= right, "light surface pitch");
        let bottom = origin[1].checked_add(height - 1).context("light viewport Y overflow")?;
        let end = bottom.checked_mul(pitch).and_then(|v| v.checked_add(right)).context("light surface range")?;
        ensure!(end <= pixels.len(), "light surface backing range");
        let mut receipt = Receipt::default();
        for y in 0..height { for x in 0..width {
            let level = row[usize::from(mask.pixels()[(y / 2) * mask.width() + x / 2] >> 3)];
            if receipt.visit(level, gate.map(|g| g[y * width + x])) {
                let at = (origin[1] + y) * pitch + origin[0] + x;
                pixels[at] = self.pixel(pixels[at], level);
            }
        }}
        Ok(receipt)
    }
    /// The modern compositor stores expanded RGB555 in RGBA. Keep every byte
    /// exact on remap31 and preserve alpha on transformed pixels as well.
    pub fn apply_rgba555(&self, pixels: &mut [u8], size: [usize; 2], mask: &Mask,
        row: &[u8; 32], gate: Option<&[u8]>) -> Result<Receipt> {
        ensure!(self.format == Format::Rgb555, "RGBA canvas requires RGB555 light channels");
        validate(size, mask, row, gate)?;
        let [width, height] = size;
        ensure!(pixels.len() == width * height * 4, "light RGBA byte count");
        let mut receipt = Receipt::default();
        for y in 0..height { for x in 0..width {
            let level = row[usize::from(mask.pixels()[(y / 2) * mask.width() + x / 2] >> 3)];
            if receipt.visit(level, gate.map(|g| g[y * width + x])) {
                let at = (y * width + x) * 4;
                let color = (u16::from(pixels[at] >> 3) << 10)
                    | (u16::from(pixels[at + 1] >> 3) << 5) | u16::from(pixels[at + 2] >> 3);
                let color = self.pixel(color, level);
                for (channel, shift) in [10, 5, 0].into_iter().enumerate() {
                    let value = ((color >> shift) & 31) as u8;
                    pixels[at + channel] = (value << 3) | (value >> 2);
                }
            }
        }}
        Ok(receipt)
    }
}

fn validate(size: [usize; 2], mask: &Mask, row: &[u8; 32], gate: Option<&[u8]>) -> Result<()> {
    let [width, height] = size;
    ensure!(width > 0 && height > 0 && width <= 4096 && height <= 4096, "light viewport dimensions");
    ensure!(mask.width() == width.div_ceil(2) && mask.height() == height.div_ceil(2), "light half-plane dimensions");
    ensure!(row.iter().all(|level| *level <= 31), "light remap outside0..31");
    ensure!(gate.is_none_or(|g| g.len() == width * height), "light gate byte count");
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Receipt { pub sampled: usize, pub transformed: usize, pub identity: usize, pub gated: usize }
impl Receipt {
    fn visit(&mut self, level: u8, gate: Option<u8>) -> bool {
        self.sampled += 1;
        if level == 31 { self.identity += 1; false }
        else if gate == Some(0) { self.gated += 1; false }
        else { self.transformed += 1; true }
    }
}

#[derive(Clone, Default)]
pub struct Remap { level: Option<u8>, row: [u8; 32] }
impl Remap {
    pub fn restore(level: Option<u8>, row: [u8; 32]) -> Result<Self> {
        ensure!(level.is_none_or(|v| v <= 15) && row.iter().all(|v| *v <= 31), "invalid retained light remap");
        Ok(Self { level, row })
    }
    pub fn row(&self) -> &[u8; 32] { &self.row }
    /// Original4BD150 caches only the clamped level. An alternate-bank change
    /// with the same cached level intentionally leaves the previous row intact.
    pub fn select(&mut self, requested: u32, alternate: bool) -> &[u8; 32] {
        let level = requested.min(15) as u8;
        if self.level != Some(level) {
            let bank = if alternate { &ALTERNATE } else { &NORMAL };
            self.row.copy_from_slice(&bank[usize::from(level) * 32..usize::from(level + 1) * 32]);
            self.level = Some(level);
        }
        &self.row
    }
    pub fn level(&self) -> Option<u8> { self.level }
}

// Original banks are appended by the pinned candidate fixture preparation.

const NORMAL: [u8; 512] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 28, 29, 29, 30, 30, 31, 31,
    10, 10, 11, 12, 12, 13, 14, 14, 15, 16, 16, 17, 18, 18, 19, 20, 21, 21, 22, 23, 23, 24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31,
    11, 11, 12, 12, 13, 14, 14, 15, 16, 16, 17, 18, 18, 19, 20, 20, 21, 22, 22, 23, 24, 24, 25, 26, 26, 27, 28, 28, 29, 30, 30, 31,
    13, 13, 14, 14, 15, 15, 16, 17, 17, 18, 18, 19, 20, 20, 21, 21, 22, 23, 23, 24, 24, 25, 26, 26, 27, 27, 28, 29, 29, 30, 30, 31,
    14, 14, 15, 15, 16, 16, 17, 17, 18, 19, 19, 20, 20, 21, 21, 22, 23, 23, 24, 24, 25, 25, 26, 26, 27, 28, 28, 29, 29, 30, 30, 31,
    16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 23, 24, 24, 25, 25, 26, 26, 27, 27, 28, 28, 29, 29, 30, 30, 31, 31,
    18, 18, 18, 19, 19, 20, 20, 21, 21, 21, 22, 22, 23, 23, 24, 24, 25, 25, 25, 26, 26, 27, 27, 28, 28, 28, 29, 29, 30, 30, 31, 31,
    19, 19, 19, 20, 20, 21, 21, 21, 22, 22, 23, 23, 23, 24, 24, 25, 25, 25, 26, 26, 27, 27, 27, 28, 28, 29, 29, 29, 30, 30, 31, 31,
    21, 21, 21, 22, 22, 22, 23, 23, 23, 24, 24, 24, 25, 25, 25, 26, 26, 26, 27, 27, 27, 28, 28, 28, 29, 29, 29, 30, 30, 30, 31, 31,
    23, 23, 23, 23, 24, 24, 24, 24, 25, 25, 25, 26, 26, 26, 26, 27, 27, 27, 28, 28, 28, 28, 29, 29, 29, 30, 30, 30, 30, 31, 31, 31,
    24, 24, 24, 24, 25, 25, 25, 25, 26, 26, 26, 26, 27, 27, 27, 27, 28, 28, 28, 28, 29, 29, 29, 29, 30, 30, 30, 30, 31, 31, 31, 31,
    26, 26, 26, 26, 26, 26, 27, 27, 27, 27, 27, 28, 28, 28, 28, 28, 29, 29, 29, 29, 29, 29, 30, 30, 30, 30, 30, 31, 31, 31, 31, 31,
    28, 28, 28, 28, 28, 28, 28, 28, 29, 29, 29, 29, 29, 29, 29, 29, 30, 30, 30, 30, 30, 30, 30, 30, 31, 31, 31, 31, 31, 31, 31, 31,
    29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
];

const ALTERNATE: [u8; 512] = [
    0, 1, 2, 4, 5, 7, 8, 10, 11, 13, 14, 16, 17, 19, 20, 22, 23, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    4, 5, 7, 8, 10, 11, 13, 14, 16, 17, 19, 20, 22, 23, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    8, 10, 11, 13, 14, 16, 17, 19, 20, 22, 23, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    10, 11, 13, 14, 16, 17, 19, 20, 22, 23, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    12, 13, 14, 16, 17, 19, 20, 22, 23, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    14, 16, 17, 19, 20, 22, 23, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    16, 17, 19, 20, 22, 23, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    18, 19, 20, 22, 23, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    20, 22, 23, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    22, 23, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    24, 25, 26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    26, 28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    28, 29, 30, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
    31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31, 31,
];
