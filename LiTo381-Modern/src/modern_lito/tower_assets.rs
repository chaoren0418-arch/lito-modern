//! Qualified shared Honor tower presentation, independent of HP or authority.
//!
//! Native type11 consumer sub_4E45F0 retains states32..35. sub_4E5B50
//! plays source1..N-1 then loads terminal0; sub_4E02C0 requeues state32
//! after terminal0's dwell, whereas33..35 hold0. This is not PC death or
//! an actor action guessed from its label. Resource loading is worker-only.

pub use super::teleport_effect::{SpriteProvenance, VisualSample};
use super::{
    combat_animation::{classify_native_cues, CrossedCue, NativeAudioGroup},
    combat_audio::{BankLimits, SoundBank},
    compositor::Canvas,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    shared_action_catalog::{SharedActionCatalog, SharedCatalogError},
    sprite::{decode_world_spr_with_limits, DecodedSprite, SpriteError, SpriteFrame, SpriteLimits},
    sprite_actions::{
        ActionError, ActionFrame, ActionKey, Animation, Directionality, FrameEvent,
        SpriteActionCatalog,
    },
    teleport_effect::{draw_frame, TeleportError},
    world_map::PixelPoint,
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

pub const GRAPHICS: [u32; 4] = [2665, 2667, 2669, 2671];
pub const ACTIONS: [u8; 5] = [0, 32, 33, 34, 35];
const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_ENCODED: usize = 64 * 1024 * 1024;
const MAX_RGBA: usize = 128 * 1024 * 1024;
const MAX_INDEX_RECORDS: usize = 1_000_000;
const NANOS: u128 = 1_000_000_000;
const LIST_SHA: [u8; 32] = [
    0x52, 0x3c, 0x61, 0xad, 0x5c, 0xc3, 0x62, 0xa5, 0x6a, 0x36, 0xf5, 0x8b, 0x26, 0x2c, 0x84, 0xef,
    0xa8, 0xb4, 0x43, 0x5f, 0x01, 0x7e, 0x70, 0x74, 0xd3, 0x5b, 0x58, 0x34, 0xe8, 0xb9, 0xf1, 0x27,
];

// Exact paired generations; never mix a pack with another generation's text.
// #163 additionally reviewed the current launcher pack's five healing definitions;
// action, attachment, SPR hash and blank-slot qualification below remains mandatory.
const TW_HEALING_GENERATIONS: [([u8; 32], [u8; 32], usize); 2] = [
    ([0xb3, 0x78, 0x8d, 0xf4, 0xc7, 0xf0, 0x77, 0xef, 0x3f, 0xf1, 0x5b, 0x07, 0x0a, 0x5d, 0x02, 0x0c, 0xcb, 0x22, 0x4e, 0x48, 0xde, 0xc3, 0x75, 0x7c, 0x4c, 0xd3, 0x45, 0x28, 0xfc, 0xa4, 0xb2, 0x18],
     [0x7e, 0xbe, 0x2b, 0x46, 0x4c, 0x8b, 0x1e, 0xa9, 0xc5, 0x8a, 0xc9, 0x85, 0xdd, 0x8f, 0xf4, 0xc8, 0x23, 0x78, 0x4d, 0x82, 0x31, 0x89, 0x42, 0x62, 0xed, 0xdc, 0xd7, 0x69, 0x1c, 0xa9, 0x40, 0x08], 5974928),
    ([0x92, 0xb5, 0x12, 0xa3, 0x23, 0x49, 0xd0, 0x77, 0x5e, 0x10, 0x36, 0xf1, 0x83, 0x2e, 0x21, 0xf1, 0x14, 0x1a, 0xe6, 0x6e, 0x64, 0x82, 0x1d, 0x69, 0x78, 0xf0, 0x87, 0x92, 0x46, 0xe5, 0x9c, 0xa7],
     [0x28, 0xc4, 0xfc, 0x60, 0x08, 0x33, 0xfb, 0x8c, 0x60, 0x9a, 0xb1, 0x95, 0xa6, 0x7f, 0xf9, 0x26, 0xe8, 0xff, 0xa5, 0x71, 0x10, 0x0d, 0x8d, 0x4b, 0x38, 0x9a, 0x23, 0xbe, 0x8c, 0x2c, 0x8d, 0xb8], 5990880),
];
fn reviewed_tw_healing_generation(pack: [u8; 32], text: [u8; 32], bytes: usize) -> bool {
    TW_HEALING_GENERATIONS.contains(&(pack, text, bytes))
}

#[derive(Debug)]
pub enum HonorAssetError {
    Cancelled,
    InvalidInput(&'static str),
    Unsupported(&'static str),
    UnsupportedSoundBank,
    MissingFrame { member: String, subframe: u32, frames: usize },
    Bound(&'static str),
    Resource(ResourceError),
    Catalog(ActionError),
    Sprite(SpriteError),
    Render(TeleportError),
}
impl fmt::Display for HonorAssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "Honor resource loading cancelled"),
            Self::InvalidInput(s) => write!(f, "invalid Honor presentation: {s}"),
            Self::Unsupported(s) => write!(f, "unsupported shared Honor presentation: {s}"),
            Self::UnsupportedSoundBank => write!(f, "shared Honor sound bank is unqualified"),
            Self::MissingFrame { member, subframe, frames } => write!(f,
                "SPR {member} requires subframe {subframe}, decoded frame count is {frames}"),
            Self::Bound(s) => write!(f, "Honor presentation exceeds {s} bound"),
            Self::Resource(e) => write!(f, "Honor shared resource: {e}"),
            Self::Catalog(e) => write!(f, "Honor action definition: {e}"),
            Self::Sprite(e) => write!(f, "Honor SPR: {e}"),
            Self::Render(e) => write!(f, "Honor drawing: {e}"),
        }
    }
}
impl std::error::Error for HonorAssetError {}
impl From<SharedCatalogError> for HonorAssetError {
    fn from(error: SharedCatalogError) -> Self {
        match error {
            SharedCatalogError::Cancelled => Self::Cancelled,
            SharedCatalogError::Bound(reason) => Self::Bound(reason),
            SharedCatalogError::Resource(error) => Self::Resource(error),
            SharedCatalogError::Action(error) => Self::Catalog(error),
        }
    }
}
impl From<ResourceError> for HonorAssetError {
    fn from(e: ResourceError) -> Self {
        Self::Resource(e)
    }
}
impl From<ActionError> for HonorAssetError {
    fn from(e: ActionError) -> Self {
        Self::Catalog(e)
    }
}
impl From<SpriteError> for HonorAssetError {
    fn from(e: SpriteError) -> Self {
        Self::Sprite(e)
    }
}
pub(crate) type Result<T> = std::result::Result<T, HonorAssetError>;
pub(crate) fn active(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(HonorAssetError::Cancelled)
    } else {
        Ok(())
    }
}

#[path = "static2941_assets.rs"]
mod static2941;
pub(crate) use static2941::Static2941Bank;

struct LoadedSprite {
    provenance: SpriteProvenance,
    decoded: DecodedSprite,
}
struct SpriteReader {
    root: PathBuf,
    base: SharedArchive,
    shards: BTreeMap<u8, SharedArchive>,
    records: usize,
}
impl SpriteReader {
    fn open(root: &Path, text_records: usize, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let base = SharedArchive::open(root, SharedArchiveKind::Sprite)?;
        let records = text_records
            .checked_add(base.len())
            .ok_or(HonorAssetError::Bound("index records"))?;
        if records > MAX_INDEX_RECORDS {
            return Err(HonorAssetError::Bound("index records"));
        }
        active(cancel)?;
        Ok(Self {
            root: base.root().to_owned(),
            base,
            shards: BTreeMap::new(),
            records,
        })
    }
    fn archive_for(&mut self, name: &str, cancel: &AtomicBool) -> Result<&mut SharedArchive> {
        active(cancel)?;
        let shard = (name.bytes().map(u32::from).sum::<u32>() & 15) as u8;
        if !self.shards.contains_key(&shard) {
            let archive = SharedArchive::open(&self.root, SharedArchiveKind::SpriteShard(shard))?;
            self.records = self
                .records
                .checked_add(archive.len())
                .ok_or(HonorAssetError::Bound("index records"))?;
            if self.records > MAX_INDEX_RECORDS {
                return Err(HonorAssetError::Bound("index records"));
            }
            self.shards.insert(shard, archive);
        }
        active(cancel)?;
        match self.shards[&shard].entry(name) {
            Ok(_) => Ok(self.shards.get_mut(&shard).expect("opened shard")),
            Err(ResourceError::MissingEntry(_)) => {
                self.base.entry(name)?;
                Ok(&mut self.base)
            }
            Err(e) => Err(e.into()),
        }
    }
}

