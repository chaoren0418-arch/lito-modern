//! Deterministic CPU reference compositor for decoded game art.
//!
//! The opaque RGBA surface provides a golden-image path independent of a future
//! GPU backend. Sprite anchors use signed coordinates; clipping never changes
//! the source stride or wraps offsets at a window edge.

use anyhow::{ensure, Context, Result};

#[path = "compositor_dib.rs"]
mod dib;

const MAX_PIXELS: u64 = 16 * 1024 * 1024;

#[derive(Clone, Copy)]
struct FragmentRow {
    start: u8,
    end: u8,
    opaque: bool,
}
#[derive(Clone)]
pub(crate) struct FragmentRows([FragmentRow; 24]);
impl FragmentRows {
    pub(crate) const BYTES: usize = std::mem::size_of::<Self>();
    pub(crate) fn new(rgba: &[u8]) -> Result<Self> {
        ensure!(rgba.len() == 2304, "invalid prepared fragment");
        let mut rows = [FragmentRow {
            start: 0,
            end: 0,
            opaque: false,
        }; 24];
        for (row, pixels) in rows.iter_mut().zip(rgba.chunks_exact(96)) {
            let mut coverage = 0u32;
            let mut solid = 0u32;
            for x in 0..24 {
                let alpha = pixels[x * 4 + 3];
                coverage |= ((alpha != 0) as u32) << x;
                solid |= ((alpha == 255) as u32) << x;
            }
            let start = coverage.trailing_zeros().min(24);
            let end = (32 - coverage.leading_zeros()).max(start);
            let span = ((1u32 << end) - 1) & !((1u32 << start) - 1);
            *row = FragmentRow {
                start: start as u8,
                end: end as u8,
                opaque: solid == span,
            };
        }
        Ok(Self(rows))
    }
}

pub struct Canvas {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        ensure!(
            width > 0 && height > 0,
            "canvas dimensions must be positive"
        );
        let pixels = u64::from(width) * u64::from(height);
        ensure!(pixels <= MAX_PIXELS, "canvas exceeds pixel budget");
        let mut rgba = Vec::new();
        rgba.try_reserve_exact(pixels as usize * 4)
            .context("allocate bounded canvas")?;
        rgba.resize(pixels as usize * 4, 0);
        let mut canvas = Self {
            width,
            height,
            rgba,
        };
        canvas.clear([0, 0, 0]);
        Ok(canvas)
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    /// Apply the original RGB555 channel path after world composition.
    #[cfg(windows)]
    pub fn apply_native_light_mask(&mut self, channels: &super::native_light_color::Channels,
        mask: &super::native_light_mask::Mask, row: &[u8; 32], gate: Option<&[u8]>)
        -> Result<super::native_light_color::Receipt> {
        channels.apply_rgba555(&mut self.rgba, [self.width as usize, self.height as usize], mask, row, gate)
    }
    #[cfg(windows)]
    pub fn apply_native_water(&mut self,channels:&super::native_water_light::Channels,
        mask:&super::native_light_mask::Mask,row:&[u8;32],shade:&super::native_water_light::Shade,
        params:super::native_water_light::Params)->Result<super::native_water_light::Receipt> {
        channels.apply_rgba555(&mut self.rgba,[self.width as usize,self.height as usize],mask,row,shade,params)
    }

    /// Restores a retained background without allocating a new frame surface.
    pub fn copy_from(&mut self, source: &Canvas) -> Result<()> {
        ensure!(
            self.width == source.width && self.height == source.height,
            "canvas copy dimensions do not match"
        );
        self.rgba.copy_from_slice(&source.rgba);
        Ok(())
    }

    /// Exact pixel crop: copy rows without re-blending retained terrain.
    pub fn crop(&self, left: u32, top: u32, width: u32, height: u32) -> Result<Self> {
        ensure!(
            u64::from(left) + u64::from(width) <= u64::from(self.width)
                && u64::from(top) + u64::from(height) <= u64::from(self.height),
            "canvas crop outside source"
        );
        let mut result = Self::new(width, height)?;
        for row in 0..height as usize {
            let start = ((top as usize + row) * self.width as usize + left as usize) * 4;
            let to = row * width as usize * 4;
            result.rgba[to..to + width as usize * 4]
                .copy_from_slice(&self.rgba[start..start + width as usize * 4]);
        }
        Ok(result)
    }

