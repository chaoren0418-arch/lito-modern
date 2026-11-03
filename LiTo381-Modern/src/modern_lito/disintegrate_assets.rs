//! Shared Disintegrate11746/11748, clothes105 layers11747/11749 and Sound717.
//! NOT server spell permission. disintegrate_effect owns the qualified one-shot
//! presentation; its modern server/event/UI integration remains separately gated.
//! No LT package, native process, placeholder graphics or base-catalog merge.
use super::{
    combat_animation::{classify_native_cues, NativeAudioGroup},
    combat_audio::{BankLimits, SoundBank},
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    sprite::{decode_world_spr_with_limits, DecodedSprite, SpriteLimits},
    sprite_actions::{ActionKey, Directionality, FrameEvent, SpriteActionCatalog},
    teleport_effect::SpriteProvenance,
    tw_action_catalog::SharedTwActionCatalog,
};
use anyhow::{ensure, Result};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

const IDS: [u32; 4] = [11746, 11747, 11748, 11749];
const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_ENCODED: usize = 16 * 1024 * 1024;
const MAX_RETAINED: usize = 32 * 1024 * 1024;
const MAX_RECORDS: usize = 1_000_000;

pub struct QualifiedSprite {
    source: SpriteProvenance,
    decoded: DecodedSprite,
}
impl QualifiedSprite {
    pub fn source(&self) -> &SpriteProvenance {
        &self.source
    }
    pub fn decoded(&self) -> &DecodedSprite {
        &self.decoded
    }
}
pub struct DisintegrateAssets {
    catalog: SharedTwActionCatalog,
    sprites: Vec<QualifiedSprite>,
    encoded_bytes: usize,
    retained_bytes: usize,
    sound: Arc<SoundBank>,
    sound_group: NativeAudioGroup,
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "disintegrate resource loading cancelled"
    );
    Ok(())
}
impl DisintegrateAssets {
    /// Blocking all-or-nothing resource-worker operation. The player capability
    /// remains gated separately from presentation and server permission.
    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let catalog = SharedTwActionCatalog::load_selected(root, &IDS.into(), cancel)?;
        Self::from_catalog(root, catalog, cancel)
    }
    /// Explicit current TW asset-data namespace. This never changes the legacy
    /// root-pack entry point, merges base definitions, or grants spell permission.
    pub fn load_current_tw(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let catalog = SharedTwActionCatalog::load_selected(&root.join("launcher"), &IDS.into(), cancel)?;
        require_current_profile(catalog.pack_sha256(), catalog.text_sha256(), catalog.decoded_bytes(), catalog.definitions().len())?;
        Self::from_catalog(root, catalog, cancel)
    }
    fn from_catalog(root: &Path, catalog: SharedTwActionCatalog, cancel: &AtomicBool) -> Result<Self> {
        qualify(catalog.catalog())?;
        active(cancel)?;
        let mut base = SharedArchive::open(root, SharedArchiveKind::Sprite)?;
        let mut records = base.len();
        ensure!(records <= MAX_RECORDS, "disintegrate index budget");
        let mut sprites = Vec::new();
        sprites.try_reserve_exact(IDS.len())?;
        let mut encoded_bytes = 0;
        let mut retained_bytes = 0;
        for id in IDS {
            active(cancel)?;
            let name = format!("{id}-0.spr");
            let shard = (name.bytes().map(u32::from).sum::<u32>() & 15) as u8;
            let mut selected = SharedArchive::open(root, SharedArchiveKind::SpriteShard(shard))?;
            records = records
                .checked_add(selected.len())
                .ok_or_else(|| anyhow::anyhow!("index overflow"))?;
            ensure!(records <= MAX_RECORDS, "disintegrate index budget");
            let archive = match selected.entry(&name) {
                Ok(_) => &mut selected,
                Err(ResourceError::MissingEntry(_)) => {
                    base.entry(&name)?;
                    &mut base
                }
                Err(e) => return Err(e.into()),
            };
            let size = archive.entry(&name)?.size as usize;
            ensure!(
                size > 0 && size <= MAX_ENTRY && size <= MAX_ENCODED - encoded_bytes,
                "disintegrate encoded budget"
            );
            let kind = archive.kind();
            active(cancel)?;
            let bytes = archive.read_entry(&name)?;
            active(cancel)?;
            ensure!(bytes.len() == size, "SPR source length changed");
            let remaining = (MAX_RETAINED - retained_bytes) / 4;
            ensure!(remaining > 0, "disintegrate retained budget");
            let decoded = decode_world_spr_with_limits(
                &bytes,
                SpriteLimits {
                    max_input_bytes: MAX_ENTRY,
                    max_frames: 18,
                    max_frame_dimension: 1024,
                    max_total_pixels: remaining,
                    ..Default::default()
                },
            )?;
            active(cancel)?;
            ensure!(decoded.frames.len() == 18, "disintegrate SPR frame count");
            // The main definition's frame indices also address its clothes105
            // resource. Preserve all18 slots, including the five no-draw slots.
            for (index, f) in decoded.frames.iter().enumerate() {
                let empty = (id == 11747 || id == 11749) && index >= 13;
                ensure!(
                    f.rgba.is_empty() == empty,
                    "unreviewed disintegrate empty frame"
                );
                if empty {
                    ensure!(
                        (f.x_offset, f.y_offset, f.width, f.height, f.unknown1, f.unknown2)
                            == (432, 132, 0, 0, u16::MAX, 0),
                        "unreviewed disintegrate marker"
                    );
                }
            }
            let count = decoded
                .retained_rgba_bytes()
                .ok_or_else(|| anyhow::anyhow!("retained overflow"))?;
            ensure!(
                count <= MAX_RETAINED - retained_bytes,
                "disintegrate retained budget"
            );
            encoded_bytes += size;
            retained_bytes += count;
            sprites.push(QualifiedSprite {
                source: SpriteProvenance {
                    archive: kind,
                    name,
                    bytes: size as u32,
                    sha256: Sha256::digest(&bytes).into(),
                },
                decoded,
            });
        }
        drop(base);
        active(cancel)?;
        let sound_group = classify_native_cues(&[FrameEvent::Sound(717)])?
            .audio
            .ok_or_else(|| anyhow::anyhow!("disintegrate audio group absent"))?;
        let sound = Arc::new(SoundBank::load_shared(
            root,
            std::slice::from_ref(&sound_group),
            BankLimits {
                max_bytes: 1024 * 1024,
                max_sounds: 1,
                max_groups: 1,
            },
            cancel,
        )?);
        active(cancel)?;
        Ok(Self {
            catalog,
            sprites,
            encoded_bytes,
            retained_bytes,
            sound,
            sound_group,
        })
    }
    pub fn catalog(&self) -> &SharedTwActionCatalog {
        &self.catalog
    }
    pub fn sprites(&self) -> &[QualifiedSprite] {
        &self.sprites
    }
    pub fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
    pub fn sound_bank(&self) -> Arc<SoundBank> {
        Arc::clone(&self.sound)
    }
    pub fn sound_group(&self) -> &NativeAudioGroup {
        &self.sound_group
    }
}