/// Only called by the two explicit, source-reviewed Honor profiles. No public
/// arbitrary-animation loader and no expected-generation bypass is exposed.
pub(crate) struct SharedEffectBank {
    actions: BTreeMap<(u32, u8), Arc<Animation>>,
    graphics_bindings: BTreeMap<u32, u32>,
    sprites: BTreeMap<String, LoadedSprite>,
    sound: Arc<SoundBank>,
    groups: Vec<NativeAudioGroup>,
    catalog_sha: [u8; 32],
    encoded: usize,
    rgba: usize,
}
#[derive(Clone, Copy)]
enum ReferencedFramePolicy { NonEmpty, TwHealingFamily, ReviewedSelectors, ReviewedAdditionalFinite, ReviewedEarthJail, ReceivedFinite, ReviewedDarkElf }
// Each tuple pins the actual SPR and its last visible original subframe.
const TW_HEALING_SPRS: &[(&str,usize,[u8;32],usize)]=&[
    ("11722-0.spr",30637,[0x85, 0xf6, 0x87, 0x20, 0x52, 0xfa, 0x33, 0x5c, 0xab, 0x1e, 0xe5, 0x65, 0xc3, 0xe1, 0x84, 0xe2, 0xd4, 0xe7, 0x18, 0xa6, 0x13, 0x7c, 0x87, 0xd2, 0x88, 0x50, 0xc8, 0x8e, 0xcc, 0xa6, 0xf8, 0xe5],12),
    ("11723-0.spr",63949,[0x15, 0x34, 0xc3, 0x45, 0x69, 0x66, 0x4d, 0x81, 0x6c, 0x25, 0x78, 0xcc, 0x69, 0x2b, 0xe3, 0xfd, 0x66, 0xeb, 0xb1, 0xa3, 0x48, 0x9b, 0x9f, 0x7b, 0x8b, 0x11, 0x87, 0x0a, 0x39, 0xa3, 0x4c, 0xfa],13),
    ("11724-0.spr",98415,[0xfa, 0xef, 0x47, 0xd9, 0xed, 0x96, 0xfe, 0xb9, 0x77, 0x3d, 0x08, 0x83, 0xc9, 0xd4, 0x68, 0x99, 0xf0, 0xa9, 0xff, 0x51, 0x54, 0x68, 0x94, 0xba, 0x3e, 0xa5, 0x77, 0xeb, 0xff, 0x93, 0xaa, 0x40],15),
    ("11725-0.spr",129975,[0x72, 0x65, 0x65, 0x34, 0xc4, 0x80, 0x22, 0x59, 0x26, 0xb4, 0xb7, 0x21, 0xec, 0x13, 0xc0, 0x88, 0xa5, 0xac, 0x88, 0x16, 0xa0, 0xe0, 0xfa, 0x16, 0x4b, 0xac, 0x79, 0x94, 0x69, 0x18, 0x16, 0xd4],17),
    ("11726-0.spr",44329,[0xef, 0xc1, 0x6b, 0xa8, 0xe0, 0x77, 0x83, 0x12, 0xab, 0x47, 0x4a, 0xa2, 0x69, 0x91, 0x5d, 0x3b, 0x6d, 0x35, 0x51, 0xb2, 0x47, 0x5c, 0x02, 0xd8, 0xa3, 0xfa, 0xe2, 0x6f, 0xff, 0xfd, 0xe4, 0xaa],9),
 ];
// Exact original files: native no-block slots keep dwell/audio but submit no pixels.
const REVIEWED_SELECTOR_EMPTY_SPRS:&[(&str,usize,[u8;32],i32)]=&[
    ("3936-0.spr",92615,[68, 72, 119, 197, 27, 186, 103, 148, 161, 102, 61, 246, 47, 44, 21, 217, 54, 224, 215, 130, 51, 144, 15, 18, 133, 22, 48, 60, 19, 122, 124, 244],192),
    ("3934-0.spr",1168552,[151, 24, 62, 190, 35, 7, 209, 167, 245, 210, 68, 69, 154, 231, 184, 67, 64, 114, 84, 77, 200, 0, 200, 45, 148, 180, 177, 196, 65, 240, 55, 83],352),
 ];
