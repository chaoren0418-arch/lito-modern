//! Shared-only skill-5 presentation. It cannot choose a destination, spend MP,
//! unlock input, or execute a Hit marker. Authority supplies both world cells.
//!
//! Evidence: shared Text/list.spr SHA256 523c61ad...b9f127 defines #169/action0
//! (not the different loose TW13081901.txt), with [119 and attr104=8. All 14
//! base characters have action19 and eight directions. Native sub_4ADDA0 attaches
//! gfx169 at the actor position; sub_44EF80 and caster sub_4E45F0 both call
//! sub_4E5B50, which plays indices 1..N-1 then terminal index0. Therefore 169 is
//! six active ticks at 24 FPS, not a seven-tick looping walk animation.
//!
//! Loading/decoding is worker-only and cancellable between bounded operations;
//! completed immutable assets retain no file handles or workers. Drawing is
//! memory-only. Native hash-selected SpriteNN then Sprite-on-member-absence
//! routing is exact; no loose, LT, malformed-file, or ambiguity fallback.
//! This deliberately does not extend BaseAction or force old melee worlds to
//! load spells. It is not a general projectile, clothing, or skill-effect engine.

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

use sha2::{Digest, Sha256};

use super::{
    animation::{AnimationClock, AnimationError, PlaybackLimits},
    combat_animation::{classify_native_cues, CrossedCue, NativeAudioGroup},
    combat_audio::{BankLimits, SoundBank},
    compositor::Canvas,
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    shared_action_catalog::{SharedActionCatalog, SharedCatalogError},
    sprite::{decode_world_spr_with_limits, DecodedSprite, SpriteError, SpriteFrame, SpriteLimits},
    sprite_actions::{
        ActionError, ActionKey, Animation, Directionality, FrameEvent, SpriteActionCatalog,
    },
    world_map::PixelPoint,
    world_scene::BaseAppearance,
};

const EFFECT: u32 = 169;
const SOUND: u32 = 119;
const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_ENCODED: usize = 32 * 1024 * 1024;
const MAX_RGBA: usize = 64 * 1024 * 1024;
const MAX_SPRITES: usize = 225;
const MAX_INDEX_RECORDS: usize = 1_000_000;
const MAX_FRAMES: usize = 256;
const MAX_CUES: usize = 1024;
const NANOS: u128 = 1_000_000_000;

#[derive(Debug)]
pub enum TeleportError {
    Cancelled,
    InvalidInput(&'static str),
    Unsupported(&'static str),
    UnsupportedSound(u32),
    Bound(&'static str),
    Resource(ResourceError),
    Catalog(ActionError),
    Sprite(SpriteError),
    Clock(AnimationError),
    Render(anyhow::Error),
}
impl fmt::Display for TeleportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "teleport resource loading cancelled"),
            Self::InvalidInput(reason) => write!(f, "invalid teleport presentation: {reason}"),
            Self::Unsupported(reason) => {
                write!(f, "unsupported shared teleport presentation: {reason}")
            }
            Self::UnsupportedSound(id) => {
                write!(f, "shared teleport Sound/{id}.wav is not qualified")
            }
            Self::Bound(reason) => write!(f, "teleport presentation exceeds {reason} bound"),
            Self::Resource(error) => write!(f, "teleport shared resource: {error}"),
            Self::Catalog(error) => write!(f, "teleport action definition: {error}"),
            Self::Sprite(error) => write!(f, "teleport SPR: {error}"),
            Self::Clock(error) => write!(f, "teleport timeline: {error}"),
            Self::Render(error) => write!(f, "teleport drawing: {error}"),
        }
    }
}
impl std::error::Error for TeleportError {}
impl From<SharedCatalogError> for TeleportError {
    fn from(error: SharedCatalogError) -> Self {
        match error {
            SharedCatalogError::Cancelled => Self::Cancelled,
            SharedCatalogError::Bound(reason) => Self::Bound(reason),
            SharedCatalogError::Resource(error) => Self::Resource(error),
            SharedCatalogError::Action(error) => Self::Catalog(error),
        }
    }
}
impl From<ResourceError> for TeleportError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}
impl From<ActionError> for TeleportError {
    fn from(value: ActionError) -> Self {
        Self::Catalog(value)
    }
}
impl From<SpriteError> for TeleportError {
    fn from(value: SpriteError) -> Self {
        Self::Sprite(value)
    }
}
impl From<AnimationError> for TeleportError {
    fn from(value: AnimationError) -> Self {
        Self::Clock(value)
    }
}
type Result<T> = std::result::Result<T, TeleportError>;

