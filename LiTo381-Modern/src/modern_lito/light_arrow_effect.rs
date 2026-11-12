//! Qualified shared class4 skills and the original eight-direction class5 ice arrow6.
//! Wind1799/1800 is undirected; its impact Hit remains a presentation cue.
//! Not a generic spell engine; no damage/MP/Hit execution or LT fallback.
//!
//! Evidence in ida-client-oep-v2/full-disassembly.asm:
//! sub_43DE10 at line133583 parses109's class/secondary gfx into647E5C/647E50;
//! sub_4A5ED0 at347263 dispatches class4 to4243F0. sub_424780 (76494)
//! snapshots target coordinates ON LAUNCH, folds heading>=4 by subtracting4,
//! and sub_4B92D0 (386862) builds a16.16 vector. sub_424540 (76228)
//! advances one native grid step immediately and every60ms thereafter,
//! then creates secondary219; callback424760 does not retarget.
//! sub_4E01A0 (470162) only unlinks render blocks, not target coordinates.
//! Therefore caller resolves latest SERVER-CONFIRMED target/corpse cell once
//! at caster18's launch marker, then retains this immutable path. It must
//! discard the presentation on entity departure or world generation change.
//! Neither marker nor arrival can delay/recompute the authoritative hit.
//!
//! 167's action is directional but class4 deliberately uses only four banks.
//! Its one-shot animation holds terminal0 while flight continues. 219 is
//! removed after its1..N-1 timeline. Sound97 starts once on launch, including
//! adjacent targets whose native flight reaches219 immediately. No homing,
//! arbitrary velocity, guessed sprites, recoloring, or new geometry collision.
//!
//! All decoding/WAV qualification is bounded and worker-only; completed assets
//! retain no files/workers. Draw uses only immutable RGBA and checked projection.

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
    world_map::{PixelPoint, ViewProjection, WorldCell},
    world_scene::BaseAppearance,
};

const EFFECT: u32 = 167;
const IMPACT: u32 = 219;
const SOUND: u32 = 97;
const STEP_NANOS: u64 = 60_000_000;
/// Presentation-only bound, not the server's ten-cell admission range. A
/// target can move/teleport between cast acceptance and the visual launch.
/// Beyond this bound the caller may omit this one effect, never fail the world.
const MAX_FLIGHT_CELLS: i64 = 256;
const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_ENCODED: usize = 32 * 1024 * 1024;
const MAX_RGBA: usize = 64 * 1024 * 1024;
const MAX_SPRITES: usize = 229;
const MAX_INDEX_RECORDS: usize = 1_000_000;
const MAX_FRAMES: usize = 256;
const MAX_CUES: usize = 1024;
const NANOS: u128 = 1_000_000_000;

#[derive(Debug)]
pub enum LightArrowError {
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
impl fmt::Display for LightArrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "light arrow resource loading cancelled"),
            Self::InvalidInput(reason) => write!(f, "invalid light arrow presentation: {reason}"),
            Self::Unsupported(reason) => {
                write!(f, "unsupported shared light arrow presentation: {reason}")
            }
            Self::UnsupportedSound(id) => {
                write!(f, "shared light arrow Sound/{id}.wav is not qualified")
            }
            Self::Bound(reason) => write!(f, "light arrow presentation exceeds {reason} bound"),
            Self::Resource(error) => write!(f, "light arrow shared resource: {error}"),
            Self::Catalog(error) => write!(f, "light arrow action definition: {error}"),
            Self::Sprite(error) => write!(f, "light arrow SPR: {error}"),
            Self::Clock(error) => write!(f, "light arrow timeline: {error}"),
            Self::Render(error) => write!(f, "light arrow drawing: {error}"),
        }
    }
}
impl std::error::Error for LightArrowError {}
impl From<SharedCatalogError> for LightArrowError {
    fn from(error: SharedCatalogError) -> Self {
        match error {
            SharedCatalogError::Cancelled => Self::Cancelled,
            SharedCatalogError::Bound(reason) => Self::Bound(reason),
            SharedCatalogError::Resource(error) => Self::Resource(error),
            SharedCatalogError::Action(error) => Self::Catalog(error),
        }
    }
}
impl From<ResourceError> for LightArrowError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}
impl From<ActionError> for LightArrowError {
    fn from(value: ActionError) -> Self {
        Self::Catalog(value)
    }
}
impl From<SpriteError> for LightArrowError {
    fn from(value: SpriteError) -> Self {
        Self::Sprite(value)
    }
}
impl From<AnimationError> for LightArrowError {
    fn from(value: AnimationError) -> Self {
        Self::Clock(value)
    }
}
type Result<T> = std::result::Result<T, LightArrowError>;

