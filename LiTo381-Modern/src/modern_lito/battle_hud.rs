//! Source-backed HP/MP widget from the shared Lineage Sprite shards.
//! The TW13081901 live UI trace places 2355/2365/2358 at these 800x600
//! anchors and crops each bar from its left edge by current/maximum.
use super::{compositor::Canvas, resources::{SharedArchive, SharedArchiveKind}};
use anyhow::{ensure, Result};
use std::path::Path;

const BASE: (u8, &str, u16, u16, i32, i32) = (13, "2355c.img", 378, 48, 268, 447);
const HP: (u8, &str, u16, u16, i32, i32) = (14, "2365c.img", 141, 20, 277, 458);
const MP: (u8, &str, u16, u16, i32, i32) = (0, "2358c.img", 136, 20, 451, 458);
/// Source-image bounds on the 800×600 world surface. The native text overlay
/// maps these exact rectangles through the same viewport as the shared art.
pub(crate) const HP_BAR: (i32, i32, i32, i32) = (HP.4, HP.5, HP.2 as i32, HP.3 as i32);
pub(crate) const MP_BAR: (i32, i32, i32, i32) = (MP.4, MP.5, MP.2 as i32, MP.3 as i32);

pub(crate) fn numbers(hp: i64, max_hp: i64, mp: i64, max_mp: i64) -> Result<[String; 2]> {
    ensure!(super::equipment_transition::Vitals { hp, maximum_hp: max_hp, mp, maximum_mp: max_mp }.valid(), "canonical HUD vitals out of range");
    Ok([format!("{hp}/{max_hp}"), format!("{mp}/{max_mp}")])
}

#[derive(Clone)]
struct Picture { width: u32, height: u32, rgba: Vec<u8> }

fn decode(bytes: &[u8], expected: (u16, u16)) -> Result<Picture> {
    ensure!(bytes.len() >= 8, "shared HUD IMG header truncated");
    let width = u16::from_le_bytes([bytes[0], bytes[1]]);
    let height = u16::from_le_bytes([bytes[2], bytes[3]]);
    let masked = u16::from_le_bytes([bytes[4], bytes[5]]);
    let key = u16::from_le_bytes([bytes[6], bytes[7]]);
    ensure!((width, height) == expected && masked == 1, "shared HUD IMG shape/mask mismatch");
    let pixels = usize::from(width) * usize::from(height);
    ensure!(bytes.len() == 8 + pixels * 2, "shared HUD IMG payload length mismatch");
    let mut rgba = Vec::with_capacity(pixels * 4);
    for pair in bytes[8..].chunks_exact(2) {
        let color = u16::from_le_bytes([pair[0], pair[1]]);
        if color == key {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
            continue;
        }
        let component = |shift| {
            let v = ((color >> shift) & 31u16) as u8;
            (v << 3) | (v >> 2)
        };
        rgba.extend_from_slice(&[component(10), component(5), component(0), 255]);
    }
    Ok(Picture { width: width.into(), height: height.into(), rgba })
}

fn load(root: &Path, spec: (u8, &str, u16, u16, i32, i32)) -> Result<Picture> {
    let mut archive = SharedArchive::open(root, SharedArchiveKind::SpriteShard(spec.0))?;
    decode(&archive.read_entry(spec.1)?, (spec.2, spec.3))
}

pub(crate) struct Assets { base: Picture, hp: Picture, mp: Picture }
impl Assets {
    /// Production combat scenes require this exact shared HUD. A missing IDX,
    /// PAK or member, or a malformed member, fails the resource qualification.
    pub(crate) fn load(root: &Path) -> Result<Self> {
        Ok(Self { base: load(root, BASE)?, hp: load(root, HP)?, mp: load(root, MP)? })
    }

    /// Only unit/integration fixture roots predating the shared HUD may omit
    /// all three archive pairs. This symbol does not exist in the app build.
    #[cfg(test)]
    pub(crate) fn load_fixture_optional(root: &Path) -> Result<Option<Self>> {
        let present = [BASE, HP, MP].map(|spec| {
            let (idx, pak) = SharedArchiveKind::SpriteShard(spec.0).file_names()?;
            Ok::<_, anyhow::Error>((root.join(idx).exists(), root.join(pak).exists()))
        }).into_iter().collect::<Result<Vec<_>>>()?;
        if present.iter().all(|&(idx, pak)| !idx && !pak) { return Ok(None); }
        Ok(Some(Self::load(root)?))
    }

    pub(crate) fn draw(&self, canvas: &mut Canvas, hp: i64, max_hp: i64, mp: i64, max_mp: i64) -> Result<()> {
        ensure!(super::equipment_transition::Vitals { hp, maximum_hp: max_hp, mp, maximum_mp: max_mp }.valid(), "canonical HUD vitals out of range");
        let width = |current: i64, maximum: i64, pixels: u32| -> u32 {
            ((i128::from(pixels) * i128::from(current)) / i128::from(maximum)) as u32
        };
        canvas.blit_rgba(&self.base.rgba, self.base.width, self.base.height, BASE.4, BASE.5)?;
        canvas.blit_rgba_left(&self.hp.rgba, self.hp.width, self.hp.height, width(hp, max_hp, self.hp.width), HP.4, HP.5)?;
        let mp_width = if max_mp == 0 { 0 } else { width(mp, max_mp, self.mp.width) };
        canvas.blit_rgba_left(&self.mp.rgba, self.mp.width, self.mp.height, mp_width, MP.4, MP.5)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};