    pub fn clear(&mut self, rgb: [u8; 3]) {
        // Canvas is nonempty and contains whole pixels. Grow an initialized
        // colour pattern with bounded bulk copies, rather than interpreting a
        // per-pixel loop for every full-frame clear in development builds.
        self.rgba[..4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        let mut filled = 4;
        while filled < self.rgba.len() {
            let count = filled.min(self.rgba.len() - filled);
            self.rgba.copy_within(..count, filled);
            filled += count;
        }
    }

    pub fn blit_rgba(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> Result<()> {
        let pixels = u64::from(width) * u64::from(height);
        ensure!(pixels <= MAX_PIXELS, "source exceeds pixel budget");
        ensure!(
            rgba.len() as u64 == pixels * 4,
            "RGBA length does not match dimensions"
        );
        let left = i64::from(x).max(0);
        let top = i64::from(y).max(0);
        let right = (i64::from(x) + i64::from(width)).min(i64::from(self.width));
        let bottom = (i64::from(y) + i64::from(height)).min(i64::from(self.height));
        for dy in top..bottom {
            for dx in left..right {
                let source = (((dy - i64::from(y)) as u64 * u64::from(width)
                    + (dx - i64::from(x)) as u64)
                    * 4) as usize;
                let target = ((dy as u64 * u64::from(self.width) + dx as u64) * 4) as usize;
                let alpha = u32::from(rgba[source + 3]);
                for component in 0..3 {
                    self.rgba[target + component] = ((u32::from(rgba[source + component]) * alpha
                        + u32::from(self.rgba[target + component]) * (255 - alpha)
                        + 127)
                        / 255) as u8;
                }
            }
        }
        Ok(())
    }

    /// Native HUD gauge: the source is fixed-size, but only its leftmost
    /// current/maximum fraction is visible. Transparent source pixels retain
    /// the already painted base; no stale pixels survive a fresh frame.
    pub(crate) fn blit_rgba_left(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        visible: u32,
        x: i32,
        y: i32,
    ) -> Result<()> {
        ensure!(
            visible <= width && rgba.len() as u64 == u64::from(width) * u64::from(height) * 4,
            "HUD gauge source/crop invalid"
        );
        let left = i64::from(x).max(0);
        let top = i64::from(y).max(0);
        let right = (i64::from(x) + i64::from(visible)).min(i64::from(self.width));
        let bottom = (i64::from(y) + i64::from(height)).min(i64::from(self.height));
        for dy in top..bottom {
            for dx in left..right {
                let source = (((dy - i64::from(y)) as u64 * u64::from(width)
                    + (dx - i64::from(x)) as u64)
                    * 4) as usize;
                if rgba[source + 3] == 0 {
                    continue;
                }
                let target = ((dy as u64 * u64::from(self.width) + dx as u64) * 4) as usize;
                self.rgba[target..target + 3].copy_from_slice(&rgba[source..source + 3]);
            }
        }
        Ok(())
    }

    /// Fixed world fragment, painting coverage and color in one clipped pass.
    /// Fast binary-coverage paths are mathematically identical to blit_rgba;
    /// partial alpha retains the already-visible underlying target owner.
    /// Visible state-tinted actor coverage remains selectable (modern policy).
    pub(crate) fn blit_actor_state_fragment(
        &mut self,
        rgba: &[u8],
        x: i32,
        y: i32,
        owners: &mut [u16],
        owner: u16,
        style: super::native_actor_states::Style,
    ) -> Result<()> {
        use super::native_actor_states::Style;
        ensure!(
            rgba.len() == 24 * 24 * 4 && owners.len() == self.width as usize * self.height as usize,
            "actor state fragment dimensions"
        );
        ensure!(
            matches!(style, Style::Half | Style::Dim | Style::Green(8..=14)),
            "unqualified actor tone"
        );
        ensure!(
            rgba.chunks_exact(4).all(|p| matches!(p[3], 0 | 255)),
            "actor state alpha"
        );
        let left = i64::from(x).max(0);
        let top = i64::from(y).max(0);
        let right = (i64::from(x) + 24).min(i64::from(self.width));
        let bottom = (i64::from(y) + 24).min(i64::from(self.height));
        for yy in top..bottom {
            for xx in left..right {
                let src = (((yy - i64::from(y)) * 24 + xx - i64::from(x)) * 4) as usize;
                if rgba[src + 3] == 0 {
                    continue;
                }
                let dst = yy as usize * self.width as usize + xx as usize;
                for c in 0..3 {
                    let s = rgba[src + c] >> 3;
                    let old = self.rgba[dst * 4 + c] >> 3;
                    let value = match style {
                        Style::Half => (s + old) / 2,
                        Style::Dim => s / 2 + 3,
                        Style::Green(boost) => {
                            if c == 1 {
                                (s + boost).min(31)
                            } else {
                                s
                            }
                        }
                        _ => unreachable!(),
                    };
                    self.rgba[dst * 4 + c] = (value << 3) | (value >> 2);
                }
                self.rgba[dst * 4 + 3] = 255;
                owners[dst] = owner;
            }
        }
        Ok(())
    }

    pub(crate) fn blit_world_fragment(
        &mut self,
        rgba: &[u8],
        x: i32,
        y: i32,
        owners: &mut [u16],
        owner: u16,
    ) -> Result<()> {
        let rows = FragmentRows::new(rgba)?;
        self.blit_prepared_world_fragment(rgba, &rows, x, y, owners, owner)
    }

    pub(crate) fn blit_prepared_world_fragment(
        &mut self,
        rgba: &[u8],
        rows: &FragmentRows,
        x: i32,
        y: i32,
        owners: &mut [u16],
        owner: u16,
    ) -> Result<()> {
        ensure!(
            rgba.len() == 24 * 24 * 4 && owners.len() == self.width as usize * self.height as usize,
            "invalid world fragment/owner surface"
        );
        let left = i64::from(x).max(0);
        let top = i64::from(y).max(0);
        let right = (i64::from(x) + 24).min(i64::from(self.width));
        let bottom = (i64::from(y) + 24).min(i64::from(self.height));
        if left >= right || top >= bottom {
            return Ok(());
        }
        for row in top..bottom {
            let sy = (row - i64::from(y)) as usize;
            let span = rows.0[sy];
            let sx = (left - i64::from(x)).max(i64::from(span.start));
            let end = (right - i64::from(x)).min(i64::from(span.end));
            if sx >= end {
                continue;
            }
            let columns = (end - sx) as usize;
            let source = (sy * 24 + sx as usize) * 4;
            let target = row as usize * self.width as usize + (i64::from(x) + sx) as usize;
            let source = &rgba[source..source + columns * 4];
            let destination = &mut self.rgba[target * 4..(target + columns) * 4];
            if span.opaque {
                destination.copy_from_slice(source);
                owners[target..target + columns].fill(owner);
                continue;
            }
            for ((s, d), hit) in source
                .chunks_exact(4)
                .zip(destination.chunks_exact_mut(4))
                .zip(&mut owners[target..target + columns])
            {
                match s[3] {
                    0 => {}
                    255 => {
                        d.copy_from_slice(s);
                        *hit = owner;
                    }
                    alpha => {
                        let a = u32::from(alpha);
                        for (s, d) in s[..3].iter().zip(&mut d[..3]) {
                            *d =
                                ((u32::from(*s) * a + u32::from(*d) * (255 - a) + 127) / 255) as u8;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// RGB555 roof transition. 4C1A17 builds the exact integer table
    /// floor((destination5 * (31-weight) + source5 * weight) / 31).
    /// Stable half-opacity uses a different average table (4C0F80/4C0FA0).
    /// Zero-alpha is uncovered; 128 is the qualified TIL 6/7 base coverage,
    /// not another multiplier. The caller retains underlying hit ownership.
    pub(crate) fn blit_roof_rgb555(
        &mut self,
        rgba: &[u8],
        x: i32,
        y: i32,
        shade: super::world_roofs::Shade,
    ) -> Result<()> {
        use super::world_roofs::Shade;
        ensure!(rgba.len() == 24 * 24 * 4, "roof fragment dimensions");
        ensure!(
            rgba.chunks_exact(4).all(|p| matches!(p[3], 0 | 128 | 255)),
            "roof requires qualified coverage"
        );
        ensure!(
            matches!(shade, Shade::Half | Shade::Hidden | Shade::Weight(0..=31)),
            "invalid roof shade"
        );
        if shade == Shade::Hidden {
            return Ok(());
        }
        let left = i64::from(x).max(0);
        let top = i64::from(y).max(0);
        let right = (i64::from(x) + 24).min(i64::from(self.width));
        let bottom = (i64::from(y) + 24).min(i64::from(self.height));
        for dy in top..bottom {
            for dx in left..right {
                let src = (((dy - i64::from(y)) * 24 + dx - i64::from(x)) * 4) as usize;
                if rgba[src + 3] == 0 {
                    continue;
                }
                let dst = ((dy * self.width as i64 + dx) * 4) as usize;
                for c in 0..3 {
                    let s = u16::from(rgba[src + c] >> 3);
                    let d = u16::from(self.rgba[dst + c] >> 3);
                    let v = match shade {
                        Shade::Half => (s + d) / 2,
                        Shade::Weight(w) => (s * u16::from(w) + d * (31 - u16::from(w))) / 31,
                        _ => unreachable!(),
                    } as u8;
                    self.rgba[dst + c] = (v << 3) | (v >> 2);
                }
            }
        }
        Ok(())
    }

    /// Blends an explicitly identified native actor-shadow layer. This is the
    /// RGB555 reference path, not a claim of RGB565 framebuffer pixel parity.
    /// Native actor+0x88 shadow callers (4E0233/4E7F03) add draw flags 5;
    /// 488DD0 -> 4C0F80/4C0FA0 uses floor((destination5 + source5) / 2).
    /// SPR block type 8 is not the selector: ordinary sprites are unchanged.
    /// Alpha is a binary coverage mask, not an additional opacity multiplier.
    pub fn blit_native_shadow_rgb555(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> Result<()> {
        let pixels = u64::from(width) * u64::from(height);
        ensure!(pixels <= MAX_PIXELS, "shadow source exceeds pixel budget");
        ensure!(
            rgba.len() as u64 == pixels * 4,
            "shadow RGBA length does not match dimensions"
        );
        // Validate the entire source before touching the destination, including
        // clipped pixels, so unsupported partial-alpha data fails atomically.
        ensure!(
            rgba.chunks_exact(4)
                .all(|pixel| matches!(pixel[3], 0 | 255)),
            "native shadow requires binary source coverage"
        );
        let left = i64::from(x).max(0);
        let top = i64::from(y).max(0);
        let right = (i64::from(x) + i64::from(width)).min(i64::from(self.width));
        let bottom = (i64::from(y) + i64::from(height)).min(i64::from(self.height));
        for dy in top..bottom {
            for dx in left..right {
                let source = (((dy - i64::from(y)) as u64 * u64::from(width)
                    + (dx - i64::from(x)) as u64)
                    * 4) as usize;
                if rgba[source + 3] == 0 {
                    continue;
                }
                let target = ((dy as u64 * u64::from(self.width) + dx as u64) * 4) as usize;
                for component in 0..3 {
                    let source5 = rgba[source + component] >> 3;
                    let destination5 = self.rgba[target + component] >> 3;
                    let average5 = (source5 + destination5) >> 1;
                    self.rgba[target + component] = (average5 << 3) | (average5 >> 2);
                }
            }
        }
        Ok(())
    }

    /// Explicit modern RGB555 floor-screen reference for a qualified attr(8)
    /// effect. Native 488DAD selects 4C0EC0/4C0EE0 and the screen-family tables
    /// generated at 4C195D, not ordinary alpha-over or the shadow average.
    /// The native external float-to-integer thunk's rounding is not established;
    /// this policy deliberately uses floor and does not claim RGB565 parity.
    /// SPR block type/depth bytes are not a selector for this method.
    /// Alpha is binary coverage. Uncovered pixels are unchanged bit-for-bit;
    /// covered source/destination channels are quantized to five bits.
    pub fn blit_screen_rgb555(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> Result<()> {
        let pixels = u64::from(width) * u64::from(height);
        ensure!(pixels <= MAX_PIXELS, "screen source exceeds pixel budget");
        ensure!(
            rgba.len() as u64 == pixels * 4,
            "screen RGBA length does not match dimensions"
        );
        // Preflight the whole source, including clipped pixels, so rejection
        // cannot leave even the first visible pixel partially composited.
        ensure!(
            rgba.chunks_exact(4)
                .all(|pixel| matches!(pixel[3], 0 | 255)),
            "screen requires binary source coverage"
        );
        let left = i64::from(x).max(0);
        let top = i64::from(y).max(0);
        let right = (i64::from(x) + i64::from(width)).min(i64::from(self.width));
        let bottom = (i64::from(y) + i64::from(height)).min(i64::from(self.height));
        for dy in top..bottom {
            for dx in left..right {
                let source = (((dy - i64::from(y)) as u64 * u64::from(width)
                    + (dx - i64::from(x)) as u64)
                    * 4) as usize;
                if rgba[source + 3] == 0 {
                    continue;
                }
                let target = ((dy as u64 * u64::from(self.width) + dx as u64) * 4) as usize;
                for component in 0..3 {
                    let source5 = u16::from(rgba[source + component] >> 3);
                    let destination5 = u16::from(self.rgba[target + component] >> 3);
                    let high = source5.max(destination5);
                    let low = source5.min(destination5);
                    let screen5 = (high + low * (31 - high) / 31) as u8;
                    self.rgba[target + component] = (screen5 << 3) | (screen5 >> 2);
                }
            }
        }
        Ok(())
    }

    /// Qualified attr104=128 multiply layer: 488DE5 -> 4C0F40/4C0F60 ->
    /// 80D89C/80E09C/80E89C. 4C1988..4C19C5 builds floor(dst5*src5/31)
    /// using signed constant division of a nonnegative product. Binary SPR
    /// coverage is not opacity; covered black darkens, transparent black does not.
    /// RGB555 reference only, not a claim of RGB565 framebuffer equivalence.
    pub fn blit_multiply_rgb555(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> Result<()> {
        let pixels = u64::from(width) * u64::from(height);
        ensure!(pixels <= MAX_PIXELS, "multiply source exceeds pixel budget");
        ensure!(
            rgba.len() as u64 == pixels * 4,
            "multiply RGBA length does not match dimensions"
        );
        ensure!(
            rgba.chunks_exact(4).all(|p| matches!(p[3], 0 | 255)),
            "multiply requires binary source coverage"
        );
        let left = i64::from(x).max(0);
        let top = i64::from(y).max(0);
        let right = (i64::from(x) + i64::from(width)).min(i64::from(self.width));
        let bottom = (i64::from(y) + i64::from(height)).min(i64::from(self.height));
        for dy in top..bottom {
            for dx in left..right {
                let src = (((dy - i64::from(y)) as u64 * u64::from(width)
                    + (dx - i64::from(x)) as u64)
                    * 4) as usize;
                if rgba[src + 3] == 0 {
                    continue;
                }
                let dst = ((dy as u64 * u64::from(self.width) + dx as u64) * 4) as usize;
                for c in 0..3 {
                    let v = (u16::from(rgba[src + c] >> 3) * u16::from(self.rgba[dst + c] >> 3)
                        / 31) as u8;
                    self.rgba[dst + c] = (v << 3) | (v >> 2);
                }
            }
        }
        Ok(())
    }

    /// Converts into a retained Windows top-down DIB buffer, not a new buffer
    /// on every WM_PAINT. The alpha byte remains opaque.
    pub fn copy_bgra(&self, output: &mut Vec<u8>) -> Result<()> {
        if output.len() != self.rgba.len() {
            output
                .try_reserve(self.rgba.len().saturating_sub(output.len()))
                .context("allocate bounded DIB")?;
            output.resize(self.rgba.len(), 0);
        }
        dib::copy(&self.rgba, output);
        Ok(())
    }
}

/// Integer aspect-preserving viewport in client coordinates, shared by
/// presentation and future inverse hit testing. No desktop/window origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Viewport {
    /// Inverse of the exact viewport used by StretchDIBits. Coordinates are
    /// client pixels (PerMonitorV2); letterbox bars and right/bottom edges miss.
    pub fn canvas_point(self, x: i32, y: i32, width: u32, height: u32) -> Option<(i32, i32)> {
        if self.width <= 0
            || self.height <= 0
            || width == 0
            || height == 0
            || width > i32::MAX as u32
            || height > i32::MAX as u32
        {
            return None;
        }
        let dx = i64::from(x) - i64::from(self.x);
        let dy = i64::from(y) - i64::from(self.y);
        if dx < 0 || dy < 0 || dx >= i64::from(self.width) || dy >= i64::from(self.height) {
            return None;
        }
        Some((
            (dx * i64::from(width) / i64::from(self.width)) as i32,
            (dy * i64::from(height) / i64::from(self.height)) as i32,
        ))
    }

    pub fn fit(client_width: i32, client_height: i32, width: u32, height: u32) -> Option<Self> {
        if client_width <= 0 || client_height <= 0 || width == 0 || height == 0 {
            return None;
        }
        let cw = client_width as u64;
        let ch = client_height as u64;
        let (w, h) = if cw * u64::from(height) <= ch * u64::from(width) {
            (cw, (cw * u64::from(height) / u64::from(width)).max(1))
        } else {
            ((ch * u64::from(width) / u64::from(height)).max(1), ch)
        };
        Some(Self {
            x: (client_width - w as i32) / 2,
            y: (client_height - h as i32) / 2,
            width: w as i32,
            height: h as i32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulk_clear_preserves_opaque_pixels_at_pattern_and_frame_boundaries() {
        for (width, height) in
            (1..130)
                .map(|width| (width, 3))
                .chain([(1, 1), (800, 600), (1920, 1080)])
        {
            let mut canvas = Canvas::new(width, height).unwrap();
            for rgb in [[0, 0, 0], [255, 255, 255], [18, 25, 35], [255, 0, 129]] {
                canvas.rgba.fill(0x71);
                canvas.clear(rgb);
                assert!(canvas
                    .rgba
                    .chunks_exact(4)
                    .all(|p| p == [rgb[0], rgb[1], rgb[2], 255]));
            }
        }
    }

    #[test]
    #[ignore = "explicit bounded CPU measurement, not a frame-rate acceptance test"]
    fn measure_full_frame_clear_and_dib_conversion() {
        use std::{hint::black_box, time::Instant};
        let mut canvas = Canvas::new(800, 600).unwrap();
        let mut dib = vec![0; 800 * 600 * 4];
        let frames = 100;
        let start = Instant::now();
        for i in 0..frames {
            canvas.clear(black_box([i as u8, 25, 35]));
            black_box(canvas.rgba());
        }
        let clear = start.elapsed();
        // Exercise nonuniform channels, not merely a repeated clear colour.
        for (i, pixel) in canvas.rgba.chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(&[i as u8, (i >> 8) as u8, (i >> 16) as u8, 255]);
        }
        let start = Instant::now();
        for _ in 0..frames {
            black_box(&canvas).copy_bgra(black_box(&mut dib)).unwrap();
            black_box(dib.as_slice());
        }
        let convert = start.elapsed();
        for (src, dst) in canvas.rgba.chunks_exact(4).zip(dib.chunks_exact(4)) {
            assert_eq!(dst, [src[2], src[1], src[0], 255]);
        }
        eprintln!("800x600 CPU / {frames} frames: clear={clear:?}, DIB conversion={convert:?}");
    }

    #[test]
    fn multiply_matches_native_positive_magic_division_for_every_channel_pair() {
        let mut canvas = Canvas::new(1, 1).unwrap();
        for s in 0u32..32 {
            for d in 0u32..32 {
                // Independent native integer sequence (imul84210843/sar4/sign
                // correction), not a copy of the implementation's `/31` formula.
                let product = (s * d) as i32;
                let high = ((i64::from(product) * i64::from(0x84210843u32 as i32)) >> 32) as i32;
                let q = high.wrapping_add(product) >> 4;
                let expected = (q + ((q as u32 >> 31) as i32)) as u8;
                let s8 = ((s << 3) | (s >> 2)) as u8;
                let d8 = ((d << 3) | (d >> 2)) as u8;
                canvas.clear([d8; 3]);
                canvas
                    .blit_multiply_rgb555(&[s8, s8, s8, 255], 1, 1, 0, 0)
                    .unwrap();
                assert_eq!(
                    &canvas.rgba()[..3],
                    &[((expected << 3) | (expected >> 2)); 3]
                );
                assert_eq!(canvas.rgba()[3], 255);
            }
        }
    }
    #[test]
    fn multiply_preserves_uncovered_pixels_and_rejects_atomically_before_clipping() {
        let mut canvas = Canvas::new(2, 1).unwrap();
        canvas.clear([81, 125, 233]);
        canvas
            .blit_multiply_rgb555(&[0, 0, 0, 0, 0, 0, 0, 255], 2, 1, 0, 0)
            .unwrap();
        assert_eq!(canvas.rgba(), &[81, 125, 233, 255, 0, 0, 0, 255]);
        canvas.clear([81, 125, 233]);
        canvas
            .blit_multiply_rgb555(&[0, 0, 0, 255, 255, 255, 255, 255], 2, 1, -1, 0)
            .unwrap();
        assert_eq!(canvas.rgba(), &[82, 123, 239, 255, 81, 125, 233, 255]);
        let unchanged = canvas.rgba().to_vec();
        for x in [i32::MIN, i32::MAX] {
            canvas
                .blit_multiply_rgb555(&[0, 0, 0, 255], 1, 1, x, x)
                .unwrap();
        }
        assert_eq!(canvas.rgba(), unchanged);
        for alpha in [1, 128, 254] {
            assert!(canvas
                .blit_multiply_rgb555(&[0, 0, 0, 255, 255, 255, 255, alpha], 2, 1, 0, 0)
                .is_err());
            assert!(canvas
                .blit_multiply_rgb555(&[0, 0, 0, alpha], 1, 1, i32::MAX, 0)
                .is_err());
            assert_eq!(canvas.rgba(), unchanged);
        }
        assert!(canvas
            .blit_multiply_rgb555(&[], u32::MAX, u32::MAX, 0, 0)
            .is_err());
        assert!(canvas.blit_multiply_rgb555(&[0; 3], 1, 1, 0, 0).is_err());
        canvas.blit_multiply_rgb555(&[], 0, 0, 0, 0).unwrap();
    }

    #[test]
    fn roof_rgb555_weight_table_covers_every_channel_and_preserves_coverage() {
        use super::super::world_roofs::Shade;
        let mut source = [0u8; 2304];
        let mut c = Canvas::new(1, 1).unwrap();
        for d in 0u16..32 {
            for s in 0u16..32 {
                let s8 = ((s << 3) | (s >> 2)) as u8;
                source[..4].copy_from_slice(&[s8, s8, s8, 255]);
                let d8 = ((d << 3) | (d >> 2)) as u8;
                for w in 0u16..32 {
                    c.clear([d8; 3]);
                    c.blit_roof_rgb555(&source, 0, 0, Shade::Weight(w as u8))
                        .unwrap();
                    let expected = (d * (31 - w) + s * w) / 31;
                    assert_eq!(c.rgba()[0], ((expected << 3) | (expected >> 2)) as u8);
                }
                for coverage in [128, 255] {
                    source[3] = coverage;
                    c.clear([d8; 3]);
                    c.blit_roof_rgb555(&source, 0, 0, Shade::Half).unwrap();
                    let expected = (d + s) / 2;
                    assert_eq!(c.rgba()[0], ((expected << 3) | (expected >> 2)) as u8);
                }
            }
        }
        source.fill(0);
        c.clear([7, 19, 29]);
        let unchanged = c.rgba().to_vec();
        c.blit_roof_rgb555(&source, 0, 0, Shade::Half).unwrap();
        assert_eq!(c.rgba(), unchanged);
        source[..4].copy_from_slice(&[0, 0, 0, 255]);
        c.blit_roof_rgb555(&source, i32::MIN, i32::MAX, Shade::Half)
            .unwrap();
        assert_eq!(c.rgba(), unchanged);
        assert!(c
            .blit_roof_rgb555(&source, 0, 0, Shade::Weight(32))
            .is_err());
        assert_eq!(c.rgba(), unchanged);
        source[2303] = 127;
        assert!(c.blit_roof_rgb555(&source, 0, 0, Shade::Half).is_err());
        assert_eq!(c.rgba(), unchanged);
    }

    #[test]
    fn prepared_rows_copy_solid_runs_but_keep_internal_holes_and_empty_rows() {
        assert_eq!(FragmentRows::BYTES, 72);
        let mut source = vec![0u8; 2304];
        for y in 3..21 {
            for x in y % 8..23 {
                if y % 2 == 0 || x != 12 {
                    source[(y * 24 + x) * 4..(y * 24 + x) * 4 + 4]
                        .copy_from_slice(&[81, 127, 255, 255]);
                }
            }
        }
        let rows = FragmentRows::new(&source).unwrap();
        for (x, y) in [(-23, -23), (-1, -1), (0, 0), (9, 11), (30, 28)] {
            let mut expected = Canvas::new(31, 29).unwrap();
            expected.clear([9, 27, 38]);
            let mut actual = Canvas::new(31, 29).unwrap();
            actual.clear([9, 27, 38]);
            let mut owners = vec![0u16; 31 * 29];
            expected.blit_rgba(&source, 24, 24, x, y).unwrap();
            actual
                .blit_prepared_world_fragment(&source, &rows, x, y, &mut owners, 7)
                .unwrap();
            assert_eq!(actual.rgba(), expected.rgba());
            for (pixel, owner) in expected.rgba().chunks_exact(4).zip(owners) {
                assert_eq!(owner, if pixel[0] == 81 { 7 } else { 0 });
            }
        }
    }

    #[test]
    fn single_scan_rows_match_scalar_bounds_and_opacity_for_all_alpha_patterns() {
        for pattern in 0..625usize {
            let mut source = [0u8; 2304];
            for y in 0..24 {
                for x in 0..24 {
                    let digit = pattern / 5usize.pow(((x + y) % 4) as u32) % 5;
                    source[(y * 24 + x) * 4 + 3] = [0, 1, 128, 254, 255][digit];
                }
            }
            let prepared = FragmentRows::new(&source).unwrap();
            for (y, row) in prepared.0.iter().enumerate() {
                let alpha: Vec<_> = source[y * 96..y * 96 + 96]
                    .chunks_exact(4)
                    .map(|p| p[3])
                    .collect();
                let start = alpha.iter().position(|&a| a != 0).unwrap_or(24);
                let end = alpha.iter().rposition(|&a| a != 0).map_or(24, |x| x + 1);
                assert_eq!(
                    (row.start as usize, row.end as usize, row.opaque),
                    (start, end, alpha[start..end].iter().all(|&a| a == 255))
                );
            }
        }
        for x in 0..24 {
            for alpha in [1, 128, 255] {
                let mut source = [0; 2304];
                source[x * 4 + 3] = alpha;
                let row = FragmentRows::new(&source).unwrap().0[0];
                assert_eq!(
                    (row.start, row.end, row.opaque),
                    (x as u8, x as u8 + 1, alpha == 255)
                );
            }
        }
        assert!(FragmentRows::new(&[0; 2303]).is_err());
    }

    #[test]
    fn fused_world_fragment_matches_reference_for_every_alpha_and_clipped_edge() {
        let source: Vec<u8> = (0..576)
            .flat_map(|i| {
                [
                    (i % 256) as u8,
                    ((i * 13) % 256) as u8,
                    ((i * 71) % 256) as u8,
                    (i % 256) as u8,
                ]
            })
            .collect();
        for background in [[0, 0, 0], [255, 255, 255], [17, 99, 231]] {
            for (x, y) in [
                (0, 0),
                (-23, -23),
                (-1, 10),
                (17, -1),
                (30, 28),
                (31, 29),
                (i32::MAX, i32::MAX),
                (i32::MIN, i32::MIN),
            ] {
                let mut reference = Canvas::new(31, 29).unwrap();
                reference.clear(background);
                let mut actual = Canvas::new(31, 29).unwrap();
                actual.clear(background);
                let mut owners = vec![12; 31 * 29];
                reference.blit_rgba(&source, 24, 24, x, y).unwrap();
                actual
                    .blit_world_fragment(&source, x, y, &mut owners, 7)
                    .unwrap();
                assert_eq!(actual.rgba(), reference.rgba());
                for dy in 0..29 {
                    for dx in 0..31 {
                        let sx = dx as i64 - i64::from(x);
                        let sy = dy as i64 - i64::from(y);
                        let covered = (0..24).contains(&sx)
                            && (0..24).contains(&sy)
                            && source[(sy * 24 + sx) as usize * 4 + 3] == 255;
                        assert_eq!(owners[dy * 31 + dx], if covered { 7 } else { 12 });
                    }
                }
                let before = actual.rgba().to_vec();
                assert!(actual
                    .blit_world_fragment(&source, x, y, &mut owners[..3], 7)
                    .is_err());
                assert!(actual
                    .blit_world_fragment(&source[..3], x, y, &mut owners, 7)
                    .is_err());
                assert_eq!(before, actual.rgba());
            }
        }
    }

    #[test]
    fn negative_anchor_clips_without_changing_source_stride() {
        let mut canvas = Canvas::new(2, 2).unwrap();
        let pixels = [
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        canvas.blit_rgba(&pixels, 2, 2, -1, -1).unwrap();
        assert_eq!(&canvas.rgba()[..4], &[255, 255, 255, 255]);
        assert_eq!(&canvas.rgba()[4..8], &[0, 0, 0, 255]);
    }

    #[test]
    fn restoring_a_background_is_exact_and_rejects_mismatched_dimensions() {
        let mut background = Canvas::new(2, 3).unwrap();
        background.clear([24, 50, 80]);
        let mut frame = Canvas::new(2, 3).unwrap();
        frame.copy_from(&background).unwrap();
        assert_eq!(frame.rgba(), background.rgba());
        assert!(frame.copy_from(&Canvas::new(3, 2).unwrap()).is_err());
        assert_eq!(frame.rgba(), background.rgba());
    }

    #[test]
    fn alpha_blends_over_opaque_background_and_zero_alpha_preserves_it() {
        let mut canvas = Canvas::new(2, 1).unwrap();
        canvas.clear([0, 100, 200]);
        canvas
            .blit_rgba(&[255, 0, 0, 128, 255, 0, 0, 0], 2, 1, 0, 0)
            .unwrap();
        assert_eq!(canvas.rgba(), &[128, 50, 100, 255, 0, 100, 200, 255]);
        let mut bgra = Vec::new();
        canvas.copy_bgra(&mut bgra).unwrap();
        assert_eq!(bgra, [100, 50, 128, 255, 200, 100, 0, 255]);
    }

    #[test]
    fn extreme_anchors_are_clipped_and_invalid_buffers_rejected() {
        let mut canvas = Canvas::new(1, 1).unwrap();
        for coordinate in [i32::MIN, i32::MAX] {
            canvas
                .blit_rgba(&[255; 4], 1, 1, coordinate, coordinate)
                .unwrap();
        }
        assert_eq!(canvas.rgba(), &[0, 0, 0, 255]);
        assert!(canvas.blit_rgba(&[0; 3], 1, 1, 0, 0).is_err());
        assert!(canvas.blit_rgba(&[], u32::MAX, u32::MAX, 0, 0).is_err());
        assert!(Canvas::new(0, 1).is_err());
        assert!(Canvas::new(u32::MAX, 2).is_err());
    }

    #[test]
    fn native_shadow_averages_five_bit_source_and_destination_not_opaque_gray() {
        let mut canvas = Canvas::new(3, 1).unwrap();
        canvas.clear([255, 255, 255]);
        canvas
            .blit_native_shadow_rgb555(
                &[0, 0, 0, 255, 255, 255, 255, 255, 16, 16, 16, 255],
                3,
                1,
                0,
                0,
            )
            .unwrap();
        assert_eq!(
            canvas.rgba(),
            &[123, 123, 123, 255, 255, 255, 255, 255, 132, 132, 132, 255]
        );
        canvas.clear([200, 100, 50]);
        canvas
            .blit_native_shadow_rgb555(&[16, 33, 57, 255], 1, 1, 0, 0)
            .unwrap();
        assert_eq!(&canvas.rgba()[..4], &[107, 66, 49, 255]);
        // Ordinary opaque SPR compositing must still preserve its original RGB.
        canvas.blit_rgba(&[16, 33, 57, 255], 1, 1, 1, 0).unwrap();
        assert_eq!(&canvas.rgba()[4..8], &[16, 33, 57, 255]);
        canvas.clear([0, 0, 0]);
        canvas
            .blit_native_shadow_rgb555(&[255, 255, 255, 255], 1, 1, 0, 0)
            .unwrap();
        assert_eq!(&canvas.rgba()[..4], &[123, 123, 123, 255]);
    }

    #[test]
    fn native_shadow_clipping_and_transparency_preserve_background_and_stride() {
        let mut canvas = Canvas::new(2, 1).unwrap();
        canvas.clear([201, 101, 51]);
        canvas
            .blit_native_shadow_rgb555(
                &[255, 255, 255, 255, 16, 33, 57, 255, 255, 255, 255, 0],
                3,
                1,
                -1,
                0,
            )
            .unwrap();
        assert_eq!(canvas.rgba(), &[107, 66, 49, 255, 201, 101, 51, 255]);
        let before = canvas.rgba().to_vec();
        for coordinate in [i32::MIN, i32::MAX] {
            canvas
                .blit_native_shadow_rgb555(&[255; 4], 1, 1, coordinate, coordinate)
                .unwrap();
        }
        assert_eq!(canvas.rgba(), before);
    }

    #[test]
    fn native_shadow_rejects_partial_alpha_and_bad_buffers_before_any_mutation() {
        let mut canvas = Canvas::new(2, 1).unwrap();
        canvas.clear([201, 101, 51]);
        let before = canvas.rgba().to_vec();
        assert!(canvas
            .blit_native_shadow_rgb555(&[0, 0, 0, 255, 0, 0, 0, 128], 2, 1, 0, 0,)
            .is_err());
        assert_eq!(canvas.rgba(), before);
        assert!(canvas
            .blit_native_shadow_rgb555(&[0, 0, 0, 128], 1, 1, i32::MAX, 0)
            .is_err());
        assert!(canvas
            .blit_native_shadow_rgb555(&[0; 3], 1, 1, 0, 0)
            .is_err());
        assert!(canvas
            .blit_native_shadow_rgb555(&[], u32::MAX, u32::MAX, 0, 0)
            .is_err());
        assert_eq!(canvas.rgba(), before);
    }

    #[test]
    fn screen_all_1024_channel_pairs_match_independent_floor_reference() {
        use sha2::{Digest, Sha256};
        let mut canvas = Canvas::new(1, 1).unwrap();
        let mut table = Vec::with_capacity(1024);
        for destination5 in 0u8..32 {
            for source5 in 0u8..32 {
                let destination = (destination5 << 3) | (destination5 >> 2);
                let source = (source5 << 3) | (source5 >> 2);
                canvas.clear([destination; 3]);
                canvas
                    .blit_screen_rgb555(&[source, source, source, 255], 1, 1, 0, 0)
                    .unwrap();
                let pixel = canvas.rgba();
                assert_eq!(pixel[0], pixel[1]);
                assert_eq!(pixel[1], pixel[2]);
                assert_eq!(pixel[3], 255);
                table.push(pixel[0] >> 3);
            }
        }
        assert_eq!(table.len(), 1024);
        // Independent C# memory oracle, destination outer/source inner:
        // 31 - (((31-d)*(31-s)+30)/31). Not a captured native thunk table.
        assert_eq!(
            format!("{:x}", Sha256::digest(&table)),
            "81c6c2652cceed348c4567d2bc1a5128080e141c6a2aadbd575fe7b89543a854"
        );
    }

    #[test]
    fn screen_black_white_floor_and_other_compositors_remain_distinct() {
        let mut canvas = Canvas::new(1, 1).unwrap();
        canvas.clear([82, 165, 247]); // exact expanded RGB555 (10,20,30)
        canvas
            .blit_screen_rgb555(&[0, 0, 0, 255], 1, 1, 0, 0)
            .unwrap();
        assert_eq!(
            canvas.rgba(),
            &[82, 165, 247, 255],
            "covered black is not an opaque black patch"
        );
        canvas.blit_screen_rgb555(&[255; 4], 1, 1, 0, 0).unwrap();
        assert_eq!(canvas.rgba(), &[255; 4]);
        canvas.clear([8, 123, 82]); // five-bit (1,15,10)
        canvas
            .blit_screen_rgb555(&[8, 123, 165, 255], 1, 1, 0, 0)
            .unwrap();
        assert_eq!(canvas.rgba(), &[8, 181, 189, 255]); // (1,22,23), floor not round-to-nearest
        canvas.clear([0; 3]);
        canvas
            .blit_screen_rgb555(&[16, 33, 57, 255], 1, 1, 0, 0)
            .unwrap();
        assert_eq!(canvas.rgba(), &[16, 33, 57, 255]);
        canvas.clear([255; 3]);
        canvas.blit_rgba(&[0, 0, 0, 255], 1, 1, 0, 0).unwrap();
        assert_eq!(
            canvas.rgba(),
            &[0, 0, 0, 255],
            "ordinary opaque blit is unchanged"
        );
        canvas.clear([255; 3]);
        canvas
            .blit_native_shadow_rgb555(&[0, 0, 0, 255], 1, 1, 0, 0)
            .unwrap();
        assert_eq!(
            canvas.rgba(),
            &[123, 123, 123, 255],
            "shadow still averages, not screen"
        );
    }

    #[test]
    fn screen_clips_without_stride_changes_and_preserves_uncovered_eight_bit_pixels() {
        let mut canvas = Canvas::new(2, 1).unwrap();
        canvas.clear([201, 101, 51]);
        canvas
            .blit_screen_rgb555(
                &[0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 0],
                3,
                1,
                -1,
                0,
            )
            .unwrap();
        assert_eq!(canvas.rgba(), &[255, 255, 255, 255, 201, 101, 51, 255]);
        let before = canvas.rgba().to_vec();
        for coordinate in [i32::MIN, i32::MAX] {
            canvas
                .blit_screen_rgb555(&[255; 4], 1, 1, coordinate, coordinate)
                .unwrap();
        }
        assert_eq!(canvas.rgba(), before);
    }

    #[test]
    fn screen_rejects_partial_coverage_and_invalid_lengths_atomically_even_offscreen() {
        let mut canvas = Canvas::new(2, 1).unwrap();
        canvas.clear([201, 101, 51]);
        let before = canvas.rgba().to_vec();
        assert!(canvas.blit_screen_rgb555(&[255; 7], 2, 1, 0, 0).is_err());
        assert!(canvas
            .blit_screen_rgb555(&[255, 255, 255, 255, 0, 0, 0, 128], 2, 1, 0, 0)
            .is_err());
        assert_eq!(canvas.rgba(), before);
        for alpha in [1, 128, 254] {
            assert!(canvas
                .blit_screen_rgb555(&[0, 0, 0, alpha], 1, 1, i32::MAX, 0)
                .is_err());
        }
        assert!(canvas
            .blit_screen_rgb555(&[], u32::MAX, u32::MAX, 0, 0)
            .is_err());
        assert!(canvas
            .blit_screen_rgb555(&[], MAX_PIXELS as u32 + 1, 1, 0, 0)
            .is_err());
        assert_eq!(canvas.rgba(), before);
    }

    #[test]
    fn viewport_preserves_aspect_and_ignores_desktop_origin() {
        assert_eq!(
            Viewport::fit(1000, 600, 800, 600),
            Some(Viewport {
                x: 100,
                y: 0,
                width: 800,
                height: 600
            })
        );
        assert_eq!(
            Viewport::fit(800, 1000, 800, 600),
            Some(Viewport {
                x: 0,
                y: 200,
                width: 800,
                height: 600
            })
        );
        assert_eq!(Viewport::fit(0, 600, 800, 600), None);
        assert_eq!(
            Viewport::fit(1, 1, 800, 600),
            Some(Viewport {
                x: 0,
                y: 0,
                width: 1,
                height: 1
            })
        );
    }

    /// Produces a reference-render artifact, not a captured native window.
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; writes a PNG under target only"]
    fn real_shared_sprite_composites_to_reference_png() {
        use super::super::{
            resources::{SharedArchive, SharedArchiveKind},
            sprite::decode_classic_spr,
        };
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared resource root");
        let mut archive = SharedArchive::open(root, SharedArchiveKind::Sprite).unwrap();
        let sprite = decode_classic_spr(&archive.read_entry("0-1.spr").unwrap()).unwrap();
        let mut canvas = Canvas::new(640, 240).unwrap();
        canvas.clear([18, 25, 35]);
        assert!(!sprite.frames.is_empty());
        for (index, frame) in sprite.frames.iter().take(4).enumerate() {
            canvas
                .blit_rgba(
                    &frame.rgba,
                    frame.width,
                    frame.height,
                    80 + index as i32 * 160 + frame.x_offset,
                    145 + frame.y_offset,
                )
                .unwrap();
        }
        assert!(canvas
            .rgba()
            .chunks_exact(4)
            .any(|pixel| pixel[..3] != [18, 25, 35]));
        assert!(canvas.rgba().chunks_exact(4).all(|pixel| pixel[3] == 255));
        let output_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/modern-lito-render-tests");
        std::fs::create_dir_all(&output_dir).unwrap();
        let output = output_dir.join("shared-sprite-reference.png");
        let file = std::fs::File::create(&output).unwrap();
        let mut encoder = png::Encoder::new(file, canvas.width(), canvas.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(canvas.rgba())
            .unwrap();
        eprintln!(
            "Reference compositor only (not a window capture): {}",
            output.display()
        );
    }
}