fn active(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(TeleportError::Cancelled)
    } else {
        Ok(())
    }
}
fn reserve<T>(count: usize) -> Result<Vec<T>> {
    let mut value = Vec::new();
    value
        .try_reserve_exact(count)
        .map_err(|_| TeleportError::Bound("allocation"))?;
    Ok(value)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct SpriteKey {
    gfx: u32,
    image: u32,
}
impl SpriteKey {
    fn name(self) -> String {
        format!("{}-{}.spr", self.gfx, self.image)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteProvenance {
    pub archive: SharedArchiveKind,
    pub name: String,
    pub bytes: u32,
    pub sha256: [u8; 32],
}
struct LoadedSprite {
    provenance: SpriteProvenance,
    decoded: DecodedSprite,
}

/// The exact index handles are only owned during loading, not by the UI.
struct SpriteReader {
    root: PathBuf,
    base: SharedArchive,
    shards: BTreeMap<u8, SharedArchive>,
    records: usize,
}
impl SpriteReader {
    fn open(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let base = SharedArchive::open(root, SharedArchiveKind::Sprite)?;
        active(cancel)?;
        let records = base.len();
        if records > MAX_INDEX_RECORDS {
            return Err(TeleportError::Bound("index records"));
        }
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
                .ok_or(TeleportError::Bound("index records"))?;
            if self.records > MAX_INDEX_RECORDS {
                return Err(TeleportError::Bound("index records"));
            }
            self.shards.insert(shard, archive);
        }
        active(cancel)?;
        match self.shards[&shard].entry(name) {
            Ok(_) => Ok(self.shards.get_mut(&shard).unwrap()),
            Err(ResourceError::MissingEntry(_)) => {
                self.base.entry(name)?;
                Ok(&mut self.base)
            }
            Err(error) => Err(error.into()),
        }
    }
}

#[derive(Clone)]
struct Sequence {
    animation: Animation,
    /// Positions 0..N-2 are source frames 1..N-1; position N-1 is terminal0.
    starts: Vec<u64>,
    ends: Vec<u64>,
    total: u64,
}
impl Sequence {
    fn new(animation: &Animation) -> Result<Arc<Self>> {
        if animation.frames.len() < 2 {
            return Err(TeleportError::Unsupported("one-shot lacks active frames"));
        }
        let clock = AnimationClock::with_limits(
            animation,
            PlaybackLimits {
                max_frames: MAX_FRAMES,
                max_events: MAX_CUES,
            },
        )?;
        let animation = clock.animation();
        let mut starts = reserve(animation.frames.len())?;
        let mut ends = reserve(animation.frames.len())?;
        let mut total = 0u64;
        for frame in &animation.frames[1..] {
            starts.push(total);
            total = total
                .checked_add(u64::from(frame.duration_ticks))
                .ok_or(TeleportError::Bound("ticks"))?;
            ends.push(total);
        }
        if total == 0 {
            return Err(TeleportError::Unsupported("zero active duration"));
        }
        starts.push(total);
        ends.push(total);
        Ok(Arc::new(Self {
            animation: animation.clone(),
            starts,
            ends,
            total,
        }))
    }
    fn source_index(&self, position: usize) -> usize {
        if position + 1 == self.starts.len() {
            0
        } else {
            position + 1
        }
    }
    fn tick(&self, elapsed: Duration) -> Result<u64> {
        let scaled = elapsed
            .as_nanos()
            .checked_mul(u128::from(self.animation.frames_per_second))
            .ok_or(TeleportError::Bound("elapsed time"))?;
        Ok((scaled / NANOS).min(u128::from(self.total)) as u64)
    }
    fn sample(&self, elapsed: Duration) -> Result<(usize, bool)> {
        let tick = self.tick(elapsed)?;
        let finished = tick == self.total;
        Ok((
            if finished {
                0
            } else {
                self.ends.partition_point(|end| *end <= tick) + 1
            },
            finished,
        ))
    }
    fn duration(&self) -> Duration {
        // max256 u32 durations at at least1FPS fit u64 seconds; ceil only once.
        let nanos =
            (u128::from(self.total) * NANOS).div_ceil(u128::from(self.animation.frames_per_second));
        Duration::new((nanos / NANOS) as u64, (nanos % NANOS) as u32)
    }
}

pub struct TeleportSample {
    pub source_frame: usize,
    pub finished: bool,
    pub crossed_cues: Vec<CrossedCue>,
}
pub struct TeleportPlayback {
    sequence: Arc<Sequence>,
    next_event: usize,
    last_elapsed: Option<Duration>,
}
impl TeleportPlayback {
    pub fn advance(&mut self, elapsed: Duration) -> Result<TeleportSample> {
        if self.last_elapsed.is_some_and(|last| elapsed < last) {
            return Err(TeleportError::InvalidInput("elapsed time regressed"));
        }
        let tick = self.sequence.tick(elapsed)?;
        let end = self.sequence.starts.partition_point(|start| *start <= tick);
        let mut count = 0usize;
        for position in self.next_event..end {
            count += self.sequence.animation.frames[self.sequence.source_index(position)]
                .events
                .len();
        }
        if count > MAX_CUES {
            return Err(TeleportError::Bound("crossed cues"));
        }
        let mut crossed_cues = reserve(count)?;
        for position in self.next_event..end {
            let source_frame = self.sequence.source_index(position);
            for (event_index, event) in self.sequence.animation.frames[source_frame]
                .events
                .iter()
                .enumerate()
            {
                crossed_cues.push(CrossedCue {
                    source_frame,
                    event_index,
                    tick: self.sequence.starts[position],
                    event: event.clone(),
                });
            }
        }
        let (source_frame, finished) = self.sequence.sample(elapsed)?;
        self.next_event = end;
        self.last_elapsed = Some(elapsed);
        Ok(TeleportSample {
            source_frame,
            finished,
            crossed_cues,
        })
    }
}

struct BoundSequence {
    sequence: Arc<Sequence>,
    gfx: u32,
}
pub struct VisualSample<'a> {
    pub frame: &'a SpriteFrame,
    pub source_frame: usize,
    pub finished: bool,
    pub graphics_id: u32,
    pub sprite_name: &'a str,
    pub subframe: usize,
}

pub struct TeleportAssets {
    effect: BoundSequence,
    casters: BTreeMap<BaseAppearance, BoundSequence>,
    sprites: BTreeMap<SpriteKey, LoadedSprite>,
    sound: Arc<SoundBank>,
    group: NativeAudioGroup,
    catalog_sha256: [u8; 32],
    encoded: usize,
    rgba: usize,
}
impl TeleportAssets {
    /// Worker-only: fixed Text/list.spr, one effect and at most14 unique base
    /// appearances. Duplicate appearances are coalesced, never per-player caches.
    /// Partial assets are not published; any invalid WAV/SPR/action fails closed.
    pub fn load(root: &Path, appearances: &[BaseAppearance], cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let wanted = Self::wanted_appearances(appearances)?;
        let catalog = SharedActionCatalog::load(root, cancel)?;
        Self::load_selected(&catalog, wanted, cancel)
    }

    pub(crate) fn load_with_catalog(
        catalog: &SharedActionCatalog,
        appearances: &[BaseAppearance],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let wanted = Self::wanted_appearances(appearances)?;
        Self::load_selected(catalog, wanted, cancel)
    }

    fn wanted_appearances(appearances: &[BaseAppearance]) -> Result<BTreeSet<BaseAppearance>> {
        if appearances.is_empty() || appearances.len() > 100 {
            return Err(TeleportError::InvalidInput("appearance count"));
        }
        let mut wanted = BTreeSet::new();
        for appearance in appearances {
            appearance
                .logical_sprite_id()
                .map_err(|_| TeleportError::InvalidInput("base class/sex"))?;
            wanted.insert(*appearance);
        }
        if wanted.len() > 14 {
            return Err(TeleportError::Bound("base appearances"));
        }
        Ok(wanted)
    }

    fn load_selected(
        context: &SharedActionCatalog,
        wanted: BTreeSet<BaseAppearance>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        if context.byte_length() > MAX_ENTRY {
            return Err(TeleportError::Bound("action catalog"));
        }
        let root = context.root();
        let catalog_sha256 = context.sha256();
        let catalog = context.catalog();
        let effect = qualify_effect(&catalog)?;
        let mut casters = BTreeMap::new();
        for appearance in wanted {
            active(cancel)?;
            casters.insert(appearance, qualify_caster(&catalog, appearance)?);
        }
        let group = classify_native_cues(&[FrameEvent::Sound(SOUND)])
            .map_err(|_| TeleportError::Unsupported("sound cue classification"))?
            .audio
            .ok_or(TeleportError::Unsupported("missing sound cue group"))?;
        let mut reader = SpriteReader::open(root, cancel)?;
        let mut sprites = BTreeMap::new();
        let mut encoded = 0usize;
        let mut rgba = 0usize;
        for bound in std::iter::once(&effect).chain(casters.values()) {
            let headings = if bound.sequence.animation.directionality == Directionality::EightWay {
                8
            } else {
                1
            };
            for heading in 0..headings {
                for (index, _) in bound.sequence.animation.frames.iter().enumerate() {
                    active(cancel)?;
                    let frame = bound
                        .sequence
                        .animation
                        .frame_asset(bound.gfx, index, heading)?;
                    let key = SpriteKey {
                        gfx: frame.graphics_id,
                        image: frame.image_id,
                    };
                    if !sprites.contains_key(&key) {
                        if sprites.len() >= MAX_SPRITES {
                            return Err(TeleportError::Bound("sprite entries"));
                        }
                        let name = key.name();
                        let archive = reader.archive_for(&name, cancel)?;
                        let size = archive.entry(&name)?.size as usize;
                        if size == 0 || size > MAX_ENTRY || size > MAX_ENCODED - encoded {
                            return Err(TeleportError::Bound("encoded sprites"));
                        }
                        let kind = archive.kind();
                        active(cancel)?;
                        let bytes = archive.read_entry(&name)?;
                        active(cancel)?;
                        if bytes.len() != size {
                            return Err(TeleportError::Unsupported("SPR length changed"));
                        }
                        let remaining = (MAX_RGBA - rgba) / 4;
                        if remaining == 0 {
                            return Err(TeleportError::Bound("decoded sprites"));
                        }
                        let decoded = decode_world_spr_with_limits(
                            &bytes,
                            SpriteLimits {
                                max_input_bytes: MAX_ENTRY,
                                max_total_pixels: remaining,
                                ..Default::default()
                            },
                        )?;
                        active(cancel)?;
                        let count = decoded
                            .retained_rgba_bytes()
                            .ok_or(TeleportError::Bound("decoded sprites"))?;
                        if count > MAX_RGBA - rgba {
                            return Err(TeleportError::Bound("decoded sprites"));
                        }
                        encoded += size;
                        rgba += count;
                        sprites.insert(
                            key,
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
                    if frame.subframe as usize >= sprites[&key].decoded.frames.len() {
                        return Err(TeleportError::Unsupported("missing SPR subframe"));
                    }
                }
            }
        }
        drop(reader);
        active(cancel)?;
        let sound = SoundBank::load_shared(
            root,
            std::slice::from_ref(&group),
            BankLimits {
                max_bytes: 1024 * 1024,
                max_sounds: 1,
                max_groups: 1,
            },
            cancel,
        )
        .map_err(|_| {
            if cancel.load(Ordering::Acquire) {
                TeleportError::Cancelled
            } else {
                TeleportError::UnsupportedSound(SOUND)
            }
        })?;
        active(cancel)?;
        Ok(Self {
            effect,
            casters,
            sprites,
            sound: Arc::new(sound),
            group,
            catalog_sha256,
            encoded,
            rgba,
        })
    }
    pub fn catalog_sha256(&self) -> [u8; 32] {
        self.catalog_sha256
    }
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.sprites.values().map(|sprite| &sprite.provenance)
    }
    pub fn encoded_bytes(&self) -> usize {
        self.encoded
    }
    pub fn rgba_bytes(&self) -> usize {
        self.rgba
    }
    pub fn sound_bank(&self) -> Arc<SoundBank> {
        Arc::clone(&self.sound)
    }
    pub fn sound_group(&self) -> &NativeAudioGroup {
        &self.group
    }
    pub fn effect_duration(&self) -> Duration {
        self.effect.sequence.duration()
    }
    pub fn cast_duration(&self, appearance: BaseAppearance) -> Result<Duration> {
        Ok(self.caster(appearance)?.sequence.duration())
    }
    pub fn start_effect(&self) -> TeleportPlayback {
        playback(&self.effect.sequence)
    }
    pub fn start_cast(&self, appearance: BaseAppearance) -> Result<TeleportPlayback> {
        Ok(playback(&self.caster(appearance)?.sequence))
    }
    fn caster(&self, appearance: BaseAppearance) -> Result<&BoundSequence> {
        self.casters
            .get(&appearance)
            .ok_or(TeleportError::Unsupported("caster was not preloaded"))
    }
    fn frame<'a>(
        &'a self,
        bound: &BoundSequence,
        heading: u8,
        elapsed: Duration,
    ) -> Result<VisualSample<'a>> {
        if heading > 7 {
            return Err(TeleportError::InvalidInput("heading"));
        }
        let (source_frame, finished) = bound.sequence.sample(elapsed)?;
        let asset = bound
            .sequence
            .animation
            .frame_asset(bound.gfx, source_frame, heading)?;
        let sprite = self
            .sprites
            .get(&SpriteKey {
                gfx: asset.graphics_id,
                image: asset.image_id,
            })
            .ok_or(TeleportError::Unsupported("unloaded SPR"))?;
        let frame = sprite
            .decoded
            .frames
            .get(asset.subframe as usize)
            .ok_or(TeleportError::Unsupported("unloaded subframe"))?;
        Ok(VisualSample {
            frame,
            source_frame,
            finished,
            graphics_id: asset.graphics_id,
            sprite_name: &sprite.provenance.name,
            subframe: asset.subframe as usize,
        })
    }
    pub fn caster_frame(
        &self,
        appearance: BaseAppearance,
        heading: u8,
        elapsed: Duration,
    ) -> Result<VisualSample<'_>> {
        self.frame(self.caster(appearance)?, heading, elapsed)
    }
    pub fn effect_frame(&self, elapsed: Duration) -> Result<VisualSample<'_>> {
        self.frame(&self.effect, 0, elapsed)
    }
    /// The anchor is an already projected world point. Caller must retain the
    /// origin of this exact authoritative event, not reuse a moved actor anchor.
    /// At completion the effect is removed; it never remains a permanent decal.
    pub fn draw_effect(
        &self,
        canvas: &mut Canvas,
        anchor: PixelPoint,
        elapsed: Duration,
    ) -> Result<bool> {
        let sample = self.effect_frame(elapsed)?;
        if sample.finished {
            return Ok(false);
        }
        draw_screen_frame(canvas, sample.frame, anchor)?;
        Ok(true)
    }
}
fn playback(sequence: &Arc<Sequence>) -> TeleportPlayback {
    TeleportPlayback {
        sequence: Arc::clone(sequence),
        next_event: 0,
        last_elapsed: None,
    }
}

