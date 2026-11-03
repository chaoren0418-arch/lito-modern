//! Original source.lmp stencils and clipped unsigned-max light composition.
//! The 320×204 case is checked against isolated original 4BB170/4BB230 code.
//! Larger viewports keep the same stencil pixels and clipping rule; this module
//! does not choose actor sources, derive ambient time, or apply the pixel LUTs.
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub const ORIGINAL_WIDTH: usize = 320;
pub const ORIGINAL_HEIGHT: usize = 204;
pub const SOURCE_SHA256: &str = "30cf051fe516a053ac494b8054e5a320bab1f3f3dd7b0010e0cf0c74aacce5aa";
const HEADER_BYTES: usize = 192;
const SOURCE_LIMIT: usize = 1 << 20;
const MAX_MASK_SIDE: usize = 2048;

#[derive(Clone, Copy, Debug)]
struct Slot {
    offset: usize,
    width: usize,
    height: usize,
}

/// Owns one bounded resource, preserving shared ranges in the original header.
pub struct Source {
    bytes: Box<[u8]>,
    slots: [Slot; 16],
    sha256: [u8; 32],
}

impl Source {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(
            (HEADER_BYTES..=SOURCE_LIMIT).contains(&bytes.len()),
            "source.lmp byte budget"
        );
        let mut slots = [Slot {
            offset: 0,
            width: 0,
            height: 0,
        }; 16];
        for (i, slot) in slots.iter_mut().enumerate() {
            let offset = u32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap()) as usize;
            let at = 64 + i * 8;
            let width = i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
            let height = i32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap());
            ensure!(
                width >= 0 && height >= 0,
                "source.lmp negative dimensions at {i}"
            );
            ensure!(
                (width == 0) == (height == 0),
                "source.lmp partial empty slot at {i}"
            );
            let (width, height) = (width as usize, height as usize);
            ensure!(
                width <= ORIGINAL_WIDTH && height <= ORIGINAL_HEIGHT,
                "source.lmp stencil dimensions at {i}"
            );
            if width == 0 {
                ensure!(offset <= bytes.len(), "source.lmp empty slot offset at {i}");
            } else {
                let count = width
                    .checked_mul(height)
                    .context("source.lmp dimension overflow")?;
                let end = offset
                    .checked_add(count)
                    .context("source.lmp range overflow")?;
                ensure!(
                    offset >= HEADER_BYTES && end <= bytes.len(),
                    "source.lmp range at {i}"
                );
            }
            *slot = Slot {
                offset,
                width,
                height,
            };
        }
        Ok(Self {
            bytes: bytes.into(),
            slots,
            sha256: Sha256::digest(bytes).into(),
        })
    }

    /// The native loader names BASE Sprite explicitly; shard/hash fallback is
    /// inappropriate for the single source.lmp entry established by the audit.
    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        ensure!(
            !cancel.load(Ordering::Acquire),
            "light source load cancelled"
        );
        let mut archive = super::resources::SharedArchive::open_for_names(
            root,
            super::resources::SharedArchiveKind::Sprite,
            &["source.lmp"],
            cancel,
        )?;
        ensure!(
            (archive.entry("source.lmp")?.size as usize) <= SOURCE_LIMIT,
            "source.lmp entry budget"
        );
        let bytes = archive.read_entry("source.lmp")?;
        ensure!(
            !cancel.load(Ordering::Acquire),
            "light source load cancelled"
        );
        ensure!(
            format!("{:x}", Sha256::digest(&bytes)) == SOURCE_SHA256,
            "source.lmp differs from qualified original"
        );
        Self::parse(&bytes)
    }

    pub fn sha256(&self) -> [u8; 32] {
        self.sha256
    }
    pub fn stencil(&self, range: u8) -> Result<Stencil<'_>> {
        let slot = self
            .slots
            .get(usize::from(range))
            .context("source.lmp range outside 0..15")?;
        Ok(Stencil {
            width: slot.width,
            height: slot.height,
            pixels: &self.bytes[slot.offset..slot.offset + slot.width * slot.height],
        })
    }
}

#[derive(Clone, Copy)]
pub struct Stencil<'a> {
    width: usize,
    height: usize,
    pixels: &'a [u8],
}
impl<'a> Stencil<'a> {
    pub fn width(self) -> usize {
        self.width
    }
    pub fn height(self) -> usize {
        self.height
    }
    pub fn pixels(self) -> &'a [u8] {
        self.pixels
    }
}

/// Reusable half-resolution intensity plane. It is presentation state only.
pub struct Mask {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}
impl Mask {
    pub fn new(width: usize, height: usize) -> Result<Self> {
        ensure!(
            (1..=MAX_MASK_SIDE).contains(&width) && (1..=MAX_MASK_SIDE).contains(&height),
            "light mask dimensions"
        );
        Ok(Self {
            width,
            height,
            pixels: vec![0; width * height],
        })
    }
    pub fn for_pixels(width: usize, height: usize) -> Result<Self> {
        ensure!(width > 0 && height > 0, "light viewport dimensions");
        Self::new(width.div_ceil(2), height.div_ceil(2))
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    /// Use the untrimmed source width as row stride, including negative clips.
    pub fn compose(&mut self, stencil: Stencil<'_>, x: i32, y: i32) {
        self.compose_valid(
            stencil.pixels,
            stencil.width,
            stencil.height,
            i64::from(x),
            i64::from(y),
        );
    }

    /// A bounded raw plane is also useful when composing additional original
    /// sources. Invalid planes are rejected before any destination mutation.
    pub fn compose_plane(
        &mut self,
        pixels: &[u8],
        width: usize,
        height: usize,
        x: i32,
        y: i32,
    ) -> Result<()> {
        ensure!(
            width <= MAX_MASK_SIDE && height <= MAX_MASK_SIDE,
            "light source plane dimensions"
        );
        let len = width
            .checked_mul(height)
            .context("light source plane overflow")?;
        ensure!(len == pixels.len(), "light source plane byte count");
        self.compose_valid(pixels, width, height, i64::from(x), i64::from(y));
        Ok(())
    }

    /// 485A40 subtracts the stencil dimensions, halves toward zero, and clears
    /// the low two x bits. Negative centers must not use floor division.
    pub fn compose_centered(&mut self, stencil: Stencil<'_>, center_x: i32, center_y: i32) {
        let x = ((i64::from(center_x) - stencil.width as i64) / 2) & !3;
        let y = (i64::from(center_y) - stencil.height as i64) / 2;
        self.compose_valid(stencil.pixels, stencil.width, stencil.height, x, y);
    }

    fn compose_valid(&mut self, pixels: &[u8], width: usize, height: usize, x: i64, y: i64) {
        let left = x.max(0).min(self.width as i64);
        let top = y.max(0).min(self.height as i64);
        let right = (x + width as i64).min(self.width as i64);
        let bottom = (y + height as i64).min(self.height as i64);
        if left >= right || top >= bottom {
            return;
        }
        let count = (right - left) as usize;
        for yy in top..bottom {
            let src = ((yy - y) as usize) * width + (left - x) as usize;
            let dst = yy as usize * self.width + left as usize;
            for (out, input) in self.pixels[dst..dst + count]
                .iter_mut()
                .zip(&pixels[src..src + count])
            {
                *out = (*out).max(*input);
            }
        }
    }
}
