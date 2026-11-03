//! Sealed batch9 dense IMG word operations and conditional pane layer order.
//! CPU descriptors replace unqualified external surface methods. Inspection
//! callbacks provide controls/grid pixels, not a live Native owner or GUI.
use super::Format;
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Member {
    Layer933,
    Frame934,
    Layer935,
}
impl Member {
    fn pin(self) -> (&'static str, [usize; 2]) {
        match self {
            Self::Layer933 => (
                "3B49C6E154334E3252BEDCD1FF101D5362F787F5892FF49EE3191FE73EF978E7",
                [191, 274],
            ),
            Self::Frame934 => (super::FRAME934_SHA, [207, 289]),
            Self::Layer935 => (
                "479EFD1EC7E4838FA33881CC0C9E5C387B7D98B4491995733D0F7C53A5191F26",
                [203, 261],
            ),
        }
    }
}
/// Original555 words and header preserved independently of format caches.
pub struct DenseSource {
    width: usize,
    height: usize,
    flag: u16,
    key: u16,
    words: Vec<u16>,
}
impl DenseSource {
    pub fn decode_raw555(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= 4 << 20, "dense IMG source budget");
        let h = bytes.get(..8).context("short dense IMG header")?;
        let w = i16::from_le_bytes([h[0], h[1]]);
        let y = i16::from_le_bytes([h[2], h[3]]);
        ensure!(
            w > 0 && y > 0,
            "positive signed dense IMG dimensions required"
        );
        let width = w as usize;
        let height = y as usize;
        ensure!(
            bytes.len() == 8 + width * height * 2,
            "exact dense IMG pixels required"
        );
        Ok(Self {
            width,
            height,
            flag: u16::from_le_bytes([h[4], h[5]]),
            key: u16::from_le_bytes([h[6], h[7]]),
            words: bytes[8..]
                .chunks_exact(2)
                .map(|v| u16::from_le_bytes([v[0], v[1]]))
                .collect(),
        })
    }
    pub fn qualified_member(member: Member, bytes: &[u8]) -> Result<Self> {
        let (pin, size) = member.pin();
        ensure!(
            format!("{:X}", Sha256::digest(bytes)) == pin,
            "qualified native LTsurf member SHA required"
        );
        let source = Self::decode_raw555(bytes)?;
        ensure!(
            [source.width, source.height] == size,
            "qualified member dimensions"
        );
        Ok(source)
    }
    pub fn cache(&self, format: Format) -> DenseCache {
        DenseCache {
            format,
            width: self.width,
            height: self.height,
            flag: self.flag,
            key: if self.flag != 0 {
                format.convert_raw555(self.key)
            } else {
                self.key
            },
            words: self
                .words
                .iter()
                .map(|v| format.convert_raw555(*v))
                .collect(),
        }
    }
}
pub struct DenseCache {
    format: Format,
    width: usize,
    height: usize,
    flag: u16,
    key: u16,
    words: Vec<u16>,
}
impl DenseCache {
    /// For independently qualified cache bytes in the declared format; no loader
    /// conversion is performed here. Use DenseSource::cache for raw555 assets.
    pub fn decode_inspected_cache(format: Format, bytes: &[u8]) -> Result<Self> {
        let s = DenseSource::decode_raw555(bytes)?;
        Ok(Self {
            format,
            width: s.width,
            height: s.height,
            flag: s.flag,
            key: s.key,
            words: s.words,
        })
    }
    pub fn format(&self) -> Format {
        self.format
    }
    pub fn dimensions(&self) -> [usize; 2] {
        [self.width, self.height]
    }
    pub fn flag(&self) -> u16 {
        self.flag
    }
    pub fn key(&self) -> u16 {
        self.key
    }
    pub fn words(&self) -> &[u16] {
        &self.words
    }
    pub fn draw(
        &self,
        surface: &mut CpuSurface,
        origin: [i32; 2],
        mode: ImgMode,
    ) -> Result<DrawReceipt> {
        ensure!(
            self.format == surface.descriptor.format,
            "IMG cache/surface format epoch mismatch"
        );
        if let ImgMode::Alpha(a) = mode {
            ensure!(
                a <= 31,
                "alpha outside qualified initialized0..31, no clamp"
            );
        }
        if !surface.bottom_guard(origin[1], self.height as i32) {
            return Ok(DrawReceipt {
                bottom_rejected: true,
                writes: 0,
            });
        }
        let stride = i64::from(surface.stride());
        let mut writes = Vec::new();
        for row in 0..self.height {
            let y = i64::from(origin[1]) + row as i64;
            if matches!(mode, ImgMode::Alpha(_)) && y < 0 {
                continue;
            }
            for col in 0..self.width {
                let x = i64::from(origin[0]) + col as i64;
                if matches!(mode, ImgMode::Alpha(_)) && x < 0 {
                    continue;
                }
                let s = self.words[row * self.width + col];
                if !matches!(mode, ImgMode::Opaque) && s == self.key {
                    continue;
                }
                let at = surface.descriptor.buffer_offset as i64 + y * stride + 2 * x;
                ensure!(
                    at >= 0 && at + 2 <= surface.bytes.len() as i64,
                    "IMG word outside allocated audit buffer; no guessed clipping"
                );
                writes.push((at as usize, s));
            }
        }
        // Read destination at commit time, preserving the original sequential
        // behavior even when a wide source overlaps the next audit row.
        for (at, s) in &writes {
            let v = match mode {
                ImgMode::Opaque | ImgMode::Keyed => *s,
                ImgMode::Alpha(a) => {
                    let d = u16::from_le_bytes([surface.bytes[*at], surface.bytes[*at + 1]]);
                    alpha_word(surface.descriptor.format, d, *s, a)
                }
            };
            surface.bytes[*at..*at + 2].copy_from_slice(&v.to_le_bytes());
        }
        Ok(DrawReceipt {
            bottom_rejected: false,
            writes: writes.len(),
        })
    }
    /// Known935 bottom rejection does not require its unresolved lookup table.
    /// Admitted full-rectangle pixels stop at the external conversion boundary.
    pub fn draw935_boundary(&self, surface: &CpuSurface, origin: [i32; 2]) -> Result<Final935> {
        ensure!(
            self.format == surface.descriptor.format,
            "IMG cache/surface format epoch mismatch"
        );
        Ok(if surface.bottom_guard(origin[1], self.height as i32) {
            Final935::UnsupportedExternalRounding
        } else {
            Final935::BottomRejected
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImgMode {
    Opaque,
    Keyed,
    Alpha(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activation933 {
    Skipped,
    Draw(ImgMode),
}
pub fn activation933(full_dword: u32) -> Activation933 {
    match full_dword {
        0 => Activation933::Skipped,
        31 => Activation933::Draw(ImgMode::Opaque),
        v => Activation933::Draw(ImgMode::Alpha(v as u16)),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawReceipt {
    pub bottom_rejected: bool,
    pub writes: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Final935 {
    Absent,
    BottomRejected,
    UnsupportedExternalRounding,
}
pub const UNSUPPORTED_935:&str="935 LUT unavailable: 4C1972 -> 4F1592 -> 7741AC20 external float-to-int rounding is unqualified";
fn shifts(format: Format) -> [u32; 3] {
    match format {
        Format::Rgb555 => [0, 5, 10],
        Format::Rgb565 => [0, 6, 11],
    }
}
pub fn alpha_word(format: Format, d: u16, s: u16, a: u16) -> u16 {
    assert!(a <= 31);
    let mut out = 0;
    for shift in shifts(format) {
        let dd = (d >> shift) & 31;
        let ss = (s >> shift) & 31;
        out |= ((dd * (31 - a) + ss * a) / 31) << shift;
    }
    out
}
/// Reconstruct all32 alpha slices under an explicit stableformat condition.
pub fn alpha_channel_table(format: Format, channel: usize) -> Result<Vec<u16>> {
    let shift = *shifts(format).get(channel).context("alpha channel index")?;
    let mut table = Vec::with_capacity(32768);
    for a in 0..32u16 {
        for d in 0..32u16 {
            for s in 0..32u16 {
                table.push(((d * (31 - a) + s * a) / 31) << shift);
            }
        }
    }
    Ok(table)
}
#[derive(Clone, Copy, Debug)]
pub struct Descriptor {
    pub format: Format,
    pub pitch: i32,
    pub height: i32,
    pub buffer_offset: usize,
    pub synthetic_native_base: u32,
}
#[derive(Clone)]
pub struct CpuSurface {
    descriptor: Descriptor,
    bytes: Vec<u8>,
}
impl CpuSurface {
    pub fn new(descriptor: Descriptor, bytes: Vec<u8>) -> Result<Self> {
        ensure!(
            descriptor.pitch >= 2 && descriptor.height > 0 && bytes.len() <= 16 << 20,
            "bounded positive CPU surface descriptor"
        );
        let stride = 2 * (i64::from(descriptor.pitch) / 2);
        ensure!(
            descriptor.buffer_offset as i64 + i64::from(descriptor.height) * stride
                <= bytes.len() as i64,
            "surface descriptor exceeds audit allocation"
        );
        Ok(Self { descriptor, bytes })
    }
    pub fn descriptor(&self) -> Descriptor {
        self.descriptor
    }
    pub fn stride(&self) -> i32 {
        2 * (self.descriptor.pitch / 2)
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
    /// Explicit synthetic32-bit buffer address lets tests reproduce the exact
    /// unsigned bottom comparison without claiming any real native pointer.
    pub fn bottom_guard(&self, y: i32, draw_height: i32) -> bool {
        let end = self
            .descriptor
            .synthetic_native_base
            .wrapping_add(y.wrapping_add(draw_height).wrapping_mul(self.stride()) as u32);
        let limit = self
            .descriptor
            .synthetic_native_base
            .wrapping_add(self.descriptor.height.wrapping_mul(self.stride()) as u32);
        end <= limit
    }
}
pub struct PaneLayers<'a> {
    pub background933: Option<&'a DenseCache>,
    pub frame934: Option<&'a DenseCache>,
    pub final935: Option<&'a DenseCache>,
}
#[derive(Clone, Debug)]
pub enum Stage {
    Background933 { mode: ImgMode, receipt: DrawReceipt },
    Frame934(DrawReceipt),
    SuppliedControls,
    SuppliedGrid,
    Final935(Final935),
}
pub struct Composition {
    pub stages: Vec<Stage>,
    pub final935: Final935,
    pub first_unsupported: Option<&'static str>,
}
fn offset(origin: [i32; 2], delta: [i32; 2]) -> Result<[i32; 2]> {
    Ok([
        origin[0]
            .checked_add(delta[0])
            .context("layer x overflow")?,
        origin[1]
            .checked_add(delta[1])
            .context("layer y overflow")?,
    ])
}
/// Exact call order for supplied inspection layers and callbacks. Preserve
/// the known prefix when935 is unqualified, with an explicit boundary receipt.
/// This result never proves child controls, skills or a native session ready.
pub fn compose_inspection<C, G>(
    surface: &mut CpuSurface,
    layers: PaneLayers<'_>,
    full_dword933: u32,
    child_plus_translation: [i32; 2],
    controls: C,
    grid: G,
) -> Result<Composition>
where
    C: FnOnce(&mut CpuSurface) -> Result<()>,
    G: FnOnce(&mut CpuSurface) -> Result<()>,
{
    let mut next = surface.clone();
    let mut stages = Vec::new();
    if let (Some(img), Activation933::Draw(mode)) =
        (layers.background933, activation933(full_dword933))
    {
        let receipt = img.draw(&mut next, offset(child_plus_translation, [8, 10])?, mode)?;
        stages.push(Stage::Background933 { mode, receipt });
    }
    if let Some(img) = layers.frame934 {
        stages.push(Stage::Frame934(img.draw(
            &mut next,
            child_plus_translation,
            ImgMode::Keyed,
        )?));
    }
    controls(&mut next)?;
    stages.push(Stage::SuppliedControls);
    grid(&mut next)?;
    stages.push(Stage::SuppliedGrid);
    let final935 = if let Some(img) = layers.final935 {
        img.draw935_boundary(&next, offset(child_plus_translation, [2, 10])?)?
    } else {
        Final935::Absent
    };
    stages.push(Stage::Final935(final935));
    *surface = next;
    Ok(Composition {
        stages,
        final935,
        first_unsupported: (final935 == Final935::UnsupportedExternalRounding)
            .then_some(UNSUPPORTED_935),
    })
}