/// Pixel offsets and clipping only: no IO, timeline advance, or actor mutation.
pub fn draw_frame(canvas: &mut Canvas, frame: &SpriteFrame, anchor: PixelPoint) -> Result<()> {
    let x = anchor
        .x
        .checked_add(frame.x_offset)
        .ok_or(TeleportError::Bound("projected x"))?;
    let y = anchor
        .y
        .checked_add(frame.y_offset)
        .ok_or(TeleportError::Bound("projected y"))?;
    canvas
        .blit_rgba(&frame.rgba, frame.width, frame.height, x, y)
        .map_err(TeleportError::Render)
}

/// Explicitly selected by qualified attr104=8 effect layers only. Keep the
/// ordinary draw_frame above unchanged: tower bodies also use that helper.
/// This is the modern RGB555 floor-screen reference, not a native RGB565 or
/// external float-to-integer-thunk rounding parity claim.
pub(crate) fn draw_screen_frame(
    canvas: &mut Canvas,
    frame: &SpriteFrame,
    anchor: PixelPoint,
) -> Result<()> {
    let x = anchor
        .x
        .checked_add(frame.x_offset)
        .ok_or(TeleportError::Bound("projected x"))?;
    let y = anchor
        .y
        .checked_add(frame.y_offset)
        .ok_or(TeleportError::Bound("projected y"))?;
    canvas
        .blit_screen_rgb555(&frame.rgba, frame.width, frame.height, x, y)
        .map_err(TeleportError::Render)
}