    struct Fixture { root: PathBuf, parent: PathBuf }
    impl Fixture {
        fn new() -> Self {
            static SERIAL: AtomicU64 = AtomicU64::new(0);
            let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target").join("modern-lito-hud-tests");
            fs::create_dir_all(&parent).unwrap();
            let parent = fs::canonicalize(parent).unwrap();
            loop {
                let root = parent.join(format!("fixture-{}-{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
                match fs::create_dir(&root) {
                    Ok(()) => return Self { root, parent },
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(e) => panic!("isolated HUD fixture: {e}"),
                }
            }
        }
        fn member(&self, spec: (u8, &str, u16, u16, i32, i32), color: u16) {
            let stem = format!("Sprite{:02}", spec.0);
            let mut index = 1u32.to_le_bytes().to_vec();
            let mut record = [0u8; 28];
            record[4..4 + spec.1.len()].copy_from_slice(spec.1.as_bytes());
            let mut image = Vec::new();
            image.extend_from_slice(&spec.2.to_le_bytes());
            image.extend_from_slice(&spec.3.to_le_bytes());
            image.extend_from_slice(&1u16.to_le_bytes());
            image.extend_from_slice(&0x7c1fu16.to_le_bytes());
            for _ in 0..usize::from(spec.2) * usize::from(spec.3) {
                image.extend_from_slice(&color.to_le_bytes());
            }
            record[24..28].copy_from_slice(&(image.len() as u32).to_le_bytes());
            index.extend_from_slice(&record);
            fs::write(self.root.join(format!("{stem}.idx")), index).unwrap();
            fs::write(self.root.join(format!("{stem}.pak")), image).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            if self.root.parent() == Some(self.parent.as_path())
                && self.root.file_name().is_some_and(|n| n.to_string_lossy().starts_with("fixture-"))
                && fs::symlink_metadata(&self.root).is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink()) {
                fs::remove_dir_all(&self.root).unwrap();
            }
        }
    }
    #[test]
    fn img_rgb555_key_and_bounds() {
        let bytes = [2,0,1,0,1,0,0x1f,0x7c, 0x1f,0x7c, 0,0x7c];
        let p = decode(&bytes, (2,1)).unwrap();
        assert_eq!(p.rgba, [0,0,0,0, 255,0,0,255]);
        assert!(decode(&bytes[..11], (2,1)).is_err());
        assert!(decode(&bytes, (3,1)).is_err());
    }
    #[test]
    fn numbers_keep_exact_wide_owner_values_and_reject_invalid_vitals() {
        assert_eq!(numbers(7, 100, 0, 0).unwrap(), ["7/100", "0/0"]);
        assert_eq!(numbers(i64::MAX, i64::MAX, 5_000_000_000, i64::MAX).unwrap(),
            ["9223372036854775807/9223372036854775807", "5000000000/9223372036854775807"]);
        for values in [(7,100,1,0), (7,100,-1,0), (-1,100,0,0), (101,100,0,0), (7,0,0,0)] {
            assert!(numbers(values.0, values.1, values.2, values.3).is_err());
        }
        assert_eq!(HP_BAR, (277, 458, 141, 20));
        assert_eq!(MP_BAR, (451, 458, 136, 20));
    }
    #[test]
    fn required_assets_reject_absent_partial_and_corrupt_members() {
        let f = Fixture::new();
        assert!(Assets::load(&f.root).is_err(), "production loader cannot silently skip all HUD assets");
        assert!(Assets::load_fixture_optional(&f.root).unwrap().is_none(), "only cfg(test) fixture API may skip an empty root");
        f.member(BASE, 0x7c00);
        assert!(Assets::load(&f.root).is_err(), "partial HUD must fail");
        assert!(Assets::load_fixture_optional(&f.root).is_err(), "partial fixture HUD must fail too");
        f.member(HP, 0x03e0);
        f.member(MP, 0x001f);
        assert!(Assets::load(&f.root).is_ok());
        assert!(Assets::load_fixture_optional(&f.root).unwrap().is_some());
        fs::write(f.root.join("Sprite00.pak"), [0u8; 8]).unwrap();
        assert!(Assets::load(&f.root).is_err(), "truncated PAK/member cannot qualify HUD");
    }
    #[test]
    fn valid_zero_maximum_mp_draws_empty_gauge_and_invalid_values_fail() {
        let f = Fixture::new();
        f.member(BASE, 0x7c00);
        f.member(HP, 0x03e0);
        f.member(MP, 0x001f);
        let assets = Assets::load(&f.root).unwrap();
        let mut empty = Canvas::new(800, 600).unwrap();
        assets.draw(&mut empty, 7, 100, 0, 0).unwrap();
        let at = (460usize * 800 + 480) * 4;
        assert_eq!(&empty.rgba()[at..at + 4], &[255, 0, 0, 255], "empty MP shows the shared base");
        let mut full = Canvas::new(800, 600).unwrap();
        assets.draw(&mut full, 7, 100, 10, 10).unwrap();
        assert_eq!(&full.rgba()[at..at + 4], &[0, 0, 255, 255]);
        for values in [(7,100,1,0), (7,100,-1,0), (7,100,0,-1), (-1,100,0,0), (101,100,0,0), (7,0,0,0)] {
            assert!(assets.draw(&mut empty, values.0, values.1, values.2, values.3).is_err(), "invalid {values:?}");
        }
    }
    #[test]
    #[ignore = "requires explicit LITO_SHARED_RESOURCE_ROOT; read-only actual Sprite13/14/00 members"]
    fn real_shared_hud_members_qualify_and_zero_mp_draws() {
        let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"));
        let assets = Assets::load(&root).unwrap();
        assert_eq!((assets.base.width, assets.base.height), (378, 48));
        assert_eq!((assets.hp.width, assets.hp.height), (141, 20));
        assert_eq!((assets.mp.width, assets.mp.height), (136, 20));
        let mut canvas = Canvas::new(800, 600).unwrap();
        assets.draw(&mut canvas, 7, 100, 0, 0).unwrap();
    }
}
