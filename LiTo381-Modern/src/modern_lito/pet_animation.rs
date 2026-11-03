//! Four source-qualified PetRace forms, not a general polymorph renderer.
//!
//! Native 4E5C00 (full-disassembly.asm 480832..480911) selects attr101's
//! shadow bank using the BODY's image+heading and subframe, without resolving
//! any shadow action. 4E0200 (470272..470288) draws it at the same cell using
//! `(actor_flags & 0x0fff)|5`. This module only supports ordinary, unmodified
//! type10 forms with attributes101/102; no invisibility, tint, attr8 or dynamic
//! draw flags are accepted. Shadow mixing uses the existing RGB555 reference,
//! not a claim of native RGB565/hardware bit parity. Body uses raw SPR offsets;
//! the walk camera pulse must NOT also be added to body/shadow positions.
//!
//! Remote idle evidence: 4E02C0/4E0399 queues action3 after 584ms; 4E9300's
//! case32 -> 4E45F0 starts source1. Breath plays 1..N-1 then terminal0, with
//! source0's own dwell before repeating. The 584ms delay is not reapplied each
//! cycle. Bare local mouse-up/STOP is NOT qualified by this API. Base source
//! durations use native integer milliseconds before unported status speed
//! modifiers; they never change an authoritative cell or movement interval.
//!
//! Loading is worker-only, bounded and cancellable between shared-only reads.
//! One load borrows one opaque catalog. Only MissingEntry in the selected hash
//! shard can select base Sprite. Immutable assets retain no handles or workers.

