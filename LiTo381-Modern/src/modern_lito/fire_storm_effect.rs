//! Shared-only skill70: caster action18 and caster-origin class0 effect1819.
//!
//! Native evidence: full-disassembly.asm sub_4A6510 (line347756) accepts an
//! empty target list and dispatches list109 class0 to sub_44EF00 at the packet
//! origin. Creation is NOT launch: sub_4E0EF0 (472057) retains actor+80h;
//! sub_4E45F0 (479843) and sub_4E34C0 (476974) test caster frame flag1(!),
//! then sub_4E45C0 (477580) calls effect vtable+4 and clears that pointer.
//! Vtable+4 is START (sub_44F350), not destruction. Its1819/0x71B branch
//! (167613..167778) creates three copies and offsets the original, then starts
//! all four. sub_4B9390 (387014), with the768828 table (563557..563570),
//! supplies native deltas(+2,+1),(-2,-1),(-2,+1),(+2,-1). Native doubled X
//! makes these logical deltas(+1,+1),(-1,-1),(-1,+1),(+1,-1), projected here
//! to(+48,0),(-48,0),(0,+24),(0,-24). This is ONE cast group, not one per
//! victim. Its Sound710 uses one cue owner and native same-ID suppression.
//! sub_4E5B50 (480266) enters source1..N-1, then source0. Unlike a
//! body pose, sub_44EF80 (166871) publishes that terminal frame and schedules
//! its dwell; sub_44F330 unlinks it on the NEXT callback before destruction.
//! Thus1819 has51 active ticks plus6 terminal ticks,57/24s in total, not51/24s.
//! Its source1 [710 is a native same-ID-suppressed audio cue. Source7 ! is
//! preserved as Hit: the old consumer uses it for target hurt presentation,
//! never as permission for this client to calculate HP or replay damage.
//! attr104=8 selects the native screen-family blend (488DAD/4C195D), not
//! alpha-over. Only this qualified effect uses the modern RGB555 floor-screen
//! reference; native external-thunk rounding, RGB565 and block-order pixel
//! parity are not claimed. Caster bodies retain their existing compositor.
//!
//! The current shared definition was independently read by the parent using
//! the C# parser. No LT archives, replacement graphics, guessed impact sprite,
//! projectile path, client damage, or server cooldown-derived animation speed.
//! Loading is bounded/worker-only. All families may borrow one opaque per-load
//! SharedActionCatalog; published assets retain neither catalog nor handles.

pub use super::teleport_effect::{SpriteProvenance, VisualSample};
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

pub const SKILL_ID: u32 = 70;
pub const EFFECT_GRAPHIC: u32 = 1819;
pub const SOUND_ID: u32 = 710;
const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_ENCODED: usize = 32 * 1024 * 1024;
const MAX_RGBA: usize = 64 * 1024 * 1024;
const MAX_SPRITES: usize = 225;
const MAX_INDEX_RECORDS: usize = 1_000_000;
const MAX_FRAMES: usize = 256;
const MAX_CUES: usize = 1024;
const NANOS: u128 = 1_000_000_000;
const GROUP_PIXEL_OFFSETS: [(i32, i32); 4] = [(48, 0), (-48, 0), (0, 24), (0, -24)];

#[derive(Debug)]
pub enum FireStormError {
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
impl fmt::Display for FireStormError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "fire storm resource loading cancelled"),
            Self::InvalidInput(s) => write!(f, "invalid fire storm presentation: {s}"),
            Self::Unsupported(s) => write!(f, "unsupported shared fire storm presentation: {s}"),
            Self::UnsupportedSound(id) => {
                write!(f, "shared fire storm Sound/{id}.wav is not qualified")
            }
            Self::Bound(s) => write!(f, "fire storm exceeds {s} bound"),
            Self::Resource(e) => write!(f, "fire storm shared resource: {e}"),
            Self::Catalog(e) => write!(f, "fire storm action definition: {e}"),
            Self::Sprite(e) => write!(f, "fire storm SPR: {e}"),
            Self::Clock(e) => write!(f, "fire storm timeline: {e}"),
            Self::Render(e) => write!(f, "fire storm drawing: {e}"),
        }
    }
}
impl std::error::Error for FireStormError {}
impl From<SharedCatalogError> for FireStormError {
    fn from(error: SharedCatalogError) -> Self {
        match error {
            SharedCatalogError::Cancelled => Self::Cancelled,
            SharedCatalogError::Bound(s) => Self::Bound(s),
            SharedCatalogError::Resource(e) => Self::Resource(e),
            SharedCatalogError::Action(e) => Self::Catalog(e),
        }
    }
}
impl From<ResourceError> for FireStormError {
    fn from(e: ResourceError) -> Self {
        Self::Resource(e)
    }
}
impl From<ActionError> for FireStormError {
    fn from(e: ActionError) -> Self {
        Self::Catalog(e)
    }
}
impl From<SpriteError> for FireStormError {
    fn from(e: SpriteError) -> Self {
        Self::Sprite(e)
    }
}
impl From<AnimationError> for FireStormError {
    fn from(e: AnimationError) -> Self {
        Self::Clock(e)
    }
}
type Result<T> = std::result::Result<T, FireStormError>;
fn active(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(FireStormError::Cancelled)
    } else {
        Ok(())
    }
}
fn reserve<T>(count: usize) -> Result<Vec<T>> {
    let mut value = Vec::new();
    value
        .try_reserve_exact(count)
        .map_err(|_| FireStormError::Bound("allocation"))?;
    Ok(value)
}
fn tick_duration(tick: u64, fps: u32) -> Duration {
    let nanos = (u128::from(tick) * NANOS).div_ceil(u128::from(fps));
    Duration::new((nanos / NANOS) as u64, (nanos % NANOS) as u32)
}
fn group_anchors(anchor: PixelPoint) -> Result<[PixelPoint; 4]> {
    let mut result = [anchor; 4];
    for (out, (dx, dy)) in result.iter_mut().zip(GROUP_PIXEL_OFFSETS) {
        *out = PixelPoint {
            x: anchor
                .x
                .checked_add(dx)
                .ok_or(FireStormError::Bound("group projected x"))?,
            y: anchor
                .y
                .checked_add(dy)
                .ok_or(FireStormError::Bound("group projected y"))?,
        };
    }
    Ok(result)
}
fn draw_group_frame(canvas: &mut Canvas, frame: &SpriteFrame, anchor: PixelPoint) -> Result<()> {
    let mut positions = group_anchors(anchor)?;
    // Validate every position before touching the canvas, including offscreen
    // copies. Overflow must not leave a partially drawn four-copy group.
    for at in &mut positions {
        at.x =
            at.x.checked_add(frame.x_offset)
                .ok_or(FireStormError::Bound("projected x"))?;
        at.y =
            at.y.checked_add(frame.y_offset)
                .ok_or(FireStormError::Bound("projected y"))?;
    }
    for at in positions {
        canvas
            .blit_screen_rgb555(&frame.rgba, frame.width, frame.height, at.x, at.y)
            .map_err(FireStormError::Render)?;
    }
    Ok(())
}

