//! Isolated canonical Cone of Cold11738/11740 with inherited105 layers and Sound715.
//! NOT server spell permission. The product integration must own the eventual
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

const IDS: [u32; 4] = [11738, 11739, 11740, 11741];
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
pub struct ConeColdAssets {
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
        "cone cold resource loading cancelled"
    );
    Ok(())
}
impl ConeColdAssets {
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
        ensure!(records <= MAX_RECORDS, "cone cold index budget");
        let mut sprites = Vec::new();
        sprites.try_reserve_exact(18)?;
        let mut encoded_bytes = 0;
        let mut retained_bytes = 0;
        for id in IDS {
          for image in 0..if id <= 11739 {8} else {1} {
            active(cancel)?;
            let name = format!("{id}-{image}.spr");
            let shard = (name.bytes().map(u32::from).sum::<u32>() & 15) as u8;
            let mut selected = SharedArchive::open(root, SharedArchiveKind::SpriteShard(shard))?;
            records = records
                .checked_add(selected.len())
                .ok_or_else(|| anyhow::anyhow!("index overflow"))?;
            ensure!(records <= MAX_RECORDS, "cone cold index budget");
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
                "cone cold encoded budget"
            );
            let kind = archive.kind();
            active(cancel)?;
            let bytes = archive.read_entry(&name)?;
            active(cancel)?;
            ensure!(bytes.len() == size, "SPR source length changed");
            let remaining = (MAX_RETAINED - retained_bytes) / 4;
            ensure!(remaining > 0, "cone cold retained budget");
            let decoded = decode_world_spr_with_limits(
                &bytes,
                SpriteLimits {
                    max_input_bytes: MAX_ENTRY,
                    max_frames: if id <= 11739 {3} else {8},
                    max_frame_dimension: 1024,
                    max_total_pixels: remaining,
                    ..Default::default()
                },
            )?;
            active(cancel)?;
            ensure!(decoded.frames.len() == if id <= 11739 {3} else {8}, "cone cold SPR frame count");
            // Preserve the reviewed no-draw terminal layer slot. Every other
            // canonical frame must contain pixels; missing layers never degrade.
            for (index, f) in decoded.frames.iter().enumerate() {
                let empty = id == 11741 && index == 7;
                ensure!(f.rgba.is_empty() == empty, "unreviewed cone cold empty frame");
                if empty {
                    ensure!((f.x_offset,f.y_offset,f.width,f.height,f.unknown1,f.unknown2)
                        == (192,120,0,0,u16::MAX,0), "unreviewed cone cold no-draw marker");
                } else {
                    ensure!(f.width > 0 && f.height > 0, "cone cold nonempty frame bounds");
                }
            }
            let count = decoded
                .retained_rgba_bytes()
                .ok_or_else(|| anyhow::anyhow!("retained overflow"))?;
            ensure!(
                count <= MAX_RETAINED - retained_bytes,
                "cone cold retained budget"
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
          }
        drop(base);
        active(cancel)?;
        let sound_group = classify_native_cues(&[FrameEvent::Sound(715)])?
            .audio
            .ok_or_else(|| anyhow::anyhow!("cone cold audio group absent"))?;
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
    pub fn fly_pair(&self, direction: u8) -> Result<(&QualifiedSprite,&QualifiedSprite)> {
        ensure!(direction<8,"cone cold native direction out of range");
        Ok((&self.sprites[usize::from(direction)],&self.sprites[8+usize::from(direction)]))
    }
    pub fn impact_pair(&self) -> (&QualifiedSprite,&QualifiedSprite) {
        (&self.sprites[16],&self.sprites[17])
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
    ensure!(pack==CURRENT_PACK && text==CURRENT_TEXT && decoded==5_974_928 && definitions==4,"unqualified current TW cone cold namespace");
    Ok(())
}

/// Parent action controls address the child SPR directly. The child has no
/// action0 and must never be resolved as a standalone animation.
fn qualify(catalog: &SpriteActionCatalog) -> Result<()> {
    ensure!(catalog.sprites.len()==4,"cone cold exact selection");
    for id in IDS {
        let row=catalog.sprites.get(&id).ok_or_else(||anyhow::anyhow!("cone cold definition absent"))?;
        ensure!(row.graphics_id==id && row.image_count==Some(if id<=11739 {8}else{1}),"cone cold alias/image count");
        let controls:Vec<_>=row.attributes.iter().map(|a|(a.key,a.body_bytes.as_slice())).collect();
        if id==11739 || id==11741 {
            ensure!(row.actions.is_empty() && controls==[
                (ActionKey::new(102),b"0".as_slice()),
                (ActionKey::new(104),b"128".as_slice())],"cone cold inherited layer controls");
            continue;
        }
        let expected=if id==11738 {
            vec![(ActionKey::new(104),b"8".as_slice()),(ActionKey::new(105),b"1 11739".as_slice()),(ActionKey::new(109),b"5 11740".as_slice())]
        } else {
            vec![(ActionKey::new(104),b"8".as_slice()),(ActionKey::new(105),b"1 11741".as_slice())]
        };
        ensure!(controls==expected,"cone cold parent controls");
        let action=catalog.resolve_action(id,ActionKey::new(0))?;
        let a=action.animation;
        ensure!(row.actions.len()==1 && action.definition_sprite_id==id && a.frames_per_second==24
            && a.directionality==if id==11738 {Directionality::EightWay}else{Directionality::Undirected}
            && a.frames.len()==if id==11738 {3}else{8},"cone cold action shape");
        for (i,f) in a.frames.iter().enumerate() {
            let expected_events=if id==11740 && i==1 {vec![FrameEvent::Sound(715),FrameEvent::Hit]}else{vec![]};
            ensure!(f.image_id==0 && f.subframe==if i==0 {if id==11738 {2}else{7}}else{i as u32-1}
                && f.duration_ticks==if id==11738 {2}else{4} && f.events==expected_events,"cone cold frame contract");
        }
    }
    Ok(())
}