fn active(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(LightArrowError::Cancelled)
    } else {
        Ok(())
    }
}
fn reserve<T>(count: usize) -> Result<Vec<T>> {
    let mut value = Vec::new();
    value
        .try_reserve_exact(count)
        .map_err(|_| LightArrowError::Bound("allocation"))?;
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
pub(crate) struct SpriteReader {
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
            return Err(LightArrowError::Bound("index records"));
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
                .ok_or(LightArrowError::Bound("index records"))?;
            if self.records > MAX_INDEX_RECORDS {
                return Err(LightArrowError::Bound("index records"));
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
            return Err(LightArrowError::Unsupported("one-shot lacks active frames"));
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
                .ok_or(LightArrowError::Bound("ticks"))?;
            ends.push(total);
        }
        if total == 0 {
            return Err(LightArrowError::Unsupported("zero active duration"));
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
            .ok_or(LightArrowError::Bound("elapsed time"))?;
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

pub struct LightArrowSample {
    pub source_frame: usize,
    pub finished: bool,
    pub crossed_cues: Vec<CrossedCue>,
}
#[derive(Clone)]
pub struct LightArrowCastPlayback {
    sequence: Arc<Sequence>,
    next_event: usize,
    last_elapsed: Option<Duration>,
}
impl LightArrowCastPlayback {
    pub fn advance(&mut self, elapsed: Duration) -> Result<LightArrowSample> {
        if self.last_elapsed.is_some_and(|last| elapsed < last) {
            return Err(LightArrowError::InvalidInput("elapsed time regressed"));
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
            return Err(LightArrowError::Bound("crossed cues"));
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
        Ok(LightArrowSample {
            source_frame,
            finished,
            crossed_cues,
        })
    }
}

/// A single launch's finite native class4/class5 path. No target identity lookup lives
/// in the asset layer. Coordinates use the shared world's u16 native domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LightArrowPath {
    from: WorldCell,
    target: WorldCell,
    heading: u8,
    steps: Vec<WorldCell>,
}
impl LightArrowPath {
    pub fn new(from: WorldCell, target: WorldCell) -> Result<Self> {
        if [from.x, from.y, target.x, target.y]
            .iter()
            .any(|&v| !(0..=65535).contains(&v))
        {
            return Err(LightArrowError::InvalidInput("native world coordinates"));
        }
        Self::build_native_path(from, target)
    }
    /// S39's missing first target falls back one heading step, even at a u16 edge.
    pub(crate) fn new_area_fireball(from: WorldCell, target: WorldCell) -> Result<Self> {
        if [from.x,from.y].iter().any(|v| !(0..=65535).contains(v))
            || [target.x,target.y].iter().any(|v| !(-1..=65536).contains(v)) {
            return Err(LightArrowError::InvalidInput("area world coordinates"));
        }
        Self::build_native_path(from, target)
    }
    fn build_native_path(from: WorldCell, target: WorldCell) -> Result<Self> {
        let dx = i64::from(target.x) - i64::from(from.x);
        let dy = i64::from(target.y) - i64::from(from.y);
        let distance = dx.abs().max(dy.abs());
        if distance > MAX_FLIGHT_CELLS {
            return Err(LightArrowError::Bound("projectile presentation lifetime"));
        }
        if distance == 0 {
            // 4B92D0 returns a zero vector; 424780 sets its counter to0 and
            // 424540 creates219 immediately. Still process launch97 once.
            let mut steps = reserve(1)?;
            steps.push(target);
            return Ok(Self {
                from,
                target,
                heading: 0,
                steps,
            });
        }
        // Native x is 2*worldX-32768, y is worldY. 4B92D0 divides before
        // doubling x velocity. Keep truncation-toward-zero; no float drift.
        let vx = (dx * 65536 / distance) * 2;
        let vy = dy * 65536 / distance;
        let mut x = (i64::from(from.x) * 2 - 32768) * 65536;
        let mut y = i64::from(from.y) * 65536;
        let mut steps = reserve((distance * 2) as usize)?;
        for _ in 0..distance * 2 {
            x += vx;
            y += vy;
            // Native rounds fixed-point, THEN clears x's low bit. It is not
            // symmetric rounding of worldX and must not be replaced by lerp.
            let native_x = (x + 32768).div_euclid(65536) & !1;
            let cell = WorldCell {
                x: i32::try_from((native_x + 32768) / 2)
                    .map_err(|_| LightArrowError::Bound("flight x"))?,
                y: i32::try_from((y + 32768).div_euclid(65536))
                    .map_err(|_| LightArrowError::Bound("flight y"))?,
            };
            steps.push(cell);
            if cell == target {
                break;
            }
        }
        if steps.last() != Some(&target) {
            return Err(LightArrowError::Unsupported("native path did not arrive"));
        }
        Ok(Self {
            from,
            target,
            heading: native_heading(dx, dy),
            steps,
        })
    }
    pub fn from(&self) -> WorldCell {
        self.from
    }
    pub fn target(&self) -> WorldCell {
        self.target
    }
    pub fn heading(&self) -> u8 {
        self.heading
    }
    pub fn impact_at(&self) -> Duration {
        Duration::from_nanos((self.steps.len() as u64 - 1) * STEP_NANOS)
    }
    pub fn position(&self, elapsed: Duration) -> WorldCell {
        let index = (elapsed.as_nanos() / u128::from(STEP_NANOS)).min(self.steps.len() as u128 - 1)
            as usize;
        self.steps[index]
    }
}
// Literal threshold branches from4B9210 after converting native dx/2 to world.
fn native_heading(dx: i64, dy: i64) -> u8 {
    let c = 20 * dy;
    let x = 2 * dx;
    if c > 24 * x {
        if c > -24 * x {
            4
        } else if c > -4 * x {
            5
        } else if c <= 4 * x {
            7
        } else {
            6
        }
    } else if c > 4 * x {
        3
    } else if c > -4 * x {
        2
    } else if c > -24 * x {
        1
    } else {
        0
    }
}
fn tick_duration(tick: u64, fps: u32) -> Duration {
    let nanos = (u128::from(tick) * NANOS).div_ceil(u128::from(fps));
    Duration::new((nanos / NANOS) as u64, (nanos % NANOS) as u32)
}
/// Owned finite event cursor. Repeated samples cannot replay launch audio.
/// Flight and impact clocks are separate; neither cursor owns native playback.
#[derive(Clone)]
pub struct LightArrowPlayback {
    path: LightArrowPath,
    flight: LightArrowCastPlayback,
    impact: LightArrowCastPlayback,
    last_elapsed: Option<Duration>,
    secondary: bool,
}
impl LightArrowPlayback {
    pub(crate) fn without_secondary(mut self)->Self{self.secondary=false;self}
    pub fn advance(&mut self, elapsed: Duration) -> Result<LightArrowSample> {
        if self.last_elapsed.is_some_and(|last| elapsed < last) {
            return Err(LightArrowError::InvalidInput("elapsed time regressed"));
        }
        // Still advance flight at0 on adjacent arrival: native invokes its
        // animation/cues before creating219 in the same immediate update.
        let flight = self.flight.advance(elapsed.min(self.path.impact_at()))?;
        let mut sample = if !self.secondary || elapsed < self.path.impact_at() {
            LightArrowSample {
                source_frame: flight.source_frame,
                finished: false,
                crossed_cues: flight.crossed_cues,
            }
        } else {
            let mut impact = self.impact.advance(elapsed - self.path.impact_at())?;
            let mut cues = flight.crossed_cues;
            cues.append(&mut impact.crossed_cues);
            if cues.len() > MAX_CUES {
                return Err(LightArrowError::Bound("crossed cues"));
            }
            impact.crossed_cues = cues;
            impact
        };
        sample.finished = elapsed >= self.path.impact_at() + if self.secondary {self.impact.sequence.duration()}else{Duration::ZERO};
        self.last_elapsed = Some(elapsed);
        Ok(sample)
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

pub struct LightArrowAssets {
    effect: BoundSequence,
    impact: BoundSequence,
    casters: BTreeMap<BaseAppearance, BoundSequence>,
    sprites: BTreeMap<SpriteKey, LoadedSprite>,
    sound: Arc<SoundBank>,
    group: NativeAudioGroup,
    catalog_sha256: [u8; 32],
    encoded: usize,
    rgba: usize,
}
impl LightArrowAssets {
    /// Original 167/219 assets only. The current world animator owns caster18.
    pub fn load_effect_only(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let catalog = SharedActionCatalog::load(root, cancel)?;
        Self::load_selected(&catalog, BTreeSet::new(), cancel)
    }
    /// Three individually qualified class4 chains; this is not a generic fallback.
    pub fn load_class4_effect_only(root:&Path, graphics:u32, cancel:&AtomicBool)->Result<Self> {
        if !matches!(graphics,167|1583|1799) {return Err(LightArrowError::Unsupported("unqualified class4 graphic"));}
        active(cancel)?;
        let catalog=SharedActionCatalog::load(root,cancel)?;
        Self::load_selected_profile(&catalog,BTreeSet::new(),graphics,cancel)
    }
    /// Original class5 chain1797->1798; all eight flight banks are required.
    pub fn load_ice_arrow_effect_only(root:&Path,cancel:&AtomicBool)->Result<Self>{
        active(cancel)?;let catalog=SharedActionCatalog::load(root,cancel)?;
        Self::load_selected_profile(&catalog,BTreeSet::new(),1797,cancel)
    }
    pub(crate) fn load_effect_profile_with_catalog(context:&SharedActionCatalog,graphics:u32,cancel:&AtomicBool)->Result<Self> {
        active(cancel)?;
        Self::load_selected_profile(context,BTreeSet::new(),graphics,cancel)
    }
    /// Worker-only: fixed Text/list.spr, exact167->219 and at most14 unique base
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
            return Err(LightArrowError::InvalidInput("appearance count"));
        }
        let mut wanted = BTreeSet::new();
        for appearance in appearances {
            appearance
                .logical_sprite_id()
                .map_err(|_| LightArrowError::InvalidInput("base class/sex"))?;
            wanted.insert(*appearance);
        }
        if wanted.len() > 14 {
            return Err(LightArrowError::Bound("base appearances"));
        }
        Ok(wanted)
    }

    fn load_selected(
        context: &SharedActionCatalog,
        wanted: BTreeSet<BaseAppearance>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::load_selected_profile(context,wanted,EFFECT,cancel)
    }
    /// Shared only during a single prepare call; no handles reach the UI.
    pub(crate) fn load_effect_profile_with_reader(context:&SharedActionCatalog,graphics:u32,cancel:&AtomicBool,slot:&mut Option<SpriteReader>)->Result<Self> {
        let result=Self::load_selected_profile_using_reader(context,BTreeSet::new(),graphics,cancel,slot);
        // Do not reuse partial state after any failed qualification/read/decode.
        if result.is_err() {*slot=None;}
        result
    }
    fn load_selected_profile(
        context:&SharedActionCatalog, wanted:BTreeSet<BaseAppearance>, graphics:u32, cancel:&AtomicBool,
    )->Result<Self> {
        Self::load_selected_profile_using_reader(context,wanted,graphics,cancel,&mut None)
    }
    fn load_selected_profile_using_reader(
        context:&SharedActionCatalog,wanted:BTreeSet<BaseAppearance>,graphics:u32,cancel:&AtomicBool,
        reader_slot:&mut Option<SpriteReader>,
    )->Result<Self> {
        active(cancel)?;
        if context.byte_length() > MAX_ENTRY {
            return Err(LightArrowError::Bound("action catalog"));
        }
        let root = context.root();
        let catalog_sha256 = context.sha256();
        let catalog = context.catalog();
        let (effect,impact)=match graphics {
            167=>(qualify_effect(&catalog)?,qualify_impact(&catalog)?),
            1583=>(qualify_fire_effect(&catalog)?,qualify_fire_impact(&catalog)?),
            1799=>(qualify_wind_effect(&catalog)?,qualify_wind_impact(&catalog)?),
            1797=>(qualify_ice_effect(&catalog)?,qualify_ice_impact(&catalog)?),
            _=>return Err(LightArrowError::Unsupported("unqualified class4 graphic")),
        };
        let mut casters = BTreeMap::new();
        for appearance in wanted {
            active(cancel)?;
            casters.insert(appearance, qualify_caster(&catalog, appearance)?);
        }
        let (launch_sound,impact_sound)=match graphics {167=>(97,None),1583=>(97,Some(99)),1799=>(719,Some(711)),1797=>(719,Some(713)),_=>unreachable!()};
        let group = classify_native_cues(&[FrameEvent::Sound(launch_sound)])
            .map_err(|_| LightArrowError::Unsupported("sound cue classification"))?
            .audio
            .ok_or(LightArrowError::Unsupported("missing sound cue group"))?;
        let mut groups=vec![group.clone()];
        if let Some(sound)=impact_sound {
            groups.push(classify_native_cues(&[FrameEvent::Sound(sound)])
                .map_err(|_|LightArrowError::Unsupported("impact sound cue classification"))?
                .audio.ok_or(LightArrowError::Unsupported("missing impact sound cue group"))?);
        }
        if reader_slot.is_none() {*reader_slot=Some(SpriteReader::open(root,cancel)?);}
        let reader=reader_slot.as_mut().expect("successful scoped Sprite reader");
        if reader.root != root {
            return Err(ResourceError::UnsafePath(root.to_path_buf()).into());
        }
        let mut sprites = BTreeMap::new();
        let mut encoded = 0usize;
        let mut rgba = 0usize;
        for bound in [&effect, &impact].into_iter().chain(casters.values()) {
            let headings = if bound.gfx == effect.gfx && effect.gfx!=1797 && bound.sequence.animation.directionality == Directionality::EightWay {
                4
            } else if bound.sequence.animation.directionality == Directionality::EightWay {
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
                            return Err(LightArrowError::Bound("sprite entries"));
                        }
                        let name = key.name();
                        let archive = reader.archive_for(&name, cancel)?;
                        let size = archive.entry(&name)?.size as usize;
                        if size == 0 || size > MAX_ENTRY || size > MAX_ENCODED - encoded {
                            return Err(LightArrowError::Bound("encoded sprites"));
                        }
                        let kind = archive.kind();
                        active(cancel)?;
                        let bytes = archive.read_entry(&name)?;
                        active(cancel)?;
                        if bytes.len() != size {
                            return Err(LightArrowError::Unsupported("SPR length changed"));
                        }
                        let remaining = (MAX_RGBA - rgba) / 4;
                        if remaining == 0 {
                            return Err(LightArrowError::Bound("decoded sprites"));
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
                            .ok_or(LightArrowError::Bound("decoded sprites"))?;
                        if count > MAX_RGBA - rgba {
                            return Err(LightArrowError::Bound("decoded sprites"));
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
                        return Err(LightArrowError::Unsupported("missing SPR subframe"));
                    }
                }
            }
        }

        active(cancel)?;
        let sound = SoundBank::load_shared(
            root,
            &groups,
            BankLimits {
                max_bytes: 1024 * 1024,
                max_sounds: 2,
                max_groups: 2,
            },
            cancel,
        )
        .map_err(|_| {
            if cancel.load(Ordering::Acquire) {
                LightArrowError::Cancelled
            } else {
                LightArrowError::Unsupported("qualified class4 sound bank failed")
            }
        })?;
        active(cancel)?;
        Ok(Self {
            effect,
            impact,
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
    pub fn effect_duration(&self, path: &LightArrowPath) -> Duration {
        path.impact_at() + self.impact.sequence.duration()
    }
    /// First Hit is a presentation launch marker, never a combat callback.
    pub fn cast_launch_delay(&self, appearance: BaseAppearance) -> Result<Duration> {
        let sequence = &self.caster(appearance)?.sequence;
        for (position, &tick) in sequence.starts.iter().enumerate() {
            if sequence.animation.frames[sequence.source_index(position)]
                .events
                .iter()
                .any(|event| *event == FrameEvent::Hit)
            {
                return Ok(tick_duration(tick, sequence.animation.frames_per_second));
            }
        }
        Err(LightArrowError::Unsupported("caster lacks launch marker"))
    }
    pub fn cast_duration(&self, appearance: BaseAppearance) -> Result<Duration> {
        Ok(self.caster(appearance)?.sequence.duration())
    }
    pub fn start_effect(&self, path: &LightArrowPath) -> LightArrowPlayback {
        LightArrowPlayback {
            path: path.clone(),
            flight: playback(&self.effect.sequence),
            impact: playback(&self.impact.sequence),
            last_elapsed: None,
            secondary: true,
        }
    }
    pub fn start_cast(&self, appearance: BaseAppearance) -> Result<LightArrowCastPlayback> {
        Ok(playback(&self.caster(appearance)?.sequence))
    }
    fn caster(&self, appearance: BaseAppearance) -> Result<&BoundSequence> {
        self.casters
            .get(&appearance)
            .ok_or(LightArrowError::Unsupported("caster was not preloaded"))
    }
    fn frame<'a>(
        &'a self,
        bound: &BoundSequence,
        heading: u8,
        elapsed: Duration,
    ) -> Result<VisualSample<'a>> {
        if heading > 7 {
            return Err(LightArrowError::InvalidInput("heading"));
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
            .ok_or(LightArrowError::Unsupported("unloaded SPR"))?;
        let frame = sprite
            .decoded
            .frames
            .get(asset.subframe as usize)
            .ok_or(LightArrowError::Unsupported("unloaded subframe"))?;
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
    pub fn effect_frame(
        &self,
        path: &LightArrowPath,
        elapsed: Duration,
    ) -> Result<VisualSample<'_>> {
        if elapsed < path.impact_at() {
            // Class4 folds direction banks. Undirected source animations ignore
            // heading and resolve their original image0, without mirroring.
            let mut sample = self.flight_frame(path,elapsed)?;
            // Native flight keeps its terminal frame until physical arrival.
            sample.finished = false;
            Ok(sample)
        } else {
            self.frame(&self.impact, 0, elapsed - path.impact_at())
        }
    }
    pub fn flight_frame(&self,path:&LightArrowPath,elapsed:Duration)->Result<VisualSample<'_>>{
        // Class5 preserves all eight headings; class4 deliberately folds banks.
        let heading=if self.effect.gfx==1797 {path.heading()}else{path.heading()&3};
        self.frame(&self.effect,heading,elapsed)
    }
    /// Uses the current paint projection of an immutable launch-time path.
    /// This never mutates a world actor, HP, MP, or hit/move authority.
    pub fn draw_effect(
        &self,
        canvas: &mut Canvas,
        path: &LightArrowPath,
        projection: ViewProjection,
        elapsed: Duration,
    ) -> Result<bool> {
        let sample = self.effect_frame(path, elapsed)?;
        if sample.finished {
            return Ok(false);
        }
        let anchor = projection
            .cell_to_pixel(path.position(elapsed))
            .ok_or(LightArrowError::Bound("projectile projection"))?;
        draw_screen_frame(canvas, sample.frame, anchor)?;
        Ok(true)
    }
}
fn playback(sequence: &Arc<Sequence>) -> LightArrowCastPlayback {
    LightArrowCastPlayback {
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
        .ok_or(LightArrowError::Bound("projected x"))?;
    let y = anchor
        .y
        .checked_add(frame.y_offset)
        .ok_or(LightArrowError::Bound("projected y"))?;
    canvas
        .blit_rgba(&frame.rgba, frame.width, frame.height, x, y)
        .map_err(LightArrowError::Render)
}

/// Both qualified167 flight and219 impact have attr104=8. Ordinary frame/body
/// drawing stays separate; screen uses the explicit modern floor RGB555 policy.
fn draw_screen_frame(canvas: &mut Canvas, frame: &SpriteFrame, anchor: PixelPoint) -> Result<()> {
    let x = anchor
        .x
        .checked_add(frame.x_offset)
        .ok_or(LightArrowError::Bound("projected x"))?;
    let y = anchor
        .y
        .checked_add(frame.y_offset)
        .ok_or(LightArrowError::Bound("projected y"))?;
    canvas
        .blit_screen_rgb555(&frame.rgba, frame.width, frame.height, x, y)
        .map_err(LightArrowError::Render)
}

fn qualify_effect(catalog: &SpriteActionCatalog) -> Result<BoundSequence> {
    qualify_fixed(
        catalog,
        EFFECT,
        Directionality::EightWay,
        4,
        &[(104, b"8"), (109, b"4 219")],
        &[(3, 3), (0, 2), (1, 3), (2, 2)],
        Some((1, SOUND)),
        None,
    )
}
fn qualify_impact(catalog: &SpriteActionCatalog) -> Result<BoundSequence> {
    qualify_fixed(
        catalog,
        IMPACT,
        Directionality::Undirected,
        1,
        &[(104, b"8")],
        &[
            (9, 1),
            (0, 4),
            (0, 1),
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 1),
            (5, 1),
            (6, 1),
            (7, 1),
            (8, 1),
        ],
        None,
        None,
    )
}
fn qualify_fire_effect(catalog:&SpriteActionCatalog)->Result<BoundSequence> {
    qualify_fixed(catalog,1583,Directionality::EightWay,4,
        &[(104,b"8"),(109,b"4 218")],&[(3,3),(0,2),(1,3),(2,2)],Some((1,97)),None)
}
fn qualify_fire_impact(catalog:&SpriteActionCatalog)->Result<BoundSequence> {
    qualify_fixed(catalog,218,Directionality::Undirected,1,&[(104,b"8")],
        &[(9,1),(0,4),(0,1),(1,1),(2,1),(3,1),(4,1),(5,1),(6,1),(7,1),(8,1)],Some((1,99)),None)
}
fn qualify_wind_effect(catalog:&SpriteActionCatalog)->Result<BoundSequence> {
    qualify_fixed(catalog,1799,Directionality::Undirected,4,
        &[(104,b"8"),(109,b"4 1800")],&[(4,2),(0,2),(1,2),(2,2),(3,2)],Some((1,719)),None)
}
fn qualify_wind_impact(catalog:&SpriteActionCatalog)->Result<BoundSequence> {
    qualify_fixed(catalog,1800,Directionality::Undirected,1,&[(104,b"8")],
        &[(5,4),(0,4),(1,3),(2,3),(3,3),(4,4)],Some((1,711)),Some(1))
}
fn qualify_ice_effect(catalog:&SpriteActionCatalog)->Result<BoundSequence>{
    qualify_fixed(catalog,1797,Directionality::EightWay,8,&[(104,b"8"),(109,b"5 1798")],
        &[(3,2),(0,2),(1,2),(2,2)],Some((1,719)),None)
}
fn qualify_ice_impact(catalog:&SpriteActionCatalog)->Result<BoundSequence>{
    qualify_fixed(catalog,1798,Directionality::Undirected,1,&[(104,b"8")],
        &[(6,3),(0,2),(1,1),(2,1),(3,2),(4,3),(5,3)],Some((1,713)),Some(1))
}
fn qualify_fixed(
    catalog: &SpriteActionCatalog,
    gfx: u32,
    direction: Directionality,
    image_count: u32,
    controls: &[(u32, &[u8])],
    frames: &[(u32, u32)],
    sound: Option<(usize, u32)>,
    hit: Option<usize>,
) -> Result<BoundSequence> {
    let sprite = catalog
        .sprites
        .get(&gfx)
        .ok_or(LightArrowError::Unsupported(
            "missing shared light arrow chain",
        ))?;
    let resolved = catalog.resolve_action(gfx, ActionKey::new(0))?;
    let animation = resolved.animation;
    if sprite.graphics_id != gfx
        || resolved.definition_sprite_id != gfx
        || sprite.image_count != Some(image_count)
        || animation.directionality != direction
        || animation.frames_per_second != 24
        || sprite.attributes.len() != controls.len()
        || animation.frames.len() != frames.len()
    {
        return Err(LightArrowError::Unsupported(
            "unverified projectile controls",
        ));
    }
    for (attribute, &(key, body)) in sprite.attributes.iter().zip(controls) {
        if attribute.key != ActionKey::new(key) || attribute.body_bytes != body {
            return Err(LightArrowError::Unsupported(
                "unverified projectile effect chain",
            ));
        }
    }
    for (index, (frame, &(subframe, duration))) in animation.frames.iter().zip(frames).enumerate() {
        let mut expected = if sound.is_some_and(|(at, _)| at == index) {
            vec![FrameEvent::Sound(sound.unwrap().1)]
        } else {
            Vec::new()
        };
        if hit == Some(index) {expected.push(FrameEvent::Hit);}
        if frame.image_id != 0
            || frame.subframe != subframe
            || frame.duration_ticks != duration
            || frame.events != expected
        {
            return Err(LightArrowError::Unsupported(
                "unverified projectile frames/cues",
            ));
        }
    }
    Ok(BoundSequence {
        sequence: Sequence::new(animation)?,
        gfx,
    })
}
fn qualify_caster(
    catalog: &SpriteActionCatalog,
    appearance: BaseAppearance,
) -> Result<BoundSequence> {
    let logical = appearance
        .logical_sprite_id()
        .map_err(|_| LightArrowError::InvalidInput("base class/sex"))?;
    let sprite = catalog
        .sprites
        .get(&logical)
        .ok_or(LightArrowError::Unsupported("missing base caster"))?;
    let resolved = catalog.resolve_action(logical, ActionKey::new(18))?;
    let definition = catalog
        .sprites
        .get(&resolved.definition_sprite_id)
        .ok_or(LightArrowError::Unsupported("missing cast definition"))?;
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
        return Err(LightArrowError::Unsupported(
            "unverified cast18 graphics or controls",
        ));
    }
    if animation
        .frames
        .iter()
        .skip(1)
        .flat_map(|frame| &frame.events)
        .filter(|event| **event == FrameEvent::Hit)
        .count()
        != 1
        || animation.frames[0].events.contains(&FrameEvent::Hit)
    {
        return Err(LightArrowError::Unsupported(
            "ambiguous caster launch marker",
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

    const ICE_SOURCE:&str="2 0 0\n#1797 8 ice arrow 0.fly(1 4,0.3:2 0.0:2[719 0.1:2 0.2:2) 104.attr(8) 109.effect(5 1798)\n#1798 1 ice impact 0.fire(0 7,0.6:3 0.0:2[713! 0.1:1 0.2:1 0.3:2 0.4:3 0.5:3) 104.attr(8)\n";
    #[test]
    fn ice_chain_requires_eight_banks_class5_and_original_cues(){
        let catalog=parse(ICE_SOURCE);assert!(qualify_ice_effect(&catalog).is_ok());assert!(qualify_ice_impact(&catalog).is_ok());
        for (old,new) in [("#1797 8","#1797 4"),("5 1798","4 1798"),("[719","[97"),("0.1:2","0.1:3")]{assert!(qualify_ice_effect(&parse(&ICE_SOURCE.replace(old,new))).is_err());}
        for (old,new) in [("[713!","[711!"),("[713!","[713"),("0.4:3","0.4:4")]{assert!(qualify_ice_impact(&parse(&ICE_SOURCE.replace(old,new))).is_err());}
    }
    #[test]
    fn ice_miss_timeline_finishes_without_secondary_events(){
        let catalog=parse(ICE_SOURCE);let path=LightArrowPath::new(cell(32730,32800),cell(32736,32800)).unwrap();
        let mut playback=LightArrowPlayback{path:path.clone(),flight:super::playback(&qualify_ice_effect(&catalog).unwrap().sequence),impact:super::playback(&qualify_ice_impact(&catalog).unwrap().sequence),last_elapsed:None,secondary:false};
        let sample=playback.advance(Duration::from_secs(5)).unwrap();assert!(sample.finished);assert_eq!(sample.crossed_cues.iter().map(|c|c.event.clone()).collect::<Vec<_>>(),vec![FrameEvent::Sound(719)]);
        assert!(playback.advance(Duration::from_secs(5)).unwrap().crossed_cues.is_empty());
    }

    const FLIGHT: &str =
        "#167 4 energy bolt 0.fly(1 4,0.3:3 0.0:2[97 0.1:3 0.2:2) 104.attr(8) 109.effect(4 219)";
    const IMPACT_TEXT: &str = "#219 1 take water 0.fire(0 11,0.9:1 0.0:4 0.0:1 0.1:1 0.2:1 0.3:1 0.4:1 0.5:1 0.6:1 0.7:1 0.8:1) 104.attr(8)";
    fn source() -> String {
        format!(
            "3 0 0\n{FLIGHT}\n{IMPACT_TEXT}\n#0 48=3225 prince\n18.cast(1 4,0.0:0 32.0:0 32.1:2! 32.2:3)\n"
        )
    }
    fn parse(text: &str) -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(text.as_bytes(), Default::default()).unwrap()
    }
    fn cell(x: i32, y: i32) -> WorldCell {
        WorldCell { x, y }
    }
    fn timeline(path: &LightArrowPath) -> LightArrowPlayback {
        let catalog = parse(&source());
        LightArrowPlayback {
            path: path.clone(),
            flight: playback(&qualify_effect(&catalog).unwrap().sequence),
            impact: playback(&qualify_impact(&catalog).unwrap().sequence),
            last_elapsed: None,
            secondary: true,
        }
    }
    #[test]
    fn exact_shared_chain_four_images_not_eight_or_guessed_hit() {
        let catalog = parse(&source());
        let flight = qualify_effect(&catalog).unwrap();
        let impact = qualify_impact(&catalog).unwrap();
        assert_eq!(flight.gfx, 167);
        assert_eq!(impact.gfx, 219);
        assert_eq!(
            flight.sequence.duration(),
            Duration::from_nanos(291_666_667)
        );
        assert_eq!(
            impact.sequence.duration(),
            Duration::from_nanos(541_666_667)
        );
        for heading in 0..8 {
            let image = flight
                .sequence
                .animation
                .frame_asset(167, 1, heading & 3)
                .unwrap();
            assert_eq!(image.image_id, u32::from(heading % 4));
            assert_eq!(image.subframe, 0);
        }
    }
    #[test]
    fn rejects_changed_class_chain_cues_direction_fps_and_missing_impact() {
        let good = source();
        for bad in [
            good.replace("4 219", "1 219"),
            good.replace("4 219", "4 220"),
            good.replace("0.fly(1", "0.fly(0"),
            good.replace("[97", "<97"),
            good.replace("[97", "[10097"),
            good.replace("[97", "[97^3"),
            good.replace("#167 4", "#167 8"),
            good.replace("104.attr(8) 109", "104.attr(8) 110.framerate(30) 109"),
        ] {
            assert!(qualify_effect(&parse(&bad)).is_err());
        }
        assert!(qualify_impact(&parse(&good.replace("#219", "#220"))).is_err());
        assert!(qualify_impact(&parse(&good.replace("0.8:1)", "0.8:1!)"))).is_err());
    }
    #[test]
    fn native_fixed_point_steps_have_immediate_first_step_then_sixty_ms() {
        let path = LightArrowPath::new(cell(100, 100), cell(105, 102)).unwrap();
        assert_eq!(path.position(Duration::ZERO), cell(101, 100));
        assert_eq!(path.position(Duration::from_millis(59)), cell(101, 100));
        assert_eq!(path.position(Duration::from_millis(60)), cell(102, 101));
        assert_eq!(path.position(Duration::from_millis(120)), cell(103, 101));
        assert_eq!(path.position(Duration::from_millis(180)), cell(104, 102));
        assert_eq!(path.impact_at(), Duration::from_millis(240));
        assert_eq!(path.position(Duration::MAX), cell(105, 102));
        let shallow = LightArrowPath::new(cell(100, 100), cell(102, 105)).unwrap();
        assert_eq!(shallow.position(Duration::ZERO), cell(100, 101));
        assert_eq!(shallow.position(Duration::from_millis(60)), cell(101, 102));
        assert_eq!(shallow.target(), cell(102, 105));
        assert_eq!(shallow.from(), cell(100, 100));
    }
    #[test]
    fn every_native_range_pair_arrives_bounded_and_heading_is_literal() {
        for dy in -10i32..=10 {
            for dx in -10i32..=10 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let target = cell(32768 + dx, 32768 + dy);
                let path = LightArrowPath::new(cell(32768, 32768), target).unwrap();
                assert_eq!(path.position(path.impact_at()), target);
                assert!(path.steps.len() <= 20);
                assert!(path.impact_at() <= Duration::from_millis(1140));
                assert!(path.heading() <= 7);
            }
        }
        for (heading, (dx, dy)) in [
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                LightArrowPath::new(cell(100, 100), cell(100 + dx, 100 + dy))
                    .unwrap()
                    .heading(),
                heading as u8
            );
        }
        let moved = LightArrowPath::new(cell(100, 100), cell(356, 100)).unwrap();
        assert_eq!(moved.impact_at(), Duration::from_millis(15_300));
        assert_eq!(moved.position(Duration::MAX), cell(356, 100));
        for (from, target) in [
            (cell(0, 0), cell(257, 0)),
            (cell(-1, 0), cell(0, 0)),
            (cell(65535, 65535), cell(65536, 65535)),
            (cell(i32::MIN, i32::MIN), cell(i32::MAX, i32::MAX)),
        ] {
            assert!(LightArrowPath::new(from, target).is_err());
        }
    }
    #[test]
    fn launch_sound_once_flight_holds_zero_then_one_impact_and_retirement() {
        let path = LightArrowPath::new(cell(100, 100), cell(110, 100)).unwrap();
        let mut playback = timeline(&path);
        let first = playback.advance(Duration::ZERO).unwrap();
        assert_eq!(first.source_frame, 1);
        assert_eq!(first.crossed_cues.len(), 1);
        assert_eq!(first.crossed_cues[0].event, FrameEvent::Sound(97));
        assert!(playback
            .advance(Duration::ZERO)
            .unwrap()
            .crossed_cues
            .is_empty());
        let holding = playback.advance(Duration::from_millis(400)).unwrap();
        assert_eq!(holding.source_frame, 0);
        assert!(!holding.finished);
        let hit = playback.advance(path.impact_at()).unwrap();
        assert_eq!(hit.source_frame, 1);
        assert!(!hit.finished);
        assert!(hit.crossed_cues.is_empty());
        let end = path.impact_at() + Duration::from_nanos(541_666_667);
        assert!(
            !playback
                .advance(end - Duration::from_nanos(1))
                .unwrap()
                .finished
        );
        assert!(playback.advance(end).unwrap().finished);
        assert!(playback
            .advance(Duration::MAX)
            .unwrap()
            .crossed_cues
            .is_empty());
        assert!(playback.advance(end).is_err());
    }
    #[test]
    fn adjacent_native_arrival_still_crosses_the_once_only_launch_audio() {
        let path = LightArrowPath::new(cell(100, 100), cell(100, 101)).unwrap();
        assert_eq!(path.impact_at(), Duration::ZERO);
        let mut playback = timeline(&path);
        let sample = playback.advance(Duration::ZERO).unwrap();
        assert_eq!(sample.source_frame, 1);
        assert!(!sample.finished);
        assert_eq!(sample.crossed_cues[0].event, FrameEvent::Sound(97));
        assert!(playback.advance(Duration::MAX).unwrap().finished);
        assert!(playback
            .advance(Duration::MAX)
            .unwrap()
            .crossed_cues
            .is_empty());
        let same = LightArrowPath::new(cell(100, 100), cell(100, 100)).unwrap();
        assert_eq!(same.impact_at(), Duration::ZERO);
        let mut playback = timeline(&same);
        assert_eq!(
            playback.advance(Duration::ZERO).unwrap().crossed_cues[0].event,
            FrameEvent::Sound(97)
        );
        assert!(playback
            .advance(Duration::ZERO)
            .unwrap()
            .crossed_cues
            .is_empty());
    }
    #[test]
    fn caster18_zero_tick_launch_is_preserved_and_unknowns_do_not_qualify() {
        let good = source();
        let catalog = parse(&good);
        let bound = qualify_caster(&catalog, BaseAppearance { class: 0, sex: 0 }).unwrap();
        let mut playback = playback(&bound.sequence);
        let start = playback.advance(Duration::ZERO).unwrap();
        assert_eq!(start.source_frame, 2);
        assert_eq!(start.crossed_cues.len(), 1);
        assert_eq!(start.crossed_cues[0].tick, 0);
        assert_eq!(start.crossed_cues[0].event, FrameEvent::Hit);
        assert!(playback
            .advance(Duration::ZERO)
            .unwrap()
            .crossed_cues
            .is_empty());
        for bad in [
            good.replace("32.1:2!", "32.1:2"),
            good.replace("32.2:3)", "32.2:3!)"),
            good.replace("32.2:3)", "32.2:3[97)"),
            good.replace("18.cast", "19.cast"),
        ] {
            assert!(qualify_caster(&parse(&bad), BaseAppearance { class: 0, sex: 0 }).is_err());
        }
    }
    #[test]
    fn finite_clock_allocation_projection_and_cancel_bounds() {
        let catalog = parse(&source());
        let mut animation = catalog
            .resolve_action(167, ActionKey::new(0))
            .unwrap()
            .animation
            .clone();
        animation.frames = vec![animation.frames[1].clone(); MAX_FRAMES + 1];
        assert!(Sequence::new(&animation).is_err());
        animation.frames.truncate(2);
        animation.frames[1].duration_ticks = 0;
        assert!(Sequence::new(&animation).is_err());
        assert!(matches!(
            LightArrowAssets::load(Path::new("absent"), &[], &AtomicBool::new(true)),
            Err(LightArrowError::Cancelled)
        ));
        assert!(matches!(
            LightArrowAssets::load(Path::new("absent"), &[], &AtomicBool::new(false)),
            Err(LightArrowError::InvalidInput(_))
        ));
        assert!(matches!(
            LightArrowAssets::load(
                Path::new("absent"),
                &[BaseAppearance { class: 7, sex: 0 }],
                &AtomicBool::new(false)
            ),
            Err(LightArrowError::InvalidInput(_))
        ));
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
        let mut canvas = Canvas::new(2, 2).unwrap();
        draw_frame(&mut canvas, &frame, PixelPoint { x: 0, y: 0 }).unwrap();
        assert_eq!(&canvas.rgba()[..4], &[255, 0, 0, 255]);
        assert!(draw_frame(&mut canvas, &frame, PixelPoint { x: i32::MIN, y: 0 }).is_err());
    }
    #[test]
    fn screen_effect_helper_does_not_reclassify_generic_frames_or_accept_partial_coverage() {
        let mut frame = SpriteFrame {
            fragments: None,
            x_offset: 0,
            y_offset: 0,
            width: 1,
            height: 1,
            frame_type: 49,
            unknown1: 0,
            unknown2: 0,
            rgba: vec![0, 0, 0, 255],
        };
        let mut canvas = Canvas::new(1, 1).unwrap();
        canvas.clear([82, 99, 123]);
        draw_screen_frame(&mut canvas, &frame, PixelPoint { x: 0, y: 0 }).unwrap();
        assert_eq!(canvas.rgba(), &[82, 99, 123, 255]);
        draw_frame(&mut canvas, &frame, PixelPoint { x: 0, y: 0 }).unwrap();
        assert_eq!(canvas.rgba(), &[0, 0, 0, 255]);
        let before = canvas.rgba().to_vec();
        frame.rgba[3] = 128;
        assert!(draw_screen_frame(&mut canvas, &frame, PixelPoint { x: i32::MAX, y: 0 }).is_err());
        frame.x_offset = 1;
        assert!(draw_screen_frame(&mut canvas, &frame, PixelPoint { x: i32::MAX, y: 0 }).is_err());
        assert_eq!(canvas.rgba(), before);
    }
    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; all14 caster18 directions +167/219/97, memory only"]
    fn actual_shared_light_arrow_full_chain_all_casters_and_rgba_match_csharp() {
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
        let appearances: Vec<_> = (0..7)
            .flat_map(|class| (0..2).map(move |sex| BaseAppearance { class, sex }))
            .collect();
        let assets =
            LightArrowAssets::load(Path::new(&root), &appearances, &AtomicBool::new(false))
                .unwrap();
        assert_eq!(
            assets.catalog_sha256(),
            hex("523c61ad5cc362a56a36f58b262c84efa8b4435f017e7074d35b5834e8b9f127")
        );
        assert_eq!(assets.sprite_sources().len(), 229);
        assert_eq!(assets.encoded_bytes(), 2_634_226);
        let flattened: usize = assets
            .sprites
            .values()
            .flat_map(|s| &s.decoded.frames)
            .map(|f| f.rgba.len())
            .sum();
        assert_eq!(flattened, 27_340_416); // Independent C# flat pixels unchanged.
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
        // Independent memory-only PakViewer SprReader.LoadRaw over every SPR,
        // ASCII filename ordering, NUL, then each frame x/y/w/h little-endian
        // i32 followed by full RGBA. No runtime renderer-derived expectations.
        let mut sprites: Vec<_> = assets.sprites.values().collect();
        sprites.sort_by(|a, b| a.provenance.name.cmp(&b.provenance.name));
        let mut digest = Sha256::new();
        for sprite in sprites {
            digest.update(sprite.provenance.name.as_bytes());
            digest.update([0]);
            for frame in &sprite.decoded.frames {
                for value in [
                    frame.x_offset,
                    frame.y_offset,
                    frame.width as i32,
                    frame.height as i32,
                ] {
                    digest.update(value.to_le_bytes());
                }
                digest.update(&frame.rgba);
            }
        }
        assert_eq!(
            <[u8; 32]>::from(digest.finalize()),
            hex("e2b44fd8464417e57c2ab580b5dd3a8e4b761955c840bdf131c6dcfa9e960238")
        );
        let hashes = [
            (
                167,
                0,
                6066,
                "272a8ef95db5932a5b07fc89960bd2e9d4aa806aa8264732d66335aff8c64263",
            ),
            (
                167,
                1,
                4646,
                "b348eb3ef8f2e53f56d8d17116d6e66413c37526d354853e8f0bda159552a089",
            ),
            (
                167,
                2,
                6195,
                "22f87556717dda1773a4a5e18697ff9933281177c25b890c4a3c88d21f897d95",
            ),
            (
                167,
                3,
                7694,
                "d99e9f6a217f3cf4d249bef4c4c4901c2945d70964745b220d2d23385be93891",
            ),
            (
                219,
                0,
                32022,
                "af95c6f53dc105296c6880ac325799452b12df858ba9588d834802bb4b4c6c32",
            ),
        ];
        for (gfx, image, size, hash) in hashes {
            let provenance = &assets.sprites[&SpriteKey { gfx, image }].provenance;
            assert_eq!(provenance.archive, SharedArchiveKind::Sprite);
            assert_eq!(provenance.bytes, size);
            assert_eq!(provenance.sha256, hex(hash));
        }
        let bank = assets.sound_bank();
        let sound = bank.get(97).unwrap();
        assert_eq!(
            sound.sha256(),
            hex("2e8661cc26238e422b3233509ba24ae11b3440d8c1d74666cb5c842e7de6f1a7")
        );
        assert_eq!(bank.file_bytes(), 47_408);
        assert_eq!(
            (
                sound.format().channels,
                sound.format().sample_rate,
                sound.format().bits_per_sample
            ),
            (1, 22_050, 16)
        );
        assert_eq!(sound.pcm_bytes().len(), 47_290);
        for appearance in appearances {
            let launch = assets.cast_launch_delay(appearance).unwrap();
            let duration = assets.cast_duration(appearance).unwrap();
            assert!(launch < duration);
            for heading in 0..8 {
                for elapsed in [Duration::ZERO, launch, duration] {
                    let visual = assets.caster_frame(appearance, heading, elapsed).unwrap();
                    assert!(!visual.frame.rgba.is_empty());
                    assert_eq!(visual.finished, elapsed == duration);
                }
            }
        }
        let path = LightArrowPath::new(cell(32768, 32768), cell(32778, 32768)).unwrap();
        let mut canvas = Canvas::new(800, 600).unwrap();
        let projection = ViewProjection {
            anchor_cell: path.from(),
            anchor_pixel: PixelPoint { x: 400, y: 300 },
        };
        for elapsed in [Duration::ZERO, Duration::from_millis(240), path.impact_at()] {
            assert!(assets
                .draw_effect(&mut canvas, &path, projection, elapsed)
                .unwrap());
        }
        assert!(!assets
            .draw_effect(
                &mut canvas,
                &path,
                projection,
                assets.effect_duration(&path)
            )
            .unwrap());
        assert!(assets
            .caster_frame(BaseAppearance { class: 0, sex: 0 }, 8, Duration::ZERO)
            .is_err());
        let mut changed = [(false, false); 2];
        for elapsed in [
            Duration::ZERO,
            Duration::from_millis(240),
            path.impact_at(),
            assets.effect_duration(&path) - Duration::from_nanos(1),
        ] {
            let sample = assets.effect_frame(&path, elapsed).unwrap();
            let phase = usize::from(elapsed >= path.impact_at());
            let anchor = projection.cell_to_pixel(path.position(elapsed)).unwrap();
            let result = super::super::teleport_effect::screen_test_support::compare(
                sample.frame,
                anchor,
                |canvas| {
                    assets
                        .draw_effect(canvas, &path, projection, elapsed)
                        .unwrap()
                },
            );
            changed[phase].0 |= result.0;
            changed[phase].1 |= result.1;
        }
        assert_eq!(
            changed,
            [(true, true); 2],
            "both167 flight and219 impact use screen, not caster/body drawing"
        );
        super::super::teleport_effect::screen_test_support::retired(|canvas| {
            assets
                .draw_effect(canvas, &path, projection, assets.effect_duration(&path))
                .unwrap()
        });
    }
    fn hex(text: &str) -> [u8; 32] {
        assert_eq!(text.len(), 64);
        std::array::from_fn(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).unwrap())
    }
    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; exact four profiles and scoped reader lifecycle, memory only"]
    fn actual_shared_four_profiles_preserve_all_assets_and_discard_failed_reader() {
        let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
        let cancel=AtomicBool::new(false);let context=SharedActionCatalog::load(Path::new(&root),&cancel).unwrap();let mut slot=None;
        for gfx in [167,1797,1799,1583] {
            let original=LightArrowAssets::load_selected_profile(&context,BTreeSet::new(),gfx,&cancel).unwrap();
            let reused=LightArrowAssets::load_effect_profile_with_reader(&context,gfx,&cancel,&mut slot).unwrap();
            assert_eq!(original.catalog_sha256,reused.catalog_sha256);assert_eq!(original.encoded,reused.encoded);assert_eq!(original.rgba,reused.rgba);
            assert_eq!(original.effect.gfx,reused.effect.gfx);assert_eq!(original.impact.gfx,reused.impact.gfx);
            for (a,b) in [(&original.effect,&reused.effect),(&original.impact,&reused.impact)] {
                assert_eq!(a.sequence.animation,b.sequence.animation);assert_eq!(a.sequence.starts,b.sequence.starts);assert_eq!(a.sequence.ends,b.sequence.ends);assert_eq!(a.sequence.total,b.sequence.total);
            }
            assert!(original.casters.is_empty()&&reused.casters.is_empty());assert_eq!(original.sprites.len(),reused.sprites.len());
            for (key,a) in &original.sprites {let b=&reused.sprites[key];assert_eq!(a.decoded,b.decoded);assert_eq!(format!("{:?}",a.provenance),format!("{:?}",b.provenance));}
            assert_eq!(format!("{:?}",original.group),format!("{:?}",reused.group));
            assert_eq!(original.sound.ids().collect::<Vec<_>>(),reused.sound.ids().collect::<Vec<_>>());
            for id in original.sound.ids() {let a=original.sound.get(id).unwrap();let b=reused.sound.get(id).unwrap();assert_eq!(a.sha256(),b.sha256());assert_eq!(a.pcm_bytes(),b.pcm_bytes());assert_eq!(a.file_bytes(),b.file_bytes());assert_eq!(a.format(),b.format());}
        }
        assert!(slot.is_some());assert!(matches!(LightArrowAssets::load_effect_profile_with_reader(&context,u32::MAX,&cancel,&mut slot),Err(LightArrowError::Unsupported(_))));assert!(slot.is_none());
        LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot).unwrap();cancel.store(true,Ordering::Release);
        assert!(matches!(LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot),Err(LightArrowError::Cancelled)));assert!(slot.is_none());cancel.store(false,Ordering::Release);
        LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot).unwrap();slot.as_mut().unwrap().root=PathBuf::from("deliberately-wrong-scope");
        assert!(matches!(LightArrowAssets::load_effect_profile_with_reader(&context,167,&cancel,&mut slot),Err(LightArrowError::Resource(ResourceError::UnsafePath(_)))));assert!(slot.is_none());
    }

}