#[cfg(test)]
pub(crate) mod screen_test_support {
    use super::*;
    pub(crate) fn background() -> Canvas {
        let mut canvas = Canvas::new(800, 600).unwrap();
        canvas.clear([82, 99, 123]);
        canvas
    }
    /// Independent complement formulation, matching the C#1024-pair oracle.
    /// This does not call either production screen helper or screen compositor.
    pub(crate) fn reference(base: &Canvas, frame: &SpriteFrame, anchor: PixelPoint) -> Vec<u8> {
        assert_eq!(
            frame.rgba.len(),
            frame.width as usize * frame.height as usize * 4
        );
        assert!(frame.rgba.chunks_exact(4).all(|p| matches!(p[3], 0 | 255)));
        let mut out = base.rgba().to_vec();
        let x = i64::from(anchor.x) + i64::from(frame.x_offset);
        let y = i64::from(anchor.y) + i64::from(frame.y_offset);
        for dy in y.max(0)..(y + i64::from(frame.height)).min(i64::from(base.height())) {
            for dx in x.max(0)..(x + i64::from(frame.width)).min(i64::from(base.width())) {
                let src = (((dy - y) as usize * frame.width as usize) + (dx - x) as usize) * 4;
                if frame.rgba[src + 3] == 0 {
                    continue;
                }
                let dst = (dy as usize * base.width() as usize + dx as usize) * 4;
                for channel in 0..3 {
                    let d = u16::from(out[dst + channel] >> 3);
                    let s = u16::from(frame.rgba[src + channel] >> 3);
                    let value = (31 - ((31 - d) * (31 - s) + 30) / 31) as u8;
                    out[dst + channel] = value * 8 + value / 4;
                }
            }
        }
        out
    }
    pub(crate) fn compare(
        frame: &SpriteFrame,
        anchor: PixelPoint,
        mut draw: impl FnMut(&mut Canvas) -> bool,
    ) -> (bool, bool) {
        let base = background();
        let expected = reference(&base, frame, anchor);
        let mut actual = background();
        assert!(draw(&mut actual));
        assert_eq!(actual.rgba(), expected);
        let mut repeated = background();
        assert!(draw(&mut repeated));
        assert_eq!(
            actual.rgba(),
            repeated.rgba(),
            "same asset/anchor/time is deterministic"
        );
        let mut opaque = background();
        draw_frame(&mut opaque, frame, anchor).unwrap();
        (actual.rgba() != base.rgba(), actual.rgba() != opaque.rgba())
    }
    pub(crate) fn retired(mut draw: impl FnMut(&mut Canvas) -> bool) {
        let mut canvas = background();
        let baseline = canvas.rgba().to_vec();
        for _ in 0..2 {
            assert!(!draw(&mut canvas));
            assert_eq!(
                canvas.rgba(),
                baseline,
                "retired effect cannot leave a decal or replay"
            );
        }
    }
}

