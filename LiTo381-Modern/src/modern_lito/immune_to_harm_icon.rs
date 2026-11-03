//! Shared status picture1562, not effect228 or missing proc effect11101.
//! LiTo packetbox40 -> status41 -> table6328D8[41*20] -> 1562.tbt.
//! Modern uses the qualified TBT reference-preview transparency. This is not
//! a claim to reproduce every native blit flag, nor a status/skill grant.
use super::{
    compositor::Canvas,
    inventory_icons::{IconSource, SharedInventoryIcons, TbtImage},
};
use anyhow::{ensure, Context, Result};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub const GRAPHIC: i64 = 1562;
const SIZE: u32 = 32;

/// Caller-owned immutable pixels; archives are dropped before returning.
/// Construct on a resource worker, never on the UI/render thread.
pub struct ImmuneToHarmIcon {
    source: IconSource,
    image: TbtImage,
    rgba: Vec<u8>,
}
impl ImmuneToHarmIcon {
    pub fn load(root: &Path, cancelled: &AtomicBool) -> Result<Self> {
        Self::load_graphic(root, GRAPHIC, cancelled)
    }
    /// The ordinary armor presentation is still restricted to its reviewed
    /// 1562 layout. The optional effect catalog proves effect 41 selects it.
    pub fn load_graphic(root: &Path, graphic: i64, cancelled: &AtomicBool) -> Result<Self> {
        ensure!(
            !cancelled.load(Ordering::Acquire),
            "protection icon cancelled"
        );
        ensure!(graphic == GRAPHIC, "unreviewed protection picture mapping");
        let mut archive = SharedInventoryIcons::open_cancellable(root, cancelled)?;
        let icon = archive.load(graphic, cancelled)?;
        let result = Self::from_image(icon.source().clone(), icon.image().clone())?;
        ensure!(
            !cancelled.load(Ordering::Acquire),
            "protection icon cancelled"
        );
        Ok(result)
    }
    fn from_image(source: IconSource, image: TbtImage) -> Result<Self> {
        ensure!(
            source.name.eq_ignore_ascii_case("1562.tbt")
                && image.raw_offset() == [0, 0]
                && image.header_width() == SIZE as u8
                && image.width() == SIZE as usize
                && image.height() == SIZE as usize,
            "unreviewed protection icon layout"
        );
        let rgba = image.reference_rgba().context("decode protection icon")?;
        ensure!(
            rgba.len() == SIZE as usize * SIZE as usize * 4,
            "invalid protection pixels"
        );
        Ok(Self {
            source,
            image,
            rgba,
        })
    }
    pub fn source(&self) -> &IconSource {
        &self.source
    }
    pub fn image(&self) -> &TbtImage {
        &self.image
    }
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }
    /// Pure drawing at the caller's status-strip origin, not the world/camera
    /// anchor. Visibility and remaining time must come from confirmed status.
    pub fn draw(&self, canvas: &mut Canvas, x: i32, y: i32) -> Result<()> {
        canvas.blit_rgba(&self.rgba, SIZE, SIZE, x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        inventory_icons::decode_tbt,
        resources::{SharedArchive, SharedArchiveKind},
    };
    use super::*;
    use sha2::{Digest, Sha256};
    fn fixture() -> (IconSource, Vec<u8>) {
        let mut bytes = vec![0, 0, 32, 32];
        for _ in 0..32 {
            bytes.extend([1, 0, 32]);
            for x in 0..32 {
                bytes.extend(if x == 0 { 0u16 } else { 0x7c00u16 }.to_le_bytes());
            }
        }
        (
            IconSource {
                archive: SharedArchiveKind::SpriteShard(6),
                name: "1562.tbt".into(),
            },
            bytes,
        )
    }
    fn compare(icon: &ImmuneToHarmIcon, x: i32, y: i32) -> bool {
        let mut canvas = Canvas::new(80, 64).unwrap();
        canvas.clear([11, 22, 33]);
        let original = canvas.rgba().to_vec();
        let mut expected = original.clone();
        // Independent binary-alpha copy reference; no production blitter.
        for sy in 0..32 {
            for sx in 0..32 {
                let dx = i64::from(x) + sx;
                let dy = i64::from(y) + sy;
                if !(0..80).contains(&dx) || !(0..64).contains(&dy) {
                    continue;
                }
                let src = (sy * 32 + sx) as usize * 4;
                let dst = (dy * 80 + dx) as usize * 4;
                assert!(matches!(icon.rgba[src + 3], 0 | 255));
                if icon.rgba[src + 3] == 255 {
                    expected[dst..dst + 3].copy_from_slice(&icon.rgba[src..src + 3]);
                }
            }
        }
        icon.draw(&mut canvas, x, y).unwrap();
        assert_eq!(canvas.rgba(), expected);
        icon.draw(&mut canvas, x, y).unwrap();
        assert_eq!(canvas.rgba(), expected);
        canvas.rgba() != original
    }
    #[test]
    fn qualified_picture_clips_without_wrapping_or_erasing_transparent_pixels() {
        let (source, bytes) = fixture();
        let icon = ImmuneToHarmIcon::from_image(source, decode_tbt(&bytes).unwrap()).unwrap();
        for (x, y) in [(10, 10), (-10, -10), (70, 50)] {
            assert!(compare(&icon, x, y));
        }
        for (x, y) in [(i32::MIN, 0), (i32::MAX, 0), (0, i32::MIN), (0, i32::MAX)] {
            assert!(!compare(&icon, x, y));
        }
    }
    #[test]
    fn offsets_dimensions_and_wrong_picture_are_not_guessed_into_layout() {
        let (source, bytes) = fixture();
        for index in 0..4 {
            let mut bad = bytes.clone();
            bad[index] = if index < 2 { 1 } else { 31 };
            match decode_tbt(&bad) {
                Ok(image) => assert!(ImmuneToHarmIcon::from_image(source.clone(), image).is_err()),
                Err(_) => {}
            }
        }
        let mut other = source;
        other.name = "11101.tbt".into();
        assert!(ImmuneToHarmIcon::from_image(other, decode_tbt(&bytes).unwrap()).is_err());
        assert!(
            ImmuneToHarmIcon::load(Path::new("absent-root"), &AtomicBool::new(true))
                .err()
                .expect("cancelled")
                .to_string()
                .contains("cancelled")
        );
    }
    #[test]
    #[ignore = "explicit shared root; picture1562 independent C# pixels and CPU draw, no HWND"]
    fn actual_shared_protection_status1562_matches_csharp_and_clips() {
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
        );
        assert!(root.is_absolute());
        let icon = ImmuneToHarmIcon::load(&root, &AtomicBool::new(false)).unwrap();
        assert_eq!(icon.source().archive, SharedArchiveKind::SpriteShard(6));
        assert_eq!(icon.source().name, "1562.tbt");
        assert_eq!(
            format!("{:X}", Sha256::digest(icon.rgba())),
            "1897E3069B08D57D7D6E48D21D94B075D1836EA23C29011D7D6CDFD68575C9DE"
        );
        let mut archive = SharedArchive::open(&root, icon.source().archive).unwrap();
        let bytes = archive.read_entry("1562.tbt").unwrap();
        assert_eq!(bytes.len(), 2054);
        assert_eq!(
            format!("{:X}", Sha256::digest(&bytes)),
            "70C842579E29B5D128C2281380743C5F748C33BFA6D1C99B61FD345712DEB1DB"
        );
        for (x, y) in [(10, 10), (-10, -10), (70, 50)] {
            assert!(compare(&icon, x, y));
        }
        for (x, y) in [(i32::MIN, 0), (i32::MAX, 0)] {
            assert!(!compare(&icon, x, y));
        }
        println!("PROTECTION_STATUS1562_OK archive=Sprite06 size=32x32 independentPixels=true liveStatus=false");
    }
}