const CURRENT_PACK: [u8;32] = [179, 120, 141, 244, 199, 240, 119, 239, 63, 241, 91, 7, 10, 93, 2, 12, 203, 34, 78, 72, 222, 195, 117, 124, 76, 211, 69, 40, 252, 164, 178, 24];
const CURRENT_TEXT: [u8;32] = [126, 190, 43, 70, 76, 139, 30, 169, 197, 138, 201, 133, 221, 143, 244, 200, 35, 120, 77, 130, 49, 137, 66, 98, 237, 220, 215, 105, 28, 169, 64, 8];
fn require_current_profile(pack:[u8;32],text:[u8;32],decoded:usize,definitions:usize)->Result<()> {
    ensure!(pack==CURRENT_PACK && text==CURRENT_TEXT && decoded==5_974_928 && definitions==4,"unqualified current TW disintegrate namespace");
    Ok(())
}

fn qualify(catalog: &SpriteActionCatalog) -> Result<()> {
    ensure!(catalog.sprites.len() == 4, "disintegrate selection");
    for id in IDS {
        let s = catalog
            .sprites
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("disintegrate definition absent"))?;
        ensure!(
            s.graphics_id == id && s.image_count == Some(1),
            "disintegrate graphics alias/count"
        );
        let controls: Vec<_> = s
            .attributes
            .iter()
            .map(|a| (a.key, a.body_bytes.as_slice()))
            .collect();
        if id == 11747 || id == 11749 {
            ensure!(
                s.actions.is_empty()
                    && controls
                        == [
                            (ActionKey::new(102), b"0".as_slice()),
                            (ActionKey::new(104), b"128".as_slice())
                        ],
                "disintegrate layer controls"
            );
            continue;
        }
        let clothes = if id == 11746 { b"1 11747" } else { b"1 11749" };
        ensure!(
            controls
                == [
                    (ActionKey::new(104), b"8".as_slice()),
                    (ActionKey::new(105), clothes.as_slice()),
                    (ActionKey::new(109), b"1 -1".as_slice())
                ],
            "disintegrate main controls"
        );
        let action = catalog.resolve_action(id, ActionKey::new(0))?;
        let a = action.animation;
        ensure!(
            s.actions.len() == 1
                && action.definition_sprite_id == id
                && a.directionality == Directionality::Undirected
                && a.frames_per_second == 24
                && a.frames.len() == 18,
            "disintegrate action controls"
        );
        let ticks = [3, 2, 2, 6, 5, 5, 3, 3, 3, 3, 2, 1, 3, 1, 4, 5, 5, 4];
        for (i, f) in a.frames.iter().enumerate() {
            let events = match i {
                1 => vec![FrameEvent::Sound(717)],
                3 => vec![FrameEvent::Hit],
                _ => vec![],
            };
            ensure!(
                f.image_id == 0
                    && f.subframe == if i == 0 { 17 } else { i as u32 - 1 }
                    && f.duration_ticks == ticks[i]
                    && f.events == events,
                "disintegrate frame/cue contract"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::sprite_actions::ActionLimits;
    use super::*;
    fn definitions() -> Vec<u8> {
        let mut text = String::new();
        for (id, layer) in [(11746, 11747), (11748, 11749)] {
            text+=&format!("#{id} 1 effect\n0.effect(0 18,0.17:3 0.0:2[717 0.1:2 0.2:6! 0.3:5 0.4:5 0.5:3 0.6:3 0.7:3 0.8:3 0.9:2 0.10:1 0.11:3 0.12:1 0.13:4 0.14:5 0.15:5 0.16:4)\n104.attr(8)\n105.clothes(1 {layer})\n109.effect(1 -1)\n#{layer} 1 layer\n102.type(0)\n104.attr(128)\n");
        }
        text.into_bytes()
    }
    #[test]
    fn data_qualification_rejects_changed_semantics_instead_of_guessing() {
        let text = String::from_utf8(definitions()).unwrap();
        let parse = |s: &str| {
            SpriteActionCatalog::parse_for_inspection(s.as_bytes(), ActionLimits::default())
                .unwrap()
        };
        qualify(&parse(&text)).unwrap();
        for (from, to) in [
            ("109.effect(1 -1)", "109.effect(0 -1)"),
            ("104.attr(128)", "104.attr(8)"),
            ("1 11747", "1 11749"),
            ("0.17:3", "0.16:3"),
            ("0.0:2[717", "0.0:2[710"),
            ("0.2:6!", "0.2:6"),
            ("0.3:5", "0.3:4"),
        ] {
            assert!(qualify(&parse(&text.replace(from, to))).is_err(), "{from}");
        }
        assert!(
            DisintegrateAssets::load(Path::new("does-not-exist"), &AtomicBool::new(true))
                .err()
                .unwrap()
                .to_string()
                .contains("cancelled")
        );
    }
    #[test]
    #[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; read-only actual TW/shared SPR qualification, not playback"]
    fn all_four_shared_sprites_match_independent_csharp_pixels() {
        all_four_shared_sprites_match_independent_csharp_pixels_with_profile(crate::modern_lito::disintegrate_assets::DisintegrateAssets::load);
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; explicit current launcher TW profile; original CSharp SPR/pixel/clock assertions retained, no legacy root fallback"]
    fn all_four_shared_sprites_match_independent_csharp_pixels_current_launcher_tw_profile() {
        all_four_shared_sprites_match_independent_csharp_pixels_with_profile(crate::modern_lito::disintegrate_assets::DisintegrateAssets::load_current_tw);
    }

    fn all_four_shared_sprites_match_independent_csharp_pixels_with_profile(profile_loader: fn(&std::path::Path, &std::sync::atomic::AtomicBool) -> anyhow::Result<crate::modern_lito::disintegrate_assets::DisintegrateAssets>) {
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"),
        );
        let assets = profile_loader(&root, &AtomicBool::new(false)).unwrap();
        assert_eq!(assets.encoded_bytes(), 716_815);
        assert!(assets.retained_bytes() <= MAX_RETAINED);
        let golden = [
            (
                3,
                337291,
                "9b9da22312cafb3cc87eb007d1e6b6d2649aa62e46e8fe5776694e408309396a",
                4487328,
                "71a6950c42d4d183448f0a84078c0cb07ff1100177fc3234756db0166b023728",
            ),
            (
                4,
                18936,
                "76301ce94b5e933dae712e1a7940d22adeee068e95e6f80096068c401fe774a7",
                1264032,
                "b9b38c755ba93844f02395b4a9bc0a4f64a0201b09d2f8ddf502f351f0883b4e",
            ),
            (
                5,
                341652,
                "86c27d8c03ef23efbaab3aa46f630c8927403fc7fdfdbacbd0e082ab2cd70a06",
                4523040,
                "a3440107cc9e5aa8cb1b5bb3a4c59e8c1d69dd22e935aecf3e26352f5ac9a185",
            ),
            (
                6,
                18936,
                "76301ce94b5e933dae712e1a7940d22adeee068e95e6f80096068c401fe774a7",
                1264032,
                "b9b38c755ba93844f02395b4a9bc0a4f64a0201b09d2f8ddf502f351f0883b4e",
            ),
        ];
        for (s, (shard, size, source, len, pixels)) in assets.sprites().iter().zip(golden) {
            assert_eq!(s.source.archive, SharedArchiveKind::SpriteShard(shard));
            assert_eq!(s.source.bytes, size);
            assert_eq!(
                s.source
                    .sha256
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>(),
                source
            );
            let mut hash = Sha256::new();
            let mut bytes = 0;
            for f in &s.decoded.frames {
                hash.update(f.x_offset.to_le_bytes());
                hash.update(f.y_offset.to_le_bytes());
                hash.update(f.width.to_le_bytes());
                hash.update(f.height.to_le_bytes());
                hash.update(&f.rgba);
                bytes += 16 + f.rgba.len();
            }
            assert_eq!(bytes, len);
            assert_eq!(format!("{:x}", hash.finalize()), pixels);
        }
        eprintln!("four qualified shared SPRs: {} encoded / {} retained bytes; no playback or skill permission",assets.encoded_bytes(),assets.retained_bytes());
    }
    #[test]
    fn current_tw_namespace_requires_exact_profile_and_cancels_before_io() {
        require_current_profile(CURRENT_PACK,CURRENT_TEXT,5_974_928,4).unwrap();
        for index in [0,15,31] {
            let mut bad=CURRENT_PACK;bad[index]^=1;assert!(require_current_profile(bad,CURRENT_TEXT,5_974_928,4).is_err());
            let mut bad=CURRENT_TEXT;bad[index]^=1;assert!(require_current_profile(CURRENT_PACK,bad,5_974_928,4).is_err());
        }
        for decoded in [0,5_974_927,5_974_929] {assert!(require_current_profile(CURRENT_PACK,CURRENT_TEXT,decoded,4).is_err());}
        for selected in [0,3,5] {assert!(require_current_profile(CURRENT_PACK,CURRENT_TEXT,5_974_928,selected).is_err());}
        assert!(DisintegrateAssets::load_current_tw(Path::new("does-not-exist"),&AtomicBool::new(true)).err().unwrap().to_string().contains("cancelled"));
    }

}
