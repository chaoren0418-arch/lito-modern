//! Conditional CPU inspection of sealed graphics batches5..8.
//! No wire admission, learned-skill owner, pane registry membership or OS input
//! normalization is supplied by these functions. Display RGBA is a reference
//! policy; word destinations follow the qualified fixed-PE routines.
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
#[path = "native_skill_layers.rs"]
pub mod layers;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Rgb555,
    Rgb565,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Namespace {
    LtIcon,
    SharedTbt,
}
impl Format {
    fn shifts(self) -> [u32; 3] {
        match self {
            Self::Rgb555 => [0, 5, 10],
            Self::Rgb565 => [0, 6, 11],
        }
    }
    pub fn convert_raw555(self, v: u16) -> u16 {
        match self {
            Self::Rgb555 => v,
            Self::Rgb565 => v.wrapping_add(v & 0x7fe0),
        }
    }
}
#[derive(Clone, Debug)]
struct Span {
    skip: u8,
    words: Vec<u16>,
}
#[derive(Clone, Debug)]
pub struct Icon {
    namespace: Namespace,
    cached565: bool,
    offset: [i8; 2],
    header_width: u8,
    rows: Vec<Vec<Span>>,
}
impl Icon {
    pub fn decode_raw555(namespace: Namespace, bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= 1 << 20, "icon source budget exceeded");
        let h = bytes.get(..4).context("short icon header")?;
        ensure!(
            (1..=127).contains(&h[3]),
            "qualified positive signed height required"
        );
        let mut at = 4;
        let mut rows = Vec::new();
        for _ in 0..h[3] {
            let count = *bytes.get(at).context("short span count")?;
            at += 1;
            let mut row = Vec::new();
            for _ in 0..count {
                let s = bytes.get(at..at + 2).context("short span header")?;
                at += 2;
                let data = bytes
                    .get(at..at + usize::from(s[1]) * 2)
                    .context("short span words")?;
                at += data.len();
                row.push(Span {
                    skip: s[0],
                    words: data
                        .chunks_exact(2)
                        .map(|p| u16::from_le_bytes([p[0], p[1]]))
                        .collect(),
                });
            }
            rows.push(row);
        }
        ensure!(at == bytes.len(), "icon trailing bytes");
        Ok(Self {
            namespace,
            cached565: false,
            offset: [h[0] as i8, h[1] as i8],
            header_width: h[2],
            rows,
        })
    }
    /// Explicit already-converted source cache. Its provenance/format must be
    /// independently established; this constructor performs no conversion.
    pub fn decode_cached565(namespace: Namespace, bytes: &[u8]) -> Result<Self> {
        let mut icon = Self::decode_raw555(namespace, bytes)?;
        icon.cached565 = true;
        Ok(icon)
    }
    /// Build a separate cache while preserving all original source words.
    pub fn converted_cache565(&self) -> Result<Self> {
        ensure!(!self.cached565, "565 cache cannot be converted twice");
        let mut cache = self.clone();
        for row in &mut cache.rows {
            for span in row {
                for word in &mut span.words {
                    *word = Format::Rgb565.convert_raw555(*word);
                }
            }
        }
        cache.cached565 = true;
        Ok(cache)
    }
    pub fn namespace(&self) -> Namespace {
        self.namespace
    }
    pub fn signed_offset(&self) -> [i8; 2] {
        self.offset
    }
    pub fn header_width(&self) -> u8 {
        self.header_width
    }
    pub fn height(&self) -> usize {
        self.rows.len()
    }
    /// No per-cell clipping. Validate every destination before mutating any byte.
    /// Raw555 source remains immutable;565 conversion happens once per draw.
    pub fn draw_normal(
        &self,
        dest: &mut [u8],
        pitch: i32,
        caller: [i32; 2],
        format: Format,
    ) -> Result<usize> {
        self.draw(dest, pitch, caller, format, None)
    }
    pub fn draw_timed(
        &self,
        dest: &mut [u8],
        pitch: i32,
        caller: [i32; 2],
        tables: &Tables,
        color: u16,
    ) -> Result<usize> {
        self.draw(dest, pitch, caller, tables.format, Some((tables, color)))
    }
    fn draw(
        &self,
        dest: &mut [u8],
        pitch: i32,
        caller: [i32; 2],
        format: Format,
        timed: Option<(&Tables, u16)>,
    ) -> Result<usize> {
        ensure!(
            pitch > 0 && dest.len() <= 16 << 20,
            "positive pitch and bounded destination required"
        );
        ensure!(
            !self.cached565 || format == Format::Rgb565,
            "565 cache requires matching destination format"
        );
        let row_pitch = if timed.is_some() {
            i64::from(pitch)
        } else {
            2 * (i64::from(pitch) / 2)
        };
        let origin = i64::from(caller[1]) * i64::from(pitch)
            + 2 * i64::from(caller[0])
            + i64::from(self.offset[1]) * row_pitch
            + 2 * i64::from(self.offset[0]);
        let mut writes = Vec::new();
        for (y, row) in self.rows.iter().enumerate() {
            let mut at = origin + y as i64 * row_pitch;
            for span in row {
                at += if timed.is_some() {
                    i64::from(span.skip)
                } else {
                    2 * (i64::from(span.skip) / 2)
                };
                for raw in &span.words {
                    ensure!(
                        at >= 0 && at + 2 <= dest.len() as i64,
                        "icon outside bounded destination"
                    );
                    let v = if self.cached565 {
                        *raw
                    } else {
                        format.convert_raw555(*raw)
                    };
                    writes.push((at as usize, timed.map_or(v, |(t, c)| t.timed_word(v, c))));
                    at += 2;
                }
            }
        }
        for (at, v) in &writes {
            dest[*at..*at + 2].copy_from_slice(&v.to_le_bytes());
        }
        Ok(writes.len())
    }
}
/// Explicit complete CPU initialization under one immutable format. This does
/// not prove native641298 or the live initialization call order.
pub struct Tables {
    format: Format,
    timed: [[u16; 1024]; 3],
    selection: [u16; 512],
}
impl Tables {
    pub fn initialized(format: Format) -> Self {
        let shifts = format.shifts();
        let mut timed = [[0u16; 1024]; 3];
        for channel in 0..3 {
            for color in 0..32 {
                for source in 0..32 {
                    timed[channel][color * 32 + source] =
                        ((color + source / 2) as u16) << shifts[channel];
                }
            }
        }
        let mut selection = [0u16; 512];
        let maxg = if format == Format::Rgb565 { 63 } else { 31 };
        for j in 0..256u16 {
            let b = ((2 * (j & 31)) / 3).min(31);
            let gl = ((2 * (j >> 5) + 7) / 3).min(maxg);
            selection[j as usize] = b + (gl << 5);
        }
        for j in 0..256u16 {
            let is565 = format == Format::Rgb565;
            let r = ((2 * ((j >> (if is565 { 3 } else { 2 })) & 31) + 31) / 3).min(31);
            let mut gh =
                (2 * ((j & (if is565 { 7 } else { 3 })) << 3) + (if is565 { 56 } else { 24 })) / 3;
            let gl = selection[j as usize] >> 5;
            if gh + gl > maxg {
                gh = maxg - gl;
            }
            selection[256 + j as usize] = (r << shifts[2]).wrapping_add(gh << 5);
        }
        Self {
            format,
            timed,
            selection,
        }
    }
    pub fn format(&self) -> Format {
        self.format
    }
    pub fn timed_table(&self, channel: usize) -> Option<&[u16; 1024]> {
        self.timed.get(channel)
    }
    pub fn selection_table(&self) -> &[u16; 512] {
        &self.selection
    }
    pub fn timed_word(&self, word: u16, color: u16) -> u16 {
        let s = self.format.shifts();
        let mut out = 0;
        for c in 0..3 {
            let a = usize::from((color >> s[c]) & 31);
            let b = usize::from((word >> s[c]) & 31);
            out |= self.timed[c][32 * a + b];
        }
        out
    }
    pub fn selection_word(&self, word: u16) -> u16 {
        self.selection[usize::from(word & 255)]
            .wrapping_add(self.selection[256 + usize::from(word >> 8)])
    }
    pub fn mark_rectangle(
        &self,
        dest: &mut [u8],
        pitch: i32,
        origin: [i32; 2],
        size: [usize; 2],
    ) -> Result<()> {
        ensure!(
            pitch > 0 && size[0] <= 256 && size[1] <= 256 && dest.len() <= 16 << 20,
            "selection rectangle budget"
        );
        let mut places = Vec::new();
        for y in 0..size[1] {
            for x in 0..size[0] {
                let at = (i64::from(origin[1]) + y as i64) * i64::from(pitch)
                    + 2 * (i64::from(origin[0]) + x as i64);
                ensure!(
                    at >= 0 && at + 2 <= dest.len() as i64,
                    "selection rectangle outside destination"
                );
                places.push(at as usize);
            }
        }
        for at in places {
            let word = u16::from_le_bytes([dest[at], dest[at + 1]]);
            dest[at..at + 2].copy_from_slice(&self.selection_word(word).to_le_bytes());
        }
        Ok(())
    }
}
pub const FRAME934_SHA: &str = "1A369143599258DEDAE971300D611CD6F82061EED0843EC6D385EA37CA9631A7";
pub struct Frame934 {
    raw: Vec<u8>,
}
impl Frame934 {
    pub fn from_qualified_bytes(bytes: Vec<u8>) -> Result<Self> {
        ensure!(
            bytes.len() == 119654 && format!("{:X}", Sha256::digest(&bytes)) == FRAME934_SHA,
            "qualified native LTsurf934 bytes required"
        );
        ensure!(
            bytes[..8] == [207, 0, 33, 1, 1, 0, 76, 50],
            "frame934 exact header"
        );
        Ok(Self { raw: bytes })
    }
    pub fn dimensions(&self) -> [usize; 2] {
        [207, 289]
    }
    /// Bit replication/key12876 is an explicit preview policy, not a qualified
    /// native IMG surface blitter or a substitute for conditional933/935 layers.
    pub fn reference_rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(207 * 289 * 4);
        for p in self.raw[8..].chunks_exact(2) {
            let w = u16::from_le_bytes([p[0], p[1]]);
            if w == 12876 {
                out.extend([0, 0, 0, 0]);
            } else {
                let c = |s: u32| {
                    let n = ((w >> s) & 31u16) as u8;
                    (n << 3) | (n >> 2)
                };
                out.extend([c(10), c(5), c(0), 255]);
            }
        }
        out
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub index: usize,
    pub caller: [i32; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub child_origin: [i32; 2],
    pub owner_input_rect: [i32; 4],
    pub draw_translation: [i32; 2],
}
fn coord(v: i64) -> Result<i32> {
    Ok(i32::try_from(v).context("pane coordinate overflow")?)
}
impl Geometry {
    pub fn qualified_frame934(
        _frame: &Frame934,
        drag: [i32; 2],
        draw_translation: [i32; 2],
    ) -> Result<Self> {
        let x = coord(433 + i64::from(drag[0]))?;
        let y = drag[1];
        Self::resolved(
            [x, y],
            [
                coord(i64::from(x) + 37)?,
                coord(i64::from(y) + 43)?,
                coord(i64::from(x) + 174)?,
                coord(i64::from(y) + 248)?,
            ],
            draw_translation,
        )
    }
    pub fn resolved(
        child_origin: [i32; 2],
        owner_input_rect: [i32; 4],
        draw_translation: [i32; 2],
    ) -> Result<Self> {
        ensure!(
            i64::from(owner_input_rect[2]) - i64::from(owner_input_rect[0]) == 137
                && i64::from(owner_input_rect[3]) - i64::from(owner_input_rect[1]) == 205,
            "qualified owner137x205 rect required"
        );
        Ok(Self {
            child_origin,
            owner_input_rect,
            draw_translation,
        })
    }
    pub fn placements(&self, count: usize, start: usize) -> Result<Vec<Placement>> {
        ensure!(count <= 256 && start <= 256, "skill slot capacity256");
        let mut out = Vec::new();
        for i in start..count.min(start + 24) {
            let k = i - start;
            out.push(Placement {
                index: i,
                caller: [
                    coord(
                        i64::from(self.owner_input_rect[0])
                            + i64::from(self.draw_translation[0])
                            + 34 * (k % 4) as i64,
                    )?,
                    coord(
                        i64::from(self.owner_input_rect[1])
                            + i64::from(self.draw_translation[1])
                            + 34 * (k / 4) as i64,
                    )?,
                ],
            });
        }
        Ok(out)
    }
    /// Code-derived raw helper result, not click authority. Bottom input pixel
    /// can yield row6 despite only6 rendered rows; draw translation is not
    /// subtracted here. Full OS normalization remains unqualified.
    pub fn native_hit_index(
        &self,
        point: [i32; 2],
        count: usize,
        start: usize,
    ) -> Result<Option<usize>> {
        ensure!(count <= 256 && start <= 256, "skill slot capacity256");
        let [l, t, r, b] = self.owner_input_rect;
        if point[0] < l || point[0] >= r || point[1] < t || point[1] >= b {
            return Ok(None);
        }
        let col = (i64::from(point[0]) - i64::from(l)) / 34;
        let row = (i64::from(point[1]) - i64::from(t)) / 34;
        let index = start + 4 * row as usize + col as usize;
        Ok((col < 4 && row <= 8 && index < count).then_some(index))
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Scroll {
    Up4,
    Down4,
    PageUp36,
    PageDown36,
    DirectRow(usize),
    Slider(i32),
}
pub fn scroll_start(start: usize, count: usize, action: Scroll) -> Result<usize> {
    ensure!(start <= 256 && count <= 256, "skill slot capacity256");
    Ok(match action {
        Scroll::Up4 => start.checked_sub(4).unwrap_or(start),
        Scroll::Down4 => {
            if start + 4 < count {
                start + 4
            } else {
                start
            }
        }
        Scroll::PageUp36 => start.saturating_sub(36) / 4 * 4,
        Scroll::PageDown36 => {
            if count == 0 {
                0
            } else {
                (start + 36).min(count - 1) / 4 * 4
            }
        }
        Scroll::DirectRow(row) => {
            ensure!(row <= 64, "direct row capacity");
            row * 4
        }
        Scroll::Slider(p) => {
            ensure!(count > 1, "slider contract requires count>1");
            4 * ((count - 1) / 4 * (p.clamp(36, 231) - 36) as usize / 195)
        }
    })
}
pub const REGISTRY_DRAW_ORDER: [usize; 4] = [3, 2, 1, 0];
pub const REGISTRY_INPUT_ORDER: [usize; 4] = [0, 1, 2, 3];
/// None represents a placeholder consuming a position. Selectionmark and
/// timedbranch are explicit inspection inputs, never availability inference.
pub struct Slot<'a> {
    pub icon: Option<&'a Icon>,
    pub timed: bool,
    pub marked: bool,
}
pub fn render_grid_words(
    dest: &mut [u8],
    pitch: i32,
    geometry: Geometry,
    slots: &[Slot<'_>],
    start: usize,
    tables: &Tables,
) -> Result<Vec<Placement>> {
    ensure!(dest.len() <= 16 << 20, "grid destination budget");
    let places = geometry.placements(slots.len(), start)?;
    let mut prepared = dest.to_vec();
    for p in &places {
        let slot = &slots[p.index];
        if let Some(icon) = slot.icon {
            if slot.timed {
                icon.draw_timed(&mut prepared, pitch, p.caller, tables, 0)?;
            } else {
                icon.draw_normal(&mut prepared, pitch, p.caller, tables.format())?;
            }
        }
        if slot.marked {
            tables.mark_rectangle(
                &mut prepared,
                pitch,
                [
                    coord(i64::from(p.caller[0]) - 1)?,
                    coord(i64::from(p.caller[1]) - 1)?,
                ],
                [34, 34],
            )?;
        }
    }
    dest.copy_from_slice(&prepared);
    Ok(places)
}
