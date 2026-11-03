//! Candidate canonical call-lightning11736 and Sound34. Not registered in product.
//! NOT server spell permission. call lightning_effect owns the qualified one-shot
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

const IDS: [u32; 1] = [11736];
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
pub struct CallLightningAssets {
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
        "call lightning resource loading cancelled"
    );
    Ok(())
}
impl CallLightningAssets {
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
        ensure!(records <= MAX_RECORDS, "call lightning index budget");
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
            ensure!(records <= MAX_RECORDS, "call lightning index budget");
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
                "call lightning encoded budget"
            );
            let kind = archive.kind();
            active(cancel)?;
            let bytes = archive.read_entry(&name)?;
            active(cancel)?;
            ensure!(bytes.len() == size, "SPR source length changed");
            let remaining = (MAX_RETAINED - retained_bytes) / 4;
            ensure!(remaining > 0, "call lightning retained budget");
            let decoded = decode_world_spr_with_limits(
                &bytes,
                SpriteLimits {
                    max_input_bytes: MAX_ENTRY,
                    max_frames: 6,
                    max_frame_dimension: 1024,
                    max_total_pixels: remaining,
                    ..Default::default()
                },
            )?;
            active(cancel)?;
            ensure!(decoded.frames.len() == 6, "call lightning SPR frame count");
            ensure!(decoded.frames.iter().all(|f| f.width > 0 && f.height > 0 && !f.rgba.is_empty()), "unexpected call-lightning empty frame");
            let count = decoded
                .retained_rgba_bytes()
                .ok_or_else(|| anyhow::anyhow!("retained overflow"))?;
            ensure!(
                count <= MAX_RETAINED - retained_bytes,
                "call lightning retained budget"
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
        let sound_group = classify_native_cues(&[FrameEvent::Sound(34)])?
            .audio
            .ok_or_else(|| anyhow::anyhow!("call lightning audio group absent"))?;
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
    ensure!(pack==CURRENT_PACK && text==CURRENT_TEXT && decoded==5_974_928 && definitions==1,"unqualified current TW call lightning namespace");
    Ok(())
}

fn qualify(catalog: &SpriteActionCatalog) -> Result<()> {
    ensure!(catalog.sprites.len()==1,"call-lightning selection");
    let sprite=catalog.sprites.get(&11736).ok_or_else(||anyhow::anyhow!("canonical11736 absent"))?;
    ensure!(sprite.graphics_id==11736 && sprite.image_count==Some(1),"call-lightning graphics alias/count");
    let controls:Vec<_>=sprite.attributes.iter().map(|a|(a.key,a.body_bytes.as_slice())).collect();
    ensure!(controls==[(ActionKey::new(104),b"8".as_slice()),(ActionKey::new(109),b"2 -1".as_slice())],"class2 controls");
    let resolved=catalog.resolve_action(11736,ActionKey::new(0))?;let a=resolved.animation;
    ensure!(sprite.actions.len()==1 && resolved.definition_sprite_id==11736 && a.directionality==Directionality::Undirected && a.frames_per_second==24 && a.frames.len()==6,"class2 animation controls");
    for (i,f) in a.frames.iter().enumerate() {
        let expected=match i {1=>vec![FrameEvent::Sound(34)],2=>vec![FrameEvent::Hit],_=>vec![]};
        ensure!(f.image_id==0 && f.subframe==if i==0 {5}else{i as u32-1} && f.duration_ticks==[1,4,3,3,3,3][i] && f.events==expected,"class2 frame/cue contract");
    }
    Ok(())
}