fn is_reviewed_selector_blank(name:&str,index:usize,frame:&SpriteFrame)->bool {
    let Some((_,_,_,left))=REVIEWED_SELECTOR_EMPTY_SPRS.iter().find(|(n,_,_,_)|*n==name) else{return false};
    matches!(index,0|12) && (frame.x_offset,frame.y_offset,frame.width,frame.height,frame.unknown1,frame.unknown2)
        ==(*left,120,0,0,u16::MAX,0) && frame.rgba.is_empty()
        && frame.fragments.as_ref().is_none_or(|f|f.references.is_empty())
}
/// Pure admission contract also exercised by the isolated resource probe.
/// Every other file remains on the ordinary nonempty referenced-frame policy.
pub(crate) fn reviewed_selector_empty_source_matches(name:&str,bytes:&[u8],decoded:&DecodedSprite)->bool {
    let Some((_,size,digest,_))=REVIEWED_SELECTOR_EMPTY_SPRS.iter().find(|(n,_,_,_)|*n==name) else{return false};
    bytes.len()==*size && Sha256::digest(bytes).as_slice()==digest && decoded.frames.len()==13
        && decoded.frames.iter().enumerate().all(|(index,frame)| {
            if matches!(index,0|12) {is_reviewed_selector_blank(name,index,frame)}
            else {frame.width>0 && frame.height>0 && !frame.rgba.is_empty()
                && (frame.width as usize).checked_mul(frame.height as usize).and_then(|n|n.checked_mul(4))==Some(frame.rgba.len())}
        })
}
fn is_tw_healing_blank(name: &str, index: usize, frame: &SpriteFrame) -> bool {
    let Some((_,_,_,last_visible))=TW_HEALING_SPRS.iter().find(|(n,_,_,_)|*n==name) else {return false};
    (index==0 || (*last_visible<index && index<21)) && frame.rgba.is_empty()
        && (frame.x_offset,frame.y_offset,frame.width,frame.height,frame.unknown1,frame.unknown2)
            == (192,120,0,0,u16::MAX,0)
}
impl SharedEffectBank {
    pub(crate) fn load_with_catalog(
        context: &SharedActionCatalog,
        cancel: &AtomicBool,
        qualify: fn(&SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>>,
    ) -> Result<Self> {
        Self::load_with_graphics(context, cancel, qualify, &[])
    }
    /// Direct, explicitly qualified #id count=gfx bank identities. Animation
    /// keys remain the wire effect IDs; no recursive graphics/action aliasing.
    pub(crate) fn load_with_graphics(
        context: &SharedActionCatalog,
        cancel: &AtomicBool,
        qualify: fn(&SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>>,
        bindings: &[(u32, u32)],
    ) -> Result<Self> {
        Self::load_profile(context, cancel, qualify, bindings, false, ReferencedFramePolicy::NonEmpty)
    }
    pub(crate) fn load_directional(
        context: &SharedActionCatalog,
        cancel: &AtomicBool,
        qualify: fn(&SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>>,
    ) -> Result<Self> {
        Self::load_profile(context, cancel, qualify, &[], true, ReferencedFramePolicy::NonEmpty)
    }
    pub(crate) fn load_reviewed_dark_elf(context: &SharedActionCatalog, cancel: &AtomicBool) -> Result<Self> {
        Self::load_profile(context, cancel, super::native_dark_elf_effects::qualify, &[], false, ReferencedFramePolicy::ReviewedDarkElf)
    }
    pub(crate) fn load_reviewed_earth_jail(
        context: &SharedActionCatalog, cancel: &AtomicBool,
        qualify: fn(&SpriteActionCatalog) -> Result<Vec<(u32,u8,Animation)>>,
    ) -> Result<Self> {
        Self::load_profile(context,cancel,qualify,&[],true,ReferencedFramePolicy::ReviewedEarthJail)
    }
    pub(crate) fn load_reviewed_selectors(context:&SharedActionCatalog,cancel:&AtomicBool)->Result<Self> {
        Self::load_profile(context,cancel,super::native_selector_effects::qualify,
            super::native_selector_effects::GRAPHICS_BINDINGS,false,ReferencedFramePolicy::ReviewedSelectors)
    }
    pub(crate) fn load_reviewed_additional_finite(context:&SharedActionCatalog,cancel:&AtomicBool)->Result<Self>{
        Self::load_profile(context,cancel,super::native_additional_finite_effects::qualify,
            super::native_additional_finite_effects::GRAPHICS_BINDINGS,false,ReferencedFramePolicy::ReviewedAdditionalFinite)
    }
    /// T027 batch 4 exact finite definitions (2230／760／230); no extra blank slots.
    pub(crate) fn load_reviewed_batch4_finite(context:&SharedActionCatalog,cancel:&AtomicBool)->Result<Self>{
        Self::load_profile(context,cancel,super::native_additional_finite_effects::qualify_batch4,
            &[],false,ReferencedFramePolicy::ReviewedAdditionalFinite)
    }
    /// A received S37 proc/weapon effect may select any simple class-0 row in
    /// the pinned shared catalog. Complex controllers retain their own loaders.
    pub(crate) fn load_finite(
        context: &SharedActionCatalog,
        cancel: &AtomicBool,
        graphics: u16,
    ) -> Result<Self> {
        let gfx = u32::from(graphics);
        let bank = context.catalog().sprites.get(&gfx)
            .ok_or(HonorAssetError::Unsupported("finite effect definition absent"))?
            .graphics_id;
        Self::load_profile(context, cancel,
            // T027 batch 6 (#150): five exact-pinned rows carry a 109 row the
            // generic qualifier refuses; everything else keeps the generic rule.
            |catalog| super::native_additional_finite_effects::qualify_batch6(catalog, gfx)
                .unwrap_or_else(|| super::native_skill_effects::qualify_finite(catalog, gfx)),
            &[(gfx, bank)], true, ReferencedFramePolicy::ReceivedFinite)
    }
    fn load_profile(
        context: &SharedActionCatalog,
        cancel: &AtomicBool,
        qualify: impl FnOnce(&SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>>,
        bindings: &[(u32,u32)],
        directional: bool,
        frame_policy: ReferencedFramePolicy,
    ) -> Result<Self> {
        active(cancel)?;
        let root = context.root();
        let records = context.index_records();
        let size = context.byte_length();
        if records > MAX_INDEX_RECORDS || size == 0 || size > MAX_ENTRY {
            return Err(HonorAssetError::Bound("catalog"));
        }
        active(cancel)?;
        let catalog_sha = context.sha256();
        if size != 4_067_417 || catalog_sha != LIST_SHA {
            return Err(HonorAssetError::Unsupported(
                "unreviewed shared action catalog generation",
            ));
        }
        let qualified = qualify(context.catalog())?;
        Self::load_qualified_profile(root, records, catalog_sha, context.catalog(), qualified, bindings, directional, frame_policy, cancel)
    }
    /// The reviewed current TW healing family. The clothing row supplies only
    /// bitmap slots inherited from11725, never a separate clock or audio owner.
    pub(crate) fn load_reviewed_tw_healing_family(
        root: &Path,
        context: &super::tw_action_catalog::SharedTwActionCatalog,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        if !reviewed_tw_healing_generation(context.pack_sha256(), context.text_sha256(), context.decoded_bytes())
            || context.catalog().sprites.len()!=5 {
            return Err(HonorAssetError::Unsupported("unreviewed TW healing catalog generation"));
        }
        let catalog=context.catalog();let mut qualified=Vec::new();
        for (gfx,sound) in [(11722,109),(11723,114),(11724,115),(11725,126)] {
            let row=catalog.sprites.get(&gfx).ok_or(HonorAssetError::Unsupported("TW healing definition absent"))?;
            let resolved=catalog.resolve_action(gfx,ActionKey::new(0))?;let a=resolved.animation;
            if row.graphics_id!=gfx || row.image_count!=Some(1) || row.header_issue.is_some()
                || row.actions.len()!=1 || resolved.definition_sprite_id!=gfx
                || row.attributes.len()!=if gfx==11725 {2}else{1}
                || !row.attributes.iter().any(|v|v.key==ActionKey::new(104) && v.body_bytes.trim_ascii()==b"8")
                || (gfx==11725 && !row.attributes.iter().any(|v|v.key==ActionKey::new(105) && v.body_bytes.trim_ascii()==b"1 11726"))
                || a.directionality!=Directionality::Undirected || a.frames_per_second!=24 || a.frames.len()!=21
                || a.frames.iter().enumerate().any(|(i,f)|f.image_id!=0 || f.subframe!=if i==0 {20}else{i as u32-1}
                    || f.duration_ticks!=3 || f.events!=if i==1 {vec![FrameEvent::Sound(sound)]}else{vec![]}) {
                return Err(HonorAssetError::Unsupported("TW healing action/attachment controls changed"));
            }
            qualified.push((gfx,0,a.clone()));
        }
        let layer=catalog.sprites.get(&11726).ok_or(HonorAssetError::Unsupported("TW full-heal clothing absent"))?;
        if layer.graphics_id!=11726 || layer.image_count!=Some(1) || layer.header_issue.is_some()
            || !layer.actions.is_empty() || layer.attributes.len()!=2
            || !layer.attributes.iter().any(|v|v.key==ActionKey::new(102) && v.body_bytes.trim_ascii()==b"0")
            || !layer.attributes.iter().any(|v|v.key==ActionKey::new(104) && v.body_bytes.trim_ascii()==b"8") {
            return Err(HonorAssetError::Unsupported("TW full-heal clothing controls changed"));
        }
        // PE4E0200/4E7000 pass the same parent byte19 subframe to both resources.
        // Reuse only its asset-slot selection; no Clip is created for11726.
        let mut inherited_slots=qualified.iter().find(|(gfx,_,_)|*gfx==11725).unwrap().2.clone();
        for f in &mut inherited_slots.frames {f.events.clear();}
        qualified.push((11726,0,inherited_slots));
        Self::load_qualified_profile(root,0,context.text_sha256(),catalog,qualified,&[],false,ReferencedFramePolicy::TwHealingFamily,cancel)
    }
    // Both callers above bind a reviewed generation before this bounded shared loader.
    fn load_qualified_profile(
        root: &Path,
        records: usize,
        catalog_sha: [u8; 32],
        catalog: &SpriteActionCatalog,
        qualified: Vec<(u32, u8, Animation)>,
        bindings: &[(u32, u32)],
        directional: bool,
        frame_policy: ReferencedFramePolicy,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let graphics_bindings: BTreeMap<_, _> = bindings.iter().copied().collect();
        if bindings.len() > 32 || graphics_bindings.len() != bindings.len()
            || bindings.iter().any(|&(id, graphics)|
                !qualified.iter().any(|a| a.0 == id)
                || catalog.sprites.get(&id)
                    .is_none_or(|row| row.header_issue.is_some() || row.graphics_id != graphics))
        {
            return Err(HonorAssetError::Unsupported("unqualified graphics bank binding"));
        }
        active(cancel)?;
        if qualified.is_empty() || qualified.len() > 32 {
            return Err(HonorAssetError::Bound("actions"));
        }
        let mut actions = BTreeMap::new();
        let mut names = BTreeSet::new();
        let mut groups = Vec::new();
        for (gfx, action, animation) in qualified {
            if animation.frames.is_empty()
                || animation.frames.len() > 256
                || !(animation.directionality == Directionality::Undirected
                    || directional && animation.directionality == Directionality::EightWay)
            {
                return Err(HonorAssetError::Unsupported("Honor action shape"));
            }
            for (index, frame) in animation.frames.iter().enumerate() {
                let cues = classify_native_cues(&frame.events)
                    .map_err(|_| HonorAssetError::Unsupported("native cue classification"))?;
                if !cues.non_audio.is_empty()
                    && !(directional && cues.non_audio.iter().all(|e|
                        matches!(e, FrameEvent::Hit))) {
                    return Err(HonorAssetError::Unsupported(
                        "unimplemented visual/gameplay cue",
                    ));
                }
                if let Some(group) = cues.audio {
                    if group.mode
                        == super::combat_animation::NativeAudioMode::LegacyCaretDistanceMode
                    {
                        return Err(HonorAssetError::Unsupported("caret distance sound"));
                    }
                    if !groups.contains(&group) {
                        groups.push(group);
                    }
                }
                for heading in 0..8 {
                    names.insert(animation.frame_asset(graphics_bindings.get(&gfx).copied().unwrap_or(gfx), index, heading)?.file_name());
                }
            }
            if actions.insert((gfx, action), Arc::new(animation)).is_some() {
                return Err(HonorAssetError::Unsupported("duplicate bound action"));
            }
        }
        if names.len() > 256 || groups.len() > 32 {
            return Err(HonorAssetError::Bound("sprites or sound groups"));
        }
        let mut reader = SpriteReader::open(root, records, cancel)?;
        let mut sprites = BTreeMap::new();
        let mut encoded = 0usize;
        let mut rgba = 0usize;
        for name in names {
            active(cancel)?;
            let archive = reader.archive_for(&name, cancel)?;
            let size = archive.entry(&name)?.size as usize;
            if size == 0 || size > MAX_ENTRY || size > MAX_ENCODED - encoded {
                return Err(HonorAssetError::Bound("encoded SPR"));
            }
            let kind = archive.kind();
            active(cancel)?;
            let bytes = archive.read_entry(&name)?;
            active(cancel)?;
            if bytes.len() != size {
                return Err(HonorAssetError::Unsupported(
                    "SPR generation length changed",
                ));
            }
            let remaining = ((MAX_RGBA - rgba) / 4).min(16 * 1024 * 1024);
            if remaining == 0 {
                return Err(HonorAssetError::Bound("decoded SPR"));
            }
            let decoded = decode_world_spr_with_limits(
                &bytes,
                SpriteLimits {
                    max_input_bytes: MAX_ENTRY,
                    max_frame_dimension: 2048,
                    max_total_pixels: remaining,
                    ..Default::default()
                },
            )?;
            active(cancel)?;
            if matches!(frame_policy,ReferencedFramePolicy::TwHealingFamily) {
                let Some((_,expected_size,expected_sha,last_visible))=TW_HEALING_SPRS.iter().find(|(n,_,_,_)|*n==name) else {
                    return Err(HonorAssetError::Unsupported("unreviewed TW healing SPR"));
                };
                if size!=*expected_size || Sha256::digest(&bytes).as_slice()!=expected_sha
                    || decoded.frames.len()!=21 || decoded.frames.iter().enumerate().any(|(i,f)|
                        (i==0 || i>*last_visible) && !is_tw_healing_blank(&name,i,f)) {
                    return Err(HonorAssetError::Unsupported("TW healing original SPR/blank slots changed"));
                }
            }
            if matches!(frame_policy,ReferencedFramePolicy::ReviewedSelectors)
                && REVIEWED_SELECTOR_EMPTY_SPRS.iter().any(|(n,_,_,_)|*n==name)
                && !reviewed_selector_empty_source_matches(&name,&bytes,&decoded) {
                return Err(HonorAssetError::Unsupported("reviewed selector original SPR/blank slots changed"));
            }
            if matches!(frame_policy,ReferencedFramePolicy::ReviewedAdditionalFinite)
                && super::native_additional_finite_effects::requires_reviewed_blank(&name)
                && !super::native_additional_finite_effects::reviewed_blank_source_matches(&name,&bytes,&decoded) {
                return Err(HonorAssetError::Unsupported("original2251 no-block source changed"));
            }
            if matches!(frame_policy,ReferencedFramePolicy::ReviewedEarthJail)
                && !super::native_attack_many::earth_jail::source_matches(&name,&bytes,&decoded) {
                return Err(HonorAssetError::Unsupported("earth jail original SPR/blank slots changed"));
            }
            if matches!(frame_policy, ReferencedFramePolicy::ReviewedDarkElf)
                && !super::native_dark_elf_effects::source_matches(&name, &bytes, &decoded) {
                return Err(HonorAssetError::Unsupported("dark elf original SPR/blank slots changed"));
            }
            let decoded_bytes = decoded
                .retained_rgba_bytes()
                .ok_or(HonorAssetError::Bound("decoded SPR"))?;
            if decoded_bytes > MAX_RGBA - rgba {
                return Err(HonorAssetError::Bound("decoded SPR"));
            }
            encoded += size;
            rgba += decoded_bytes;
            sprites.insert(
                name.clone(),
                LoadedSprite {
                    provenance: SpriteProvenance {
                        archive: kind,
                        name,
                        bytes: size as u32,
                        sha256: Sha256::digest(&bytes).into(),
                    },
                    decoded,
                },
            );
        }
        drop(reader);
        for (&(gfx, _), animation) in &actions {
            for index in 0..animation.frames.len() {
                for heading in 0..8 {
                    let asset = animation.frame_asset(graphics_bindings.get(&gfx).copied().unwrap_or(gfx), index, heading)?;
                    let sprite = sprites.get(&asset.file_name())
                        .ok_or(HonorAssetError::Unsupported("referenced SPR absent"))?;
                    let frame = sprite.decoded.frames.get(asset.subframe as usize)
                        .ok_or_else(|| HonorAssetError::MissingFrame {
                            member: asset.file_name(), subframe: asset.subframe,
                            frames: sprite.decoded.frames.len(),
                        })?;
                    let reviewed_blank = matches!(frame_policy,ReferencedFramePolicy::ReviewedDarkElf)
                        && super::native_dark_elf_effects::is_blank(&asset.file_name(),asset.subframe as usize,frame)
                        || matches!(frame_policy,ReferencedFramePolicy::TwHealingFamily)
                        && is_tw_healing_blank(&asset.file_name(),asset.subframe as usize,frame)
                        || matches!(frame_policy,ReferencedFramePolicy::ReviewedSelectors)
                            && is_reviewed_selector_blank(&asset.file_name(),asset.subframe as usize,frame)
                        || matches!(frame_policy,ReferencedFramePolicy::ReviewedAdditionalFinite)
                            && super::native_additional_finite_effects::is_reviewed_blank(&asset.file_name(),asset.subframe as usize,frame)
                        || matches!(frame_policy,ReferencedFramePolicy::ReviewedEarthJail)
                            && super::native_attack_many::earth_jail::is_blank(&asset.file_name(),asset.subframe as usize,frame)
                        || matches!(frame_policy, ReferencedFramePolicy::ReceivedFinite)
                            && (frame.x_offset, frame.y_offset, frame.width, frame.height, frame.unknown1, frame.unknown2)
                                == (192, 120, 0, 0, u16::MAX, 0)
                            && frame.rgba.is_empty()
                            && frame.fragments.as_ref().is_some_and(|fragments| fragments.references.is_empty());
                    if !reviewed_blank && (frame.width == 0
                        || frame.height == 0
                        || frame.rgba.len() != frame.width as usize * frame.height as usize * 4)
                    {
                        return Err(HonorAssetError::Unsupported(
                            "empty or malformed referenced frame",
                        ));
                    }
                }
            }
        }
        active(cancel)?;
        let sound = SoundBank::load_shared(
            root,
            &groups,
            BankLimits {
                max_bytes: 8 * 1024 * 1024,
                max_sounds: 32,
                max_groups: 32,
            },
            cancel,
        )
        .map_err(|_| {
            if cancel.load(Ordering::Acquire) {
                HonorAssetError::Cancelled
            } else {
                HonorAssetError::UnsupportedSoundBank
            }
        })?;
        active(cancel)?;
        Ok(Self {
            actions,
            graphics_bindings,
            sprites,
            sound: Arc::new(sound),
            groups,
            catalog_sha,
            encoded,
            rgba,
        })
    }
    pub(crate) fn catalog_sha256(&self) -> [u8; 32] { self.catalog_sha }
    pub(crate) fn animation(&self, gfx: u32, action: u8) -> Result<&Arc<Animation>> {
        self.actions
            .get(&(gfx, action))
            .ok_or(HonorAssetError::Unsupported("action not preloaded"))
    }
    pub(crate) fn frame(
        &self,
        gfx: u32,
        action: u8,
        source_frame: usize,
        finished: bool,
    ) -> Result<VisualSample<'_>> {
        self.frame_heading(gfx, action, source_frame, finished, 0)
    }
    pub(crate) fn frame_heading(
        &self, gfx:u32, action:u8, source_frame:usize, finished:bool, heading:u8,
    ) -> Result<VisualSample<'_>> {
        let asset = self
            .animation(gfx, action)?
            .frame_asset(self.graphics_bindings.get(&gfx).copied().unwrap_or(gfx), source_frame, heading)?;
        let sprite = self
            .sprites
            .get(&asset.file_name())
            .ok_or(HonorAssetError::Unsupported("sprite not preloaded"))?;
        let subframe = asset.subframe as usize;
        let frame = sprite
            .decoded
            .frames
            .get(subframe)
            .ok_or(HonorAssetError::Unsupported("subframe not preloaded"))?;
        Ok(VisualSample {
            frame,
            source_frame,
            finished,
            graphics_id: gfx,
            sprite_name: &sprite.provenance.name,
            subframe,
        })
    }
    pub(crate) fn sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.sprites.values().map(|s| &s.provenance)
    }
    pub(crate) fn frames(&self, name: &str) -> Result<&[SpriteFrame]> {
        self.sprites
            .get(name)
            .map(|s| s.decoded.frames.as_slice())
            .ok_or(HonorAssetError::Unsupported("sprite not preloaded"))
    }
    pub(crate) fn retained_rgba_bytes(&self)->usize {self.rgba}
    pub(crate) fn encoded_bytes(&self)->usize {self.encoded}
    pub(crate) fn sound_bank(&self) -> Arc<SoundBank> {
        Arc::clone(&self.sound)
    }
    pub(crate) fn groups(&self) -> &[NativeAudioGroup] {
        &self.groups
    }
    pub(crate) fn catalog_sha(&self) -> [u8; 32] {
        self.catalog_sha
    }
    pub(crate) fn encoded(&self) -> usize {
        self.encoded
    }
    pub(crate) fn rgba(&self) -> usize {
        self.rgba
    }
}

/// Fixed source-frame timeline, not a simulation timer. Empty active span is
/// valid for the native single-frame tower states: source0 is already visible.
#[derive(Clone)]
pub(crate) struct OneShot {
    animation: Arc<Animation>,
    ends: Vec<u128>,
    ticks: u128,
}
impl OneShot {
    pub(crate) fn new(animation: Arc<Animation>) -> Result<Self> {
        if animation.frames.is_empty()
            || animation.frames.len() > 256
            || animation.frames_per_second != 24
        {
            return Err(HonorAssetError::Unsupported("one-shot shape/FPS"));
        }
        let mut ticks = 0u128;
        let mut ends = Vec::new();
        for frame in &animation.frames[1..] {
            ticks = ticks
                .checked_add(u128::from(frame.duration_ticks))
                .ok_or(HonorAssetError::Bound("ticks"))?;
            ends.push(ticks);
        }
        if ticks > 24 * 60 {
            return Err(HonorAssetError::Bound("action duration"));
        }
        Ok(Self {
            animation,
            ends,
            ticks,
        })
    }
    pub(crate) fn duration(&self) -> Duration {
        Duration::from_nanos((self.ticks * NANOS).div_ceil(24) as u64)
    }
    pub(crate) fn sample(&self, elapsed: Duration) -> Result<(usize, bool)> {
        let tick = elapsed
            .as_nanos()
            .checked_mul(24)
            .ok_or(HonorAssetError::Bound("elapsed"))?
            / NANOS;
        if tick >= self.ticks {
            Ok((0, true))
        } else {
            Ok((self.ends.partition_point(|end| *end <= tick) + 1, false))
        }
    }
    pub(crate) fn start(&self) -> HonorPlayback {
        HonorPlayback {
            sequence: self.clone(),
            last: None,
            repeat_single: false,
        }
    }
}
pub struct HonorSample {
    pub source_frame: usize,
    pub finished: bool,
    pub crossed_cues: Vec<CrossedCue>,
}
#[derive(Clone)]
pub struct HonorPlayback {
    sequence: OneShot,
    last: Option<Duration>,
    repeat_single: bool,
}
impl HonorPlayback {
    /// Exact native markers, once per crossing including tick0. Never changes
    /// HP, world state or PCM rate. Caller deduplicates authoritative events.
    pub fn advance(&mut self, elapsed: Duration) -> Result<HonorSample> {
        if self.last.is_some_and(|last| elapsed < last) {
            return Err(HonorAssetError::InvalidInput("elapsed regressed"));
        }
        let (source_frame, finished) = self.sequence.sample(elapsed)?;
        let to = elapsed
            .as_nanos()
            .checked_mul(24)
            .ok_or(HonorAssetError::Bound("elapsed"))?
            / NANOS;
        let from = self.last.map(|last| last.as_nanos() * 24 / NANOS);
        let mut crossed_cues = Vec::new();
        let mut tick = 0u128;
        for index in (1..self.sequence.animation.frames.len()).chain(std::iter::once(0)) {
            if tick <= to && from.is_none_or(|last| tick > last) {
                for (event_index, event) in self.sequence.animation.frames[index]
                    .events
                    .iter()
                    .enumerate()
                {
                    if crossed_cues.len() >= 1024 {
                        return Err(HonorAssetError::Bound("crossed cues"));
                    }
                    crossed_cues.push(CrossedCue {
                        source_frame: index,
                        event_index,
                        tick: tick as u64,
                        event: event.clone(),
                    });
                }
            }
            if index != 0 {
                tick += u128::from(self.sequence.animation.frames[index].duration_ticks);
            }
        }
        self.last = Some(elapsed);
        Ok(HonorSample {
            source_frame,
            finished: finished && !self.repeat_single,
            crossed_cues,
        })
    }
}

/// Direct native image route, NOT a made-up list.spr action definition.
fn direct_frames(count: u32) -> Animation {
    Animation {
        label_bytes: Vec::new(),
        directionality: Directionality::Undirected,
        frames_per_second: 24,
        frames: (0..count)
            .map(|subframe| ActionFrame {
                image_id: 0,
                subframe,
                duration_ticks: 0,
                events: Vec::new(),
            })
            .collect(),
    }
}
fn qualify_towers(catalog: &SpriteActionCatalog) -> Result<Vec<(u32, u8, Animation)>> {
    let mut actions = Vec::new();
    for gfx in GRAPHICS {
        let sprite = catalog
            .sprites
            .get(&gfx)
            .ok_or(HonorAssetError::Unsupported("missing tower definition"))?;
        if sprite.graphics_id != gfx
            || sprite.image_count != Some(1)
            || sprite.actions.len() != 4
            || sprite.attributes.len() != 2
            || !sprite
                .attributes
                .iter()
                .any(|a| a.key == ActionKey::new(101) && a.body_bytes == b"2673")
            || !sprite
                .attributes
                .iter()
                .any(|a| a.key == ActionKey::new(102) && a.body_bytes == b"11")
        {
            return Err(HonorAssetError::Unsupported(
                "tower controls / shadow / type",
            ));
        }
        // Native action0 absent: sub4E5C00 loc4E62A1 loads gfx-0 directly,
        // loc4E639B selects subframe0. Do not alias this semantic state to32.
        actions.push((gfx, 0, direct_frames(1)));
        for action in [32u8, 33, 34, 35] {
            let resolved = catalog.resolve_action(gfx, ActionKey::new(u32::from(action)))?;
            let a = resolved.animation;
            if resolved.definition_sprite_id != gfx
                || a.directionality != Directionality::Undirected
                || a.frames_per_second != 24
                || a.frames.len() != if action == 35 { 18 } else { 1 }
            {
                return Err(HonorAssetError::Unsupported("tower action shape"));
            }
            for (index, frame) in a.frames.iter().enumerate() {
                let subframe = if action == 35 {
                    if index == 0 {
                        20
                    } else {
                        index as u32 + 2
                    }
                } else if gfx == 2665 {
                    0
                } else {
                    u32::from(action - 32)
                };
                let events = if action == 35 && index == 1 {
                    vec![FrameEvent::Sound(360)]
                } else {
                    Vec::new()
                };
                if frame.image_id != 0
                    || frame.subframe != subframe
                    || frame.duration_ticks != 4
                    || frame.events != events
                {
                    return Err(HonorAssetError::Unsupported("unreviewed tower frame/cue"));
                }
            }
            actions.push((gfx, action, a.clone()));
        }
    }
    let shadow = catalog
        .sprites
        .get(&2673)
        .ok_or(HonorAssetError::Unsupported(
            "missing tower shadow definition",
        ))?;
    if shadow.graphics_id != 2673
        || shadow.image_count != Some(1)
        || !shadow.actions.is_empty()
        || shadow.attributes.len() != 1
        || shadow.attributes[0].key != ActionKey::new(102)
        || shadow.attributes[0].body_bytes != b"0"
    {
        return Err(HonorAssetError::Unsupported(
            "unreviewed direct shadow definition",
        ));
    }
    // Native loc4E5FA8/4E5FE5 chooses attr101 shadow, same image and subframe
    // as the body's resolved SPR. All21 required subframes are qualified.
    actions.push((2673, 0, direct_frames(21)));
    Ok(actions)
}

pub struct TowerAssets {
    bank: SharedEffectBank,
    states: BTreeMap<(u32, u8), OneShot>,
}
impl TowerAssets {
    pub fn load(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let catalog = SharedActionCatalog::load(root, cancel)?;
        Self::load_with_catalog(&catalog, cancel)
    }
    pub(crate) fn load_with_catalog(
        catalog: &SharedActionCatalog,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let bank = SharedEffectBank::load_with_catalog(catalog, cancel, qualify_towers)?;
        let mut states = BTreeMap::new();
        for gfx in GRAPHICS {
            for action in ACTIONS {
                states.insert(
                    (gfx, action),
                    OneShot::new(Arc::clone(bank.animation(gfx, action)?))?,
                );
            }
        }
        active(cancel)?;
        Ok(Self { bank, states })
    }
    fn state(&self, gfx: u32, action: u8) -> Result<&OneShot> {
        self.states
            .get(&(gfx, action))
            .ok_or(HonorAssetError::Unsupported(
                "tower graphics/state not qualified",
            ))
    }
    pub fn state_duration(&self, gfx: u32, action: u8) -> Result<Duration> {
        let state = self.state(gfx, action)?;
        // state32 is the native requeued one-frame terminal dwell; state0 and
        // 33/34 are static,35 is a finite collapse followed by a held ruin.
        Ok(if action == 32 {
            Duration::from_nanos(166_666_667)
        } else {
            state.duration()
        })
    }
    pub fn start_state(&self, gfx: u32, action: u8) -> Result<HonorPlayback> {
        let mut playback = self.state(gfx, action)?.start();
        playback.repeat_single = action == 32;
        Ok(playback)
    }
    pub fn state_frame(&self, gfx: u32, action: u8, elapsed: Duration) -> Result<VisualSample<'_>> {
        let (source_frame, finished) = self.state(gfx, action)?.sample(elapsed)?;
        self.bank
            .frame(gfx, action, source_frame, finished && action != 32)
    }
    pub fn shadow_frame(
        &self,
        gfx: u32,
        action: u8,
        elapsed: Duration,
    ) -> Result<VisualSample<'_>> {
        let body = self.state_frame(gfx, action, elapsed)?;
        self.bank.frame(2673, 0, body.subframe, body.finished)
    }
    /// Draw shadow first, then body at the same authoritative world anchor.
    /// Native offsets are retained; no invented centering or heading fold.
    /// The explicit shadow uses the same RGB555 reference blend as world_scene.
    pub fn draw_state(
        &self,
        canvas: &mut Canvas,
        gfx: u32,
        action: u8,
        anchor: PixelPoint,
        elapsed: Duration,
    ) -> Result<()> {
        let shadow = self.shadow_frame(gfx, action, elapsed)?;
        let x = anchor
            .x
            .checked_add(shadow.frame.x_offset)
            .ok_or(HonorAssetError::Bound("shadow projected x"))?;
        let y = anchor
            .y
            .checked_add(shadow.frame.y_offset)
            .ok_or(HonorAssetError::Bound("shadow projected y"))?;
        canvas
            .blit_native_shadow_rgb555(
                &shadow.frame.rgba,
                shadow.frame.width,
                shadow.frame.height,
                x,
                y,
            )
            .map_err(|error| HonorAssetError::Render(TeleportError::Render(error)))?;
        let body = self.state_frame(gfx, action, elapsed)?;
        draw_frame(canvas, body.frame, anchor).map_err(HonorAssetError::Render)
    }
    pub fn sound_bank(&self) -> Arc<SoundBank> {
        self.bank.sound_bank()
    }
    pub fn sound_groups(&self) -> &[NativeAudioGroup] {
        self.bank.groups()
    }
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.bank.sources()
    }
    pub fn sprite_frames(&self, name: &str) -> Result<&[SpriteFrame]> {
        self.bank.frames(name)
    }
    pub fn catalog_sha256(&self) -> [u8; 32] {
        self.bank.catalog_sha()
    }
    pub fn encoded_bytes(&self) -> usize {
        self.bank.encoded()
    }
    pub fn rgba_bytes(&self) -> usize {
        self.bank.rgba()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tw_healing_generations_reject_mixed_changed_and_unknown_identity() {
        for (pack, text, bytes) in super::TW_HEALING_GENERATIONS {
            assert!(super::reviewed_tw_healing_generation(pack, text, bytes));
            assert!(!super::reviewed_tw_healing_generation(pack, text, bytes + 1));
            let mut changed_pack = pack; changed_pack[0] ^= 1;
            assert!(!super::reviewed_tw_healing_generation(changed_pack, text, bytes));
            let mut changed_text = text; changed_text[0] ^= 1;
            assert!(!super::reviewed_tw_healing_generation(pack, changed_text, bytes));
            for (other_pack, other_text, other_bytes) in super::TW_HEALING_GENERATIONS {
                if pack == other_pack { continue; }
                assert!(!super::reviewed_tw_healing_generation(pack, other_text, bytes));
                assert!(!super::reviewed_tw_healing_generation(other_pack, text, other_bytes));
            }
        }
        assert!(!super::reviewed_tw_healing_generation([0; 32], [0; 32], 0));
    }
    use super::super::{
        honor_healing_effect::HonorHealingAssets, sprite_actions::ActionDefinition,
    };
    use super::*;

    fn catalog() -> SpriteActionCatalog {
        let mut text = String::from("5 0 0\n");
        for gfx in GRAPHICS {
            text.push_str(&format!("#{gfx} 1 tower\n"));
            for action in [32, 33, 34] {
                text.push_str(&format!(
                    "{action}.state(0 1,0.{}:4)\n",
                    if gfx == 2665 { 0 } else { action - 32 }
                ));
            }
            text.push_str("35.collapse(0 18,0.20:4");
            for subframe in 3..20 {
                text.push_str(&format!(
                    " 0.{subframe}:4{}",
                    if subframe == 3 { "[360" } else { "" }
                ));
            }
            text.push_str(")101.shadow(2673)102.type(11)\n");
        }
        text.push_str("#2673 1 shadow\n102.type(0)\n");
        SpriteActionCatalog::parse_for_inspection(text.as_bytes(), Default::default()).unwrap()
    }
    fn fixture() -> TowerAssets {
        let qualified = qualify_towers(&catalog()).unwrap();
        let mut actions = BTreeMap::new();
        let mut states = BTreeMap::new();
        let mut sprites = BTreeMap::new();
        for (gfx, action, a) in qualified {
            let a = Arc::new(a);
            if gfx != 2673 {
                states.insert((gfx, action), OneShot::new(Arc::clone(&a)).unwrap());
            }
            actions.insert((gfx, action), a);
        }
        for gfx in GRAPHICS.into_iter().chain([2673]) {
            let name = format!("{gfx}-0.spr");
            let frames = (0..21)
                .map(|sub| SpriteFrame {
                    fragments: None,
                    x_offset: if gfx == 2673 { -2 } else { -1 },
                    y_offset: -1,
                    width: 2,
                    height: 2,
                    frame_type: 0,
                    unknown1: 0,
                    unknown2: 0,
                    rgba: [if gfx == 2673 { 5 } else { 100 + sub }, 20, 30, 255].repeat(4),
                })
                .collect();
            sprites.insert(
                name.clone(),
                LoadedSprite {
                    provenance: SpriteProvenance {
                        archive: SharedArchiveKind::Sprite,
                        name,
                        bytes: 0,
                        sha256: [0; 32],
                    },
                    decoded: DecodedSprite {
                        frames,
                        has_palette: false,
                        block_table_metadata: 0,
                    },
                },
            );
        }
        // Empty group input invokes no resource reader; not a hardware sound test.
        let sound = SoundBank::load_shared(
            "absent",
            &[],
            BankLimits::default(),
            &AtomicBool::new(false),
        )
        .unwrap();
        TowerAssets {
            states,
            bank: SharedEffectBank {
                actions,
                graphics_bindings: BTreeMap::new(),
                sprites,
                sound: Arc::new(sound),
                groups: Vec::new(),
                catalog_sha: [0; 32],
                encoded: 0,
                rgba: 0,
            },
        }
    }
    #[test]
    fn healthy_is_native_direct_image_zero_not_a_catalog_action_alias() {
        let cat = catalog();
        let assets = fixture();
        for gfx in GRAPHICS {
            assert!(!cat.sprites[&gfx].actions.contains_key(&ActionKey::new(0)));
            let healthy = assets.state_frame(gfx, 0, Duration::MAX).unwrap();
            assert_eq!(
                (healthy.graphics_id, healthy.source_frame, healthy.subframe),
                (gfx, 0, 0)
            );
            assert_eq!(healthy.sprite_name, format!("{gfx}-0.spr"));
            let shadow = assets.shadow_frame(gfx, 0, Duration::MAX).unwrap();
            assert_eq!((shadow.graphics_id, shadow.subframe), (2673, 0));
        }
    }
    #[test]
    fn native_state32_repeats_single_terminal_dwell_33_34_hold_without_cues() {
        let assets = fixture();
        for gfx in GRAPHICS {
            for action in [32u8, 33, 34] {
                let mut playback = assets.start_state(gfx, action).unwrap();
                for elapsed in [
                    Duration::ZERO,
                    Duration::from_millis(166),
                    Duration::from_nanos(166_666_667),
                    Duration::MAX,
                ] {
                    let sample = playback.advance(elapsed).unwrap();
                    assert_eq!(sample.source_frame, 0);
                    assert_eq!(sample.finished, action != 32);
                    assert!(sample.crossed_cues.is_empty());
                    let body = assets.state_frame(gfx, action, elapsed).unwrap();
                    assert_eq!(
                        body.subframe,
                        if gfx == 2665 {
                            0
                        } else {
                            usize::from(action - 32)
                        }
                    );
                    assert_eq!(
                        body.subframe,
                        assets.shadow_frame(gfx, action, elapsed).unwrap().subframe
                    );
                }
            }
            assert_eq!(
                assets.state_duration(gfx, 32).unwrap(),
                Duration::from_nanos(166_666_667)
            );
            assert_eq!(assets.state_duration(gfx, 33).unwrap(), Duration::ZERO);
        }
    }
    #[test]
    fn collapse_has_seventeen_active_frames_then_holds_ruin_and_sound_once() {
        let assets = fixture();
        for gfx in GRAPHICS {
            let end = Duration::from_nanos(2_833_333_334);
            assert_eq!(assets.state_duration(gfx, 35).unwrap(), end);
            let initial = assets.state_frame(gfx, 35, Duration::ZERO).unwrap();
            assert_eq!((initial.source_frame, initial.subframe), (1, 3));
            assert!(!initial.finished);
            for i in 0..17 {
                let elapsed = Duration::from_nanos(((i as u128 * 4 * NANOS).div_ceil(24)) as u64);
                let frame = assets.state_frame(gfx, 35, elapsed).unwrap();
                assert_eq!((frame.source_frame, frame.subframe), (i + 1, i + 3));
                assert_eq!(
                    assets.shadow_frame(gfx, 35, elapsed).unwrap().subframe,
                    i + 3
                );
            }
            assert_eq!(
                assets
                    .state_frame(gfx, 35, end - Duration::from_nanos(1))
                    .unwrap()
                    .subframe,
                19
            );
            for elapsed in [end, Duration::MAX] {
                let frame = assets.state_frame(gfx, 35, elapsed).unwrap();
                assert_eq!((frame.source_frame, frame.subframe), (0, 20));
                assert!(frame.finished);
            }
            let mut playback = assets.start_state(gfx, 35).unwrap();
            let cues = playback.advance(Duration::ZERO).unwrap().crossed_cues;
            assert_eq!(cues.len(), 1);
            assert_eq!(cues[0].event, FrameEvent::Sound(360));
            assert_eq!(cues[0].tick, 0);
            assert!(playback
                .advance(Duration::ZERO)
                .unwrap()
                .crossed_cues
                .is_empty());
            assert!(playback
                .advance(Duration::MAX)
                .unwrap()
                .crossed_cues
                .is_empty());
            assert!(playback.advance(Duration::ZERO).is_err());
        }
    }
    #[test]
    fn unreviewed_type_shadow_graphics_action_and_event_fail_closed() {
        for mutation in 0..6 {
            let mut cat = catalog();
            let sprite = cat.sprites.get_mut(&2667).unwrap();
            match mutation {
                0 => sprite.graphics_id = 2665,
                1 => sprite.attributes[0].body_bytes = b"2674".to_vec(),
                2 => sprite.attributes[1].body_bytes = b"10".to_vec(),
                3 => {
                    sprite.actions.remove(&ActionKey::new(32));
                }
                _ => {
                    let ActionDefinition::Animation(a) =
                        sprite.actions.get_mut(&ActionKey::new(35)).unwrap()
                    else {
                        panic!("fixture")
                    };
                    if mutation == 4 {
                        a.frames[1].events = vec![FrameEvent::Hit];
                    } else {
                        a.frames[1].events = vec![FrameEvent::Effect(360)];
                    }
                }
            }
            assert!(qualify_towers(&cat).is_err());
        }
        let mut cat = catalog();
        cat.sprites.remove(&2673);
        assert!(qualify_towers(&cat).is_err());
        let assets = fixture();
        for (gfx, action) in [(0, 0), (11722, 0), (2667, 8), (2667, 31), (2667, 36)] {
            assert!(assets.state_frame(gfx, action, Duration::ZERO).is_err());
        }
    }
    #[test]
    fn shadow_and_body_keep_source_offsets_and_terminal_pixels_are_stable() {
        let assets = fixture();
        let anchor = PixelPoint { x: 5, y: 5 };
        let mut combined = Canvas::new(12, 12).unwrap();
        let mut body = Canvas::new(12, 12).unwrap();
        assets
            .draw_state(&mut combined, 2667, 0, anchor, Duration::ZERO)
            .unwrap();
        draw_frame(
            &mut body,
            assets.state_frame(2667, 0, Duration::ZERO).unwrap().frame,
            anchor,
        )
        .unwrap();
        assert_ne!(combined.rgba(), body.rgba());
        let mut a = Canvas::new(12, 12).unwrap();
        let mut b = Canvas::new(12, 12).unwrap();
        assets
            .draw_state(
                &mut a,
                2667,
                35,
                anchor,
                assets.state_duration(2667, 35).unwrap(),
            )
            .unwrap();
        assets
            .draw_state(&mut b, 2667, 35, anchor, Duration::MAX)
            .unwrap();
        assert_eq!(a.rgba(), b.rgba());
    }
    #[test]
    fn cancellation_is_checked_before_opening_any_resource() {
        assert!(matches!(
            TowerAssets::load(Path::new("absent"), &AtomicBool::new(true)),
            Err(HonorAssetError::Cancelled)
        ));
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
    fn aggregate(frames: &[SpriteFrame]) -> String {
        let mut bytes = Vec::new();
        for (index, frame) in frames.iter().enumerate() {
            for value in [
                index as i32,
                frame.x_offset,
                frame.y_offset,
                frame.width as i32,
                frame.height as i32,
                i32::from(frame.frame_type),
            ] {
                bytes.extend(value.to_be_bytes());
            }
            bytes.extend(Sha256::digest(&frame.rgba));
        }
        assert_eq!(bytes.len(), 1176);
        hex(&Sha256::digest(bytes))
    }
    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; independent C#126-frame oracle, no UI or audio device"]
    fn actual_honor_towers_shadow_and_explicit_classic_heal_match_independent_reference() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
        let cancel = AtomicBool::new(false);
        let towers = TowerAssets::load(Path::new(&root), &cancel).unwrap();
        let heal = HonorHealingAssets::load(Path::new(&root), &cancel).unwrap();
        assert_eq!(towers.catalog_sha256(), LIST_SHA);
        assert_eq!(heal.catalog_sha256(), LIST_SHA);
        assert_eq!(towers.sprite_sources().len(), 5);
        assert_eq!(heal.sprite_sources().len(), 1);
        assert_eq!(towers.encoded_bytes() + heal.encoded_bytes(), 488154);
        let rows = [
            (
                2665,
                110527,
                "ab67c6b1c79ec3eb2d86cafa62a072b876d1bf01ae0461fd850be8f871ceb793",
                "1b6d0d344597eadadb329210f84380a31b06f6783852c8290effbb06cec39877",
            ),
            (
                2667,
                110527,
                "2fa14266055ecceeae500a1e6aae63e18ca01295b944385d0e914a5431faaa00",
                "ec594009cc8dc29ab198801bb481a53291cc6d4aaeba4e9fd6ae79a816f04f68",
            ),
            (
                2669,
                110527,
                "9f9190e81f5698db105b2a0d6360d55594ca35f21e8f4cec29dd257b80491c2b",
                "9c8fead851f194b45bef61e9a1190194b8c2758cd2c94c2ebef2885505f3474f",
            ),
            (
                2671,
                110527,
                "d13f56c2a1fe2aac07477e88f6106d58c6ef9f49b9375776efe1c40a8385ca1f",
                "fce0678ea9950cee7509893d22f340508293ba225d036489df4c43a33a040aed",
            ),
            (
                2673,
                38787,
                "4257ca0470b28f1d0104c5eb215fcab41df66936edcb198104d4c40412c88812",
                "f510468e45a66066d4f870b7aa9c7fd159cb841f15a94ca31698347ef544f05e",
            ),
            (
                744,
                7259,
                "5532e36edcd31152777b044f7f15cc1424c0d178bde875c31b35358a99a2819c",
                "ae5c65630208e77fd57684f07ad11b40ecefa4b6facc885cc1ef05e2c8080047",
            ),
        ];
        for (gfx, size, sha, rgba) in rows {
            let name = format!("{gfx}-0.spr");
            let source = if gfx == 744 {
                heal.sprite_sources().find(|s| s.name == name).unwrap()
            } else {
                towers.sprite_sources().find(|s| s.name == name).unwrap()
            };
            assert_eq!(source.archive, SharedArchiveKind::Sprite);
            assert_eq!(source.bytes, size);
            assert_eq!(hex(&source.sha256), sha);
            let frames = if gfx == 744 {
                heal.sprite_frames(&name).unwrap()
            } else {
                towers.sprite_frames(&name).unwrap()
            };
            assert_eq!(frames.len(), 21);
            assert_eq!(aggregate(frames), rgba, "gfx{gfx}");
            if gfx != 744 {
                let sprite = &towers.bank.sprites[&name];
                assert!(sprite.decoded.has_palette);
                assert_eq!(
                    sprite.decoded.block_table_metadata,
                    if gfx == 2673 { 60557 } else { 175376 }
                );
            }
        }
        for gfx in GRAPHICS {
            for action in ACTIONS {
                let elapsed = if action == 35 {
                    Duration::ZERO
                } else {
                    Duration::MAX
                };
                let body = towers.state_frame(gfx, action, elapsed).unwrap();
                let shadow = towers.shadow_frame(gfx, action, elapsed).unwrap();
                assert_eq!(body.subframe, shadow.subframe);
                let mut canvas = Canvas::new(800, 600).unwrap();
                let baseline = canvas.rgba().to_vec();
                towers
                    .draw_state(
                        &mut canvas,
                        gfx,
                        action,
                        PixelPoint { x: 400, y: 400 },
                        elapsed,
                    )
                    .unwrap();
                assert_ne!(canvas.rgba(), baseline, "gfx{gfx} action{action}");
            }
        }
        let mut visible = Canvas::new(800, 600).unwrap();
        let blank = visible.rgba().to_vec();
        assert!(heal
            .draw_effect(&mut visible, PixelPoint { x: 400, y: 300 }, Duration::ZERO)
            .unwrap());
        assert_ne!(visible.rgba(), blank);
        let mut terminal = Canvas::new(800, 600).unwrap();
        assert!(!heal
            .draw_effect(
                &mut terminal,
                PixelPoint { x: 400, y: 300 },
                heal.duration()
            )
            .unwrap());
        assert_eq!(terminal.rgba(), blank);
        assert_eq!(towers.sound_groups().len(), 1);
        assert_eq!(towers.sound_groups()[0].candidates(), [360]);
        assert_eq!(heal.sound_group().unwrap().candidates(), [109]);
        for (bank, id, file_bytes, pcm_bytes, sha) in [
            (
                heal.sound_bank(),
                109,
                61046,
                60928,
                "664c8d8b353e2a03cbc337df7a7d36ce009d0213d33064e43779f4c76471b4d1",
            ),
            (
                towers.sound_bank(),
                360,
                194954,
                194838,
                "260729a31402a80dc1a63b73195fa676156ac2911a08dcccc0fd2dcf80187032",
            ),
        ] {
            let sound = bank.get(id).unwrap();
            assert_eq!(sound.file_bytes(), file_bytes);
            assert_eq!(sound.pcm_bytes().len(), pcm_bytes);
            assert_eq!(hex(&sound.sha256()), sha);
            let format = sound.format();
            assert_eq!(format.channels, 1);
            assert_eq!(format.sample_rate, 22050);
            assert_eq!(format.bits_per_sample, 16);
            assert_eq!(format.byte_rate, 44100);
            assert_eq!(format.block_align, 2);
        }
    }
}

#[cfg(test)]
#[path = "tower_assets_selector_blank_tests.rs"]
mod selector_blank_tests;