/// Owned one-shot order is1..N-1,0. Only the class0 effect includes source0's
/// dwell; a caster relinquishes its body override upon entering terminal0.
struct Sequence {
    animation: Animation,
    starts: Vec<u64>,
    ends: Vec<u64>,
    active_ticks: u64,
    total: u64,
}
impl Sequence {
    fn new(animation: &Animation, terminal_dwell: bool) -> Result<Arc<Self>> {
        if animation.frames.len() < 2 {
            return Err(FireStormError::Unsupported("one-shot lacks active frames"));
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
                .ok_or(FireStormError::Bound("ticks"))?;
            ends.push(total);
        }
        if total == 0 {
            return Err(FireStormError::Unsupported("zero active duration"));
        }
        let active_ticks = total;
        starts.push(total);
        if terminal_dwell {
            total = total
                .checked_add(u64::from(animation.frames[0].duration_ticks))
                .ok_or(FireStormError::Bound("ticks"))?;
        }
        ends.push(total);
        Ok(Arc::new(Self {
            animation: animation.clone(),
            starts,
            ends,
            active_ticks,
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
            .ok_or(FireStormError::Bound("elapsed time"))?;
        Ok((scaled / NANOS).min(u128::from(self.total)) as u64)
    }
    fn sample(&self, elapsed: Duration) -> Result<(usize, bool)> {
        let tick = self.tick(elapsed)?;
        let finished = tick == self.total;
        let source = if tick >= self.active_ticks {
            0
        } else {
            self.source_index(self.ends.partition_point(|end| *end <= tick))
        };
        Ok((source, finished))
    }
    fn duration(&self) -> Duration {
        tick_duration(self.total, self.animation.frames_per_second)
    }
    fn start(self: &Arc<Self>) -> FireStormPlayback {
        FireStormPlayback {
            sequence: Arc::clone(self),
            next_event: 0,
            last_elapsed: None,
        }
    }
}

pub struct FireStormSample {
    pub source_frame: usize,
    pub finished: bool,
    pub crossed_cues: Vec<CrossedCue>,
}
/// A fresh instance belongs to one authoritative event. Advancing the same
/// elapsed/revision does not replay cues; caller must discard it on world exit.
#[derive(Clone)]
pub struct FireStormPlayback {
    sequence: Arc<Sequence>,
    next_event: usize,
    last_elapsed: Option<Duration>,
}
impl FireStormPlayback {
    pub fn advance(&mut self, elapsed: Duration) -> Result<FireStormSample> {
        if self.last_elapsed.is_some_and(|last| elapsed < last) {
            return Err(FireStormError::InvalidInput("elapsed time regressed"));
        }
        let tick = self.sequence.tick(elapsed)?;
        let end = self.sequence.starts.partition_point(|start| *start <= tick);
        let mut count = 0usize;
        for position in self.next_event..end {
            count = count
                .checked_add(
                    self.sequence.animation.frames[self.sequence.source_index(position)]
                        .events
                        .len(),
                )
                .ok_or(FireStormError::Bound("crossed cues"))?;
        }
        if count > MAX_CUES {
            return Err(FireStormError::Bound("crossed cues"));
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
        Ok(FireStormSample {
            source_frame,
            finished,
            crossed_cues,
        })
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct SpriteKey {
    gfx: u32,
    image: u32,
}
impl SpriteKey {
    fn name(self) -> String {
        format!("{}-{}.spr", self.gfx, self.image)
    }
}
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
    fn open(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let base = SharedArchive::open(root, SharedArchiveKind::Sprite)?;
        active(cancel)?;
        let records = base.len();
        if records > MAX_INDEX_RECORDS {
            return Err(FireStormError::Bound("index records"));
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
                .ok_or(FireStormError::Bound("index records"))?;
            if self.records > MAX_INDEX_RECORDS {
                return Err(FireStormError::Bound("index records"));
            }
            self.shards.insert(shard, archive);
        }
        active(cancel)?;
        match self.shards[&shard].entry(name) {
            Ok(_) => Ok(self.shards.get_mut(&shard).unwrap()),
            // Native hash-selected archive then base ONLY when this member is
            // absent. An absent/corrupt archive or ambiguous entry never falls back.
            Err(ResourceError::MissingEntry(_)) => {
                self.base.entry(name)?;
                Ok(&mut self.base)
            }
            Err(e) => Err(e.into()),
        }
    }
}
struct BoundSequence {
    gfx: u32,
    sequence: Arc<Sequence>,
}

fn qualify_effect(catalog: &SpriteActionCatalog) -> Result<BoundSequence> {
    let sprite = catalog
        .sprites
        .get(&EFFECT_GRAPHIC)
        .ok_or(FireStormError::Unsupported("missing shared1819"))?;
    let resolved = catalog.resolve_action(EFFECT_GRAPHIC, ActionKey::new(0))?;
    let animation = resolved.animation;
    if sprite.graphics_id != EFFECT_GRAPHIC
        || resolved.definition_sprite_id != EFFECT_GRAPHIC
        || sprite.image_count != Some(1)
        || sprite.actions.len() != 1
        || animation.directionality != Directionality::Undirected
        || animation.frames_per_second != 24
        || animation.frames.len() != 14
        || sprite.attributes.len() != 2
        || sprite.attributes[0].key != ActionKey::new(104)
        || sprite.attributes[0].body_bytes != b"8"
        || sprite.attributes[1].key != ActionKey::new(109)
        || sprite.attributes[1].body_bytes != b"0 -1"
    {
        return Err(FireStormError::Unsupported("unreviewed1819 class/controls"));
    }
    let ticks = [6, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5];
    for (index, frame) in animation.frames.iter().enumerate() {
        let expected_events = match index {
            1 => vec![FrameEvent::Sound(SOUND_ID)],
            7 => vec![FrameEvent::Hit],
            _ => Vec::new(),
        };
        if frame.image_id != 0
            || frame.subframe != if index == 0 { 13 } else { index as u32 - 1 }
            || frame.duration_ticks != ticks[index]
            || frame.events != expected_events
        {
            return Err(FireStormError::Unsupported("unreviewed1819 frame/cue"));
        }
    }
    Ok(BoundSequence {
        gfx: EFFECT_GRAPHIC,
        sequence: Sequence::new(animation, true)?,
    })
}
fn qualify_caster(
    catalog: &SpriteActionCatalog,
    appearance: BaseAppearance,
) -> Result<BoundSequence> {
    let logical = appearance
        .logical_sprite_id()
        .map_err(|_| FireStormError::InvalidInput("base class/sex"))?;
    let sprite = catalog
        .sprites
        .get(&logical)
        .ok_or(FireStormError::Unsupported("missing base caster"))?;
    let resolved = catalog.resolve_action(logical, ActionKey::new(18))?;
    let definition = catalog
        .sprites
        .get(&resolved.definition_sprite_id)
        .ok_or(FireStormError::Unsupported("missing cast definition"))?;
    let animation = resolved.animation;
    if definition.graphics_id != sprite.graphics_id
        || animation.directionality != Directionality::EightWay
        || animation.frames_per_second != 24
        || animation.frames.len() < 2
        || animation
            .frames
            .iter()
            .flat_map(|f| &f.events)
            .any(|e| !matches!(e, FrameEvent::Hit))
        || animation.frames[0].events.contains(&FrameEvent::Hit)
        || animation
            .frames
            .iter()
            .skip(1)
            .flat_map(|f| &f.events)
            .filter(|e| **e == FrameEvent::Hit)
            .count()
            != 1
    {
        return Err(FireStormError::Unsupported(
            "unreviewed caster18 graphics/launch cue",
        ));
    }
    Ok(BoundSequence {
        gfx: sprite.graphics_id,
        sequence: Sequence::new(animation, false)?,
    })
}

pub struct FireStormAssets {
    effect: BoundSequence,
    casters: BTreeMap<BaseAppearance, BoundSequence>,
    sprites: BTreeMap<SpriteKey, LoadedSprite>,
    sound: Arc<SoundBank>,
    groups: [NativeAudioGroup; 1],
    catalog_sha256: [u8; 32],
    encoded: usize,
    rgba: usize,
}
impl FireStormAssets {
    /// All-or-nothing worker load. Input/cancellation checks precede root IO.
    pub fn load(root: &Path, appearances: &[BaseAppearance], cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let wanted = Self::wanted_appearances(appearances)?;
        let catalog = SharedActionCatalog::load(root, cancel)?;
        Self::load_selected(&catalog, wanted, cancel)
    }
    pub(crate) fn load_with_catalog(
        context: &SharedActionCatalog,
        appearances: &[BaseAppearance],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        let wanted = Self::wanted_appearances(appearances)?;
        Self::load_selected(context, wanted, cancel)
    }
    fn wanted_appearances(appearances: &[BaseAppearance]) -> Result<BTreeSet<BaseAppearance>> {
        if appearances.is_empty() || appearances.len() > 100 {
            return Err(FireStormError::InvalidInput("appearance count"));
        }
        let mut wanted = BTreeSet::new();
        for appearance in appearances {
            appearance
                .logical_sprite_id()
                .map_err(|_| FireStormError::InvalidInput("base class/sex"))?;
            wanted.insert(*appearance);
        }
        if wanted.len() > 14 {
            return Err(FireStormError::Bound("base appearances"));
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
            return Err(FireStormError::Bound("action catalog"));
        }
        let root = context.root();
        let effect = qualify_effect(context.catalog())?;
        let mut casters = BTreeMap::new();
        for appearance in wanted {
            active(cancel)?;
            casters.insert(appearance, qualify_caster(context.catalog(), appearance)?);
        }
        let group = classify_native_cues(&[FrameEvent::Sound(SOUND_ID)])
            .map_err(|_| FireStormError::Unsupported("sound cue classification"))?
            .audio
            .ok_or(FireStormError::Unsupported("missing native sound group"))?;
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
                for index in 0..bound.sequence.animation.frames.len() {
                    active(cancel)?;
                    let asset = bound
                        .sequence
                        .animation
                        .frame_asset(bound.gfx, index, heading)?;
                    let key = SpriteKey {
                        gfx: asset.graphics_id,
                        image: asset.image_id,
                    };
                    if !sprites.contains_key(&key) {
                        if sprites.len() >= MAX_SPRITES {
                            return Err(FireStormError::Bound("sprite entries"));
                        }
                        let name = key.name();
                        let archive = reader.archive_for(&name, cancel)?;
                        let size = archive.entry(&name)?.size as usize;
                        if size == 0 || size > MAX_ENTRY || size > MAX_ENCODED - encoded {
                            return Err(FireStormError::Bound("encoded sprites"));
                        }
                        let kind = archive.kind();
                        active(cancel)?;
                        let bytes = archive.read_entry(&name)?;
                        active(cancel)?;
                        if bytes.len() != size {
                            return Err(FireStormError::Unsupported("SPR length changed"));
                        }
                        let remaining = (MAX_RGBA - rgba) / 4;
                        if remaining == 0 {
                            return Err(FireStormError::Bound("decoded sprites"));
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
                            .ok_or(FireStormError::Bound("decoded sprites"))?;
                        if count > MAX_RGBA - rgba {
                            return Err(FireStormError::Bound("decoded sprites"));
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
                    if asset.subframe as usize >= sprites[&key].decoded.frames.len() {
                        return Err(FireStormError::Unsupported("missing SPR subframe"));
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
                FireStormError::Cancelled
            } else {
                FireStormError::UnsupportedSound(SOUND_ID)
            }
        })?;
        active(cancel)?;
        Ok(Self {
            effect,
            casters,
            sprites,
            sound: Arc::new(sound),
            groups: [group],
            catalog_sha256: context.sha256(),
            encoded,
            rgba,
        })
    }
    fn caster(&self, appearance: BaseAppearance) -> Result<&BoundSequence> {
        self.casters
            .get(&appearance)
            .ok_or(FireStormError::Unsupported("caster was not preloaded"))
    }
    pub fn cast_duration(&self, appearance: BaseAppearance) -> Result<Duration> {
        Ok(self.caster(appearance)?.sequence.duration())
    }
    /// Caster18 launch marker is presentation-only, not the effect's own Hit.
    pub fn cast_launch_delay(&self, appearance: BaseAppearance) -> Result<Duration> {
        let s = &self.caster(appearance)?.sequence;
        for (position, &tick) in s.starts.iter().enumerate() {
            if s.animation.frames[s.source_index(position)]
                .events
                .contains(&FrameEvent::Hit)
            {
                return Ok(tick_duration(tick, s.animation.frames_per_second));
            }
        }
        Err(FireStormError::Unsupported("caster lacks launch marker"))
    }
    pub fn effect_duration(&self) -> Duration {
        self.effect.sequence.duration()
    }
    pub fn effect_terminal_start(&self) -> Duration {
        tick_duration(
            self.effect.sequence.active_ticks,
            self.effect.sequence.animation.frames_per_second,
        )
    }
    pub fn start_effect(&self) -> FireStormPlayback {
        self.effect.sequence.start()
    }
    pub fn start_cast(&self, appearance: BaseAppearance) -> Result<FireStormPlayback> {
        Ok(self.caster(appearance)?.sequence.start())
    }
    fn frame<'a>(
        &'a self,
        bound: &BoundSequence,
        heading: u8,
        elapsed: Duration,
    ) -> Result<VisualSample<'a>> {
        if heading > 7 {
            return Err(FireStormError::InvalidInput("heading"));
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
            .ok_or(FireStormError::Unsupported("unloaded SPR"))?;
        let frame = sprite
            .decoded
            .frames
            .get(asset.subframe as usize)
            .ok_or(FireStormError::Unsupported("unloaded subframe"))?;
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
    /// Four native copies around the fixed authoritative cast-origin anchor.
    /// This is not a target-following projectile and does not move with the
    /// caster after launch. One playback supplies all four copies and its cues.
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
        draw_group_frame(canvas, sample.frame, anchor)?;
        Ok(true)
    }
    pub fn sound_bank(&self) -> Arc<SoundBank> {
        Arc::clone(&self.sound)
    }
    pub fn sound_group(&self) -> &NativeAudioGroup {
        &self.groups[0]
    }
    pub fn sound_groups(&self) -> &[NativeAudioGroup] {
        &self.groups
    }
    pub fn catalog_sha256(&self) -> [u8; 32] {
        self.catalog_sha256
    }
    pub fn sprite_sources(&self) -> impl ExactSizeIterator<Item = &SpriteProvenance> {
        self.sprites.values().map(|s| &s.provenance)
    }
    /// Read-only access to every decoded source subframe, including unused ones,
    /// for independent parent-owned known-answer tests. Names are canonical.
    pub fn sprite_frames(&self, name: &str) -> Result<&[SpriteFrame]> {
        self.sprites
            .values()
            .find(|s| s.provenance.name.eq_ignore_ascii_case(name))
            .map(|s| s.decoded.frames.as_slice())
            .ok_or(FireStormError::InvalidInput("unloaded sprite name"))
    }
    pub fn encoded_bytes(&self) -> usize {
        self.encoded
    }
    pub fn rgba_bytes(&self) -> usize {
        self.rgba
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const EFFECT_TEXT: &str = "#1819 1 firestorm 0.fire(0 14,0.13:6 0.0:3[710 0.1:3 0.2:3 0.3:4 0.4:4 0.5:4 0.6:4! 0.7:4 0.8:4 0.9:4 0.10:4 0.11:5 0.12:5) 104.attr(8) 109.effect(0 -1)";
    fn parse(text: &str) -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(text.as_bytes(), Default::default()).unwrap()
    }
    fn effect() -> BoundSequence {
        qualify_effect(&parse(EFFECT_TEXT)).unwrap()
    }
    #[test]
    fn actual_definition_keeps_terminal_dwell_and_exact_native_markers() {
        let bound = effect();
        let s = &bound.sequence;
        assert_eq!(s.active_ticks, 51);
        assert_eq!(s.total, 57);
        assert_eq!(s.duration(), Duration::from_millis(2375));
        assert_eq!(s.sample(Duration::ZERO).unwrap(), (1, false));
        assert_eq!(
            s.sample(Duration::from_nanos(2_124_999_999)).unwrap(),
            (13, false)
        );
        assert_eq!(s.sample(Duration::from_millis(2125)).unwrap(), (0, false));
        assert_eq!(
            s.sample(Duration::from_nanos(2_374_999_999)).unwrap(),
            (0, false)
        );
        assert_eq!(s.sample(Duration::from_millis(2375)).unwrap(), (0, true));
        for heading in 0..8 {
            let frame = s.animation.frame_asset(EFFECT_GRAPHIC, 0, heading).unwrap();
            assert_eq!(
                (frame.graphics_id, frame.image_id, frame.subframe),
                (1819, 0, 13)
            );
        }
        let mut p = s.start();
        let first = p.advance(Duration::ZERO).unwrap();
        assert_eq!(first.crossed_cues.len(), 1);
        assert_eq!(first.crossed_cues[0].event, FrameEvent::Sound(710));
        assert_eq!(first.crossed_cues[0].source_frame, 1);
        assert_eq!(first.crossed_cues[0].tick, 0);
        assert!(p.advance(Duration::ZERO).unwrap().crossed_cues.is_empty());
        assert!(p
            .advance(Duration::from_nanos(874_999_999))
            .unwrap()
            .crossed_cues
            .is_empty());
        let hit = p.advance(Duration::from_millis(875)).unwrap();
        assert_eq!(hit.crossed_cues.len(), 1);
        assert_eq!(hit.crossed_cues[0].event, FrameEvent::Hit);
        assert_eq!(
            (hit.crossed_cues[0].source_frame, hit.crossed_cues[0].tick),
            (7, 21)
        );
        assert!(p
            .advance(Duration::from_millis(875))
            .unwrap()
            .crossed_cues
            .is_empty());
        assert!(p.advance(Duration::MAX).unwrap().finished);
        assert!(p.advance(Duration::MAX).unwrap().crossed_cues.is_empty());
        assert!(matches!(
            p.advance(Duration::ZERO),
            Err(FireStormError::InvalidInput(_))
        ));
    }
    #[test]
    fn late_advance_is_bounded_ordered_and_does_not_replay_events() {
        let mut p = effect().sequence.start();
        let sample = p.advance(Duration::MAX).unwrap();
        assert!(sample.finished);
        assert_eq!(sample.source_frame, 0);
        assert_eq!(
            sample
                .crossed_cues
                .iter()
                .map(|c| (c.tick, c.event.clone()))
                .collect::<Vec<_>>(),
            [(0, FrameEvent::Sound(710)), (21, FrameEvent::Hit)]
        );
        assert!(p.advance(Duration::MAX).unwrap().crossed_cues.is_empty());
    }
    #[test]
    fn altered_shape_chain_cues_or_fps_are_not_silent_fallbacks() {
        for (old, new) in [
            ("109.effect(0 -1)", "109.effect(1 -1)"),
            ("109.effect(0 -1)", "109.effect(0 219)"),
            ("104.attr(8)", "104.attr(0)"),
            ("0.fire(0 14", "0.fire(1 14"),
            ("0.13:6", "0.13:0"),
            ("0.0:3[710", "0.0:3<710"),
            ("0.6:4!", "0.6:4"),
            ("0.6:4!", "0.6:4[710"),
            ("0.1:3", "0.1:3!"),
            ("#1819 1", "#1819 2"),
        ] {
            assert!(
                qualify_effect(&parse(&EFFECT_TEXT.replace(old, new))).is_err(),
                "{old} -> {new}"
            );
        }
        assert!(
            qualify_effect(&parse(&EFFECT_TEXT.replace("0.fire", "110.fps(12) 0.fire"))).is_err()
        );
        assert!(qualify_effect(&parse(&EFFECT_TEXT.replace("#1819", "#189"))).is_err());
    }
    fn caster_catalog() -> SpriteActionCatalog {
        parse("#0 48=3225 prince\n18.cast(1 4,0.0:6 32.0:0 32.1:2! 32.2:3)\n")
    }
    #[test]
    fn caster18_uses_same_graphics_owner_all_headings_and_one_launch() {
        let appearance = BaseAppearance { class: 0, sex: 0 };
        let bound = qualify_caster(&caster_catalog(), appearance).unwrap();
        assert_eq!(bound.gfx, 3225);
        assert_eq!(bound.sequence.total, 5); // body terminal6 is NOT effect dwell
        assert_eq!(bound.sequence.sample(Duration::ZERO).unwrap(), (2, false));
        let mut playback = bound.sequence.start();
        assert_eq!(
            playback.advance(Duration::ZERO).unwrap().crossed_cues[0].event,
            FrameEvent::Hit
        );
        assert!(playback
            .advance(Duration::ZERO)
            .unwrap()
            .crossed_cues
            .is_empty());
        assert!(playback.advance(tick_duration(5, 24)).unwrap().finished);
        for heading in 0..8 {
            let frame = bound
                .sequence
                .animation
                .frame_asset(bound.gfx, 2, heading)
                .unwrap();
            assert_eq!(
                (frame.graphics_id, frame.image_id, frame.subframe),
                (3225, 32 + u32::from(heading), 1)
            );
        }
    }
    #[test]
    fn malformed_caster_launch_and_unqualified_cues_fail_closed() {
        let appearance = BaseAppearance { class: 0, sex: 0 };
        let source = "#0 48=3225 prince\n18.cast(1 4,0.0:0 32.0:0 32.1:2! 32.2:3)\n";
        for text in [
            source.replace("2!", "2"),
            source.replace("0.0:0", "0.0:0!"),
            source.replace("32.2:3", "32.2:3!"),
            source.replace("2!", "2![710"),
            source.replace("18.cast(1", "18.cast(0"),
        ] {
            assert!(qualify_caster(&parse(&text), appearance).is_err());
        }
    }
    #[test]
    fn sequence_bounds_and_source_zero_cues_are_exact() {
        let a = parse("#1 1 generic 0.fire(0 3,0.2:2[3 0.0:0[1 0.1:1[2)");
        let animation = a.resolve_action(1, ActionKey::new(0)).unwrap().animation;
        let s = Sequence::new(animation, true).unwrap();
        assert_eq!(s.sample(Duration::ZERO).unwrap(), (2, false));
        let mut p = s.start();
        let first = p.advance(Duration::ZERO).unwrap();
        assert_eq!(
            first
                .crossed_cues
                .iter()
                .map(|c| c.event.clone())
                .collect::<Vec<_>>(),
            [FrameEvent::Sound(1), FrameEvent::Sound(2)]
        );
        let terminal = p.advance(tick_duration(1, 24)).unwrap();
        assert_eq!((terminal.source_frame, terminal.finished), (0, false));
        assert_eq!(terminal.crossed_cues[0].event, FrameEvent::Sound(3));
        assert_eq!(terminal.crossed_cues[0].source_frame, 0);
        assert!(p.advance(tick_duration(3, 24)).unwrap().finished);
        let mut oversized = animation.clone();
        oversized.frames = vec![animation.frames[1].clone(); MAX_FRAMES + 1];
        assert!(matches!(
            Sequence::new(&oversized, true),
            Err(FireStormError::Clock(AnimationError::LimitExceeded(_)))
        ));
        let mut no_active = animation.clone();
        no_active.frames[2].duration_ticks = 0;
        assert!(matches!(
            Sequence::new(&no_active, true),
            Err(FireStormError::Unsupported(_))
        ));
    }
    #[test]
    fn cancellation_and_invalid_appearance_precede_any_catalog_acquisition() {
        use super::super::shared_action_catalog::test_counts;
        test_counts::reset();
        let root = Path::new("absent-fire-storm-unit-root");
        assert!(matches!(
            FireStormAssets::load(root, &[], &AtomicBool::new(true)),
            Err(FireStormError::Cancelled)
        ));
        assert!(matches!(
            FireStormAssets::load(root, &[], &AtomicBool::new(false)),
            Err(FireStormError::InvalidInput(_))
        ));
        assert!(matches!(
            FireStormAssets::load(
                root,
                &[BaseAppearance { class: 7, sex: 0 }],
                &AtomicBool::new(false)
            ),
            Err(FireStormError::InvalidInput(_))
        ));
        assert_eq!(test_counts::snapshot(), Default::default());
    }
    #[test]
    fn native_four_copy_offsets_match_world_projection_not_four_targets() {
        use super::super::world_map::{ViewProjection, WorldCell};
        let origin = WorldCell { x: 32792, y: 32843 };
        let pixel = PixelPoint { x: 400, y: 300 };
        let projection = ViewProjection {
            anchor_cell: origin,
            anchor_pixel: pixel,
        };
        let actual = group_anchors(pixel).unwrap();
        let logical = [(1, 1), (-1, -1), (-1, 1), (1, -1)];
        for (at, (dx, dy)) in actual.iter().zip(logical) {
            assert_eq!(
                Some(*at),
                projection.cell_to_pixel(WorldCell {
                    x: origin.x + dx,
                    y: origin.y + dy
                })
            );
        }
        assert_eq!(
            actual,
            [
                PixelPoint { x: 448, y: 300 },
                PixelPoint { x: 352, y: 300 },
                PixelPoint { x: 400, y: 324 },
                PixelPoint { x: 400, y: 276 }
            ]
        );
        assert!(!actual.contains(&pixel));
    }
    #[test]
    fn group_draw_is_four_copies_with_clipping_and_atomic_overflow_rejection() {
        let frame = SpriteFrame {
            fragments: None,
            x_offset: 0,
            y_offset: 0,
            width: 1,
            height: 1,
            frame_type: 1,
            unknown1: 0,
            unknown2: 0,
            rgba: vec![250, 7, 19, 255],
        };
        let mut full = Canvas::new(136, 100).unwrap();
        draw_group_frame(&mut full, &frame, PixelPoint { x: 64, y: 50 }).unwrap();
        let expected_pixel: [u8; 4] = [255, 0, 16, 255]; // the source quantized to RGB555 on black
        let colored: Vec<_> = full
            .rgba()
            .chunks_exact(4)
            .enumerate()
            .filter(|(_, p)| *p == expected_pixel.as_slice())
            .map(|(i, _)| (i % 136, i / 136))
            .collect();
        assert_eq!(colored, [(64, 26), (16, 50), (112, 50), (64, 74)]);
        let mut single = Canvas::new(136, 100).unwrap();
        single
            .blit_screen_rgb555(&frame.rgba, 1, 1, 64, 50)
            .unwrap();
        assert_ne!(full.rgba(), single.rgba());
        let mut clipped = Canvas::new(136, 100).unwrap();
        draw_group_frame(&mut clipped, &frame, PixelPoint { x: 0, y: 0 }).unwrap();
        assert_eq!(
            clipped
                .rgba()
                .chunks_exact(4)
                .filter(|p| *p == expected_pixel.as_slice())
                .count(),
            2
        );
        let before = clipped.rgba().to_vec();
        assert!(draw_group_frame(&mut clipped, &frame, PixelPoint { x: i32::MAX, y: 0 }).is_err());
        assert_eq!(clipped.rgba(), before);
    }
    #[test]
    fn four_copy_screen_accumulates_covered_pixels_instead_of_overwriting_them() {
        let frame = SpriteFrame {
            fragments: None,
            x_offset: -50,
            y_offset: -26,
            width: 101,
            height: 53,
            frame_type: 41,
            unknown1: 0,
            unknown2: 0,
            rgba: [123, 123, 123, 255].repeat(101 * 53),
        };
        let mut canvas = Canvas::new(301, 301).unwrap();
        draw_group_frame(&mut canvas, &frame, PixelPoint { x: 150, y: 150 }).unwrap();
        let center = (150 * 301 + 150) * 4;
        // All four copies cover the origin: five-bit levels 0 -> 15 -> 22 ->
        // 26 -> 28. Ordinary opaque drawing would remain at15 (RGB123).
        assert_eq!(&canvas.rgba()[center..center + 4], &[231, 231, 231, 255]);
        let mut invalid = frame.clone();
        invalid.rgba[7] = 128;
        let before = canvas.rgba().to_vec();
        assert!(draw_group_frame(&mut canvas, &invalid, PixelPoint { x: 150, y: 150 }).is_err());
        assert_eq!(
            canvas.rgba(),
            before,
            "invalid coverage cannot partially draw the first copy"
        );
    }
    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
    fn rgba_aggregate(frames: &[SpriteFrame]) -> [u8; 32] {
        // Independent C# oracle: exactly14 records for this SPR.
        // Each record is six BE i32 then SHA256 of its straight-RGBA pixels.
        let mut records = Vec::new();
        for (index, frame) in frames.iter().enumerate() {
            for value in [
                i32::try_from(index).unwrap(),
                frame.x_offset,
                frame.y_offset,
                i32::try_from(frame.width).unwrap(),
                i32::try_from(frame.height).unwrap(),
                i32::from(frame.frame_type),
            ] {
                records.extend_from_slice(&value.to_be_bytes());
            }
            records.extend_from_slice(&Sha256::digest(&frame.rgba));
        }
        assert_eq!(records.len(), 14 * 56);
        Sha256::digest(&records).into()
    }
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; independent C#1819 all14frames/710RIFF plus all14 caster18/8headings; memory only"]
    fn real_shared_fire_storm_full_sprite_and_sound_match_independent_csharp() {
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let appearances: Vec<_> = (0..7)
            .flat_map(|class| (0..2).map(move |sex| BaseAppearance { class, sex }))
            .collect();
        let assets =
            FireStormAssets::load(Path::new(&root), &appearances, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            hex(&assets.catalog_sha256()),
            "523c61ad5cc362a56a36f58b262c84efa8b4435f017e7074d35b5834e8b9f127"
        );
        let source = assets
            .sprite_sources()
            .find(|s| s.name == "1819-0.spr")
            .unwrap();
        assert_eq!(source.archive, SharedArchiveKind::Sprite);
        assert_eq!(source.bytes, 203281);
        assert_eq!(
            hex(&source.sha256),
            "91cd448ff2f1433b0204598eb015c55cd175dd22d5c3584903f6e3bb18347884"
        );
        let decoded = &assets.sprites[&SpriteKey {
            gfx: 1819,
            image: 0,
        }]
            .decoded;
        assert!(decoded.has_palette);
        assert_eq!(decoded.block_table_metadata, 348226);
        let frames = assets.sprite_frames("1819-0.SPR").unwrap();
        assert_eq!(frames.len(), 14);
        assert_eq!(
            hex(&rgba_aggregate(frames)),
            "a0c41b79c547c9919d55518a4803067622bcd95a67f7766cdfde2737e0e4f7f5"
        );
        assert_eq!(
            frames.iter().map(|f| f.frame_type).collect::<Vec<_>>(),
            [41, 41, 49, 49, 49, 49, 49, 49, 49, 57, 49, 49, 49, 41]
        );
        assert_eq!(assets.sound_groups().len(), 1);
        assert_eq!(assets.sound_group().candidates(), [710]);
        assert_eq!(
            assets.sound_group().mode,
            super::super::combat_animation::NativeAudioMode::SuppressWhileSameIdPlaying
        );
        let bank = assets.sound_bank();
        assert_eq!(bank.ids().collect::<Vec<_>>(), [710]);
        let sound = bank.get(710).unwrap();
        assert_eq!(sound.file_bytes(), 137846);
        assert_eq!(sound.pcm_bytes().len(), 137726); // trailing LIST is not PCM
        assert_eq!(
            hex(&sound.sha256()),
            "606e81f106b9c9063f5594ea4dc140ff8e15ef86cb7cbfd3dedc3cde3e5a3078"
        );
        let format = sound.format();
        assert_eq!(
            (format.channels, format.sample_rate, format.bits_per_sample),
            (1, 22050, 16)
        );
        assert_eq!((format.byte_rate, format.block_align), (44100, 2));
        assert!(assets.encoded_bytes() <= MAX_ENCODED);
        assert!(assets.rgba_bytes() <= MAX_RGBA);
        assert_eq!(assets.casters.len(), 14);
        for appearance in appearances {
            let cast = assets.caster(appearance).unwrap();
            let delay = assets.cast_launch_delay(appearance).unwrap();
            assert!(delay < assets.cast_duration(appearance).unwrap());
            let mut playback = assets.start_cast(appearance).unwrap();
            let cues = playback.advance(Duration::MAX).unwrap();
            assert_eq!(
                cues.crossed_cues
                    .iter()
                    .filter(|c| c.event == FrameEvent::Hit)
                    .count(),
                1
            );
            for heading in 0..8 {
                // Loader qualified ALL subframes, including zero-tick setup and
                // terminal frames; this checks the published per-heading mapping.
                for position in 0..cast.sequence.starts.len() {
                    let at = tick_duration(cast.sequence.starts[position], 24);
                    let visual = assets.caster_frame(appearance, heading, at).unwrap();
                    assert_eq!(visual.graphics_id, cast.gfx);
                    assert_eq!(
                        visual.frame.rgba.len(),
                        visual.frame.width as usize * visual.frame.height as usize * 4
                    );
                }
            }
        }
        let anchor = PixelPoint { x: 400, y: 400 };
        // Compare only this true resource layer at the same canvas/anchor/time.
        // The terminal SPR may be visually blank; its raw pixels still must be
        // selected and submitted during dwell, not replaced by another frame.
        for (elapsed, subframe) in [
            (Duration::ZERO, 0),
            (Duration::from_millis(2125), 13),
            (Duration::from_nanos(2_374_999_999), 13),
        ] {
            let mut actual = Canvas::new(800, 600).unwrap();
            actual.clear([37, 91, 143]);
            let mut expected = Canvas::new(800, 600).unwrap();
            expected.clear([37, 91, 143]);
            let frame = &frames[subframe];
            // Literal native offsets, with the modern floor-screen reference
            // independently qualified by the compositor's1024-pair C# oracle.
            for (dx, dy) in [(48, 0), (-48, 0), (0, 24), (0, -24)] {
                expected
                    .blit_screen_rgb555(
                        &frame.rgba,
                        frame.width,
                        frame.height,
                        anchor.x + dx + frame.x_offset,
                        anchor.y + dy + frame.y_offset,
                    )
                    .unwrap();
            }
            assert!(assets.draw_effect(&mut actual, anchor, elapsed).unwrap());
            assert_eq!(actual.rgba(), expected.rgba());
            assert_eq!(assets.effect_frame(elapsed).unwrap().subframe, subframe);
        }
        let mut active_canvas = Canvas::new(800, 600).unwrap();
        active_canvas.clear([37, 91, 143]);
        let blank = active_canvas.rgba().to_vec();
        assets
            .draw_effect(&mut active_canvas, anchor, Duration::from_millis(875))
            .unwrap();
        assert_ne!(active_canvas.rgba(), blank);
        let mut ended = Canvas::new(800, 600).unwrap();
        ended.clear([37, 91, 143]);
        assert!(!assets
            .draw_effect(&mut ended, anchor, assets.effect_duration())
            .unwrap());
        assert_eq!(ended.rgba(), blank);
        let mut playback = assets.start_effect();
        assert_eq!(
            playback.advance(Duration::ZERO).unwrap().crossed_cues[0].event,
            FrameEvent::Sound(710)
        );
        assert!(playback
            .advance(Duration::ZERO)
            .unwrap()
            .crossed_cues
            .is_empty());
        assert!(
            !playback
                .advance(assets.effect_terminal_start())
                .unwrap()
                .finished
        );
        assert!(playback.advance(assets.effect_duration()).unwrap().finished);
        assert!(playback
            .advance(Duration::MAX)
            .unwrap()
            .crossed_cues
            .is_empty());
    }
}