fn qualify_effect(catalog: &SpriteActionCatalog) -> Result<BoundSequence> {
    let sprite = catalog
        .sprites
        .get(&EFFECT)
        .ok_or(TeleportError::Unsupported("shared #169 is absent"))?;
    let resolved = catalog.resolve_action(EFFECT, ActionKey::new(0))?;
    let animation = resolved.animation;
    if sprite.graphics_id != EFFECT
        || resolved.definition_sprite_id != EFFECT
        || animation.directionality != Directionality::Undirected
        || animation.frames_per_second != 24
        || animation.frames.len() != 7
        || sprite.attributes.len() != 1
        || sprite.attributes[0].key != ActionKey::new(104)
        || sprite.attributes[0].body_bytes != b"8"
    {
        return Err(TeleportError::Unsupported(
            "unverified #169 controls or graphics",
        ));
    }
    for (index, frame) in animation.frames.iter().enumerate() {
        let expected_events = if index == 1 {
            vec![FrameEvent::Sound(SOUND)]
        } else {
            Vec::new()
        };
        if frame.image_id != 0
            || frame.subframe != if index == 0 { 6 } else { (index - 1) as u32 }
            || frame.duration_ticks != 1
            || frame.events != expected_events
        {
            return Err(TeleportError::Unsupported("unverified #169 action frames"));
        }
    }
    Ok(BoundSequence {
        sequence: Sequence::new(animation)?,
        gfx: EFFECT,
    })
}
fn qualify_caster(
    catalog: &SpriteActionCatalog,
    appearance: BaseAppearance,
) -> Result<BoundSequence> {
    let logical = appearance
        .logical_sprite_id()
        .map_err(|_| TeleportError::InvalidInput("base class/sex"))?;
    let sprite = catalog
        .sprites
        .get(&logical)
        .ok_or(TeleportError::Unsupported("missing base caster"))?;
    let resolved = catalog.resolve_action(logical, ActionKey::new(19))?;
    let definition = catalog
        .sprites
        .get(&resolved.definition_sprite_id)
        .ok_or(TeleportError::Unsupported("missing cast definition"))?;
    let animation = resolved.animation;
    if definition.graphics_id != sprite.graphics_id
        || animation.directionality != Directionality::EightWay
        || animation.frames_per_second != 24
        || animation
            .frames
            .iter()
            .flat_map(|frame| &frame.events)
            .any(|event| !matches!(event, FrameEvent::Hit))
    {
        return Err(TeleportError::Unsupported(
            "unverified cast19 graphics or controls",
        ));
    }
    Ok(BoundSequence {
        sequence: Sequence::new(animation)?,
        gfx: sprite.graphics_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn catalog(extra: &str) -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(format!("2 0 0\n#169 1 teleport 0.fire(0 7,0.6:1 0.0:1[119 0.1:1 0.2:1 0.3:1 0.4:1 0.5:1) 104.attr(8)\n#0 48=3225 prince\n19.cast(1 4,0.0:4 40.0:0! 40.1:2 40.2:3!)\n{extra}").as_bytes(), Default::default()).unwrap()
    }
    #[test]
    fn effect_starts_at_one_sound_is_once_and_terminal_zero_is_not_a_dwell() {
        let value = qualify_effect(&catalog("")).unwrap();
        assert_eq!(value.sequence.duration(), Duration::from_millis(250));
        let mut play = playback(&value.sequence);
        let first = play.advance(Duration::ZERO).unwrap();
        assert_eq!(first.source_frame, 1);
        assert!(!first.finished);
        assert_eq!(
            first.crossed_cues,
            vec![CrossedCue {
                source_frame: 1,
                event_index: 0,
                tick: 0,
                event: FrameEvent::Sound(119)
            }]
        );
        assert!(play
            .advance(Duration::ZERO)
            .unwrap()
            .crossed_cues
            .is_empty());
        assert_eq!(
            play.advance(Duration::from_nanos(41_666_667))
                .unwrap()
                .source_frame,
            2
        );
        let end = play.advance(Duration::from_millis(250)).unwrap();
        assert_eq!(end.source_frame, 0);
        assert!(end.finished);
        assert!(play.advance(Duration::MAX).unwrap().crossed_cues.is_empty());
        assert!(play.advance(Duration::ZERO).is_err());
    }
    #[test]
    fn zero_tick_cast_hit_is_retained_not_an_authoritative_command() {
        let bound = qualify_caster(&catalog(""), BaseAppearance { class: 0, sex: 0 }).unwrap();
        assert_eq!(bound.sequence.total, 5);
        let mut play = playback(&bound.sequence);
        let first = play.advance(Duration::ZERO).unwrap();
        assert_eq!(first.source_frame, 2);
        assert_eq!(first.crossed_cues[0].event, FrameEvent::Hit);
        assert_eq!(first.crossed_cues[0].source_frame, 1);
        assert_eq!(
            play.advance(Duration::from_secs(1))
                .unwrap()
                .crossed_cues
                .len(),
            1
        );
        assert!(play
            .advance(Duration::from_secs(1))
            .unwrap()
            .crossed_cues
            .is_empty());
    }
    #[test]
    fn effect_does_not_accept_other_visual_audio_controls_or_guessed_ids() {
        let source = "1 0 0\n#169 1 teleport 0.fire(0 7,0.6:1 0.0:1[119 0.1:1 0.2:1 0.3:1 0.4:1 0.5:1) 104.attr(8)";
        for malformed in [
            source.replace("[119", "<119"),
            source.replace("[119", "[10000"),
            source.replace("104.attr(8)", "104.attr(8) 105.clothes(1 170)"),
            source.replace("#169", "#11746"),
        ] {
            let cat =
                SpriteActionCatalog::parse_for_inspection(malformed.as_bytes(), Default::default())
                    .unwrap();
            assert!(qualify_effect(&cat).is_err());
        }
    }
    #[test]
    fn drawing_uses_signed_offsets_clip_and_checked_anchor() {
        let mut canvas = Canvas::new(2, 2).unwrap();
        let frame = SpriteFrame {
            fragments: None,
            x_offset: -1,
            y_offset: -1,
            width: 2,
            height: 2,
            frame_type: 0,
            unknown1: 0,
            unknown2: 0,
            rgba: [255, 0, 0, 255].repeat(4),
        };
        draw_frame(&mut canvas, &frame, PixelPoint { x: 0, y: 0 }).unwrap();
        assert_eq!(&canvas.rgba()[..4], &[255, 0, 0, 255]);
        assert_eq!(&canvas.rgba()[4..8], &[0, 0, 0, 255]);
        assert!(draw_frame(&mut canvas, &frame, PixelPoint { x: i32::MIN, y: 0 }).is_err());
    }
    #[test]
    fn attr8_helper_is_explicit_and_does_not_change_the_generic_tower_body_path() {
        let mut frame = SpriteFrame {
            fragments: None,
            x_offset: -1,
            y_offset: 0,
            width: 2,
            height: 1,
            frame_type: 8,
            unknown1: 0,
            unknown2: 0,
            rgba: vec![255, 255, 255, 255, 0, 0, 0, 255],
        };
        let mut screen = Canvas::new(1, 1).unwrap();
        screen.clear([82, 99, 123]);
        draw_screen_frame(&mut screen, &frame, PixelPoint { x: 0, y: 0 }).unwrap();
        assert_eq!(screen.rgba(), &[82, 99, 123, 255]);
        draw_frame(&mut screen, &frame, PixelPoint { x: 0, y: 0 }).unwrap();
        assert_eq!(
            screen.rgba(),
            &[0, 0, 0, 255],
            "generic helper remains ordinary even for type8"
        );
        let before = screen.rgba().to_vec();
        frame.rgba[3] = 128; // clipped pixel must still fail preflight
        assert!(draw_screen_frame(&mut screen, &frame, PixelPoint { x: 0, y: 0 }).is_err());
        assert!(draw_screen_frame(&mut screen, &frame, PixelPoint { x: i32::MIN, y: 0 }).is_err());
        assert_eq!(screen.rgba(), before);
    }
    #[test]
    fn cancellation_and_bad_input_do_not_open_resources() {
        assert!(matches!(
            TeleportAssets::load(Path::new("absent"), &[], &AtomicBool::new(true)),
            Err(TeleportError::Cancelled)
        ));
        assert!(matches!(
            TeleportAssets::load(Path::new("absent"), &[], &AtomicBool::new(false)),
            Err(TeleportError::InvalidInput(_))
        ));
        assert!(matches!(
            TeleportAssets::load(
                Path::new("absent"),
                &[BaseAppearance { class: 7, sex: 0 }],
                &AtomicBool::new(false)
            ),
            Err(TeleportError::InvalidInput(_))
        ));
    }
    #[test]
    fn one_shot_bounds_and_unverified_caster_modifiers_remain_closed() {
        let cat = catalog("");
        let mut animation = cat
            .resolve_action(0, ActionKey::new(19))
            .unwrap()
            .animation
            .clone();
        animation.frames = vec![animation.frames[1].clone(); MAX_FRAMES + 1];
        assert!(Sequence::new(&animation).is_err());
        animation.frames.truncate(2);
        animation.frames[1].duration_ticks = 0;
        assert!(Sequence::new(&animation).is_err());
        let source = "1 0 0\n#0 48=3225 prince\n19.cast(1 2,0.0:1 40.0:1[119)\n";
        let cat = SpriteActionCatalog::parse_for_inspection(source.as_bytes(), Default::default())
            .unwrap();
        assert!(matches!(
            qualify_caster(&cat, BaseAppearance { class: 0, sex: 0 }),
            Err(TeleportError::Unsupported(_))
        ));
        let wrong_bank =
            "2 0 0\n#0 48=3225 prince\n19=1\n#1 48=3227 princess\n19.cast(1 2,0.0:1 40.0:1!)\n";
        let cat =
            SpriteActionCatalog::parse_for_inspection(wrong_bank.as_bytes(), Default::default())
                .unwrap();
        assert!(qualify_caster(&cat, BaseAppearance { class: 0, sex: 0 }).is_err());
    }
    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; reads all14 cast19 directions +169 +119, memory only"]
    fn actual_shared_teleport_and_all_base_casts_match_reference() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
        let appearances: Vec<_> = (0..7)
            .flat_map(|class| (0..2).map(move |sex| BaseAppearance { class, sex }))
            .collect();
        let assets =
            TeleportAssets::load(Path::new(&root), &appearances, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            assets.catalog_sha256(),
            hex("523c61ad5cc362a56a36f58b262c84efa8b4435f017e7074d35b5834e8b9f127")
        );
        assert_eq!(assets.sprite_sources().len(), 225);
        assert_eq!(assets.encoded_bytes(), 2_768_674 + 78_980);
        let flattened: usize = assets
            .sprites
            .values()
            .flat_map(|s| &s.decoded.frames)
            .map(|f| f.rgba.len())
            .sum();
        assert_eq!(flattened, 28_341_504 + 579_456); // Independent C# flat pixels unchanged.
        let retained: usize = assets
            .sprites
            .values()
            .map(|s| s.decoded.retained_rgba_bytes().unwrap())
            .sum();
        assert_eq!(assets.rgba_bytes(), retained);
        assert!(retained > flattened);
        assert!(assets
            .sprites
            .values()
            .flat_map(|s| &s.decoded.frames)
            .all(|f| f.fragments.is_some()));
        let effect = assets
            .sprite_sources()
            .find(|source| source.name == "169-0.spr")
            .unwrap();
        assert_eq!(effect.archive, SharedArchiveKind::Sprite);
        assert_eq!(
            effect.sha256,
            hex("1fa822be91d0eb9500ce1dc9501396a61623168786c92d4fa6fd4bf71d08b080")
        );
        assert_eq!(
            assets.sound_bank().get(119).unwrap().sha256(),
            hex("aaa46b30f6816cfa6fc609c3da5943e21c074137130cf021faa46c6accf78f16")
        );
        assert_eq!(assets.sound_bank().file_bytes(), 70_772);
        let bank = assets.sound_bank();
        let sound = bank.get(119).unwrap();
        assert_eq!(
            (
                sound.format().channels,
                sound.format().sample_rate,
                sound.format().bits_per_sample
            ),
            (1, 22_050, 16)
        );
        assert_eq!(sound.pcm_bytes().len(), 70_654);
        // Independent PakViewer SprReader.LoadRaw / CreateRgbaBytes on this
        // exact shared Sprite entry, memory-only; no Rust-derived expected pixels.
        let frames = &assets.sprites[&SpriteKey { gfx: 169, image: 0 }]
            .decoded
            .frames;
        let hashes = [
            "c607882781df963e5b41794c0ee470b3019b850773d78990ffc3fe3c3d66c925",
            "eef648562fdd97f3cbd8fcdf22c2e5edab48a8fa00a3fbba1a7867f6dab674d7",
            "80081f7f103cac7e750b50a72be3e9ab90349521197136a86d1bf76f086e65a6",
            "b3515579fc36c959a3c29bde450e79be037b1a61f3a8e0c965ee87e5a867c76d",
            "2387844c4c0883c9d61b4ab3226a815d8ac9e9df415419e563ff3d88d2bf6114",
            "3c2989db0666f49d2d29356d5375d6a6cc80215894e77859bf58d0f2aab1a397",
            "c887bc93beb6835019b3f3cd093a3802674cd92d335b782b771c9af294c7e9e2",
        ];
        let bounds = [
            (-48, -36, 120, 96),
            (-24, -84, 96, 144),
            (-24, -132, 96, 192),
            (-24, -180, 96, 240),
            (-24, -264, 96, 324),
            (-24, -288, 96, 336),
            (0, -300, 72, 204),
        ];
        for (index, frame) in frames.iter().enumerate() {
            assert_eq!(format!("{:x}", Sha256::digest(&frame.rgba)), hashes[index]);
            assert_eq!(
                (frame.x_offset, frame.y_offset, frame.width, frame.height),
                bounds[index]
            );
        }
        let totals = [
            [33, 35],
            [34, 32],
            [29, 31],
            [26, 28],
            [38, 40],
            [30, 29],
            [29, 28],
        ];
        for appearance in appearances {
            assert_eq!(
                assets.caster(appearance).unwrap().sequence.total,
                totals[appearance.class as usize][appearance.sex as usize]
            );
            for heading in 0..8 {
                assert!(
                    !assets
                        .caster_frame(appearance, heading, Duration::ZERO)
                        .unwrap()
                        .finished
                );
                assert!(
                    assets
                        .caster_frame(appearance, heading, Duration::MAX)
                        .unwrap()
                        .finished
                );
            }
        }
        let mut canvas = Canvas::new(800, 600).unwrap();
        assert!(assets
            .draw_effect(&mut canvas, PixelPoint { x: 400, y: 300 }, Duration::ZERO)
            .unwrap());
        assert!(!assets
            .draw_effect(
                &mut canvas,
                PixelPoint { x: 400, y: 300 },
                Duration::from_millis(250)
            )
            .unwrap());
        let anchor = PixelPoint { x: 400, y: 350 };
        let mut changed = (false, false);
        for ms in [0, 50, 125, 249] {
            let elapsed = Duration::from_millis(ms);
            let sample = assets.effect_frame(elapsed).unwrap();
            let result = screen_test_support::compare(sample.frame, anchor, |canvas| {
                assets.draw_effect(canvas, anchor, elapsed).unwrap()
            });
            changed.0 |= result.0;
            changed.1 |= result.1;
        }
        assert_eq!(
            changed,
            (true, true),
            "actual169 must be visible and differ from opaque rendering"
        );
        screen_test_support::retired(|canvas| {
            assets
                .draw_effect(canvas, anchor, Duration::from_millis(250))
                .unwrap()
        });
        eprintln!(
            "teleport169 +all14cast19: SPR={} encoded={} RGBA={} sound={} bytes; no playback/authority",
            assets.sprite_sources().len(),
            assets.encoded_bytes(),
            assets.rgba_bytes(),
            assets.sound_bank().file_bytes()
        );
    }
    fn hex(value: &str) -> [u8; 32] {
        std::array::from_fn(|i| u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).unwrap())
    }
}
