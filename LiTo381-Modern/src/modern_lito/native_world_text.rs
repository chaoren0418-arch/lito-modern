//! CPU label raster for the qualified CP950 native branch. The original uses
//! a 12px, weight400, charset136 system font and four cardinal outline draws.
//! The actual face is recorded because font mapping belongs to the host OS.
use super::compositor::Canvas;
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
};
#[derive(Clone, Debug, serde::Serialize)]
pub struct GlyphMask {
    pub width: u32,
    pub height: u32,
    pub advance: i32,
    pub face: String,
    pub coverage: Vec<u8>,
}
impl GlyphMask {
    pub fn draw(
        &self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        foreground: u16,
        outline: u16,
    ) -> Result<()> {
        let color = |v: u16| {
            [
                ((v >> 10) & 31) as u8 * 8,
                ((v >> 5) & 31) as u8 * 8,
                (v & 31) as u8 * 8,
            ]
        };
        for (dx, dy, c) in [
            (-1, 0, outline),
            (1, 0, outline),
            (0, -1, outline),
            (0, 1, outline),
            (0, 0, foreground),
        ] {
            let rgb = color(c);
            let mut rgba = Vec::with_capacity(self.coverage.len() * 4);
            for a in &self.coverage {
                rgba.extend([rgb[0], rgb[1], rgb[2], *a]);
            }
            let px = x.checked_add(dx).context("text X overflow")?;
            let py = y.checked_add(dy).context("text Y overflow")?;
            canvas.blit_rgba(&rgba, self.width, self.height, px, py)?;
        }
        Ok(())
    }
}
#[derive(Default)]
pub struct Cache {
    entries: BTreeMap<String, GlyphMask>,
    bytes: usize,
}
impl Cache {
    pub fn get(&mut self, text: &str, cancel: &AtomicBool) -> Result<&GlyphMask> {
        ensure!(!cancel.load(Ordering::Acquire), "text raster cancelled");
        ensure!(
            !text.is_empty() && text.len() <= 768 && !text.chars().any(char::is_control),
            "invalid world text"
        );
        super::native_login::encode_text(text, 256)?;
        if !self.entries.contains_key(text) {
            let raster = raster(text)?;
            let cost = raster.coverage.len() + text.len() + raster.face.len();
            ensure!(cost <= 1 << 20, "text cache entry budget");
            if self.entries.len() >= 256 || self.bytes + cost > 4 << 20 {
                self.entries.clear();
                self.bytes = 0;
            }
            ensure!(!cancel.load(Ordering::Acquire), "text raster cancelled");
            self.bytes += cost;
            self.entries.insert(text.into(), raster);
        }
        Ok(&self.entries[text])
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}
#[cfg(windows)]
fn raster(text: &str) -> Result<GlyphMask> {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::{COLORREF, SIZE},
            Graphics::Gdi::*,
        },
    };
    struct Resources {
        dc: HDC,
        font: HFONT,
        bitmap: HBITMAP,
        old_font: HGDIOBJ,
        old_bitmap: HGDIOBJ,
    }
    impl Drop for Resources {
        fn drop(&mut self) {
            unsafe {
                if !self.dc.is_invalid() {
                    if !self.old_bitmap.is_invalid() {
                        SelectObject(self.dc, self.old_bitmap);
                    }
                    if !self.old_font.is_invalid() {
                        SelectObject(self.dc, self.old_font);
                    }
                    let _ = DeleteDC(self.dc);
                }
                if !self.bitmap.is_invalid() {
                    let _ = DeleteObject(self.bitmap.into());
                }
                if !self.font.is_invalid() {
                    let _ = DeleteObject(self.font.into());
                }
            }
        }
    }
    unsafe {
        let mut r = Resources {
            dc: CreateCompatibleDC(None),
            font: HFONT::default(),
            bitmap: HBITMAP::default(),
            old_font: HGDIOBJ::default(),
            old_bitmap: HGDIOBJ::default(),
        };
        ensure!(!r.dc.is_invalid(), "create label DC");
        r.font = CreateFontW(
            12,
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            CHINESEBIG5_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY,
            0,
            PCWSTR::null(),
        );
        ensure!(!r.font.is_invalid(), "create CP950 label font");
        r.old_font = SelectObject(r.dc, r.font.into());
        ensure!(!r.old_font.is_invalid(), "select label font");
        let utf16: Vec<_> = text.encode_utf16().collect();
        ensure!(utf16.len() <= 256, "label glyph budget");
        let mut size = SIZE::default();
        ensure!(
            GetTextExtentPoint32W(r.dc, &utf16, &mut size).as_bool(),
            "measure label"
        );
        ensure!(
            (1..=3072).contains(&size.cx) && (1..=64).contains(&size.cy),
            "label dimensions"
        );
        let width = size.cx as u32 + 2;
        let height = size.cy as u32 + 2;
        let mut info = BITMAPINFO::default();
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let mut ptr = std::ptr::null_mut();
        r.bitmap = CreateDIBSection(Some(r.dc), &info, DIB_RGB_COLORS, &mut ptr, None, 0)?;
        ensure!(!ptr.is_null(), "label DIB memory");
        r.old_bitmap = SelectObject(r.dc, r.bitmap.into());
        ensure!(!r.old_bitmap.is_invalid(), "select label bitmap");
        let pixels =
            std::slice::from_raw_parts_mut(ptr.cast::<u8>(), width as usize * height as usize * 4);
        pixels.fill(0);
        SetBkMode(r.dc, TRANSPARENT);
        SetTextColor(r.dc, COLORREF(0xffffff));
        ensure!(
            TextOutW(r.dc, 0, 0, &utf16).as_bool() && GdiFlush().as_bool(),
            "raster label"
        );
        let mut face = [0u16; 128];
        let count = GetTextFaceW(r.dc, Some(&mut face));
        ensure!(count > 0, "label font identity");
        let face = String::from_utf16_lossy(
            &face[..count as usize]
                .split(|c| *c == 0)
                .next()
                .unwrap_or_default(),
        );
        let coverage = pixels
            .chunks_exact(4)
            .map(|p| p[0].max(p[1]).max(p[2]))
            .collect();
        Ok(GlyphMask {
            width,
            height,
            advance: size.cx,
            face,
            coverage,
        })
    }
}
#[cfg(not(windows))]
fn raster(_text: &str) -> Result<GlyphMask> {
    anyhow::bail!("CP950 native labels require Windows GDI")
}
