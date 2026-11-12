//! Native suffix image after a world name (48A1C8 all-name pass, 48A6F6 hover
//! pass). Both read slot 773 of the IMG table at 834BD4 and colour-key copy it
//! (4BA220) at `name x + 432330(name) + 7` on the name row. Which actors get it
//! is decided by `native_world_labels`; this module only loads and draws.
use super::{
    compositor::Canvas,
    native_offline_menu_skin::{decode_img, CopyMode, Img},
    native_world_labels::Line,
    native_world_text::Cache,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
};
use anyhow::{bail, ensure, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub const MEMBER: &str = "773.img";
pub const WIDTH: u32 = 13;
pub const HEIGHT: u32 = 12;
/// Native `lea reg, [extent + x + 7]`.
pub const GAP: i32 = 7;

fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "name badge cancelled");
    Ok(())
}
/// 8-byte IMG header and 13x12 RGB555 with colour key 0, as found in the BASE
/// Sprite archive. Any other shape is rejected instead of being scaled.
pub fn decode(bytes: &[u8]) -> Result<Img> {
    let image = decode_img(bytes, CopyMode::ColorKey)?;
    ensure!(
        image.width == WIDTH && image.height == HEIGHT,
        "qualified name badge dimensions"
    );
    Ok(image)
}
struct Bank {
    image: Img,
    source: Value,
}
/// Only the BASE Sprite member is qualified. A Sprite shard copy would take
/// precedence in the original route, so it is refused rather than ignored.
/// `LiTo/LTsurf` holds no `773*.img` in the surveyed client and is not opened
/// here: pinning that 92 MB archive costs more than the whole label pass.
fn load(root: &Path, cancel: &AtomicBool) -> Result<Bank> {
    active(cancel)?;
    let shard = (MEMBER.bytes().map(u32::from).sum::<u32>() % 16) as u8;
    let overlay = SharedArchive::open_for_names(
        root,
        SharedArchiveKind::SpriteShard(shard),
        &[MEMBER],
        cancel,
    )?;
    match overlay.entry(MEMBER) {
        Err(ResourceError::MissingEntry(_)) => {}
        Ok(_) => bail!("name badge shard override requires qualification"),
        Err(error) => return Err(error.into()),
    }
    let mut base = SharedArchive::open_for_names(root, SharedArchiveKind::Sprite, &[MEMBER], cancel)?;
    let bytes = base.read_entry(MEMBER)?;
    active(cancel)?;
    let image = decode(&bytes)?;
    let source = json!({"archive":"Sprite","member":MEMBER,"bytes":bytes.len(),
        "sha256":format!("{:X}",Sha256::digest(&bytes)),
        "rgba_sha256":format!("{:X}",Sha256::digest(&image.rgba))});
    Ok(Bank { image, source })
}
/// Native `x + extent + 7`; the row is the name row itself.
pub fn position(line: &Line, extent: i32) -> [i32; 2] {
    [line.x.saturating_add(extent).saturating_add(GAP), line.y]
}
#[derive(Default)]
pub struct Runtime {
    bank: Option<std::result::Result<Bank, String>>,
}
impl Runtime {
    /// The archive is opened on the first line that needs the image. A load
    /// failure is kept and reported per row; names are still drawn.
    pub fn draw(
        &mut self,
        root: &Path,
        lines: &[Line],
        text: &mut Cache,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
    ) -> Result<Value> {
        let mut rows = Vec::new();
        for line in lines {
            let Some(measured) = &line.badge else {
                continue;
            };
            active(cancel)?;
            if self.bank.is_none() {
                let loaded = load(root, cancel).map_err(|e| format!("{e:#}"));
                active(cancel)?;
                self.bank = Some(loaded);
            }
            let mut row = json!({"actor":line.id,"kind":line.kind,"drawn":false});
            match self.bank.as_ref().expect("bank assigned above") {
                Ok(bank) => match text.get(measured, cancel).map(|mask| mask.advance) {
                    Ok(extent) => {
                        let [x, y] = position(line, extent);
                        canvas.blit_rgba(&bank.image.rgba, bank.image.width, bank.image.height, x, y)?;
                        row["drawn"] = json!(true);
                        row["x"] = json!(x);
                        row["y"] = json!(y);
                        row["extent"] = json!(extent);
                    }
                    Err(error) => row["error"] = json!(format!("{error:#}")),
                },
                Err(error) => row["error"] = json!(error),
            }
            rows.push(row);
        }
        active(cancel)?;
        Ok(json!({"rows":rows,"member":MEMBER,"gap":GAP,"size":[WIDTH,HEIGHT],
            "source":self.bank.as_ref().and_then(|b|b.as_ref().ok()).map(|b|&b.source),
            "ownership":"click_through","extent":"host_gdi_text_extent_of_full_name"}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn img(width: i16, height: i16, fill: u16) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(width.to_le_bytes());
        bytes.extend(height.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(0u16.to_le_bytes());
        for index in 0..(width as usize * height as usize) {
            let word: u16 = if index == 0 { 0 } else { fill };
            bytes.extend(word.to_le_bytes());
        }
        bytes
    }
    #[test]
    fn qualified_shape_is_13_by_12_with_zero_colour_key() {
        let bytes = img(13, 12, 0x7fff);
        assert_eq!(bytes.len(), 320);
        let image = decode(&bytes).unwrap();
        assert_eq!((image.width, image.height), (WIDTH, HEIGHT));
        assert_eq!(&image.rgba[..4], &[0, 0, 0, 0]);
        assert_eq!(&image.rgba[4..8], &[255, 255, 255, 255]);
        assert!(decode(&img(24, 18, 1)).is_err());
        assert!(decode(&bytes[..319]).is_err());
    }
    #[test]
    fn suffix_follows_the_measured_name_by_seven_pixels() {
        let line = Line {
            id: 9,
            kind: "hover",
            text: " Sword".into(),
            x: 100,
            y: 40,
            color: 0x7fff,
            badge: Some("+7 Sword".into()),
        };
        assert_eq!(position(&line, 48), [155, 40]);
        assert_eq!(position(&Line { x: i32::MAX, ..line }, 48), [i32::MAX, 40]);
    }
}