use super::{
    combat_animation::{classify_native_cues, CrossedCue, NativeAudioGroup, NativeAudioMode},
    combat_audio::{BankLimits, SoundBank},
    compositor::Canvas,
    fixed_walk::{FixedWalkError, QualifiedFixedWalk, WalkPlayback},
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    shared_action_catalog::{SharedActionCatalog, SharedCatalogError},
    sprite::{decode_world_spr_with_limits, DecodedSprite, SpriteError, SpriteFrame, SpriteLimits},
    sprite_actions::{
        ActionError, ActionFrame, ActionKey, Animation, Directionality, FrameAsset,
        SpriteActionCatalog,
    },
    teleport_effect::SpriteProvenance,
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

const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_ENCODED: usize = 64 * 1024 * 1024;
const MAX_RGBA: usize = 128 * 1024 * 1024;
const MAX_SPRITES: usize = 256;
const MAX_RECORDS: usize = 1_000_000;
const REMOTE_IDLE_DELAY: Duration = Duration::from_millis(584);

#[derive(Debug)]
pub enum PetAssetError {
    Cancelled,
    InvalidInput(&'static str),
    Unsupported(&'static str),
    Bound(&'static str),
    Catalog(ActionError),
    FixedWalk(FixedWalkError),
    Resource(ResourceError),
    Sprite(SpriteError),
    Audio(anyhow::Error),
    Render(anyhow::Error),
}
impl fmt::Display for PetAssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pet presentation: {self:?}")
    }
}
impl std::error::Error for PetAssetError {}
impl From<ActionError> for PetAssetError {
    fn from(e: ActionError) -> Self {
        Self::Catalog(e)
    }
}
impl From<FixedWalkError> for PetAssetError {
    fn from(e: FixedWalkError) -> Self {
        Self::FixedWalk(e)
    }
}
impl From<ResourceError> for PetAssetError {
    fn from(e: ResourceError) -> Self {
        Self::Resource(e)
    }
}
impl From<SpriteError> for PetAssetError {
    fn from(e: SpriteError) -> Self {
        Self::Sprite(e)
    }
}
impl From<SharedCatalogError> for PetAssetError {
    fn from(e: SharedCatalogError) -> Self {
        match e {
            SharedCatalogError::Cancelled => Self::Cancelled,
            SharedCatalogError::Bound(s) => Self::Bound(s),
            SharedCatalogError::Resource(e) => Self::Resource(e),
            SharedCatalogError::Action(e) => Self::Catalog(e),
        }
    }
}
type Result<T> = std::result::Result<T, PetAssetError>;
fn active(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(PetAssetError::Cancelled)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PetAppearance(u32);
impl PetAppearance {
    pub const ALL: [Self; 4] = [Self(5065), Self(938), Self(2145), Self(3182)];
    pub fn new(graphics_id: u32) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|p| p.0 == graphics_id)
            .ok_or(PetAssetError::Unsupported("unqualified pet form"))
    }
    pub fn graphics_id(self) -> u32 {
        self.0
    }
    pub fn shadow_graphics_id(self) -> u32 {
        self.0 + 1
    }
    fn breath_count(self) -> usize {
        if self.0 == 5065 || self.0 == 938 {
            12
        } else {
            8
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PetAction {
    Walk,
    Breath,
}
impl PetAction {
    pub fn key(self) -> ActionKey {
        ActionKey::new(match self {
            Self::Walk => 0,
            Self::Breath => 3,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PetActionProvenance {
    pub appearance: PetAppearance,
    pub action: PetAction,
    pub definition_sprite_id: u32,
    pub body_graphics_id: u32,
    pub shadow_graphics_id: u32,
}
#[derive(Clone)]
enum BodyAction {
    Walk(QualifiedFixedWalk),
    Breath(Animation),
}
#[derive(Clone)]
pub struct QualifiedPetAction {
    provenance: PetActionProvenance,
    body: BodyAction,
    groups: Vec<Option<NativeAudioGroup>>,
    // Chronological 1..N-1,0 intervals, in unpaced native milliseconds.
    starts: Vec<Duration>,
    ends: Vec<Duration>,
    ticks: Vec<u64>,
    period: Duration,
}
fn scalar(catalog: &SpriteActionCatalog, gfx: u32, key: u32, expected: &[u8]) -> Result<()> {
    let sprite = catalog
        .sprites
        .get(&gfx)
        .ok_or(PetAssetError::Unsupported("missing pet definition"))?;
    let mut values = sprite
        .attributes
        .iter()
        .filter(|a| a.key == ActionKey::new(key));
    if values.next().is_none_or(|a| a.body_bytes != expected) || values.next().is_some() {
        return Err(PetAssetError::Unsupported(
            "pet attribute identity/ambiguity",
        ));
    }
    Ok(())
}
fn native_duration(frame: &ActionFrame, fps: u32) -> Result<Duration> {
    let ms = frame
        .duration_ticks
        .checked_mul(1000)
        .ok_or(PetAssetError::Bound("duration multiplication"))?
        / fps;
    if ms == 0 || ms > u32::from(u16::MAX) {
        return Err(PetAssetError::Unsupported(
            "unpaced duration outside native WORD",
        ));
    }
    Ok(Duration::from_millis(u64::from(ms)))
}
impl QualifiedPetAction {
    pub fn from_catalog(
        catalog: &SpriteActionCatalog,
        appearance: PetAppearance,
        action: PetAction,
    ) -> Result<Self> {
        let gfx = appearance.graphics_id();
        let shadow = appearance.shadow_graphics_id();
        let source = catalog
            .sprites
            .get(&gfx)
            .ok_or(PetAssetError::Unsupported("missing pet body definition"))?;
        if source.id != gfx
            || source.graphics_id != gfx
            || source.header_issue.is_some()
            || source.image_count.is_none_or(|n| n == 0)
            || source.attributes.len() != 2
        {
            return Err(PetAssetError::Unsupported(
                "pet body header/bank/extra control",
            ));
        }
        scalar(catalog, gfx, 101, shadow.to_string().as_bytes())?;
        scalar(catalog, gfx, 102, b"10")?;
        let shadow_definition = catalog
            .sprites
            .get(&shadow)
            .ok_or(PetAssetError::Unsupported("missing pet shadow definition"))?;
        if shadow_definition.id != shadow
            || shadow_definition.graphics_id != shadow
            || shadow_definition.header_issue.is_some()
            || shadow_definition.image_count != Some(48)
            || !shadow_definition.actions.is_empty()
            || shadow_definition.attributes.len() != 1
        {
            return Err(PetAssetError::Unsupported(
                "unreviewed direct pet shadow definition",
            ));
        }
        scalar(catalog, shadow, 102, b"0")?;
        let (body, definition_id) = match action {
            PetAction::Walk => {
                let walk = QualifiedFixedWalk::from_catalog(catalog, gfx)?;
                let definition = walk.provenance().definition_sprite_id;
                (BodyAction::Walk(walk), definition)
            }
            PetAction::Breath => {
                let resolved = catalog.resolve_action(gfx, action.key())?;
                if resolved.definition_sprite_id != gfx
                    || resolved.animation.directionality != Directionality::EightWay
                    || resolved.animation.frames_per_second != 24
                    || resolved.animation.frames.len() != appearance.breath_count()
                    || resolved.animation.label_bytes.len() > 256 * 1024
                {
                    return Err(PetAssetError::Unsupported(
                        "unreviewed pet breath shape/alias/FPS",
                    ));
                }
                (
                    BodyAction::Breath(resolved.animation.clone()),
                    resolved.definition_sprite_id,
                )
            }
        };
        // No cross-action or inherited pet profile is silently substituted.
        if definition_id != gfx {
            return Err(PetAssetError::Unsupported("pet action definition alias"));
        }
        let frames = match &body {
            BodyAction::Walk(w) => w.source_frames(),
            BodyAction::Breath(a) => &a.frames,
        };
        let fps = match &body {
            BodyAction::Walk(w) => w.frames_per_second(),
            BodyAction::Breath(a) => a.frames_per_second,
        };
        if fps != 24 {
            return Err(PetAssetError::Unsupported("unreviewed pet FPS"));
        }
        let mut groups = Vec::with_capacity(frames.len());
        for frame in frames {
            if frame.events.len() > 32 {
                return Err(PetAssetError::Bound("pet cue count"));
            }
            let cues = classify_native_cues(&frame.events).map_err(PetAssetError::Audio)?;
            if cues
                .audio
                .as_ref()
                .is_some_and(|g| g.mode == NativeAudioMode::LegacyCaretDistanceMode)
                || (action == PetAction::Breath && !cues.non_audio.is_empty())
            {
                return Err(PetAssetError::Unsupported("unqualified pet event consumer"));
            }
            groups.push(cues.audio);
        }
        let mut starts = Vec::with_capacity(frames.len());
        let mut ends = Vec::with_capacity(frames.len());
        let mut ticks = Vec::with_capacity(frames.len());
        let mut total = Duration::ZERO;
        let mut tick = 0u64;
        for source in (1..frames.len()).chain(std::iter::once(0)) {
            starts.push(total);
            ticks.push(tick);
            total = total
                .checked_add(native_duration(&frames[source], fps)?)
                .ok_or(PetAssetError::Bound("cycle duration"))?;
            tick = tick
                .checked_add(u64::from(frames[source].duration_ticks))
                .ok_or(PetAssetError::Bound("cycle ticks"))?;
            ends.push(total);
        }
        let value = Self {
            provenance: PetActionProvenance {
                appearance,
                action,
                definition_sprite_id: definition_id,
                body_graphics_id: gfx,
                shadow_graphics_id: shadow,
            },
            body,
            groups,
            starts,
            ends,
            ticks,
            period: total,
        };
        for source_index in 0..value.source_frames().len() {
            for heading in 0..8 {
                let asset = value.body_asset(source_index, heading)?;
                if asset.image_id >= source.image_count.unwrap()
                    || asset.image_id >= 48
                    || asset.subframe >= 255
                {
                    return Err(PetAssetError::Unsupported(
                        "pet frame exceeds body/shadow image bank",
                    ));
                }
            }
        }
        Ok(value)
    }
    pub fn provenance(&self) -> PetActionProvenance {
        self.provenance
    }
    pub fn source_frames(&self) -> &[ActionFrame] {
        match &self.body {
            BodyAction::Walk(w) => w.source_frames(),
            BodyAction::Breath(a) => &a.frames,
        }
    }
    pub fn frames_per_second(&self) -> u32 {
        24
    }
    pub fn unpaced_cycle_duration(&self) -> Duration {
        self.period
    }
    pub fn body_asset(&self, source: usize, heading: u8) -> Result<FrameAsset> {
        Ok(match &self.body {
            BodyAction::Walk(w) => w.frame_asset(source, heading)?,
            BodyAction::Breath(a) => {
                a.frame_asset(self.provenance.body_graphics_id, source, heading)?
            }
        })
    }
    pub fn shadow_asset(&self, source: usize, heading: u8) -> Result<FrameAsset> {
        let body = self.body_asset(source, heading)?;
        Ok(FrameAsset {
            graphics_id: self.provenance.shadow_graphics_id,
            ..body
        })
    }
    pub fn sound_group(&self, source: usize) -> Result<Option<&NativeAudioGroup>> {
        self.groups
            .get(source)
            .map(|g| g.as_ref())
            .ok_or(PetAssetError::InvalidInput("source frame"))
    }
    fn start_walk(&self, heading: u8) -> Result<WalkPlayback> {
        match &self.body {
            BodyAction::Walk(w) => Ok(w.start_unpaced_step(heading)?),
            _ => Err(PetAssetError::InvalidInput("not walk")),
        }
    }
}

struct SpriteReader {
    root: PathBuf,
    base: SharedArchive,
    shards: BTreeMap<u8, SharedArchive>,
    records: usize,
}
impl SpriteReader {
    fn new(context: &SharedActionCatalog, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let base = SharedArchive::open(context.root(), SharedArchiveKind::Sprite)?;
        let records = context
            .index_records()
            .checked_add(base.len())
            .ok_or(PetAssetError::Bound("index records"))?;
        if records > MAX_RECORDS {
            return Err(PetAssetError::Bound("index records"));
        }
        Ok(Self {
            root: base.root().to_owned(),
            base,
            shards: BTreeMap::new(),
            records,
        })
    }
    fn archive(&mut self, name: &str, cancel: &AtomicBool) -> Result<&mut SharedArchive> {
        active(cancel)?;
        let shard = (name.bytes().map(u32::from).sum::<u32>() & 15) as u8;
        if !self.shards.contains_key(&shard) {
            let a = SharedArchive::open(&self.root, SharedArchiveKind::SpriteShard(shard))?;
            self.records = self
                .records
                .checked_add(a.len())
                .ok_or(PetAssetError::Bound("index records"))?;
            if self.records > MAX_RECORDS {
                return Err(PetAssetError::Bound("index records"));
            }
            self.shards.insert(shard, a);
        }
        match self.shards[&shard].entry(name) {
            Ok(_) => Ok(self.shards.get_mut(&shard).unwrap()),
            Err(ResourceError::MissingEntry(_)) => {
                self.base.entry(name)?;
                Ok(&mut self.base)
            }
            Err(e) => Err(e.into()),
        }
    }
}
struct LoadedSprite {
    provenance: SpriteProvenance,
    decoded: DecodedSprite,
}
pub struct PetAssets {
    actions: BTreeMap<(PetAppearance, PetAction), QualifiedPetAction>,
    sprites: BTreeMap<String, LoadedSprite>,
    groups: Vec<NativeAudioGroup>,
    sound: Arc<SoundBank>,
    catalog_sha: [u8; 32],
    encoded: usize,
    rgba: usize,
}
fn validate_appearances(appearances: &[PetAppearance]) -> Result<()> {
    let unique: BTreeSet<_> = appearances.iter().copied().collect();
    if appearances.is_empty() || appearances.len() > 4 || unique.len() != appearances.len() {
        return Err(PetAssetError::InvalidInput("pet roster"));
    }
    Ok(())
}
impl PetAssets {
    pub fn load(root: &Path, appearances: &[PetAppearance], cancel: &AtomicBool) -> Result<Self> {
        validate_appearances(appearances)?;
        active(cancel)?;
        let catalog = SharedActionCatalog::load(root, cancel)?;
        Self::load_with_catalog(&catalog, appearances, cancel)
    }
    pub(crate) fn load_with_catalog(
        context: &SharedActionCatalog,
        appearances: &[PetAppearance],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        validate_appearances(appearances)?;
        active(cancel)?;
        let mut actions = BTreeMap::new();
        let mut plan = BTreeMap::<String, BTreeSet<usize>>::new();
        let mut groups = Vec::new();
        for &appearance in appearances {
            for kind in [PetAction::Walk, PetAction::Breath] {
                active(cancel)?;
                let action = QualifiedPetAction::from_catalog(context.catalog(), appearance, kind)?;
                for source in 0..action.source_frames().len() {
                    if let Some(group) = action.sound_group(source)? {
                        groups.push(group.clone());
                    }
                    for heading in 0..8 {
                        for asset in [
                            action.body_asset(source, heading)?,
                            action.shadow_asset(source, heading)?,
                        ] {
                            let name = asset.file_name();
                            if name.len() > 20 {
                                return Err(PetAssetError::Bound("SPR filename"));
                            }
                            plan.entry(name)
                                .or_default()
                                .insert(asset.subframe as usize);
                        }
                    }
                }
                actions.insert((appearance, kind), action);
            }
        }
        if plan.len() > MAX_SPRITES {
            return Err(PetAssetError::Bound("unique SPR count"));
        }
        let mut reader = SpriteReader::new(context, cancel)?;
        let mut encoded = 0usize;
        for name in plan.keys() {
            let size = reader.archive(name, cancel)?.entry(name)?.size as usize;
            if !(1..=MAX_ENTRY).contains(&size) {
                return Err(PetAssetError::Bound("SPR byte length"));
            }
            encoded = encoded
                .checked_add(size)
                .ok_or(PetAssetError::Bound("encoded bytes"))?;
            if encoded > MAX_ENCODED {
                return Err(PetAssetError::Bound("encoded bytes"));
            }
        }
        let mut sprites = BTreeMap::new();
        let mut rgba = 0usize;
        for (name, subframes) in plan {
            let archive = reader.archive(&name, cancel)?;
            let kind = archive.kind();
            let bytes = archive.read_entry(&name)?;
            active(cancel)?;
            let remaining = MAX_RGBA
                .checked_sub(rgba)
                .ok_or(PetAssetError::Bound("RGBA bytes"))?;
            if remaining < 4 {
                return Err(PetAssetError::Bound("RGBA budget exhausted"));
            }
            let decoded = decode_world_spr_with_limits(
                &bytes,
                SpriteLimits {
                    max_input_bytes: MAX_ENTRY,
                    max_total_pixels: (remaining / 4).min(SpriteLimits::default().max_total_pixels),
                    ..Default::default()
                },
            )?;
            active(cancel)?;
            for subframe in subframes {
                if subframe >= decoded.frames.len() {
                    return Err(PetAssetError::Unsupported("referenced pet subframe absent"));
                }
            }
            rgba = rgba
                .checked_add(
                    decoded
                        .retained_rgba_bytes()
                        .ok_or(PetAssetError::Bound("RGBA bytes"))?,
                )
                .ok_or(PetAssetError::Bound("RGBA bytes"))?;
            if rgba > MAX_RGBA {
                return Err(PetAssetError::Bound("RGBA bytes"));
            }
            let provenance = SpriteProvenance {
                archive: kind,
                name: name.clone(),
                bytes: bytes.len() as u32,
                sha256: Sha256::digest(&bytes).into(),
            };
            sprites.insert(
                name,
                LoadedSprite {
                    provenance,
                    decoded,
                },
            );
        }
        let sound = SoundBank::load_shared(
            context.root(),
            &groups,
            BankLimits {
                max_bytes: 32 * 1024 * 1024,
                max_sounds: 128,
                max_groups: 256,
            },
            cancel,
        )
        .map_err(|e| {
            if cancel.load(Ordering::Acquire) {
                PetAssetError::Cancelled
            } else {
                PetAssetError::Audio(e)
            }
        })?;
        active(cancel)?;
        Ok(Self {
            actions,
            sprites,
            groups,
            sound: Arc::new(sound),
            catalog_sha: context.sha256(),
            encoded,
            rgba,
        })
    }
    pub fn action(
        &self,
        appearance: PetAppearance,
        kind: PetAction,
    ) -> Result<&QualifiedPetAction> {
        self.actions
            .get(&(appearance, kind))
            .ok_or(PetAssetError::InvalidInput("pet action not loaded"))
    }
    fn layer(&self, asset: FrameAsset, source: usize) -> Result<PetLayerSample<'_>> {
        let name = asset.file_name();
        let sprite = self
            .sprites
            .get(&name)
            .ok_or(PetAssetError::Unsupported("pet SPR was not preloaded"))?;
        let frame = sprite
            .decoded
            .frames
            .get(asset.subframe as usize)
            .ok_or(PetAssetError::Unsupported("pet subframe was not preloaded"))?;
        Ok(PetLayerSample {
            frame,
            provenance: &sprite.provenance,
            source_frame: source,
            graphics_id: asset.graphics_id,
            image_id: asset.image_id,
            subframe: asset.subframe as usize,
        })
    }
    pub fn frame(
        &self,
        appearance: PetAppearance,
        kind: PetAction,
        heading: u8,
        source: usize,
    ) -> Result<PetFrameSample<'_>> {
        let action = self.action(appearance, kind)?;
        Ok(PetFrameSample {
            body: self.layer(action.body_asset(source, heading)?, source)?,
            shadow: self.layer(action.shadow_asset(source, heading)?, source)?,
        })
    }
    /// Ordinary flags only. Shadow and body keep their independent raw offsets.
    pub fn draw_frame(
        &self,
        canvas: &mut Canvas,
        appearance: PetAppearance,
        kind: PetAction,
        heading: u8,
        source: usize,
        anchor: PixelPoint,
    ) -> Result<()> {
        let sample = self.frame(appearance, kind, heading, source)?;
        let s = sample.shadow.frame;
        let b = sample.body.frame;
        let sx = anchor
            .x
            .checked_add(s.x_offset)
            .ok_or(PetAssetError::Bound("shadow x"))?;
        let sy = anchor
            .y
            .checked_add(s.y_offset)
            .ok_or(PetAssetError::Bound("shadow y"))?;
        let bx = anchor
            .x
            .checked_add(b.x_offset)
            .ok_or(PetAssetError::Bound("body x"))?;
        let by = anchor
            .y
            .checked_add(b.y_offset)
            .ok_or(PetAssetError::Bound("body y"))?;
        canvas
            .blit_native_shadow_rgb555(&s.rgba, s.width, s.height, sx, sy)
            .map_err(PetAssetError::Render)?;
        canvas
            .blit_rgba(&b.rgba, b.width, b.height, bx, by)
            .map_err(PetAssetError::Render)
    }
    pub fn start_walk(&self, appearance: PetAppearance, heading: u8) -> Result<WalkPlayback> {
        self.action(appearance, PetAction::Walk)?
            .start_walk(heading)
    }
    /// Call only at the proven remote terminal/idle admission point, not merely
    /// because a local mouse button was released or a network poll was quiet.
    pub fn start_remote_idle(
        &self,
        appearance: PetAppearance,
        heading: u8,
    ) -> Result<PetIdlePlayback> {
        self.frame(appearance, PetAction::Walk, heading, 0)?;
        let breath = self.action(appearance, PetAction::Breath)?;
        breath.body_asset(0, heading)?;
        Ok(PetIdlePlayback {
            breath: breath.clone(),
            last: None,
            retired: false,
        })
    }
    pub fn sound_bank(&self) -> Arc<SoundBank> {
        self.sound.clone()
    }
    pub fn sound_groups(&self) -> &[NativeAudioGroup] {
        &self.groups
    }
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.sprites.values().map(|s| &s.provenance)
    }
    pub fn sprite_frames(&self, name: &str) -> Result<&[SpriteFrame]> {
        self.sprites
            .get(name)
            .map(|s| s.decoded.frames.as_slice())
            .ok_or(PetAssetError::InvalidInput("unknown preloaded SPR"))
    }
    pub fn sprite_metadata(&self, name: &str) -> Result<(bool, u32)> {
        self.sprites
            .get(name)
            .map(|s| (s.decoded.has_palette, s.decoded.block_table_metadata))
            .ok_or(PetAssetError::InvalidInput("unknown preloaded SPR"))
    }
    pub fn catalog_sha256(&self) -> [u8; 32] {
        self.catalog_sha
    }
    pub fn encoded_bytes(&self) -> usize {
        self.encoded
    }
    pub fn rgba_bytes(&self) -> usize {
        self.rgba
    }
}
pub struct PetLayerSample<'a> {
    pub frame: &'a SpriteFrame,
    pub provenance: &'a SpriteProvenance,
    pub source_frame: usize,
    pub graphics_id: u32,
    pub image_id: u32,
    pub subframe: usize,
}
pub struct PetFrameSample<'a> {
    pub body: PetLayerSample<'a>,
    pub shadow: PetLayerSample<'a>,
}
#[derive(Debug)]
pub struct PetIdleSample {
    pub action: PetAction,
    pub source_frame: usize,
    pub cycle: Option<u128>,
    pub crossed_cues: Vec<CrossedCue>,
    /// Explicit bounded late-paint policy: do not replay ancient breath cycles.
    pub omitted_cycles: u128,
    pub next_deadline: Option<Duration>,
}
/// Owned bounded metadata, not a reference into its renderer's asset owner.
#[derive(Clone)]
pub struct PetIdlePlayback {
    breath: QualifiedPetAction,
    last: Option<Duration>,
    retired: bool,
}
fn duration_from_nanos(ns: u128) -> Option<Duration> {
    Some(Duration::new(
        u64::try_from(ns / 1_000_000_000).ok()?,
        (ns % 1_000_000_000) as u32,
    ))
}
impl PetIdlePlayback {
    /// Same elapsed never repeats cues. At most previous+current cycles are
    /// visited; a jump over older cycles reports omission, not a sound burst.
    /// Cancellation retires permanently. No world revision/STOP is inferred.
    pub fn advance(&mut self, elapsed: Duration, cancel: &AtomicBool) -> Result<PetIdleSample> {
        if self.retired || cancel.load(Ordering::Acquire) {
            self.retired = true;
            return Err(PetAssetError::Cancelled);
        }
        if self.last.is_some_and(|last| elapsed < last) {
            return Err(PetAssetError::InvalidInput("idle elapsed regressed"));
        }
        if elapsed < REMOTE_IDLE_DELAY {
            self.last = Some(elapsed);
            return Ok(PetIdleSample {
                action: PetAction::Walk,
                source_frame: 0,
                cycle: None,
                crossed_cues: vec![],
                omitted_cycles: 0,
                next_deadline: Some(REMOTE_IDLE_DELAY),
            });
        }
        let delay = REMOTE_IDLE_DELAY.as_nanos();
        let period = self.breath.period.as_nanos();
        let now = elapsed.as_nanos();
        let cycle = (now - delay) / period;
        let within = (now - delay) % period;
        let phase = self
            .breath
            .ends
            .iter()
            .position(|end| within < end.as_nanos())
            .expect("positive qualified cycle");
        let source = if phase + 1 == self.breath.ends.len() {
            0
        } else {
            phase + 1
        };
        let previous = self
            .last
            .filter(|last| *last >= REMOTE_IDLE_DELAY)
            .map(|last| (last.as_nanos() - delay) / period);
        let first_cycle = if previous.is_some_and(|p| p + 1 == cycle) {
            cycle - 1
        } else {
            cycle
        };
        let omitted = previous.map_or(cycle, |p| cycle.saturating_sub(p + 1));
        let mut crossed = Vec::new();
        for visited in first_cycle..=cycle {
            let base = delay + visited * period;
            for (phase, start) in self.breath.starts.iter().enumerate() {
                let at = base + start.as_nanos();
                if at > now || self.last.is_some_and(|last| at <= last.as_nanos()) {
                    continue;
                }
                let frame = if phase + 1 == self.breath.starts.len() {
                    0
                } else {
                    phase + 1
                };
                for (event_index, event) in
                    self.breath.source_frames()[frame].events.iter().enumerate()
                {
                    if crossed.len() >= 1024 {
                        return Err(PetAssetError::Bound("idle crossed cues"));
                    }
                    crossed.push(CrossedCue {
                        source_frame: frame,
                        event_index,
                        tick: self.breath.ticks[phase],
                        event: event.clone(),
                    });
                }
            }
        }
        let next = duration_from_nanos(delay + cycle * period + self.breath.ends[phase].as_nanos());
        self.last = Some(elapsed);
        Ok(PetIdleSample {
            action: PetAction::Breath,
            source_frame: source,
            cycle: Some(cycle),
            crossed_cues: crossed,
            omitted_cycles: omitted,
            next_deadline: next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> SpriteActionCatalog {
        let mut text =
            String::from("#938 48 beagle\n0.walk(1 4,0.0:5 0.1:4 0.2:5 0.3:4)\n3.breath(1 12,");
        for i in 0..12 {
            if i != 0 {
                text.push(' ');
            }
            text.push_str(&format!("24.{i}:2{}", if i == 1 { "[13" } else { "" }));
        }
        text.push_str(")\n101.shadow(939)102.type(10)\n#939 48 shadow\n102.type(0)\n");
        SpriteActionCatalog::parse_for_inspection(text.as_bytes(), Default::default()).unwrap()
    }
    #[test]
    fn shadow_is_direct_same_body_image_heading_and_subframe_without_shadow_action() {
        let c = fixture();
        let p = PetAppearance::new(938).unwrap();
        assert!(c.sprites[&939].actions.is_empty());
        assert!(c.resolve_action(939, PetAction::Breath.key()).is_err());
        for kind in [PetAction::Walk, PetAction::Breath] {
            let q = QualifiedPetAction::from_catalog(&c, p, kind).unwrap();
            for source in 0..q.source_frames().len() {
                for heading in 0..8 {
                    let body = q.body_asset(source, heading).unwrap();
                    let shadow = q.shadow_asset(source, heading).unwrap();
                    assert_eq!((body.graphics_id, shadow.graphics_id), (938, 939));
                    assert_eq!(
                        (body.image_id, body.subframe),
                        (shadow.image_id, shadow.subframe)
                    );
                }
            }
        }
    }
    #[test]
    fn remote_idle_delays_once_then_breath_terminal_dwell_repeats_without_cue_replay() {
        let c = fixture();
        let q = QualifiedPetAction::from_catalog(
            &c,
            PetAppearance::new(938).unwrap(),
            PetAction::Breath,
        )
        .unwrap();
        let mut playback = PetIdlePlayback {
            breath: q.clone(),
            last: None,
            retired: false,
        };
        let cancel = AtomicBool::new(false);
        let before = playback
            .advance(Duration::from_millis(583), &cancel)
            .unwrap();
        assert_eq!((before.action, before.source_frame), (PetAction::Walk, 0));
        let first = playback.advance(REMOTE_IDLE_DELAY, &cancel).unwrap();
        assert_eq!(
            (first.action, first.source_frame, first.cycle),
            (PetAction::Breath, 1, Some(0))
        );
        assert_eq!(first.crossed_cues.len(), 1);
        assert!(playback
            .advance(REMOTE_IDLE_DELAY, &cancel)
            .unwrap()
            .crossed_cues
            .is_empty());
        let terminal = REMOTE_IDLE_DELAY + Duration::from_millis(11 * 83);
        assert_eq!(
            playback
                .advance(terminal - Duration::from_nanos(1), &cancel)
                .unwrap()
                .source_frame,
            11
        );
        assert_eq!(playback.advance(terminal, &cancel).unwrap().source_frame, 0);
        let repeat = REMOTE_IDLE_DELAY + q.period;
        assert_eq!(
            playback
                .advance(repeat - Duration::from_nanos(1), &cancel)
                .unwrap()
                .source_frame,
            0
        );
        let second = playback.advance(repeat, &cancel).unwrap();
        assert_eq!(
            (second.source_frame, second.cycle, second.crossed_cues.len()),
            (1, Some(1), 1)
        );
        assert!(playback
            .advance(repeat, &cancel)
            .unwrap()
            .crossed_cues
            .is_empty());
        let late = playback
            .advance(REMOTE_IDLE_DELAY + q.period * 100, &cancel)
            .unwrap();
        assert_eq!(
            (late.cycle, late.omitted_cycles, late.crossed_cues.len()),
            (Some(100), 98, 1)
        );
        assert!(
            playback
                .advance(Duration::MAX, &cancel)
                .unwrap()
                .crossed_cues
                .len()
                <= 24
        );
    }
    #[test]
    fn unknown_form_shadow_controls_alias_or_breath_skip_are_not_fallbacks() {
        assert!(PetAppearance::new(945).is_err());
        for case in 0..6 {
            let mut c = fixture();
            match case {
                0 => {
                    c.sprites.remove(&939);
                }
                1 => c.sprites.get_mut(&939).unwrap().graphics_id = 938,
                2 => c.sprites.get_mut(&939).unwrap().attributes[0].body_bytes = b"10".to_vec(),
                3 => c.sprites.get_mut(&938).unwrap().attributes.push(
                    super::super::sprite_actions::OpaqueAttribute {
                        key: ActionKey::new(104),
                        label_bytes: b"attr".to_vec(),
                        body_bytes: b"8".to_vec(),
                    },
                ),
                4 => {
                    let body = c.sprites[&938].actions[&PetAction::Breath.key()].clone();
                    c.sprites
                        .get_mut(&939)
                        .unwrap()
                        .actions
                        .insert(PetAction::Breath.key(), body);
                }
                _ => {
                    let super::super::sprite_actions::ActionDefinition::Animation(a) = c
                        .sprites
                        .get_mut(&938)
                        .unwrap()
                        .actions
                        .get_mut(&PetAction::Breath.key())
                        .unwrap()
                    else {
                        panic!()
                    };
                    a.frames[2]
                        .events
                        .push(super::super::sprite_actions::FrameEvent::Skip);
                }
            }
            assert!(QualifiedPetAction::from_catalog(
                &c,
                PetAppearance::new(938).unwrap(),
                PetAction::Breath
            )
            .is_err());
        }
    }
    #[test]
    fn idle_cancel_is_permanent_and_bad_elapsed_does_not_consume_cues() {
        let c = fixture();
        let q = QualifiedPetAction::from_catalog(
            &c,
            PetAppearance::new(938).unwrap(),
            PetAction::Breath,
        )
        .unwrap();
        let mut p = PetIdlePlayback {
            breath: q.clone(),
            last: None,
            retired: false,
        };
        let cancel = AtomicBool::new(false);
        p.advance(REMOTE_IDLE_DELAY, &cancel).unwrap();
        assert!(p.advance(Duration::ZERO, &cancel).is_err());
        assert!(p
            .advance(REMOTE_IDLE_DELAY, &cancel)
            .unwrap()
            .crossed_cues
            .is_empty());
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            p.advance(Duration::MAX, &cancel),
            Err(PetAssetError::Cancelled)
        ));
        cancel.store(false, Ordering::Release);
        assert!(matches!(
            p.advance(Duration::MAX, &cancel),
            Err(PetAssetError::Cancelled)
        ));
        assert!(validate_appearances(&[]).is_err());
        assert!(validate_appearances(&[PetAppearance::ALL[0]; 2]).is_err());
    }
}
