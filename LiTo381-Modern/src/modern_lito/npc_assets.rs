//! Bounded shared-only presentation assets for ten explicit Quest NPC forms.
//!
//! Not an NPC spawn/AI/combat authority. No participant/BOT identity, source SQL,
//! world cadence, pathfinding, animation scheduler or sound device is invented.
//! Hit markers remain presentation metadata (including BOTH living-armor hits).
//! Four/eight-source walk `>` is accepted only through QualifiedFixedWalk;
//! twelve-source locomotion is resource-qualified, NOT a new movement proof.
//!
//! Native sub_4E5C00 -> 4E6EA0 loads attr105 attachments with the BODY image
//! (including heading). 4E7000 uses the BODY subframe and cell, with each
//! attachment's own attr104 flags. Attr101 shadows likewise use direct body
//! image/subframe, never the shadow bank's actions (important for 145 -> 128).
//! Attr107 at 43E668 consumes TWO integers, packed into nibbles; the reviewed
//! turtle has (1,1). This metadata never grants an authoritative collision size.
//!
//! Modern flat reference composition explicitly draws shadow, body, attachment.
//! It is not native block-list ordering/RGB565/invisibility/tint parity. Only
//! ordinary bodies and attr8 attachments are qualified. No other flags are
//! silently dropped. Loading is all-or-nothing, worker-only and cancellable;
//! only MissingEntry in the selected native hash shard may fall back to base.

use super::{
    combat_animation::{classify_native_cues, NativeAudioGroup, NativeAudioMode},
    combat_audio::{BankLimits, SoundBank},
    compositor::Canvas,
    fixed_walk::QualifiedFixedWalk,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    shared_action_catalog::SharedActionCatalog,
    sprite::{decode_world_spr_with_limits, DecodedSprite, SpriteFrame, SpriteLimits},
    sprite_actions::{
        ActionFrame, ActionKey, Animation, Directionality, FrameAsset, FrameEvent,
        SpriteActionCatalog, SpriteActions,
    },
    teleport_effect::SpriteProvenance,
    world_map::PixelPoint,
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
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
const MAX_SPRITES: usize = 1024;
const MAX_RECORDS: usize = 1_000_000;

fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "NPC asset load cancelled");
    Ok(())
}

