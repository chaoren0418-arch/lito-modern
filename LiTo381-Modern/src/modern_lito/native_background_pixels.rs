//! Bounded background RGB555/565 sampling, verified against the original kernels.
//! Numeric Rust only. Archive selection, geometry and scene ordering live outside
//! this primitive; a nonzero terrain gate preserves the existing destination.
use anyhow::{ensure, Context, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Rgb555,
    Rgb565,
}
impl Format {
    fn shifts(self) -> (u32, u32) {
        match self {
            Self::Rgb555 => (10, 5),
            Self::Rgb565 => (11, 6),
        }
    }
    fn valid(self, word: u16) -> bool {
        self == Self::Rgb565 || word & 0x8000 == 0
    }
    fn channels(self, word: u16) -> [u16; 3] {
        let (red, green) = self.shifts();
        [(word >> red) & 31, (word >> green) & 31, word & 31]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blend {
    Screen,
    Multiply,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Strength {
    Full,
    Attenuated(u8),
}
impl Strength {
    fn factor(self) -> Result<Option<u16>> {
        match self {
            Self::Full => Ok(None),
            Self::Attenuated(n) => {
                ensure!(n <= 9, "background attenuation outside native table");
                Ok(Some(15 + u16::from(n) * 8 / 5))
            }
        }
    }
}

fn combine(format: Format, a: u16, b: u16, blend: Blend, factor: Option<u16>) -> u16 {
    let a = format.channels(a);
    let b = format.channels(b);
    let mut rgb = [0; 3];
    for index in 0..3 {
        let value = match blend {
            Blend::Screen => 31 - ((31 - a[index]) * (31 - b[index]) + 30) / 31,
            Blend::Multiply => a[index] * b[index] / 31,
        };
        rgb[index] = factor.map_or(value, |factor| value * factor / 31);
    }
    let (red, green) = format.shifts();
    (rgb[0] << red) | (rgb[1] << green) | rgb[2]
}

/// RGB565 retains the original five-bit green lookup; bit5 is not an input
/// channel bit. Attenuation follows the native intermediate quantization.
pub fn mix_word(format: Format, a: u16, b: u16, blend: Blend, strength: Strength) -> Result<u16> {
    ensure!(format.valid(a) && format.valid(b), "background source format");
    Ok(combine(format, a, b, blend, strength.factor()?))
}

#[derive(Clone, Copy)]
pub struct Image<'a> {
    width: usize,
    height: usize,
    words: &'a [u16],
    format: Format,
}
impl<'a> Image<'a> {
    pub fn new(width: usize, height: usize, words: &'a [u16], format: Format) -> Result<Self> {
        ensure!(
            (1..=1024).contains(&width) && (1..=1024).contains(&height),
            "background image dimensions"
        );
        ensure!(words.len() == width * height, "background image length");
        ensure!(words.iter().all(|&word| format.valid(word)), "background image format");
        Ok(Self { width, height, words, format })
    }
}

/// Immutable decoded storage; validate once when loading, borrow each frame.
pub struct OwnedImage {
    width: usize,
    height: usize,
    words: Vec<u16>,
    format: Format,
}
impl OwnedImage {
    pub fn new(width: usize, height: usize, words: Vec<u16>, format: Format) -> Result<Self> {
        Image::new(width, height, &words, format)?;
        Ok(Self { width, height, words, format })
    }
    pub fn view(&self) -> Image<'_> {
        Image { width:self.width, height:self.height, words:&self.words, format:self.format }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub left: usize,
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
}
pub struct Target<'a> {
    pub words: &'a mut [u16],
    pub pitch: usize,
    pub height: usize,
    pub gate: &'a [u8],
    pub gate_pitch: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Paint {
    pub source: Rect,
    pub destination: [usize; 2],
    pub offsets: [i32; 2],
    pub blend: Blend,
    pub strength: Strength,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stats {
    pub clipped_source_bottom: usize,
    pub written_pixels: usize,
}

/// Validate every image, stride and sample range before changing any pixel.
/// Source top/left and destination clipping are the caller's geometry contract;
/// this kernel additionally clamps the bottom to the destination height.
struct Checked {
    factor: Option<u16>,
    bottom: usize,
    rows: usize,
    width: usize,
    x: usize,
    y: usize,
    overlay_x: usize,
    overlay_y: usize,
}
fn checked(base: Image<'_>, overlay: Image<'_>, call: Paint, target: &Target<'_>) -> Result<Checked> {
    let factor = call.strength.factor()?;
    ensure!(base.format == overlay.format, "background image formats differ");
    ensure!(
        overlay.width == base.width * 2 && overlay.height == base.height * 2,
        "background overlay must contain the doubled source period"
    );
    ensure!(
        (1..=1024).contains(&target.pitch) && (1..=1024).contains(&target.gate_pitch)
            && (1..=1024).contains(&target.height),
        "background target dimensions"
    );
    ensure!(
        target.words.len() == target.pitch * target.height
            && target.gate.len() == target.gate_pitch * target.height,
        "background target plane lengths"
    );
    let source = call.source;
    ensure!(
        source.left < source.right && source.right <= base.width
            && source.top < source.bottom && source.bottom <= base.height,
        "background source rectangle"
    );
    let [x, y] = call.destination;
    let width = source.right - source.left;
    let end_x = x.checked_add(width).context("background destination X overflow")?;
    ensure!(
        end_x <= target.pitch && end_x <= target.gate_pitch && y <= target.height,
        "background destination rectangle"
    );
    let bottom = source.bottom.min(source.top + target.height - y);
    let rows = bottom - source.top;
    let offset_y = if (call.offsets[0] / base.width as i32) % 2 != 0 {
        call.offsets[1]
            .checked_add((base.height / 2) as i32)
            .context("background stagger phase overflow")?
    } else {
        call.offsets[1]
    };
    let overlay_x = base.width as i64 - i64::from(call.offsets[0] % base.width as i32)
        + source.left as i64;
    let overlay_y = base.height as i64 - i64::from(offset_y % base.height as i32)
        + source.top as i64;
    ensure!(
        overlay_x >= 0 && overlay_y >= 0
            && overlay_x + width as i64 <= overlay.width as i64
            && overlay_y + rows as i64 <= overlay.height as i64,
        "background overlay sample range"
    );
    let overlay_x = overlay_x as usize;
    let overlay_y = overlay_y as usize;
    Ok(Checked { factor,bottom,rows,width,x,y,overlay_x,overlay_y })
}

/// A caller composing multiple native rectangles can validate the whole batch
/// before the first destination write.
pub fn validate_paint(base: Image<'_>, overlay: Image<'_>, call: Paint, target: &Target<'_>) -> Result<()> {
    checked(base,overlay,call,target).map(|_| ())
}

pub fn paint(base: Image<'_>, overlay: Image<'_>, call: Paint, target: Target<'_>) -> Result<Stats> {
    let Checked { factor,bottom,rows,width,x,y,overlay_x,overlay_y } = checked(base,overlay,call,&target)?;
    let source=call.source;
    let mut written_pixels = 0;
    for row in 0..rows {
        for column in 0..width {
            if target.gate[(y + row) * target.gate_pitch + x + column] != 0 {
                continue;
            }
            let a = base.words[(source.top + row) * base.width + source.left + column];
            let b = overlay.words[(overlay_y + row) * overlay.width + overlay_x + column];
            target.words[(y + row) * target.pitch + x + column] =
                combine(base.format, a, b, call.blend, factor);
            written_pixels += 1;
        }
    }
    Ok(Stats { clipped_source_bottom: bottom, written_pixels })
}