/// A visual identity only; never a native NPC ID, player class or database key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NpcAppearance(u32);
impl NpcAppearance {
    pub const ALL: [Self; 10] = [
        Self(1649),
        Self(2158),
        Self(1082),
        Self(1047),
        Self(1020),
        Self(1578),
        Self(1517),
        Self(2535),
        Self(240),
        Self(145),
    ];
    pub fn new(graphics_id: u32) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|a| a.0 == graphics_id)
            .context("unqualified shared NPC form")
    }
    pub fn graphics_id(self) -> u32 {
        self.0
    }
    pub fn shadow_graphics_id(self) -> u32 {
        if self.0 == 145 {
            128
        } else {
            self.0 + 1
        }
    }
    pub fn attachment_graphics_id(self) -> Option<u32> {
        match self.0 {
            1578 => Some(1582),
            1517 => Some(1519),
            240 => Some(242),
            _ => None,
        }
    }
    fn image_count(self) -> u32 {
        match self.0 {
            2158 => 56,
            1578 => 40,
            240 => 64,
            _ => 48,
        }
    }
    fn counts(self) -> [usize; 5] {
        match self.0 {
            1649 => [12, 8, 3, 11, 8],
            2158 => [12, 9, 3, 8, 11],
            1082 => [8, 3, 3, 7, 7],
            1047 => [8, 4, 3, 7, 6],
            1020 => [8, 7, 3, 8, 7],
            1578 => [12, 13, 3, 6, 10],
            1517 => [8, 7, 3, 8, 8],
            2535 => [12, 7, 3, 10, 11],
            240 => [4, 6, 3, 6, 7],
            145 => [4, 5, 2, 4, 6],
            _ => unreachable!(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NpcAction {
    Walk,
    Attack,
    Damage,
    Breath,
    Death,
}
impl NpcAction {
    pub const ALL: [Self; 5] = [
        Self::Walk,
        Self::Attack,
        Self::Damage,
        Self::Breath,
        Self::Death,
    ];
    pub fn key(self) -> ActionKey {
        ActionKey::new(match self {
            Self::Walk => 0,
            Self::Attack => 1,
            Self::Damage => 2,
            Self::Breath => 3,
            Self::Death => 8,
        })
    }
    fn index(self) -> usize {
        match self {
            Self::Walk => 0,
            Self::Attack => 1,
            Self::Damage => 2,
            Self::Breath => 3,
            Self::Death => 4,
        }
    }
}

fn header(catalog: &SpriteActionCatalog, id: u32, images: u32) -> Result<&SpriteActions> {
    let s = catalog
        .sprites
        .get(&id)
        .context("NPC layer definition missing")?;
    ensure!(
        s.id == id
            && s.graphics_id == id
            && s.header_issue.is_none()
            && s.image_count == Some(images),
        "unreviewed NPC layer identity/header"
    );
    ensure!(
        !s.actions.keys().any(|k| k.id >= 100),
        "NPC layer has quarantined control"
    );
    Ok(s)
}
fn attributes(sprite: &SpriteActions, expected: &[(u32, String)]) -> Result<()> {
    ensure!(
        sprite.attributes.len() == expected.len(),
        "unreviewed NPC attribute count"
    );
    for (id, body) in expected {
        let mut values = sprite
            .attributes
            .iter()
            .filter(|a| a.key == ActionKey::new(*id));
        let value = values.next().context("required NPC attribute absent")?;
        ensure!(
            values.next().is_none() && value.body_bytes == body.as_bytes(),
            "NPC attribute value/duplicate"
        );
    }
    Ok(())
}
fn profile(catalog: &SpriteActionCatalog, a: NpcAppearance) -> Result<()> {
    let body = header(catalog, a.0, a.image_count())?;
    let mut attrs = vec![
        (101, a.shadow_graphics_id().to_string()),
        (102, "10".into()),
    ];
    if a.0 == 1649 {
        attrs.push((107, "1 1".into()));
    }
    if let Some(gfx) = a.attachment_graphics_id() {
        attrs.push((105, format!("1 {gfx}")));
    }
    attributes(body, &attrs)?;
    let shadow = header(catalog, a.shadow_graphics_id(), a.image_count())?;
    if a.0 == 145 {
        // Bank128 has its own, DIFFERENT action table. Direct shadow routing
        // deliberately never resolves it, inherits its timings or plays cues.
        attributes(shadow, &[])?;
    } else {
        attributes(shadow, &[(102, "0".into())])?;
        ensure!(
            shadow.actions.is_empty(),
            "unreviewed NPC shadow action table"
        );
    }
    if let Some(gfx) = a.attachment_graphics_id() {
        let attachment = header(catalog, gfx, a.image_count())?;
        attributes(attachment, &[(102, "0".into()), (104, "8".into())])?;
        ensure!(
            attachment.actions.is_empty(),
            "unreviewed NPC attachment actions"
        );
    }
    Ok(())
}

#[derive(Clone)]
enum BodyAction {
    Ordinary(Animation),
    FixedWalk(QualifiedFixedWalk),
}
#[derive(Clone)]
pub struct QualifiedNpcAction {
    appearance: NpcAppearance,
    kind: NpcAction,
    body: BodyAction,
    groups: Vec<Option<NativeAudioGroup>>,
}
impl QualifiedNpcAction {
    pub fn from_catalog(
        catalog: &SpriteActionCatalog,
        appearance: NpcAppearance,
        kind: NpcAction,
    ) -> Result<Self> {
        profile(catalog, appearance)?;
        let body = if kind == NpcAction::Walk && appearance.counts()[0] <= 8 {
            let walk = QualifiedFixedWalk::from_catalog(catalog, appearance.0)?;
            ensure!(
                walk.provenance().definition_sprite_id == appearance.0,
                "NPC walk alias not qualified"
            );
            BodyAction::FixedWalk(walk)
        } else {
            let resolved = catalog.resolve_action(appearance.0, kind.key())?;
            ensure!(
                resolved.definition_sprite_id == appearance.0,
                "NPC action alias not qualified"
            );
            BodyAction::Ordinary(resolved.animation.clone())
        };
        let frames = match &body {
            BodyAction::Ordinary(a) => {
                ensure!(
                    a.directionality == Directionality::EightWay
                        && a.frames_per_second == 24
                        && a.label_bytes.len() <= 256 * 1024,
                    "NPC action direction/FPS/label"
                );
                &a.frames
            }
            BodyAction::FixedWalk(w) => {
                ensure!(w.frames_per_second() == 24, "NPC walk FPS");
                w.source_frames()
            }
        };
        ensure!(
            frames.len() == appearance.counts()[kind.index()],
            "unreviewed NPC action frame count"
        );
        let mut groups = Vec::with_capacity(frames.len());
        for frame in frames {
            ensure!(
                frame
                    .image_id
                    .checked_add(7)
                    .is_some_and(|i| i < appearance.image_count())
                    && frame.subframe < 255,
                "NPC source image/subframe bound"
            );
            let ms = frame
                .duration_ticks
                .checked_mul(1000)
                .context("NPC duration overflow")?
                / 24;
            ensure!(
                (1..=u32::from(u16::MAX)).contains(&ms),
                "NPC duration outside native WORD"
            );
            ensure!(frame.events.len() <= 8, "NPC source cue bound");
            let cues = classify_native_cues(&frame.events)?;
            ensure!(
                cues.audio
                    .as_ref()
                    .is_none_or(|g| g.mode != NativeAudioMode::LegacyCaretDistanceMode),
                "NPC caret audio policy unported"
            );
            ensure!(
                cues.non_audio.iter().all(|e| matches!(e, FrameEvent::Hit)
                    && kind == NpcAction::Attack
                    || matches!(e, FrameEvent::Skip) && matches!(body, BodyAction::FixedWalk(_))),
                "unqualified NPC event control"
            );
            ensure!(
                kind != NpcAction::Walk || cues.audio.is_none(),
                "NPC walk audio not qualified"
            );
            groups.push(cues.audio);
        }
        Ok(Self {
            appearance,
            kind,
            body,
            groups,
        })
    }
    pub fn appearance(&self) -> NpcAppearance {
        self.appearance
    }
    pub fn kind(&self) -> NpcAction {
        self.kind
    }
    pub fn source_frames(&self) -> &[ActionFrame] {
        match &self.body {
            BodyAction::Ordinary(a) => &a.frames,
            BodyAction::FixedWalk(w) => w.source_frames(),
        }
    }
    /// Presentation cadence only. Ordinary sources use the reviewed native
    /// 1..N-1,0 order and integer WORD durations. Twelve-source walking still
    /// does NOT grant native interpolation/camera pulses or movement authority.
    pub(crate) fn duration(&self) -> Duration {
        match &self.body {
            BodyAction::FixedWalk(w) => w.step_duration(),
            BodyAction::Ordinary(a) => Duration::from_millis(
                a.frames
                    .iter()
                    .map(|f| u64::from(f.duration_ticks * 1000 / 24))
                    .sum(),
            ),
        }
    }
    pub(crate) fn sample(&self, elapsed: Duration, heading: u8) -> Result<(usize, u128)> {
        if let BodyAction::FixedWalk(w) = &self.body {
            return Ok((
                w.start_unpaced_step(heading)?
                    .advance(elapsed, &AtomicBool::new(false))?
                    .source_frame,
                0,
            ));
        }
        ensure!(heading <= 7, "NPC sample heading");
        let duration = self.duration();
        let repeating = self.kind == NpcAction::Breath;
        if !repeating && elapsed >= duration {
            return Ok((0, 0));
        }
        let cycle = if repeating {
            elapsed.as_nanos() / duration.as_nanos()
        } else {
            0
        };
        let phase = elapsed.as_nanos() % duration.as_nanos();
        let mut end = 0u128;
        let frames = self.source_frames();
        for source in (1..frames.len()).chain(std::iter::once(0)) {
            end += u128::from(frames[source].duration_ticks * 1000 / 24) * 1_000_000;
            if phase < end {
                return Ok((source, cycle));
            }
        }
        anyhow::bail!("NPC animation phase outside qualified duration")
    }
    pub fn sound_group(&self, source: usize) -> Result<Option<&NativeAudioGroup>> {
        Ok(self
            .groups
            .get(source)
            .context("NPC cue source outside action")?
            .as_ref())
    }
    pub fn body_asset(&self, source: usize, heading: u8) -> Result<FrameAsset> {
        Ok(match &self.body {
            BodyAction::Ordinary(a) => a.frame_asset(self.appearance.0, source, heading)?,
            BodyAction::FixedWalk(w) => w.frame_asset(source, heading)?,
        })
    }
    pub fn shadow_asset(&self, source: usize, heading: u8) -> Result<FrameAsset> {
        Ok(FrameAsset {
            graphics_id: self.appearance.shadow_graphics_id(),
            ..self.body_asset(source, heading)?
        })
    }
    pub fn attachment_asset(&self, source: usize, heading: u8) -> Result<Option<FrameAsset>> {
        let body = self.body_asset(source, heading)?;
        Ok(self
            .appearance
            .attachment_graphics_id()
            .map(|graphics_id| FrameAsset {
                graphics_id,
                ..body
            }))
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
            .context("NPC index overflow")?;
        ensure!(records <= MAX_RECORDS, "NPC index budget");
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
                .context("NPC index overflow")?;
            ensure!(self.records <= MAX_RECORDS, "NPC index budget");
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
/// Shared bounded layer storage; callers must qualify every planned route first.
pub(crate) struct LoadedLayers {
    sprites: BTreeMap<String, LoadedSprite>,
    pub(crate) encoded: usize,
    pub(crate) rgba: usize,
}
impl LoadedLayers {
    pub(crate) fn load(
        context: &SharedActionCatalog,
        plan: BTreeMap<String, BTreeSet<usize>>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(plan.len() <= MAX_SPRITES, "NPC unique SPR bound");
        let mut reader = SpriteReader::new(context, cancel)?;
        let mut encoded = 0usize;
        for name in plan.keys() {
            let size = reader.archive(name, cancel)?.entry(name)?.size as usize;
            ensure!((1..=MAX_ENTRY).contains(&size), "NPC SPR entry byte bound");
            encoded = encoded.checked_add(size).context("NPC encoded overflow")?;
            ensure!(encoded <= MAX_ENCODED, "NPC total encoded budget");
        }
        let mut sprites = BTreeMap::new();
        let mut rgba = 0usize;
        for (name, subframes) in plan {
            let archive = reader.archive(&name, cancel)?;
            let kind = archive.kind();
            let bytes = archive.read_entry(&name)?;
            active(cancel)?;
            let remaining = MAX_RGBA.checked_sub(rgba).context("NPC RGBA overflow")?;
            ensure!(remaining >= 4, "NPC RGBA budget exhausted");
            let decoded = decode_world_spr_with_limits(
                &bytes,
                SpriteLimits {
                    max_input_bytes: MAX_ENTRY,
                    max_total_pixels: (remaining / 4).min(SpriteLimits::default().max_total_pixels),
                    ..Default::default()
                },
            )
            .with_context(|| format!("NPC SPR {kind:?}/{name}"))?;
            active(cancel)?;
            ensure!(
                subframes.iter().all(|&s| s < decoded.frames.len()),
                "NPC referenced subframe absent in {name}"
            );
            rgba = rgba
                .checked_add(decoded.retained_rgba_bytes().context("NPC RGBA overflow")?)
                .context("NPC RGBA overflow")?;
            ensure!(rgba <= MAX_RGBA, "NPC RGBA budget");
            for frame in &decoded.frames {
                ensure!(
                    frame.rgba.chunks_exact(4).all(|p| p[3] == 0 || p[3] == 255),
                    "NPC nonbinary source coverage"
                );
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
        active(cancel)?;
        Ok(Self {
            sprites,
            encoded,
            rgba,
        })
    }
    pub(crate) fn layer(&self, asset: FrameAsset) -> Result<NpcLayer<'_>> {
        let sprite = self
            .sprites
            .get(&asset.file_name())
            .context("NPC SPR not preloaded")?;
        let frame = sprite
            .decoded
            .frames
            .get(asset.subframe as usize)
            .context("NPC subframe not preloaded")?;
        Ok(NpcLayer {
            asset,
            frame,
            provenance: &sprite.provenance,
        })
    }
    #[cfg(test)]
    pub(crate) fn sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.sprites.values().map(|s| &s.provenance)
    }
}
pub struct NpcAssets {
    actions: BTreeMap<(NpcAppearance, NpcAction), QualifiedNpcAction>,
    sprites: BTreeMap<String, LoadedSprite>,
    groups: Vec<NativeAudioGroup>,
    sound: Arc<SoundBank>,
    catalog_sha: [u8; 32],
    encoded: usize,
    rgba: usize,
}
fn roster(appearances: &[NpcAppearance]) -> Result<()> {
    ensure!(
        !appearances.is_empty()
            && appearances.len() <= 10
            && appearances.iter().copied().collect::<BTreeSet<_>>().len() == appearances.len(),
        "invalid NPC appearance roster"
    );
    Ok(())
}
impl NpcAssets {
    pub fn load(root: &Path, appearances: &[NpcAppearance], cancel: &AtomicBool) -> Result<Self> {
        roster(appearances)?;
        active(cancel)?;
        Self::load_with_catalog(
            &SharedActionCatalog::load(root, cancel)?,
            appearances,
            cancel,
        )
    }
    pub(crate) fn load_with_catalog(
        context: &SharedActionCatalog,
        appearances: &[NpcAppearance],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        roster(appearances)?;
        active(cancel)?;
        let mut actions = BTreeMap::new();
        let mut groups = Vec::new();
        let mut plan = BTreeMap::<String, BTreeSet<usize>>::new();
        for &appearance in appearances {
            for kind in NpcAction::ALL {
                active(cancel)?;
                let action = QualifiedNpcAction::from_catalog(context.catalog(), appearance, kind)?;
                for source in 0..action.source_frames().len() {
                    if let Some(group) = action.sound_group(source)? {
                        groups.push(group.clone());
                    }
                    for heading in 0..8 {
                        for asset in [
                            Some(action.body_asset(source, heading)?),
                            Some(action.shadow_asset(source, heading)?),
                            action.attachment_asset(source, heading)?,
                        ]
                        .into_iter()
                        .flatten()
                        {
                            plan.entry(asset.file_name())
                                .or_default()
                                .insert(asset.subframe as usize);
                        }
                    }
                }
                actions.insert((appearance, kind), action);
            }
        }
        let LoadedLayers {
            sprites,
            encoded,
            rgba,
        } = LoadedLayers::load(context, plan, cancel)?;
        let sound = SoundBank::load_shared(
            context.root(),
            &groups,
            BankLimits {
                max_bytes: 32 * 1024 * 1024,
                max_sounds: 128,
                max_groups: 256,
            },
            cancel,
        )?;
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
        appearance: NpcAppearance,
        kind: NpcAction,
    ) -> Result<&QualifiedNpcAction> {
        self.actions
            .get(&(appearance, kind))
            .context("NPC action not preloaded")
    }
    fn layer(&self, asset: FrameAsset) -> Result<NpcLayer<'_>> {
        let sprite = self
            .sprites
            .get(&asset.file_name())
            .context("NPC SPR not preloaded")?;
        let frame = sprite
            .decoded
            .frames
            .get(asset.subframe as usize)
            .context("NPC subframe not preloaded")?;
        Ok(NpcLayer {
            asset,
            frame,
            provenance: &sprite.provenance,
        })
    }
    pub fn frame(
        &self,
        appearance: NpcAppearance,
        kind: NpcAction,
        heading: u8,
        source: usize,
    ) -> Result<NpcFrame<'_>> {
        let a = self.action(appearance, kind)?;
        Ok(NpcFrame {
            action: kind,
            heading,
            source,
            body: self.layer(a.body_asset(source, heading)?)?,
            shadow: self.layer(a.shadow_asset(source, heading)?)?,
            attachment: a
                .attachment_asset(source, heading)?
                .map(|a| self.layer(a))
                .transpose()?,
        })
    }
    /// Flat modern reference only. Validate all anchors before any mutation;
    /// each layer keeps its own raw SPR offsets. No camera pulse is added.
    pub fn draw_frame(
        &self,
        canvas: &mut Canvas,
        appearance: NpcAppearance,
        kind: NpcAction,
        heading: u8,
        source: usize,
        anchor: PixelPoint,
    ) -> Result<()> {
        let sample = self.frame(appearance, kind, heading, source)?;
        let mut layers = vec![(&sample.shadow, 0), (&sample.body, 1)];
        if let Some(a) = &sample.attachment {
            layers.push((a, 2));
        }
        let mut positions = Vec::with_capacity(layers.len());
        for (layer, _) in &layers {
            positions.push((
                anchor
                    .x
                    .checked_add(layer.frame.x_offset)
                    .context("NPC anchor x overflow")?,
                anchor
                    .y
                    .checked_add(layer.frame.y_offset)
                    .context("NPC anchor y overflow")?,
            ));
        }
        for ((layer, kind), (x, y)) in layers.into_iter().zip(positions) {
            let f = layer.frame;
            match kind {
                0 => canvas.blit_native_shadow_rgb555(&f.rgba, f.width, f.height, x, y)?,
                1 => canvas.blit_rgba(&f.rgba, f.width, f.height, x, y)?,
                _ => canvas.blit_screen_rgb555(&f.rgba, f.width, f.height, x, y)?,
            }
        }
        Ok(())
    }
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.sprites.values().map(|s| &s.provenance)
    }
    pub fn sprite_frames(&self, name: &str) -> Result<&[SpriteFrame]> {
        self.sprites
            .get(name)
            .map(|s| s.decoded.frames.as_slice())
            .context("unknown NPC SPR")
    }
    pub fn sprite_metadata(&self, name: &str) -> Result<(bool, u32)> {
        self.sprites
            .get(name)
            .map(|s| (s.decoded.has_palette, s.decoded.block_table_metadata))
            .context("unknown NPC SPR")
    }
    pub fn sound_groups(&self) -> &[NativeAudioGroup] {
        &self.groups
    }
    pub fn sound_bank(&self) -> Arc<SoundBank> {
        self.sound.clone()
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
pub struct NpcLayer<'a> {
    pub asset: FrameAsset,
    pub frame: &'a SpriteFrame,
    pub provenance: &'a SpriteProvenance,
}
pub struct NpcFrame<'a> {
    pub(crate) action: NpcAction,
    pub(crate) heading: u8,
    pub(crate) source: usize,
    pub body: NpcLayer<'a>,
    pub shadow: NpcLayer<'a>,
    pub attachment: Option<NpcLayer<'a>>,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::sprite_actions::{ActionDefinition, OpaqueAttribute};
    use super::*;
    fn fixture(a: NpcAppearance) -> SpriteActionCatalog {
        let mut text = format!("#{} {} npc\n", a.0, a.image_count());
        for k in NpcAction::ALL {
            let n = a.counts()[k.index()];
            text.push_str(&format!("{}.action(1 {n},", k.key().id));
            for i in 0..n {
                if i > 0 {
                    text.push(' ');
                }
                text.push_str(&format!("0.{i}:3"));
            }
            text.push_str(")\n");
        }
        text.push_str(&format!(
            "101.shadow({})102.type(10)\n",
            a.shadow_graphics_id()
        ));
        if a.0 == 1649 {
            text.push_str("107.size(1 1)\n");
        }
        if let Some(g) = a.attachment_graphics_id() {
            text.push_str(&format!("105.attach(1 {g})\n"));
        }
        text.push_str(&format!(
            "#{} {} shadow\n",
            a.shadow_graphics_id(),
            a.image_count()
        ));
        if a.0 != 145 {
            text.push_str("102.type(0)\n");
        } else {
            text.push_str("1.different(1 1,8.99:1[57)\n");
        }
        if let Some(g) = a.attachment_graphics_id() {
            text.push_str(&format!(
                "#{g} {} attachment\n102.type(0)104.attr(8)\n",
                a.image_count()
            ));
        }
        SpriteActionCatalog::parse_for_inspection(text.as_bytes(), Default::default()).unwrap()
    }
    #[test]
    fn npc_cadence_starts_at_one_holds_terminal_zero_and_bounds_huge_elapsed() {
        for a in NpcAppearance::ALL {
            for k in NpcAction::ALL {
                let q = QualifiedNpcAction::from_catalog(&fixture(a), a, k).unwrap();
                assert_eq!(q.sample(Duration::ZERO, 0).unwrap().0, 1);
                assert!(q.duration() > Duration::ZERO);
                assert!(q.sample(Duration::MAX, 7).unwrap().0 < q.source_frames().len());
                assert!(q.sample(Duration::ZERO, 8).is_err());
                if k == NpcAction::Breath {
                    assert_eq!(q.sample(q.duration(), 0).unwrap(), (1, 1));
                } else {
                    assert_eq!(q.sample(q.duration(), 0).unwrap().0, 0);
                }
            }
        }
    }
    pub(crate) fn presentation_fixture(a: NpcAppearance) -> NpcAssets {
        let mut c = fixture(a);
        // Silence sentinel is valid audio metadata and needs no file/device.
        animation(&mut c, a, NpcAction::Attack).frames[1]
            .events
            .push(FrameEvent::Sound(0));
        if let Some(frame) = animation(&mut c, a, NpcAction::Attack).frames.get_mut(6) {
            frame.events.push(FrameEvent::Sound(0));
        }
        let actions: BTreeMap<_, _> = NpcAction::ALL
            .into_iter()
            .map(|k| ((a, k), QualifiedNpcAction::from_catalog(&c, a, k).unwrap()))
            .collect();
        let mut sprites = BTreeMap::new();
        for q in actions.values() {
            for source in 0..q.source_frames().len() {
                for heading in 0..8 {
                    for asset in [
                        Some(q.body_asset(source, heading).unwrap()),
                        Some(q.shadow_asset(source, heading).unwrap()),
                        q.attachment_asset(source, heading).unwrap(),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        sprites
                            .entry(asset.file_name())
                            .or_insert_with(|| LoadedSprite {
                                provenance: SpriteProvenance {
                                    archive: SharedArchiveKind::Sprite,
                                    name: asset.file_name(),
                                    bytes: 0,
                                    sha256: [0; 32],
                                },
                                decoded: DecodedSprite {
                                    has_palette: false,
                                    block_table_metadata: 0,
                                    frames: (0..16)
                                        .map(|_| SpriteFrame {
                                            fragments: None,
                                            x_offset: 0,
                                            y_offset: 0,
                                            width: 1,
                                            height: 1,
                                            frame_type: 0,
                                            unknown1: 0,
                                            unknown2: 0,
                                            rgba: vec![80, 120, 160, 255],
                                        })
                                        .collect(),
                                },
                            });
                    }
                }
            }
        }
        NpcAssets {
            actions,
            sprites,
            groups: vec![],
            sound: Arc::new(
                SoundBank::load_shared(
                    Path::new("unused"),
                    &[],
                    BankLimits::default(),
                    &AtomicBool::new(false),
                )
                .unwrap(),
            ),
            catalog_sha: [0; 32],
            encoded: 0,
            rgba: 0,
        }
    }
    fn animation(c: &mut SpriteActionCatalog, a: NpcAppearance, k: NpcAction) -> &mut Animation {
        let ActionDefinition::Animation(action) = c
            .sprites
            .get_mut(&a.0)
            .unwrap()
            .actions
            .get_mut(&k.key())
            .unwrap()
        else {
            panic!()
        };
        action
    }
    #[test]
    fn all_fifty_actions_route_layers_by_body_source_without_shadow_action_resolution() {
        for a in NpcAppearance::ALL {
            let c = fixture(a);
            for k in NpcAction::ALL {
                let q = QualifiedNpcAction::from_catalog(&c, a, k).unwrap();
                for source in 0..q.source_frames().len() {
                    for heading in 0..8 {
                        let body = q.body_asset(source, heading).unwrap();
                        let shadow = q.shadow_asset(source, heading).unwrap();
                        assert_eq!(
                            shadow,
                            FrameAsset {
                                graphics_id: a.shadow_graphics_id(),
                                ..body
                            }
                        );
                        assert_eq!(
                            q.attachment_asset(source, heading).unwrap(),
                            a.attachment_graphics_id().map(|g| FrameAsset {
                                graphics_id: g,
                                ..body
                            })
                        );
                    }
                }
                assert!(q.body_asset(usize::MAX, 0).is_err());
                assert!(q.body_asset(0, 8).is_err());
                assert!(q.attachment_asset(0, 255).is_err());
            }
        }
    }
    #[test]
    fn two_attack_hits_and_overlap_sound_stay_metadata_not_damage_or_sprite() {
        let a = NpcAppearance::new(1578).unwrap();
        let mut c = fixture(a);
        let f = &mut animation(&mut c, a, NpcAction::Attack).frames;
        f[2].events.push(FrameEvent::Sound(731));
        f[4].events.push(FrameEvent::Hit);
        f[8].events.push(FrameEvent::Hit);
        let q = QualifiedNpcAction::from_catalog(&c, a, NpcAction::Attack).unwrap();
        assert_eq!(
            q.source_frames()
                .iter()
                .filter(|f| f.events.contains(&FrameEvent::Hit))
                .count(),
            2
        );
        assert_eq!(q.sound_group(2).unwrap().unwrap().candidates(), [731]);
        assert!(q.sound_group(4).unwrap().is_none());
        let a = NpcAppearance::new(240).unwrap();
        let mut c = fixture(a);
        animation(&mut c, a, NpcAction::Attack).frames[2]
            .events
            .push(FrameEvent::Effect(86));
        let q = QualifiedNpcAction::from_catalog(&c, a, NpcAction::Attack).unwrap();
        assert_eq!(
            q.sound_group(2).unwrap().unwrap().mode,
            NativeAudioMode::AllowOverlappingVoices
        );
        assert_eq!(q.sound_group(2).unwrap().unwrap().candidates(), [86]);
    }
    #[test]
    fn fixed_walk_consumes_original_skip_without_weakening_generic_or_twelve_frame_path() {
        for id in [1082, 1047, 1020] {
            let a = NpcAppearance::new(id).unwrap();
            let mut c = fixture(a);
            for i in [1, 3, 5, 7] {
                animation(&mut c, a, NpcAction::Walk).frames[i]
                    .events
                    .push(FrameEvent::Skip);
            }
            assert!(c.resolve_action(id, NpcAction::Walk.key()).is_err());
            let q = QualifiedNpcAction::from_catalog(&c, a, NpcAction::Walk).unwrap();
            assert!(q.source_frames()[1].events.contains(&FrameEvent::Skip));
            assert_eq!(q.body_asset(1, 7).unwrap().subframe, 1);
            assert!(c.resolve_action(id, NpcAction::Walk.key()).is_err());
        }
        let a = NpcAppearance::new(1649).unwrap();
        let mut c = fixture(a);
        animation(&mut c, a, NpcAction::Walk).frames[1]
            .events
            .push(FrameEvent::Skip);
        assert!(QualifiedNpcAction::from_catalog(&c, a, NpcAction::Walk).is_err());
    }
    #[test]
    fn missing_layer_extra_flags_bad_size_and_control_quarantine_fail_closed() {
        let a = NpcAppearance::new(1578).unwrap();
        for case in 0..8 {
            let mut c = fixture(a);
            match case {
                0 => {
                    c.sprites.remove(&1582);
                }
                1 => c.sprites.get_mut(&1582).unwrap().attributes[1].body_bytes = b"9".to_vec(),
                2 => c.sprites.get_mut(&1579).unwrap().graphics_id = 1578,
                3 => {
                    c.sprites.get_mut(&1578).unwrap().attributes[2].body_bytes =
                        b"2 1582 1582".to_vec()
                }
                4 => c
                    .sprites
                    .get_mut(&1578)
                    .unwrap()
                    .attributes
                    .push(OpaqueAttribute {
                        key: ActionKey::new(104),
                        body_bytes: b"8".to_vec(),
                        label_bytes: vec![],
                    }),
                5 => c.sprites.get_mut(&1582).unwrap().image_count = Some(0),
                6 => {
                    c.sprites.get_mut(&1578).unwrap().actions.insert(
                        ActionKey::new(105),
                        ActionDefinition::Reference {
                            sprite_id: 1582,
                            label_bytes: vec![],
                        },
                    );
                }
                _ => {
                    let duplicate = c.sprites[&1578].attributes[0].clone();
                    c.sprites.get_mut(&1578).unwrap().attributes[1] = duplicate;
                }
            }
            assert!(
                QualifiedNpcAction::from_catalog(&c, a, NpcAction::Attack).is_err(),
                "case {case}"
            );
        }
        let a = NpcAppearance::new(1649).unwrap();
        let mut c = fixture(a);
        c.sprites.get_mut(&1649).unwrap().attributes[2].body_bytes = b"1".to_vec();
        assert!(QualifiedNpcAction::from_catalog(&c, a, NpcAction::Walk).is_err());
    }
    #[test]
    fn malformed_action_bounds_audio_markers_and_aliases_are_not_sanitized() {
        let a = NpcAppearance::new(1517).unwrap();
        for case in 0..12 {
            let mut c = fixture(a);
            let k = NpcAction::Attack;
            let f = animation(&mut c, a, k);
            match case {
                0 => f.directionality = Directionality::Undirected,
                1 => f.frames_per_second = 0,
                2 => f.frames.clear(),
                3 => f.frames[1].duration_ticks = 0,
                4 => f.frames[1].duration_ticks = u32::MAX,
                5 => f.frames[1].image_id = u32::MAX,
                6 => f.frames[1].subframe = 255,
                7 => f.frames[1].events.push(FrameEvent::Overlay(1519)),
                8 => f.frames[1].events.push(FrameEvent::Sound(10000)),
                9 => f.frames[1].events.push(FrameEvent::UninterpretedNumeric {
                    marker: b'^',
                    value: 514,
                }),
                10 => f.frames[1].events.push(FrameEvent::Skip),
                _ => {
                    c.sprites.get_mut(&a.0).unwrap().actions.insert(
                        k.key(),
                        ActionDefinition::Reference {
                            sprite_id: a.shadow_graphics_id(),
                            label_bytes: vec![],
                        },
                    );
                }
            }
            assert!(
                QualifiedNpcAction::from_catalog(&c, a, k).is_err(),
                "case {case}"
            );
        }
    }
    #[test]
    fn unknown_empty_duplicate_and_precancelled_rosters_fail_before_io() {
        assert!(NpcAppearance::new(10000).is_err());
        assert!(NpcAppearance::new(0).is_err());
        let root = Path::new("not-a-shared-root");
        assert!(NpcAssets::load(root, &[], &AtomicBool::new(false))
            .unwrap_err_text()
            .contains("roster"));
        assert!(
            NpcAssets::load(root, &[NpcAppearance::ALL[0]; 2], &AtomicBool::new(false))
                .unwrap_err_text()
                .contains("roster")
        );
        assert!(
            NpcAssets::load(root, &NpcAppearance::ALL, &AtomicBool::new(true))
                .unwrap_err_text()
                .contains("cancelled")
        );
    }
    fn render_fixture() -> NpcAssets {
        let a = NpcAppearance::new(1578).unwrap();
        let c = fixture(a);
        let q = QualifiedNpcAction::from_catalog(&c, a, NpcAction::Attack).unwrap();
        let mut sprites = BTreeMap::new();
        for (gfx, rgb) in [
            (1578, [80, 120, 160]),
            (1579, [16, 24, 32]),
            (1582, [24, 48, 96]),
        ] {
            let frame = SpriteFrame {
                fragments: None,
                x_offset: 0,
                y_offset: 0,
                width: 1,
                height: 1,
                frame_type: 0,
                unknown1: 0,
                unknown2: 0,
                rgba: vec![rgb[0], rgb[1], rgb[2], 255],
            };
            let name = format!("{gfx}-0.spr");
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
                        frames: vec![frame],
                        has_palette: false,
                        block_table_metadata: 0,
                    },
                },
            );
        }
        NpcAssets {
            actions: BTreeMap::from([((a, NpcAction::Attack), q)]),
            sprites,
            groups: vec![],
            sound: Arc::new(
                SoundBank::load_shared(
                    Path::new("unused-root"),
                    &[],
                    BankLimits::default(),
                    &AtomicBool::new(false),
                )
                .unwrap(),
            ),
            catalog_sha: [0; 32],
            encoded: 0,
            rgba: 12,
        }
    }
    #[test]
    fn modern_flat_draw_keeps_independent_offsets_and_screen_attachment() {
        let assets = render_fixture();
        let a = NpcAppearance::new(1578).unwrap();
        let mut canvas = Canvas::new(3, 3).unwrap();
        canvas.clear([32, 64, 96]);
        assets
            .draw_frame(
                &mut canvas,
                a,
                NpcAction::Attack,
                0,
                0,
                PixelPoint { x: 1, y: 1 },
            )
            .unwrap();
        let mut expected = Canvas::new(3, 3).unwrap();
        expected.clear([32, 64, 96]);
        expected
            .blit_native_shadow_rgb555(&[16, 24, 32, 255], 1, 1, 1, 1)
            .unwrap();
        expected
            .blit_rgba(&[80, 120, 160, 255], 1, 1, 1, 1)
            .unwrap();
        expected
            .blit_screen_rgb555(&[24, 48, 96, 255], 1, 1, 1, 1)
            .unwrap();
        assert_eq!(canvas.rgba(), expected.rgba());
        assert_ne!(&canvas.rgba()[16..20], &[24, 48, 96, 255]);
        let mut assets = assets;
        assets.sprites.get_mut("1582-0.spr").unwrap().decoded.frames[0].x_offset = 1;
        canvas.clear([32, 64, 96]);
        assets
            .draw_frame(
                &mut canvas,
                a,
                NpcAction::Attack,
                0,
                0,
                PixelPoint { x: 1, y: 1 },
            )
            .unwrap();
        assert_eq!(&canvas.rgba()[16..20], &[80, 120, 160, 255]);
        assert_ne!(&canvas.rgba()[20..24], &[32, 64, 96, 255]);
    }
    #[test]
    fn render_errors_validate_every_layer_before_mutation_and_empty_overlay_is_no_draw() {
        let mut assets = render_fixture();
        let a = NpcAppearance::new(1578).unwrap();
        let mut canvas = Canvas::new(3, 3).unwrap();
        canvas.clear([32, 64, 96]);
        let before = canvas.rgba().to_vec();
        assets.sprites.get_mut("1582-0.spr").unwrap().decoded.frames[0].x_offset = i32::MAX;
        assert!(assets
            .draw_frame(
                &mut canvas,
                a,
                NpcAction::Attack,
                0,
                0,
                PixelPoint { x: 1, y: 1 }
            )
            .is_err());
        assert_eq!(canvas.rgba(), before);
        for (kind, heading, source) in [
            (NpcAction::Attack, 8, 0),
            (NpcAction::Attack, 0, usize::MAX),
            (NpcAction::Death, 0, 0),
        ] {
            assert!(assets
                .draw_frame(
                    &mut canvas,
                    a,
                    kind,
                    heading,
                    source,
                    PixelPoint { x: 1, y: 1 }
                )
                .is_err());
            assert_eq!(canvas.rgba(), before);
        }
        let f = &mut assets.sprites.get_mut("1582-0.spr").unwrap().decoded.frames[0];
        f.x_offset = 192;
        f.y_offset = 120;
        f.width = 0;
        f.height = 0;
        f.rgba.clear();
        assets
            .draw_frame(
                &mut canvas,
                a,
                NpcAction::Attack,
                0,
                0,
                PixelPoint { x: 1, y: 1 },
            )
            .unwrap();
        assert_eq!(&canvas.rgba()[16..20], &[80, 120, 160, 255]);
    }
    #[test]
    #[ignore = "explicit shared Lineage NPC walk resources; no device or GUI"]
    fn real_shared_npc_walk_source_geometry() {
        let root = PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
        );
        assert!(root.is_absolute());
        let cancel = AtomicBool::new(false);
        let context = SharedActionCatalog::load(&root, &cancel).unwrap();
        for form in NpcAppearance::ALL {
            let bank = NpcAssets::load_with_catalog(&context, &[form], &cancel).unwrap();
            let walk = bank.action(form, NpcAction::Walk).unwrap();
            for heading in [0, 3] {
                let values = (0..walk.source_frames().len())
                    .map(|source| {
                        let f = bank.frame(form, NpcAction::Walk, heading, source).unwrap();
                        (
                            source,
                            f.body.asset.image_id,
                            f.body.asset.subframe,
                            f.body.frame.x_offset,
                            f.body.frame.y_offset,
                            f.body.frame.width,
                            f.body.frame.height,
                        )
                    })
                    .collect::<Vec<_>>();
                eprintln!(
                    "NPC_WALK form={} heading={} duration={:?} sources={:?}",
                    form.graphics_id(),
                    heading,
                    walk.duration(),
                    values
                );
            }
        }
    }
    trait ErrorText {
        fn unwrap_err_text(self) -> String;
    }
    impl<T> ErrorText for Result<T> {
        fn unwrap_err_text(self) -> String {
            match self {
                Ok(_) => panic!("expected error"),
                Err(e) => format!("{e:#}"),
            }
        }
    }
}
